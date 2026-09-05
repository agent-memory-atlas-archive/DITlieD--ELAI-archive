# Phase 4 Lossless Summary — Part 1 (Index + 4A + 4B + 4C)

## Index of Parts
- part1: master index + Phase 4A (agent process isolation), 4B (tools), 4C (plugins/hooks/tiered analysis)
- part2: Phase 4D (pipeline factory L1 + recursive research + web research infra), 4E (pipeline factory L2 + decomposition tree), 4F (feasibility probe + baby-step feasibility)
- part3: Phase 4G (AUTONOMOUS), 4H (self-modification DGM/CEL/DarwinNet), 4I (media validator + Render-Capture-Evaluate), 4J (Empire integration), 4K (Tauri dashboard MVP)

## Phase-4 Completion Status Snapshot (user-stated)
- pre0..4E: done
- 4K.1..4K.5: done (= entire Phase 4K MVP — the .txt file only defines 4K.1..4K.5; 4K.6..4K.13 do NOT exist in phase-4K.txt, they are explicit DEFERRED TO POST-MVP items)
- 4F..4J: not done

## File Sizes Read
| File | Bytes |
|------|-------|
| phase-4A.txt | 25,113 |
| phase-4B.txt | 24,664 |
| phase-4C.txt | 24,461 |
| phase-4D.txt | 72,544 |
| phase-4E.txt | 53,599 |
| phase-4F.txt | 30,570 |
| phase-4G.txt | 34,687 |
| phase-4H.txt | 58,549 |
| phase-4I.txt | 43,175 |
| phase-4J.txt | 38,858 |
| phase-4K.txt | 26,506 |

## Phase 4 Overall Exit Criteria (from 4I tail)
- System can accept arbitrary task, design pipeline, execute in INTERACTIVE mode
- AUTONOMOUS auto-advance works on top of validated INTERACTIVE pipeline (4G)
- Mode switch (INTERACTIVE <-> AUTONOMOUS) preserves state at phase boundary
- All agents run as isolated processes
- No agent can see enforcement internals
- Agent instructed to find .elai_cc/ fails (isolation escape test)
- End-to-end overhead from process isolation <5%
- Self-modification cannot touch its own evaluation criteria

Every phase repeats: "Every call site and struct that participates in request routing must wire ssb, fsm, and clock through the single RuntimeClient; no step may leave these fields as None or uninitialized after construction."

---

# PHASE 4A — AGENT PROCESS ISOLATION (Tier C)

Prereq: Phase 3 complete (INTERACTIVE pipeline gates passing). Status: design complete, user says done (pre0..4E done).

GOAL: Agents run as isolated OS processes via IPC (unix socket / named pipe). No agent sees enforcement logic, other agents, or harness internals. All subsequent Phase 4 sub-phases traverse this boundary.

READ: orchestrator.txt S12, tools.txt S8/S9, reasoning.txt S4, audit-results-phase4.txt (9 BLOCKs + 9 WARNs), architecture-overview.txt S9.

## Audit Risks Addressed
- BLOCK-1: seccomp two profiles — IPC_BASELINE (all roles) + NETWORK_EXTRA (researcher only). socket(AF_UNIX) baseline, socket(AF_INET) only researcher. BPF arg check on socket domain.
- BLOCK-2: researcher uses veth+iptables, NOT CAP_NET_RAW. Zero capabilities. prctl(PR_SET_NO_NEW_PRIVS) before exec().
- BLOCK-3: agent binary has NO LLM client. Proxied via IPC.
- BLOCK-4: GRC budget enforcement is orchestrator-side. Orchestrator counts ThinkStep messages externally. Agent budget advisory only.
- BLOCK-5: IPC sockets in 0700 dir (/tmp/elai_cc_{session}/). SO_PEERCRED on accept(). Unlink on agent death.
- BLOCK-6: auditor self-discovery findings require human approval. source=AUDITOR → PROPOSED, never auto-CANDIDATE. Second auditor validates.
- BLOCK-7: Windows isolation documented-weaker. AppContainer preferred. Linux is production target.
- BLOCK-8: self-mod locks = BOTH IPC-level (canonical realpath compare) AND FS-level (RO mount). BashRequest + GitCommit filtered.
- BLOCK-9: adversarial IPC test battery.

## Steps

