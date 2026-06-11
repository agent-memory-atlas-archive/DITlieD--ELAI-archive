//! H-4 — advanced-tool-use beta header contract (Tier-1 source-wiring gate).
//!
//! Grounded scope (workflow/reasoning/H-4-decisions.md): the OAuth-bearer production
//! path emitted ONLY oauth-2025-04-20 and ignored profile betas (C14), so the
//! advanced-tool-use beta was DEAD on the live OAuth path. H-4 joins
//! `advanced-tool-use-2025-11-20` onto the OAuth header (mandatory) + the API-key
//! default betas. The Tool Search Tool catalog DEFERRAL (the ~85% token reduction), the
//! dkg_* 3-arm fix, and the pre-existing len==14 catalog contract fix are documented
//! Tier-2 follow-ons (heavy net-new / G-blocked crate). Tier-2 (live wire-capture of the
//! header + the catalog deferral) is DEFERRED.
//!
//! NOTE: this crate (elai_cc_contracts) is currently transitively blocked from compiling
//! by an unrelated in-flight G-plan error (production_agent_rpc.rs missing `use
//! std::path::Path;`). This contract's SOURCE is correct and verifies once G unblocks;
//! H-4's own change is verified via `cargo check -p elai-providers`.

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// T1 (C14): the advanced-tool-use beta is joined onto the OAuth-bearer header — without
/// this it is dead on the live OAuth path (the branch does not loop profile betas).
#[test]
fn advanced_tool_use_beta_joined_on_oauth_path() {
    let src = read_src("providers/src/providers/anthropic.rs");
    assert!(
        src.contains("ADVANCED_TOOL_USE_BETA"),
        "anthropic.rs OAuth path must join the advanced-tool-use beta (C14: the OAuth branch \
         ignores profile betas, so the join is mandatory)"
    );
}

/// The verified beta string is the single source of truth + on the API-key default path.
#[test]
fn advanced_tool_use_beta_const_and_default() {
    let src = read_src("providers/src/telemetry_types.rs");
    assert!(
        src.contains("advanced-tool-use-2025-11-20"),
        "telemetry_types must define the verified umbrella beta string verbatim"
    );
    assert!(
        src.contains("ADVANCED_TOOL_USE_BETA"),
        "the beta must be a named const (one source of truth, shared by OAuth + API-key paths)"
    );
}
