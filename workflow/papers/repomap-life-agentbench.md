repomap-life-agentbench

Repo map of the AgentBench leg of the Life-Harness bundle. METHOD only (protocol, runner, agent loop, harness runtime layers, stagnation counters). Task/data contents (AgentBench/data/**, *.jsonl corpora) deliberately not read. Sibling agent maps the TauBench leg.

Source: C:\Users\user\Downloads\new\Life-Harness-main\AgentBench
Paper: Xu, Wen, Li, "Adapting the Interface, Not the Model: Runtime Harness Adaptation for Deterministic LLM Agents", arXiv:2605.22166 (2026).


1. WHAT IT IS

Life-Harness is a training-free method that improves a FROZEN LLM agent by adapting the runtime harness (the interface between model and environment) instead of retraining the model or modifying the benchmark. The core claim: most agent failures in DETERMINISTIC environments are not reasoning-capability failures, they are mechanically-identifiable execution-level failures (malformed actions, wrong tool conventions, repeated no-ops, A<->B loops, premature submit, budget exhaustion). Because the environment is deterministic, trigger conditions can be binary/exact rather than probabilistic, so a small set of rule-based runtime "layers" can detect and intervene reliably with low false-trigger rate.

The bundle has two benchmark families kept in separate folders with separate deps:
- AgentBench/  (THIS map) — Docker-based AgentBench-style tasks: ALFWorld, DBBench, OS, WebShop. Python 3.9, Docker Compose (Redis + controller + per-task worker).
- TauBench/    — uv-based tau-bench-style tasks: Airline, Retail, Telecom. (sibling agent.)

AgentBench's role in the bundle: it is the harder, multi-environment, distributed-worker leg. It supplies four AgentBench environments that each get their own bespoke harness runtime (the H2-H5 layers re-implemented per environment with environment-specific triggers). It is the leg where the "deterministic signal -> exact trigger -> targeted intervention" thesis is exercised against shell, SQL, embodied-text, and web-shopping action spaces.

The harness is built by an EVOLUTION loop (README "Evolving the Harness"): run eval -> feed the prior run's trajectories + current harness code + a design guide (Harenss.md) to a CLI coding agent (e.g. Codex CLI) -> the agent edits the harness code directly (not a report) to kill recurring deterministic failures at the earliest lifecycle point -> rerun. The shipped harness/*.py files are the crystallized output of that loop (visible as dense inline comments referencing "v2/v5 analysis", removed heuristics, false-match post-mortems).

Headline result: 7 benchmarks x 18 model backbones, 116/126 model-environment settings improved, 88.5% avg relative gain, training-free.


2. STRUCTURE MAP (code modules, data ignored)

Entry / orchestration
- src/assigner.py (520 ln) — the MULTI-ENV RUNNER. Max-flow scheduler assigning (agent x task x sample) across concurrent Docker workers; resume/retry/aggregate. `python -m src.assigner --config configs/assignments/<env>.yaml`.
- src/configs.py (134 ln) — ConfigLoader: YAML import/merge, recursive "module + parameters -> .create()" instantiation indirection (ModuleConfig).
- run.sh, count_datasets.py, scripts/validate_lite_configs.py — helper scripts.

Client side (agent + task driver)
- src/client/task.py (704 ln) — TaskClient: the AGENT<->CONTROLLER interaction loop over HTTP (/start_sample, /interact, /cancel, /calculate_overall); dual protocol (legacy session-id-in-body vs new OpenAI-messages + session-id-in-header); per-episode token-usage accounting; local mirror of server scoring (_local_calculate_overall) + trial metrics (avg@k / pass@k / pass^j).
- src/client/agent.py (9 ln) — AgentClient ABC (inference(history)->str).
- src/client/agents/http_agent.py (257 ln) — HTTPAgent: OpenAI-compatible POST client; inference() (text protocol) + inference_openai(messages, tools)->(msg, usage) (tool-call protocol); Prompter family (role/content vs prompt-string formatting); 3x retry; context-limit detection via keyword AND/OR rules.
- src/client/agents/{claude_agent,fastchat_client,test_agent}.py — alternate backends.
- src/client/agent_test.py — config smoke-test (`python -m src.client.agent_test`).

Server side — HARNESS RUNTIME LAYERS (the method core; per environment)
- src/server/harness/__init__.py (42 ln) — exports each env's HarnessConfig, HarnessRuntime, and the H3 patch_* tool-description functions + OS rescue_tool_call_from_text.
- src/server/harness/os_interaction.py (2514 ln) — OS (shell) harness: OSHarnessConfig (flags+thresholds), OSShellState (counters), OSHarnessRuntime (init_task/cold_start_skill_hints/pre_validate_action/update_state_after_bash/post_step_monitor/step_guidance/normalize_answer), BM25 skill retrieval, rescue_tool_call_from_text (JSON repair), _is_dangerous_bash, parse_task_context.
- src/server/harness/dbbench.py (2736 ln) — DBBench (SQL) harness: same shape; loop_streak/error_streak/empty_streak counters; patch_dbbench_system_prompt + patch_dbbench_tool_descriptions.
- src/server/harness/alfworld.py (1580 ln) — ALFWorld (embodied text) harness: world-model + subgoal state machine + "nothing happens" loop counter; patch_take_action_tool_description.
- src/server/harness/webshop.py (2364 ln) — WebShop harness: search-loop / duplicate-query / product-stall counters; repeat-click block; force-Buy-Now budget branch.
- Harenss.md (50 ln) — the DESIGN GUIDE (paper method 3.1): three failure types -> H2/H3/H4 mapping, layer trigger-timing table, validation properties. Fed to the evolution agent.

Server side — TASK ENVIRONMENTS (where harness is wired into the loop)
- src/server/tasks/os_interaction/{task.py, environment.py} — OS task; task.py is the per-round interaction driver that calls the harness lifecycle methods (the wiring).
- src/server/tasks/dbbench/{task.py, environment.py, interaction.py, result_processor.py} — DB task (MySQL container).
- src/server/tasks/alfworld/{task.py, environment.py, utils.py} — ALFWorld.
- src/server/tasks/webshop/task.py — WebShop.
- src/server/tasks/knowledgegraph/** — KG task (present but not in the paper's 4).

Typings / utils
- src/typings/{config,request,output,status,general,exception}.py — pydantic models: AssignmentConfig, ModuleConfig, StartSampleRequest, InteractRequest, AgentOutput, TaskOutput, TaskClientOutput, SampleStatus, WorkerStatus, ChatHistoryItem, exceptions (AgentContextLimitException, AgentBenchException).
- src/utils/{max_flow,rules,others}.py — MaxFlow/Graph (scheduler), AndRule/OrRule/ContainRule (keyword matchers), ColorMessage.
- src/analysis.py — post-hoc trajectory analysis.

Config
- configs/agents/*.yaml — model endpoint, model name, sampling, max tokens.
- configs/tasks/*.yaml — task split, max steps/rounds, and the HARNESS SWITCHES (enabled/h2/h3/h4/h5, h5_top_k, thresholds).
- configs/assignments/*.yaml — selected agent+task profile, concurrency, trials, output dir.
- extra/docker-compose.yml + *-entrypoint.sh — Redis, controller, per-task worker services.


3. ARCHITECTURE + METHOD (the bulk)

3.1 Distributed server-client topology
Three tiers, all over HTTP, coordinated by Redis:
  Assigner/TaskClient (client process)  <-->  Controller (:5000/api)  <-->  Task Workers (Docker containers, one env each)
The CONTROLLER is a separate process (shipped as a Docker image, not in this repo's src/) exposing /get_indices, /list_workers, /start_sample, /interact, /cancel, /calculate_overall. A WORKER is a Docker container running one environment (e.g. os-std, dbbench-std with a MySQL:8 sidecar, alfworld-std, webshop-std). The worker owns the deterministic environment AND the harness runtime; the controller multiplexes sessions and reports worker capacity. This separation is why the harness lives server-side (in the worker) and the agent stays a thin HTTP client: the harness can inspect real environment observations (bash stdout, SQL rows, web page state) the agent's text alone does not carry.

3.2 The agent-task interaction protocol (src/client/task.py TaskClient.run_sample)
Per sample:
1. POST /start_sample {name, index} -> session_id (from body OR response header; dual-protocol detection: use_header_sid = "session_id" not in start_payload). Start payload also carries the seed messages and (new protocol) the tools array.
2. Loop while SampleStatus == RUNNING:
   - New (agentrl/OpenAI) protocol + agent.inference_openai: send accumulated api_messages + api_tools to the model endpoint, get assistant message (may include tool_calls), accumulate per-turn token usage, POST /interact {messages:[assistant_msg]} with session_id header.
   - Legacy protocol: normalize history to user/agent roles, agent.inference(history)->str, POST /interact {InteractRequest(session_id, AgentOutput(content))}.
   - Server returns updated status + new messages (+ possibly refreshed tools). Parse via _normalize_output (handles 3 payload shapes: {output:{}}, direct TaskOutput, OpenAI-style {status,messages,reward,metrics}).
3. On any agent exception / non-200: POST /cancel and return a typed TaskError (START_FAILED/INTERACT_FAILED/AGENT_FAILED/NETWORK_ERROR/NOT_AVAILABLE). AgentContextLimitException is swallowed to an empty assistant turn (lets the episode terminate cleanly).
4. On finish: package TaskOutput with result{reward, metrics, openai_messages (full trace), token_usage}.
Key property: the agent is STATELESS across the run; all state (the harness counters, the environment) lives in the worker keyed by session_id. The client just shuttles messages.

3.3 The multi-environment runner (src/assigner.py)
This is the standout systems piece. The Assigner schedules (agent, task, sample) triples across heterogeneous, capacity-limited Docker workers using MAX-FLOW:
- Build a bipartite-ish flow graph SRC -> agent_i (cap = free agent slots) -> task_j (cap = free worker slots from controller /list_workers capacity-current) with middle edges agent_i->task_j capacity = remaining samples of that (agent,task). Run MaxFlow(src=SRC,dst=DST); each saturated agent->task edge yields that many (agent,task,index) tuples to dispatch as threads. This maximizes concurrent utilization across all environments simultaneously without starving any one (worker_generator, lines 172-251).
- RESUME: on init, walk output/{agent}/{task}/runs.jsonl, parse finished samples, remove them from remaining_tasks; skip a (agent,task) entirely if overall.json already exists (lines 80-124).
- RETRY: finish_callback re-inserts the index on NOT_AVAILABLE (worker not ready, e.g. WebShop slow boot) and ALWAYS retries transient INTERACT_FAILED/NETWORK_ERROR once regardless of --auto-retry; terminal failures (with --no-auto-retry) are recorded as TASK_ERROR so overall.json still aggregates mixed pass/fail.
- TRIALS: trials>1 expands each index into [base_idx, trial]; the trial dimension is stripped before talking to the controller and re-attached for pass@k/pass^j computation (_compute_trial_metrics in task.py).
- Aggregation: when all samples of a (agent,task) complete, a background thread calls TaskClient.calculate_overall -> writes overall.json (local mirror + remote worker aggregate).

3.4 The four HARNESS RUNTIME LAYERS (method core; Harenss.md + harness/*.py)
The paper frames three dominant deterministic failure modes and one general layer. Each layer fires at a distinct lifecycle point:

  Layer  Trigger timing                              Target failure              Core function
  H2     after agent output, BEFORE tool execution   action-interface errors     format repair, validity check, FORCE action
  H3     at episode init, ONCE                        tool-usage mismatch         augment tool.description with env contract
  H4     AFTER tool execution, before next turn       self-reinforcing trajectory error recovery, STAGNATION detect, budget mgmt
  H5     at episode start                             general (all 3)             skill retrieval + injection, per-step guidance

Master switch: enabled (top-level or nested harness.enabled). If false, no layer applies even if its flag is true. Each layer has its own boolean (h2_enabled..h5_enabled) for ablation. Thresholds are named constants on the per-env *HarnessConfig dataclass.

H2 Action Gate (OS: OSHarnessRuntime.pre_validate_action). Two jobs:
  (a) RESCUE (repair): rescue_tool_call_from_text(content) extracts a tool call from malformed assistant text via balanced-JSON scan + regex fallback + _fix_json_escapes. Turns "almost a valid call" into an executable action without inferring intent. The OS task wires this in task.py (line 744): if no tool_call but text present, try rescue; on hit, synthesize a tool call and note_rescue_hit().
  (b) GATE (block/force): _is_dangerous_bash blocks destructive commands. Duplicate-action gate: if the last N (h2_repeat_bash_block_after=2) bash commands are identical AND a PLAUSIBLE candidate answer is already captured AND answer_shape is integer AND not answered -> block the re-run and FORCE an answer_action (force_next_action). normalize_answer strips trailing unit tokens so exact-match scoring passes.
  The force mechanism: harness writes runtime.force_next_action; on the next round task.py (line 715) consumes it before reading model output and injects it as the executed action.

H3 Environment Contract (OS: patch_os_tool_descriptions; DB: patch_dbbench_tool_descriptions + patch_dbbench_system_prompt; ALFWorld: patch_take_action_tool_description; WebShop: patch_webshop_tool_descriptions). At episode init (os task.py line 186, only if h3_enabled), rewrites the tools array's descriptions to embed environment-specific constraints and correct calling conventions INTO the text the model reads when deciding to call a tool. Fixes "wrong tool / wrong arg format / wrong convention" mismatches that the agent will not self-correct because they stem from pretrain-vs-this-env semantics. One-shot, no per-step cost.

H4 Trajectory Regulation / Post-Execution Monitor (OS: post_step_monitor + step_guidance). Runs after each environment execution; this is where ALL stagnation/loop detection and budget management live. It is a single insertion point (os task.py line 960) returning {audit_reason, recovery_prompt, force_action}:
  - Reads the per-env STATE struct of counters (OSShellState etc.), updated by update_state_after_bash from the deterministic observation (truncation marker, error regex, empty output, numeric-candidate extraction).
  - Branches on deterministic signatures: truncated output (-> "refine, do NOT re-run"); error signatures (command-not-found / no-such-file / permission-denied -> targeted corrective hint, and explicitly tells the agent NOT to submit a number that came from an error exit); empty-output streak; STALL (same action+output over h4_stall_window turns); and a BUDGET sub-branch keyed on remaining_rounds (round_limit - current): remaining <= h4_budget_warn_threshold -> soft urgency hint; remaining <= h4_budget_force_threshold AND a plausible candidate ready -> FORCE submit.
  - recovery_prompt is injected as a user-role harness message; force_action is written to force_next_action for H2 to consume next round. h4_fired_last_round lets the next round's "post-ls nudge" chain off the previous intervention.
  STAGNATION/LOOP COUNTERS PER ENV (the directly ELAI-relevant piece):
    OS (os_interaction.py): bash_history (exact dup over h2_repeat_bash_block_after=2), empty_output_streak (h4_empty_output_threshold=2), text_only_streak (h2_text_only_streak_force=2 -> hard force), h4_stall_window=3 (same bash+output), bash_loop_count, candidate_implausible (refuse force on garbage like awk overflow / nan / int64 sentinels).
    DBBench (dbbench.py): error_streak, empty_streak, loop_streak, text_only_streak; h2_repeat_sql_block_after=2 -> auto-commit candidate; h4_stall_window=3 -> loop alert; h4_empty_threshold=2; budget warn=3/force=2.
    ALFWorld (alfworld.py): world-model + subgoal state machine; h4_stall_window=4; "nothing happens" loop: h4_nothing_happens_window=4 / threshold=3 (counts the env's "Nothing happens" observation); h4_min_rounds_before_stall=8 (no stall intervention early); h4_post_put_grace=4; budget warn=7/force=4 (12 for multi-step pick_two tasks); h2_empty_turn_threshold=2.
    WebShop (webshop.py): h2_repeat_click_block_after=2 (block repeated identical click); h4_search_loop_threshold=4 (back_to_search count); h4_duplicate_search_threshold=2 (same query repeated); h4_product_stall_turns=3 (_stall_asin/_stall_turns on same product page); asins_visited set; budget warn=5/force=3 (forced Buy Now only when "buy now" already in clickables); price_tolerance=0.05.
  The unifying design (Harenss.md 3.1, Failure Type 3): loop/stall triggers are COUNTS over the action/observation sequence, content-agnostic ("repeated 3x" / "A<->B" needs no understanding of the action's meaning), which is what makes them generalize across new tasks in the same environment.

H5 Procedural Skill Layer (OS: cold_start_skill_hints + retrieve_os_skills, BM25). At episode start, retrieves task-relevant procedural skills from a curated per-env skill library (OS_SKILLS: id/task_types/keywords/text) and injects the top-k as guidance. Retrieval = hand-rolled BM25 (k1=1.5, b=0.75) with a domain-tuned stopword list and a SCORE THRESHOLD (h5_score_threshold=7.5) below which nothing is injected (prevents noisy low-relevance hints). top_k is deliberately tiny (OS=1, DB=2) because over-injection crowds the real task instruction. Sparse high-confidence lexical OVERRIDES force a specific skill when the task text matches a known failure pattern (e.g. "unique words" -> count_unique_words skill). Also supplies per-step guidance (step_guidance) during H4. Skills are knowledge distilled from recurring SUCCESSFUL recoveries in prior trajectories (closes the evolution loop).

3.5 Scoring / evaluation
- Server worker computes the env-native score; client mirrors it in _local_calculate_overall per env: ALFWORLD/OS pass = result==1 or reward>=1.0; DBBench acc = reward>=1.0; WebShop = completed_rate + success_at_1 (reward>=1.0) + average_reward.
- Token accounting: per-episode prompt/completion/total tokens accumulated in run_sample, aggregated to avg-per-episode (lets you measure the harness's token overhead vs gain).
- trials>1 -> avg@k, pass@k, and pass^j (all j of 2..trials trials pass) — a determinism/robustness metric.
- AUDIT TRAIL: every harness intervention is recorded into a harness_trace dict ({h2:[],h3:[],h4:[],h5:[]}) and persisted into the transcript as a tagged "[HARNESS_TRACE_V1]" user message (os task.py _persist_harness_trace), so each run's overall.json/trajectory shows exactly which layer fired, when, and why. This is the substrate the evolution loop reads back.

3.6 The evolution loop (how the harness is built; README + Harenss.md)
Not run at eval time. Offline iterative code-edit: feed (current harness src, prior run trajectory dir, Harenss.md design guide) to a CLI coding agent; it finds recurring deterministic failures, assigns each to the EARLIEST lifecycle layer that can catch it (before interaction=H5/H3, before exec=H2, after exec=H4), edits the harness code, runs narrowest regression checks, and self-audits for over-triggering (blocking valid actions / misleading hints / regressions on previously-passing trajectories). The shipped files' "v2/v5 removed this heuristic because..." comments are this loop's fossil record.


4. KEY TECHNIQUES PORTABLE TO ELAI

T1. Content-agnostic stagnation counters as a deterministic anti-spiral signal.
  File: src/server/harness/*.py STATE structs + post_step_monitor; Harenss.md Failure Type 3.
  ELAI map: this is exactly the anti-spiral skill's job (.claude/skills/anti-spiral, the Spinning/Oscillation/NoDrift/DiminishingReturns patterns). Life-Harness gives crisp, named, threshold-driven definitions ELAI can adopt verbatim: exact-action-repeat over N (Spinning == h2_repeat_*_after / h4_stall_window), A<->B oscillation (== Oscillation, WebShop back_to_search/_stall_asin), empty/error streaks (== NoDrift), and a budget-remaining force (== escalate). ELAI currently triggers anti-spiral on "same file edited 3x / same hypothesis fails 3x / auditor BLOCK twice" — those are the SAME class of count-over-the-trace triggers. Porting value: replace any soft/heuristic stagnation judgement in anti-spiral with these exact deterministic counts keyed off the executor's tool-call history and auditor verdict history. HIGH portability — ELAI's loop is deterministic enough (FSM states, discrete tool calls) for binary triggers.

T2. Layered intervention by EARLIEST lifecycle point (the H2/H3/H4/H5 separation).
  File: Harenss.md 3.1 layer table; per-env runtime methods.
  ELAI map: maps onto ELAI's multi-agent seams. H2 (pre-execution action gate: repair-or-block) == a pre-Edit/pre-Bash gate in the FSM (analogous to the existing PreToolUse hook + production_caller_check self-verify, but for malformed/duplicate actions). H3 (embed env contract into tool descriptions, once) == ELAI injecting wiring-contract / spec constraints into the executor's tool/prompt at sub-phase start (context-compiler output already does some of this; H3 says do it AT the tool-description layer). H4 (post-execution monitor) == the auditor + anti-spiral after each change. H5 (skill retrieval) == ELAI's RAG/gotchas.md retrieval. The portable insight is the DISCIPLINE: assign each recurring failure to the earliest layer that can catch it, and keep layers independent + ablatable. MEDIUM-HIGH portability.

T3. Force-action + candidate-promotion with an implausibility guard.
  File: os_interaction.py pre_validate_action (force_next_action) + is_plausible_numeric_candidate + candidate_implausible.
  ELAI map: when ELAI's executor spins (same edit/test loop) but a plausible solution is already in hand, the harness can FORCE convergence (submit/commit) under a budget — but ONLY if the candidate passes a plausibility guard (refuse to force-submit garbage). This directly informs ELAI's "budget overflow forces convergence or escalation" GRC rule and the anti-spiral persona switch: don't just escalate blindly; if a verified-good candidate exists, force it; if the candidate is implausible (e.g. tests still red, wire-dark fn), escalate instead. The implausibility guard is the safety that keeps a force-convergence from shipping broken work. MEDIUM portability — needs ELAI to define "plausible candidate" per task (e.g. auditor PASS + production_caller_check WIRED).

T4. Persisted intervention audit-trail feeding an offline evolution loop.
  File: harness_trace + _persist_harness_trace ([HARNESS_TRACE_V1]); README "Evolving the Harness".
  ELAI map: ELAI already emits SSB rows (off-agent artifacts) and decision docs. The portable pattern is the CLOSED LOOP: tag every harness/auditor intervention into the trajectory, then periodically feed (harness code + failure trajectories + design guide) to a coding agent that EDITS the gates/rules to kill recurring failures. This is a concrete, training-free recipe for ELAI to self-improve its anti-spiral/auditor rule set from its own gotchas.md + SSB history — adapt-the-harness rather than the model, which aligns with ELAI's "smarter orchestration not bigger model" direction (project_efficiency_trust_engineering). HIGH portability and strongly aligned with ELAI doctrine, but treat as method, not as a thing to auto-run unsupervised.

T5. Max-flow scheduler for heterogeneous capacity-limited workers.
  File: src/assigner.py worker_generator + src/utils/max_flow.py.
  ELAI map: ELAI caps parallel agents at 3 with a simple ladder. If ELAI ever fans out across multiple capacity-limited backends (local sidecar slots + cloud rate limits + RunPod), the max-flow assignment (SRC->agent->task->DST with live capacities) is a cleaner allocator than a fixed cap. LOW-MEDIUM portability — likely overkill for ELAI's current 3-agent cap; flag as a future-scale technique only.

T6. Schema-tolerant dual-protocol client + per-episode token accounting.
  File: src/client/task.py (_normalize_output 3-shape parse, header-vs-body session id, _ep_usage).
  ELAI map: ELAI's runtime_client/provider abstraction already speaks Anthropic protocol; the portable bit is the per-episode token ledger accumulated across turns (mirrors ELAI's J-28/J-29 token ledger work) and the robust multi-shape response normalization. Mostly confirmatory of existing ELAI work; low novel value.

NON-PORTABLE / caveats:
- The harness layers are HAND-WRITTEN PER ENVIRONMENT (2.5K-line files of env-specific regexes, bash/SQL/webshop heuristics, curated skill libraries). The METHOD (layers, counters, force, evolution) ports; the specific rule bodies do not — ELAI would author its own per-domain trigger set.
- The whole thesis is gated on DETERMINISM ("the more precise the environment signals, the more reliable the triggers"; Harenss.md 3.1 warns it produces many false triggers in open-domain/noisy settings). ELAI's coding environment is mostly deterministic (compile/test/wiring-check are exact), so the thesis holds for ELAI's gate-able signals — but NL-judgement steps (design taste, "is this maintainable") are the noisy case where binary triggers would over-fire. Apply T1/T3 only to ELAI's deterministic seams (build/test/wiring/dup-action), not to its taste/quality judgements.


5. BENCHMARK NUMBERS (stated; grounding only)

- Life-Harness overall (top README): 7 deterministic benchmarks, 18 model backbones, 116/126 model-environment settings improved, 88.5% average relative gain, training-free. Source: Life-Harness/README.md badges + Results section.
- AgentBench default eval settings (AgentBench/README.md table): temperature 0.0 for all four; agent max_tokens 4096; max step/rounds: ALFWorld 50, DBBench 15, OS 8, WebShop 20.
- Reference agent profile used in harness runs: qwen3-4b-instruct (openai/Qwen/Qwen3-4B-Instruct), temperature 0.0, max_tokens 4096, local OpenAI-compatible endpoint (Bearer EMPTY).
- No per-environment AgentBench accuracy tables are stated in the code/READMEs read (they live in the paper, arXiv:2605.22166, and in data/ result corpora which were intentionally not read).


6. DEPENDENCIES / STACK

- Python 3.9 (conda env agent-bench), requirements.txt. Notable: requests (HTTP client), pydantic v1 (.parse_obj/.dict()/.copy(deep=True) idioms in typings), tqdm (+ tqdm.contrib DummyTqdmFile), pyyaml, urllib3. Hand-rolled BM25 (math only, no rank-bm25 dep). Hand-rolled max-flow (src/utils/max_flow.py).
- Docker + Docker Compose mandatory for workers: Redis (coordination), a controller image (:5000/api, not in repo src), per-task worker images. OS env builds local-os/{default,packages,ubuntu} from data/os_interaction/res/dockerfiles; DBBench needs mysql:8 sidecar; ubuntu base; WebShop worker has a long warm-up.
- Agent backend: any OpenAI-compatible chat/completions endpoint (self-hosted vLLM/fastchat, or commercial). Two protocols supported: legacy text + new OpenAI-messages-with-tools.
- AgentBench/ and TauBench/ are intentionally separate subprojects with separate dep stacks (Docker+pip vs uv).
