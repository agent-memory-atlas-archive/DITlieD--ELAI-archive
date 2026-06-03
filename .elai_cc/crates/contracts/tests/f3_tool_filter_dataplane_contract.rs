//! F-3 Slice C4 -- tool-filter data-plane contract tests.
//!
//! Covers:
//!   1. commit_minimal_set freezes the set: the blake3 hash is stable and
//!      64 hex chars, and allowed_categories is sorted + deduplicated.
//!   2. check_against_commitment DENIES an out-of-set tool
//!      (CategoryNotCommitted) + ALLOWS an in-set tool (Ok).
//!   3. Fixture replay via the embedded AgentDojo-style fixture:
//!      asr_after < asr_before (i.e. every out-of-set injection is blocked).
//!   4. ToolFilterEvent{Committed} + {UntrustedQuarantined} SSB round-trip
//!      through msgpack + SQL CHECK constraint + bijection.
//!   5. The string "capability_flow_blocked" is absent everywhere in the
//!      workspace (guarding against a renamed / stale wire label).
//!
//! No #[ignore] / stubs / placeholder assertions.
//!
//! Paper grounding: arXiv:2406.13352 (AgentDojo), OWASP LLM Top 10 LLM01,
//! arXiv:2503.18813 (CaMeL).

use std::sync::Arc;

use blake3;
use elai_core::clock::OsClock;
use elai_orchestrator::{
    check_against_commitment, commit_minimal_set, taint_from_verdict, taint_text,
    CapabilityFlowDenial, CommittedToolSet, ToolFilterEventKind, ToolFilterPayload,
    UntrustedTaint,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::tool_filter::ToolCategory;
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// 1. commit_minimal_set stability + dedup + sort
// ---------------------------------------------------------------------------

#[test]
fn commit_minimal_set_hash_is_64_hex() {
    let committed = commit_minimal_set(
        "task-c4-test",
        &[ToolCategory::Read, ToolCategory::Web],
        1_748_000_000_000,
    );
    assert_eq!(
        committed.commit_hash.len(),
        64,
        "blake3 hex must be 64 chars; got {}",
        committed.commit_hash.len()
    );
    assert!(
        committed.commit_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "hash must be hex; got {}",
        committed.commit_hash
    );
}

#[test]
fn commit_minimal_set_is_sorted_and_deduped() {
    // Supply in reverse order + duplicate.
    let committed = commit_minimal_set(
        "task-c4-sort",
        &[
            ToolCategory::Web,
            ToolCategory::Read,
            ToolCategory::Read, // duplicate
        ],
        0,
    );
    // Sorted lexicographically by snake_case label: "read" < "web".
    assert_eq!(
        committed.allowed_categories,
        vec![ToolCategory::Read, ToolCategory::Web],
        "categories must be sorted lexicographically and deduplicated"
    );
}

#[test]
fn commit_minimal_set_is_deterministic() {
    let a = commit_minimal_set(
        "task-c4-determinism",
        &[ToolCategory::Web, ToolCategory::Read],
        0,
    );
    let b = commit_minimal_set(
        "task-c4-determinism",
        &[ToolCategory::Read, ToolCategory::Web],
        0,
    );
    assert_eq!(a.commit_hash, b.commit_hash, "same inputs must produce same hash regardless of input order");
}

#[test]
fn commit_minimal_set_hash_changes_with_task_id() {
    let a = commit_minimal_set("task-A", &[ToolCategory::Read], 0);
    let b = commit_minimal_set("task-B", &[ToolCategory::Read], 0);
    assert_ne!(a.commit_hash, b.commit_hash, "different task_ids must produce different hashes");
}

// ---------------------------------------------------------------------------
// 2. check_against_commitment: deny out-of-set + allow in-set
// ---------------------------------------------------------------------------

#[test]
fn check_against_commitment_denies_out_of_set_tool() {
    let committed = commit_minimal_set(
        "task-c4-deny",
        &[ToolCategory::Read, ToolCategory::Web],
        0,
    );
    let result = check_against_commitment(&committed, "bash");
    match result {
        Err(CapabilityFlowDenial::CategoryNotCommitted {
            tool_name,
            category,
            commit_hash,
        }) => {
            assert_eq!(tool_name, "bash");
            assert_eq!(category, ToolCategory::Bash);
            assert_eq!(commit_hash, committed.commit_hash, "commit_hash in denial must match committed set");
        }
        Ok(()) => panic!("expected denial for out-of-set tool 'bash'"),
    }
}

#[test]
fn check_against_commitment_denies_powershell_as_bash_category() {
    let committed = commit_minimal_set(
        "task-c4-deny-ps",
        &[ToolCategory::Read, ToolCategory::Web],
        0,
    );
    // PowerShell maps to ToolCategory::Bash -- must also be denied.
    let result = check_against_commitment(&committed, "PowerShell");
    assert!(
        result.is_err(),
        "PowerShell (Bash category) must be denied when Bash is not in the committed set"
    );
    match result.unwrap_err() {
        CapabilityFlowDenial::CategoryNotCommitted { category, .. } => {
            assert_eq!(category, ToolCategory::Bash);
        }
    }
}

#[test]
fn check_against_commitment_denies_write_file_as_edit_category() {
    let committed = commit_minimal_set(
        "task-c4-deny-edit",
        &[ToolCategory::Read, ToolCategory::Web],
        0,
    );
    let result = check_against_commitment(&committed, "write_file");
    assert!(result.is_err(), "write_file (Edit category) must be denied");
    match result.unwrap_err() {
        CapabilityFlowDenial::CategoryNotCommitted { category, .. } => {
            assert_eq!(category, ToolCategory::Edit);
        }
    }
}

#[test]
fn check_against_commitment_allows_in_set_tool() {
    let committed = commit_minimal_set(
        "task-c4-allow",
        &[ToolCategory::Read, ToolCategory::Web],
        0,
    );
    check_against_commitment(&committed, "read")
        .expect("read is in-set and must be allowed");
    check_against_commitment(&committed, "web_search")
        .expect("web_search (Web category) is in-set and must be allowed");
}

// ---------------------------------------------------------------------------
// 3. Fixture replay: asr_after < asr_before
// ---------------------------------------------------------------------------

/// Inline struct mirroring the fixture JSON shape so we don't need serde in
/// the contract (contracts crate already has serde in dev-deps).
#[derive(serde::Deserialize)]
struct FixtureInjection {
    expected_denied_tool: Option<String>,
}

#[derive(serde::Deserialize)]
struct ToolFilterFixture {
    committed_categories: Vec<String>,
    injections: Vec<FixtureInjection>,
}

/// Resolve the `fixtures/tool_filter/` directory relative to this crate's
/// manifest. `CARGO_MANIFEST_DIR` is a compile-time constant pointing at the
/// contracts crate source tree, so this resolves deterministically regardless
/// of the process cwd (mirrors the workspace-root walk in
/// [`capability_flow_blocked_absent_from_workspace`]).
fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("tool_filter")
}

