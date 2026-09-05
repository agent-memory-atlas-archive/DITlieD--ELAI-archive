Algorithm: Agentless
ArXiv: 2407.01489
Paper: Agentless: Demystifying LLM-based Software Engineering Agents (Xia et al., 2024, UIUC)
Version: arXiv:2407.01489v2 [cs.SE] 29 Oct 2024

---

CONTEXT ISOLATION QUESTION (explicit vs. inferred)
====================================================

The paper does NOT contain the phrases "clean context", "KV reset", "context isolation",
"stateless", or "fresh context". The per-stage context boundary is INFERRED from three sources,
not explicitly stated as a design requirement:

(1) Section 1, paragraph 5 (bullet "Limited ability to self-reflect"):
    "That is to say they [agents] tend to take all information/feedback and do not know how to
    filter out or correct irrelevant, incorrect, or misleading information. The limited ability
    to self-reflect means that an incorrect step can be easily amplified and negatively affect
    all future decisions made by the agent."
    -> Motivation to avoid accumulated context, but stated as a property of agent-based tools,
       not as a prescription for Agentless internals.

(2) Section 3.1.2, paragraph 1:
    "Directly providing the complete context of all files can be large."
    -> Justification for skeleton format is context-size management, not KV isolation.

(3) Section 5.2.1 (Localization ablation), paragraph 3:
    "We also find that when merging multiple samples together, the amount of ground truth
    localized is higher but at the expense of having to add more context as the input during
    the repair phase. Our default settings also sample the edit locations multiple times,
    however instead of merging, we perform repair on them separately to make use of the fact
    each sampled location set can provide similar localization performance while also limiting
    the input context."
    -> Explicit statement that repair calls are run SEPARATELY per sampled location set,
       specifically to LIMIT INPUT CONTEXT. This is the closest the paper comes to justifying
       per-instance context isolation, but it is scoped to the intra-repair fan-out (4 location
       sets x 10 patches each), not to the cross-phase localize->repair->validate boundary.

VERDICT: "Context isolation" as a per-stage KV-reset architecture is only INFERRED from the
absence of a shared agent loop. The paper describes a pipeline where each LLM call receives a
freshly constructed prompt containing only the outputs of prior stages (structured lists, not
prior LLM conversation turns). There is no explicit statement that KV cache is reset between
phases, or that phases must not share an open conversation context. The closest explicit
justification is the ablation result in Section 5.2.1 showing that separately-run repair calls
(each with a different location set as input context) outperform merged calls, citing context
confusion as the mechanism ("LLMs cannot handle long context very well", Section 5.2.1 p2,
line 1618 in HTML source). This is an empirical context-size argument, not a KV-isolation
architectural prescription.

---

PIPELINE OVERVIEW
=================

Three sequential phases: Localization -> Repair -> Patch Validation.
No autonomous agent loop. LLM does not decide future actions.
Each phase receives structured outputs from prior phase as new prompt inputs.

Phase 1: Localization (3 sub-steps)
  Step 1.1 - File localization:
    Input:  issue description + repository tree structure (Linux tree format)
    Input:  embedding similarity scores (text-embedding-3-small, chunk size 512, overlap 0)
    LLM call: greedy decode, prompt LLM with repo tree + issue description
    Output: ranked list of top-N suspicious files (default N=3), combined from LLM + embedding

  Step 1.2 - Element localization (skeleton format):
    Input:  issue description + skeleton of top-N files (class/function headers only, no bodies)
    LLM call: single prompt, greedy decode
    Output: list of related classes/functions (unrestricted count)
    Context reduction: from >3000 tokens (full files) to <800 tokens (skeletons), per Section 5.2.1

  Step 1.3 - Edit location localization:
    Input:  issue description + full code content of elements from Step 1.2
    LLM call: 4 SEPARATE samples (temperature 0.8) producing 4 independent location sets
    Output: 4 sets of edit locations (line numbers, function names, or class names)
    NOTE: location sets are NOT merged; each is fed independently into Repair

Phase 2: Repair
  Input per location set: issue description + code context window (edit location +/- 10 lines)
  Multiple edit locations within one set: concatenated with "..." separator
  LLM calls: 1 greedy + 9 sampled (temperature 0.8) per location set = 10 patches per set
  Total patches per issue: 4 location sets x 10 patches = 40 patches
  Output format: Search/Replace edit format (search block: original code, replace block: new code)
  Post-processing: apply Search/Replace diff to original file by string match + replace

Phase 3: Patch Validation (2 sub-steps)
  Step 3.1 - Reproduction test generation:
    Input:  issue description + example test format
    LLM calls: 1 greedy + 39 sampled = 40 candidate reproduction tests
    Execution filter: run each test on ORIGINAL (unpatched) repo; keep only those printing
                      "Issue reproduced" (discard "Issue resolved" or "Other issues")
    Selection: normalize (strip comments, spaces, test names) -> pick most frequent test
    Output: single reproduction test file

  Step 3.2 - Patch selection:
    Step A: run all existing repo tests; collect passing tests as candidate regression set
    Step B: LLM call with passing test list; LLM identifies non-regression tests to exclude
    Step C: run regression tests on all 40 patches; keep patches with lowest failure count
    Step D: run reproduction test on remaining patches; keep those outputting "Issue resolved"
            (if no patch passes reproduction test, fall back to Step C results only)
    Step E: majority voting on normalized patches (AST parse -> unparse canonical form ->
            textual diff -> pick highest occurrence count)
    Output: single final patch

---

