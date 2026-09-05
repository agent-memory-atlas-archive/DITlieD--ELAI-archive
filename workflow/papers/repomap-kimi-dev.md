repomap: Kimi-Dev (MoonshotAI) — SWE-bench issue-resolution coding LLM + agentless scaffold

Source repo: C:/Users/user/Downloads/new/Kimi-Dev-master  (~39 files, ~74K tok)
Tech report: arXiv:2509.23045 "Kimi-Dev: Agentless Training as Skill Prior for SWE-Agents"
Model: moonshotai/Kimi-Dev-72B (HF). License: see LICENSE.md (Moonshot). Mapped 2026-06-15.

=====================================================================
1. WHAT IT IS
=====================================================================
BOTH a model and a (deliberately minimal) scaffold, but the headline artifact is the MODEL.
Kimi-Dev-72B is an open-source 72B coding LLM trained (large-scale RL in Docker, reward = full
test suite passes) for GitHub-issue resolution on SWE-bench Verified. The repo here is NOT the
training code; it is the lightweight INFERENCE scaffold + SWE-bench Docker eval harness used to
roll out and grade the model. The scaffold is a stripped-down fork of Agentless (OpenAutoCoder),
self-described in setup.py as "a lightweight scaffolding for SWE-Bench". Package name: kimidev,
core module: kimidev.agentlessnano (= "Agentless, nano").

The thesis (from the paper title, embodied by the scaffold): a model RL-trained inside a
fixed, dead-simple two-step *agentless* pipeline acquires a "skill prior" that ALSO transfers to
full agentic scaffolds. So the design point is the opposite of most SWE agents: instead of a
clever multi-tool ReAct loop compensating for a weak model, it is a STRONG model under a
near-trivial fixed pipeline (file-localize -> edit), with capability pushed into weights, not scaffold.

Problem solved: given a GitHub issue text + repo, produce (a) a code patch that fixes it and
(b) a reproduction test, verified by the real test suite in a per-repo Docker image.

=====================================================================
2. STRUCTURE MAP (modules, one-line roles)
=====================================================================
kimidev/agentlessnano/        — the inference scaffold (the interesting part)
  prompt_template.py          — all prompts: file-localization, fn/var-localization, line-localization,
                                repair (SEARCH/REPLACE), reproduction-test generation.
  model_api.py                — LLM client abstraction: DecoderBase + OpenAI/Anthropic/DeepSeek/KimiDev
                                decoders; make_model(); KimiDev decoder hits a local vLLM at 127.0.0.1:8000.
                                Also an Anthropic str_replace_editor TOOL-USE agentic decoder (codegen_w_tool).
  post_process.py             — patch synthesis: parse SEARCH/REPLACE blocks, apply edits, produce git diff
                                via "fake git repo", syntax-check (ast), lint-gate (flake8 fatal codes),
                                normalize_patch (strip comments/docstrings/whitespace for dedup).
  testwritter_utils.py        — test-writer helpers: AST-based test-body stripping (replace_test_functions),
                                build a reproduce_bug.py patch from generated test code, normalize_test.
  utils.py (1544 lines)       — Agentless repo-structure machinery: show_project_structure, skeletonization,
                                get_repo_files, construct_topn_file_context, line-interval context windows,
                                parse_patch, file-path correction, GT edit-loc extraction.
kimidev/examples/
  rollout_messages_bugfixer.py    — ENTRYPOINT: bug-fix rollout over SWE-bench Verified (localize -> repair).
  rollout_messages_testwriter.py  — ENTRYPOINT: reproduction-test rollout (localize test file -> insert test).
kimidev/harness/              — standard SWE-bench Docker eval harness (ported from princeton-nlp/SWE-bench)
  test_spec.py                — builds Dockerfile + eval shell script + F2P/P2P test command per instance.
  grading.py                  — applies patch, parses test log, computes FAIL_TO_PASS/PASS_TO_PASS -> resolved.
  log_parsers.py              — per-repo pytest/unittest output parsers (MAP_REPO_TO_PARSER).
  constants.py / dockerfiles.py / utils.py — repo-version constants, Dockerfile templates, helpers.
kimidev/assets/               — requirements.txt / yml templates per repo env (json).
resources/                    — example result jsonl + the 40-patch/40-test test-time-scaling artifacts.

=====================================================================
3. ARCHITECTURE + METHOD (the detail)
=====================================================================

