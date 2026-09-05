# D-6: Time-windowed held-out task store + 9-gram contamination filter

Objective: maintain a curated held-out task store at .elai_cc/held-out/{date}/
{task-id}.toml with date-tagged tasks + 9-gram overlap filter (LiveCodeBench
pattern, arxiv:2403.07974, T1); auto-retire uniformly-passing tasks > 6 months
old as probably-contaminated; gate template promotion on held-out NOT
regressing by >k% (suggested k=5 per companion §3.4).
Estimated effort: 2 sessions.
Pre-requisites: D-5 (BAI promotion consumes held-out signal as gate input).
  spec-grounder verifies DONE.
Parallel-safe with: D-7 (cross-domain canary; held-out tasks are tagged per
  family but are NOT the canary's source). NOT parallel-safe with D-4 (Tripwire
  monitors held-out pass-rate; both write to ArchiveChain).

## Context

Companion §3.4 (lines 258-266): LiveCodeBench (arxiv:2403.07974, T1) demonstrates
sliding time-window splits + 9-gram overlap filter detected contamination on
GPT-4O, Claude, DeepSeek, Codestral — a tier-1 hyperscaler-validated baseline.
Apply to ELAI:
- Maintain held-out task set tagged with submission date
- Auto-refresh every N sessions: retire tasks > 6 months old that pass
  uniformly (probably contaminated)
- Pull new tasks from sources with verifiable recency (GitHub issues with
  proven novelty, newly-published arxiv reproducibility tasks)
- Mutation-promotion gated on held-out NOT regressing by >k% (k=5)

Soft Contamination (arxiv:2602.12413, T1) extends: even tasks that pass 9-gram
filter can have semantic contamination if training data covers near-equivalent
problems. Defense: time-window AND semantic-fingerprint (rolling embedding
similarity to recent ingest dates).

Companion §1.16 (lines 143-148): ARC Prize 2025 Tech Report (arxiv:2601.10904,
T1) — "ARC data is well represented in underlying models — enough to make
correct inferences from just the 2D JSON array structure." Even hand-crafted
novelty benchmarks leak. Time-windowed + structural-prior randomization are
the two defenses.

Partial foundation present and usable:
- C-17 (knowledge indexing resilience) adds Kotlin grammar + Heading/Paragraph
  chunking + auto_resume. The chunker is consumed here for task content
  hashing (9-gram extraction reuses tokenization).
- ArchiveChain (D-1) records held-out auto-retire events as ArchiveEntry
  with task_id in payload (not template_id).
- SeededClock (C-15) for date-tagging on ingest.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.4 LiveCodeBench pattern; §1.16 ARC contamination — lines 258-266 + 143-148)
- F:\.ELAI_workflow\workflow\papers\statistical-limits-2510.04399-interface.md (D-1)
- .elai_cc/crates/knowledge/src/symbol_graph/types.rs (DKG; if held-out task
  ingest needs symbol-graph cross-reference for novelty labeling — actually
  D-15 does this; D-6 only does the task store)
- .elai_cc/crates/orchestrator/src/archive_chain.rs (D-1)
- .elai_cc/crates/core/src/clock.rs (SeededClock)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2403.07974 LiveCodeBench (load-bearing contamination-resistance pattern)
- arxiv:2602.12413 Soft Contamination (semantic-fingerprint extension)
- arxiv:2601.10904 ARC Prize 2025 Tech Report (contamination warnings)

Verify workflow/papers/{livecodebench-2403.07974,soft-contamination-2602.12413,
arc-prize-2025-2601.10904}-interface.md.

## Tasks

1. **HeldOutTask types** in new file
   .elai_cc/crates/orchestrator/src/held_out.rs:
   - `struct HeldOutTask { task_id: TaskId, submission_date: NaiveDate,
     domain: Domain, problem_statement: BoundedString<4096>, expected_outcome:
     ExpectedOutcome, source: TaskSource, content_hash: Blake3Hash,
     status: HeldOutStatus }`
   - `enum ExpectedOutcome { Code { test_cmd: String }, Math { sympy_expr:
     String }, Prose { rubric: BoundedString<1024> }, Planning { final_state:
     BoundedString<2048> }, KnowledgeQa { reference_answer: BoundedString<2048> } }`
   - `enum TaskSource { GitHubIssue { repo: String, issue_num: u32, fetched_at:
     DateTime<Utc> }, ArxivRepro { arxiv_id: String, fetched_at: DateTime<Utc> },
     Synthetic { generator_version: String }, HumanCurated { curator: String } }`
   - `enum HeldOutStatus { Active, AutoRetired { reason: RetireReason }, ManuallyExcluded }`
   - `enum RetireReason { UniformPassSuspicion, NgramContamination, SemanticContamination, AgeExceedsTtl }`
   - `struct TaskId(Uuid)` newtype.

