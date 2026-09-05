# ClaimRisk score + convergence fuse (L-2)

- **ELAI ids:** L-2 / L-T1 / NN-2 / HG-1 / HG-2 / HG-3 / WAVE 1 KEYSTONE
- **Spec status (as designed):** PLANNED (`L-plan/L-2_claimrisk_score_convergence_fuse.md:3`). Live tree has a **cousin fill** in `crates/orchestrator/src/claim/risk.rs` plus `ClaimRiskScored` SSB; that fill is not the rebuild organ until G-7 is generalized (not parallel-scored), “agree” is distinct from “two fired”, L-15 R1 receipt-for-Low holds, and `causal_supported` is only an L-5 interventional **pass**.
- **Crate / module (spec):** `crates/orchestrator/src/claim/risk.rs`; production caller is the orchestrator claim-intake seam after L-1 extract (`score_claim_risk`) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:37-44`).
- **Purpose of this file:** enough architecture to rebuild the model-agnostic ClaimRisk object and the ≥2-predictor fuse. Not a status ledger.

## 1. What it is

The **ClaimRisk convergence fuse** is the keystone object of the Causal Grounding Gate. It generalizes G-7’s `claim_score` from the agent’s own output to **any** backend’s claim (HG-3) and applies the paper’s convergence rule (§4.2): a **High/Critical** verdict requires **≥2 mechanistically-distinct predictors to agree**. Predictors are P1 round-trip residue (L-3), P2 reasoning oscillation (J-85, consumed not rebuilt), P3 semantic entropy (L-4), plus the L-5 counterfactual pass as the causal bit. A single predictor may caveat but **never** blocks (HG-2). The output is a per-claim tier `{Low, Medium, High, Critical}` that drives L-6/L-7. L-2 **fuses + tiers**; it does **not** decide the action (L-7) and does **not** itself run predictor bodies (it orchestrates L-3/L-4/L-5) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:5`, `L-plan/L-SUMMARY_INDEX.md:18-23`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:82-83`).

## 2. Why it exists

ELAI already encodes “convergence = trust” (D-3 ≥2-concur, H-40 decorrelation) for **verifier promotion**, not for hallucination risk (`L-plan/L-2_claimrisk_score_convergence_fuse.md:13`). The paper’s deepest design move is that three mechanistically-distinct predictors **agreeing** is itself the evidence the signal is real (`L-plan/L-2_claimrisk_score_convergence_fuse.md:13`, arXiv:2606.27326 §4.2 [EXTERNAL/DESIGN-RATIONALE]).

If this organ is missing, L-6/L-7 have no tier, and each predictor can independently block (cost + false alarms). If it is faked (LLM picks the tier, or two resampling-family signals “agree” a stable-wrong claim is safe), HG-2 is theatre. L-15 R1 hardens that last case: a Low/Allow on a receipt-available factual claim requires **≥1 passing evidence-grounded receipt** (P1 or do-test), not absence-of-disagreement (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:104-110`, `L-plan/L-0_INDEX.md:186-189`).

HG-1 binds here: a claim is only `causal_supported` when L-5’s interventional do-test **passes**; static/temporal evidence stays at its weaker tier (`L-plan/L-2_claimrisk_score_convergence_fuse.md:13,48`). Thresholds are ELAI-measured at L-10 (ID-5), never the paper’s ρ/AUROC (`L-plan/L-2_claimrisk_score_convergence_fuse.md:13,83`).

## 3. How it works (runtime)

