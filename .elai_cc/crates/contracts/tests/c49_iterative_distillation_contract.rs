//! C-49 — Iterative-distillation experience contract.
//!
//! Paper grounding: arXiv:2605.14443 (Sayana, Todi, Jash 2026 —
//! "Prompting Policies for Multi-step Reasoning and Tool-Use in Black-
//! box LLMs with Iterative Distillation of Experience"). Verified via
//! arXiv abstract page on 2026-05-24. The interface map is at
//! `workflow/papers/iterative-distillation-interface.md`.
//!
//! GROUNDING CAVEATS (mandatory per paper-ingestor findings):
//!   - The paper's method is an RL-trained PROMPTER POLICY (GRPO +
//!     KL anchor to a reference policy, Algorithm 1 sec 4.2.3). ELAI's
//!     C-49 ships a HEURISTIC counting-based fragment distiller — the
//!     RL trainer is RUNPOD-PENDING. This contract test fences the
//!     heuristic surface only.
//!   - The paper assumes a BLACK-BOX worker LLM. ELAI's local sidecar
//!     is controllable; the heuristic distiller does not exploit that
//!     fact (so it works for both black-box and controllable workers).
//!   - The paper's headline numbers (Dyck 63 -> 91 with buffer, Web of
//!     Lies 52 -> 90 WITHOUT buffer, Tau-bench Retail 73 -> 91) are
//!     PER-TABLE-ROW. Do NOT cite the abstract aggregate "55 -> 90 on
//!     logic, 74 -> 91 on tool-use" as expected ELAI outcomes.
//!   - The paper's Section 5.3 Web of Lies result explicitly ran
//!     WITHOUT the experience buffer. Buffer-vs-no-buffer is
//!     task-dependent — buffer is NOT what makes the method work in
//!     general. The contract test does NOT assert "buffer helps".
//!   - The paper's Section 5.5 cross-model transfer is poor (in-distrib
//!     Flash +28.0 vs Pro +7.5 vs Flash-lite +5.0). ELAI's per-skill
//!     binning keeps fragments scoped to ONE skill_name; cross-model
//!     sharing is structurally avoided.
//!
//! Contract fences (G1..G6):
//!   G1. [`ExperienceCorpus`] stores per-skill traces with LRU cap
//!       [`MAX_TRACES_PER_SKILL`] (=100; matches C-47 precedent).
//!   G2. [`distill_fragments`] returns fragments meeting `min_freq` +
//!       `min_success_rate` and respects determinism.
//!   G3. [`lookup_distilled_fragments`] returns `Vec<PromptFragment>`
//!       (or empty for unknown skill — greenfield no-panic invariant).
//!   G4. Three SSB action types
//!       (PromptPolicyDistillationRun / PromptFragmentSelected /
//!       PromptFragmentDiscarded) round-trip through the SSB enum
//!       surface.
//!   G5. [`SkillManifest::distilled_fragments`] field exists +
//!       `set_distilled_fragments` write-through accepts a
//!       `Vec<PromptFragment>`.
//!   G6. SCHEMA_VERSION floor for the C-49 cascade.
//!
//! Build target: `cargo check -p elai_cc_contracts`.
//! Test target: `cargo test -p elai_cc_contracts --test
//!   c49_iterative_distillation_contract`.

use std::path::PathBuf;

use elai_orchestrator::experience::{
    distill_fragments, lookup_distilled_fragments, ExperienceCorpus, PromptFragment, SuccessTrace,
    DEFAULT_MIN_FREQ, DEFAULT_MIN_SUCCESS_RATE, MAX_TRACES_PER_SKILL,
};
use elai_orchestrator::skill_registry::{SkillManifest, SkillOrigin};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, PromptFragmentDiscardedPayload, PromptFragmentSelectedPayload,
    PromptPolicyDistillationRunPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// G1 — ExperienceCorpus LRU cap (paper Sec 4.2.2 buffer notion;
// matches the C-47 CASE_BASE_LRU_CAP=100 precedent for the same memory
// budget across per-bin observability surfaces).
// ---------------------------------------------------------------------------

fn frag(text: &str, ts: u64) -> PromptFragment {
    PromptFragment::new(text.into(), 1, 1.0, ts)
}

fn trace(skill: &str, ts: u64, fragments: Vec<PromptFragment>) -> SuccessTrace {
    SuccessTrace::new(skill.into(), 0xAAAA, 0xBBBB, true, ts, fragments)
}

