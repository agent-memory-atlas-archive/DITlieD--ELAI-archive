# D-17: Honest-refusal SLA monitoring

Objective: track session metric `refusal_rate = refusals / total_claims`;
auto-flag if <5% (model confabulating) or >50% (checkers misconfigured OR
task suite too hard). Structural defense against "Mode E is the easy route"
gaming. Real-problem §4.6 (lines 288-296): "treat honest 'i-don't-know'
rate as a first-class quality metric next to accuracy."
Estimated effort: 1 session.
Pre-requisites: D-16 (RefusalDecision events emitted). spec-grounder verifies.
Parallel-safe with: D-18 (ObservableEffect; orthogonal).

## Context

Real-problem §4.6 (lines 288-296). Without an SLA, the system can game Mode E
by routing everything to refuse. UC-5 (real-problem §8): 10-30% healthy band
is a guess; needs tuning per ELAI session statistics. D-17 instruments the
metric so the band emerges empirically.

Healthy band defaults: 5-50% per real-problem §4.6 (broader than UC-5's
10-30% because empirical-unknown). Out-of-band triggers operator notification.

Partial foundation present and usable:
- D-16 RefusalDecision SSB rows.
- SsbViewer for session-level aggregation.
- Dashboard surface (D-27 will render gauge).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.6 SLA — lines 288-296; UC-5 healthy band guess)
- .elai_cc/crates/orchestrator/src/conformal_calibration.rs (D-16
  RefusalDecision)

## Tasks

1. **RefusalRateMonitor types** in new file
   .elai_cc/crates/orchestrator/src/refusal_sla.rs:
   - `struct RefusalRateMonitor { window_size_minutes: u32, healthy_min:
     f32, healthy_max: f32, ssb_db_path: PathBuf }`
   - `struct RefusalRateSnapshot { session_id: String, window_start:
     DateTime<Utc>, refusal_count: u32, total_claims: u32, rate: f32,
     band: HealthBand }`
   - `enum HealthBand { Below { rate: f32, min: f32 }, Healthy, Above {
     rate: f32, max: f32 } }`

2. **Snapshot computation** in refusal_sla.rs:
   - `fn snapshot(&self, session_id: &str, ssb_viewer: &SsbViewer) ->
     RefusalRateSnapshot`:
     - Query RefusalDecision rows in window.
     - Query MODE_SELECTED rows in window (denominator).
     - Compute rate = refusal_count / total_claims.
     - Determine band; if Below or Above, emit RefusalSlaBreach SSB row.

3. **Alert dispatch** in refusal_sla.rs:
   - `fn alert_on_breach(&self, snapshot: &RefusalRateSnapshot, alert_tx:
     &AlertSender)`: emit Bubble-Up event (existing alert pattern from B-13)
     when band != Healthy; AlertKind::RefusalRateOutOfBand.

4. **Periodic monitoring**:
   - Hook into task_runner.rs (or orchestration loop): every N minutes (60
     default), call snapshot + alert_on_breach.
   - Configurable via elai_cc.toml.

5. **SSB action type** in ssb.rs (SCHEMA_VERSION 62→63):
   - `RefusalSlaBreach { session_id: String, rate: f64, healthy_min: f64,
     healthy_max: f64, band: String, total_claims: u32 }`
   - Round-trip test.

6. **elai_cc.toml [refusal_sla] section**:
   - `[refusal_sla]\nenabled = true\nwindow_size_minutes = 60\n
     healthy_min = 0.05\nhealthy_max = 0.50\ncheck_interval_minutes = 30`
   - L3-LOCKED.

7. **Unit tests** in refusal_sla.rs:
   - snapshot computation correct on synthetic SSB queries.
   - band classification: rate=0.02 → Below; rate=0.30 → Healthy; rate=0.60
     → Above.
   - alert_on_breach emits only on band breach (not Healthy).