1. **Who calls it.** Orchestrator claim-intake, after L-1 `extract_claims`, via `score_claim_risk`. A `#[cfg(test)]` caller never satisfies production (R7) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:42-44`).
2. **Input.** An L-1 `ClaimSet`. Cheap signals available immediately: L-1 `refs-unresolved`; optionally G-7 `claim_score` components when the backend exposes them. Predictor bodies are **neighbors** (L-3/L-4/L-5/J-85), invoked through the cost-gate (`L-plan/L-2_claimrisk_score_convergence_fuse.md:31-32`).
3. **Cost-gate (deterministic, cheap first).** Run cheap signals: L-1 refs-unresolved + L-3 round-trip residue. Invoke expensive predictors (L-4 resampling, L-5 do-test) **only** after cheap risk is medium-or-above, routed by J-84 (`L-plan/L-2_claimrisk_score_convergence_fuse.md:5,31,47`). Low cheap-signal → skip expensive layer (test e).
4. **Collect `PredictorSignal`s (engine, not model).** Each source emits `{source: PredictorKind, value, fired}`. Logprob is **optional** (HG-3): a backend with no logprobs still yields a valid tier (`L-plan/L-2_claimrisk_score_convergence_fuse.md:29-30,47`).
5. **Fuse (deterministic, HG-2).** Tier is a function of how many **distinct** predictors **fired AND agreed**:
   - 0 fired → `Low` (subject to L-15 R1 below)
   - 1 distinct fired → at most `Medium` (may caveat; never High/Critical)
   - ≥2 distinct **agree** on high residual → `High` or `Critical`
   Exact Low/Medium/High/Critical boundaries are ELAI-measured at L-10; seed conservatively + observe-first (GD-4) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:30,100`).
6. **L-15 R1 (binding on this organ).** A `Low`/`Allow` verdict on a **receipt-available** (class A/B/C) factual claim **requires ≥1 passing evidence-grounded receipt** (P1 round-trip **or** P-cf do-test). Two resampling-family predictors (P2 oscillation + P3 entropy) share the confident-consistent blind spot and must not “agree” a stable-wrong claim into Low (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:104-110`).
7. **`causal_supported` (HG-1).** Set **only** by L-5’s interventional **pass**. An L-5 run that does not change B, or static/import-edge evidence, leaves it false (`L-plan/L-2_claimrisk_score_convergence_fuse.md:30,48`).
8. **Commit.** One `ClaimRisk` per claim. Journal `ClaimRiskScored` (Pattern-2 Kind LowRisk/MediumRisk/HighRisk/CriticalRisk) **or** generalize G-7 rows (net 0..+1, W-9) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:54`, `L-plan/L-0_INDEX.md:131`). Default behavior = **OBSERVE-only** (record, do not act) until L-7’s gate flag (`L-plan/L-2_claimrisk_score_convergence_fuse.md:32,71`).
9. **Abstain / fail-closed.** Missing expensive predictor on a Low cheap-signal is skip, not High. Missing logprob is valid Low/Medium/High from the other signals, never a hard fail. LLM cannot write or override `tier` (TD-3) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:49`). A reasoning **trace** is never the confidence signal (RR-1 / K-16) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:26,97`).
10. **The model may generate** claim text (already typed by L-1). It may **not** pick `RiskTier`, set `causal_supported`, mark a predictor `fired`, or call the fuse.

Live cousin (do not rebuild as this organ): `score_claim_risk_with` in `claim/risk.rs:58-79` fuses by **count of distinct fired kinds** (`0→Low, 1→Medium, ≥2→High` unless mean value `≥ 0.90` → Critical, `risk.rs:38-91`). That is “two fired”, not “two agreed”, and `CRITICAL_MEAN = 0.90` is an unmeasured seed. `causal_supported` is true if any `Counterfactual` signal **fired**, not if the do-test **passed** (`risk.rs:93-95`). Cost-gate `expensive_predictors_for` treats `RoundTrip` as expensive on Medium (`risk.rs:43-51`), against the plan’s cheap = L-1 + P1 residue. Extra live kind `RefsUnresolved` is compatible as the cheap L-1 signal. `ClaimRisk.tier` is private with a getter (`risk.rs:35-36`) — LLM-override resistance by API, not by a policy check.

G-7 pin `production_agent_rpc.rs:334` is **stale** against current source (that region is trifecta-quarantine comments, `production_agent_rpc.rs:329-337`). Live G-7 fuse is `claim_confidence.rs::compute_claim_score` (`claim_confidence.rs:292-306`): logprob (optional) + self-consistency + exec-agreement, Platt-rescaled. L-2 **generalizes** that object over a ClaimSet; it does not re-implement Platt inside `risk.rs` (GD-1).

