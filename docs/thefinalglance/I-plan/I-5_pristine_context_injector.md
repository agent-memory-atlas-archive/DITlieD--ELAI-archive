# I-5: pristine context injector — hard-exclude failed siblings and measure the vividness tax before claiming it

- Track 13 — EVIDENCE-RANKED SEARCH | lever TRUST-STEERING | grounding STRONG | nature BUILD | effort M | WAVE 2 | binds ID-1

- Premise: when a branch is expanded, accumulated failed-sibling tracebacks and raw error output occupy context
  and pollute the generator's next step — the attention-sink / vividness-trap hypothesis (ID-1). ELAI's
  verifier stack makes the MEASUREMENT possible: a sampled A/B on the same decision (pristine rebuild vs
  accumulated context) via the existing E-19 probe quantifies the tax. The claim is INSTRUMENTED before
  it is trusted, not asserted from literature alone.

## Steal/build
vividness/attention-sink discipline from Consolidated Part II review 1.4-4: when the orchestrator expands a
branch, context is rebuilt from the parent chain (pristine path). Failed siblings and their raw tracebacks are
HARD-EXCLUDED (ID-1). The negative signal survives ONLY as a compressed contrastive constraint: a <=30-token
structured message ("attempt A failed at line-42 type X — avoid X"). This is not suppressing information —
it is a different encoding that preserves the causal signal without the attention sink. The A/B delta is
measured via the E-19 reasoning-quality probe and ledgered so the claim is recomputable under ELAI_SEED (GD-2).
EXCLUDED: carrying raw tracebacks ("the model should see what failed" is the anti-pattern, not the fix);
truncating context rather than rebuilding it (truncation preserves the sink at a shrunken size; rebuilding
eliminates it); implementing a new masking layer that conflicts with H-6 (I-5 EXTENDS H-6, sharing the
machinery, never duplicating the turn-drop logic). DECLINED: claiming a numeric vividness-tax improvement
before the A/B measurement runs (ID-5 doctrine; the number is a local measurement, not a paper import).

## Existing ground truth
- IS-6: context_budget.rs update_breakpoints :1507 (blake3 prefix stability), adaptive_reduce :1551,
  ContextSlot enum :1079 — the assembly machinery I-5 hooks into for per-branch context rebuild; prefix
  stability must be preserved (HD-2).
- H-6 masking substrate: PLAN-asserted (H-6 is a Wave-2 plan file); executor re-greps per W-7. I-5 extends
  the H-6 turn-masking machinery rather than building a parallel layer.
- H-50 regime policy: PLAN-asserted (H-50 is a Wave-2 context-lifecycle plan); executor re-greps per W-7.
  I-5 shares the regime-aware assembly path.
- E-19 reasoning-quality probe: PLAN-asserted; executor re-greps per W-7. The probe provides the A/B
  measurement surface; I-5 does not implement a new probe, it calls the existing one.
- ContextPruned action_type: net-new (IN-n coverage not specifically ID'd in the map for this row but
  confirmed absent as a named action_type — I-0 SSB table allocates it).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- pristine_path_filter.rs (new, crates/orchestrator/src/): per-branch context rebuilder that excludes
  sibling payloads, compresses tracebacks into the <=30-token constraint format, and logs the byte-delta
- ContextPruned action_type registration (ssb.rs + action_type_lock.rs, baseline+1; Kind:
  SiblingExcluded/TracebackContrasted/VividnessProbe)
- A/B probe wiring: on a sampled fraction of branch expansions, run BOTH paths and emit a VividnessProbe
  row with the outcome delta (recomputable under ELAI_SEED)
- integration into the I-4 per-branch context assembly call site
- decision doc workflow/reasoning/I-5-decisions.md
DEFER (Tier-2 / gated):
- vividness-tax dashboard tile in ReasoningDrawer (frontend_pair surface) — gated behind >=3 measured A/B
  runs showing a consistent delta; tile is wire-dark until GD-4 bound measured
- contrastive-constraint auto-tuning (optimal token budget per task class) — post-measurement

