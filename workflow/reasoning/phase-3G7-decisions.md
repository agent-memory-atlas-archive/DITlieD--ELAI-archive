Phase 3G.7 decision doc

What: Integration test for plan_skeleton lifecycle (5-phase topology) + 100-run determinism proof.
Key decision: Inline in plan_skeleton.rs mod tests (not separate file) — matches existing pattern, avoids pub visibility issues with crate-internal types.
Expected test outcome: 2 new tests GREEN (lifecycle_full_5phase_integration, lifecycle_determinism_100_runs), all 87 existing tests GREEN, 0 warnings.
Unknowns: Whether coherence_check import works from plan_skeleton mod tests (it should — pub mod in lib.rs). Whether load_phase_context with no detail file returns Ok with phase_detail=None (need to verify).
