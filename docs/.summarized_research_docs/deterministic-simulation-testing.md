Architecting Deterministic Simulation Testing for LLM-Orchestrated Stateful Systems in Rust

Source: Internal research document
Domain: DST, Rust async testing, FSM verification, LLM integration
Timeframe: 2025-2026

---

1. Problem Statement

Traditional testing (unit, integration, E2E) fails against the combinatorial explosion of thread scheduling, network latencies, hardware faults, and non-deterministic dependencies. Deterministic Simulation Testing (DST) replaces OS/hardware interactions with a deterministic runtime: massive swarms of randomized tests where any failure is perfectly reproducible given its initial seed.

Target architecture: Non-deterministic LLM inference wrapped inside a deterministic shell using FSMs, SQLite WAL state, and Bulk Synchronous Parallel (BSP) supersteps.

---

2. State of the Art: Three Foundational Systems

2.1 FoundationDB

- Language: C++ (Flow actor language)
- Engine: Single-threaded discrete-event simulator
- Time: Entirely decoupled from wall clock; logical clock with priority-queue timer
- Randomness: deterministicRandom() PRNG seeded at simulation start
- Network: In-memory queues; Sim2Conn checks g_clogging.disconnected() state and refuses delivery based on deterministic RNG
- Chaos: BUGGIFY macros randomly modify connection timeouts, file I/O block sizes, trigger machine crashes
- Performance: 24 hours simulated uptime in seconds of wall-clock time (fast-forwarding logical clock when idle)
- Notable bug caught: commit_unknown_result — atomic counters incrementing twice due to dropped connections during commit acknowledgment (months to surface in production, immediate in simulation)

2.2 TigerBeetle

- Language: Zig
- Engine: VOPR (Viewstamped Operation Replicator) — entire cluster on single thread, 1000x real speed
- Memory: Strict static allocation (NASA Power of Ten rules); designs away fragmentation and OOM
- Physical determinism: Not only logical state but exact byte layout of LSM trees on disk is identical across replicas given same history
- Storage fault injection: Bit flips, torn writes, simulated disk latency; verifies physical repair via checksums and peer-to-peer sync
- Multiverse exploration: Checkpoint system state, inject fault, run forward, backtrack to explore alternative timeline — systematically uncovers bugs in overlapping failure modes
- Validation: Passed extensive Jepsen testing

2.3 RisingWave (madsim)

- Language: Rust
- Engine: madsim — drop-in replacement for tokio async runtime
- Integration: Cargo [patch] feature replaces tokio with single-threaded runtime governed by global PRNG + priority-queue timer
- Task scheduling: First-In-Random-Out (FIRO) queue — aggressively exposes hidden concurrency bugs by shuffling execution order deterministically
- Usage: Unit, E2E, recovery, and scaling tests with perfect trace correlation via tracing spans

| System | Language | Engine | Key Strategy | Fault Targets |
|---|---|---|---|---|
| FoundationDB | C++ (Flow) | Flow Simulator | Single-threaded event loop, mocked logical clock | Machine crashes, network degradation, BUGGIFY macros |
| TigerBeetle | Zig | VOPR | Static memory, physical LSM layout determinism | Storage bit flips, deterministic LSM compaction, alternate timelines |
| RisingWave | Rust | Madsim | Cargo [patch] for tokio, FIRO queueing | OS/Network layer, Etcd/S3 API mocks, RPC tracing |

---

3. Madsim Architecture

3.1 Cargo Patching and Libc Hooking

Determinism via compile-time dependency hijacking:
- [dependencies]: madsim-tokio
- [patch.crates-io]: Swaps tokio, tonic, rdkafka, reqwest with deterministic equivalents
- Activation: --cfg madsim compiler flag
- Deep hooks: libc symbol overrides for gettimeofday, clock_gettime, sysconf, getrandom
- Time behavior: SystemTime::now() halts calling task, registers wake-up in deterministic priority queue, yields thread. Clock advances only when all ready tasks yield, jumping to next scheduled event.
- Entropy: Patches quanta and getrandom crates so third-party libraries reading hardware entropy receive deterministic bytes from simulator's global PRNG

3.2 Network and Fault Injection

- In-memory network topology driven by IP addresses
- TCP/UDP streams replaced with in-memory message channels
- Programmable chaos via control APIs:
  - Packet loss: Link properties modified to drop with probability from global PRNG
  - Network partitions: Sever simulated link at exact simulated timestamps
  - Delays/jitter: Latency adhering to deterministic distributions
