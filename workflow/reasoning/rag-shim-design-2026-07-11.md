rag secondary shim - recon + design (ram-build-efficiency Phase 5a, unit B part 1)

Session ram-P5a executor, 2026-07-11. DOC-ONLY. Ships no code, no wiring. Decides whether P5b (build the shim + edit .mcp.json) is worth the risk.

SCOPE / GROUNDING
All numbers below are measured on this host 2026-07-11 or read from installed source. rag core (search/embedding/reranking) was never touched. Every design claim is grounded in C:\Users\user\.claude\rag\server.py, rag_ipc.py, or the installed mcp 1.26.0 package - no training-memory precedent is cited (the fabricated "walk_state._StateMutex" precedent was already removed by the P1 redteam; RECON_NEEDED R7 replaces it with the real rag_ipc election).

===================================================================
FORK F1 VERDICT: DIMINISHING  (P5b is NOT worth building; owner-decision only)
===================================================================

Rule: IF python+mcp probe RSS < 60% of live-secondary RSS -> route SHIM. ELSE -> DIMINISHING.

  python+mcp probe RSS = 61.3 MB
  live-secondary RSS   = 68.0 MB (representative; fleet cluster 68.0-69.0)
  ratio                = 61.3 / 68.0 = 90.1%   (threshold for SHIM was < 60%)

90.1% is far above the 60% SHIM gate. Route = DIMINISHING. The realistic shim floor (a constructed FastMCP + rag_ipc import, probe C below) is 62.0 MB = 91.2% of the secondary - i.e. a purpose-built shim would weigh essentially the same as the process it replaces. The RAM floor of a secondary IS the mcp/pydantic/anyio import graph, which any MCP stdio server (shim included) must pay in full.

MEASURED RSS TABLE (child interpreter, not the venv launcher shim; measured after ~8s import settle)
  role                                    RSS_MB   PRIV_MB   notes
  bare python (probe A)                    11.2      5.3     import sys,os,time only
  python + import mcp (probe B, plan lit)  61.3     48.0     import mcp,json,uuid,time,os  (the F1 number)
  FastMCP()+rag_ipc floor (probe C)        62.0     48.3     from mcp.server.fastmcp import FastMCP; FastMCP("x"); import rag_ipc  (realistic shim floor)
  live secondary (full server.py)          68.0     55.0     26 @mcp.tool registered, proxying, models lazy/never loaded
  primary (models loaded, PID 42608)     1887.8   2362.7     for scale only; not a shim candidate

  Probe method: venv Scripts\python.exe is a launcher redirector (measures ~3.9 MB); the real interpreter is its child. Measured the child via Win32_Process ParentProcessId. Probes started + killed by this session only (started-by-me, per preamble R-2). Probe scripts in scratchpad probe_{a,b,c}.py.

  Delta a real 5-tool shim could save vs a secondary: 68.0 - 62.0 = ~6 MB RSS (~7 MB private), about 9%. And that 9% is the OPTIMISTIC ceiling that assumes a 5-tool-only shim, which the tool-surface finding below shows is INVALID - a correct shim must host all 26 tools and lands back near 68 MB, i.e. near-zero real saving.

WHY the floor is what it is: server.py's heavy imports (sentence_transformers, CrossEncoder, qdrant, torch) are ALL lazy (server.py:99-118 _get_embed_model/_get_reranker, and _get_qdrant returns None for a secondary at server.py:127). A secondary NEVER loads them - it loses the primary election and every heavy tool proxies. So a secondary is ALREADY effectively a thin shim; its 68 MB is unavoidable interpreter + mcp + pydantic + anyio + FastMCP tool-registration overhead. There is nothing heavy left to strip.

===================================================================
DESIGN (recorded for completeness; NOT to be built unless the owner overrides F1)
===================================================================

1. TOOL SURFACE
Naive spec was: the shim serves exactly the PROXIED_TOOLS frozenset (rag_ipc.py:34-40, 5 exact names):
  mcp_search_context, mcp_memorize, mcp_ingest_file, mcp_list_topics, mcp_forget
