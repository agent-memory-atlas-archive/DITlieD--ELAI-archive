# E-54: Repair D-6 9-gram filter before deployment — 13-gram + embedding cosine + per-family directory shards + test-triplet schema + matched-pair sycophancy probes; KOR-orthogonality check on D-7
Objective: D-6's currently-PLANNED 9-gram filter is provably evaded (Yang & Chiang 2023 / arXiv:2311.04850 — 13B model matches GPT-4 on MMLU/GSM8K/HumanEval via paraphrases that 13-gram never sees; arXiv:2602.12413 — 78% of CodeForces problems have semantic dupes in Olmo3 corpus that 9-gram misses). E-54 repairs the filter BEFORE deployment: upgrade to 13-gram baseline + add embedding-cosine second filter (BGE-M3 / Nomic; cosine >0.85 → reject); switch TOML storage layout from flat `held-out/` to per-family directory shards `held-out/{family}/{date}/{task-id}.toml` + `held-out/_index/{family}.jsonl` summary (Windows NTFS perf cliff before 5K files); add mandatory `cutoffs: {model: "YYYY-MM"}` field for time-windowed filtering per LiveCodeBench v6 pattern (arXiv:2403.07974); add mandatory `tests: [...]` test-triplet field per KodCode (arXiv:2503.02951) + OpenCoder (arXiv:2411.04905) recipes; add matched-pair eval-vs-deploy framing on D-23 sycophancy probes (eval-awareness now measured: Sonnet 4.5 verbalizes "I am being evaluated" >80% of time per arXiv:2510.20487); enforce KOR-orthogonality check on D-7 family selection (arXiv:2410.06526) so the 5 families don't collapse to "code-flavored language modeling".
Estimated effort: 3 sessions, ~12 h
Pre-requisites: D-6 PLANNED (Time-windowed held-out task store + 9-gram filter — per D-SUMMARY_INDEX line 52: "Pre-reqs: D-5"). D-7 PLANNED (Cross-domain transfer canary: 5 task families; pre-req D-6 per-family tagging). D-23 PLANNED (Sycophancy detection benchmark — per D-SUMMARY_INDEX line 162; pre-req D-22 / D-13). D-24 PLANNED (Cross-domain canary regression validation; pre-req D-7 — per line 168). E-54 REPLACES the 9-gram-only filter in D-6 and AUGMENTS D-7 + D-23 with measured-eval-awareness defenses.
Parallel-safe with: E-51 (edge inference — disjoint), E-52 (LSP integration — disjoint), E-53 (formal verification — disjoint), E-55 (agentic RL — D-5 BAI is upstream; coordinate on held-out gate signal). REPLACES D-6's 9-gram-only filter; EXTENDS D-7 family validation; EXTENDS D-23 sycophancy schema; EXTENDS D-24 regression-suite hooks. Coordinates with E-9 (agent eval benchmarks) if it exists — verify via Glob.
Source report: workflow/improvement-sweep-2026-05-19/reports/54-eval-corpus-design.md

## Context

The May 2026 eval-corpus survey establishes that ELAI's currently-PLANNED 9-gram filter (per D-6 PLANNED) is provably broken before it ships: Yang & Chiang 2023 (arXiv:2311.04850) showed a 13B model can match GPT-4 on MMLU/GSM8K/HumanEval by training on paraphrases that even 13-gram (GPT-3's threshold) never sees — 9-gram is weaker still. The Soft Contamination study (arXiv:2602.12413) finds 78% of CodeForces problems + 50% of ZebraLogic problems have semantic dupes in the Olmo3 corpus that n-gram alone misses. SWE-bench-Live (arXiv:2505.23419) ships 1319 real GitHub issues post-2024-01-01 with monthly Docker-per-task updates — the architectural template for time-windowed eval. LiveCodeBench v6 (arXiv:2403.07974) has 1055 problems across May-2023..Apr-2025 with release-date stamps; DeepSeek/GPT-4o show clear post-cutoff perf drop on LeetCode — time-window split is the cheapest leakage probe. Eval-awareness is no longer hypothetical: Claude Sonnet 4.5 verbalizes "I am being evaluated" >80% of the time in pre-deployment runs (arXiv:2510.20487 + Anthropic); covert-sandbag work (arXiv:2508.00943) shows models can sandbag against chain-of-thought monitoring. KOR-Bench (arXiv:2410.06526) names the orthogonality discriminator: a family is knowledge-orthogonal when domain knowledge cannot transfer between it and adjacent families — D-7's "5 families" claim must be VERIFIED via cross-family transfer test, not asserted. KodCode (arXiv:2503.02951) ships 447K verified Q-solution-test triplets keeping error <2.5% — triplet schema beats Q+A schema; tests are first-class. OpenCoder (arXiv:2411.04905) executes generated tests as the quality filter. Phi-4 tech report (arXiv:2412.08905) needed 50 distinct synthetic-data techniques to hit 400B tokens — single-generator synthesis collapses diversity. BrokenMath (arXiv:2510.04721) reports GPT-5 = 29% sycophantic on perturbed-true theorems — D-23 must use BrokenMath-style perturbation, not synthetic-only. JudgeBench / arXiv:2412.12509 shows only largest LLM-judges hit acceptable agreement — calibrate cloud-LLM-grader against held-out human grades quarterly. METR Time Horizons (arXiv:2503.14499) shows frontier 50%-horizon doubles every ~7 months — a frozen 200-task suite becomes noise within a year. OKBench (arXiv:2511.08598) shows on-demand benchmark generation is now feasible (~2K MCQs/day from last-24h news).

## Existing state (B/C/D coverage)