3.1 OVERALL PIPELINE ("agentless, two stage")
The README is explicit and the code confirms it. Per issue:
  Stage 1 FILE LOCALIZATION — feed (issue text + whole-repo STRUCTURE, not contents) to the model,
    ask for "at most 5 files ... ordered most-to-least important, wrapped in ```". This is a SINGLE LLM
    call. Crucially localization is at FILE granularity only — README: "Compared to multi-step
    localization methods, we perform localization at the file level and then pass the COMPLETE FILE to
    the repair step for more detailed reasoning." (i.e. the model does fine-grained localization
    implicitly during repair, not as separate retrieval hops.)
  Stage 2 REPAIR / EDIT — feed (issue text + full contents of the located files) and ask the model to
    "first localize the bug ... then generate *SEARCH/REPLACE* edits". Also a single LLM call.
Two model calls total per fix. No tool loop, no retrieval iteration, no agentic state in the default path.
solve_instance_data() in rollout_messages_bugfixer.py is the whole controller (~90 lines).

3.2 FAULT LOCALIZATION (kimidev/agentlessnano)
  - Repo is pre-processed offline into a "structure" tree (PROJECT_FILE_LOC env; downloadable
    swebench_repo_structure.zip). show_project_structure() renders it compactly into the prompt.
  - File-localization prompt = obtain_relevant_files_prompt (prompt_template.py:4). Output parsed by
    splitting lines, then correct_file_path_in_structure() + correct_file_paths() snap hallucinated/loose
    paths back onto the real tree, preserving model's importance order.
  - Two RICHER localization prompts exist but are NOT on the default 2-step path (legacy Agentless multi-hop):
    obtain_relevant_functions_and_vars_from_compressed_files_prompt_more (skeleton -> class/fn/var locs)
    and obtain_relevant_code_combine_top_n_prompt (-> exact class/function/line numbers). extract_locs_for_files()
    + construct_topn_file_context() turn those into line-interval context windows. Kimi-Dev intentionally
    skips these and passes whole files instead.
  - GT (ground-truth) mode: --enable_gt feeds the gold patch's file set into repair (generate_found_edit_locs
    parses the real patch) — used for ablation/training-data generation, NOT a fair eval setting.

3.3 PATCH GENERATION (SEARCH/REPLACE, the load-bearing format)
  - Repair prompt forces a strict edit DSL: `### path` then `<<<<<<< SEARCH` / search-lines / `=======` /
    replace-lines / `>>>>>>> REPLACE`, wrapped in ```python```. Prompt hammers EXACT indentation.
  - llm_chat() post-processes the raw model output: strips Kimi reasoning markers (the model emits
    ◁think▷ ... ◁/think▷ — they keep only the post-/think segment), then regex-extracts ```...``` blocks.
  - generate_model_patch_difflib() (post_process.py:1329) is the production path: regex-parse each
    SEARCH/REPLACE triple (pattern r'### ([^\n]+)\n<{4,} SEARCH\n(.*?)\n={4,}\n(.*?)\n>{4,} REPLACE'),
    do a literal str.replace(search, replace) on the original file content, then difflib.unified_diff to
    emit a `diff --git` patch. No fuzzy matching — search block must match verbatim.
  - generate_model_patch() (older path) goes through fake_git_repo() (init a throwaway git repo, write
    old then new content, `git diff`) to get a real unified diff, AND runs gates:
      * check_syntax() via ast.parse,
      * lint_code(): flake8 with ONLY fatal selectors (E9,F821,F823,F831,F406,F407,F70x) on an isolated
        temp file, comparing error sets before/after — a patch that INTRODUCES a new fatal error is rejected,
      * check_code_differ_by_just_empty_lines() — reject no-op whitespace-only diffs.
  - normalize_patch(): canonicalize a diff by ast.unparse + strip comments/docstrings, and if the change
    is "just a new function" reduce the diff to +/- lines only. Used for DEDUP across rollouts (test-time scaling).

3.4 RETRY LOOP (the only "loop" in the default scaffold)
  solve_instance_data() retries up to retry_times (default 5): generate patch; if model_patch=="" OR
  no '@@' hunk header present, BUMP TEMPERATURE 0.0 -> 1.0 and retry. So pass@1 is greedy (temp 0); only
  on a malformed/empty patch does it switch to sampling. passk arg does multi-pass (pass0=temp0, rest temp1).
  This is purely a "did we get a syntactically-applyable patch" loop — it does NOT run tests in the loop.

