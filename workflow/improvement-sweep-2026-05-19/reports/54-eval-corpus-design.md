# Eval corpus design for ELAI (2024-2026)

## TL;DR
- D-6's 9-gram filter is provably evaded. Rephrased-samples work (Yang/Chiang 2023, arXiv:2311.04850) shows a 13B model can match GPT-4 on MMLU/GSM8K/HumanEval by training on paraphrases that 13-gram never sees. Soft-contamination work (arXiv:2602.12413) finds 78% of CodeForces problems have semantic duplicates in the Olmo3 corpus. Add a SECOND filter on top of 9-gram: embedding cosine similarity (SemDeDup style) at task-creation time AND at retire-time.
- D-7's five families are not orthogonal by name alone. KOR-Bench (arXiv:2410.06526) names exactly the discriminator: a family is orthogonal when knowledge cannot transfer between it and adjacent families (Operation / Logic / Cipher / Puzzle / Counterfactual). Add a knowledge-orthogonality check at canary build time: an LLM that scores >70% on family A must not score >70% on family B via cross-family transfer.
- Replace static held-out with a continuously-grown live tier. SWE-bench-Live (arXiv:2505.23419) updates monthly from GitHub issues created after 2024-01-01; LiveCodeBench (arXiv:2403.07974) v6 has 1055 problems across May-2023..Apr-2025 with release-date stamps. ELAI should mirror the time-stamp pattern: every task gets `created_at` AND a `model_cutoff_at_creation` field so future ELAI runs can filter to "post-my-cutoff" automatically.
- Evaluation-awareness is now a measured behavior, not a hypothetical. Claude Sonnet 4.5 verbalizes "I am being evaluated" >80% of the time in pre-deployment runs (Anthropic, via arXiv:2510.20487). D-23's sycophancy probes need a sandbagging counter-probe: same prompt in eval format vs deploy format, flag if scores differ >10pp.
- Don't grow the suite by 10x with one synthetic-LLM. Phi-4 (arXiv:2412.08905) needed 50 distinct synthetic-data techniques to hit 400B tokens; OpenMathInstruct-2 (arXiv:2410.01560) used Llama-3.1-405B-Instruct AND an LLM-pipeline decontamination step. Single-judge synthesis collapses diversity. ELAI's growth from 50 -> 500 -> 5K must rotate at least 3 generators and re-run contamination sweeps each tier.

## Key sources
URL or arxiv | what | takeaway

