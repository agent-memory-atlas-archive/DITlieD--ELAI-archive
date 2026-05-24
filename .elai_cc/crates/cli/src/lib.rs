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

pub mod mcp_client;
