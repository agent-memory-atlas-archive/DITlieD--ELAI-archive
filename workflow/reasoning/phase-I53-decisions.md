I-53 execution decisions
date: 2026-06-13
session: s-107b022a / task A129-I53-EXECUTE

DECISION-1: catalog-drift (both catalogs)
The explorer confirmed two divergent registries:
  (a) mvp_tool_specs() in crates/tools/src/lib.rs — drives CLI agent GlobalToolRegistry::definitions()
  (b) build_tool_catalog() in crates/orchestrator/src/production_agent_rpc.rs — drives the production LLM agent

The plan says "reachable_from: production agent tool dispatch (live agents calling list/run on a real repo)".
Action: add all three tools (recipe_list, recipe_run, archive_read, json_tree_query) to BOTH registries.

For build_tool_catalog the tools are simple JSON-described ToolDefinition entries (no postcard wire struct needed;
the production catalog accepts JSON args directly via translate_tool_request dispatch arms added for these tools).
The translate_tool_request dispatch arms call a helper that parses JSON -> then calls the same handlers as the CLI path.
The contract test both-catalogs arm source-scans both sites.

drift risk note: build_tool_catalog is hand-authored and does NOT auto-derive from mvp_tool_specs(). A future
patch touching one catalog without the other risks re-creating the omission. The I-53 contract test guards this by
asserting both registration sites at the source level (grep-based scan).

DECISION-2: archive scope (plain tar, no gz)
flate2 is in Cargo.lock as transitive dep of png and syntect only — it is NOT a direct dep of elai-tools.
zip and tar crates: absent from the workspace entirely.
Action: implement plain .tar listing via hand-rolled 512-byte block header walk (std::io only, no new dep).
.tar.gz (gz-compressed tar): DEFERRED (2026-06-13). Requires adding flate2 as a direct dep of elai-tools per
the anti-slopsquat protocol. Verified: crates.io/crates/flate2 version 1.1.9 exists, MIT+Apache-2.0, ~37M dl/mo.
Acceptable dep when needed; defer to a follow-on patch after user approval.
.zip: DEFERRED. zip crate would need to be added separately; also deferred.
honest_error: archive_reader returns Err("unsupported format: only .tar is supported; .tar.gz and .zip require a dep addition (defer to follow-on patch; 2026-06-13)") for .gz/.zip extensions.

DECISION-3: validate_command routing boundary
recipe_run builds the shell command string (runner_prefix + task + args) and calls bash_validation::validate_command.
- ValidationResult::Allow -> std::process::Command::new("cmd").args(...).output()
- ValidationResult::Warn { message } -> surface warning string to caller AND proceed (same as bash tool Warn behavior)
- ValidationResult::Block { reason } -> return Err without spawning (R1, observable_effect gate)
recipe_run does NOT go through run_bash (the bash tool dispatcher). This means I-46's interceptor does not fire
yet. Per plan: "recipe run is a redirect TARGET for I-46's bash interceptor once shipped — do NOT wait."
The boundary: recipe_run builds a command, validates it, spawns it directly via std::process::Command (sync).

DECISION-4: runner binary detection on this machine
cargo: FOUND (C:\Users\user\.cargo\bin\cargo.exe)
just: NOT FOUND on this machine
make: NOT FOUND on this machine
detect_runners() must degrade honestly: if binary not in PATH, runner is excluded (not an error).
Contract tests for just/make use tempdir fixture files (Justfile, Makefile) and check detect_runners degrades
rather than panics. The binary-absent path for just/make is the correct path on this machine.

DECISION-5: json_tree shape
gajae json-tree.ts is a TUI renderer for in-memory values, not a file-query tool.
Rust port: read file -> parse serde_json -> navigate dot-path -> bounded render with depth+line caps.
Constants ported from gajae (collapsed defaults): MAX_DEPTH_DEFAULT=2, MAX_LINES_DEFAULT=6, SCALAR_LEN_DEFAULT=60.
DECISION-5a: MAX_LINES_EXPANDED=200 is available but json_tree_query caller chooses via max_lines param.

DECISION-6: SSB observable_effect interpretation
ToolCallLogged rows are emitted by DefaultMcpBridge only (MCP path, not GlobalToolRegistry builtins).
The "validation verdict row" = the return value / tracing log from validate_command inside recipe_run,
assertable in the contract test by: (a) calling recipe_run on a fixture command that should Allow -> assert Ok(_);
(b) calling recipe_run on a blocked command (rm -rf or similar) -> assert Err(_) without actually spawning.
No SSB table query needed. GD-1 confirmed: SCHEMA_VERSION=241, TOTAL_VARIANTS=550, both UNCHANGED.

DECISION-7: production_agent_rpc translate_tool_request integration
translate_tool_request uses postcard wire structs for most tools, but for the new recipe/archive/json tools
there are no ipc-types structs to define (out of scope, would require editing elai-ipc-types). Instead,
the dispatch arms parse the JSON arguments directly using serde_json (same approach as the fsm_get_state arm
which uses raw input.get() without postcard). This keeps the diff contained to production_agent_rpc.rs only
(no ipc-types changes). The production tool dispatch routes JSON -> serde_json::from_value -> handler functions.

DECISION-8: no new SSB variants
SCHEMA_VERSION=241, TOTAL_VARIANTS=550 confirmed at ssb.rs:3152 and action_type_lock.rs:705.
GD-1: zero new action_type variants in I-53.