## Integration
New module: crates/orchestrator/src/pristine_path_filter.rs.
Existing seams: context_budget.rs assembly path (the per-branch rebuild hook — PLAN-asserted; executor
re-greps per W-7); H-6 masking turn-drop machinery (extend, never duplicate); E-19 probe call site (the
VividnessProbe A/B sampling leg).
frontend_pair: vividness-tax tile in ReasoningDrawer — wire-dark behind GD-4 bound; W-6 justified for v1
(Tier-2 deferred until measurement stabilises).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/pristine_path_filter.rs::PristinePathFilter::build_branch_context
  - ContextPruned (ssb.rs + action_type_lock.rs — Kind: SiblingExcluded/TracebackContrasted/VividnessProbe)
consumes_symbols:
  - context_budget.rs::ContextSlot / update_breakpoints (why: rebuild must preserve bp stability) — GD-1 IS-6
  - H-6 masking turn-drop machinery (why: extend, not re-implement; share the engine) — GD-1
  - H-50 regime policy assembly path (why: pristine rebuild is regime-aware) — GD-1
  - E-19 reasoning-quality probe (why: A/B measurement surface) — GD-1
reachable_from:
  - I-4 search/ladder.rs::SearchLadder::run_best_of_n -> PristinePathFilter::build_branch_context
    [PRODUCTION search-ladder per-branch expansion]
  - (test callers in crates/contracts/tests/ never satisfy this — R7)
stub_ban:
  - NO raw traceback carry-over: the prior failed branch's full error output in the next branch's context
    is the anti-pattern; the ONLY permitted negative signal is the <=30-token contrastive constraint
  - NO implementing a new masking layer that bypasses or duplicates H-6 turn-drop logic (extend H-6, do not
    fork it)
  - NO claiming a vividness-tax improvement without the A/B measurement row recomputable under ELAI_SEED
    (ID-5; the measurement must precede the claim)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2; NO fabricated measurements
frontend_pair:
  vividness-tax tile in ReasoningDrawer — wire-dark until GD-4 bound; W-6 justified for v1. When the
  dashboard surface ships (H-15 era), cross_lang_diff.py PASS required (W-4). Justified.
observable_effect:
  trigger: "cargo run --bin elai -- run --task fixtures/search/retry_branch.toml --search-ladder
            (requires I-4 v1; run after I-4 ships)"
  expect_ssb: "ContextPruned rows land with Kind=SiblingExcluded for the failed sibling payload and
    Kind=TracebackContrasted for the compressed constraint; on a sampled expansion a VividnessProbe row
    carries a measured outcome delta recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "the retry branch's outbound payload (from the journaled request) contains ZERO bytes
    of the prior 5K-line traceback while the <=30-token constraint phrase is present; the --carry-tracebacks
    broken arm re-inflates the payload to >=5K tokens and FAILS (R2/ID-1)"

## Grounding
- ID-1 (I-0_INDEX): stateless-generator / trajectory-pruning doctrine — failed siblings are HARD-EXCLUDED;
  at most a compressed contrastive constraint carries the negative signal; claim is instrumented first.
- IS-6 (I-GROUNDING-MAP): context_budget.rs ContextSlot :1079 + update_breakpoints :1507 — the seams.
- H-6 and H-50 PLAN-asserted (Wave-2 predecessors); executor re-greps per W-7.
- E-19 probe PLAN-asserted; executor re-greps per W-7.
- ID-5 (I-0_INDEX): local baseline before paper threshold — vividness-tax number is MEASURED on ELAI's
  own corpus before any external literature number is quoted.
- CL-1 (H-0_INDEX): context-lifecycle regime awareness binds I-5 per I-0 doctrine table.
- Review 1.4-4 from Review_ELAI-Competitive: vividness/attention-sink analysis (dossiers); arXiv IDs if
  cited in the decision doc: WebSearch-confirm before headline.
- SSB: +ContextPruned (Kind: SiblingExcluded/TracebackContrasted/VividnessProbe) — baseline+1 from LIVE
  221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
- H-6 masking (shipped): the turn-drop machinery — extend the shared engine, do NOT re-implement.
- H-50 regime policy (Wave 2 plan): context assembly regime-awareness — compose, not fork.
- E-19 reasoning-quality probe (shipped plan): the A/B measurement surface — call it, do not rebuild.
- I-4 search ladder (Wave 4): the caller that triggers per-branch context assembly; I-5 is a Wave-2
  building block that I-4 composes in Wave 4.

End of I-5.