## 4. Types, enums, objects

Copy from `L-plan/L-2_claimrisk_score_convergence_fuse.md:29`. Do not invent fields.

```
enum RiskTier { Low, Medium, High, Critical }

enum PredictorKind {
  RoundTrip,             // P1  L-3
  Oscillation,           // P2  J-85 (consumed)
  SemanticEntropy,       // P3  L-4
  Counterfactual,        // L-5 do-test (causal bit)
  LogprobOptional,       // G-7 booster; never required (HG-3)
}

PredictorSignal {
  source: PredictorKind,
  value,                 // numeric; exact domain UNCERTAIN (§16)
  fired: bool,
}

ClaimRisk {
  tier: RiskTier,
  signals: Vec<PredictorSignal>,
  causal_supported: bool,   // ONLY L-5 interventional pass
}
```

**Live extras (cousin, not plan fields):** `consult: Vec<PredictorKind>`, `claim_digest: [u8; 32]`, `PredictorKind::RefsUnresolved`, `PredictorSource` trait (`risk.rs:11-32`). Rebuild may keep digest for GD-2; do not treat `RefsUnresolved` as a fourth **mechanistically distinct** High/Critical voter alongside P1 — it is the cheap gate, not a second mechanism that “agrees” with P1 residue on the same missing file.

**G-7 seed (generalize, do not copy into `risk.rs`):**

```
compute_claim_score(
  mean_logprob: Option<f64>,      // None on cloud lane (HG-3 already)
  sample_fingerprints: &[String],
  sample_stdout_hashes: &[String],
  params: &PlattParams,
) -> (f64, ClaimScoreComponents)
```

(`claim_confidence.rs:292-306`). Platt ECE 0.32→0.03 is G-7’s calibration story (`L-plan/_grounding/L-CGG-GROUNDING-MAP.md:28-30`); L-2 does not re-Platt.

**D-3 fuse seed:** `VerificationResult` promotion needs ≥2 verifiers (`D-03-multi-verifier-consensus.md` §1). L-2 **reuses the ≥2-concur rule** for risk, not the four-signal promotion object (GD-1) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:5`).

**H-39 penalty term:** `correct=+1 / abstain=0 / confident-wrong=-t/(1-t)` (`L-plan/_grounding/L-CGG-GROUNDING-MAP.md:35-36`). How it enters `value` is **UNCERTAIN** (§16).

**D-16 tau:** abstain threshold consumed by L-7, not a tier boundary here (`L-plan/L-2_claimrisk_score_convergence_fuse.md:8`).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator fuse per claim. Predictors are callees. No token-level FSM. Observe-only until L-7 (`L-plan/L-2_claimrisk_score_convergence_fuse.md:32`).

**Decision table (must be unit-tested):**

| Cheap signals | Distinct predictors agreeing high-residual | Receipt (R1, class A/B/C factual) | Tier cap |
|---|---|---|---|
| none fired | 0 | n/a (no receipt available) | Low |
| none fired | 0 | receipt **available** but none **passing** | **not Low** (R1) — at least Medium |
| L-1 unresolved XOR P1 residue | 1 | — | Medium (HG-2) |
| P1 residue **and** P3 diverge | ≥2 distinct | — | High or Critical (observable) |
| P2 **and** P3 only, no P1/P-cf | 2 resampling-family | receipt available | **must not Low**; do not treat as HG-2 “agree” for Allow (R1 + H-40) |
| L-5 do-test pass | — | — | `causal_supported=true`; tier still from HG-2 count |
| L-5 do-test fail / not run | — | — | `causal_supported=false` |

**Forced rules:**

- High/Critical ⇒ ≥2 distinct predictors agree (stub_ban) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:46`).
- Logprob absence must still produce a tier (`:47`).
- WAVE 1: not parallel-safe with anything; L-3/L-4/L-5 depend on this interface (`:9`).
- L-15 R2 does **not** change L-2’s interface; it reorders when P1/P-cf vs P3 **land** (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:112-117`).

## 6. Grounding, receipts, gates

This organ **does not verify files or run tests**. It consumes predictor signals and an optional G-7 score.

**Selector hierarchy (what may raise a tier):**

1. External predictor receipts (P1 reconstruction fail, L-5 outcome hash flip) — evidence-grounded.
2. ≥2 distinct mechanisms agreeing — HG-2.
3. Optional logprob / G-7 components — booster only.
4. Reasoning trace / LLM “I’m unsure” — **never** (RR-1).

**Never allowed as a confidence signal / tier chooser:** LLM self-rating, scalar 0–1 from the policy model, window fill, paper ρ≈0.80, a single predictor, P2+P3 agreement as a substitute for a missing P1/P-cf receipt on a checkable claim (R1), static import-edge as `causal_supported` (HG-1) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:46-49,97`, `L-plan/L-0_INDEX.md:49-53,162`).

