Phase 2 — dashboard contract test split decisions

What: Create new file crates/contracts/tests/ssb_4k3_4k4_4k5_contract.rs as a copy of dashboard_4k3_4k4_4k5_contract.rs without `#![cfg(any())]`, with the schema-version assertion updated from v==9 to v==25 (current SCHEMA_VERSION post Track B in s-1430428).

Key decisions:
- Source file (dashboard_4k3_4k4_4k5_contract.rs) stays `#![cfg(any())]`-gated and byte-identical (dashboard crate still backed up; ungating breaks workspace).
- Schema-version constant extracted as `const EXPECTED_SCHEMA: i64 = 25;` — no bare magic numbers.
- Helpers (fresh_db / open_writer / open_viewer) duplicated verbatim in the new file. No tests/common/mod.rs (out of scope for ~16 tests).
- Test ordering preserved (4K.3 -> 4K.4 -> 4K.5 -> shape-lock).
- No Cargo.toml edits (orchestrator dev-dep already present, dashboard dev-dep already commented out per s-1430428 Track B).

Expected test outcome:
- `cargo test -p elai-contracts ssb_4k3_4k4_4k5` — 16 tests pass, 0 fail.
- `cargo check --workspace` — 0.
- Existing dashboard_*_contract.rs files byte-unchanged.

Unknowns:
- Whether all 16 tests still pass against the current ssb.rs API (post-bump from v9 -> v25). Context-compiler verified imports map cleanly; auditor will confirm at run time.

Audit cycle budget: 1.
