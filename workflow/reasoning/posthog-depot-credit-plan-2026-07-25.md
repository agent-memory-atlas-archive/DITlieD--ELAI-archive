POSTHOG + DEPOT CREDIT DEPLOYMENT PLAN FOR ELAI
written 2026-07-25. sources: posthog.com/startups + docs (fetched today), depot.dev/pricing + docs (fetched today), the Depot support email from Pedro Guerra (2026-07-23), elai-status.json / proof-spine-state.json / plan-registry.json, ORCHESTRATION-GUIDE.md, 5-SYNTHESIS / 6-RUNNABLE-NOW / 7-BENCHMARK-READINESS-PLAN, the B..Q summary-index extracts, ElaiStatus.svelte.
this is a strategy/ops doc, not an executable corpus plan. any unit promoted into the walk gets authored wargame-grade and must pass .claude/tools/plan_gate_check.py before execution.

GOAL MAPPING (the three asks > the workstreams below)
finish the implementation  > WS-2 walk observability, WS-3 CI on Depot, WS-4 confined bench host
find and fix bugs          > WS-1 dashboard instrumentation, WS-2 walk observability, WS-5 self-driving + MCP
polish the dashboard       > WS-1 evidence-driven polish queue, WS-5 scouts and prompted PRs

WHAT WE HAVE (verified today)
- PostHog for Startups: $50,000 credit, valid 12 months from application, spendable across every PostHog product and add-on (BAA excluded). unused credit expires, then standard billing with the normal free tier. source: posthog.com/startups + handbook.
- PostHog self-driving: their agent loop (scouts > signals > reports > sandboxed coding agent > GitHub draft PR). reports free, first 3 PRs each month free, $15 per completed PR after, refundable when the report or fix was wrong, default $150/mo spend ceiling. whether startup credits offset the PR line item is not publicly stated, confirm in billing before scaling it.
- Depot perk: $5,000 via code POSTHOGPARTNER26. mechanics per Pedro Guerra (Depot support): Stripe checkout shows 100% off the first prorated month and never mentions the $5,000, the backend grants the credit balance after checkout, every invoice draws from that balance until empty, credits valid 1 year from activation, the anti-abuse flag on the account is cleared.
- Depot Startup plan: $200/mo, drawn from the balance. included monthly: 5,000 Docker build minutes, 20,000 GitHub Actions runner minutes, 20,000 Depot CI minutes, 250 GB cache storage, unlimited users. usage rates beyond that: GHA $0.004/min at 2 vCPU (larger sizes are clean multiples, 16 vCPU = 8x), Docker builds $0.04/min, Depot CI $0.00005 per vCPU-second, agent sandboxes $0.01/min, cache $0.20/GB/mo. so $5,000 = the year of plan ($2,400) + roughly $2,600 of overage headroom on top of quotas we are unlikely to exhaust.
- repos are GitHub-attached already: elai > github.com/maintainer/.ELAI_workflow (the land spine pushes there), teikoku monorepo incl SIGNAL > github.com/maintainer/teikoku. Depot runners and PostHog self-driving can hook both today.

WHAT THE CREDITS DO NOT BUY
- model tokens. no Anthropic/OpenAI/Novita spend is covered. the owner ruling of 2026-07-18 already pinned the benchmark generator to the free lane (glm-5-2 bridge, local :8181), so nothing on the critical path needs paid tokens anyway.
- human testers, a QA lab, or design taste. PostHog analyzes evidence that you, Playwright, CI, or the walk generate.
- authority. SSB, RRA, registry.json and proof-spine-state.json remain the source of truth. a PostHog row is an advisory projection and never discharges a done-gate, the same boundary proof-spine-state.json already declares for itself.
- the P1-TIER2 build itself. the single most load-bearing piece of remaining work (the model-in-loop harness that unblocks PS-1, qdelta-live and PS-4-live) is engineering time in this repo. the credits remove the infra excuses around it (confined host, CI, receipts), they do not write it.

WHERE ELAI STANDS (the targets this money must serve, from the SoT today)
- backend 10.5% verified (58/551 units), 30.3% claimed. the drain engine is the conductor walk, and its documented weak points are exactly observability-shaped: silent 66-minute turns, a novel provider error string hard-halts the whole ladder, gate-set completeness (ORCHESTRATION-GUIDE upgrade surface items 1-6).
- frontend 46%: wiring health 92% but ux keep-rate 0, every view flagged redo. feeder health counts 17 orphan components, 4 unreachable, 5 dead ipc invokes, 16 unused handlers.
- benchmarks 10/26 with artifacts, proof spine 2/6 measured, both measured numbers are the model-free ones. critical path per 7-BENCHMARK-READINESS-PLAN: P0-HARDEN corpus > P1-TIER2 harness > W1 kill-gate on the free lane. the one infra item it needs is a confined Linux host to execute untrusted model-authored patches.