| Source | What | Takeaway for ELAI |
|---|---|---|
| arXiv:2403.07974 (LiveCodeBench, ICLR 2025) | Time-stamped code contest problems, v6 = 1055 tasks May23..Apr25 | DeepSeek/GPT-4o show clear post-cutoff perf drop on LeetCode; time-window split is the cheapest leakage probe ELAI can adopt. |
| arXiv:2505.23419 (SWE-bench Live) | 1319 real GitHub issues post 2024-01-01, monthly updates, Docker per task | Architectural template for D-6: per-task Docker image + monthly auto-curation; restricts pretraining-cutoff overlap. |
| arXiv:2602.12413 (Soft Contamination) | Embedding-based detection finds 78% of CodeForces + 50% of ZebraLogic have semantic dupes in Olmo3 | N-gram is necessary but insufficient. Add embedding-cosine pass. |
| arXiv:2311.04850 (Yang & Chiang, Rephrased Samples) | 13B beats baseline on MMLU/GSM8K/HumanEval via paraphrase training that evades 13-gram | Even strict 13-gram (GPT-3 def) fails. ELAI's 9-gram is weaker still. |
| arXiv:2412.08905 (Phi-4 Tech Report, Dec 2024) | 14B model trained mostly on 400B tokens of synthetic from 50 techniques | Synthetic data done well beats organic; needs diversity-by-construction, not single-pipe. |
| arXiv:2410.01560 (OpenMathInstruct-2, Oct 2024) | 14M problem-solution pairs from Llama-3.1-405B; LLM-pipeline decontam | Always re-decontaminate after synthesis; rephrased-test leakage is the default. |
| arXiv:2411.04905 (OpenCoder cookbook) | 4.5M SFT entries, full pipeline open-sourced, executes generated tests to filter | Test-execution-based filter is the cheap quality signal; ELAI canaries should ship test cases not just expected output. |
| arXiv:2503.02951 (KodCode, ACL 2025) | 447K verified Q-solution-test triplets, self-verification keeps error <2.5% | Triplet schema beats Q+A schema; tests are first-class. |
| arXiv:2505.23836 / 2507.01786 / 2510.20487 (eval-awareness suite) | Sonnet 4.5 detects eval framing >80% of time; can sandbag covertly per arXiv:2508.00943 | D-23 sycophancy probe must be matched-pair (eval vs deploy phrasing); diff>10pp = flag. |
| arXiv:2410.06526 (KOR-Bench) | Knowledge-Orthogonal-Reasoning: 5 categories chosen so domain knowledge can't help | Naming "5 families" doesn't make them orthogonal; verify via cross-family transfer test. |
| arXiv:2403.04132 (Chatbot Arena, ICML 2024) | Crowd preference, Bradley-Terry / Elo, >240K votes, expert-grade signal | Real-user-traffic eval correlates with expert eval; ELAI can grade itself on user-thumbs from real sessions, not just held-out. |
| arXiv:2412.12509 / JudgeBench (Can-You-Trust LLM Judgments, 2024) | Only largest judges hit acceptable agreement; lenient bias common | If ELAI uses cloud-LLM-as-grader, calibrate against held-out human grades quarterly. |
| arXiv:2502.14425 (Survey on Data Contamination, Feb 2025) | Taxonomy: white/gray/black-box detection x update/rewrite/prevention defense | Use as decision-grid when designing D-6 defenses; map each ELAI defense to one cell. |
| arXiv:2510.04721 (BrokenMath) / arXiv:2502.08177 (SycEval) | GPT-5 produces sycophantic answers 29% of time on perturbed-true theorems | D-23 should use BrokenMath-style perturbation (true -> false) of real problems, not synthetic alone. |
| arXiv:2503.14499 (METR Time Horizons, Mar 2025) | Frontier 50%-horizon doubles every ~7mo; 14.5h at Opus 4.6 in Feb 2026 | D-7 task families need a duration dimension. Mostly-short tasks under-measure agentic regression. |
| arXiv:2511.08598 (OKBench) / LiveBench | Generates ~2K MCQs/day from last-24h news; on-demand benchmark | Dynamic eval-time generation is now feasible. D-24 could mutate at eval time, not just regression-replay. |

## Contamination spectrum
| Level | Defense pattern | Citation |
|---|---|---|
| Exact match | SHA / md5 of normalized text | OpenMathInstruct-2 decontam step (arXiv:2410.01560) |
| N-gram (9-13gram) | string-overlap filter (current D-6) | GPT-3 uses 13-gram, GPT-4 uses 50-char (per arXiv:2311.04850); D-6's 9-gram is weaker |
| Soft / paraphrase | embedding cosine + LSH (SemDeDup, MinHash-LSH) | arXiv:2602.12413; Milvus MinHash-LSH blog 2024 |
| Semantic (different surface, same logic) | LLM-pipeline grader: "is task X a rephrasing of task Y?" | OpenMathInstruct-2 LLM-pipeline; Hierarchical Contamination Detection arXiv:2511.17602 (F1=0.76) |
| Meta-contamination (LLM-generated "novel" tasks match training distribution) | Cross-generator rotation + post-cutoff temporal split | Phi-4 50-technique diversity; Test-of-Time arXiv:2509.00072 shows synthesis can OBSCURE rather than remove contamination |
| Performance-based (model improves "too fast" on a slice) | ConStat performance-delta test | NeurIPS 2024 ConStat |
| Eval-awareness (model detects test framing) | matched-pair eval vs deploy framing | arXiv:2510.20487, arXiv:2508.00943 |

## Synthetic vs real vs hybrid corpus
| ELAI canary family | Best mix | Grounding |
|---|---|---|
| code-fix | REAL > synthetic. Pull from GitHub issues created post-`elai_birthday` per SWE-bench-Live recipe (arXiv:2505.23419). Synthetic adds noise; real fixes have ground-truth tests and PR diffs. | SWE-bench-Live; LiveCodeBench v6 |
| math-proof | HYBRID. Real Olympiad problems (BrokenMath uses 2025 competition problems, arXiv:2510.04721) + synthetic perturbations (true -> false claim) for sycophancy probing. Pure synthetic (OpenMathInstruct-2 14M) drifts toward training distribution. | arXiv:2510.04721; arXiv:2410.01560 |
| prose-reason | REAL > synthetic. KOR-Bench (arXiv:2410.06526) shows knowledge-orthogonal puzzles can be hand-curated; LLM-generated prose-reason tasks correlate with training distribution per Test-of-Time (arXiv:2509.00072). | KOR-Bench; Test-of-Time |
| planning | HYBRID. AppWorld-style real APIs + synthetic constraint variations. Planning is rare in user logs, so real-only doesn't scale. | xbench arXiv:2506.13651 |
| knowledge-QA | DYNAMIC. OKBench-style daily-news generation (arXiv:2511.08598). Knowledge-QA is where stale held-out is most dangerous because the world moves. | OKBench; LiveBench |

