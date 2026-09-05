TEIKOKU SELF-TEST + AUTO-IMPROVE HARNESS — DESIGN PLAN

name: TEIKOKU-E2E-SELFTEST
status: v1 draft, 2026-06-28, authored from the live handoffs + the E2E portable guide + grounded conductor source reads
purpose: stand up an autonomous "fake-human + deterministic" tester that exercises EVERY TEIKOKU feature (skills, MCP, orchestration, the 3 worker lanes, gates, memory/RAG, auto-migration, the whole dashboard), turns each found defect into a fix task, fixes it with free worker agents, verifies the fix against a frozen off-agent gate, and commits — driven by the 2 codex accounts on the vision lane, running while the owner sleeps.
grounding: workflow/reasoning/E2E-TESTING-SYSTEM-PORTABLE-GUIDE.md (the machinery + porting checklist), workflow/handoff/teikoku-build.md (TEIKOKU is built P0-P5), workflow/handoff/teikoku-native-dashboard.md (dashboard wiring + launch story), workflow/handoff/teikoku-gate-redesign.md (the gate spine + commit sentinel), and direct reads of conductor/tester_lane.py + tester_vision.py (the 2 codex accounts + codex-vm transport).

================================================================================
0. THE ONE IDEA (do not lose this)
================================================================================
A tester that scores its OWN output rubber-stamps. So every verdict is fail-closed (missing evidence == FAIL), grounded in an artifact the tester did NOT author (AX role/name, a control.db journal row, a process exit code), and the gate the tester runs is FROZEN so the thing under test cannot weaken it. Paid model credits (codex/GPT-5.5 vision) are spent ONLY on the one thing nothing else can do: see a visual/interaction bug. Everything else is free and deterministic.

This is a RETARGET, not a rebuild. The conductor/ E2E machinery is app-agnostic and already proven on ELAI. We keep it almost verbatim and supply the five TEIKOKU-specific pieces the porting checklist names: the rubric file, the off-agent evidence source, the launch story, the deny-list/safe-list, the goldens.

================================================================================
1. LOCKED / GROUNDED FACTS (do not re-derive)
================================================================================
F-01  Machinery exists and is portable (conductor/, verified present): tester_sandbox, cdp_action_driver.mjs, cdp_driver, tester_vision (see->act loop + verify + safety guard), tester_runtime, vision_gate (pixel-diff-first), tester_lane (2 codex lanes), test_modes (4 deterministic modes), checkpoint_map (fail-closed loader), trigger_safety, tester_pass, tester_metrics.
F-02  The 2 codex accounts are real + wired (conductor/tester_lane.py:32-36): account-a -> /root/.codex-accounts/account-a, account-b -> /root/.codex-accounts/account-b, both inside WSL distro codex-vm. Transport (tester_vision.py:99-141): CODEX_HOME=<home> codex exec -i <png>, prompt on stdin, final agent_message parsed out of the JSONL. Lanes split a milestone round-robin (a=[0::2], b=[1::2]); a journey flagged `critical` runs on BOTH for a 2-vote cross-check.
F-03  Lane-loop integration is PARTLY WIRE-DARK (e2e-A4): TesterLane/LaneResult/default_lanes carry WIRE-DARK markers and are NOT yet consumed by conductor_loop.run_backlog. The daemon-stage hooks (dashboard_tester_stage / test_modes / autofix_stage behind default-OFF flags) exist per the guide; the lane pool wiring is a deliverable, not an assumption — the executor grounds the exact seam.
F-04  Test SUBJECT = TEIKOKU. Its testable surface is the SAME Svelte frontend the owner's Tauri exe embeds, served by vite :1420 and driven by headless Chrome over CDP (every audit in the handoffs already CDP-drives this frontend — proven). The native Tauri exe is the OWNER's UI; the tester never needs it. Off-agent evidence = the TEIKOKU control.db event journal (SSB) + the AX tree.
F-05  TEIKOKU backend is up via systemd in the teikoku distro: conductor :8787, free-proxy :4000, devin-bridge :8810. WSL2 forwards localhost to Windows. control.db lives under ~/teikoku.
F-06  codex-vm is the P6-deferred standalone-conductor distro. HARD GUARD: using its codex ACCOUNTS via `codex exec` for vision is read-only and SAFE. Its conductor SERVICE must NEVER be started (one start launches a real wave on the live 383-task ELAI backlog). The tester calls `codex exec` directly; it never touches the codex conductor daemon.
F-07  TEIKOKU has a gate spine with a commit sentinel (teikoku-gate-redesign): commit_gate (PreToolUse) ALLOWS a git commit/push only when ~/.teikoku/approvals/commit.sentinel is present; commit_consume (PostToolUse) deletes it on success. reflect_gate roadblocks every action without visible reasoning. This is the safety floor the overnight loop runs inside.
F-08  TEIKOKU is its own git repo (github.com/maintainer/teikoku); current main shipped through P5 (f037e46); a large amount of dashboard + orchestrator work is UNCOMMITTED in the WSL tree (owner hold). The selftest harness commits to THIS repo, never to /mnt/f/.ELAI_workflow (the 4a4df4f stray-commit incident is the cautionary tale).

