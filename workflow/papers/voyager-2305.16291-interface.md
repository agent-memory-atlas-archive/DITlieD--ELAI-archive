Algorithm: Voyager — Open-Ended Embodied Lifelong Learning Agent
ArXiv: 2305.16291
Paper: Voyager: An Open-Ended Embodied Agent with Large Language Models
Authors: Guanzhi Wang, Yuqi Xie, Yunfan Jiang, Ajay Mandlekar, Chaowei Xiao, Yuke Zhu, Linxi Fan, Anima Anandkumar (NVIDIA, Caltech, UT Austin)
Published: May 2023

=== D-12 SCOPE NOTE ===
D-12 operationalization-check predicate ground: Voyager retains a skill in the
skill library ONLY after the critic agent returns success=True, which requires
a measurable state change observable in the Minecraft environment (inventory
delta, position change, block state change). This is the canonical definition
of "skill with measurable effect" used in D-12. A skill that produces no
detectable state change is never added to the library regardless of whether
code executed without error.

Inputs:
environment        | Minecraft JS env (Mineflayer)  | stateful object     | must expose bot.chat(), inventory, position, biome, time, health, entities, blocks
task               | string                          | natural language    | proposed by curriculum agent; must be achievable given current agent_state
agent_state        | dict                            | {inventory, equipment, blocks, entities, biome, time, health, position} | read after each environment.step()
skills (context)   | list[string]                   | top-5 retrieved JS functions | retrieved by embedding similarity; optional at first iteration
code (previous)    | string | JS function body | output of prior action_agent round; None on first round
environment_feedback | string                       | bot.chat() output   | intermediate execution progress messages; None on first round
execution_errors   | string                         | interpreter stderr  | syntax/API errors from JS runtime; None on first round
critique           | string                         | critic agent text   | suggestions for improvement on failure; None on first round

Outputs:
code               | string  | JS async function  | must be syntactically valid JS, must call only defined Mineflayer APIs
success            | bool    | scalar             | True iff critic agent confirms task completion from agent_state
critique           | string  | natural language   | non-empty iff success=False; contains actionable suggestions
skill_added        | bool    | scalar             | True iff success=True AND code added to skill library this round
agent_state_post   | dict    | same schema as input agent_state | reflects environment after code execution

Hyperparameters:
max_rounds_per_task  | 4        | [1, inf)  | high sensitivity: ablation not shown but "stuck after 4 rounds" is hard cutoff
top_k_skills         | 5        | [1, 20]   | number of retrieved skills injected into action_agent context; authors use 5
embedding_model      | GPT-3.5  | -         | used for skill description embedding; substituting changes retrieval quality
action_agent_model   | GPT-4    | -         | high sensitivity: GPT-3.5 substitution yields 5.7x fewer unique items (Section 3.4)
critic_agent_model   | GPT-4    | -         | same sensitivity as action_agent_model
curriculum_model     | GPT-4    | -         | replaces GPT-4 with random tasks -> -93% item discovery (Section 3.4 ablation)
exploration_goal     | string   | "discover as many diverse things as possible" | fixed; changing this changes the curriculum distribution

Critical invariants (testable assertions):
# INV-1 (Section 2.2, skill library): a skill is added iff and only iff success=True
assert skill_added == success

# INV-2 (Section 2.3, self-verification): critic must receive BOTH task AND post-execution agent_state
assert task is not None and agent_state_post is not None  # before critic call

# INV-3 (Section 2.2, retrieval): retrieved skills are top-K by cosine similarity of description embeddings
assert len(retrieved_skills) <= 5

# INV-4 (Section 2.3, iteration cap): at most 4 refinement rounds per task before abandonment
assert round_index <= 4

# INV-5 (skill-with-measurable-effect, D-12 ground): a skill produces a measurable effect iff
# agent_state_post differs from agent_state_pre on at least one observable field
# (inventory delta OR position delta OR block state delta OR entity state delta)
assert (agent_state_post['inventory'] != agent_state_pre['inventory'] or
        agent_state_post['position'] != agent_state_pre['position'] or
        agent_state_post['blocks'] != agent_state_pre['blocks'] or
        agent_state_post['entities'] != agent_state_pre['entities'])
# NOTE: success=True without any state delta is a critic hallucination (known failure mode)

# INV-6 (curriculum): failed tasks are recorded and influence future proposals
assert failed_task in curriculum_agent.failed_tasks

# INV-7 (compositionality): skills in library are callable from within other skill code
# verified structurally: each skill is a named async JS function with a docstring
assert skill_code.strip().startswith('async function') or 'async function' in skill_code

Complexity:
Time per task        | O(R * (embed + LLM_call + env_step)) where R <= 4 rounds
Space (skill library)| O(N * (embed_dim + avg_code_len)); grows unboundedly with N skills discovered
Retrieval            | O(N * embed_dim) per query (cosine scan); no index pruning mentioned
Parallelism          | None within a task; curriculum proposes tasks sequentially
LLM calls per task   | 3 per round (action_agent + critic_agent + optional curriculum) * up to 4 rounds = up to 12 GPT-4 calls per task

Failure modes (from paper, Section 4 / ablations):
1. Curriculum hallucination: proposes non-existent items (e.g. "copper sword") -> task always fails, wastes 4 rounds
2. Critic hallucination: self-verification incorrectly reports success (e.g. fails to recognize spider string as kill evidence) -> invalid skill added to library
3. API hallucination: action agent calls undefined Mineflayer APIs -> execution_errors non-empty, caught by iterative prompting
4. Stuck agent: agent fails 4 rounds and moves to next task; skill never learned; no retry mechanism
5. Skill library cold-start: first few iterations have no retrieved skills; code quality lower
6. GPT-4 dependency: substituting GPT-3.5 for action or critic agent causes 5.7x performance collapse (Section 3.4)
7. Curriculum removal: random tasks cause -93% item discovery; curriculum is non-optional for exploration efficiency

Implementation pitfalls (do NOT do this):
1. Do NOT add a skill to the library on execution-success (no errors) alone; success requires critic agent confirmation of state change, not just clean execution
2. Do NOT skip self-verification to save LLM calls; it is the single most important component (-73% without it, Section 3.4 ablation)
3. Do NOT use a fixed task sequence instead of the adaptive curriculum; random tasks yield -93% performance
4. Do NOT substitute GPT-3.5 for GPT-4 in action or critic roles expecting similar results; 5.7x degradation is documented
5. Do NOT allow unbounded rounds per task; hard cutoff at 4 is required to prevent infinite loops on impossible tasks
6. Do NOT retrieve skills by keyword match; embedding-based similarity retrieval is required for compositional reuse
7. Do NOT treat execution without errors as a proxy for "measurable effect"; the critic checks agent_state_post, not just absence of exceptions

D-12 Operationalization-Check Predicate (canonical ground):
A skill S qualifies as "operationalizable" (retainable) iff ALL of:
  (a) S is syntactically valid executable code (no interpreter errors)
  (b) S produces a measurable state change: agent_state_post != agent_state_pre on at least one observable field
  (c) The critic agent (GPT-4, given task + agent_state_post) returns success=True
  (d) The state change is attributable to S (not pre-existing or concurrent)
Condition (b) is the "measurable effect" criterion. A skill that runs cleanly but produces no state delta
fails condition (b) and must NOT be added to the library. This maps directly to D-12's predicate:
"is this claim/skill actually verifiable via measurable side effects?"
The answer is YES iff conditions (a)+(b)+(c) all hold after at most 4 execution rounds.
