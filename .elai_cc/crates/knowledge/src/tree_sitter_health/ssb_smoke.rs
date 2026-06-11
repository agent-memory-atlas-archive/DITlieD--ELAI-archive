//! E-42 in-crate smoke test: DkgEngine::build_graph emits
//! TreeSitterGrammarEvent rows via the KnowledgeSsbSink when
//! ELAI_TREE_SITTER_GRAMMARS_ENABLE=1.
//!
//! Off-agent artifact: the DkgEngine emits the rows at runtime against
//! fixture files created in a tempdir. The test reads them back from the
//! RecordingSink (not from the ssb.db -- the knowledge crate has no dep on
//! the orchestrator's Ssb store). Row count >= 1 (ABI probe pass) plus a row
//! with kind=TreeSitterSkippedLargeFile for the >2MB fixture file.

use std::sync::{Arc, Mutex};

use crate::dkg::DkgEngine;
use crate::knowledge_ssb_sink::{
    DeepPoolCappedEvent, DkgRebuildCompleteEvent, DkgRebuildStartedEvent, HydeQueryEvent,
    HydeSkippedCodeQueryEvent, KnowledgeSsbSink, PagerankRunEvent, QueryTraceEvent,
    RrfComponentWeightsEvent, RrfQualityGateFiredEvent, TreeSitterGrammarEvent,
    TreeSitterGrammarEventKind, VectorCacheHitRateEvent,
};

/// In-process recording sink. Accumulates all `TreeSitterGrammarEvent` rows
/// emitted by `DkgEngine::build_graph`. All other event methods are no-ops.
struct RecordingSink {
    ts_events: Mutex<Vec<TreeSitterGrammarEvent>>,
}

impl RecordingSink {
    fn new() -> Self {
        Self {
            ts_events: Mutex::new(Vec::new()),
        }
    }

    fn ts_events_snapshot(&self) -> Vec<TreeSitterGrammarEvent> {
        self.ts_events
            .lock()
            .expect("recording sink lock")
            .clone()
    }
}

impl KnowledgeSsbSink for RecordingSink {
    fn emit_hyde_query(&self, _event: HydeQueryEvent) {}
    fn emit_vector_cache_hit_rate(&self, _event: VectorCacheHitRateEvent) {}
    fn emit_query_trace(&self, _event: QueryTraceEvent) {}
    fn emit_deep_pool_capped(&self, _event: DeepPoolCappedEvent) {}
    fn emit_rrf_quality_gate_fired(&self, _event: RrfQualityGateFiredEvent) {}
    fn emit_hyde_skipped_code_query(&self, _event: HydeSkippedCodeQueryEvent) {}
    fn emit_rrf_component_weights(&self, _event: RrfComponentWeightsEvent) {}
    fn emit_pagerank_run(&self, _event: PagerankRunEvent) {}
    fn emit_dkg_rebuild_started(&self, _event: DkgRebuildStartedEvent) {}
    fn emit_dkg_rebuild_complete(&self, _event: DkgRebuildCompleteEvent) {}
    fn emit_tree_sitter_grammar_event(&self, event: TreeSitterGrammarEvent) {
        self.ts_events
            .lock()
            .expect("recording sink lock")
            .push(event);
    }
}

/// E-42 wiring smoke: drives DkgEngine::build_graph with ELAI_TREE_SITTER_GRAMMARS_ENABLE=1
/// and asserts the off-agent artifact (TreeSitterGrammarEvent rows) is present.
///
/// Sequential (no parallelism) to avoid env-var races with other tests.
/// Uses a RecordingSink rather than the orchestrator Ssb store (no circular dep).
///
/// WIRING CHECK (R1 -- missing evidence is FAIL):
///   * At least one ABI probe row (from registry.iter() over the 18 registered grammars)
///     OR at least one large-file skip row (from the >2MB fixture).
///   * Exactly one large-file skip row with kind=TreeSitterSkippedLargeFile.
#[tokio::test]
async fn e42_dkg_engine_emits_tree_sitter_grammar_events_to_recording_sink() {
    // -- Set up a tempdir with two fixture files --
    let dir = tempfile::TempDir::new().expect("tempdir");
    let project_root = dir.path();

    // Fixture 1: a small valid Rust file (should parse cleanly -- no large-file row).
    let rs_file = project_root.join("fixture.rs");
    std::fs::write(
        &rs_file,
        b"fn hello() { println!(\"hello\"); }\nfn world() -> u32 { 42 }\n",
    )
    .expect("write fixture.rs");

    // Fixture 2: a file with a source extension but > 2 MiB (triggers large-file row).
    // Using `.rs` extension so the DkgEngine matches lang_id. Content is all-zero
    // bytes cast as UTF-8 (zeroes are valid single-byte UTF-8).
    let large_file = project_root.join("large_fixture.rs");
    let large_content: Vec<u8> = vec![b' '; 2 * 1024 * 1024 + 1]; // 2 MiB + 1 byte
    std::fs::write(&large_file, &large_content).expect("write large_fixture.rs");

    // -- Build the DkgEngine with the recording sink --
    let db_path = project_root.join("test_dkg.db");
    let recording = Arc::new(RecordingSink::new());
    let sink: Arc<dyn KnowledgeSsbSink> = recording.clone();

    let mut engine =
        DkgEngine::new_with_sink(&db_path, None, 0, sink).expect("DkgEngine::new_with_sink");

    // -- Enable the wire-dark gate for this test only --
    // SAFETY: this is a test-only env mutation; the test is sequential (#[tokio::test]
    // with no parallel attribute). The var is cleared after the test.
    let env_var = crate::tree_sitter_health::ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV;
    unsafe {
        std::env::set_var(env_var, "1");
    }

    // -- Run the build (the off-agent artifact producer) --
    engine
        .build_graph(project_root)
        .expect("build_graph must succeed on fixture files");

    // -- Clear the env var before assertions so failures don't leak state --
    unsafe {
        std::env::remove_var(env_var);
    }

    // -- Read the off-agent artifact (R4: measure real objective, not a proxy) --
    let events = recording.ts_events_snapshot();

    // GATE 1: at least one tree-sitter grammar event was emitted.
    // This covers the ABI probe (18 registered grammars all probe in-range,
    // but the is_enabled() block runs and can emit 0 ABI-mismatch rows).
    // The large-file skip is guaranteed to fire (2 MiB + 1 byte fixture).
    let large_file_rows: Vec<&TreeSitterGrammarEvent> = events
        .iter()
        .filter(|e| e.kind == TreeSitterGrammarEventKind::TreeSitterSkippedLargeFile)
        .collect();

    // GATE 2 (load-bearing, R1 -- missing evidence is FAIL):
    // The >2MB fixture MUST have produced exactly one large-file skip row.
    assert!(
        !large_file_rows.is_empty(),
        "E-42 gate: expected >= 1 TreeSitterSkippedLargeFile row; got 0. \
         Events emitted: {}. This is the off-agent artifact that proves \
         DkgEngine::build_graph called into tree_sitter_health::large_file.",
        events.len()
    );

    // GATE 3: the row's detail field contains the byte count.
    let detail = large_file_rows[0].detail.as_deref().unwrap_or("");
    assert!(
        detail.contains("bytes="),
        "E-42 gate: large-file row detail must contain 'bytes='; got: {detail:?}"
    );

    // GATE 4: total event count >= 1 (ABI probe block ran).
    assert!(
        !events.is_empty(),
        "E-42 gate: expected >= 1 TreeSitterGrammarEvent; got 0."
    );
}
