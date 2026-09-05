# C-35: Tool Attention Dynamic Gating + Lazy MCP Schema Loading

Objective: cut MCP-bridge tool-schema prompt cost via dynamic gating (only the K tools
the FSM state needs are visible to the model) and lazy schema loading (per-tool JSON
schema fetched on first reference, not at prompt assembly time). Closes the most
expensive token-cost leak in agentic workflows per arXiv:2604.21816 (95% measured
reduction in tool-prompt tokens).

Pre-requisites: Phase 4K MCP bridge SHIPPED (plugins/ HTTP JSON-RPC 2.0 at L274-400).
Parallel-safe with: C-36, C-37, C-38, C-39 (orthogonal). NOT parallel-safe with C-42
(Semantic Gateway zero-trust fuzzing) which touches the same plugins/mcp/server.rs
file paths — sequence C-35 first to avoid merge conflicts on the schema layer.

Paper grounding (arXiv:2604.21816, 2026-04-25, verified abs page 2026-05-17):
"Tool Attention Is All You Need: Dynamic Tool Gating and Lazy Schema Loading for
Eliminating the MCP/Tools Tax in Scalable Agentic Workflows."
Measured claim: ~95% reduction in tool-related prompt tokens for an agent with 30+
registered tools, with no measured degradation on tool-selection accuracy when the
gate is per-FSM-state. CAVEAT surfaced by verifier 2026-05-17: the paper's downstream
quality figures are PROJECTIONS (Section 4 ablation), not measured on live agents.
Only the 95% token-reduction figure is grounded measurement. Treat the quality claim
as hypothesis, gate adoption on ELAI's own SSB-emitted accuracy delta.

## Context

ELAI's MCP bridge (plugins/) currently emits the full tool catalog into every
LLM prompt at every FSM step. With 30+ tools registered (workspace, knowledge,
git, fs, runtime backends, paper-ingestor, source-fetcher, contract-validator, etc),
the static schema dump dominates prompt assembly cost — assessment file §9 already
flags "20-40 LLM calls per feature, wall clock per feature is HIGH" and the tool
schema is a multiplier on each call.

The fix is two-layer:
1. **Dynamic gating** — per FSM state declare the allowed tool set in a static
   manifest (toml/json). The MCP bridge intersects the registered catalog with the
   state's allowed set at prompt assembly time. Tools outside the gate emit a
   single-line "<N tools available on request>" marker instead of their full schema.
2. **Lazy schema loading** — the gated tools emit only their NAME + 1-line summary
   in the prompt. Full JSON schema for params/return is fetched ONLY when the model
   emits a tool-call of that name. Reduces typical prompt cost from ~6-10K tokens
   of catalog to ~300-500 tokens of summary.

This is the highest-leverage TRIVIAL patch in the C-35..C-59 sequence. Direct
fit to existing Phase 4K MCP bridge.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md (Phase 4K MCP bridge section)
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4K.txt
- .elai_cc/crates/plugins/src/mcp/server.rs (current static-catalog emission path)
- .elai_cc/crates/plugins/src/mcp/schema.rs (current tool schema definitions)
- .elai_cc/crates/orchestrator/src/fsm.rs (FSM state declarations for gate mapping)
- workflow/papers/tool-attention-interface.md (paper-ingestor output, see Task 1)

## Tasks

1. **Source-grounding (4K.X)** — spawn paper-ingestor (mode=paper) against
   arXiv:2604.21816. Produces workflow/papers/tool-attention-interface.md
   capturing the per-FSM-state gating algorithm and the lazy-loading protocol.
   Required before any code change.

2. **elai_cc.toml [tool_gating] section** — add config block with keys:
   enabled (bool, default true after acceptance), gate_manifest_path (string,
   default "plugins/tool-gate.toml"), lazy_schema_default (bool, default true),
   max_eager_schemas (int, default 4 — number of tools whose full schema is
   ALWAYS in prompt regardless of gate). Validate at startup.

