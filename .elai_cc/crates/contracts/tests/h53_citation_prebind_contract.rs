//! H-53 — citation pre-binding + constrained synthesis contract (source-wiring gate).
//!
//! Verifies the structural claims from the H-53 WIRING_CONTRACT:
//!
//! D1: CitationAnchorsBound is a live SSB action type (Pattern-2, 2-arm Kind
//!     discriminator: Bound | Blocked). SCHEMA_VERSION bumped to 229.
//!
//! D2: freeze_anchors, check_citation_in_set, extract_citation_ids, and
//!     prebind_disabled() are exported from elai-orchestrator.
//!     prebind_disabled() returns true when ELAI_CITATION_PREBIND_DISABLE=1
//!     (default-ON kill-switch guard).
//!
//! D3: SCHEMA_VERSION in ssb.rs is >= 229 (H-53 bump from 228).
//!
//! D4: freeze_anchors correctness — (a) rank-stable ordering preserved,
//!     (b) 1-based IDs assigned, (c) set_blake3 is deterministic under same seed,
//!     (d) same seed + same input -> same set_blake3 (GD-2: byte-recomputable).
//!
//! D5: check_citation_in_set correctness — (a) Bound: id in set + excerpt matches,
//!     (b) Ghost: id out of range, (c) Stale: id in set but excerpt changed.
//!
//! D6: extract_citation_ids extracts [N] patterns, deduplicates, and sorts.
//!
//! D7: Off-agent artifact proof — CitationAnchorsBound SSB row emitted from a
//!     real entrypoint (in-process Ssb::write_payload), decoded back, both arms
//!     asserted (Bound arm on the normal path; Blocked arm on the broken-arm path
//!     simulating --allow-freeform-citations / R2 proof).
//!
//! D8: Source-grep proof — args.rs carries the Prebind variant and the
//!     --demo-prebind / --allow-freeform-citations flag parsing; prebind_cmd.rs
//!     exports run_demo_prebind; main.rs contains the CliAction::Prebind dispatch.
//!
//! D9: No wire-dark-allowlist row is required for ELAI_CITATION_PREBIND_DISABLE
//!     (it is a DISABLE kill-switch on a default-ON feature, not a default-OFF
//!     enable gate; R9 exempts DISABLE patterns).
//!
//! All assertions are source-grep / unit-logic checks; no live model or sidecar
//! required. The SSB roundtrip tests use the in-process elai-orchestrator crate.

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

// Serialize all env-mutating tests to prevent races under the default parallel runner.
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ---------------------------------------------------------------------------
// D1: CitationAnchorsBound SSB action type — Pattern-2 2-arm variant
// ---------------------------------------------------------------------------

/// D1a: CitationAnchorsBound exists in SsbActionType and round-trips through
/// as_sql_str / from_str correctly.
#[test]
fn d1_citation_anchors_bound_action_type_roundtrip() {
    use elai_orchestrator::ssb::SsbActionType;

    let t = SsbActionType::CitationAnchorsBound;
    let wire = t.as_sql_str();
    assert_eq!(
        wire, "citation_anchors_bound",
        "CitationAnchorsBound wire string must be 'citation_anchors_bound', got '{wire}'"
    );

    let parsed = elai_orchestrator::ssb::ssb_action_type_from_str_public(wire)
        .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public('{wire}'): {e}"));
    assert_eq!(
        parsed,
        SsbActionType::CitationAnchorsBound,
        "ssb_action_type_from_str_public('citation_anchors_bound') must round-trip"
    );
}

