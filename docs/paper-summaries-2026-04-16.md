Technical Summary: Three Papers on AI Agent Architecture
Date: 2026-04-16

================================================================================
1. Compiled AI: Deterministic Code Generation for LLM-Based Workflow Automation
================================================================================
arXiv: 2604.05150 | April 6, 2026
Authors: Geert Trooskens, Aaron Karlsberg, Anmol Sharma, Lamara De Brouwer, Max Van Puyvelde, Matthew Young, John Thickstun, Gil Alterovitz, Walter A. De Brouwer
Affiliations: XY.AI Labs (Palo Alto), Stanford Medicine, Cornell, Harvard/BWH

Core Idea
---------
LLMs run ONCE at compile/generation time to produce static, deterministic code.
Deployed workflows execute as plain code with zero runtime LLM calls and zero
inference tokens. This eliminates the 18-75% variance documented in temperature=0
models and achieves 100% reproducibility (H=0 entropy).

Architecture
------------
Four components:
- Orchestrator: receives YAML workflow specs, selects templates/modules
- Template Library: tested code patterns (sync handlers, streaming, batch, validators)
- Module Library: database access, HTTP clients, notification delivery
- Prompt Blocks: domain compliance constraints (HIPAA, PCI-DSS, SOC 2)

Four-Stage Validation Pipeline
------------------------------
1. Security: static analysis (Bandit, Semgrep) for SQLi, command injection, path traversal, secrets
2. Syntax: AST parsing, type checking (mypy), linting (ruff)
3. Execution: sandboxed testing against fixtures
4. Accuracy: golden dataset comparison with task-specific thresholds

"Safety Sandwich" Pattern (Healthcare)
--------------------------------------
Three isolated phases for prior authorization workflows:
1. Bounded agentic invocation for clinical data extraction (schema-constrained)
2. Hallucination validation (plausibility checks on extracted values)
3. Deterministic coverage rules (CMS decision timeframes)
Probabilistic extraction sandwiched between deterministic input/output validation.

Quantitative Results
--------------------
Function Calling (BFCL benchmark):
- 96% task completion (384/400), all failures caught at compile-time
- 100% deployed workflow reliability
- Generation cost: 9,600 tokens (one-time)
- Break-even: ~17 transactions
- At 1,000 tx: 57x token reduction vs Direct LLM, 84x vs AutoGen
- At 1M monthly tx: $555 TCO vs $22,000 (Direct LLM)
- Latency: 4.5ms P50 (vs 2,004ms Direct LLM) = 450x faster
- Jitter: 10.5ms (vs 1,123ms)