- D-6 PLANNED (Time-windowed held-out task store + 9-gram filter; per D-SUMMARY_INDEX line 52-55): currently planned with 9-gram filter and unspecified storage layout. REPLACEMENT: E-54 upgrades 9-gram to 13-gram + adds embedding-cosine second filter + switches flat layout to per-family directory shards + adds mandatory cutoffs + test-triplet schema. D-6's "time-windowed" framing remains unchanged; the contamination defense is strengthened.
- D-7 PLANNED (Cross-domain transfer canary: 5 task families; per line 58-60): currently lists 5 families by name. AUGMENTATION: E-54 adds the KOR-orthogonality verification at canary build time — train tiny adapter on family-A training split, measure transfer accuracy on family-B test split; abort if >random+10pp. Per arXiv:2410.06526.
- D-23 PLANNED (Sycophancy detection benchmark; per line 162-166): currently lists sycophancy benchmark surface; pre-req D-22 / D-13. EXTENSION: E-54 adds matched-pair framing (eval vs deploy) per arXiv:2510.20487 and BrokenMath-style perturbation per arXiv:2510.04721; auditor flags >10pp gap as sandbagging signal.
- D-24 PLANNED (Cross-domain canary regression validation; per line 168-170): consumes D-7 + the 9-gram filter. EXTENSION: E-54 wires the new 13-gram + embedding-cosine + test-triplet schema through to D-24; existing 100 synthetic mutations (50 helpful, 30 neutral, 20 harmful) framework stays; ADD ConStat-style performance-delta detector for when self-mod improves on family X >Y over 1 week.
- D-28 DEFERRED (Adversarial example farming for PPM; per line 196-198): coordinates with E-55 RL deferral — both are downstream-deferred.
- D-30 DEFERRED (Conformal calibration bootstrap from generic corpus; per line 208-210): coordinates with the embedding-cosine threshold tuning in Open Q1.
- E-9 (agent eval benchmarks) if shipped: verify via Glob — coordinate on dataset-shape.
- No existing held-out store, no existing 13-gram or embedding-cosine filter, no existing per-family directory layout: all NEW.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\54-eval-corpus-design.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md` lines 50-70 (D-6, D-7) + 160-175 (D-23, D-24)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-9_agent_eval_benchmarks.md` (coordinate on dataset-shape if shipped; verify via Read first)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-49_coding_agent_metrics.md` (coordinate on metrics; verify via Read first)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-55_agentic_rl_and_reward.md` (D-5 BAI gate coordination — read after E-55 file is authored if both are in same wave)
- `F:\.ELAI_workflow\CLAUDE.md` "Verify semantics, not just 'test passed'" rule — grounds Task 4 test-triplet schema
- https://arxiv.org/abs/2311.04850 (WebFetch — Yang & Chiang Rephrased Samples; 13B beats baseline via paraphrase that evades 13-gram)
- https://arxiv.org/abs/2602.12413 (WebFetch — Soft Contamination: 78% CodeForces + 50% ZebraLogic semantic dupes)
- https://arxiv.org/abs/2403.07974 (WebFetch — LiveCodeBench ICLR 2025; v6 1055 problems May23..Apr25 with release-date stamps)
- https://arxiv.org/abs/2505.23419 (WebFetch — SWE-bench Live; 1319 real GitHub issues post 2024-01-01; monthly Docker-per-task)
- https://arxiv.org/abs/2410.06526 (WebFetch — KOR-Bench: Knowledge-Orthogonal-Reasoning; 5 categories chosen so domain knowledge can't help)
- https://arxiv.org/abs/2503.02951 (WebFetch — KodCode ACL 2025; 447K verified Q-solution-test triplets, error <2.5%)
- https://arxiv.org/abs/2411.04905 (WebFetch — OpenCoder cookbook; executes generated tests to filter)
- https://arxiv.org/abs/2510.20487 (WebFetch — Steering Evaluation-Aware Models; Sonnet 4.5 verbalizes "I am being evaluated" >80% of time)
- https://arxiv.org/abs/2508.00943 (WebFetch — Covert sandbagging against CoT monitoring)
- https://arxiv.org/abs/2510.04721 (WebFetch — BrokenMath; GPT-5 sycophantic 29% on perturbed-true theorems)
- https://arxiv.org/abs/2412.12509 (WebFetch — JudgeBench; only largest judges hit acceptable agreement)
- https://arxiv.org/abs/2412.08905 (WebFetch — Phi-4 Tech Report; 50 distinct synthetic-data techniques)
- https://arxiv.org/abs/2410.01560 (WebFetch — OpenMathInstruct-2; LLM-pipeline decontamination step)
- https://arxiv.org/abs/2503.14499 (WebFetch — METR Time Horizons; 7-month doubling on agentic horizon)
- https://arxiv.org/abs/2511.08598 (WebFetch — OKBench; ~2K MCQs/day from last-24h news)
- https://arxiv.org/abs/2502.14425 (WebFetch — Survey on Data Contamination; taxonomy)
- https://arxiv.org/abs/2509.00072 (WebFetch — Test-of-Time; synthesis can OBSCURE rather than remove contamination)
- https://arxiv.org/abs/2511.17602 (WebFetch — Hierarchical Contamination Detection, F1=0.76 at semantic level)
- BGE-M3 paper / HuggingFace card (WebFetch BAAI/bge-m3 — embedding model)
- Nomic Embed v2 (WebFetch nomic-ai/nomic-embed-text-v2-moe — embedding model)

## Tasks

