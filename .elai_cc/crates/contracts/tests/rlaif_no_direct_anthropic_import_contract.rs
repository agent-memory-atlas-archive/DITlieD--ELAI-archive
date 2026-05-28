// E-55 contract: the RLAIF judge + sibling RL modules under
// .elai_cc/crates/orchestrator/src/rl/ MUST NOT import the Anthropic SDK
// directly. All cloud Sonnet 4.6 invocations route through
// `crate::per_role_runtime::PerRoleRuntime` (B-37 SHIPPED commit fd5c2e5).
//
// Why this exists:
//   - The PerRoleRuntime indirection is the single dispatch point for
//     cloud spend tracking, per-role rate limits, retry policy, and
//     adversary heterogeneous-family checks (adversary.rs:644).
//   - A direct `anthropic::` / `anthropic-sdk` / `claude_api` import
//     bypasses ALL of those + breaks the runtime adapter abstraction
//     B-37 ships.
//   - The substantive check is already enforced at audit time (E-55
//     audit DETERMINISTIC_CHECK step 9). This contract test makes the
//     check permanent so future commits cannot quietly re-introduce
//     a direct import.
//
// Scope: every .rs file under crates/orchestrator/src/rl/ (including
// `mod.rs` and the 6 module bodies). The contract test source itself is
// in crates/contracts/tests/ and therefore outside the scan path -- no
// allowlist exception is needed.
//
// Grounding: workflow/reasoning/E-55-decisions.md DECISION-3 (ModelFamily
// for local Qwen) + plan WIRING_CONTRACT.no_anthropic_direct_import
// stub_ban. The E-55 audit cited absent contract test as BLOCK B-4.

use std::path::{Path, PathBuf};

/// Forbidden import substrings. Each must NOT appear anywhere in any
/// .rs file under crates/orchestrator/src/rl/.
///
/// `anthropic::` covers `use anthropic::Foo;` and `anthropic::Foo::bar()`.
/// `anthropic_sdk::` covers any future-tier rebrand using snake_case.
/// `anthropic-sdk` (kebab-case) covers Cargo.toml-style references
///   that occasionally leak into source comments.
/// `claude_api::` covers a hypothetical wrapper crate name.
const FORBIDDEN_IMPORT_TOKENS: &[&str] = &[
    "use anthropic::",
    "use anthropic_sdk::",
    "anthropic-sdk",
    "use claude_api::",
];

/// The scan root relative to CARGO_MANIFEST_DIR. The contracts crate's
/// Cargo.toml lives at .elai_cc/crates/contracts/Cargo.toml so two
/// `parent()` calls climb to `.elai_cc/`.
fn rl_module_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("rl")
}

/// Recursively walk a directory and collect every .rs file path. Stdlib
/// only -- mirrors the no_dpo_no_grpo_until_unlock.rs scan idiom.
fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

#[test]
fn rl_module_dir_exists() {
    let dir = rl_module_dir();
    assert!(
        dir.exists(),
        "rl module dir missing -- E-55 substrate not present: {}",
        dir.display()
    );
}

#[test]
fn no_direct_anthropic_import_in_rl_modules() {
    let dir = rl_module_dir();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs_files(&dir, &mut files).expect("walk rl/ tree");
    assert!(
        !files.is_empty(),
        "rl/ tree is empty -- expected at least mod.rs + 6 module files"
    );
    let mut violations: Vec<(PathBuf, &'static str)> = Vec::new();
    for path in &files {
        let body = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => panic!("read {}: {e}", path.display()),
        };
        for token in FORBIDDEN_IMPORT_TOKENS {
            // Skip occurrences inside line comments that document the
            // ban itself (e.g. `// no anthropic-sdk` doc strings). Only
            // active code lines without a leading `//` count.
            for line in body.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") || trimmed.starts_with("//!") {
                    continue;
                }
                if trimmed.contains(token) {
                    violations.push((path.clone(), token));
                    break;
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "forbidden anthropic-direct imports found in rl/ modules: {violations:#?}. \
         All cloud Sonnet calls must route through PerRoleRuntime (B-37)."
    );
}

#[test]
fn forbidden_token_list_documented() {
    // Regression guard: if FORBIDDEN_IMPORT_TOKENS changes, the
    // decisions doc must be updated. We assert the canonical set
    // (4 tokens) so a silent expansion + missing doc update fails
    // visibly in CI.
    assert_eq!(
        FORBIDDEN_IMPORT_TOKENS.len(),
        4,
        "forbidden token list drifted -- update workflow/reasoning/E-55-decisions.md"
    );
}