**Decorrelation (H-40):** P1/P2/P3 must be genuinely distinct or ≥2-agree is fake convergence. Verify at L-4 that semantic-entropy and round-trip do not collapse to the same signal (`L-plan/L-2_claimrisk_score_convergence_fuse.md:101`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| types + fuse + cost-gate | ~40 + ~70 + ~50 LOC | `claim/risk.rs` (~160 total) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:29-32,89`) |
| seam | ~25 LOC | claim-intake after extract |
| contract | ~90 LOC | `l2_claim_risk_contract.rs` |
| expensive predictors | L-4/L-5 only on medium-or-above cheap risk | per claim, J-84 routed |
| fixture vs live model | L-4/L-5 fixture tiers ship now; live local model RunPod-gated | W-10 (`:7`) |
| SCHEMA | +`ClaimRiskScored` or generalize G-7; net 0..+1 | W-9 (`L-plan/L-0_INDEX.md:131`) |
| observe-only | records tiers, does not act | until L-7 (`:71`) |
| live seed `CRITICAL_MEAN` | `0.90` (`risk.rs:38-39`) | **not** spec; L-10 measures; do not copy as authority |

No per-topic GRC budget here (2A/4A). Overflow of `value: f64` (NaN/Inf): live contract `l2_risk_finite_tier_with_extreme_signal` caps `f64::MAX` + one predictor at Medium (`l2_claim_risk_contract.rs:32-37`); plan is silent (§16).

## 8. Roles and neighbors

Role-agnostic: any B-37 backend (HG-3).

**Consumes:** L-1 `ClaimSet`; G-7 `compute_claim_score` (generalize); D-3 ≥2-concur rule; H-40 decorrelation; H-39 `AbstentionScored`; D-16 tau (for L-7, not fused here); J-84 effort classifier (cost-gate); J-85 oscillation as P2 **input** (`L-plan/L-2_claimrisk_score_convergence_fuse.md:39-41`).

**Produces:** `ClaimRisk` per claim for L-6/L-7; `ClaimRiskScored` SSB.

**Must not fork:** a second fuse inside the agent; re-implementation of G-7 or D-3 (GD-1 P0); LLM tier override; treating J-85 as rebuilt here.

**Frontend:** ClaimRisk tile on trust/context panel (decide at edit) or justified N/A (`L-plan/L-2_claimrisk_score_convergence_fuse.md:50-51`).

**L-22 rider:** demarcation fuse — Causal/Behavior with empty falsifier → High **floor** (HG-5), later (`L-plan/L-SUMMARY_INDEX.md:241`). Not L-2 WAVE-1 scope; leave a slot, do not invent the floor here.

## 9. Config / flags / defaults

- **No behavior gate on L-2.** Observe-only. Action flag is L-7 (`ELAI_HALLUCINATION_GATE_DISABLE`) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:76`).
- G-7 remains behind `ELAI_CONFORMAL_REFUSAL_ENABLE` (DARK). L-2 **activates** the generalized score under the CGG; it does not require that flag to emit a black-box tier (HG-3, DC-2) (`L-plan/_grounding/L-CGG-GROUNDING-MAP.md:118-119`).
- Seed tier function: conservative + observe-first; never a paper constant (`L-plan/L-2_claimrisk_score_convergence_fuse.md:100`).
- Decision doc: `workflow/reasoning/L-2-decisions.md` (`:34`). **Absent** at architecture time.
- Hash: blake3. No `.unwrap()` on prod (`:49`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/claim/risk.rs` | `ClaimRisk` / `RiskTier` / `PredictorSignal` + fuse + cost-gate (`L-plan/L-2_claimrisk_score_convergence_fuse.md:37`) |
| `score_claim_risk` | production name in wiring (`:43`) |
| G-7 `compute_claim_score` | `claim_confidence.rs:292` (live); plan pin `production_agent_rpc.rs:334` is stale |
| D-3 aggregator | `verification_result.rs` / D-03 architecture |
| J-84 | `reasoning_classifier.rs` (plan; see `J-84-grcv3-effort-classifier.md`) |
| J-85 oscillation | `overthinking_guard.rs` (plan; P2 input only) |
| `crates/contracts/tests/l2_claim_risk_contract.rs` | tests (a)–(g) (`:86`) |
| `workflow/reasoning/L-2-decisions.md` | seed function + W-4 |

**Live neighbors:** `claim.rs:307` `pub mod risk`; SSB `ClaimRiskScoredKind` / `ClaimRiskScoredPayload` (`ssb/payloads.rs:22054-22075`); production log in `l2_executor_claim_risk_logs_per_tier` (`l2_claim_risk_contract.rs:125`); L-9 research profile writes one aggregate `ClaimRiskScored` (`reasoning/research_profile.rs:5,97`).

## 11. Journal / SSB events

Plan (`L-plan/L-2_claimrisk_score_convergence_fuse.md:54`, `L-plan/L-0_INDEX.md:131`):

```
ClaimRiskScored   // Pattern-2; OR generalize G-7 rows
  Kind: LowRisk | MediumRisk | HighRisk | CriticalRisk
```

Live payload (cousin shape — rebuild Kind pair; field list is a later fill):

```
ClaimRiskScoredPayload {
  kind: ClaimRiskScoredKind,
  low_count, medium_count, high_count, critical_count: usize,
  predictors: Vec<PredictorKind>,
  consult: Vec<PredictorKind>,
  unresolved_ref_count, claim_count: usize,
  claim_digests: Vec<[u8; 32]>,
}
```

(`ssb/payloads.rs:22064-22075`). Aggregate-over-ClaimSet vs per-claim row is a live choice; plan says **per claim** (`L-plan/L-2_claimrisk_score_convergence_fuse.md:32`). **UNCERTAIN** which wins (§16). GD-2: tier recomputable from recorded signals under `ELAI_SEED` (`:82`).

## 12. Tests and observables

Named contract (`L-plan/L-2_claimrisk_score_convergence_fuse.md:86`):

| Id | Assert |
|---|---|
| (a) | two agreeing distinct predictors → High/Critical |
| (b) | one predictor firing alone → at most Medium (HG-2) |
| (c) | valid tier with logprob absent (HG-3) |
| (d) | `causal_supported` false until an L-5 **interventional** signal is present (HG-1) |
| (e) | cost-gate skips expensive predictors on a Low cheap-signal claim |
| (f) | tier recomputable from recorded signals (GD-2) |
| (g) | LLM cannot override a tier (TD-3) |
| R1 | class A/B/C factual cannot score Low without a passing P1 or P-cf receipt (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:186`) |

