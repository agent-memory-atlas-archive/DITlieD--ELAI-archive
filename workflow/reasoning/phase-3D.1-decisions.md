Phase 3D.1 decisions

What: BSP channel type definitions — DeepSizeOf, ChannelId, ChannelValue, ChannelState, 4 channel types, ChannelWrite, ChannelSnapshot, ChannelError, ChannelRegistry trait, InMemoryChannelRegistry.

Key decisions:
1. Typed read/write methods per channel kind (read/read_topic/read_ephemeral/read_untracked) instead of single generic. Reason: compile-time type safety, each channel kind has different semantics (seq ordering, append, write_count, raw storage).
2. Read methods return Result<ChannelValue<T>, ChannelError> — deserialization errors propagated, not silently swallowed. Audit BLOCK fix.
3. clear_ephemeral returns Vec<String> (cleared channel names) instead of usize. Reason: caller (superstep loop in 3D.4) needs names to record SSB action_type='checkpoint' subtype='ephemeral_clear' per spec. The registry is a data structure, not the event source.
4. DeepSizeOf uses max(element_deep, capacity * size_of::<T>()) for Vec<T>. Reason: audit BLOCK — Vec::with_capacity(N) allocates N*sizeof(T) even with len=0, must be measured for R19 memory monitoring.
5. TopicEntry.accumulate conflict detection via ChannelError::AccumulateConflict. Reason: audit BLOCK — two agents publishing to same Topic with different accumulate expectations = silent state corruption.
6. UntrackedValue stored as Box<dyn Any + Send + Sync> with size closure. Only type-erased storage in the crate — justified because UntrackedValue is never serialized and force_clear/deep_size operate on erased ChannelId<()>.
7. No new dependencies added. rmp-serde, serde, thiserror all pre-existing in Cargo.toml.

Spec divergences (require Amendment 3 before 3E):
- ChannelRegistry trait has typed read/write methods (not the 6-method interface in phase-3D.txt line 88-95)
- clear_ephemeral returns Vec<String> not usize (phase-3D.txt line 93)

Expected test outcome: 20 unit + 17 contract = 37/37 GREEN, 0 warnings.

Unknowns:
- Ephemeral(None) entries accumulate unboundedly in snapshots — deferred to 3D.4 snapshot optimization
- ChannelWrite<T> defined but unused — needed in 3D.2 for superstep write-batch protocol
