Source: axi (Agent eXperience Interface standard)
Reliability: MEDIUM (reviewed source repo github.com/kunchenguid/axi; axi.md site: MEDIUM-doc)
Files examined:
  docs/.source/axi/.agents/skills/axi/SKILL.md
  docs/.source/axi/bench-browser/config/conditions.yaml
  docs/.source/axi/bench-browser/config/tasks.yaml
  axi.md website (WebFetch 2026-06-24)

Repo: github.com/kunchenguid/axi (confirmed cloned successfully, non-empty).
TOON spec: https://toonformat.dev/reference/spec.html (external, not cloned here).
Related repos (from axi.md site): github.com/kunchenguid/gh-axi, github.com/kunchenguid/chrome-devtools-axi,
  github.com/kunchenguid/lavish-axi; npm packages: npm-axi, sqlite-axi, slack-axi, gws-axi, harvest-axi,
  gitsheets-axi, metabase-axi, otter-axi (community implementations, NOT cloned or verified).

THE 10 AXI PRINCIPLES (verbatim from SKILL.md, which is the canonical source):

1. TOKEN-EFFICIENT OUTPUT
   "Use TOON (Token-Oriented Object Notation) as the output format on stdout.
   TOON provides ~40% token savings over equivalent JSON while remaining readable by agents.
   Convert to TOON at the output boundary — keep internal logic on JSON."
   TOON example (verbatim from SKILL.md):
     tasks[2]{id,title,status,assignee}:
       "1",Fix auth bug,open,alice
       "2",Add pagination,closed,bob

2. MINIMAL DEFAULT SCHEMAS
   "Every field in stdout costs tokens — multiplied by row count in collections.
   Default to the smallest schema that lets the agent decide what to do next: typically an identifier,
   a title, and a status."
   Rules:
     "Default list schemas: 3-4 fields, not 10"
     "Default limits: high enough to cover common cases in one call"
     "Long-form content (bodies, descriptions) belongs in detail views, not lists"
     "Offer a --fields flag to let agents request additional fields explicitly"

3. CONTENT TRUNCATION
   "Detail views often contain large text fields. Omitting them forces agents to hunt;
   including them wastes tokens. Truncate by default and tell the agent how to get the full version."
   TOON example (verbatim):
     task:
       number: 42
       title: Fix auth bug
       state: open
       body: First 500 chars of the issue body...
         ... (truncated, 8432 chars total)
     help[1]: Run `tasks view 42 --full` to see complete body
   Rules:
     "Never omit large fields entirely — include a truncated preview"
     "Show the total size so the agent knows how much it's missing"
     "Suggest the escape hatch (--full) only when content is actually truncated"
     "Choose a truncation limit that covers most use cases (500-1500 chars)"

4. PRE-COMPUTED AGGREGATES
   "The most expensive token cost is often not a longer response — it's a follow-up call.
   If your backend has data that agents commonly need as a next step, compute it and include it."
   Rules:
     "Aggregate counts: include the total count in list output, not just the page size."
     TOON example: "count: 30 of 847 total"
     "Derived status fields: include a lightweight summary inline when next step almost always checks them."
     "Only include derived fields your backend can provide cheaply — a summary ('3/3 passed'), not the full data."

5. DEFINITIVE EMPTY STATES
   "When the answer is 'nothing', say so explicitly. Ambiguous empty output causes agents to re-run
   with different flags to verify."
   Example: "tasks: 0 closed tasks found in this repository"
   "State the zero with context. Make it clear the command succeeded — the absence of results is the answer."

6. STRUCTURED ERRORS & EXIT CODES
   Idempotent mutations:
     "Don't error when the desired state already exists. If the agent closes something already closed,
     acknowledge and move on with exit code 0."
     "Reserve non-zero exit codes for situations where the agent's intent genuinely cannot be satisfied."
     Example: "task: #42 already closed (no-op)    # exit 0"
   Structured errors on stdout:
     "Errors go to stdout in the same structured format as normal output, so the agent can read and
     act on them. Include what went wrong and an actionable suggestion."
     "Never let raw dependency output (API errors, stack traces) leak through."
     "Validate required flags before calling any dependency"
     "Translate errors — extract actionable meaning, discard noise"
     "Never leak dependency names — suggestions reference your CLI's commands, not the underlying tool"
   No interactive prompts:
     "Every operation must be completable with flags alone. If a required value is missing, fail
     immediately with a clear error — don't prompt for it. Suppress prompts from wrapped tools."
   Output channels (verbatim):
     "stdout: all structured output the agent consumes — data, errors, suggestions"
     "stderr: debug logging, progress indicators, diagnostics (agents don't read this)"
     "Exit codes: 0 = success (including no-ops), 1 = error, 2 = usage error"
     "Never mix progress messages into stdout. An agent that reads 'Fetching data...' will try to
     interpret it as data."

