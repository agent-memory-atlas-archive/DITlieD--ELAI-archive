E-49 coding_agent_metrics — closure decisions
session: s-25120 completion-drive cluster, session 2
shipped: 2026-05-25
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-49_coding_agent_metrics.md
audit: workflow/sessions/s-25120/completion-drive/audit/E-49-audit.md (PASS, 0 BLOCK, 1 WARN, 6 INFO)

What
14-file manifest (8 NEW + 6 MODIFIED) closing coding_agent_metrics surface as a 6-file orchestrator module (mod + types + collector + swebench_pro + refinebench + aggregator) under orchestrator/src/coding_metrics/ + metrics_cmd CLI as WIRE-DARK defender + e49_coding_agent_metrics_contract test over substrate at SCHEMA 137. SCHEMA_VERSION bumped 137 to 138 atomic + 5 SsbActionType variants 9-tuple cascade x5 + TOTAL_VARIANTS 394 to 399. CodingAgentMetricsCaptured + 4 sibling payloads added for SWE-Bench Pro + RefineBench run lifecycle (started / completed / failed / aggregated).

Key decisions
1. LEAN_SCOPE per E13_LEAN_SCOPE_DISCIPLINE — original plan surfaced 12 tasks across collector + executor + ingestion + reporting + dashboard + CI yaml. Folded to 6-file module: types (DTOs) + collector (metric capture API) + swebench_pro (benchmark adapter shell) + refinebench (benchmark adapter shell) + aggregator (cross-run rollup) + mod (re-exports). CLI subverb (metrics_cmd) anchors all 5 surfaces as production callers per E40_WIRE_DARK_AVOIDANCE_VIA_CLI 5th application.
2. RUNPOD-PENDING boundary — run_swebench_pro + run_refinebench return Err(RunpodPending) instead of stubbing fake results or panic. SWE-Bench Pro + RefineBench live runs require GPU pod with the actual harness; deferred to RUNPOD-PENDING list (entries 18 + 19). Pattern: graceful Err return at the WIRE-DARK boundary preserves contract test coverage while blocking ship of unverified numbers.
3. 5-variant SSB cascade — CodingAgentMetricsCaptured + SwebenchProRunStarted + SwebenchProRunCompleted + RefinebenchRunStarted + RefinebenchRunCompleted at 9-tuple x5 single-commit; sibling E-37 x9 + E-41 x5 + E-44 x5 precedent extended. 10-site lockstep INCLUDED at action_type_lock.rs TOTAL_VARIANTS prospectively per E44_TENTH_LOCKSTEP_SITE lesson.
4. CliAction::Metrics 16th application + `elai metrics` namespace distinct from sibling E-37/E-18/E-40/E-41/E-44/E-45 namespaces.

Expected test outcome
cargo check clean workspace-wide (VERIFIED: 4/4 deterministic checks pass per audit). Contract test compiles clean at cargo check tier; runtime invariants per feedback_no_cargo_test_unless_permitted policy NOT exercised this audit (cargo test substituted with cargo check --tests).

Unknowns / KNOWN-RESIDUAL
1. Real-pod cost + throughput numbers for SWE-Bench Pro + RefineBench (RUNPOD-PENDING entries 18 + 19).
2. SWE-Bench Pro authoritative result schema — adapter returns DTOs matching public docs at time of plan but may drift; aggregator tolerates extra fields via serde flatten.
3. RefineBench scoring rubric exact wording — adapter scores at file-equality tier; semantic-equality tier deferred until benchmark publishes canonical rubric.
4. 1 WARN cosmetic + 6 INFO per audit; none blocking ship.

Session 2 cumulative: 25/28 E-plans shipped (89.3% mark); 3 plans remaining (E-52 LSP integration + E-53 formal-verification codegen + E-15 sandboxed exec + E-35 structured output 2026).
