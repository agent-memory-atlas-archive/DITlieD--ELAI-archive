//! H-8 — persist OracleAnswerCache contract (Tier-1 source-wiring gate).
//!
//! Grounded scope (workflow/reasoning/H-8-decisions.md): the OracleAnswerCache was
//! in-process only (Mutex<CacheInner>, lost on exit). H-8 disk-backs it (load-on-
//! construct + persist-on-insert) so idempotent grounding answers survive restart,
//! mirroring PromptCache (GD-1). Confinement: only the /research|/ask arm constructs the
//! disk-backed cache + is its single insert caller; code-gen never touches it. No schema
//! bump (OracleCacheHit/Miss reused). Tier-2 (test exec + the live restart-survival run)
//! is DEFERRED.
//!
//! Placed in orchestrator/tests (NOT the G-blocked elai_cc_contracts crate) so it
//! compile-verifies via `cargo check -p elai-orchestrator`.

use std::fs;
use std::path::PathBuf;

fn read_abs(parts: &[&str]) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for p in parts {
        path.push(p);
    }
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// D1: OracleAnswerCache gains a disk-backed constructor + persist path, mirroring
/// PromptCache's $HOME/.elai/cache layout (an oracle-cache leaf).
#[test]
fn oracle_cache_is_disk_backed() {
    let src = read_abs(&["src", "oracle_cache.rs"]);
    assert!(
        src.contains("persisted") && src.contains("persist_to_disk"),
        "OracleAnswerCache must gain a disk-backed constructor (persisted) + a persist_to_disk path"
    );
    assert!(
        src.contains("oracle-cache"),
        "the on-disk cache must live under the $HOME/.elai/cache/oracle-cache leaf (mirrors PromptCache)"
    );
}

/// D3: the /research|/ask consumer lazy-inits the DISK-BACKED cache (not the in-process
/// new()), so durable caching is confined to the idempotent grounding path.
#[test]
fn research_arm_uses_persisted_cache() {
    let src = read_abs(&["..", "cli", "src", "repl_command.rs"]);
    assert!(
        src.contains("persisted_default") || src.contains("persisted("),
        "the /research|/ask arm must construct the disk-backed OracleAnswerCache (persisted_default)"
    );
}
