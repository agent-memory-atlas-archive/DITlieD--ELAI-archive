# ELAI Workflow — How It All Works

A map of the orchestration harness living in `F:\.ELAI_workflow\`. This documents the
*tooling* (the `.claude/` control plane, the RAG, the MCPs, the rules), not the product
code. Generated 2026-06-19 by direct exploration of the repo.

---

## 1. The two roots (read this first)

There are two nested projects and the distinction governs everything:

- **`F:\.ELAI_workflow\`** = the **workflow / tooling root**. Holds the harness: `.claude/`
  (agents, skills, rules, tools, MCP servers), `workflow/` (memory-bank, sessions, papers,
  handoffs, reasoning), `docs/` (specs + ingested source repos), and the governing
  `CLAUDE.md`. **Almost all of this is gitignored.**
- **`F:\.ELAI_workflow\.elai_cc\`** = the **actual product** being built. A Rust-native
  autonomous LLM workflow *engine* (9-crate cargo workspace + a Svelte/Tauri dashboard +
  a Python inference sidecar). **This is the only thing tracked in git.**

HARD RULE: all code generation, edits, and builds target `.elai_cc\` unless the change is
explicitly to the workflow tooling itself. When a path is ambiguous, default to `.elai_cc\`.

### What `.elai_cc` is
A deterministic-first, model-agnostic (cloud + local) autonomous development engine. The
human gives a task; the system runs the full dev cycle (plan → test-first → execute → audit
→ memorize) with evidence gates at every transition. Key properties: entropy-quarantined
(all time/randomness behind injectable traits for deterministic replay), per-project
isolation (all state inside the project dir), no context compaction, code-driven
orchestration (~330 tokens of meta-instruction per turn, logic lives in Rust not prompts).

Stack: Rust edition 2021, tokio (madsim in deterministic-sim tests), 9 crates (core,
providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli). Local models
run on a remote RTX 3090 sidecar (Qwen2.5-Coder abliterated, NF4 4-bit); cloud lane is
Sonnet 4.6 workhorse + Opus confirmation-gated.

---

## 2. The core idea: a governed, multi-agent FSM

The main LLM (this assistant) is an **orchestrator, not an implementer**. It selects a tier,
dispatches specialized sub-agents in a fixed order, reads their compact returns, surfaces
decisions to the user, and commits once at the end. The heavy lifting (reading plans,
exploring code, writing files, running tests, writing memory) is delegated to agents that
each run in their own garbage-collected context window.

### The FSM (finite state machine)
```
IDLE → PLANNING → FEASIBILITY_PROBE → TESTING_FIRST → EXECUTING → AUDITING → MEMORIZING → COMPLETE → IDLE
```
- State is tracked via the `local-rag` MCP (`mcp_fsm_get_state` / `mcp_fsm_transition` /
  `mcp_fsm_reset`). The main LLM must check state before any Edit/Write/Bash.
- Each transition requires an **artifact** (evidence), never a self-assessment.
- AUDITING and MEMORIZING have agent sub-states.
- Hooks enforce the gates: PreToolUse (path/tool gating + file locks), PostToolUse (syntax
  checks, change recording), Stop (blocks mid-workflow termination), PreCompact (autosaves a
  handoff before context truncation).

### Role boundaries (who is allowed to do what)
- **Main LLM**: orchestrates, dispatches agents, reads one-line returns, asks the user on
  forks, manages FSM, authors the cross-session handoff. Does NOT run tests, write
  memory-bank files, or read plan/context/exploration files directly.
- **Auditor only** runs tests.
- **Memory-writer only** writes memory-bank files (via RAG-indexed `mcp_memorize`).
- **Sub-agents** read plans/specs/context so the main context stays clean.

### Tiers (set per task)
- **SIMPLE / FOCUSED / HEAVY** — picks how many agents and how much exploration. HEAVY flows
  fan out context-compiler + parallel integration-explorers.

---

## 3. Agents (`.claude/agents/`, 31 files)

ELAI agents are NOT in the Agent-tool's `subagent_type` enum. They're dispatched via
`subagent_type=general-purpose` with the prompt: *"Read F:\.ELAI_workflow\.claude\agents\<name>.md
and follow it strictly. <args>"*. They all run on **model=sonnet** by default. Every dispatch
must begin with the `agent-dispatch-preamble.md` pointer (makes workers git-read-only, scope-bound,
handoff-on-block).

### Core workflow agents
| Agent | Role |
|---|---|
| `context-compiler` | Reads memory-bank, plans, specs; emits a compact context package. Main LLM never reads those directly. |
| `integration-explorer-elai` | Explores subsystems/wiring for a sub-phase; returns the `PRODUCTION_HOOK` seam where new code gets wired. |
| `scope-planner-elai` | Partitions work into scoped units. |
| `synthesizer-elai` | Synthesizes multi-agent outputs. |
| `auditor` | The sole test runner. Runs the wiring gate, maintainability gate, production-caller gate off-agent. Returns PASS/BLOCK. |
| `memory-writer` | Writes memory-bank files via RAG-indexed `mcp_memorize`/`mcp_log_change`. |
| `prompt-engineer` | Transforms prompts, sets `mcp_think` budgets. |
| `impact-mapper` | Pre-edit blast-radius analysis over `.rs/.py/.ts/.svelte`. |
| `spec-grounder` | Verifies "Spec references" claims map to real spec content. |
| `paper-ingestor` | Ingests papers (mode=paper) or source/SDKs (mode=source) into RAG; emits an interface map. |
| `fixer` | Applies fixes. |
| `test-attributer` | Attributes build/test failures. |
| `wiring-contract-validator` | Validates WIRING_CONTRACT blocks (frozen gate artifact). |
| `sidecar-health` | 3-step remote-sidecar health check. |
| `dashboard-smoker` | svelte-check + cargo check + optional tauri:dev smoke for dashboard work. |

### Idea-intake decomposition agents (Layer-A pipeline)
`idea-framer` ↔ `idea-critic` (loop) → `researcher` → `concept-decomposer` → `phase-decomposer`
→ `step-decomposer` → `babystep-author`, with `designer` and per-level auditors. These
recursively decompose a raw idea into Concepts → Phases → Steps → BabySteps with grounding at
every level. (Generic `integration-explorer`, `scope-planner`, `synthesizer` siblings exist
alongside the `-elai` variants.)

### Anti-spiral personas (lateral-thinking switchers)
`persona-hacker`, `persona-researcher`, `persona-simplifier`, `persona-architect`,
`persona-contrarian` — invoked when work stalls (see anti-spiral skill).

### Adversarial
`adversary` — cross-family adversarial critic (generates counter-evidence from a *different*
model family). Plus the Agent-tool's built-in `adversarial-auditor`.

---

## 4. Skills (`.claude/skills/`, 17 + a few built-ins)

Skills are trigger-activated workflows. The three mandatory ones run in order:
`prompt-transform` → `planning-workflow` → `rag-tools`.

### Mandatory / core
- **prompt-transform** — MUST run before any implementation (except `!`-prefixed or yes/no
  replies). Reformats the user prompt, sets reasoning budget.
- **planning-workflow** — plans, FSM states, TDD, audits, phase completion, the 9-step
  per-phase Plan Improvement protocol, context-exhaustion handoff.
- **rag-tools** — rules for every `mcp_*` RAG call.
- **anti-spiral** — stagnation detector (4 patterns: Spinning, Oscillation, NoDrift,
  DiminishingReturns) + 5-persona switch. Fires on repeated edits / failed hypotheses /
  double BLOCK / plateaued pass rate.

### Orchestration
- **orchestrate** — walks a phase markdown file through the FSM per sub-phase; agent dispatch
  is BLOCKING (governed by `orchestrate-protocol.md`, H-1..H-11). Commits once at the end.
- **auto-orchestrate** — general autonomous orchestration for *any* task (not just plan files).
- **b-plan-runner** — full B-N plan FSM cycle.
- **paper-to-plan** — drop a paper/repo link, get new plan files + index updates (steal pipeline).
- **idea-intake** — Layer-A recursive idea decomposition (the agent pipeline in §3).

### Design / UI (when the look is undecided or being iterated)
- **cook** — co-creation front-end that *discovers* a design via real gpt_image renders
  (FRAME → DIVERGE → CONVERGE → EMIT brief). User-invoked only.
- **grill** — relentless one-question-at-a-time interrogation to drive decisions out of a
  fuzzy plan before prompt-transform. User-invoked only.
- **spec-eval** — judge ELAI's *design* on merits (shipped-vs-planned kept separate from the verdict).

### Grounding / Q&A / utilities
- **oracle** (`/oracle`, `/ask`) — interactive grounded Q&A via the researcher in oracle mode.
- **sidecar-protocol** — remote sidecar boot/health/inference (the 3-step check).
- **impact-check** — pre-edit blast-radius (spawns impact-mapper) on first EXECUTING edit.
- **dashboard-handoff** — post-edit dashboard verification.
- **pdf** (`/pdf`) — PDF/Office → Markdown via MinerU.
- **skill-audit** (`/skill-audit`) — runs the NVIDIA skillspector security gate before importing a third-party skill.
- **canvas-mode** (`/canvas-mode`) — sets the canvas-workbench mode flag.

Built-in (harness-level) skills also present: code-review, simplify, verify, run, deep-research,
loop, schedule, security-review, update-config, claude-api, init, etc.

---

## 5. Rules (`.claude/rules/`, 15 files) — the constitution

These auto-load (or load on trigger via `conditional-routing.md`) with the same authority as
CLAUDE.md. The heavy hitters:

- **conditional-routing.md** — the dispatcher: "if editing X, read rule Y first." Routes to
  every other rule and rulebook entry.
- **token-discipline.md** — *always loaded.* Anti-read-big (refuse Read >200 lines / Grep
  >50 lines without a subagent), grep-before-read, Zero-Lazy-Code (no stubs/TODO/force-unwrap),
  anti-slopsquatting (verify every dependency exists), anti-stale-training, reasoning-after-action.
- **wiring-enforcement.md** — *the signature rule.* Fail-closed doctrine R1–R10 against
  "wire-dark" code (compiles + tests pass but no production caller). Missing evidence = HARD
  FAIL. A 4-layer gate (compile-time → cross-language contract → entrypoint coverage → runtime
  trace assertion). The gate must be frozen and read-only to the run it judges.
- **production-caller-gate.md** (D-030) — the runnable, changeset-grain leg of R6/R7: every new
  `pub` fn needs a non-test caller outside its file or a `WIRE-DARK[plan-id]` marker. Checked by
  `production_caller_check.py` (Rust) / `python_wire_check.py` (Python).
- **maintainability-review.md** — 8-dimension maintainability catalog + `maintainability_check.py`
  (file-size 1000-line boundary, `as any`/unsafe escape hatches, long functions, branching density).
  Auditor-applied, never executor-self-applied.
- **tracing-test-assert.md** — `#[traced_test]` must wrap `logs_contain` in an `assert!`; bare
  calls silently pass. Checked by `traced_test_assert_check.py`.