Document Intelligence (DocILE benchmark):
- Key field extraction: 80.0% (matches Direct LLM)
- Line item recognition: 80.4% (beats Direct LLM's 74.5%)
- Latency: 2,695ms (vs 6,339ms Direct LLM)
- Deterministic regex baseline: 20.3% KILE, 59.7% LIR, 0.6ms

Security Evaluation (135 test cases):
- Prompt injection detection: 95.8% recall, 100% precision
- Code safety analysis: 75% recall, 100% precision (zero false positives)
- Overall precision: 96.1%

Code Quality:
- Type coverage: 99%
- Test pass rate: 96%
- Cyclomatic complexity: 23.8 (higher than human code's ~8, expected for generated code)

Economic Model
--------------
Break-even formula: n* = GenTokens_compiled / (RuntimeTokens_per_tx_runtime - RuntimeTokens_per_tx_compiled)
Fixed generation cost amortizes rapidly; marginal execution cost approaches zero.

Key Takeaway
------------
For deterministic, high-volume workflows (especially regulated domains like healthcare),
compile-time code generation eliminates runtime inference entirely. The tradeoff is
higher cyclomatic complexity and a one-time generation cost that pays off after ~17 uses.


================================================================================
2. Sema Code: Decoupling AI Coding Agents into Programmable, Embeddable Infrastructure
================================================================================
arXiv: 2604.11045 | April 13, 2026
Authors: Huacan Wang, Jie Zhou, Ningyan Zhu, Shuo Zhang, Feiyu Chen, Jiarou Wu, Ge Chen, Chen Liu, Wangyi Chen, Xiaofeng Mou, Yi Xu

Core Idea
---------
Current AI coding agents lock reasoning capabilities inside a specific delivery form
(CLI, IDE plugin, web app). Sema Code decouples the core engine as a standalone npm
library, enabling the same reasoning kernel to power different clients without
modification. Validated with zero engine changes across VSCode extension and SemaClaw
(multi-platform gateway for Telegram/Feishu/Web).

Three-Layer Architecture
------------------------
1. Client Layer: UI/channel implementations (VSCode, messaging platforms)
2. Core Engine Layer: reasoning, tool invocation, state management (standalone npm lib)
3. Service Layer: external integrations (MCP, Skills, Plugins)

Event-driven architecture (not RPC-first). Heterogeneous outputs (text fragments,
tool results, permission requests) map onto a typed event stream.

Eight Core Mechanisms
---------------------

1. MULTI-TENANT ISOLATION
   Node.js AsyncLocalStorage (ALS) for continuation-local storage across async boundaries.
   Each engine instance binds to dedicated resource bundle: event bus, state manager,
   tool orchestrator, tenant configs. Two-tier resolution: ALS context first, global
   singleton fallback. Prevents conversation history leakage between concurrent users.

2. HIERARCHICAL STATE PARTITIONING
   S_session = <S_local, S_global>
   Agent-Local: S_local(ai) = <ei, Hi, Ti, Fi> where ei=execution status, Hi=isolated
   conversation history, Ti=agent-specific task list, Fi=file-read timestamps.
   Session-Global: g_edit (file modification rights), C_abort (unified abort controller).

3. FIFO INPUT QUEUE & SAFE SESSION RECONSTRUCTION
   dispatch(q) = enqueue if processing, startQuery if idle.
   Semantic batching: merges consecutive non-command messages; slash-commands execute
   individually. New session: pendingSession stored, abort signal to C_abort, aborted
   task's finally block purges S_local entirely before init. Zero state residue guaranteed.

4. ADAPTIVE CONTEXT COMPRESSION
   Uses cumulative token metadata from LLM API for O(1) monitoring (vs O(k) recalculation).
   Effective context = cumulative tokens + 8,000-token forward buffer.
   Compression at 75% of max context limit (25% safety margin).
   Primary path: semantic summarization partitioned at most recent user message (pivot).
   Degradation path: safe truncation at assistant messages to preserve API role alternation.
   Sub-agents bypass compression entirely; main agent manages their context lifecycles.

5. MULTI-AGENT COLLABORATIVE SCHEDULING
   One-level delegation only (no recursive spawning).
   Sub-agent gets fully isolated state: <H, T, F, S> with only C_abort shared.
   Three-phase lifecycle: Creation (unique session, dedicated prompt, restricted toolset)
   -> Execution (private state boundary) -> Cleanup (atomic purge, telemetry, synthesize result).
   Solves verbose tool output polluting main agent context.

6. INTERRUPT PROPAGATION & TOOL SCHEDULING
   Four execution boundary controls:
   - Post-Inference: cancellation placeholders for pending ops
   - Pre-Execution: bypass operation, yield cancellation context
   - Active Execution: distinguish user refusals (retained) from abort signals
   - Recursion Termination: flush unfinished results, collapse boundary
   Read-only ops execute in parallel; any write present forces strict serialization.

7. INTELLIGENT TODO-BASED PROCESS MANAGEMENT
   Deterministic task-tracking state machine. Unique ID per task, discrete states
   (pending/active/completed), mutual-exclusion (one active subtask at a time).
   Subset update if all IDs known; full replacement on unknown IDs.
   Sub-agent updates scoped locally, never pollute main agent UI.
   Eliminates UI flicker from LLM rephrasing variations.

8. BACKGROUND TASK EXECUTION
   Execution privileges decoupled from observation rights.
   Proactive route: agent flags operation, engine spawns isolated process, returns tracking ID.
   Reactive takeover: timeout exceeded, engine detaches shell session/file descriptors to
   background manager, resets primary shell singleton.
   Dual-write output: memory (low-latency streaming) + disk (persistent snapshots).
   Adaptive polling reduces sampling frequency during long executions.

Security: Four-Layer Permission Model
--------------------------------------
P: O x C -> {allow, deny, request}

L1 - File editing: session-scoped, fast-pass on edit flag
L2 - Shell commands: project-scoped whitelist prefix matching
     P_Bash(c) = allow iff head(c) in W for simple commands, all sub-commands in W for composed
     LLM-assisted static analysis for injection patterns (backtick substitution, process sub, chained destructive ops)
L3 - Skill invocation: project-scoped authorization
L4 - MCP operations: project-scoped name authorization

Asynchronous approval protocol (non-blocking): transient approval, persistent authorization,
explicit rejection, or user-guided correction (NL feedback injection).

Multi-Model Adaptation
----------------------
Two built-in adapters: Anthropic native (streaming thinking blocks as first-class events)
and OpenAI-compatible (opaque reasoning tokens). Unified interface for Extended Thinking,
Prompt Caching, tool-use schemas. Supports open-source (Code Llama, DeepSeek-Coder) and
commercial (DeepSeek V3/R1, Qwen3, Gemini 2.5 Pro).

Three-Tier Marketplace
----------------------
- MCP Service Marketplace: pre-integrated services from ClaWHub registry, one-click install
- Skill Marketplace: markdown + YAML frontmatter, priority loading (project > user > built-in)
- Plugin Ecosystem: command plugins (slash-commands) + hook plugins (lifecycle callbacks)

Validated Deployments
---------------------
VSCode Extension: exercises compression, permissions, background tasks, ecosystem.
SemaClaw: exercises multi-tenant isolation, FIFO queuing, async approval with inline buttons.
Both required zero engine modifications.

Acknowledged Limitations
------------------------
- Only 2 product forms validated (no CI/CD, Jupyter)
- No production-scale stress testing (hundreds of concurrent users)
- Context compression quality depends on underlying LLM; technical code may lose details
- WebSocket/gRPC streaming throughput and adversarial network recovery not benchmarked
- Single-process architecture; multi-instance state sync unaddressed


================================================================================
3. Agent RuleZ: A Deterministic Policy Engine for AI Coding Agents
================================================================================
Source: pub.spillwave.com (Spillwave Solutions)
GitHub: github.com/SpillwaveSolutions/agent_rulez

Core Idea
---------
A Rust binary that intercepts tool calls through Claude Code's hooks protocol,
enforcing governance via human-readable YAML rules. Pure deterministic rule engine
evaluation — no LLM interpretation. Sub-10ms evaluation guaranteed across dozens of rules.

Evaluation Pipeline (5 steps)
-----------------------------
1. Config Loading (config.rs): loads .claude/hooks.yaml, validates regex, rule uniqueness, expressions
2. Rule Sorting: by priority (higher numbers first)
3. Rule Evaluation (hooks.rs): matchers use AND logic; actions execute based on policy mode
4. Response Merging: blocks take precedence; injections accumulate
5. Logging (logging.rs): JSON Lines to ~/.claude/logs/rulez.log with governance metadata

Performance: LRU cache (100-entry) for compiled regexes. Sub-10ms evaluation.

Eight Matchers (AND logic — all active must pass)
--------------------------------------------------
tools          - filter by tool name               ["Bash", "Write"]
extensions     - match file types                   [".py", ".ts"]
directories    - target paths (glob)                ["src/api/**"]
operations     - trigger on event type              ["PreToolUse"]
command_match  - regex on command content            "git push.*--force"
prompt_match   - patterns in user input             ["deploy", "production"]
require_fields - validate field presence            ["file_path"]
field_types    - enforce field types                {"command": "string"}

Eight Actions
-------------
Blocking:
  block           - unconditional termination
  block_if_match  - block only if regex matches

Injection:
  inject          - load content from file path
  inject_inline   - embed markdown directly
  inject_command  - execute shell command, inject stdout

Validation:
  run             - execute external validator script (exit 0 = allow, non-zero = block)
  validate_expr   - boolean expression evaluation (evalexpr library)
  inline_script   - run embedded shell script

Sixteen Event Types (full lifecycle coverage)
---------------------------------------------
SessionStart, SessionEnd, UserPromptSubmit, PreToolUse, PostToolUse,
PostToolUseFailure, PermissionRequest, PreCompact, BeforeAgent, AfterAgent,
BeforeModel, AfterModel, BeforeToolSelection, Stop, Notification, Setup

Most rules target PreToolUse (before execution).

Three Policy Modes
------------------
Enforce: full activation (blocks, injections)
Warn: blocks become injected warnings, operations continue (training mode)
Audit: silent observation, log-only

Recommended rollout: Audit -> Warn -> Enforce.

Just-in-Time Context Injection
------------------------------
Solves context window pollution by injecting relevant context only when matched:

  - name: inject-python-standards
    matchers:
      tools: ["Write", "Edit"]
      extensions: [".py"]
    actions:
      inject: ".claude/skills/python_standards.md"

Python standards only loaded when editing .py files. No wasted context otherwise.

Governance Metadata
-------------------
Every rule can carry:
  governance:
    author: "security-team"
    reason: "Prevent accidental force pushes"
    confidence: high
    tags: ["security", "git"]

Logged with every decision for compliance dashboard integration.

Audit Log Structure
-------------------
JSON Lines format with: timestamp, event_type, tool, decision, matched_rules,
governance metadata, duration_ms. Log rotation: 10MB max, 5 rotated files.

Conditional Activation
----------------------
enabled_when expressions using evalexpr:
  enabled_when: 'env_CI != "true"'
Variables: env_* (environment), tool_name, event_type, prompt.

Multi-Agent Adapter Architecture
--------------------------------
Shared core (models.rs, config.rs, hooks.rs, logging.rs) with adapter pattern:
- Claude Code adapter: native hooks protocol (stdin/stdout JSON) — implemented
- Planned adapters: Gemini CLI, GitHub Copilot CLI, OpenCode
One hooks.yaml intended to work across all tools.

Configuration Example
---------------------
  version: "1.0"
  settings:
    log_level: "info"
    max_context_size: 1048576
    script_timeout: 5
    fail_open: true
    debug_logs: false
  rules:
    - name: block-force-push
      priority: 100
      mode: enforce
      matchers:
        tools: ["Bash"]
        command_match: "git push.*--force"
      actions:
        block: true
      governance:
        author: "security-team"
        reason: "Protect branch history"
        confidence: high

Advantages Over Native Claude Code Hooks
-----------------------------------------
- YAML readability vs custom scripts
- Built-in governance/audit metadata
- Graduated policy modes (audit/warn/enforce) vs binary hooks
- Just-in-time injection solves context pollution
- Multi-agent support (single rule set across tools)
- Deterministic evaluation, no LLM interpretation

Best Practices
--------------
Priority tiers: security (100+), standards (50-99), workflow (1-49).
Separate concerns: one rule for injection, another for blocking.
Version control hooks.yaml. Review rule changes in PRs.
Include governance metadata from day one.

Technical Specs
---------------
Language: Rust
Evaluation: <10ms
Config: YAML 1.0
Expressions: evalexpr library
Logs: JSON Lines
Cache: LRU (100 entries) for compiled regexes
Subprocess timeout: 5 seconds default
