projectBrief.md

Load when: starting a new session on an unfamiliar area, or answering a scope question.


What this project is

elai_cc is a Rust-native autonomous LLM workflow engine. Not a CLI assistant — an autonomous development engine where the human provides a task and the system executes the full development cycle with minimal human intervention. It is model-agnostic (cloud + local), per-project (all state lives inside project directory), deterministic-first (all outputs verified by deterministic scripts), and entropy-quarantined (all time/randomness through injectable traits for deterministic simulation testing). Two deliverables: D1 (standalone elai_cc, personal use first) and D2 (Empire integration, deferred until D1 proven).


Hard constraints

Entropy quarantine: zero raw std::time or rand access anywhere in codebase. All through ClockProvider/EntropySource traits. Non-negotiable because DST (Phase 3I) requires deterministic replay — same seed must produce identical output.
Never compact: no context summarization or compression. Use full context smartly, hand off to new session if exhausted.
Code-driven orchestration: ~330 tokens of meta-instructions per turn, not 1600+. Workflow logic is Rust code, not prompts.
Per-project isolation: all state, indexes, graphs, RAG live inside project directory. No shared global state.
Evidence gates: every FSM transition requires artifact, not self-assessment. No phase advances on vibes.
Agent isolation: agents run as OS-isolated processes. No agent sees enforcement logic, other agents, or harness internals.
No proprietary deps: open-source compliant, clean licensing.
Path-agnostic: Windows + Linux without path assumptions.
Token efficiency: msgpack storage, postcard wire format, phase-aware context loading.


Stack

Language: Rust (edition 2021)
Runtime: tokio async runtime (replaced by madsim in DST tests)
Build tool: cargo (cargo check, cargo test, cargo build)
Target environment: Windows 11 + Linux, 1x RTX 3060 12GB (Phase 0), 2x RTX 3090 24GB (production)
Local models: Gemma 4 31B (executor), DeepSeek-R1-Distill-32B (auditor), Qwen 3.5 27B (prompt-engineer)
Base fork: claw-code-rust-port (59,449 LOC, 69 .rs files, 666 tests, compiles clean)


Scope

In scope:
  - 9-crate workspace: core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli
  - FSM: IDLE->PLANNING->FEASIBILITY_PROBE->TESTING_FIRST->EXECUTING->AUDITING->MEMORIZING->COMPLETE->IDLE
  - Knowledge engine: DKG, BM25, dual-vector, SPLADE, PageRank, CatRAG, A-RAG, cross-encoder (7 layers)
  - GRC reasoning engine with grounding verification and per-agent budgets
  - SSB (SQLite WAL inter-agent state), BSP supersteps, session DAG handoff
  - Self-improving rules engine (Predicate DSL, 3-tier enforcement, self-discovery)
  - Pipeline factory (L1 designer, L2 builder, L3 executor)
  - Self-modification (DGM/CEL/DarwinNet-derived, 3 layers, staged evaluation)
  - Agent process isolation (Tier C: PID/mount/network namespace, seccomp, IPC)
  - Media validator (conditional, 2-tier: deterministic + VLM)

Out of scope (locked):
  - Cloud-only operation (must work fully local)
  - GUI/web interface (CLI only for D1)
  - Empire integration (D2, deferred until D1 proven)
  - Multi-user/multi-tenant (single user, single machine)
  - Real-time streaming/trading (this is a development engine, not a trading system)


Key references

docs/specs/guide/implementation-guide.txt: LLM implementation guide (entry point for phase work)
docs/specs/guide/steps/: per-phase step files with READ directives
docs/specs/architecture-overview.txt: system architecture map (crates, traits, data flow)
docs/specs/implementation-plan.txt: full phase breakdown (source of truth)
docs/specs/design_index.txt: index of all 20 spec files
docs/specs/crates/: per-crate detailed specs (9 files)
docs/specs/integration/: integration specs (5 files)
docs/specs/decisions/: decision log, audit results, risk register (5 files)
docs/elai_cc_design_document.txt: ARCHIVE — original monolith, do not edit
