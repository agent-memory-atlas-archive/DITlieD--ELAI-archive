P3J.PIPELINE decisions

What: Wire all phase 0-3J infrastructure into main CLI pipeline so elai --local routes through ConversationRuntime with FSM-aware tool filtering, ContextBudget, and sidecar /v1/messages.

Key decision: enum dispatch (CliApiClient) over trait objects
ConversationRuntime<C: ApiClient> is generic. Runtime client selection requires compile-time type resolution. Options considered:
(a) Box<dyn ApiClient> — requires making ApiClient object-safe, changes orchestrator crate public API.
(b) Separate build paths returning different concrete types — duplicates entire interactive loop.
(c) Enum dispatch CliApiClient { Api(AnthropicRuntimeClient), Local(LocalRuntimeClient) } — single type, impl ApiClient delegates to variant. Zero changes to orchestrator.
Chose (c): lowest blast radius (4 type annotation changes in cli/main.rs), zero orchestrator changes, no trait object overhead.

Key decision: config via existing JSON ConfigLoader
Plan spec said elai_cc.toml. Actual ConfigLoader uses JSON (settings.json). Did not add TOML parser. BackendConfig deserialized via serde_json, can be embedded in settings.json "backend" key. For now, LocalRuntimeClient reads config from build_local_runtime parameters (CLI flag --local provides port).

Key decision: reuse sidecar spawn logic via start_sidecar()
Extracted from run_local_repl into start_sidecar(port) -> Result<Option<Child>>. Returns None if sidecar already running. Health-poll uses exponential backoff (2s -> 3s -> 4.5s -> 5s cap). Timeout 120s.

Key decision: stream() body duplication vs shared helper
LocalRuntimeClient::stream() is ~140 lines, mostly identical to AnthropicRuntimeClient::stream(). Differences: (1) no AuthSource, (2) tool pre-filtering with max cap, (3) ContextBudget::compute call, (4) no prompt cache record. Chose to duplicate with differences rather than extract a shared function that would need 8+ parameters. Acceptable tech debt — if a third client appears, extract then.

Expected test outcome: 7/7 dispatcher tests GREEN, cargo check --workspace clean.
Unknowns: runtime behavior untested until sidecar is live (requires WSL2 + GPU).
