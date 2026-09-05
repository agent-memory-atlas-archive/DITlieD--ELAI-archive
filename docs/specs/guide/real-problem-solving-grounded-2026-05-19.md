# Real Problem Solving — Grounded Design (2026-05-19)

Scope: how ELAI should actually SOLVE problems (especially novel ones) without hallucinating. Five-mode taxonomy of grounding mechanisms that do NOT require pre-existing comparable problems. Grounded against 9 research agents and 80+ tier-1 arxiv sources across two oracle research passes 2026-05-17 + 2026-05-19.

Companion file: `reasoning-improvement-grounded-2026-05-19.md` covers the orthogonal problem of how the reasoning loop itself improves over time via DGM-style self-modification. Read together.

Three-section convention:
- **GROUNDED**: claim cites tier-1 source with measurement or theorem
- **ARCHITECTURAL SYNTHESIS**: derivation from grounded claims into ELAI-specific design
- **UNGROUNDED CONJECTURE**: training-only or speculative, isolated section for user challenge

Tier legend: T1 = peer-reviewed (Nature, ICLR, NeurIPS, TMLR) or arxiv with verified org authorship. T2 = recognized expert blog or org eng post. T3 = personal blog, substack, wiki.

---

## 0. THE PROBLEM (honestly)

LLMs are probabilistic next-token predictors. The 2025-2026 literature documents three distinct failure modes that combine to make raw LLM output unreliable for novel problems:

- **Factual confabulation**: model emits plausible-sounding but false claims when no grounding is available
- **Sycophancy**: model agrees with the user's framing even when the framing is wrong (Sharma et al, arxiv:2310.13548, T1)
- **Self-correction blind spot** (arxiv:2507.02778, T1): 64.5% blind-spot rate across 14 open-source models. They correct external errors but not their own.

Separately documented constraint: small models (sub-10B) cannot reliably self-verify. First Try Matters (arxiv:2510.08308, T1) — reflective steps are 90%+ confirmatory and net-degrade accuracy when they do overturn. "Small Language Models Need Strong Verifiers to Self-Correct Reasoning" (arxiv:2404.17140, T1) — a weak self-verifier on a ≤13B reasoner brings minor improvements at best.

**The user's pushback is correct**: "reduce-to-known" cannot be the only answer. Reducing every novel problem to a known one makes the system a glorified lookup. Novelty must be groundable on its own terms — but that requires a grounding mechanism that doesn't depend on prior art.

This file maps the five mechanisms the literature offers, with explicit honest treatment of where each fails.

---

## 1. THE FIVE-MODE TAXONOMY (each grounded against T1 sources)

### MODE A — Constructive Proof (the answer can be built and run)

**Principle**: when a claim is OPERATIONALIZABLE — expressible as a program, proof, function, or measurable construction — the EXTERNAL CHECKER is the ground truth, not the LLM's confidence. The construction IS the grounding.

**Grounded by**:
- Program-of-Thought (PoT, arxiv:2211.12588, T1, TMLR 2023): +12% over CoT on numerical benchmarks. LLM emits Python; interpreter evaluates. The interpreter's verdict is ground truth. "Compute the answer, don't generate it."
- CodeAct (arxiv:2402.01030, T1, ICML 2024): Python as the agent's unified action space; error messages close the loop. +20% success vs JSON-formatted actions.
- AlphaProof (Nature 2025, doi:10.1038/s41586-025-09833-y, T1): autoformalizer translates natural-language problem into Lean 4; AlphaZero-style RL inside Lean. IMO 2024 silver medal (3/6 problems). **Bypasses hallucination because the verifier is a formal proof checker, not another LLM.**
- AlphaGeometry-2 (arxiv:2502.03544, T1): symbolic deduction engine grounds neural proposer. 84% of IMO 2000-2024 geometry problems.
- FunSearch (Nature 2023, doi:10.1038/s41586-023-06924-6, T1): LLM mutates programs; human-written scorer rates them; evolutionary loop retains best. Discovered new cap-set lower bounds, novel bin-packing heuristics.
- AlphaEvolve (arxiv:2506.13131, T1, DeepMind 2025): same evaluator-bound evolutionary pattern. Found 48-multiplication algorithm for 4x4 complex matrix mult — **first improvement over Strassen in 56 years**. Deployed at Google scale.

**Applicability**: anything expressible in formal language with decidable checker. Proofs, programs, algorithms with measurable performance, optimization problems with computable objectives.

**Does NOT ground**: claims about meaning, intent, external-world facts the checker cannot evaluate, philosophical/ethical claims, subjective preference, counterfactual histories.

**Critical observation** from the research: even "discovery" wins like AlphaEvolve and FunSearch are large-scale SEARCH through program spaces evaluated by AUTOMATED CHECKERS. The LLM is the mutation operator. The checker is the grounding. The grounding does NOT require prior art — it requires only a decidable checker.