WS-1  SIGNAL DASHBOARD: INSTRUMENT, FIND BUGS, THEN POLISH ON EVIDENCE
- instrument SIGNAL with posthog-js: autocapture, error tracking, masked session replay, plus a small hand vocabulary (screen_opened, letter_locked, panel_error, walk_strip_used, bench_row_opened, status_stale_shown). properties: build sha, screen, panel, duration, error_class. never objective text, plan titles, paths, or code.
- two approaches for the capture path, pick by observable:
  (a) posthog-js direct in the webview. SIGNAL runs under WebKitGTK (Tauri), so apply the known fixes up front: import posthog-js/dist/module.full.no-external, set crossOrigin=anonymous on stylesheet links before init plus a MutationObserver for injected styles (PostHog/posthog issue 12259 workaround), confirm the CSP allows *.posthog.com. one dependency, no Rust.
  (b) route events through the backend (the tauri-plugin-better-posthog pattern: before_send returns null, Rust posts). heavier, immune to the silent-drop reports in Tauri v2 (posthog-js issue 1760).
  start with (a). the fork trigger: if a test event is not visible in PostHog activity within 60s from the packaged app, switch to (b).
- replay privacy: mask everything data-bearing (objective strings, punch list rows), record chrome and navigation. review the first sessions by hand before trusting the config.
- traffic source, honest: nobody uses elai today (the walk is its only user) and the owner opens this dashboard occasionally, not daily. so replay/error volume stays thin until a Playwright journey suite generates deterministic traffic on a schedule: open each screen, lock a letter chip, open bench rows, drive the panels, force a known error. the corpus already plans this class of test (C-7 visual.spec, E-12 browser E2E). computer-use operators (P-12) are the heavier later option, not needed to start. human dogfood replay becomes a real source only once elai is usable enough that someone operates it.
- polish loop: replays + dead/rage clicks + the frontend error groups become the ranked queue for the ux redo (which of the 220 components gets reworked first). the design work itself follows design-taste + visual-ui-loop rules, PostHog supplies the queue and the before/after numbers (error rate per view, dead-click rate, panel render p95). the 5 dead ipc invokes and 17 orphans from the feeder health leg are the first fix batch, verify the counters drop in the next snapshot.

WS-2  WALK OBSERVABILITY: THROUGHPUT FOR FINISH-THE-PLANS
- the 493 unverified units get drained by the walk, so walk downtime is the finish-date slipping. today a stall is invisible until someone tails state files. PostHog gives dashboards + alerts over events the conductor already journals.
- two approaches:
  (a) inline capture calls at the conductor seams (turn start/done, dispatch, land, classify_failure). simple, but touches the hot path and every call needs fail-open discipline.
  (b) an exporter daemon that tails conductor/state/control.db (the journal already carries FALLBACK_TURN_START/DONE, OPUS_WORKER_DISPATCHED/DONE, OPUS_LAND_*, WATCHDOG_FIRE) with an event_seq cursor and forwards sanitized projections. zero risk to the walk, replayable, can backfill history.
  take (b). add (a) only as python exception capture in the supervisor so an uncaught crash is grouped in error tracking.
- properties: provider, model, plan letter + id, failure_class, rc, duration, token counts where worker_telemetry/usage_meter already have them. never prompt text, worktree paths, diff content.
- alerts, matching the recorded 3am deaths: unexpected failure-class (the hard-halt), no turn_done for N minutes while queue nonempty (the silent wedge), WATCHDOG_FIRE, disk guard. the walk dashboard shows turns/day, land rate, failure classes by provider, cooldown time, autoseed spins that land nothing (the spinning trap).
- this is the cheap version of ORCHESTRATION-GUIDE upgrade item 6 (turn observability) and it feeds item 4 (failure-class coverage) with real frequency data.

WS-3  ELAI CI ON DEPOT RUNNERS: GATES AND BENCH RECEIPTS OFF THIS MACHINE
- the corpus already plans GitHub CI (C-15 windows.yml, C-21 anti-stale, C-27 wiring-audit.yml, E-41 PR review, K-3 github_pr adapter already wired, K-11 PR evidence check-runs). stand the workflows up in .ELAI_workflow with runs-on: depot-ubuntu-24.04-8, Depot cache + sccache for cargo so runs finish in minutes.
- jobs:
  1. on push: cargo check + clippy + the frozen checker set (production_caller_check.py, wiring contracts, plan_gate_check on touched plans) rendered as a check-run. K-11 makes this the distribution leg of the wedge later, deterministic hash-bound evidence in the PR.
  2. nightly: the WAVE-0 fixture bench receipts (gate-tax, routing-economics, case-reuse, safedelta static, seccodeplt, canary-regression, crane, sequence-probe...) with artifacts uploaded as SoT, plus a weekly PS-6 wire-dark sweep. summaries also export to PostHog as trend points with an alert on regression.
