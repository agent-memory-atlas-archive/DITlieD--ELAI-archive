# TEIKOKU Intelligent Provider Routing — full plan (TR-1..TR-8)

name: TEIKOKU-IR (intelligent routing) — port ELAI's cost/model/context routing intelligence into TEIKOKU's shipped conductor.
status: PLANNED, authored 2026-07-04 by the account-isolation session after reading the shipped TEIKOKU routing code end-to-end + ELAI-COST-MODEL-CONTEXT-PIPELINE-2026-07-04.md.
purpose: TEIKOKU today routes a sub-task to a provider by ROLE alone (route_pool_for), with a difficulty axis that is DORMANT, NO context-size awareness, and NO failure escalation — so a hard or large-context task lands on a weak/small-window FREE model purely because its role is EXECUTE. This plan replaces the one-line role rule with a single grounded RouteDecision that composes role + computed difficulty + context-size class + per-model capability floor + outcome-ledger signal, adds runtime failure/overflow escalation, and instruments the whole thing. Target executor: ONE Opus-1M context agent implements TR-1..TR-8 in a single session (the plan is self-contained); the audit/gate stays OFF-AGENT (TEIKOKU gate spine).
grounding: every seam below is a real file:line in the WSL teikoku distro (~/teikoku/conductor), read this session. Every unit cites the ELAI design unit it ports (the cost-model doc) so the design is not invented. Honest scope boundaries in Section 5 (what is NOT portable because TEIKOKU drives base-cc, not raw decoding).
how to run: hand this file to a fresh Opus session at the TEIKOKU distro (cd ~/teikoku). base-cc edits are sanctioned. ONE commit at the end (maintainer voice, no AI trailer), owner-hold + no-auto-commit convention — do NOT push. Do NOT touch the ELAI FSM. PowerShell->wsl for all distro ops; edits over \\wsl.localhost strip +x on shell scripts (re-chmod).

================================================================================
0. LOCKED DECISIONS (scope; the owner asked for "not the cheapest fix — everything in one go")
================================================================================
D-01  Replace the scattered role rule with ONE decision object. route_pool_for stays a pure leg but is SUBSUMED by a new RouteDecider.decide(ctx) -> RouteDecision. NO new conditionals bolted onto route_pool_for (maintainability MR / anti-spaghetti). The decision is deterministic + auditable (no LLM in the routing decision itself), mirroring the existing "no LLM in the decision" property.
D-02  Context-size awareness is the headline unit (TR-1). A FREE model whose context_window cannot hold the estimated task context is DISQUALIFIED at routing time, with a runtime overflow backstop. This is the owner's core concern and the single biggest correctness gap.
D-03  Difficulty is COMPUTED, not caller-optional (TR-2). The dormant difficulty param gets a real deterministic classifier feeding it at both live call sites.
D-04  Escalation is BIDIRECTIONAL and FAILURE-DRIVEN (TR-3), not only user-keyword. A FREE task that fails the auditor / overflows its window beyond a cap promotes to a bigger lane or PAID, bounded + journaled.
D-05  Everything is DATA-DRIVEN off TEIKOKU's own journal where possible (TR-5). Capability floors + the pre-router read the outcome ledger; seeded conservative, refined from real (task-shape, lane, verified-outcome) rows. No paper constants.
D-06  Honesty on portability (Section 5). TEIKOKU dispatches base-cc as the worker; it does NOT own the model's decoding, so ELAI's XGrammar/CodeAct/FIM modality router (J-55) is only PARTIALLY portable — scoped to what TEIKOKU actually controls (model choice, tool-set exposure, base-cc flags). Do not overclaim a modality router TEIKOKU cannot enforce.
D-07  Fail-closed + gated. Missing size estimate -> treat as LARGE (route up, never down). Unknown model window -> treat as SMALLEST (never assume it fits). Every new default-behavior change ships behind a config flag defaulting ON only after the R10 smoke passes; the flag has a wire-dark-allowlist row until then.
D-08  One commit, owner-hold. TEIKOKU convention: build all units, audit off-agent, hold uncommitted for the owner. This plan file lives in the workflow root (gitignored), the code lives in ~/teikoku (its own repo).