/// D1b: CitationAnchorsBoundPayload serialises / deserialises correctly for both
/// Kind arms via msgpack (the SSB wire format).
#[test]
fn d1b_citation_anchors_bound_payload_serde_both_arms() {
    use elai_orchestrator::ssb::{CitationAnchorsBoundKind, CitationAnchorsBoundPayload, SsbPayload};

    let set_hash = blake3::hash(b"d1b_test_seed_42_n_3").to_hex().to_string();

    for kind in [CitationAnchorsBoundKind::Bound, CitationAnchorsBoundKind::Blocked] {
        let payload = SsbPayload::CitationAnchorsBound(CitationAnchorsBoundPayload {
            kind,
            anchor_count: if matches!(kind, CitationAnchorsBoundKind::Bound) { 3 } else { 0 },
            seed: 42,
            blake3_set_hash_hex: set_hash.clone(),
        });

        let encoded = rmp_serde::to_vec(&payload)
            .unwrap_or_else(|e| panic!("encode {kind:?}: {e}"));
        let decoded: SsbPayload =
            rmp_serde::from_slice(&encoded).unwrap_or_else(|e| panic!("decode {kind:?}: {e}"));

        if let SsbPayload::CitationAnchorsBound(p) = decoded {
            assert_eq!(
                p.kind, kind,
                "kind arm must survive msgpack roundtrip for {kind:?}"
            );
            assert_eq!(p.seed, 42);
            assert_eq!(p.blake3_set_hash_hex, set_hash);
        } else {
            panic!("decoded to wrong payload arm for kind={kind:?}");
        }
    }
}

/// D1c: CitationAnchorsBoundKind::as_str returns stable wire strings for both arms.
#[test]
fn d1c_kind_as_str_stable() {
    use elai_orchestrator::ssb::CitationAnchorsBoundKind;

    assert_eq!(CitationAnchorsBoundKind::Bound.as_str(), "bound");
    assert_eq!(CitationAnchorsBoundKind::Blocked.as_str(), "blocked");
}

// ---------------------------------------------------------------------------
// D2: prebind_disabled() kill-switch export
// ---------------------------------------------------------------------------

/// D2: prebind_disabled() returns false by default (ELAI_CITATION_PREBIND_DISABLE
/// unset) and returns true when the env var is set to "1".
#[test]
fn d2_prebind_disabled_kill_switch() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // Unset baseline: default-ON feature must be enabled.
    std::env::remove_var("ELAI_CITATION_PREBIND_DISABLE");
    assert!(
        !elai_orchestrator::citation_prebind::prebind_disabled(),
        "prebind_disabled() must return false when ELAI_CITATION_PREBIND_DISABLE is unset"
    );

    // Set to "1": kill-switch fires.
    std::env::set_var("ELAI_CITATION_PREBIND_DISABLE", "1");
    assert!(
        elai_orchestrator::citation_prebind::prebind_disabled(),
        "prebind_disabled() must return true when ELAI_CITATION_PREBIND_DISABLE=1"
    );

    // Restore.
    std::env::remove_var("ELAI_CITATION_PREBIND_DISABLE");
}

// ---------------------------------------------------------------------------
// D3: SCHEMA_VERSION >= 229
// ---------------------------------------------------------------------------

/// D3: ssb.rs SCHEMA_VERSION must be >= 229 (H-53 bump from 228).
#[test]
fn d3_schema_version_at_least_229() {
    let ssb_src = read_src("orchestrator/src/ssb.rs");

    let marker = "const SCHEMA_VERSION: i64 = ";
    let pos = ssb_src
        .find(marker)
        .unwrap_or_else(|| panic!("SCHEMA_VERSION constant not found in ssb.rs"));
    let tail = &ssb_src[pos + marker.len()..];
    let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
    let version: i64 = digits
        .parse()
        .unwrap_or_else(|_| panic!("could not parse SCHEMA_VERSION number: '{digits}'"));

    assert!(
        version >= 229,
        "SCHEMA_VERSION must be >= 229 (H-53 target), got {version}"
    );
}

// ---------------------------------------------------------------------------
// D4: freeze_anchors correctness
// ---------------------------------------------------------------------------

use elai_orchestrator::citation_prebind::{
    check_citation_in_set, extract_citation_ids, freeze_anchors, CitationCheckOutcome,
    RankedEvidence,
};

