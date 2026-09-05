E-10 — Hostile-critic structural simplifications — DECISIONS

Scope: ship the measurement infrastructure the hostile critic flagged as load-bearing on
2026 evidence (FSM-state ablation, role ablation, 3-layer RAG challenger, MistakeBook
poisoning defense extension, typed-CoT certificate verifier, per-VRAM tier matrix collapse
decision sheet, cloud-default switch test, GRC-blockquote empirical eval, prompt-cache
hit-rate probe, critic decision dashboard panel, elai bench ablation + elai tier eval +
elai critic report CLI), substrate-only, wire-dark unless ELAI_CRITIC_OF_DESIGN_ENABLE=1.

DECISION-1 (SSB single-variant scope-lean — Pattern-2 15th application).
Plan asked for +9 atomic SsbActionType variants. REFUSED per the prior corrective
re-check. Collapse to ONE variant CriticDecisionEvent carrying a 9-arm
CriticPhase discriminator on the payload (FsmAblation, RoleAblation, RagChallenger,
MistakeBookSignal, TypedCotVerified, TierMatrix, CloudDefault, GrcBlockquote,
PromptCacheHitRate). SCHEMA 162 -> 163. TOTAL_VARIANTS 435 -> 436. Precedent:
E-7 DarwinianGenerationCompleted + E-8 PromptOptimizationCompleted.

DECISION-2 (iso_budget_harness::run_ablation wrap, never mutate).
Plan claimed HarnessCfg signature. Actual at iso_budget_harness.rs:132 is positional
(ssb, agent_id, superstep, task_fixture, budget_tokens). E-10 sibling modules WRAP
this; the E-6 signature stays locked.

DECISION-3 (research_pipeline, NOT knowledge_pipeline).
Plan consumes_symbols cites knowledge_pipeline.rs which does not exist. The
actual 7-layer baseline file is research_pipeline.rs (phase_a..phase_g). rag_challenger
runs in PARALLEL — not as a replacement.

DECISION-4 (ort = "=2.0.0-rc.12" pin — document, do not extend).
Pre-existing CLAUDE.md anti-stale violation. E-10 does NOT extend the violation:
rag_challenger.rs uses a deterministic placeholder rerank stub until the ort pin is
bumped or audited for ABI compat. RUNPOD-PENDING marker on the rerank path.

DECISION-5 (Self-MoA — wrapper, NOT PerRoleRuntime extension).
PerRoleRuntime has NO samples_per_role field. E-10 role_ablation introduces a
sibling RoleProfileConfig struct that owns N=3 sampling — PerRoleRuntime stays
stable. Self-MoA grounding: arXiv:2502.00674.

DECISION-6 (cache_check is CONSUMER-ONLY).
conversation.rs:141 already surfaces prompt_cache_events. cache_check.rs is a
READ-side aggregator; no new producer wiring. Expected hit rate is low (Anthropic
compat lacks prompt caching) — that low value IS the diagnostic signal.

DECISION-7 (Paper-cite verification — three POST-CUTOFF IDs).
Corrective re-check flagged arXiv:2603.15611, 2603.28063, 2603.13426. Triage:
2603.15611 is REAL (Code-A1) but NOT in E-10 plan body. 2603.28063 and 2603.13426
do NOT appear here. No Pattern-5 substitution needed. Pre-existing
mistake_book.rs:151 CTIM-Rover mislabel of 2603.15611 (real CTIM-Rover is
arXiv:2505.23422) is a separate incident, out of E-10 scope.

DECISION-8 (ICLAttack -> ProAttack rename).
arXiv:2305.01219 is ProAttack, NOT ICLAttack (real ICLAttack = arXiv:2401.05949).
E-10 module docs use the labels distinctly. Both papers REAL.

DECISION-9 (Typed CoT InferenceRule variants are implementor's extension).
arXiv:2510.01069 defines arithmetic-domain rules. E-10 cot_proof.rs uses general
logical rules (ModusPonens, AndElim, CaseSplit, Refutation, Citation) NOT in the
paper. Module doc labels them as the implementor's generalisation. Certificate
coverage threshold set at 0.4 (paper-observed floor) not 1.0.

DECISION-10 (CLAUDE.md restraint).
Plan asked for +30 LOC. REFUSED per token-economy rule. E-10 adds at most a
one-line pointer to workflow/reasoning/critic-decisions-{ts}.md. <10 LOC ceiling.

DECISION-11 (Dashboard mount in components/, NOT drawers/).
CriticDecisionPanel.svelte is an inline 6-tile panel matching MastFailurePanel.
Mount inside MonitorWindow.svelte.

DECISION-12 (E-9 paired-bootstrap dep — inline helper).
bench_significance.rs (E-9) NOT shipped. E-10 inlines a minimum-viable
paired_bootstrap helper in crates/orchestrator/src/critic_stats.rs (~60 LOC,
deterministic seed, no external statistics crate). When E-9 lands, the helper
becomes a thin re-export shim.

DECISION-13 (MistakeBook delta — additive, not rewrite).
elai_core::mistake_book::MistakeEntry ALREADY enforces structural defense. E-10
adds a sibling StructuredFailureSignal struct in orchestrator/src/mistake_book.rs
with affected_files, structured_diff: DiffStat, mast_mode: Option<MastMode>.
Raw past-attempt code stays absent at the type level.

DECISION-14 (RAG-tier rewrite redundancy).
Plan Tasks 3 and 4 both describe rewrite semantics that are actually additive
against existing structural defenses. rag_challenger runs in parallel;
StructuredFailureSignal lives alongside MistakeEntry. No existing surface is mutated.

DECISION-15 (Dashboard 6 tiles vs 9 fire-points).
CriticDecisionEvent carries CriticPhase (9 phases) but the panel renders 6 tiles.
Mapping: Cloud + Cache fold into the Tier tile; GrcBlockquote becomes a header
badge; MistakeBookSignal becomes a status badge. Tile domains: FSM, Roles, RAG,
Typed CoT, Tier (Cloud+Cache folded), header (MistakeBook + GRC).

Implementation effort: 8 orchestrator modules (fsm_ablation, role_ablation,
rag_challenger, cot_proof, cache_check, grc_blockquote, critic_stats,
StructuredFailureSignal in mistake_book), 3 CLI modules (bench_ablation_cmd,
tier_eval_cmd, critic_report), 1 dashboard panel + 1 Tauri commands file. LIVE
inference loops are RUNPOD-PENDING.
