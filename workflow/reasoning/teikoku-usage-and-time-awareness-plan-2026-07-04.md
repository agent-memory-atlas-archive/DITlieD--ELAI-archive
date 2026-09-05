TEIKOKU USAGE-AWARENESS + TIME-AWARENESS PLAN
Date: 2026-07-04
Scope: TEIKOKU only (host widget + WSL conductor + base-cc gate spine). No .elai_cc changes.
Author: main LLM (Opus), in-band, grounded-by-default.
Owner ask (verbatim intent): a Windows widget showing usage % for any Anthropic account;
a way for the model to check its OWN usage % on demand ("be aware of your usage"), for the
autonomous J-plan-onward orchestration AND the normal chat models, across all providers;
time awareness so models feel time passing; and a more detailed token-in/out + per-provider
cost view so a run can make itself more efficient.

================================================================================
0. GROUNDING LEDGER (strongest first; verified THIS session unless noted)
================================================================================
U1  Usage endpoint is REAL and returns the exact panel data. Live call this session with the
    default account's OAuth token:
      GET https://api.anthropic.com/api/oauth/usage
      Authorization: Bearer <claudeAiOauth.accessToken from .credentials.json>
      anthropic-beta: oauth-2025-04-20
    Response carries a limits[] array: {kind, group, percent, severity, resets_at,
    scope:{model:{display_name}}, is_active}. kind in {session (5h), weekly_all,
    weekly_scoped (per-model)}. Also top-level five_hour/seven_day {utilization, resets_at},
    extra_usage, spend. Measured: MAX account weekly_all=40 / weekly_scoped(Fable)=35 --
    byte-matches the /model panel the owner pasted. This is tier-1 (our own measured data).
U2  Both accounts' tokens are on disk in plaintext, isolated by CLAUDE_CONFIG_DIR:
    default C:\Users\user\.claude and dedicated C:\Users\user\.claude-elai. credentials shape:
    claudeAiOauth.{accessToken, refreshToken, expiresAt (epoch ms), scopes, subscriptionType,
    rateLimitTier}. (workflow/handoff/claude-multi-account-isolation.md.)
U3  Endpoint caveats (WebSearch, GitHub issues): the endpoint rate-limits (persistent 429,
    anthropics/claude-code#31021) -> poll infrequently (>=60-90s) + cache + degrade, never
    hammer. No cross-account/offline query exists officially (#44328/#45392) -> reading each
    account's own token locally is the only path, which is exactly what we do.
U4  Time awareness is ALREADY PARTLY BUILT in TEIKOKU. clock_stamp hook (UserPromptSubmit
    additionalContext injector) prepends [NOW date hm tz] / [DAY ... | session-start] /
    [DAY ... rollover] per .claude/rules/time-awareness.md; per-session state ~/.teikoku/
    state/clock; advisory, never blocks; committed e59ae07; frozen in the gate-manifest.
    (workflow/handoff/teikoku-gate-redesign.md 2026-06-24.) Plane-1 (time-as-context) exists
    at the day/now grain; the deltas + agent-time + Plane-2 control are the gaps.
U5  A cost ledger already exists on the monitor (W8). build_orchestration_snapshot carries a
    "cost" section from budget_counters run_cost:* (spent_cents, runs, ceiling_cents_per_run),
    fail-soft, surfaced as a chip in CanvasMonitor. (teikoku-general-orchestrator.md W8.) This
    is the seam to extend for per-provider token-in/out, not a new subsystem.
U6  Provider reality: only the Anthropic-subscription lanes (claude/base-cc on the two
    accounts) have an oauth/usage %. Free/devin/glm/codex lanes have NO subscription %; their
    "usage" is token-count + $ estimate, which lives in the cost ledger (U5). So "usage
    awareness for all models" is two data sources joined, not one.
U7  Design doctrine for time awareness already reasoned out in
    workflow/reasoning/time-awareness-design-2026-06-25.md: two planes (time-as-CONTEXT vs
    time-as-CONTROL), determinism-safe placement (volatile slot after the last cache
    breakpoint, OsClock-sourced, never a gate input), agent-time extension, notification-
    primary async-tool loop, autonomous-only wall-clock watchdog. That doc is the design; this
    plan is the TEIKOKU-scoped build sequencing of it.

================================================================================
1. WHAT IS ALREADY DONE (this session)
================================================================================
W-U1  USAGE WIDGET -- BUILT + LIVE-VERIFIED (both accounts).
      C:\Users\user\bin\teikoku-usage-widget.ps1  (WPF, always-on-top, frameless, SIGNAL-
        themed: near-black #0A0D0C + teal #14A78F, square marks, no red; the "hot" usage band
        brightens toward #35F1E2 instead of turning red, per DESIGN-TEIKOKU).
      C:\Users\user\bin\teikoku-usage.vbs  (flashless launcher; double-click / pin).
      C:\Users\user\bin\teikoku-usage-accounts.json  (auto-seeded: MAX -> .claude,
        ELAI -> .claude-elai; ADD A LINE to watch any other account = the "any account" ask).
      Behavior: per account reads .credentials.json, GETs the usage endpoint, renders Session
        5h / Weekly / Weekly-<model> bars with % + reset-countdown tooltip; 90s poll (429-safe)
        + manual refresh; drag to move, position persisted; degrades visibly on
        nologin/expired/ratelimited/401 (never silently re-auths -- token refresh stays Claude
        Code's job). -SelfTest prints the live snapshot + validates XAML; -Once prints text.
      VERIFIED: -SelfTest live, both accounts ok; MAX matches the owner's panel (40/35);
        ELAI(pro) reads clean; XAML parses. Host tooling in C:\Users\user\bin (NOT .elai_cc,
        NOT the workflow repo). Uncommitted / outside git like the other teikoku launchers.

================================================================================
2. THE PLAN (sequenced; each unit is a vertical slice: producer + consumer + smoke)
================================================================================

W-U2  MODEL-FACING USAGE SELF-CHECK ("be aware of your usage"). [next]
  Goal: any TEIKOKU model -- the autonomous orchestrator AND a normal chat turn -- can learn
  its own remaining headroom on demand and mid-run.
  Parts:
   a) conductor/usage_probe.py (NEW): the single source of truth. Given an account tag
      (which config dir / which lane), returns the parsed limits[] (session/weekly/scoped %)
      for Anthropic lanes, or the ledger token+$ figure (U5) for free/devin lanes. Caches
      >=60s per account (U3 429-guard); fail-soft (returns state=unknown, never throws).
      This is the Python twin of the widget's Get-AccountUsage -- share the endpoint constants.
   b) GET /usage route in slice_api.py (NEW): {accounts:[{label,tier,state,rows[]}],
      providers:[{lane,tokens_in,tokens_out,spent_cents}]}. The widget's PowerShell and the
      FE both can read this too (single seam).
   c) A base-cc UserPromptSubmit hook usage_stamp.py (sibling of clock_stamp), gated by a
      trigger: when the prompt contains an explicit "be aware of your usage" style directive
      OR when running autonomous, inject an additionalContext line:
        [USAGE MAX weekly 40% (resets 5d) | session 3% || free-lane 1.2M tok $0.00]
      Advisory, never blocks (same posture as clock_stamp). For the autonomous J-plan run this
      is the on-by-default path; for interactive it fires on the directive.
   d) A /usage control-plane verb (the "/" command set already exists per TEIKOKU-MASTER-PLAN
      D-17) that renders the same snapshot in-chat on demand.
  Smoke: a live turn with the directive shows the USAGE line sourced from the real endpoint;
    a free-lane turn shows tokens+$ not a fake %. Off-agent: assert the injected bytes match
    GET /usage at that instant.
  Grounding risk: token expiry mid-run -> the probe reports state=expired, the model is told
    "usage unknown (token stale)", NEVER a fabricated %.