2. **HeldOutStore** in held_out.rs:
   - `struct HeldOutStore { root_path: PathBuf, index: HashMap<TaskId, HeldOutTask>,
     by_date: BTreeMap<NaiveDate, Vec<TaskId>> }`
   - `impl HeldOutStore`:
     - `load(root: &Path) -> Self`: walk root/{date}/*.toml; parse each as
       HeldOutTask; index by id + date.
     - `append(task: HeldOutTask)`: write to root/{date}/{task_id}.toml;
       update index.
     - `retire(task_id, reason)`: update status field + rewrite toml file;
       emit HeldOutAutoRetire SSB row.
     - `query(domain: Option<Domain>, since: Option<NaiveDate>) -> Vec<&HeldOutTask>`:
       filter by domain + date; return active tasks only.

3. **N-gram filter** in held_out.rs:
   - `fn extract_ngrams(text: &str, n: usize) -> HashSet<String>`: split on
     whitespace + punctuation (use unicode-segmentation if already in
     workspace; otherwise simple split); sliding window n-gram; lowercase
     normalized.
   - `fn ngram_overlap(text_a: &str, text_b: &str, n: usize) -> f32`:
     |A ∩ B| / |A ∪ B|, jaccard on 9-gram sets.
   - `fn detect_ngram_contamination(task: &HeldOutTask, training_corpus_hashes:
     &Vec<Blake3Hash>) -> bool`: blake3 each n-gram; check against pre-
     computed training-corpus n-gram bloom filter (load from
     .elai_cc/held-out/ngram-bloom.bin); threshold > 0.3 jaccard ≈ contamination.

4. **Auto-refresh policy** in held_out.rs:
   - `struct AutoRefreshPolicy { ttl_months: u32, uniform_pass_threshold: f32,
     min_evaluations_for_retire: u32 }`
   - Defaults: ttl_months=6, uniform_pass_threshold=0.95, min_evaluations=20.
   - `fn check_for_auto_retire(store: &HeldOutStore, eval_history:
     &EvalHistory, clock: &dyn ClockProvider, policy: &AutoRefreshPolicy) ->
     Vec<(TaskId, RetireReason)>`:
     For each task in store: if age > ttl_months AND pass-rate > threshold
     AND evaluations >= min_evaluations → mark for retire with reason
     UniformPassSuspicion.

5. **Held-out promotion gate** in held_out.rs:
   - `fn held_out_regression(template_id: TemplateId, store: &HeldOutStore,
     baseline_eval: &EvalHistory, candidate_eval: &EvalHistory) -> f32`:
     compute pass-rate delta on shared held-out subset; return percentage
     delta (positive = improvement, negative = regression).
   - `fn gate_promotion(delta: f32, regression_threshold: f32) -> GateResult`:
     `GateResult { allow: bool, reason: String }`; allow = (delta >= -regression_
     threshold); reason includes delta value.

6. **SSB action types** in ssb.rs (SCHEMA_VERSION 51→52):
   - `HeldOutRefresh { date: String, action: String, task_count_added: u32,
     task_count_retired: u32 }`
   - `HeldOutAutoRetire { task_id: TaskId, reason: String, age_days: u32,
     uniform_pass_rate: f64 }`
   - `HeldOutPromotionGate { template_id: TemplateId, delta: f64, threshold: f64,
     allowed: bool }`
   - Round-trip tests.

7. **elai_cc.toml [held_out] section**:
   - `[held_out]\nroot_path = ".elai_cc/held-out"\n
     ttl_months = 6\nuniform_pass_threshold = 0.95\n
     min_evaluations_for_retire = 20\nregression_threshold_pct = 5.0\n
     ngram_size = 9\nngram_jaccard_threshold = 0.3`
   - L3-LOCKED.

8. **Seed held-out tasks** (manual step documented; not auto-generated):
   - Author 25 initial held-out tasks across 5 domains (5 per family).
   - Store at .elai_cc/held-out/2026-05-19/{task_id}.toml.
   - Sources: 10 GitHub issues with verifiable recency (use API or manual
     curation), 10 ARC-AGI-public-tasks, 5 hand-curated math/prose.
   - For each, populate ExpectedOutcome; compute content_hash; commit.

9. **N-gram bloom filter pre-compute** (deferred to first auto-refresh run):
   - Optional Phase: pre-compute training-corpus n-gram bloom filter from
     a subset of public-known-training-data (HuggingFace `the-stack` for code;
     skip for D-6 if not available; mark as DEFERRED).
   - For D-6 scope: implement filter logic + interface; assume bloom file may
     be empty initially. Auto-refresh will skip n-gram check until file populated.

10. **Unit tests**:
    - HeldOutStore load + append + retire round-trip on tmpdir.
    - extract_ngrams("a b c d e", 3) returns {"a b c", "b c d", "c d e"}.
    - ngram_overlap on identical text = 1.0; on disjoint = 0.0.
    - detect_ngram_contamination on synthetic overlapping vs disjoint corpus
      with synthetic bloom filter.
    - AutoRefreshPolicy retire-decision table: 7-month task with 100% pass-rate
      and 25 evals → retire; 7-month with 80% pass-rate → keep; 5-month with
      100% pass-rate → keep; 7-month with 100% pass-rate and 10 evals → keep
      (insufficient samples).
    - gate_promotion: delta=-3% with threshold=5% → allow; delta=-7% → block.

11. **Integration test** in
    .elai_cc/crates/orchestrator/tests/held_out_integration.rs:
    - Synthesize 50 fake held-out tasks; mock eval history.
    - Run check_for_auto_retire across 12-month time spans; verify expected
      retire set.
    - Run gate_promotion against baseline + candidate eval histories.

12. **Decision doc** workflow/reasoning/D-6-decisions.md (3 decisions):
    - (a) Why per-task TOML file (not single JSON db): file-per-task is git-
      diff-friendly + sled-locking-free + scales to ~10K tasks at TOML's
      reasonable bound.
    - (b) Why n=9 for n-gram filter (not 7 or 13): LiveCodeBench published
      default; provides good signal-to-false-positive ratio per paper §3.2.
    - (c) Why ttl_months=6 (not 12): post-training-cutoff is rapidly updating
      for frontier models; 6mo gives safety margin; balanced against curator
      cost.

## WIRING_CONTRACT

produces_symbols:
  - HeldOutTask + TaskId + ExpectedOutcome + TaskSource + HeldOutStatus +
    RetireReason
  - HeldOutStore + load/append/retire/query
  - AutoRefreshPolicy + check_for_auto_retire
  - extract_ngrams + ngram_overlap + detect_ngram_contamination
  - held_out_regression + gate_promotion + GateResult
  - SsbActionType::HeldOutRefresh + ::HeldOutAutoRetire + ::HeldOutPromotionGate
    + payloads
  - SCHEMA_VERSION bump 51→52
  - [held_out] elai_cc.toml section
  - .elai_cc/held-out/{date}/{task_id}.toml file layout
consumes_symbols:
  - SeededClock (C-15) for clock.now()
  - ArchiveChain (D-1) for promotion-gate decision audit trail
  - BaiPromotionEngine (D-5) consumes gate_promotion result
  - blake3 (already verified)
  - C-17 chunker / tokenization for n-gram extraction
reachable_from:
  - BAI promotion gate (D-5 calls held_out_regression before promotion)
  - elai_cc.toml [held_out] section
  - .elai_cc/held-out/ directory layout
stub_ban:
  - HeldOutStore MUST NOT delete task files; AutoRetired is status-marked
  - extract_ngrams MUST handle empty input (return empty set, not panic)
  - check_for_auto_retire MUST require min_evaluations_for_retire (defense
    against retiring tasks with too-small sample)
  - gate_promotion threshold MUST load from elai_cc.toml; not hardcoded
observable_effect:
  trigger: seed 25 held-out tasks; simulate 12-month aging with all tasks
           uniformly passing
  expect_ssb: ≥20 HeldOutAutoRetire rows with reason=UniformPassSuspicion;
              0 retire rows for tasks younger than 6 months
  expect_artifact: .elai_cc/held-out/2026-05-19/*.toml contains 25 task files;
                   status field updated post-retire

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 52
rg "SCHEMA_VERSION: i64 = 52" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "HeldOutRefresh|HeldOutAutoRetire|HeldOutPromotionGate" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l

# 3. HeldOut types
rg "struct HeldOutTask|struct HeldOutStore|struct AutoRefreshPolicy" \
  .elai_cc/crates/orchestrator/src/held_out.rs

# 4. N-gram filter
rg "fn extract_ngrams|fn ngram_overlap|fn detect_ngram_contamination" \
  .elai_cc/crates/orchestrator/src/held_out.rs

# 5. elai_cc.toml [held_out]
rg "\[held_out\]" .elai_cc/elai_cc.toml
rg "ttl_months\s*=\s*6\|ngram_size\s*=\s*9\|regression_threshold_pct" \
  .elai_cc/elai_cc.toml

# 6. Held-out directory + seed tasks
test -d .elai_cc/held-out/2026-05-19 && ls .elai_cc/held-out/2026-05-19/*.toml | wc -l
# Expected: >= 20

# 7. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- held_out 2>&1 | grep "test result"

# 8. Paper-ingestor outputs
test -f workflow/papers/livecodebench-2403.07974-interface.md && \
  test -f workflow/papers/soft-contamination-2602.12413-interface.md && \
  test -f workflow/papers/arc-prize-2025-2601.10904-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 52
- [ ] 3 SSB variants + payloads round-trip
- [ ] HeldOutStore load/append/retire on tmpdir; idempotent
- [ ] N-gram filter correct on canonical inputs (table-driven, ≥10 scenarios)
- [ ] AutoRefreshPolicy retire-decision table (4 scenarios) passes
- [ ] gate_promotion threshold logic correct
- [ ] elai_cc.toml [held_out] loaded; defaults applied
- [ ] 25 seed held-out tasks present at .elai_cc/held-out/2026-05-19/
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- held_out: all pass, ≥12 new tests
- [ ] Cloud smoke test A: simulate 12-month aging (mocked clock); verify
      HeldOutAutoRetire SSB rows for uniformly-passing aged tasks
- [ ] Cloud smoke test B: BAI promotion with candidate that regresses on
      held-out by 7% → HeldOutPromotionGate row with allowed=false; no
      TemplatePromoted row
- [ ] Cloud smoke test C: held-out task ingest via paper-ingestor (e.g. fetch
      newly-published arxiv repro task) → HeldOutRefresh SSB row + file created
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-6-decisions.md (3 decisions)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/held_out.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 51→52;
    3 new variants + payloads)
- MODIFY: .elai_cc/crates/orchestrator/src/bai.rs (gate_promotion integration)
- MODIFY: .elai_cc/elai_cc.toml ([held_out] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod held_out)
- ADD: .elai_cc/held-out/2026-05-19/{task_id}.toml × 25 (seed tasks)
- ADD: .elai_cc/crates/orchestrator/tests/held_out_integration.rs
- ADD: workflow/reasoning/D-6-decisions.md
- ADD: workflow/papers/{livecodebench-2403.07974,soft-contamination-2602.12413,
       arc-prize-2025-2601.10904}-interface.md

## Out-of-scope

- D-7 cross-domain canary (consumes held-out per-family pass-rate)
- D-15 NoveltyClass (uses DKG, not held-out, for near-duplicate detection)
- D-20 ARC-AGI-2 OOD probe (uses ARC tasks as held-out subset; specialized
  benchmark)
- N-gram bloom filter from training corpus (DEFERRED until data access)

## Next

D-7 (cross-domain canary) consumes per-family held-out pass-rates. D-22
(conformal calibration validation) reuses held-out store as one of three
distributions (in-domain, near-OOD, far-OOD).

## J-11 AMENDMENT — S-2 B_dev/B_test TEMPORAL SPLIT PRIOR ART (source-cited)
Grounding: Arbor `src/coordinator/tools/git_ops.py:374-446` (GitMergeBranchTool), `src/coordinator/prompts.py:442-450`
(Evaluation Discipline), read directly per W-3.
GD-1 prior-art pointer: Arbor's B_dev/B_test split is a simpler form of D-6's temporal held-out split. Arbor
separates the iteration set (B_dev, `prompts.py:444` "B_dev is for iteration: executor experiments, score
tracking, idea selection") from the milestone set (B_test, `prompts.py:446` "B_test is for milestones only:
run B_test before merging to trunk"). The merge gate (`git_ops.py:374` eval_cmd_test from tree.meta, `:390`
_run_eval_in_worktree) re-runs B_test in a detached worktree — the executor never touches B_test. This is D-6's
temporal split in a simpler form: D-6 adds the time-windowed + 9-gram contamination filter + auto-retire by
uniform pass-rate; Arbor's split is purely manual (the human sets eval_cmd_test). Compose, do NOT restate —
D-6 owns the time-windowed store with the ngram filter and auto-retire.

## J-11 AMENDMENT — S-8c .git RE-INIT STORE HYGIENE (source-cited)
Grounding: DeNovoSWE paper `2606.10728.html` §3.3 (S3.SS3), `S3.I1.i3.p1.1` Git history sanitization, read directly per W-3.
DeNovoSWE's pre-cleanup phase includes a critical store-hygiene step D-6 must adopt for held-out fixtures:
**re-initialize .git entirely** (`S3.I1.i3.p1.1`: "By completely destroying and re-initializing the .git
directory, this step eliminates the vulnerability where agents could exploit git reflog or loose Git objects
to reverse-engineer and reconstruct the original repository history, ensuring a true 'closed-book' generation
task"). D-6's held-out task store must ensure that held-out fixtures do not carry .git history that an agent
could recover via `git reflog` or loose-object inspection. The fix: when seeding or refreshing held-out tasks
from real repos, destroy and re-init the .git directory in the fixture so no reference implementation history
is recoverable. This is store hygiene, not a runtime gate — it runs at fixture-build time, not at eval time.
The runtime leakage prevention (command blocking + trace audit) is H-47's scope (S-8c amendment to H-47).
