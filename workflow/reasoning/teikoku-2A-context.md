TEIKOKU PHASE-2 (MEMORY) — P2-A CONTEXT PACKAGE
task_id: TEIKOKU-P2-A-context
scope: master-plan 2.1 (per-project append-only event log) + 2.2 (RAG indexer, grep+semantic, queryable like codegraph) + 2.3 (global cross-project index) as ONE cycle
goal (D-08): events + index ARE the memory; markdown memory-bank retired/migrated; codegraph owns CODE structure, this owns DECISIONS/STATE/PROSE
author: P2-A context-compiler (controller-redirected for TEIKOKU), 2026-06-20
grounding rule: every claim below carries a file:line or a captured-command result. UNVERIFIED items are labelled. WSL distro = teikoku (Ubuntu 24.04, ext4 on F:). Read-only run; this file is the only write.

================================================================================
SECTION 0 — THE THREE ARCHITECTURE FORKS (grounded recommendation each; OWNER decides)
================================================================================

FORK 1 — RUNTIME/LANGUAGE + LOCATION of TEIKOKU's memory service
Options: (a) GENERALIZE the conductor's Python+SQLite control.db event store; (b) port/generalize the local-rag server.py into a WSL TEIKOKU instance; (c) a fresh lean WSL service; (d) bridge to the Windows local-rag.
RECOMMENDATION: a HYBRID of (a)+(b), NOT a single choice — they are two different layers, both already exist, both align with N-02 (generalize/wrap, do not rebuild).
  - The EVENT LOG (2.1) = GENERALIZE the conductor control.db `events` table (option a). It is already an append-only typed event store, already in the build, already the substrate the 0.5 slice wrote SLICE_COMPLETE to (3479 rows live; verified `events: 3479` row count). It already has the append-only invariant enforced by DB triggers (`events_no_update` / `events_no_delete` RAISE(ABORT,'events are append-only') — captured from `.schema`). The append/query API is `conductor/conductor_db.py::ControlDB.append_event(event_type, task_id, payload)` + `conductor/events.py::iter_events(after_seq)` + `ControlDB.recent_events(limit)`. Generalizing this is a ~1-table extension (add project_id + event scoping), NOT a new store. Grounding: control.db .schema captured; conductor_db.py read in full; events.py read in full.
  - The RAG INDEX (2.2/2.3) = a WSL TEIKOKU local-rag INSTANCE patterned on server.py (option b), reusing its hybrid-search code, NOT the shared Windows one. The shared Windows local-rag (C:\Users\user\.claude\rag\server.py) is MACHINE-GLOBAL infra used by ELAI/apps/liteGAME sessions RIGHT NOW (registered in F:\.ELAI_workflow\.mcp.json:13-17 as the live `local-rag` MCP). TEIKOKU must NOT fork/modify it (the boot prompt + scope say so). It needs its OWN instance scoped to the teikoku distro. Reuse the proven hybrid-search algorithm (nomic + BM25 + BGE-reranker, server.py:222-338) — do not re-invent it.
WHY NOT (c) fresh service: rebuilds two things that already work and are battle-tested (the append-only event store + the hybrid-search stack); violates N-02.
WHY NOT (d) bridge-to-Windows-only: couples TEIKOKU's memory to a shared global service it must not mutate, breaks D-12 WSL isolation, and the localhost-bridge add-latency-per-query is gratuitous when the model files are already WSL-reachable (verified below).

