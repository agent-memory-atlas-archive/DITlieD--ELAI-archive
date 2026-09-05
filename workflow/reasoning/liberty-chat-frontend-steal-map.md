Liberty chat frontend - steal map (Elia skeleton + Crush look, on Textual)

WHAT THIS IS: the file-by-file port map for building Liberty's interactive chat front-end by
stealing Elia's Textual chat TUI and wiring Liberty's existing engine underneath. Companion to
workflow/reasoning/liberty-chat-interface-plan.md (read that first for the why). This file is the
HOW: which Elia code to take, which Elia "project stuff" to strip, what to ADD that Elia lacks,
and the exact Liberty seams to wire into. Form factor was chosen by the user: full-screen Textual
TUI (not a line REPL). Look target = Charm Crush design language (accent + status bar + tool-call
cards); code skeleton = Elia.

GROUNDED SOURCES (verified by reading actual code 2026-06-16, not inference)
  Elia clone (real source to port FROM): F:\.ELAI_workflow\workflow\reference\elia\elia_chat\
    Apache-2.0, Textual framework, by Darren Burns (Textual core maintainer). Last release Sept
    2024 - EXPECT Textual API drift; pin/verify textual version on PyPI before adding.
  Liberty (implement INTO): F:\.ELAI_workflow\.liberty\liberty\
    Engine is already multi-turn + stateful. Only one real loop refactor needed (see THE SEAM).

THE SEAM (the single most important wiring point)
  Elia's core is Chat.stream_agent_response() at widgets/chat.py:156-249. It is a
  @work(thread=True) Textual worker that calls litellm.acompletion(stream=True) DIRECTLY and pipes
  text deltas (chunk.choices[0].delta.content) into a response Chatbox via call_from_thread.
  Two facts from reading it:
   (a) THIS is where Liberty's engine replaces litellm. The worker must instead drive Liberty's
       per-user-turn loop and stream BOTH assistant text AND tool-call steps.
   (b) Elia has ZERO tool-call rendering - it only handles text deltas. Liberty is agentic (runs
       workspace_tools across many internal turns before yielding to the user). So tool-step
       rendering is NEW code we ADD; it is the biggest single gap and the main reason a bare Elia
       copy is not enough.

LIBERTY-SIDE: THE ONE REAL REFACTOR (everything else already works per-turn)
  agent_loop.py LibertyAgent.run(self, initial_messages) -> str  (def at line 243)
    Today: loops litellm.acompletion until a DONE signal (regex on message-type output, line 29)
    or max_steps, then returns the final assistant text. The per-step loop body (roughly 243-413)
    already does: call model, detect tool calls, execute tools (435-533), compaction, subagent
    drain. It is reusable AS-IS for the per-step work.
    Refactor: expose run_one_turn() that runs the SAME per-step loop but STOPS when the agent
    yields control to the user (assistant message with no tool calls / an explicit question)
    instead of stopping only on DONE, and KEEPS the session + message list alive across calls.
    The REPL/TUI then calls run_one_turn() repeatedly on ONE persistent session, appending each
    new user line as a user message. DO NOT fork the loop - refactor run() to delegate to the
    per-turn primitive; run() becomes "loop run_one_turn() until DONE" so existing callers
    (cmd_run/_run_single) are unchanged in behavior.
    Streaming hook: run_one_turn() must emit incremental events the TUI consumes - assistant text
    chunks AND tool-step start/end (tool name, args summary, ok/err, duration). Cleanest shape: an
    async generator yielding typed events, OR a callback/sink passed in. The TUI worker adapts
    these to Chatbox mounts. Liberty already prints bare text in agent_loop.py:942 - that print
    path is the model for what to surface, but the TUI consumes events, not stdout.