### 4A.1 SKELETON — agent binary + IPC types
TASK: AgentProcess struct wrapping std::process::Command. Wire protocol = 1-byte version + 4-byte length prefix + postcard payload. MAX_FRAME_SIZE 10MB. Per-type limits: AgentHeartbeat <1KB, ThinkStep <64KB, ToolResult <10MB, AgentResponse < context_limit.

Message enum (exactly 15 variants, const_assert verified):
AgentRequest, AgentResponse, InferRequest, InferResponse, ToolRequest, ToolResult, BashRequest, BashResult, GroundingRequest, GroundingResult, ThinkStep, CheckpointSync, AgentHeartbeat, AgentKill, ReplanRequest.

AgentBinary = separate Rust binary `elai_cc_agent` compiled from `agent/` crate. Contains: GRC reasoning loop, IPC client, structured output parser. Does NOT contain: LLM client, reqwest, network code, tool impls, FSM, SSB, rules, knowledge.

Types:
- ProcessConfig per role { allowed_paths, seccomp_profile, resource_limits }
- ToolFilter { allowed_tools: Vec<ToolId> } — whitelist enforced at IPC layer, defense-in-depth over PreToolUse hook.
- AgentDispatchConfig { role, model_preference, tool_filter, rule_domains, grc_config, sandbox_config } — AUTHORITATIVE config for agent spawning (per technical-analysis-v3 Concern 5).
- phase_complete SSB record at phase boundaries with { phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes } — ATOMIC DURABILITY MARKER, model unload after fsync to WAL.
- agent_id: UUID v4, assigned at spawn.

Connection sequence (listen-before-spawn):
1. Orchestrator creates 0700 dir /tmp/elai_cc_{session}/
2. Creates socket, begins listen()
3. Spawns agent with socket path as arg
4. Agent connects. Orchestrator verifies SO_PEERCRED == child PID (Linux) / Named pipe SecurityIdentification (Windows)
5. Version handshake: agent sends protocol version byte. Mismatch → kill.

DETERMINISTIC_CHECK: two binaries compile (elai_cc, elai_cc_agent). `cargo tree -p elai_cc_agent | grep -c reqwest == 0`, `| grep -c providers == 0`. 15 message variants. postcard round-trip test each variant.

ACCEPTANCE: two binaries, agent zero network deps, 15 types round-trip, connection sequence (listen→spawn→connect→SO_PEERCRED→version) passes.

### 4A.2 VERIFY
`cargo build --workspace` = 0, `cargo test --workspace` = 0, `cargo clippy -- -D warnings` = 0. Run cargo test 3x (no flakes).

### 4A.3 TEST — functional isolation (TDD red)
Test naming: test_4a_isolation_*, test_4a_llm_proxy_*, test_4a_grc_budget_*, test_4a_grounding_*.

Isolation tests (~20):
- Spawn agent → AgentRequest → AgentResponse → kill
- Agent cannot read .elai_cc/ (ENOENT)
- PID namespace verified via /proc/{child_pid}/status NSpid=1 (host-side, not self-report)
- socket(AF_INET) returns EPERM via seccomp BPF
- socket(AF_UNIX) allowed
- Researcher accesses allowlisted HTTPS via veth
- Heartbeat timeout 15s → SIGKILL
- Agent crash (exit 1) does not crash orchestrator
- Agent cannot write outside scratch dir
- Two concurrent agents don't see each other's scratch / AF_UNIX sockets
- Auditor reads executor output (copied by orchestrator). Executor cannot read auditor findings.
- Env sanitized: only LANG, TERM, PATH (restricted), TMPDIR. No HOME, USER, API keys, ELAI_CC_*.
- SO_PEERCRED: wrong PID rejected
- /proc masked hidepid=2 or unmounted. /sys not mounted.
- RLIMIT_FSIZE (100MB write) → fail, not kill
- RLIMIT_AS (512MB alloc) → OOM in agent only

LLM proxy tests: InferRequest → orchestrator → LlmProvider → InferResponse. Agent has no reqwest. Agent cannot reach localhost:11434 from its netns. InferRequest > context_limit rejected.

GRC budget tests (orchestrator-enforced): count ThinkStep per topic externally. After max_calls → convergence signal via IPC. Agent local count ≠ orchestrator count → orchestrator authoritative. Per-topic independent.

