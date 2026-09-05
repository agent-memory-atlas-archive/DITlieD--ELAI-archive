F2-emitter-gap-decisions.md

Session: s-22300 (Tranche A follow-up fixes)
Date: 2026-05-31
Status: DEFERRED with registered TODO marker (no half-wire)

Context

Tranche A F2 was queued as "D-17 SLA tick main-loop splice" carry-over from s-21636. The s-21636 work landed the helper run_refusal_sla_tick + dispatch_refusal_sla_check inside task_runner.rs with #[allow(dead_code)] until callers wire onto it. F2 was meant to inline the call site into the main task-runner loop, gated by ELAI_REFUSAL_SLA_TICK_ENABLE.

What was actually shipped in Tranche A

ONLY a registered TODO marker at .elai_cc/crates/orchestrator/src/task_runner.rs:4066-4075. No code emitted. The TODO references babystep id F2 (s-22300) per stub-tracking registry pattern and explicitly cites both blockers.

Decision: defer F2 inline rather than half-wire

Two blockers prevent honest wire-up:

1. No SSB-backed production emitters. The only RefusalRateSnapshotEmitter and RefusalSlaBreachEmitter implementations live at .elai_cc/crates/orchestrator/src/refusal_sla.rs:453 and refusal_sla.rs:488 and are Vec<T>-backed test doubles. There is no SsbBackedRefusalRateSnapshotEmitter or SsbBackedRefusalSlaBreachEmitter impl on disk.

2. No plumbing. ResolvedConfig has no monitor or emitter field today. dispatch_refusal_sla_check takes &impl RefusalRateSnapshotEmitter + &impl RefusalSlaBreachEmitter directly, not an Arc<Ssb>. The task-runner main loop has no path to materialize the emitter pair from ResolvedConfig.

Half-wire path rejected

The naive splice — wire the Vec emitters into the loop behind the env gate — would compile, would run, and would silently drop every SSB row the SLA tick produces. The Vec sinks have no flush-to-disk path. Tracing-only mirror is also unacceptable because the downstream consumers (RefusalSlaBreachPanel, D-17 rollup) read SSB, not log lines. Half-wiring violates the anti-spiral rule and the dashboard-fed observation contract from D-17 decisions.

Resolution

The TODO marker is registered in workflow/sessions/s-22300/stubs.md with will_resolve_in = TBD-future-emitter-impl. Inline splice unblocked when either:

a. An SsbBackedRefusalRateSnapshotEmitter + SsbBackedRefusalSlaBreachEmitter impl ships at refusal_sla.rs (mirrors the SsbBacked* pattern used by 5+ existing emitters in the same crate), AND ResolvedConfig grows an Arc<Ssb> field accessible from the task-runner loop. OR

b. dispatch_refusal_sla_check is refactored to take Arc<Ssb> directly and construct ad-hoc Ssb-writing emitters internally, mirroring the pattern at .elai_cc/crates/orchestrator/src/durability/observability.rs notify_* helpers.

Both paths are non-trivial (touch refusal_sla.rs public surface, ResolvedConfig struct, task_runner.rs constructor sites, and ~4 contract tests). Out of scope for Tranche A small-fix posture.

Grounding

CLAUDE.md HARD RULE: Zero Lazy Code via registered stub registry exemption (stub-tracking.md).
CLAUDE.md HARD RULE: anti-spiral; silently dropping SSB rows would re-trigger F2 the next time someone audits the dashboard and finds the SlaBreach panel empty.
Tranche A audit verdict at workflow/sessions/s-22300/audit/TRANCHE-A-AUDIT.md SECTION 2 PER-FIX VERDICT F2: PASS with WARN-soft for stubs.md row addition (this doc + the stubs row close that WARN).

Next session unblockers

F2 stays open. Pick up alongside any plan that touches refusal_sla.rs or ResolvedConfig (likely D-17 follow-on or any SLA-adjacent E-plan). The decision doc above is the design-time guardrail; the stubs row is the registry pin.

RESOLUTION (s-22600 Tranche B ITEM B.1)

Status: SHIPPED 2026-05-31. F2 closed via path (a) -- SsbBacked* emitter adapters ship at refusal_sla.rs alongside the trait + Vec test doubles; ResolvedConfig left untouched (P2 path from context-package, minimum-diff). Inline tokio::spawn lives in run_task_runner immediately after replay_backlog, gated by refusal_sla_tick_enabled().

What landed:
- refusal_sla.rs: SsbBackedRefusalRateSnapshotEmitter + SsbBackedRefusalSlaBreachEmitter (owned Arc<Ssb> + i64 superstep; native async-fn-in-trait via Pin<Box<dyn Future>>); REFUSAL_SLA_AGENT_ID="task_runner" const; tracing::warn on Ssb::write_payload err (mirrors retry_emit warn pattern); use crate::ssb::{Ssb, SsbPayload} import added.
- task_runner.rs (was L4097-L4106 TODO[F2]): replaced with `if refusal_sla_tick_enabled() { tokio::spawn(...) }` block; constructs RefusalRateMonitor::with_defaults + two SsbBacked emitter adapters + run_refusal_sla_tick at REFUSAL_SLA_TICK_INTERVAL_SECS using &*cancel.wake for cooperative shutdown.

Wire-dark: default OFF until operator sets ELAI_REFUSAL_SLA_TICK_ENABLE truthy. SsbActionType::RefusalRateSnapshot + ::RefusalSlaBreach pre-existed (no SCHEMA bump). RefusalRateMonitor is local-only (no upstream observe() producer wired yet, by design -- the periodic tick surfaces a structural healthy-band row each interval which is what the dashboard sparkline expects pre-D-16 closure wire). When D-16 decide_and_emit consumers start feeding observe() the rows will reflect real refusal traffic.

Grounding for resolution:
- Emitter shape: task_runner.rs:10383-10446 TaskRunnerConformal*Emitter pair (canonical SSB-adapter precedent the explorer flagged).
- Wire-dark gate semantics: refusal_sla_tick_enabled() at task_runner.rs:12286 honours ELAI_REFUSAL_SLA_TICK_ENABLE without panic-on-unset.
- Trait surface preserved: RefusalRateSnapshotEmitter + RefusalSlaBreachEmitter signatures untouched, Vec* test doubles still compile.
- stubs.md row F2 flipped to RESOLVED prefix (not deleted, per fixer protocol).