8. **Integration test** in
   .elai_cc/crates/orchestrator/tests/refusal_sla_integration.rs:
   - Mock SSB DB with 100 MODE_SELECTED rows + 2 RefusalDecision rows
     (rate=0.02) → snapshot.band = Below → alert dispatched.
   - Same with 60 RefusalDecision rows (rate=0.60) → Above → alert.
   - 20 refusals (rate=0.20) → Healthy → no alert.

9. **Decision doc** workflow/reasoning/D-17-decisions.md (3 decisions):
    - (a) Why 5-50% default band (not 10-30%): broader range pending empirical
      tuning per UC-5; allow Mode E to range during early sessions.
    - (b) Why session-level metric (not global): session contexts vary;
      per-session band tracking surfaces drift; aggregate analytics in D-27
      dashboard.
    - (c) Why Bubble-Up alert (not silent log): operator intervention may be
      needed if checkers misconfigured; SLA breach is action-required.

## WIRING_CONTRACT

produces_symbols:
  - RefusalRateMonitor + RefusalRateSnapshot + HealthBand
  - snapshot + alert_on_breach
  - SsbActionType::RefusalSlaBreach + payload
  - SCHEMA_VERSION bump 62→63
  - [refusal_sla] elai_cc.toml section
consumes_symbols:
  - RefusalDecision (D-16)
  - ModeSelected (D-10)
  - SsbViewer + SsbWriter
  - Bubble-Up alert system (B-13)
reachable_from:
  - task_runner.rs periodic check hook
  - dashboard surface (D-27 renders gauge)
stub_ban:
  - snapshot MUST handle total_claims=0 (band=Healthy by definition, no
    division by zero)
  - alert_on_breach MUST be idempotent on repeated breaches in same window
    (suppress duplicate alerts via window-end key)
  - thresholds MUST load from elai_cc.toml
observable_effect:
  trigger: synthetic session with rate=0.02 (mostly hallucinated claims)
  expect_ssb: 1 RefusalSlaBreach row with band=Below; Bubble-Up alert dispatched
  expect_artifact: dashboard gauge (D-27) shows red on this session

## DETERMINISTIC_CHECK

```bash
rg "SCHEMA_VERSION: i64 = 63" .elai_cc/crates/orchestrator/src/ssb.rs
rg "RefusalSlaBreach" .elai_cc/crates/orchestrator/src/ssb.rs
rg "struct RefusalRateMonitor|fn snapshot" .elai_cc/crates/orchestrator/src/refusal_sla.rs
rg "\[refusal_sla\]" .elai_cc/elai_cc.toml
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- refusal_sla 2>&1 | grep "test result"
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 63
- [ ] RefusalSlaBreach round-trip
- [ ] RefusalRateMonitor + snapshot correct on synthetic SSB
- [ ] Band classification 5/50% default
- [ ] alert_on_breach dispatch via Bubble-Up
- [ ] elai_cc.toml [refusal_sla] loaded
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- refusal_sla: all pass, ≥6 tests
- [ ] Cloud smoke test A: synthetic session 100 claims / 2 refusals
      (rate=2%) → RefusalSlaBreach band=Below; Bubble-Up alert appears in
      dashboard
- [ ] Cloud smoke test B: normal session with rate in [5%, 50%] → no breach;
      no alert
- [ ] Decision doc workflow/reasoning/D-17-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/refusal_sla.rs
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 62→63;
    1 new variant)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (periodic hook)
- MODIFY: .elai_cc/elai_cc.toml ([refusal_sla] section)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod refusal_sla)
- ADD: .elai_cc/crates/orchestrator/tests/refusal_sla_integration.rs
- ADD: workflow/reasoning/D-17-decisions.md

## Out-of-scope

- D-27 dashboard rendering (RefusalRateGauge.svelte)
- Per-task-family band tuning (DEFERRED; global band first)
- Long-term SLA trend report (DEFERRED; D-27 dashboard adequate)

## Next

D-18 (ObservableEffect typed predicate) wraps up Wave 2.
