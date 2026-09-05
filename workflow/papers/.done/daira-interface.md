Algorithm: DAIRA (Dynamic Analysis-enhanced Issue Resolution Agent)
ArXiv: 2603.22048
Paper: Dynamic analysis enhances issue resolution
Authors: Mingwei Liu, Zihao Wang, Zhenxi Chen, Zheng Pei, Yanlin Wang, Zibin Zheng
Published: 2026-03-24 (cs.SE)
Updated: 2026-04-05 (session s-current; enriched for elai_cc grounding, verification, hallucination mitigation)


WHAT DAIRA ACTUALLY IS

DAIRA is a tool-augmented LLM agent framework for autonomous software bug repair. It does NOT
replace the LLM. It adds an observability layer on top of the agent decision loop: before the
agent reasons about a bug, it must first collect runtime execution evidence. The framework
enforces that claims about code behavior are derived from observed traces, not from LLM priors.

The paper's explicit framing: "equipping agents with mechanisms to visualize intermediate
execution states is a critical strategy for augmenting their problem-solving capabilities."

The key behavioral shift the paper measures: from "speculative trial-and-error exploration"
to "evidence-based, deterministic deduction." The ablation results quantify the cost of
skipping each component of this shift (see ABLATION below).


ARCHITECTURE — THREE SUBSYSTEMS

Subsystem 1: Dynamic Analysis Tool (DAT)
  Engine: customized Hunter tracer (Python open-source) + sys.settrace hooks
  Strategy: "trigger-and-collect" — tracer is dormant until target_function is invoked
  Execution: scripts run via runpy inside an isolated sandbox
  Event capture: ONLY three event types — call | return | exception
  Filtering:
    Temporal: on-demand, not continuous; only active during target_function scope
    Spatial: whitelist-bounded; stdlib and unrelated deps excluded
    Granularity: function-level only, not line-by-line
    Adaptive: if log size approaches context window, overflow signal fires; agent
              reduces trace_depth and reruns — prevents context explosion
  Output: raw formatted trace log (variable size, structured text)

Subsystem 2: Trace Log Semantic Analysis (TLSA)
  Input:  raw trace log + target source code + tracing parameters
  Output: Execution Trace Report (three-part structured document)
  Implemented as: LLM-driven transformation (extra LLM call per trace invocation)

  Three sub-components of the Execution Trace Report (ALL THREE REQUIRED):
    1. Hierarchical Logic Reconstruction
       Visual ASCII execution tree: nested calls, recursive paths, intermediate data states,
       branching points, and divergence from expected execution path.
       Example from paper: shows convert() receiving empty numpy array, entering wrong validation branch.

    2. Key Function Analysis
       Per-function role description: "workflow role" + "functional boundary."
       Cross-validates roles across multiple input scenarios to separate design intent from
       actual behavior. Example: convert() orchestrates; _validate_unit() is guard clause.

    3. Workflow Process Introduction
       Natural language narrative translating low-level trace events into high-level intent.
       Bridges the gap between raw execution data and architectural understanding.

  CRITICAL: ablation shows raw trace logs without TLSA drop resolution by 11.3 pp.
  Raw logs are NOT equivalent to TLSA output. The transformation is non-optional.

Subsystem 3: Test-Trace Driven Workflow (TTDW)
  A three-phase FSM with explicit gate conditions between phases.
  Phase 1 -> Phase 2 gate: at least one valid Execution Trace Report must exist
  Phase 2 -> Phase 3 gate: root cause must be localized to specific function/branch
  Phase 3 submission gate: patch must pass dynamic verification AND regression tests

  Phase 1 — Execution Tracing:
    Inputs:  issue_description (text)
    Actions: generate Error Reproduction Script; generate Edge Case Scripts; invoke DAT;
             invoke TLSA on each raw log
    Outputs: set of Execution Trace Reports (one per script run)

  Phase 2 — Fault Diagnosis:
    Inputs:  set of Execution Trace Reports
    Actions: Key Function Specification Summary (cross-validate reports across input
             scenarios); Root Cause Analysis (design intent vs. observed behavior)
    Outputs: root_cause_statement localized to specific function/branch

  Phase 3 — Iterative Repair and Verification:
    Inputs:  root_cause_statement + codebase
    Actions: generate candidate patch; dynamic verification using Phase 1 scripts;
             regression tests; iterate if verification fails
    Outputs: verified_patch (passes all gates)


INPUTS