/// Enumerate every `*.json` fixture under `fixtures/tool_filter/`, sorted by
/// file name for deterministic ordering. FAIL (panic) if the directory is
/// missing or empty — a dead evidence path must scream, not shrug (R2 /
/// failOnEmptyShould). Returns `(file_name, raw_json)` pairs.
fn enumerate_fixtures() -> Vec<(String, String)> {
    let dir = fixtures_dir();
    let rd = std::fs::read_dir(&dir).unwrap_or_else(|e| {
        panic!(
            "tool_filter fixtures dir not found at {}: {e} (R2: missing evidence path is a hard FAIL)",
            dir.display()
        )
    });

    let mut fixtures: Vec<(String, String)> = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()));
        fixtures.push((name, raw));
    }
    fixtures.sort_by(|a, b| a.0.cmp(&b.0));

    assert!(
        !fixtures.is_empty(),
        "no *.json fixtures found under {} (R1: a gate that finds ZERO contracts FAILS)",
        dir.display()
    );
    fixtures
}

/// Map the string label back to ToolCategory (mirrors safety_cmd.rs).
fn label_to_category(s: &str) -> Option<ToolCategory> {
    match s {
        "read" => Some(ToolCategory::Read),
        "web" => Some(ToolCategory::Web),
        "bash" => Some(ToolCategory::Bash),
        "edit" => Some(ToolCategory::Edit),
        "planning" => Some(ToolCategory::Planning),
        "testing" => Some(ToolCategory::Testing),
        "validation" => Some(ToolCategory::Validation),
        "memory" => Some(ToolCategory::Memory),
        "config_only" => Some(ToolCategory::ConfigOnly),
        "meta" => Some(ToolCategory::Meta),
        "task_management" => Some(ToolCategory::TaskManagement),
        "mcp" => Some(ToolCategory::Mcp),
        "fsm_internal" => Some(ToolCategory::FsmInternal),
        "git" => Some(ToolCategory::Git),
        "knowledge" => Some(ToolCategory::Knowledge),
        "stub_write" => Some(ToolCategory::StubWrite),
        _ => None,
    }
}