### MODE B — Falsifiability Inversion (usually fails as primary grounding; useful as auxiliary)

**Principle (Popper-style)**: ground a claim by trying to disprove it. Devil's advocate, adversarial probing, debate.

**Grounded LIMITATIONS** (the literature is much harsher than surface intuition):

- CounterBench (arxiv:2502.11008, T1): frontier LLMs at "random-guess level" on rule-based counterfactual reasoning. The task adversarial probing requires the model to do well at — they cannot.
- Self-Correction Bench (arxiv:2507.02778, T1): **64.5% blind-spot rate** across 14 open-source models on detecting own errors. They correct external errors but not their own. A simple "Wait" prompt reduces this 89.3% — interesting but interim auxiliary signal only.
- Constitutional AI on small models (arxiv:2504.04918, arxiv:2503.17365, both T1): architecture-dependent reliability. Llama-3-8B self-critique works for harm reduction. DeepSeek-R1-8B, Gemma-2-9B, Qwen2.5-7B show "less improvement in harm detection." Recursive self-critique training can cause MODEL COLLAPSE on small models — quality too low to bootstrap.
- AI Safety via Debate (arxiv:1805.00899, T1): theoretical PSPACE result, only MNIST empirical demo. No convincing LLM-scale empirical demonstration in the public literature.
- "Small Language Models Need Strong Verifiers" (arxiv:2404.17140, T1): a weak self-verifier on a ≤13B reasoner brings minor improvements at best and frequently misguides the refiner.

**Verdict**: adversarial probing is a USEFUL AUXILIARY signal but CANNOT be the primary grounding for a novel claim, especially on a 7B local model. Architecture-dependent catch rate; produces confabulation roughly as often as it catches false claims.

**Salvageable**: heterogeneous-verifier adversarial probing (cross-MODEL-FAMILY Adversary, NOT same-model self-critique) per Lightweight Latent Verifiers (arxiv:2504.16760, T1) — verifier transfer fails between base LLMs, so a different-family probe IS informative.

### MODE C — Discovery vs Search (the boundary of current LLM capability)

This mode is included because the user explicitly pushed back: "novelty and creativity are necessary." The honest answer is that the literature has a real contradiction here.

**The grounded contradiction**:

- FunSearch defenders (Nature 2023, T1): genuine discovery — surpassed established SOTA on open mathematical problems; the result didn't exist to retrieve from training data.
- FunSearch critics (Ernest Davis NYU note, T2; Gary Marcus Substack, T3; Mouret Nature commentary hedged "proof of concept", T1): humans wrote problem-specific scoring code; LLM is narrow mutation operator inside an evolutionary loop. The discovery framing is overstated.

**Chollet's ARC framing** (arxiv:1911.01547, T1): ARC built on explicit set of priors close to innate human priors to resist BOTH memorization AND search. 

**ARC-AGI-2 SOTA 2025**: top score 24.03% (NVARC) vs 60-70% for humans (arxiv:2601.10904, T1). LLMs trail dramatically on a benchmark specifically designed to test genuine novelty. Tiny 7M-parameter recursive models reach 45% on ARC-AGI-1 — **scale is not the bottleneck**.

**The honest engineering answer**: at the current frontier, "discovery" by LLM-driven systems is large-scale SEARCH through program spaces no human enumerated, evaluated by automated CHECKERS. That is not nothing — Strassen-beating matrix multiplication, new cap-set bounds — but it is also not what a mathematician means by Erdős "from the book" discovery.

**Both readings are defensible.** The actionable position:
1. Don't market autonomous LLM "discovery" — too contested
2. DO use the FunSearch / AlphaEvolve pattern (LLM-mutates-evaluator-scores) for novel-output problems where a scorer is writable
3. Mark novel claims with a CONFIDENCE that distinguishes "verified by checker" from "search produced this, no checker available"
4. Recognize that ARC-AGI-2 24% vs 60-70% human gap means there's no current 7B path to closing genuine open-ended novelty. Be honest about this.

### MODE D — Operational Falsification (build a test, run it, observe outcome)

**Principle**: when no prior test exists and the claim is empirical, design an experiment, execute it, let the world's response be the ground truth. Science applied to LLM-scale.

**Grounded by**:
- Voyager (arxiv:2305.16291, T1): learns Minecraft skills as executable code, retains only those that produce measurable environment effects. 3.3x more unique items, 15.3x faster tech-tree milestones than prior SOTA. **Minecraft simulator is the verifier.**
- Eureka (arxiv:2310.12931, T1, ICLR 2024): designs reward functions, tests them by training RL agents. Reward design wins on 83% of 29 RL benchmark tasks. **The RL training run IS the falsifier** — a bad reward produces a bad policy, observably.
- Coscientist (arxiv:2304.05332, T1): runs physical chemistry experiments. Catalyzed cross-coupling reactions succeed or fail at the bench. Authors document safety failure modes — the most important grounding signal.
- NVARC ARC-AGI-2 4B at $0.20/task (NVIDIA 2025, T1): test-time training + synthetic data + refinement loops — operational falsification compressed into inference budget.

