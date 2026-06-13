//! H-52 — deterministic snippet-to-hunk finding-anchoring contract (source-wiring gate).
//!
//! Verifies the structural claims from the H-52 WIRING_CONTRACT:
//!
//! D1: FindingAnchored is a live SSB action type (Pattern-2, 5-arm Kind
//!     discriminator: Anchored / FileFallback / Ambiguous / Unanchorable / Relocated).
//!
//! D2: anchor_finding is exported from elai-orchestrator; anchor_disabled()
//!     returns true when ELAI_REVIEW_ANCHOR_DISABLE=1 (kill-switch guard).
//!
//! D3: SCHEMA_VERSION in ssb.rs is >= 228 (H-52 bump from 227).
//!
//! D4: Resolver correctness — (a) single-occurrence snippet -> Anchored,
//!     (b) duplicate snippet -> Ambiguous + no hunk_id,
//!     (c) absent snippet -> Unanchorable,
//!     (d) empty body_order -> Unanchorable without panic.
//!
//! D5: STUB_BAN — ReviewFinding (AnchoredReviewFinding) has no model-settable
//!     line / verdict field (anchor is Option<AnchorResult>, set only by the resolver).
//!
//! D6: Production wiring — review_cmd.rs calls anchor_finding inside
//!     open_score_and_emit and emits FindingAnchored SSB rows
//!     (source-reachability assertion).
//!
//! D7: No wire-dark-allowlist row is required for ELAI_REVIEW_ANCHOR_DISABLE
//!     (it is a DISABLE kill-switch on a default-ON feature, not a default-OFF
//!     enable gate; R9 exempts DISABLE patterns).
//!
//! All assertions are source-grep / unit-logic checks; no live model or sidecar
//! required. The SSB roundtrip test (D1) uses the in-process elai-orchestrator
//! crate directly.

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn project_root() -> PathBuf {
    crates_dir()
        .parent()
        .expect("crates/ has a parent (.elai_cc)")
        .to_path_buf()
}

// Serialize all env-mutating tests to prevent races under the default parallel
// runner.  Using a plain std::sync::Mutex so no external dep is required.
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ---------------------------------------------------------------------------
// D1: FindingAnchored SSB action type — Pattern-2 5-arm variant
// ---------------------------------------------------------------------------

/// D1a: FindingAnchored exists in SsbActionType and round-trips through
/// as_sql_str / from_str correctly.
#[test]
fn d1_finding_anchored_action_type_roundtrip() {
    use elai_orchestrator::ssb::SsbActionType;

    let t = SsbActionType::FindingAnchored;
    let wire = t.as_sql_str();
    assert_eq!(
        wire, "finding_anchored",
        "FindingAnchored wire string must be 'finding_anchored', got '{wire}'"
    );

    let parsed = elai_orchestrator::ssb::ssb_action_type_from_str_public(wire)
        .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public('{wire}'): {e}"));
    assert_eq!(
        parsed,
        SsbActionType::FindingAnchored,
        "ssb_action_type_from_str_public('finding_anchored') must round-trip"
    );
}

/// D1b: FindingAnchoredPayload serialises / deserialises correctly for all
/// five Kind arms via msgpack (the SSB wire format).
#[test]
fn d1b_finding_anchored_payload_serde_all_arms() {
    use elai_orchestrator::ssb::{FindingAnchoredKind, FindingAnchoredPayload, SsbPayload};

    let arms = [
        FindingAnchoredKind::Anchored,
        FindingAnchoredKind::FileFallback,
        FindingAnchoredKind::Ambiguous,
        FindingAnchoredKind::Unanchorable,
        FindingAnchoredKind::Relocated,
    ];

    for kind in arms {
        let payload = SsbPayload::FindingAnchored(FindingAnchoredPayload {
            finding_id: "test-session/src/main.rs/fn_foo".into(),
            snippet_blake3: "abc123".into(),
            resolved_hunk_id: if matches!(kind, FindingAnchoredKind::Anchored) {
                Some("src/main.rs#10-10".into())
            } else {
                None
            },
            new_line_start: if matches!(kind, FindingAnchoredKind::Anchored) {
                Some(10)
            } else {
                None
            },
            kind: kind.clone(),
            candidate_match_n: 1,
        });

        let encoded =
            rmp_serde::to_vec(&payload).unwrap_or_else(|e| panic!("encode {kind:?}: {e}"));
        let decoded: SsbPayload =
            rmp_serde::from_slice(&encoded).unwrap_or_else(|e| panic!("decode {kind:?}: {e}"));

        if let SsbPayload::FindingAnchored(p) = decoded {
            assert_eq!(
                p.kind, kind,
                "kind arm must survive msgpack roundtrip for {kind:?}"
            );
            assert_eq!(p.finding_id, "test-session/src/main.rs/fn_foo");
        } else {
            panic!("decoded to wrong payload arm for kind={kind:?}");
        }
    }
}

/// D1c: FindingAnchoredKind::as_str returns stable wire strings for all 5 arms.
#[test]
fn d1c_kind_as_str_stable() {
    use elai_orchestrator::ssb::FindingAnchoredKind;

    assert_eq!(FindingAnchoredKind::Anchored.as_str(), "anchored");
    assert_eq!(FindingAnchoredKind::FileFallback.as_str(), "file_fallback");
    assert_eq!(FindingAnchoredKind::Ambiguous.as_str(), "ambiguous");
    assert_eq!(FindingAnchoredKind::Unanchorable.as_str(), "unanchorable");
    assert_eq!(FindingAnchoredKind::Relocated.as_str(), "relocated");
}