/// Replay one fixture's injections through the REAL
/// [`check_against_commitment`] production predicate.
///
/// Returns `(asr_before, asr_after, deny_injections, in_set_negatives,
/// over_blocked)`:
///   * `asr_before` / `asr_after` — attack-success rate over the
///     `expected_denied_tool`-carrying injections (1.0 before any filtering;
///     0.0 once every out-of-set tool is denied).
///   * `deny_injections` — count of injections carrying an
///     `expected_denied_tool` (the ASR denominator).
///   * `in_set_negatives` — count of injections whose `expected_denied_tool`
///     is `None` (the "must not over-block" negatives).
///   * `over_blocked` — count of in-set negatives whose canonical in-set tool
///     `"read"` was wrongly DENIED (must be 0).
fn replay_fixture(raw: &str) -> (f64, f64, usize, usize, usize) {
    let fixture: ToolFilterFixture =
        serde_json::from_str(raw).expect("fixture parse");

    let allowed: Vec<ToolCategory> = fixture
        .committed_categories
        .iter()
        .filter_map(|s| label_to_category(s))
        .collect();
    let committed = commit_minimal_set("contract-test-replay", &allowed, 0);

    let deny_injections = fixture
        .injections
        .iter()
        .filter(|i| i.expected_denied_tool.is_some())
        .count();

    let mut before = 0usize;
    let mut after = 0usize;
    let mut in_set_negatives = 0usize;
    let mut over_blocked = 0usize;

    for inj in &fixture.injections {
        match &inj.expected_denied_tool {
            Some(tool) => {
                before += 1;
                if check_against_commitment(&committed, tool).is_ok() {
                    after += 1; // filter bypass (must NOT happen for these fixtures)
                }
            }
            None => {
                // In-set negative: the canonical in-set tool "read" must be
                // ALLOWED (Ok). A denial here is over-blocking.
                in_set_negatives += 1;
                if check_against_commitment(&committed, "read").is_err() {
                    over_blocked += 1;
                }
            }
        }
    }

    let asr_before = if deny_injections == 0 { 0.0 } else { before as f64 / deny_injections as f64 };
    let asr_after  = if deny_injections == 0 { 0.0 } else { after  as f64 / deny_injections as f64 };
    (asr_before, asr_after, deny_injections, in_set_negatives, over_blocked)
}