- Tracing: Context information injected into logs correlating perfectly with distributed traces (identical sequence of events for given seed)

3.3 Madsim vs Turmoil

Turmoil: Network simulation framework for tokio. Excels at pure network sim but historically lacks deep libc symbol overriding for time/entropy.

mad-turmoil (by S2): Hybrid crate blending Turmoil's ergonomic network simulation with madsim's rigorous libc overrides. For LLM-orchestrated systems with SQLite + BSP, full entropy control is non-negotiable — madsim or mad-turmoil required.

---

4. Stateful Property-Based Testing

4.1 proptest-state-machine Crate

Two traits to implement:

ReferenceStateMachine: Defines abstract state + enumeration of all possible transitions. Pure functions via apply method. Provides init_state strategy and transitions strategy.

StateMachineTest: Wraps actual async SUT. Maps abstract transitions to real async calls driving FSM through BSP supersteps.

4.2 Invariant Expression

check_invariants executes after every single transition:

SSB monotonicity: Track last known sequence number in reference state. After transition, assert SUT's current sequence number > previously recorded reference.

FSM determinism: Using TestRunner::deterministic() or hardcoded TestRng fixes operation sequence. If SUT branches non-deterministically (different state than reference under identical conditions), apply/check_invariants assertion fails — instantly flags entropy leak.

---

5. The 10,000-Step Shrinking Problem

5.1 Stateful Shrinking Mechanics

On failure deep in sequence, proptest attempts binary-search-like minimization via ValueTree simplify()/complicate(). For sequential stateful tests: primarily removes operations from sequence (not mutating individual operations, which tends to break preconditions of subsequent operations).

Example: Insert(A) -> Update(A) -> Delete(A). Mutating Insert to Insert(B) invalidates Update(A) for reasons unrelated to original bug.

5.2 Performance Workarounds

Failure at step 9,500 = shrinker must re-execute from scratch thousands of times.

Mitigations:
- Timeouts: PROPTEST_MAX_SHRINK_TIME=60s prevents CI hanging
- Iteration limits: PROPTEST_MAX_SHRINK_ITERS (may yield sub-optimal larger failing sequence)
- Alphabet reduction: Aggressively scope transition data payloads during initial fuzzing
- Manual bisection: Replicate locally using printed seed, inject tracing spans, leverage absolute determinism to isolate root cause (often faster than 2-hour automated shrink)

---

6. SQLite WAL in Deterministic Simulation

6.1 Virtual File System (VFS)

Custom SQLite VFS in Rust: Trap write()/fsync() syscalls, redirect to in-memory byte buffer managed by simulation. Simulator's deterministic PRNG dictates exact latency of read/write operations. When SQLite requests flush, VFS yields to simulation's timer queue — database waits for precise simulated duration before acknowledgment.

6.2 Deterministic WAL Recovery and Torn Writes

Simulator can intentionally truncate WAL at arbitrary byte boundary before simulated fsync completes during BSP superstep. On simulated "reboot," SQLite performs WAL recovery. Because bytes written to in-memory VFS are strictly deterministic, developers verify recovery to exact pre-crash state without data loss.

Limbo project: Complete async rewrite of SQLite in Rust. Designs async I/O around DST principles — memory management and disk interactions completely observable/controllable by deterministic simulation harness.

---

7. Taming Non-Deterministic LLM Inference

7.1 The temperature=0 and Seed Illusion

Setting temperature=0 + fixed seed does NOT guarantee absolute determinism:

- Floating-point/hardware divergence: GPUs/CPUs execute matrix multiplications differently; micro-variations alter tie-breaking when tokens have near-identical probabilities
- "First run artifact": Ollama caching anomalies produce different output on first vs subsequent runs with identical params
- MoE/batching non-determinism: Concurrent batch scheduling alters token routing; disabling multiprocessing (VLLM_ENABLE_V1_MULTIPROCESSING="0") cripples performance
- Context window truncation: num_ctx must be explicitly hardcoded alongside seed in Ollama; dynamic sizing introduces variability

Conclusion: Live LLM inference in a 10,000-step DST sequence will inevitably cause state divergence.

7.2 VCR (Video Cassette Recorder) Pattern

During initial non-deterministic run against live LLM:
1. Capture exact HTTP requests (prompt, context arrays, hyperparameters) and responses
2. Serialize as "cassette" files (JSON/YAML)