**Failure modes** (well-documented):
- Goodhart on test design itself (Eureka can over-optimize the reward proxy)
- Simulator-reality gap (Voyager works in Minecraft, not in the field)
- Confounded experiments (Coscientist safety paper explicitly raises this)

**Maps directly to ELAI's existing Phase 4F Feasibility Probe** — <50 LOC stub in Tier C sandbox with observable outcomes. The upgrade needed: typed `ObservableEffect` predicate per the spec in section 3 below.

### MODE E — The Ungroundable Boundary (honest refusal as quality metric)

**Principle**: when none of Modes A-D apply (no constructive checker, adversarial probing unreliable, not a search target with evaluator, no executable experiment) — **REFUSE rather than hallucinate**. Honest "i don't know" is a first-class output.

**Grounded by**:
- UniCR (arxiv:2509.01455, T1): unified framework for confidence calibration and risk-controlled refusal via **conformal risk control**. Distribution-free FORMAL guarantee that refusal rate stays within user-specified error budget regardless of distribution shift. **This is the strongest mathematical guarantee available for "i don't know."**
- Examining LLMs' Uncertainty Expression Toward Questions Outside Parametric Knowledge (arxiv:2311.09731, T1): calibration must be PAIRED with TRAINED textual expression of uncertainty.
- Rewarding Doubt (arxiv:2503.02623, T1): RL can produce calibrated confidence (expressed confidence ≈ empirical accuracy).
- LLM Honesty Survey (TMLR 2025): catalogs the field.

**The boundary rule**: when no constructive checker exists (A), adversarial probing is unreliable on the model class (B), the claim isn't a search target with an evaluator (C), and no executable experiment is available (D) — refuse. UniCR-style conformal calibration is the only mode that can give a FORMAL refusal guarantee.

**Treat "honest i-don't-know rate" as a first-class quality metric next to accuracy.**

---

## 2. SURFACED CONTRADICTIONS

Two real contradictions in the literature that the user should know about before adopting either side:

**C1 — Is FunSearch real discovery, or sophisticated search?**

Both readings cite the same artifacts. Nature framing + DeepMind blog (T1/T2) → "discovery" because the result didnt exist to retrieve. Davis NYU note (T2) + Marcus Substack (T3) + Mouret hedged commentary (T1) → "human-scaffolded search with an LLM as mutation operator." Engineering-actionable resolution: treat the pattern as a powerful grounding tool for novel-output problems where a human can write a scoring function; do NOT market it as autonomous AI discovery.

**C2 — Does small-model self-critique work?**

Llama-3 results show it works for HARM REDUCTION (Constitutional AI). Self-Correction Bench shows it FAILS for factual correctness at 64.5% blind-spot rate across 14 models. Both true because they measure different things. Engineering-actionable resolution: never use a 7B model's own critique as the sole grounding for a novel FACTUAL claim from that same model. Cross-family Adversary is the literature defense.

---

## 3. ELAI INTEGRATION

ELAI already has primitives mapping to each of the five modes. The integration task is to (a) make them explicit, (b) wire the routing, (c) require the system to PICK a mode before answering.

### 3.1 Existing primitives → mode mapping

| Mode | Existing ELAI primitive | Strength |
|------|------------------------|----------|
| A Constructive | T1/T2/T3 deterministic hooks; xcodebuild test runner; FILE_CHECK/SOURCE_CHECK/TEST_CHECK predicates; sidecar /v1/messages → PoT-compatible | Strong |
| B Falsifiability | Auditor agent + REFLECT pipeline + CSQ counterfactual safety (Phase 2D) | Medium — known limits per § 1 Mode B |
| C Discovery vs search | Idea-intake decomposition tree + concept-decomposer + plan-auditor mode=concepts | Partial — no explicit discovery-vs-search labeling |
| D Operational | Feasibility probe Phase 4F: <50 LOC stub in Tier C sandbox, observe outcomes | Strong primitive; needs upgrade per § 3.4 |
| E Ungroundable | Oracle Mode "I don't know" first-class + GroundingMethod::Unsupported verdict | Strong primitive; missing UniCR conformal calibration |

### 3.2 Required routing predicate (orchestrator decides, not the LLM)

Before any reasoning pass that emits a CLAIM about NOVEL behavior (defined: no GROUNDED FILE_CHECK / SOURCE_CHECK / DKG-blast-radius hit, AND no exact-match prior in MistakeBook), the orchestrator MUST select one of A/B/C/D/E. Selection logic:

