Token Optimization Tier 1+2 Decision Doc

What: Added priority: u8 and compact_schema: Option<Value> fields to ToolSpec. Assigned P0/P1/P2 priority values to all 52 tool entries (49 mvp + 3 internal). Replaced FIFO take() with priority-sorted selection in both dispatcher.rs and cli/main.rs.

Key decision: Circular dependency (tools -> orchestrator) prevented importing elai_tools from orchestrator. Solution: added priority_map: &HashMap<String, u8> parameter to build_dispatch_context so the caller (who can import elai_tools) provides priority data. cli/main.rs builds its own priority_map from mvp_tool_specs() at the sort site.

Priority assignment:
P0 (0): bash, read_file, write_file, edit_file, glob_search, grep_search, fsm_get_state, fsm_transition, list_hidden_paths
P1 (1): WebFetch, WebSearch, TodoWrite, Agent, REPL, AskUserQuestion
P2 (2): all remaining 37 tools

Expected test outcome: All existing tests pass with no behavioral change when tool count is below max_tools (no sorting needed). When tool count exceeds max_tools, highest-priority tools are retained instead of insertion-order first tools.

Test results: tools 60/60 GREEN, orchestrator 736/736 GREEN, workspace cargo check clean.

Unknowns: compact_schema is None for all entries (schema compression deferred to follow-on). Per-turn HashMap allocation in main.rs stream() is O(49) and negligible but could be cached if profiling shows otherwise.
