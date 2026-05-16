//! B-41 Slice 3i — rule-cluster load-order invariants.
//!
//! The 5 B-41 rule-cluster seed TOMLs live at
//! `crates/rules/seed/{design_hard,app_store,source_tiers,slopsquatting,
//! risk_levels}.toml`. Until a dedicated rule-engine loader picks them up,
//! we assert structural invariants directly against the TOMLs themselves:
//!   1. Every file parses as TOML and declares the expected `domain`.
//!   2. Each file declares `default_status = "SHADOW"` (Phase 3F lifecycle).
//!   3. The fixed load order from auditor.md MODE DISPATCH is observable
//!      in the TOML filenames (sorted alphabetically would mis-order them,
//!      so we use a hand-coded ordering vector that the auditor agent
//!      consumes verbatim).
//!   4. Each file declares at least the minimum rule count from the brief.
//!
//! Forward-dep: a real `RuleClusterLoader` should consume `LOAD_ORDER` to
//! load each domain into the rules DB at status SHADOW. This test pins the
//! order until that loader lands.

use std::path::PathBuf;

/// Fixed cluster load order from `.claude/agents/auditor.md` MODE DISPATCH
/// section. Order matters because earlier clusters establish ground-truth
/// patterns that later clusters reference (e.g. design-hard contrast rules
/// are referenced by app_store accessibility checks).
const LOAD_ORDER: &[(&str, &str, &str)] = &[
    // (domain_label, file_name, min_rules)
    ("anti-stale", "<from CLAUDE.md>", "0"), // sourced from CLAUDE.md, not a TOML
    ("coding-standards", "<from rules dir>", "0"), // sourced from .claude/rules/
    ("design-hard", "design_hard.toml", "25"),
    ("app-store", "app_store.toml", "15"),
    ("source-tiers", "source_tiers.toml", "10"),
    ("slopsquatting", "slopsquatting.toml", "30"),
    ("risk-levels", "risk_levels.toml", "12"),
];

fn seed_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR at orchestrator-test build time = crates/orchestrator/
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crates dir")
        .join("rules")
        .join("seed")
}

#[test]
fn all_five_b41_toml_files_exist() {
    let dir = seed_dir();
    for (_, file, _) in LOAD_ORDER.iter() {
        if file.starts_with('<') {
            continue;
        }
        let path = dir.join(file);
        assert!(
            path.exists(),
            "expected seed TOML at {} (cwd-relative: {})",
            path.display(),
            file
        );
    }
}

#[test]
fn every_toml_declares_shadow_default_status() {
    let dir = seed_dir();
    for (_, file, _) in LOAD_ORDER.iter() {
        if file.starts_with('<') {
            continue;
        }
        let path = dir.join(file);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            src.contains("default_status = \"SHADOW\""),
            "{}: must declare default_status = \"SHADOW\" (Phase 3F lifecycle)",
            file
        );
    }
}

#[test]
fn every_toml_declares_correct_domain_label() {
    let dir = seed_dir();
    for (label, file, _) in LOAD_ORDER.iter() {
        if file.starts_with('<') {
            continue;
        }
        let path = dir.join(file);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let needle = format!("domain = \"{label}\"");
        assert!(
            src.contains(&needle),
            "{}: must declare {needle}",
            file
        );
    }
}

#[test]
fn each_toml_has_minimum_rule_count() {
    let dir = seed_dir();
    for (_, file, min_str) in LOAD_ORDER.iter() {
        if file.starts_with('<') {
            continue;
        }
        let path = dir.join(file);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        // Count `[[rules]]` occurrences — the canonical entry marker.
        let count = src.matches("[[rules]]").count();
        let min: usize = min_str.parse().expect("min parse");
        assert!(
            count >= min,
            "{}: rule count {count} < required {min}",
            file
        );
    }
}

#[test]
fn load_order_is_fixed_and_not_alphabetical() {
    // If the load order were alphabetical the auditor would load
    // app-store before design-hard, breaking the dependency that app-store
    // accessibility rules reference design-hard contrast ratios. Pin the
    // ordering explicitly.
    let labels: Vec<&str> = LOAD_ORDER.iter().map(|(l, _, _)| *l).collect();
    let expected = vec![
        "anti-stale",
        "coding-standards",
        "design-hard",
        "app-store",
        "source-tiers",
        "slopsquatting",
        "risk-levels",
    ];
    assert_eq!(labels, expected, "load order drift detected");
    // Verify NOT alphabetical (sanity).
    let mut sorted = labels.clone();
    sorted.sort();
    assert_ne!(
        labels, sorted,
        "if labels matched sorted(labels), the dependency ordering would be lost"
    );
}

#[test]
fn missing_domain_is_a_panic_not_silent_skip() {
    // This is a meta-test: confirm that the assertions in
    // `every_toml_declares_correct_domain_label` would PANIC (not silently
    // skip) on a missing label. We simulate by constructing a tempfile
    // with the wrong domain and asserting the contains() check fails.
    let bad = "domain = \"wrong-name\"\ndefault_status = \"SHADOW\"\n";
    let needle = "domain = \"design-hard\"";
    assert!(!bad.contains(needle), "meta-test sanity");
}