```rust
fn select_grounding_mode(claim: &Claim, ctx: &OrchContext) -> GroundingMode {
    // Mode A — constructive checker available
    if claim.is_operationalizable() && ctx.checker_registry.applicable(claim) {
        return GroundingMode::Constructive;
    }
    // Mode D — sandbox experiment possible
    if claim.permits_experiment() && ctx.sandbox.can_run(claim) {
        return GroundingMode::OperationalFalsification;
    }
    // Mode E — refuse if intrinsic signals say uncertain
    if claim.intrinsic_signals_below_threshold(ctx.unicr_tau) {
        return GroundingMode::Refuse;
    }
    // Mode B — falsifiability as last RESORT, with cross-family Adversary required
    if ctx.has_alternate_family_model() {
        return GroundingMode::FalsifiabilityCrossFamily;
    }
    // Mode E — refuse if no cross-family verifier
    return GroundingMode::Refuse;
}
```

The orchestrator (not the LLM) decides. The LLM has NO input to mode selection. This prevents the optimizer-decides-its-own-grade failure mode per Everitt current-RF principle (arxiv:1908.04734, T1).

Mode C (discovery-vs-search) is not a runtime grounding mode — it is a CLASSIFICATION applied AFTER A or D produced a checker-grounded result, to LABEL the result for human review.

---

## 4. NEW MODULES PROPOSED

### 4.1 Operationalization-required predicate (Mode A enforcement)

Every novel claim must produce a TEST that would fail if the claim is wrong, IN THE SAME BABY STEP as the claim itself.

Extend baby-step block 7 (DETERMINISTIC_CHECK) with `OperationalizationCheck`:
- If baby step deliverable is a CLAIM (e.g. "this algorithm has complexity O(n log n)"), require a co-emitted `MEASURABLE_PROCEDURE` that operationalizes the claim (timing harness, asymptotic-empirical curve fit, comparison against ground-truth instances, etc).
- Without the procedure, the baby step CANNOT enter MEMORIZING. Auditor BLOCKs at acceptance gate.

This is the unfalsifiability filter. Adapted from Voyager (arxiv:2305.16291, T1) skill-with-measurable-effect pattern.

**Grounding source**: Voyager — every retained skill produces a measurable environment effect. ELAI's "every retained claim produces a measurable procedure" is the same structural rule.

### 4.2 Constructive-checker registry (Mode A scaling)

Add `.elai_cc/checkers/` directory containing per-domain executable checkers:

```
.elai_cc/checkers/
  math/
    sympy.py          # numeric + symbolic verification
    z3.py             # SAT/SMT for finite domains
    lean.py           # Lean 4 kernel for formal proofs (where feasible)
  code/
    compile_rust.sh
    compile_py.sh
    compile_ts.sh
    test_rust.sh
    test_py.sh
    test_ts.sh
  text/
    regex_assertion.py    # rule-based text-property check
    schema_validate.py    # JSON schema, etc
  graph/
    networkx.py           # graph-theoretic properties
  api/
    registry_check.py     # npm, crates.io, pypi existence
    # extensible
  perf/
    timing_harness.py
    asymptotic_curve_fit.py
```

Each checker has typed signature:
```rust
fn check(claim: Claim, candidate: Candidate, sandbox: SandboxHandle) -> CheckerResult {
    Pass,
    Fail { evidence: Vec<String> },
    Inapplicable { reason: String },
}
```

Orchestrator dispatches appropriate checker based on claim type. Mode A is only AVAILABLE when an applicable checker exists. If no checker applies, system falls back to Mode D (operational) or Mode E (refuse).

**Grounded by**: PoT (arxiv:2211.12588, T1), CodeAct (arxiv:2402.01030, T1), AlphaProof Nature 2025 (T1) — all three demonstrate the pattern. AlphaProof specifically uses Lean as the math checker; ELAI's `checkers/math/lean.py` slot exists for the same purpose where feasible.

### 4.3 Adversary role (Mode B with hard constraints)

Add `AgentRole::Adversary` to ELAI role taxonomy. When FSM enters AUDITING for a CLAIM (not code), spawn Adversary with INVERTED framing: "produce the strongest evidence that this claim is FALSE."

**Hard constraints** (per Mode B research):
1. Adversary spawned on a DIFFERENT model family from the executor. Heterogeneous verifier per arxiv:2504.16760 (T1) — verifier transfer fails between base LLMs.
2. Adversary output STRUCTURED (counter-evidence with source-tier citation), not narrative. Per CTIM-Rover (REALM 2025, T1) — narrative degrades small-model attention; structured doesnt.
3. Auditor inspects (counter-evidence, original claim) and decides; NEVER the executor itself.
4. If no alternate-family model available, Mode B is SKIPPED — system falls to Mode E. NO same-model adversarial probing per arxiv:2404.17140 (T1) findings.

