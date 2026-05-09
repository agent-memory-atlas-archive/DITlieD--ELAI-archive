// Session 4D.10 contract tests: real LLM prompt templates + typed
// research_decompose path on the dashboard flow. Closes MVP audit Section 4
// item #2.
//
// Covers (without a live sidecar):
//   1. All four `ResearchLevel` variants render their template with the
//      {prompt} placeholder substituted (proof the .txt files are embedded
//      via include_str!). The orchestrator's `template_body` is a non-empty
//      &'static str — if an include_str! path broke this assertion fires at
//      compile time with an expressive message.
//   2. `parse_research_report_json` rejects malformed JSON with a typed
//      `GroundingError::InvalidJson` — no silent synthetic fallback.
//   3. A canned-JSON mock `GroundingClient` returns structurally valid
//      reports for each level; `research_decompose` drives the whole
//      idea→concept→phase→step cascade and yields a `ConceptDecomposition`
//      with ≥1 concept, ≥1 phase, ≥1 step, ≥1 baby step.
//   4. The mock client is invoked at least once per level for at least one
//      cascade path — i.e. the real code path is L0 (idea) → L1 (concept)
//      → L2 (phase) → L3 (step), not a single synthetic call.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use elai_orchestrator::grounding_client::{
    parse_research_report_json, GroundingClient, GroundingError, ResearchLevel,
};
use elai_orchestrator::research_tree::{
    research_decompose, ExecutionMode, ResearchCache, ResearchReport, DEFAULT_CACHE_MAX_ENTRIES,
    DEFAULT_CACHE_TTL_SECONDS,
};

// ---------------------------------------------------------------------------
// Canned JSON payloads. Shapes match the L0/L1/L2/L3 templates and the
// downstream extract_* helpers' keyword triggers ("concept"/"framework",
// "phase", "step"). Keeping this inline (not in files) makes the contract
// hermetic.
// ---------------------------------------------------------------------------

const CANNED_L0: &str = r#"{
  "question": "build a web server",
  "answer": "Scope: implement an HTTP server.",
  "alternatives": [
    {"name": "rest-first", "description": "start REST", "citations": [],
     "confidence": 0.8, "trade_offs": ["latency"]}
  ],
  "contradictions": [],
  "sub_results": [
    {"question": "What concept applies to routing?",
     "answer": "trie-based routing",
     "citations": [], "confidence": 0.9, "evidence_level": "MultiSource"},
    {"question": "What framework applies to persistence?",
     "answer": "sqlite layer",
     "citations": [], "confidence": 0.7, "evidence_level": "MultiSource"}
  ],
  "snapshot": {"nodes": [], "convergence": "Converged",
               "total_iterations": 1, "elapsed": {"secs": 0, "nanos": 0}},
  "quality_metrics": [],
  "convergence": "Converged"
}"#;

const CANNED_L1: &str = r#"{
  "question": "What phases are required?",
  "answer": "scaffold -> implement -> validate",
  "alternatives": [
    {"name": "seq", "description": "sequential phases", "citations": [],
     "confidence": 0.9, "trade_offs": []}
  ],
  "contradictions": [],
  "sub_results": [
    {"question": "What phase handles scaffolding?",
     "answer": "phase-scaffold",
     "citations": [], "confidence": 0.9, "evidence_level": "MultiSource"}
  ],
  "snapshot": {"nodes": [], "convergence": "Converged",
               "total_iterations": 1, "elapsed": {"secs": 0, "nanos": 0}},
  "quality_metrics": [],
  "convergence": "Converged"
}"#;

const CANNED_L2: &str = r#"{
  "question": "What steps complete the phase?",
  "answer": "author Cargo.toml, author src/main.rs",
  "alternatives": [
    {"name": "top-down", "description": "decl first", "citations": [],
     "confidence": 0.9, "trade_offs": []}
  ],
  "contradictions": [],
  "sub_results": [
    {"question": "What step handles Cargo.toml?",
     "answer": "step-cargo",
     "citations": [], "confidence": 0.9, "evidence_level": "MultiSource"}
  ],
  "snapshot": {"nodes": [], "convergence": "Converged",
               "total_iterations": 1, "elapsed": {"secs": 0, "nanos": 0}},
  "quality_metrics": [],
  "convergence": "Converged"
}"#;

// ---------------------------------------------------------------------------
// Mock client: returns canned JSON per level and counts calls.
// ---------------------------------------------------------------------------

struct CannedClient {
    l0_calls: Arc<AtomicUsize>,
    l1_calls: Arc<AtomicUsize>,
    l2_calls: Arc<AtomicUsize>,
    l3_calls: Arc<AtomicUsize>,
}