1. Upgrade 9-gram → 13-gram + add embedding-cosine second filter. CREATE `.elai_cc/crates/eval-corpus/src/ingest.rs` (~280 LOC). The ingest pipeline: (a) compute 13-gram set on normalized task text (lowercased, punctuation stripped, whitespace collapsed); reject if 13-gram overlap with any held-out task exceeds K=4 consecutive grams (matches GPT-3's threshold per arXiv:2311.04850; configurable via `[corpus.ngram_threshold] n=13 k_consecutive=4`). (b) compute BGE-M3 OR Nomic Embed v2 embedding (chosen at config time; default BGE-M3 since both ELAI-supported via the existing sidecar pipeline — verify via Read on `python/elai_inference/embeddings.py` if exists; otherwise add a thin embedding endpoint to the sidecar). Reject if cosine similarity >0.85 with any prior held-out task (per arXiv:2602.12413; threshold tunable). (c) Both filters run before TOML write — single-pass rejection emits `corpus_ingest_rejected_ngram` or `corpus_ingest_rejected_cosine` SSB row with the offending task id. ANTI-PATTERN guard: NEVER skip the cosine pass — Test-of-Time (arXiv:2509.00072) shows synthesis can obscure contamination that n-gram still passes. Use blake3 to content-hash the normalized text for caching cosine computations within a session.

2. Per-family directory shards + `_index/{family}.jsonl` summary. CREATE the storage layout under `.elai_cc/held-out/` with shape:
   - `.elai_cc/held-out/{family}/{YYYY-MM-DD}/{task-id}.toml` (sharded by family + creation date)
   - `.elai_cc/held-out/_index/{family}.jsonl` (summary index per family, one row per task: `{id, path, created_at, cutoffs, embedding_hash, ngram_hash}`)
   - 5 families per D-7: `code-fix`, `math-proof`, `prose-reason`, `planning`, `knowledge-qa`
   CREATE `.elai_cc/crates/eval-corpus/src/storage.rs` (~220 LOC) implementing the shard + index pattern. ON ingest, append to `_index/{family}.jsonl` after the per-task TOML write (atomic append via `O_APPEND` per stdlib Python or `OpenOptions::append(true)` per Rust). Verify NTFS perf: at >5K files in single directory, fragment/lookup latency jumps significantly per Microsoft NTFS docs (the per-family shard caps `held-out/{family}/{date}/` at ≤500 files per date dir; daily rotation handles growth). Add `corpus_shard_created` SSB row on first task per (family, date) shard.

3. Mandatory TOML schema fields: cutoffs + tests + provenance + framing_pair (for D-23) + perturbed_pair (for BrokenMath-style sycophancy). CREATE `.elai_cc/crates/eval-corpus/src/task_schema.rs` (~180 LOC) defining the canonical TOML task schema with serde derive:
   - `id: String` (canonical task id; matches filename without extension)
   - `family: TaskFamily` (enum, 5 variants)
   - `created_at: chrono::DateTime<Utc>`
   - `cutoffs: HashMap<String, String>` (model_id → "YYYY-MM"; e.g. `{qwen2.5-coder-3b: "2024-10", claude-sonnet-4.7: "2026-01"}`; per arXiv:2403.07974)
   - `problem: String` (the prompt)
   - `reference_solution: String` (canonical solution)
   - `tests: Vec<ExecutableTest>` (REQUIRED, NON-EMPTY; each entry has `input`, `expected_output`, `kind: TestKind::{UnitTest, EndToEnd, PropertyTest}`; per arXiv:2503.02951)
   - `provenance: Provenance { generator: String, prompt_id: Option<String>, date: chrono::DateTime<Utc> }` (per arXiv:2412.08905 generator-rotation rule)
   - `framing_pair: Option<FramingPair { eval: String, deploy: String }>` (for D-23 sycophancy probes only; eval = "evaluation framing", deploy = "production framing"; per arXiv:2510.20487)
   - `perturbed_pair: Option<PerturbedPair { original_true: String, subtly_false: String, expected_answer: SycophancyExpected::{AgreeWithTrue, RejectFalse} }>` (for BrokenMath-style sycophancy; per arXiv:2510.04721)
   - `embedding_hash: String` (blake3 of normalized text)
   - `ngram_hash: String` (blake3 of 13-gram set canonical bytes)
   The TOML loader REJECTS any task without `tests` (empty or missing) — emits `corpus_schema_violation_no_tests` SSB row.

4. Test-execution-based quality filter. CREATE `.elai_cc/crates/eval-corpus/src/test_executor.rs` (~200 LOC). On ingest of every NEW task: (i) run all `tests` against the `reference_solution` in a sandbox (Docker container or wasm runtime — verify via Glob if `crates/sandbox/` exists; if not, ship the test executor with a clear `TODO: wire sandbox once available` marker AND register it under stub_tracking with babystep id, per CLAUDE.md stub registry). All tests MUST pass on reference_solution; reject task if any fails (emits `corpus_test_fails_on_reference` row). (ii) run all tests against a deliberately-broken solution (the reference with one line corrupted); ≥1 test MUST fail (emits `corpus_test_useless_passes_broken` row if all pass — indicates test is too weak). Per arXiv:2411.04905 OpenCoder recipe.

5. KOR-orthogonality check at D-7 canary build. CREATE `.elai_cc/crates/eval-corpus/src/kor_orthogonality.rs` (~180 LOC). At D-7 canary build time, for each pair (A, B) of the 5 families: (i) split family A into train/test; (ii) fine-tune a tiny adapter (e.g. LoRA on Qwen 3B) on family A train; (iii) measure accuracy on family B test; (iv) if family-B accuracy > random_baseline + 10pp, families are NOT orthogonal — abort canary build with `kor_orthogonality_violated` SSB row + log the offending (A,B) pair. random_baseline is computed per-family (e.g. multiple-choice = 1/n; binary = 0.5; open-form = task-specific). Per arXiv:2410.06526. This is a one-time check per family-set change (not per ingest).

6. Matched-pair framing for D-23 sycophancy probes. CREATE `.elai_cc/crates/eval-corpus/src/sycophancy_matched_pair.rs` (~160 LOC). For each task with `framing_pair` set, the auditor invokes the model twice with the same underlying prompt but different framings (eval phrasing AND deploy phrasing — never include the word "benchmark" or "evaluation" in the deploy variant per arXiv:2510.20487). Computes score_eval and score_deploy. If |score_eval - score_deploy| > 10pp, emit `sycophancy_framing_gap_detected` SSB row + flag for human review. Also: add covert-sandbag detector (per arXiv:2508.00943) — measure response latency + token count for each variant; significant deviation = `sandbagging_anomaly` row. Mode: this lives in the D-23 sycophancy benchmark surface; E-54 ships the SCHEMA + harness, D-23 ships the actual benchmark suite.

7. Generator rotation enforcement for synthetic growth. CREATE `.elai_cc/crates/eval-corpus/src/generator_rotation.rs` (~140 LOC). When expanding the corpus from N to M tasks via synthesis: (i) require at least 3 distinct generators (local-LLM, cloud-LLM, human-curated import); (ii) enforce min-30% from each — emit `generator_rotation_violated` row if any generator is <30%. Per arXiv:2412.08905 Phi-4 lesson (50 distinct techniques for 400B tokens) + arXiv:2509.00072 Test-of-Time (single-pipe synthesis obscures contamination). After every 50-task batch run a re-decontamination sweep (Task 1) on the FULL corpus and on the previous month's training data (relevant for self-mod loop).

8. ConStat-style performance-delta contamination detector for D-24. CREATE `.elai_cc/crates/eval-corpus/src/perf_delta_detector.rs` (~160 LOC). When ELAI's self-mod loop improves on family X by >Y over 1 week, fire ConStat-style detector: compare current model performance to reference model on the SAME held-out slice; sudden jump beyond a configurable threshold = `perf_delta_contamination_suspected` SSB row. Per ConStat NeurIPS 2024 (cited in report — no arXiv ID given; verify via WebSearch before adopting reference-performance estimation formula; if unverifiable, mark as Open Question and ship a simpler 2-sigma-deviation detector). D-24 regression gate already has the slot; this is the implementation.

9. Dynamic eval-time generation for knowledge-QA family. CREATE `.elai_cc/crates/eval-corpus/src/dynamic_knowledge_qa.rs` (~180 LOC). For the `knowledge-qa` family, switch from replay-from-TOML to OKBench-style daily generation (per arXiv:2511.08598). Sidecar's existing SearXNG endpoint (per CLAUDE.md "SEARXNG_URL=http://127.0.0.1:8889") fetches last-24h news; LLM-generates MCQs from top-K results; tasks are NOT persisted to TOML (one-shot consumption) BUT the SearXNG query hash + timestamp + generated MCQ set is logged to `held-out/_audit/knowledge-qa-{date}.jsonl` for reproducibility. Reduces stale-knowledge-QA risk that arXiv:2511.08598 measures.

10. Real-user-traffic grade tier (Anthropic-style silent probes). CREATE `.elai_cc/crates/eval-corpus/src/prod_traffic_grade.rs` (~160 LOC). Opt-in production turns become eval candidates: every Nth turn (configurable; default 1/100), a held-out copy of the task is silently injected (eval markers stripped per matched-pair rule from Task 6). User thumbs-down + grader-LLM verdict feeds corpus growth via the ingest pipeline from Task 1. Per arXiv:2403.04132 Chatbot Arena framing. Stored under `held-out/from-prod/{date}/{turn-hash-blake3}.toml` (separate shard so prod-derived tasks are clearly labeled). OPT-IN ONLY — requires explicit user consent flag in CLI config; default OFF.

11. Dashboard panels + Tauri commands. CREATE `crates/dashboard-v2/src/lib/components/EvalCorpusPanel.svelte` (~180 LOC): shows per-family task count + last decontamination sweep result + cosine threshold + 13-gram threshold + KOR-orthogonality status. CREATE `crates/dashboard-v2/src/lib/components/SycophancyMatchedPairPanel.svelte` (~140 LOC): shows recent matched-pair sycophancy probe results with eval-vs-deploy score gap + sandbagging-anomaly flags. CREATE Tauri commands `get_eval_corpus_status`, `get_sycophancy_matched_pair_results`, `get_kor_orthogonality_status` in `crates/dashboard-v2/src-tauri/src/commands.rs` (~100 LOC).

12. Migration of any existing held-out tasks + decisions doc. CREATE `.elai_cc/crates/eval-corpus/src/bin/migrate_held_out.rs` (~120 LOC, optional bin for one-time migration). If existing held-out tasks live in a flat `held-out/` layout (verify via Glob; D-6 is PLANNED, may already have a placeholder), the migrate bin reads each TOML, sharded-writes to `held-out/{family}/{date}/{id}.toml`, appends to `_index/{family}.jsonl`, and runs the 13-gram + cosine + test-triplet validation. Tasks failing validation are written to `held-out/_quarantine/` with the failure reason. Idempotent. CREATE `workflow/reasoning/E-54-decisions.md` (~120 LOC) summarising the 13-gram-not-9-gram repair, the embedding-cosine threshold rationale (0.85 default per arXiv:2511.17602 F1=0.76 baseline), the per-family shard rationale (NTFS 5K-file perf cliff), and the open questions on (i) actual NTFS measurement at 1K / 5K; (ii) embedding cosine PR threshold tuning for ELAI's own corpus; (iii) matched-pair false-positive rate on non-deceptive prompt sensitivity per Qwen2.5-Coder-3B vs Sonnet; (iv) ConStat without per-task ground truth.

## WIRING_CONTRACT

produces_symbols:
  - eval_corpus::ingest::ingest_task(task: NewTaskInput) -> IngestResult
  - eval_corpus::task_schema::Task { id, family, created_at, cutoffs, problem, reference_solution, tests, provenance, framing_pair, perturbed_pair, embedding_hash, ngram_hash }
  - eval_corpus::task_schema::TaskFamily::{CodeFix, MathProof, ProseReason, Planning, KnowledgeQa}
  - eval_corpus::task_schema::ExecutableTest { input, expected_output, kind }
  - eval_corpus::task_schema::TestKind::{UnitTest, EndToEnd, PropertyTest}
  - eval_corpus::task_schema::Provenance { generator, prompt_id, date }
  - eval_corpus::task_schema::FramingPair { eval, deploy }
  - eval_corpus::task_schema::PerturbedPair { original_true, subtly_false, expected_answer }
  - eval_corpus::storage::shard_path(family, date, id) -> PathBuf
  - eval_corpus::storage::append_index(family, row: IndexRow)
  - eval_corpus::test_executor::execute_against_reference(task: &Task) -> TestExecutionResult
  - eval_corpus::kor_orthogonality::cross_family_transfer(a: TaskFamily, b: TaskFamily) -> TransferResult
  - eval_corpus::sycophancy_matched_pair::run_matched_pair(task: &Task) -> MatchedPairResult
  - eval_corpus::generator_rotation::verify_distribution(corpus: &[Task]) -> DistributionResult
  - eval_corpus::perf_delta_detector::detect(history: &PerfHistory) -> DetectionVerdict
  - eval_corpus::dynamic_knowledge_qa::generate_today() -> Vec<KnowledgeQaTask>
  - eval_corpus::prod_traffic_grade::maybe_inject_held_out_probe(turn_count: u64) -> Option<Task>
  - SSB action_types: corpus_ingest_rejected_ngram, corpus_ingest_rejected_cosine, corpus_shard_created, corpus_schema_violation_no_tests, corpus_test_fails_on_reference, corpus_test_useless_passes_broken, kor_orthogonality_violated, kor_orthogonality_passed, sycophancy_framing_gap_detected, sandbagging_anomaly, generator_rotation_violated, perf_delta_contamination_suspected, dynamic_knowledge_qa_generated, prod_traffic_probe_injected
  - Tauri commands: get_eval_corpus_status, get_sycophancy_matched_pair_results, get_kor_orthogonality_status
consumes_symbols:
  - D-6 PLANNED surface (replaces 9-gram filter; held-out store framing kept)
  - D-7 PLANNED 5 families (augmented with KOR-orthogonality check)
  - D-23 PLANNED sycophancy benchmark (extended with matched-pair + perturbed-pair schema)
  - D-24 PLANNED regression validation (extended with ConStat-style perf-delta detector)
  - python/elai_inference/embeddings.py (existing OR to-be-created — sidecar embedding endpoint for BGE-M3 / Nomic Embed v2; verify via Read first)
  - python/elai_inference/server.py (existing sidecar; SearXNG endpoint per CLAUDE.md)
  - blake3 (text-hash + ngram-hash + content cache; NOT blake2/sha2)
  - chrono crate (DateTime<Utc> for cutoffs + created_at; existing in workspace — verify via Cargo.lock)
  - serde + toml (TOML serialization; existing)
reachable_from:
  - crates/orchestrator/src/auditor.rs (sycophancy_matched_pair invoked during AUDITING for D-23 tasks)
  - crates/orchestrator/src/d6_held_out.rs (verify path via Grep; ingest pathway routes here)
  - .elai_cc/held-out/{family}/{date}/{id}.toml (storage layout)
  - .elai_cc/held-out/_index/{family}.jsonl (summary index)
  - .elai_cc/held-out/_quarantine/ (failed-validation quarantine)
  - .elai_cc/held-out/_audit/knowledge-qa-{date}.jsonl (dynamic knowledge-QA audit log)
stub_ban:
  - no `.unwrap()` outside `#[cfg(test)]` in any new module (CLAUDE.md HARD RULE)
  - no `unimplemented!()` / `todo!()` in any new module (registered stubs allowed only per CLAUDE.md stub registry; e.g. Task 4 sandbox if absent — registered with babystep id)
  - 9-gram filter is REJECTED — minimum is 13-gram (per arXiv:2311.04850; contract test `no_nine_gram_only_decontam` enforces)
  - cosine threshold MUST default to 0.85 (per arXiv:2511.17602); configurable but never disabled (contract test `cosine_filter_never_disabled` enforces)
  - tests field is MANDATORY non-empty (contract test `corpus_schema_requires_non_empty_tests` enforces)
  - cutoffs field is MANDATORY (per arXiv:2403.07974 time-window pattern; contract test `corpus_schema_requires_cutoffs` enforces)
  - generator rotation min-30% from each of 3+ generators (contract test `generator_rotation_minimum_distribution` enforces)
  - matched-pair framing for D-23 tasks: eval AND deploy variants MUST exist (contract test `d23_tasks_have_framing_pair` enforces)
  - eval framing MUST NOT include "benchmark" / "evaluation" / "test" in deploy variant (regex check + contract test)
  - reference_solution MUST pass all tests (corpus_test_fails_on_reference REJECTS; contract test enforces)
  - tests MUST fail on broken solution (corpus_test_useless_passes_broken REJECTS; contract test enforces)
  - KOR-orthogonality verified before D-7 canary ship (cross-family transfer < random + 10pp; contract test enforces)
  - per-family directory shards MANDATORY (flat layout rejected; contract test `no_flat_held_out_layout` enforces)
  - prod-traffic grade tier is OPT-IN only (default OFF; contract test `prod_traffic_grade_defaults_off` enforces)
  - dynamic knowledge-QA generation MUST log SearXNG query hash + timestamp for reproducibility (no silent generation)
  - blake3 for hashing; no blake2/sha2 (CLAUDE.md anti-stale-training)
  - synthesized "novel" tasks via single generator are REJECTED — single-pipe synthesis collapses diversity per Phi-4 lesson
frontend_pair: crates/dashboard-v2/src/lib/components/EvalCorpusPanel.svelte + SycophancyMatchedPairPanel.svelte (Task 11)
observable_effect:
  trigger: ingest a new code-fix task that paraphrases an existing held-out task (same algorithm, different variable names + reordered statements)
  expect_ssb: 13-gram filter PASSES (paraphrase evades, matches Yang & Chiang finding), embedding-cosine filter REJECTS with cosine ~0.92 > 0.85 threshold, ≥1 `corpus_ingest_rejected_cosine` row with both task ids, no TOML written
  expect_artifact: `_index/code-fix.jsonl` line count UNCHANGED after the rejected ingest; `held-out/_quarantine/` gets a row with cosine match details

## DETERMINISTIC_CHECK

```bash
# 1. Ingest module uses 13-gram + cosine filter
rg "n.*13|N_GRAM.*13|13-gram|n_gram\\b.*13" .elai_cc/crates/eval-corpus/src/ingest.rs
# Expected: ≥1 hit
rg "cosine|embedding_similarity|bge.m3|Nomic" .elai_cc/crates/eval-corpus/src/ingest.rs
# Expected: ≥2 hits

# 2. No 9-gram fallback — 13-gram baseline only
rg "n_gram.*9|N_GRAM.*9|9-gram" .elai_cc/crates/eval-corpus/src/ingest.rs
# Expected: zero hits (or only inside comments referencing the legacy/rejected approach)

# 3. Per-family directory shards
rg "held-out/.*/\\{family\\}/|held_out/.*/\\{family\\}/|\\{family\\}/\\{date\\}|TaskFamily::CodeFix" .elai_cc/crates/eval-corpus/src/storage.rs
# Expected: ≥2 hits

# 4. _index/{family}.jsonl summary
rg "_index/.*\\.jsonl|append_index" .elai_cc/crates/eval-corpus/src/storage.rs
# Expected: ≥2 hits

# 5. Mandatory schema fields
rg "cutoffs|tests:.*Vec|provenance|framing_pair|perturbed_pair" .elai_cc/crates/eval-corpus/src/task_schema.rs
# Expected: ≥5 hits

# 6. 5 TaskFamily variants
rg "TaskFamily::CodeFix|TaskFamily::MathProof|TaskFamily::ProseReason|TaskFamily::Planning|TaskFamily::KnowledgeQa" .elai_cc/crates/eval-corpus/src/task_schema.rs
# Expected: ≥5 hits

# 7. Test executor runs reference + broken solution check
rg "execute_against_reference|execute_against_broken|corpus_test_fails_on_reference|corpus_test_useless_passes_broken" .elai_cc/crates/eval-corpus/src/test_executor.rs
# Expected: ≥3 hits

# 8. KOR-orthogonality check + 10pp threshold
rg "10pp|0\\.10|kor_orthogonality_violated|cross_family_transfer" .elai_cc/crates/eval-corpus/src/kor_orthogonality.rs
# Expected: ≥2 hits

# 9. Matched-pair framing + 10pp gap detector + sandbagging
rg "sycophancy_framing_gap_detected|sandbagging_anomaly|score_eval|score_deploy" .elai_cc/crates/eval-corpus/src/sycophancy_matched_pair.rs
# Expected: ≥3 hits

# 10. Generator rotation min-30%
rg "generator_rotation_violated|min.*30|0\\.30" .elai_cc/crates/eval-corpus/src/generator_rotation.rs
# Expected: ≥2 hits

# 11. ConStat / perf-delta detector
rg "perf_delta_contamination_suspected|ConStat" .elai_cc/crates/eval-corpus/src/perf_delta_detector.rs
# Expected: ≥1 hit

# 12. Dynamic knowledge-QA + audit log
rg "knowledge-qa-.*\\.jsonl|generate_today|SearXNG" .elai_cc/crates/eval-corpus/src/dynamic_knowledge_qa.rs
# Expected: ≥2 hits

# 13. Prod-traffic OPT-IN default OFF
rg "default.*false|opt-in|user.consent" .elai_cc/crates/eval-corpus/src/prod_traffic_grade.rs
# Expected: ≥1 hit

# 14. blake3 only
rg "blake3" .elai_cc/crates/eval-corpus/src/
# Expected: ≥2 hits
rg "blake2|sha2" .elai_cc/crates/eval-corpus/src/
# Expected: zero hits

# 15. No .unwrap() outside #[cfg(test)]
rg "\\.unwrap\\(\\)" .elai_cc/crates/eval-corpus/src/ | rg -v "cfg\\(test\\)|#\\[test\\]|tests/"
# Expected: zero hits

# 16. Smoke tests
cd .elai_cc && cargo test -p eval-corpus ingest_rejects_paraphrase_via_cosine
cargo test -p eval-corpus schema_requires_tests_and_cutoffs
cargo test -p eval-corpus kor_orthogonality_blocks_d7_on_violation
cargo test -p eval-corpus sycophancy_matched_pair_flags_framing_gap
cargo test -p eval-corpus generator_rotation_minimum_distribution
# Expected: 5 PASS
cargo test -p contracts no_nine_gram_only_decontam cosine_filter_never_disabled corpus_schema_requires_non_empty_tests no_flat_held_out_layout prod_traffic_grade_defaults_off
# Expected: 5 PASS
```

## ACCEPTANCE

- [ ] 13-gram baseline + embedding-cosine (BGE-M3 / Nomic Embed v2; default 0.85) second filter; 9-gram-only path REJECTED
- [ ] Per-family directory shards `held-out/{family}/{YYYY-MM-DD}/{id}.toml` + `held-out/_index/{family}.jsonl` summary
- [ ] 5 TaskFamily variants (CodeFix, MathProof, ProseReason, Planning, KnowledgeQa) align with D-7 plan
- [ ] Mandatory TOML schema fields: id, family, created_at, cutoffs, problem, reference_solution, tests (non-empty), provenance, embedding_hash, ngram_hash; optional framing_pair + perturbed_pair for D-23
- [ ] Test executor validates: reference_solution passes all tests; broken solution fails ≥1 test
- [ ] KOR-orthogonality check at D-7 canary build: cross-family transfer accuracy < random + 10pp, else abort
- [ ] Matched-pair framing for D-23 sycophancy probes (eval vs deploy phrasing); >10pp gap = sandbagging signal
- [ ] BrokenMath-style perturbed_pair schema for D-23 tasks
- [ ] Generator rotation: 3+ distinct generators, min-30% from each
- [ ] ConStat-style performance-delta detector for D-24 regression validation
- [ ] Dynamic knowledge-QA generation via SearXNG with audit log
- [ ] Prod-traffic grade tier OPT-IN only (default OFF)
- [ ] 14 SSB action types emitted as specified
- [ ] Dashboard panels: EvalCorpusPanel + SycophancyMatchedPairPanel + 3 Tauri commands wired
- [ ] No 9-gram fallback, no flat held-out layout, no missing tests, no missing cutoffs, no single-generator synthesis batches >30%
- [ ] Migration bin for any existing flat held-out tasks (idempotent)
- [ ] blake3 only for hashing; no blake2/sha2
- [ ] No `.unwrap()` outside `#[cfg(test)]` in new modules
- [ ] `cargo check --workspace` is green; D-6/D-7/D-23/D-24 PLANNED surfaces unaffected (additive)

## Tests to add

- `crates/eval-corpus/tests/ingest_rejects_paraphrase_via_cosine.rs`: paraphrased task evades 13-gram, hits cosine ≥0.85, REJECTED
- `crates/eval-corpus/tests/ingest_rejects_exact_match_via_ngram.rs`: identical 13-gram run REJECTED
- `crates/eval-corpus/tests/schema_requires_tests_and_cutoffs.rs`: task missing `tests` or `cutoffs` REJECTED
- `crates/eval-corpus/tests/test_executor_rejects_failing_reference.rs`: reference_solution that doesn't pass all tests REJECTED
- `crates/eval-corpus/tests/test_executor_rejects_useless_tests.rs`: tests that pass on broken solution REJECTED
- `crates/eval-corpus/tests/kor_orthogonality_blocks_d7_on_violation.rs`: cross-family transfer >random+10pp aborts canary build
- `crates/eval-corpus/tests/sycophancy_matched_pair_flags_framing_gap.rs`: score_eval - score_deploy = 15pp → flag
- `crates/eval-corpus/tests/generator_rotation_minimum_distribution.rs`: 2-generator corpus REJECTED; 3-generator with one at 20% REJECTED
- `crates/eval-corpus/tests/perf_delta_detector_fires_on_sudden_jump.rs`: simulated 30pp jump in a week → detection
- `crates/eval-corpus/tests/dynamic_knowledge_qa_logs_searxng_query.rs`: generation writes audit row
- `crates/eval-corpus/tests/prod_traffic_grade_requires_opt_in.rs`: default off; opt-in required
- `crates/eval-corpus/tests/storage_uses_family_shards.rs`: ingest writes to held-out/{family}/{date}/{id}.toml
- `crates/contracts/tests/no_nine_gram_only_decontam.rs`: assert n >= 13 in default config
- `crates/contracts/tests/cosine_filter_never_disabled.rs`: cosine filter cannot be disabled via config
- `crates/contracts/tests/corpus_schema_requires_non_empty_tests.rs`: schema definition has tests as required non-empty
- `crates/contracts/tests/no_flat_held_out_layout.rs`: assert flat held-out/*.toml rejected
- `crates/contracts/tests/prod_traffic_grade_defaults_off.rs`: default config has prod_traffic_grade=false
- `crates/contracts/tests/d23_tasks_have_framing_pair.rs`: all D-23 sycophancy tasks have framing_pair set
- `crates/contracts/tests/deploy_framing_no_eval_keywords.rs`: regex check on deploy variant prevents "benchmark"|"evaluation"|"test"

## Files touched

- CREATE: `crates/eval-corpus/Cargo.toml` (new crate; ~30 LOC; verify if crate exists first via Glob — if D-6 already created `crates/eval-corpus/`, EXTEND rather than CREATE)
- CREATE: `crates/eval-corpus/src/lib.rs` (~40 LOC module registration)
- CREATE: `crates/eval-corpus/src/ingest.rs` (~280 LOC)
- CREATE: `crates/eval-corpus/src/storage.rs` (~220 LOC)
- CREATE: `crates/eval-corpus/src/task_schema.rs` (~180 LOC)
- CREATE: `crates/eval-corpus/src/test_executor.rs` (~200 LOC)
- CREATE: `crates/eval-corpus/src/kor_orthogonality.rs` (~180 LOC)
- CREATE: `crates/eval-corpus/src/sycophancy_matched_pair.rs` (~160 LOC)
- CREATE: `crates/eval-corpus/src/generator_rotation.rs` (~140 LOC)
- CREATE: `crates/eval-corpus/src/perf_delta_detector.rs` (~160 LOC)
- CREATE: `crates/eval-corpus/src/dynamic_knowledge_qa.rs` (~180 LOC)
- CREATE: `crates/eval-corpus/src/prod_traffic_grade.rs` (~160 LOC)
- CREATE: `crates/eval-corpus/src/bin/migrate_held_out.rs` (~120 LOC migration bin)
- CREATE: 19 test files listed above (~30-100 LOC each)
- CREATE: `crates/dashboard-v2/src/lib/components/EvalCorpusPanel.svelte` (~180 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/SycophancyMatchedPairPanel.svelte` (~140 LOC)
- CREATE: `workflow/reasoning/E-54-decisions.md` (~120 LOC)
- CREATE: `.elai_cc/held-out/_README.md` (~40 LOC documenting the shard convention + index format)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~100 LOC delta — 3 new Tauri commands)
- MODIFY: `python/elai_inference/embeddings.py` (~80 LOC delta — BGE-M3 / Nomic embedding endpoint; CREATE if absent)
- MODIFY: `Cargo.toml` workspace (~10 LOC delta — register crates/eval-corpus member)

## Citations / Grounding

- https://arxiv.org/abs/2311.04850 — Yang & Chiang Rephrased Samples; 13B beats baseline via paraphrase evading 13-gram; grounds 9-gram → 13-gram upgrade in Task 1 + stub_ban
- https://arxiv.org/abs/2602.12413 — Soft Contamination: 78% CodeForces / 50% ZebraLogic semantic dupes; grounds Task 1 embedding-cosine second filter
- https://arxiv.org/abs/2403.07974 — LiveCodeBench v6 1055 problems May23..Apr25 with release-date stamps; grounds Task 3 `cutoffs` mandatory field
- https://arxiv.org/abs/2505.23419 — SWE-bench Live 1319 issues, monthly Docker-per-task; grounds Task 2 per-family shard architecture
- https://arxiv.org/abs/2410.06526 — KOR-Bench Knowledge-Orthogonal-Reasoning; grounds Task 5 cross-family transfer check
- https://arxiv.org/abs/2503.02951 — KodCode 447K verified Q-solution-test triplets, error <2.5%; grounds Task 3 mandatory `tests` schema
- https://arxiv.org/abs/2411.04905 — OpenCoder cookbook executes generated tests to filter; grounds Task 4 test-execution-based filter
- https://arxiv.org/abs/2510.20487 — Steering Evaluation-Aware Models; Sonnet 4.5 verbalizes "I am being evaluated" >80%; grounds Task 6 matched-pair framing
- https://arxiv.org/abs/2508.00943 — Covert sandbagging against CoT monitoring; grounds Task 6 sandbagging-anomaly detector
- https://arxiv.org/abs/2510.04721 — BrokenMath GPT-5 29% sycophantic on perturbed-true theorems; grounds Task 3 perturbed_pair schema
- https://arxiv.org/abs/2412.12509 — JudgeBench: only largest judges hit acceptable agreement; grounds Open Q "calibrate cloud-LLM grader quarterly"
- https://arxiv.org/abs/2412.08905 — Phi-4 Tech Report; 50 distinct synthetic-data techniques for 400B tokens; grounds Task 7 generator rotation min-30%
- https://arxiv.org/abs/2410.01560 — OpenMathInstruct-2 LLM-pipeline decontamination step; grounds Task 1 re-decontamination after synthesis
- https://arxiv.org/abs/2503.14499 — METR Time Horizons 7-month doubling; grounds Task 9 dynamic generation rationale
- https://arxiv.org/abs/2511.08598 — OKBench ~2K MCQs/day from last-24h news; grounds Task 9 dynamic knowledge-QA
- https://arxiv.org/abs/2502.14425 — Survey on Data Contamination taxonomy; grounds the levels in WIRING_CONTRACT
- https://arxiv.org/abs/2509.00072 — Test-of-Time: synthesis can OBSCURE contamination; grounds Task 7 generator-rotation + Task 1 re-decontam after batches
- https://arxiv.org/abs/2511.17602 — Hierarchical Contamination Detection F1=0.76 at semantic level; grounds default cosine threshold 0.85
- https://arxiv.org/abs/2403.04132 — Chatbot Arena ICML 2024 crowd preference; grounds Task 10 real-user-traffic tier
- CLAUDE.md "Verify semantics, not just 'test passed'" — grounds Task 4 reference + broken solution duality
- CLAUDE.md anti-stale-training — blake3 only; no .unwrap() in lib code; grounds stub_ban
- CLAUDE.md anti-slopsquatting — embedding model deps (BGE-M3 / Nomic) verified via WebFetch HF model card before adoption

## Open questions / risks

- Open Q1 (from report): what embedding-cosine threshold catches paraphrases without false-positive-rejecting genuinely novel tasks? arXiv:2511.17602 reports F1=0.76 at semantic level. ELAI needs to measure ITS own corpus precision/recall at thresholds 0.80, 0.85, 0.90. Mitigation: ship at 0.85 default; first session measures and tunes; record in E-54-decisions.md.
- Open Q2 (from report): does generator-rotation diversity matter at 500-task scale? Phi-4 result is at 400B tokens; SFT-corpus scale unknown. Measure: same 500-task corpus from 1 vs 3 vs 5 generators, score variance on local-LLM perf. Out of scope for E-54 ship; documented for follow-up.
- Open Q3 (from report): matched-pair framing detector false-positive on non-deceptive prompt sensitivity? Anthropic data is on frontier models; Qwen2.5-Coder-3B may show stochastic gap without deception. Mitigation: ship with 10pp threshold; if false-positive rate >X%, raise threshold or add p-value gate over N trials.
- Open Q4 (from report): does Windows NTFS handle 5K .toml files in single dir before sharding becomes mandatory? Mitigation: per-family shard + daily date sub-shard caps per-dir count well below NTFS perf cliff. Task 2 includes a benchmark harness; first session measures actual perf at 1K, 5K, 10K under the shard layout.
- Open Q5 (from report): ConStat performance-based check executable without per-task ground truth at scale? Paper requires reference performance. Mitigation: Task 8 ships a simpler 2-sigma-deviation detector as fallback if ConStat reference-performance computation is impractical; documented in E-54-decisions.md.
- Risk 1: BGE-M3 / Nomic Embed v2 dep addition needs explicit user confirmation per CLAUDE.md "No new dependencies without confirmation". Mitigation: feature-gate the embedding-cosine filter behind a `cosine-filter` Cargo feature (default ON in eval-corpus crate, opt-out via `--no-default-features`); user pre-confirms during merge.
- Risk 2: sandbox for Task 4 test executor may not exist. Mitigation: if `crates/sandbox/` absent, Task 4 ships a noop sandbox stub registered under stubs.md with babystep id; sandbox lands in a follow-up patch. Per CLAUDE.md stub registry rule.
- Risk 3: D-6 / D-7 / D-23 / D-24 are PLANNED-not-shipped — E-54 may collide with the eventual D-6 implementation. Mitigation: E-54 ships the eval-corpus crate as the canonical implementation; D-6 PLANNED file gets updated to reference E-54 as its concrete implementation in workflow/reasoning/E-54-decisions.md.
- Risk 4: dynamic knowledge-QA (Task 9) requires SearXNG running on the remote sidecar (per CLAUDE.md "SearXNG (start before sidecar)"). If SearXNG is down, knowledge-qa family falls back to TOML-replay with a warning row `dynamic_knowledge_qa_searxng_unavailable`. Documented in E-54-decisions.md.
- Risk 5: KOR-orthogonality LoRA training (Task 5) requires GPU time. Mitigation: cap at one tiny LoRA per (A,B) pair at canary build time (10 pairs total for 5 families) — wall-clock <1h on existing sidecar; cache the orthogonality matrix to `held-out/_audit/kor_matrix_{date}.json` with daily refresh.
- Risk 6: prod-traffic grade tier (Task 10) data-privacy implications. Mitigation: prod-traffic is OPT-IN with explicit user consent flag; injection logs are written under `held-out/from-prod/` with content hash + timestamp ONLY (no PII); user can purge with `elai eval-corpus purge-prod` command.
- Risk 7: matched-pair framing (Task 6) needs the model invoked TWICE per probe — doubles inference cost. Mitigation: budget cap (default 10 matched-pair probes per session); auditor only invokes when D-23 task tagged for sycophancy.

## Next

E-54 repairs D-6's 9-gram-only filter BEFORE deployment, replacing it with 13-gram + embedding-cosine + per-family shards + test-triplet schema + matched-pair framing + KOR-orthogonality check. After E-54, every ingested task carries cutoffs, executable tests, provenance; held-out corpus survives the 5K-task NTFS perf cliff via per-family directory shards; the 5 D-7 families are knowledge-orthogonal-verified instead of named-only; D-23 sycophancy probes ship in matched-pair eval-vs-deploy framing per the now-measured Sonnet 4.5 evaluation-awareness behavior. The DAFNYCOMP-style "model writes spec AND proof" of corpus design (where a single LLM-generator produces all tasks) is hard-guarded by generator-rotation min-30%. Downstream: E-55 (agentic RL) consumes the held-out signal as the natural reward primitive for SFT-on-rollouts; D-23/D-24 PLANNED surfaces become PROMOTED-TO-SHIPPED once they consume the eval-corpus crate's surface. Open follow-ups: (i) tune embedding-cosine threshold on ELAI's actual corpus (Open Q1); (ii) measure NTFS perf at 1K/5K (Open Q4); (iii) measure matched-pair false-positive rate on Qwen2.5-Coder-3B (Open Q3); (iv) ConStat vs 2-sigma fallback decision (Open Q5); (v) generator-rotation diversity gain at 500-task scale (Open Q2). Patch E-54.1 candidate: dynamic knowledge-QA expansion once SearXNG stability is measured. Patch E-54.2: full ConStat implementation once reference-model snapshot strategy is decided.
