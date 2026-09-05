# Cross-Domain Transfer Canary (D-7)

- **ELAI ids:** D-7 / Phase 4H.VIII part 2 / companion §3.5 / UC-3
- **Spec status (as designed):** PLANNED. Feature-map tags PARTIAL (5 families; `Domain::KnowledgeQa` absent; fixtures 8/50) (`ELAI-FEATURE-MAP.md:1686-1687`) — implementation-neighbor.
- **Crate / module (spec):** `crates/orchestrator/src/canary.rs`; extends D-1 `Domain` in `reasoning_template.rs`; gate in `bai.rs`
- **Purpose of this file:** enough architecture to rebuild the five-family NanoFlux canary and the >5% non-target (see gaps) regression block. Not a status ledger.

## 1. What it is

The **cross-domain transfer canary** evaluates every L2 mutation against **all five** task families — code-fix, math-proof, prose-reason, planning, knowledge-QA — even when the candidate targets one family. Promotion is **blocked** if a family pass-rate drops by more than **5%**. This is NanoFlux (arxiv:2509.23252): a math win must not buy a medical/scientific loss (`D-7_cross_domain_canary.md:3-7`, `D-SUMMARY_INDEX.md:58-62`).

## 2. Why it exists

A single-domain verifier will bless a template that hacks the target family and quietly wrecks the others. If this organ is missing, BAI promotes on code-fix alone. If it is faked (the authoring model scores the other families), the model evaluates its own modification. Families and the 5% threshold are L3-locked (`D-7_cross_domain_canary.md:17-20`, `:116-117`).

## 3. How it works (runtime)

1. **Who calls it.** `BaiPromotionEngine.promote_round` invokes `CanaryRunner::run` **before** any promotion (`D-7_cross_domain_canary.md:119-125`, `:169-171`).
2. **Inputs.** `template_id`, `baseline_template` = current Promoted for `target_family`, `candidate_template` = Candidate awaiting promotion, `target_family: CanaryFamily` (`:86-88`, `:124-125`).
3. **Per family (deterministic orchestration, evaluator runs tasks).** Query D-6 `HeldOutStore` for that family; run **both** baseline and candidate; `FamilyEvalResult { pass_count, total_count, pass_rate = pass/total }` (`:88-89`, `:128`).
4. **Deltas.** `candidate_rate - baseline_rate` per family (`:89`).
5. **Decision (deterministic):**
   - `BlockRegression { family, delta }` if **ANY** family `delta < -regression_threshold` (0.05) (`:90`).
   - `InsufficientData { family }` if any family `total_count < min_samples_per_family` (5) (`:91`).
   - else `Allow`.
6. **BAI coupling.** `BlockRegression` → skip promotion, emit `CrossDomainRegression`, candidate stays Candidate. `InsufficientData` → emit row; BAI **defers** until samples accumulate (`:122-123`, `:150-151`, `:183-184`).
7. **Journal.** One `CrossDomainRegression` per blocking family (cloud A: prose_reason Δ=-0.08). `CanaryFamilyPass` per non-regressing family. `CanaryInsufficientData` when short (`:103-110`, `:177-182`).
8. **Persist.** `CanaryReport` written to `workflow/sessions/{id}/canary-reports/` (`:184-185`).
9. **Domain enum (atomic with D-7).** Extend D-1 `{Code, Math, Prose, Mixed}` with `Planning`, `KnowledgeQa`. Existing `Mixed` rows stay valid; **no on-disk migration**; **no rename** of old variants (`:63-69`, `:175`).
10. **Fail-closed.** Missing samples → InsufficientData + SSB, not silent Allow. Threshold from toml (`:173-176`).

The evaluator may execute fixtures. The **candidate-authoring model** does not set `CanaryDecision` and does not pick which families run.

**Header vs body:** objective / D-SUMMARY say block if any **non-target** family regresses >5% (`:5`, `D-SUMMARY_INDEX.md:59`). `run` decision logic says **ANY** family (`:90`). Rebuild both sentences; do not pick a winner (section 16). Cloud A uses a **non-target** prose −8% with code-fix +10%.

## 4. Types, enums, objects

Copy from `D-7_cross_domain_canary.md:63-91`. Do not invent fields.

```
// D-1 Domain AFTER D-7 (existing variants stable)
enum Domain { Code, Math, Prose, Mixed, Planning, KnowledgeQa }

enum CanaryFamily { CodeFix, MathProof, ProseReason, Planning, KnowledgeQa }
  // closed; L3-locked
  // mapping to Domain 1:1 except Code→CodeFix, Math→MathProof
  // (Prose→ProseReason, Planning→Planning, KnowledgeQa→KnowledgeQa implied)

struct FamilyEvalResult {
  family: CanaryFamily,
  pass_count: u32,
  total_count: u32,
  pass_rate: f32,            // pass_count / total_count
}

struct CanaryReport {
  template_id: TemplateId,
  baseline: Vec<FamilyEvalResult>,
  candidate: Vec<FamilyEvalResult>,
  deltas: Vec<(CanaryFamily, f32)>,
  target_family: CanaryFamily,
  decision: CanaryDecision,
}

enum CanaryDecision {
  Allow,
  BlockRegression { family: CanaryFamily, delta: f32 },
  InsufficientData { family: CanaryFamily },
}

struct CanaryRunner {
  held_out: Arc<HeldOutStore>,
  evaluator: Arc<dyn Evaluator>,
  regression_threshold: f32,      // 0.05
  min_samples_per_family: u32,    // 5
}
  fn run(template_id, baseline_template, candidate_template, target_family) -> CanaryReport
```