**Grounded by**: Self-Correction Bench (arxiv:2507.02778, T1) 64.5% blind-spot rate confirms same-model self-critique fails; "Wait" prompt 89.3% reduction confirms cross-family critique IS auxiliary-useful. Constitutional AI small-model collapse (arxiv:2504.04918, T1) confirms same-model recursive critique is structurally unsafe.

### 4.4 Experiment-emits-observable-effect predicate (Mode D upgrade)

Extend existing Phase 4F Feasibility Probe to require every probe emit a typed `ObservableEffect`:

```rust
struct ObservableEffect {
    trigger: TriggerSpec,                  // how to invoke probe
    expected_observation: TypedObservation, // what counts as the answer
    success_predicate: Predicate,          // when does experiment succeed
    bounded_runtime: Duration,             // max execution time
    sandbox_tier: SandboxTier,             // A | B | C per existing classification
    confound_flags: Vec<ConfoundFlag>,     // documented confound risks
}
```

Failure modes acknowledged:
- **Goodhart on test design**: require ≥2 ObservableEffects per claim, each measuring an orthogonal aspect. Single-effect probes get a Goodhart-risk flag.
- **Simulator-reality gap**: distinguish `SandboxObservation` from `ProductionObservation`. NEVER claim production grounding from sandbox alone. Mark all sandbox-grounded claims as "Sandbox-Verified, Production-Pending."
- **Confounded experiment**: log all environmental variables; the `confound_flags` field surfaces known risks (e.g. "performance measured on cold cache, may differ on warm").

**Grounded by**: Voyager (arxiv:2305.16291, T1), Eureka (arxiv:2310.12931, T1), Coscientist (arxiv:2304.05332, T1, especially Boiko et al safety paper documenting confounds).

### 4.5 Conformal refusal calibration (Mode E formal guarantee)

Implement UniCR-style conformal risk control on ELAI's i-don't-know outputs (arxiv:2509.01455, T1).

Mechanism:
- Maintain a calibration set of (claim, ground_truth_available, model_confidence) tuples from prior sessions. Storage: `.elai_cc/calibration/`.
- Compute a refusal threshold τ such that empirical risk (refusing-when-truth-exists AND not-refusing-when-no-truth) stays within user-specified ε. ε default 0.1.
- τ is updated nightly from new calibration data (L3 — system cannot self-modify this calibration loop).
- System REFUSES any claim whose model confidence (mean_logprob, entropy, novelty composite per GG signals) falls below τ.

This is the ONLY Mode E grounding with a FORMAL distribution-free guarantee per the literature.

**Grounded by**: UniCR (arxiv:2509.01455, T1) conformal risk control. Distribution-free guarantee regardless of distribution shift. The strongest math available for "i don't know."

### 4.6 Honest-refusal SLA

Add session-level metric: `refusal_rate = refusals / total_claims`.

Healthy band (UNGROUNDED on specific numbers; the literature does not publish ELAI-specific targets): 10-30% on hard task suites for calibrated reasoners. Outside band:
- < 5% refusal: model is confabulating. System auto-flags for L3 review.
- > 50% refusal: checkers are misconfigured OR task suite is too hard for current model. System auto-flags.

This is structural defense against the "Mode E is the easy way out" failure where a self-improving system learns to route everything to refuse.

### 4.7 Sycophancy probe (orthogonal to Mode B)