M3 ON_FAIL TRIGGERED - PROXIED_TOOLS does NOT cover the advertised tool surface. server.py registers 26 @mcp.tool() functions. Only those 5 call rag_ipc.proxy_call when _is_secondary (verified: proxy_call appears at server.py lines 387/521/537/548/609/623/653/658/682/690 - all inside the 5 heavy _impl_ funcs, nowhere else). The other ~21 tools run IN-PROCESS in every secondary, they do not proxy:
  mcp_update_context, mcp_report_build, mcp_log_change,
  mcp_fsm_get_state, mcp_fsm_transition, mcp_fsm_agent_transition, mcp_fsm_increment_attempts, mcp_fsm_reset,
  mcp_session_register, mcp_session_claim, mcp_session_list, mcp_session_release, mcp_session_send, mcp_session_inbox, mcp_session_wait,
  mcp_attribute_build_errors,
  mcp_think, mcp_think_set_budget, mcp_think_extend, mcp_think_reset
These operate on local session/FSM/think state files, need no Qdrant, and are heavily used by the ELAI workflow (mcp_fsm_get_state, mcp_think, mcp_session_* fire constantly). A shim scoped to only the 5 proxied tools would DELETE these 21 tools from that session's MCP endpoint and break the harness.
CONSEQUENCE for a correct shim: it must serve all 26 tools - forward the 5 via proxy_call, and host the 21 local tools by importing their in-process logic from server.py. That import graph is almost the whole of server.py (minus the already-lazy heavy models), so the shim reconverges on ~68 MB. This is the decisive strengthener of the DIMINISHING verdict: there is no "thin 5-tool shim" that keeps the session functional.

2. TRANSPORT (reused verbatim, never reinvented)
The shim would reuse rag_ipc.proxy_call (rag_ipc.py:199) exactly as-is for the 5 heavy tools. File IPC under C:\Users\user\.claude\rag\data\ipc\:
  request  req-{uuid8}.json   {"fn": name, "args": {...}}   (rag_ipc.py:204-213)
  response resp-{uuid8}.json  {"ok": bool, "result"|"error"} (written tmp-then-rename for atomicity, rag_ipc.py:190-194)
  poll     100 ms interval, 120 s deadline (rag_ipc.py:218-240), primary-liveness re-checked each loop
No wire-format divergence is possible by construction - the shim calls the same proxy_call the current secondary calls, so a running primary answers it identically. This is why the "IPC protocol/version skew" attack fails (recorded in the P1 redteam).

3. LIFECYCLE (PPID watchdog, mirror of server.py:1823 _start_parent_watchdog)
Windows does not cascade-kill: when Claude Code kills the venv launcher shim it spawned, the child server keeps running and piles up (server.py:1826-1827 docstring: "a dozen were found alive at once"). The shim must replicate the secondary branch of _start_parent_watchdog exactly: capture os.getppid() at start, a daemon thread sleeps 5 s, and on TWO consecutive dead-parent readings (guards a transient OpenProcess miss, server.py:1858) call os._exit(0). A shim is always a secondary (it holds nothing others need), so it uses the immediate-reap branch (server.py:1860-1862) - it never needs the primary's PRIMARY_ORPHAN_IDLE_GRACE=120 s idle path. Parent-liveness check = _parent_alive (server.py:1796-1817, OpenProcess + GetExitCodeProcess==STILL_ACTIVE on nt).

4. RACE-SAFE PRIMARY START (grounded in R7 real pattern, no invented precedent)
The election the shim must coexist with is rag_ipc.try_become_primary (rag_ipc.py:97-126). Its REAL pattern is: _check_primary_alive() reads primary.pid and OpenProcess-probes it; if none alive, write our PID to primary.pid, time.sleep(0.1), re-read, and if the stored PID != our PID we lost the race and become secondary (rag_ipc.py:111-122). It is a write-PID + sleep + reread election, explicitly NOT an os.open(O_CREAT|O_EXCL) lock (the code comments it "Race-safe enough for our use case", rag_ipc.py:109-110). Design decision for the shim: REUSE this existing election untouched by calling rag_ipc.try_become_primary(). If it returns True the shim would have to become a real primary (open Qdrant + load nothing heavy but serve the poller) - which defeats the shim's purpose, so a shim only makes sense when a primary already exists; if the election is won, the correct shim behaviour is to hand off to the full server.py path rather than run as a model-less primary. If a stricter guard were ever wanted it would be os.open(lock, os.O_CREAT|os.O_EXCL) grounded in the Python os module docs (os.O_EXCL) - the only sanctioned alternative; no other precedent is cited. Given F1=DIMINISHING this is not built.