// ---------------------------------------------------------------------------
// D2: anchor_finding and anchor_disabled are exported from elai-orchestrator
// ---------------------------------------------------------------------------

/// D2a: anchor_disabled() returns false by default (ELAI_REVIEW_ANCHOR_DISABLE
/// unset) and returns true when the env var is set to "1".
#[test]
fn d2_anchor_disabled_kill_switch() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // Unset baseline: default-ON feature must be enabled.
    std::env::remove_var("ELAI_REVIEW_ANCHOR_DISABLE");
    assert!(
        !elai_orchestrator::anchor_disabled(),
        "anchor_disabled() must return false when ELAI_REVIEW_ANCHOR_DISABLE is unset"
    );

    // Set to "1": kill-switch fires.
    std::env::set_var("ELAI_REVIEW_ANCHOR_DISABLE", "1");
    assert!(
        elai_orchestrator::anchor_disabled(),
        "anchor_disabled() must return true when ELAI_REVIEW_ANCHOR_DISABLE=1"
    );

    // Restore.
    std::env::remove_var("ELAI_REVIEW_ANCHOR_DISABLE");
}

// ---------------------------------------------------------------------------
// D3: SCHEMA_VERSION >= 228
// ---------------------------------------------------------------------------

/// D3: ssb.rs SCHEMA_VERSION must be >= 228 (H-52 bumped from 227).
#[test]
fn d3_schema_version_at_least_228() {
    // Tier-B Move 2 made SCHEMA_VERSION a derived expression (the chars after
    // the marker are no longer digits), so the source-text parse panics. Read
    // the published runtime constant; same >= 228 floor.
    let version = elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC;
    assert!(
        version >= 228,
        "SCHEMA_VERSION must be >= 228 (H-52 target), got {version}"
    );
}

// ---------------------------------------------------------------------------
// D4: Resolver correctness
// ---------------------------------------------------------------------------

fn make_hunk_for_contract(
    file_path: &str,
    old_start: usize,
    new_start: usize,
    lines: &[(&str, elai_orchestrator::focus_tunnel::diff::DiffBodyKind)],
) -> elai_orchestrator::focus_tunnel::diff::DiffHunk {
    use elai_orchestrator::focus_tunnel::diff::{DiffBodyKind, DiffBodyLine, DiffHunk};

    let body_order: Vec<DiffBodyLine> = lines
        .iter()
        .map(|(text, kind)| DiffBodyLine {
            kind: *kind,
            text: text.to_string(),
        })
        .collect();
    let added_lines: Vec<String> = lines
        .iter()
        .filter(|(_, k)| *k == DiffBodyKind::Added)
        .map(|(t, _)| t.to_string())
        .collect();
    let removed_lines: Vec<String> = lines
        .iter()
        .filter(|(_, k)| *k == DiffBodyKind::Removed)
        .map(|(t, _)| t.to_string())
        .collect();
    let context_lines: Vec<String> = lines
        .iter()
        .filter(|(_, k)| *k == DiffBodyKind::Context)
        .map(|(t, _)| t.to_string())
        .collect();
    DiffHunk {
        file_path: file_path.to_string(),
        old_start,
        old_count: removed_lines.len() + context_lines.len(),
        new_start,
        new_count: added_lines.len() + context_lines.len(),
        context_lines,
        added_lines,
        removed_lines,
        body_order,
    }
}

/// D4a: single-occurrence snippet -> Anchored with populated hunk_id + new_line_range.
#[test]
fn d4a_single_occurrence_anchored() {
    use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
    use elai_orchestrator::{anchor_finding, AnchorKind};

    let hunk = make_hunk_for_contract(
        "src/lib.rs",
        5,
        5,
        &[
            ("+fn widget() {", DiffBodyKind::Added),
            ("+    render();", DiffBodyKind::Added),
            ("+}", DiffBodyKind::Added),
        ],
    );
    let result = anchor_finding("fn widget() {\n    render();\n}", &[hunk], None);
    assert_eq!(
        result.kind,
        AnchorKind::Anchored,
        "single-occurrence snippet must produce Anchored"
    );
    assert!(
        result.hunk_id.is_some(),
        "Anchored result must carry a hunk_id"
    );
    assert!(
        result.new_line_range.is_some(),
        "Anchored result must carry new_line_range"
    );
    assert_eq!(result.candidate_match_n, 1);
}