FORK 2 — SEMANTIC INDEX viability (grep deterministic-always vs semantic needs an embedding model that crashed on Windows)
RECOMMENDATION: GREP-PRIMARY + SEMANTIC-OPTIONAL, and semantic is viable in this WSL.
  - GREP leg = always works: the event log is SQLite; "grep" over prose = SQL LIKE / FTS5 over payload text + a literal ripgrep over the handoff files. Zero model dependency, zero crash surface. This is the hard floor: memory query NEVER hard-fails on a missing model.
  - SEMANTIC leg = OPTIONAL, loads if the embedding stack is present; degrade-to-grep if not (mirror server.py's own `if not _init_server(): return "...try again"` posture, server.py:223).
  - WSL VIABILITY (the load-bearing UNKNOWN — assessed by ACTUAL checks, not assumed):
    * Both model files are ALREADY CACHED and WSL-reachable: `/mnt/c/Users/user/.cache/huggingface/hub/models--nomic-ai--nomic-embed-text-v1.5` (523M, verified `du -sh`) + `models--nomic-bert-2048` + `models--BAAI--bge-reranker-v2-m3` (verified `ls`). No re-download needed; local_files_only=True is how server.py loads them (server.py:62,72).
    * The embedding model is `nomic-ai/nomic-embed-text-v1.5` (server.py:31, EMBED_DIM=768 server.py:34); reranker `BAAI/bge-reranker-v2-m3` (server.py:32).
    * The compute footprint is PURE CPU: the existing Windows rag venv runs `torch 2.11.0+cpu` (verified `cat .venv/.../torch/version.py` -> `__version__ = '2.11.0+cpu'`; no nvidia/triton site-packages). So semantic RAG needs NO GPU — N-01-clean, and the GPU-VRAM constraints in memory do not apply.
    * The CRASH HISTORY was Windows-specific (memory project_rag_reranker_crash = "Windows access violation" on reranker load). That is a Windows-DLL/access-violation class fault; moving to Linux/WSL is the standard fix for exactly that class. NOT yet empirically re-confirmed on WSL (see UNKNOWN-1) because the teikoku distro has no pip yet (`python3 -m pip` -> "No module named pip"; `python3 -m ensurepip` -> missing; apt python3-venv is `un` not installed — captured). Installing the stack is the EXECUTOR's job (a state mutation out of this read-only run's scope).
  - DESIGN CONSEQUENCE: the memory tool returns grep results unconditionally and ENRICHES with semantic when the model loaded; a failed/absent embedding model is a WARN that downgrades to grep-only, never a hard memory failure (wiring-enforcement R1-style posture, but grep is the always-available evidence floor so "missing model" != "missing memory").
  - DEP STACK the executor installs (from C:\Users\user\.claude\rag\requirements.txt, read in full; anti-slopsquat C-05 — executor verifies each at edit time): mcp>=1.26.0, sentence-transformers==5.3.0, qdrant-client==1.17.1, rank-bm25==0.2.2, nomic>=3.9.0, einops. Plus torch CPU wheel (==2.11.0+cpu proven flavor). Install path needs apt python3-venv (or uv) FIRST since the distro has no ensurepip.

FORK 3 — EXPOSURE: how the LLM + dashboard query memory
RECOMMENDATION: BOTH, mirroring what each consumer already speaks.
  - LLM consumer = an MCP stdio tool, exactly the codegraph template. Verified template: codegraph-elai is registered `"type":"stdio","command":"node",args:[...,"serve","--mcp","-p","<project>"]` (.mcp.json:3-12) and local-rag is `command:<venv python>, args:[server.py]` MCP stdio (.mcp.json:13-17). Codegraph's own MCP instructions (in this session's MCP-server-instructions block) confirm the model: per-project SQLite knowledge graph, sub-ms reads, file-watcher lag ~1s, tools codegraph_search/explore/callers/callees/impact/node/files/status. TEIKOKU's memory tool = the SAME shape: an MCP stdio server exposing e.g. memory_search (grep+semantic), memory_recent, memory_timeline, memory_log (append an event). This is the "queryable like codegraph for prose" the master plan D-08 names.
  - DASHBOARD consumer = the localhost HTTP + one-shot-CLI dual transport already proven for the 0.5 slice. Verified template: conductor/slice_api.py serves a ThreadingHTTPServer on 127.0.0.1:8787 (GET /slice/latest, POST /slice/run, GET /harness/manifest) AND a one-shot CLI mode (`python3 slice_api.py latest|run`) so the Tauri NATIVE command shells the CLI while the vite-dev web build fetch()es the HTTP — BOTH read the SAME control.db, neither fabricates (slice_api.py docstring + main(), read in full). TEIKOKU memory surface (2.5) reuses this exact dual seam: add GET /memory/timeline?project=X and GET /memory/global to slice_api.py (or a sibling memory_api.py with the same shape) + a one-shot CLI, both reading the generalized event store + index.
  - WHY BOTH not one: the LLM speaks MCP (codegraph/local-rag are both MCP today); the Tauri+web dashboard speaks the slice_api dual transport today. Forcing either onto the other's protocol is gratuitous; both seams already exist and are proven by the shipped 0.5/1.x surfaces.

