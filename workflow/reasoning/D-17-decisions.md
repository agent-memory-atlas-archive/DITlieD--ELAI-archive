D-17 honest-refusal SLA monitoring: design decisions

Plan: docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md
Code: .elai_cc/crates/orchestrator/src/refusal_sla.rs (types + producer + read-surface)
      .elai_cc/crates/orchestrator/src/agent_pool.rs:2922 (dispatch_refusal_sla_check)
      .elai_cc/crates/orchestrator/src/task_runner.rs:13680 (run_refusal_sla_tick), :13656 (refusal_sla_tick_enabled)
      .elai_cc/crates/orchestrator/src/ssb.rs:21049 RefusalRateSnapshotPayload, :21076 RefusalSlaBreachPayload (READ-ONLY here)
Off-agent artifact: production rows refusal_rate_snapshot (every tick) + refusal_sla_breach (on breach),
      written via SsbBackedRefusal*Emitter -> Ssb::write_payload into ssb.db; SSB round-trip golden at ssb.rs:31987-32000.
Spec: docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.6 (lines 288-296).
Grounding: arXiv:2503.01345 (UniCR — Unified Conformal Refusal, formal Mode E surface).

This doc records the three design decisions the plan calls for (section 9: band, metric scope,
alert path). It captures the rationale of the SHIPPED implementation, not the pre-implementation
plan text, and flags where the two diverged so the divergence is not re-litigated.

Decision (a): healthy band defaults

Chosen: healthy_band_lo = 0.05, healthy_band_hi = 0.20, target_rate = 0.10
(refusal_sla.rs:81 DEFAULT_HEALTHY_BAND_LO, :86 DEFAULT_HEALTHY_BAND_HI, :75 DEFAULT_TARGET_RATE).

Rationale. The band is anchored to the UniCR (arXiv:2503.01345) epsilon = 0.1 operating point: a
correctly calibrated conformal-refusal policy abstains on ~10% of claims, so the target sits at
0.10 and the healthy band brackets it on both sides. A rate below the 0.05 floor means the policy
is confabulating (abstaining too rarely, the "Mode E is the easy route" failure mode reads the
other way: a model that never abstains looks productive while answering claims it should refuse).
A rate above the 0.20 ceiling means the system is over-abstaining: checkers misconfigured, or the
task suite is outside the calibration range. Both are structural failures the monitor is meant to
surface; see refusal_sla.rs:78-86 doc comments.

Divergence from plan text (not re-litigated). The plan (section 9(a), lines 88-94) proposed a
broader 5-50% band "pending empirical tuning per UC-5; allow Mode E to range during early
sessions." That 5-50% was a pre-implementation guess. It was superseded at implementation time by
the UniCR-grounded 5-20% band because (1) a 50% ceiling is not a quality gate — at 50% the system
is refusing half of everything and "looks honest while delivering nothing" (refusal_sla.rs:15-20),
which is exactly the gaming the SLA exists to catch; (2) the UniCR epsilon = 0.1 operating point
gives a principled center to bracket, where UC-5's 10-30% was itself a guess; (3) the ceiling
stays operator-tunable via RefusalSlaConfig::healthy_band_hi (refusal_sla.rs:164), so an operator
who wants the wider early-session range sets it explicitly rather than shipping a permissive
default. The narrower default is the conservative, fail-loud choice; widening is a deliberate
operator override, not the out-of-the-box behavior. The empirical-tuning intent of the plan is
preserved through RefusalSlaConfig, not through a permissive default.

Decision (b): sliding-window metric, caller-scoped (not a single global aggregate)

Chosen: the metric is a bounded sliding window of the most recent window_size = 100 decisions held
in one RefusalRateMonitor instance (refusal_sla.rs:70 DEFAULT_WINDOW_SIZE, :235-240 VecDeque, :283
observe with FIFO eviction). The monitor is Send + Sync with no interior mutability; the caller
chooses how many monitors to run and what each scopes over (refusal_sla.rs:42-44, :231-233).

Rationale. Refusal rate is only meaningful over a recent, bounded window: a lifetime cumulative
rate washes out drift (a model that confabulated for 1000 claims then started refusing healthily
would still read "below band" forever under a global counter). The sliding window means an
early-session snapshot already has enough samples to be informative without staying anchored to
ancient history (refusal_sla.rs:67-69). Contexts vary across sessions and task families, so a
per-instance window lets the caller scope the metric to whatever boundary surfaces drift best —
spawning one monitor per session yields the session-level band tracking the plan section 9(b)
called for, while a single shared monitor gives a rolling global view. The implementation does not
hard-bind to "session" in the metric core; it leaves scoping to the caller (the MVP keeps a single
global VecDeque per monitor, refusal_sla.rs:42-44). Aggregate cross-session analytics and the
rendered gauge are the dashboard's job (D-27 RefusalRateGauge, read via recent_snapshot_rows /
recent_breach_rows at refusal_sla.rs:672, :700), not the monitor's — the monitor produces rows, the
dashboard aggregates them.

