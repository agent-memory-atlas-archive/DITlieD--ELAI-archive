> [!WARNING]
> Historical documentation for an abandoned experiment. This text is not evidence
> that the described behavior works. Read the [archive warning](../README.md) first.

WHAT ELAI IS

Short version
ELAI is a Rust-native development engine. You give it a task, it designs, builds, tests, audits, and ships the solution itself. Runs on consumer GPUs (12-48GB VRAM) using a local 3B-32B model, with optional cloud fallback. It is not a chat wrapper.

Core thesis
"The intelligence is in the infrastructure, not the model."
A 3B local model produces correct code because the system makes each task so small, so precisely defined, and so deterministically verified that even a weak model succeeds. The Rust binary controls the workflow. The model is a worker that receives a structured prompt, produces output, and the code validates it. The model never sees enforcement logic, never decides FSM transitions, never spawns agents, never picks context, never sees rules.

What "multi-agent" looks like at this level
Most "multi-agent" setups are: prompt -> agent A -> prompt -> agent B -> stitch outputs. Slow, leaky, brittle. ELAI is not that.

ELAI has 16 Rust crates organized into 9 architectural layers:
core, providers, knowledge, reasoning, rules, tools, plugins, orchestrator, cli — plus agent/ipc-types/web-search/scaffold/web-validator/dashboard/contracts.

The orchestrator is a 9-state finite state machine (IDLE, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE, CONFIGURING) with 7 evidence gates — every transition requires verifiable artifacts, not self-assessment. Each agent (Executor, Auditor, Designer, Researcher, ContextCompiler, MemoryWriter, PromptEngineer, MediaValidator) gets its own GRC budget, its own grounding constraints, its own anti-rubber-stamp rules. The auditor must produce >=1 FINDING: before it can converge, or it's rejected.

Some of what's actually inside

Knowledge engine (7 layers stacked):
DKG (tree-sitter -> SQLite graph) -> BM25 (tantivy) -> dual-vector (GraphCodeBERT 768d + Granite 384d via ONNX) -> SPLADE (gated) -> PageRank repo map (aider-derived, 5 improvements) -> CatRAG graph traversal -> A-RAG hierarchical -> cross-encoder rerank (ms-marco-MiniLM-L12-v2). Plus HyDE for complex queries, query tier routing (FAST/STANDARD/DEEP), AST-aware adaptive chunking. Per-project isolation. Sub-millisecond p99 on BM25.

Reasoning (GRC — Grounded Reasoning Chain):
State machine per topic: observe -> hypothesize -> plan_action -> act -> verify. Six grounding verification methods (FILE_CHECK, SOURCE_CHECK, TEST_CHECK, WEB_CHECK, API_CHECK, API_REGISTRY_CHECK). Counterfactual Safety Questions on functions touching files/data/network/auth — CRITICAL severity blocks progression without a guard. Plan grounding sweep: if any claim in the plan is ungrounded, EXECUTING is blocked.

Rules engine:
Rules are DATA in SQLite, not markdown. 7 predicate types (AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE, GRAPH_REACHABILITY, VERSION_GATE). Three enforcement tiers (deterministic <1ms, constrained decoding, prompt guidance). Rule lifecycle PROPOSED -> CANDIDATE -> SHADOW -> ACTIVE -> RETIRED is pure arithmetic — zero LLM in promotion. When the auditor catches the same violation 3+ times, the system auto-proposes a new rule.

Pipeline factory (3 layers):
L1 Designer (LLM, grounded): tree-structured research with parallel expansion, dual-threshold convergence, deterministic source quality tiering (arxiv/docs = T1, GitHub/SO = T2, blogs = T3, spam = T4). Recursive decomposition: Idea -> Concepts -> Phases -> Steps -> BabySteps. Each level triggers grounded research, not just the top.
L2 Builder (deterministic, no LLM): validates, sizes phases against the model's actual context budget, runs Kahn's topological sort + AutoB2G-derived DAG repair (max 10 rounds).
L3 Executor: runs the ExecutablePipeline through the FSM in INTERACTIVE or AUTONOMOUS mode.

BSP supersteps:
Macro-level orchestration via Bulk Synchronous Parallel model. 4 channel types (LastValue, Topic, EphemeralValue, UntrackedValue). snapshot -> clear topics -> execute -> apply writes -> clear ephemeral -> evidence gate -> checkpoint. Same seed = identical checkpoints. Full deterministic simulation testing via proptest-state-machine and madsim for IPC fault injection.

Agent isolation (3 tiers):
Tier A: filesystem visibility filter — agents see only allowed paths.
Tier B: bash in Firecracker/gVisor microVMs.
Tier C: each agent is a separate OS process with no LLM client, no network, no tools — everything proxied via 1+4+postcard IPC. 15 message types. Linux: clone(NEWNS|NEWPID|NEWNET) + seccomp-bpf + zero capabilities. Windows: AppContainer.

Self-modification (DGM/CEL/DarwinNet-derived):
The system improves its own configs, rules, and data tables from pipeline outcome data. Three layers: data (automatic, no approval), config (full pipeline, auditor approval, staged eval at 10/50/200 tasks), structure (human approval). Modification Solidification Index — exponential-decay weighted moving average. Auto-revert on >10% degradation over 3 pipelines. Anti-reward-hacking: metrics code, gate definitions, auditor config, test suite all read-locked during self-modification. The modifying agent never evaluates its own modifications — deterministic benchmarks judge.

The bit that matters
Agent reasoning chains run in separate processes and never enter the orchestrator's context window. Average prompt size: ~330 tokens of meta-instructions per turn, vs 1600-2600 in prompt-based "multi-agent" workflows. That difference is what makes a 3B local model viable.
