# BAI Thompson Promotion (D-5)

- **ELAI ids:** D-5 / Phase 4H.VII / companion §3.7
- **Spec status (as designed):** PLANNED. Feature-map tags DARK(`ELAI_BAI_ENABLE`, `ELAI_BAI_MODE`); `BaiPromotionDecision` SSB noted 06-07 (`ELAI-FEATURE-MAP.md:1834-1837`). Those env names are **not** in the D-5 session file (section 16).
- **Crate / module (spec):** `crates/orchestrator/src/bai.rs`; after-eval hook in `task_runner.rs`
- **Purpose of this file:** enough architecture to rebuild Beta-posterior Thompson sampling, Successive Rejects, LUCB1, and staged Candidate→Promoted / Superseded retire. Not a status ledger.

## 1. What it is

The **BAI promotion engine** is Best-Arm Identification over `ReasoningTemplate` arms. Each candidate has an `ArmPosterior { template_id, pass_count, fail_count }` with Beta(`pass+1`, `fail+1`). Thompson sampling draws θ_i and sends the max arm to the next staged eval (10 → 50 → 200). Successive Rejects eliminates losers; LUCB1 chooses the next pull. This replaces Phase 4H MSI sigmoid Boltzmann as the **gate** (`D-5_bai_thompson_promotion.md:3-7`, `:25-38`, `D-SUMMARY_INDEX.md:46-50`).

## 2. Why it exists

MSI sigmoid Boltzmann is a heuristic with no published regret bound. Audibert COLT 2010 and variance-dependent BAI (arxiv:2106.10417) are. If this organ is missing, templates promote on a vibe score. If it is faked (the authoring model ranks its own arms), the model evaluates its own modification. **Pass/fail is D-3 `VerifierConsensusPass` / Reject — binary, ≥2 verifiers, never a self-grade** (`D-5_bai_thompson_promotion.md:18-23`, `:182-187`, `:163-165`).

## 3. How it works (runtime)

1. **Who calls it.** `task_runner.rs` after-eval hook: every D-3 Pass/Reject → `BaiPromotionEngine::ingest_eval_result(template_id, passed)` (`D-5_bai_thompson_promotion.md:109`, `:188-189`).
2. **Posterior update (deterministic given `passed`).** Immutable `ArmPosterior::update(passed)` increments pass or fail; `last_updated` from clock. `alpha = pass+1`, `beta = fail+1`, `mean = alpha/(alpha+beta)` (`:73-83`).
3. **Credible interval.** `credible_interval(α=0.05) -> (lower, upper)` via `statrs` Beta or hand-rolled incomplete-beta `<40 LOC`. `(0,0)` → Beta(1,1) → `(0, 1)`, not NaN (`:77-79`, `:196-197`).
4. **Thompson select (seeded stochastic).** For each arm sample `θ_i ~ Beta(α_i, β_i)` via gamma-ratio `G_a/(G_a+G_b)` on **`SeededRng` only** (never `thread_rng`). Return `TemplateId` of max θ; `None` if empty (`:85-91`, `:193`).
5. **Staged eval.** Promote-to-next-stage sequence **10 → 50 → 200** pulls (`PromotionStage { Stage10, Stage50, Stage200 }`). Populate D-1 `StagedEvalResult.stage_*_pass/total` (`:31-32`, `:41-42`, `:117`).
6. **Promotion rule (deterministic given posterior + median).** Candidate → Promoted when `credible_interval(arm, α=0.05).lower_bound > current_promoted_median + δ`. `δ` per `Domain`: code 0.05, math 0.03, prose 0.05, mixed 0.04 (toml; typically 0.02–0.05) (`:33-35`, `:128-132`).
7. **`promote_round(stage)`.** Apply the rule; emit `BaiPromotionDecision` per arm; `ArchiveChain.append` the promotion (`:110-113`).
8. **`retire_round(stage)`.** Successive Rejects on the bottom quartile; retire `RetirementReason::Superseded` via `ArchiveChain.retire` (`:114-116`).
9. **Successive Rejects (Audibert §3).** Round-by-round: eliminate worst empirical mean; remaining get `n_k = budget / (K-k)` pulls; return ordered survivors (`:93-97`).
10. **LUCB1 next pull.** Empirically-best by mean; competitor = highest UCB `mean_i + sqrt(log(1/δ)/(2 n_i))`; **return the competitor** (the arm to pull), not the empirical best (`:99-103`).
11. **Fail-closed.** Empty arms → empty `Vec`, no panic. Agent cannot edit `delta_per_family`. Two runs with same `ELAI_SEED` → byte-identical decisions (`:134-137`, `:194-198`).