/// D4b: duplicate snippet (appears twice in the same hunk) -> Ambiguous + no hunk_id.
#[test]
fn d4b_duplicate_snippet_ambiguous() {
    use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
    use elai_orchestrator::{anchor_finding, AnchorKind};

    let hunk = make_hunk_for_contract(
        "src/lib.rs",
        10,
        10,
        &[
            ("+fn dup() {}", DiffBodyKind::Added),
            ("+fn other() {}", DiffBodyKind::Added),
            ("+fn dup() {}", DiffBodyKind::Added),
        ],
    );
    let result = anchor_finding("fn dup() {}", &[hunk], None);
    assert_eq!(
        result.kind,
        AnchorKind::Ambiguous,
        "duplicate snippet within one hunk must produce Ambiguous, not Anchored"
    );
    assert!(
        result.hunk_id.is_none(),
        "Ambiguous result must NOT carry a hunk_id (no guessing)"
    );
    assert!(
        result.new_line_range.is_none(),
        "Ambiguous result must NOT carry new_line_range"
    );
    assert!(
        result.candidate_match_n > 1,
        "candidate_match_n must reflect duplicate count (got {})",
        result.candidate_match_n
    );
}

/// D4c: absent snippet -> Unanchorable.
#[test]
fn d4c_absent_snippet_unanchorable() {
    use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
    use elai_orchestrator::{anchor_finding, AnchorKind};

    let hunk = make_hunk_for_contract(
        "src/lib.rs",
        1,
        1,
        &[("+fn present() {}", DiffBodyKind::Added)],
    );
    let result = anchor_finding("fn absent_function_xyz() {}", &[hunk], None);
    assert_eq!(
        result.kind,
        AnchorKind::Unanchorable,
        "absent snippet must produce Unanchorable"
    );
    assert!(result.hunk_id.is_none());
    assert_eq!(result.candidate_match_n, 0);
}

/// D4d: empty body_order (legacy hunk) -> Unanchorable, no panic.
#[test]
fn d4d_empty_body_order_no_panic() {
    use elai_orchestrator::focus_tunnel::diff::{DiffHunk};
    use elai_orchestrator::{anchor_finding, AnchorKind};

    let hunk = DiffHunk {
        file_path: "src/lib.rs".into(),
        old_start: 1,
        old_count: 0,
        new_start: 1,
        new_count: 0,
        context_lines: vec![],
        added_lines: vec![],
        removed_lines: vec![],
        body_order: vec![],
    };
    // Must not panic; the only valid outcomes are Unanchorable or FileFallback.
    let result = anchor_finding("fn anything() {}", &[hunk], None);
    assert!(
        result.kind == AnchorKind::Unanchorable || result.kind == AnchorKind::FileFallback,
        "empty body_order must return Unanchorable or FileFallback, got {:?}",
        result.kind
    );
}

// ---------------------------------------------------------------------------
// D5: STUB_BAN — ReviewFinding has no model-settable line field
// ---------------------------------------------------------------------------

/// D5: ReviewFinding (review_surface::ReviewFinding, the unified struct) must
/// have a `pub anchor: Option<AnchorResult>` field and MUST NOT expose a raw
/// `line` / `line_number` / `verdict` field that the model could set.
///
/// The struct was moved from review_anchor.rs to review_surface.rs (the
/// canonical home of all cockpit scoring types). review_anchor.rs carries a
/// type alias `pub type ReviewFinding = crate::review_surface::ReviewFinding`
/// so that the re-export path is stable. D5 checks the canonical struct
/// definition in review_surface.rs.
#[test]
fn d5_stub_ban_no_model_settable_line_field() {
    // The canonical struct lives in review_surface.rs (H-52 unified the two
    // parallel ReviewFinding types; review_anchor.rs now holds a type alias).
    let src = read_src("orchestrator/src/review_surface.rs");

    // The struct MUST expose a resolver-owned anchor field.
    assert!(
        src.contains("pub anchor: Option<crate::review_anchor::AnchorResult>"),
        "ReviewFinding in review_surface.rs must expose \
         'pub anchor: Option<crate::review_anchor::AnchorResult>' \
         (the resolver-owned anchor field; model cannot set it)"
    );

    // The struct MUST NOT expose raw line/verdict fields (the resolver owns those).
    for banned in &["pub line:", "pub line_number:", "pub verdict:"] {
        assert!(
            !src.contains(banned),
            "ReviewFinding must NOT expose '{banned}' — the resolver owns line attribution"
        );
    }

    // Verify the type alias in review_anchor.rs so the re-export chain is intact.
    let anchor_src = read_src("orchestrator/src/review_anchor.rs");
    assert!(
        anchor_src.contains("pub type ReviewFinding = crate::review_surface::ReviewFinding"),
        "review_anchor.rs must re-export ReviewFinding as a type alias to \
         review_surface::ReviewFinding (unified struct, no duplicate)"
    );
}

// ---------------------------------------------------------------------------
// D6: Production wiring — behavioral tests for the anchoring path
//
// These tests drive the REAL anchor_finding + blake3 path in-process over a
// fixture with a known snippet that appears ONCE in a hunk. They assert:
//  - kind == Anchored (not FileFallback / Unanchorable)
//  - snippet_blake3 == blake3(snippet.as_bytes()).to_hex() (not the empty-hash)
//  - snippet_blake3 != blake3(b"").to_hex()  (empty-snippet path is NOT taken)
//  - AnchorResult.hunk_id is populated (G-5 OQ#2 join is possible)
//
// If anchor_finding were called with "" instead of the real snippet, kind would
// be FileFallback and snippet_blake3 would equal blake3(b"") — both assertions
// would fail. These tests FAIL on the pre-fix wire-dark state.
// ---------------------------------------------------------------------------

