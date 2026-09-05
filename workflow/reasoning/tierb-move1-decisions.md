Tier-B Move 1 decisions (ssb.rs split, behavior-preserving)
session s-b9bea968-10d0-4a32-8806-b65115cd2a4b, 2026-06-13, tree base df18539

What I will implement
Split crates/orchestrator/src/ssb.rs (39,832 lines) into a module directory by pure code movement, zero logic change. ssb.rs STAYS as the module-root file; submodules land in crates/orchestrator/src/ssb/. Everything re-exported at the ssb root via `pub use child::*` so every external path elai_orchestrator::ssb::X is unchanged.

Module layout (verified blank-line cut boundaries, original 1-indexed line ranges)
root ssb.rs keeps: 1-39 (module doc + use block), 3295-3305 (SCHEMA_VERSION, SCHEMA_VERSION_PUBLIC, PAYLOAD_MAX_BYTES), 30893-30967 (CANVAS/ELAI consts + env_gate_truthy/blake3_label_prefix/system_time_ms_i64), plus new mod decls + pub use re-exports + `#[cfg(test)] mod tests;`.
ssb/schema.rs: 40-3294 (schema bump-history comment) + 3306-5301 (CREATE_SSB_TABLE incl CHECK constraint + CREATE_SSB_FILES_TABLE).
ssb/action_type.rs: 5302-11309 (SsbActionType enum + impl as_sql_str + ssb_action_type_from_str(_public) + decode_ssb_payload + validate_payload_matches_action_type + compute_isolation_metric).
ssb/payloads.rs: 11310-15637 (payload structs batch 1 + SsbPayload enum + impl SsbPayload) + 16858-30892 (payload structs batch 2 + *Kind enums + ELAI_*_ENABLE_ENV consts + CREATE_STRATEGY_TRIALS/HOLDOUT/DRIFT consts).
ssb/store.rs: 15638-16857 (impl Ssb now_ms + SsbTraceable + SsbEntry/SsbFileEntry/SsbWriteRequest/SsbConfig/SsbError + pool/SqliteManager/WriteMessage + Ssb struct + impl Ssb + apply_pragmas + run_migration + insert_session_archived_row + execute_write + row_to_entry + impl SsbReader + path helpers + impl ReflectEventSink).
ssb/tests.rs: 30974-39831 (body of `mod tests`).

Key decision
Keep ssb.rs as the root file (NOT ssb/mod.rs). Reason: e12 (ssb_schema_version_at_or_above_128, contracts/tests/e12:251) and h51 (d3_schema_version_at_least_227, contracts/tests/h51:204) read the literal file path crates/orchestrator/src/ssb.rs and grep for `const SCHEMA_VERSION: i64 = NNN;`. Keeping the root file + that literal const line means Move 1 touches NOTHING outside the ssb module. A child `mod ssb` declaring `mod schema;` resolves submodules to src/ssb/*.rs even though the module file is src/ssb.rs (Rust 2018+).

Visibility
Root-private items (SCHEMA_VERSION) stay visible to child submodules via `super` (descendant modules see parent privates). Cross-SIBLING private references must become pub(crate): expected set = CREATE_SSB_TABLE, CREATE_SSB_FILES_TABLE, ssb_action_type_from_str, validate_payload_matches_action_type, CREATE_STRATEGY_TRIALS_TABLE/CREATE_HOLDOUT_BUDGET_TABLE/CREATE_DRIFT_INTERLOCK_TABLE. Converged via `cargo check -p elai-orchestrator` (compiler enumerates each), NOT a blanket regex (the SQL string literals contain column-0 lines that a `^const|^fn` rewrite could corrupt).
Each submodule preamble: `#![allow(unused_imports)]` + the original external use block (std/rusqlite/serde/tokio/elai_core/elai_reasoning/crate::architecture/crate::mast) + `use super::*;` + `use crate::ssb::{schema::*, action_type::*, payloads::*, store::*};`. Covers both pub (via super re-export) and pub(crate) (via direct sibling glob) cross-references; glob-cycle with root pub use is legal in Rust.

Expected test outcome (auditor gate, off-agent)
cargo check --workspace 0 errors; action_type_lock golden+count+exhaustive_match_guard pass UNCHANGED (no variant moved/renamed/reordered); ELAI_SEED=42 SSB output byte-identical before/after; cross_lang_diff clean; maintainability_check no new as-any/unsafe and the new files far under the 40k god-file (action_type.rs and payloads.rs remain large because SsbActionType/SsbPayload are single irreducible enums, accepted for a pure split).

Unknowns
Exact pub(crate) set beyond the expected list (converge via cargo check). Whether any payload struct references a root-private helper across the new boundary (cargo check catches). Move 2 footprint is larger than the brief's named files (e12/h51/h12 also literal-grep the counters) and conflicts with the user's "ssb.rs only" fence; flagged for a user decision at the Move-1/Move-2 boundary.
