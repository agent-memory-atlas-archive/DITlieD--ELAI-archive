//! Seed-rule loader shared by CLI and dashboard wiring (Session 4E.14).
//!
//! The CLI's `load_seed_rules` (cli/src/main.rs:6286) compiles rule TOML
//! files from `data/rules/` into a `Vec<(Rule, CompiledPredicate)>`. The
//! dashboard needs the same list so its `TaskRunnerConfig.rule_evaluator`
//! carries the seeded rule set. Rather than re-implement the loader in
//! every binary, we expose one here that takes the directory path and a
//! compiled engine.
//!
//! This loader is intentionally minimal: it silently skips unreadable,
//! unparsable, and uncompilable rules (logged via `tracing`), mirroring the
//! CLI loader's eprintln-based fallback. A rule that fails to compile is
//! not load-bearing for correctness — the rest of the seed set continues to
//! be enforced.

use std::fs;
use std::path::Path;

use serde::Deserialize;
use tracing::{info, warn};

use elai_rules::rule_engine::{CompiledPredicate, DefaultRuleEngine, Rule, RuleEngine};

/// Structural mirror of the on-disk seed TOML:
/// `[[rules]]` blocks populate a `rules: Vec<Rule>` field.
#[derive(Deserialize)]
struct SeedFile {
    rules: Vec<Rule>,
}

/// Load every `.toml` file under `rules_dir`, parse `[[rules]]` entries into
/// [`Rule`] values, and compile their `predicate_json` fields through the
/// supplied [`DefaultRuleEngine`]. Returns the compiled pairs suitable for
/// [`elai_rules::rule_engine::RuleEngine::evaluate`].
///
/// Failures at any stage are logged and skipped; the return value reflects
/// only successfully compiled rules. Called from both `cli::load_seed_rules`
/// and the dashboard setup path so the two binaries share one source of
/// truth about which rules are live.
pub fn load_seed_rules_from_dir(
    rules_dir: &Path,
    engine: &DefaultRuleEngine,
) -> Vec<(Rule, CompiledPredicate)> {
    let entries = match fs::read_dir(rules_dir) {
        Ok(e) => e,
        Err(e) => {
            warn!(
                dir = %rules_dir.display(),
                error = %e,
                "rule_loader: seed directory unreadable; returning empty rule set",
            );
            return Vec::new();
        }
    };

    let mut compiled: Vec<(Rule, CompiledPredicate)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "toml") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "rule_loader: read failed");
                continue;
            }
        };
        let seed: SeedFile = match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                warn!(path = %path.display(), error = %e, "rule_loader: parse failed");
                continue;
            }
        };
        for rule in seed.rules {
            match engine.compile_predicate(&rule.predicate_json) {
                Ok(cp) => compiled.push((rule, cp)),
                Err(e) => {
                    warn!(
                        rule_id = %rule.rule_id,
                        error = %e,
                        "rule_loader: compile failed; rule skipped",
                    );
                }
            }
        }
    }
    info!(
        rules_loaded = compiled.len(),
        dir = %rules_dir.display(),
        "rule_loader: seed rules compiled",
    );
    compiled
}