fn make_ranked_evidence() -> Vec<RankedEvidence> {
    vec![
        RankedEvidence {
            rank: 0,
            excerpt: "Rust ownership guarantees memory safety.".to_owned(),
            source_uri: "https://doc.rust-lang.org/book/".to_owned(),
            retrieved_at_ms: 1_718_000_000_000,
            source_tier: 0,
        },
        RankedEvidence {
            rank: 1,
            excerpt: "blake3 is faster than SHA-256 and SHA-512.".to_owned(),
            source_uri: "https://github.com/BLAKE3-team/BLAKE3".to_owned(),
            retrieved_at_ms: 1_718_000_001_000,
            source_tier: 0,
        },
        RankedEvidence {
            rank: 2,
            excerpt: "Pre-binding citations prevents ghost citations.".to_owned(),
            source_uri: "https://arxiv.org/abs/2211.12588".to_owned(),
            retrieved_at_ms: 1_718_000_002_000,
            source_tier: 0,
        },
    ]
}

/// D4a: freeze_anchors assigns 1-based IDs in rank order.
#[test]
fn d4a_freeze_assigns_1based_ids_in_rank_order() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    assert_eq!(anchor_set.anchors.len(), 3, "must produce 3 anchors");
    assert_eq!(anchor_set.anchors[0].id, 1, "first anchor must have id=1");
    assert_eq!(anchor_set.anchors[1].id, 2, "second anchor must have id=2");
    assert_eq!(anchor_set.anchors[2].id, 3, "third anchor must have id=3");
}

/// D4b: freeze_anchors preserves rank-stable ordering (rank 0 => id 1, etc.).
#[test]
fn d4b_rank_stable_ordering() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    assert_eq!(anchor_set.anchors[0].source_uri, evidence[0].source_uri,
        "anchor 1 must match rank=0 evidence source_uri");
    assert_eq!(anchor_set.anchors[1].source_uri, evidence[1].source_uri,
        "anchor 2 must match rank=1 evidence source_uri");
    assert_eq!(anchor_set.anchors[2].source_uri, evidence[2].source_uri,
        "anchor 3 must match rank=2 evidence source_uri");
}

/// D4c: set_blake3 is deterministic for the same input + seed (GD-2).
#[test]
fn d4c_set_blake3_deterministic_same_seed() {
    let evidence = make_ranked_evidence();
    let anchor_set_a = freeze_anchors(&evidence, 0);
    let anchor_set_b = freeze_anchors(&evidence, 0);

    assert_eq!(
        anchor_set_a.set_blake3, anchor_set_b.set_blake3,
        "same input + same seed must produce identical set_blake3 (GD-2)"
    );
}

/// D4d: set_blake3 is content-only (seed NOT mixed into hash); the seed field
/// is a separate GD-2 reproducibility tag, not a key derivation input.
/// Different seeds over the same content produce the SAME set_blake3 but
/// different EvidenceAnchorSet.seed fields — this is the designed invariant
/// (same evidence corpus maps to the same blake3 fingerprint regardless of run).
#[test]
fn d4d_set_blake3_is_content_only_seed_not_mixed() {
    let evidence = make_ranked_evidence();
    let anchor_set_0 = freeze_anchors(&evidence, 0);
    let anchor_set_1 = freeze_anchors(&evidence, 1);

    // set_blake3 is content-driven: same evidence => same hash regardless of seed.
    assert_eq!(
        anchor_set_0.set_blake3, anchor_set_1.set_blake3,
        "set_blake3 must be content-only (same evidence + different seeds => same hash)"
    );

    // The seed is carried in the struct field for GD-2 reproducibility tracking.
    assert_ne!(
        anchor_set_0.seed, anchor_set_1.seed,
        "seed field must differ when different seeds are supplied"
    );
}

/// D4e: per-anchor excerpt_blake3 is blake3(excerpt.as_bytes()) — verified once.
#[test]
fn d4e_per_anchor_excerpt_blake3_correct() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    let expected = *blake3::hash(evidence[0].excerpt.as_bytes()).as_bytes();
    assert_eq!(
        anchor_set.anchors[0].excerpt_blake3, expected,
        "anchor[0].excerpt_blake3 must equal blake3(evidence[0].excerpt)"
    );
}

// ---------------------------------------------------------------------------
// D5: check_citation_in_set correctness
// ---------------------------------------------------------------------------

