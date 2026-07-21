// C-39: thin library surface exposing the minimal subset of `elai-cli`
// modules that integration tests under `crates/cli/tests/` need.
//
// The CLI crate is primarily a binary (`src/main.rs`); this `lib.rs` adds
// a parallel library target so `use elai_cli::mcp_client::CliMcpClient`
// resolves from sibling integration-test crates (binary-only crates cannot
// be imported).
//
// SCOPE: only re-expose modules that are SELF-CONTAINED (no cross-module
// dependencies on private CLI types). `mcp_client.rs` is self-contained
// (it depends only on `elai-plugins` + `tokio` + `serde_json`). Other
// CLI modules (args, commands, repl) reference `crate::*` and private
// types like `AllowedToolSet` from `main.rs`, so they cannot be exposed
// without a much larger structural change. See workflow/reasoning/
// C-39-decisions.md DECISION D3.
//
// I-38 STEP 4: `bench_qdelta.rs` is also self-contained — it depends only
// on `elai-core`, `elai-orchestrator`, `serde`, `thiserror`, `toml`,
// `blake3`, `serde_json`, and `tokio` (all external crates, no `crate::*`
// refs in non-test code). Exposed so contract tests under
// `crates/contracts/tests/` can drive `floor_check()` and `run_inner()`
// with on-disk fixture TOML files.

pub mod mcp_client;
pub mod bench_qdelta;
pub mod bench_positional_probe;
pub mod bench_steering_vs_prompt;