The user explicitly raised sycophancy. Add a structural defense (NOT a prompt-based one — prompts don't reliably fix sycophancy per arxiv:2310.13548, T1):

For every claim the model emits in agreement with the user's framing, RE-RUN with the framing INVERTED ("the user just said X. assume X is wrong. justify the alternative"). Compare the two outputs:
- If outputs converge → agreement is robust, not sycophantic
- If outputs diverge significantly → SYCOPHANCY FLAG, claim quarantined for human review

Sycophancy is a measurable bias (Anthropic Sharma et al, arxiv:2310.13548, T1). This catches it STRUCTURALLY.

**Grounded by**: Sharma et al sycophancy work demonstrates the bias is reproducible and probe-able with framing inversion. ELAI's role taxonomy can carry this as a `SycophancyProbe` operation invoked by the Auditor on every user-aligned claim.

### 4.8 Discovery vs search labeling (Mode C as output annotation, not runtime mode)

Every checker-grounded result from Mode A or Mode D gets labeled with a discovery-vs-search confidence:

```rust
enum NoveltyClass {
    Search,                  // result obtained by enumerating known primitives
    DistantSearch,           // result required novel combination of known primitives
    DiscoveryCandidate,      // result not obviously reducible to known primitives;
                             // requires human review to ratify or reject
}
```

Heuristic for labeling (UNGROUNDED specifically, but each component is grounded individually):
- Search: result has near-duplicate in DKG / MistakeBook / prior session archive
- DistantSearch: result is novel COMPOSITION of known primitives, each in DKG
- DiscoveryCandidate: result references primitives or relationships NOT in DKG; flag for human review

This is the honesty layer. The system never claims discovery autonomously; it surfaces candidates for human ratification, per the FunSearch defenders-vs-critics debate (§ 2 C1 contradiction). Removes the marketing risk while preserving the engineering capability.

---

## 5. WHAT THIS DOES NOT SOLVE (honest scope)

Honest list of open problems that this design does NOT address:

- **Genuine open-ended discovery** (Mode C contradiction): the literature has no published solution at sub-10B scale. Tiny 7M-parameter recursive models reach 45% on ARC-AGI-1 but ceiling 24% on ARC-AGI-2 vs 60-70% human. Scale is not the bottleneck. ARC-AGI-2 results suggest a different inductive bias is needed; nobody has it yet.
- **Hidden-pattern reasoning where no constructive checker exists AND no operational experiment is feasible**: philosophical, ethical, aesthetic claims. Mode E refusal is the answer, but it's not "solving the problem."
- **Goodhart at the meta level on the grounding modes themselves**: a sufficiently clever optimizer may learn to classify hard claims as Mode E (refuse) to avoid being graded. Partial defense via the honest-refusal SLA bounds (§ 4.6); not a solution.
- **Verifier-of-verifier regress for Mode A checkers**: a faulty Lean axiom set, a buggy compiler, an over-permissive SymPy default. Partial defense: redundant checkers per domain, periodic checker audits, content-hashed checker registry (so a buggy checker version is traceable to specific outputs). The regress is real.
- **The sub-10B verifier transfer problem**: even cross-family Adversary may share failure modes if both models trained on similar data. Heterogeneous verifier is the strongest practical defense but it's not proof.

These are open problems for the field, not bugs in ELAI. The user should know what isn't promised.

---

## 6. ROLLOUT PLAN

### Phase RPS.I — Mode routing predicate (1-2 sessions, low risk)
- Implement `select_grounding_mode` orchestrator function
- Wire to all CLAIM-emitting baby steps in EXECUTING
- No new modules; just routes existing primitives through the typed mode classifier
- Validate: synthetic claim with no applicable checker should route to Mode E

### Phase RPS.II — Constructive-checker registry (3-4 sessions, medium risk)
- Bootstrap `.elai_cc/checkers/` with 4-6 initial checkers (sympy, z3, compile-rust, compile-py, registry-check, regex)
- Define `CheckerResult` typed return
- Dispatch logic in orchestrator
- Mode A becomes operational
- Validate: math claim → SymPy passes/fails; code claim → compile passes/fails; novel API claim → registry-check passes/fails

### Phase RPS.III — Operationalization-required predicate (1-2 sessions, low risk)
- Extend `BabyStepSpec` block 7 with `MEASURABLE_PROCEDURE` field
- Audit rule: claim-emitting baby steps without procedure BLOCKED at AUDITING gate
- Update babystep-author agent to emit procedures alongside claims
- Validate: contract test where babystep-author tries to emit claim-without-procedure must fail audit

### Phase RPS.IV — Adversary role (2-3 sessions, medium risk)
- Add `AgentRole::Adversary` with mandatory cross-model invocation
- Wire to AUDITING for CLAIM baby steps
- Heterogeneous-verifier constraint: skip if no alternative model available (do NOT fall back to same-model)
- Validate: synthetic claim that the executor confidently asserts should be flagged when Adversary disagrees

### Phase RPS.V — Sycophancy probe (1 session, low risk)
- Inverted-framing re-run on agreement-with-user claims
- Divergence threshold tunable per arxiv:2310.13548 measurements
- Auto-flag for human review on threshold breach
- Validate: synthetic sycophancy task (user asserts false thing, model agrees) must trigger probe

### Phase RPS.VI — Discovery-vs-search labeling (1-2 sessions, medium risk)
- Implement `NoveltyClass` heuristic
- Wire to result-emission from Mode A and Mode D
- DiscoveryCandidate output routes to human-review queue (L3)
- Validate: known-search task labels Search; known-novel task labels DiscoveryCandidate

### Phase RPS.VII — Conformal refusal calibration (3-4 sessions, OPTIONAL on calibration data availability)
- UniCR-style calibration set persistence
- τ computation nightly (L3 — system cannot self-modify)
- Hook to GG signals composite confidence
- Defer until N=500 calibration tuples available from prior sessions
- Validate: distribution-shift test (held-out task family from different domain) should preserve empirical refusal rate within ε

### Phase RPS.VIII — Honest-refusal SLA monitoring (1 session, low risk)
- Add `refusal_rate` session metric
- Auto-flag outside healthy band (initial guess 10-30%; tune per session data)
- Validate: instrument the metric across last 20 sessions; healthy band emerges empirically

Total estimated effort: 12-19 sessions across 8 sub-phases. Each independently shippable.

Priority sequencing: I, II, III (Mode A operational) first; IV, V (Adversary + sycophancy) next; VI (novelty labeling) parallel; VII deferred on calibration data; VIII anytime.

---

## 7. OPEN QUESTIONS

**Q1 — Checker auto-install vs refuse**: when a claim COULD be operationalized cheaply but the required tool isn't installed on the local box, should ELAI auto-install vs refuse vs partial-grounding? UNGROUNDED — no published source. Recommend: refuse with explicit reason "checker missing: install X to enable Mode A grounding for this claim class."

**Q2 — Adversary cloud fallback**: when no alternative-family local model is available, is cloud fallback (e.g. Sonnet as Adversary) acceptable? Per Cohen Boxed Myopic principle (arxiv:1908.04734, T1) — cloud Adversary is acceptable IF the cloud model is genuinely different-family AND cloud calls are audit-logged. Recommend YES with audit trail. UNGROUNDED on cost-quality tradeoff.

**Q3 — Discovery-vs-search heuristic threshold**: how to define "near-duplicate in DKG" precisely (cosine similarity threshold? exact-match? structural-match?). UNGROUNDED — needs empirical tuning per ELAI's DKG schema.

**Q4 — Multi-step sycophancy**: probe applies cleanly to single claims. How to extend to multi-step reasoning where individual steps might be neutral but the chain is sycophantic? UNGROUNDED — Anthropic sycophancy work covers single-turn primarily.

**Q5 — Conformal calibration data bootstrap**: needs N=500 tuples to be useful. How do early sessions calibrate before that data exists? UNGROUNDED — possible bootstrap from generic LLM-calibration corpus, then refine per project; would need verification.

**Q6 — Mode E cost**: every refusal is a "no answer" for the user. What's the user-experience cost of high refusal rates, and does it justify the safety win? Inherently a values judgment, not a research question.

---

## 8. UNGROUNDED CONJECTURE (isolated for user challenge)

Items not directly cited as full-stack patterns. Each component is grounded individually; the combination is synthesis.

**UC-1** — The five-mode taxonomy as the COMPLETE coverage of grounding mechanisms. The taxonomy is comprehensive across the literature surveyed, but I can't prove there's no sixth mode I missed. User should challenge if they think one is missing.

**UC-2** — The orchestrator-decides-mode (not LLM-decides-mode) framing as STRICTLY safer. Plausible by Cohen Boxed Myopic + Everitt current-RF but not directly proven as theorem.

**UC-3** — Mode B (falsifiability inversion) is reduced to last-resort-only with hard cross-family constraint. Literature supports the limits but doesn't explicitly define this as "last resort"; my judgment call.

**UC-4** — `NoveltyClass` heuristic (Search / DistantSearch / DiscoveryCandidate). Three-bucket classification is mine. The underlying primitives (DKG presence, primitive composition) are grounded; the bucket boundaries are not.

**UC-5** — 10-30% healthy refusal band. No source published this. My guess from "calibrated reasoners refuse a non-trivial fraction" — needs tuning per ELAI's session statistics.

**UC-6** — Cross-domain canary task family count of 5 (carried from companion file UC-3). Same UNGROUNDED status.

**UC-7** — Sycophancy threshold for divergence. Empirically tuned, no published default for this exact probe form.

**UC-8** — Discovery vs search HEURISTIC for labeling. The contradiction (§ 2 C1) is grounded; the resolution-by-labeling is mine.

**UC-9** — The `OperationalizationCheck` predicate as a HARD baby-step block. Voyager's analog grounds the spirit; the literal predicate-shaped enforcement is my synthesis.

---

## 9. RELATIONSHIP TO COMPANION FILE

This file (`real-problem-solving-grounded-2026-05-19.md`) addresses inference-time grounding: how to answer a single novel claim well.

Companion (`reasoning-improvement-grounded-2026-05-19.md`) addresses self-improvement: how the reasoning loop itself gets better over time.

The two are orthogonal but interlock:
- The reasoning-improvement file's PPM (§ 3.2 of companion) serves as a Mode A constructive verifier in this file's § 1 Mode A. PPM acts as a checker for step-level claims.
- The reasoning-improvement file's typed reasoning templates (§ 3.1 of companion) become mutable artifacts that the Mode classifier in this file (§ 3.2) routes through.
- The reasoning-improvement file's heterogeneous verifier (§ 3.3 of companion) IS the cross-family Adversary in this file's § 4.3.
- The reasoning-improvement file's MSI/BAI promotion (§ 3.7 of companion) determines when a Mode B Adversary's catches become trusted enough to feed back into L1A data tables.

Build them sequentially. Companion first (template schema + PPM + tripwires) provides the substrate this file's modes consume. This file second (mode routing + checkers + Adversary + UniCR) operates on top.

---

## SOURCES (tier-classified)

T1 (peer-reviewed / arxiv with verified org / Nature):
- [Program of Thoughts PoT (arxiv:2211.12588)](https://arxiv.org/abs/2211.12588)
- [CodeAct (arxiv:2402.01030)](https://arxiv.org/abs/2402.01030)
- [AlphaProof (Nature 2025, doi:10.1038/s41586-025-09833-y)](https://www.nature.com/articles/s41586-025-09833-y)
- [AlphaGeometry (Nature 2024)](https://www.nature.com/articles/s41586-023-06747-5)
- [AlphaGeometry-2 (arxiv:2502.03544)](https://arxiv.org/abs/2502.03544)
- [FunSearch (Nature 2023, doi:10.1038/s41586-023-06924-6)](https://www.nature.com/articles/s41586-023-06924-6)
- [AlphaEvolve (arxiv:2506.13131)](https://arxiv.org/abs/2506.13131)
- [Voyager (arxiv:2305.16291)](https://arxiv.org/abs/2305.16291)
- [Eureka (arxiv:2310.12931)](https://arxiv.org/abs/2310.12931)
- [Coscientist (arxiv:2304.05332)](https://arxiv.org/abs/2304.05332)
- [AI Safety via Debate (arxiv:1805.00899)](https://arxiv.org/abs/1805.00899)
- [CounterBench (arxiv:2502.11008)](https://arxiv.org/abs/2502.11008)
- [Self-Correction Bench (arxiv:2507.02778)](https://arxiv.org/abs/2507.02778)
- [Constitution or Collapse (arxiv:2504.04918)](https://arxiv.org/abs/2504.04918)
- [Constitutional AI in Small LLMs (arxiv:2503.17365)](https://arxiv.org/abs/2503.17365)
- [Chollet On the Measure of Intelligence (arxiv:1911.01547)](https://arxiv.org/abs/1911.01547)
- [ARC Prize 2025 Technical Report (arxiv:2601.10904)](https://arxiv.org/abs/2601.10904)
- [UniCR Trusted Uncertainty (arxiv:2509.01455)](https://arxiv.org/abs/2509.01455)
- [Uncertainty Outside Parametric Knowledge (arxiv:2311.09731)](https://arxiv.org/abs/2311.09731)
- [Rewarding Doubt (arxiv:2503.02623)](https://arxiv.org/abs/2503.02623)
- [LLM Honesty Survey TMLR 2025](https://github.com/SihengLi99/LLM-Honesty-Survey)
- [Sharma et al Sycophancy (arxiv:2310.13548)](https://arxiv.org/abs/2310.13548)
- [Small LMs Need Strong Verifiers (arxiv:2404.17140)](https://arxiv.org/abs/2404.17140)
- [First Try Matters (arxiv:2510.08308)](https://arxiv.org/abs/2510.08308)
- [Lightweight Latent Verifiers (arxiv:2504.16760)](https://arxiv.org/abs/2504.16760)
- [Hubinger Risks from Learned Optimization (arxiv:1906.01820)](https://arxiv.org/abs/1906.01820)
- [Cohen Hutter Osborne 2022 (AI Magazine 43(3))](https://onlinelibrary.wiley.com/doi/10.1002/aaai.12064)
- [Everitt Reward Tampering (arxiv:1908.04734)](https://arxiv.org/abs/1908.04734)
- [NVIDIA NVARC ARC-AGI win](https://developer.nvidia.com/blog/nvidia-kaggle-grandmasters-win-artificial-general-intelligence-competition/)

T2 (expert/org blog):
- [DeepMind AlphaProof IMO blog](https://deepmind.google/blog/ai-solves-imo-problems-at-silver-medal-level/)
- [DeepMind AlphaEvolve blog](https://deepmind.google/blog/alphaevolve-a-gemini-powered-coding-agent-for-designing-advanced-algorithms/)
- [DeepMind FunSearch blog](https://deepmind.google/blog/funsearch-making-new-discoveries-in-mathematical-sciences-using-large-language-models/)
- [ARC Prize 2025 Results](https://arcprize.org/blog/arc-prize-2025-results-analysis)
- [Davis FunSearch comment](https://cs.nyu.edu/~davise/papers/FunSearchComment.pdf)

T3 (general / wiki / opinion):
- [FunSearch Wikipedia](https://en.wikipedia.org/wiki/FunSearch)
- [Marcus FunSearch critique substack](https://garymarcus.substack.com/p/sorry-but-funsearch-probably-isnt)

---

End of real-problem-solving-grounded-2026-05-19.md