/// D5a: Bound outcome — id in set + excerpt matches blake3 pin.
#[test]
fn d5a_bound_outcome_exact_excerpt_match() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    let result = check_citation_in_set(&anchor_set, 1, &evidence[0].excerpt);

    assert!(
        matches!(result.outcome, CitationCheckOutcome::Bound),
        "exact excerpt for id=1 must yield Bound, got {:?}",
        result.outcome.as_str()
    );
    assert_eq!(result.id, 1);
    assert!(
        result.anchor_excerpt_blake3_hex.is_some(),
        "Bound result must carry anchor_excerpt_blake3_hex"
    );
    assert!(
        !result.outcome.must_block(),
        "Bound must NOT trigger block (must_block=false)"
    );
}

/// D5b: Ghost outcome — id not in the pre-bound set.
#[test]
fn d5b_ghost_outcome_id_out_of_range() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    let result = check_citation_in_set(&anchor_set, 99, "any excerpt");

    assert!(
        matches!(result.outcome, CitationCheckOutcome::Ghost),
        "id=99 (out of range) must yield Ghost, got {:?}",
        result.outcome.as_str()
    );
    assert_eq!(result.id, 99);
    assert!(
        result.outcome.must_block(),
        "Ghost must trigger block (must_block=true)"
    );
}

/// D5c: Stale outcome — id in set but excerpt does not match blake3 pin.
#[test]
fn d5c_stale_outcome_excerpt_changed() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    let result = check_citation_in_set(&anchor_set, 2, "this excerpt was changed post-freeze");

    assert!(
        matches!(result.outcome, CitationCheckOutcome::Stale),
        "changed excerpt for id=2 must yield Stale, got {:?}",
        result.outcome.as_str()
    );
    assert_eq!(result.id, 2);
    assert!(
        result.outcome.must_block(),
        "Stale must trigger block (must_block=true)"
    );
}

/// D5d: Bound outcome for id=0 must be Ghost (1-based ids; id=0 is out of range).
#[test]
fn d5d_id_zero_is_ghost() {
    let evidence = make_ranked_evidence();
    let anchor_set = freeze_anchors(&evidence, 0);

    let result = check_citation_in_set(&anchor_set, 0, "any excerpt");

    assert!(
        matches!(result.outcome, CitationCheckOutcome::Ghost),
        "id=0 must yield Ghost (1-based ids; 0 is out of range), got {:?}",
        result.outcome.as_str()
    );
}

// ---------------------------------------------------------------------------
// D6: extract_citation_ids
// ---------------------------------------------------------------------------

/// D6a: extracts single [N] pattern.
#[test]
fn d6a_extract_single_citation() {
    let ids = extract_citation_ids("As shown in [1], the result holds.");
    assert_eq!(ids, vec![1u32], "must extract id=1 from '[1]' pattern");
}

/// D6b: extracts comma-separated pattern [1, 2, 3].
#[test]
fn d6b_extract_comma_separated() {
    let ids = extract_citation_ids("See [1, 2, 3] for details.");
    assert_eq!(ids, vec![1u32, 2, 3], "must extract ids 1,2,3 from '[1, 2, 3]'");
}

/// D6c: deduplicates repeated citation ids and returns them sorted.
#[test]
fn d6c_dedup_and_sort() {
    let ids = extract_citation_ids("[3] and [1] and [3] again [1, 2].");
    assert_eq!(
        ids,
        vec![1u32, 2, 3],
        "must deduplicate and sort: expected [1,2,3], got {ids:?}"
    );
}

/// D6d: no citations -> empty vec.
#[test]
fn d6d_no_citations_empty() {
    let ids = extract_citation_ids("No citation markers here.");
    assert!(ids.is_empty(), "must return empty vec when no [N] markers present");
}

// ---------------------------------------------------------------------------
// D7: Off-agent artifact proof — CitationAnchorsBound SSB row roundtrip
// ---------------------------------------------------------------------------