5. ROLLBACK (recorded old launch command)
The ELAI-root local-rag launch entry today (F:\.ELAI_workflow\.mcp.json:13-17, read-only, NOT edited):
  command: C:\Users\user\.claude\rag\.venv\Scripts\python.exe
  args:    ["C:\Users\user\.claude\rag\server.py"]
  env:     { "PYTHONIOENCODING": "utf-8" }
The rollback mechanism (only relevant if the owner overrides F1 and P5b ships): P5b M2 backs up this exact command block into the plan phase log BEFORE editing, and any rollback restore is a re-Write of that backed-up block into .mcp.json, never a git operation (.mcp.json is gitignored - blanket * with only .elai_cc/** tracked, DoD in the plan). The global rag ABORT (any live session breaks after the .mcp.json edit) has highest priority and triggers the immediate restore-Write.

6. RSS TABLE + F1 VERDICT
See the FORK F1 VERDICT block and MEASURED RSS TABLE above.

===================================================================
RECOMMENDATION on P5b
===================================================================
Do NOT build P5b, and do not spend the .mcp.json edit risk on it. Two independent reasons, either sufficient:
  (a) The measured per-secondary saving is ~6 MB RSS / ~9% (probe B vs secondary), and that is the optimistic 5-tool-only ceiling.
  (b) A functional shim cannot be 5-tool-only - it must host the 21 local fsm/session/think tools, importing most of server.py, so it reconverges on ~68 MB and the real saving approaches zero.
Even at the optimistic ceiling, ~6 MB times the ~30 concurrent secondaries observed on this host is ~180 MB - marginal against a 63.2 GB host with a 1.9 GB primary and against the .mcp.json breakage risk (Global rag ABORT).

OUT-OF-SCOPE FINDING (the real RAM lever, for a future owner decision - not this plan)
The per-secondary WEIGHT is not the problem; the secondary COUNT is. This host currently shows ~30 live rag secondaries (~68 MB each ~= ~2 GB) plus ~30 venv-launcher stubs (~4 MB each) plus one 1.9 GB primary. server.py's own watchdog docstring records the historical leak it already fixed ("a dozen idle model-holding primaries at >1GB each", server.py:1837-1838). The remaining lever is fewer concurrent primaries/secondaries and reap tuning (PRIMARY_ORPHAN_IDLE_GRACE, the launcher-count), NOT trimming a 68 MB secondary to 62 MB. This is launch-topology, adjacent to but distinct from the shim; flagged here so it is not lost.

RECON SETTLED (R3, R4, R7)
R3 (server.py import weight in secondary role): top-of-file module imports (server.py:6-20) are all light stdlib + anyio; the only non-stdlib module-level import in the whole hot path is anyio and, at line 512, from mcp.server.fastmcp import FastMCP. Every heavy import (sentence_transformers, CrossEncoder, qdrant_client) is lazy inside a getter and never runs in a secondary. Measured floor = 61-62 MB (mcp graph), secondary = 68 MB.
R4 (how installed mcp runs a stdio server): mcp dist version 1.26.0. FastMCP at C:\Users\user\.claude\rag\.venv\Lib\site-packages\mcp\server\fastmcp\server.py. A minimal stdio server uses: constructor FastMCP.__init__ (server.py:147) e.g. FastMCP("name"); tool registration via the @mcp.tool() decorator (FastMCP.tool, server.py:446) or FastMCP.add_tool (server.py:397); and blocking run via FastMCP.run(transport="stdio") (server.py:279) which calls anyio.run(self.run_stdio_async) -> run_stdio_async (server.py:753) -> stdio_server() context. rag's server.py drives it identically at server.py:2022 mcp.run().
R7 (primary-election pattern the shim coexists with): rag_ipc.try_become_primary (rag_ipc.py:97-126) - write-PID + time.sleep(0.1) + reread, explicitly NOT O_CREAT|O_EXCL (self-commented "Race-safe enough", rag_ipc.py:109-110). PROXIED_TOOLS (rag_ipc.py:34-40) = exactly 5: mcp_search_context, mcp_memorize, mcp_ingest_file, mcp_list_topics, mcp_forget. proxy_call at rag_ipc.py:199.
