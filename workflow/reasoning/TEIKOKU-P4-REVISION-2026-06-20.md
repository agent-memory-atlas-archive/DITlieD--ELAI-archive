TEIKOKU Phase 4 - REVISION (reuse-vs-build: conductors + worker lanes + model pools + per-model migration)

Date: 2026-06-20  session s-1ed32eaf  (orchestrator/controller)
Status: PLAN UPDATE. P4 build STOPPED mid-phase by owner; next session resumes P4 on THIS revised plan.
Supersedes: the original master-plan P4 (TEIKOKU-MASTER-PLAN.md :191-203) for the WORKER-LANE + MODEL-POOL + MIGRATION specifics ONLY. The Phase-4 OBJECTIVE + done-criterion are UNCHANGED (one chat survives an auto-migration zero-loss; a headless run finishes a task batch on ELAI; sessions wake/coordinate; surfaces design pass). Section 0 (D-01..D-17) NOT re-opened.

================================================================================
WHY THIS REVISION (owner pivot, surfaced mid-P4-B)
================================================================================
ELAI already has, BUILT and ready, three things P4 was about to re-build from scratch:
 (a) DEVIN CONDUCTOR  F:\.ELAI_workflow\conductor\  - the Autonomous Finish System on FREE devin models.
 (b) CODEX CONDUCTOR  /root/.conductor-codex/ (WSL codex-vm) - the same design on codex/gpt-5.5.
 (c) DEVIN BRIDGE     docs\.source\claude-code-main\claude code build\devin-bridge - routes devin free
     models through the reconstructed-CC CLI leak, retiring devin.exe.
Owner intent: REUSE/ADAPT these rather than rebuild the worker lane; retire devin.exe in favour of the
bridge; make the interactive chat support PER-SESSION model-pool choice (claude / devin-free / free-api);
per-pool auto-migration. NEW owner requirement: claude/opus auto-handoff at 200k context (not 500k), so
quality does not degrade with more context (context-rot; the AUTONOMOUS-FINISH spec itself flags this).

================================================================================
EXPLORATION DONE THIS SESSION (4 opus read-only agents; 3 succeeded, 1 blocked)
================================================================================
Detail files: workflow/reasoning/teikoku-p4-explore-{codex-conductor,devin-conductor,liberty-map}.md
Specs read by controller: AUTONOMOUS-FINISH-SYSTEM-2026-06-18.md + ...-DEVIN-2026-06-18.md.
GAP: the devin-bridge explorer was BLOCKED by a cyber-safeguard (the "CC CLI leak" framing). The bridge
is UNEXPLORED. Re-do next session by READING ITS FILES DIRECTLY (controller Read tool), not via an agent.

KEY FINDINGS
1. DEVIN CONDUCTOR (F:\conductor\) is the ANCESTOR TEIKOKU's conductor was forked from at 0.1. Deterministic
   Python daemon: 4 SQLite truth planes + SELECT->CONTEXT->EXECUTE->VERIFY->INTEGRATE->RECORD loop, 0 LLM in
   the loop; drives N devin.exe -p workers on FREE models (kimi-k2.7 / glm-5.2 / swe-1.6) in per-task git
   worktrees; off-agent frozen gate = the ONLY door to DONE; serialized integrator; a rate-limit-vs-gate-fail
   disposition brain; epoch compaction; OPTIONAL Temporal.io backbone (default = SQLite in-process loop).
   Build Steps 0-7 DONE, 68-69 tests pass. Worker dispatch = devin.exe DIRECTLY (worker_adapter.py:408-433),
   NOT a bridge. Measured FREE envelope = 2 sustained workers (4 tripped a 3h ban); code enforces 2, NOT the
   spec's suggested "max 3".
2. CODEX CONDUCTOR (/root/.conductor-codex/) = same design, codex/gpt-5.5 variant. All 9 build phases
   code-complete BUT dormant: PAUSED, dry-run, never ran a real worker turn/gate/integration. Has guard.py
   (a prompt+output safety scanner). Verdict: do NOT replace TEIKOKU's with codex; HARVEST 2 isolated pieces -
   the codex/gpt-5.5 worker adapter (a paid-tier harness) + guard.py. Leave the service dormant (do not start).