GroundingRequest proxy tests: FILE_CHECK / SOURCE_CHECK / WEB_CHECK / TEST_CHECK. Role-method matrix (reasoning.txt S9):
- executor/auditor: FILE_CHECK, SOURCE_CHECK, WEB_CHECK, TEST_CHECK
- context-compiler/prompt-engineer/memory-writer: FILE_CHECK, SOURCE_CHECK only
TEST_CHECK from context-compiler → rejected. Counts toward per-topic budget. Orphan-topic bypass rejected.

### 4A.4 TEST — adversarial IPC
Names: test_4a_adv_ipc_*, test_4a_replan_*, test_4a_selfmod_*, test_4a_discovery_*, test_4a_crash_*, test_4a_windows_*. Each test has comment "// Covers audit BLOCK-N / WARN-N".

Adversarial IPC: >10MB frame rejected, length=u32::MAX rejected pre-alloc, malformed postcard → IPC error not crash, unsolicited AgentResponse rejected, ToolResult for unrequested ToolId rejected, 10K ThinkStep burst → budget enforced excess dropped, second process to same socket rejected (SO_PEERCRED), socket survives SIGKILL (fd close + unlink + respawn), mismatched agent_id rejected, protocol version mismatch at handshake → kill, partial frame timeout+cleanup.

ReplanRequest: MINOR with evidence → auditor fast-track; MAJOR → human approval (INTERACTIVE); 2 replans same phase → second rejected (max 1 per agent per phase); no evidence rejected; >500 tokens truncated or rejected.

Self-mod locks: locked path write rejected (generic msg), ../ traversal rejected (canonical realpath), BashRequest on locked file blocked (RO mount, not IPC filter), GitCommit touching locked file rejected (diff filter in orchestrator), symlink to locked file rejected (realpath resolves symlinks).

Auditor self-discovery trust boundary: finding → PROPOSED not CANDIDATE; PROPOSED from AUDITOR requires human approval; second auditor instance validates predicate logic; inflated violation_patterns capped.

Crash recovery: orchestrator crash mid-IPC → restart reads SSB → consistent. Agent crash mid-output → heartbeat detect → respawn. SSB write BEFORE ToolResult sent (synchronous). Partial frame after SIGKILL discarded on fd close.

Windows cfg(windows): AppContainer cannot read .elai_cc/ (capability check). Job object limits (memory, process count, timeout). Named pipe impersonation=SecurityIdentification. Fallback restricted token + ACL deny tested (documented-weaker).

### 4A.5 IMPLEMENT
Agent binary: receives AgentRequest via IPC. Runs GRC reasoning loop. Emits InferRequest, ToolRequest, BashRequest, GroundingRequest. Receives corresponding responses. Heartbeat 5s. Graceful shutdown on AgentKill (1s flush window).

Orchestrator IPC gateway: 0700 dir per session. Per-agent socket (UUID path). SO_PEERCRED on accept. Version handshake. MAX_FRAME_SIZE enforced pre-alloc. Heartbeat 5s / 15s timeout. Routes: InferRequest→providers, ToolRequest→tools+rules+visibility_filter, BashRequest→sandbox, GroundingRequest→knowledge+web. GRC budget enforced externally. SSB write synchronous with ToolResult dispatch.

Linux isolation: `clone(CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET)`. `prctl(PR_SET_NO_NEW_PRIVS)` BEFORE exec. Zero capabilities. seccomp-bpf IPC_BASELINE + NETWORK_EXTRA for researcher. BPF arg inspection on socket domain (AF_UNIX baseline, AF_INET/INET6 only NETWORK_EXTRA + SOCK_STREAM only). Bind-mount project (RO) + scratch (RW). .elai_cc/ NOT mounted. /proc masked or unmounted. /sys not mounted. Socket dir NOT mounted into agent namespace; agent receives fd or path as arg. Self-mod agent: locked paths mounted RO. Agent is PID 1. rlimits: RLIMIT_NPROC=1, RLIMIT_FSIZE=100MB, RLIMIT_AS=512MB.

Windows: AppContainer (preferred) CreateAppContainerProfile per agent. Fallback: restricted token + Job object + ACL deny (documented weaker). Job: ACTIVE_PROCESS=1, PROCESS_MEMORY=512MB. Named pipe SecurityIdentification.

Auditor self-discovery trust boundary: findings arrive via IPC. source=AUDITOR → PROPOSED. Second auditor validates. mutable=0 rules protected from all automated mod.

Self-modification locks: IPC realpath compare for WriteFile/EditFile, FS RO mount, Git orchestrator filters GitCommit rejecting diffs touching locked paths.