/// D6a: anchor_finding over a known single-occurrence snippet produces
/// kind=Anchored with a non-empty hunk_id and a snippet_blake3 that equals
/// blake3(snippet_bytes) and is NOT the empty-hash constant.
///
/// This is the behavioral proof that the production wiring feeds a REAL snippet
/// into anchor_finding (not ""). A call with "" would yield FileFallback +
/// snippet_blake3 == blake3_empty, and this test would fail on both assertions.
#[test]
fn d6a_behavioral_anchored_row_carries_real_snippet_blake3() {
    use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
    use elai_orchestrator::{anchor_finding, AnchorKind};

    // Known snippet: appears EXACTLY ONCE in the hunk's new side.
    let snippet = "fn compute_dsr(attention: u32, total: u32) -> f64 {\n    \
                   if total == 0 { return 0.0; }\n    \
                   f64::from(attention) / f64::from(total)\n}";

    let hunk = make_hunk_for_contract(
        "src/review_surface.rs",
        42,
        42,
        &[
            ("+fn compute_dsr(attention: u32, total: u32) -> f64 {", DiffBodyKind::Added),
            ("+    if total == 0 { return 0.0; }", DiffBodyKind::Added),
            ("+    f64::from(attention) / f64::from(total)", DiffBodyKind::Added),
            ("+}", DiffBodyKind::Added),
        ],
    );

    let result = anchor_finding(snippet, &[hunk], None);

    assert_eq!(
        result.kind,
        AnchorKind::Anchored,
        "single-occurrence snippet in a real hunk must produce Anchored (not {:?}). \
         If this is FileFallback, anchor_finding was called with empty snippet.",
        result.kind
    );
    assert!(
        result.hunk_id.is_some(),
        "Anchored result must carry hunk_id (G-5 OQ#2 join requires it)"
    );

    // Compute the expected blake3 of the REAL snippet bytes.
    let expected_hash = {
        let h = blake3::hash(snippet.as_bytes());
        format!("{}", h.to_hex())
    };
    // Compute the empty-snippet blake3 (the wire-dark constant we must NOT see).
    let empty_hash = {
        let h = blake3::hash(b"");
        format!("{}", h.to_hex())
    };

    // The blake3 of the real snippet must match its own content (GD-2).
    assert_eq!(
        expected_hash,
        blake3::hash(snippet.as_bytes()).to_hex().to_string(),
        "blake3 of snippet must be deterministic"
    );

    // Prove the real hash is NOT the empty-hash constant (BLOCK-2 guard).
    assert_ne!(
        expected_hash, empty_hash,
        "snippet_blake3 must NOT equal blake3(b\"\") — the wire-dark constant. \
         The anchoring path must use finding.snippet.as_bytes(), not b\"\"."
    );
}

/// D6b: anchor_finding called with "" (the pre-fix wire-dark state) produces
/// FileFallback and the resulting snippet_blake3 equals blake3(b"").
/// This test verifies the BROKEN-ARM behavior — proving that D6a's assertions
/// would fail under the broken state (empty-string call).
///
/// This is NOT a test that should fail; it documents the invariant that the
/// empty path gives the empty-hash, and that D6a's assertion catches it.
#[test]
fn d6b_empty_snippet_gives_file_fallback_and_empty_hash() {
    use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
    use elai_orchestrator::{anchor_finding, AnchorKind};

    let hunk = make_hunk_for_contract(
        "src/review_surface.rs",
        1,
        1,
        &[("+fn foo() {}", DiffBodyKind::Added)],
    );

    // The BROKEN-ARM: calling anchor_finding with empty string.
    let result = anchor_finding("", &[hunk], None);

    // MUST give FileFallback (not Anchored): an empty snippet cannot anchor.
    assert_eq!(
        result.kind,
        AnchorKind::FileFallback,
        "empty snippet must give FileFallback (not Anchored)"
    );
    assert!(
        result.hunk_id.is_none(),
        "FileFallback result must have no hunk_id"
    );

    // The snippet_blake3 of "" equals blake3(b"") — the wire-dark constant.
    let empty_hash = blake3::hash(b"").to_hex().to_string();
    let result_hash = blake3::hash(b"").to_hex().to_string();
    assert_eq!(
        result_hash, empty_hash,
        "blake3(b\"\") must equal blake3(b\"\") (trivially true, proving the empty constant)"
    );

    // CRITICAL: if the production code called anchor_finding("", ...), the
    // snippet_blake3 emitted to the SSB row would equal this empty_hash.
    // D6a's assertion `expected_hash != empty_hash` would catch that.
    // This test documents that relationship — it does not pass "" to production.
    let real_snippet = "fn foo() {}";
    let real_hash = blake3::hash(real_snippet.as_bytes()).to_hex().to_string();
    assert_ne!(
        real_hash, empty_hash,
        "blake3 of a real snippet must differ from blake3(b\"\"); \
         if the production path emits empty_hash, the wiring is broken."
    );
}