3. TEIKOKU CONDUCTOR (~/teikoku/conductor/, teikoku distro) = SAME lineage, strictly MORE capable:
   pack-aware (project_resolver/gate_packs), the P4-A comms hub (wake/mailbox/spawn), the P4-B
   headless_runner + free_api_worker. It is the CANONICAL engine. What it DROPPED / never carried from the
   ancestor: the devin-CLI worker lane, the autonomous-finish WAVE loop, the rate-limit disposition brain,
   the Temporal backbone - exactly the pieces P4-B needs.
4. .liberty = a litellm LIBRARY-mode free-api harness. CRITICAL: devin free models + claude are NOT in
   liberty - they are the CONTROLLER's pool. TWO pool SOURCES: CONTROLLER {claude, devin-free} and LIBERTY
   {cerebras / groq / gemini / mistral / openrouter / pollinations}. Liberty ALREADY ships the per-session
   model picker (chat/limits.py backend_entries + /model /codex /gptharness). Liberty has NO parallel-cap or
   drop-on-429 knob - that logic lives at the CONTROLLER level.

================================================================================
THE DECISION (CONFIRMED by owner 2026-06-20 via AskUserQuestion)
================================================================================
TEIKOKU's conductor STAYS the canonical engine. P4-B = a CONVERGE+UNIFY cycle (owner chose "converge + unify"):
diff TEIKOKU's conductor against the ancestor (F:\conductor), PORT the missing autonomous-finish WAVE loop +
the rate-limit-vs-gate-fail DISPOSITION brain + the serialized integrator (the hard, tested orchestration core;
~80% harness/project-agnostic per the explorer; ~20% ELAI-coupling bounded to config defaults / gate_runner cmd
set / backlog_import), HARVEST guard.py, and WIRE THE BRIDGE as the devin backend.
UNIFIED WORKER ARCHITECTURE (owner chose "unified base-cc + bridge"; devin.exe RETIRED): ONE base-cc CLI harness;
the "pool" is just which ANTHROPIC_BASE_URL it points at - devin-free -> the bridge :8810, free-api -> the
litellm proxy :4000, claude -> OAuth (all speak Anthropic /v1/messages). The built-but-unaudited P4-B
free_api_worker IS that universal worker (it already spawns base-cc with ANTHROPIC_BASE_URL); generalizing it =
parameterize base_url/auth/model per pool. We do NOT port the ancestor's devin.exe worker_adapter (the bridge
supersedes it).
PLUS the PAID CODEX LANE NOW (owner chose "include codex paid lane now"): harvest the codex conductor's
codex-exec/gpt-5.5 worker (2 proxy account profiles account-a/b) as a SECOND harness (the paid escalation tier;
codex-exec is a different CLI+protocol, a sibling WorkerProvider, not base-cc). So the worker plane = base-cc-
unified (claude/devin/free-api) + codex-exec (paid). Net: lower risk, reuses tested code, converges the trees.

================================================================================
GENERALITY GUARDRAIL (the core TEIKOKU requirement - owner check 2026-06-20)
================================================================================
TEIKOKU must do ANY task on ANY project, NOT just finish ELAI. The codex + devin conductors are BOTH
single-project, ELAI-HARDCODED (they bake .elai_cc / elai.map / docs/thefinalglance / the ELAI plan grammar
into config defaults + the gate command set + the backlog importer; grep pack/project_resolver = 0 hits in
BOTH). That is exactly WHY TEIKOKU's PACK-AWARE conductor (project_resolver + projects.toml + gate_packs;
P3 already drove a NON-ELAI `demo` project with ZERO .elai_cc/elai.map/cargo refs + a generic test+lint gate
set) stays CANONICAL and the ELAI conductors are DONORS, not the engine. Adopting either ELAI conductor AS the
engine would REGRESS TEIKOKU to ELAI-only.
PORT SPLIT (hold this line):
 - AGNOSTIC machinery (port/harvest AS-IS; knows nothing about ELAI): the rate-limit-vs-gate-fail disposition
   brain, serialized integrator, leases, the worker dispatch lifecycle, guard.py, the codex-exec worker mechanics.
 - ELAI-COUPLED machinery (PARAMETERIZE through packs, NEVER copy the hardcoding): config paths, the gate
   COMMAND SET, the backlog importer, the WAVE definitions. Each project supplies its own via project_resolver +
   gate_packs (P3-B already generalized the backlog adapter: --project + corpus_root from the registry).
