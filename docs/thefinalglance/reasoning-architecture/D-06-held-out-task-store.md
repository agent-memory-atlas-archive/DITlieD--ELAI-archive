# Time-Windowed Held-Out Task Store (D-6)

- **ELAI ids:** D-6 / Phase 4H.VIII part 1 / companion §3.4 / J-11 amendments S-2 + S-8c
- **Spec status (as designed):** PLANNED. Feature-map tags DARK (`ELAI_HELD_OUT_ENABLE`; `sample_held_out_task` test-only) (`ELAI-FEATURE-MAP.md:1682-1685`).
- **Crate / module (spec):** `crates/orchestrator/src/held_out.rs`; layout `.elai_cc/held-out/{date}/{task-id}.toml`
- **Purpose of this file:** enough architecture to rebuild the dated held-out store, 9-gram contamination filter (13-gram is a later E-54 neighbor), 6-month auto-retire, and promotion regression gate. Not a status ledger.

## 1. What it is

The **held-out task store** is a date-tagged set of evaluation tasks at `.elai_cc/held-out/{date}/{task-id}.toml`. Ingest runs a **9-gram** Jaccard overlap filter (LiveCodeBench, arxiv:2403.07974). Tasks older than **6 months** that pass uniformly (≥0.95 over ≥20 evals) auto-retire as probable contamination. Template promotion is blocked if held-out pass-rate regresses by more than **k=5%**. BAI (D-5) calls this gate before Candidate→Promoted (`D-6_held_out_task_store.md:3-7`, `D-SUMMARY_INDEX.md:52-56`).

## 2. Why it exists

Frontier models leak even “novel” benches (ARC Prize 2025: structure alone is enough). 9-gram overlap caught contamination on GPT-4o, Claude, DeepSeek, Codestral. Soft contamination (arxiv:2602.12413) remains after 9-gram; time-window + semantic fingerprint are the extra legs. If this organ is missing, BAI promotes on in-sample tasks the model has memorized. If it is faked (the authoring model picks the held-out set), the model evaluates its own modification (`D-6_held_out_task_store.md:17-37`).

## 3. How it works (runtime)

1. **Who calls it.** D-5 `BaiPromotionEngine` calls `held_out_regression` / `gate_promotion` before promotion. Auto-refresh walks the store on a clock tick. Ingest writes new toml files (`D-6_held_out_task_store.md:122-129`, `:210-211`).
2. **Load (deterministic).** `HeldOutStore::load` walks `root/{date}/*.toml`, indexes by `TaskId` and `NaiveDate` (`:87-91`).
3. **Append (deterministic).** Write `root/{date}/{task_id}.toml`; update index. **No file delete** — retire is a status rewrite (`:92-93`, `:215`).
4. **N-gram filter (deterministic).** `extract_ngrams(text, n)`: whitespace+punctuation split, lowercase, sliding window. `ngram_overlap` = Jaccard `|A∩B|/|A∪B|` on **9-gram** sets. `detect_ngram_contamination` blake3s each n-gram against `.elai_cc/held-out/ngram-bloom.bin`; Jaccard **> 0.3** ≈ contamination → `RetireReason::NgramContamination`. Empty bloom → **skip** n-gram check (deferred corpus) (`:99-109`, `:154-159`).
5. **Soft-contamination leg (specified, not a second n).** Time-window **and** rolling embedding similarity to recent ingest dates (`:28-31`). Structural-prior randomization is the ARC defense (`:36-37`).
6. **Auto-retire (deterministic, SeededClock).** For each task: age > `ttl_months` (6) **AND** pass-rate > 0.95 **AND** evaluations ≥ 20 → `UniformPassSuspicion`. Rewrite toml `HeldOutStatus::AutoRetired`; emit `HeldOutAutoRetire` (`:111-120`).
7. **Promotion gate (deterministic).** `held_out_regression` = pass-rate delta on the **shared** held-out subset (positive = improvement). `gate_promotion`: `allow = (delta >= -regression_threshold)` with threshold **5.0** percent. Emit `HeldOutPromotionGate`. If `allowed=false`, **no** `TemplatePromoted` (`:122-129`, `:279-281`).
8. **Refresh journal.** Ingest/retire batches emit `HeldOutRefresh { date, action, task_count_added, task_count_retired }` (`:131-133`).
9. **Fixture hygiene (J-11 S-8c).** When seeding from real repos, **destroy and re-init `.git`** in the fixture so reflog / loose objects cannot reconstruct the reference implementation. Fixture-build time, not eval time. Runtime leakage is H-47 (`:327-337`).
10. **Fail-closed.** Empty n-gram input → empty set, no panic. Auto-retire **requires** `min_evaluations_for_retire`. Thresholds from toml, not constructor literals (`:216-219`).

