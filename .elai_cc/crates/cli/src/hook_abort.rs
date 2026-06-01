#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, UNIX_EPOCH};

use elai_core::ClockProvider;

pub(crate) struct HookAbortMonitor {
    pub(crate) stop_tx: Option<Sender<()>>,
    pub(crate) join_handle: Option<JoinHandle<()>>,
}

impl HookAbortMonitor {
    /// Production spawn — Ctrl+C handler decides via `StopHookGate`:
    ///   * FSM = Idle    -> abort immediately on first press.
    ///   * FSM != Idle   -> first press warns + records SSB intervention_request.
    ///                       Second press within 3s aborts. Outside the window:
    ///                       a new warn cycle starts.
    /// See `StopHookGate::observe_signal` for the canonical decision policy.
    pub(crate) fn spawn(
        abort_signal: elai_orchestrator::HookAbortSignal,
        fsm: Arc<elai_orchestrator::Fsm>,
        ssb: Option<Arc<elai_orchestrator::ssb::Ssb>>,
        clock: Arc<dyn ClockProvider>,
    ) -> Self {
        let gate = elai_orchestrator::StopHookGate::new();
        Self::spawn_with_waiter(abort_signal, move |stop_rx, abort_signal| {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };

            runtime.block_on(async move {
                let wait_for_stop = tokio::task::spawn_blocking(move || {
                    let _ = stop_rx.recv();
                });

                // Loop: every Ctrl+C runs the gate. If the gate decides Abort,
                // we set the abort signal and exit. WarnAndRecord goes back to
                // listening for the next Ctrl+C (within or outside the window).
                tokio::pin!(wait_for_stop);
                loop {
                    tokio::select! {
                        result = tokio::signal::ctrl_c() => {
                            if result.is_err() {
                                // Signal subsystem died — fall back to legacy behaviour.
                                abort_signal.abort();
                                break;
                            }
                            let now_ms = clock_to_epoch_ms(&clock);
                            // double-Ctrl+C / second Ctrl+C / 3s window logic.
                            let fsm_state = fsm.get_state().await;
                            let decision = gate.observe_signal(fsm_state, now_ms);
                            match decision {
                                elai_orchestrator::StopHookDecision::Abort => {
                                    tracing::info!("hook_abort_monitor: Ctrl+C confirmed — aborting");
                                    abort_signal.abort();
                                    break;
                                }
                                elai_orchestrator::StopHookDecision::WarnAndRecord => {
                                    tracing::warn!(
                                        fsm_state = %fsm_state,
                                        "Task running. Press Ctrl+C again within 3s to abort, or wait for current step to complete."
                                    );
                                    if let Some(ssb_ref) = ssb.as_ref() {
                                        let payload = elai_orchestrator::ssb::SsbPayload::InterventionRequest(
                                            elai_orchestrator::ssb::InterventionRequestPayload {
                                                step_id: format!("ctrl_c_{now_ms}"),
                                                intervention_type: elai_orchestrator::ssb::InterventionType::ApproveChange,
                                                description: format!(
                                                    "Ctrl+C received while FSM={fsm_state}. Press Ctrl+C again within 3s to abort."
                                                ),
                                                diff_summary: None,
                                                options: vec![
                                                    elai_orchestrator::ssb::InterventionOption::Keyword(
                                                        elai_orchestrator::ssb::InterventionKeyword::Accept,
                                                    ),
                                                    elai_orchestrator::ssb::InterventionOption::Keyword(
                                                        elai_orchestrator::ssb::InterventionKeyword::Decline,
                                                    ),
                                                ],
                                            },
                                        );
                                        if let Err(e) = ssb_ref
                                            .write_payload("hook_abort_monitor", 0, payload, Vec::new())
                                            .await
                                        {
                                            tracing::warn!(error = %e, "hook_abort_monitor: failed to emit intervention_request row");
                                        }
                                    }
                                }
                            }
                        }
                        _ = &mut wait_for_stop => break,
                    }
                }
            });
        })
    }

    pub(crate) fn spawn_with_waiter<F>(abort_signal: elai_orchestrator::HookAbortSignal, wait_for_interrupt: F) -> Self
    where
        F: FnOnce(Receiver<()>, elai_orchestrator::HookAbortSignal) + Send + 'static,
    {
        let (stop_tx, stop_rx) = mpsc::channel();
        let join_handle = thread::spawn(move || wait_for_interrupt(stop_rx, abort_signal));

        Self {
            stop_tx: Some(stop_tx),
            join_handle: Some(join_handle),
        }
    }

    pub(crate) fn stop(mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

/// Helper: convert a `ClockProvider`'s system_now() to epoch milliseconds,
/// saturating on the unlikely case of pre-1970 system clocks. Used by
/// HookAbortMonitor to feed StopHookGate without invoking std::time::* directly.
pub(crate) fn clock_to_epoch_ms(clock: &Arc<dyn ClockProvider>) -> i64 {
    let dur = clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    i64::try_from(dur.as_millis()).unwrap_or(i64::MAX)
}
