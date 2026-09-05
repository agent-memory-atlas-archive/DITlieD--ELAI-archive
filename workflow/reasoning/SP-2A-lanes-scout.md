SP-2A LANES SCOUT (unit 2.1, TEIKOKU-ELAI-FINISH-PLAN.md lines 118-137 / D2,D4 lines 24-32)
session_id=s-finish-0705. all paths below are on the teikoku WSL distro unless marked F:\.

1. PAID LANE CONFIG (conductor/config/conductor.elai-opus.toml, EXISTS, 93 lines)
Current slot table (all account_id="claude-elai", pool="claude", model_tier="PAID", max_concurrency=1):
  opus-exec   model claude-opus-4-8[1m]  roles ["EXECUTE"]              (line 54-62)
  opus-wire   model claude-opus-4-8[1m]  roles ["WIRE","INTEGRATE"]     (line 64-72)
  opus-audit  model claude-opus-4-8[1m]  roles ["AUDIT"]                (line 74-82)  <- delete, this is the D2 delta
  sonnet-rest model claude-sonnet-5      roles ["POLISH"]               (line 84-92)  <- extend roles
D2 delta as a diff (only change needed in this file):
  --- a/conductor.elai-opus.toml
  @@ -73,20 +73,11 @@
  -[[workers]]
  -slot_id = "opus-audit"
  -harness = "claude"
  -model = "claude-opus-4-8[1m]"
  -model_tier = "PAID"
  -account_id = "claude-elai"
  -max_concurrency = 1
  -pool = "claude"
  -roles = ["AUDIT"]
  -
   [[workers]]
   slot_id = "sonnet-rest"
   harness = "claude"
   model = "claude-sonnet-5"
   model_tier = "PAID"
   account_id = "claude-elai"
   max_concurrency = 1
   pool = "claude"
  -roles = ["POLISH"]
  +roles = ["AUDIT", "POLISH"]
"+rest" needs no extra roles entry: STAGE_ROLES is the closed 5-value set (EXECUTE,WIRE,INTEGRATE,AUDIT,POLISH per retier_elai.py:11), and EXECUTE/WIRE/INTEGRATE are already claimed by the opus slots, so AUDIT+POLISH on sonnet-rest already covers 100% of "everything else". leases.py:158-164 role affinity: a slot with a non-empty roles list serves ONLY those roles (NULL/empty = serves all) -- confirmed this is how the affinity match works, no ambiguity.
worker_cap=4, daily_paid_attempt_cap=500 already set (conductor.elai-opus.toml:31,38) -- no change needed there. Model-id NOTE (file header line 13) is still open per WAVE 0.1 (owner must confirm claude-opus-4-8[1m]/claude-sonnet-5 are what the upgraded Max account actually serves) -- out of scope for 2.1, flagging as a dependency.

2. FREE LANE (conductor/config/conductor.elai-walk.toml, EXISTS, 118 lines) + DEVIN BRIDGE
Current: 4 slots devin1..devin4, harness="devin-free", model="glm-5-2", model_tier="FREE", pool="free", proxy_base_url="http://127.0.0.1:8810" (lines 69-103). [limits] worker_cap=2 (line 33, comment: pinned from 3->2 on 2026-07-05 for a cargo-build MEMORY reason, unrelated to devin rate limits).
DEVIN BRIDGE EXISTS: ~/teikoku/devin-bridge/proxy/{devin_proxy.py,devin_wire.py,account_catalog.py} is a live FastAPI/uvicorn service on :8810. This is NOT a stub -- it is the SAME service currently serving glm-5-2. NO PORTING TASK: model_pool.py:120-126 already carries "swe-1-6" as a first-class POOL_DEVIN_FREE roster entry (PoolModel("devin-free","swe-1-6",150_000,120_000,label="devin swe-1-6",note="bridge :8810",tool_reliable=True)), and model_pool.py:285-286 confirms "the bridge build_request prefers [slot.model] over DEVIN_PROXY_MODEL" -- i.e. changing the TOML's `model` field is sufficient, the bridge already passes swe-1-6 through per-session (devin_wire.py:243 confirmed set: {"kimi-k2-7","kimi-k2-6","glm-5-2","swe-1-6","SWE-1.5","swe-check"}).
worker_adapter.py:604-606 dispatch requires harness stays "devin-free" and proxy_base_url stays set -- neither changes.
2.1b diff (apply to all 4 slots; shown once):
  --- a/conductor.elai-walk.toml
  @@ slot devin1..devin4
   harness = "devin-free"
  -model = "glm-5-2"
  +model = "swe-1-6"
   model_tier = "FREE"
   max_concurrency = 1
  -pool = "free"
  +pool = "devin"
   proxy_base_url = "http://127.0.0.1:8810"