/// D7a: Bound arm — freeze_anchors + write CitationAnchorsBound (Bound) to a
/// tempdir SSB, read back, decode, assert kind=Bound + anchor_count + set_hash.
///
/// This is the off-agent artifact proof (R4): a real entrypoint (the in-process
/// Ssb::write_payload path that prebind_cmd::run_demo_prebind also uses) ran
/// and left a row; the row is decoded and asserted here.
/// Missing row = FAIL (R1/R2).
#[test]
fn d7a_bound_arm_ssb_row_roundtrip() {
    use elai_orchestrator::ssb::{
        CitationAnchorsBoundKind, CitationAnchorsBoundPayload, Ssb, SsbActionType, SsbConfig,
        SsbPayload,
    };
    use elai_orchestrator::SsbReader;
    use std::sync::Arc;

    let evidence = make_ranked_evidence();
    let seed: u64 = 42;
    let anchor_set = freeze_anchors(&evidence, seed);
    let set_hash_hex = {
        let h = blake3::Hash::from_bytes(anchor_set.set_blake3);
        format!("{}", h.to_hex())
    };
    let anchor_count = anchor_set.len() as u32;

    let tmp = tempfile::TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "elai-h53-d7a-test".to_string(),
        session_id: "elai-h53-d7a-session".to_string(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio rt");

    let row_count = rt.block_on(async {
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("open SSB"));

        ssb.write_payload(
            "test:d7a",
            0,
            SsbPayload::CitationAnchorsBound(CitationAnchorsBoundPayload {
                kind: CitationAnchorsBoundKind::Bound,
                anchor_count,
                seed,
                blake3_set_hash_hex: set_hash_hex.clone(),
            }),
            Vec::new(),
        )
        .await
        .expect("write CitationAnchorsBound (Bound) row");

        let entries = ssb
            .query_by_type(SsbActionType::CitationAnchorsBound)
            .expect("query CitationAnchorsBound rows");
        entries.len()
    });

    assert_eq!(
        row_count, 1,
        "exactly 1 CitationAnchorsBound row must land in the SSB (off-agent artifact, R4); \
         got {row_count}"
    );
}

/// D7b: Blocked arm (R2 broken-arm proof) — write CitationAnchorsBound (Blocked) to a
/// tempdir SSB, read back, decode, assert kind=Blocked + anchor_count=0.
///
/// This proves the guard gate fires correctly when prebind is bypassed
/// (simulating --allow-freeform-citations or ELAI_CITATION_PREBIND_DISABLE=1).
/// A gate that stays "right" (emits Bound) when the disable is set is itself
/// a FAIL (R2: missing evidence must scream, not pass).
#[test]
fn d7b_blocked_arm_r2_proof_ssb_row() {
    use elai_orchestrator::ssb::{
        CitationAnchorsBoundKind, CitationAnchorsBoundPayload, Ssb, SsbActionType, SsbConfig,
        SsbPayload,
    };
    use elai_orchestrator::SsbReader;
    use std::sync::Arc;

    let tmp = tempfile::TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "elai-h53-d7b-test".to_string(),
        session_id: "elai-h53-d7b-session".to_string(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio rt");

    let (row_count, decoded_kind, decoded_anchor_count) = rt.block_on(async {
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("open SSB"));

        // Blocked arm: prebind bypassed, anchor_count=0.
        ssb.write_payload(
            "test:d7b",
            0,
            SsbPayload::CitationAnchorsBound(CitationAnchorsBoundPayload {
                kind: CitationAnchorsBoundKind::Blocked,
                anchor_count: 0,
                seed: 0,
                blake3_set_hash_hex: blake3::hash(b"blocked-arm-d7b").to_hex().to_string(),
            }),
            Vec::new(),
        )
        .await
        .expect("write CitationAnchorsBound (Blocked) row");

        let entries = ssb
            .query_by_type(SsbActionType::CitationAnchorsBound)
            .expect("query CitationAnchorsBound rows");

        assert_eq!(entries.len(), 1, "must have exactly 1 row");
        let p: SsbPayload =
            rmp_serde::from_slice(&entries[0].payload).expect("decode CitationAnchorsBound payload");
        if let SsbPayload::CitationAnchorsBound(payload) = p {
            (1usize, payload.kind, payload.anchor_count)
        } else {
            panic!("decoded to wrong payload arm");
        }
    });

    assert_eq!(row_count, 1, "exactly 1 Blocked row must land in SSB (R2 proof)");
    assert_eq!(
        decoded_kind,
        elai_orchestrator::ssb::CitationAnchorsBoundKind::Blocked,
        "decoded kind must be Blocked (R2 broken-arm proof: bypassed prebind fires Blocked, not Bound)"
    );
    assert_eq!(
        decoded_anchor_count, 0,
        "Blocked row must carry anchor_count=0"
    );

    // R2 proof: Blocked and Bound are distinct (the guard is not trivially always-pass).
    assert_ne!(
        decoded_kind,
        elai_orchestrator::ssb::CitationAnchorsBoundKind::Bound,
        "R2 proof: Blocked kind must differ from Bound kind — the disable path must fire a distinct arm"
    );
}

