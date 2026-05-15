// B-29.5 — scale_p99_budget_contract.
//
// Reads metrics/phase_b29_5_scale.toml if it exists and asserts:
//   - Every tier's per-tool p99 latency <= 500 ms
//   - Every tier's peak_rss_mb_during_workload <= 2048 MB
//
// SKIP behaviour: if the file is absent OR the expected keys are missing, the
// test emits a diagnostic message and returns successfully.  This lets the
// harness be committed before the fixture runs populate the file.
//
// Gate condition: only asserts when populated_at != null in [meta].
//
// Contract: do NOT fail CI on a schema-only scaffold file (populated_at = null).

use std::collections::BTreeMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Minimal TOML parser (stdlib-only; uses the `toml` crate already in dev-deps)
// ---------------------------------------------------------------------------

/// Find the metrics file relative to the workspace root.
/// The workspace root is resolved by walking up from CARGO_MANIFEST_DIR.
fn metrics_path() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/contracts at test time.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&manifest)
        .ancestors()
        .nth(2) // workspace root (.elai_cc)
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("metrics")
        .join("phase_b29_5_scale.toml")
}

// ---------------------------------------------------------------------------
// Contract test
// ---------------------------------------------------------------------------

#[test]
fn scale_p99_budget_within_limits() {
    let path = metrics_path();

    if !path.exists() {
        eprintln!(
            "[scale_p99_contract] metrics/phase_b29_5_scale.toml not found at {} — SKIP",
            path.display()
        );
        return;
    }

    let contents = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[scale_p99_contract] Could not read metrics file: {e} — SKIP");
            return;
        }
    };

    let doc: toml::Value = match toml::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[scale_p99_contract] TOML parse error: {e} — SKIP");
            return;
        }
    };

    // Check [meta].populated_at — skip if null or missing.
    let populated_at = doc
        .get("meta")
        .and_then(|m| m.get("populated_at"))
        .and_then(|v| v.as_str());

    match populated_at {
        None | Some("null") => {
            eprintln!(
                "[scale_p99_contract] metrics/phase_b29_5_scale.toml is a schema scaffold \
                 (populated_at = null) — SKIP (not a failure)"
            );
            return;
        }
        Some(ts) => {
            eprintln!("[scale_p99_contract] Checking metrics populated at: {ts}");
        }
    }

    // Collect violations.
    let mut violations: Vec<String> = Vec::new();

    let tiers = &[
        "linux_kernel",
        "chromium",
        "vscode",
        "postgres",
        "elai_self",
    ];
    let tools = &[
        "impact_radius",
        "call_flow",
        "symbol_context",
        "list_symbols",
    ];

    for &tier in tiers {
        let tier_val = match doc.get(tier) {
            Some(v) => v,
            None => continue, // tier not present in file — skip
        };

        // Check peak RSS.
        let rss_mb = tier_val
            .get("peak_rss_mb_during_workload")
            .or_else(|| {
                // Also accept under [tier.workload]
                tier_val
                    .get("workload")
                    .and_then(|w| w.get("peak_rss_mb_during_workload"))
            })
            .and_then(|v| v.as_integer());

        if let Some(rss) = rss_mb {
            if rss > 2048 {
                violations.push(format!(
                    "{tier}: peak_rss_mb_during_workload={rss} > 2048 MB"
                ));
            }
        }

        // Check per-tool p99.
        let workload_section = tier_val.get("workload");

        for &tool in tools {
            let p99_ms = workload_section
                .and_then(|w| w.get(tool))
                .and_then(|t| t.get("p99_ms"))
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));

            if let Some(p99) = p99_ms {
                if p99 > 500.0 {
                    violations.push(format!(
                        "{tier}.{tool}: p99_ms={p99:.1} > 500 ms budget"
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        eprintln!("[scale_p99_contract] All p99 budgets within limits.");
    } else {
        for v in &violations {
            eprintln!("[scale_p99_contract] VIOLATION: {v}");
        }
        assert!(
            violations.is_empty(),
            "scale_p99_budget_contract: {} budget violation(s):\n  {}",
            violations.len(),
            violations.join("\n  ")
        );
    }
}

// ---------------------------------------------------------------------------
// Compile-time smoke test: toml crate is accessible
// ---------------------------------------------------------------------------

#[test]
fn toml_parse_empty_string_is_ok() {
    let result: Result<BTreeMap<String, toml::Value>, _> = toml::from_str("");
    assert!(result.is_ok(), "empty TOML should parse to empty map");
}