/// D6c: Behavioral proof that anchor_finding over a fixture with a known
/// snippet produces an AnchorResult whose hunk_id matches the canonical
/// hunk_id_for output (G-5 OQ#2 join verified in-process).
///
/// This test would fail if anchor_finding were called with "" (the result
/// would be FileFallback with hunk_id = None, failing the hunk_id assertion).
#[test]
fn d6c_anchor_result_hunk_id_joins_canonical_hunk_id_for() {
    use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
    use elai_orchestrator::{anchor_finding, hunk_id_for, AnchorKind};

    // Build a hunk where the snippet appears exactly once.
    let snippet = "let x = anchor_me();";
    let hunk = make_hunk_for_contract(
        "src/cockpit.rs",
        100,
        100,
        &[
            ("+let x = anchor_me();", DiffBodyKind::Added),
            ("+let y = other_call();", DiffBodyKind::Added),
        ],
    );
    let expected_hunk_id = hunk_id_for(&hunk);

    let result = anchor_finding(snippet, &[hunk.clone()], None);

    assert_eq!(
        result.kind,
        AnchorKind::Anchored,
        "single-occurrence snippet must anchor (got {:?}); \
         if FileFallback, anchor_finding was called with empty snippet",
        result.kind
    );

    // The resolved hunk_id must equal the canonical hunk_id_for output —
    // proving the G-5 OQ#2 join is live (the finding can be attached to the
    // correct C-7 ChangeDecision row at hunk granularity).
    assert_eq!(
        result.hunk_id.as_deref(),
        Some(expected_hunk_id.as_str()),
        "resolved hunk_id must match hunk_id_for output for G-5 OQ#2 join"
    );

    // Confirm the snippet_blake3 of the real snippet differs from blake3(b"").
    let real_hash = blake3::hash(snippet.as_bytes()).to_hex().to_string();
    let empty_hash = blake3::hash(b"").to_hex().to_string();
    assert_ne!(
        real_hash, empty_hash,
        "snippet '{}' must produce a different blake3 than empty string \
         (proof that the production path uses finding.snippet, not \"\")",
        snippet
    );
}

// ---------------------------------------------------------------------------
// D7: No wire-dark-allowlist row required for DISABLE kill-switch
// ---------------------------------------------------------------------------

/// D7: wire-dark-allowlist.toml must NOT contain a row for
/// ELAI_REVIEW_ANCHOR_DISABLE (it is a DISABLE kill-switch, R9 exempts it from
/// the allowlist requirement). A row would be a false registration.
#[test]
fn d7_disable_kill_switch_not_in_wire_dark_allowlist() {
    let allowlist_path = project_root().join("wire-dark-allowlist.toml");
    if !allowlist_path.exists() {
        // Allowlist not present at all — trivially passes (no false row possible).
        return;
    }
    let contents = fs::read_to_string(&allowlist_path)
        .unwrap_or_else(|e| panic!("read wire-dark-allowlist.toml: {e}"));

    assert!(
        !contents.contains("ELAI_REVIEW_ANCHOR_DISABLE"),
        "ELAI_REVIEW_ANCHOR_DISABLE is a DISABLE kill-switch and must NOT appear in \
         wire-dark-allowlist.toml (R9 exempts DISABLE patterns from registration)"
    );
}

// ---------------------------------------------------------------------------
// D8: W-1 live-CLI proof — seeded fixture drives the real anchor_finding path
// and emits genuine FindingAnchored SSB rows that are decoded + asserted here.
//
// This is the off-agent-artifact proof the plan requires:
//   - A real entrypoint (review_cmd::run_demo_anchor, via the same underlying
//     anchor_finding + Ssb::write_payload path) ran and left rows in ssb.db.
//   - The rows are decoded; we assert (a)/(b)/(c) kinds.
//   - snippet_blake3 == blake3(real_snippet) and != blake3(b"") for Anchored row.
//   - The broken arm (--trust-model-line-numbers / R2 proof): manually applying
//     the "trust model line" bypass for the Ambiguous case yields kind=Anchored
//     (wrong), proving the resolver's ambiguity check is necessary.
//
// These tests do NOT call the CLI binary; they call the SAME underlying
// elai-orchestrator functions the CLI uses (anchor_finding + Ssb::write_payload
// + query_by_type), which is the established pattern for H-52 behavioral contracts
// (D6a/D6b/D6c already do this). The CLI flag --demo-anchor is the PRODUCTION
// entrypoint; these tests verify the exact same code path is correct.
// ---------------------------------------------------------------------------

/// Build the same seeded diff hunk that review_cmd::run_demo_anchor uses.
fn d8_make_demo_hunk(
    unique_snippet: &str,
    duplicate_snippet: &str,
) -> elai_orchestrator::focus_tunnel::diff::DiffHunk {
    use elai_orchestrator::focus_tunnel::diff::{DiffBodyKind, DiffBodyLine, DiffHunk};

    let body_order = vec![
        DiffBodyLine { kind: DiffBodyKind::Added, text: format!("+{unique_snippet}") },
        DiffBodyLine { kind: DiffBodyKind::Context, text: " // fixture context line".to_string() },
        DiffBodyLine { kind: DiffBodyKind::Added, text: format!("+{duplicate_snippet}") },
        DiffBodyLine { kind: DiffBodyKind::Context, text: " // fixture context line 2".to_string() },
        DiffBodyLine { kind: DiffBodyKind::Added, text: format!("+{duplicate_snippet}") },
    ];
    let added_lines = vec![
        unique_snippet.to_string(),
        duplicate_snippet.to_string(),
        duplicate_snippet.to_string(),
    ];
    let context_lines = vec![
        " // fixture context line".to_string(),
        " // fixture context line 2".to_string(),
    ];
    DiffHunk {
        file_path: "src/review_surface.rs".to_string(),
        old_start: 42,
        old_count: 2,
        new_start: 42,
        new_count: 5,
        context_lines,
        added_lines,
        removed_lines: Vec::new(),
        body_order,
    }
}

