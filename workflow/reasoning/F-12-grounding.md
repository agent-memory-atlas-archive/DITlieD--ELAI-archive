GROUNDING SWEEP -- phase F-12
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-12_semantic_contradiction_local_nli.md
Date: 2026-06-03
FSM: PLANNING | Session: s-6312

SPEC REFERENCES

crates/memory/src/store.rs:200-360 | GROUNDED | store.rs:204-207 docstring "Per Zep §2.2.3 + the E-5 plan Tests::bi_temporal_round_trip" confirmed; supersede at :208, soft_close at :303 confirmed; valid_from_ms field on MemoryFact at fact.rs:164 confirmed
crates/memory/src/contradiction.rs | GROUNDED | audit_conflicts DEF at :64; #[cfg(test)] only callers at :221/:256/:285; lib.rs:58 re-export confirmed; zero production callers confirmed across full repo grep
crates/knowledge/src/reranker.rs:130-260 | GROUNDED | with_project_root :193, enable_deterministic_compute :153 (default=true), intra_op_threads=1 :149/:172, models/onnx/ path :165/:195, ReRankerModel :159 -- all confirmed verbatim
crates/orchestrator/src/ssb.rs (MemoryConflictDetected + Payload) | VERIFY_NEEDED | Plan cites MemoryConflictDetected at :5598 (actual: :5622), MemoryConflictDetectedPayload at :16518 (actual: :16735), free-form conflict_kind docstring at :16513-16516 (actual: :16730-16733). Line numbers are stale by ~120-135 lines in each case. The struct content and free-form semantics ARE confirmed at the real lines. Stale line numbers only -- substance correct.
crates/memory/src/fact.rs:130-200 | GROUNDED | MemoryFact struct at :138; fact_id :141; valid_from_ms :164; value :156; key :153; scope :147 -- all confirmed

CLAIM CLASSIFICATION

Graphiti invalid_at = valid_at, non-destructive supersede/soft_close | GROUNDED | store.rs:204-207 "Zep §2.2.3" docstring; soft_close :303 non-destructive UPDATE confirmed; plan intent to pass newer.valid_from_ms as the close timestamp is an honest design-intent read of Graphiti semantics -- not a code claim
cloud-LLM judge breaks ELAI_SEED determinism (arXiv:2506.09501) | VERIFY | arXiv:2506.09501 resolves; title "Understanding and Mitigating Numerical Sources of Nondeterminism in LLM Inference" matches claim exactly; FP associativity / batch-size root cause confirmed in abstract; plan framing is accurate. CAVEAT: plan labels it "NeurIPS 2025" but no NeurIPS affiliation visible on abstract page (submitted 2025-06-11, online 2025-10-24); venue label is unverified
NLI cross-encoder determinism: "practically stable, NOT bit-identical" | GROUNDED | plan explicitly states "NOT provably bit-identical" at line 25; W-10 repeats "practically deterministic / robustly stable, NOT formally seed-invariant"; Open Questions section repeats the honest framing; no over-claim of formal seed-invariance found anywhere in the file
audit_conflicts wire-dark, zero production callers | GROUNDED | grep crates/memory + full .elai_cc grep returns contradiction.rs:221/256/285 (#[cfg(test)] only) + lib.rs:58 (re-export, not a call); live_cli.rs has zero hits for audit_conflicts
cross-encoder/nli-deberta-v3-base HF model exists | GROUNDED | HF page resolves, model card confirmed: DeBERTa-v3, NLI-trained, 3-class output {contradiction/entailment/neutral}, argmax pattern; xsmall/small/base family confirmed
NLI model is Tier-2 / ops-gated, NOT required for Tier-1 | GROUNDED | plan lines 5/83-84: "separately-gated checkbox (W-5/W-10)"; WIRING_CONTRACT deferred_produces_symbols block explicitly flags model file as "Tier-2, model-staging gated; NOT part of W-1 done-gate"; production tick SKIPS gracefully when model absent
NeurIPS 2025 as venue for arXiv:2506.09501 | UNGROUNDED | abstract page has no NeurIPS affiliation; submission date 2025-06-11, online 2025-10-24; no workshop/proceedings link visible; venue label in plan line 249 is unverified -- paper content is correct but venue attribution is unsupported

ACTIONS REQUIRED
- verify arXiv:2506.09501 NeurIPS 2025 venue claim -- WebFetch https://arxiv.org/abs/2506.09501 for proceedings note or WebSearch "Yuan 2025 LLM nondeterminism NeurIPS" to confirm or drop the venue label from the decision doc (F-12-graphiti-contradiction-decisions.md); safe to proceed but venue must not appear as a grounding claim in the decisions doc without confirmation
- stale SSB line numbers in Spec references and consumes_symbols: MemoryConflictDetected at :5598 (real :5622), MemoryConflictDetectedPayload at :16518 (real :16735), conflict_kind docstring at :16513-16516 (real :16730-16733); update the citations in F-12 before TESTING_FIRST (substance is correct, line numbers are stale by ~120-135 lines -- context-compiler will mis-range-read on these)

VALIDATION WARNINGS
SSB line number citations are stale (~120-135 line offset in two locations); substance confirmed correct but context-compiler reads at cited ranges will land on wrong structs. Update before implementation.
arXiv:2506.09501 NeurIPS 2025 venue unverified -- paper content and title are confirmed correct; only the conference label is at risk.