7. AMBIENT CONTEXT VIA SESSION INTEGRATIONS
   "Register your tool into the agent's session lifecycle so every conversation starts with relevant
   state already visible — before the agent takes any action."
   Pattern (verbatim):
     "1. Provide an explicit setup command that installs or repairs a session hook or plugin after
     user intent is clear"
     "2. At session start, the integration runs your tool and provides a compact dashboard as context"
     "3. The agent receives this as initial context and can act immediately"
   Rules (verbatim):
     "Default app targets: by default, support Claude Code, Codex, and OpenCode."
     "Explicit opt-in: register hooks or plugins only from a user-invoked setup command, not from
     ordinary CLI commands"
     "Portable commands: hook commands should use a PATH-verified binary name when it resolves to the
     current executable, and fall back to the full absolute path otherwise."
     "Path repair: setup commands should check existing hooks and update the executable path if it has
     changed (e.g., after reinstall or relocation)"
     "Idempotent: repeated installs with the same path are silent no-ops"
     "Directory-scoped: show only state relevant to the current working directory"
     "Token-budget-aware: this context loads on every session — ruthlessly minimize it."
     "Lifecycle capture: use session-end hooks to capture what happened (transcripts, files touched,
     specs referenced) so future session-start context gets richer over time"
   Agent-specific integrations (verbatim):
     "Claude Code: use native hooks in ~/.claude/settings.json or project .claude/settings.json.
     Prefer SessionStart to inject compact context via stdout"
     "Codex: use native hooks in ~/.codex/hooks.json or <repo>/.codex/hooks.json, and ensure
     [features].hooks = true in config.toml. Prefer SessionStart for ambient context via stdout"
     "OpenCode: use a managed plugin in ~/.config/opencode/plugins/."
   Also ship an installable skill (SECONDARY):
     "npx skills add <owner>/<repo> --skill <name>"
     "Single source of truth: generate SKILL.md from the same content your no-args home view prints"
     "Add a --check build step to CI that fails if the committed skill is stale"
     "Strip live state: a skill is static, so omit dynamic data (open sessions, current items)"
     "Non-interactive commands: rewrite command examples to a form the agent can run without a global
     install (e.g. npx -y mytool ...), since a skill may be installed without the binary on PATH"
     "Trigger-shaped frontmatter: include name and a description written as a trigger"

8. CONTENT FIRST
   "Running your CLI with no arguments should show the most relevant live content — not a usage manual.
   When an agent sees actual state it can act immediately. When it sees help text, it has to make a
   second call."
   Example (verbatim):
     $ tasks
     tasks[3]{id,title,status}:
       1,Fix auth bug,open
       2,Add pagination,open
       3,Update docs,closed
     help[2]:
       Run `tasks view <id>` to see full details
       Run `tasks create --title "..."` to add a task

9. CONTEXTUAL DISCLOSURE
   "Include a few next steps that follow logically from the current output.
   The agent discovers your CLI's surface area organically by using it, not by reading a manual upfront."
   Rules (verbatim):
     "Relevant: after an open item → suggest closing; after an empty list → suggest creating; after a list → suggest viewing"
     "Actionable: every suggestion is a complete command (or template) carrying forward any disambiguating flags"
     "Parameterize dynamic values: use placeholders like <id> or '<title>' instead of guessing a concrete value"
     "Omit when self-contained: when the output fully answers the query (a detail view, a count, a confirmation), suggestions are noise"
     "Guide discovery, not workflows: suggest a variety of possible next actions, don't prescribe a fixed sequence"
     "Reveal truncated lists: when a list shows only N items out of a larger total, add a help hint telling how to see all"
     "Don't encode pagination into TOON array headers — use help hints instead"
     "Resolve errors: on errors, suggest the specific command that fixes the problem, not 'see --help'"

10. CONSISTENT WAY TO GET HELP
    "The top-level home view should also identify the tool itself before the live data:
    Include the absolute path of the current executable, with the user's home directory collapsed to ~
    Include a one-sentence description of what this AXI does"
    Example:
      $ tasks
      bin: ~/.local/bin/tasks
      description: Manage project tasks in the current workspace
    "Every subcommand should support --help with a concise, complete reference: available flags with
    defaults, required arguments, and 2-3 usage examples. Keep it focused on the requested subcommand
    — don't dump the entire CLI's manual."

TOON FORMAT SPECIFICATION (from SKILL.md + axi.md site):

Full spec at: https://toonformat.dev/reference/spec.html (NOT fetched; do not invent rules from training).
Go library: github.com/toon-format/toon-go v0.0.0-20251202084852-7ca0e27c4e8c (used by no-mistakes go.mod).
JS library: axi-sdk-js ^0.1.7 (used by lavish-axi package.json; npm package, not cloned here).

Key TOON constructs (verbatim from SKILL.md examples only; complete spec requires toonformat.dev):

Collection header: <name>[<count>]{<field>,<field>,...}:
  tasks[2]{id,title,status,assignee}:
    "1",Fix auth bug,open,alice
    "2",Add pagination,closed,bob

Scalar:
  count: 30 of 847 total

Detail object:
  task:
    number: 42
    title: Fix auth bug
    state: open

Truncation hint (no formal TOON tag name given in SKILL.md; appears as plain continuation):
  body: First 500 chars of the issue body...
    ... (truncated, 8432 chars total)

Help hints:
  help[N]:
    Run `cmd arg` to do X
    Run `cmd other` to do Y