W-U3  USAGE-AWARE ORCHESTRATION (the real point of the ask for the J-plan walk).
  Goal: the conductor throttles/defers Anthropic-lane (PAID) dispatch as the account's weekly
  window fills, so an autonomous run cannot burn the account to a hard limit unattended.
  Parts:
   a) conductor_loop scheduling reads usage_probe before leasing a PAID/claude task: soft
      thresholds (config): >=85% weekly -> stop leasing new PAID tasks, drain in-flight, prefer
      FREE lanes; >=95% -> PAID hard-pause (INFRA_GUARD-style HALT event, blocked_reason=
      usage_ceiling). Mirrors the existing W5 infra-guard control-plane pause pattern, reusing
      ControlManager.
   b) Emit a USAGE_THROTTLE event + surface it on the canvas cost chip (extends U5) so the
      throttle is visible, not silent.
   c) Respect the two-account role split already built (conductor.elai-opus.toml): when the
      Opus/Max account crosses a threshold, the retier path can fall back FREE without a manual
      switch. (Config-flag it; do NOT auto-flip silently without an event.)
  Smoke: a fixture usage_probe returning 96% weekly makes the loop refuse a PAID lease and emit
    USAGE_THROTTLE; FREE leases still flow. Determinism: the throttle is a supervisor-plane
    liveness decision (wall-clock/quota), NOT a replay gate input (per U7 ND-1).
  Note: this is the unit that makes "be aware of your usage" actually protect the account
  during the mostly-autonomous J-plan-onward walk, not just print a number.

