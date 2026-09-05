Source: RLM Rubric + Recursive Coding Agents Taxonomy
URLs:
  https://github.com/rawwerks/recursive-coding-agents/blob/main/rlm-rubric/README.md
  https://raw.githubusercontent.com/rawwerks/recursive-coding-agents/main/rlm-rubric/rlm-rubric.md
  https://recursivecodingagents.com/
Author: Raymond Weitekamp (rawwerks)
Reliability: MEDIUM (reviewed source, public repo; methodology is design doc, not peer-reviewed experiment)
Note: rlm-rubric.json and rlm-judging-methodology.md exist in the repo but were not fetched.
       Full definitions from raw rlm-rubric.md fetched directly (highest-fidelity source here).

PURPOSE OF THIS MAP

The rubric is an EVAL TOOL, not an algorithm. Use it to judge whether any coding agent
(ELAI, Claude Code, Cursor, etc.) qualifies as an RLM. Directly applicable as an
acceptance rubric for ELAI's own recursive agent implementation.

THE SEVEN HARD GATES (all required; one fail = not a full RLM)

Gate | Name                   | Requirement (from rlm-rubric.md, verified)
G1   | model-call outer shape | System functions as a model call: prompt/context in, answer out
G2   | prompt externalization | Full prompt/context exists as symbolic state in environment, NOT pasted into root model context
G3   | symbolic handle        | Root model references context via handles (tagged files, paths, objects, indexes) — not raw content
G4   | persistent exec env    | REPL, runtime, or workspace persists intermediate state across steps
G5   | programmatic subcalls  | Environment code can invoke sub-LMs / sub-RLMs on constructed slices
G6   | model-chosen decomp    | Model decides decomposition, inspection, delegation, AND stopping logic (not hardcoded in script)
G7   | symbolic intermediate  | Large intermediates stay in environment variables/files, NOT fully re-verbalized into context

CLASSIFICATION TIERS (from rubric and recursivecodingagents.com)

Full RLM          | passes all G1-G7
RLM-family ablation | has externalization + executable env; lacks recursive subcalls; matches RLM(depth=0)
RLM-adjacent      | shares some mechanisms (tools, agents, retrieval, code execution) without meeting hard gates
Not RLM           | fails one or more hard gates

TAXONOMY: FOUR IMPLEMENTATION LEVELS (from recursivecodingagents.com)

Level 1: Plain long-context call
  Pattern: RAG / reasoning-only; context pasted in
  Fails: G2, G3, G5, G6, G7 (most gates)
  Examples: naive RAG, bare llm_call with stuffed context

Level 2: Coding agents + subagents
  Pattern: agent loops with tools; developer determines task structure at design time
  Fails: G6 (model does not pick decomposition — developer authored the structure)
  Examples: early Claude Code (Q4 2025), fixed-pipeline coding agents

Level 3: Hardcoded map-reduce (lambda-RLM)
  Pattern: developer-authored pipeline; fixed windows or fan-out; e.g. lambda-RLM
  Fails: G6 (decomposition is hardcoded, not model-chosen)
  Examples: fixed summarization loops, hardcoded chunked-map + reduce pipelines

Level 4: Recursive Language Model
  Pattern: passes every check; model controls decomposition strategy
  Examples: dspy.RLM, ypi, ELAI with dynamic workflows (per author claim, Q2 2026)

KEY DISTINCTION: G6 IS THE DIVIDING LINE

The difference between Level 3 and Level 4 is entirely G6.
If the developer's script determines how tasks split (fixed windows, fixed fan-out, fixed depth),
it is Level 3 regardless of whether it uses sub-agents or a code REPL.
An RLM requires the LLM itself to choose: whether to decompose, how to decompose, when to stop.

WHAT EXPLICITLY FAILS THE RUBRIC (from rlm-rubric.md)

Systems that fail and why:
  Long-context direct calls          | fails G2, G3, G5, G6, G7
  Fixed summarization loops          | fails G6 (fixed decomposition)
  RAG / retrieval alone              | fails G2, G4, G5, G6
  Subagents without programmatic calling | fails G5
  Repeated self-calls without external state | fails G2, G3, G7
  Hardcoded pipelines                | fails G6
  Tree-of-Thought (standard)        | fails G2, G4, G5 (no REPL, no external env)
  Memory systems not built on RLM    | fails G5, G6 unless they expose rlm_query equivalent

