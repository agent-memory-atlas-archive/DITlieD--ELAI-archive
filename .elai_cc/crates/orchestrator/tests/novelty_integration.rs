//! D-15 — observable_effect integration test for the discovery-vs-search
//! NoveltyClass labeling pass.
//!
//! Trigger: classify 3 synthetic reasoning templates with different
//! [`NoveltyEvidence`] bundles (high-match -> Search, medium -> DistantSearch,
//! zero-match -> DiscoveryCandidate) through the
//! `agent_pool::dispatch_novelty` producer.
//!
//! Asserts:
//!   1. Each dispatch returns the expected closed
//!      [`NoveltyClass`] variant.
//!   2. Each dispatch emits exactly one `NoveltyClassified` SSB payload
//!      through the in-process [`VecNoveltyClassifiedEmitter`].
//!   3. The recorded payloads carry the correct class label, the
//!      caller-supplied `dkg_match_count`, the
//!      `near_duplicate_threshold` (clamped to `[0.0, 1.0]`), and the
//!      injected `ts_ms`.
//!
//! Grounding:
//!   * arXiv:2406.05761 — *LLM Novelty Detection*: three-bucket
//!     classification.
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
//!     §1 Mode C + §4.8.

use std::path::PathBuf;

use elai_orchestrator::{
    agent_pool::dispatch_novelty,
    archive_chain::ArchiveChain,
    checker_registry::CheckerResult,
    novelty::{
        novelty_evidence_from_checker_result, NoveltyClass, NoveltyConfig,
        NoveltyEvidence, VecNoveltyClassifiedEmitter,
    },
    reasoning_template::{
        compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
        TemplateBody, TemplateId, VerifierKind,
    },
    runtime_config::{NoveltyRuntimeConfig, ELAI_NOVELTY_LABEL_ENABLE_ENV},
};