**Toml `[canary]` L3-LOCKED** (`:113-117`):

```
regression_threshold = 0.05
min_samples_per_family = 5
enabled_families = ["code_fix", "math_proof", "prose_reason", "planning", "knowledge_qa"]
```

**SSB SCHEMA 52→53** (`:103-110`):

```
CrossDomainRegression { template_id, family: String, baseline_pass_rate, candidate_pass_rate,
                        delta, threshold, audit_id }
CanaryFamilyPass { template_id, family, pass_rate, sample_count }
CanaryInsufficientData { template_id, family, sample_count, required }
```

**Fixture bootstrap (into D-6 store)** (`:93-101`, public benches):

| Family | Seed sketch |
|---|---|
| code-fix | 10 exercism rust + 5 leetcode-public |
| math-proof | 10 PRMBench math (D-19 ingests) |
| prose-reason | 10 HotpotQA + 5 AdversarialQA |
| planning | 10 PlanBench |
| knowledge-QA | 10 PopQA + 5 TriviaQA |

Acceptance also says **50 fixture tasks across 5 families** (`:232`). Deterministic-check expects **≥10 per family** under `.elai_cc/held-out/2026-05-19/` (`:209-213`). UC-3: five-family count is a **guess**; instrument (`:22-23`, `:146-147`).

## 5. Control flow / state machine

**Owner of the loop:** `CanaryRunner` inside D-5 `promote_round`. Not the model.

```
promote_round
  → CanaryRunner.run(all 5 families vs Promoted baseline)
  → Allow            → D-5 CI rule may Promoted
  → BlockRegression  → skip; CrossDomainRegression; stay Candidate
  → InsufficientData → skip/defer; CanaryInsufficientData
```

**Forced:** every L2 mutation hits **all** families, not only `target_family` (`:3-5`). `enabled_families` can theoretically subset; stub-ban still requires min-samples **per (enabled) family** (`:113-116`, `:173`).

**Domain matchers:** all existing `match Domain` must be exhaustive after the two new variants (`:67`).

## 6. Grounding, receipts, gates

Evidence is per-family pass counts on D-6 Active tasks plus `CanaryReport` on disk plus SSB.

**Never allowed as a confidence signal:** target-family PPM/MSI alone; LLM “I didn’t regress prose”; skipping a family because fixtures are annoying; treating InsufficientData as Allow (`:173-174`).

**Promotion still needs D-3 ≥2 verifiers** (Bernoulli for BAI) **and** this canary **and** D-6 held-out Δ gate. The canary is not a self-grade.