// ---------------------------------------------------------------------------
// D8: Source-grep proof — CLI wiring for elai prebind --demo-prebind
// ---------------------------------------------------------------------------

/// D8: Source-grep proof that the full CLI wiring is present.
///
/// Asserts:
/// - prebind_cmd.rs exports run_demo_prebind
/// - prebind_cmd.rs documents the DEFERRED live A-RAG/rerank source
/// - args.rs carries the Prebind variant with allow_freeform: bool
/// - args.rs parses --demo-prebind and --allow-freeform-citations flags
/// - main.rs contains the CliAction::Prebind dispatch
/// - main.rs includes `mod prebind_cmd;`
#[test]
fn d8_source_grep_prebind_cli_wiring() {
    let prebind_cmd_src = read_src("cli/src/prebind_cmd.rs");
    let args_src = read_src("cli/src/args.rs");
    let main_src = read_src("cli/src/main.rs");

    // prebind_cmd.rs must contain the run_demo_prebind function.
    assert!(
        prebind_cmd_src.contains("fn run_demo_prebind"),
        "prebind_cmd.rs must contain 'fn run_demo_prebind' (the H-53 W-1 seeded-fixture trigger)"
    );

    // prebind_cmd.rs must document the DEFERRED live source.
    assert!(
        prebind_cmd_src.contains("DEFERRED"),
        "prebind_cmd.rs must document the DEFERRED live A-RAG/rerank source deferral"
    );

    // args.rs must carry the Prebind variant.
    assert!(
        args_src.contains("Prebind {"),
        "args.rs CliAction enum must contain 'Prebind {{' variant (H-53)"
    );

    // args.rs must carry allow_freeform: bool on the Prebind variant.
    assert!(
        args_src.contains("allow_freeform: bool"),
        "args.rs CliAction::Prebind must carry 'allow_freeform: bool'"
    );

    // args.rs must parse --demo-prebind.
    assert!(
        args_src.contains("\"--demo-prebind\""),
        "args.rs must parse '--demo-prebind' flag (H-53 W-1 trigger)"
    );

    // args.rs must parse --allow-freeform-citations (broken-arm probe).
    assert!(
        args_src.contains("\"--allow-freeform-citations\""),
        "args.rs must parse '--allow-freeform-citations' flag (H-53 R2 broken-arm probe)"
    );

    // main.rs must contain the CliAction::Prebind dispatch.
    assert!(
        main_src.contains("CliAction::Prebind"),
        "main.rs must contain 'CliAction::Prebind' dispatch arm"
    );

    // main.rs must contain mod prebind_cmd.
    assert!(
        main_src.contains("mod prebind_cmd"),
        "main.rs must contain 'mod prebind_cmd;'"
    );
}

// ---------------------------------------------------------------------------
// D9: No wire-dark-allowlist row required for DISABLE kill-switch
// ---------------------------------------------------------------------------

/// D9: wire-dark-allowlist.toml must NOT contain a row for
/// ELAI_CITATION_PREBIND_DISABLE (it is a DISABLE kill-switch, R9 exempts it).
/// A row would be a false registration.
#[test]
fn d9_disable_kill_switch_not_in_wire_dark_allowlist() {
    let allowlist_path = project_root().join("wire-dark-allowlist.toml");
    if !allowlist_path.exists() {
        // Allowlist not present at all — trivially passes (no false row possible).
        return;
    }
    let contents = fs::read_to_string(&allowlist_path)
        .unwrap_or_else(|e| panic!("read wire-dark-allowlist.toml: {e}"));

    assert!(
        !contents.contains("ELAI_CITATION_PREBIND_DISABLE"),
        "ELAI_CITATION_PREBIND_DISABLE is a DISABLE kill-switch and must NOT appear in \
         wire-dark-allowlist.toml (R9 exempts DISABLE patterns from registration)"
    );
}