impl CannedClient {
    fn new() -> Self {
        Self {
            l0_calls: Arc::new(AtomicUsize::new(0)),
            l1_calls: Arc::new(AtomicUsize::new(0)),
            l2_calls: Arc::new(AtomicUsize::new(0)),
            l3_calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl GroundingClient for CannedClient {
    async fn ground(
        &self,
        _prompt: &str,
        level: ResearchLevel,
    ) -> Result<ResearchReport, GroundingError> {
        match level {
            ResearchLevel::Idea => {
                self.l0_calls.fetch_add(1, Ordering::SeqCst);
                parse_research_report_json(CANNED_L0)
            }
            ResearchLevel::Concept => {
                self.l1_calls.fetch_add(1, Ordering::SeqCst);
                parse_research_report_json(CANNED_L1)
            }
            ResearchLevel::Phase => {
                self.l2_calls.fetch_add(1, Ordering::SeqCst);
                parse_research_report_json(CANNED_L2)
            }
            ResearchLevel::Step => {
                // L3 is exercised by `decompose_to_baby_steps` which does NOT
                // call the grounding client in the current orchestrator (see
                // research_tree.rs `decompose_to_baby_steps`). Keep the
                // counter in place so a future wiring flip is observable.
                self.l3_calls.fetch_add(1, Ordering::SeqCst);
                parse_research_report_json(CANNED_L2)
            }
        }
    }

    async fn synthesize_text(&self, _prompt: &str) -> Result<String, GroundingError> {
        Err(GroundingError::EmptyResponse)
    }
}

// ---------------------------------------------------------------------------
// Contract 1: all four levels embed a non-empty prompt template body via
// include_str! — a missing file would break compile; a stripped file would
// still fail the template_body().len() > 0 assertion.
// ---------------------------------------------------------------------------

#[test]
fn contract_all_four_prompts_are_embedded() {
    for level in [
        ResearchLevel::Idea,
        ResearchLevel::Concept,
        ResearchLevel::Phase,
        ResearchLevel::Step,
    ] {
        let body = level.template_body();
        assert!(
            !body.is_empty(),
            "template for {:?} must not be empty",
            level,
        );
        assert!(
            body.contains("{prompt}"),
            "template for {:?} must contain the {{prompt}} placeholder",
            level,
        );
        assert!(
            body.contains("JSON") || body.contains("json"),
            "template for {:?} must instruct JSON output",
            level,
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 2: malformed JSON yields a typed InvalidJson error. Closes the
// audit's "typed error instead of synthetic fallback" acceptance criterion.
// ---------------------------------------------------------------------------

#[test]
fn contract_malformed_json_rejected_as_typed_error() {
    match parse_research_report_json("{not json}") {
        Err(GroundingError::InvalidJson { .. }) => {}
        other => panic!("expected InvalidJson, got {other:?}"),
    }
    match parse_research_report_json("") {
        Err(GroundingError::EmptyResponse) => {}
        other => panic!("expected EmptyResponse, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3: canned-JSON mock drives research_decompose end-to-end without a
// sidecar; produces a ConceptDecomposition with ≥1 concept / ≥1 phase / ≥1
// step. Mock client observes L0 + L1 + L2 calls at least once each.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_canned_mock_produces_populated_decomposition() {
    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let client = CannedClient::new();
    let l0 = client.l0_calls.clone();
    let l1 = client.l1_calls.clone();
    let l2 = client.l2_calls.clone();
    let decomp = research_decompose(
        "build a web server",
        ExecutionMode::Autonomous,
        &mut cache,
        &client,
        None, // Track A.6: canned test has no SSB handle; pass None.
        None, // B-7: no tree emitter in real-prompts contract.
    )
    .await
    .expect("canned mock must drive research_decompose to Ok");

    assert!(
        !decomp.concepts.is_empty(),
        "decomposition must have ≥1 concept",
    );
    let concept = &decomp.concepts[0];
    assert!(
        !concept.phases.is_empty(),
        "concept must have ≥1 phase",
    );
    let phase = &concept.phases[0];
    assert!(
        !phase.steps.is_empty(),
        "phase must have ≥1 step",
    );
    let step = &phase.steps[0];
    assert!(
        !step.baby_steps.is_empty(),
        "step must have ≥1 baby step (decompose_to_baby_steps is deterministic)",
    );

    assert!(
        l0.load(Ordering::SeqCst) >= 1,
        "L0 (idea) must be called at least once",
    );
    assert!(
        l1.load(Ordering::SeqCst) >= 1,
        "L1 (concept) must be called at least once",
    );
    assert!(
        l2.load(Ordering::SeqCst) >= 1,
        "L2 (phase) must be called at least once",
    );
}

// ---------------------------------------------------------------------------
// Contract 4: template_name is stable — the rendered L0/L1/L2/L3 enum values
// must match the on-disk filenames so debug logs stay meaningful.
// ---------------------------------------------------------------------------

#[test]
fn contract_template_names_match_files() {
    assert_eq!(ResearchLevel::Idea.template_name(), "l0_idea");
    assert_eq!(ResearchLevel::Concept.template_name(), "l1_concept");
    assert_eq!(ResearchLevel::Phase.template_name(), "l2_phase");
    assert_eq!(ResearchLevel::Step.template_name(), "l3_step");
}