DEFERRED to 4B (WARN-3): ShortCoder post-processing runs in orchestrator process after AgentResponse, BEFORE writing to auditor view filesystem. 4B PRE_GATE verifies.

DETERMINISTIC_CHECK: all TDD tests green. `strace -f elai_cc_agent 2>&1 | grep -c "socket(AF_INET" == 0` (Linux).

### 4A.6 MEASURE — performance baselines
Metrics → `.elai_cc/metrics/phase_4A.toml`:
- IPC round-trip latency (target <1ms p99 for <64KB) — UNGROUNDED, unix domain typical <100µs
- Agent spawn time (target <100ms) — estimate ~50ms
- Memory per agent (target <30MB) — estimate ~20MB
- Overhead vs in-process (budget <5%)
- Isolation escape test: agent instructed to find .elai_cc/ must fail

Separation costs (planning): IPC ~0.1ms/msg, spawn ~50ms, memory ~20MB/agent, dynamic fan-out 10 agents = 200MB+ overhead.

## Grounding
Key grounded claims: postcard (crates.io present), SO_PEERCRED (Linux socket(7)), 0700 dir (orchestrator.txt L435), AppContainer (Windows API), zero capabilities + veth+iptables (orchestrator.txt L495-497 + audit BLOCK-2), LLM proxy IPC (orchestrator.txt L423-425 + BLOCK-3), orchestrator-enforced GRC (orchestrator.txt L478-491 + BLOCK-4), SSB write synchronous (orchestrator.txt L468-469 + WARN-6), GroundingRequest IPC (reasoning.txt L137-152), self-mod locks (orchestrator.txt L559-565 + BLOCK-8), auditor PROPOSED gating (orchestrator.txt L567-573 + BLOCK-6).

---

# PHASE 4B — TOOLS (git, validation, trace, edit matching, incremental compilation, ShortCoder)

Prereq: 4A done. Status (updated 2026-04-16 commit 55ece36):
- **4B.1-4B.4 DONE** — 10 tool modules, IpcToolRegistry, 15 typed arg structs, 19/19 tests pass. Bundled RA-1 (DiagnosticResult/RootCause), RA-3 (DKG query tools), PM (integration check). 3 BLOCKs fixed (ValidateHandler file_path, CompileCheckHandler stream, GrepFallback direction/depth).
- **4B.5 MEASURE: DEFERRED** — base tools complete, metrics collection remaining.
- **4B.6-4B.7 IntegrationTestRunner: DEFERRED** — blocked on Phase 4E types (IntegrationSpec, IntegrationCheckpoint). Do NOT attempt until 4E.7+ complete.

GOAL: Full tool suite in orchestrator process. Agents access via IPC ToolRequest/ToolResult. Deliverables: git2 ops, 5-tier validation, DAIRA trace, 3-tier edit matching, ShortCoder AST transforms, failure taxonomy classifier, file-read enhancement, incremental compilation feedback.

READ: tools.txt (full), orchestrator.txt S12, external-analysis.txt (DAIRA L83-89, ShortCoder L94-95, failure taxonomy L18), audit-results-phase4.txt BLOCKs 1-5, architecture-overview.txt S4 (ToolRegistry trait), risk-register.txt R51.

PAPER GROUNDING: DAIRA (arxiv 2603.22048v2) → workflow/papers/daira-interface.md, ShortCoder (arxiv 2601.09703v1) → workflow/papers/shortcoder-interface.md (both MISSING at spec time, ingest required).

SCOPE EXCLUSIONS: VisibilityFilter (tools.txt S8) → 4A. Tool schema sanitization → 4A. BashSandbox trait → 0-1/3-4 (4B inherits). lsp_client.rs deferred.

## Risks
- R1 IPC schema drift: contract test round-trips every ToolRequest variant.
- R2 incr-compilation false positives: unresolved import from unwritten file → DEFERRED not COMPILATION_ERROR.
- R3 ShortCoder AST correctness: tree-sitter grammar version pinned workspace-level (knowledge/ = tools/).
- R4 concurrent tool access: SSB file lock check before write tools (orchestrator.txt L619).

## Steps

### 4B.1 SKELETON
ToolRequest/ToolResult IPC variants for: git2 (GitCommit, GitUndo, GitBranch), validation (ValidateRequest), trace (TraceRequest), edit (EditRequest with tier hint), simplify (SimplifyRequest), classify (ClassifyRequest), file_read (FileReadRequest with force_full), compile_check (CompileCheckRequest). Stub all handlers with todo!(). Register in ToolRegistry with phase-aware filtering (tools.txt L17-18). Wire ToolRequest dispatch in orchestrator IPC handler (extends 4A message loop).