JUDGING METHODOLOGY (from rlm-rubric.md)

Classify OBSERVED BEHAVIOR, not marketing claims.
Evidence packet required per system evaluation:
  - Run shape (how the system executes in practice)
  - Tools enabled (what primitives are available)
  - Exact execution traces
  - Per-gate assessment (pass/fail/partial for each G1-G7)
  - Strongest arguments for AND against RLM classification
  - What evidence would change the verdict

Rubric is a conformance contract, not aspirational guidance.
Credit systems for subagents, loops, bash, durable files, repo handles, symbolic state
ONLY when observed in actual execution runs.

CLAUDE CODE STATUS TRANSITION (author claim, from recursivecodingagents.com)

Q4 2025: Not an RLM (subagents without model-chosen splits — fails G6)
Q2 2026: Became an RLM (dynamic workflows enable model decomposition selection)
Reliability: AUTHOR CLAIM, not independently verified.

CRITICAL ASSERTIONS (testable against any candidate system)

assert system_has_model_call_interface(system), "G1 fail: not a model-call outer shape"
assert prompt_lives_in_env_not_context(system), "G2 fail: prompt pasted into LLM context"
assert model_uses_handles_not_raw_content(system), "G3 fail: no symbolic handle"
assert env_persists_across_steps(system), "G4 fail: no persistent executable environment"
assert env_code_can_spawn_sub_lms(system), "G5 fail: no programmatic subcalls"
assert model_chooses_decomposition_not_script(system), "G6 fail: hardcoded decomposition"
assert intermediates_stay_in_env(system), "G7 fail: intermediates re-verbalized into context"
assert all_gates_pass(system), "Not a full RLM: partial implementation is RLM-family ablation"

APPLICATION TO ELAI

To claim ELAI is a full RLM, verify per gate:
G1: ELAI exposes a prompt-in/answer-out interface — likely PASS (CLI/IPC)
G2: Does ELAI store the full task context as a file/variable, NOT pasted into root LLM? — VERIFY
G3: Does the root LLM see file handles / metadata, not raw content? — VERIFY
G4: Is there a persistent REPL or workspace across steps? — VERIFY (session_dag, REPL env)
G5: Can the LLM-generated code invoke sub-LMs programmatically? — VERIFY (rlm_query equivalent?)
G6: Does the LLM decide task decomposition, or does the orchestrator script fix it? — LIKELY PARTIAL
G7: Do large intermediates stay in files/vars, not echoed back into context? — VERIFY
Honest current assessment: ELAI is likely Level 2 or Level 3 (not full RLM) pending G5/G6 verification.

FAILURE MODES (rubric application failures)

Granting G6 based on documentation claims instead of observed execution traces.
Conflating "uses subagents" with G5 (G5 requires programmatic invocation from env code, not just agent spawning).
Treating memory/RAG as satisfying G2/G3 when the retrieved content is pasted into context (fails G2).
Classifying a system as RLM-adjacent without documenting which gates it fails and why.

IMPLEMENTATION PITFALLS (do NOT do this when building toward RLM conformance)

Do not store the task prompt in the LLM system message or user message — it must live in the env (G2).
Do not hardcode how many sub-agents to spawn or what slices they receive — the LLM must choose (G6).
Do not return large sub-agent outputs as inline strings to the parent — return file paths or handles (G7).
Do not design orchestration where the developer's code decides when recursion stops — that is G6 failure.
Do not count a system as G5-compliant if sub-LM calls are only triggered by the framework, not by model-generated code.

RELATED MAPS

rlm-interface.md — the RLM algorithm itself (arXiv:2512.24601)
ypi-recursive-coding-agent-interface.md — ypi as a concrete full-RLM implementation
rlm-as-memory-rawworks-interface.md — RLM applied as a memory system (LongMemEval results)

UNVERIFIED

rlm-rubric.json and rlm-judging-methodology.md not fetched; may contain more detailed scoring.
Author's Claude Code Q2 2026 transition claim not independently verified.
Whether the four-level taxonomy has been applied to ELAI in any prior session.
