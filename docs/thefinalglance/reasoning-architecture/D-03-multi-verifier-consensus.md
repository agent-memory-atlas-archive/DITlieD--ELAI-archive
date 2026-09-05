# Multi-Verifier Consensus Aggregator (D-3)

- **ELAI ids:** D-3 / Phase 4H.V part 2 / companion §3.3
- **Spec status (as designed):** PLANNED. Feature-map tags DARK (`ELAI_VERIFIER_ENABLE` allowlisted 06-07) (`ELAI-FEATURE-MAP.md:1674-1677`); that flag name is **not** in the D-3 session file (section 16).
- **Crate / module (spec):** `crates/orchestrator/src/verification_result.rs`, `crates/orchestrator/src/cross_context.rs`; dispatch in `task_runner.rs`
- **Purpose of this file:** enough architecture to rebuild the four-signal `VerificationResult` and the ≥2-verifier promotion gate. Not a status ledger.

## 1. What it is

The **VerificationResult aggregator** is the typed consensus over four independent signals: deterministic check, PPM score, intrinsic GG signals, and cross-context agreement. Promotion of a reasoning template is blocked unless the configured `ConsensusRule` is satisfied. Default rule is `AtLeastTwo`. This is V-STaR’s separable verifier (arxiv:2402.06457) plus PRMBench Deception / Circular Logic defense by structure, not by prompt (`D-3_verification_result_aggregator.md:3-10`, `D-SUMMARY_INDEX.md:34-38`).

## 2. Why it exists

Single-verifier sign-off is the failure mode. PRMs and LLM-judges score worst on Deception Resistance and Circular Logic — the categories a gaming self-improver targets (arxiv:2501.03124 §4.2). If this organ is missing, a high PPM score promotes a template that fails compile. If it is faked (policy model “I verified myself”), the model evaluates its own modification. **The model never evaluates its own modification. Promotion needs ≥2 verifiers** (`D-3_verification_result_aggregator.md:21-32`, `:277-279`).

## 3. How it works (runtime)

1. **Who calls it.** `task_runner.rs` `dispatch_verifiers(template, candidate, ctx)` on every CLAIM-emitting baby step after EXECUTING (`D-3_verification_result_aggregator.md:124-135`, `:228-230`).
2. **Signal selection.** `ReasoningTemplate.verifier_required` (D-1) selects applicable signals. Deterministic always dispatched (compile/test if applicable; skip Inapplicable via later D-11 registry — until then, existing hooks). PPM iff `verifier_required == Prm` AND `ELAI_PPM_ENABLE` AND `PpmClient.health()` OK. Intrinsic **always** from current GG-signal block. Cross-context iff the loaded rule requires `SignalKind::CrossContext` (`:48-49`, `:127-134`).
3. **Deterministic (code).** `DeterministicCheck { kind, passed, stderr_snippet }` from compile/test/lint/schema/registry hooks (`:86-88`, `:45`).
4. **PPM (D-2).** Optional `PpmScore`. Pass of this signal = `score >= 0.7` (`:102-104`).
5. **Intrinsic (Phase 4G).** `IntrinsicSignals { mean_logprob, entropy_max, novelty }` collected from scattered `task_runner` / GRC metrics into one struct. Pass of this signal in `compute_verdict` = `mean_logprob >= intrinsic_threshold` (`:33-37`, `:89-90`, `:102-104`).
6. **Cross-context (arxiv:2603.21454).** `CrossContextRunner::run` spawns **two** parallel executor calls with `prior_steps` shuffled; blake3 each output; Jaccard on token shingles **n=3**; `agreement = similarity >= threshold` (default 0.7). 30s max **per side**; timeout → Inconclusive, no panic. Async, both futures concurrent (`:39-42`, `:110-122`).
7. **Verdict (deterministic).** `compute_verdict(rule)`:
   - `AtLeastTwo`: count signals whose individual verdict is Pass. `>=2` → Pass; `0 or 1` → Reject; **Inconclusive iff a required signal is `None`** (`:101-105`).
   - `AtLeastN(n)`: same, threshold n (`:106`).
   - `RequireAll(kinds)`: every listed kind present **and** Pass (`:107`).
   - Default: `AtLeastTwo` (`:108`).