Inputs (mandatory):
issue_description | string | free text | GitHub issue report; required for all LLM calls
repository_codebase | filesystem | Python project | source of tree structure, file content, test suite
top_n_files | int | scalar | number of suspicious files to localize (default=3)
context_window | int | scalar | lines before/after edit location for repair context (default=10)
n_location_samples | int | scalar | number of edit location sample sets (default=4)
n_patch_samples | int | scalar | patches per location set (default=10; 1 greedy + 9 sampled)
n_repro_samples | int | scalar | reproduction test candidates (default=40; 1 greedy + 39 sampled)
sampling_temperature | float | scalar | temperature for all sampling calls (default=0.8)
embedding_model | string | model ID | OpenAI text-embedding-3-small (chunk_size=512, overlap=0)

Inputs (optional):
embedding_retrieval | bool | enables embedding-based file retrieval alongside LLM prompting (default=on)

Outputs:
final_patch | Search/Replace diff | text | applied to repository to resolve issue
reproduction_test | Python test file | text | prints "Issue reproduced"/"Issue resolved"
localized_files | list[str] | top-N file paths | sorted by suspicion score
edit_locations | list[list[location]] | 4 sets | each set = line/function/class specs
candidate_patches | list[patch] | 40 diffs | before majority-vote selection

Hyperparameters:
top_n_files | 3 | 1-10 | moderate sensitivity; paper tests N=3 as default
context_window | 10 | 5-50 | lines around edit location for repair; wider = more token cost
n_location_samples | 4 | 1-8 | directly multiplies total patch count; 4 shown optimal
n_patch_samples | 10 | 1-40 | performance plateaus ~40 total patches; 10/set is sweet spot
sampling_temperature | 0.8 | 0-1 | standard for diversity; greedy=0 always used for first sample
embedding_chunk_size | 512 | 128-1024 | chunk overlap 0 in all experiments

Critical invariants (testable assertions):
assert len(candidate_patches) == n_location_samples * n_patch_samples  # Section 4: 4x10=40
assert all(test_execution_output in ["Issue reproduced", "Issue resolved", "Other issues"] for test in repro_tests)  # Section 3.3.1
assert final_repro_test is None or execute_on_original_repo(final_repro_test) == "Issue reproduced"  # Section 3.3.1 filter
assert len(selected_patches_after_regression) <= len(candidate_patches)  # monotone reduction
assert majority_vote_patch in candidate_patches_normalized  # Section 3.3.2

Complexity:
Localization: O(F * C) where F=files, C=chunk embedding cost; LLM calls = 1 (file-level) + 1 (skeleton) + n_location_samples (edit locations) = 6 LLM calls
Repair: O(n_location_samples * n_patch_samples) LLM calls = 40 total; parallelizable across location sets
Validation: O(n_repro_samples + n_patches * n_regression_tests) execution calls; LLM calls = 1 (non-regression filtering) + n_repro_samples (40 test generation) = 41
Total LLM calls per issue: ~47 (6 localization + 40 repair + 1 non-regression filter)
Space: each LLM call context <= 800 tokens for localization steps; repair context = 2*context_window*n_edit_locations tokens

Failure modes (cited in paper):
Long context degrades localization: "LLMs cannot handle long context very well, so providing the entire file contents can confuse the model" (Section 5.2.1, line 1618). Mitigation: hierarchical narrowing.
Merging location sets increases context noise during repair: "merging, we perform repair on them separately... limiting the input context" (Section 5.2.1, line 1627). Mitigation: separate repair per location set.
Incorrect step in agent loop propagates: agent-based approaches suffer because "an incorrect step can be easily amplified and negatively affect all future decisions" (Section 1, bullet 3). Agentless avoids by not accumulating turns.
Reproduction test false negative: "the reproduction test is generated by the LLM and can potentially be incorrect/imprecise, it could be possible that no patch can pass the reproduction test; in that case, Agentless will fall back on only using the regression test results" (Section 3.3.2).
Majority voting saturation: "performance plateaus at around 40 samples where adding additional candidate patches does not improve performance" because majority vote makes later samples unable to offset the leading patch (Section 5.2.2).
Tool design fragility in agent alternatives: agents can "easily lead to incorrect or imprecise tool design/usage, especially for more complex action spaces" (Section 1, bullet 1). Agentless avoids by using no tools.

Implementation pitfalls (do NOT do this):
Do NOT merge multiple edit location samples into a single repair prompt; this increases context size and degrades repair performance per Section 5.2.1 ablation.
Do NOT use the full file content for skeleton-step localization; provide only headers. Full content causes confusion (Section 3.1.2, Section 5.2.1 line 1618).
Do NOT skip the hierarchical localization steps by jumping from file-level directly to edit locations; "both the cost and performance are worse" and "model can become confused when providing a large context" (Section 5.2.1 line 1624-1625).
Do NOT allow the LLM to autonomously decide future actions or use complex tools; this introduces the three agent failure modes identified in Section 1 (tool design errors, planning confusion, self-reflection failure).
Do NOT use PASS_TO_PASS tests from SWE-bench directly as regression tests without LLM filtering; inflates performance to 98 vs. 96 on SWE-bench Lite (Section 4, footnote 1).
Do NOT normalize reproduction tests before execution filtering; filter first (must print "Issue reproduced" on original repo), then normalize and vote (Section 3.3.1).
Do NOT apply majority voting before regression test filtering; regression filter reduces candidate pool first, then voting operates on the filtered set (Section 3.3.2).
Do NOT feed prior LLM conversation turns as context to subsequent phases; each LLM call receives only the freshly-constructed prompt for that step (inferred from pipeline design, no shared agent loop).