================================================================================
2. ARCHITECTURE — WHERE EVERYTHING RUNS
================================================================================
  +-----------------------------------------------------------------------+
  | TESTER ORCHESTRATOR (python)  -- the daemon loop + lanes + gate runner |
  |   home: a TEIKOKU-targeted instance of the conductor/ tester           |
  +-----------------------------------------------------------------------+
     | (1) launch app          | (2) drive UI         | (3) vision call
     v                         v                      v
  teikoku distro            headless Chrome        codex-vm distro
  vite :1420 + backend      (CDP, pinned capture)  CODEX_HOME=account-b
  :8787 / control.db SSB    AX tree + screenshot    codex exec -i <png>
     ^                                                     |
     | (5) fix lands + frozen gate verifies + commit       | (4) raw vision text
     |                                                      v
  free worker agents (devin-free / free-api via the bridge+proxy) = the HANDS

Three boundaries cross WSL distros: the app lives in `teikoku`, the eyes live in `codex-vm`, the orchestrator + headless Chrome run Windows-side (the existing tester_runtime idiom). This is exactly the cross-distro shape the F: tester already runs for ELAI — we point a NEW target config at TEIKOKU instead of disturbing the ELAI tester.

DECISION (harness home, recommended): a TEIKOKU-targeted instance derived from the proven F:\.ELAI_workflow\conductor\ tester (a target-config + the 5 retarget pieces), NOT a fresh port into ~/teikoku/conductor. Rationale: the machinery + the cross-distro codex transport are already proven there; porting into the teikoku distro re-opens solved problems (cross-distro codex from inside teikoku, a second chrome/bun toolchain) for no test-coverage gain. The harness STILL tests teikoku and commits to the teikoku repo; only the tester process lives F:-side. (Overridable — see Section 8 fork 2.)