8. **Commit.** One `VerificationResult`-shaped SSB row per candidate. Pass → `VerifierConsensusPass`. Fail → `VerifierConsensusReject`. Mixed ≥1 Pass + ≥1 Reject → also `VerifierDisagreement`. Cross-context always logs `CrossContextDispatched` when run (`:135`, `:141-149`, `:236-237`).
9. **Abstain / fail-closed.** `None` required signal is Inconclusive, not a silent skip. PPM offline + template requires PPM → Inconclusive (cloud C). **No promotion** without consensus Pass: observable forbids `TemplatePromoted` on deceptive PPM-pass / compile-fail (`:233-246`, `:304-310`).
10. **Who must not decide.** The candidate-authoring model does not set `verdict`. `ConsensusRule` is loaded from `elai_cc.toml` `[verification]` (L3-locked; D-9 enforces). No in-code live constant as the authority (`:156-157`, `:238-239`).

**PRMBench-deceptive path (spec self-correction).** Under raw `AtLeastTwo`, `{Ppm Pass, Intrinsic Pass, Deterministic Fail}` counts **2** Passes → consensus Pass, which **defeats** the cloud example. Spec updates Scenario 2: **code claims use `RequireAll([Deterministic, Ppm])`** so Deterministic Fail → Reject (`:171-184`, `:240-246`). Rebuild both the default rule **and** this code-claim override. Do not “fix” AtLeastTwo by inventing a deterministic veto unless D-9 L3 config says so.

## 4. Types, enums, objects

Copy from `D-3_verification_result_aggregator.md:80-98`. Do not invent fields.

```
struct VerificationResult {
  deterministic: Option<DeterministicCheck>,
  ppm: Option<PpmScore>,           // D-2
  intrinsic: Option<IntrinsicSignals>,
  cross_context: Option<CrossContextAgreement>,
  consensus_rule: ConsensusRule,
  verdict: Verdict,
}

struct DeterministicCheck {
  kind: DeterministicKind,
  passed: bool,
  stderr_snippet: Option<String>,
}
enum DeterministicKind { Compile, Test, Lint, Schema, RegistryCheck }

struct IntrinsicSignals { mean_logprob: f32, entropy_max: f32, novelty: f32 }
  // “each in well-defined range; documented in struct” — ranges not numeric in D-3 body

struct CrossContextAgreement {
  context_a_hash: Blake3Hash,
  context_b_hash: Blake3Hash,
  jaccard_similarity: f32,
  agreement: bool,                 // similarity >= threshold (default 0.7)
}

enum ConsensusRule {
  AtLeastTwo,
  AtLeastThree,
  AtLeastN(u8),
  RequireAll(Vec<SignalKind>),
}  // closed; L3-locked companion §4

enum SignalKind { Deterministic, Ppm, Intrinsic, CrossContext }
enum Verdict { Pass, Reject, Inconclusive }

struct CrossContextRunner {
  executor: Arc<dyn Executor>,
  jaccard_threshold: f32,
}
  fn run(&self, claim: &Claim, ctx: &OrchContext) -> CrossContextAgreement
```

**Individual Pass predicates used by `compute_verdict`** (`:102-104`):

| Signal | Pass iff |
|---|---|
| Deterministic | `passed == true` |
| Ppm | `PpmScore.score >= 0.7` |
| Intrinsic | `mean_logprob >= intrinsic_threshold` |
| CrossContext | `agreement == true` |

**Toml `[verification]`** (`:152-155`):

```
consensus_rule = "at_least_two"
intrinsic_logprob_threshold = -3.0
intrinsic_entropy_max = 3.5
intrinsic_novelty_min = 0.2
cross_context_jaccard_threshold = 0.7
```