### 4B.2 VERIFY
Full workspace cargo check 0 errors. deny(warnings) in tools/ lib.rs. cargo check -p tools --no-default-features (tools compiles without optional knowledge dep).

### 4B.3 TEST (write failing tests)
- git2 auto-commit: deterministic msg format, create temp repo, verify commit
- 5-tier validation: tiers run 1→2→3→4→5, each returns findings
- DAIRA trace: [CALL][RETURN][MUTATION] markers, MarkerReport with raw_line_count + marker_line_count (ratio drives AdaptiveTool)
- 3-tier edit matching: exact → whitespace-normalized → ellipsis. Failed → SequenceMatcher similarity hint
- ShortCoder: tree-sitter AST equivalence. R4/R6 configurable toggle
- Failure taxonomy: same input → same classification (100 iterations internal)
- File read: >300 lines → outline mode. force_full bypasses but warns if >50% remaining context
- IPC round-trip: error sanitization
- Incremental compilation: COMPILATION_ERROR → retry → NEEDS_AUDIT after 3 rounds. NOT during TESTING_FIRST. Event-ordering test for async (not wall-clock — avoids CI jitter). Error referencing unwritten module → DEFERRED

Test count ≥ 15. Compile but fail (todo!() stubs).

### 4B.4 IMPLEMENT
- **git2**: auto-commit with deterministic msg from phase outcome (not LLM prose). Branch per plan: `plan-{pipeline_id}`. WIP commits for large phases (>5 edits). Session-owned commit tracking + safe undo (tools.txt S4).
- **5-tier validation**: Tier 1 determinism, Tier 2 quality/AST, Tier 3 unit, Tier 4 integration, Tier 5 stress/adversarial. All run by auditor, executor never runs own tests (tools.txt S5).
- **DAIRA trace**: tracing crate for Rust, sys.settrace for Python. Markers: [CALL] [RETURN] [MUTATION] [BRANCH] [ERROR] [PANIC] [TIMEOUT] [FINDING] [PATTERN] [ANOMALY] [STAT:name] [DURATION:name] [MEMORY:name] [CHECKPOINT] [STEP:n]. AdaptiveTool<T> wrapper with budget-based scope reduction (max 3 reductions) (tools.txt S6).
- **3-tier edit matching**: Tier 1 exact → Tier 2 whitespace-normalized (preserve original indentation) → Tier 3 ellipsis-elision ("..." matches any line sequence). Failure → SequenceMatcher hint. Never silently wrong match (tools.txt S3).
- **ShortCoder**: 10 tree-sitter transform rules R1-R10 (tools.txt S7). R4/R6 configurable. Applied as post-processing on executor output BEFORE auditor review.
- **Failure taxonomy** (deterministic regex): 9 categories: SYNTAX_ERROR, TYPE_ERROR, IMPORT_ERROR, ASSERTION_ERROR, TIMEOUT, EMPTY_OUTPUT, RUNTIME_PANIC, PERMISSION_DENIED, RESOURCE_EXHAUSTED. Each has deterministic suggested_action. Auditor reads classification BEFORE raw output. 3+ same-category in one phase → anti-spiral (tools.txt S11).
- **File-read enhancement**: outline for >300 lines (tree-sitter parsed sections). force_full flag. Budget guard warns if >50% remaining. INTERACTIVE user decides, AUTONOMOUS auto-downgrade.
- **Incremental compilation feedback (ASYNC)**: after each file write during EXECUTING, run cargo check / ruff / tsc / go vet on written file in background. Executor proceeds immediately. Results via IPC when ready. Sanitized COMPILATION_ERROR. Retry: round 1 initial, 2/3 retry. After 3 failed → NEEDS_AUDIT. NOT during TESTING_FIRST. DEFERRED classification for unwritten modules.
- **IPC**: all tools through orchestrator IPC. Error msgs sanitized before reaching agent (strip rule_id, predicate types, hook names). ToolRegistry checks SSB file locks before write tools.