#[test]
fn g1_corpus_evicts_oldest_when_at_cap() {
    let mut c = ExperienceCorpus::new();
    for ts in 0..(MAX_TRACES_PER_SKILL as u64 + 4) {
        c.record_trace(
            "planning_workflow",
            trace("planning_workflow", ts, vec![frag("anchor", ts)]),
        )
        .expect("record");
    }
    let bin = c.traces_for("planning_workflow");
    assert_eq!(bin.len(), MAX_TRACES_PER_SKILL, "LRU cap honoured");
    // Oldest 4 (ts 0..4) evicted; front is now ts=4.
    assert_eq!(bin.front().expect("front").ts_ms, 4);
}

// ---------------------------------------------------------------------------
// G2 — distill_fragments returns fragments meeting min_freq +
// min_success_rate. Single-observation fragments below DEFAULT_MIN_FREQ
// (=2) are filtered.
// ---------------------------------------------------------------------------

#[test]
fn g2_distill_filters_by_min_freq_and_is_deterministic() {
    let mut c = ExperienceCorpus::new();
    // "popular" appears in 3 traces, "rare" in 1.
    for ts in 1..=3 {
        c.record_trace(
            "skill_x",
            trace("skill_x", ts, vec![frag("popular", ts)]),
        )
        .expect("rec");
    }
    c.record_trace(
        "skill_x",
        trace("skill_x", 100, vec![frag("rare", 100)]),
    )
    .expect("rec");

    let a = distill_fragments(&c, "skill_x", DEFAULT_MIN_FREQ, 0.0);
    let b = distill_fragments(&c, "skill_x", DEFAULT_MIN_FREQ, 0.0);
    assert_eq!(a, b, "distillation must be deterministic");
    assert_eq!(a.len(), 1, "only 'popular' meets min_freq=2");
    assert_eq!(a[0].text, "popular");
    assert_eq!(a[0].freq_used, 3);
}

#[test]
fn g2_distill_filters_audit_pass_false_traces() {
    let mut c = ExperienceCorpus::new();
    let neg = SuccessTrace::new(
        "skill_x".into(),
        0,
        0,
        false,
        10,
        vec![frag("never_emitted", 10)],
    );
    c.record_trace("skill_x", neg.clone()).expect("rec");
    c.record_trace("skill_x", neg).expect("rec");
    let out = distill_fragments(&c, "skill_x", 1, 0.0);
    assert!(
        out.is_empty(),
        "negatives are NOT used in distillation \
         (heuristic simplification of paper Eq. 5 buffer-insertion gate)"
    );
}

// ---------------------------------------------------------------------------
// G3 — lookup_distilled_fragments returns Vec<PromptFragment>; greenfield /
// unknown skill returns empty WITHOUT panic (plan stub_ban: greenfield
// must be transparent skip).
// ---------------------------------------------------------------------------

#[test]
fn g3_lookup_returns_vec_promptfragment_type() {
    let c = ExperienceCorpus::new();
    let out: Vec<PromptFragment> = lookup_distilled_fragments("anything", &c);
    assert!(out.is_empty(), "unknown skill returns empty Vec");
}

#[test]
fn g3_lookup_greenfield_no_panic() {
    let c = ExperienceCorpus::new();
    // Multiple unrelated names — must not panic for any of them.
    for skill in ["", "x", "very-long-skill-name-with-dashes", "snake_case_name"] {
        let _ = lookup_distilled_fragments(skill, &c);
    }
}

#[test]
fn g3_lookup_returns_meaningful_for_populated_skill() {
    let mut c = ExperienceCorpus::new();
    for ts in 1..=4 {
        c.record_trace(
            "planning_workflow",
            trace(
                "planning_workflow",
                ts,
                vec![frag("decompose first", ts)],
            ),
        )
        .expect("rec");
    }
    let out = lookup_distilled_fragments("planning_workflow", &c);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].text, "decompose first");
    assert!(out[0].freq_used >= DEFAULT_MIN_FREQ);
    assert!(out[0].success_rate >= DEFAULT_MIN_SUCCESS_RATE);
}

// ---------------------------------------------------------------------------
// G4 — Three SSB action types round-trip through the SSB enum surface.
// Standard 9-tuple roundtrip pattern (matches C-45 / C-46 / C-47 sibling
// contract tests).
// ---------------------------------------------------------------------------

#[test]
fn g4_distillation_run_action_type_round_trips() {
    let s = SsbActionType::PromptPolicyDistillationRun.as_sql_str();
    assert_eq!(s, "prompt_policy_distillation_run");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::PromptPolicyDistillationRun);
}

