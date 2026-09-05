Algorithm: Life-Harness (Lifecycle-Aware Runtime Harness Adaptation)
ArXiv: 2605.22166
Paper: Adapting the Interface, Not the Model: Runtime Harness Adaptation for Deterministic LLM Agents
Authors: Peking University (affiliation stated)
Published: May 27, 2026 (v2)

SCOPE CONSTRAINT
Applies to deterministic rule-governed agent environments only (stable tool interface,
stable feedback rules, stable evaluation criteria). NOT applicable to open-ended tasks
with varying goals, tools, or success criteria.

EPISODE FORMALISM
x = task description
C = environment contract (static, provided by environment)
s_t = environment state at step t
o_t = observation at step t
tau_t = trajectory = (C, x, o_0, a_0, o_1, ..., a_{t-1}, o_t)
B = step budget
a_t ~ pi_theta(.|tau_t) = LLM output at step t

Initialization: s_0, o_0 = E.Init(x)
Environment step: s_{t+1}, o_{t+1} = E.Step(s_t, a_t)

HARNESS FORMAL DEFINITION
H = {EnvContract, SkillLayer, RealizeAction, RegulateTrajectory}
H' = A_harness(H, T_train), theta fixed  [runtime adaptation, not parameter adaptation]
Contrast: theta' = A_param(theta, T_train)  [parameter adaptation, H fixed]

