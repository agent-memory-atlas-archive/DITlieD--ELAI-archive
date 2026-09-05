Phase 3I.5 decisions

What: TDD red-phase test file bsp_determinism.rs with 5 integration tests for BSP superstep determinism.

Key decisions:
- Construct BspCheckpoint manually (no BspEngine::run) since Ssb not test-constructible.
- Use ChannelSnapshot::iter() to convert to BTreeMap for checkpoint construction in test 1.
- Fixed u64 literal for created_at_secs (no SeededClock needed — testing serialization determinism, not clock behavior).
- REPETITIONS=100 named constant per codeRules rule 17.
- All .expect("reason") instead of .unwrap() per project rules.
- Structural exhaustive match on ChannelState (3 variants) to prove UntrackedValue excluded at compile time.

Expected test outcomes:
- cargo check --tests: exit 0 (confirmed).
- cargo test: TDD red — runtime failures expected for topic_ordering if Topic entries stored in insertion order (not seq-sorted). Other tests should pass at runtime.

Unknowns:
- Whether Topic entries are stored sorted by ssb_seq or in insertion order — test 4 will reveal this.
