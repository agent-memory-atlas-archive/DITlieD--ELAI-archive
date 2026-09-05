# Conformal refusal (UniCR) (D-16)

- **ELAI ids:** D-16 / RPS.VII / D0-020 / D0-051 / D003-D16-E001..E031
- **Spec status (as designed):** DARK(`ELAI_CONFORMAL_REFUSAL_ENABLE`) (`ELAI-FEATURE-MAP.md:602-605`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:116`). FEATURE-MAP: `WIRED_STATIC` 06-07b; unified single-impl calibration with 3 consumers = I-23 PLANNED; consumed by G-7, H-39, H-53.
- **Crate / module (spec):** `crates/orchestrator/src/conformal_calibration.rs`; τ file `.elai_cc/calibration/tau.txt`; corpus `.elai_cc/calibration/corpus/`; CLI `elai calibration status|update`
- **Purpose of this file:** enough architecture to rebuild formal Mode E refusal. Not a status ledger.

## 1. What it is

**UniCR conformal refusal** is the **only Mode E grounding with a mathematical guarantee**: distribution-free conformal risk control so the refusal rate stays within user-specified **ε** under distribution shift (`arxiv:2509.01455`, T1) (`D-plan/D-16_unicr_conformal_refusal.md:1-8,18-30`). A claim whose **composite** GG confidence falls **below τ** is refused (“I don’t know”). τ is recomputed **nightly**, **L3-only** — the agent cannot bias calibration.

## 2. Why it exists

Prompt-only or ad-hoc “abstain if unsure” has no guarantee. If this organ is missing, Mode E is a vibe. If it is faked (the model sets τ, or raw mean_logprob without the composite), the optimizer tampers with the grade (Everitt current-RF / Cohen Boxed Myopic — same reason D-10 is L3). Real-problem §4.5: strongest math available for “I don’t know”.

## 3. How it works (runtime)

1. **Who calls it.** D-10 `select_grounding_mode` third branch: `claim.intrinsic_signals_below_threshold(ctx.unicr_tau)` wraps `refusal_decision(signals, tau)` (`D-plan/D-16_unicr_conformal_refusal.md:101-105`, `D-plan/D-10_mode_routing_predicate.md:42-43`). `ctx.unicr_tau` loads from `.elai_cc/calibration/tau.txt` at startup.
2. **Calibration tuples.** After claim resolution, `task_runner` **appends** `CalibrationTuple` (L1A data-table only — no LLM judgment) (`:107-111`):
   - `ground_truth_available` = whether a check produced a result
   - `model_confidence` = composite at decision time
3. **`compute_tau(corpus, epsilon)` (UniCR §3.2, deterministic):**
   - Empirical risk `r(τ) = P(refuse | truth_exists) + P(accept | truth_doesnt_exist)` over the corpus.
   - Find **minimal** τ such that `r(τ) ≤ ε`. Binary search over candidates in `[0,1]` with **0.01** grid.
   - `corpus.size() < min_corpus_size` (default **500**) → `InsufficientData`. **No small-sample approximation** (`:72-80,181-182`).
4. **`TauUpdater::update_nightly` (L3).** Load corpus → compute τ → persist `tau.txt` **with timestamp** → emit `CalibrationUpdate`. Agent cannot edit this function (`:83-92,135-143`).
5. **`refusal_decision(signals, tau) -> bool`.** Composite:

   `w1 * mean_logprob + w2 * (-entropy_max) + w3 * novelty`

   Weights from L3-locked TOML. Return **true** (refuse) iff composite **< τ** (`:94-99`).
6. **CLI.** `elai calibration status` prints size, current τ, last update, ε, empirical refusal rate last N sessions; **always exit 0**. `elai calibration update` runs the nightly logic; CI needs commit tag `L3_OVERRIDE:calibration_update`; dev sessions unrestricted (`:113-124`).
7. **The model may generate** claims and (indirectly) raw GG signals. It may **not** write `tau.txt`, change weights, skip refusal_decision, or call an LLM inside calibration.

D-10 is **not parallel-safe** with D-16 (UniCR formalizes Mode E). Parallel-safe with D-17 and D-22 (`:9-14`).

## 4. Types, enums, objects

Copy from `D-plan/D-16_unicr_conformal_refusal.md:59-99`. Do not invent fields.

```
struct CalibrationTuple {
  claim_id: ClaimId,
  ground_truth_available: bool,
  model_confidence: f32,
  claim_type: ClaimType,
  claim_outcome: ClaimOutcome,
  recorded_at: DateTime<Utc>,
}
enum ClaimOutcome { Verified, Falsified, Refused, NoCheck }

struct CalibrationCorpus {
  tuples: Vec<CalibrationTuple>,
  storage_path: PathBuf,
}
  load(path)      // walk path/*.toml
  append(tuple)   // path/{claim_id}.toml
  size() -> usize

fn compute_tau(corpus, epsilon) -> Result<f32, ConformalError>
  // InsufficientData if size < min_corpus_size

struct TauUpdater {
  storage_path: PathBuf,
  epsilon: f32,
  current_tau: f32,
}
fn update_nightly(&mut self) -> Result<f32, ConformalError>

fn refusal_decision(signals: &IntrinsicSignals, tau: f32) -> bool
```

`IntrinsicSignals` from D-3: at least `mean_logprob`, entropy (`entropy_max`), `novelty` (`:26-28,94-99`). Exact D-3 field names beyond those three: use D-3; do not invent extras here.

## 5. Control flow / state machine

**Owner:** L3 `TauUpdater` for τ; orchestrator `refusal_decision` at Mode E selection; task_runner for corpus growth.

```
startup: unicr_tau ← tau.txt
claim emit → select_grounding_mode
  if A/D unavailable AND refusal_decision(signals, tau) → Mode E Refuse
  else … D-10 remainder (B or E-no-alt)
post-resolution → append CalibrationTuple   // no LLM
nightly / `elai calibration update` → compute_tau → tau.txt + CalibrationUpdate
```

**Forced:** size < 500 → do not invent τ. `tau.txt` written **only** by TauUpdater; LockedKeysAuditor protects (`:186-187`).

Live `locked_keys_registry.toml:109-123` has the D-16 lock **commented as bootstrap seed**. Rebuild: the lock must be **active**, not a comment.

## 6. Grounding, receipts, gates

Receipts: corpus TOMLs, `tau.txt`+timestamp, `RefusalDecision` (composite, τ, refused bool), `CalibrationUpdate`. Formal guarantee is **empirical risk ≤ ε** on the calibration construction; D-22 later checks shift.

**Never allowed as a confidence signal:** raw mean_logprob alone (`:184-185`); LLM “I’m X% sure”; agent-edited τ; approximating τ on N<500.

Prompt-only / calibration-only cousins **lack the formal guarantee** (`:30-31`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| ε default | **0.1** | `[conformal].epsilon` |
| min_corpus_size | **500** | compute_tau |
| τ search grid | 0.01 on [0,1] | compute_tau |
| composite weights default | mean_logprob **0.4**, entropy **−0.3**, novelty **0.3** | L3 TOML |
| integration corpus | 600 tuples | assert rate in [τ−ε, τ+ε] |
| SCHEMA_VERSION (plan) | 61 → 62 | SSB |
| unit tests | ≥12 | acceptance |
| D-22 later | in/near/far-OOD rates in [τ−ε, τ+ε] | not this organ |
| D-30 | bootstrap until N=500 | deferred |

**Overflow:** N≥500 required; below that InsufficientData, not a guessed τ.

## 8. Roles and neighbors

Consumes D-3 `IntrinsicSignals`, D-10 Mode E slot, D-9 locked keys, SsbWriter. Produces τ for D-10, `RefusalDecision` for D-17 SLA, corpus for D-22 / D-30. FEATURE-MAP consumers G-7, H-39, H-53 share **one** impl (I-23 PLANNED) — do not fork a second calibrator (`ELAI-FEATURE-MAP.md:602-605`).

**Must not fork:** per-family τ (deferred to D-30 if needed); agent-writable τ; prompt abstention as Mode E.

## 9. Config / flags / defaults

- **`ELAI_CONFORMAL_REFUSAL_ENABLE`** — opt-in (`ELAI-FEATURE-MAP.md:602`, `wire-dark-allowlist.toml:882-887`). Copy this flag.
- **L3-locked `[conformal]`:**

```toml
[conformal]
epsilon = 0.1
min_corpus_size = 500
composite_weights = { mean_logprob = 0.4, entropy = -0.3, novelty = 0.3 }
tau_storage_path = ".elai_cc/calibration/tau.txt"
corpus_storage_path = ".elai_cc/calibration/corpus/"
```

(`D-plan/D-16_unicr_conformal_refusal.md:133-138`)

- L3 lock symbols: `TauUpdater`, `compute_tau`, `tau.txt` (`:245-246`, `:90-92`).
- CLI override tag: `L3_OVERRIDE:calibration_update`.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `conformal_calibration.rs` | tuples, corpus, tau, refusal_decision |
| `cli/src/calibration_status.rs` | status + update |
| `cli/src/commands.rs` | SlashCommand::CalibrationStatus / CalibrationUpdate |
| `grounding_mode.rs` | `intrinsic_signals_below_threshold` |
| `task_runner.rs` | post-claim corpus append |
| `elai_cc.toml` | `[conformal]` |
| `scripts/locked_keys_registry.toml` | TauUpdater L3 |
| `ssb.rs` 61→62 | RefusalDecision, CalibrationUpdate |
| `orchestrator/tests/conformal_integration.rs` | 600-tuple |
| papers | unicr-2509.01455, uncertainty-outside-parametric-2311.09731, rewarding-doubt-2503.02623 |

## 11. Journal / SSB events

Plan SCHEMA_VERSION **61 → 62**:

```
RefusalDecision { claim_id: ClaimId, composite_confidence: f64, tau: f64, refused: bool }
CalibrationUpdate { tuple_count: u32, new_tau: f64, old_tau: f64, epsilon: f64, audit_id: Uuid }
```

(`:126-131`). D-SUMMARY: `REFUSAL_DECISION`, `CALIBRATION_UPDATE` (`D-plan/D-SUMMARY_INDEX.md:118`). Every nightly update **must** journal (`:183`).

## 12. Tests and observables

| Case | Assert |
|---|---|
| known-split synthetic corpus | τ matches hand-computed |
| size < 500 | InsufficientData |
| refusal_decision table | ≥8 scenarios |
| update_nightly identical corpus | same τ (idempotent) |
| 600-tuple integration | empirical refusal in [τ−ε, τ+ε] |
| observable | CalibrationUpdate + tau.txt + ≥600 files; later low-conf claim RefusalDecision refused=true |
| cloud A | low-confidence → Mode E + refused=true |
| cloud B | high-confidence → **not** Mode E; no RefusalDecision on that branch |
| cloud C | `elai calibration update` with L3_OVERRIDE → CalibrationUpdate + tau.txt |

≥12 tests. Three paper interfaces. Decision doc: ε=0.1; N≥500 finite-sample; composite vs raw logprob (`:152-159`).

## 13. Rebuild contract

Another harness has **this** organ iff:

- Calibration corpus of `(claim, ground_truth_available, model_confidence)` tuples on disk.
- `compute_tau` implements r(τ) as specified, min τ on a 0.01 grid, **InsufficientData** below 500.
- `refusal_decision` uses the **weighted composite**, refuse iff composite < τ.
- τ updated only by L3 `TauUpdater` / tagged CLI; agent cannot edit.
- D-10 Mode E branch consumes `unicr_tau`.
- `RefusalDecision` + `CalibrationUpdate` SSB.
- Default ε=0.1, weights 0.4 / −0.3 / 0.3.
- `ELAI_CONFORMAL_REFUSAL_ENABLE` default-OFF isolation allowed; when off, do not pretend UniCR held.

A prompt “say I don’t know sometimes” is a cousin.

## 14. Non-goals / banned designs

- D-22 distribution-shift benchmark (gates this).
- D-30 generic-corpus bootstrap (deferred until N=500).
- Multi-domain / per-family τ (deferred) (`:254-259`).
- Small-sample τ approximation.
- Raw mean_logprob as the refusal score.
- Agent-writable `tau.txt`.

## 15. Source citations

Primary: `D-plan/D-16_unicr_conformal_refusal.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:116-120`. Feature map: `ELAI-FEATURE-MAP.md:602-605`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:237-238`. Exhaustive: D0-020, D0-051, D003-D16-E001..E031. Live: `locked_keys_registry.toml:109-123` (commented seed), `wire-dark-allowlist.toml:882-887`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact `IntrinsicSignals` field types/ranges and how mean_logprob is normalized to mix with entropy/novelty.
- Weight key `entropy = -0.3` vs formula already multiplying `(-entropy_max)` — double negation risk.
- `P(·)` empirical estimator (Laplace? raw frequency?) and how “accept” is defined vs D-10 B/A/D.
- Behavior of `select_grounding_mode` when `intrinsic_signals` is `None` (D10-N03) **and** UniCR is wired.
- Cloud B “no RefusalDecision in that branch” vs always-append corpus tuples.
- Last-N sessions for CLI status: N unspecified.
- Nightly cron owner / host schedule.
- Live D-16 lock still commented.
- I-23 “single impl, three consumers” vs this module’s surface.
- Schema 62 vs live schema already ≫ 62.