fn make_template(body: &str) -> ReasoningTemplate {
    let bounded = TemplateBody::try_new(body).expect("body fits TEMPLATE_BODY_BOUND");
    let hash = compute_content_hash(bounded.as_str(), None);
    ReasoningTemplate {
        id: TemplateId::new_v4(),
        domain: Domain::Code,
        pattern_kind: PatternKind::Decomposition,
        template_body: bounded,
        eligibility_predicate: Predicate::Always,
        verifier_required: VerifierKind::Refuse,
        content_hash: hash,
        created_at_ms: 1_700_000_000_000,
        parent_id: None,
        archive_status: ArchiveStatus::Promoted,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_novelty_routes_three_classes_and_emits_three_ssb_rows() {
    let archive = ArchiveChain::new(PathBuf::from("test-novelty-archive.jsonl"));
    let emitter = VecNoveltyClassifiedEmitter::new();
    let cfg = NoveltyConfig::default();

    // 1) Search arm — many DKG matches, high near-duplicate score.
    let t_search = make_template("body-search-template");
    let c_search = dispatch_novelty(
        &t_search,
        &archive,
        NoveltyEvidence::new(8, 0.92, 0.05, 0.85),
        cfg,
        &emitter,
        1_700_000_900_000,
    )
    .await;
    assert_eq!(c_search, NoveltyClass::Search);

    // 2) DistantSearch arm — some DKG matches, mid-range near-duplicate.
    let t_distant = make_template("body-distant-template");
    let c_distant = dispatch_novelty(
        &t_distant,
        &archive,
        NoveltyEvidence::new(3, 0.55, 0.30, 0.40),
        cfg,
        &emitter,
        1_700_000_900_001,
    )
    .await;
    assert_eq!(c_distant, NoveltyClass::DistantSearch);

    // 3) DiscoveryCandidate arm — no DKG matches, very low near-duplicate.
    let t_discovery = make_template("body-discovery-template");
    let c_discovery = dispatch_novelty(
        &t_discovery,
        &archive,
        NoveltyEvidence::new(0, 0.05, 1.30, 0.02),
        cfg,
        &emitter,
        1_700_000_900_002,
    )
    .await;
    assert_eq!(c_discovery, NoveltyClass::DiscoveryCandidate);

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 3, "expected one SSB payload per dispatch");

    let search_row = &rows[0];
    assert_eq!(search_row.template_id, t_search.id.as_hex());
    assert_eq!(search_row.class, "search");
    assert_eq!(search_row.dkg_match_count, 8);
    assert!((search_row.near_duplicate_threshold - 0.92).abs() < 1e-9);
    assert_eq!(search_row.ts_ms, 1_700_000_900_000);

    let distant_row = &rows[1];
    assert_eq!(distant_row.template_id, t_distant.id.as_hex());
    assert_eq!(distant_row.class, "distant_search");
    assert_eq!(distant_row.dkg_match_count, 3);
    assert!((distant_row.near_duplicate_threshold - 0.55).abs() < 1e-9);
    assert_eq!(distant_row.ts_ms, 1_700_000_900_001);

    let discovery_row = &rows[2];
    assert_eq!(discovery_row.template_id, t_discovery.id.as_hex());
    assert_eq!(discovery_row.class, "discovery_candidate");
    assert_eq!(discovery_row.dkg_match_count, 0);
    assert!((discovery_row.near_duplicate_threshold - 0.05).abs() < 1e-9);
    assert_eq!(discovery_row.ts_ms, 1_700_000_900_002);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_novelty_routes_near_duplicate_to_search() {
    // The brief's GAP-7 named-threshold case: cosine similarity >=
    // 0.85 with the DKG-match floor should route to Search under
    // the default NoveltyConfig (which uses 0.8 as its
    // near_duplicate_high — 0.85 clears it).
    let archive = ArchiveChain::new(PathBuf::from("test-novelty-search.jsonl"));
    let emitter = VecNoveltyClassifiedEmitter::new();
    let t = make_template("body-near-duplicate");
    let class = dispatch_novelty(
        &t,
        &archive,
        NoveltyEvidence::new(6, 0.86, 0.0, 0.0),
        NoveltyConfig::default(),
        &emitter,
        1_700_000_920_000,
    )
    .await;
    assert_eq!(class, NoveltyClass::Search);
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].class, "search");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_novelty_routes_distant_band_to_distant_search() {
    // The brief's GAP-7 named-threshold case: cosine similarity in
    // (0.40, 0.85) should route to DistantSearch under the default
    // NoveltyConfig.
    let archive = ArchiveChain::new(PathBuf::from("test-novelty-distant.jsonl"));
    let emitter = VecNoveltyClassifiedEmitter::new();
    let t = make_template("body-distant-band");
    let class = dispatch_novelty(
        &t,
        &archive,
        NoveltyEvidence::new(2, 0.60, 0.0, 0.0),
        NoveltyConfig::default(),
        &emitter,
        1_700_000_920_001,
    )
    .await;
    assert_eq!(class, NoveltyClass::DistantSearch);
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].class, "distant_search");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_novelty_routes_highly_novel_to_discovery_candidate() {
    // The brief's GAP-7 named-threshold case: cosine similarity <
    // 0.40 with zero DKG matches should route to DiscoveryCandidate
    // under the default NoveltyConfig.
    let archive = ArchiveChain::new(PathBuf::from("test-novelty-discovery.jsonl"));
    let emitter = VecNoveltyClassifiedEmitter::new();
    let t = make_template("body-highly-novel");
    let class = dispatch_novelty(
        &t,
        &archive,
        NoveltyEvidence::new(0, 0.30, 0.0, 0.0),
        NoveltyConfig::default(),
        &emitter,
        1_700_000_920_002,
    )
    .await;
    assert_eq!(class, NoveltyClass::DiscoveryCandidate);
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].class, "discovery_candidate");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn novelty_evidence_from_checker_result_round_trips_through_dispatch() {
    // Caller path: D-11 CheckerResult -> NoveltyEvidence (via the
    // shipped projection) -> dispatch_novelty -> NoveltyClass. The
    // projection is deterministic, so the dispatch arm assignment
    // is stable across runs.
    let archive = ArchiveChain::new(PathBuf::from("test-novelty-projection.jsonl"));
    let cfg = NoveltyConfig::default();

    // Pass -> Search.
    let emitter_pass = VecNoveltyClassifiedEmitter::new();
    let t_pass = make_template("body-checker-pass");
    let class_pass = dispatch_novelty(
        &t_pass,
        &archive,
        novelty_evidence_from_checker_result(&CheckerResult::Pass),
        cfg,
        &emitter_pass,
        1_700_000_930_000,
    )
    .await;
    assert_eq!(class_pass, NoveltyClass::Search);
    let rows_pass = emitter_pass.rows().await;
    assert_eq!(rows_pass.len(), 1);
    assert_eq!(rows_pass[0].class, "search");
    assert_eq!(rows_pass[0].dkg_match_count, 8);

    // Fail -> DistantSearch.
    let emitter_fail = VecNoveltyClassifiedEmitter::new();
    let t_fail = make_template("body-checker-fail");
    let class_fail = dispatch_novelty(
        &t_fail,
        &archive,
        novelty_evidence_from_checker_result(&CheckerResult::Fail {
            reason: "checker rejected".into(),
        }),
        cfg,
        &emitter_fail,
        1_700_000_930_001,
    )
    .await;
    assert_eq!(class_fail, NoveltyClass::DistantSearch);
    let rows_fail = emitter_fail.rows().await;
    assert_eq!(rows_fail.len(), 1);
    assert_eq!(rows_fail[0].class, "distant_search");
    assert_eq!(rows_fail[0].dkg_match_count, 3);

    // Inapplicable -> DiscoveryCandidate.
    let emitter_inap = VecNoveltyClassifiedEmitter::new();
    let t_inap = make_template("body-checker-inapplicable");
    let class_inap = dispatch_novelty(
        &t_inap,
        &archive,
        novelty_evidence_from_checker_result(&CheckerResult::Inapplicable {
            reason: "wrong claim type".into(),
        }),
        cfg,
        &emitter_inap,
        1_700_000_930_002,
    )
    .await;
    assert_eq!(class_inap, NoveltyClass::DiscoveryCandidate);
    let rows_inap = emitter_inap.rows().await;
    assert_eq!(rows_inap.len(), 1);
    assert_eq!(rows_inap[0].class, "discovery_candidate");
    assert_eq!(rows_inap[0].dkg_match_count, 0);

    // Error -> DiscoveryCandidate (conservative zero floor).
    let emitter_err = VecNoveltyClassifiedEmitter::new();
    let t_err = make_template("body-checker-error");
    let class_err = dispatch_novelty(
        &t_err,
        &archive,
        novelty_evidence_from_checker_result(&CheckerResult::Error {
            source: "subprocess timed out".into(),
        }),
        cfg,
        &emitter_err,
        1_700_000_930_003,
    )
    .await;
    assert_eq!(class_err, NoveltyClass::DiscoveryCandidate);
    let rows_err = emitter_err.rows().await;
    assert_eq!(rows_err.len(), 1);
    assert_eq!(rows_err[0].class, "discovery_candidate");
}

