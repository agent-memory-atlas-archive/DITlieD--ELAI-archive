//! E-38 Tier-A — anti-pattern regression guards.
//!
//! Four grep-style contract tests that lock current-state invariants so a
//! future "refactor" cannot silently re-introduce known footguns. Each test
//! walks the workspace source tree and asserts that a specific anti-pattern
//! is absent.
//!
//! The four anti-patterns are derived from the May 2026 AI-roadmap-signals
//! survey + the EU AI Act / Digital Omnibus posture review:
//!   1. RuntimeAdapter trait MUST stay arch-agnostic (no dense/moe routing).
//!   2. No hard-coded "flash_attention_2" assumption in the sidecar boot path.
//!   3. No bespoke "agent protocol" implementations parallel to MCP.
//!   4. No browser-screenshot polling loop for operator-style automation.
//!
//! These are regression guards, NOT new functional requirements. Today's
//! codebase already complies (verified by integration-explorer-elai in
//! s-orch-e38). The contract makes the compliance loud.
//!
//! Tier-B will add: the EU AI Act compliance gate at orchestrator/fsm.rs:622
//! + 5 new SsbActionType variants (EuAiActComplianceBlocked / _Verified /
//! McpTransportVersionLogged / MrcrUnverified / PostureDocStale) + a
//! ComplianceDrawer.svelte. See workflow/reasoning/E-38-decisions.md.

use std::fs;
use std::path::{Path, PathBuf};

/// Workspace root resolved from CARGO_MANIFEST_DIR. The contracts crate sits
/// at `.elai_cc/crates/contracts/`; the workspace root is two parents up.
fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

/// Recursively walk a directory, collecting all `.rs` files. Skips target/,
/// .git/, node_modules/, generated SCIP indices, and anything under a
/// `target` segment so the walker doesn't enter build artefacts.
fn collect_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    walk_dir(root, &mut out);
    out
}

fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            // Skip noisy / generated / build dirs.
            if matches!(
                name,
                "target"
                    | ".git"
                    | "node_modules"
                    | ".elai"
                    | "indexes"
                    | "scip_indices"
                    | "models"
            ) {
                continue;
            }
            walk_dir(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // Skip the contract test file itself so its own mention of the
            // anti-pattern string does not trip the contract.
            if matches!(
                name,
                "e38_anti_pattern_contracts.rs"
                    | "e48_research_contracts.rs"
                    | "e25_compaction_architecture_contracts.rs"
                    | "e11_idea_to_saas_contracts.rs"
            ) {
                continue;
            }
            out.push(path);
        }
    }
}

fn file_contains(path: &Path, needle: &str) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    contents.contains(needle)
}

fn collect_hits(root: &Path, needles: &[&str]) -> Vec<(PathBuf, String)> {
    let mut hits: Vec<(PathBuf, String)> = Vec::new();
    for file in collect_rust_sources(root) {
        for needle in needles {
            if file_contains(&file, needle) {
                hits.push((file.clone(), (*needle).to_string()));
            }
        }
    }
    hits
}

#[test]
fn runtime_adapter_trait_stays_arch_agnostic() {
    // RuntimeAdapter (providers/src/runtime/types.rs:504-531) must remain
    // arch-blind — no method that branches on dense vs MoE vs other shape.
    // If a future "specialize for MoE" pass adds such a method, this fails.
    let root = workspace_root();
    let banned = [
        "fn requires_dense",
        "fn is_moe",
        "fn requires_moe",
        "fn dense_only",
        "fn moe_only",
        "DenseRuntime",
        "MoeRuntime",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "RuntimeAdapter must stay arch-agnostic; found banned identifiers:\n  {}",
        hits.iter()
            .map(|(p, n)| format!("{} :: {}", p.display(), n))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn no_hardcoded_flash_attention_2_assumption() {
    // The sidecar boot path uses SDPA by default with FA2 as opt-in
    // (ELAI_FORCE_SDPA=1 + ELAI_TRY_FLASH_ATTN_2 env). A hard-coded
    // require/assert/assume on FA2 means non-FA2 hosts (older Ampere /
    // non-Ampere / CPU-only) silently break. Watch for it.
    let root = workspace_root();
    let banned = [
        "require_flash_attention_2",
        "assert_flash_attention_2",
        "must_have_flash_attn_2",
        "REQUIRES_FA2",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "No hard-coded FA2 requirement allowed; found:\n  {}",
        hits.iter()
            .map(|(p, n)| format!("{} :: {}", p.display(), n))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn no_bespoke_agent_protocol_parallel_to_mcp() {
    // ELAI standardises on the Model Context Protocol (MCP) for tool /
    // agent communication. A bespoke alternative would fragment the
    // ecosystem + make external tool integration brittle. Anyone proposing
    // one needs to delete this contract first — i.e. argue for it openly.
    let root = workspace_root();
    let banned = [
        "trait ElaiAgentProtocol",
        "struct ElaiAgentProtocol",
        "enum ElaiAgentProtocol",
        "trait BespokeAgentProtocol",
        "struct BespokeAgentProtocol",
        "trait CustomAgentProto",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "No bespoke agent protocol allowed (use MCP); found:\n  {}",
        hits.iter()
            .map(|(p, n)| format!("{} :: {}", p.display(), n))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn no_operator_screenshot_polling_loop() {
    // OpenAI Operator and Claude Computer Use take browser screenshots
    // every N seconds to drive a vision-LLM. That loop is expensive
    // ($/screenshot), latency-bound, and a privacy footgun. ELAI's
    // automation path uses LSP + structured tool output, NOT screenshot
    // polling. Watch for anyone wiring it up.
    let root = workspace_root();
    let banned = [
        "screenshot_loop",
        "browser_screenshot_poll",
        "operator_capture_loop",
        "screenshot_every_ms",
    ];
    let hits = collect_hits(&root, &banned);
    assert!(
        hits.is_empty(),
        "No operator-style screenshot loop allowed; found:\n  {}",
        hits.iter()
            .map(|(p, n)| format!("{} :: {}", p.display(), n))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
fn workspace_walker_finds_at_least_one_rs_file() {
    // Sanity: the walker isn't silently producing empty results. If this
    // breaks, the other four assertions might be falsely passing.
    let root = workspace_root();
    let files = collect_rust_sources(&root);
    assert!(
        !files.is_empty(),
        "workspace walker found zero .rs files under {} — contract walker is broken, the four anti-pattern asserts cannot be trusted",
        root.display()
    );
}
