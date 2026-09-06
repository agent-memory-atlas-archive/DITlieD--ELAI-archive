# Long-running agent task durability (May 2026)

## TL;DR
- ELAI's SessionDag is solid (CMV-derived, blake3, branch-bounded). Three concrete gaps vs 2025-2026 SOTA: (a) no idempotency-key layer between AgentCheckpoint and tool calls — required to defeat semantic rollback attacks (ACRFence, arXiv:2603.20625); (b) event-history bloat ceiling not enforced — Temporal's "Continue-As-New" pattern needs an ELAI analog (SessionDag rollover at N events); (c) no DST harness — madsim/Antithesis-style deterministic replay tests for the AgentPool supervisor would catch race-condition crash bugs before they reach a 4-hour run.
- Add a hierarchical checkpoint policy: task / phase / step / GRC-step. Right granularity per Temporal+LangGraph is "after each meaningful step" (activity boundary), not after every token — ELAI's 5-msgpack-per-agent is correct activity-level; supplement with a per-step WAL append so a crash mid-step replays from the last activity, not last phase.
- Promote SSB's atomic durability marker into a proper SAGA log: every irreversible tool call gets a compensating-action entry written BEFORE the call, so crash-after-effect can still unwind. Inngest/Temporal/DBOS all converged on this in 2025.
- Adopt postcard over msgpack for AgentCheckpoints if Rust-only — 30% smaller, no_std, deterministic byte order (better for blake3 hashing). Keep msgpack only at the IPC boundary where the dashboard reads it.
- Schema migration: today ELAI bumps SCHEMA_VERSION (B-40=41, B-42=44, B-41=45) but has no documented lazy migrator. LangGraph hit this in 2025 (langgraphjs#536) — answer is forward-compatible reader + lazy state transformer at load time, not big-bang migration.

## Key sources
| URL | what | takeaway |
|---|---|---|
| https://reference.langchain.com/python/langgraph/checkpoints | LangGraph checkpointer API (Dec 2025, langgraph-checkpoint-postgres 3.0.2) | Per-node checkpoint, thread_id is stable key, MemorySaver banned in prod, async checkpointing required to avoid throughput collapse. |
| https://temporal.io/blog/durable-execution-meets-ai-why-temporal-is-the-perfect-foundation-for-ai | Temporal durable execution for AI agents | Replay event history reconstructs in-memory state; workflows must be deterministic; activities wrap all non-determinism. OpenAI Agents SDK x Temporal launched 2025. |
| https://www.inngest.com/blog/durable-execution-key-to-harnessing-ai-agents | Inngest durable execution | step.run = retryable+cached unit, agents retry 15-30% of tool calls, no framework enforces exactly-once at tool boundary. |
| https://www.dbos.dev/blog/durable-execution-crashproof-ai-agents | DBOS (MIT-Stanford) | Postgres-backed checkpoint per step+workflow with auto-generated idempotency keys; recovers from crash to last completed step. SE Radio 681 (Aug 2025). |
| https://arxiv.org/abs/2603.20625 | ACRFence (Mar 2026, UCSC+UConn) | Semantic rollback attacks: retried tool calls produce subtly different requests (even at temp=0, GPU floating-point drift); servers treat as new. LangGraph, Cursor, Claude Code all vulnerable. Mitigation: record irreversible effects + replay-or-fork on restore. |
| https://arxiv.org/abs/2604.28138 | Crab: semantics-aware C/R runtime for agent sandboxes | Process-level checkpoint with semantic awareness — extends OS-level CRIU with agent-state hooks. |
| https://arxiv.org/abs/2602.21806 | Empirical study of 998 LangChain/CrewAI bugs (2026) | 15 root causes / 7 symptoms across 5 lifecycle stages. Top root causes: API misuse, API incompatibility, Documentation Desync. Top symptoms: Functional Error, Crash, Build Failure. Concentrated in "Self-Action" stage. |
| https://arxiv.org/abs/2508.19504 | Aegis (Aug 2025) — agent-environment failures | 6 failure modes; mitigations: env observability, computation offloading, speculative actions; 6.7-12.5% success-rate gain on 142 traces / 3656 turns. |
| https://arxiv.org/pdf/2511.07424 | Production LLM inference incidents (Nov 2025) | 60% of failures are inference-engine failures, ~40% of those are timeouts. OOM kills, driver crashes, model corruption, queue saturation. |
| https://arxiv.org/pdf/2511.03094 | ALAS: Transactional multi-agent LLM planning (Nov 2025) | Transactions over agent plans; ACID-style commit/rollback at plan boundaries. |
| https://arxiv.org/pdf/2602.14849 | Atomix: Timely, transactional tool use | Idempotency + transactional commit at tool boundary. |
| https://nightlies.apache.org/flink/flink-docs-master/docs/concepts/stateful-stream-processing/ | Flink checkpoint barriers | Stream barriers injected at sources flow with data; operator emits barrier downstream after seeing all upstream barriers — distributed snapshot algorithm (Chandy-Lamport variant). Maps to BSP supersteps for agents. |
| https://docs.confluent.io/cloud/current/ai/streaming-agents/agent-runtime-guide.html | Confluent Cloud streaming agents on Flink | Checkpoint periodically persists app state; jobs resume from last successful checkpoint. |
| https://sqlite.org/wal.html | SQLite WAL (current 2025 docs) | 1000-page default checkpoint threshold; readers/writers concurrent in WAL mode; `.db-wal` + `.db-shm` sidecar files. |
| https://github.com/madsim-rs/madsim | madsim deterministic simulator | Rust async runtime (tokio-shaped) with deterministic scheduling; used by RisingWave. Replay any crash exactly. |
| https://www.newswire.ca/news-releases/jane-street-leads-antithesis-s-105m-series-a-... | Antithesis $105M Series A (Dec 2025) | DST goes mainstream; targets agentic systems explicitly. |
| https://blog.lambdaclass.com/introducing-spawned-erlang-style-actors-for-rust/ | Spawned (Rust) | Erlang-style actors + supervision trees in Rust; OneForOne/OneForAll/RestForOne ported. |
| https://docs.rs/supertrees | supertrees (Rust) | Supervision trees for async Rust, OTP-inspired (experimental). |
| https://docs.cursor.com/en/background-agent | Cursor Background Agents | Isolated remote env per agent, branch-isolated (`agent/` prefix), pre-edit zip checkpoint before every code modification; explicit support for "multi-hour refactors." |
| https://stevekinney.com/courses/ai-development/cursor-checkpoints | Cursor checkpoint mechanics | Every Agent edit creates a pre-change zip — fine-grained undo, not crash recovery. |
| https://www.morphllm.com/comparisons/devin-vs-cursor | Devin/Replit Agent overnight pattern | Live on own VM, hours-long runs, open PR while user sleeps; Devin 3.0 adds Interactive Planning checkpoint. |
| https://david.kolo.ski/blog/rkyv-is-faster-than/ | Rust serialization benchmarks | rkyv > bincode, capnp, cbor, flatbuffers, postcard, prost, serde_json on speed; postcard ~70% the size of bincode, ~1.5x slower. |
| https://github.com/jamesmunns/postcard | postcard | no_std, serde-compatible, embedded-first, deterministic encoding. |
| https://github.com/langchain-ai/langgraphjs/issues/536 | LangGraph state schema versioning (open issue) | No built-in detection of schema-incompatible state; lazy transform at load time recommended. |
| https://redis.io/blog/langgraph-redis-checkpoint-010/ | LangGraph Redis checkpoint 0.1.0 (2025) | In-memory checkpoint redesign for high throughput. |
| https://arxiv.org/pdf/2603.29231 | Reliability Decay Curve / Variance Amplification (long-horizon agents) | 4 metrics: RDC, VAF, Graceful Degradation Score, Meltdown Onset Point — quantify long-run failure. |
| https://arxiv.org/pdf/2603.04257 | Memex(RL) | External-DB-backed full-fidelity history, compact working context via index dereference. |
| https://medium.com/@build_break_learn/let-it-crash-7-erlang-lessons-every-java-go-and-rust-engineer-should-steal-in-2025-ac4d65017a10 | OTP lessons for Rust 2025 | Separate fault-handling from business logic; restart intensity limits prevent infinite crash loops; fast restart > slow spiral. |
| https://learn.microsoft.com/en-us/azure/architecture/patterns/compensating-transaction | SAGA / compensating tx (Azure) | Record reverse for every step succeeded; replay reversals in reverse order on failure. |
| https://temporal.io/blog/compensating-actions-part-of-a-complete-breakfast-with-sagas | Temporal SAGA | "AI agent workflows are sagas in disguise" — compensable/retriable/observable primitives. |

## Durability pattern landscape

| Pattern | System | Granularity | Replay support | Scale | Citation |
|---|---|---|---|---|---|
| Per-node DAG checkpoint | LangGraph + SqliteSaver/PostgresSaver | Every graph node | Yes, lazy load by thread_id | Single tenant -> multi-tenant Postgres | reference.langchain.com/python/langgraph/checkpoints |
| Event-history replay | Temporal | Per Activity invocation | Yes, deterministic workflow replay | Distributed workers | temporal.io durable execution |
| Postgres step checkpoint | DBOS | Per workflow step | Yes, auto idempotency key | App-embedded library | dbos.dev/blog/durable-execution-crashproof-ai-agents |
| step.run cached unit | Inngest | Developer-defined step | Yes, retried+cached | Serverless | inngest.com/blog/durable-execution-key-to-harnessing-ai-agents |
| Stream barrier snapshot | Flink (Chandy-Lamport) | At barrier injection points | Yes, exactly-once | Massive distributed | nightlies.apache.org/flink |
| Pre-edit zip snapshot | Cursor Background Agents | Per code edit | Manual undo (not crash recovery) | Per-user remote sandbox | docs.cursor.com/en/background-agent |
| DAG-versioned trim (CMV) | ELAI SessionDag | Per phase/step + 85% token trigger | Yes, branch-and-explore | Single workstation + remote sidecar | arXiv:2602.22402 |
| 5-msgpack agent state | ELAI AgentCheckpoints | Per agent activity | Yes, last-checkpoint restore | Single-host pool | this report |
| Process-level C/R | CRIU + Crab | OS process | Yes, semantic-aware (Crab adds agent hooks) | Sandboxes | arXiv:2604.28138 |
| Compensating SAGA | Temporal, DBOS, Inngest, Restate | Per irreversible action | Forward-only with reverse-on-fail | Cross-system | learn.microsoft.com/azure compensating-transaction |
| Indexed external memory | Memex(RL) | Per interaction | Index dereference | Long-horizon | arXiv:2603.04257 |

## SessionDag vs LangGraph SqliteSaver vs Temporal

For ELAI's stated goal (single-workstation + remote sidecar, multi-hour runs, no central server, ~3 active branches, blake3 integrity):

- **LangGraph SqliteSaver** is the closest existing analog to SessionDag — both are local, file-backed, threaded by stable ID. Difference: LangGraph checkpoints at every node (50 nodes = 50 rows) while SessionDag uses CMV semantics (trim/branch/snapshot primitives). SessionDag is structurally richer but inherits the same blockers: AsyncSqliteSaver is explicitly not-for-prod due to SQLite write-throughput ceiling under contention (Medium / sparkco.ai). Mitigation: SQLite WAL mode (already on?) + batch writes per superstep, not per state mutation. Source: sqlite.org/wal.html (1000-page default threshold, async checkpoint).

- **Temporal** is the wrong fit for ELAI's single-host topology — it needs a server cluster and gRPC. But its primitives are the right primitives: every external call is an Activity, history is the source of truth, "Continue-As-New" caps history size. ELAI should steal three ideas: (1) wrap every irreversible tool call as a named activity with idempotency key, (2) make SessionDag history rollover when event count crosses a threshold (CMV trim already does this — just enforce a hard cap), (3) record activity start AND end so a crash mid-activity is distinguishable from crash-after-activity. Source: temporal.io/blog/durable-execution-meets-ai-why-temporal-is-the-perfect-foundation-for-ai.

- **What SessionDag already does better than either**: branch-and-explore. LangGraph time-travel exists but is single-thread; Temporal has no native branching. ELAI's 3-active-branch cap is good architecture; keep it.

- **What SessionDag is missing**: idempotency keys per tool call. ACRFence (arXiv:2603.20625) shows this is exploitable in LangGraph/Cursor/Claude Code today. ELAI should auto-generate `{session_id}:{phase_id}:{step_id}:{tool_name}:{call_seq}` and require every tool wrapper to accept it.

## Checkpoint compaction strategies

Cited:
- **Continue-As-New (Temporal)**: workflow atomically completes and starts a fresh workflow with carried-over state; event history bounded. Maps to ELAI: when SessionDag DAG depth crosses a threshold, snapshot current minimal state, mark old DAG as archived, start a new root. Source: zylos.ai/research/2026-03-04-ai-agent-workflow-checkpointing-resumability.
- **CMV trim primitive** (arXiv:2602.22402): formally defined structurally-lossless trim — ELAI already implements this. Pair it with a hard event-count ceiling.
- **ACON failure-driven guideline optimization** (Oct 2025): paired trajectories (full vs compressed) feed back into compression guidelines. Useful for ELAI as a post-mortem learning loop, not a hot-path checkpointer.
- **"Withdraw" / Sawtooth pattern** (jetbrains.com/research/2025/12/efficient-context-management): summary appended to persistent Knowledge block, delete messages between last checkpoint and current step. Converts monotonic log into sawtooth.
- **ExCP** (arXiv:2406.11257): weight+momentum joint shrinking for model checkpoint compression (more relevant for the sidecar's model state than agent state, but worth noting if model deltas ever get checkpointed).
- **Microsoft Agent Framework Compaction docs** (learn.microsoft.com/.../conversations/compaction): "near context limit -> summarize -> restart with summary" is the same pattern Anthropic's auto-compact uses; ELAI already triggers at 85%.

When to GC: bound by event count AND by elapsed wall-clock (Temporal docs note both). Summary checkpoint never deletes blake3-verified raw events — keep them on cold storage so any branch can re-derive deterministically.

## Crash-class taxonomy

What actually crashes long-running agents (cited, May 2026):

1. **Inference-engine timeouts** — ~40% of all inference failures, the single largest crash class (arXiv:2511.07424). Cause: model API hang, vLLM queue saturation, GPU OOM during decode.
2. **OOM kills** — Linux oom-killer or Windows job-object kill. Triggered by KV cache growth (real measured: KV at 25K tokens ~921 MB on Qwen 2.5-Coder 3B NF4), context bloat, RAG embedding cache leaks. Source: sitepoint.com/the-2026-definitive-guide-to-running-local-llms-in-production.
3. **Model API errors** — 429, 503, 400 (over-context). Anthropic + OpenAI APIs throttle on burst. LLM agents retry 15-30% of tool calls (Inngest "AI Agent Reliability Report 2025").
4. **Tool errors / API misuse** — top root cause in the 998-bug study (arXiv:2602.21806): API misuse + API incompatibility + Documentation Desync, concentrated in Self-Action stage.
5. **GPU thermal throttle / driver crash** — local-LLM-specific. Driver reset wipes CUDA context, model must reload. Source: sitepoint.com 2026 guide.
6. **Network partition / SSH tunnel drop** — ELAI-specific given the remote-sidecar topology. The PowerShell tunnel can die; sidecar OK, local agent hangs.
7. **Deterministic non-determinism** — even temp=0 generates different token sequences across runs due to GPU floating-point order (ACRFence, arXiv:2603.20625). Breaks any naive retry-equals-replay assumption.
8. **Multi-agent coordination failures** — MAST taxonomy: 14 failure modes / 3 categories (specification ambiguity, coordination breakdowns, verification gaps); 41-86.7% failure rate in production multi-agent systems (arXiv:2503.13657).
9. **Schema-version mismatch on resume** — checkpoint written by old code, resumed by new code (langgraphjs#536).
10. **Compaction/PreCompact races** — auto-compact fires mid-write to a tool, half-state snapshotted.

## Best additions for ELAI

Ranked by ROI:

1. **Idempotency-keyed tool wrapper layer.** Pattern: ACRFence + Temporal activity model. Grounding: arXiv:2603.20625 explicitly names Claude Code as vulnerable. Integration hook: every `mcp_*` call and every Bash/Edit/Write tool wrapper accepts an idempotency key `{session_id}:{phase}:{step}:{tool}:{seq}` and ELAI checks the key against AgentCheckpoint's tool_results before issuing the real call. Replay-or-fork semantic: if the prior call had irreversible effect, replay returns cached result; if not, fork allowed.

2. **SAGA log in SSB.** Pattern: Temporal / DBOS / Inngest compensating actions. Grounding: temporal.io/blog/compensating-actions, learn.microsoft.com compensating-transaction. Integration hook: SSB grows a `compensation_actions` table — every irreversible operation (file delete, git push, model unload, pod terminate) writes its inverse BEFORE the action executes. On crash-recovery, if SSB shows committed action without phase_complete marker, replay compensation in reverse order.

3. **Continue-As-New rollover for SessionDag.** Pattern: Temporal event-history cap. Grounding: zylos.ai checkpointing-resumability. Integration hook: when DAG node count crosses a tunable threshold (suggest 500 nodes or 100 phases), snapshot minimal carry-over state, archive old DAG under blake3 hash, start new root. Trim primitive already exists — add the cap-and-rollover policy.

4. **DST harness for AgentPool supervision.** Pattern: madsim / Antithesis deterministic replay. Grounding: github.com/madsim-rs/madsim, antithesis.com 2025 series A. Integration hook: replace tokio with madsim in `#[cfg(test)]` for AgentPool tests; inject crashes at every await point; check that 3-restart Erlang limit holds. Anti-spiral skill formalized as a test.

5. **Postcard for on-disk AgentCheckpoint format.** Pattern: postcard's deterministic byte order plays well with blake3 integrity check. Grounding: github.com/jamesmunns/postcard, david.kolo.ski/blog/rkyv-is-faster-than. Integration hook: behind a feature flag, switch the 5 msgpack files to postcard; keep msgpack only at the dashboard IPC boundary. Side benefit: 30% smaller checkpoints, less disk IO at 5-minute checkpoint cadence.

6. **Lazy schema migrator at checkpoint load.** Pattern: LangGraph's missing-feature workaround. Grounding: langgraphjs#536. Integration hook: each AgentCheckpoint embeds `schema_version`; on load, if version < current, run registered transform fns in order. Forward-compat reader; never big-bang. This unblocks the SCHEMA_VERSION 41/44/45 trio already in flight.

7. **Hierarchical checkpoint policy (task / phase / step / GRC-step).** Pattern: hierarchical preference learning (arXiv:2510.03253), HiPlan (arXiv:2508.19076). Grounding: granularity-mismatch papers all point to multi-level. Integration hook: task = SessionDag root snapshot; phase = SSB phase_complete marker; step = AgentCheckpoint 5-msgpack; GRC-step = WAL append into SSB (cheap, append-only). Crash recovery uses the finest available level.

8. **Reliability metrics (RDC / VAF / GDS / MOP) as observability.** Pattern: arXiv:2603.29231. Grounding: that paper. Integration hook: AgentPool emits these 4 metrics per run; auditor reads them to detect "meltdown onset" before a 4-hour run derails.

9. **Tool-error fast retry vs whole-task retry split.** Pattern: from arXiv:2603.22744 — runtime hooks recover 70% of errors with structured feedback. Integration hook: classify error at tool boundary; transient (network/429) -> tool-level retry with backoff; semantic (wrong arg) -> step-level replan; spec-level -> escalate to phase replan. Three buckets, three recovery paths. Not "always retry the call" and not "always retry the task."

10. **Indexed external memory (Memex(RL) pattern).** Pattern: arXiv:2603.04257. Grounding: that paper. Integration hook: SessionDag trim deletes nothing — moves trimmed nodes into a content-addressed store keyed by blake3 hash; agent can dereference by index. Already aligns with "structurally lossless" goal but adds an explicit retrieval API.

## Anti-patterns

- **MemorySaver-equivalent in prod.** LangGraph docs and 2025 best-practice guides explicitly ban this. ELAI's AgentCheckpoint is already disk-backed, but the dashboard's in-memory mirror MUST NOT be the source of truth on crash. (sparkco.ai 2025 best practices)
- **Synchronous checkpoint on hot path.** Slow writes become bottlenecks under load. Use async writer + bounded queue. (langgraph docs, sparkco.ai)
- **Random UUID thread_id per retry.** Stable IDs tied to business task only — random UUID per retry orphans prior checkpoints. (zylos.ai 2026-03-04)
- **Non-deterministic workflow code.** Temporal: anything that touches randomness, time, or I/O outside an Activity breaks replay. (temporal.io durable-execution)
- **"Retry the whole task" on tool error.** Inngest blog: 15-30% retry rate, must be at the right layer or you re-burn LLM tokens. Also wastes GPU on a 4-hour task.
- **Assuming `temperature=0` is deterministic.** False on GPU due to floating-point order across runs. ACRFence (arXiv:2603.20625) shows this enables semantic-rollback attacks.
- **Big-bang schema migration.** LangGraph's hard lesson — lazy transform at load is the only sane option for an append-only checkpoint store.
- **Single checkpoint level.** Hierarchical-planning papers (HiPlan, HPL) and hierarchical-RL paper STEP-HRL all show multi-granularity beats fixed granularity. ELAI 5-msgpack-per-agent alone is too coarse for a 4-hour run.
- **Compensation written AFTER the action.** SAGA pattern requires reverse logged BEFORE side effect, or you can crash with effect-applied-but-no-undo. (learn.microsoft.com compensating-transaction)
- **No event-count ceiling.** Unbounded history grows until SQLite stalls. Temporal's Continue-As-New exists precisely for this. (zylos.ai)

## Open questions

1. What event-count threshold triggers SessionDag Continue-As-New rollover without hurting branch coverage? Suggested measurement: instrument current 4-hour runs, plot DAG depth and SQLite write latency; pick threshold where p99 write exceeds 50ms.
2. Is the GPU floating-point non-determinism (ACRFence) actually exploitable in ELAI's threat model, or is it only an issue when an attacker controls a tool server? If only the latter, idempotency keys are enough; if the former, need replay-or-fork.
3. What's the right level for the WAL append — every mcp_think call, every tool call, or every state mutation? Measure: WAL write rate during a HEAVY-tier task; bound by SQLite WAL checkpoint interval (default 1000 pages).
4. Does the existing 85% token trigger compact race with the PreCompact hook's SESSION_HANDOFF write? Reproduce by injecting a crash 100ms into compaction; if SSB ends inconsistent, need a lock/barrier.
5. Can the dashboard tolerate postcard at the IPC boundary, or must msgpack remain? Measure: dashboard parse latency for postcard vs msgpack with current AgentCheckpoint sizes.