/// F-3 acceptance: enumerate EVERY fixture under `fixtures/tool_filter/`,
/// replay each through the real committed-set predicate, and assert:
///   * >= 20 fixture files exist (the acceptance floor),
///   * >= 20 deny-injections total are denied (20-of-20-denied), with the
///     aggregate post-filter ASR == 0.0,
///   * zero over-blocking on the in-set negatives (>= 1 negative present).
///
/// This replaces the prior 3 hand-bound `include_str!` consts: the harness is
/// now blind to no fixture — dropping a new `*.json` into the dir is replayed
/// automatically, and the floor is enforced by enumeration, not a const list.
#[test]
fn all_tool_filter_fixtures_deny_every_out_of_set_injection() {
    let fixtures = enumerate_fixtures();

    assert!(
        fixtures.len() >= 20,
        "F-3 acceptance requires >= 20 AgentDojo-style fixtures; found {}",
        fixtures.len()
    );

    let mut total_deny_injections = 0usize;
    let mut total_bypassed = 0usize; // deny-injections that slipped through
    let mut total_in_set_negatives = 0usize;
    let mut total_over_blocked = 0usize;

    for (name, raw) in &fixtures {
        let (asr_before, asr_after, deny_n, neg_n, over_blocked) = replay_fixture(raw);

        if deny_n > 0 {
            // Every deny-fixture must demonstrate a strict reduction to 0.0.
            assert_eq!(
                asr_before, 1.0,
                "fixture '{name}': asr_before must be 1.0 (every injection initially counts as a hit); got {asr_before}"
            );
            assert_eq!(
                asr_after, 0.0,
                "fixture '{name}': every out-of-set injection must be denied; asr_after={asr_after}"
            );
        }

        // bypassed = deny_n * asr_after (asr_after is 0.0 on success).
        total_bypassed += (asr_after * deny_n as f64).round() as usize;
        total_deny_injections += deny_n;
        total_in_set_negatives += neg_n;
        total_over_blocked += over_blocked;
    }

    assert!(
        total_deny_injections >= 20,
        "F-3 acceptance requires >= 20 deny-injections (20-of-20 denied); found {total_deny_injections}"
    );
    assert_eq!(
        total_bypassed, 0,
        "20-of-20-denied: zero out-of-set injections may bypass the committed-set filter; {total_bypassed} bypassed"
    );
    assert!(
        total_in_set_negatives >= 1,
        "at least one in-set negative fixture must exist to prove the filter does not over-block"
    );
    assert_eq!(
        total_over_blocked, 0,
        "the filter must NOT over-block in-set tools; {total_over_blocked} in-set negatives were wrongly denied"
    );
}