During DST runs:
1. Network layer intercepts outgoing prompt
2. Compute cryptographic hash of prompt payload
3. Look up in cassette file
4. Return recorded response with deterministic time delay (simulated latency from PRNG)

Tools: llmock, custom Rust interceptors via reqwest middleware.

If code change alters prompt structure -> test fails -> alerts to prompt drift -> requires cassette regeneration.

7.3 TraceOps (Behavioral Layer)

For complex agentic FSMs with RAG and tool-calling, HTTP-level caching is too brittle (minor prompt variations break it).

TraceOps intercepts SDK functions orchestrating the LLM. Records entire execution trajectory:
- Sequence of tool invocations
- Intermediate reasoning steps
- Branching decisions
- Token expenditures

During DST, FSM fed exact deterministic trajectories. Isolates FSM transition logic — can test orchestrator's reaction to hallucination pathways, malformed JSON, complex LLM trajectories without firing single live inference.

---

8. Quarantining Entropy in Rust

8.1 HashMap Ordering

std::collections::HashMap/HashSet are fundamentally non-deterministic (SipHash 1-3 with random OS seed). Iteration order changes every execution.

Replacements:
- BTreeMap: Natural key ordering (when iteration order matters)
- FnvHashMap / FxHashMap: Deterministic hasher (when O(1) lookup needed)
- indexmap: Maintains insertion order regardless of hash seed
- deterministic_rand: Wrapper forcing sort on key iteration when determinism feature flag enabled

8.2 SystemTime and Thread Scheduling

SystemTime::now() / Instant::now(): Route through madsim executor (intercepts libc time syscalls) or mocked Clock trait returning logical timestamps from simulation loop.

tokio multi-threaded runtime: OS thread scheduling + work-stealing = severe non-determinism. Must use tokio::runtime::Builder::new_current_thread() — all futures cooperatively scheduled on single thread, execution order purely from deterministic timer queue.

8.3 Automated Auditing

| Approach | Target | Mechanism |
|---|---|---|
| Clippy Restrictions | HashMap::new(), SystemTime::now(), rand::thread_rng() | #![deny(clippy::disallowed_methods)] in clippy.toml forbids non-deterministic stdlib calls |
| Syscall Interception | getrandom, OS time calls | Run under strace / seccomp filters; panic on entropy syscalls from third-party deps |
| Cargo Scan | Transitive dependencies | Side-effect analysis across crate boundaries for filesystem/network access |

---

9. Unified Architectural Pipeline

Layer-by-layer isolation:

1. Substrate: Single-threaded tokio runtime via madsim. Cargo [patch] replaces networking/timing dependencies. Foundational deterministic event loop.

2. State Layer: SQLite in custom in-memory VFS. Exposed to simulator for deterministic I/O delays and torn write simulation during WAL commits.

3. Orchestration Layer: FSMs driven by BSP supersteps. Barrier synchronization governed by simulator's logical clock. Distributed actors process messages in strictly deterministic sequence.

4. Testing Driver: proptest-state-machine generates massive FSM interaction sequences. Validates SQLite state and FSM sequence numbers after every tick for invariant preservation.

5. LLM Layer: Entirely severed from live network during testing. VCR cassettes or TraceOps behavioral graphs transform probabilistic AI responses into pure deterministic data payloads mapped to prompt hashes.

6. Audit: Strict clippy configurations + seccomp filters enforce entropy quarantine. SystemTime, HashMap, rand never leak into execution path.

---

10. Key Takeaways

1. DST replaces combinatorial failure exploration with seed-reproducible deterministic testing — any failure is perfectly replayable
2. madsim is the most mature Rust pathway to DST; full entropy control (libc hooks for time, randomness, network) is non-negotiable
3. LLM inference MUST be cordoned off via VCR/TraceOps patterns — temperature=0 + seed does not guarantee determinism
4. SQLite integration requires custom VFS redirecting all I/O to in-memory buffers under simulator control
5. proptest-state-machine enables massive stateful testing but 10,000-step shrinking requires explicit timeouts and alphabet reduction
6. HashMap, SystemTime, thread scheduling are the three primary entropy sources in Rust that must be quarantined
7. The TigerBeetle multiverse exploration pattern (checkpoint -> inject fault -> run forward -> backtrack) is the gold standard for overlapping failure mode discovery