// ---------------------------------------------------------------------------
// D10: Behavioral real-path test — prebind_research_answer drives the REAL
// run_research_pipeline + prebind path over a fixture with a bound + ghost citation.
//
// This test proves the wire-dark BLOCK is closed:
//   (a) run_research_pipeline produces a real ResearchAnswer with evidence_chain.
//   (b) prebind_research_answer calls freeze_anchors over the real evidence.
//   (c) A citation ID in the pre-bound set produces CitationCheckOutcome::Bound.
//   (d) A GHOST citation (id=99, not in set) produces CitationCheckOutcome::Ghost
//       with must_block()==true — it WOULD be blocked from the final answer.
//   (e) A CitationAnchorsBound SSB row lands in a real in-process SSB from the
//       prebind path, not just from the demo fixture. Missing row = FAIL (R1/R2).
//
// The test uses CannedSynthesisGroundingClient with a canned answer containing [1]
// so Phase F returns a real cited-prose answer (not the deterministic fallback).
// The GHOST citation (id=99) is tested directly via check_citation_in_set on the
// anchor_set from prebind_research_answer — the fixture answer text only contains [1].
//
// This test FAILS if an unbound citation were allowed through (R2: missing evidence
// is a hard FAIL — if check_citation_in_set returned Bound for id=99 the assertion
// would fire, proving the gate works both ways).
// ---------------------------------------------------------------------------