3. **plugins/tool-gate.toml manifest** — declarative gate per FSM state.
   Example:
   ```
   [gate.PLANNING]
   eager = ["knowledge.search", "knowledge.symbol_impact"]
   gated = ["fs.read", "fs.grep"]
   denied = ["git.commit", "runtime.invoke"]

   [gate.EXECUTING]
   eager = ["fs.read", "fs.edit", "fs.write"]
   gated = ["knowledge.search", "git.diff", "runtime.invoke"]
   denied = []
   ```
   Each state lists eager (full schema in prompt), gated (name + 1-line summary),
   denied (omitted entirely, tool-call attempt returns error).

4. **plugins/src/mcp/gate.rs (NEW)** — ToolGate struct + load_gate(manifest_path)
   + filter_for_state(fsm_state, full_catalog) -> GatedCatalog. GatedCatalog
   has three fields: eager_schemas (Vec<FullSchema>), gated_summaries
   (Vec<SummaryRow>), denied_names (HashSet<String>).

5. **plugins/src/mcp/server.rs (MODIFY)** — replace the static
   render_full_catalog() call with render_gated_catalog(fsm_state). When the
   model emits a tool-call of a gated name, fetch the full schema lazily via
   resolve_schema_on_demand(tool_name) and inject for that single call's
   response cycle (NOT into the next prompt assembly unless the tool stays
   in active use, controlled by an LRU of size max_eager_schemas).

6. **plugins/src/mcp/lazy.rs (NEW)** — LazySchemaCache with LRU eviction,
   per-tool lock to prevent thundering-herd on concurrent first-references,
   metric emission (cache_hit, cache_miss, schema_resolve_ms).

7. **SSB action types** — add 4 new strings to the SSB schema:
   tool_gated, tool_eager, tool_lazy_resolved, tool_denied. Each row carries
   tool_name, fsm_state, prompt_tokens_saved. SCHEMA_VERSION bump required.

8. **Tests first (TDD, 4K.X.3 - 4K.X.7)** — write before production wiring:
   - 4K.X.3: ToolGate filter_for_state produces correct three-way split on a
     synthetic 10-tool catalog
   - 4K.X.4: render_gated_catalog respects max_eager_schemas LRU under
     repeated state transitions
   - 4K.X.5: resolve_schema_on_demand returns the FULL schema (round-trip
     equal to the registered schema, byte-for-byte)
   - 4K.X.6: denied tool-call attempt returns a structured error visible in
     SSB (tool_denied row, NOT a panic)
   - 4K.X.7: prompt_tokens_saved metric on tool_gated row is non-zero and
     matches the difference between full and summary serializations

9. **Production wiring (4K.X.8)** — wire ToolGate into the orchestrator's
   prompt assembly path (plugins/src/mcp/server.rs::assemble_prompt). Bind
   SchemaCache to the LazySchemaCache instance. Wire SSB emissions.

10. **--tool-gate-off CLI flag** — escape hatch to disable gating at runtime
    (debugging). Logs "[WARN] tool gating disabled; prompt costs will be 5-10x
    higher" at startup.

11. **Dashboard panel (dashboard-v2/ToolGateInspector.svelte, PLANNED)** —
    shows current FSM state's gate, eager schemas, gated summaries, and
    a rolling 60s histogram of tool_*_tokens_saved deltas. Uses existing
    SSB stream.

12. **Contract tests** — tests/contracts/tool_gate_contract.rs verifying
    ToolGate interface shape against the wiring contract (stub_ban enforced
    on render_gated_catalog).

## WIRING_CONTRACT

produces_symbols:
  - ToolGate, GatedCatalog, SummaryRow (plugins/src/mcp/gate.rs)
  - LazySchemaCache (plugins/src/mcp/lazy.rs)
  - render_gated_catalog, resolve_schema_on_demand (plugins/src/mcp/server.rs)
  - SSB action_types: tool_gated, tool_eager, tool_lazy_resolved, tool_denied
  - ToolGateInspector.svelte (dashboard-v2, PLANNED)
  - --tool-gate-off CLI flag
  - plugins/tool-gate.toml manifest
  - elai_cc.toml [tool_gating] config section