The model may author a candidate template body (D-1). It may **not** increment `pass_count`, sample θ, or set `archive_status`. Promotion needs D-3 consensus as the Bernoulli observation.

## 4. Types, enums, objects

Copy from `D-5_bai_thompson_promotion.md:71-132`. Do not invent fields.

```
struct ArmPosterior {
  template_id: TemplateId,
  pass_count: u32,
  fail_count: u32,
  last_updated: DateTime<Utc>,
}
  alpha() = pass_count + 1
  beta()  = fail_count + 1
  mean()  = alpha / (alpha + beta) as f64
  credible_interval(alpha_level: f32) -> (f32, f32)
  update(passed: bool) -> Self   // immutable

enum PromotionStage { Stage10, Stage50, Stage200 }

struct BaiPromotionEngine {
  arms: HashMap<TemplateId, ArmPosterior>,
  promoted: Vec<TemplateId>,
  retired: Vec<TemplateId>,
  eval_budget: u32,
  delta_per_family: HashMap<Domain, f32>,
  rng: SeededRng,
}
  ingest_eval_result(template_id, passed: bool)
  promote_round(stage: PromotionStage) -> Vec<TemplateId>
  retire_round(stage: PromotionStage) -> Vec<TemplateId>

fn sample_beta(rng: &mut SeededRng, alpha: f32, beta: f32) -> f32
fn thompson_select(arms: &[ArmPosterior], rng: &mut SeededRng) -> Option<TemplateId>
fn successive_rejects(arms: Vec<ArmPosterior>, eval_budget: u32) -> Vec<TemplateId>
fn lucb1_next_pull(arms: &[ArmPosterior], delta: f32) -> Option<TemplateId>
```

**Toml `[bai]` L3-LOCKED** (`:128-132`):

```
eval_budget = 200
ci_alpha = 0.05
delta_code = 0.05
delta_math = 0.03
delta_prose = 0.05
delta_mixed = 0.04
```

**SSB SCHEMA 50→51** (`:119-126`):

```
ArmPosteriorUpdated { template_id, pass_count, fail_count, mean, ci_lower, ci_upper }
ThompsonSampled { round_id: Uuid, sampled_arms: Vec<(TemplateId, f64)>, selected: TemplateId }
BaiPromotionDecision { template_id, decision: String, stage: String,
                       evidence_pass: u32, evidence_total: u32, audit_id: Uuid }
```

**Optional dep:** `statrs::distribution::Beta`; if absent, ask owner before add, else incomplete-beta <40 LOC (`:78-81`, `:279-280`).

## 5. Control flow / state machine

**Owner of the loop:** `BaiPromotionEngine`. Archive status still obeys D-1: Candidate→Promoted valid; Promoted→Retired (here `Superseded`) valid; Frozen/Retired terminal.

**Stage ladder:** `Stage10 → Stage50 → Stage200` (`:31-32`, `:117`). `eval_budget` default 200 matches the last rung (`:129`).

**Per round:**

```
D-3 verdict ──bool──► ingest_eval_result ──► ArmPosteriorUpdated
promote_round(stage):
  if CI_lower > median_promoted + δ_family  → Candidate→Promoted, append, BaiPromotionDecision
  else keep Candidate (or emit decision string — field is String)
retire_round(stage):
  successive_rejects(bottom quartile) → Retired/Superseded
thompson_select / lucb1_next_pull choose WHO is evaluated next, not the gate
```

**Forced:**

- Bernoulli observation = D-3 consensus, not PPM scalar, not MSI (`:163-165`).
- MSI may remain a **secondary operator-visibility signal**, never the gate (`:168-170`).
- D-4 auto-revert and D-5 promote share `ArchiveChain` locks — **not parallel-safe** (`:12-14`).