**SSB SCHEMA 48→49 (atomic with D-2 scaffolds)** (`:137-149`):

```
VerifierConsensusPass { template_id, candidate_id: Uuid, signals: Vec<SignalKind>, consensus_rule: String }
VerifierConsensusReject { template_id, candidate_id, failing_signals: Vec<(SignalKind, String)>, consensus_rule: String }
VerifierDisagreement { template_id, signals: Vec<(SignalKind, String)> }
CrossContextDispatched { claim_id: Uuid, context_a_hash, context_b_hash, jaccard: f64, agreement: bool }
+ D-2 PpmScoreEmitted, PpmUnavailable
```

Phase 2A forward stub `VerificationResult { claim, passes, evidence, method_used: String }` is **not** this organ (`02A-grc-state-machine.md` stubs). D-3 replaces it.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `dispatch_verifiers` + `compute_verdict`. Model proposes a candidate; engine returns `Verdict`.

**Verdict table (AtLeastTwo, all four Options Some):**

| # Pass signals | Verdict |
|---|---|
| ≥2 | Pass |
| 0 or 1 | Reject |
| required signal `None` | Inconclusive (`:105`) |

**RequireAll([Deterministic, Ppm])** (code-claim / PRMBench deceptive):

| Deterministic | PPM | Verdict |
|---|---|---|
| Pass | Pass | Pass |
| Fail | Pass | **Reject** (cloud A / wiring observable) |
| Pass | None | Inconclusive |
| Fail | None | Inconclusive if PPM required (`:161-162`, `:308-310`) |

**Forced rules:**

- Default rule AtLeastTwo when `[verification]` absent (`:192`).
- Live rule loaded from toml, not a hard-coded authority (`:238-239`).
- Cross-context timeout per side → Inconclusive, not panic (`:119-120`, `:164-165`).
- ≥1 Pass + ≥1 Reject → extra `VerifierDisagreement` row (D-4 monitors rate) (`:167-168`, `:202-205`).
- One VerificationResult SSB per candidate; no silent skip (`:236-237`).

**UNCERTAIN:** whether CrossContext `agreement=false` is Reject of that signal (AtLeastTwo may still Pass on other two) vs Scenario 3 “disagreement → Inconclusive” (`:185` vs `:102-104`). Section 16.

## 6. Grounding, receipts, gates

Evidence is the four typed signals plus SSB. Selector: `verifier_required` + enable flags + rule.

**Never allowed as a confidence signal:** LLM self-rating of its own candidate; single-PPM sign-off; treating `None` as Pass; swallowing unavailable signals; in-code edit of `ConsensusRule` by the agent (`:233-239`, `:277-279`).

**Promotion gate:** D-5 consumes **`VerifierConsensusPass` count** as the pass/fail for BAI. No `TemplatePromoted` without consensus Pass (`:231`, `:246`).