- capacity: included quotas (20k GHA min + 20k Depot CI min + 5k Docker min monthly) dwarf this workload even at 8-16 vCPU sizes. the $2,600 remainder exists for bench campaigns, not for worry.
- guard: CI must not become an authority the local frozen checkers are not. a red Depot day never blocks the walk, the same checkers run locally.

WS-4  THE CONFINED EXEC HOST (P4) ON DEPOT COMPUTE
- P1-TIER2s dynamic exploit scorer executes untrusted model-authored patches, and the ABORT rule forbids doing that unconfined on this workstation. Depot supplies three shapes:
  (a) a GHA job on a Depot runner: ephemeral VM, destroyed after the job. right for scheduled kill-gate and publishable runs.
  (b) Depot CI sandboxes via depot ci run: drivable by an agent against the local tree without commit/push, depot ci ssh for inspection, per-second vCPU billing. right for the P1 development loop.
  (c) a local container: free, but it puts untrusted exec back on the machine the rule protects.
  take (b) for development, (a) for runs. RECON before building: does the scorer need KVM/firecracker (E-15 lineage) or does a container jail suffice, and if firecracker, verify Depot runner virtualization support first. settling check: grep confinement/sandbox in crates/ for the reusable primitive, then one depot ci run smoke.
- the free-lane generator (glm bridge, :8181) runs from here. the Depot job receives only the stripped corpus + candidate patches + the scorer, so no model keys ever enter the sandbox.
- Depot registry + container builds: bake the P0-HARDEN corpus tarball and scorer into an OCI image pinned by digest. that is the J-91 OCI-pinned re-exec environment story, and it strengthens the RRA environment-pin claim for the eventual published number.
- observable: a Depot job executes a known-leaky fixture patch inside the jail, the escape test is BLOCKED, the artifact lands back in the repo.

WS-5  SELF-DRIVING + MCP: A BOUNDED EXTRA PAIR OF HANDS
- PostHog MCP into Cursor/Claude sessions (npx @posthog/wizard mcp add): query error groups, replays, trends mid-fix. free to connect.
- once WS-1 data flows for ~2 weeks, enable exactly two scouts on the SIGNAL project: dashboard-crash regression + broken-journey. inbox reviewed weekly, PRs stay drafts, human merge only, ceiling stays at the $150 default until the credit-coverage answer lands.
- repo scope decision: the narrowest GitHub app grant on maintainer/teikoku (SIGNAL lives there), or mirror signal/ out to its own repo if the grant feels too broad for the monorepo. the elai product repo can follow later, an outside agent PR there passes the same frozen gates as any walk worker or it does not land.
- the walk stays the primary implementation engine. self-driving is for the evidence-backed dashboard-maintenance class of work, flat $15 per PR, refundable.

WS-6  PRODUCT TELEMETRY STANCE (do not poison the wedge)
- ELAI the product ships zero-telemetry by default and M-plan sells that (fleet rollup is journal-computed, zero telemetry). PostHog lives in: dev/dogfood builds behind a default-off [telemetry.posthog] config, the SIGNAL ops dashboard, the conductor, CI, the website. never a customer seat by default.
- PostHog feature flags only for dev-surface rollouts. an execution-affecting flag follows the freeze rule (evaluate once at run start, journal the snapshot, hash into the RRA) or it does not exist.

LETTER-PLAN HOOKS (where the corpus already planned for this, per the B..Q extracts 2026-07-25)
- PostHog-shaped plan units: E-29 (OTel GenAI spans + transcript/failure/spend drawers, PostHog can be the sink), H-9 (cost cockpit, currently reads all-zeros dark spend), J-28/J-29 (usage ledger + live meter, shipped, the natural event source for cost projections), K-2/K-4 (routing-economics + gate-tax product metrics), L-10 (hallucination-delta), G-5 (reviewer DSR study), E-49 (coding-agent metrics tuple), B-39 (REFLECT observability v0), E-42 (svelte error-rate telemetry), P-17 (failover + billed-vs-estimate).
- Depot-shaped plan units: C-15/C-21/C-27 (windows.yml, anti-stale CI, wiring-audit.yml), C-54..C-59 + E-9 (flagship bench campaigns, the "RunPod" labels are legacy, CPU runners serve them), B-29.5/B-31.5 (multi-hour scale-indexing runs), C-11/E-15/E-22 + P-12/P-24 (sandbox/firecracker/container isolation legs), J-56 (eval corpus CI), J-68 (cargo kani + TLC on the critical core, heavy compute), J-91 (OCI-pinned re-exec environments > Depot registry), Q-6/Q-8/Q-9 (gate-kill-rate, verifier-size lint, command-registry CI), K-3/K-11 (github check-run evidence, K-3 adapter already wired).
- read: when the walk picks observability or CI plan units, these are the ones this credit plan compounds with, prioritize accordingly.

