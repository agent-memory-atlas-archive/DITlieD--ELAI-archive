# ACI Tool-Shape Audit (E-21 task 4)

Audit date: 2026-05-28
Plan: docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md
Reference: arXiv:2405.15793 (SWE-agent, NeurIPS 2024) section 4 ACI Principles

Scope: the registered tool surface exposed to the main LLM in this codebase as of 2026-05-28. Covers the Claude Code built-in tools (Read, Grep, Edit, Write, Glob, Bash, PowerShell, WebFetch, WebSearch) plus the local-rag MCP tool family (mcp_think, mcp_search_context, mcp_memorize, mcp_fsm_*, mcp_ingest_file, mcp_list_topics, mcp_report_build) and the pod-lifecycle MCP family (pod_*). Tauri-only dashboard commands and Cargo build tools are out of scope here — grade them in the dashboard-audit follow-up.

Grading rubric per SWE-agent ACI Principles, axes (a) (b) (c):
- a (LM-terse): output is LM-shaped (terse, no decorative ANSI, no human-only progress bars)
- b (Structured): tool emits structured fields (JSON / TSV / fixed schema) that the LM can consume without regex
- c (Scoped default): the default scope is narrow (head_limit set / glob narrowed / depth bounded), not "return everything"

Grades: A exemplary, B acceptable, C needs attention, D failing.

## Grades

| Tool | LM-terse (a) | Structured (b) | Scoped default (c) | Composite | Notes |
|---|---|---|---|---|---|
| Bash | B | C | A | 6/9 | output is raw stdout/stderr (LM-friendly); structured fields absent (regex required to parse); default cwd is project root and timeout=120s |
| Edit | A | A | A | 9/9 | terse "updated" confirmation; structured failure modes; default is single-occurrence (replace_all opt-in) |
| Glob | A | A | A | 9/9 | one path per line; results are file paths only; default sorts by mtime |
| Grep | A | B | A | 8/9 | output is grep-format; files_with_matches mode is LM-ideal; head_limit defaults to 250 |
| PowerShell | B | C | A | 6/9 | windows-only counterpart to Bash; same LM-terse profile; structured fields absent |
| Read | A | A | A | 9/9 | line-numbered output; offset+limit gates wide reads; default limit=2000 lines |
| WebFetch | C | C | C | 3/9 | full-page body returned (unbounded by default); structured fields absent (prose only); no per-domain scope |
| WebSearch | B | C | B | 5/9 | search results are bullet list; result count cap is implicit not enforceable; no allow-list default |
| Write | A | A | A | 9/9 | terse "wrote N bytes"; structured failure modes; requires prior Read for overwrite (sane default) |
| mcp_fsm_get_state | A | A | A | 9/9 | one-line state output; structured allowed_transitions; default scope is the active session FSM |
| mcp_fsm_transition | A | A | A | 9/9 | transition succeeded/failed structured; rejects illegal transitions with reason |
| mcp_ingest_file | A | B | A | 8/9 | one-line "ingested N chunks" confirmation; structured chunk count; defaults to one file at a time |
| mcp_memorize | A | A | A | 9/9 | "stored to topic <name>" terse; topic is required (no implicit "stuff anywhere"); namespacing forced |
| mcp_search_context | A | B | B | 7/9 | terse results; structured topic+score per result; top_k defaults to 5 but is not always passed explicitly |
| mcp_think | A | A | A | 9/9 | "Recorded [step_type] (n/N)" one-liner; topic+budget tracked per call; budget bounded at 12 calls/2000 chars |
| mcp_report_build | A | A | A | 9/9 | pass/fail counts in structured form; auto-aggregates by build_id |
| pod_create | B | A | B | 7/9 | full pod spec output; structured fields available; defaults to a billable pod (manual stop required) |
| pod_get_state | A | A | A | 9/9 | terse status block; structured fields; scoped to one pod |
| pod_list | B | A | C | 6/9 | tabular output; structured fields per row; default returns ALL pods including stopped (no filter) |
| pod_stop | A | A | A | 9/9 | terse "stopped <id>" confirmation; idempotent; scoped to one pod |

## Distribution

Cell-level (3 cells per tool x 20 tools = 60 cells total):

- A: 41 cells
- B: 12 cells
- C: 7 cells
- D: 0 cells

No tool grades D on any axis in this audit. The C-grade cells cluster on web-facing tools (WebFetch on all three axes; WebSearch on (b); pod_list on (c); Bash and PowerShell on (b)).

## Fixer queue (lowest composite first)

- WebFetch (3/9): full-page body returned by default — needs head_limit-style truncation + a per-host allow-list + structured field emission (URL + extracted-text + length).
- WebSearch (5/9): result count cap is implicit; structured fields could be added (title + url + summary per row, JSON mode).
- Bash (6/9): default output is raw stdout/stderr — a structured-output mode (exit_code + stdout_bytes + stderr_bytes + duration_ms JSON envelope) would help. Already scoped well (default timeout=120s, default cwd).
- pod_list (6/9): default filter should be "running" only, not all pods including stopped (the stopped pods inflate context with no actionable signal).
- PowerShell (6/9): mirrors Bash — same structured-output gap; same recommendation.

## Follow-ups

- Per D-B-7 (workflow/reasoning/E-21-decisions.md) this artifact is one-shot prose, not a recurring deterministic check. Re-audit when new tools are registered or the existing tools change their default behavior.
- Tools rated C or D on any axis are candidates for a follow-up E-N or B-N refactor. The two highest-value refactors (per Pareto): WebFetch (largest blast radius on context budget) and a Bash/PowerShell --json envelope mode (touches every shell-invoking workflow).
- This audit covers the tool surface AS REGISTERED on 2026-05-28. Tools added after that date (and tools that change their scope-default) are NOT graded here.
- The audit is grade-by-author, not auditor-agent-graded. The plan's Open Question on auditor-driven grading remains open; deferring to a follow-on plan once test-attributer can score axes (b) and (c) deterministically from tool docs.