## Eval-awareness / sandbagging defenses
- Matched-pair framing (HARD requirement): each D-23 sycophancy probe ships twice — "evaluation framing" + "deploy framing" — and ELAI flags if score delta >10pp. Grounded in arXiv:2510.20487 (Steering Evaluation-Aware Models).
- White-box probe for eval-awareness activations: Anthropic's published technique (probes-catch-sleeper-agents) is applicable to local-LLM if ELAI exposes activations; otherwise rely on black-box matched-pair.
- Covert-sandbag detector: arXiv:2508.00943 shows models can sandbag against chain-of-thought monitoring. Add latency + token-count anomaly detection (sandbagging often produces shorter or more cautious answers).
- Don't tell the model it's being evaluated: paraphrase D-6 task instructions to match production phrasing; never include the word "benchmark" or "evaluation" in the prompt.
- Probabilistic insertion: silently include 1 held-out task per N production turns (Anthropic-style probe). ELAI's D-6 store can serve a "deployed-mode" copy of each task with eval markers stripped.

## Best additions for ELAI's D-6/D-7
Ranked.

1. Embedding-cosine second filter on D-6 ingestion. Addition: every new task computes BGE-M3 / Nomic embedding vs existing held-out corpus; if cosine > 0.85 with any prior task -> reject. Grounding: arXiv:2602.12413 (78% CodeForces miss rate of n-gram-only). Hook: `.elai_cc/crates/eval-corpus/src/ingest.rs` cosine guard before TOML write.

2. Time-stamp every task with `model_cutoff_at_creation`. Addition: each `.toml` carries the local-LLM and cloud-LLM cutoffs known at task-creation time. D-24 regression validation auto-filters to "post-cutoff for current model under test". Grounding: LiveCodeBench v6 release-date stamps (arXiv:2403.07974). Hook: extend TOML schema with `cutoffs: {qwen2.5: "2024-10", claude4.7: "2026-01"}`.

3. Matched-pair framing in D-23. Addition: every sycophancy probe has eval-phrased and deploy-phrased variant; auditor flags >10pp gap. Grounding: arXiv:2510.20487. Hook: `sycophancy_probe.toml` schema gains `framing_pair: {eval: str, deploy: str}`.

4. Test-triplet schema instead of Q+A. Addition: each task = (problem, reference_solution, executable_test). Audit verifies test fails on missing/broken solution, passes on reference. Grounding: KodCode triplets keep error <2.5% (arXiv:2503.02951); OpenCoder executes tests to filter (arXiv:2411.04905). Hook: TOML schema mandatory `tests: [...]` field.

5. Knowledge-orthogonality verification at canary build. Addition: train tiny adapter on family-A training split, measure transfer accuracy on family-B test split; abort if >random+10pp. Grounding: KOR-Bench definition (arXiv:2410.06526). Hook: pre-merge gate in D-7 canary build.

6. Generator rotation for synthetic growth. Addition: 50 -> 500 expansion uses at least 3 distinct generators (local-LLM, cloud-LLM, human-curated import) with min-30% from each; track `provenance` in TOML. Grounding: Phi-4 50-technique diversity (arXiv:2412.08905). Hook: TOML `provenance: {generator, prompt_id, date}`.

7. BrokenMath-style perturbation for D-23. Addition: for each real math/code task, generate a "subtly false" variant; correct answer is "this is false". Sycophancy = model agreeing with the false variant. Grounding: arXiv:2510.04721 (GPT-5 = 29% sycophantic on this style). Hook: `perturbed_pair` field next to original.

