# Hierarchical task decomposition for ELAI (May 2026)

Scope: 2024-2026 SOTA in hierarchical decomposition for autonomous agents, mapped onto ELAI's 4-level recursive tree (Idea > Concept > Phase > Step > BabyStep) with per-level auditor dispatch (B-41) and 5-action bubble-up cascade.

## TL;DR
- Add an explicit dual-ledger (task-facts + progress-ledger) at the Concept and Phase levels, mirroring Magentic-One's outer/inner loop; this gives ELAI a structured replan trigger condition rather than relying solely on bubble-up cascade. (arxiv 2411.04468)
- AGENTLESS evidence (32.00% SWE-bench Lite at $0.70/issue, beating agentic systems) argues for short-circuiting decomposition at the leaf for narrow, well-localized fixes; add a "decomposition skip" gate at the Step level when AGENTLESS-style file>class>line hierarchical localization is sufficient. (arxiv 2407.01489)
- Adopt a hard recursion depth limit (max 3 sub-levels under any BabyStep) plus a "decomposition entropy" guard - ELAI already caps at depth 2 for Phase Sizing; extend the rule to all levels and emit a STAGNATION bubble-up when divisions stop yielding new acceptance criteria.
- Add a Reflexion-style verbal episodic memory layer between auditor failure and the next decomposition attempt; ELAI's Critic Loop (B-40) and bubble-up handler cover this partially, but the canonical Reflexion pattern keeps the textual reflection in an episodic buffer rather than re-embedding it in the next plan prompt. (arxiv 2303.11366)
- LATS/MCTS-style tree search is overkill for ELAI's deterministic depth-first TreeNavigator, but the LATS value-function + self-reflection pair is reusable: spawn the per-level auditor as a value-function evaluator at every decomposition fork, not just at execution time. (arxiv 2310.04406)

## Key sources
| arxiv/URL | what | takeaway |
|---|---|---|
| arxiv 2407.01489 (AGENTLESS) | Three-phase localization+repair+validation, no agency | Hierarchical file>class>line localization wins SWE-bench Lite at 32.00% / $0.70 - less decomposition can dominate more |
| arxiv 2411.04468 (Magentic-One) | Orchestrator with dual-ledger outer/inner loop | Task ledger (facts/guesses/plan) + progress ledger (current/assignment) as canonical replan trigger structure |
| arxiv 2310.04406 (LATS) | MCTS over reasoning+acting, LM-powered value functions + self-reflection | Value-function-as-auditor pattern; deliberate search beats greedy on coding tasks |
| arxiv 2305.16291 (Voyager) | Skill library + automatic curriculum + iterative prompting | 3.3x more items, 15.3x faster tech tree milestones via compositional skill reuse |
| arxiv 2308.00352 (MetaGPT) | SOP-encoded role-based decomposition, ICLR 2024 | Standardized procedures + role assembly line reduces cascading hallucinations |
| arxiv 2405.15793 (SWE-agent) | Agent-Computer Interface (ACI), NeurIPS 2024 | Constrained action surface + structured per-turn feedback gives SOTA on SWE-bench among agentic systems |
| arxiv 2308.09687 (Graph-of-Thoughts) | Arbitrary graph over LLM thought units | +62% sort quality and -31% cost vs ToT - graph topology beats tree where merges/feedback matter |
| arxiv 2305.04091 (Plan-and-Solve) | Plan first, then execute subtasks zero-shot | Foundational two-step decompose-then-solve pattern; PS+ adds detail instructions for reasoning quality |
| arxiv 2305.18323 (ReWOO) | Planner/Worker/Solver decoupling | Plan blueprint independent of observations - cheaper, parallelizable workers |
| arxiv 2303.11366 (Reflexion) | Verbal RL via episodic reflection buffer | SOTA on code-gen via textual semantic-gradient feedback; canonical plan-repair signal |
| arxiv 2506.12508 (AgentOrchestra) | TEA-protocol hierarchical orchestrator | 82.42% GAIA, 95.3% SimpleQA - central planner + specialized sub-agents with versioned interfaces |
| arxiv 2505.02709 (Goal Drift, AIES 2025) | Evaluating goal drift in LM agents | Drift is measurable; PRMs (process reward models) supervise step-by-step to correct deviations |
| arxiv 2603.14248 (Web Agents Hierarchical) | High-level plan / low-level exec / replan failure analysis | Three-layer evaluation isolates planning vs execution vs feedback failures |
| arxiv 2511.18165 (HTN with LLMs) | General framework for HTN modeling with LLMs | Classical HTN (Erol-Hendler-Nau) reusable as backbone with LLM filling decomposition methods |
| arxiv 2505.11814 (ChatHTN) | Interleaves symbolic HTN with LLM fallback | SHOP-style depth-first; LLM only called when no symbolic method matches - sharp GPT call rate reduction |
| arxiv 2503.13657 (Why Multi-Agent Fails) | Failure-mode taxonomy of multi-agent LLM systems | Improper task decomposition (too granular / too broad / not serializable) is a documented top failure |
| arxiv 2305.14078 (LLM-MCTS) | LLM as commonsense world model inside MCTS | Heuristic policy + prior belief makes MCTS scale to novel tasks |
| LangChain plan-and-execute blog | Production plan-and-execute pattern with replan loop | Re-plan node inspects results, decides revise-or-continue - directly maps to ELAI bubble-up |
| arxiv 2401.14295 (Demystifying Chains/Trees/Graphs) | Comparative analysis of CoT/ToT/GoT topologies | Topology choice matters: graph wins on tasks with cross-branch synergy, tree on independent branches |