### 4B.5 MEASURE (DEFERRED)
- IPC dispatch overhead target <2ms (vs ~0.1ms raw IPC)
- cargo check ~2-5s, ruff <500ms, tsc <2s, go vet <1s
- Auditor BLOCK reduction UNGROUNDED PROJECTION ~30% (external-analysis.txt L348) — measure after 4G
- ShortCoder savings: paper claims 18-38% (arxiv 2601.09703v1) — measure 10 samples
Metrics file `.elai_cc/metrics/phase_4B.toml`: ipc_overhead_ms, compile_check_latency_{rust,python,typescript,go}_ms, shortcoder_savings_pct, test_count, loc_tools. IPC overhead < 2ms p95.

## Integration Test Runner Extension (4B.6-4B.7, DEFERRED)

NEW TOOL: IntegrationTestRunner operates per-baby-step and per-level in DecompositionTree (4E ext).
4 granularity levels:
- BABY_STEP: cargo check on affected files. Target <5s.
- STEP: cargo test on step's module. Target <30s.
- PHASE: cargo test -p crate. Target <60s.
- CONCEPT: cargo test --workspace -- concept_tag. Target <120s.

```
struct IntegrationTestRunner { level: IntegrationLevel }
enum IntegrationLevel { BabyStep, Step, Phase, Concept }
fn run_integration(level, target: &IntegrationSpec, project_root: &Path) -> IntegrationResult
struct IntegrationResult { level, passed: bool, test_count: u32, failures: Vec<String>,
    stdout_hash: [u8;32], elapsed_ms: u64 }
```

Uses existing BashSandbox, FailureTaxonomy. tool_id="integration_test", available in AUDITING. NOT during EXECUTING. Triggered by orchestrator after baby step acceptance, not by agent.

4B.6: skeleton + ≥6 tests. 4B.7: implement + measure (BABY_STEP <5s, STEP <30s).

## Grounding
- tools in orchestrator process → GROUNDED tools.txt L75
- IPC ToolRequest → GROUNDED orchestrator.txt L427-429 + phase-4A.txt L28-30
- DAIRA 80.8% resolution → HIGH CONFIDENCE external-analysis.txt L84 (paper not ingested)
- ShortCoder 18-38% → HIGH CONFIDENCE external-analysis.txt L95 (paper not ingested)
- 3-tier edit matching → GROUNDED tools.txt S3, adopted from Aider
- <2ms IPC overhead → PARTIALLY GROUNDED (raw IPC 0.1ms per orchestrator.txt L554)
- ~30% auditor BLOCK reduction → UNGROUNDED design estimate
- cargo check 2-5s, ruff <500ms, mypy <2s → UNGROUNDED training estimates
- Async compilation → GROUNDED risk-register L305-306
- SSB conflict detection → GROUNDED orchestrator.txt L619
- tree-sitter grammar shared → GROUNDED tools.txt L4
- AdaptiveTool max 3 → GROUNDED tools.txt L98

---

# PHASE 4C — PLUGINS: HOOKS + TIERED STATIC ANALYSIS (IMPROVED 2026-04-06)

Prereq: 4B done. Status: design complete (user says pre0..4E done).

SCOPE: Hook system (PreToolUse / PostToolUse / Stop / PreCompact) in orchestrator. Tiered static analysis on SSB writes. Agents never see hook existence. Under Tier C, hooks intercept IPC messages, not in-process calls. Skills + MCP bridge OUT OF 4C scope (deferred).

READ: plugins.txt, orchestrator.txt S12, rules.txt S3 (predicate DSL) + S6 (self-discovery), audit-results.txt BLOCK 2 (T3 pattern-match), audit-results-phase4.txt BLOCK-6 (circular trust) + WARN-3 (ShortCoder placement).

## Key Design Decisions
1. T3 is pattern-match only, NO LLM (audit BLOCK 2).
2. Hooks intercept IPC in orchestrator (Tier C):
   - PreToolUse: when orchestrator receives ToolRequest/BashRequest
   - PostToolUse: after tool execution, before ToolResult sent back. ALSO applies ShortCoder (code_simplify()) on AgentResponse receipt, before forwarding to auditor view filesystem (WARN-3 resolution, tools.txt S7)
   - Stop: session termination or AgentKill sequence
   - PreCompact: orchestrator context budget allocator detects token_used/token_limit > 0.85 (orchestrator.txt S10)