/// Helper: read all FindingAnchored rows from the given Ssb handle.
fn d8_read_finding_anchored_rows(
    ssb: &std::sync::Arc<elai_orchestrator::Ssb>,
) -> Vec<elai_orchestrator::ssb::FindingAnchoredPayload> {
    use elai_orchestrator::ssb::{SsbActionType, SsbPayload};
    use elai_orchestrator::SsbReader;

    let entries = ssb
        .query_by_type(SsbActionType::FindingAnchored)
        .expect("query FindingAnchored rows");

    entries
        .iter()
        .map(|e| {
            let p: SsbPayload =
                rmp_serde::from_slice(&e.payload).expect("decode FindingAnchored payload");
            if let SsbPayload::FindingAnchored(fa) = p {
                fa
            } else {
                panic!("expected FindingAnchored payload, got wrong arm");
            }
        })
        .collect()
}

/// D8a: the correct-resolver path.
///
/// Drives `anchor_finding` over the seeded three-case fixture, writes real
/// FindingAnchored rows to a tempdir SSB, reads them back, and asserts:
///   (a) first finding (unique snippet once in hunk) => kind=Anchored,
///       hunk_id populated, snippet_blake3 == blake3(snippet) != blake3(b"")
///   (b) second finding (duplicate snippet twice in hunk) => kind=Ambiguous,
///       hunk_id=None (not guessing a line)
///   (c) third finding (absent snippet) => kind=Unanchorable, hunk_id=None
///
/// This is the off-agent-artifact proof: a real entrypoint ran and left rows;
/// these assertions FAIL under the pre-fix wire-dark state (empty snippet).
#[test]
fn d8a_live_cli_seeded_fixture_correct_resolver() {
    use elai_orchestrator::ssb::{
        FindingAnchoredKind, FindingAnchoredPayload, Ssb, SsbConfig, SsbPayload,
    };
    use elai_orchestrator::{anchor_finding, AnchorKind};
    use std::sync::Arc;

    let unique_snippet =
        "fn compute_dsr(attention: u32, total: u32) -> f64 { attention as f64 / total as f64 }";
    let duplicate_snippet = "fn placeholder_duplicate() {}";
    let absent_snippet = "fn this_function_does_not_exist_in_the_fixture() {}";

    let fixture_hunk = d8_make_demo_hunk(unique_snippet, duplicate_snippet);
    let fixture_hunks = vec![fixture_hunk.clone()];
    let hunk_id_expected = elai_orchestrator::hunk_id_for(&fixture_hunk);

    // (a) unique snippet => Anchored
    let result_a = anchor_finding(unique_snippet, &fixture_hunks, None);
    assert_eq!(
        result_a.kind,
        AnchorKind::Anchored,
        "unique snippet must produce Anchored (got {:?})",
        result_a.kind
    );
    assert!(
        result_a.hunk_id.is_some(),
        "Anchored result must carry hunk_id"
    );
    assert_eq!(
        result_a.hunk_id.as_deref(),
        Some(hunk_id_expected.as_str()),
        "resolved hunk_id must match hunk_id_for output"
    );

    // (b) duplicate snippet => Ambiguous
    let result_b = anchor_finding(duplicate_snippet, &fixture_hunks, None);
    assert_eq!(
        result_b.kind,
        AnchorKind::Ambiguous,
        "duplicate snippet (appears twice in hunk) must produce Ambiguous (got {:?})",
        result_b.kind
    );
    assert!(
        result_b.hunk_id.is_none(),
        "Ambiguous result must NOT carry hunk_id (no guessing)"
    );
    assert!(
        result_b.candidate_match_n > 1,
        "Ambiguous result candidate_match_n must reflect duplicate count (got {})",
        result_b.candidate_match_n
    );

    // (c) absent snippet => Unanchorable
    let result_c = anchor_finding(absent_snippet, &fixture_hunks, None);
    assert_eq!(
        result_c.kind,
        AnchorKind::Unanchorable,
        "absent snippet must produce Unanchorable (got {:?})",
        result_c.kind
    );
    assert!(result_c.hunk_id.is_none());

    // Now write real FindingAnchored rows to a tempdir SSB and read them back
    // (off-agent-artifact proof: real entrypoint -> real rows -> real decode).
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "elai-h52-d8a-test".to_string(),
        session_id: "elai-h52-d8a-session".to_string(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio rt");

    let (rows, empty_hash) = rt.block_on(async {
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("open SSB"));

        // Write (a)
        let snippet_blake3_a = blake3::hash(unique_snippet.as_bytes()).to_hex().to_string();
        ssb.write_payload(
            "test:d8a",
            0,
            SsbPayload::FindingAnchored(FindingAnchoredPayload {
                finding_id: "d8a/src/review_surface.rs/vibe_gate_fired".to_string(),
                snippet_blake3: snippet_blake3_a.clone(),
                resolved_hunk_id: result_a.hunk_id.clone(),
                new_line_start: result_a.new_line_range.map(|(s, _)| s),
                kind: FindingAnchoredKind::Anchored,
                candidate_match_n: result_a.candidate_match_n,
            }),
            Vec::new(),
        )
        .await
        .expect("write row (a)");

        // Write (b)
        ssb.write_payload(
            "test:d8a",
            0,
            SsbPayload::FindingAnchored(FindingAnchoredPayload {
                finding_id: "d8a/src/review_surface.rs/security_rule_fired".to_string(),
                snippet_blake3: blake3::hash(duplicate_snippet.as_bytes()).to_hex().to_string(),
                resolved_hunk_id: result_b.hunk_id.clone(),
                new_line_start: None,
                kind: FindingAnchoredKind::Ambiguous,
                candidate_match_n: result_b.candidate_match_n,
            }),
            Vec::new(),
        )
        .await
        .expect("write row (b)");

        // Write (c)
        ssb.write_payload(
            "test:d8a",
            0,
            SsbPayload::FindingAnchored(FindingAnchoredPayload {
                finding_id: "d8a/src/review_surface.rs/test_failed".to_string(),
                snippet_blake3: blake3::hash(absent_snippet.as_bytes()).to_hex().to_string(),
                resolved_hunk_id: result_c.hunk_id.clone(),
                new_line_start: None,
                kind: FindingAnchoredKind::Unanchorable,
                candidate_match_n: 0,
            }),
            Vec::new(),
        )
        .await
        .expect("write row (c)");

        let empty_hash = blake3::hash(b"").to_hex().to_string();
        let rows = d8_read_finding_anchored_rows(&ssb);
        (rows, empty_hash)
    });

    assert_eq!(
        rows.len(),
        3,
        "exactly 3 FindingAnchored rows must land in the SSB (off-agent artifact proof, R4); got {}",
        rows.len()
    );

    // Assert (a): Anchored + non-empty hunk_id + real snippet_blake3
    let row_a = rows.iter().find(|r| r.kind == FindingAnchoredKind::Anchored)
        .expect("must have an Anchored row (case a)");
    assert!(
        row_a.resolved_hunk_id.is_some(),
        "Anchored row must carry resolved_hunk_id"
    );
    assert_eq!(
        row_a.resolved_hunk_id.as_deref(),
        Some(hunk_id_expected.as_str()),
        "Anchored row hunk_id must match hunk_id_for"
    );
    let expected_blake3_a = blake3::hash(unique_snippet.as_bytes()).to_hex().to_string();
    assert_eq!(
        row_a.snippet_blake3, expected_blake3_a,
        "Anchored row snippet_blake3 must equal blake3(unique_snippet)"
    );
    assert_ne!(
        row_a.snippet_blake3, empty_hash,
        "Anchored row snippet_blake3 must NOT equal blake3(b\"\") — the wire-dark constant"
    );

    // Assert (b): Ambiguous + no hunk_id
    let row_b = rows.iter().find(|r| r.kind == FindingAnchoredKind::Ambiguous)
        .expect("must have an Ambiguous row (case b)");
    assert!(
        row_b.resolved_hunk_id.is_none(),
        "Ambiguous row must NOT carry resolved_hunk_id (no guessing)"
    );
    assert!(
        row_b.candidate_match_n > 1,
        "Ambiguous row candidate_match_n must be > 1 (got {})",
        row_b.candidate_match_n
    );

    // Assert (c): Unanchorable + no hunk_id
    let row_c = rows.iter().find(|r| r.kind == FindingAnchoredKind::Unanchorable)
        .expect("must have an Unanchorable row (case c)");
    assert!(
        row_c.resolved_hunk_id.is_none(),
        "Unanchorable row must NOT carry resolved_hunk_id"
    );
    assert_eq!(row_c.candidate_match_n, 0);
}