Error:
  error: --title is required
  help: tasks create --title "..." [--body "..."]

No-op:
  task: #42 already closed (no-op)

Token savings claim: "~40% token savings over equivalent JSON" (AUTHOR-SELF-REPORTED, not measured by us).

BENCHMARK METHODOLOGY (verbatim from axi.md WebFetch 2026-06-24):

Infrastructure: "Each run creates a fresh workspace with condition-specific CLAUDE.md. All MCP-based
  conditions use the same backend: chrome-devtools-mcp@latest --headless --isolated.
  Agent: Claude Sonnet 4.6. Judge: Claude Sonnet 4.6.
  Five repeats per condition-task pair: 14 × 7 × 5 = 490 total runs."

Tasks: "14 tasks across four categories" (confirmed by bench-browser/config/tasks.yaml):
  single_step (4): read_static_page, wikipedia_fact_lookup, github_repo_stars, wikipedia_table_read
  multi_step (5): wikipedia_link_follow, github_navigate_to_file, wikipedia_infobox_hop,
    multi_page_comparison, wikipedia_search_click
  investigation (4): wikipedia_deep_extraction, github_issue_investigation, multi_site_research,
    tabular_data_analysis
  error_recovery (1): navigate_404

Conditions (from bench-browser/config/conditions.yaml):
  agent-browser          — Vercel Agent Browser (Rust CLI), AXI-compliant
  chrome-devtools-axi    — Chrome DevTools AXI (CLI wrapper), AXI-compliant
  chrome-devtools-mcp    — Chrome DevTools MCP (no ToolSearch)
  chrome-devtools-mcp-search — Chrome DevTools MCP (with ToolSearch)
  chrome-devtools-mcp-code   — Chrome DevTools (code execution via tsx scripts)
  chrome-devtools-mcp-compressed-cli — MCP Compressor CLI
  dev-browser            — Dev Browser (sandboxed JS CLI)
  NOTE: bench conditions.yaml lists 7+ conditions, not the 3 discussed on axi.md site.

Grading: "LLM-as-judge (claude-sonnet-4-6) to evaluate the agent's trajectory.
  grading_hint — optional context to help the judge verify correctness."
  Judge compares agent's answer against reference answer + rubric.

BENCHMARK RESULTS (verbatim from axi.md, AUTHOR-SELF-REPORTED, NOT measured by us):

Browser automation (490 runs; conditions in site: agent-browser vs MCP variants):
  AXI:            100% success | $0.074/task | 21.5s | 4.5 turns
  MCP standard:   99% success  | $0.100/task | 26.0s | 6.2 turns
  MCP code mode:  100% success | $0.120/task | 36.2s | (turns not stated)

GitHub operations (425 runs; conditions: gh-axi vs GitHub MCP):
  AXI:            100% success | $0.050/task
  MCP:            82-87% success | $0.101-$0.148/task

Schema overhead claim (AUTHOR-SELF-REPORTED):
  "MCP conditions average 185K tokens per task vs. 79K for AXI"

IMPORTANT: These are the author's self-reported benchmark numbers. We have the raw published results
in docs/.source/axi/bench-browser/published-results/ (jsonl files per run); verification of the
aggregate numbers requires reading those files. Numbers as stated above are UNVERIFIED aggregates.

DESIGN PHILOSOPHY (from SKILL.md verbatim):
"AXI defines ergonomic standards for building CLI tools that autonomous agents interact with through
shell execution."

The standard targets CLI tools specifically — not MCP servers, not REST APIs. The comparison against
MCP is the primary motivation: MCP tools that return large JSON schemas impose per-call token overhead;
a CLI tool built to AXI principles returns TOON (compact text) with minimal schema bloat.

Gotchas:
  TOON format requires reading toonformat.dev/reference/spec.html; the SKILL.md only shows examples,
    not the complete grammar. Do NOT invent TOON syntax from these examples.
  axi-sdk-js ^0.1.7 (JS library for building AXI-compliant tools); package NOT cloned here.
    Import from npm; verify before use: npm view axi-sdk-js version.
  toon-format/toon-go is the Go library (already pinned in no-mistakes go.mod).
  Benchmark compared agent-browser (AXI CLI) vs chrome-devtools-mcp (MCP). The agent-browser tool
    itself is a separate Rust CLI not in this repo; its AXI compliance is what's being measured.
  The "~40% token savings" claim applies to TOON vs JSON for the same data; it is not a claim about
    reduced total task cost (though cost savings are claimed separately in the benchmark).

Critical assertions (testable):
  assert AXI exit codes: 0 = success (including no-ops), 1 = error, 2 = usage error
  assert errors go to stdout (not stderr); progress/debug go to stderr
  assert no-arg invocation returns live content, not help text
  assert idempotent mutations return exit 0 with no-op message
  assert every structured output uses TOON format (not raw JSON) on stdout
  assert help[N] hints are included on list/mutation output but omitted on fully self-contained outputs

RELIABILITY: MEDIUM (SKILL.md and bench config verified from source; benchmark aggregate numbers are
author-self-reported and not independently verified by us from the raw jsonl results)
