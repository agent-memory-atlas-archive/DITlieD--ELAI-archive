I-26 — dynamic thinking budgets: key decisions

what: depth-scaled node compute budget controller + cloud effort-parameter mapping
      (Anthropic effort="auto"|"expert" for Opus 4.7+ cloud reasoning roles)

key decisions

26a (effort field plumbing) — EXTEND approach, not a new action_type

decision: extend MessageRequest + CompletionConfig with Option<String> effort field;
wire build_request in anthropic_api.rs to forward config.effort to the wire struct.
this is the fully buildable leg — it adds the delivery mechanism without requiring
I-4 SearchLadder to exist. any caller that sets config.effort gets the benefit
immediately (existing cloud paths default to None, no behavior change).

rejected alternative: new SSB action_type for "effort_selected" event.
reason: I-0 CANONICAL says SCHEMA_VERSION = DDL_EPOCH + SSB_ACTION_TYPES.len().
a new action_type was explicitly banned by the controller (SCHEMA_VERSION+TOTAL_VARIANTS
must NOT change). the effort field is a request-time parameter, not a post-hoc audit
event; it belongs in the request struct, not the event log.

26b (BudgetController) — wire-dark via pre-approved stub, not forced wiring

decision: ship BudgetController at crates/orchestrator/src/routing/budget_controller.rs,
mark cloud_effort_for_node + search-node methods WIRE-DARK[I-26] with backing
wire-dark-allowlist.toml row (ELAI_DYNAMIC_BUDGET_ENABLE, experiment).
the production host (I-4 SearchLadder, search/ladder.rs::SearchLadder::run_best_of_n)
does not exist at HEAD (Wave-4 KEYSTONE dependency).

rejected alternative: force-wire BudgetController into task_runner as a fake
"production call". reason: would violate R3 (scope contract — forcing a fake
entrypoint is not the real consumer) and produce dead-end telemetry. the pre-approved
WIRE-DARK escape (fork resolution A181) is the correct path.

ReasoningBudgetScaledPayload extension — Option<> fields, serde(default) + skip_serializing_if

decision: add depth_level: Option<u32>, budget_pct: Option<f32>, plateau_fired: Option<bool>
to the EXISTING H-42 struct (serde(default) + skip_serializing_if = "Option::is_none").
this preserves HD-2 byte-stability: existing rows decode with None on new fields.
I-26 search-node rows populate all three. no new action_type variant needed.

E-39 ceiling representation — LOCKED_BUDGET_CEILING=3 mirror

decision: MAX_DEPTH_MULT=3 in budget_controller.rs mirrors reasoning_budget.rs::LOCKED_BUDGET_CEILING.
no importable E-39 USD cap symbol exists at HEAD (E-39 is wire-dark with a different allowlist row).
the concrete ceiling at HEAD is the realized value 3 in reasoning_budget.rs.

ID-5 compliance — starter constants, not paper-derived

decision: DEPTH_FACTOR_NUMERATOR, DEPTH_FACTOR_DENOMINATOR, PLATEAU_DELTA_MIN are
calibration starters tagged DEPTH_FACTOR_CALIBRATION_PENDING=true. they are NOT
drawn from any paper. I-27 convergence curve is the grounding source. no paper tuned
these to ELAI's workload; pretending otherwise would violate ID-5.

expected test outcomes

cargo check --workspace: EXIT 0 (no unwrap/expect in lib code, all struct literals updated)
action_type_lock: PASS, count UNCHANGED (no new action_type variants)
e45_cross_language_drift_contract: PASS (no TS side affected)
h42_serial_depth_budget_contract: PASS (new fields = None, backward-compatible)
i26_dynamic_thinking_budgets_contract: PASS (all 7 test groups)

unknowns

SearchLadder (I-4, Wave-4 KEYSTONE) — required to make BudgetController production-live.
promotion_plan_id I-26-promotion will author the wiring after SearchLadder ships.
calibration constants — ID-5 starters; I-27 measured curves will ground them.