PREREQUISITE (blocks `liberty chat` regardless of frontend) - FIX FIRST
  BUG-1 ENTRY-POINT SPLIT. liberty/__main__.py _parse_argv understands only a bare task or
  --orchestrate; it has NO subcommands. The real subcommand CLI is cli.py:main (it HAS the
  subparsers - p_run line 680, models/sync/list-sessions/resume/quota/usage, dispatch in main at
  line 827). So `python -m liberty chat` would be swallowed as an agent task. FIX: make
  __main__.py delegate to cli.py:main (single source of truth) and/or ensure the console script
  installs. Then add a `chat` subparser + cmd_chat dispatch in cli.py. Verify by running
  `python -m liberty chat` and confirming it reaches cmd_chat, not the agent.
  BUG-2 (subagent group-alias not resolved) is OUT OF SCOPE for chat v1 - it only affects
  delegate_general subagents, not the main chat loop. Note it, do not fix it here.

PORT MAP - per Elia file: STEAL (keep+adapt) / STRIP (delete, it is Elia project stuff) / WIRE

  app.py  ->  STEAL the App scaffold (class Elia(App), themes load, get_css_variables/watch_theme,
    RuntimeConfig signal pattern, BINDINGS). STRIP: ChatsManager.create_chat (SQLite, line 111),
    HomeScreen chat-list flow (Liberty chat is one live session, not a saved-chat browser - launch
    straight into the chat screen). WIRE: build Liberty router (cli.py _build_router line 61) +
    one SessionManager session ONCE at app construction; pass them down to the chat screen.
    Rename the app class to Liberty/LibertyChat.

  screens/chat_screen.py  ->  STEAL mostly as-is (thin screen hosting the Chat widget).
  screens/home_screen.py, screens/rename_chat_screen.py, screens/chat_details.py  ->  STRIP for
    v1 (saved-chat browser / rename / per-chat DB details are all SQLite-chat-app features Liberty
    does not need). Keep help_screen.py (STEAL, retarget bindings text).

  widgets/chat.py  ->  THE CORE. STEAL the event-driven message machinery: the Textual Messages
    (NewUserMessage, AgentResponseStarted/Complete/Failed), new_user_message() mount-user-chatbox
    flow, scroll-to-latest, input-lock reactive (allow_input_submit / submit_ready). STRIP:
    ChatsManager.add_message_to_chat (148), rename handling (291-297), the litellm import + the
    entire acompletion call (161-181). REWRITE stream_agent_response(): keep the @work(thread=True)
    + call_from_thread pattern, but drive Liberty's run_one_turn() instead of litellm; on each
    assistant text chunk append to the response Chatbox (existing path), on each tool-step event
    mount/update a ToolStepBox (NEW, see ADD). Persist turns via Liberty SessionManager, not
    ChatsManager.

  widgets/chatbox.py  ->  STEAL (markdown rendering of a message; append_chunk for streaming).
    Adapt to Liberty's message shape (agent_loop_helpers.message_text / to_api_messages) instead
    of Elia's ChatMessage(litellm param) in models.py.
  widgets/prompt_input.py  ->  STEAL as-is (the multiline input + PromptSubmitted/CursorEscaping
    messages). This is the input box.
  widgets/app_header.py, widgets/chat_header.py  ->  STEAL; retarget text to Liberty branding.
  widgets/agent_is_typing.py (ResponseStatus)  ->  STEAL (the "agent is responding" status line).
  widgets/chat_list.py, widgets/chat_options.py, widgets/welcome.py, widgets/token_analysis.py  ->
    chat_list = STRIP (saved-chat browser). chat_options/welcome/token_analysis = OPTIONAL steal
    later; not v1.

  themes.py + elia.scss (+ widgets/*.tcss if any)  ->  STEAL wholesale (this IS the look). Then
    apply the Crush design language on top: one locked accent color, soft borders, a PERSISTENT
    bottom status bar. Keep Elia's theme system; add a Liberty default theme.

  models.py  ->  STRIP/REPLACE. Elia's ChatMessage wraps a litellm ChatCompletionMessageParam and
    ChatData carries an EliaChatModel + DB id. Liberty has its own message dicts and session
    transcript. Define a thin Liberty view-model (role, text, optional tool-steps) backed by
    Liberty's message list; do NOT import litellm types.
  chats_manager.py, database/*  ->  STRIP entirely (SQLite persistence). Liberty persistence is
    session.py SessionManager + resume.py.
  config.py, runtime_config.py, launch_args.py, locations.py  ->  STRIP the EliaChatModel /
    LaunchConfig TOML provider config. Liberty model selection = catalog.py groups + router.py.
    Keep a minimal RuntimeConfig (selected group + system prompt) wired to Liberty catalog.

ADD (Elia has none of this)
  ToolStepBox widget (NEW): renders one agent tool step - tool name, short arg summary, live
    status (running / ok / err), duration. Crush-style card. Mounted into the same chat-container
    VerticalScroll between assistant text, in call order. Fed by run_one_turn() tool-step events.
  Status bar (NEW, Crush signature): persistent bottom line showing current model/group (from
    router/catalog) + live session token total + cost from usage_ledger.py. Update after each
    turn and ideally live during a turn.
  Slash commands (keep <=6 verbs per plan): /exit /reset /model <group> /usage /save (reuse
    resume.py) /help. Parse in the prompt-submit handler before treating input as a user message.
  Context surfacing: on auto-compact (context/context.py ContextOverflowCallback fires
    compaction.py compact_messages), show a transient notice line so the user sees it happened.

LIBERTY MODULES TO WIRE (all exist, verified)
  cli.py _build_router (61) + main subparsers (827)   router/agent construction + new chat subcmd
  agent_loop.py LibertyAgent.run (243)                refactor -> run_one_turn (THE seam)
  agent_loop_helpers.py message_text/to_api_messages  message shape for Chatbox
  session.py SessionManager                           one persistent session for the chat
  resume.py                                           /save + reload
  router.py build_router + _HealingProxy              death/rate-limit/swap (already per-turn)
  self_heal.py, catalog.py, quota.py                  pool-exhaust queue, groups+windows, cooldown
  context/context.py, context/compaction.py           auto-compact (already per-turn; surface it)
  usage_ledger.py                                     live token/cost for the status bar
  tools/workspace_tools.py, tools/external_tools.py   the tools whose steps ToolStepBox renders
  prompt.py PromptBuilder                             conversational system-prompt variant

DEPENDENCIES (CONFIRM-FIRST per plan; anti-slopsquat verify on PyPI before adding)
  textual (REQUIRED for this form factor; user chose the TUI - verify latest version on PyPI,
    pin it, expect drift from Elia's Sept-2024 pin). rich comes transitively with textual.
  Elia imports litellm for its own streaming - Liberty does NOT need litellm in the TUI path
  (Liberty's router/agent owns model calls). Do not add litellm as a new TUI dep.
  prompt_toolkit NOT needed (Textual owns input). Keep new deps minimal; nothing beyond textual.

TESTS (per plan: a fake-backend turn test + the loop-refactor unit test)
  1. run_one_turn() unit test against a fake/stub model backend: one user message in, assert it
     processes tool turns and STOPS at yield-to-user with session state intact; a second call
     continues the same session.
  2. run() still returns final text on DONE (behavior-preserving refactor regression test).
  3. A Textual snapshot/pilot test of the chat screen: submit a line, assert user Chatbox +
     response Chatbox (+ a ToolStepBox when the fake backend emits a tool call) mount.
  4. BUG-1: `python -m liberty chat` reaches cmd_chat (not swallowed as a task).
  Liberty uses cargo? No - Python. Run pytest in .liberty. Do not run a full unrelated suite.

SCOPE + GUARDRAILS FOR THE IMPLEMENTING AGENT
  v1 DELIVERABLE (land this, working + tested): BUG-1 fix; run_one_turn refactor; cmd_chat; a
  Textual app (chat screen, prompt input, message list, streaming text, ToolStepBox, status bar,
  the 6 slash commands) wired to Liberty router+session+ledger; the 4 tests green.
  DEFER to a handoff (do NOT sprawl trying to finish in one pass): mid-run steer-inbox (plan v2),
  saved-chat browser, BUG-2, chat_options/token_analysis widgets, polishing the theme to pixel-Crush.
  If you cannot land the full v1 in budget: STOP and write a handoff per the preamble R-4 with
  exactly what is done + the next seam - a producer without its consumer is a BLOCKED return, not
  a green DONE. Do not fork the agent loop. Do not weaken or skip the tests. Git is read-only to
  you (preamble R-1): no reset/checkout/clean/stash/commit - the orchestrator commits.
