repomap: Agentless (OpenAutoCoder/Agentless) — METHOD-focused map for ELAI steal analysis

Source: C:\Users\user\Downloads\new\Agentless-main (~71K tokens, 35 files). Paper arXiv:2407.01489 "Agentless: Demystifying LLM-based Software Engineering Agents" (Xia, Deng, Dunn, Zhang, 2024). License MIT. Verified by reading the real source, not training memory; numbers below are from the repo's README/README_swebench/cost.py.


1. WHAT IT IS

Agentless is a deliberately NON-agentic pipeline for automatically resolving GitHub issues (SWE-bench). The "agentless" thesis: instead of letting an LLM autonomously plan, decide tools, and navigate the repo in an open loop (the agent paradigm), use a FIXED, deterministic three-phase process where the LLM is only ever asked narrow, bounded sub-questions and ALL control flow is hard-coded Python. The three phases are Localization -> Repair -> Patch Validation.

Why simple beats agentic (the paper's argument, reflected in the code):
- Agentic systems hand the LLM control over tool selection and multi-step planning. That control is where they fail: the LLM gets lost in self-invented exploration, the action space is huge, costs blow up, and behavior is non-reproducible.
- Agentless removes ALL autonomous decision-making. The LLM never decides "what to do next"; the Python harness decides. The LLM only answers: "which files?", "which functions?", "which lines?", "produce a patch for THIS context", "which tests to skip?", "write a reproduction test". Each is a single, stateless, retryable query.
- Reliability comes from SAMPLING + DETERMINISTIC SELECTION, not from a smarter loop. Generate many candidate patches cheaply, then use an un-gameable external signal (test execution) plus majority voting to pick one.
- The bulk of the system is plain Python: AST/CST parsing, diff parsing, normalization, voting, subprocess test runners. The LLM is a bounded sub-component, not the controller.


2. STRUCTURE MAP (modules + one-line role)

Entry / phase drivers:
- agentless/fl/localize.py — CLI driver for all 3 localization stages (--file_level / --related_level / --fine_grain_line_level / --merge / --irrelevant). Thread-pooled per-instance, skip-existing, retry-with-higher-temperature on invalid locations.
- agentless/fl/FL.py — LLMFL class: holds every localization PROMPT TEMPLATE and the per-stage LLM call methods (localize, localize_function_from_compressed_files, localize_line_from_coarse_function_locs, raw-text ablations). MAX_CONTEXT_LENGTH=128000; max_tokens=300 for loc output.
- agentless/fl/retrieve.py — embedding-based file retrieval driver (complements LLM file localization).
- agentless/fl/Index.py — EmbeddingIndex over the repo (llama-index, OpenAI text-embedding-3-small).
- agentless/fl/combine.py — merges LLM-predicted suspicious files with retrieval files via a Counter (most_common) into the final file list.
- agentless/repair/repair.py — Repair phase: builds top-N localized context, samples patches (1 greedy + N-1 @ temp 0.8), post-processes into git diffs with syntax+empty-line filtering.
- agentless/repair/rerank.py — Patch selection: normalize -> load regression+reproduction results -> majority voting -> emit final all_preds.jsonl.

Validation / test phase:
- agentless/test/select_regression_tests.py — LLM removes tests that SHOULD change after the fix; remaining passing tests become the regression set.
- agentless/test/run_regression_tests.py — runs the selected regression set against each candidate patch (SWE-bench harness).
- agentless/test/generate_reproduction_tests.py — generates+normalizes+selects a reproduction test that prints "Issue reproduced/resolved/Other issues"; majority-votes one test per issue.
- agentless/test/run_reproduction_tests.py — executes reproduction tests (on original repo to VERIFY the test reproduces; on patched repo to verify the FIX).
- agentless/test/run_tests.py — shared SWE-bench execution wrappers.

Utilities (the deterministic bulk):
- agentless/util/postprocess_data.py — THE deterministic core: diff/edit-command parsing (3 formats), fake_git_repo (diff via temp git), normalize_patch (AST-normalize + strip comments/docstrings), check_syntax, is_just_new_function, extract_locs_for_files, line-interval overlap math.
- agentless/util/compress_file.py — get_skeleton: libcst transformer that replaces function bodies with "..." (the "skeleton" view for related-element localization); optional long-assignment compression.
- agentless/util/preprocess_data.py — repo-structure parsing, transfer_arb_locs_to_locs (class/function/line names -> concrete line intervals + context window), line_wrap_content (numbered/sticky-scroll context rendering), file filters.
- agentless/util/model.py + api_requests.py — make_model abstraction over openai/anthropic/deepseek backends; codegen / codegen_w_tool (Anthropic str_replace tool); prompt_cache flag.
- agentless/util/index_skeleton.py, parse_global_var.py — supporting parse helpers.
- get_repo_structure/get_repo_structure.py — clones+checks out base_commit, parses every .py into files/classes/functions structure dict.
- get_repo_structure/get_patch_info.py — parse_patch (used by normalize).
- dev/util/cost.py — token+dollar accounting from traj fields.
- classification/ — manual SWE-bench-lite difficulty classifications + a graph-based classifier (research artifact, not part of the pipeline).


3. ARCHITECTURE + METHOD (detailed)

PHASE 1 — HIERARCHICAL FAULT LOCALIZATION (file -> element -> edit-location)

The whole design principle is coarse-to-fine narrowing so the model is never shown the whole repo and never the whole file at the fine stage. Three LLM stages, each consuming the previous stage's output.

Stage 1a — File-level (LLMFL.localize, FL.py:313):
- Input: problem_statement + show_project_structure(repo tree, files filtered to .py, test files removed). Prompt obtain_relevant_files_prompt (FL.py:29) asks for "at most 5 files, full path, ordered most-to-least important, wrapped in ```". Temperature 0, max_tokens 300.
- Output parsed line-by-line; correct_file_paths maps model strings to real repo paths preserving model order.

Stage 1b — Embedding retrieval complement (retrieve.py + Index.py):
- Separately, an LLM lists IRRELEVANT folders (obtain_irrelevant_files_prompt, FL.py:51, localize_irrelevant) to prune the retrieval space, then OpenAI embedding retrieval (text-embedding-3-small, chunk 512) ranks remaining files.
- combine.py fuses LLM-file list (top_n) + retrieval-file list (top_n) with a Counter; files appearing in both float to the top (most_common). Result = final found_files.

Stage 2 — Related-element localization (localize_function_from_compressed_files, FL.py:363):
- Takes top_n files. Renders each as a SKELETON via get_skeleton (compress_file.py): libcst replaces every function body with "..." and strips docstrings, keeping signatures, class structure, and (optionally) top-level constants. Long global assignments can be elided (compress_assign).
- Prompt obtain_relevant_functions_and_vars_from_compressed_files_prompt_more (FL.py:152) asks for a set of locations as class/function/variable NAMES (not lines yet). Output is per-file lists of "function:/class:/variable:" lines via extract_locs_for_files.
- Robustness: localize_instance retries up to MAX_RETRIES=5; starts at temp 0, escalates to temp 1.0 if check_contains_valid_loc returns nothing (a name that doesn't resolve in the structure).
- If message exceeds 128K tokens, it drops the last file and rebuilds (FL.py:407) — deterministic context-fitting, never a smart summarizer.

Stage 3 — Edit-location (line-level) localization (localize_line_from_coarse_function_locs, FL.py:546):
- Takes the stage-2 element names, resolves them to concrete line intervals (transfer_arb_locs_to_locs), and builds top-N context windows: construct_topn_file_context (repair.py:226) renders ONLY the localized intervals +/- context_window lines, with line numbers (line_wrap_content, optional sticky_scroll showing enclosing def/class headers).
- Prompt obtain_relevant_code_combine_top_n_prompt (FL.py:88) asks for exact "line:" numbers (and class/function). SAMPLED: num_samples=4 at temperature 0.8 to get 4 DIFFERENT sets of edit locations. Retry-on-invalid as above. prompt_cache=True reuses the prompt prefix across the 4 samples (cost lever).
- localize.py --merge (localize.py:469) then splits the 4 samples into 4 separate loc files (loc_merged_0-0 .. 3-3). Each becomes an INDEPENDENT repair run -> diversity at the localization level, not just temperature.

Key deterministic helpers in localization:
- get_skeleton (CST body-elision) — the compression that lets whole files fit in context as signatures.
- check_contains_valid_loc — validates predicted names/lines actually exist in the repo structure; gate for the retry loop.
- 128K context-fit by dropping trailing files — no LLM in the loop.


PHASE 2 — REPAIR (sampling + patch generation + normalization)

Driver repair.py::process_loc (repair.py:272). For each instance:
- Build context: top_n files, but ONLY the localized edit intervals +/- context_window (default 10) lines, via construct_topn_file_context. If no locations -> emit empty patch and skip (deterministic short-circuit).
- Prompt: one of 4 templates chosen by flags (repair.py:384):
  * repair_prompt_combine_topn (edit_file(filename,start,end,content) function-call format),
  * _cot (same + "first localize then edit" chain-of-thought),
  * _cot_diff (SEARCH/REPLACE blocks, aider-style — the production format, --diff_format),
  * _cot_str_replace (Anthropic str_replace tool, --str_replace_format, anthropic-only).
- SAMPLING (repair.py:404): 1 greedy sample (temp 0) + (max_samples-1) samples at temp 0.8. With prompt_cache=True for the later samples. Per the README, 4 localization sets x 10 samples = 40 candidate patches per bug.
- Post-process EACH raw output (_post_process_multifile_repair, repair.py:151): extract code blocks -> split into per-file edit commands (split_edit_multifile_commands) -> apply with the format-specific parser:
  * parse_diff_edit_commands (SEARCH/REPLACE; applies edits from END of file backwards so earlier line numbers stay valid; handles "..." elision search blocks),
  * parse_str_replace_edit_commands (Anthropic tool semantics, also back-to-front),
  * parse_edit_commands (edit_file() function calls; sorts subcommands by start line descending, eval()s the content arg with fallbacks for triple-quote strings).
- Materialize the patch as a real git diff via fake_git_repo (postprocess_data.py:107): write old content into a throwaway git repo, write new content, `git diff` it. This guarantees a valid unified diff that git apply will accept.
- DETERMINISTIC PATCH FILTERS (post_process_raw_output, repair.py:568): check_syntax (ast.parse must pass on every new file) AND check_code_differ_by_just_empty_lines (reject no-op whitespace patches). Failing either -> patch set to "" (not even evaluated). Cheap pre-filter before any test execution.

NORMALIZATION (normalize_patch, postprocess_data.py:408) — the de-duplication key:
- ast.parse + ast.unparse both old and new file content (canonicalizes formatting), then remove_comments_and_docstrings, then re-diff via fake_git_repo. is_just_new_function: if the change only ADDS functions, strip context lines from the diff. Result is a semantically-canonical patch string used purely as a dedup/voting key — two patches that differ only in whitespace/comments/added-fn-context collapse to one. This is what makes majority voting meaningful.


PHASE 3 — PATCH VALIDATION + SELECTION (regression + reproduction + rerank)

Three deterministic signals feed the final vote.

3a — Regression test selection (select_regression_tests.py):
- First run_regression_tests.py finds ALL tests that PASS on the unpatched repo (note: deliberately NOT using SWE-bench's provided PASS_TO_PASS field — they re-derive it).
- LLM is then asked (select_tests_prompt_template) which of those tests SHOULD be expected to change/break because of the fix, and those are EXCLUDED. Remaining = the regression set that a correct patch must keep green. (If the test list exceeds 128K tokens it is halved deterministically.)
- Each candidate patch is run against this set; a patch's "regression_test_result" = number of failing tests.

3b — Reproduction test generation (generate_reproduction_tests.py):
- LLM generates a standalone test from the issue text that prints exactly "Issue reproduced" / "Issue resolved" / "Other issues" (generate_tests_prompt_template:18, with a worked example). 40 samples (1 greedy + 39 @ temp 0.8).
- Each generated test is wrapped into a patch (create_patch_from_code adds reproduce_bug.py) and EXECUTED ON THE ORIGINAL (unpatched) repo (run_reproduction_tests.py --testing). A test is "verified" only if it actually reproduces the bug on the original code — i.e. the test itself is validated by execution before it is trusted. This is the un-gameable filter on test quality.
- normalize_test (ast.unparse + strip docstrings + rename the test fn to a generic name) gives a dedup key; test_selection majority-votes ONE verified reproduction test per issue.
- The chosen reproduction test is then run on each PATCHED candidate; "reproduction_test_result" = did the patch make the test report resolved.

3c — Reranking / final selection (rerank.py::majority_voting, rerank.py:156):
Deterministic tiered selection per instance:
- Candidate filter: keep patches that are non-empty AND have minimal regression failures (regression_tests == min) AND pass the reproduction test. If that set is empty, relax to "non-empty + min regression". If still empty, vote over all non-empty normalized patches.
- Within the surviving set, MAJORITY VOTE on the normalized_patch key (Counter); tie-break by EARLIEST first-appearance index (max over (vote_count, -first_appear_idx)). The raw (un-normalized) patch of the winning normalized key is submitted.
- Output all_preds.jsonl (one model_patch per instance_id) — directly SWE-bench-evaluable.

THE DETERMINISTIC FLOW (the bulk): every arrow between phases is a jsonl file on disk; every stage is idempotent (skip_existing / "patch already generated" checks) and thread-pooled per instance. The only LLM touches are: file list, irrelevant folders, element names, 4x edit-location sets, patch samples, regression-exclusion list, reproduction-test samples. Everything else — context construction, skeletonization, diff parsing/application, syntax/whitespace filtering, normalization, test execution, voting — is pure Python with no model in the loop.


4. KEY TECHNIQUES PORTABLE TO ELAI

Mapping note: ELAI's loop is explorer(localization) / executor(repair) / auditor(validation). Agentless's three phases line up almost 1:1.

T1. HIERARCHICAL COARSE-TO-FINE LOCALIZATION with SKELETON COMPRESSION.
   Files: agentless/fl/FL.py (3 stages), agentless/util/compress_file.py::get_skeleton, agentless/util/preprocess_data.py::transfer_arb_locs_to_locs.
   -> ELAI explorer (integration-explorer-elai). ELAI already has a structural code graph (elai.map, codegraph-elai) which is STRONGER than Agentless's textual repo-tree + libcst skeleton. The portable idea is the STAGED prompt narrowing: file -> named element -> exact line interval, each stage feeding concrete line intervals +/- a fixed context_window into the next. The skeleton trick (replace fn bodies with "...") is a cheap way to show a whole file's shape in-context to a small local model (Qwen2.5-Coder) — directly useful for ELAI's local lane where context is the binding constraint (25-28K). Fully portable; ELAI can substitute its code graph for the libcst skeleton.

T2. SAMPLE-MANY + NORMALIZE + DETERMINISTIC SELECTION (no agentic loop).
   Files: agentless/repair/repair.py (1 greedy + N@0.8), agentless/util/postprocess_data.py::normalize_patch, agentless/repair/rerank.py::majority_voting.
   -> ELAI executor + auditor. The whole "generate 40 candidates, AST-normalize to a dedup key, then pick by external test signal + majority vote with earliest-appearance tie-break" is the single biggest steal. It replaces "trust one LLM attempt" with "cheap diversity + un-gameable selection". normalize_patch (ast.unparse + strip comments/docstrings + re-diff) as a SEMANTIC dedup key is portable verbatim for Python and is exactly the kind of off-agent, deterministic artifact ELAI's wiring-enforcement doctrine wants. Maps cleanly onto ELAI's "creator-verifier" framing. Highly portable.

T3. EXECUTION-GROUNDED VALIDATION: self-verified reproduction tests + LLM-pruned regression set.
   Files: agentless/test/generate_reproduction_tests.py (test must reproduce on ORIGINAL repo to be trusted), agentless/test/select_regression_tests.py (LLM removes tests expected to change), agentless/repair/rerank.py (tiered filter: min-regression AND reproduction-pass).
   -> ELAI auditor. The key insight ELAI should steal: a generated test is only trusted AFTER it is executed and shown to fail on the buggy code (validates the validator). This is precisely ELAI's "measure an artifact the agent did not author / missing-evidence-is-FAIL" principle, applied to issue-resolution. The reproduction-test print-protocol ("Issue reproduced/resolved/Other issues") is a clean, parseable oracle. Portable; needs ELAI to have a sandbox runner (it has the sidecar + SWE harness interest).

Secondary portable bits:
- fake_git_repo / fake_git_apply (postprocess_data.py): generate/apply diffs via a throwaway git repo to guarantee git-apply-valid unified diffs. Trivial, robust, portable (ELAI executor patch emission).
- Cheap pre-execution filters: check_syntax (ast.parse) + check_code_differ_by_just_empty_lines reject obviously-dead patches before any expensive test run. Portable as an auditor fast-path.
- Retry-with-rising-temperature on structurally-invalid output (check_contains_valid_loc gate, temp 0 -> 1.0). Portable as a generic "valid-or-resample" wrapper.
- prompt_cache reuse across samples sharing a prefix (repair.py, FL.py): direct cost lever for ELAI's cloud lane caching work.
- Deterministic 128K context-fit by dropping trailing files (no summarizer LLM): portable, aligns with ELAI's local-lane context policy.

NON-PORTABLE / flag:
- SWE-bench-specific glue: load_dataset(princeton-nlp/SWE-bench_*), get_repo_structure clone+checkout per instance, the run_tests SWE-bench docker harness — these are benchmark scaffolding, not method. ELAI would replace with its own repo + sandbox.
- Python-ONLY assumptions: get_skeleton (libcst), normalize_patch (ast.parse/unparse), remove_comments_and_docstrings (tokenize), check_syntax — all Python-AST-bound. ELAI is Rust+TS+Python; the normalization/skeleton/syntax-filter techniques need per-language reimplementation (tree-sitter etc). The ALGORITHM ports; the implementation does not.
- eval() on model-produced edit_file() content args (parse_edit_commands, postprocess_data.py:849) — convenient but an injection/robustness hazard; ELAI should prefer the SEARCH/REPLACE or str_replace formats (which it parses without eval).
- Embedding retrieval (OpenAI text-embedding-3-small) as the file-localization complement — ELAI's code graph supersedes this; not worth porting.
- "Re-derive PASS_TO_PASS instead of trusting the benchmark field" is a SWE-bench evaluation-integrity choice, not generally applicable.


5. BENCHMARK NUMBERS (from the repo, README.md / README_swebench.md)

- SWE-bench Lite, GPT-4o (v1.0, July 2024): 82 fixes = 27.3% solve rate, avg $0.34 per issue. Stated as "best open-source approach on SWE-bench lite" at release.
- SWE-bench Lite, Claude 3.5 Sonnet (Dec 2024): 40.7% solve rate.
- SWE-bench Verified, Claude 3.5 Sonnet (Dec 2024): 50.8% solve rate.
- Cost model used in dev/util/cost.py: prompt $5/M tokens, completion $15/M tokens (GPT-4o pricing); embeddings $0.02/M (cost.py:37, :54). Avg-cost = total/num_instances.
- Default sampling budget producing these: 4 edit-location sets x 10 repair samples = 40 patches/issue; 40 reproduction-test samples/issue.
Note: numbers are self-reported in the repo README; not independently re-run here.


6. DEPENDENCIES / STACK

Python 3.11 (conda env). requirements.txt: anthropic, openai, pre-commit, datasets (HF), tiktoken (token counting), libcst (skeletonization), llama-index (embedding retrieval), jsonlines, swebench (@ git+princeton-nlp/SWE-bench, the execution harness). Backends supported: openai / anthropic / deepseek. Models wired in code: gpt-4o-2024-05-13, gpt-4o-mini-2024-07-18, claude-3-5-sonnet-20241022, deepseek-coder. Embeddings: OpenAI text-embedding-3-small. Concurrency: stdlib ThreadPoolExecutor + Lock; all inter-stage state is jsonl on disk. Test execution: subprocess + SWE-bench harness (docker-based per swebench). No agent framework, no tool-routing library — that absence is the point.