// ---------------------------------------------------------------------------
// 4. SSB round-trip: ToolFilterEvent{Committed} + {UntrustedQuarantined}
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tool_filter_committed_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb_f3.db"),
        pool_size: Some(2),
        project_id: "f3-c4-committed".into(),
        session_id: "f3-c4-session-1".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let committed = commit_minimal_set("task-ssb-roundtrip", &[ToolCategory::Read, ToolCategory::Web], 0);
    let payload = SsbPayload::ToolFilterEvent(ToolFilterPayload {
        commit_hash: committed.commit_hash.clone(),
        allowed_categories: vec!["read".into(), "web".into()],
        denied_tool: None,
        taint_source: None,
        task_id: "task-ssb-roundtrip".into(),
        kind: ToolFilterEventKind::Committed,
    });

    assert_eq!(
        payload.action_type(),
        SsbActionType::ToolFilterEvent(ToolFilterEventKind::Committed)
    );

    ssb.write_payload("executing", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::ToolFilterEvent(ToolFilterEventKind::Committed))
        .expect("query");
    assert_eq!(rows.len(), 1, "must read back exactly 1 Committed row");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::ToolFilterEvent(p) => {
            assert_eq!(p.kind, ToolFilterEventKind::Committed);
            assert_eq!(p.commit_hash, committed.commit_hash);
            assert_eq!(p.task_id, "task-ssb-roundtrip");
            assert!(p.denied_tool.is_none());
        }
        other => panic!("expected ToolFilterEvent(Committed), got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tool_filter_untrusted_quarantined_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb_f3_q.db"),
        pool_size: Some(2),
        project_id: "f3-c4-quarantine".into(),
        session_id: "f3-c4-session-2".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::ToolFilterEvent(ToolFilterPayload {
        commit_hash: blake3::hash(b"f3-c4-quarantine-test").to_hex().to_string(),
        allowed_categories: vec!["read".into(), "web".into()],
        denied_tool: Some("bash".into()),
        taint_source: Some("malicious".into()),
        task_id: "task-ssb-quarantine".into(),
        kind: ToolFilterEventKind::UntrustedQuarantined,
    });

    assert_eq!(
        payload.action_type(),
        SsbActionType::ToolFilterEvent(ToolFilterEventKind::UntrustedQuarantined)
    );

    ssb.write_payload("executing", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::ToolFilterEvent(
            ToolFilterEventKind::UntrustedQuarantined,
        ))
        .expect("query");
    assert_eq!(rows.len(), 1, "must read back exactly 1 UntrustedQuarantined row");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::ToolFilterEvent(p) => {
            assert_eq!(p.kind, ToolFilterEventKind::UntrustedQuarantined);
            assert_eq!(p.denied_tool.as_deref(), Some("bash"));
            assert_eq!(p.taint_source.as_deref(), Some("malicious"));
        }
        other => panic!("expected ToolFilterEvent(UntrustedQuarantined), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Bijection: as_sql_str round-trips through ssb_action_type_from_str_public
// ---------------------------------------------------------------------------

#[test]
fn tool_filter_committed_sql_str_bijection() {
    let variant = SsbActionType::ToolFilterEvent(ToolFilterEventKind::Committed);
    let s = variant.as_sql_str();
    assert_eq!(s, "tool_filter_committed");
    let round = ssb_action_type_from_str_public(s).expect("parse");
    assert_eq!(round, variant);
}

#[test]
fn tool_filter_untrusted_sql_str_bijection() {
    let variant = SsbActionType::ToolFilterEvent(ToolFilterEventKind::UntrustedQuarantined);
    let s = variant.as_sql_str();
    assert_eq!(s, "untrusted_var_quarantined");
    let round = ssb_action_type_from_str_public(s).expect("parse");
    assert_eq!(round, variant);
}

// ---------------------------------------------------------------------------
// 5. Negative-space: "capability_flow_blocked" must be absent everywhere
// ---------------------------------------------------------------------------

#[test]
fn capability_flow_blocked_absent_from_workspace() {
    // Guarding against a stale / renamed wire PRODUCER label sneaking in.
    // We walk Rust source files under the workspace root looking for the bare
    // snake_case literal in a *producer* context — a string literal or an
    // identifier — NOT inside a `//` line comment. The reserved variant may be
    // DOCUMENTED in doc-comments (`CapabilityFlowBlocked` CamelCase, or the
    // snake_case form inside prose) without being a live producer; what must
    // stay absent is an actual wire label / enum arm that emits the string.
    //
    // We also exclude this contract file itself: it necessarily mentions the
    // literal in its own assertions + the `target` constant below, which is not
    // a producer. Without this exclusion the test would flag itself (it did,
    // before this robustness fix — the prior version walked its own source).
    let workspace_root = {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| manifest.to_path_buf())
    };

    let target = "capability_flow_blocked";
    let self_path = std::path::Path::new(file!())
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("f3_tool_filter_dataplane_contract.rs")
        .to_owned();
    let mut violations: Vec<String> = Vec::new();

    /// Strip a trailing `//`-style line comment from a source line. Returns the
    /// code portion only, so a `target` occurrence inside doc/line-comment prose
    /// (e.g. the reserved-variant documentation) is not treated as a producer.
    /// This is a deliberately conservative line-level approximation: it does not
    /// model block comments or `//` inside string literals, which do not occur
    /// for this label in the workspace (verified at authoring time).
    fn code_portion(line: &str) -> &str {
        match line.find("//") {
            Some(idx) => &line[..idx],
            None => line,
        }
    }

    fn walk(
        dir: &std::path::Path,
        target: &str,
        self_path: &str,
        violations: &mut Vec<String>,
    ) {
        let rd = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target/ and .git/ to keep the scan fast.
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "target" || name == ".git" {
                    continue;
                }
                walk(&path, target, self_path, violations);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                // Exclude this contract file itself (its assertions + `target`
                // const reference the literal but are not a producer).
                if path.file_name().and_then(|n| n.to_str()) == Some(self_path) {
                    continue;
                }
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    // Only flag occurrences in the CODE portion of a line (a
                    // real producer), never in `//` comment prose.
                    if contents
                        .lines()
                        .any(|line| code_portion(line).contains(target))
                    {
                        violations.push(path.display().to_string());
                    }
                }
            }
        }
    }

    walk(&workspace_root, target, &self_path, &mut violations);

    assert!(
        violations.is_empty(),
        "Found producer-context '{}' in these files (must be absent — use CategoryNotCommitted / the CapabilityFlowBlocked reserved variant instead): {:?}",
        target,
        violations
    );
}

// ---------------------------------------------------------------------------
// Emission is unconditional default-ON (W-2)
// ---------------------------------------------------------------------------
// The former `elai_tool_filter_event_enable_env_const_value` test was removed
// with the dead `ELAI_TOOL_FILTER_EVENT_ENABLE_ENV` const (F-Unit-C WARN-2):
// ToolFilterEvent emission is unconditional whenever an SSB writer is present,
// with no enable-env gate, so there is no const to assert against.
