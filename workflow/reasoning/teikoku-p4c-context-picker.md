TEIKOKU P4-C CONTEXT PACKAGE - per-session model-pool picker + 5 toggles + always-grill

task_id P4-C-CC2 | session s-1ed32eaf (controller) | scope READ-ONLY analysis
sources: .liberty/liberty/{catalog.py, chat/limits.py, chat/commands.py, chat/screen.py, chat/session_factory.py}; teikoku distro ~/teikoku/conductor/{worker_providers.py, worker_adapter.py, free_api_worker.py, codex_provider.py, conductor_config.py, control_plane.py, conductor_db.py, headless_runner.py, project_resolver.py}; .claude/skills/grill/SKILL.md; TEIKOKU-P4-REVISION-2026-06-20.md; handoff head.

PINNED TERMS (overloaded - fix before the plan):
- "pool" = a SELECTABLE BACKEND CLASS in the picker (claude / devin-free / free-api group). NOT the conductor "pool" (a RATE-LIMIT bucket on WorkerSlotConfig.pool keyed to workers.toml). The picker's pool maps to the worker's HARNESS, the picker's model maps to WorkerSlotConfig.model; the rate pool is derived downstream.
- "session" = an interactive CHAT session (one chat = N conductor sessions per D-10), distinct from an FSM session and a conductor task/run.
- "model pool" toggle (toggle #2) = which SOURCE+GROUP the picker offers; "model choices" toggle (#3) = the concrete model id within it.

================================================================================
1. LIBERTY MODEL-POOL PICKER - the exact shape to reuse
================================================================================

DATA SHAPE (the catalog is the source of truth; the picker is a pure formatter over it):
- catalog.FreeModel (.liberty/liberty/catalog.py:47-81) frozen dataclass, the per-deployment descriptor TEIKOKU reuses verbatim:
  id (litellm routing string), provider, group, context_window:int, max_output, rpm, tpm:Optional, rpd:Optional, tpd:Optional, is_free, cost_* (0.0 on free), notes.
- catalog.FREE_MODELS:list[FreeModel] (catalog.py:100-352) is the literal roster: groups fast-small / big / gemini-flash / mistral / openrouter-free / uncensored. DENYLISTED_MODEL_IDS (catalog.py:36-40) prunes dead ids.
- catalog helpers TEIKOKU consumes: groups() (catalog.py:468 - distinct group names, insertion-stable), model_ids() (catalog.py:479 - every FreeModel.id), get(id) (catalog.py:484), models_by_group() (catalog.py:359).

THE PICKER FORMATTER (.liberty/liberty/chat/limits.py - reuse this whole module shape):
- BackendEntry (limits.py:77-95) frozen dataclass = ONE selectable picker row: kind / value / label / category / enabled / note.
  kind tags: BACKEND_AGENT="agent" (a router group OR a concrete model id), BACKEND_CODEX="codex", BACKEND_GPT_HARNESS="gptharness" (limits.py:40-42). category sorts entries under section headers "group"|"model"|"codex"|"gptharness".
- backend_entries(groups, model_ids, *, current="") (limits.py:128-196) builds the ordered list: every group first (with a "up to <N> ctx" hint from _group_context_label which reads max(context_window) across the group, limits.py:98-125), then every concrete model id, then the always-present-but-conditionally-disabled external lanes (codex / gptharness). The active selection is flagged in-label " (active)".
- the external lanes are gated by opt-in env flags, NOT liveness probes: codex_backend_available() / gpt_harness_backend_available() read LIBERTY_CODEX_ENABLE / LIBERTY_GPT_HARNESS_ENABLE (limits.py:29-69). enabled=False renders the row greyed + non-selectable with a "set X=1" note. This is the EXACT pattern TEIKOKU needs for "feature visible but off".

THE SELECTORS (.liberty/liberty/chat/commands.py): pure parse, no UI dep. SLASH_COMMANDS (commands.py:17-20) includes model / codex / gptharness. /model [target] opens the full picker with no arg, or switches by group-name OR model-id with an arg; /codex /gptharness jump straight to those lanes (commands.py:80-110 HELP_TEXT). SLASH_MENU_ITEMS (commands.py:26-38) drives the "/"-typed dropdown.

min_context_window_for_group (catalog.py:391-431) - NEEDED VERBATIM FOR P4-D MIGRATION:
- returns the MINIMUM context window the router could serve a group from, across the group's own models AND its cross-group fallback chain (_FALLBACK_REACHABILITY catalog.py:382-388, mirrors router._FALLBACK_CHAIN). A group with no registered models contributes SAFE_SMALL_CONTEXT_TOKENS=4_096 (catalog.py:375); an unknown group clamps to 4_096. This is the "use MIN window across reachable fallback models" rule the revision doc P4-D demands (revision :128-130) - it already exists, do not re-derive it.
- min_context_window_for_model_or_group(name) (catalog.py:434-465) takes EITHER a group name OR a concrete model id and resolves to the per-call window. P4-D's per-model migrate-at threshold should call THIS off response.model.

WHAT MAPS CLEANLY vs WHAT TEIKOKU MUST ADD:
- CLEAN REUSE (LIBERTY source = the free-api pool): FreeModel + FREE_MODELS + groups()/model_ids() + BackendEntry + backend_entries() + min_context_window_for_group. These are the LIBERTY {cerebras/groq/gemini/mistral/openrouter/pollinations} half of the two-source picker, 1:1.
- TEIKOKU MUST ADD (CONTROLLER source - NOT in liberty, confirmed revision :47-50): the {claude, devin-free: kimi-k2-7/glm-5-2/swe-1-6} entries are CONTROLLER-LEVEL backends, not catalog FreeModels. They have no litellm id and no per-provider rpm/tpm row. TEIKOKU adds a SECOND entry source: either (a) hand-authored BackendEntry rows for claude + the 3 devin models (kind="agent"-equivalent but tagged with a TEIKOKU backend-kind that maps to a worker HARNESS, see section 2), or (b) a parallel mini-catalog of controller-backends carrying just {label, harness, model, context_window, migrate_at} so P4-D's per-model table (revision :131-141) has a window source for claude=200k(owner-override)/kimi=260k/glm+swe=150k. The picker UNION = controller entries ++ liberty backend_entries(). Liberty's codex/gptharness external-lane pattern is the precedent for adding non-catalog backends to the same picker list.
- TEIKOKU MUST RE-TARGET the backend KIND tags: liberty's BACKEND_AGENT means "a LibertyAgent run_one_turn loop in-process"; TEIKOKU's chat spawns a base-cc CLI (section 2), so the picker's selection must resolve to a worker HARNESS + slot.model, not an in-process agent.set_model(). The BackendEntry.value carries the resolved target; what that value MEANS differs (liberty: router group/id; TEIKOKU: harness+model).

================================================================================
2. HOW THE PICKER BINDS TO THE P4-B WORKER PLANE
================================================================================

The worker plane resolves a slot to (base_url, auth, model) PURELY off two slot fields: WorkerSlotConfig.harness + WorkerSlotConfig.proxy_base_url + WorkerSlotConfig.model. The picker's job is to SET those three on the session/slot.

THE BINDING CHAIN (verified, conductor distro):
- WorkerSlotConfig (conductor_config.py:25-44): slot_id, harness:str, model:str, model_tier, account_id, max_concurrency, codex_home, wsl_distro, proxy_base_url:str|None (P4-B D1/D2, the bridge :8810 for devin-free / litellm :4000 for free-api, None->backend default), pool:str|None (rate bucket). Loaded from workers.toml (conductor_config.py:187-194: harness/model/proxy_base_url keys).
- worker_adapter._build_env (the resolver, worker_adapter.py:496-525) is a HARNESS SWITCH that produces the final env per backend:
  * harness=="codex"  -> CODEX_HOME = slot.codex_home (codex_provider, OpenAI Responses lane). (worker_adapter.py:501-505)
  * is_claude_harness(slot.harness) ("claude"/"claude-oauth") -> claude_env() POPS ANTHROPIC_BASE_URL/AUTH_TOKEN/API_KEY = plan-OAuth, no override. (worker_adapter.py:507-511, worker_providers.py:105-112)
  * is_devin_free_harness(slot.harness) ("devin-free"/"devin-bridge") -> devin_free_env(env, bridge_url=slot.proxy_base_url): sets ANTHROPIC_BASE_URL=bridge (proxy_base_url or bridge_base_url() which resolves WIN_HOST:8810 off the WSL default route, worker_providers.py:70-132) + a dummy bearer DEVIN_FREE_DUMMY_TOKEN. (worker_adapter.py:512-516)
  * is_free_api_harness(slot.harness) -> free_api_env(env, proxy_base_url=slot.proxy_base_url, account_id): sets ANTHROPIC_BASE_URL=litellm :4000 + master_key bearer. (worker_adapter.py:517-521, free_api_worker.py:128-148)
- the COMMAND is built off harness too (worker_adapter.py:438-443): codex -> codex_provider.build_codex_command(slot,...); ALL base-cc backends -> free_api_worker.build_free_api_command(slot.model, taskspec_file) (free_api_worker.py:102-126) where slot.model is the --model alias (a free-proxy alias for free-api; for devin-free the bridge picks the devin model itself via DEVIN_PROXY_MODEL on the PROXY launch env, NOT --model).

THE EXACT FIELD THE PICKER SETS so dispatch uses the chosen backend:
  picker selection -> { slot.harness, slot.proxy_base_url, slot.model, slot.account_id (codex) }.
  - pick "claude"      -> harness="claude", proxy_base_url=None, model=<the claude model id>.
  - pick "devin-free kimi-k2-7" -> harness="devin-free", proxy_base_url=bridge :8810 (or None->resolved), model=irrelevant-to-base-cc (the devin model is selected on the PROXY side; the picker's chosen devin model id threads to DEVIN_PROXY_MODEL on the bridge launch env, worker_providers.py:124-127 note - NOT a base-cc --model). THIS IS A SEAM TO CONFIRM: how the picked devin model (kimi/glm/swe) reaches the proxy's DEVIN_PROXY_MODEL per-session.
  - pick a liberty free-api group/model -> harness=<a free-api harness>, proxy_base_url=litellm :4000, model=<the free-proxy alias>.
  - pick codex (paid) -> harness="codex", account_id=account-a/b, codex_home resolved.
  So the picker writes a (harness, model, proxy_base_url, account_id) tuple onto the session; conductor builds a WorkerSlotConfig from it; worker_adapter._build_env + _build_command do the rest UNCHANGED. No worker-plane edits needed - the picker is purely a slot-config producer.

PRODUCTION_HOOK for any new picker->slot resolver fn: the seam is the construction of WorkerSlotConfig before dispatch (the workers.toml loader at conductor_config.py:180-194 / the slot rows worker_adapter.py:610-635 read). A per-session picker overrides those fields for that session's slot rather than reading them from workers.toml. Name the exact override fn site at executor dispatch (a new "session_slot_override" applied in the chat/headless spawn path) so it is not wire-dark.

================================================================================
3. THE 5 TOGGLES - where they live, persist, and are read
================================================================================

PERSISTENCE SEAM (pack-general, no ELAI hardcode): the conductor already has a general JSON key/value control store - control_state table (conductor_db.py:65, schema is key TEXT PK + value_json + updated_at), driven by ControlManager._set_value/_read_values (control_plane.py:185-200/169-184) with an upsert. CONTROL_KEYS (control_plane.py:14-21) is currently {paused, draining, emergency_stop, dry_run, run_id, active_wave} - GLOBAL flags. The toggles are PER-SESSION, so the clean shape is a sibling per-session row: either (a) extend control_state with session-scoped keys (key = "session:<sid>:orchestration" etc), or (b) a new per-session config table keyed by chat-session-id. The KV upsert pattern is the precedent; reuse it, do NOT invent a new store. Toggles are session state, NOT project state - ResolvedProject (project_resolver.py:47-65) is frozen and carries packs/lanes/done/corpus_root only, no skills/objective/model_pool, correctly: per-session choices never belong on the project row.

THE 5 TOGGLES (what exists vs what the executor adds):
1. ORCHESTRATION ON/OFF - gates orchestrator-front vs direct chat. EXISTS as the COMPOSITION primitive: liberty's chat already has BOTH paths - direct turn (_apply_agent_backend/run_one_turn) AND fan-out (_start_orchestration -> orchestrator.orchestrate, screen.py:469-558). TEIKOKU ADDS: the per-session boolean that routes a user message to (orchestrator-front spawn) vs (direct base-cc turn). Read at message dispatch in the chat loop. NOTE the interaction with always-grill (section 4): orchestration-on means grill runs BEFORE the orchestrator spawns.
2. MODEL POOL - which SOURCE+group the picker offers (section 1 union). Persist the chosen pool; read by the picker to filter entries and by the slot resolver (section 2) to pick the harness.
3. MODEL CHOICES - the concrete model id within the pool. Persist; feeds slot.model (+ DEVIN_PROXY_MODEL for devin). Liberty's /model picker is the existing UI; TEIKOKU reuses backend_entries.
4. SKILLS - which .claude skills load for this session. EXISTS conceptually (base-cc reads .claude/skills); TEIKOKU ADDS the per-session skills allow/deny list that the spawned base-cc turn honors (an env/flag threaded into the base-cc command or its CLAUDE.md/skills resolution). Keep PACK-GENERAL: the skill SET is per-project (a non-ELAI project ships its own skills), the toggle is which subset is active this session.
5. OBJECTIVE - the run goal. EXISTS at the HEADLESS layer as free-text/template intake: HeadlessTemplate (headless_runner.py:65-104) + select_unfinished_features grep + free-text -> spawn.enqueue_spawn. For the INTERACTIVE chat the objective is the per-session goal string. TEIKOKU ADDS: persist the objective on the session row; it seeds prompt-transform / the orchestrator task. WARNING - GENERALITY: the built-in HeadlessTemplate registry has ELAI-HARDCODED template_ids ("finish-elai", "finish-elai-backlog", "gpu-train-elai", headless_runner.py:80-101). The interactive objective toggle must NOT inherit those as defaults for a non-ELAI project; objective is free-text or a PROJECT-SUPPLIED template, never the elai presets baked into the engine.

GENERALITY: persist toggles in the session-scoped KV (control_state pattern) keyed by chat-session-id; the VALUES (pool/model/skills/objective) are data, never ELAI literals. The store is project-agnostic already.

================================================================================
4. ALWAYS-GRILL - full-depth gate before the chat acts
================================================================================

grill SKILL (.claude/skills/grill/SKILL.md) is a USER-INVOKED interrogation that runs BEFORE prompt-transform. Its loop (SKILL.md THE METHOD 1-7):
1. MAP THE DECISION TREE (silent, via mcp_think/GRC) - lay out the branches the plan depends on, resolve parent before child.
2. CODEBASE-FIRST - check whether the code already settles a question before asking (codegraph/elai.map/feature-map); a settled question becomes a stated finding, not a question.
3. ONE QUESTION AT A TIME via AskUserQuestion, recommended answer FIRST (labelled "(Recommended)") with grounding, plus real alternatives.
4. PIN OVERLOADED TERMS on sight. 5. SURFACE CONTRADICTIONS (code vs claim) flatly. 6. STRESS-TEST edge cases per branch. 7. ROUTE UNGROUNDED CLAIMS to grounding (oracle/paper-ingestor/WebSearch/gpt-bridge), never guess.
STOP CONDITION: the "can i predict the next 3 answers?" ~95% confidence test, THEN a two-pass replay of the resolved decision list requiring an EXPLICIT user yes. ANTI-SPIRAL: a branch that flips 3x is marked DEFERRED, not looped.
OUTPUT: a GRILL RESULT block (RESOLVED DECISIONS + GLOSSARY + CONTRADICTIONS RESOLVED + EDGE CASES DECIDED + DEFERRED + OPEN-FOR-GROUNDING) handed to prompt-transform / planning-workflow / idea-intake (SKILL.md OUTPUT). The resolved-decisions block is the spec prompt-transform reformats.

HOW THE INTERACTIVE CHAT INVOKES IT AT FULL DEPTH FOR EVERY NON-TRIVIAL ACTION:
- grill normally fires only on explicit user invocation (SKILL.md HARD CONSTRAINT - USER-INVOKED ONLY, line 18-20; never auto-fire off a keyword). "always-grill" is a TOGGLE that flips this default: when on, every non-trivial chat action runs the full grill loop (tree-map -> one-q-at-a-time AskUserQuestion -> resolved-decisions block) BEFORE the action proceeds. This is a deliberate per-session opt-IN to the auto-fire that grill otherwise forbids - the toggle is the user's standing consent, satisfying the user-invoked constraint at session grain rather than per-message.
- COMPOSITION WITH THE ORCHESTRATOR-FRONT: grill runs BEFORE the orchestrator spawns. Flow when always-grill ON + orchestration ON: user message -> grill loop (resolve every decision, hand a resolved-decisions block to prompt-transform) -> prompt-transform reformats -> orchestrator.orchestrate spawns the fan-out off the CLEANED task. grill is the front-most gate; the orchestrator never sees an un-grilled task. (matches SKILL.md flow: grill -> prompt-transform -> grounding sweep -> planning/orchestrate.)
- TRIVIAL-ACTION SKIP: grill's own WHEN NOT TO INVOKE (SKILL.md 29-34) already exempts already-spec'd tasks, pure fact lookups, and direct concrete commands. "always-grill full-depth" means non-trivial actions get the full loop; a trivial/already-spec'd message still skips (otherwise every "hi" triggers an interrogation). Define "non-trivial" as the gate predicate.

TOGGLE INTERACTION (#1 orchestration on/off x always-grill):
- always-grill ON + orchestration ON: grill -> prompt-transform -> orchestrator spawn (grill gates the fan-out).
- always-grill ON + orchestration OFF: grill -> prompt-transform -> direct base-cc turn (grill gates the single turn).
- always-grill OFF: prompt-transform's normal auto-fire only (grill silent unless the user types /grill). This is a 2x2; the executor wires both toggles into the message-dispatch predicate.

================================================================================
5. GENERALITY + WIRING
================================================================================

NO ELAI HARDCODE confirmation:
- PICKER: catalog.FREE_MODELS is a GENERAL free-model roster (litellm provider strings, zero ELAI refs). The controller-side {claude, devin-free} entries are model backends, not ELAI. CLEAN.
- TOGGLES: control_state KV + session-scoped rows are pure data. ResolvedProject correctly holds NO per-session toggle. CLEAN. THE ONE HARDCODE RISK: HeadlessTemplate registry ELAI ids (headless_runner.py:80-101) - the OBJECTIVE toggle must not default to these for a non-ELAI project (use free-text or project-supplied templates). The generality gate (revision :90-93, BLOCKING) already requires a non-ELAI task end-to-end with ZERO ELAI reachable; the objective-default path is exactly where to test it.
- GRILL: grill is a general interrogation skill; its codebase-first step uses ELAI tools (codegraph-elai/elai.map) but that is the HOST harness's grounding, not baked into TEIKOKU's per-project chat. For a non-ELAI project grill grounds against THAT project's code. CLEAN, but confirm the grill skill's grounding tools are pack-parameterized (it currently hard-names codegraph-elai/elai.map/ELAI-FEATURE-MAP in SKILL.md step 2 - that is the ELAI host's grounding, acceptable for the ELAI-hosted controller, but a per-project chat should ground against the per-project graph).

WIRING TO REAL ENTRYPOINTS (not wire-dark):
- picker -> slot resolver: wired at WorkerSlotConfig construction in the chat/headless spawn path (the override applied before worker_adapter.dispatch, section 2). Real entrypoint = the conductor spawn that builds the slot.
- toggles -> control_state upsert (ControlManager pattern) read at message dispatch. Real entrypoint = the chat message handler + the spawn path.
- grill gate -> the chat message handler (fires grill before prompt-transform/orchestrate). Real entrypoint = the interactive front-end's per-message dispatch.

NEW TESTS NEEDED (all on a NON-ELAI project per the generality gate):
- PICKER RESOLVES EACH POOL TO THE RIGHT BACKEND: assert pick(claude)->harness=claude/no base_url; pick(devin-free kimi)->harness=devin-free/base_url=bridge/devin model threads to DEVIN_PROXY_MODEL; pick(liberty group X)->harness=free-api/base_url=:4000/model=alias; pick(codex)->harness=codex/account set. Drive worker_adapter._build_env off each resolved slot and assert the env (precedent: existing base-cc backend tests).
- TOGGLES PERSIST + READ: write each of the 5 toggles to the session-scoped store, re-read across a simulated session resume, assert round-trip; assert a non-ELAI project's objective default is NOT an elai template.
- GRILL GATE FIRES BEFORE ACTION: assert always-grill ON intercepts a non-trivial message and hands a resolved-decisions block to prompt-transform BEFORE any orchestrate/direct-turn spawn; assert always-grill OFF does not intercept; assert the 2x2 toggle interaction (orchestration on/off x grill on/off).
- GENERALITY GATE (BLOCKING): run the picker + toggles + grill path end-to-end on the P3 demo pack with ZERO ELAI paths/templates/grounding reachable.

================================================================================
VALIDATION WARNINGS
================================================================================
- SEAM TO CONFIRM (devin model selection): the picked devin model (kimi-k2-7/glm-5-2/swe-1-6) is selected on the PROXY side via DEVIN_PROXY_MODEL (worker_providers.py:124-127), NOT a base-cc --model. How a per-session devin-model choice reaches the bridge's per-request model is UNVERIFIED here (the bridge runs on the Windows host; its per-session model routing was not read this task). Confirm before wiring toggle #3 for the devin pool.
- HARDCODE RISK (objective): HeadlessTemplate ELAI template_ids must not be the interactive-objective default for non-ELAI projects (headless_runner.py:80-101).
- bridge parallel tool-calls UNVERIFIED (revision CAVEAT 2) - orthogonal to the picker but bounds devin-pool concurrency.
- grill SKILL.md step-2 grounding tools (codegraph-elai/elai.map/ELAI-FEATURE-MAP) are ELAI-host-named; pack-parameterization for a per-project chat is a generality follow-up, not a blocker for the ELAI-hosted controller.
- The chat front-end is LIBERTY's TUI (Textual) shape; TEIKOKU's chat is a Tauri/Svelte dashboard pane (P4-E) over the base-cc worker plane - the picker DATA SHAPE ports, the Textual widget code does NOT. Reuse limits.py/catalog.py (pure, no Textual dep), not screen.py's UI.

SOURCES CONSULTED
.liberty/liberty/catalog.py (full); chat/limits.py (full); chat/commands.py (full); chat/session_factory.py (full); chat/screen.py:418-640
~/teikoku/conductor/worker_providers.py (full); worker_adapter.py:438-525,610-635; free_api_worker.py (grep); codex_provider.py (grep); conductor_config.py:25-44,180-194; control_plane.py:1-32,169-210; conductor_db.py:60-75 (grep); headless_runner.py (grep); project_resolver.py:47-160
.claude/skills/grill/SKILL.md (full); TEIKOKU-P4-REVISION-2026-06-20.md (full); workflow/handoff/teikoku-build.md (head)