================================================================================
1. THE PROBLEM (grounded from the shipped code, this session)
================================================================================
Shipped routing = ONE function, worker_providers.py:74 route_pool_for(role, difficulty):
  - ORCHESTRATE / AUDIT -> claude (PAID)
  - EXECUTE + difficulty=="hard" -> claude (PAID)
  - everything else -> devin-free (FREE), free-api fallback
Three defects, all confirmed by reading the callers + roster:
  (1) difficulty is a DEAD HOOK. chat_orchestrator.py:514 and headless_runner.py:113 both call route_pool_for(role) with NO difficulty -> the "hard -> PAID" branch never fires automatically. No classifier exists.
  (2) NO context-size input. route_pool_for takes only (role, difficulty). model_pool KNOWS every window (PoolModel.context_window, model_pool.py:64/100-154) and uses it ONLY for auto-migration (P4-D) + stamping base-cc's window env (worker_providers.py:177 stamp_context_window). resolve_slot(pool, model) (model_pool.py:227) picks by pool+model, NEVER by the task's context size. The FREE roster spans 32k (free-mistral / free-fast / free-pollinations) to 1M (free-gemini) — a large task routed to FREE by role can land on a 32k model, or MISS the 1M model that would have served it for $0.
  (3) NO failure escalation. Escalation to PAID is USER-keyword (chat_orchestrator.py:191 _is_escalation matches "escalate/use claude/use opus/paid lane") or grill-gate driven. record_worker_result (scheduler.py:127) dispositions FAILED/NEEDS_REVIEW but the deterministic rule re-picks the SAME pool for the same role on retry (scheduler retry caps free_retry_cap/paid_retry_cap exist, scheduler.py:52, but nothing promotes a failing FREE task to PAID).
Net: TEIKOKU has the SKELETON (role->pool + per-model windows + migration + cost ceiling W8) but not the INTELLIGENCE (ELAI doc Stage 4: J-84 difficulty, J-51 capability floor, J-55 modality, I-7 escalate-on-failure, I-25 ledger router, context-size-aware selection). This plan adds that intelligence.

================================================================================
2. ARCHITECTURE — the unified RouteDecision (new core object)
================================================================================
NEW module conductor/route_decider.py. ONE entry:

  @dataclass(frozen=True)
  class RouteContext:
      role: str                      # ORCHESTRATE|AUDIT|EXECUTE|CONTEXT_COMPILER|...
      objective: str                 # the task/turn text (for difficulty + size floor)
      attached_context_chars: int    # size of any pre-assembled context-package/attachments (0 if none)
      declared_context_tokens: int|None   # optional caller hint (e.g. resumed-session snapshot size)
      prior_attempts: list[AttemptOutcome]  # from the ledger (TR-5): (lane, model, status_hint, overflow)
      vision_required: bool
      tool_heavy: bool               # role/objective needs many tool calls (advisory)

  @dataclass(frozen=True)
  class RouteDecision:
      pool: str                      # model_pool.POOLS id
      model: str                     # concrete model within the pool
      model_tier: str                # FREE|PAID (mirrors SlotResolution)
      est_context_tokens: int        # the estimate used
      difficulty: str                # computed (trivial|easy|medium|hard)
      reason: str                    # human-readable WHY (audited, journaled)
      disqualified: list[tuple[str,str]]  # (model, reason) rows for what was ruled out
      action_profile: str            # TR-6 scoped emission profile (default|constrained|code-native)

  class RouteDecider:
      def decide(self, ctx: RouteContext) -> RouteDecision: ...

