operatingManual.md

RULE: Do not delete the template examples below. Add real entries above each template block.


Paths and environment

Target environment: Rust (edition 2021), tokio async runtime, Windows 11 + Linux
Source path: /workspace/ELAI\.elai_cc\
Workflow path: /workspace/ELAI\ (workflow tooling, CLAUDE.md, .claude/, memory-bank/)
Spec path: /workspace/ELAI\docs\specs\ (architecture, crate specs, decisions)
Guide path: /workspace/ELAI\docs\specs\guide\ (LLM implementation guide + step files)
Deploy path: .elai_cc/target/ (cargo build output)
Build command: cd .elai_cc && cargo check (fast) / cargo test (full) / cargo build --release (prod)
Log path: tracing-subscriber output to stderr (structured spans, no external telemetry)
Metrics path: .elai_cc/.elai_cc/metrics/phase_{N}.toml (per-phase measurements)
Config path: .elai_cc/elai_cc.toml (project config)
SSB path: .elai_cc/.elai_cc/projects/{id}/ssb.db (SQLite WAL)
Rules path: .elai_cc/.elai_cc/projects/{id}/rules.db (SQLite)

Template format:
Target environment: [engine, framework, or runtime version]
Source path: [absolute path to source code]
Deploy path: [absolute path to output directory]
Build command: [exact terminal command to compile and deploy]
Log path: [absolute path to primary debug log file]


Log tags

Format: [tag] / [when to grep]

[PhaseTransition]: FSM state change — grep when debugging workflow flow
[AgentSpawned]: agent lifecycle start — grep when debugging agent issues
[AgentCrashed]: agent failure — grep immediately on unexpected behavior
[SSBWrite]: state board mutation — grep when debugging inter-agent state
[GrcStep]: reasoning step — grep when debugging grounding/budget issues
[ToolExecuted]: tool call completion — grep when debugging tool behavior
[HookBlock]: pre-tool enforcement blocked an action — grep when tool calls fail silently
[EvidenceGateFailed]: phase transition blocked for missing artifact — grep when workflow stalls
[CircuitBreakerOpen]: knowledge engine component failure — grep when search degrades
[BudgetOverflow]: GRC topic budget exceeded — grep when reasoning seems truncated

Template format:
[tag]: [reason to check this tag]


Project hard rules

Entropy quarantine: zero raw std::time or rand. ClockProvider/EntropySource only. Clippy lints enforce.
Single-writer SSB: all SQLite WAL writes through tokio::mpsc channel. Never direct concurrent writes.
Evidence gates: no FSM transition without artifact proof. Test output, compilation result, etc.
Agent isolation: agents are OS processes. No network (except researcher via veth). No .elai_cc/ access. No LLM client in agent binary.
Self-mod safety: modifying agent never evaluates own modifications. 3 enforcement layers (IPC, filesystem, git).
No prompt-driven orchestration: workflow logic is Rust code. Rules are SQLite data evaluated by Rust interpreter.
Contract tests first: write failing tests before implementation. Auditor is sole test runner.
Measure everything: per-phase metrics in .elai_cc/metrics/phase_{N}.toml. Data decides, not the plan.

Template format:
[rule]: [why it must be strictly followed here]


Architecture and flow

elai_cc.toml -> core/ parses config, provides to all crates
cli/ -> reads user input, routes slash-commands to orchestrator/
orchestrator/ -> FSM engine drives phase transitions, spawns agents, manages SSB
agents (elai_cc_agent binary) -> isolated processes, communicate via IPC only
orchestrator/IPC gateway -> routes InferRequest to providers/, ToolRequest to tools/ (through rules/ + plugins/)
knowledge/ -> builds codebase index, answers queries from DKG+BM25+vectors+PageRank+CatRAG+A-RAG
reasoning/ -> GRC engine enforces grounded reasoning on all agent actions
rules/ -> deterministic Predicate DSL evaluation on every tool call (<1ms)
plugins/ -> hooks intercept tool calls, tiered static analysis on SSB writes
tools/ -> execute file ops, bash, git2, validation in orchestrator process (agents access via IPC)

Template format:
[component]: [how it interacts with the rest of the system]
[dependency]: [strict rules for using this dependency]
