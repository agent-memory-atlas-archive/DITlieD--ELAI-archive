P0.1 Decision Doc: cfg-gate Unix-only imports for Windows compilation

What: Added #[cfg(unix)] attributes to 3 use statements and wrapped 7 set_mode() call sites in #[cfg(unix)] blocks across 3 files: mcp_stdio.rs, mcp_tool_bridge.rs, mock_parity_harness.rs.

Key decision: Attribute form (#[cfg(unix)] on preceding line) for use statements at mod/file top level. Block form (#[cfg(unix)] { ... }) for set_mode() calls inside function bodies, with the permissions variable declaration inside the block to avoid unused-variable warnings on Windows.

Expected test outcome: cargo test --no-run drops from 10 compile errors to 0. Confirmed.

Unknowns: Tests using /bin/sh scripts will compile on Windows but fail at runtime (pre-existing, out of P0.1 scope). The plugins/src/lib.rs:2357 unix import uses inline-in-block pattern and was not in scope.

Skipped WARN: Pre-existing compiler warnings in out-of-scope files (hooks.rs unused import, hooks.rs unneeded mut, file_ops.rs unused link_path) not addressed by P0.1. The file_ops.rs warning uses the same cfg pattern but with the variable declared outside the block — should be fixed in a future cleanup pass.
