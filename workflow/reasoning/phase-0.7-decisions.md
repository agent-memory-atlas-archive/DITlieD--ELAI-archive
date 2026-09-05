Phase 0.7 decisions

What: make 5 contract tests GREEN (provider_trait, config_basic, fsm_basic, tool_basic, visibility_filter)

Key decisions:
1. provider_name as Option<String> with serde(default) on MessageResponse. All 6 existing construction sites remain valid (None default). Debug output contains "provider_name" satisfying contract assertion.
2. Add 6th ConfigEntry for elai_cc.toml in discover() — cwd-relative, ConfigSource::Project.
3. Visibility filter: intercept read_file/glob_search before dispatch. Paths containing ".elai_cc" return structured JSON error with error_code "visibility_filtered". Glob returns empty array "[]".
4. Structured errors: run_read_file wraps Err as JSON {"error":"...", "error_code":"file_not_found"}. Contract test parses this and checks error_code exists.
5. New tool arms: fsm_get_state returns JSON state, fsm_transition validates target and returns new state, list_hidden_paths returns JSON array of hidden patterns. All in execute_tool_with_enforcer match block.
6. FSM kept minimal: no external state — function-local. Enough for contract test which only checks Ok return.

Expected test outcome: 5 contract tests pass. 578+ existing tests unaffected (Option<String> with default is backward-compatible for serde, construction sites use None).

Unknowns: cli/main.rs construction sites (3) must compile with new field — but Option with default means existing JSON deserialization works and struct literals just need provider_name: None added. Wait — struct literals DO need the field. Let me check if cli tests construct MessageResponse directly... Yes, 3 sites at lines 8088, 8123, 8158. Must add provider_name: None to each.