Wiring (`:58-62`): `rg "RiskTier|>= ?2|two_distinct|convergen" claim/risk.rs`; `rg -ci "logprob.*required|require.*logprob"` MUST be 0; `L-2-decisions.md` exists.

**observable_effect** (`:52-55`, `L-plan/L-SUMMARY_INDEX.md:23`): claim with P1-residue-high **and** P3-divergent scores Critical (two distinct agree); claim with only **one** predictor firing scores at most Medium. Negative: feeding logprob-only on a backend that exposes none still produces a valid score (HG-3). Recomputable from the journal (GD-2).

Adversarial: single-predictor High; logprob required; `causal_supported` from import-edge; paper AUROC as threshold; P2+P3 Allow on a checkable claim with no receipt.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `RiskTier` is exactly `{Low, Medium, High, Critical}`.
- `PredictorKind` includes `{RoundTrip, Oscillation, SemanticEntropy, Counterfactual, LogprobOptional}`.
- High/Critical requires ≥2 **mechanistically distinct** predictors that **agree**; one signal caps at Medium.
- Logprob is optional; a no-logprob backend still tiers.
- `causal_supported` is true only after L-5 interventional **pass** (HG-1).
- Cheap-first cost-gate: L-4/L-5 do not run on Low cheap risk; J-84 routes the expensive layer.
- G-7 `claim_score` is generalized, not re-implemented; D-3 ≥2-concur is reused, not forked.
- LLM cannot set/override the tier. A reasoning trace is not a signal (RR-1).
- Observe-only until L-7.
- L-15 R1: receipt-available factual Low requires a passing P1 or do-test receipt.
- Thresholds are ELAI-measured (L-10), never paper ρ/AUROC.
- Tests (a)–(g) + R1 exist. Production seam is post-extract `score_claim_risk`.

