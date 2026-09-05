Phase 3C.6: AntiSpiralDetector module

What: ring-buffer spiral detector for GRC agents. Monitors (step_type, topic, content_hash) tuples per agent. Escalation after SPIRAL_THRESHOLD (3) consecutive repeats, HardStop if agent ignores escalation.

Key decisions:
- blake3 over sha2 for content hashing: 3x faster on x86 (SIMD), same 32-byte output, 1.8.4 stable. sha2 kept for checkpoint.rs compatibility.
- PhantomData<C: ClockProvider> instead of storing clock instance: detector does not use timestamps (spiral detection is purely count-based). Type parameter kept for consistency with other orchestrator modules (SupervisionManager, CheckpointManager).
- Sync API (no async): record_step returns SpiralResult immediately. SSB writes happen at caller level, not inside detector. Avoids async infection in hot path.
- VecDeque ring buffer over fixed array: VecDeque::with_capacity(4) avoids unsafe, provides pop_front/push_back. Ring size is small (4), allocation cost negligible.
- recovery_target = spiral_start - 1: recovery must resume BEFORE the spiral, not at the latest checkpoint (which is inside the spiral loop).

Expected test outcome: 13 unit + 3 contract = 16 tests, 0 failures.
Unknowns: wiring into SupervisionManager.handle_spiral() deferred to 3C.7+. SpiralDetected SSB payload is structurally ready but not exercised by live detector-to-SSB pipeline yet.
