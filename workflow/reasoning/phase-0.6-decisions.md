Phase 0.6 decisions

What: contracts crate (elai_cc_contracts) — test-only TDD skeleton with 5 integration test files.

Key decisions:
1. Empty src/lib.rs required for Cargo target — spec says "no lib.rs" meaning no implementation, not no file.
2. elai-cli excluded from dev-deps — it has no lib target (only bin), cannot be imported.
3. Tool dispatch names verified against execute_tool_with_enforcer: read_file (not file_read), glob_search (not glob).
4. Runtime-fail strategy: import existing types, assert on absent contract surface (provider_name field, fsm tools, structured error_code, elai_cc.toml config entry, visibility_filtered error code).
5. visibility_filter has 3 tests (not 1) to cover spec's 3 sub-requirements.
6. All 8 non-cli crate dev-deps kept per spec even though only 3 are directly imported — spec mandates all workspace crate deps.

Expected test outcome: 5 binaries FAIL, 7 test functions fail, 0 pass. Verified.

Unknowns:
- Whether fsm contract should test tool registration (current) or direct Fsm trait (doesn't exist yet). Chose tool-based approach since FSM will be exposed via tools.
- tool_basic panic message is misleading (panics on JSON parse before reaching error_code assertion). Not a correctness issue but diagnostic clarity issue.
