Phase 3D.3 decisions: BSP deferred node execution

What: single-pass deferred execution block in BspEngine::run, replacing the 3D.3 hook comment.
After the main barrier (execute_tasks_with_timeout + mark completed), iterate deferred+Pending nodes
whose depends_on are all Complete. Batch spawn via execute_tasks_with_timeout. Mark Complete or
TimedOut with death events. Runs once per superstep, no re-entry.

Key decision: depends_on as dep-proxy (not new input_channels field).
Rationale: prepare_next_tasks already uses depends_on for non-deferred scheduling. Adding input_channels
to NodeSpec would require changes across all NodeSpec construction sites. Using depends_on is consistent
and minimal blast radius.

Key decision: batch deferred into single execute_tasks_with_timeout call.
Rationale: reuses existing timeout/death-event infrastructure. Identical error handling path as main
barrier execution. No special-case code for deferred nodes.

Key decision: cascading prevention is implicit.
Rationale: deferred nodes are Pending at deferred-phase entry. A deferred node depending on another
deferred node finds it Pending (not Complete), so it never fires. No explicit guard variable needed.
This is guaranteed by the single-pass design.

Expected test outcome: 12 unit + 3 contract = 15 green, 0 warnings.
Actual: 12 unit + 3 contract = 15/15 green, 0 warnings.

Unknowns: deferred nodes with Failed dependencies silently strand (remain Pending). Acceptable for
3D.3 scope — future phases may add diagnostic logging or error propagation for stranded nodes.
