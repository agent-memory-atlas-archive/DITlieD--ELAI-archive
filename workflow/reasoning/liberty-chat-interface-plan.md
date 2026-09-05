Liberty frontend chat interface - planning notes

WHAT THIS IS: a plan to add an interactive CHAT front-end to Liberty (the free-LLM harness at
F:\.ELAI_workflow\.liberty). Today the only interface is one-shot `liberty run "<task>"` (inherited
from the ale_claw / agents-last-exam BENCHMARK harness, which by design runs to DONE and exits, no REPL).
The engine underneath is already multi-turn + stateful; the chat mode is a front-end + one loop refactor.
NOT built yet. This is the orientation doc the implementer reads first.

THE ONE REAL REFACTOR (everything else already works per-turn)
- liberty/agent_loop.py  LibertyAgent.run() runs to a DONE signal then returns final text. Chat needs a
  "run ONE user turn: process tools/turns until the agent yields the turn back to the user (assistant
  message with no tool calls, or an explicit question), return control, KEEP session state alive."
  Then a REPL calls that repeatedly on ONE persistent session, feeding each new user line as a user message.
- Mid-task interjection (v2): mirror the subagent steer-inbox pattern (subagent_session.py _poll_inbox /
  self._inbox) so the user can redirect a long run without killing it.

IMPORTANT FILE PATHS (all under F:\.ELAI_workflow\.liberty\)
  ENTRY / CLI
  - liberty/cli.py            CLI subcommands, .env load (load_dotenv), router build; cmd_run/_run_single
                              is the TEMPLATE for a new cmd_chat. Add a `chat` subparser + dispatch entry.
  - liberty/__main__.py       module entry (python -m liberty ...)
  CORE LOOP / SESSION
  - liberty/agent_loop.py     LibertyAgent.run() <- the loop to refactor for "yield-to-user"
  - liberty/agent_loop_helpers.py  message_text(), to_api_messages() (reasoning-model + sanitize helpers)
  - liberty/session.py        SessionManager (transcript persistence; chat reuses one session)
  - liberty/resume.py         resume an interrupted session (persistence cousin of a live chat)
  ROUTING / RELIABILITY (already handle death + rate-limit + swap)
  - liberty/router.py         build_router() + _HealingProxy
  - liberty/self_heal.py      pool-exhaustion queue+retry
  - liberty/catalog.py        model groups + per-model context windows
  - liberty/quota.py          per-key quota / cooldown tracker
  CONTEXT / TOKENS (already per-turn; chat should SURFACE these live)
  - liberty/context/context.py     ContextOverflowCallback (auto-compact trigger + model-aware window)
  - liberty/context/compaction.py  compact_messages() (summarize older turns)
  - liberty/usage_ledger.py        the real-time token ledger -> show a live token/cost line in chat
  TOOLS / PROMPT
  - liberty/tools/workspace_tools.py   run_shell / read_file / write_file / list_dir
  - liberty/tools/external_tools.py    opt-in codex/bridge lanes (ask_codex etc.)
  - liberty/prompt.py          PromptBuilder (system prompt; chat wants a conversational variant)
  DOCS
  - README.md  ARCHITECTURE.md  docs/FREE_PROVIDER_KEYS.md
  LAUNCHER
  - start-and-test.bat         add a [C] chat option once cmd_chat exists

OPEN-SOURCE FRONT-ENDS TO STEAL FROM (study the UX, port patterns; Python ones first since Liberty is Python)
  (anti-slopsquat: VERIFY any package name+version on PyPI before it becomes a real dependency.)
  - Aider              github.com/Aider-AI/aider           terminal chat CODING agent; slash-commands,
                                                           streaming, /undo /diff, prompt_toolkit input. Closest fit.
  - Open Interpreter   github.com/OpenInterpreter/open-interpreter   Python chat REPL for a code agent; clean message loop, confirm-before-run.
  - prompt_toolkit     github.com/prompt-toolkit/python-prompt-toolkit   THE Python REPL input lib (history, multiline, keybinds).
  - Rich / Textual     github.com/Textualize/rich , /textual           terminal rendering: markdown, panels, a LIVE token-meter widget.
  - OpenAI Codex CLI   github.com/openai/codex             terminal agent UX (TS/Rust) - patterns: approval modes, status line.
  - Gemini CLI         github.com/google-gemini/gemini-cli  TS - slash-commands, session UX, /chat save+resume.

MINIMUM-VIABLE CHAT (v1 scope suggestion)
  1. cmd_chat in cli.py: build router+agent ONCE, one SessionManager session.
  2. REPL: read a line (prompt_toolkit or stdlib input), append as user msg, run one user-turn, print reply, loop.
  3. Per-turn the existing machinery runs untouched: auto-compact, model-aware window, self-heal swap, token ledger.
  4. Show a one-line live status after each turn (model/group used + session token total from usage_ledger).
  5. Slash-commands: /exit /reset /model <group> /usage /save (reuse resume.py). Keep it small (<=6 verbs).
  v2: streaming tokens, mid-run steer (steer-inbox), Rich markdown render, Textual TUI.

PREREQUISITE BUGS (found 2026-06-16 while testing codex; FIX BEFORE/WITH the chat build - they block it)
  BUG-1 ENTRY-POINT SPLIT (blocks `liberty chat`): `python -m liberty` runs liberty/__main__.py, whose _parse_argv
    ONLY understands a bare task string or --orchestrate - it has NO subcommands. So `python -m liberty ask-codex|models|
    usage|quota|list-sessions|resume` are all swallowed as AGENT TASKS, not dispatched. The real subcommand CLI is
    liberty/cli.py:main (it HAS cmd_ask_codex/cmd_usage/etc.), reachable only via the `liberty` console script
    ([project.scripts] liberty=liberty.cli:main) - which is NOT installed (pip install -e . predates that entry).
    IMPACT: start-and-test.bat options 1/2 (models/quota/usage) are broken (they run the agent on task="models" etc.);
    run/orchestrate/smoke only work by accident (leading word becomes part of the task). A `chat` subcommand would be
    unreachable the same way. FIX: make __main__.py delegate to cli.py:main (single source of truth for the CLI) AND/OR
    ensure the console script installs; then python -m liberty <subcommand> works and the .bat can call real subcommands.
  BUG-2 SUBAGENT GROUP-ALIAS NOT RESOLVED: delegate_general subagents fail with litellm BadRequestError "LLM Provider NOT
    provided. You passed model=fast-small" - a router group alias reached litellm.acompletion without router resolution
    (same class as the old Defect-4). The delegate_general spawn path must resolve the group alias to a concrete model or
    route through the healing router (the main loop already does; the subagent delegate path does not in this case).

CONSTRAINTS (carry into the build)
  - stdlib + existing deps by default; any new dep (prompt_toolkit/rich/textual) is a CONFIRM-FIRST decision, verified on PyPI.
  - reuse the engine; do NOT fork the agent loop - refactor run() to expose a yield-to-user turn the REPL drives.
  - no new key handling; .env already loads. tests: a fake-backend REPL turn test + the loop-refactor unit test.