**UNCERTAIN:** `BaiPromotionDecision.decision: String` allowed values (`:124`). Whether `Inconclusive` updates fail_count, skip, or neither (`:188` names Pass/Reject only).

## 6. Grounding, receipts, gates

The promotion receipt is **Beta posterior + CI test + D-3 binary**, journaled as the three SSB types plus `ArchiveEntry.staged_eval_result`.

**Never allowed as a confidence signal:** MSI sigmoid as gate; LLM ranking of arms; raw PPM score as `passed`; `thread_rng`; NaN CI on empty counts; agent-edited δ (`:193-198`).

**Regret receipt (integration):** 10 arms rates `[0.9, 0.85, 0.7, 0.65, 0.5, 0.5, 0.4, 0.3, 0.2, 0.1]`, budget 500, 100 trials; cumulative regret ≤ Audibert `O(sqrt(K * budget * log(budget)))` **× 2**; best arm Promoted in ≥85% trials (`:154-161`). D-SUMMARY cloud line is “50-round synthetic noisy-arm; empirical regret within published Audibert bound” (`D-SUMMARY_INDEX.md:50`) — see gaps.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| stages | 10 / 50 / 200 evals | per arm ladder |
| `eval_budget` | 200 | toml default |
| `ci_alpha` | 0.05 | CI |
| δ code/math/prose/mixed | 0.05 / 0.03 / 0.05 / 0.04 | per `Domain` |
| δ typical range | 0.02–0.05 abs pass-rate | companion (`:35`) |
| prior | Beta(1,1) → mean 0.5 | `(0,0)` |
| regret bench | K=10, n=500, 100 trials | integration |
| cloud A | 50-cycle bootstrap | (`:257`) |
| seed | `ELAI_SEED` | process |
| unit tests | ≥15 | orchestrator |
| SCHEMA | 50 → 51 | SSB |

**Overflow:** empty engine returns `[]`. Small sample CI is `(0,1)`, not panic (`:194-197`).

## 8. Roles and neighbors

Orchestrator-owned. No agent role samples θ.

**Consumes:** D-1 archive + `archive_status` + `StagedEvalResult` placeholders; D-3 Pass count; C-15 `SeededRng`; D-6/D-7 later feed extra pass/fail via the same `ingest_eval_result` (`:182-187`, `:290-292`).

**Produces:** posteriors, promotion/retire appends, SCHEMA 51.

**Must not fork:** a second MSI gate that can overrule BAI; unseeded RNG; Gaussian-on-score cousin (decision a: Pass/Reject is Bernoulli) (`:163-165`).

**Coordination:** D-4 revert vs D-5 promote — `ArchiveChain` locks (`:12-14`).

## 9. Config / flags / defaults

`[bai]` keys in section 4, L3-LOCKED (`:132`). Defaults applied when absent (`:254`).

`ELAI_SEED` required for determinism tests (`:134-137`). Feature-map `ELAI_BAI_ENABLE` / `ELAI_BAI_MODE` not in D-5 (section 16).

