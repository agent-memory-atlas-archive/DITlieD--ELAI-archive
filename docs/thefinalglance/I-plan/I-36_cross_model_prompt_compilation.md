# I-36: cross-model prompt compilation — per-backend compiled variants, transfer gap measured, deliberative hygiene

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever CHEAPER-PORTABLE | grounding STRONG | nature BUILD | effort M | WAVE 4 | binds ID-3 | binds ID-6

- Premise: ELAI routes across 11 backends (B-37) and prompts measurably do NOT transfer between model
  families. The optimizer must run on the cloud lane (landmine #12 — OPRO-style self-optimization fails
  under ~100B). Without compiled prompt artifacts, every backend renegotiates the instruction set
  empirically at runtime. The compiled-prompt store gives the orchestrator a versioned artifact it can
  promote, roll back, and replay — turning prompt work into a config under ID-6.

## Steal/build
Build a compiled-prompt artifact store for per-(role, backend) prompt variants:

OFFLINE compilation: for each (role, backend) pair, a cloud-lane optimizer (MIPROv2-class instruction +
exemplar search over the C-28 fixture suite) produces a compiled variant as a plain-string artifact
(promptolution-style — call sites do not refactor; the store swaps the string). The cloud-lane optimizer
is the ONLY optimizer; no local-lane self-optimization (landmine #12, IC-17 reaffirmed).

TRANSFER GAP MEASUREMENT: every compiled variant is validated on ELAI's own fixtures with the source
prompt on the target backend. The gap (delta fixture pass-rate: source vs compiled) is ledgered as a
TransferGapMeasured SSB row, recomputable from the same seed (GD-2). A variant that regresses its
fixture floor auto-rolls-back (Rolledback row emitted). This is the ID-5 measurement that makes the
gap a known quantity instead of an asserted one.

DELIBERATIVE HYGIENE built into the compiler constraints: no explicit step-by-step CoT scaffolds are
injected under extended thinking (landmine #6: Opus 4.7+ auto-interleaves thinking; explicit scaffolds
are redundant and may suppress the native reasoning). Temperature defaults are respected per backend
(the temperature trap: temp=0 is not determinism — landmine #10).

PROMOTION through ID-6 ledger: a compiled variant promotes only through the same ledger gates as any
config promotion (H-17 persisted N, H-18 deflation, D-6 held-out, D-7 canary). The evaluator lives
outside the optimizer's write boundary (gate-freeze R8 applied to the store's promotion logic).

## Existing ground truth
- IN-8 (grounding map): NO per-backend compiled-prompt artifact store — confirms BUILD
- IS-13 (grounding map): compiled_pipeline.rs TemplateCrystallizer:293 + CompiledPipelineStore:669 (C-25 shipped) — the storage precedent and architectural pattern to mirror (GD-1)
- PLAN-asserted: B-37 PerRoleRuntime (the routing key — 11 backends); executor re-greps per W-7
- PLAN-asserted: E-8 GEPA (the optimizer lane — shared); executor re-greps per W-7
- PLAN-asserted: I-12 playbook curator (the same ledger-gated promotion machinery); executor re-greps per W-7
- Landmine #6 (I-0): Opus 4.7+ rejects manual budget_tokens; no CoT scaffold under extended thinking
- Landmine #12 (I-0): OPRO-style self-optimization fails under ~100B; optimizer is always the cloud lane

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/prompt_artifacts.rs (~200 LOC): CompiledPromptVariant (per role+backend), CompiledPromptStore, promote_variant(), rollback_on_regression()
- MODIFY prompt routing: after role_contracts (I-32) apply, look up CompiledPromptStore for the (role, backend) key and substitute if a promoted variant exists
- Transfer-gap measurement harness: fixture runner comparing source vs compiled on the target backend
- PromptCompiled SSB rows (Compiled / TransferGapMeasured / Rolledback)
- Deliberative-lane hygiene enforced in the compiler output constraints (grep-verifiable contract on the builder: no CoT scaffold present)
- Per-role compiled-variant badge in SettingsWindow (frontend_pair)
- ID-6 promotion path: variant promotion wired to IS-16 ledger gates
- decision doc: workflow/reasoning/I-36-decisions.md

DEFER (Tier-2 / gated):
- Automated per-backend optimizer runs (the offline compilation pipeline) — requires cloud optimizer budget; Tier-2 behind ELAI_PROMPT_COMPILE_DISABLE kill-switch
- Cross-family transfer gap curves (local Qwen vs cloud Sonnet vs Opus) — I-36 measures; trend analysis is post Wave 4 data

## Integration
- new crates/orchestrator/src/prompt_artifacts.rs (~200 LOC)
- MODIFY prompt routing layer: CompiledPromptStore lookup after I-32 role_contracts apply
- seam to IS-13 CompiledPipelineStore (the storage pattern precedent — GD-1)
- seam to B-37 PerRoleRuntime (the routing key — GD-1)
- seam to E-8 GEPA optimizer (the offline compilation lane — GD-1)
- seam to I-12 ledger-gated promotion (the ID-6 binding — GD-1)
- frontend_pair: per-role compiled-variant badge in SettingsWindow — Svelte chip + Tauri read; cross_lang_diff.py PASS (W-4)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/prompt_artifacts.rs::CompiledPromptVariant (the per-role-backend artifact)
  - crates/orchestrator/src/prompt_artifacts.rs::CompiledPromptStore (the version-tracked registry)
  - crates/orchestrator/src/prompt_artifacts.rs::promote_variant() (ID-6 ledger-gated promotion)
  - ssb action_type::PromptCompiled (Kind: Compiled / TransferGapMeasured / Rolledback) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
consumes_symbols:
  - IS-13 compiled_pipeline.rs TemplateCrystallizer:293 + CompiledPipelineStore:669 (storage pattern precedent; GD-1)
  - B-37 PerRoleRuntime (the routing key; GD-1)
  - E-8 GEPA optimizer (the offline compilation lane; GD-1)
  - I-12 ledger gates (ID-6 promotion path; GD-1)
  - I-32 role_contracts (the compiled variant substitutes AFTER role_contracts applies; GD-1)
reachable_from:
  - prompt routing -> CompiledPromptStore::lookup(role, backend) -> PromptCompiled(Compiled) SSB row  [PRODUCTION prompt-routing surface]
  - transfer-gap fixture runner -> TransferGapMeasured SSB row -> ledger  [PRODUCTION measurement surface]
  - regression detection -> rollback_on_regression -> Rolledback SSB row  [PRODUCTION store surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO CoT scaffolds under extended thinking in compiled variants (the compiler output constraint is grep-verifiable; a prompt with explicit step-by-step scaffold under the extended-thinking flag is a contract violation)
  - NO local-lane self-optimization (the optimizer is always the cloud lane — landmine #12)
  - NO promotion without ID-6 ledger gates (a variant that bypasses IS-16 ledger is a P0)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  per-role compiled-variant badge in SettingsWindow — Svelte chip + Tauri read; cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "elai run --role executor --backend claude-sonnet (or equivalent two-backend fixture with a promoted compiled variant) --seed 42"
  expect_ssb: "PromptCompiled rows: Compiled row naming variant hash + backend, TransferGapMeasured row with fixture delta recomputable from same seed (GD-2), Rolledback row if the variant regresses its fixture floor — all under ELAI_SEED"
  expect_artifact: "the compiled variant's extended-thinking requests carry ZERO CoT scaffold (contract grep on the compiled string passes); the per-role badge renders in SettingsWindow (cross_lang_diff.py PASS); a cloud extended-thinking request provably carries no step-by-step scaffold"

## Grounding
- IN-8 (grounding map): compiled-prompt artifact store absent — confirms BUILD
- IS-13 (grounding map): CompiledPipelineStore:669 (C-25) — the storage precedent pattern
- Landmine #6 (I-0): Opus 4.7+ extended thinking; no explicit CoT scaffold
- Landmine #12 (I-0): OPRO-style self-optimization fails under ~100B; optimizer = cloud lane only
- Prompt-ROI dossier (docs/prefableresearch/Research_PromptHeavy-vs-Infrastructure.*): the transfer gap evidence and the optimizer-is-cloud-lane rule
- ID-3 (I-0 doctrine): every middleware transform lands a journaled receipt (PromptCompiled row is the receipt)
- ID-6 (I-0 doctrine): compiled variants are configs; they promote through the same ledger gates
- arXiv: SELF-DISCOVER 2402.03620, MIPROv2/DSPy — WebSearch-confirm before headline
- Compiled-AI arXiv:2604.05150 — WebSearch-confirm before headline
- SSB: +PromptCompiled (Kind: Compiled/TransferGapMeasured/Rolledback) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)

## COMPOSES-WITH
- IS-13 CompiledPipelineStore / TemplateCrystallizer — the storage pattern; do NOT re-implement the crystallizer pipeline
- B-37 PerRoleRuntime — the routing key; do NOT re-implement the runtime backend adapter
- E-8 GEPA — the offline optimizer lane; do NOT re-implement the optimization loop
- I-12 playbook ledger gates — the ID-6 promotion machinery; do NOT re-implement ledger logic
- I-32 role_contracts — compiled substitution applies AFTER role scoping; the two are composable

End of I-36.
