Phase 3D.4: Memory Monitoring Integration

What: Added memory pressure monitoring to BSP superstep loop and crash recovery interface.

Key decisions:
- iter_untracked_sizes() returns Vec<(String, usize)> (collected, not borrowing iterator) to avoid borrow conflicts across async SSB write await points.
- MemoryMonitoringPayload field mapping: metric_name=channel_name, value=size as f64, threshold=limit as f64, action_taken="measure"|"force_clear". No new struct — reuses existing SSB payload.
- recover_from_checkpoint() uses &dyn SsbReader (trait object) instead of &Ssb (concrete) for testability. channels param is used (reset_ephemeral_write_counts) to establish interface contract; full restoration is future-phase.
- Force-clear threshold: UNTRACKED_DEFAULT_LIMIT_BYTES * UNTRACKED_FORCE_CLEAR_MULTIPLIER (10MB * 2 = 20MB). Named constants only, no magic numbers.

Expected test outcome: 16 engine unit tests + 20 channel unit tests + 4 contract tests = 40/40 green.

Unknowns: recover_from_checkpoint() does not yet replay writes from SSB — that requires deserializing channel-specific payloads which are not yet defined in the SSB schema.