statrs: confirm with owner (`:80-81`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `orchestrator/src/bai.rs` | all algorithms (`:71-117`) |
| `task_runner.rs` after-eval | (`:271-272`) |
| `ssb.rs` SCHEMA 51 | 3 variants (`:269-270`) |
| `elai_cc.toml [bai]` | (`:273`) |
| regret test | `orchestrator/tests/bai_synthetic_regret.rs` (`:154-161`) |
| contract | `contracts/tests/bai_promotion_contract.rs` (`:276`) |
| papers | `audibert-colt-2010-bai`, `variance-bai-2106.10417` (`:62-67`) |

D-1 anti-stale used `FxHashMap`; D-5 writes `HashMap` (`:106-108`). Rebuild: do not silently switch; gap in section 16.

## 11. Journal / SSB events

Three variants (section 4). `rg` ≥12 hits (`:213-216`).

Observable (`:199-205`): 10-arm seeded bench → ≥100 `ArmPosteriorUpdated`; ≥1 `BaiPromotionDecision` per survived arm; two same-seed runs → identical SSB **diff except timestamps** (`:260-262`).

`ThompsonSampled.sampled_arms` records `(TemplateId, f64)` draws (`:122-123`).

## 12. Tests and observables

Named unit tests (`:139-152`):

| Id | Assert |
|---|---|
| T-mean-prior | (0,0) mean = 0.5 |
| T-mean-10-0 | (10,0) ≈ 0.917 |
| T-mean-0-10 | (0,10) ≈ 0.083 |
| T-ci-sym | (50,50) CI symmetric about 0.5 |
| T-ci-skew | (95,5) skewed high |
| T-sample | `sample_beta` deterministic under seed |
| T-thomp | `thompson_select` deterministic under seed |
| T-sr | 4 arms budget=12 round allocation (4,3,2,1 survivors per Audibert §3) |
| T-lucb | returns competitor, not empirical best, when widths differ |
| T-engine | 5-arm, true 0.9 vs four 0.5 → 0.9 Promoted within budget; seeded |

Integration (`:154-161`): regret ≤ Audibert×2; best arm ≥85%.

Cloud A: 50-cycle synthetic templates, rows + status transitions (`:257-259`). Cloud B: `ELAI_SEED=42` twice, BAI rows byte-identical (`:260-262`).

Adversarial: `thread_rng` / `rand::random` = 0 hits (`:222-225`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `ArmPosterior` Beta(`pass+1`,`fail+1`) with immutable `update(bool)`.
- Thompson sample on `SeededRng`; empty → `None`.
- Promotion gate: CI lower (α=0.05) > median Promoted + per-Domain δ.
- Stages 10/50/200 filling `StagedEvalResult`.
- Successive Rejects + LUCB1 as specified; LUCB returns the **competitor**.
- Pass bit from D-3 consensus, not self-eval / MSI / PPM scalar.
- Same seed → same promotion sequence.
- SCHEMA 51 three events; L3 `[bai]` deltas.
- MSI, if kept, is display-only.

A Boltzmann-over-MSI picker, unseeded Thompson, or “ask the model which template is better” is a cousin, not this organ.

## 14. Non-goals / banned designs

- Hierarchical non-IID BAI — D-29 DEFERRED (`:283-284`).
- Adversarial farming — D-28 (`:285`).
- Reward-model retraining (`:286`).
- `thread_rng` (`:193`).
- Panic on empty `promote_round` (`:194-195`).
- NaN CI on (0,0) (`:196-197`).
- Agent-edited δ (`:198`).
- MSI as the promotion gate (`:168-170`).
- Gaussian posterior on continuous verifier scores (decision a) (`:163-165`).
- Model self-certifying a template it wrote.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md` (full file). Index: `D-SUMMARY_INDEX.md:46-50`, `:273`. Feature map: `ELAI-FEATURE-MAP.md:1834-1837`. Companion (not in this tree): §3.7 lines 296-317, §1.14 as cited `:49-50`. Papers: Audibert COLT 2010, arxiv:2106.10417. D-3 Bernoulli source: `D-3_verification_result_aggregator.md` `Verdict`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- D-SUMMARY “50-round” cloud vs D-5 “500 pulls × 100 trials” vs cloud A “50-cycle” (`D-SUMMARY_INDEX.md:50` vs `:154-161` vs `:257`).
- `BaiPromotionDecision.decision` string enum (`:124`).
- D-3 `Inconclusive` → pass, fail, or skip (`:188`).
- `HashMap` vs D-1 `FxHashMap` ban (`:106` vs D-1).
- `n_i` in LUCB: `pass+fail` or stage total (`:102`).
- `current_promoted_median` over which set (all Promoted? same Domain? same stage?) (`:33-35`).
- `retire_round` “bottom quartile” vs Audibert eliminate-one-per-round (`:114-116` vs `:93-97`).
- T-sr “4,3,2,1 surviving arms per round” vs standard SR leftover counts (`:146-147`).
- Variance-dependent BAI paper ingested but **no variance term in `ArmPosterior`** (`:60-65` vs `:73-74`).
- `ELAI_BAI_ENABLE` / `ELAI_BAI_MODE` (`ELAI-FEATURE-MAP.md:1834`).
- Feature-map “FamilyId(0) hardcoded” is D-29 neighbor, not D-5 (`:1836`).
- Whether Thompson and LUCB both run every round or LUCB only selects pulls while Thompson selects stage promotion (`:85-103` vs `:110-113`).