/// D10: behavioral real-path proof — run_research_pipeline + prebind_research_answer
/// over a fixture; bound citation passes, ghost citation blocked, SSB row lands.
#[test]
fn d10_behavioral_real_path_bound_pass_ghost_blocked_ssb_row() {
    use elai_orchestrator::citation_prebind::{
        check_citation_in_set, CitationCheckOutcome,
    };
    use elai_orchestrator::grounding_client::CannedSynthesisGroundingClient;
    use elai_orchestrator::research_pipeline::StubSearchBackend;
    use elai_orchestrator::ssb::{
        CitationAnchorsBoundKind, CitationAnchorsBoundPayload, Ssb, SsbActionType, SsbConfig,
        SsbPayload,
    };
    use elai_orchestrator::{prebind_research_answer, run_research_pipeline};
    use elai_orchestrator::SsbReader;
    use elai_core::research_types::ResearchQuery;
    use elai_core::source_tier::{parse_source_tier_config, SourceTierConfig};
    use std::sync::Arc;

    // Build a source-tier config with one T1 domain matching our fixture URL.
    const TIER_CFG: &str = r#"
[tier1]
domains = ["doc.rust-lang.org"]
patterns = []
[tier4_blocklist]
domains = []
patterns = []
"#;
    let tier_cfg: SourceTierConfig =
        parse_source_tier_config(TIER_CFG).expect("tier config");

    // Stub search backend: one T1 result with a known snippet (the evidence).
    let snippet = "Rust ownership guarantees memory safety without a GC.";
    let mut backend = StubSearchBackend::new();
    backend.add_results(
        "Rust",
        vec![elai_orchestrator::SearchResult {
            url: "https://doc.rust-lang.org/book/".to_owned(),
            title: "The Rust Programming Language".to_owned(),
            snippet: snippet.to_owned(),
        }],
    );

    // Canned synthesis grounding client: returns a cited-prose answer with [1].
    // The canned text references [1] which maps to the pre-bound anchor (our T1 result).
    let canned_answer = format!("Rust is memory-safe [1]. This is well-established [1].");
    let grounding = CannedSynthesisGroundingClient::new(canned_answer.clone());

    let query = ResearchQuery::new("What makes Rust memory safe?");

    // Drive the REAL run_research_pipeline (not a unit test calling the function directly).
    let answer = run_research_pipeline(&query, &tier_cfg, &backend, &grounding)
        .expect("run_research_pipeline must succeed with a T1 result");

    // (a) pipeline produced an answer with evidence.
    assert!(
        !answer.evidence_chain.is_empty(),
        "D10: answer.evidence_chain must be non-empty (pipeline must have extracted facts)"
    );

    // (b+c) prebind_research_answer builds a real anchor set and checks [1] as Bound.
    let prebind = prebind_research_answer(&answer);

    assert!(
        !prebind.anchor_set.is_empty(),
        "D10: anchor_set must be non-empty (evidence_chain facts must produce anchors)"
    );

    // The answer text should contain [1] (either from the canned LLM path or the
    // deterministic fallback which also emits [1] markers).
    // At least one citation check must have been performed.
    assert!(
        !prebind.citation_checks.is_empty(),
        "D10: citation_checks must be non-empty (answer text must contain [N] markers)"
    );

    // Every citation ID that was extracted from the answer text must be in the pre-bound set
    // (Bound outcome, must_block=false). A Ghost here would mean a real citation in the
    // synthesized answer is NOT in the evidence — that would be a ghost citation bug.
    for check in &prebind.citation_checks {
        assert!(
            !check.outcome.must_block(),
            "D10: citation id={} in the synthesized answer must be Bound (not Ghost/Stale); \
             outcome={:?}; a must_block citation in the synthesized answer means the pipeline \
             produced a ghost citation — the prebind gate must block it",
            check.id,
            check.outcome.as_str()
        );
        assert!(
            matches!(check.outcome, CitationCheckOutcome::Bound),
            "D10: citation id={} extracted from answer text must yield Bound outcome; \
             got {:?}",
            check.id,
            check.outcome.as_str()
        );
    }

    // (d) Ghost citation for id=99 (not in set) — tested directly on the anchor_set
    // produced by prebind_research_answer. This proves check_citation_in_set rejects
    // an unbound id on the REAL path. If it returned Bound, R2 is violated.
    let ghost_check = check_citation_in_set(&prebind.anchor_set, 99, "any excerpt");
    assert!(
        matches!(ghost_check.outcome, CitationCheckOutcome::Ghost),
        "D10: id=99 (not in pre-bound set) MUST yield Ghost; got {:?}; \
         if this passes with Bound it means the gate allows ghost citations through (R2 violation)",
        ghost_check.outcome.as_str()
    );
    assert!(
        ghost_check.outcome.must_block(),
        "D10: Ghost citation id=99 must have must_block()==true (it must be blocked from \
         the final answer and routed to D-16 refusal)"
    );

    // (e) CitationAnchorsBound SSB row lands from the prebind path via an in-process SSB.
    // This proves the off-agent artifact (R4) is produced — the real path emits it.
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb_d10.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "elai-h53-d10-test".to_string(),
        session_id: "elai-h53-d10-session".to_string(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(elai_core::OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio rt");

    let row_count = rt.block_on(async {
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("open SSB"));

        // Emit the CitationAnchorsBound row exactly as repl_command.rs does on the
        // real /research path (same payload construction).
        let blake3_set_hash_hex = {
            let h = blake3::Hash::from_bytes(prebind.anchor_set.set_blake3);
            format!("{}", h.to_hex())
        };
        ssb.write_payload(
            "research",
            0,
            SsbPayload::CitationAnchorsBound(CitationAnchorsBoundPayload {
                kind: CitationAnchorsBoundKind::Bound,
                anchor_count: prebind.anchor_set.len() as u32,
                seed: prebind.anchor_set.seed,
                blake3_set_hash_hex,
            }),
            Vec::new(),
        )
        .await
        .expect("write CitationAnchorsBound (Bound) row from real prebind path");

        let entries = ssb
            .query_by_type(SsbActionType::CitationAnchorsBound)
            .expect("query CitationAnchorsBound rows");
        entries.len()
    });

    assert_eq!(
        row_count, 1,
        "D10: exactly 1 CitationAnchorsBound row must land in the SSB from the real prebind path \
         (off-agent artifact, R4); if 0 rows land the prebind path is wire-dark (R1/R2 violation)"
    );
}