Alternative considered: a single global running counter (cheaper, one i64 pair). Rejected because
it cannot distinguish recent drift from historical noise and cannot be scoped to a session/task
family without re-introducing per-scope state anyway — at which point the sliding window is the
simpler primitive and the bounded VecDeque caps memory at window_size regardless of session count.

Decision (c): breach surfaced as an off-agent SSB row (the wire signal an alert subscribes to),
not a silent log

Chosen: every snapshot emits one refusal_rate_snapshot SSB row, and a breach (health_band Below or
Above) emits one additional refusal_sla_breach SSB row (refusal_sla.rs:327-387 emit_snapshot;
ssb.rs:21049 / :21076 payloads). The rows land on the canonical substrate via
SsbBackedRefusal*Emitter -> Ssb::write_payload (refusal_sla.rs:567, :611). The breach row is the
durable, action-required signal — not a tracing::warn that scrolls past.

Rationale. An SLA breach is action-required: if the rate is below floor the model is confabulating
and promotion should freeze; if above ceiling the checkers are likely misconfigured and an operator
needs to look. The plan section 9(c) chose Bubble-Up alert "not silent log" for exactly this. A
tracing log is observational and ephemeral; an SSB row is a durable, queryable, off-agent artifact
that the dashboard reads back and that a breach-counter can apply RefusalSlaConfig::
alarm_threshold_breaches (default 3 consecutive, refusal_sla.rs:91) against to suppress transient
noise before alarming.

Wiring note (not re-litigated). The SHIPPED MVP makes the refusal_sla_breach SSB row the wire
signal and defers the Bubble-Up alert dispatcher itself to a follow-up that subscribes to that row
(refusal_sla.rs:39-46 out-of-scope note; agent_pool.rs:2918-2921 "downstream alert wiring
(Bubble-Up) is the caller's concern"). This is the same row-first / consumer-subscribes shape every
other D-wave beacon uses, and it satisfies the "not a silent log" requirement at the producer
boundary: the breach leaves a durable artifact regardless of whether the alert consumer is wired
yet. emit failures are warn-and-continue (refusal_sla.rs:355-363, :376-384) so a transient SSB
write error never masks the snapshot result.

Config decision: in-code defaults are the active configuration; elai_cc.toml [refusal_sla] stays
commented

The elai_cc.toml [refusal_sla] section (lines 928-934) is intentionally left commented. The
commented values (window_size=100, target_rate=0.10, healthy_band_lo=0.05, healthy_band_hi=0.20,
alarm_threshold_breaches=3) match the in-code DEFAULT_* constants exactly (refusal_sla.rs:70-91),
and refusal_sla.rs:152-154 documents the section as OPTIONAL — the monitor falls back to the
DEFAULT_* constants when the section is absent. Uncommenting the section would not activate the
feature: activation is the ELAI_REFUSAL_SLA_TICK_ENABLE env gate (wire-dark default OFF,
task_runner.rs:13656 refusal_sla_tick_enabled, :13637 env const), which controls whether
run_refusal_sla_tick spawns the periodic loop in run_task_runner (agent_pool.rs:4581-4594). The
toml only re-states the same defaults. Therefore the in-code DEFAULT_* constants are the intentional
active configuration and the toml stays commented: editing it provides no behavioral change, and
the plan-text band (5-50%) must NOT be copied into the toml because it is stale versus the shipped
UniCR-grounded band (5-20%). An operator who wants to override does so by uncommenting and setting
explicit values, which is the documented escape hatch, not the default.

Status note: D-17 ships wire-dark (tick gated behind ELAI_REFUSAL_SLA_TICK_ENABLE). The flag needs a
wire-dark-allowlist.toml row (type=kill_switch, owner, expiry_date, promotion_plan_id) and the D-27
dashboard gauge to promote it out of wire-dark; both are out of scope for this decisions doc and
tracked against the D-17 status file at
workflow/sessions/s-plan-audit-2026-06-04/plan-status/D/D-17_honest_refusal_sla.md.