================================================================================
SECTION 1 — GENERALIZED EVENT-LOG SCHEMA (2.1)
================================================================================

EXISTING CONDUCTOR `events` TABLE (the seed; verified from control.db .schema):
  events(event_seq INTEGER PK AUTOINCREMENT, event_id TEXT UNIQUE, task_id TEXT, event_type TEXT NOT NULL, payload_json TEXT NOT NULL, created_at TEXT NOT NULL)
  + INDEX idx_events_task ON events(task_id, event_seq)
  + TRIGGER events_no_update / events_no_delete -> append-only enforced at the DB layer (RAISE ABORT). KEEP THESE; they are the append-only guarantee 2.1 needs.
  API: ControlDB.append_event(event_type, task_id=None, payload=None) -> event_id (conductor_db.py); iter_events(db_path, after_seq=0) (events.py); recent_events(limit=50) (conductor_db.py). Payload is redact_json()'d then json_dumps()'d before insert (conductor_db.py append_event body — redaction is already wired, redaction.py).
  LIVE: 3479 events, schema_version '2' (schema_meta), WAL journal mode (conductor_db.py init()).

ELAI SSB SHAPE (the CONCEPT to generalize — do NOT port Rust; verified from ssb/schema.rs:3294 + ssb/store.rs):
  ssb(seq INTEGER PK AUTOINCREMENT, project_id TEXT NOT NULL, session_id TEXT NOT NULL, agent_id TEXT NOT NULL, superstep INTEGER, action_type TEXT CHECK(action_type IN (...~250 variants...)), payload BLOB, created_at)  [ssb/schema.rs:3294-3305]
  + ssb_files(seq, file_path) side table for file-scoped events [schema.rs:5477]
  typed query API (ssb/store.rs:252-273): query_by_session / query_by_file / query_by_type(action_type) / query_by_superstep / query_by_seq / query_reasoning_trace(agent_id) / query_phase_complete(phase_id) / query_isolation_metrics + query_session_stream_asc (store.rs:476, "SELECT ... FROM ssb WHERE session_id=?1 ORDER BY seq ASC").
  write API: Ssb::write(SsbWriteRequest)->seq, write_payload(...) (store.rs:407,423).
  EVENT-DRIVEN WAKE SEED (relevant to D-07/P3.5): Ssb::subscribe()->broadcast::Receiver<SsbEntry> (store.rs:447) — a live pub/sub on appends. This is the in-process primitive a "session woken by an event" can build on; note it for the P3 handoff, NOT a P2 deliverable.
  SCHEMA_VERSION is DERIVED: SSB_ACTION_TYPES registry len + epoch (ssb.rs:1503-1508); TOTAL_VARIANTS auto-derives. THIS IS A KEY LESSON: ELAI's pain is the hand-counted-counter collisions — the generalization should NOT copy the giant CHECK(action_type IN (...)) enum; an open TEXT event_type (like the conductor already uses) avoids the entire SCHEMA_VERSION/TOTAL_VARIANTS treadmill. Recommend: open event_type TEXT + a documented vocabulary, NOT a frozen CHECK enum.

