//! B-31.5 Tier B contract — crash-resume identity.
//!
//! Reads `metrics/phase_b31_5_tier_b.toml` (relative to CARGO_MANIFEST_DIR's
//! grandparent, which is the workspace root) and asserts:
//!   * Every `[<tier>_<pct>pct]` section has `identity_diff_lines == 0`.
//!   * `files_skipped_on_resume > 0` on every non-zero kill point (the 0%
//!     case has nothing to skip — exempt).
//!   * `ssb_index_resume_action_count == 1` on every section.
//!
//! Section names are not hard-coded — every TOML table whose key contains
//! `pct` is treated as a tier B sub-scenario. This keeps the test robust as
//! we add tiers later (linux_10pct, chromium_50pct, etc.).
//!
//! Uses the `toml` crate (workspace dep) — no external runtime parse helper.

use std::path::PathBuf;

fn metric_path(name: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/orchestrator. The workspace root is two
    // levels up, and metrics/ lives there.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set under cargo test");
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .expect("orchestrator parent")
        .parent()
        .expect("workspace root")
        .to_path_buf();
    workspace_root.join("metrics").join(name)
}

#[test]
fn scale_tier_b_identity_diff_zero_and_skipped_positive() {
    let path = metric_path("phase_b31_5_tier_b.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let value: toml::Value = toml::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));

    let table = value.as_table().expect("top-level must be a table");
    let mut sections_seen = 0usize;
    for (section_name, section) in table {
        if !section_name.contains("pct") {
            continue;
        }
        let section = section
            .as_table()
            .unwrap_or_else(|| panic!("section [{section_name}] must be a table"));
        let diff_lines = section
            .get("identity_diff_lines")
            .and_then(|v| v.as_integer())
            .unwrap_or_else(|| {
                panic!("[{section_name}] missing identity_diff_lines:i64")
            });
        assert_eq!(
            diff_lines, 0,
            "[{section_name}] identity_diff_lines must be 0; resume must produce identical DKG output, got {diff_lines}"
        );
        let resume_count = section
            .get("ssb_index_resume_action_count")
            .and_then(|v| v.as_integer())
            .unwrap_or_else(|| {
                panic!("[{section_name}] missing ssb_index_resume_action_count:i64")
            });
        assert_eq!(
            resume_count, 1,
            "[{section_name}] ssb_index_resume_action_count must be exactly 1, got {resume_count}"
        );
        let files_skipped = section
            .get("files_skipped_on_resume")
            .and_then(|v| v.as_integer())
            .unwrap_or_else(|| {
                panic!("[{section_name}] missing files_skipped_on_resume:i64")
            });
        assert!(
            files_skipped > 0,
            "[{section_name}] files_skipped_on_resume must be > 0 (resume should skip already-hashed files), got {files_skipped}"
        );
        sections_seen += 1;
    }
    assert!(
        sections_seen >= 3,
        "expected >=3 *_pct sections in {}; saw {}",
        path.display(),
        sections_seen
    );
}