- **agent-dispatch-preamble.md** — R-1..R-8 binding on every dispatched worker: git is
  read-only, no destructive ops, scope is a contract, STOP = handoff entry (origin: a 2026-06-11
  incident where an agent ran `git reset --hard` 8× and lost tracked edits).
- **orchestrate-protocol.md** — H-1..H-11: agent dispatch during orchestrate is non-optional;
  no Edit before the context-compiler + integration-explorer batch returns; no audit-skip; no
  memory-write-skip.
- **main-llm-role.md** — orchestrate/manage/review; don't implement what an agent can; the role-boundary list.
- **commit-cadence.md** — ONE commit per shipped unit of work, after the final audit passes.
  Commit messages in "maintainer's voice" (lowercase i, no em-dashes, no robotic prefixes, no AI
  attribution trailers).
- **handoff-protocol.md** — ONE topic-keyed file at `workflow/handoff/{topic-slug}.md`, appended
  across sessions (never dated/per-session sprawl). Living CURRENT STATUS + INDEX.
- **output-formatting.md** — deterministic output, zero filler/emoji, plain-text file authoring,
  update memory-bank after every build/test.
- **bun-not-node.md** — default JS/TS runtime is `bun`; no new node/npm/npx; rewrite npx wrappers to bunx.
- **skill-import-policy.md** — scan every external skill (whole directory) with skillspector before import.
- **visual-ui-loop.md** — schema-constrained screenshot→diff→critique loop for dashboard visual work
  (deterministic pixel diff decides where to look; VLM critique forced into pass|fail|unknown).

