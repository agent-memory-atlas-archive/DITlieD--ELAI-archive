//! B-31.5 Tier E contract — long-tail memory plateau.
//!
//! Reads `metrics/phase_b31_5_tier_e.toml`, finds the `*_long_tail` section,
//! and asserts on the `rss_30min_samples_mb` array:
//!   * At least 3 samples present (>=90 minutes of observation).
//!   * NOT monotonically increasing — there must be at least one plateau
//!     or dip (within a 5% noise tolerance) to rule out a steady linear
//!     leak.

use std::path::PathBuf;

fn metric_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set under cargo test");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("orchestrator parent")
        .parent()
        .expect("workspace root")
        .join("metrics")
        .join(name)
}

#[test]
fn scale_tier_e_rss_samples_show_plateau_or_dip() {
    let path = metric_path("phase_b31_5_tier_e.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let value: toml::Value = toml::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));

    // Find a `*_long_tail` section.
    let table = value.as_table().expect("top-level must be a table");
    let long_tail_section = table
        .iter()
        .find(|(name, _)| name.ends_with("_long_tail"))
        .map(|(_, v)| v)
        .expect("no *_long_tail section found");
    let lt = long_tail_section
        .as_table()
        .expect("*_long_tail section must be a table");

    let samples = lt
        .get("rss_30min_samples_mb")
        .and_then(|v| v.as_array())
        .expect("rss_30min_samples_mb missing or not an array");
    let values: Vec<i64> = samples
        .iter()
        .map(|v| {
            v.as_integer()
                .unwrap_or_else(|| panic!("rss_30min_samples_mb element must be integer, got {v:?}"))
        })
        .collect();

    assert!(
        values.len() >= 3,
        "rss_30min_samples_mb must have >=3 samples (>=90 minutes of observation); got {}",
        values.len()
    );

    // Plateau / dip detection: any pair (i, i+1) where value[i+1] <=
    // value[i] * 1.05 counts as "not strictly leaking" — allow 5% noise.
    let mut saw_plateau_or_dip = false;
    for window in values.windows(2) {
        let a = window[0] as f64;
        let b = window[1] as f64;
        if a <= 0.0 {
            continue;
        }
        // Plateau == growth <= 5%.
        if b <= a * 1.05 {
            saw_plateau_or_dip = true;
            break;
        }
    }
    assert!(
        saw_plateau_or_dip,
        "rss_30min_samples_mb is monotonically increasing past the 5% noise threshold ({values:?}); \
         that looks like a steady memory leak — fail the contract"
    );
}