issue_description       | str         | natural language bug report       | mandatory; non-empty
codebase                | directory   | Python repository under repair    | mandatory; must be executable
target_file             | str         | path to source file for DAT       | mandatory (agent-derived from Phase 1)
target_function         | str         | function name as DAT entry point  | mandatory (agent-derived)
trace_scope             | str         | spatial whitelist for DAT         | mandatory (agent-derived)
trace_depth             | int         | call stack depth for DAT          | mandatory; adaptive; >0 always
edge_case_scripts       | list[str]   | scripts beyond error reproduction | optional; >=1 strongly recommended


OUTPUTS

execution_trace_reports | list[TraceReport] | at least 1 required; one per script run
root_cause_statement    | str               | non-empty; localized to function/branch
candidate_patch         | diff              | unified diff
verified_patch          | diff              | passes error repro + regression gates
resolution_rate         | float             | in [0.0, 1.0]; SOTA 0.794 on SWE-bench Verified


OPERATING CONSTRAINTS (not tunable; hard limits)

max_steps_per_instance    | 250    | hard budget; applies to total agent steps
cost_budget_per_instance  | $1.00  | hard budget; applies to total LLM API cost
context_window_limit      | model-dependent | overflow triggers adaptive trace_depth reduction


CRITICAL INVARIANTS (testable assertions)

assert len(execution_trace_reports) >= 1, "Phase 2 blocked without trace report"
assert root_cause_statement is not None and len(root_cause_statement) > 0, "Phase 3 blocked without root cause"
assert trace_event_types.issubset({'call', 'return', 'exception'}), "DAT only captures these three types"
assert trace_depth > 0, "overflow causes depth reduction, never zero-out"
assert len(tlsa_output_tokens) <= model_context_window, "TLSA output must fit context window"
assert verified_patch_passes_error_repro_script == True, "patch must fix the original failure"
assert verified_patch_passes_regression_tests == True, "patch must not introduce regressions"
assert all(three_tlsa_components_present for report in execution_trace_reports), "all three TLSA sub-components required"


ABLATION RESULTS (impact of removing each component, RQ4)

Full DAIRA (DeepSeek V3.2 backbone):        74.2% success
w/o Trace Log Semantic Analysis (TLSA):     62.9%  (-11.3 pp; largest single-component loss)
w/o Dynamic Analysis Tool (DAT):            66.1%  (-8.1 pp)
w/o Test-Trace Driven Workflow (TTDW):      68.8%  (-5.4 pp)

Interpretation: structural transformation of traces (TLSA) matters more than raw tracing alone.
An agent with raw traces and no TLSA performs near-baseline. The semantic bridge is the core value.


PERFORMANCE (SWE-bench Verified, 500 instances)

DAIRA + Gemini 3 Flash Preview:  79.4% (SOTA as of 2026-03-24)
DAIRA + DeepSeek V3.2:           74.2%
DAIRA + Qwen-3-Coder Flash:      noted instability on medium tasks despite improvement on hard

vs. OpenHands:       77.6%
vs. Live-SWE-agent:  77.4%
vs. SWE-agent:       73.6%
Unique resolutions:  9 complex instances not resolved by any other top-5 method

By difficulty:
  Medium (15min–1hr, n=261): 80.8% (SOTA)
  Hard   (>1hr,    n=45):    44.4% (SOTA)

Token efficiency:
  Input token reduction:  15.6%–39.1% across difficulty levels (~25% overall)
  Cost:  -10% overall; but DeepSeek +10% cost due to deeper exploration on hard tasks


FAILURE MODES (cited in paper)

1. Context explosion: trace_depth too large before adaptive reduction fires; mitigated by
   the adaptive depth-reduction loop, which is mandatory, not optional
2. Raw log overload: feeding raw Hunter output to LLM without TLSA causes information overload;
   ablation shows -11.3 pp; effectively reverts to baseline static-analysis performance
3. Phase gate failure: if agent-generated scripts crash before target_function is reached,
   DAT produces no valid trace; Phase 2 is blocked with no evidence to reason from
4. Script quality dependency: if Error Reproduction Script fails to trigger the actual fault,
   Phase 2 diagnosis operates on incomplete evidence; fault localization will be wrong
5. Python-only tooling: sys.settrace is Python-specific; DAT cannot be used unchanged on
   JVM, CLR, or compiled languages without substituting the tracing engine
6. Model-specific instability: Qwen-3-Coder shows cognitive overload from high-density context;
   DAIRA's structured reports do not fully compensate for model capacity limits