**Deception receipt (wiring observable):** PPM Pass + compile Fail → `VerifierConsensusReject` with `failing_signals` containing Deterministic + stderr snippet + `PpmScoreEmitted` showing PPM Pass (`:240-246`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| default rule | `AtLeastTwo` | global toml; tunable per template_kind later in D-9 (`:194-198`) |
| PPM Pass threshold | `score >= 0.7` | per step |
| CCV Jaccard | 0.7 | per pair |
| CCV shingle n | 3 | token-level (`:116-118`) |
| CCV timeout | 30 s **per side** | (`:119-120`) |
| intrinsic_logprob_threshold | -3.0 | toml |
| intrinsic_entropy_max | 3.5 | toml (not used in `compute_verdict` body — gap) |
| intrinsic_novelty_min | 0.2 | toml (not used in `compute_verdict` body — gap) |
| SCHEMA | 48 → 49 | atomic D-2+D-3 |
| unit tests | ≥12 | orchestrator |
| table-driven consensus | ≥10 scenarios | (`:294-295`) |

**Overflow:** CCV timeout → Inconclusive. PPM down when required → Inconclusive, not auto-pass (`:308-310`).

## 8. Roles and neighbors

Orchestrator-owned. Executor is invoked **twice** for CCV with shuffled prior steps; that executor is not the verdict authority.

**Consumes:** D-2 `PpmClient`, D-1 `verifier_required`, Phase 4G GG-signal / `AgentState`, `SsbWriter`, `SeededClock` for CCV deadline (`:222-227`).

**Produces:** `VerificationResult` + four new SSB types + SCHEMA 49.

**Downstream:** D-5 BAI (`VerifierConsensusPass`), D-4 tripwire (`VerifierDisagreement`). D-11 will replace ad-hoc deterministic hooks. D-13 Adversary injects a later cross-family signal (`:331-336`).

**Must not fork:** a second aggregator inside the agent; single-verifier sign-off path; policy self-eval.

**Parallel-safe with D-5; not with D-4** (`:15-17`).

## 9. Config / flags / defaults

- `[verification]` keys in section 4; L3-LOCKED (`:156-157`).
- Defaults applied when section absent: AtLeastTwo (`:192`).
- PPM still gated by `ELAI_PPM_ENABLE` (D-2).
- Feature-map `ELAI_VERIFIER_ENABLE` is **not** specified in D-3 (section 16).
- Decision (a): AtLeastTwo not AtLeastThree to avoid gate explosion + PPM latency; D-9 may tune per `template_kind` (`:194-198`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `verification_result.rs` | types + `compute_verdict` (`:80-108`) |
| `cross_context.rs` | `CrossContextRunner` (`:110-122`) |
| `task_runner.rs` `dispatch_verifiers` | CLAIM-emit sites (`:124-135`) |
| `ssb.rs` SCHEMA 49 | six variants (`:137-150`) |
| `elai_cc.toml` `[verification]` | (`:152-155`) |
| integration | `orchestrator/tests/verifier_consensus_integration.rs` (`:170-186`) |
| contract | `contracts/tests/verifier_consensus_contract.rs` (`:188-192`) |
| decisions | `workflow/reasoning/D-3-decisions.md` |
| papers | `vstar-2402.06457`, `ccv-2603.21454` (`:71-75`) |

Deterministic hooks until D-11: existing `SlashCommand::Dispatch` compile/test/lint (`:45`, `:127-129`).

## 11. Journal / SSB events

SCHEMA 49. Six variants listed in section 4. Round-trip + SQL. `rg` expect ≥24 hits across enum/sql/from_str/payload (`:251-257`).

Observable: deceptive cloud task → 1 `VerifierConsensusReject` + `PpmScoreEmitted`; **zero** `TemplatePromoted` (`:240-246`).

`VerifierDisagreement` is a **separate** row, not folded into Reject, so D-4 can watch disagreement **rate** as verifier-drift (`:202-205`).

## 12. Tests and observables

Named unit tests (`:159-168`):

| Id | Assert |
|---|---|
| T-two-pass | AtLeastTwo, 2 passing → Pass |
| T-two-one | 1 passing → Reject |
| T-two-zero | 0 passing → Reject |
| T-require-ok | RequireAll([Det, Ppm]) both present+pass → Pass |
| T-require-none | Ppm=None → Inconclusive |
| T-toml | rule deserializes from toml string |
| T-ccv-timeout | either-side timeout → Inconclusive, no panic |
| T-jaccard | identical inputs → deterministic similarity |
| T-disagree | 1 Pass + 1 Reject + others None → `VerifierDisagreement` |

Integration (`:170-186`): (1) det+PPM+intrinsic Pass → consensus Pass. (2) code claim `RequireAll([Det, Ppm])`, det Fail, PPM Pass → Reject. (3) CCV disagreement → Inconclusive (per test list; see gaps). In-memory SSB writer asserts rows.

Cloud (`:304-310`): (A) PPM pass / compile fail → Reject, not promoted. (B) simple math CCV Jaccard ≥0.7, `agreement=true`. (C) PPM offline + required → Inconclusive; `PpmUnavailable` + Reject **or** Inconclusive-flag depending on rule.

Done-gates: ≥12 orchestrator tests; contracts pass; SCHEMA 49; papers present (`:286-313`).

Adversarial: no `single.verifier` / skip-consensus path (`:277-279`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `VerificationResult` with four optional signals + `ConsensusRule` + `Verdict {Pass, Reject, Inconclusive}`.
- Closed `SignalKind` (4) and `ConsensusRule` (`AtLeastTwo`, `AtLeastThree`, `AtLeastN`, `RequireAll`).
- Default AtLeastTwo; `None` required signal → Inconclusive, not Pass.
- PPM-only Pass cannot promote a compile-fail **code** candidate (`RequireAll([Deterministic, Ppm])` or equivalent L3 rule).
- CCV: two shuffled contexts, blake3 outputs, Jaccard shingles n=3, 30s/side, timeout Inconclusive.
- Rule loaded from config (L3); agent cannot edit thresholds in-process.
- One SSB result per candidate; disagreement row distinct from reject.
- SCHEMA 49 includes D-2 PPM events + four D-3 events.
- Model that wrote the candidate does not set `verdict`.

A single LLM-as-judge, a PPM threshold alone, or “ask the model if it is sure” is a cousin, not this organ.

## 14. Non-goals / banned designs

- Tripwire stack — D-4 (`:332`).
- BAI posterior updates — D-5 (`:333-334`).
- Typed checker registry — D-11 (`:335`).
- Adversary cross-family signal — D-13 (`:336`).
- Single-verifier sign-off (`:25-26`, `:277-279`).
- Swallowing `None` as Pass (`:233-234`).
- Infinite CCV wait (`:235`).
- Silent skip when a signal is unavailable (`:236-237`).
- In-code live `ConsensusRule` as authority (`:238-239`).
- Model self-certifying its own template modification.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-3_verification_result_aggregator.md` (full file). Index: `D-SUMMARY_INDEX.md:34-38`, `:270`. Feature map: `ELAI-FEATURE-MAP.md:1674-1677`. Companion (not in this tree): §3.3 / §1.7 / §1.13 as cited `:53-55`. Papers: V-STaR 2402.06457, CCV 2603.21454, PRMBench 2501.03124, rStar-Math 2501.04519, latent-verifiers 2504.16760.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- AtLeastTwo vs cloud-A deception: default math would Pass; Scenario 2 requires `RequireAll` for code — **who selects that override** (template_kind? `verifier_required`? D-9?) (`:171-184` vs `:108`).
- Scenario 3 CCV disagreement → Inconclusive vs `agreement=false` as a failed signal under AtLeastTwo (`:185` vs `:102-105`).
- Cloud C “Reject or Inconclusive-flag depending on rule” (`:308-310`).
- `intrinsic_entropy_max` / `intrinsic_novelty_min` in toml but `compute_verdict` only names `mean_logprob` (`:89-90`, `:102-104`, `:152-155`).
- Numeric ranges for the three intrinsic fields (“documented in struct” — not in D-3) (`:89-90`).
- `Claim` / `Candidate` / `Executor` types (`:114`, `:125`).
- Whether deterministic Fail is a **veto** in any rule besides RequireAll.
- `ELAI_VERIFIER_ENABLE` in feature-map, absent from D-3 (`ELAI-FEATURE-MAP.md:1676`).
- How `AtLeastThree` behaves when only two signals were dispatched (`:95`).
- Tokenization for Jaccard shingles (`:116-118`).
- Whether Phase 2A stub type is deleted, aliased, or left in `reasoning/` (`02A` stubs vs D-3 file).
- `dispatch_verifiers` always-on vs feature-map default `verifier_adapters=None` (`ELAI-FEATURE-MAP.md:1675`).