ALGORITHM 1: Life-Harness Loop (fixed execution order, NOT configurable)
Input: task x, environment E, contract C, budget B
Step 1:  C' = EnvContract(C)        [Layer 1: pre-interaction]
Step 2:  x' = SkillLayer(x)         [Layer 2: pre-interaction, modifies task prompt]
Step 3:  s_0, o_0 = E.Init(x')
Step 4:  tau_0 = (C', x', o_0)
Step 5:  for t = 0..B-1:
Step 6:    a_t = LLM(tau_t)
Step 7:    z_t = RealizeAction(a_t, tau_t, C', s_t)   [Layer 3: post-LLM, pre-execution]
Step 8:    s_{t+1}, o_{t+1} = ExecuteOrBlock(E, s_t, z_t)
Step 9:    tau_{t+1} = tau_t + (z_t, o_{t+1})
Step 10:   tau_{t+1} = RegulateTrajectory(tau_{t+1}, C', s_{t+1}, b_t)  [Layer 4: post-execution]
Step 11:   if IsEnd(s_{t+1}, o_{t+1}): break

LAYER 1: ENVIRONMENT CONTRACT
Lifecycle position: pre-interaction (runs once before episode begins)
Formal:  C' = C + delta_C
         delta_C = concise updates from environment policies, API behavior,
                   recurring failures in training trajectories
Input:   C (original environment contract)
Output:  C' (enhanced contract, appended to model-visible context)
Content: tool-use rules, policy constraints, common pitfalls, tool calling
         conventions, admissible action constraints
Storage: text fragments appended to system prompt
Examples from paper:
  - Airline: "flight search takes only origin, destination, and date"
  - WebShop: "search with concise keywords, click only visible options"
Triggering: fires once per episode at initialization

LAYER 2: PROCEDURAL SKILL
Lifecycle position: pre-interaction (runs once before episode begins)
Formal:  K_x = TopK_{k in S} score(x, k)
         where score is BM25 (k1/b parameters NOT specified in paper)
         S = skill memory constructed from training trajectories
Input:   x (task description)
Output:  K_x inserted into initial system prompt
Storage per skill entry: compact natural-language procedural strategy
  (verbatim format: text description, NOT pseudocode or code)
Examples from paper (verbatim):
  - ALFWorld: "pick up the object before cleaning/heating/cooling"
  - Telecom: "calls get_data_usage before chasing APN/VPN causes"
  - Retail: "count every variant with available=true for availability-count
    tasks; copy hidden target addresses from profile or previous orders
    before writing"
Retrieval config: top-1 skill per experiment (hard cap to prevent contamination)
BM25: standard BM25 over task description string vs skill entries;
      no specific k1/b values stated; use library defaults
Skill construction: derived from training trajectories (summarization method
  not specified in paper)
Size of S: not specified in paper

LAYER 3: ACTION REALIZATION GATE
Lifecycle position: post-LLM-output, pre-environment-execution (runs every step)
Formal:  z_t = RealizeAction(a_t, tau_t, s_t)
         z_t in {EXEC(a_t), Block(m_t)}
Input:   a_t = LLM-generated action
         tau_t = current trajectory
         s_t = current environment state
Output:  EXEC(a_t): submit action to environment
         Block(m_t): return model-visible block message m_t (NOT executed)
Deterministic evidence used: tool schemas, admissible action sets,
  argument constraints, task policies
Operations: (a) verify action is executable under environment contract
            (b) canonicalize unambiguous interface-level errors (limited
                argument modification IS supported per paper)
            (c) block actions that would deterministically fail
Blocking triggers (Appendix A.1 priority order):
  - Non-executable tool_calls (free-form, natural language as action)
  - Invalid function name
  - JSON parsing failure
  - Missing required arguments
  - Type errors on arguments
  - SQL dialect/interface violations
Key invariant: ONLY deterministically-verifiable failures are blocked;
  ambiguous cases are NOT blocked (paper constraint: "do not override
  model reasoning when correct action is ambiguous")
Block message m_t: model-visible, enables LLM to self-correct

LAYER 4: TRAJECTORY REGULATION MONITOR
Lifecycle position: post-environment-execution (runs every step)
Formal:  r_t = RegulateTrajectory(tau_t, a_t, o_{t+1}, b_t)
         b_t = B - t - 1  (remaining budget, explicit parameter)
Input:   tau_t = trajectory so far
         a_t = action just taken
         o_{t+1} = environment observation after execution
         b_t = remaining step budget
Output (4 types, returned as appended observation):
  1. empty (no intervention)
  2. soft recovery message
  3. warning regarding repeated failures
  4. stronger corrective directive
Detection targets:
  - repetition (same action repeated)
  - stagnation (state not progressing)
  - ineffective recovery (agent retrying failed strategy)
  - non-progressing patterns
  - trajectory degeneration (committing to incorrect strategy early)
  - budget exhaustion signals (b_t context)

OFFLINE INTERVENTION-AUTHORING PIPELINE

Step 1: Collect training trajectories T_train from target environment
Step 2: Annotate failures using priority-based protocol (Appendix A.1)

FAILURE CLASSIFICATION (Appendix A.1, verbatim priority order):
  Priority 1 - Action Realization failures (check FIRST):
    - non-executable tool_calls
    - invalid function name, JSON failure, missing args, type errors
    - SQL interface/dialect violations
  Priority 2 - Environment Contract mismatches (check SECOND):
    - wrong tool for critical steps
    - tool choice/ordering violates task protocol
    - argument has incorrect semantic format despite valid JSON schema
  Priority 3 - Trajectory Degeneration (check THIRD):
    - episode ends at limit with clear repetition or oscillation
    - agent commits to incorrect strategy early and reinforces
    - environment repeatedly returns no-progress feedback
  Priority 4 - Residual Reasoning failures (FALLBACK):
    - incorrect inference, computation, SQL, retrieval, or decision-making
      despite largely following the protocol

Step 3: Run harness evolution via LLM-authored prompt (Appendix A.2)

HARNESS EVOLUTION PROMPT STRUCTURE (Appendix A.2):
Sections: System Instruction, Inputs, Harness Design Principles,
          Analysis Requirements, Update Requirements, Regression Check, Output
Inputs provided to prompt:
  1. current harness implementation (code)
  2. trajectory directory with summary metrics from previous iteration
  3. design guide describing four lifecycle layers
Key analysis instructions:
  - "Identify recurring failure patterns"
  - "Determine the earliest lifecycle point where it can be reliably detected"
  - "Focus on failures that are mechanically identifiable from deterministic
    environment signals"
Update constraints (verbatim from paper):
  - "Triggered by precise environment or trajectory evidence"
  - "Local and minimal"
  - "Do not override model reasoning when correct action is ambiguous"
  - "Preserve original environment and evaluation protocol"
  - "Robust to unseen tasks from same environment"
Output format required:
  1. Summary of dominant failure patterns
  2. Harness layer responsible for each update
  3. Implemented code changes
  4. Explanation of safety under deterministic contract
  5. Remaining failure modes for next iteration

REGRESSION CHECK (Appendix A.2, mandatory step):
"Inspect cases where the harness may over-trigger, block a valid action,
inject misleading guidance, or reduce performance on previously successful
trajectories."
This check runs WITHIN the evolution prompt before finalizing changes.

Step 4: Iterate evolution rounds until performance saturates
  - No formal stopping criterion stated; paper uses "performance improves
    steadily...and eventually saturates" heuristic
  - Evolution run on single backbone (Qwen3-4B-Instruct) only

EVALUATION PROTOCOL

Benchmarks (7 total):
  tau-bench: Airline, Retail (tool-agent-user interaction)
  tau2-bench: Telecom (conversational dual-control)
  AgentBench: ALFWorld, WebShop, OS, DBBench

Backbone models (18 total):
  1 source (Qwen3-4B-Instruct, harness evolved on this)
  17 evaluation-only (Qwen-family, Llama-family, xLAM-family)
  Includes: instruction-tuned, reasoning models, agentic post-trained models

Sampling: temperature = 0.0 for all evaluations
User LLM for tau/tau2-bench: DeepSeek-V4-Flash
Pass^3 metric: task repeated 3 times; all 3 must succeed

Mean relative improvement: 88.5%
  Computation: per-benchmark relative improvement = (harness - baseline) / baseline * 100
  Mean: average across 7 benchmarks
  Improved settings: 116 of 126 (92%)

Per-benchmark absolute results (baseline -> harness):
  ALFWorld:  41.1% -> 75.7%  (+84% relative)
  DBBench:   48.4% -> 64.6%  (+34% relative)
  WebShop:   31.4% -> 44.0%  (+40% relative)
  OS:        34.7% -> 41.2%  (+19% relative)
  Airline Pass@1: +26% relative; Pass^3: +50% relative
  Retail Pass@1:  +10% relative; Pass^3: +19% relative
  Telecom Pass@1: +25% relative; Pass^3: +27% relative

vs. prompt-only evolution: Life-Harness achieves +120% relative improvement
vs. xLAM-2-32B (tool-use trained): Qwen2.5-32B+harness surpasses by 7.5pp
Stacking on trained model: applying to xLAM improves all groups +6.8 to +28.9pp

ABLATION (leave-one-layer-out, Qwen3-4B-Instruct, Table 2):
Layer removed    | Airline | Retail | Telecom | ALFWorld | WebShop | OS    | DBBench
Contract removed | -8.3%   | -17.5% | -16.0%  | -1.0%    | -4.4%   | -14.1%| -16.9%
Skill removed    | -8.3%   | -15.9% | -17.4%  | -1.0%    | -2.2%   | -14.1%| -3.1%
Action removed   | -61.7%  | -15.9% | -10.1%  | -1.0%    | -6.6%   | -59.6%| -4.6%
Trajectory rem.  | -3.3%   | -16.7% | -36.2%  | -86.5%   | -26.4%  | -14.1%| -4.6%

Interpretation: Action Realization dominates for tool-schema-heavy tasks
(Airline, OS). Trajectory Regulation dominates for state-tracking tasks
(ALFWorld). Contract/Skill dominate for protocol-heavy tasks (Retail, Telecom).

INPUTS to Life-Harness system
  x          | string  | task description                       | mandatory
  C          | string  | environment contract (from env)        | mandatory
  E          | object  | environment instance with Init/Step    | mandatory
  B          | int     | step budget                            | mandatory, >0
  H          | object  | harness (4 layers)                     | mandatory, evolved offline
  S          | list    | skill memory from training trajectories| optional (Layer 2)

OUTPUTS
  tau_T      | list    | complete trajectory                    | terminal episode
  o_T        | string  | final observation                      | may encode success/fail
  success    | bool    | as determined by IsEnd(s_T, o_T)       | env-defined

HYPERPARAMETERS / CONFIGURATION
  K (top-k retrieval) | 1     | {1..N} | paper uses top-1 only; higher K risks context contamination
  temperature         | 0.0   | [0,1]  | paper uses 0.0 for determinism
  B (step budget)     | env-defined | varies per benchmark | runtime parameter

CRITICAL INVARIANTS (testable assertions)

assert layer_execution_order == ["contract", "skill", "action_realization", "trajectory_regulation"]
# Layer order is FIXED per Algorithm 1; not configurable

assert z_t in ("EXEC", "BLOCK")
# RealizeAction has exactly two output types; no partial execution

assert b_t == B - t - 1
# Remaining budget passed to RegulateTrajectory must equal this formula

assert len(retrieved_skills) <= 1
# Top-1 skill retrieval is a hard cap; >1 skill risks context contamination

assert delta_C_is_text_only == True
# Contract layer output is text fragment, NOT executable code or tool calls

assert action_blocked_only_if_deterministic == True
# RealizeAction MUST NOT block on ambiguous cases; only deterministically wrong

assert harness_not_evolved_during_evaluation == True
# H is frozen at eval time; harness_evolution runs ONLY on training trajectories

assert failure_annotation_priority == ["action_realization", "env_contract", "trajectory_degeneration", "residual"]
# Appendix A.1 verbatim ordering; mutually exclusive categories, checked in order

COMPLEXITY
Layer 1 (Contract):   O(1) per episode, text concatenation
Layer 2 (Skill):      O(|S| * |x|) BM25 retrieval at episode start
Layer 3 (Action):     O(1) per step, deterministic validation against schema
Layer 4 (Trajectory): O(|tau_t|) per step, pattern matching over trajectory history
Total per episode:    O(B * max(O(|tau|), O(|S|*|x|)))
Parallelism:          Layers 1 and 2 are independent and CAN run in parallel;
                      Layers 3 and 4 are sequential (3 before execution, 4 after)

STORAGE FORMATS PER INTERVENTION TYPE
Contract (Layer 1): text fragment, appended to system prompt, UTF-8 string
Skill (Layer 2):    text strategy description, stored in list S, keyed by
                    BM25-retrievable terms from task description
Action rules (Layer 3): code rules implementing deterministic validation
                    (concrete examples: duplicate payment ID checks, argument
                    constraint validators); stored as executable code in harness
Trajectory rules (Layer 4): code monitoring patterns + text feedback messages;
                    stored as executable code + string templates in harness

BM25 RETRIEVAL KEYING
  Key:   task description x (free text)
  Value: skill text entry k
  Score: BM25(x, k) using standard term-frequency / inverse-document-frequency
  No specific k1, b, epsilon values stated; implementation-defined
  Library: not specified; any BM25 implementation is valid

FAILURE MODES (from paper)
  1. Over-triggering in Layer 3: blocks valid actions; regression check required
  2. Misleading guidance injection in Layer 4: soft messages misdirect recovery
  3. Context contamination in Layer 2: >1 retrieved skill pollutes reasoning
  4. Harness-induced rigidity: blocks ambiguous-but-correct actions
  5. Environment coverage gap: harness evolved on one env fails on unseen envs
  6. Residual reasoning failures NOT addressable by harness (paper's explicit limit)
  7. Open-ended tasks: harness is explicitly inapplicable (authors' scope warning)

IMPLEMENTATION PITFALLS (do NOT do this)

Do NOT apply to open-ended tasks with varying goals/tools -- paper is explicit:
  "Extension to fully open-ended agent tasks remains challenging"

Do NOT block on ambiguous actions in Layer 3 -- constraint from paper:
  "do not override model reasoning when correct action is ambiguous"

Do NOT retrieve more than top-1 skill -- paper finding:
  "use only the top-1 retrieved skill across all experiments" to prevent
  irrelevant skills contaminating model context

Do NOT evolve the harness using evaluation trajectories -- only training
  trajectories; harness is frozen at evaluation time

Do NOT add Layer 3 rules based on semantic / reasoning failures -- those are
  Priority 4 residual; Layer 3 handles ONLY deterministic interface failures

Do NOT modify tau (trajectory) directly from harness layers -- Layer 3 returns
  EXEC or BLOCK (the block message is appended as observation, not a raw edit)

Do NOT skip the regression check during harness evolution -- the Appendix A.2
  template includes it as a mandatory step; over-trigger errors compound across
  iterations if unchecked

Do NOT conflate Environment Contract (Layer 1) with Procedural Skill (Layer 2):
  Contract = stable environment rules (policy, tool schemas, pitfalls)
  Skill = task-specific procedural strategy retrieved per task

RAG INGEST STATUS
mcp_ingest_file call failed with IPC timeout (known reranker crash).
Interface map written from direct HTML extraction only.
Local file: F:\.ELAI_workflow\.claude\papers\2605.22166.html