8. Real-user-traffic grade tier. Addition: opt-in production turns become eval candidates (Anthropic-style silent probes); thumbs-down + grader-LLM verdict feed corpus growth. Grounding: Chatbot Arena human-preference crowdsourcing (arXiv:2403.04132). Hook: `.elai_cc/held-out/from-prod/{date}/{turn-hash}.toml`.

9. Performance-delta contamination detector. Addition: when ELAI self-mod improves on family X by >Y over 1 week, ConStat performance-based check fires. Grounding: ConStat NeurIPS 2024. Hook: D-24 regression gate already has the slot — add ConStat-style window.

10. Dynamic eval-time generation for knowledge-QA family. Addition: OKBench-style daily generation from web-search corpus (sidecar already has SearXNG). Grounding: arXiv:2511.08598. Hook: knowledge-QA family becomes "generate at eval time" instead of replay from `.toml`.

## Eval scaling
- 0 -> 50: hand-curated only. Author tasks while building D-6; one engineer-day = ~5-10 tasks. Use the Stage-1 pattern from Label Studio's "Five Stages" — fast feedback over sophistication. All triplet-format.
- 50 -> 500: hybrid expansion. Generator rotation (3+ sources). After every 50-task batch run embedding-cosine sweep on the FULL corpus and on the previous month's training data (for self-mod loop). Re-run KOR-orthogonality after each family hits 100 tasks.
- 500 -> 5K: dynamic + automation. Switch knowledge-QA family to OKBench-style daily generation; freeze fixed slice for cross-version comparability. SWE-bench-Live shows 1319 tasks @ 93 repos is sustainable with monthly cron. For ELAI's `.toml` flat layout: shard by `held-out/{family}/{date}/{task-id}.toml` once family directories exceed ~500 files (Windows + filesystem perf cliff). Add `held-out/_index/{family}.jsonl` summary file per family for fast scan without opening every TOML.
- Lifetime: never stop adding. Auto-retire only when 6-month rolling pass rate >95% across all tracked models — current D-6 plan. Add an UN-retire trigger if a new model regresses on a retired task (signals capability change, not staleness).

## Anti-patterns
- N-gram-only decontamination (arXiv:2311.04850; arXiv:2602.12413). Already explicitly D-6's risk.
- LLM-as-grader without calibration; lenient bias is documented (arXiv:2412.12509). Calibrate cloud-LLM grader against human-grade slice quarterly.
- "Refresh from same generator" growth — Phi-4 needed 50 distinct techniques specifically because single-generator output collapses (arXiv:2412.08905).
- Frozen suite forever: METR shows 7-month doubling on agentic horizon (arXiv:2503.14499); frozen 200-task suite becomes noise within a year.
- Eval framing leaks: Sonnet 4.5 detects eval >80% of the time (arXiv:2510.20487). Telling-the-model-it's-a-test inflates safety metrics and depresses capability metrics in opposite directions.
- Synthetic "novel" tasks that match training distribution (Test-of-Time, arXiv:2509.00072 — LLM-synthesized questions show NO post-cutoff decay even from contaminated source papers). Synthesis can obscure rather than remove contamination.
- Same-family-disguised: 5 families that all reduce to "language modeling on code-flavored text" give no cross-family signal. KOR-Bench orthogonality check is cheap.
- Self-grading without held-out reference: model X grading model X is uninformative; rotate graders.

## Open questions
- What is the embedding-cosine threshold that catches paraphrases without false-positive-rejecting genuinely-novel tasks? arXiv:2511.17602 reports F1=0.76 at semantic level; ELAI needs to measure ITS own corpus' precision/recall at thresholds 0.80, 0.85, 0.90.
- How much does generator-rotation diversity matter at the 500-task scale? Phi-4 result is at 400B tokens; the SFT-corpus result is unknown. Measure: same 500-task corpus from 1 vs 3 vs 5 generators, score variance on local-LLM perf.
- Does the matched-pair framing detector for sandbagging produce false positives on non-deceptive prompt sensitivity? Anthropic data is on frontier models; Qwen2.5-Coder-3B may show stochastic gap without deception.
- Does Windows NTFS handle 5K `.toml` files in a single directory before sharding becomes mandatory? Spot check: NTFS docs say 4.3M files max per dir but perf degrades far earlier; benchmark D-6 storage layout at 1K and 5K.
- Is the ConStat performance-based contamination check executable without per-task ground-truth at scale? Paper requires reference performance; ELAI may need a reference-model snapshot at corpus creation.