## Decomposition pattern landscape
| Pattern | System | Levels | Replan support | Benchmark | Citation |
|---|---|---|---|---|---|
| Recursive decomp | ELAI | 4 (Idea>Concept>Phase>Step>Baby) | Bubble-up 5-action | internal | this repo |
| 3-phase localization | AGENTLESS | 3 (file>class>line) | None (one-shot, no agency) | SWE-bench Lite 32.00% | 2407.01489 |
| Dual-ledger orchestrator | Magentic-One | 2 (task ledger / progress ledger) | Outer-loop ledger reset | GAIA top results 2024 | 2411.04468 |
| MCTS search | LATS | tree of (reason,act) nodes | Self-reflect + value rollback | HotpotQA/HumanEval | 2310.04406 |
| Plan-and-solve | PS / ReWOO | 2 (plan / execute) | ReWOO Solver re-aggregates | GSM8K, multi-step QA | 2305.04091, 2305.18323 |
| Role SOP | MetaGPT | PM>Architect>Engineer>QA | SOP step retry | HumanEval, MBPP | 2308.00352 |
| Skill library | Voyager | continuous curriculum | Iterative prompt + verify | Minecraft tech tree | 2305.16291 |
| ACI flat loop | SWE-agent | 1 (constrained tool surface) | None - flat ReAct | SWE-bench | 2405.15793 |
| HTN symbolic+LLM | ChatHTN | symbolic depth-first + LLM fallback | Backtrack on method fail | classical IPC | 2505.11814 |
| TEA hierarchy | AgentOrchestra | central planner + sub-agents | Versioned interfaces | GAIA 82.42% | 2506.12508 |
| Plan-and-execute | LangGraph | 2 (plan / step exec) | Re-plan node | applied | LangChain blog |
| Verbal RL | Reflexion | episodic reflection buffer | Per-trial verbal update | LeetcodeHardGym SOTA | 2303.11366 |
| Graph topology | GoT | DAG of thoughts | Feedback edges | sort/set tasks | 2308.09687 |

## When less decomposition wins
AGENTLESS shows the most decisive case: a non-agentic three-phase pipeline (hierarchical localization > repair > validation) reached 32.00% on SWE-bench Lite at $0.70 per issue, beating every open-source agentic system at the time (arxiv 2407.01489 v2, Oct 2024). The agent never decides next actions; the structure is fixed. Two implications for ELAI:

1. For narrow, well-localized BabySteps where the fix is "edit N lines in file F", the per-level auditor should skip further decomposition and dispatch a direct AGENTLESS-style file>class>line localizer. ELAI already has an impact-mapper agent; this is the missing leaf-shortcut.
2. The Kimi-Dev follow-up (arxiv 2509.23045, 2025) treats AGENTLESS-style traces as skill priors for SWE-agents - i.e. the non-agentic pipeline becomes a training signal, not a competitor. ELAI could log successful BabyStep traces as "skill prior" exemplars for future runs (Voyager-style skill library, arxiv 2305.16291).

The over-decomposition failure mode is explicit: arxiv 2503.13657 (Why Multi-Agent LLM Systems Fail) lists "improper task decomposition - too granular, too broad, not serializable" as a top failure category, with poorly partitioned code-generation producing incoherent outputs that cannot be reintegrated. ELAI's deterministic TreeNavigator + DAG repair already mitigate this, but the *trigger* to stop decomposing is currently implicit (depth cap = 2 for Phase Sizing only). A measurable "decomposition entropy" stop rule is missing.

## Plan repair patterns (when execution reveals plan flaws)
Three canonical patterns from the literature:

1. **Reflexion verbal reflection** (arxiv 2303.11366): convert environment feedback to textual reflection, append to episodic memory, re-run. Acts as a semantic gradient. ELAI's bubble-up cascade approximates this but does not maintain a persistent episodic reflection buffer between BabyStep retries - the failure context is in the audit log, not a structured per-tree-node memory.

2. **LATS MCTS rollback** (arxiv 2310.04406): backtrack to the highest-value uncommitted ancestor and re-expand. ELAI's TreeNavigator is depth-first deterministic; the closest analogue is AutoB2G's DAG repair (Reorder + Substitute, 10 rounds). LATS-style backtracking with a value function (= per-level auditor verdict score) would let ELAI re-expand a Step instead of the entire Phase when a BabyStep fails repeatedly.

3. **LangGraph re-plan node** (LangChain plan-and-execute blog): an explicit graph node that inspects results and routes to "continue" vs "revise". Maps onto ELAI's bubble-up handler but is more structured: the re-plan node has its own state and can call the planner with the failed plan + error + original goal as inputs. ELAI's bubble-up already has 5 actions; a 6th could be "REPLAN_PHASE" that re-runs phase-decomposer with the failure transcript attached.

The hierarchical-web-agents work (arxiv 2603.14248) decomposes failure into (a) high-level planning error, (b) low-level execution error, (c) feedback-use error - mirroring ELAI's per-level auditor modes. A measurable wins: classify each bubble-up event into these three buckets before dispatching the action.

The "task-decoupled DAG with scoped replan" pattern reports up to 82% token reduction by confining replanning to the active sub-task. ELAI's per-level scoping (B-41) should support this; verify the implementation does not re-load the parent Phase context on a BabyStep replan.

## Best additions for ELAI's 4-level
Ranked by expected impact x integration cost.

1. **Dual-ledger at Concept and Phase levels** (Magentic-One, arxiv 2411.04468). Add `TaskLedger { facts, guesses, plan }` and `ProgressLedger { current, assignment, blockers }` to ConceptNode and PhaseNode. Integration hook: spawn ledger update in the per-level auditor (auditor_mode = concept | phase). BubbleUp action: "LEDGER_RESET" forces outer-loop replan when progress ledger shows blockers > threshold.

2. **AGENTLESS leaf-shortcut** (arxiv 2407.01489). At Step level, detect "localizable fix" (single-file, single-symbol, well-grounded acceptance criteria) and skip BabyStep decomposition; dispatch a 3-phase localize+repair+validate pipeline directly. Integration hook: Step-level auditor's audit_mode adds `agentless_shortcut` verdict. Maps to per-level auditor (B-41) and could be triggered by impact-mapper detecting depth-1 blast radius.

3. **Reflexion episodic buffer per tree node** (arxiv 2303.11366). Persist textual failure reflections on every ConceptNode/PhaseNode/StepNode/BabyStepSpec, separate from audit logs. On retry, prepend the buffer to the agent prompt. Integration hook: bubble-up handler writes to node.reflection_buffer; tree navigator reads it before re-dispatching the same level.

4. **REPLAN_PHASE bubble-up action** (LangGraph re-plan node + LATS value backtrack). Add a 6th bubble-up action that re-runs phase-decomposer with (failed plan, error transcript, original Phase goal). Integration hook: bubble-up cascade action[6] = REPLAN_PHASE, triggered when same BabyStep fails twice OR when a sibling BabyStep contradicts an earlier one (goal-drift signal, see below).

5. **Decomposition entropy stop rule** (over-decomp failure, arxiv 2503.13657). Compute Shannon-style entropy over child node acceptance-criteria token sets; if entropy of children < entropy_threshold relative to parent, halt further decomposition and treat parent as atomic. Integration hook: ConceptDecomposition / PhaseDecomposition exit condition. Companion: hard depth-3 ceiling under any BabyStep (extend Phase Sizing rule).

6. **Value-function-as-auditor at every fork** (LATS, arxiv 2310.04406). Per-level auditor (B-41) currently runs at execution time; also run it at decomposition forks with a "value estimate" verdict (LOW/MED/HIGH likelihood of executable acceptance criteria). Integration hook: scope-planner-elai consumes the value estimate; low-value forks get pruned before TreeNavigator commits.

7. **Skill library promotion** (Voyager, arxiv 2305.16291). Successful BabyStep traces become "skill priors" indexed by goal embedding. On future Phase decomposition, retrieve top-K skill priors and condition the decomposer. Integration hook: ResearchCache already exists - extend with skill embeddings and surface in scope-planner-elai inputs.

## Goal-drift detection
arxiv 2505.02709 (AIES 2025) is the canonical evaluation: goal drift = deviation from instruction-specified goal over extended horizons; frontier models surfaced conditional deception and self-preservation in long-horizon scenarios. Mitigations they validate:

- **Process Reward Models (PRMs)** provide dense step-by-step supervision; ELAI's per-level auditor (B-41) is structurally a PRM if we re-frame audit verdicts as scalar process rewards rather than pass/fail gates.
- **Task-attention weighted embeddings** maintain a stateful reference to the original goal; useful at the Concept level where decomposition into Phases can re-frame the user's actual ask.

Concrete ELAI hook: at the start of every Phase audit, compute cosine similarity between current Phase's acceptance criteria embedding and the original Idea embedding. If similarity drops below a threshold across two consecutive Phases, raise a `GOAL_DRIFT` bubble-up event. This is a cheap addition because the Idea-Framer (B-40) already produces a goal embedding.

DeepContext (arxiv 2602.16935) and AgentDebug (arxiv 2509.25370 framework summary) show error-propagation tracing as the dominant route to robust agents - i.e. root-cause the drift to the specific decomposition step, not the final output. Per-level auditor + per-node reflection buffer give ELAI the substrate for this.

## Anti-patterns
- **Unbounded recursion via Task tool nesting** (OpenCode issue 18100, RLM analyses arxiv 2603.02615). Suggested community default: max nesting depth = 3. ELAI's 4 levels are already a hard tree topology; the risk is decomposer agents themselves recursing internally. Enforce explicit depth tracking in agent spawning.
- **Single-message agent looping until token budget** (multi-agent failure taxonomy, arxiv 2503.13657). The "improper decomposition - too granular" failure happens when the model fragments a single coherent edit into 10 micro-edits across files. Anti-pattern signal: > N BabySteps per Step where N depends on Step complexity; auditor should reject decompositions with uniform tiny acceptance criteria.
- **Over-reliance on tree topology when graph edges exist** (GoT, arxiv 2308.09687; arxiv 2401.14295 comparative). When BabySteps share dependencies, a tree forces redundant context. ELAI's DAG repair (AutoB2G) acknowledges this at the Concept level; extend the same DAG awareness to Steps within a Phase (cross-Step shared state should be modeled as edges, not duplicated context).
- **Decomposing past the model's atomic capability** (depth ceiling, arxiv 2604.06427). When a leaf BabyStep is already smaller than the model can usefully separate, further decomp produces noise. The decomposition entropy rule (Best Additions #5) addresses this.
- **Treating audit as binary pass/fail** (PRM literature). Throws away the gradient signal Reflexion uses. Per-level auditor (B-41) should return a scalar plus structured reasons, not just OK/BLOCK.
- **Stale plan re-execution after partial success** (LangChain plan-and-execute blog). When some BabySteps succeed and some fail, naively retrying the failed ones without re-checking upstream assumptions risks compounding bad state. ELAI's bubble-up cascade should validate sibling acceptance still holds before retrying a failed sibling.

## Open questions
1. Does ELAI's per-level auditor (B-41) produce scalar verdicts or only categorical? If categorical, can it be retrofitted to emit a process reward without breaking existing audit_mode dispatch? Measurable: re-run last 20 BabyStep audits and check verdict variance.
2. At what tree depth does ELAI's decomposition stop adding measurable acceptance-criteria entropy? Measurable: instrument ConceptDecomposition/PhaseDecomposition with the entropy metric and log per-Idea.
3. Does AGENTLESS-style leaf-shortcut win on ELAI's actual workload mix, or is its 32% SWE-bench Lite an artifact of that benchmark's locality assumption? Measurable: classify last 50 BabySteps as "single-file localizable" vs "multi-file" and compare pass rates.
4. Does the Reflexion episodic buffer per node measurably reduce retry count vs current bubble-up cascade? Measurable: A/B on next 30 BabyStep failures.
5. Is goal drift detectable in current ELAI traces via Idea-embedding cosine across Phases? Measurable: backfill cosine on B-37/B-40/B-41/B-42 trees and look for monotonic divergence.

## Saved PDFs
- /workspace/ELAI/workflow/improvement-sweep-2026-05-19/pdfs/agentless-2407.01489.pdf
- /workspace/ELAI/workflow/improvement-sweep-2026-05-19/pdfs/magentic-one-2411.04468.pdf
- /workspace/ELAI/workflow/improvement-sweep-2026-05-19/pdfs/lats-2310.04406.pdf
- /workspace/ELAI/workflow/improvement-sweep-2026-05-19/pdfs/goal-drift-2505.02709.pdf
- /workspace/ELAI/workflow/improvement-sweep-2026-05-19/pdfs/agentorchestra-2506.12508.pdf