GAP FOUND (not asked, load-bearing): pool="free" does not match any workers.toml table. pool_concurrency.py:78-82 (load_pool_envelope docstring, confirmed) states pool keys mirror workers.toml table names verbatim: "[devin]" -> pool key "devin". ~/teikoku/workers.toml has exactly 8 tables ([devin],[free_api.groq/pollinations/cerebras/gemini/mistral/openrouter],[codex]) -- there is NO "[free]" table. So today pools_at_capacity() / effective_pool_caps() (pool_concurrency.py:162-172, 100-119) silently find no envelope for pool "free" and never enforce the measured devin cap at all. It is currently masked only because global worker_cap=2 already keeps concurrency under the unenforced ceiling. The pool="devin" fix above is required for the cap-3/drop-to-2 mechanism (next paragraph) to mean anything.
"cap 3, drop to 2 on rate-limit": workers.toml:11-17 [devin] already carries max_concurrency=3 (owner-MEASURED) and fallback_concurrency=2 ("drop to 2 if a continuous run later hits a rate limit") plus continuous_untested=true. BUT grep across conductor/*.py for "fallback_concurrency" returns ZERO hits -- it is a documented-only field, nothing consumes it. Rate-limit reaction that DOES exist (worker_adapter.py:749 classify_rate_limit -> scheduler.py:271 budgets.mark_model_cooldown) is a per-MODEL 300s cooldown, not a pool-concurrency change. The only live lever for an actual "drop to 2" is the owner-facing pool_caps.py::set_pool_cap("devin", 2) (POST /config/pool-caps route), which is a MANUAL action, floor-clamped so it can never exceed workers.toml's 3 (pool_caps.py:10-14, "devin 3; 4 = a 3h ban"). So 2.1b can ship the STATIC side (worker_cap reflecting the measured 3, or staying at 2) but "drop to 2 on rate-limit" is not automatic today -- it is either a manual runbook step or a new automation (scheduler.py wiring classify_rate_limit -> pool_caps.set_pool_cap) that is out of scope for a config-only sub-phase. Recommend documenting this as a runbook note, not implying automation that does not exist.
worker_cap fork: raising conductor.elai-walk.toml's worker_cap from 2 toward 3 to match the devin envelope directly conflicts with the SAME file's own 2026-07-05 comment (line 29-32) that 2 was chosen for a cargo-build memory/disk reason (7GB target dir per worker), not a devin-rate-limit reason. WAVE 1's 1.5 vhdx-move-to-D: is still OPTIONAL/pending. Do not raise worker_cap past 2 without confirming that headroom landed.
escalation_workers "esc" (codex, model_tier=PAID, lines 110-117) is untouched by 2.1b and is Wave-3 scope, noted only because daily_paid_attempt_cap=0 in this same file (line 40) disables budgets.py::_paid_escalation_enabled() (line 515-520) -- adjacent, not part of 2.1.

3. MODEL_TIER VOCABULARY + BUDGETS
No enum type anywhere; model_tier is a plain string column, conventionally "FREE"|"PAID" (uppercased at conductor_config.py:209, WorkerSlotConfig default "FREE"). The only place two literal values are enforced is retier_elai.py's argparse `choices=["FREE","PAID"]` (line 16) -- there is no third value in use anywhere in either lane config (escalation_workers reuses "PAID").
leases.py:156 `if slot["model_tier"] != row["model_tier"]: continue` -- straight string-equality match between a task row's model_tier and a candidate slot's model_tier; this is the entire routing mechanism (plus role-affinity at :158-164, pool-exclusion at :168-174, and the pin/window check at :182 `_slot_can_serve`).
budgets.py:518 `_paid_escalation_enabled`: daily_paid_attempt_cap<=0 disables FREE-lane mid-task escalation-to-paid (this is the codex "esc" mechanism, not the dedicated opus lane). conductor.elai-opus.toml's own daily_paid_attempt_cap=500 (line 38) is unrelated and already sufficient.
WHAT ACTUALLY MUST CHANGE for a paid-lane walk to dispatch (this is the real seam, more load-bearing than budgets.py): conductor/elai_walk_driver.sh:13 hard-codes `CFG=config/conductor.elai-walk.toml` as a plain assignment (not `${CFG:-...}`), which unconditionally clobbers any exported CFG env var from a caller. conductor.elai-opus.toml's OWN header comment (lines 9-11) already assumes this is overridable ("nohup bash elai_walk_driver.sh (with CFG pointed at this file)") but the script as written does not support that today. Required one-line fix:
  --- a/elai_walk_driver.sh
  @@ -13
  -CFG=config/conductor.elai-walk.toml
  +CFG="${CFG:-config/conductor.elai-walk.toml}"
Secondary, same file, line 22: `--limit-per-wave 2` is also hard-coded (comment ties it to worker_cap=2 for the free/mem-fix reason). If the opus lane (worker_cap=4) reuses this driver unmodified it will only lease 2 of its 4 configured slots per pass. Recommend the same parametrization: `LIMIT_PER_WAVE="${LIMIT_PER_WAVE:-2}"` and pass `--limit-per-wave "$LIMIT_PER_WAVE"`. Without one of these two fixes, retiering tasks to PAID and launching elai_walk_driver.sh unmodified will still run everything against the FREE/devin config file -- the retier alone does not change which conductor config process is running.
restart-conductor.sh (repo root) is unrelated: it only restarts slice_api.py (the dashboard/API on :8787) with TEIKOKU_WIN_HOST=127.0.0.1; it does not select a conductor config and is not the walk loop.

4. TEIKOKU_AUTONOMOUS PROPAGATION
Set unconditionally in worker_adapter.py::_worker_env at line 591 (`env[AUTONOMOUS_ENV_VAR] = "1"`, AUTONOMOUS_ENV_VAR="TEIKOKU_AUTONOMOUS" per line 62), BEFORE any harness branch (codex :592, claude :598, devin-free :604, free-api :610 all come after and each does `dict(base_env)`-style carry-forward per the inline comment at :588-590). Confirmed: every slot, every harness, including the claude-lane opus-exec/opus-wire/sonnet-rest slots (account_id=claude-elai), gets TEIKOKU_AUTONOMOUS=1 today. Backed by tests/test_worker_usage_capture.py:86-89 ("W2: every spawned worker session carries TEIKOKU_AUTONOMOUS=1"). 2.1c's ask is ALREADY SATISFIED in code; no fix needed, only a confirmation smoke (spawn one opus-lane task, inspect its child env for TEIKOKU_AUTONOMOUS=1) recommended at the WAVE-2 gate rather than any code change.

5. retier_elai.py (conductor/retier_elai.py, 34 lines, read in full)
CLI surface: `--tier {FREE,PAID}` (required), `--db` (default the live control.db). No `--letter`/`--wave`/`--plan-prefix` flag of any kind.
What it does: `UPDATE tasks SET model_tier=? WHERE status!='DONE' AND role IN ('EXECUTE','WIRE','INTEGRATE','AUDIT','POLISH')` -- a single blanket UPDATE across the ENTIRE non-DONE backlog, scoped only by role (excludes self-test tasks, per its own docstring), never by plan/letter/wave. Cannot scope to a subset today.
Consequence: `--tier PAID` flips EVERY non-DONE stage-role task to PAID in one shot; there is no way to retier "just wave 4" or "just the C/D/E hard tasks" without either (a) adding a --letter/--plan-prefix WHERE clause, or (b) relying on WAVE 1's M1 precheck routing having already resolved most RECOGNIZE/certify-class work to DONE first, so that what remains non-DONE when this runs is already mostly genuine greenfield (the plan's own Wave-1-before-Wave-2 sequencing covers this reading, but it is a dependency, not a guarantee -- if M1 has not fully run, --tier PAID will also flip FREE-appropriate certify-class remainder onto the paid lane). Flagging as a design dependency on Wave 1 landing cleanly, not a hard blocker for 2.1 itself.

RISKS SUMMARY (severity order)
R1 (real, silent today): devin pool="free" vs workers.toml key "devin" mismatch means the measured 3-cap safety net is currently unenforced; must fix pool= alongside the model= swap in 2.1b, not just the model id.
R2 (real, blocks the stated goal): elai_walk_driver.sh's hard-coded CFG means "flip tier to PAID and launch the driver" does not by itself run the opus lane; the driver needs the one-line parametrization (or a second driver script) as part of 2.1, or WAVE-2's live smoke will fail for a reason unrelated to the lane configs themselves.
R3 (expectation mismatch): "drop to 2 on rate-limit" reads as automatic in the plan prose; the codebase has the field (workers.toml fallback_concurrency) but zero consuming code -- ship this changeset as static-config + a runbook line, do not claim automated backoff exists.
R4 (unrelated collision risk): worker_cap 2->3 on the free lane reopens the 2026-07-05 disk/mem incident this same file documents fixing, unless 1.5 (vhdx move) has landed -- check before raising it.
R5 (low, scope note): retier_elai.py has no wave/letter scope; fine if Wave 1 (M1 precheck) runs first and drains RECOGNIZE-class work to DONE before --tier PAID runs, otherwise it over-flips the backlog.

STATUS: DONE
FILES TOUCHED: F:\.ELAI_workflow\workflow\reasoning\SP-2A-lanes-scout.md (created, this file). No other writes; task is read-only research.
VERIFIED: every citation above is a direct Read of the named file/line on the teikoku WSL distro (\\wsl.localhost\teikoku\home\teikoku\teikoku\...) or a grep with visible zero/nonzero hit count (fallback_concurrency: 0 hits confirmed; TEIKOKU_AUTONOMOUS, devin swe-1-6 roster, pool table names: nonzero hits shown above). Nothing in this file is training-memory conjecture.
QUESTION: none -- no blocking fork for the scout itself. Two decisions are left for SP-2A's executor/owner: (a) worker_cap 2 vs 3 on the free lane (R4), (b) whether the CFG/limit-per-wave parametrization in elai_walk_driver.sh ships inside 2.1 or as a fast-follow before the WAVE-2 gate smoke.
NOTES: conductor.elai-opus.toml needs only the opus-audit-deletion + sonnet-rest roles diff (section 1). conductor.elai-walk.toml needs model + pool per slot (section 2). elai_walk_driver.sh needs the CFG line (section 3) for the paid lane to be reachable at all.