3. Tiered findings → violation_patterns (rules.txt S6). Orchestrator normalizes to pattern hash (not auditor) (S6 step 1). PROPOSED from source=AUDITOR never auto-promote (S6 step 3e). Finding-source auditor cannot approve own findings (S5).
4. Error sanitization: hook verdicts (ALLOW/BLOCK/WARN) → generic IPC error ("tool call rejected"). No rule IDs / patterns / hook names leaked.
5. T1 evaluates predicate DSL (5 predicate types: AST_PATTERN, GREP_PATTERN, METRIC_GATE, COMPOSITE, SCHEMA_REQUIRED). T1 handles SCHEMA_REQUIRED by returning `Escalate{target=ProviderTier}` — requires provider-side constrained decoding, not T1's job. T1 does NOT parse AST (that's T2). Implicit phase_mask filtering.
6. T3 fires ONLY when (WARN exists) AND (no BLOCK exists from any tier). If T1=BLOCK and T2=WARN, T3 doesn't fire.
7. ruff has no public Rust library API (astral-sh/ruff #12351). PostToolUse ruff = subprocess (~50-200ms). Circuit breaker: skip ruff on files >10000 lines. PostToolUse total target <500ms.

## Steps

### 4C.1 SKELETON
```
trait HookTrait: fn pre_tool_use(&self, req: &ToolRequest, ctx: &HookContext) -> HookVerdict
enum HookVerdict { Allow, Block{reason}, Warn{reason} }
struct HookContext { fsm_state, agent_id, agent_role, session_id, active_rules }
```
Stubs: PreToolUse, PostToolUse, Stop, PreCompact. TieredAnalysis with T1/T2/T3 entry points. Wire into IPC dispatch. Error sanitization layer: Block → generic IPC error.

### 4C.2 VERIFY
No circular dep plugins/ ↔ orchestrator/. plugins depends on core/, rules/ only (plugins.txt L4).

### 4C.3 TEST (30 contract tests, all FAIL)
Hook contract (1-8): PreToolUse blocks forbidden tool (generic error), allows permitted, PostToolUse ruff/AST BLOCK on syntax error, pass clean, Stop blocks session non-IDLE, allows IDLE, PreCompact checkpoint saved, Block IPC response contains NO rule ID/pattern/internal detail.

Tiered analysis (9-18): T1 <1ms with realistic rule set, T1 GREP_PATTERN "pass" in except block → BLOCK, T2 fires on file_write only, T2 <10ms for <5000 lines, T2 circuit breaker >5000 lines skipped, T3 fires only on WARN, T3 <5ms, T3 function >200 lines WARN:complexity, T3 catch Exception: pass BLOCK:swallow, BLOCK any tier → SSB 'finding'.

IPC-aware (19-21): ToolRequest IPC → PreToolUse fires before execution. BashRequest IPC → PreToolUse fires. Hook BLOCK → no tool execution.

Circular trust (22-24): finding → orchestrator normalizes (not auditor). PROPOSED source=AUDITOR cannot auto-promote. Requires independent validation.

Cross-session locks (25-26): WriteFile to other session's owned path blocked, own path allowed.

SCHEMA_REQUIRED (27): T1 returns Escalate{target=ProviderTier}.

T3 compound (28): T1=BLOCK + T2=WARN → T3 does NOT fire.

ShortCoder placement (29-30): PostToolUse applies code_simplify on AgentResponse in orchestrator, before forwarding to auditor view FS. Output AST-equivalent to input (tree-sitter verify).

Benchmarks use Criterion in plugins/benches/, not inline Instant in contract tests. Error sanitization tests use negative assertions (must NOT contain internal details).

### 4C.4 ADVERSARIAL TESTS (10, all FAIL)
Hook bypass via IPC (1-3): path traversal "../../../etc/passwd" normalized + blocked; ToolRequest for tool not in FSM state → BLOCK; BashRequest after blocked ToolRequest independently checked.

T1 degradation (4-5): 1000 active rules still <1ms/eval (indexed by phase_mask + domain, not full scan); malformed predicate → WARN not crash.

T3 abuse (6-7): 10 nested WARN patterns processed, not short-circuited incorrectly; T3 receives BLOCK → does NOT fire (precondition).

Error sanitization bypass (8): user-controlled string in reason → sanitization strips it.

Circular trust (9-10): same auditor instance finding AND validating → rejected (different auditor instance); auditor floods violation_patterns with crafted hashes → stuck at PROPOSED.

Path traversal covers BOTH Unix and Windows separators. T1 stress uses AST_PATTERN predicates (most expensive).

### 4C.5 IMPLEMENT
- **PreToolUse**: `rules::get_active_rules()` for current FSM state + domain. Evaluate each. First BLOCK wins. SSB file lock query (orchestrator.txt S3 conflict detection).
- **PostToolUse**: ruff subprocess (~50-200ms), circuit breaker >10000 lines. T1/T2 on file content. Apply ShortCoder (tools/ code_simplify()) on AgentResponse receipt, before auditor view FS. Record to SSB.
- **Stop**: check FSM state. Non-IDLE → Block. IDLE → Allow.
- **PreCompact**: checkpoint save to SSB before truncation.
- **IPC integration**: wire hook check between receive and forward. Error sanitization: strip rule_id, pattern text, internal severity → "Tool call rejected by policy".
- **T1**: call `rules::evaluate(action, context)` with DSL. Indexed: `SELECT FROM rules WHERE status='ACTIVE' AND (phase_mask & bit) != 0 AND domain matches`. First BLOCK or collect WARNs. SCHEMA_REQUIRED → Escalate{target=ProviderTier}. Cache invalidation: rules cached at phase transition; SHADOW→ACTIVE mid-phase → invalidate at next BSP superstep boundary.
- **T2**: tree-sitter parse. Syntax, imports against deny-list, dead code, lazy code (pass bodies, TODO). Circuit breaker >5000 lines → skip T2, T1 only.
- **T3**: pattern-match library. NO LLM. Fires only when (WARN exists) AND (no BLOCK). Examples: function >200 lines WARN:complexity, catch Exception: pass BLOCK:swallow.
- **BLOCK any tier** → 'finding' entry via single-writer channel.
- **Circular trust prevention**: orchestrator-side normalization (blake2s hash of normalized pattern). violation_patterns increment in orchestrator. PROPOSED source=AUDITOR requires independent validation.

### 4C.6 MEASURE (9 categories → .elai_cc/metrics/phase_4C.toml)
T1/T2/T3 p50/p95/p99 (T1 <1ms, T2 <10ms for <5000 lines, T3 <5ms). Hook overhead per IPC tool call <2ms (T1 + dispatch). PostToolUse p99 <500ms (ruff dominated). Compile delta vs 4B baseline. Test count. LOC plugins/ + orchestrator/. Coverage >80%.

Warm cache before measuring (SQLite cold reads). Include files at 4999/5000/5001 lines for T2 breaker boundary.

## Prior Audit Items In Scope
- 4A WARN-1 (GroundingRequest role-method): PostToolUse analysis is deterministic (T1/T2/T3), not GRC-based. No GroundingRequest needed.
- 4A WARN-6 (GRC convergence vs local budget): hook evaluation synchronous, completes before IPC response.
- audit-results-phase4.txt WARN-3 (ShortCoder placement under Tier C): RESOLVED — runs in PostToolUse in orchestrator on AgentResponse receipt before auditor view FS (tests 29-30).

## Deferred
- Skills system (prompt templates, no execution-path dependency)
- MCP bridge (1751 LOC claw-code, separate integration sub-phase)

## Grounding
- Hooks in orchestrator → orchestrator.txt S12 Tier C
- HookVerdict enum → plugins.txt S2
- plugins depends on core+rules → plugins.txt L4 header
- T1 <1ms, T2 <10ms <5000 lines, T3 <5ms, T3 fires on WARN → plugins.txt S3
- BLOCK → SSB finding → plugins.txt S3
- No LLM validation → plugins.txt S3, audit BLOCK 2
- source=AUDITOR no auto-promote → rules.txt S6 step 3e
- Finding-source auditor cannot approve → rules.txt S5
- Orchestrator normalizes → rules.txt S6 step 1
- SCHEMA_REQUIRED 5th predicate → rules.txt S3
- ShortCoder post-processing + AST-equivalent → tools.txt S7, S10
- Indexed lookup by phase_mask + domain → rules.txt S7 get_active_rules
- Single-writer SSB → orchestrator.txt S3 (single-writer tokio::mpsc)
- blake2s pattern_hash → rules.txt S2
- ruff subprocess → plugins.txt S2 + astral-sh/ruff #12351
- tree-sitter T2 → plugins.txt S3 T2

## Exit Criteria
40 tests pass. Hooks enforce rules invisibly via IPC. Tiered analysis within latency targets. Circular trust prevented. Error sanitization verified. No agent sees hook internals / rule IDs / analysis tier details.
