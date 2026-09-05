phase 4K.3-4K.5 RED state log

Tests written to disk:
- .elai_cc/crates/contracts/tests/dashboard_4k3_4k4_4k5_contract.rs (15 tests)
- .elai_cc/crates/dashboard/tests/integration_dashboard_state.rs (5 tests)

Expected RED reasons (compile-time):
- TaskSubmittedPayload not exported from elai_orchestrator (B1 will add)
- StepStatus + SsbActionDetail not exported from elai_orchestrator (B2 will add)
- SsbViewer::step_progress + SsbViewer::get_action_detail not implemented (B2)
- SsbPayload::TaskSubmitted variant missing (B1)
- DashboardState fields writer/projects/current_project not present (B3)
- elai_dashboard_lib::projects module not present (B3)

cargo test -p elai_cc_contracts --test dashboard_4k3_4k4_4k5_contract = expected fail (cannot resolve symbol).
cargo test -p elai-dashboard = expected fail (cannot resolve module/symbol).

Transitioning PLANNING-confirmed contracts written -> EXECUTING.