A prompt “rate this claim 1–4”, a G-7-only score on the agent’s own output, or the live “≥2 fired ⇒ High / mean≥0.90 ⇒ Critical” fuse **without agree + R1** is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Deciding allow/block/abstain (L-7) (`L-plan/L-2_claimrisk_score_convergence_fuse.md:5`).
- Running predictor bodies inside `risk.rs` (L-3/L-4/L-5/J-85).
- Re-implementing G-7 or D-3 (GD-1 P0) (`:49`).
- Single-predictor High/Critical (`:46`).
- Logprob as a required input (`:47`).
- Causal tier without L-5 pass (`:48`).
- LLM tier override (TD-3).
- Paper ρ/AUROC as ELAI thresholds (ID-5) (`:83`).
- Treating P2+P3 agreement as grounding on a checkable claim (L-15 R1).
- Acting on High/Critical here (observe-only; L-6/L-7 act).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-2_claimrisk_score_convergence_fuse.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:18-23`. Doctrine: `L-plan/L-0_INDEX.md:49-53,89-90,131,186-189`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:28-36,69-73,82-83,99-111,118-119`. Binding revision: `L-plan/L-15_repair_controller_and_evidence_first_reorder.md:104-110,186`. Neighbors: `reasoning-architecture/D-03-multi-verifier-consensus.md`, `J-84-grcv3-effort-classifier.md`, `J-85-grcv3-overthinking-guard.md`, `H-39-abstention-aware-scoring.md`. Live: `claim/risk.rs`, `claim_confidence.rs:292-306`, `ssb/payloads.rs:22054-22075`, `l2_claim_risk_contract.rs`. Paper: arXiv:2606.27326 §4.2 [EXTERNAL/DESIGN-RATIONALE].

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact map from (count, agreement, values) to Low vs Medium vs High vs Critical. Plan defers to L-10 + L-2-decisions.md (`L-plan/L-2_claimrisk_score_convergence_fuse.md:100`).
- What “agree” means operationally (same direction vs both `fired` vs value thresholds vs NLI).
- `PredictorSignal.value` domain, bounds, NaN/Inf policy.
- Whether `RefsUnresolved` is a `PredictorKind` or a cheap pre-signal (plan text vs live enum).
- Whether P1 residue is cheap (plan §3 task 3) or expensive (live `expensive_predictors_for(Medium)` includes `RoundTrip`).
- How H-39 confident-wrong penalty and D-16 tau enter the fuse vs L-7 only.
- Per-claim SSB row vs live aggregate `ClaimRiskScoredPayload` counts.
- Production_agent_rpc.rs:334 pin vs `claim_confidence.rs:292` — rebuild the **function**, not the stale line.
- Class A/B/C receipt-available definition (L-15 names it; the class table is not in L-2).
- HG-5 High-floor for empty falsifier (L-22) vs WAVE-1 fuse.
- Frontend tile vs N/A.
- How J-84’s `{Zero, Depth, Width, EscalateOrAbstain}` maps onto “run L-4/L-5” (J-84 plan tiers ≠ L-2’s medium-or-above sentence).