The model may generate candidate templates. It may **not** choose held-out tasks, mark them Active, or waive the 5% gate. Promotion still needs D-3 ≥2 verifiers **and** this gate.

**9 → 13-gram (do not collapse).** D-6 ships **n=9** (LiveCodeBench default; decision (b) explicitly rejects 7 and 13 for this organ) (`:184-185`). Feature-map / E-54 later raise the **public-row** filter to **13-gram** (`ELAI-FEATURE-MAP.md:1682-1683`, `:2792-2793`). Rebuild D-6 as 9 with a config key `ngram_size`; do not silently ship 13 as D-6.

## 4. Types, enums, objects

Copy from `D-6_held_out_task_store.md:70-129`. Do not invent fields.

```
struct HeldOutTask {
  task_id: TaskId,
  submission_date: NaiveDate,
  domain: Domain,                         // D-1 Domain
  problem_statement: BoundedString<4096>,
  expected_outcome: ExpectedOutcome,
  source: TaskSource,
  content_hash: Blake3Hash,
  status: HeldOutStatus,
}
struct TaskId(Uuid)

enum ExpectedOutcome {
  Code { test_cmd: String },
  Math { sympy_expr: String },
  Prose { rubric: BoundedString<1024> },
  Planning { final_state: BoundedString<2048> },
  KnowledgeQa { reference_answer: BoundedString<2048> },
}

enum TaskSource {
  GitHubIssue { repo: String, issue_num: u32, fetched_at: DateTime<Utc> },
  ArxivRepro { arxiv_id: String, fetched_at: DateTime<Utc> },
  Synthetic { generator_version: String },
  HumanCurated { curator: String },
}

enum HeldOutStatus {
  Active,
  AutoRetired { reason: RetireReason },
  ManuallyExcluded,
}
enum RetireReason { UniformPassSuspicion, NgramContamination, SemanticContamination, AgeExceedsTtl }

struct HeldOutStore {
  root_path: PathBuf,
  index: HashMap<TaskId, HeldOutTask>,
  by_date: BTreeMap<NaiveDate, Vec<TaskId>>,
}
  load(root), append(task), retire(task_id, reason),
  query(domain, since) -> active only

struct AutoRefreshPolicy {
  ttl_months: u32,                 // 6
  uniform_pass_threshold: f32,     // 0.95
  min_evaluations_for_retire: u32, // 20
}

struct GateResult { allow: bool, reason: String }  // reason includes delta
```

**Toml `[held_out]` L3-LOCKED** (`:140-145`):

```
root_path = ".elai_cc/held-out"
ttl_months = 6
uniform_pass_threshold = 0.95
min_evaluations_for_retire = 20
regression_threshold_pct = 5.0
ngram_size = 9
ngram_jaccard_threshold = 0.3
```

**SSB SCHEMA 51→52** (`:131-137`):

```
HeldOutRefresh { date: String, action: String, task_count_added: u32, task_count_retired: u32 }
HeldOutAutoRetire { task_id: TaskId, reason: String, age_days: u32, uniform_pass_rate: f64 }
HeldOutPromotionGate { template_id: TemplateId, delta: f64, threshold: f64, allowed: bool }
```

**Seed set:** 25 tasks at `.elai_cc/held-out/2026-05-19/` — 5 per family, mixed sources (10 GitHub recency, 10 ARC-AGI-public, 5 hand math/prose) (`:147-152`).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `HeldOutStore` + `AutoRefreshPolicy`. Curators ingest; the model does not.

**`HeldOutStatus`:**

| From | To | Who |
|---|---|---|
| (new) | Active | ingest/append |
| Active | AutoRetired{UniformPassSuspicion} | age>6mo ∧ pass>0.95 ∧ n≥20 |
| Active | AutoRetired{NgramContamination} | Jaccard > 0.3 vs bloom |
| Active | AutoRetired{SemanticContamination} | embedding fingerprint (soft-contam) |
| Active | AutoRetired{AgeExceedsTtl} | named; table tests do **not** retire on age alone |
| Active | ManuallyExcluded | human |
| AutoRetired / ManuallyExcluded | (none named) | files stay |