---

## 6. Rulebook (`.claude/rulebook/`, 18 files) — lazy-loaded depth

Loaded only on trigger (cited in `conditional-routing.md`). Deeper how-to for specific
situations:

`session-protocol` (session start / tier classification), `agent-spawning`, `risk-levels`
(LOW/MED/HIGH node classification), `source-tiers` (research grounding hierarchy),
`stub-tracking`, `bubble-up` (failure escalation), `file-discipline` (150-line authoring rule),
`folder-ownership` (multi-dev gigs), `integration-exploration` (HEAVY tier with 3+ subsystems),
`memorizing-and-audit`, `multi-session` (cross-session file locks), `coding-standards`
(Python ML / trading / eval), `codegraph-vs-map` (when to use the live codegraph MCP vs the
static elai.map grep), `design-taste` (anti-slop visual taste for Svelte), `gpt-image-prompting`,
`gpt-second-opinion` (cross-model diversity vote), `canvas-image-regen`,
`orchestrate-protocol` (rulebook copy).

---

## 7. MCP servers (`.mcp.json` + session-connected)

Configured in `F:\.ELAI_workflow\.mcp.json`:

- **local-rag** — the workhorse. Python stdio server at `C:\Users\user\.claude\rag\server.py`.
  Provides: the **FSM** (`mcp_fsm_*`), the **GRC reasoning engine** (`mcp_think`,
  `mcp_think_set_budget`, `mcp_think_extend`, `mcp_think_reset` — per-topic budgets), **RAG**
  (`mcp_search_context`, `mcp_memorize`, `mcp_ingest_file`, `mcp_forget`, `mcp_list_topics`),
  **change/build logging** (`mcp_log_change`, `mcp_report_build`, `mcp_attribute_build_errors`),
  **context** (`mcp_update_context`), and **cross-session messaging** (`mcp_session_*`:
  register/claim/release/send/wait/inbox/list).