decide() composes the legs in a FIXED order (each leg is its own pure function, unit-tested independently; no leg mutates another's state — anti-spaghetti):
  1. difficulty = classify_difficulty(ctx)                     [TR-2]
  2. est = estimate_context_tokens(ctx)                        [TR-1]
  3. base_pool = route_pool_for(role, difficulty)  (REUSED, unchanged)  [existing]
  4. candidates = capability_filter(base_pool, role, difficulty, ctx)   [TR-4]
  5. model = pick_model_for_window(candidates, est)            [TR-1] — smallest capable model whose window >= est*safety; else climb the ladder (devin-free -> free-gemini 1M -> PAID)
  6. model = ledger_veto(model, ctx)                           [TR-5] — swap off a known-bad (model, task-shape) pair
  7. action_profile = resolve_action_profile(model)            [TR-6]
  8. return RouteDecision(... reason=join(all leg reasons) ...)

WIRE: replace the two live call sites (chat_orchestrator.py:514, headless_runner.py:113) to build a RouteContext and call RouteDecider.decide, then model_pool.resolve_slot(decision.pool, decision.model) UNCHANGED. route_pool_for stays as leg 3 (do NOT delete it — it is the auditable role floor). Emit a ROUTE_DECISION SSB event (reason + disqualified) per dispatch — the observability that makes the router falsifiable (ELAI H-9/K-2 analog).

================================================================================
3. UNITS (TR-1..TR-8) — each: goal / ports / seams / deliverable / done-gate
================================================================================

TR-1  CONTEXT-SIZE-AWARE MODEL SELECTION  [ports: doc Stage 4 pick-by-window + Stage 5 budget; the owner's core]
  GOAL: never route a task to a model whose window cannot hold it; prefer the smallest FREE model that fits, climb to a bigger FREE lane (kimi 260k -> gemini 1M) before spending PAID.
  SEAMS: NEW estimate_context_tokens (route_decider.py) — floor = (len(objective)+attached_context_chars)//4 + a fixed base-cc overhead constant (system+skills+rules+repo-map; measure once, name it a constant, default ~18_000 per the E-32 ELAI_LOCAL_MAX_CONTEXT headroom note) + declared_context_tokens if given. NEW pick_model_for_window over model_pool._ALL_MODELS rows (model_pool.py:100-154) using PoolModel.context_window with a SAFETY_FRACTION (route only if window*0.8 >= est, reusing the migrate_at ratio already in the roster). RUNTIME BACKSTOP: in scheduler.record_worker_result (scheduler.py:127), a result carrying an overflow signal (base-cc context_window_blocked / a new overflow flag parsed from worker_status_hint/error_category) re-queues the task with declared_context_tokens bumped to force a bigger-window lane (feeds TR-3).
  DELIVERABLE: pick_model_for_window + estimate_context_tokens + the overflow parse in worker_adapter's result mapping + the re-queue leg in record_worker_result.
  DONE-GATE (WIRING_CONTRACT): drive the REAL headless entrypoint (POST /headless/run) on a HARD input whose est context > 32k; assert via the ROUTE_DECISION SSB row that it did NOT resolve to any 32k model (free-mistral/free-fast/free-pollinations) and DID resolve to a >=est-window lane; assert an artificially-forced overflow result re-queues onto a bigger window (not the same model). Missing SSB row = FAIL (fail-closed).

TR-2  COMPUTED DIFFICULTY CLASSIFIER  [ports: J-84 2-axis reasoning classifier]
  GOAL: fill the dead difficulty hook with a deterministic classifier (NO LLM) so "hard -> PAID" fires automatically.
  SEAMS: NEW classify_difficulty(ctx) -> trivial|easy|medium|hard from deterministic signals: objective length + imperative-verb/complexity keyword set (refactor/migrate/design/concurrency/security vs typo/rename/bump), touched-file count if known, dependency/wave depth from the task row, and PRIOR-FAILURE (>=1 failed attempt on this task auto-raises difficulty — the escalation feeder). Feed the result into route_pool_for at leg 3. The lambda-style override (a caller-supplied difficulty) still wins if explicitly set (owner keyword override kept).
  DELIVERABLE: classify_difficulty + wiring into RouteContext build at both call sites.
  DONE-GATE: unit table over fixture objectives (trivial->easy->hard) asserts the tier; an EXECUTE task classified "hard" routes to claude via the composed decide() (not just the leg); a prior-failed task's re-route raises its difficulty (composes with TR-3).

TR-3  FAILURE / OVERFLOW ESCALATION  [ports: I-7 local-first waterfall, escalate-on-failed-receipt; I-8 dead-end]
  GOAL: a FREE task that fails the auditor or overflows its window promotes to a bigger lane / PAID, bounded + journaled — not an infinite free retry.
  SEAMS: scheduler.record_worker_result (scheduler.py:127) — on status_hint in {FAILED, NEEDS_REVIEW} OR an overflow signal, from a FREE lane (lease_pool != claude), when the task's FREE attempt count >= free_retry_cap (scheduler.py:52, already threaded): re-queue with an escalation stamp (declared_context_tokens carried for overflow; difficulty="hard" forced for quality failure) so the NEXT decide() climbs. Bounded by the existing paid_retry_cap + the W8 cost ceiling (never escalate past the $/run gate). Emit ESCALATION_PROMOTED SSB (from_lane/to_lane/reason/attempt). I-8 dead-end: cache the (objective-hash, failed-lane) so an isomorphic re-queue does not re-try the identical dead lane.
  DELIVERABLE: the promote leg in record_worker_result + the escalation stamp carried through re-queue + the dead-lane cache + SSB event.
  DONE-GATE: an independent smoke drives a task that FAILS on a forced-FREE lane and asserts the re-queue lands on a bigger/PAID lane (SSB ESCALATION_PROMOTED), bounded by paid_retry_cap (no infinite loop), and that a second identical failure does NOT re-pick the dead lane. Cost-ceiling interaction: escalation HALTS at the W8 ceiling, does not blow it.

TR-4  PER-MODEL CAPABILITY FLOOR  [ports: J-51 tool-call adherence floor + J-55's "CodeAct loses on 7B" viability finding]
  GOAL: keep weak/small models off roles they cannot do (hard executor, tool-heavy, vision) even when role+size would otherwise route there.
  SEAMS: EXTEND PoolModel (model_pool.py:64) with capability fields seeded conservative: tool_reliable: bool (default False for sub-coding-grade free models — free-mistral/free-fast/free-pollinations/free-groq-8b; True for kimi/glm/swe/groq-big/claude/codex), min_role_tier (a model may serve up to a role class), vision_capable already exists (reuse). NEW capability_filter(base_pool, role, difficulty, ctx) drops candidates that fail the floor (a tool_heavy or "hard" EXECUTE role removes non-tool_reliable models; vision_required removes non-vision). If the filter empties the FREE set -> climb to PAID (never dispatch a known-incapable model). Seed values are DATA in the roster (auditable), refined by TR-5.
  DELIVERABLE: PoolModel capability fields + seeded roster values + capability_filter + the empty-set climb.
  DONE-GATE: a tool_heavy hard-EXECUTE RouteContext never resolves to a non-tool_reliable model (SSB disqualified rows name why); a vision_required task never resolves to a text-only lane; an emptied FREE set climbs to PAID with reason="capability_floor".

TR-5  OUTCOME-LEDGER ROUTING SIGNAL  [ports: I-25 ledger-trained router — data foundation, deterministic pre-router]
  GOAL: route off TEIKOKU's OWN verified outcomes, not a static rule — a (task-shape, lane) pair with a bad track record is vetoed before it burns another attempt.
  SEAMS: record_worker_result (scheduler.py:127) already has model_used/lease_pool/status_hint/error_category — WRITE a routing_ledger row per disposition (task-shape features: role, difficulty, size-class, tool_heavy; + lane, model, verified status, overflow, cost from the W8 ledger). NEW ledger_veto(model, ctx): a deterministic pre-router that reads recent routing_ledger rows for this task-shape and swaps OFF a model whose recent (shape, model) success rate is below a floor (min N rows, else no veto — fail-open on thin data, the ONLY fail-open in the plan and justified: no data = trust the role rule). NO ML training in this plan — the ledger is the substrate; a trained router is a future unit.
  DELIVERABLE: routing_ledger table (migration) + the write in record_worker_result + ledger_veto + prior_attempts population of RouteContext.
  DONE-GATE: after seeding N failing (shape, model) rows, decide() for that shape vetoes that model with reason="ledger_veto"; thin data (<N rows) does NOT veto (fail-open proven).

TR-6  ACTION / EMISSION PROFILE (SCOPED)  [ports: J-55 modality router — HONESTLY scoped to what TEIKOKU controls]
  GOAL: per-model action profile that sets the base-cc knobs TEIKOKU actually owns; do NOT claim a decoding-level modality router TEIKOKU cannot enforce (D-06).
  SEAMS: resolve_action_profile(model) -> default|constrained|code-native. TEIKOKU controls: which model (already), the tool-set exposure (base-cc allowed-tools/MCP subset per session, already threaded via session_toggles/_load_skill_context), and base-cc flags. constrained = a reduced tool surface + tighter system note for sub-14B free lanes (grounded: the doc's J-55 keystone measured open models collapse ~12% on multi-tool code-mode, viability ~14B). code-native/default for capable lanes. This is a PROFILE SELECTION, not decoding control. Emit the profile on the ROUTE_DECISION row.
  DELIVERABLE: resolve_action_profile + the tool-surface reduction wired through the existing session toggle/MCP-subset seam for the constrained profile.
  HONEST LIMIT (recorded, not a defect): TEIKOKU cannot force FIM/DIFF/XGrammar on base-cc; those ELAI units (B-28/H-3/J-55 decoding legs) are NOT portable here. This unit ships the portable subset only.
  DONE-GATE: a sub-14B free model resolves action_profile="constrained" and the dispatched base-cc session carries the reduced tool surface (verified through the real session-toggle seam); a capable lane carries default. No overclaim of decoding control in the reason string.

TR-7  MEASUREMENT + COST/QUALITY LEDGER TIE-IN  [ports: K-2 routing-economics bench, H-9 cost cockpit]
  GOAL: make the router falsifiable — every decision + outcome + cost is recomputable from the journal, and a bench reports the offload % and quality delta.
  SEAMS: the ROUTE_DECISION + ESCALATION_PROMOTED SSB events (TR-1/3) + the routing_ledger (TR-5) + the W8 cost ledger already on the monitor snapshot -> a NEW `teikoku route-economics` report leg (reuse build_orchestration_snapshot's cost section) computing % steps served FREE vs PAID, $ delta vs claude-only, and verified-outcome rate per lane. Surface a routing chip on the CanvasMonitor (SIGNAL: square marks, zero hex, fail=light-inversion).
  DELIVERABLE: the report leg + the monitor chip.
  DONE-GATE: the report reproduces the FREE/PAID split + $ delta from journal rows on a real run; the chip renders (svelte-check 0/0, zero hex).

TR-8  INTEGRATION GATE (R10) + WIRE-DARK DISCIPLINE  [wiring-enforcement R10, production-caller gate]
  GOAL: prove the composed router works end-to-end on a HARD input through the REAL entrypoint, not per-unit happy paths.
  SEAMS: an INDEPENDENT smoke (auditor-authored, not the executor's) drives POST /headless/run AND a real chat turn with: a large-context objective (est > 260k -> must pick free-gemini 1M or PAID, never a 32k lane and never overflow), a tool-heavy hard objective (must clear the capability floor -> claude or a tool_reliable free lane), and a forced-failure objective (must escalate FREE->PAID bounded). Assert every new pub fn has a production caller (production_caller_check.py / python_wire_check.py). Every new default-ON flag has a wire-dark-allowlist row until the smoke is green.
  DONE-GATE: the R10 smoke passes on all three hard inputs; wire checkers clean; slop_semantic_check (TT/TV/DUP) clean; MR-1 (no file over 1000 lines — route_decider.py is a NEW focused module, model_pool/scheduler edits stay under boundary).

================================================================================
4. NEW / CHANGED FILES (grounded targets)
================================================================================
NEW  conductor/route_decider.py            — RouteContext/RouteDecision/RouteDecider + the 7 leg functions (TR-1..TR-6). Target < 500 lines (MR-1).
NEW  conductor/routing_ledger.py           — write/read the outcome-ledger rows (TR-5).
NEW  conductor/migrations/00X_routing_ledger.sql — the ledger table.
EDIT conductor/model_pool.py               — PoolModel capability fields + seeded roster values (TR-4). ~+15 lines of data.
EDIT conductor/worker_providers.py         — route_pool_for UNCHANGED (kept as leg 3); add nothing here except maybe a re-export.
EDIT conductor/chat_orchestrator.py:514    — build RouteContext + call RouteDecider (TR-2/1). Keep _is_escalation user-override.
EDIT conductor/headless_runner.py:113      — same RouteContext build for the autonomous lane.
EDIT conductor/scheduler.py:127            — record_worker_result: ledger write (TR-5) + failure/overflow escalation re-queue (TR-3) + dead-lane cache.
EDIT conductor/worker_adapter.py           — parse the base-cc overflow signal into the result dict the scheduler reads (TR-1 backstop input).
EDIT conductor/orchestration_api.py / CanvasMonitor.svelte — route-economics report + chip (TR-7).
EDIT .elai/gate + wire-dark-allowlist       — flags + the R10 smoke manifest (TR-8).

================================================================================
5. NON-GOALS / HONEST SCOPE BOUNDARIES (grounded-by-default)
================================================================================
N-01  NO decoding-level modality control. TEIKOKU drives base-cc; it cannot enforce XGrammar-constrained single-tool-call, CodeAct-as-imports, or FIM/DIFF emission (ELAI J-55 decoding legs, B-28/H-3). TR-6 ships only the PORTABLE subset (model choice + tool-surface reduction + system-note). Do not claim otherwise in any reason string or the monitor.
N-02  NO measured context-degradation curve (ELAI J-34). This plan uses each model's DECLARED window (already in the roster) with a static 0.8 safety fraction. The measured per-(model,lane) knee is a FUTURE MEASURE-FIRST unit needing a bench + fixtures. Using the declared window is honest and safe (fail-closed to smaller).
N-03  NO trained ML router (ELAI I-25 full). TR-5 builds the outcome ledger + a deterministic veto only. Training a router on it is future.
N-04  NO change to the auto-migration mechanism (P4-D) — it already works and is TEIKOKU's strong area. This plan is about CHOOSING the model for a task, not managing a session that grows; the two compose (TR-1 picks a big-enough window; P4-D migrates if it still grows).
N-05  NO free-pool quota/rotation rework (that is the existing free_api_worker + ELAI J-54 territory). This plan consumes the existing FREE_POOL_PREFERENCE order; it adds the size/capability/ledger GATES in front of it.
N-06  NO new dependency. All deterministic Python in the conductor; the ledger is SQLite via the existing db layer.

================================================================================
6. EXECUTION METHOD + GATES
================================================================================
- ONE Opus-1M executor implements TR-1..TR-7 in a single session (self-contained plan). TR-8 (the R10 integration smoke + wire/slop checks) is AUTHORED + RUN OFF-AGENT by the auditor, never by the executor (wiring-enforcement: an agent that writes its own smoke routes around the real path). If run under orchestrate/C-02 instead: context-compiler -> executor -> auditor, cap 2 parallel, agent-dispatch-preamble on every dispatch.
- Per-unit: producer + consumer + smoke in the SAME changeset (vertical slice). No new pub fn without a production caller (route_decider legs are all called by decide(); decide() is called by the two live dispatch sites — that IS the production wiring, prove it with python_wire_check.py).
- Gates: TEIKOKU gate spine (gate_runner) + slop_semantic_check (TT/TV/DUP) + production/python wire check + MR-1 file-size + zero-hex on any FE. Fail-closed.
- The R10 smoke (TR-8) is the definition of done for the whole plan: three HARD inputs (large-context, tool-heavy-hard, forced-failure) through the REAL entrypoints, asserting the ROUTE_DECISION/ESCALATION_PROMOTED SSB rows. Missing evidence = FAIL.
- ONE commit at the end (maintainer voice, no AI trailer, test counts in the message), owner-hold — do NOT push.

================================================================================
7. GROUNDING INDEX
================================================================================
Shipped TEIKOKU seams (read this session, ~/teikoku/conductor):
  worker_providers.py:74 route_pool_for (the role rule) / :177 stamp_context_window + resolve_window use.
  model_pool.py:64 PoolModel(context_window,migrate_at,vision_capable) / :100-154 the CONTROLLER+FREE_API+CODEX rosters (windows 32k..1M) / :227 resolve_slot / :256 resolve_window (P4-D).
  chat_orchestrator.py:514 dispatch (route_pool_for(role), no difficulty) / :191 _is_escalation (user-keyword) .
  headless_runner.py:113 autonomous routing (route_pool_for(role), no difficulty).
  scheduler.py:52 free_retry_cap/paid_retry_cap/attempt caps / :127 record_worker_result (the disposition seam for TR-3/TR-5).
Design source (the units ported): workflow/reasoning/ELAI-COST-MODEL-CONTEXT-PIPELINE-2026-07-04.md — Stage 4 (role->model, J-55 modality, J-51 floor, I-25 ledger), Stage 5 (context budget), Stage 8-9 (I-7 escalation, I-8 dead-end, H-50 lifecycle). Note the doc's own honesty: most ELAI units here are PLANNED/wire-dark; this plan ports the DESIGN, implemented on TEIKOKU's shipped conductor.
TEIKOKU context: workflow/handoff/teikoku-general-orchestrator.md (W6 free-worker routing = the rule this plan replaces; W8 cost ledger this plan ties into), teikoku-build.md (P4-C model_pool, P4-D migration), TEIKOKU-MASTER-PLAN.md (D-05 co-equal pools, D-06 done-authority, D-16 concurrency).