#[test]
fn novelty_runtime_env_gate_defaults_to_disabled() {
    // The dispatch_novelty_hook in task_runner.rs is a strict no-op
    // when ELAI_NOVELTY_LABEL_ENABLE is unset. The hook's first
    // branch checks NoveltyRuntimeConfig::is_enabled() and returns
    // early. We assert the env-gate constant token + the default
    // disabled state here; the task_runner-side hook test is the
    // dispatch_sycophancy_probe_hook test's mirror.
    assert_eq!(
        ELAI_NOVELTY_LABEL_ENABLE_ENV,
        "ELAI_NOVELTY_LABEL_ENABLE"
    );
    // The default is disabled (`enable=false`). The runtime gate
    // requires BOTH the toml `enable` flag AND the env var to be
    // truthy before the hook fires.
    let cfg = NoveltyRuntimeConfig::default();
    assert!(!cfg.enable);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_novelty_clamps_out_of_range_evidence() {
    // Caller may slip a non-finite or out-of-range value into the
    // evidence bundle (e.g. an embedding pipeline returning NaN); the
    // dispatcher must clamp via `NoveltyEvidence::new` so the SSB row
    // never carries an out-of-range cosine score.
    let archive = ArchiveChain::new(PathBuf::from("test-novelty-clamp.jsonl"));
    let emitter = VecNoveltyClassifiedEmitter::new();

    let t = make_template("body-clamp");
    let class = dispatch_novelty(
        &t,
        &archive,
        NoveltyEvidence::new(0, f64::NAN, -1.0, 1.5),
        NoveltyConfig::default(),
        &emitter,
        1_700_000_910_000,
    )
    .await;
    assert_eq!(class, NoveltyClass::DiscoveryCandidate);

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    let r = &rows[0];
    assert!(r.near_duplicate_threshold >= 0.0 && r.near_duplicate_threshold <= 1.0);
    assert_eq!(r.near_duplicate_threshold, 0.0);
}