- **codegraph-elai** — codegraph 0.9.9 (MIT), live file-watched SQLite call-graph scoped to
  `.elai_cc`. `codegraph_explore/search/callers/callees/impact/node/files/status`. Complements
  the static `elai.map`. Index at `.elai_cc/.codegraph/`.
- **canvas** — per-root canvas-workbench (full copy, port 7323). Serves a localhost code-graph
  tab off the dependency-cruiser depGraph; clicking an element hands it to the LLM via the
  UserPromptSubmit handoff hook. Tools: `canvas_open_project/read/writeback/reload_*/preview_image/ping`.
- **gpt-bridge** — SHARED single install at `D:\.apps_workflow\` (one ChatGPT/Grok account pool).
  Cross-model second opinions: `gpt_chat`, `grok_chat` (effort auto/fast/expert/heavy),
  `gemini_chat`, `claude_chat`, `perplexity_chat`, `gpt_image`, `gpt_deep_research(_status/_result)`.
  No repo access — a weak ONE-vote grounding source, never on a critical path.
- **pod-lifecycle** — RunPod GPU pod power/migration/cost (shared infra at `D:\.liteGAME_workflow\`).
  `pod_create/start/stop/terminate/list/get_state/cost_status/idle_check/...`. Mutating calls
  incur real billing; cheap reads pre-approved. Pods MUST be stopped at session end.

Session-connected (claude.ai connectors, not in `.mcp.json`): **notion**, **Gmail**,
**Google Calendar**, **Google Drive**.

---

## 8. Local RAG (`C:\Users\user\.claude\rag\`)

The `local-rag` MCP is the brain stem of the harness. It is one Python server that unifies:
- **FSM persistence** — per-session state keyed by `CLAUDE_CODE_SESSION_ID`, root resolved by
  the outermost `workflow/` marker, with PID-liveness (a single `paths.py` resolver fixed a
  cross-session FSM bug; 53/53 tests).
- **GRC (Grounded Reasoning Chain)** — built-in thinking is disabled; all reasoning goes
  through `mcp_think` with mandatory grounding citations and per-topic budgets (simple 3/600,
  moderate 5/1000, complex 8/1500). After each call the LLM echoes a blockquote
  `> **[step_type]** content *[grounding]* (call/budget, confidence)`.
- **Vector + BM25 retrieval** over ingested memory-bank, papers, and source.
- **Change recording / build attribution** feeding the memory-bank.

Note: the RAG primary can intermittently die on reranker load (Windows access violation);
recovery is clearing `primary.pid` + restart.

---

## 9. Memory-bank (`workflow/memory-bank/`, gitignored)

The project's working memory, maintained by the memory-writer agent (not the main LLM) and
updated after every build/test:

- **activeContext.md** — current state (overwrite, <80 lines).
- **changelog.md** — one entry per file modification, ≤5 lines, fixed Format: template. Grep by
  path for history.
- **progress.md**, **projectIndex.md** (topological map — grep this before reading source).
- **gotchas.md** — failure patterns, added immediately on a new one.
- **projectBrief.md** — what the project is / scope / constraints / key spec references.
- **systemPatterns.md**, **operatingManual.md**.
- Rule-set files loaded conditionally: **codeRules.md**, **tradingRules.md**, **gameRules.md**,
  **dashboardRules.md**, **applicationRules.md**, **reRules.md**.

Separately, there's an **external auto-memory** at
`C:\Users\user\.claude\projects\f---ELAI-workflow\memory\` (the `MEMORY.md` index + per-fact
files) for user/project/reference context — but it is **read-only for rules**: no coding
standards or workflow feedback go there.

---

## 10. Tools (`.claude/tools/`, ~40 scripts) — the deterministic checkers

Stdlib-only Python (+ a few `.mjs`/`.ps1`). The gate + map machinery:

- **codebase_mapper.py** → `elai.map` (the grep-optimal ~375K-line SCIP-enriched code map, all
  langs) + `elai_graph.json` (structured twin). Regen after large changes.
- **impact_radius.py**, **graph_reconcile.py** (union codegraph ↔ elai.map, flag disagreements).
- Wiring/quality gates (auditor-run, frozen): **wiring_contract_check.py**,
  **production_caller_check.py**, **python_wire_check.py**, **maintainability_check.py**,
  **traced_test_assert_check.py**, **gate_freeze_check.py**, **failure_taxonomy.py**,
  **cross_lang_diff.py**, **spec_link_check.py**, **bn_status_report.py**, **elai_map_freshness.py**.
- Trading/ML validators: **validate_execution_engine.py**, **validate_predictions.py**,
  **validate_drift.py**, **validate_ohlcv.py**, **scan_backtesting.py**, **trading_ast_scan.py**, etc.
- RAG/paper: **fetch_arxiv.py**, **extract_pdf_figures.py**, **measure_paper_tokens.py**, **query_bm25.py**.
- Security: **skill-audit.py** (skillspector wrapper), **agentskills_validator.py**.
- Canvas/hooks: **canvas-handoff-hook.mjs**, **canvas-activity-hook.ps1**, **session-rag-sync.ps1**,
  **deny-enterplanmode.ps1**, **stop_verified_lint.py**.

### The frozen gate manifest (`.claude/gate-manifest.txt`)
The single most important enforcement artifact. It lists every file that constitutes a
"gate" (the wiring rule, the checkers, the maintainability rule, and ~30 `.elai_cc` Rust
gate modules + their TOML seeds). `gate_freeze_check.py` blocks any run that modifies a file
in this set — *a run cannot rewrite its own evaluator*. Re-locking is a human, out-of-band step.

---

## 11. Hooks (`.claude/settings.json` + RAG hooks)

- **UserPromptSubmit** → `canvas-handoff-hook.mjs` injects a clicked canvas element into the next prompt.
- **PostToolUse(Edit|Write|MultiEdit|Read)** → `canvas-activity-hook.ps1` (live-activity overlay).
- **PostToolUse(Edit|Write|MultiEdit)** → `session-rag-sync.ps1` keeps one searchable RAG
  breadcrumb per session (refreshed every 5th edit; fail-open).
- **SubagentStop** → canvas activity feed.
- RAG-registered hooks (in `settings.local.json`): **PreToolUse** (path/tool gating + cross-session
  file locks), **PostToolUse** (ruff syntax, template preservation, change recording), **Stop**
  (blocks mid-workflow termination + Zero-Lazy-Code enforcement), **PreCompact** (autosaves a
  SESSION_HANDOFF FSM snapshot before truncation).

`settings.local.json` holds permissions + dev-personal hook paths; `settings.json` holds the
shared, cross-machine-portable hook config.

---

## 12. Papers & ingested source (`.claude/papers/`, `docs/.source/`)

- **`.claude/papers/`** — ~100 ingested research papers (arXiv HTML/PDF + extracted text) that
  ground design decisions: DGM/AlphaEvolve self-improvement, reward-hacking, wiring/verification,
  speculative decoding (EAGLE3, Lookahead, KIVI), memory (Mem0, Zep-Graphiti), prompt evolution
  (GEPA, TextGrad, OPRO), and more. Ingested via the paper-ingestor agent / `/pdf` skill (MinerU).
- **`docs/.source/`** — cloned competitor/reference repos (claude-code, swarm, lazycodex,
  AlphaCodium, SocratiCode, etc.) used by the paper-to-plan "steal" pipeline.
- **`docs/specs/`** — the `.elai_cc` product spec (implementation guide, per-phase step files,
  architecture, crate specs). Per-phase step files are huge; only summaries
  (`phase-index-*.txt`) are read each session.

---

## 13. How a real task flows (end to end)

1. User states a task (or `!`-prefix to skip transform).
2. **prompt-transform** reformats it, sets the `mcp_think` budget.
3. Grounding sweep — claims about external behavior classified GROUNDED / VERIFY / INGEST.
4. **planning-workflow** + tier selection. A decision doc is written before EXECUTING.
5. FSM: PLANNING → FEASIBILITY_PROBE → TESTING_FIRST. The main LLM dispatches
   **context-compiler** + **integration-explorer(s)** in parallel (cap 3) — and may NOT edit
   plan-scope files until they return.
6. EXECUTING — one change at a time. New `pub` fns must be wired to a production seam in the
   same changeset (vertical slice). impact-check runs on the first edit.
7. AUDITING — the **auditor** (and only the auditor) runs tests + the frozen wiring /
   maintainability / production-caller / tracing-assert gates off-agent. Returns PASS or BLOCK.
8. If BLOCK → fixer / anti-spiral if stalling. If PASS → MEMORIZING.
9. MEMORIZING — the **memory-writer** updates the memory-bank via RAG-indexed writes.
10. COMPLETE → IDLE. ONE commit at the end (maintainer voice, no AI trailers), pushed only when
    asked. `.elai_cc` is the only tracked tree.

---

## 14. The governing principles (the "why")

- **Grounded-by-default** — every claim cited to tests > code-just-read > docs > papers >
  third-party. "I think X" is never enough.
- **Fail-closed** — missing evidence is a FAIL, not a SKIP/WARN. Gates are frozen against the
  run they judge.
- **Off-agent verification** — an agent never grades its own output; the auditor runs in a clean
  context. (Origin: agents reviewing their own work rubber-stamp ~100%.)
- **Context economy via delegation** — sub-agents have throwaway contexts, so dispatching
  *saves* the main context, never costs it.
- **No wire-dark code** — "compiles + passes + is wired + not a stub + is maintainable" are four
  orthogonal, separately-enforced axes.
- **Anti-sycophancy, two-approaches-minimum, abstention-over-guessing.**
