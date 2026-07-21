// J-24 LEG5=A contract: wire-dark-allowlist rows for steering flags.
//
// Verifies that wire-dark-allowlist.toml contains rows for both
// ELAI_STEERING_ENABLE and ELAI_PRECOMPUTE_STEERING with all 6 required
// fields, type=experiment, and expiry_date in the future.
//
// The allowlist file has pre-existing unquoted values that trip the toml
// crate, so we parse the two [[flag]] blocks we care about with a simple
// line scanner instead of full TOML deserialization.

use std::path::PathBuf;

fn allowlist_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("wire-dark-allowlist.toml")
}

struct FlagRow {
    name: String,
    flag_type: String,
    owner: String,
    expiry_date: String,
    promotion_plan_id: String,
    smoke_test: String,
}

fn parse_flag_block(lines: &[&str], start: usize) -> Option<(FlagRow, usize)> {
    let mut row = FlagRow {
        name: String::new(),
        flag_type: String::new(),
        owner: String::new(),
        expiry_date: String::new(),
        promotion_plan_id: String::new(),
        smoke_test: String::new(),
    };
    let mut i = start;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("[[flag]]") {
            if i > start {
                return Some((row, i));
            }
        } else if line.starts_with('[') {
            if i > start {
                return Some((row, i));
            }
        } else if !line.is_empty() && !line.starts_with('#') {
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"');
                match key {
                    "name" => row.name = val.to_string(),
                    "type" => row.flag_type = val.to_string(),
                    "owner" => row.owner = val.to_string(),
                    "expiry_date" => row.expiry_date = val.to_string(),
                    "promotion_plan_id" => row.promotion_plan_id = val.to_string(),
                    "smoke_test" => row.smoke_test = val.to_string(),
                    _ => {}
                }
            }
        }
        i += 1;
    }
    if i > start {
        Some((row, i))
    } else {
        None
    }
}

fn find_flag_by_name(content: &str, target: &str) -> Option<FlagRow> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim().starts_with("[[flag]]") {
            if let Some((row, next)) = parse_flag_block(&lines, i) {
                if row.name == target {
                    return Some(row);
                }
                i = next;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    None
}

fn load_row(name: &str) -> FlagRow {
    let path = allowlist_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    find_flag_by_name(&content, name)
        .unwrap_or_else(|| panic!("no allowlist row for {name}"))
}

fn assert_all_fields_present(row: &FlagRow) {
    assert!(!row.name.is_empty(), "name must be non-empty");
    assert!(!row.flag_type.is_empty(), "type must be non-empty");
    assert!(!row.owner.is_empty(), "owner must be non-empty");
    assert!(!row.expiry_date.is_empty(), "expiry_date must be non-empty");
    assert!(!row.promotion_plan_id.is_empty(), "promotion_plan_id must be non-empty");
    assert!(!row.smoke_test.is_empty(), "smoke_test must be non-empty");
}

fn assert_expiry_in_future(row: &FlagRow) {
    let today = "2026-07-22";
    assert!(
        row.expiry_date.as_str() > today,
        "expiry_date {} must be in the future (after {})",
        row.expiry_date,
        today
    );
}

#[test]
fn both_rows_present_with_all_fields() {
    let steering = load_row("ELAI_STEERING_ENABLE");
    let precompute = load_row("ELAI_PRECOMPUTE_STEERING");

    assert_all_fields_present(&steering);
    assert_all_fields_present(&precompute);

    assert_eq!(
        steering.flag_type, "experiment",
        "ELAI_STEERING_ENABLE type must be experiment"
    );
    assert_eq!(
        precompute.flag_type, "experiment",
        "ELAI_PRECOMPUTE_STEERING type must be experiment"
    );

    assert_expiry_in_future(&steering);
    assert_expiry_in_future(&precompute);
}

#[test]
fn steering_enable_row_has_correct_plan_id() {
    let row = load_row("ELAI_STEERING_ENABLE");
    assert_eq!(row.promotion_plan_id, "J-24");
}

#[test]
fn precompute_steering_row_has_correct_plan_id() {
    let row = load_row("ELAI_PRECOMPUTE_STEERING");
    assert_eq!(row.promotion_plan_id, "J-24");
}
