Phase 3I.9 MEASURE decisions

What: collect and record 7 DST metrics in metrics/phase_3I.toml
Key decision: ops_per_sec measured via examples/measure_3i.rs (10K apply() calls in closed FSM cycle)
Expected test outcome: 2116/2116 green, 1 ignored, 0 BLOCKs
Unknowns:
  - compile_time_delta_sec baseline is phase_1D single-crate (9.45s), not workspace. No phase_3H baseline exists. Delta (32.81s) conflates P1A-P3I growth.
  - madsim_compile_time_sec (83s) is cargo check, not cargo build. Key name is slightly misleading but documented in TOML comment.
  - ops_per_sec (49M) far exceeds 100K ungrounded target. Measurement is valid: closed-loop event sequence ensures 100% apply() invocation rate.