BUDGET MATH (honest)
- PostHog: solo-dev usage (dogfood analytics, replay, errors, walk events, bench summaries) plausibly lands in the $0-300/mo band, much of it inside free allowances. the $50k will not be burned down and does not need to be. its value is: everything on, no sampling, long retention, self-driving and AI features usable without meter anxiety, for a year.
- Depot: $2,400 of the $5k is the plan year. quotas dwarf expected use. the ~$2,600 remainder funds big-runner bench campaigns (16 vCPU overage $0.032/min, that is ~81k minutes if it ever came to it) and agent-sandbox minutes. capacity is not the constraint, engineering time is.
- unchanged constraint: live bench legs use the free lane per the owner ruling, paid walk lanes stay under existing caps. no new money required by this plan.

CLOCKS
- PostHog credit: 12 months from application, already running. Depot credit: 12 months from activation, first drawing invoice 2026-08-01. both expire worthless, so the wiring (WS-1..4) belongs in the first 30-60 days and the consumption spreads over the year.

RISKS AND GUARDS
- privacy: only sanitized projections leave the machine. no prompts, no code, no paths, no objective text, masked replay. hand-audit the first week of captured events.
- coupling: no gate, done-decision, or product feature may depend on PostHog/Depot availability. exporters fail open, CI mirrors local checkers instead of replacing them.
- Tauri capture flakiness is documented upstream, WS-1 carries the fix and the 60s observable before any dashboard data is trusted.
- self-driving GitHub access: narrowest grant, drafts only, low ceiling, confirm credit coverage of the $15 items early.
- local gate hooks deny some shell shapes (a /dev/null redirect was refused today), exporter code and CI scripts should not assume devnull.

FIRST MOVES (ordered, each with its observable)
1. finish the Depot checkout with POSTHOGPARTNER26 > Stripe shows the prorated month 100% off, and within a day the org billing page shows a ~$5,000 balance (Pedro: granted post-checkout). if it is missing before 2026-08-01, reply on the support thread before the first invoice draws.
2. in PostHog billing, confirm the $50k balance and ask support whether self-driving PR charges draw from startup credits > a written yes/no, ceiling set accordingly.
3. branch, then instrument SIGNAL (wizard or hand-init with the Tauri fixes) > a test event and a masked replay visible in PostHog activity within 60s from the packaged app.
4. add exception capture to SIGNAL error paths and the conductor supervisor > a forced test exception appears grouped in error tracking.
5. write the control.db exporter (event_seq cursor, sanitized, fail-open) + the walk dashboard + 2 alerts (hard-halt, walk-silent) > the test alert fires.
6. stand up .github/workflows/ci.yml in .ELAI_workflow on Depot runners with sccache + Depot cache > a green check-run on a real push, minutes visible in Depot usage.
7. nightly bench workflow producing the WAVE-0 receipts + PostHog trend export > first nightly artifact and its trend point.
8. RECON the scorer confinement need (firecracker vs container jail) > a one-line verdict recorded in the P1 planning file, then the depot ci sandbox job for the exploit scorer > fixture patch executes jailed, escape test BLOCKED.
9. connect PostHog MCP in Cursor > a chat query answers with yesterdays SIGNAL error groups.
10. after 2 weeks of data: enable the two scouts, review the first inbox, let at most one PR through the full loop > merged fix, its error signature at zero through the soak window.

ASSUMPTIONS RECORDED
- SIGNAL here means the teikoku status dashboard (the ElaiStatus screen path the owner gave), which is distinct from elai's own in-product Tauri dashboard (B/H plans). the owner named the signal app as the polish target. the same instrumentation pattern applies to the product dashboard later, once Playwright or a human drives it. signal lives in the teikoku monorepo and runs in a WebKitGTK webview via teikoku-desktop.sh, if it actually runs as a plain browser tab the Tauri caveats simply do not apply.
- no human uses elai today, the walk is its only operator. all near-term signal volume is machine traffic (walk events, CI, benches, Playwright), human dogfood is a later phase, and WS-1 is written to that reality.
- letter-plan statuses quoted from the B..Q extracts were read from the summary indexes this session, not re-verified row by row against the registry.
- self-driving GitHub app permission scopes were not inspected, do that at install time.
- PostHog AI-observability SDK specifics were not pinned here, the exporter starts with plain capture events, the AI-event mapping is a later refinement.