7. Hard-task ceiling: >1hr complexity issues resolve at 44.4% maximum; long dependency chains
   and cross-module state remain partially opaque even with full tracing
8. Speculative repair (the baseline failure DAIRA corrects): without runtime evidence, agents
   produce defensive patches that mask symptoms rather than fixing root causes


IMPLEMENTATION PITFALLS (do NOT do this)

Do NOT feed raw Hunter/sys.settrace logs directly to the LLM — TLSA transformation is required;
  raw logs without semantic reconstruction drop performance by 11.3 pp (ablation RQ4)
Do NOT capture line-by-line trace events — DAIRA explicitly restricts to call/return/exception
  to avoid trace bloat; line-level tracing is a known failure vector
Do NOT let trace_depth grow without an adaptive overflow correction loop — context explosion
  is the primary runtime failure mode; the loop must be first-class, not a fallback
Do NOT run Phase 2 without at least one valid Execution Trace Report — this is a hard gate
Do NOT submit a patch before dynamic verification AND regression tests both pass — Phase 3
  verification is not optional QA; it is the only correctness signal
Do NOT couple DAIRA's workflow tightly to SWE-agent internals — paper uses only edit_anthropic
  from SWE-agent; the rest is replaceable, and tight coupling creates portability debt
Do NOT assume the three-phase FSM is language-agnostic at the tool level — the TTDW logic is
  language-agnostic, but DAT requires sys.settrace; these must not be conflated
Do NOT batch TLSA across all traces before starting Phase 2 — each report should be available
  for cross-validation as it is produced
Do NOT treat DAIRA as a model replacement — it is an observability and reasoning-grounding layer;
  model capability still determines the hard ceiling (44.4% on hard tasks)


ELAI_CC INTEGRATION MAP

The following maps DAIRA concepts to elai_cc design requirements. DAIRA provides empirical
validation for each mapping via ablation and benchmark data.

1. GROUNDED REASONING (GRC engine)

   DAIRA finding: speculative reasoning without traces costs 8.1–11.3 pp vs. evidence-first
   reasoning. This is the empirical cost of ungrounded reasoning in an autonomous agent.

   DAIRA pattern: Phase 1 produces grounding evidence (trace reports) BEFORE Phase 2 may
   hypothesize about root cause. The gate is structural and enforced by the FSM, not advisory.

   elai_cc mapping: GRC's observe -> hypothesize -> plan_action -> act -> verify protocol is
   structurally isomorphic to DAIRA's Phase 1 -> Phase 2 -> Phase 3 FSM.
   Gate enforcement: mcp_think calls with step_type="hypothesize" must cite a grounding data
   source (equivalent to "at least one trace report exists before Phase 2 begins").
   Without a cited grounding source, the hypothesis is structurally equivalent to DAIRA running
   Phase 2 with no trace evidence — the known -8.1 pp failure mode.

2. HALLUCINATION MITIGATION

   DAIRA finding: the -11.3 pp ablation for removing TLSA (not DAT, but the transformation layer)
   shows that structured intermediate representation is the primary hallucination mitigation.
   Raw data fed to LLMs without structure = near-baseline performance = the agent hallucinates
   structure from training priors rather than from observed evidence.

   DAIRA pattern: TLSA transforms raw runtime data into a three-part structured report before
   the LLM reasons from it. Each of the three sub-components addresses a different failure mode:
     - Hierarchical Logic Reconstruction: prevents misattribution of branching behavior
     - Key Function Analysis: prevents incorrect role assignment to functions
     - Workflow Process Introduction: prevents narrative fabrication about code purpose

   elai_cc mapping: any tool that produces unstructured output (log files, raw API responses,
   file contents) must pass through a structured transformation before being used as a grounding
   source in GRC. The context-compiler agent is the architectural equivalent of TLSA.
   Context-compiler output that lacks structure is not safe grounding.

3. DETERMINISTIC VERIFICATION

   DAIRA finding: Phase 3 re-executes the original failure scripts against the candidate patch
   before submission. This is deterministic: the patch either reproduces the fix or it does not.
   No LLM judgment is involved in the verification gate; it is execution-based.

   DAIRA pattern: verification uses the SAME scripts generated in Phase 1 (the grounding phase),
   closing the loop between evidence collection and correctness validation. The verification is
   not a separate quality step — it is the terminal condition of the workflow.

   elai_cc mapping: every plan phase that produces an output must have a deterministic
   verification condition derived from the phase's own grounding evidence. The verification
   must be execution-based or schema-based, not LLM self-evaluation. LLM self-evaluation of
   its own output has no equivalent in DAIRA — it is structurally absent.