================================================================================
3. THE ISOLATION CONTRACT (hard owner requirements — fail-closed)
================================================================================
OWNER DECISION (2026-06-28): codex SEES AND FIXES teikoku code. Codex is fed the teikoku harness + dashboard source (it needs it to fix what it saw). The boundary is an ALLOWLIST-minus-DENYLIST mediated by the harness — codex never gets raw repo access, only files the harness serves it, and the harness only serves teikoku-source files that pass the denylist.
I-00  THE ALLOWLIST. The fix lane may serve codex ONLY teikoku harness/dashboard source: ~/teikoku/{conductor, signal, base-cc grafts, .claude, scripts} and the tester harness itself. Everything else is denied. The harness copies the allowlisted file(s) into codex's scratch, codex returns a patch, the harness applies it in the teikoku tree, the frozen gate verifies, the daemon commits to the branch. Codex is given files, not a filesystem.
I-01  DENYLIST: codex NEVER sees the elai copy or the litegame/embervow copy. A path denylist (fail-closed) hard-excludes the elai copy + litegame/embervow copies under ~/teikoku and anywhere on /mnt/*. The vision lane stays source-free regardless (goal + rubric + AX list + crop only) so SEEING leaks nothing; the FIX lane is gated by the allowlist∩denylist so FIXING can only touch teikoku source. A probe that resolves to a denied path returns "no evidence", never content.
I-02  DENYLIST: codex NEVER gets the apis / keys / secrets. The same denylist hard-excludes .env / .liberty / credentials.toml / free-proxy .master-key / the OAuth token / any *.key|*.pem|*.credentials. Belt-and-suspenders: every outbound fix-prompt is scanned for a key-shaped string before dispatch and refused if one appears; the codex-vm account auth is codex's own and is never teikoku's.
I-03  TEIKOKU OPERATES ONLY ON CLEAN TEST-TARGET REPOS. When a journey makes TEIKOKU act ON a project (orchestrate a plan, run a worker, run a pack gate), the target project is a downloaded free OSS repo registered in a test-only projects.toml — NEVER the elai/litegame copies. This is what lets us exercise the full orchestration/worker/gate machinery without the codex eyes (or a free fix-worker) ever touching the real project copies. Pick small, license-clean repos that hit each pack: one small Rust crate (rust-pack: cargo+wiring+maintainability gates), one small TS/Svelte app (dep-cruiser/knip + dashboard gates), one small Python CLI (python_wire_check + generic gates). (Unity pack left to a later pass — it needs the native Editor bridge, out of tonight's scope.)
I-04  THE TESTER NEVER MUTATES THE REAL PROJECTS. Triggers are read-only by default; mutating/billable commands are behind a CLI safe-list + a dashboard destructive-action deny-list (Run cloud task / spawn / commit / push / delete / reset / pay / billing). The vision safety guard resolves the smallest AX element under a proposed click and refuses the journey if its name/role matches the deny-list. The deterministic lane only walks read-only CLI leaves.

================================================================================
4. THE PHASES (the overnight build — orchestrated, opus agents, C-02 cycle)
================================================================================
Each sub-phase = context-compiler (sonnet/haiku-ok for read-heavy gathering) -> executor (opus) -> auditor (opus, the sole gate runner), max 2 parallel, agent-dispatch-preamble pointer on every dispatch, append to the handoff. Phase 0 first (risk-burn); later phases depend on it.

--------------------------------------------------------------------------------
PHASE 0 — RISK-BURN + ISOLATION SCAFFOLD  (prove the load-bearing integrations FIRST)
--------------------------------------------------------------------------------
0.1  Codex vision liveness. Confirm account-b (primary) and account-a in codex-vm each answer a real `CODEX_HOME=... codex exec -i <png>` with parseable text. Owner focus = B; A is the parallel/2nd-vote lane. HARD GUARD F-06: never start the codex conductor service.
0.2  App drivability + pinned capture. Launch the teikoku frontend (vite :1420, backend already systemd-up) and CDP-drive headless Chrome: navigate, screenshot (deviceScaleFactor=1, fromSurface=true, --font-render-hinting=none, motion killed, 2 rAF settle), AX tree. One real screenshot + AX list of a real teikoku screen.
0.3  Off-agent evidence source. Locate the teikoku control.db event journal; build the watermark+action_type reader (the make_production_ssb_probe analog for teikoku). Confirm a real journal row can be read off-agent after a driven action.
0.4  Isolation boundary (I-01/I-02). Implement the path denylist + the key-shaped-string scan + wire them into the (future) fix-lane + any context pack. Prove it: a probe of .env / the elai copy / litegame copy returns "no evidence", not content.
0.5  Clean test-target repos (I-03). Download 2-3 small license-clean OSS repos (one Rust, one TS/Svelte, one Python), register them in a test-only projects.toml, skip heavy artifacts. Prove teikoku's project_resolver resolves a test repo + its pack with ZERO elai/litegame reachable.
done(0): a live codex vision call returns text; a pinned screenshot+AX of a real teikoku screen; an off-agent journal row read; the denylist blocks a secret/elai/litegame probe; clean test repos registered.

--------------------------------------------------------------------------------
PHASE 1 — THE TEIKOKU CHECKPOINT MAP  (the rubric — the bulk of retargeting)
--------------------------------------------------------------------------------
1.1  Author workflow/reasoning/TEIKOKU-TEST-CHECKPOINT-MAP.json against TEIKOKU-CURRENT-STATE.md (22 slice_api routes, 8 toggles, 7 screens, 18 tauri commands) + the gate-manifest + the skills/MCP registries. Milestones -> features; each feature: feature_id, name, surface(cli|dashboard|both|internal), trigger{cli_argv|dashboard interaction string}, expected_effect (a REAL observable: an AX role/name OR a named control.db action_type row), rubric (one line), status (aside, never a skip reason — spec-eval posture: every feature is treated as real).
1.2  COVERAGE (the user's "test everything"): skills (per-skill toggle actually loads the SKILL.md into the turn), MCP (each wired MCP server/tool answers), orchestration (grill scope-gate, /spawn depth/fanout/$10 ceiling, drop-in, auto-migration zero-info-loss, the worker lanes claude|free|devin routing), the gate spine (commit sentinel, reflect roadblock, scope_ack, destructive-deny), memory/RAG (event log + semantic recall), every dashboard surface (chat/sessions/orchestration-monitor/config/memory-timeline + full state coverage empty/loading/error-inversion/offline/stale). Each maps to a real route/action/journal row — if a feature has no off-agent observable, it degrades to AX-only and is flagged, not faked.
1.3  Wire the fail-closed loader (load_checkpoint_map raises on a malformed map) + the two selectors (dashboard_journeys, cli_features) for the teikoku map.
done(1): the map loads fail-closed; dashboard_journeys + cli_features return non-empty; every expected_effect names a verified-real observable; coverage spans all the user-named feature classes.

--------------------------------------------------------------------------------
PHASE 2 — RETARGET THE HARNESS  (keep machinery, supply the 5 glue pieces)
--------------------------------------------------------------------------------
2.1  Launch story: a teikoku DashboardRuntime (start vite/backend, headless Chrome, isolated user-data-dir, stop tears down only what it started).
2.2  Evidence source: the teikoku control.db reader (from 0.3) as the production ssb-probe factory.
2.3  CLI runner + safe-list: locate the teikoku CLI binary; a trigger_safety safe-list of read-only teikoku CLI leaves; the destructive dashboard deny-list (I-04).
2.4  Lanes: wire the 2 codex lanes for teikoku (focus B), critical journeys 2-vote. Goldens: capture ~the key teikoku screens as known-good PNGs for the pixel-diff-first gate (the main credit saver).
2.5  Deterministic lane (free, every pass) retargeted to teikoku: metamorphic/differential CLI (same cmd twice == same action_type signature modulo nonce; --json vs human agree on success class; idempotent cmd adds no new row), arg-fuzz (malformed flags fail cleanly, never crash; launcher-side failures SKIPPED not counted), journal invariants (schema_version monotonic, action_type in teikoku's golden lock set, seq strictly increasing, no orphan rows), replay-verify.
done(2): one full vision pass over teikoku dashboard journeys yields pass/fail/unknown grounded in AX/journal; one deterministic pass yields findings; isolation holds (no key/elai/litegame leakage in any prompt); pixel-diff gate skips an identical frame with zero model call.

--------------------------------------------------------------------------------
PHASE 3 — THE FLYWHEEL  (find -> fix -> verify -> commit, autonomous)
--------------------------------------------------------------------------------
3.1  Resolve the e2e-A4 wire-dark: integrate the tester lanes + the test-modes pass + the autofix stage into the daemon loop (conductor_loop run_backlog_waves) behind config flags, fail-soft (a tester crash never breaks the loop) — and add the daemon-path integration test (test_daemon_tester_lanes_integration.py analog) so a refactor can't silently kill the lanes (the guide's section-9 lesson).
3.2  Defect -> FIX task: autofix_stage seeds an idempotent FIX task per defect, DEDUPED by command/element (one bug -> one root cause, not one-per-feature). The HANDS are CODEX (owner decision: codex sees AND fixes): the lane that saw the defect is handed the allowlisted teikoku source file(s) (I-00, denylist-filtered) + the defect evidence and returns a patch; the harness applies it in the teikoku tree. Free worker agents (devin-free/free-api) are SECONDARY/overflow fix capacity for when codex credit is throttled or for bulk deterministic-lane findings. Vision still uses the pixel-diff-first gate so credits are not wasted on identical frames.
3.3  Fix -> verify: the FROZEN wiring/gate verifies the fix off-agent (the tester never authors the gate it runs; the gate is in the freeze manifest). A fix only flips to VERIFIED on off-agent evidence.
3.4  Verify -> commit: a verified fix commits to the teikoku repo in maintainer voice (no AI trailer), gated by the commit sentinel (F-07). Commit POLICY for the unattended run is Section 8 fork 3.
done(3): the proven flywheel runs end-to-end on teikoku — a real defect is found, fixed by a free worker, gate-verified, and committed, with no human in the loop and codex spent only on vision.

--------------------------------------------------------------------------------
PHASE 4 — LIVE AUTONOMOUS RUN  (overnight, monitored)
--------------------------------------------------------------------------------
4.1  Kick the daemon loop with the 2 codex lanes + free fixers, bounded: a codex-credit budget + a max-pass cap + the pixel-diff-first gate (vision sparingly, deterministic every pass). All journeys operate on the clean test repos (I-03); the dashboard-self journeys (chat/config/etc) operate on the teikoku UI itself (read-only + safe-list).
4.2  Tripwire set (breaks autonomy, surfaces to the monitor/owner): an irreversible op proposed, a gate BLOCK that loops, an isolation breach (a denied path or a key-shaped string reaching a prompt), a codex-credit stall, the codex-vm conductor service being touched (must be never).
done(4): the loop runs unattended for the session, the monitor sees find/fix/verify/commit counts + credit spend, and every tripwire is honored.

================================================================================
5. GOVERNORS (the part that makes it safe to leave running)
================================================================================
- Pixel-diff FIRST, vision SECOND: an identical-to-golden frame is a free pass, no model call (the single biggest credit saver).
- Vision sparingly (periodic / per-pass end), deterministic EVERY pass (free, catches the recurring regression classes).
- Fail-closed everywhere: missing evidence == FAIL; unknown == FAIL; a dead evidence path screams.
- Fail-soft daemon: every tester capability behind a flag, swallowed on error, so the tester can never take the loop down — BACKED by the daemon-path integration test so fail-soft can't silently hide a dead lane.
- Frozen gate: the verifier is read-only to the thing under test (freeze manifest), so a fix run can't rewrite its own evaluator to pass.
- Injectable edges: model transport / browser / journal / CLI are injected callables — the whole thing is unit-testable with zero credits / zero live processes.
- Data minimization: source-free prompts + outbound/inbound guard-scan (doubly important with a third-party codex proxy).
- One bug -> one root cause: dedup defects by command/element, not by feature.
- DO NOT rebuild a single-shot LLM acceptance judge (the guide killed it: it only emits unknown=fail noise). Acceptance = contract tests; liveness = the deterministic lane; paid credits = vision only.

================================================================================
6. HOW THE OVERNIGHT BUILD IS ORCHESTRATED (my role)
================================================================================
- I am the ORCHESTRATOR (main-llm-role): decompose, dispatch, read returns, decide forks, enforce rules, verify off-agent, append the handoff. I do not write the harness code myself.
- OWNER OVERRIDE (2026-06-28): FEW BIG OPUS AGENTS, MAX 1 AT A TIME, SEQUENTIAL. Opus 4.8 has a 1M context, so ONE agent eats a WHOLE PHASE (gather its own context + execute), not a sub-phase. Target ~6-8 sequential opus runs total: one big executor per phase (P0,P1,P2,P3) + a separate opus AUDITOR at the two integration gates (after P2 harness-complete, after P3 flywheel-complete) so the builder never certifies itself. Never 2 agents at once.
- Every dispatch: the agent-dispatch-preamble.md pointer line; scope = the teikoku tree + F:\conductor tester only; HARD bans = no touching the live ELAI .elai_cc except to read, no starting the codex-vm conductor service, git read-only for the worker (the orchestrator commits at phase boundaries), commit target = the selftest branch only.
- I (orchestrator) commit at phase boundaries via PowerShell->wsl git on the selftest/auto-fixes branch (workers stay git-read-only per the preamble). The harness's OWN overnight fix-commits are the daemon's, auto-committed to the same branch via the commit sentinel (fork 3 = branch).
- Memory: append to workflow/handoff/teikoku-selftest.md (one topic-keyed handoff) + a run-state file.

================================================================================
7. MONITORING RUNBOOK (the "another opus" monitor plugs in here)
================================================================================
The monitor opus (a separate session) watches, never drives:
- tester pass counts + per-journey pass/fail/unknown (tester_metrics + the event journal)
- defects found / fix tasks seeded / fixes VERIFIED / commits landed
- codex credit spend per lane (budget vs spent)
- the tripwire log (Section 4.2)
- isolation audit: grep the outbound-prompt log for any denied path or key-shaped string (must be zero)
It escalates to the owner ONLY on a tripwire. Everything is in the handoff + the event journal + the harness log; the monitor reads those, it does not touch the FSM or the daemon.

================================================================================
8. DECISIONS (RESOLVED with owner 2026-06-28 — ground truth, do not re-open)
================================================================================
FORK 1  Codex role = SEE AND FIX. Codex finds defects (vision) AND authors the fixes (handed allowlisted teikoku source per I-00). Free workers are secondary/overflow. (Sections 3 + 4.3 updated.)
FORK 2  Harness home = a TEIKOKU-targeted instance derived from the proven F:\conductor tester, pointed at the teikoku dashboard (least friction; commits to the teikoku repo).
FORK 3  Commit policy = AUTO-COMMIT to a dedicated branch selftest/auto-fixes of the teikoku repo. main stays owner-controlled; the owner merges in the morning.
FORK 4  Tonight = BUILD phases 0-3 + RUN the full unattended phase-4 autonomous find/fix/commit loop straight through the night (against the clean test repos + the teikoku UI itself, read-only + safe-list).
FLEET  = few big opus agents, MAX 1 AT A TIME sequential (~6-8 total), the orchestrator (me) drives. See Section 6.