#[test]
fn g4_fragment_selected_action_type_round_trips() {
    let s = SsbActionType::PromptFragmentSelected.as_sql_str();
    assert_eq!(s, "prompt_fragment_selected");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::PromptFragmentSelected);
}

#[test]
fn g4_fragment_discarded_action_type_round_trips() {
    let s = SsbActionType::PromptFragmentDiscarded.as_sql_str();
    assert_eq!(s, "prompt_fragment_discarded");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::PromptFragmentDiscarded);
}

#[test]
fn g4_payloads_carry_their_action_type_discriminant() {
    let run = SsbPayload::PromptPolicyDistillationRun(PromptPolicyDistillationRunPayload {
        skill_name_hash: 0xAAAA,
        trace_count: 5,
        fragments_emitted: 2,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(run.action_type(), SsbActionType::PromptPolicyDistillationRun);

    let sel = SsbPayload::PromptFragmentSelected(PromptFragmentSelectedPayload {
        skill_name_hash: 0xAAAA,
        fragment_hash: 0xBBBB,
        success_rate: 0.8,
        freq: 3,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(sel.action_type(), SsbActionType::PromptFragmentSelected);

    let dis = SsbPayload::PromptFragmentDiscarded(PromptFragmentDiscardedPayload {
        skill_name_hash: 0xAAAA,
        fragment_hash: 0xCCCC,
        reason_hash: 0xDDDD,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(dis.action_type(), SsbActionType::PromptFragmentDiscarded);
}

#[test]
fn g4_payloads_serde_round_trip_via_rmp() {
    let original = SsbPayload::PromptPolicyDistillationRun(PromptPolicyDistillationRunPayload {
        skill_name_hash: 0x1111_2222_3333_4444,
        trace_count: 7,
        fragments_emitted: 3,
        ts_ms: 1_748_044_900_000,
    });
    let bytes = rmp_serde::to_vec_named(&original).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded.action_type(), original.action_type());
}

// ---------------------------------------------------------------------------
// G5 — SkillManifest carries the distilled_fragments field; the
// write-through setter accepts a Vec<PromptFragment>. The field defaults
// to empty for back-compat with pre-C-49 manifests (serde_default).
// ---------------------------------------------------------------------------

#[test]
fn g5_skill_manifest_carries_distilled_fragments_field() {
    let mut m = SkillManifest {
        name: "planning_workflow".into(),
        version: "0.0.0".into(),
        triggers: vec![],
        inputs: vec![],
        outputs: vec![],
        scope: String::new(),
        origin: SkillOrigin::SkillsDir,
        file_path: PathBuf::from("planning_workflow"),
        learned_credit_score: None,
        distilled_fragments: Vec::new(),
    };
    assert!(m.distilled_fragments.is_empty());
    let fragment = PromptFragment::new("decompose first".into(), 4, 0.9, 1000);
    m.set_distilled_fragments(vec![fragment.clone()]);
    assert_eq!(m.distilled_fragments.len(), 1);
    assert_eq!(m.distilled_fragments[0].text, "decompose first");
}

// ---------------------------------------------------------------------------
// G6 — SCHEMA_VERSION floor for the C-49 cascade. The C-49 cascade lands
// at SCHEMA_VERSION >= 123 (122 was C-47); concurrent sibling plans
// (C-50..C-53) may bump higher. Floor-not-equality so the test does not
// false-fail when a sibling has already landed.
// ---------------------------------------------------------------------------

#[test]
fn g6_schema_version_at_or_above_c49_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 123,
        "SCHEMA_VERSION must be >= 123 after C-49 cascade (got {})",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// G7 — Fragment text-hash is deterministic + collision-resistant for
// realistic fragment text. Used by SSB PromptFragmentSelected /
// PromptFragmentDiscarded payload encoding so raw text stays off the
// wire (matches the C-41 skill_name_hash precedent).
// ---------------------------------------------------------------------------

#[test]
fn g7_fragment_hash_is_stable_and_distinct() {
    let a = PromptFragment::new("decompose first".into(), 0, 0.0, 0);
    let b = PromptFragment::new("decompose first".into(), 99, 1.0, 12345);
    let c = PromptFragment::new("avoid silent swallows".into(), 0, 0.0, 0);
    assert_eq!(a.text_hash(), b.text_hash(), "same text must hash equal");
    assert_ne!(
        a.text_hash(),
        c.text_hash(),
        "distinct text must hash distinctly"
    );
}