4. FACT-BASED PLANNING

   DAIRA finding: Phase 2 "design intent analysis" explicitly compares observed runtime behavior
   (from traces) against intended behavior (from code design). The plan (root cause statement)
   is the delta between these two. Planning from priors alone without this comparison = the
   speculative patching failure mode.

   DAIRA pattern: plans (root cause statements, patch candidates) are anchored to two facts:
     (a) what the code does (observed, from trace)
     (b) what the code should do (intended, from issue description)
   The plan body is the transformation from (a) to (b). No other information is used.

   elai_cc mapping: plan phases must anchor to:
     (a) observed current state (from context-compiler output, tool results, prior audit)
     (b) target state (from phase specification)
   Plans that do not cite both anchors are speculative. The decision doc requirement
   (workflow/reasoning/phase-X-decisions.md) enforces this structurally.

5. ADAPTIVE CONTEXT MANAGEMENT

   DAIRA finding: adaptive trace_depth reduction prevents context explosion. Without this
   mechanism, large traces overflow the context window, making analysis impossible.

   DAIRA pattern: the overflow signal fires BEFORE context is exceeded; the agent reduces
   trace_depth and reruns from the same entry point. The reduction is a first-class
   orchestration operation, not error recovery.

   elai_cc mapping: the context-compiler must implement equivalent adaptive filtering.
   When accumulated context approaches the model's context window, the compiler must reduce
   granularity (fewer files, shorter excerpts, filtered changelog entries) and signal this
   reduction to the main model. The reduction is not a failure; it is correct operation.


BENCHMARK CALIBRATION DATA (for setting expectations on automated agent tasks)

SWE-bench Verified characteristics:
  Total: 500 instances, 12 Python repositories
  Easy  (<15min):   n=194
  Medium (15min–1hr): n=261; DAIRA resolves 80.8%
  Hard  (>1hr):     n=45;  DAIRA resolves 44.4%
  
Ceiling observation: even SOTA autonomous agents with runtime tracing fail 55.6% of hard
software tasks (>1hr human effort). This is the hard ceiling for current LLM agent systems.
Tasks requiring architectural understanding of long cross-module dependency chains are
systematically under-resolved.


CONTRACT TEST STUBS
Copy to: d:\.ELAI_workflow\.elai_cc\tests\contracts\test_daira_contract.py

import pytest

def test_phase2_blocked_without_trace_report():
    # Invariant Section 3.1: Phase 2 requires at least one Execution Trace Report
    # Ablation RQ4: removing DAT costs 8.1 pp; removing TLSA costs 11.3 pp
    execution_trace_reports = []
    assert len(execution_trace_reports) >= 1, \
        "DAIRA invariant violated: Phase 2 (fault diagnosis) cannot begin without trace evidence"

def test_tlsa_all_three_components_present():
    # Invariant Section 3.3: Execution Trace Report must contain all three sub-components
    # Ablation RQ4: partial TLSA output is not equivalent to full TLSA output
    report = {}  # replace with actual TraceReport object
    required_keys = {"hierarchical_logic", "key_function_analysis", "workflow_process"}
    assert required_keys.issubset(report.keys()), \
        "DAIRA invariant violated: all three TLSA sub-components required per trace report"

def test_patch_submission_requires_both_verification_gates():
    # Invariant Section 3.1 Phase 3: both error repro and regression must pass before submission
    error_repro_passes = False    # replace with actual verification result
    regression_passes = False     # replace with actual regression result
    assert error_repro_passes and regression_passes, \
        "DAIRA invariant violated: patch must pass error reproduction AND regression before submission"

def test_trace_depth_never_zero():
    # Invariant Section 3.2: adaptive reduction reduces depth but never zeroes it
    trace_depth = 0  # replace with actual adaptive depth value
    assert trace_depth > 0, \
        "DAIRA invariant violated: trace_depth must be positive; overflow causes reduction not elimination"

def test_grounding_hypothesis_requires_evidence():
    # Maps to elai_cc GRC invariant derived from DAIRA Phase 1->2 gate
    # Analogous to: mcp_think(step_type="hypothesize") must cite grounding source
    grounding_source = None  # replace with actual grounding reference
    hypothesis = "some claim about code behavior"
    if hypothesis is not None:
        assert grounding_source is not None, \
            "GRC invariant (DAIRA-derived): hypothesis without grounding evidence = speculative reasoning"