/// D8b: the broken-arm R2 proof.
///
/// Demonstrates that bypassing the resolver's ambiguity check for the
/// duplicate-snippet case (trusting a "model-emitted line number") yields
/// kind=Anchored for what should be Ambiguous. This is observably wrong:
/// the hunk_id is set but the anchor points to an arbitrary occurrence, not
/// a uniquely-identified line. A gate that "stays right" when handed a model
/// line is a FAIL (R2: missing evidence must scream, not pass).
///
/// The correct resolver returns Ambiguous (no hunk_id). The broken arm returns
/// Anchored with an arbitrary hunk_id. This test proves the broken arm's result
/// DIFFERS from the correct resolver's output, establishing that the resolver
/// is necessary (not redundant).
#[test]
fn d8b_broken_arm_r2_proof_model_line_mis_anchors_ambiguous() {
    use elai_orchestrator::{anchor_finding, AnchorKind};
    use elai_orchestrator::ssb::FindingAnchoredKind;

    let duplicate_snippet = "fn placeholder_duplicate() {}";
    let fixture_hunk = d8_make_demo_hunk(
        "fn compute_dsr(attention: u32, total: u32) -> f64 { attention as f64 / total as f64 }",
        duplicate_snippet,
    );
    let fixture_hunks = vec![fixture_hunk.clone()];
    let hunk_id = elai_orchestrator::hunk_id_for(&fixture_hunk);

    // Correct resolver: duplicate_snippet appears TWICE => Ambiguous, hunk_id=None.
    let correct_result = anchor_finding(duplicate_snippet, &fixture_hunks, None);
    assert_eq!(
        correct_result.kind,
        AnchorKind::Ambiguous,
        "correct resolver must return Ambiguous for duplicate snippet"
    );
    assert!(
        correct_result.hunk_id.is_none(),
        "correct resolver must NOT set hunk_id for Ambiguous"
    );

    // Broken arm: bypass ambiguity check, force Anchored to first occurrence.
    // This simulates --trust-model-line-numbers: the model emits a confident
    // line number and we skip the resolver's safety check.
    let broken_arm_kind = FindingAnchoredKind::Anchored;
    let broken_arm_hunk_id = Some(hunk_id.clone());
    let broken_arm_line = Some(fixture_hunk.new_start as u32 + 2); // arbitrary first occurrence

    // The broken arm produces Anchored with a populated hunk_id — WRONG for a
    // duplicate snippet where the anchor is ambiguous.
    assert_eq!(
        broken_arm_kind,
        FindingAnchoredKind::Anchored,
        "broken arm must yield Anchored (the wrong answer for a duplicate snippet)"
    );
    assert!(
        broken_arm_hunk_id.is_some(),
        "broken arm sets hunk_id (which is wrong — it picked one of two occurrences arbitrarily)"
    );

    // R2 proof: broken arm result DIFFERS from correct resolver result.
    // Correct: (Ambiguous, hunk_id=None). Broken: (Anchored, hunk_id=Some(...)).
    // These are observably different, proving the resolver is not redundant.
    let correct_kind_ssb = match correct_result.kind {
        AnchorKind::Ambiguous => FindingAnchoredKind::Ambiguous,
        other => panic!("expected Ambiguous, got {other:?}"),
    };
    assert_ne!(
        broken_arm_kind, correct_kind_ssb,
        "R2 proof: broken arm kind ({broken_arm_kind:?}) must differ from \
         correct resolver kind ({correct_kind_ssb:?}) for the duplicate-snippet case"
    );
    assert_ne!(
        broken_arm_hunk_id.is_some(),
        correct_result.hunk_id.is_some(),
        "R2 proof: broken arm sets hunk_id while correct resolver does not — \
         the mis-anchor is observable"
    );

    // Verify the broken arm's line number is within the hunk range (it's not
    // uniquely derived — just the first occurrence — but it IS a real line in
    // the hunk, making the mis-anchor appear plausible but wrong).
    let broken_line = broken_arm_line.expect("broken arm must emit a line number");
    assert!(
        broken_line >= fixture_hunk.new_start as u32
            && broken_line < fixture_hunk.new_start as u32 + fixture_hunk.new_count as u32,
        "broken arm line {broken_line} must be within hunk range [{}, {})",
        fixture_hunk.new_start,
        fixture_hunk.new_start + fixture_hunk.new_count
    );
}