W-U4  TIME AWARENESS -- close the gaps on the existing clock_stamp (U4) toward the U7 design.
  Plane 1 (time-as-context), cheap, extends the shipped hook:
   a) DELTA-SINCE-LAST-MESSAGE: clock_stamp currently does now + day-rollover; add
      "last message N ago" from the per-session ~/.teikoku/state/clock last-turn stamp.
   b) SESSION-RESUME ELAPSED: on a resumed chat, inject "elapsed since last session: N days"
      from the last stored turn stamp vs now (the reliable carrier for a multi-day gap, U7 IMP-4).
   c) AGENT-TIME (autonomous): during a multi-step autonomous turn, inject elapsed-since-turn
      and elapsed-since-last-tool on each re-entry ("turn 3m12s; last tool cargo build 2m,
      running"). This is the piece U7/G15 shows actually fixes tool-timing blindness; it is
      what lets the model decide to background/abandon a long tool.
  Plane 2 (time-as-control) is mostly already covered in TEIKOKU by W5 (30-min hard
  subprocess_timeout + stale-heartbeat reclaim + infra-guard) -- so for TEIKOKU this is
  EXPOSURE not invention: surface the watchdog fire back to the model as a structured
  observation ("tool X exceeded budget; killed; partial output") rather than a silent reclaim,
  and default the wall-clock kill to AUTONOMOUS-only (interactive keeps the human pace, U7 IMP-7).
  Placement rule (non-negotiable, U7 IMP-2): the volatile time/usage lines are additionalContext
  injected per-turn AFTER the cached system prompt -- clock_stamp already does this, so the
  W-U2 usage line rides the same seam and neither busts caching.
  Smoke: a 2-turn fixture across a simulated day boundary shows the delta + rollover; a resumed
  backdated session shows the elapsed header; an autonomous multi-tool fixture shows monotonic
  agent-time; a hung autonomous fixture is killed + reported, the same interactive one is not.

W-U5  DETAILED TOKEN-IN/OUT + PER-PROVIDER COST (self-efficiency view).
  Goal: the "more detailed token in/out and per-provider costs so it can improve/make itself
  more efficient" ask. Extend the W8 ledger (U5), do not build a parallel system.
   a) Capture tokens_in/tokens_out per turn per lane at the worker_adapter/chat_runner seam
      (base-cc reports usage; free-api proxy returns usage; devin bridge returns usage) into
      budget_counters alongside run_cost:*. Anthropic lanes can ALSO reconcile against the
      local base-cc JSONL (ccusage-style) for ground-truth in/out/cache tokens.
   b) build_orchestration_snapshot "cost" section gains per-provider {lane, tokens_in,
      tokens_out, cache_read, spent_cents} rows (fail-soft, never fabricated).
   c) Widget + FE cost chip + GET /usage (W-U2b) all read the same rows -> one number
      everywhere. The autonomous run (W-U3) can read tokens/$ per lane to prefer the cheaper
      lane when quality allows (a real efficiency lever, not a vanity meter).
  Smoke: a live turn on each lane lands a per-provider row with non-zero in/out; the sum
    reconciles with run_cost within tolerance.

================================================================================
3. SEQUENCING + WHY
================================================================================
Done:  W-U1 (widget) -- owner asked me to build it directly; shipped + live-verified.
Next:  W-U2 (probe + /usage + self-check hook) -- the shared seam everything else reads;
       smallest, unblocks the rest.
Then:  W-U5 (token/cost capture) in parallel with W-U4 (time gaps) -- independent seams.
Then:  W-U3 (usage-aware throttle) -- depends on W-U2's probe; it is the unit that makes the
       autonomous J-plan walk safe on the account, so it is the highest-value non-widget piece.
All of W-U2..W-U5 are conductor/base-cc work on the WSL canonical tree ~/teikoku, owner-hold
uncommitted like the rest of TEIKOKU, one maintainer-voice commit when the owner asks.

================================================================================
4. RISKS / CAVEATS (pre-implementation, top)
================================================================================
R-1  Endpoint 429 (U3): every reader (widget, probe, hook, throttle) MUST share one cached
     fetch per account with a >=60s floor; N independent callers hammering the endpoint will
     self-inflict the 429 the issue tracker documents. usage_probe.py owns the cache; the
     widget already self-limits at 90s.
R-2  Token expiry: readers surface state=expired and NEVER attempt the OAuth refresh flow
     (risk of clobbering the shared credentials file the two accounts + TEIKOKU sync depend on).
     Refresh stays Claude Code's job (multi-account-isolation handoff).
R-3  Determinism (U7 ND-1): usage % and wall-clock are supervisor-plane / advisory context only.
     No FSM transition, gate, or promotion may branch on them; they are recorded as model-input
     bytes, never re-timed on replay. The throttle (W-U3) is a liveness decision outside the
     seeded replay set.
R-4  Free-lane has no %: do not fabricate one. "usage awareness for all models" = % for
     Anthropic lanes, tokens+$ for the rest (U6). Honest "--" where no source exists.
R-5  Scope creep into a general cron/scheduler is DECLINED (I-53 ledger): stay per-turn /
     per-account / per-run liveness; no background poller beyond the widget's own timer and the
     probe cache.

================================================================================
GROUNDING / VERIFICATION
================================================================================
Built + verified this session: W-U1 widget (-SelfTest live, both accounts, MAX matches the
  owner's panel 40/35; XAML parses; launcher written). Endpoint shape is tier-1 measured data.
Not built (plan only): W-U2..W-U5 -- conductor/base-cc slices, sequenced above, each with a
  named producer + consumer + smoke. clock_stamp (U4) and the W8 ledger (U5) are the existing
  seams W-U4/W-U5 extend rather than rebuild. External caveats (U3) WebSearch-grounded to the
  claude-code issue tracker. Design doctrine is workflow/reasoning/time-awareness-design-
  2026-06-25.md (U7).
END.
