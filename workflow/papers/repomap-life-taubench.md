repomap: Life-Harness / TauBench (tau2-bench) — METHOD MAP for ELAI steal

Source: C:\Users\user\Downloads\new\Life-Harness-main\TauBench
Top README: C:\Users\user\Downloads\new\Life-Harness-main\README.md
Paper: "Adapting the Interface, Not the Model: Runtime Harness Adaptation for Deterministic LLM Agents" (arXiv:2605.22166, Xu/Wen/Li 2026).
Underlying benchmark: this is tau2-bench (the Sierra tau-bench v2, pyproject name "tau2", v1.0.0, MIT, authors Barres/Dong). Life-Harness is a fork that bolts a 4-layer runtime "harness" onto tau2-bench. Data files (data/**, tasks*.json, *.jsonl, *.wav, web/, leaderboard/) were NOT read — they are ~900K tokens of pure benchmark content and out of scope. The method below is from the harness/env/agent/runner/evaluator Python only.

=========================================================================
1. WHAT IT IS
=========================================================================
tau2-bench is a tool-agent + conversation evaluation harness. It runs a frozen LLM "agent" (customer-service rep) against an LLM "user simulator" inside a deterministic, stateful domain environment (airline / retail / telecom / banking_knowledge). The agent reads a domain policy, calls domain tools that mutate a database, and must satisfy the user's goal. After the dialogue terminates, the run is graded by comparing the FINAL DATABASE STATE against a gold environment (DB-hash equality), plus action-trace checks, communicate-info checks, and optional LLM-judged NL assertions. Reliability is reported as pass^k (all k independent trials must succeed), not just average reward.

WHAT LIFE-HARNESS ADDS on top of upstream tau-bench (this is the paper's contribution — "adapt the interface, not the model"):
A 4-layer runtime "harness" that wraps the SAME frozen model and the SAME (intact) environment, intervening at four lifecycle points to mechanically eliminate execution-level failures. The layers are evolved offline by a CLI coding agent (Codex) that reads the previous run's failed trajectories + a design guide and edits the harness code — a closed self-improvement loop on the HARNESS, not the model. Reported: 7 benchmarks x 18 backbones, 116/126 model-env settings improved, +88.5% avg relative gain, fully training-free.
- h2 Action Realization Layer — pre-execution tool-call validation/repair/block (file: harness/base.py HarnessRule + per-domain *.py rules).
- h3 Environment Contract Layer — policy constraints embedded into tool.description, re-sent every API call so they never decay (harness/h3_tools.py).
- h4 Trajectory Regulation Layer — post-execution state-aware annotations appended to tool results; orchestrator owns step/error-budget termination (harness/base.py HarnessAnnotator).
- h5 Procedural Skill Layer — BM25-retrieved learned "skills" (tips mined from training trajectories) prepended to the system prompt at episode start (harness/skills.py).
A master --enabled switch gates all four. The harness reads no oracle/test labels and never edits the evaluator (the gate is frozen to it).

=========================================================================
2. STRUCTURE MAP (code modules, one-line roles)
=========================================================================
src/tau2/
  orchestrator/orchestrator.py     — BaseOrchestrator + half-duplex Orchestrator: the turn-based AGENT<->USER<->ENV message loop, termination, finalize->SimulationRun.
  orchestrator/full_duplex_orchestrator.py — streaming/voice variant (trajectory of Ticks not Messages).
  orchestrator/modes.py            — CommunicationMode (HALF_DUPLEX / FULL_DUPLEX), solo mode.
  user/user_simulator.py           — LLM-as-USER: role-flipped LLM driven by scenario instructions + global guidelines + persona; emits STOP/TRANSFER/OUT_OF_SCOPE sentinels.
  user/user_simulator_base.py      — HalfDuplexUser base, UserState, stop sentinels.
  agent/llm_agent.py               — the agent under test (LLMAgent / LLMSoloAgent); system prompt = domain policy (+H5 prefix).
  agent/base_agent.py              — HalfDuplexAgent base, ###STOP### handling, history validation.
  environment/environment.py       — Environment: tool execution, DB-hash, set_state REPLAY (deterministic state reconstruction from message history), env assertions.
  environment/toolkit.py           — ToolKitBase, ToolType (read/write/think/generic), tool-signature extraction.
  environment/tool.py              — Tool: wraps a python fn -> openai_schema from the docstring (short_desc/long_desc).
  environment/db.py                — DB base with content hash (the un-gameable end-state signal).
  data_model/tasks.py              — Task schema: UserScenario, InitialState, EvaluationCriteria(actions/env_assertions/nl_assertions/reward_basis), Action.compare_with_tool_call.
  data_model/simulation.py         — SimulationRun, RewardInfo, Results, TerminationReason, Review/ReviewError.
  evaluator/evaluator.py           — dispatcher: ENV / ACTION / COMMUNICATE / NL / ALL; final reward = PRODUCT of applicable component rewards gated by task.reward_basis.
  evaluator/evaluator_env.py       — end-state grader: build gold env, replay golden actions, compare DB hashes (agent + user db) + run env_assertions.
  evaluator/evaluator_action.py    — action grader: each golden Action must be matched by some tool call (compare_args subset match); all-or-nothing reward.
  evaluator/evaluator_communicate.py — checks required info strings were said to the user.
  evaluator/evaluator_nl_assertions.py — LLM-judge: grades each NL assertion true/false over the conversation transcript.
  evaluator/review_llm_judge.py    — LLM-judge "conversation reviewer": tags agent AND user errors (severity + error_tags + turn idx) — diagnostic, not reward.
  evaluator/review_llm_judge_user_only.py — user-only review variant.
  evaluator/hallucination_reviewer.py / auth_classifier.py — specialized post-hoc classifiers.
  metrics/agent_metrics.py         — pass^k = comb(successes,k)/comb(trials,k); avg reward, cost, read/write action accuracy, auth/termination/review stats.
  harness/base.py                  — HarnessRule (H2 pre-exec, raises ValueError->fed back as tool error), HarnessAnnotator (H4 post-exec note), HarnessedToolKitMixin.use_tool.
  harness/h3_tools.py              — H3: per-domain {tool_name: policy_hint} appended into tool docstrings via a wraps() shim -> new openai_schema.
  harness/skills.py                — H5: Skill dataclass + per-domain skill libraries + BM25-lite retrieval + issue-type routing + per-skill precision guards + format block.
  harness/retail.py / airline.py / telecom.py / banking_knowledge.py — concrete H2 rules + H4 annotators wired per domain.
  harness/policy_rag.py            — optional retrieve_policy tool (RAG over policy chunks).
  knowledge/                       — BM25/embedding indexers, rerankers, retrievers for the banking_knowledge KB domain (RAG eval).
  runner/batch.py / simulation.py / build.py / checkpoint.py — parallel task runner, single-sim driver, environment/agent factory, resumable checkpoints.
  registry.py / run.py / cli.py    — domain/agent registry + CLI entrypoints (tau2 run).
  voice/**                         — large orthogonal subtree: audio-native real-time (gemini/openai/nova/xai/qwen/livekit) + TTS synthesis + transcription. Not core method.
scripts/eval_harness.py            — the paper's experiment driver: flags --enabled --h2 --h3 --h4 --h5 --h5-top-k, sweeps domains/splits/trials, reports reward + pass^k + agent_tokens.
Harness.md                         — the 4-layer design guide fed to the Codex evolution loop (the method section in prose).

=========================================================================
3. ARCHITECTURE + METHOD (detailed)
=========================================================================

3.1 THE AGENT-ENVIRONMENT-USER LOOP (orchestrator/orchestrator.py)
- Three roles: AGENT, USER, ENV. Strict half-duplex turn routing held in (from_role,to_role,message).
- BaseOrchestrator.run() is a template method: initialize() -> while not done: step(); _check_termination() -> _finalize() -> SimulationRun. A finally-block emergency-cleanup releases agent/user resources on exception.
- step() routes one message:
    AGENT/ENV -> USER: user.generate_next_message(); USER stop sentinel ends sim.
    USER/ENV -> AGENT: agent.generate_next_message(); ###STOP### ends sim.
    AGENT/USER -> ENV: _execute_tool_calls() runs each ToolCall through environment.get_response(); errors counted; results wrapped (ToolMessage or MultiToolMessage) and routed back to the requestor.
- PROTOCOL INVARIANTS (the agent-env contract, enforced if validate_communication=True): a message has EITHER text OR tool calls, never both; never empty; n tool calls must be followed by n tool messages whose requestor matches. Violations -> AGENT_ERROR/USER_ERROR termination.
- SOLO MODE: user replaced by DummyUser; agent may ONLY emit tool calls (or ###STOP###); used for ticket-style autonomous solving. Agent gets both agent+user tools merged (name-overlap forbidden).
- TerminationReason taxonomy: AGENT_STOP, USER_STOP, MAX_STEPS, TOO_MANY_ERRORS, AGENT_ERROR, USER_ERROR, TIMEOUT, INFRASTRUCTURE_ERROR. CRITICAL: evaluate_simulation returns reward 0.0 unless termination is AGENT_STOP or USER_STOP — a run that hit max_steps/errors/timeout auto-fails regardless of DB state. INFRASTRUCTURE_ERROR sims are excluded from metrics entirely (not counted as failures).
- Init can resume from a task's initial_state.message_history (replays prior turns + reconstructs agent/user state).

3.2 USER SIMULATOR — LLM-as-USER (user/user_simulator.py)
- A second LLM is prompted to ROLE-PLAY the customer. system_prompt = global simulation guidelines (data/tau2/user_simulator/*.md) + persona guidelines + <scenario>{task instructions}. Behaviour is split across exactly 3 layers (deliberately non-overlapping): (1) global guidelines for all users, (2) task-baked persona in the task JSON, (3) runtime PersonaConfig.
- KEY TRICK — role flip: state.flip_roles() swaps assistant<->user so the user-LLM "sees" the agent as the user and itself as assistant; then generate() produces the next customer turn. tool_calls (if the user has tools) are re-stamped requestor="user".
- Stop control by SENTINELS in the text: ###STOP### / ###TRANSFER### / ###OUT-OF-SCOPE### end the conversation. Leaked </think> reasoning is stripped.
- Guideline variants selectable by env var: malicious-user (TAU2_MALICIOUS_USER_SIMULATION) loads an adversarial guideline; tools / voice variants exist. eval_harness.py can append a per-domain malicious attack script (fake "[SYSTEM OVERRIDE / PRIOR CASE NOTE ...]" payloads, cabin-upgrade chain attacks) to stress the agent's policy adherence — a built-in red-team / prompt-injection robustness harness.
- DummyUser = no-op user for solo mode.

3.3 TOOL / ENVIRONMENT ABSTRACTION (environment/*)
- Tool wraps a plain python function; its openai_schema (name/params/description) is parsed FROM THE DOCSTRING (docstring-parser). This is what makes H3 work: editing the docstring edits what the model sees.
- ToolKitBase holds tools + a DB; ToolType tags each tool read/write/think/generic (drives read/write accuracy metrics AND replay skip-logic). Environment exposes agent tools and user tools separately.
- DETERMINISM is the whole leverage point. environment.set_state() RECONSTRUCTS state by replaying only the mutating (write) tool calls from a message history; non-mutating reads are skipped (avoids re-execution + nondeterministic output). During replay it re-executes each write and asserts the re-execution output equals the recorded ToolMessage content — a self-consistency check that the trajectory is replayable. DB content hash (get_db_hash / get_user_db_hash) is the canonical end-state fingerprint.
- get_response() executes a ToolCall, catches any exception into "Error: {e}" with error=True (so the agent sees the error text and can self-correct), serialises result to JSON.

3.4 TASK SCHEMA (data_model/tasks.py)
- Task = id + UserScenario(persona + instructions) + optional ticket (solo) + InitialState(initialization_data + initialization_actions + message_history) + EvaluationCriteria + issues/required_documents/user_tools.
- EvaluationCriteria = actions (golden Action list) + env_assertions (boolean env-function checks) + communicate_info (strings the agent must say) + nl_assertions (LLM-judged) + reward_basis (which RewardTypes count: DB / ENV_ASSERTION / ACTION / COMMUNICATE / NL_ASSERTION).
- Action.compare_with_tool_call: name must match; if compare_args set, only those arg values must match (subset/partial-arg matching — robust to extra args); else all args must match. This is the action-grading primitive.

3.5 GRADING / REWARD (evaluator/*)
- evaluate_simulation dispatches by EvaluationType. ALL mode = run ENV + ACTION + COMMUNICATE (+NL), but the FINAL reward is the PRODUCT of only the components named in the task's reward_basis. Product (AND) semantics: any failed component zeroes the task. reward_breakdown records each component.
- ENV evaluator (the load-bearing one): builds a fresh predicted env, set_state(full_trajectory) -> hash; builds a gold env, replays the task's golden actions -> hash; reward 1.0 iff agent-DB AND user-DB hashes both match. Plus env_assertions (each an EnvAssertion boolean function run against predicted env; product of results). This grades END STATE, not the path — many paths can reach the same DB hash. The agent never authored this hash (R4/R5 style off-agent artifact).
- ACTION evaluator: every golden Action must be matched by SOME tool call in the trajectory; all-or-nothing (1.0 iff all matched). Partial read/write accuracy is tracked separately for metrics.
- COMMUNICATE evaluator: required info strings present in agent->user text.
- NL-ASSERTIONS evaluator (LLM-as-JUDGE): builds a transcript ("role: content"), asks a judge LLM to return JSON [{expectedOutcome, reasoning, metExpectation}] per assertion; reward 1.0 iff all met. Schema-constrained JSON output, reasoning required per item.
- is_successful(reward) uses an epsilon band [1-1e-6, 1+1e-6] — only a perfect reward counts as success.

3.6 pass^k RELIABILITY METRIC (metrics/agent_metrics.py)
- Run each task num_trials times independently. pass_hat_k(num_trials, success_count, k) = C(success_count, k) / C(num_trials, k) — the probability that k randomly chosen trials are ALL successes. pass^1 = avg success; pass^k decays fast with any flakiness, so it MEASURES RELIABILITY/CONSISTENCY not peak ability. Reported for every k from 1..min_trials. (cite in code: arXiv:2406.12045, the original tau-bench paper.)
- compute_metrics also aggregates: avg_reward, avg_agent_cost, read/write action accuracy, DB match/mismatch, auth succeeded/failed, termination-reason histogram, and the LLM-judge review error stats (errors by severity, sims by max severity, first-critical-source agent-vs-user, error tags by severity).
- eval_harness.py additionally reports pass@k (any of first k) vs pass^k (all of first k) and per-role token totals (agent_tokens is the paper's cost axis).

3.7 LLM-AS-JUDGE "REVIEWER" (evaluator/review_llm_judge.py) — diagnostic, NOT reward
- A separate judge LLM reads Policy + Simulation Guidelines + User Instructions + example action trajectory + NL assertions + the full conversation, and emits structured JSON errors. Each error: source (agent|user), error_tags (from a FIXED taxonomy: hallucination, incorrect_interpretation, guideline_violation, revealed_info_early, inconsistent_behavior, tool_call_schema_error, tool_call_argument_error, irrelevant_tool_call, premature_termination, missed_required_action, wrong_sequence, other), severity (agent: minor|critical; user: minor|critical_helped|critical_hindered), turn_idx, reasoning, correct_behavior.
- It explicitly grades the USER simulator too (fact-check every user claim against instructions = hallucination detection; don't blame user for agent failures) — i.e. it audits the test harness itself, separating "agent failed" from "the test was unfair". critical_helped/critical_hindered quantify when a flaky user changed the outcome. Parse failures degrade to a recorded unknown-source error, never a silent pass.

3.8 H2/H3/H4/H5 MECHANICS (the Life-Harness contribution)
- H2 (harness/base.py + per-domain rules): HarnessRule.check(db, **kwargs) raises ValueError on a bad call BEFORE execution; HarnessedToolKitMixin.use_tool runs all rules for that tool, then super().use_tool, then annotators. The ValueError is caught upstream and returned as a tool error message, giving the agent feedback to self-correct. Rules are STATELESS and DB-only (replay-safe). Retail ships ~19 rules (status guards, arg parity, hallucinated-item-id catch, same-product-variant enforcement) — see retail.py docstring index.
- H3 (harness/h3_tools.py): for write/decision tools, append a dense POLICY CONSTRAINTS block into the docstring via functools.wraps shim -> new Tool -> new openai_schema. Rationale (documented in-file): the system prompt decays in salience as context grows, but tool descriptions are re-sent at full attention EVERY API call, so the contract is always in the model's immediate window at the point of decision. Per-domain hint dicts; retail deliberately excludes READ tools (hints there caused over-scanning).
- H4 (harness/base.py HarnessAnnotator + per-domain annotators): on the SUCCESS path, annotate(db, result, **kwargs) returns a short note appended to the tool result (never blocks). Injects state-aware context at the exact moment it helps: order eligibility flags, remaining eligible orders, exchange price differences, refund amounts, remaining gift-card balance, reservation payment summaries, round-trip completeness reminders, correct-line identification. The orchestrator owns the other half of "trajectory regulation": max_steps / max_errors / timeout budgets that force termination (and auto-fail). NOTE: the count-based loop/stagnation/no-op detector described in Harness.md (Failure Type 3) is the DESIGN doctrine; in the tau2 leg H4 is realized mainly as annotators + budget termination (the AgentBench leg, not read here, likely carries the explicit A<->B-loop / repeated-query counters).
- H5 (harness/skills.py): Skill = {id,title,pattern(keywords),tip,source(failure|success),issue_type}. Libraries are MINED OFFLINE from train-split trajectory analysis (each skill cites the training task_ids that motivated it — a failure-pattern -> reusable-tip distillation). At agent build time: detect issue-type from the user's first message (keyword routing pre-filter, e.g. telecom mms/abroad/service/data with expansion edges), BM25-lite score each candidate skill's pattern vs the task description, apply per-skill PRECISION GUARDS (hand-written context predicates so a narrow skill only fires on the right wording — fights BM25 over-injection), add small score boosts, take top-k (capped low: retail<=2, airline<=1), threshold by min_score, format as a "## Task-Specific Guidance" block PREPENDED before the domain policy (highest-attention position). Skills are strategic/task-level (planning), explicitly NOT per-tool constraints (those belong in H2/H3).

3.9 THE HARNESS-EVOLUTION LOOP (README "Evolving the Harness")
- Offline, training-free self-improvement of the INTERFACE: run eval -> hand a CLI coding agent (Codex) {current harness/ source, previous run's trajectory dir + summary metrics, Harness.md design guide}. The agent diagnoses recurring DETERMINISTIC failures, finds the EARLIEST lifecycle point to catch each (before-interaction / task-conditioning / before-execution / after-execution = H5/H3/H2/H4), and directly edits the harness layer code (not just an analysis report). Constraints: no oracle/test labels, no env/eval/task edits, must check for over-triggering/regressions, must run narrowest regression checks. Re-run, feed new trajectories back. This is an outer agentic loop that frozen-model-improves by editing the harness, judged on pass^k delta.

=========================================================================
4. KEY TECHNIQUES PORTABLE TO ELAI
=========================================================================
(ELAI = Rust orchestrator, local+cloud routing, FSM plan exec, explorer/executor/auditor loop, wiring-enforcement gates, reliability measurement.)

T1. pass^k AS THE RELIABILITY GATE (metrics/agent_metrics.py::pass_hat_k).
  ELAI already wants "one measured SAFEDELTA number" and judges executor reliability. pass^k = C(succ,k)/C(trials,k) is the canonical reliability (not capability) metric: it punishes flakiness, which is exactly what a wiring-enforcement/audit posture cares about. PORT: run each plan/baby-step audit k times, report pass^k. Maps onto the auditor: an executor changeset is "reliable" only if all k independent re-runs of the auditor gate pass. Directly portable as a Rust/Python metric; no LLM needed.

T2. END-STATE DB-HASH GRADING = OFF-AGENT, UN-GAMEABLE ARTIFACT (evaluator/evaluator_env.py + environment.set_state replay).
  This is precisely ELAI's wiring-enforcement R4/R5/R6 doctrine implemented: grade the FINAL STATE the agent reached (DB hash) vs a gold reference the agent never authored, by REPLAYING only mutating actions and comparing a content hash. The replay self-consistency assert (re-execute write, assert output == recorded) is a strong "the trajectory is real and reproducible" check. PORT: for ELAI tasks that mutate real artifacts (files, SSB rows, env state), grade by hashing the resulting state against a gold run, replaying only write-tools — an objective grader the executor can't satisfy with a unit test. Maps onto auditor + production-caller gate (off-agent measurement). Non-portable part: requires a deterministic, replayable, hashable domain DB; ELAI's real codebase mutations are not as clean as a toy retail DB — would need a scoped sandbox state.

T3. POLICY-IN-TOOL-DESCRIPTION (H3, harness/h3_tools.py) — anti-context-decay contract injection.
  Insight (verified in-file + grounded against how tool schemas are re-sent every call): tool descriptions sit at full attention on EVERY API request while the system prompt decays as context grows. PORT: ELAI should put the load-bearing contract (the WIRING_CONTRACT produces/consumes/observable_effect, the "you MUST wire the new fn into seam Y", anti-stub rules) into the TOOL/function descriptions the executor model sees each turn, not only in the one-shot system prompt — keeps the gate in the model's immediate window at decision time. Directly portable to ELAI's agent tool definitions (Tauri/IPC tool catalog, executor tool schemas). Cheap, training-free.

T4. PRE-EXECUTION VALIDATION GATE THAT FEEDS THE ERROR BACK (H2, harness/base.py HarnessRule).
  A stateless rule checks the proposed action against known constraints BEFORE execution; on violation it raises an error that is returned to the model AS the tool result so the model self-corrects in-loop. This is the runtime analog of ELAI's PreToolUse hook + production-caller-check, but in-conversation and self-healing. PORT: ELAI's executor tool layer can run cheap deterministic pre-checks (path/scope gating, "this pub fn has no production caller", anti-magic-number) and return the failure as tool feedback so the executor fixes it within the same turn instead of failing audit later. Maps onto the FSM PreToolUse gate + impact-check skill. Portable.

T5. THREE-WAY LLM-JUDGE THAT AUDITS BOTH SIDES + FIXED ERROR TAXONOMY (evaluator/review_llm_judge.py).
  The reviewer grades the AGENT and the USER-SIMULATOR with a fixed error-tag taxonomy + severity, and explicitly attributes whether a failure was the agent's fault or the test's (critical_helped / critical_hindered). PORT: ELAI's auditor already renders verdicts; adopting (a) a frozen error-tag taxonomy with per-finding severity + turn/line index + correct_behavior, and (b) attribution of "executor fault vs gate/spec fault" would sharpen audit output and feed the anti-spiral/test-attributer agents. The fact-check-every-claim-against-instructions = hallucination-grounding check maps onto ELAI's grounded-by-default rule. Schema-constrained JSON with parse-failure-as-error (never silent pass) matches ELAI's fail-closed posture. Portable as auditor prompt structure (it's a vote/diagnostic, weak grounding — keep it off the critical reward path, same as tau2 keeps it out of reward).

T6. SKILL DISTILLATION FROM FAILED TRAJECTORIES + ROUTED BM25 RETRIEVAL (H5, harness/skills.py).
  Failures from past runs are mined into short reusable tips, retrieved per-task by issue-type routing + BM25 + per-skill precision guards, injected at the highest-attention prompt position, top-k kept deliberately TINY (1-2) to avoid becoming a second policy. PORT: ELAI's gotchas.md + memory-bank are already a failure ledger; this is the retrieval/injection discipline ELAI lacks — turn gotchas into routed, BM25-retrieved, precision-guarded tips injected into the executor/explorer prompt for the matching task type, capped tiny. Maps onto memory-writer (mine) + context-compiler (retrieve+inject). Caution: H5's precision guards are heavy hand-tuned per-skill predicates (skills.py is ~1900 lines of context rules) — that brittleness is a warning, not a feature; ELAI should prefer semantic retrieval over hand-coded keyword guards.

T7. THE HARNESS-EVOLUTION OUTER LOOP (README) — improve the INTERFACE not the model.
  A coding agent reads last run's failures + a design guide and edits the harness code, training-free, judged on pass^k delta, forbidden from touching the evaluator. This is ELAI's exact thesis ("smarter orchestration not bigger model" + "frozen-gate, can't-edit-your-own-evaluator"). PORT: an ELAI meta-loop where an agent reads aggregated audit failures across plans and proposes edits to the HARNESS LAYER (hooks, tool descriptions, pre-checks, skill library) — never the frozen gate — then re-measures pass^k. Maps onto auto-orchestrate + the DGM/self-improvement tracks (H-43..). The "must edit code not just write a report" and "check for over-triggering/regression" constraints are good guardrails to copy verbatim. Non-portable caveat: tau2's loop relies on a clean deterministic benchmark to measure delta; ELAI needs its own stable measured number (SAFEDELTA) first or the loop optimizes noise.

T8. PROTOCOL-VIOLATION-AS-TERMINATION + termination-gates-reward (orchestrator.py).
  Reward is 0 unless the run ended cleanly (AGENT_STOP/USER_STOP); max_steps/errors/timeout = auto-fail; INFRASTRUCTURE_ERROR excluded from metrics. The message-shape invariants (text XOR tool-call, n-calls->n-results) are validated and a violation kills the run. PORT: ELAI should distinguish "infra error" (don't count) from "agent failure" (count as fail) in its reliability stats, and enforce strict turn-shape invariants on executor output. Portable to the orchestrator FSM.

Non-portable / orthogonal: the entire voice/ subtree (audio-native real-time, TTS, transcription, full-duplex Tick orchestration) and the banking_knowledge RAG/KB leg are domain-specific to tau2 and not relevant to ELAI's coding-agent reliability goal.

=========================================================================
5. BENCHMARK NUMBERS (as stated in README/docs; grounding only)
=========================================================================
From the top-level README (Life-Harness):
- 7 deterministic agent benchmarks, 18 model backbones.
- 116 / 126 model-environment settings improved by the harness.
- +88.5% average RELATIVE improvement (paper-reported).
- Training-free (no weight updates).
Default eval settings (TauBench/Harness.md): airline/retail/telecom, agent temperature 0.0, agent max_tokens 2048, max_step 200, paper uses 3 trials for final pass^k.
tau2-bench domains used in this fork: Airline, Retail, Telecom (+ banking_knowledge, mock). No specific per-domain reward numbers are stated in the READMEs read; the per-run numbers live in data/ result JSONs (NOT read — out of scope). Default agent LLM in eval_harness.py is openai/Qwen/Qwen3-4B-Instruct-2507 via a local OpenAI-compatible endpoint; default user-sim LLM openai/deepseek-v4-flash. NOTE: arXiv:2605.22166 is a 2026 future-dated id and the relative-gain figure is paper-reported, not independently verified here.

=========================================================================
6. DEPENDENCIES / STACK
=========================================================================
- Python >=3.12,<3.14, packaged as "tau2" v1.0.0, MIT, build=hatchling, env-managed by uv.
- Core: litellm (1.80.x, the multi-provider LLM gateway — OpenAI-compatible, OpenRouter, DashScope, DeepSeek), pydantic (data models), pandas (metrics), fastapi+uvicorn (api_service simulation server), loguru, tenacity (retry), docstring-parser (tool schema from docstrings), deepdiff, typer/rich (CLI), httpx/requests, numpy.
- Optional [voice]: elevenlabs, deepgram-sdk, websockets, pyaudio, jiwer, scipy, pydub, google-genai, boto3 + aws bedrock runtime, google-cloud-aiplatform, livekit-agents + plugins. (All orthogonal to core method.)
- Knowledge/RAG leg (banking_knowledge): BM25 + embedding indexers, bge/qwen/pointwise-LLM rerankers, openai/openrouter embedders (deps likely under the knowledge extra).
- LLM access is provider-agnostic via litellm; agent + user + judge are all just model identifiers + api_base/api_key, set by env/.env or CLI flags. Self-improvement loop uses Codex CLI externally (not a python dep).