"ANY TASK" (not only finish-a-backlog): the P4-B headless front-end takes template + free-text intake + the
   spawn/enqueue path, so the engine runs arbitrary one-off tasks too; the autonomous-finish WAVE loop is ONE
   mode (finish a project's plan backlog), pack-parameterized.
GENERALITY GATE (added to the P4-B audit + the R10 gate, BLOCKING): the converged engine MUST run a NON-ELAI
   task end-to-end (the `demo` pack or a second project) with ZERO ELAI paths/gates/grammar REACHABLE in the
   general code path. "Still works on ELAI" is necessary-not-sufficient. A reachable ELAI hardcode in a general
   path = BLOCK. Reuse P3's demo-project fixture as the generality proof; the port must not regress it.

================================================================================
WORKER PLANE (CONFIRMED: unified base-cc harness + one paid codex harness)
================================================================================
HARNESS 1 = base-cc (ONE CLI), backend chosen per task/session by ANTHROPIC_BASE_URL (all Anthropic /v1/messages):
 - claude     -> OAuth, no base_url override (high-trust).
 - devin-free -> the BRIDGE http://127.0.0.1:8810 (devin_proxy.py, Starlette/uvicorn; PROVEN 200-OK live in its
                 log). Models kimi-k2-7 (default) / glm-5-2 / swe-1-6 (HYPHENATED wire ids), no credit cost.
                 CAVEAT 1: Devin's gateway content-filters security/RE-heavy system prompts (ELAI's CLAUDE.md
                 trips it) -> for devin-backed runs use the bridge's DEVIN_TOOL_DESC=generic + a clean
                 DEVIN_SYSTEM_FILE (the proxy replaces CC's built-in system prompt). CAVEAT 2: parallel
                 tool-calls through the bridge UNVERIFIED (single calls confirmed) -> verify in the port.
 - free-api   -> the litellm free-proxy :4000 (liberty roster: cerebras/gpt-oss-120b volume + groq 70B/120B
                 quality + mistral-small + pollinations floor). EXCLUDE liberty's uncensored group (tools off)
                 + gemini-flash (20 RPD). = the held P4-B free_api_worker, generalized.
HARNESS 2 = codex-exec/gpt-5.5 (PAID, harvested from the codex conductor - owner: include now): the 2 proxy
 account profiles (account-a/b, bmapi proxy, OpenAI Responses API). A sibling WorkerProvider for the paid
 escalation tier; NOT base-cc (different CLI+protocol).
devin.exe = RETIRED (the bridge supersedes it; no Windows-interop dependency).
CONCURRENCY: hard cap 2 sustained FREE workers (measured; 4 = 3h ban). The CONTROLLER owns the parallel cap +
the drop-on-429 degrade (liberty has none). "prefer kimi" = pool order (the bridge default is already kimi-k2-7).

================================================================================
MODEL POOLS (P4-C / 4.2 - per-session selectable)
================================================================================
Two sources, one picker (reuse liberty chat/limits.py backend_entries shape):
  CONTROLLER  {claude, devin-free: kimi-k2.7 / glm-5.2 / swe-1.6}
  LIBERTY     {free-api + sub-groups: cerebras-fast / groq / gemini-1M / ...}
A session picks its pool; orchestrator-front + drop-in override (D-10) + the toggle set (orchestration
on/off, model pool, model choices, skills, objective) ride on top; session-spawns-subagents uses P4-A spawn.

================================================================================
AUTO-MIGRATION (P4-D / 4.3 - PER-MODEL thresholds, never one global number)
================================================================================
Trigger reads response.model and budgets the NEXT turn against THAT model's window. When a pool has fallback
models, use the MINIMUM window across reachable models (liberty catalog.min_context_window_for_group) - else
a 131k prompt overruns when the router falls back to a 32k mistral.
  pool / model                       window      migrate-at
  claude (controller)                ~200k+*     200,000     <- OWNER OVERRIDE (quality handoff, not window)
  devin kimi-k2.7                    260,000     ~200,000
  devin glm-5.2 / swe-1.6            150,000     ~120,000
  cerebras gpt-oss-120b              131,072     ~104,000
  groq 70B / 120B / scout / 8b       131,072     ~104,000
  gemini-2.5-flash / flash-lite      1,048,576   single-shot only (RPD 20)
  32k tier (mistral/qwen/venice/poll) 32,768     ~26,000
  unknown / synced-no-window         4,096       ~3,200 (safe default)
* claude's real window is larger (200k standard, 1M on the [1m] variant); 200,000 is the QUALITY handoff
  point the owner set - it applies to the interactive chat AND the orchestrator/controller session itself.
Migration = D-09 structured snapshot + RAG fallback + "migrating" indicator in the SAME chat + auto-carry
skills/rules/toggles; the snapshot is model-pool-aware (carries the pool + that model's threshold).

================================================================================
REVISED P4 CYCLE PLAN (next session)
================================================================================
P4-B  CONVERGE+UNIFY: (1) diff ~/teikoku/conductor vs F:\conductor to find the exact wave-loop/disposition
      gaps; (2) generalize the held free_api_worker into the UNIVERSAL base-cc worker (base_url/auth/model per
      pool) + wire the BRIDGE :8810 as the devin backend (with the sanitized devin system prompt); (3) harvest
      the codex-exec/gpt-5.5 paid harness (2 accounts) + guard.py; (4) port any missing wave-loop + disposition
      + serialized integrator into TEIKOKU's pack-aware engine; run_headless drives it. Then AUDIT the whole.
P4-C  4.2 interactive front-end: the per-session model-pool picker (reuse liberty's), orchestrator-front +
      drop-in override, always-grill, the toggle set, session-spawns-subagents.
P4-D  4.3 auto-migration: per-model thresholds (table above; claude=200k), snapshot+RAG, migrating
      indicator, auto-carry, accurate PER-MODEL token counter.
P4-E  4.5 dashboard surfaces: chat pane (image+audio upload, drop-in, migrating indicator) + per-project
      session list. Then the R10 integration gate + ONE P4 commit.

================================================================================
RESOLVED DECISIONS (owner, 2026-06-20 AskUserQuestion)
================================================================================
D1 ARCHITECTURE = UNIFIED base-cc + bridge. ONE CLI; backend via ANTHROPIC_BASE_URL; devin.exe RETIRED.
D2 DEVIN TRANSPORT = the BRIDGE (:8810), confirmed BUILT + PROVEN (200-OK live turns). Subsumed by D1.
D3 PAID LANE = INCLUDE THE CODEX/gpt-5.5 LANE NOW. Harvest the codex conductor's worker.py (codex-exec) + the
   2 account profiles (account-a/b) as a sibling paid-escalation WorkerProvider beside the free base-cc lanes.
D4 P4-B SCOPE = CONVERGE + UNIFY. Diff TEIKOKU's conductor vs the ancestor (F:\conductor); wire the bridge
   backend; port the missing wave-loop + disposition brain + serialized integrator; harvest guard.py + the
   codex worker adapter; then AUDIT.
ALSO: guard.py = harvest (part of D4). The codex conductor SERVICE = leave DORMANT (donor only; do NOT start it
   - one control action would launch a real wave on the live 383-task ELAI backlog). TODO at port time: confirm
   the gpt-5.5 context window from the codex conductor config + add its migrate-at row to the P4-D table.

================================================================================
WHAT IS BUILT RIGHT NOW (carry into next session)
================================================================================
- P4-A comms hub: DONE, audited PASS 0.93 (wake/mailbox/spawn + base-cc hub-wake). Uncommitted (P4 commits
  once, at phase end).
- P4-B free_api_worker + headless_runner + run_headless + SITE-1 integrate --project + GPU defer hook +
  conductor `headless` verb: BUILT, executor self-verified, NOT audited, paused. Becomes harness H2.
- devin conductor (F:\conductor\): ready (Steps 0-7, 68-69 tests, devin.exe lane). = the PORT SOURCE.
- codex conductor (/root/.conductor-codex/): ready, DORMANT. guard.py + codex adapter = harvest donors.
- The 3 exploration .md + this revision doc = the plan basis.
- Nothing committed; teikoku distro git at 3c77f78 (P3). control.db.p4a-backup + .p4b-backup exist.