Cloud A receipt: `family=prose_reason`, `delta=-0.08`, zero `TemplatePromoted` (`:177-182`, `:237-238`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| families | 5 (UC-3: 3 too few, 12 too many) | closed enum |
| regression k | 0.05 (5%) | per family delta |
| fire example | −0.06 blocks; −0.04 does not | unit (`:132-133`) |
| cloud A | prose −8% (−0.08) | (`:237-238`) |
| min samples | 5 / family | InsufficientData |
| noise rationale | ≥10 samples → ≥3% std | decision (b) (`:148-149`) |
| fixtures | ≥10/family; acceptance 50 total | (`:209-213`, `:232`) |
| SCHEMA | 52 → 53 | SSB |
| unit tests | ≥10 | orchestrator |

**Overflow / sparse:** <5 samples → InsufficientData, promotion deferred (`:150-151`).

## 8. Roles and neighbors

Orchestrator + BAI. Evaluator trait runs fixtures (mock in tests) (`:168`).

**Consumes:** D-6 `HeldOutStore.query`; D-5 engine (integration); D-1 `Domain` (extended here) (`:164-168`).

**Produces:** `CanaryDecision` into BAI; SCHEMA 53; D-1 `StagedEvalResult.cross_domain_deltas` is the D-1 placeholder this organ fills conceptually (D-1 named D-7 as populator).

**Must not fork:** a target-only eval; renaming `Code`/`Math`; a sixth family without L3.

**Not parallel-safe with D-4** (Δρ source) (`:12-13`). Parallel-safe with D-5 consume-as-gate and D-8 (`:11-12`). D-24 is the later 100-mutation suite (`:261`).

## 9. Config / flags / defaults

`[canary]` keys in section 4, L3-LOCKED (`:117`). Defaults applied when absent (`:234`).

No `ELAI_CANARY_*` flag in the D-7 file. Feature-map PARTIAL/`KnowledgeQa` absent is neighbor, not a license to ship a 4-family cousin (`ELAI-FEATURE-MAP.md:1686-1687`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `canary.rs` | families, report, runner (`:71-91`) |
| `reasoning_template.rs` Domain +2 | (`:63-69`, `:248-249`) |
| `bai.rs` `CanaryRunner` | (`:119-125`, `:252`) |
| `ssb.rs` SCHEMA 53 | (`:250-251`) |
| fixtures | `.elai_cc/held-out/2026-05-19/canary-{family}-*.toml` (`:255`) |
| reports | `workflow/sessions/{id}/canary-reports/` (`:184-185`) |
| paper | `nanoflux-2509.23252-interface.md` |

## 11. Journal / SSB events

Three variants (section 4). `rg` ≥12 hits (`:193-196`).

Observable (`:177-185`): code-fix +10% / prose −8% → 1 `CrossDomainRegression` prose_reason Δ=-0.08; **4** `CanaryFamilyPass` for the others; 0 `TemplatePromoted`; archive still Candidate.

Cloud B: +10% code-fix / +5% math / neutral → **5** `CanaryFamilyPass` + `TemplatePromoted` (`:239-240`) — implies canary Allow **and** D-5 CI rule also fire; do not treat canary Allow as sufficient alone if D-3/D-5 fail.

## 12. Tests and observables

Named unit tests (`:127-133`):

| Id | Assert |
|---|---|
| T-rate | `pass_rate = passes/total` |
| T-table | ≥10 scenarios Allow / BlockRegression / InsufficientData |
| T-map | CanaryFamily ↔ Domain bijection |
| T-thr | Δ=-0.06 blocks; Δ=-0.04 does not (k=0.05) |

Integration (`:135-143`): mock 50 tasks / 5 families. (A) +10% code-fix, no other regression → Allow. (B) +10% code-fix, prose −8% → BlockRegression. (C) planning only 3 tasks → InsufficientData.

Cloud A/B/C match those three (`:237-241`).

Done-gates: Domain +Planning +KnowledgeQa; ≥10 orchestrator tests; 50 fixtures (`:226-243`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Five `CanaryFamily` values; every L2 mutation evaluated on all of them.
- `Domain` gains `Planning` and `KnowledgeQa` without renaming Code/Math/Prose/Mixed.
- Block when a family delta `< -0.05` (and document whether target is included).
- `<5` samples → InsufficientData + SSB, not Allow.
- BAI cannot Promoted on BlockRegression; candidate stays Candidate.
- D-6 query is the fixture source; reports land on disk.
- Thresholds from L3 toml.
- Model that wrote the candidate does not set `CanaryDecision`.

A single-family unit test, a prompt “don’t regress other skills,” or a 4-value Domain with Mixed standing in for QA is a cousin, not this organ.

## 14. Non-goals / banned designs

- D-24 100-mutation validation suite (`:261`).
- D-15 NoveltyClass (`:262`).
- D-18 ObservableEffect (`:263`).
- Renaming existing `Domain` variants (`:175`).
- Hardcoded 5% in `CanaryRunner::new` (`:176`).
- Silent skip of a family with too few samples (`:173-174`).
- Target-only evaluation (`:3-5`).
- Model self-scoring other families.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-7_cross_domain_canary.md` (full file). Index: `D-SUMMARY_INDEX.md:58-62`, `:275`. Feature map: `ELAI-FEATURE-MAP.md:1686-1687`. Companion (not in this tree): §3.5 lines 268-276, UC-3 line 433 as cited `:47-48`. Paper: NanoFlux 2509.23252. D-1 Domain four-value start: `D-1_reasoning_template_schema.md:96-97`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- **ANY family** vs **non-target only** (`:5`, `D-SUMMARY_INDEX.md:59` vs `:90`).
- `Domain::Mixed` has no `CanaryFamily` partner; bijection test vs “1:1 except Code/Math” (`:74-75`, `:131`).
- Whether `CanaryDecision` can name **multiple** blocking families or only the first (`:81` single `family` field vs “ANY family”).
- Allow vs InsufficientData priority if one family is short and another regresses (`:90-91`).
- Cloud B `TemplatePromoted` vs D-5 CI + D-3 + D-6 gates still required (`:239-240`).
- Fixture counts: bootstrap table (~15+10+15+10+15) vs acceptance 50 vs check ≥10 each (`:93-101`, `:232`, `:209-213`).
- `Evaluator` trait methods (`:84-85`).
- How `HeldOutStore.query(domain)` maps `CanaryFamily` onto D-6 `Domain` / `ExpectedOutcome`.
- `enabled_families` subset vs “ALL 5” hard rule (`:3` vs `:114-116`).
- Feature-map `KnowledgeQa` absent / 8/50 fixtures (`ELAI-FEATURE-MAP.md:1686-1687`) vs rebuild-all-five.
- D-1 `StagedEvalResult.cross_domain_deltas: Vec<(Domain, f32)>` vs `Vec<(CanaryFamily, f32)>` (`D-1:131` vs D-7 `:79`).
