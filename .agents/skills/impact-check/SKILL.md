---
name: impact-check
description: Pre-edit blast-radius safety. Triggers on the FIRST Write or Edit of an EXECUTING phase that touches files under F:\.ELAI_workflow\.elai_cc\crates\, F:\.ELAI_workflow\.elai_cc\python\, or any cross-language pair location (sidecar/dashboard). Spawns impact-mapper agent. Caches result so subsequent edits in the same session don't re-run unless files change.
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

Impact-check pre-edit safety

Trigger: any of these
- First Write/Edit of session targeting F:\.ELAI_workflow\.elai_cc\crates\**\*.rs
- First Write/Edit targeting F:\.ELAI_workflow\.elai_cc\python\**\*.py
- First Write/Edit targeting F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\**
- After any cargo refactor that touches >5 files

Skip: edits to memory-bank, workflow/, .Codex/ tooling, or test files only.

PROTOCOL

1. Check cache. If workflow/sessions/{session_id}/blast-radius.md exists AND its mtime is newer than the newest file in the planned-edit list, use the cached version. Skip to step 5.

2. Determine planned files. Get the list of files the main model intends to edit in this batch. (If unclear, ask user; default to the last 5 files mentioned in the active task.)

3. Determine planned symbols (optional, improves precision). For each planned file, peek the public API and identify which symbols (function names, struct names, trait names) are being changed. If unsure, omit.

4. Spawn impact-mapper agent (.Codex/agents/impact-mapper.md) with:
     files_planned: <comma-separated absolute paths>
     symbols_planned: <comma-separated symbols, optional>
     session_id: <session_id>
     depth: 2
   Wait for completion. The agent writes workflow/sessions/{session_id}/blast-radius.md.

5. Read blast-radius.md. Apply gates:

   GATE A — Cross-language risk
   If CROSS_LANG_RISK lists any pair (Rust↔Python, Tauri↔Svelte, etc.):
     a. Check whether all paired files are in the planned-edits list.
     b. If only one side is queued: tell user "Cross-language pair detected. Edit <other-side> in the same EXECUTING pass, or annotate with `// cross-lang-skip: <pair>` to opt out."
     c. Block until user confirms (cap 1 round-trip).

   GATE B — Wide blast radius without WIRING_CONTRACT update
   If consumers count >= 3 cross-crate AND the active B-N file's WIRING_CONTRACT block was not updated this session:
     Tell user: "Edits affect <N> consumers across <M> crates. Recommend updating WIRING_CONTRACT.consumes_symbols in B-{N}. Continue without update?"
   Don't block — just warn.

   GATE C — Low SCIP confidence
   If overall SCIP confidence is LOW (<0.3): emit a note suggesting that callers may exist that elai.map missed (macro-generated code, conditional compilation). Recommend: regen elai.map first via the script in AGENTS.md.
   Don't block.

   GATE D — Spec-known monolith touch
   If any planned file is in the auto-monolith list (task_runner.rs, live_cli.rs, pipeline.rs, dashboard-v2/src-tauri/src/lib.rs, python/elai_inference/server.py): emit a note that this is a known hub; downstream impact may be larger than computed.
   Don't block.

6. If user proceeds: write workflow/sessions/{session_id}/blast-radius-acknowledged.flag to skip GATE A on subsequent edits in same session.

CACHE INVALIDATION
- Invalidate when elai.map mtime changes (regen happened).
- Invalidate when blast-radius.md is older than 1 hour.
- Invalidate when planned-files changes (new files added).

INTEGRATION WITH B-PLAN-RUNNER
The b-plan-runner skill calls this skill at step 7. After approval, b-plan-runner proceeds to EXECUTING.