GENERALIZED PER-PROJECT EVENT-LOG SCHEMA (the 2.1 deliverable; extend the conductor table, do not replace):
  events(
    event_seq    INTEGER PK AUTOINCREMENT,   -- keep
    event_id     TEXT UNIQUE NOT NULL,        -- keep (uuid4, conductor_db.py)
    project_id   TEXT NOT NULL,               -- ADD (the generalization; ELAI ssb already has it, conductor lacks it). Index it.
    session_id   TEXT,                        -- ADD (ELAI ssb has it; carries the cross-session-comms join, D-07)
    task_id      TEXT,                         -- keep (conductor join to tasks)
    event_type   TEXT NOT NULL,               -- keep as OPEN TEXT (NOT a CHECK enum — avoid ELAI's counter treadmill)
    payload_json TEXT NOT NULL,               -- keep (redacted JSON)
    created_at   TEXT NOT NULL                -- keep (iso_utc)
  ) + idx_events_project(project_id, event_seq) + keep idx_events_task + KEEP the two append-only triggers.
  EVENT-TYPE VOCABULARY for the four D-08 categories (grounded against what BOTH stores already emit, so this is naming-an-existing-shape not inventing):
    DECISION  -> decision (cf conductor TASK_STATUS_CHANGED w/ reason; ELAI ssb `change_decision` verified present in schema.rs grep, `gate_decision`, `plan_decision`)
    CHANGE    -> change/file_write/diff_applied (ELAI ssb `file_write`, `file_read`; conductor `TASK_IMPORTED`/`INTEGRATION_*`)
    STATE     -> state/status_changed/phase (conductor `TASK_STATUS_CHANGED`/`TASK_IMPORTED` verified driving replay_task_statuses in events.py; ELAI ssb `phase_transition`/`fsm_transition`/`task_advanced` verified present)
    SESSION-MESSAGE -> session_message (ELAI ssb `chat_message`/`chat_response` verified present; the local-rag session inbox model server.py:1120-1187 is the cross-session-comms analog)
  APPEND API (generalize ControlDB.append_event): append_event(project_id, event_type, session_id=None, task_id=None, payload=None) -> event_id. Reuse redact_json + json_dumps + the BEGIN IMMEDIATE transaction (conductor_db.py transaction()).
  QUERY API (generalize, drawing the typed-query set from ELAI ssb/store.rs): iter_events(after_seq, project_id=None), recent_events(limit, project_id=None), query_by_type(event_type, project_id), query_by_session(session_id), query_by_task(task_id). The conductor already has iter_events/recent_events; add project scoping + by_type/by_session.

================================================================================
SECTION 2 — RAG INDEXER DESIGN (2.2): grep-primary + semantic-optional
================================================================================

SOURCE OF TRUTH = the event log (Section 1) + the handoff files (workflow/handoff/*, which D-08 + the 2.4 rework make the canonical changelog/memory unit). The indexer indexes BOTH: event payload prose AND handoff prose.
GREP LEG (always-on, no model):
  - over events: SQLite query on payload_json (LIKE, or an FTS5 virtual table over payload text for speed) + event_type/project filters. Deterministic, no model.
  - over handoffs: literal ripgrep over workflow/handoff/*.md (the conductor pattern of reading text files directly, cf slice_api reading events; handoffs are plain md).
SEMANTIC LEG (optional, degrade-to-grep on absence):
  - REUSE the proven hybrid pipeline from server.py:222-338 verbatim-in-spirit (do not re-derive): embed query with nomic (server.py:233-234) -> Qdrant vector top-K (BM25_TOP_K=20, server.py:244-250) UNION per-project BM25 candidates (rank-bm25, server.py:260-297) -> BGE cross-encoder rerank (server.py:302-318) -> recency blend (final = rerank_norm*0.85 + recency*0.15, server.py:317) -> CONFIDENCE_THRESHOLD=0.3 LOW-CONFIDENCE flag (server.py:323,336).
  - PROJECT SCOPING already exists: server.py uses a Qdrant FieldCondition match on payload "project" (server.py:239-242) AND a per-project BM25 index (_bm25_per_project, server.py:51-52,263) to avoid cross-project IDF pollution. This is exactly the per-project-vs-global split 2.3 needs — reuse it.
INDEX STORAGE:
  - vectors: Qdrant LOCAL mode (server.py:30 DB_PATH=data/qdrant, server.py:91 QdrantClient(path=...)), collection "knowledge", 768-dim COSINE (server.py:33,34,108-110). Per the WSL instance, put it under ~/teikoku/.../data/qdrant (NOT the shared Windows one).
  - BM25: built in-memory from a Qdrant scroll at load (server.py:113-120 _build_bm25_index), per-project buckets.
  - chunking: CHUNK_WORDS=350 / overlap 40 (server.py:36-37); embed batch 64 (server.py:38).
  - LOCK CAVEAT (verified, must design around): Qdrant local mode is single-writer; server.py has a `_qdrant_locked` fallback that returns None if "lock"/"already" in the open error (server.py:96-98) and an IPC primary/secondary split (rag_ipc.py, _is_secondary proxy_call, server.py:225-227). The TEIKOKU instance is its own DB so no contention with the Windows one, but if BOTH the MCP tool process AND the dashboard HTTP process open the same Qdrant dir concurrently they will lock-contend — design: ONE writer process, others proxy to it (reuse rag_ipc.py's primary/secondary pattern) OR the dashboard reads via the MCP/CLI seam not by opening Qdrant directly.
EXPOSED AS A TOOL (the codegraph template): memory_search(query, project="", n_results=5) returning the same "[score:.. source:.. project:.. tags:..]\n<doc>" line format server.py:331-333 already produces. Plus memory_recent / memory_timeline / memory_log (append).

================================================================================
SECTION 3 — GLOBAL CROSS-PROJECT INDEX (2.3): aggregates + feeds session comms + dashboard
================================================================================

The per-project split is ALREADY the mechanism — global = "no project filter":
  - EVENTS: the generalized events table is ONE table keyed by project_id; global = SELECT across all project_id, per-project = WHERE project_id=?. No second store.
  - SEMANTIC: server.py's project filter is OPTIONAL (server.py:238 qdrant_filter=None when no project; the global BM25 _bm25_index/_bm25_ids server.py:265-267 is the all-project index that already coexists with _bm25_per_project). Global query = call _hybrid_search with project="" -> uses the global BM25 + unfiltered Qdrant. This is built; 2.3 just exposes it.
  - SESSION COMMS feed (D-07 hub): the global event stream filtered to event_type='session_message' (Section 1) IS the cross-session mailbox content. The shared append-only log + the global index = the "HUB" D-07 names (one log, no ad-hoc peer DMs). The local-rag session registry (server.py:954-1213: register/claim/list/release/send/inbox/wait, with mcp_session_wait parking a session for event-driven resume server.py:1190-1213) is the EXISTING comms primitive; 2.3 feeds it from the global event log instead of the ad-hoc state.json inbox. NOTE the session inbox today is a JSON blob in state.json (server.py:1145 setdefault("inbox")), not the event log — unifying it onto the global event log is the 2.3 + D-07 synergy (flag for P3/P4 where the hub is actually built; 2.3's job is to make the global index FEED it).
  - DASHBOARD feed: the global timeline = recent_events(limit) with no project filter, served via the slice_api dual transport (Section 0 fork 3).

================================================================================
SECTION 4 — EXPOSURE SEAM (the tool/API the LLM + dashboard query)
================================================================================

LLM (MCP stdio, codegraph template — verified .mcp.json:3-17):
  a new MCP server (python, FastMCP — server.py:350-352 uses `from mcp.server.fastmcp import FastMCP; mcp=FastMCP("local-rag")`; same dep already in requirements) registered in the TEIKOKU harness .mcp.json as e.g. "teikoku-memory", command=<wsl venv python>, args=[memory_server.py]. Tools: memory_search / memory_recent / memory_timeline / memory_log (mirror @mcp.tool() decorators server.py:355,376 etc).
DASHBOARD (HTTP + one-shot CLI dual transport — verified slice_api.py):
  extend conductor/slice_api.py (or sibling) with GET /memory/timeline?project= and GET /memory/global + a one-shot CLI (`python3 memory_api.py timeline <project>`), both reading the generalized events table via iter_events/recent_events + the index. CORS already set (slice_api.py _send adds Access-Control-Allow-Origin '*'). Tauri native command shells the CLI; vite-dev web fetch()es the HTTP — the proven 0.5 pattern.
APPEND seam (who writes events): the 2.4 reworked memory-writer ROLE (Section 5) + the conductor itself (it already append_event's on every task transition — keep). NOT the main LLM (main-llm-role: memory is not main-LLM-authored).

================================================================================
SECTION 5 — 2.4 MEMORY-WRITER REWORK SEAM + 2.5 DASHBOARD DATA SHAPE (clean handoff to P2-B / P2-C)
================================================================================

2.4 (P2-B) — memory-writer ROLE rework (the current role, for context, from F:\.ELAI_workflow\.claude\agents\memory-writer.md, read in full):
  TODAY: an agent (model:haiku) that in MEMORIZING sub-states (READ_TEMPLATES->WRITE_UPDATES->MEMORIZE_RAG) writes the 6 markdown memory-bank files (changelog/activeContext/progress/projectIndex/systemPatterns/gotchas) via mcp_log_change/mcp_update_context/mcp_report_build/Edit, then mcp_memorize LAST. It is template-driven hand-authoring of markdown.
  D-08 REWORK (2.4's job, NOT P2-A's — but P2-A must leave the seam): the role STOPS hand-authoring markdown and instead AUTO-INDEXES handoff files (workflow/handoff/*) into the new RAG (grep+semantic). Handoffs ARE the canonical memory unit. So P2-A's indexer (Section 2) MUST accept "index this handoff file" as a first-class input (a memory_log/index-file entrypoint over a path), and the event log must accept the changelog-shaped events the writer used to put in markdown.
  CONTRACT P2-A OWES 2.4: (1) an append entrypoint append_event(project_id, event_type, payload) the reworked writer calls instead of Edit-ing changelog.md; (2) an index-handoff entrypoint that chunks+embeds a handoff md path into the same index memory_search reads; (3) the event_type vocabulary (Section 1) the writer emits. C-02 NOTE: for the TEIKOKU BUILD ITSELF there is NO memory-writer agent (owner override) — the controller appends to teikoku-build.md and the RAG auto-indexes it; 2.4 is about TEIKOKU's RUNTIME role for consumer projects.
  AUDIT-FLAGGED touch set (~5 core artifacts, per master plan 2.4): memorizing-and-audit rulebook + output-formatting rule reference the markdown memory-bank; they get updated to point at the event-log/RAG. (P2-B scope; P2-A only needs to not contradict it.)

2.5 (P2-C) — dashboard memory/timeline surface DATA SHAPE (what P2-A must make queryable):
  per-project timeline row = {event_seq, event_id, project_id, session_id, task_id, event_type, payload(json), created_at} (exactly the generalized events columns, Section 1) — this is what recent_events already returns (dict per row, conductor_db.py recent_events). The surface renders these as a timeline (event_type as the square status mark per DESIGN-TEIKOKU C-07; payload behind a click-`?`).
  global view = same rows, no project filter (Section 3).
  query box = memory_search results in the "[score:.. source:.. project:.. tags:..]\n<doc>" shape (server.py:331).
  served via the slice_api dual transport (Section 4). DESIGN: SIGNAL language, near-black + ONE dark-teal accent, square status marks not circles/checks, full state coverage (empty/loading/error-inversion/offline) per DESIGN-TEIKOKU.md:109 §10 + master plan C-01/C-07. ONE canonical token set + shared components (no per-screen copy — cross-screen consistency check at audit).

================================================================================
SECTION 6 — AUDIT CRITERIA for P2-A (the master-plan 2.x "done", made concrete + off-agent)
================================================================================

The auditor (Opus, sole gate runner) must verify END-TO-END with its OWN commands, missing evidence = FAIL (C-04/C-06, wiring-enforcement R1/R4 — drive the REAL entrypoint, not a unit test):
  AC-1 EVENT APPEND (2.1): append a REAL event via the production append entrypoint (the MCP memory_log tool OR the generalized append_event called through a real seam, NOT a test shim) for a chosen project_id; then verify the row landed by an INDEPENDENT read (SELECT over the events table / iter_events), event_seq monotonic, append-only triggers present (the two triggers in .schema). The append-only invariant is provable: an attempted UPDATE/DELETE must RAISE ABORT.
  AC-2 INDEX GREP (2.2): query the just-appended event's prose via the grep leg (memory_search or the SQL/ripgrep path) with NO embedding model loaded, and retrieve it. Grep MUST work model-free (the always-available floor).
  AC-3 INDEX SEMANTIC (2.2, if model loaded): with the embedding stack installed, memory_search by MEANING (a paraphrase that shares no exact token with the event) retrieves it via the nomic+rerank path. If the model could not load in WSL, this AC degrades to a documented WARN (not a silent pass) AND grep (AC-2) must still pass — semantic-optional, grep-floor.
  AC-4 GLOBAL AGGREGATION (2.3): append events for >1 distinct project_id; a global query (no project filter) returns events from BOTH; a per-project query returns only that project's. Cross-project isolation (per-project BM25 / project filter) verified.
  AC-5 EXPOSURE (Section 4): the MCP tool answers an LLM-style query end-to-end (real MCP call, not a function call) AND the dashboard HTTP/CLI seam returns the timeline (real GET /memory/timeline or CLI), both reading the SAME store (no fabrication — the slice_api anti-fabrication posture).
  AC-6 NO-REGRESSION: the conductor's existing 3479 events + slice_api /slice/latest still work after the events-table generalization (the table is shared — verify the SLICE_COMPLETE read path slice_api.py latest_slice() still returns).
  AC-7 (C-05 anti-slopsquat): every dep added (requirements.txt set) verified against its registry at edit time; torch is the CPU wheel (2.11.0+cpu), no GPU dep (N-01).
  R10 INTEGRATION SMOKE (since P2-A is multi-unit 2.1+2.2+2.3): an independent end-to-end smoke driving the REAL entrypoints on a HARD input — append events across 2 projects + a session_message event, then grep AND semantic retrieve, then global aggregate, then the dashboard timeline shows them — run as ONE flow, not three per-unit happy paths.

================================================================================
SECTION 7 — PRODUCTION-HOOK SEAMS (D-030 / R-8, for the executor)
================================================================================

New public symbols this sub-phase ships must be wired to a real entrypoint in the SAME changeset (these are .py, so python_wire_check.py is the checker):
  - append_event(project_id, event_type, ...) -> wired into: the MCP memory_log tool (the LLM seam) AND the conductor's existing transition sites (it already calls ControlDB.append_event on TASK_STATUS_CHANGED etc, conductor_db.py — the generalized signature must keep those callers working) AND the 2.4 reworked writer (lands P2-B; if a caller genuinely lands later, that is a material fork -> handoff, not a dark ship).
  - memory_search / memory_recent / memory_timeline / memory_log -> wired as @mcp.tool() registered on the FastMCP instance (server.py:355 pattern) = the production caller is the MCP runtime; ALSO the dashboard HTTP/CLI handlers (slice_api.py pattern) for the timeline ones.
  - the generalized hybrid_search internals -> called only by the tool functions (INTERNAL is fine; the tool is the wired entrypoint).
  Any symbol that must ship dark needs a `# WIRE-DARK[P2-A]` marker + a stubs/allowlist row (R-8).

================================================================================
SECTION 8 — UNKNOWNS (could not ground in this read-only run; flag for the executor)
================================================================================

UNKNOWN-1 (semantic WSL load — the headline risk): I could NOT empirically load nomic-embed in the teikoku distro because it has NO pip/ensurepip and apt python3-venv is not installed (`python3 -m pip` -> No module named pip; `python3 -m ensurepip` -> missing; `dpkg -l python3-venv` -> `un`). Installing the stack is a state mutation = the EXECUTOR's job (out of this run's read-only scope). STRONG INDIRECT EVIDENCE it will work: models cached + WSL-reachable (verified), torch is CPU-only (verified 2.11.0+cpu), and the crash history is a Windows-access-violation class that Linux is the standard fix for. But the actual `SentenceTransformer(...).encode([...])` call in WSL is UNVERIFIED. The grep-primary design (Fork 2) makes this non-blocking: even if semantic fails, memory still works. The executor's FIRST P2-A step should be the install + a one-line encode smoke; if it crashes, ship grep-only + register the semantic leg as WIRE-DARK with a follow-up.
UNKNOWN-2 (mcp pkg pin): requirements pins sentence-transformers==5.3.0 / qdrant-client==1.17.1 / rank-bm25==0.2.2 but mcp>=1.26.0 and nomic>=3.9.0 are floating; the executor verifies current versions at install (C-05). Whether mcp 1.26+ FastMCP API matches server.py's `from mcp.server.fastmcp import FastMCP` on the version that actually installs in WSL is UNVERIFIED.
UNKNOWN-3 (Qdrant concurrent-writer in WSL): the lock fallback (server.py:96-98) + IPC primary/secondary (rag_ipc.py) is designed for the Windows single-machine multi-session case; whether the TEIKOKU MCP-tool process and the dashboard HTTP process will contend on one Qdrant dir depends on the final process topology (Fork 3 recommends one-writer + proxy/read-via-seam, but the exact wiring is the executor's design call within that constraint).
UNKNOWN-4 (migration of legacy markdown memory-bank, master-plan RISK "migration data loss"): the existing F: memory-bank markdown (changelog/activeContext/progress/...) migration into the event log/RAG is a 2.4 (P2-B) concern; P2-A only needs the index to ACCEPT a file path. The COPY-MANIFEST corpus gap noted in the handoff CURRENT STATUS is a P2.4 migration input, not a P2-A blocker.

================================================================================
SOURCES CONSULTED (file:line or captured command)
================================================================================
F:\.ELAI_workflow\workflow\reasoning\TEIKOKU-MASTER-PLAN.md (D-07/D-08 L22-23, Phase 2 L157-169, N-01/N-02 L73-74, C-02 L85, Appendix A)
F:\.ELAI_workflow\workflow\handoff\teikoku-build.md (head 60 lines: CURRENT STATUS, ORCHESTRATOR CONTRACT, P2 plan)
F:\.ELAI_workflow\.claude\agents\memory-writer.md (full — the 2.4 role to rework)
F:\.ELAI_workflow\.claude\agents\context-compiler.md; F:\.ELAI_workflow\.claude\rules\agent-dispatch-preamble.md (R-1..R-8)
F:\.ELAI_workflow\.mcp.json:3-17 (codegraph-elai + local-rag MCP stdio registration = the query-tool template)
C:\Users\user\.claude\rag\server.py (read: 1-120 init/models, 222-423 hybrid_search+memorize, 954-1213 session register/claim/list/release/send/inbox/wait)
C:\Users\user\.claude\rag\requirements.txt (full dep set)
wsl teikoku: control.db .schema + row counts (events 3479, append-only triggers, tasks/gate_runs/verdicts/leases/task_deps)
wsl teikoku: conductor/conductor_db.py (full — ControlDB append_event/iter/recent/upsert/update_status/record_verdict), events.py (full — append_event/iter_events/replay), harness_api.py (full), slice_api.py (full — dual HTTP+CLI transport)
wsl teikoku: ssb.rs:57-360 (SSB_ACTION_TYPES registry), 1499-1583 (derived SCHEMA_VERSION), ssb/schema.rs:3294 (CREATE TABLE ssb), ssb/store.rs:252-510 (typed query trait + write + subscribe)
wsl teikoku probes: python3 3.12.3 / no pip / no ensurepip / python3-venv un-installed; sentence-transformers + torch NOT installed in distro; models CACHED at /mnt/c/.../hub/{nomic-embed-text-v1.5 523M, nomic-bert-2048, BAAI--bge-reranker-v2-m3}; Windows rag venv torch == 2.11.0+cpu (CPU-only); distro disk 946G free; bun at /usr/local/bin/bun