consumes_symbols:
  - FullSchema, ToolName (plugins/src/mcp/schema.rs — extend non-destructively)
  - FsmState (orchestrator/src/fsm.rs)
  - SSB event bus (orchestrator)
reachable_from:
  - prompt assembly path (every LLM call)
  - elai_cc.toml [tool_gating] config
  - --tool-gate-off flag
stub_ban:
  - render_gated_catalog MUST NOT silently fall through to full catalog when
    gate manifest missing — must emit tool_gated SSB with reason="manifest_absent"
    and use a safe default (eager=[], gated=ALL, denied=[])
  - resolve_schema_on_demand MUST NOT cache silently; cache_hit/cache_miss SSB
    rows are mandatory
  - denied tool-call returns structured error, never panic
frontend_pair: ToolGateInspector.svelte (PLANNED — implement after Rust types)
observable_effect:
  trigger: /task --mode local "implement X" with 30+ tools registered
  expect_ssb: tool_gated rows appear with prompt_tokens_saved > 0 on every
              prompt assembly; tool_lazy_resolved on first reference of any
              gated tool; cumulative prompt_tokens_saved per feature in the
              5-10K range (matching paper's measured 95% catalog reduction)
  expect_artifact: dashboard ToolGateInspector shows active gate + non-empty
                   histogram

## DETERMINISTIC_CHECK

```bash
# 1. gate.rs + lazy.rs exist
test -f .elai_cc/crates/plugins/src/mcp/gate.rs
test -f .elai_cc/crates/plugins/src/mcp/lazy.rs

# 2. Six new symbols present
rg "ToolGate|GatedCatalog|SummaryRow|LazySchemaCache|render_gated_catalog|resolve_schema_on_demand" .elai_cc/crates/plugins/src

# 3. 4 SSB tool action types
rg "tool_gated|tool_eager|tool_lazy_resolved|tool_denied" .elai_cc/crates/orchestrator/src

# 4. Gate manifest exists with at least 3 FSM states
test -f .elai_cc/plugins/tool-gate.toml
grep -c "^\[gate\." .elai_cc/plugins/tool-gate.toml | awk '$1 >= 3 {exit 0} {exit 1}'

# 5. Tests pass
cargo test -p elai_plugins -- tool_gate 2>&1 | grep "test result.*ok"

# 6. CLI flag wired
rg '"tool-gate-off"' .elai_cc/crates/cli/src/main.rs

# 7. Contract test exists and passes
cargo test -p elai_plugins -- tool_gate_contract 2>&1 | grep "test result.*ok"

# 8. SCHEMA_VERSION bumped
grep "SCHEMA_VERSION" .elai_cc/crates/orchestrator/src/ssb/schema.rs | head -1
```

## ACCEPTANCE

SHIPPED-ON-V2 (library tier, 20 items): gate.rs + lazy.rs + tool-gate.toml +
elai_cc.toml [tool_gating] + 4 SSB arms (SCHEMA_VERSION 88 bump at original C-35
ship; now LOCKED at 103 -- no bump this session) + 5 unit + 3 contract tests +
ToolGateInspector.svelte + --tool-gate-off CLI parse / WARN. Completion-drive
rebaseline this session shipped: render_gated_catalog + resolve_schema_on_demand
free-function aliases (M1+M2), `mcp::server` re-export of mcp_bridge (M3),
Summarize trait + McpToolSpec impl (M4), DefaultMcpBridge::assemble_prompt seam
(M5), ToolGate + LazySchemaCache as DefaultMcpBridge fields (N1), use statements
in production code (N2), 4 tool_* SSB emit sites via ToolGateSsbEmitter trait
(N3), ToolGateInspector mounted in SettingsWindow (N4), get_tool_gate_snapshot +
list_tool_gate_rows Tauri commands registered in invoke_handler! (N5), Svelte
panel invoke() calls (N6), ELAI_TOOL_GATE_OFF env reader at bridge construction
(N7), OrchestratorToolGateSsbEmitter adapter, +2 new wiring tests (assemble
emission, SafeDefault emission-skip), +2 dashboard DTO round-trip tests.

DEFERRED: A5 1000-iter LRU stress (fast-follow plan); A10 5-10K prompt_tokens_saved
measurement (POST-SESSION; requires live agent run); A15 SWE-bench ±2pp solve-rate
delta (RUNPOD-PENDING, C-54 dependency).

- [x] paper-ingestor produces workflow/papers/tool-attention-interface.md
- [x] elai_cc.toml [tool_gating] block validates at startup
- [x] plugins/tool-gate.toml manifest covers ALL FSM states declared in fsm.rs
- [x] ToolGate, GatedCatalog, LazySchemaCache compiled with no placeholder bodies
- [ ] LazySchemaCache LRU honors max_eager_schemas under 1000-iteration stress
      test (DEFERRED A5)
- [x] denied tool-call returns structured error, NOT panic (verified by 4K.X.6)
- [x] resolve_schema_on_demand returns byte-equal schema vs registered (4K.X.5)
- [x] SCHEMA_VERSION bumped; SSB migration tested (bump at v88; LOCKED at 103
      this session per pre-resolved decision #2)
- [x] All 4 SSB tool action types emitted with non-null tool_name + fsm_state
      (via OrchestratorToolGateSsbEmitter + DefaultMcpBridge::assemble_prompt /
      resolve_schema_on_demand / dispatch_tool_call_with_gate)
- [ ] prompt_tokens_saved per feature falls in 5-10K range on 30-tool synthetic
      workload (matches paper's 95% catalog reduction expectation) (DEFERRED A10)
- [x] TDD tests 4K.X.3..4K.X.7 written and passing BEFORE 4K.X.8 wiring
- [x] Contract tests in tests/contracts/tool_gate_contract.rs pass
- [x] --tool-gate-off CLI flag disables gating and logs warning (env reader
      added at bridge construction; double-set path also logs WARN)
- [x] ToolGateInspector.svelte renders gate + histogram from SSB stream
      (mounted in SettingsWindow alongside EmpireBridgePanel; reads via invoke())
- [ ] **ELAI-side accuracy delta measured** (NOT taken on faith from paper's
      projection): run C-54 SWE-bench Verified subset twice -- once with gating
      on, once with --tool-gate-off -- and confirm task-solve delta is within
      ±2pp. (DEFERRED A15: RUNPOD-PENDING via C-54)

## Files

- ADD: .elai_cc/crates/plugins/src/mcp/gate.rs
- ADD: .elai_cc/crates/plugins/src/mcp/lazy.rs
- ADD: .elai_cc/plugins/tool-gate.toml
- ADD: .elai_cc/crates/plugins/tests/tool_gate.rs (TDD 4K.X.3..4K.X.7)
- ADD: .elai_cc/crates/plugins/tests/contracts/tool_gate_contract.rs
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/ToolGateInspector.svelte (PLANNED)
- ADD: workflow/papers/tool-attention-interface.md
- MODIFY: .elai_cc/crates/plugins/src/mcp/server.rs (render_gated_catalog wiring)
- MODIFY: .elai_cc/crates/plugins/src/mcp/schema.rs (extend with Summary trait)
- MODIFY: .elai_cc/crates/plugins/src/lib.rs (module exports)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (4 new action types + SCHEMA_VERSION bump)
- MODIFY: .elai_cc/crates/orchestrator/src/fsm.rs (expose fsm_state to MCP bridge)
- MODIFY: .elai_cc/crates/cli/src/main.rs (--tool-gate-off flag)
- MODIFY: .elai_cc/elai_cc.toml ([tool_gating] section)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4K.report.md
  — add C-35 closure annotation upon completion.

## Next

After C-35 lands: confirm prompt_tokens_saved metric in SSB before moving to
C-36 (which is orthogonal but benefits from the lower per-prompt baseline cost
when running mutation-test sweeps). The gating layer must be live BEFORE C-54
benchmark publication so the published numbers reflect the production prompt
shape, not the legacy full-catalog cost.