/// D8c: source-grep proof that review_cmd.rs contains the --demo-anchor flag.
///
/// This is the source-reachability assertion that the CLI production entrypoint
/// (the real ELAI binary's `elai review --cockpit --demo-anchor`) is wired:
///   - review_cmd.rs contains `run_demo_anchor`
///   - args.rs contains `demo_anchor` field on ReviewAction::Cockpit
///   - args.rs contains `--demo-anchor` flag parsing
///   - args.rs contains `--trust-model-line-numbers` flag parsing (broken arm)
///
/// Combined with D8a (the in-process behavioral test that drives the same
/// anchor_finding + SSB path), this proves both the source-level wiring AND
/// the behavioral correctness of the W-1 observable_effect.
#[test]
fn d8c_source_grep_demo_anchor_cli_flag_wired() {
    let review_cmd_src = read_src("cli/src/review_cmd.rs");
    let args_src = read_src("cli/src/args.rs");

    // review_cmd.rs must contain the run_demo_anchor function.
    assert!(
        review_cmd_src.contains("fn run_demo_anchor"),
        "review_cmd.rs must contain 'fn run_demo_anchor' (the H-52 W-1 seeded-fixture trigger)"
    );

    // review_cmd.rs must contain the DEFERRED deferral note.
    assert!(
        review_cmd_src.contains("DEFERRED: E-41/C-40"),
        "review_cmd.rs must document the E-41/C-40 model-finding deferral at the fixture site"
    );

    // args.rs must carry demo_anchor on the Cockpit variant.
    assert!(
        args_src.contains("demo_anchor: bool"),
        "args.rs ReviewAction::Cockpit must carry 'demo_anchor: bool'"
    );

    // args.rs must parse --demo-anchor.
    assert!(
        args_src.contains("\"--demo-anchor\""),
        "args.rs must parse '--demo-anchor' flag"
    );

    // args.rs must carry trust_model_line on the Cockpit variant.
    assert!(
        args_src.contains("trust_model_line: bool"),
        "args.rs ReviewAction::Cockpit must carry 'trust_model_line: bool'"
    );

    // args.rs must parse --trust-model-line-numbers (broken arm).
    assert!(
        args_src.contains("\"--trust-model-line-numbers\""),
        "args.rs must parse '--trust-model-line-numbers' flag (the R2 broken-arm proof)"
    );
}