**Promotion path:**

```
D-5 promote_round
  → held_out_regression(shared active subset)
  → gate_promotion(delta, 5%)
  → if allow: D-5 CI rule may Promoted
  → if !allow: HeldOutPromotionGate allowed=false; no TemplatePromoted
```

**Forced:** `query` returns **Active only** (`:96-97`). Auto-retire never deletes files (`:215`).

**J-11 S-2 compose:** Arbor `B_dev`/`B_test` is a simpler manual split (executor never touches B_test). D-6 **owns** time-window + n-gram + auto-retire; do not restate Arbor as this organ (`:315-325`).

## 6. Grounding, receipts, gates

Receipts: per-task `content_hash`, bloom membership, `GateResult.reason`, three SSB types. Date from `SeededClock`, not wall clock at eval (`:45`, `:205`).

**Never allowed as a confidence signal:** in-sample train accuracy; model claim “this is held-out”; 9-gram skip because bloom is empty treated as **clean forever** (skip is only until the file is populated — still time-window gated) (`:158-159`); agent-selected fixtures with intact `.git` history (`:327-337`).

**Selector:** `query(domain, since)` then shared-subset delta (`:96-97`, `:122-126`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| n-gram n | 9 (config `ngram_size`) | overlap |
| Jaccard τ | 0.3 | contamination |
| TTL | 6 months | auto-retire |
| uniform pass | 0.95 | auto-retire |
| min evals | 20 | auto-retire |
| regression k | 5% | promotion gate |
| body bound | `BoundedString<4096>` | problem |
| seed N | 25 (5×5 families) | bootstrap |
| dir scale rationale | ~10K toml files | decision (a) (`:181-183`) |
| SCHEMA | 51 → 52 | SSB |
| unit tests | ≥12 | orchestrator |
| n-gram table | ≥10 scenarios | (`:270`) |

**13-gram ceiling (neighbor, not D-6 default):** E-54 / eval-corpus public rows (`ELAI-FEATURE-MAP.md:2792-2793`). D-6 toml key exists so n can move **without** a cousin organ.

## 8. Roles and neighbors

Orchestrator-owned. Human curator / paper-ingestor ingest. Agent must not rewrite Active sets.

**Consumes:** `SeededClock`, D-1 `ArchiveChain` audit trail, D-5 engine (consumer of the gate), blake3, C-17 chunker/tokenization (`:204-209`).

**Produces:** store + SCHEMA 52. D-7 consumes **per-family** pass-rates (tasks are tagged; they are **not** the canary source) (`:11-12`, `:302`). D-15 NoveltyClass uses DKG, not this store (`:303-304`). D-22 uses the store as one of three calibration distributions (`:311-313`).

**Must not fork:** a hidden eval set the executor can `git reflog`; deleting toml on retire; n=13 silently replacing n=9 inside D-6.

**Not parallel-safe with D-4** (both touch archive / held-out pass-rate) (`:12-13`).

## 9. Config / flags / defaults

`[held_out]` keys in section 4, L3-LOCKED (`:145`). Defaults applied when absent (`:273`).

Feature-map `ELAI_HELD_OUT_ENABLE` is not in the D-6 session file (section 16). Bloom precompute is DEFERRED (`:154-159`, `:307`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `orchestrator/src/held_out.rs` | types, store, n-gram, policy, gate (`:70-129`) |
| `bai.rs` `gate_promotion` call | (`:292`) |
| `ssb.rs` SCHEMA 52 | 3 variants (`:290-291`) |
| seed toml | `.elai_cc/held-out/2026-05-19/*.toml` ×25 (`:295`) |
| bloom | `.elai_cc/held-out/ngram-bloom.bin` (may be empty) (`:107-109`) |
| integration | `orchestrator/tests/held_out_integration.rs` |
| papers | LiveCodeBench, Soft Contamination, ARC Prize 2025 (`:60-66`) |

D-1 `ArchiveEntry` is template-shaped; D-6 says auto-retire events recorded as `ArchiveEntry` with **`task_id` in payload (not template_id)** (`:43-44`). Rebuild: extend payload, do not overload `template_id`.

## 11. Journal / SSB events

Three variants (section 4). Observable (`:220-226`): seed 25; simulate 12-month aging all uniformly passing → **≥20** `HeldOutAutoRetire` `UniformPassSuspicion`; **0** retire for tasks younger than 6 months; files remain, status updated.

Cloud C: new arxiv repro ingest → `HeldOutRefresh` + file (`:282-283`).

## 12. Tests and observables

Named unit tests (`:161-171`):

| Id | Assert |
|---|---|
| T-store | load/append/retire tmpdir round-trip |
| T-grams | `extract_ngrams("a b c d e", 3)` = `{a b c, b c d, c d e}` |
| T-jac | identical → 1.0; disjoint → 0.0 |
| T-contam | synthetic bloom overlapping vs disjoint |
| T-retire-7-100-25 | 7mo, 100%, 25 evals → retire |
| T-keep-7-80 | 7mo, 80% → keep |
| T-keep-5-100 | 5mo, 100% → keep |
| T-keep-7-100-10 | 7mo, 100%, 10 evals → keep |
| T-gate-3 | Δ=-3%, k=5% → allow |
| T-gate-7 | Δ=-7% → block |

Integration (`:173-178`): 50 fake tasks; 12-month spans; expected retire set; gate on baseline vs candidate.

Cloud A: mocked clock 12-month aging (`:277-278`). Cloud B: 7% held-out regression → `allowed=false`, no `TemplatePromoted` (`:279-281`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Per-task dated toml under `.elai_cc/held-out/{date}/{id}.toml`.
- 9-gram Jaccard filter, τ=0.3, empty-bloom skip (not panic, not “clean forever”).
- 6-month TTL **and** ≥0.95 pass **and** ≥20 evals → AutoRetired, files kept.
- Promotion blocked on held-out Δ < −5% (`allow = delta >= -k`).
- Five `ExpectedOutcome` families; query Active only.
- Fixture `.git` destroyed/re-inited at seed time.
- D-5 cannot Promoted when the gate says no.
- Model does not curate or waive the set.
- Config `ngram_size=9`; 13-gram is E-54, not a silent D-6 default.

A random eval split, Arbor-only B_test, or “the model promises it did not train on this” is a cousin, not this organ.

## 14. Non-goals / banned designs

- D-7 canary (consumes per-family rates; different source) (`:302`).
- D-15 NoveltyClass / DKG near-dup (`:303-304`).
- D-20 ARC-AGI-2 probe (`:305-306`).
- Training-corpus bloom population this session (`:307`).
- Deleting task files (`:215`).
- Auto-retire without `min_evaluations` (`:217-218`).
- Hardcoded gate k (`:219`).
- n=13 as D-6 default (decision b) (`:184-185`).
- Leaving `.git` history in fixtures (J-11 S-8c) (`:327-337`).
- Restating Arbor B_dev/B_test as D-6 (`:315-325`).
- Model-chosen held-out set.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-6_held_out_task_store.md` (full file, including J-11 amendments `:315-337`). Index: `D-SUMMARY_INDEX.md:52-56`, `:274`. Feature map: `ELAI-FEATURE-MAP.md:1682-1685`, `:2792-2793` (13-gram neighbor). Companion (not in this tree): §3.4, §1.16 as cited `:49-50`. Papers: 2403.07974, 2602.12413, 2601.10904. Arbor prior-art pointers in-file: `src/coordinator/tools/git_ops.py:374-446`, `prompts.py:442-450`. DeNovoSWE `2606.10728` §3.3 S3.SS3.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `EvalHistory` type (`:115-116`, `:123-124`).
- Semantic-fingerprint embedding model / threshold (`:28-31`) vs `RetireReason::SemanticContamination` with no detector function in Tasks.
- `AgeExceedsTtl` vs table that keeps 7-month 80% and 5-month 100% — age-only retire never specified as a standalone path (`:83` vs `:167-170`).
- D-1 `Domain` (4 values) vs `ExpectedOutcome` (5 families) vs “5 per family” seed (`D-1 Domain` vs `:75-78`, `:148`).
- `ArchiveEntry` with `task_id` payload vs D-1 struct (`:43-44`).
- `HashMap` vs D-1 `FxHashMap` (`:87`).
- Bloom empty: skip vs treat as 0 overlap (`:158-159`).
- `ELAI_HELD_OUT_ENABLE` (`ELAI-FEATURE-MAP.md:1684`).
- `unicode-segmentation` vs simple split (`:100-103`).
- Soft-contam “rolling embedding similarity” vs D-15 DKG (`:28-31` vs `:303-304`).
- J-11 “Difficulty-banded inclusion floor” named in feature-map, not in D-6 body (`ELAI-FEATURE-MAP.md:1685`).