3.5 TEST WRITER (separate rollout, reproduction-test generation)
  rollout_messages_testwriter.py mirrors the bugfixer but:
    - localizes TEST files (prompt asks for test files to run post-patch), then keeps only basenames
      starting with "test" and takes top-1.
    - repair prompt asks the model to insert a NEW test (via SEARCH/REPLACE into the test file) that
      REPRODUCES the issue. Example test prints "Issue reproduced" / "Issue resolved" / "Other issues".
    - generate_reproduction_test_prompt (prompt_template.py:197) is the standalone reproduce-from-scratch
      variant: emit a complete test that prints those three sentinel strings — a self-judging test where
      stdout string == verdict (no assertion-framework dependency).
    - testwritter_utils: replace_test_functions() AST-rewrites existing test bodies to `pass` (so the
      model's new test is isolated); create_patch_from_code() wraps generated code as a reproduce_bug.py diff.

3.6 VERIFICATION (kimidev/harness — standard SWE-bench Docker eval)
  - test_spec.py builds a per-instance Dockerfile (repo @ base commit, pinned deps from assets json) and an
    eval script that: resets env, applies the model patch, runs the F2P + P2P test commands, captures log.
  - log_parsers.py: per-repo parser (MAP_REPO_TO_PARSER, fallback parse_log_pytest_swesmith) turns raw
    pytest/unittest output into {test_case: PASSED|FAILED|ERROR|XFAIL}.
  - grading.py get_eval_tests_report(): the SWE-bench resolution definition — FAIL_TO_PASS must all PASS
    (resolution) AND PASS_TO_PASS must all still PASS (no regression). resolved = no failure in either set.
    get_logs_eval() first checks the patch actually APPLIED (scans for APPLY_PATCH_FAIL / RESET_FAILED /
    TESTS_ERROR / TESTS_TIMEOUT) — apply-failure => unresolved, fail-closed.

3.7 TEST-TIME SCALING (the selection method, from resources/ + paper)
  - Generate 40 independent patch rollouts (40patches.jsonl) AND 40 independent test rollouts (40tests.jsonl).
  - Cross-evaluate: run each candidate patch against the generated reproduction tests; SELECT the patch by
    consensus/best-test-pass (the final picked patch's eval is in test-time-final-single-pass-eval.jsonl).
    This is a generator(patch) x verifier(self-generated tests) majority/selection scheme — the model is its
    own test oracle, and dedup via normalize_patch collapses equivalent patches before voting.

3.8 TRAINING METHOD (described, NOT in this repo — model weights only)
  README + paper: large-scale RL, agent autonomously patches real repos in Docker, REWARD ONLY when the
  ENTIRE test suite passes (binary, outcome-grounded, no process reward). Mid-training scaling + RL scaling +
  self-play (assets show RL_scale / mid_train_scale / self_play / test_time_scale plots). The "agentless
  training as skill prior" claim: train under the fixed 2-step pipeline, transfer to agentic scaffolds.
  This is NON-PORTABLE here (no training code shipped) but the REWARD DESIGN is the portable idea.

=====================================================================
4. KEY TECHNIQUES PORTABLE TO ELAI
=====================================================================
T1. WHOLE-FILE LOCALIZATION, FINE-GRAIN-DURING-REPAIR (prompt_template.py + rollout_messages_bugfixer.py)
    Instead of multi-hop retrieval (file -> class -> line) before editing, do ONE file-level localization
    call then hand the COMPLETE file(s) to the repair call and let the model localize the exact lines while
    reasoning about the fix. Maps onto ELAI: this is a cheaper, fewer-call alternative to the
    integration-explorer's deep retrieval for SMALL targets. ELAI already has codegraph/elai.map for exact
    localization (stronger than name-grep), so the steal is the INVERSE lesson: for a strong model, collapse
    localize-stages and spend context on full-file repair reasoning rather than retrieval round-trips. Flag:
    only viable when files fit the window — ELAI local sidecar (Qwen 7B, 25-32K ctx) caps this hard;
    whole-file works for cloud Sonnet, not for the local lane on big files.

T2. STRICT SEARCH/REPLACE EDIT DSL + LITERAL-MATCH PATCH SYNTH (post_process.py:1329 generate_model_patch_difflib,
    prompt_template.py:108 repair_prompt). Model emits `<<<<<<< SEARCH / ======= / >>>>>>> REPLACE` blocks,
    code does verbatim str.replace then difflib -> git diff. This is a clean, model-agnostic alternative to
    Anthropic str_replace_editor tool-use for producing applyable patches from ANY backend (incl. a local
    OpenAI-compatible vLLM that has no tool-calling). Maps onto ELAI executor: a deterministic, tool-free
    patch path the local sidecar can drive; pairs with ELAI's existing fake-git-apply style diff generation.

T3. POST-GENERATION FAIL-CLOSED PATCH GATES (post_process.py: check_syntax via ast.parse + lint_code flake8
    FATAL-only delta + check_code_differ_by_just_empty_lines + apply-success check in grading.get_logs_eval).
    BEFORE running expensive tests, cheaply reject patches that: don't parse, INTRODUCE a new fatal lint
    error (set-difference vs pre-patch errors, so pre-existing noise is ignored), are whitespace-only no-ops,
    or fail to git-apply. Maps DIRECTLY onto ELAI's auditor/wiring loop as a cheap pre-test Layer-1 gate:
    same fail-closed posture as wiring-enforcement (missing/invalid => FAIL, not SKIP). The flake8-fatal-delta
    trick (only block NEW fatal errors) is a concrete, stealable maintainability gate.

T4. SELF-GENERATED REPRODUCTION TEST AS VERIFIER + GENERATOR x VERIFIER SELECTION (rollout_messages_testwriter.py,
    generate_reproduction_test_prompt, resources/ 40x40 scheme). Generate N patches and N reproduction tests
    independently, cross-run, select by test-pass consensus; the sentinel-string test ("Issue reproduced" /
    "Issue resolved" / "Other issues") is a framework-free self-judging oracle. Maps onto ELAI's Creator-Verifier
    /auditor doctrine: the auditor can commission a model-written reproduction test (off-agent, the agent that
    wrote the patch must NOT write its own verifier — matches ELAI's "auditor authors the smoke test" rule) and
    use it to rank multiple executor candidates. This is the un-gameable runtime-evidence leg ELAI already
    wants. Dedup-before-vote (normalize_patch) is a free add-on.

T5. OUTCOME-ONLY BINARY REWARD (full-suite-pass) AS THE TRAINING/SELECTION SIGNAL (README, paper).
    Reward only when the WHOLE test suite passes — no partial/process reward. NON-PORTABLE as training (no RL
    code here, and ELAI does not train the model), but PORTABLE as ELAI's done-gate philosophy: it is exactly
    the wiring-enforcement "measure the real objective (tests green from a real entrypoint), not a proxy"
    principle, validated at SOTA scale. Reinforces ELAI doctrine; not new code.

NON-PORTABLE / SKIP: the RL/mid-training/self-play (weights only, no code); the SWE-bench Docker harness
(kimidev/harness/* is standard princeton-nlp/SWE-bench, ELAI has its own eval seam); the GT-localization mode
(eval-cheat / data-gen only); the 72B model itself (ELAI local lane is 3B/7B Qwen — Kimi-Dev-72B could only be
a CLOUD-tier option, not the sidecar).

=====================================================================
5. BENCHMARK NUMBERS (as stated in repo README, grounding only)
=====================================================================
- SWE-bench Verified: 60.4% (Kimi-Dev-72B). README claim: new SOTA among OPEN-SOURCE models, surpasses
  the runner-up. Source: README.md:29 and the open_performance_white.png plot. (Single number from the
  repo; effect-size vs named competitors not quantified in the repo text — see arXiv:2509.23045 for the
  full table before citing comparatives.)

=====================================================================
6. DEPENDENCIES / STACK
=====================================================================
Python 3.12, conda env "kimidev", `pip install -e .`.
Serving: vLLM (CUDA 12.8), 8x tensor-parallel, max-seq-len 131072 (128K ctx), served-model-name "kimi-dev",
  local OpenAI-compatible endpoint at http://127.0.0.1:8000/v1 (KimiDevChatDecoder hardcodes this).
Scaffold deps (setup.py): anthropic, openai, datasets, tiktoken, libcst (CST for structure/skeleton),
  llama-index, jsonlines, unidiff, GitPython, termcolor, tqdm, tenacity.
Eval deps: docker, modal, ghapi (SWE-bench Docker harness).
Patch tooling shells out to system git + flake8 (fake-git-repo / fake-git-apply / lint gates).
Parallelism: ThreadPoolExecutor, default 100 workers, one instance per task, output one jsonl per instance.
