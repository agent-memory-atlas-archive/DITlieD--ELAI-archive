# ELAI reasoning / thinking architecture pack

This directory is a rebuild kit. Each file is one reasoning/thinking organ extracted from the ELAI spec corpus (Phases 0–4M + letter plans B–Q). Read a file to implement that organ in another harness. This is architecture, not a wiring/status ledger.

Template every organ file follows: `_TEMPLATE.md`.

## How to use this pack

1. Start with **core GRC** (`02A` → `02B` → `02C` → `02D`). That is the Grounded Reasoning Chain.
2. Add **authority + context**: orchestrator-owned budget (`04A`) and the GRC history slot (`03E`).
3. Add **surfaces**: producer wiring, PipelineGrc, `/reason` REFLECT, idea-framer, per-level auditor, FlashResearch (`B-*`, `04D`).
4. Add **self-improvement substrate** (`D-01`–`D-09`) then **inference-time grounding modes A–E** (`D-10`–`D-18`).
5. Add **hardening**: abstention scoring, serial-depth budgets, anti-spiral, quality hooks (`H-*`, `C-10`, `E-*`).
6. Add **search + thinking budgets** (`I-*`).
7. Add **GRC-v3**: BeliefState carrier, effort classifier, overthinking guard, receipt-gated ladder (`J-83`–`J-86`) plus the DD reasoning axis (`J-39`–`J-44`).
8. Add **recursive DEPTH**, receipt doctrine, budget bench (`J-107`/`J-108`, `K-15`/`K-16`).
9. Add **outward claim gates** (`L-01`–`L-09`) then **explore-lane conjecture memory** (`L-22`–`L-24`) and the canonical Claim/Belief object (`N-04`).

Selector hierarchy that later organs share: **external receipt > vote over independent samples (receipt-absent only) > LLM self-eval (banned)**.

## File list (80 organs)

See `_MANIFEST.md` for the worker split. Paths below are relative to this directory.

### Core GRC (Phase 2 + isolation/context)

- `02A-grc-state-machine.md`
- `02B-grounding-verification.md`
- `02C-per-agent-grc-config.md`
- `02D-csq-inline-feasibility.md`
- `04A-orchestrator-authoritative-grc-budget.md`
- `03E-grc-history-context-slot.md`

### Surfaces + research GRC

- `B-05-reasoning-producer-wiring.md`
- `B-08-pipeline-grc-activation.md`
- `B-38-ambiguity-score-gate.md`
- `B-39-chat-reflect-pipeline.md`
- `B-40-idea-framer-grc.md`
- `B-41-per-level-auditor-grc.md`
- `04D-flashresearch-grc-research-agent.md`

### D Wave 1 — reasoning-improvement substrate

- `D-01-reasoning-template-archive.md`
- `D-02-ppm-sidecar.md`
- `D-03-multi-verifier-consensus.md`
- `D-04-tripwire-stack.md`
- `D-05-bai-thompson-promotion.md`
- `D-06-held-out-task-store.md`
- `D-07-cross-domain-canary.md`
- `D-08-crypto-archive.md`
- `D-09-l3-frozen-surfaces.md`

### D Wave 2 — five-mode inference-time grounding

- `D-10-mode-routing-predicate.md`
- `D-11-constructive-checker-registry.md`
- `D-12-operationalization-check.md`
- `D-13-adversary-role-cross-family.md`
- `D-14-sycophancy-probe.md`
- `D-15-novelty-class-labeling.md`
- `D-16-conformal-refusal.md`
- `D-17-honest-refusal-sla.md`
- `D-18-observable-effect.md`

### H / E / C thinking hardening

- `H-39-abstention-aware-scoring.md`
- `H-40-verifier-retention-decorrelation.md`
- `H-41-checker-coverage-instrument.md`
- `H-42-serial-depth-reasoning-budget.md`
- `E-19-reasoning-quality-hooks.md`
- `C-10-anti-spiral-lateral-thinking.md`
- `E-46-planning-decomposition-extras.md`

### I — search, belief, budgets, decoding

- `I-04-evidence-ranked-search-ladder.md`
- `I-05-pristine-context-injector.md`
- `I-07-cascading-escalation.md`
- `I-08-aletheia-voluntary-abort.md`
- `I-11-typed-belief-state.md`
- `I-26-dynamic-thinking-budgets.md`
- `I-27-budgeted-convergence-instrument.md`
- `I-28-dag-credit-assignment.md`
- `I-32-structural-role-scoping.md`
- `I-33-mechanical-decision-governance.md`
- `I-34-crane-grammar-pinning.md`

### J — GRC-v3 + DD reasoning axis

- `J-39-decomposition-node-admission.md`
- `J-40-extractive-research-compression.md`
- `J-41-cheap-grounding-stack.md`
- `J-42-within-step-distill-reset.md`
- `J-44-reasoning-method-artifact-shape.md`
- `J-45-cross-family-advisory.md`
- `J-83-grcv3-beliefstate-carrier.md`
- `J-84-grcv3-effort-classifier.md`
- `J-85-grcv3-overthinking-guard.md`
- `J-86-grcv3-receipt-gated-candidate-ladder.md`

### Recursive reasoning + prove/discipline

- `J-107-longcot-recursive-scaffold.md`
- `J-108-depth-operator.md`
- `J-73-retrieval-abstention-gate.md`
- `K-15-reasoning-budget-bench.md`
- `K-16-reasoning-receipts-doctrine.md`
- `J-106-output-metacommentary-boundary.md`

### L — causal claim gates + deep-think

- `L-01-typed-claim-extraction.md`
- `L-02-claimrisk-convergence-fuse.md`
- `L-03-evidence-roundtrip-predictor.md`
- `L-04-semantic-entropy-predictor.md`
- `L-05-counterfactual-do-test.md`
- `L-06-risk-as-curiosity-router.md`
- `L-07-hallucination-policy-gate.md`
- `L-08-working-memory-hygiene.md`
- `L-09-research-math-deepthink-profile.md`

### L explore + N canonical claim

- `L-22-claimant-falsifiers.md`
- `L-23-explore-profile.md`
- `L-24-conjecture-memory.md`
- `N-04-claim-belief-receipt-memo.md`
- `N-09-counterfactual-ablation-engine.md`
- `N-10-clarity-novelty-budget-wires.md`

## Source corpus (do not treat this pack as replacing these)

- `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`
- `docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md`
- `docs/specs/guide/steps/phase-index-MASTER.txt`
- `docs/thefinalglance/{B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q}-plan/*-SUMMARY_INDEX.md` or `B-PLAN-SUMMARY.md`
- Original phase files under `docs/specs/guide/steps/`
- Original letter plans under `docs/thefinalglance/*-plan/`
