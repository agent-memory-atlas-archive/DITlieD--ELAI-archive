//! Golden-file lock for `pub struct *Payload` field sets in `ssb/payloads.rs`.
//!
//! Enforces the invariant that the set of fields (name + type text) on every
//! `pub struct *Payload` never silently changes via rename, add, remove, or
//! type-text edit. The existing `action_type_lock` locks wire-tag NAMES only;
//! this lock closes the field-shape hole.
//!
//! Three tests:
//!   - `payload_field_shape_golden_matches_source` -- byte-equality between
//!     the scanned field sets (sorted by struct name, fields sorted
//!     alphabetically) and the body lines of
//!     tests/payload_field_shape.golden.
//!   - `payload_field_shape_coverage` -- every scanned `pub struct *Payload`
//!     appears in the golden; count > 500.
//!   - `regen_payload_field_shape_golden` (#[ignore]'d helper) -- rewrites the
//!     golden from the current source. Run with `--ignored --nocapture` after
//!     a deliberate, reviewed schema change.
//!
//! Mechanism: stdlib only. Reads `../src/ssb/payloads.rs` relative to
//! CARGO_MANIFEST_DIR/tests, brace-aware scan collecting `pub name: type`
//! pairs. No new Cargo dep.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Path to the payloads.rs source file.
fn payloads_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ssb")
        .join("payloads.rs")
}

/// Path to the golden file.
fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("payload_field_shape.golden")
}

/// One scanned struct: name + sorted (field_name, type_text) pairs.
struct PayloadShape {
    name: String,
    fields: Vec<(String, String)>,
}

/// Scan payloads.rs for every `pub struct NamePayload { ... }` and collect
/// field name + type text. Fields are sorted alphabetically within each
/// struct; structs are sorted alphabetically by name.
fn scan_payloads() -> Vec<PayloadShape> {
    let src = fs::read_to_string(payloads_path()).unwrap_or_else(|e| {
        panic!(
            "cannot read payloads.rs at {}: {}",
            payloads_path().display(),
            e
        )
    });
    let lines: Vec<&str> = src.lines().collect();
    let mut structs: Vec<PayloadShape> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if let Some(name) = parse_struct_header(line) {
            let mut fields: Vec<(String, String)> = Vec::new();
            i += 1;
            while i < lines.len() {
                let fl = lines[i].trim();
                if fl == "}" {
                    i += 1;
                    break;
                }
                if fl.is_empty()
                    || fl.starts_with("///")
                    || fl.starts_with("//")
                    || fl.starts_with("#[")
                {
                    i += 1;
                    continue;
                }
                if let Some((fname, ftype)) = parse_field(fl) {
                    fields.push((fname, ftype));
                }
                i += 1;
            }
            fields.sort_by(|a, b| a.0.cmp(&b.0));
            structs.push(PayloadShape { name, fields });
        } else {
            i += 1;
        }
    }
    structs.sort_by(|a, b| a.name.cmp(&b.name));
    structs
}

/// If the line is `pub struct NamePayload {`, return `Some(Name)`.
fn parse_struct_header(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("pub struct ") {
        return None;
    }
    if !trimmed.ends_with('{') {
        return None;
    }
    let mid = &trimmed["pub struct ".len()..trimmed.len() - 1].trim();
    if mid.ends_with("Payload") {
        Some(mid.to_string())
    } else {
        None
    }
}

/// If the line is `pub name: type,` return `Some((name, type))`.
fn parse_field(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("pub ") {
        return None;
    }
    let rest = &trimmed["pub ".len()..];
    let colon = rest.find(':')?;
    let name = rest[..colon].trim().to_string();
    let mut type_text = rest[colon + 1..].trim().to_string();
    if type_text.ends_with(',') {
        type_text.pop();
        type_text = type_text.trim().to_string();
    }
    if name.is_empty() || type_text.is_empty() {
        return None;
    }
    Some((name, type_text))
}

/// Build the sorted body lines the golden file should contain.
fn runtime_body_lines() -> Vec<String> {
    let structs = scan_payloads();
    structs
        .iter()
        .map(|s| {
            let field_str = s
                .fields
                .iter()
                .map(|(n, t)| format!("{}:{}", n, t))
                .collect::<Vec<_>>()
                .join(",");
            format!("{}\t{}", s.name, field_str)
        })
        .collect()
}

/// Split the golden file into (header_comments, body_lines).
fn split_golden(text: &str) -> (Vec<String>, Vec<String>) {
    let mut header = Vec::new();
    let mut body = Vec::new();
    for raw in text.lines() {
        if raw.starts_with('#') || raw.is_empty() {
            header.push(raw.to_string());
        } else {
            body.push(raw.to_string());
        }
    }
    (header, body)
}

#[test]
fn payload_field_shape_golden_matches_source() {
    let path = golden_path();
    let golden_text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => panic!(
            "cannot read golden file {}: {}. Regenerate via \
             `cargo test -p elai-orchestrator --test payload_field_shape_lock \
             regen_payload_field_shape_golden -- --ignored --nocapture`.",
            path.display(),
            e
        ),
    };
    let (_header, golden_body) = split_golden(&golden_text);
    let runtime_body = runtime_body_lines();

    if golden_body == runtime_body {
        return;
    }

    let golden_set: BTreeSet<&str> = golden_body.iter().map(|s| s.as_str()).collect();
    let runtime_set: BTreeSet<&str> = runtime_body.iter().map(|s| s.as_str()).collect();
    let only_runtime: Vec<&&str> = runtime_set.difference(&golden_set).collect();
    let only_golden: Vec<&&str> = golden_set.difference(&runtime_set).collect();

    let mut msg = String::new();
    msg.push_str("payload field-shape golden-file drift detected.\n\n");
    if !only_runtime.is_empty() {
        msg.push_str(
            "Lines present in source but missing from golden \
             (new structs OR field renames/adds/type changes):\n",
        );
        for line in &only_runtime {
            msg.push_str("  + ");
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !only_golden.is_empty() {
        msg.push_str(
            "Lines present in golden but missing from source \
             (struct removed OR field renamed/removed):\n",
        );
        for line in &only_golden {
            msg.push_str("  - ");
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if only_runtime.is_empty() && only_golden.is_empty() {
        msg.push_str(
            "Set-equal but ordering differs -- alphabetical sort \
             invariant broken. This should be impossible unless the test \
             itself was edited.\n",
        );
    }
    msg.push_str(
        "\nTo accept this change (after confirming it is NOT an accidental rename):\n",
    );
    msg.push_str("  cargo test -p elai-orchestrator --test payload_field_shape_lock regen_payload_field_shape_golden -- --ignored --nocapture\n");
    msg.push_str(
        "Then review the diff in `git diff -- .elai_cc/crates/orchestrator/tests/payload_field_shape.golden` before committing.\n",
    );
    panic!("{}", msg);
}

#[test]
fn payload_field_shape_coverage() {
    let structs = scan_payloads();
    let count = structs.len();
    assert!(
        count > 500,
        "expected >500 pub struct *Payload in payloads.rs, found {}. \
         If this is a deliberate mass-removal, update the threshold.",
        count
    );

    let scanned_names: BTreeSet<&str> = structs.iter().map(|s| s.name.as_str()).collect();

    let golden_text = fs::read_to_string(golden_path()).unwrap_or_else(|e| {
        panic!(
            "cannot read golden file {}: {}",
            golden_path().display(),
            e
        )
    });
    let (_header, golden_body) = split_golden(&golden_text);
    let golden_names: BTreeSet<&str> = golden_body
        .iter()
        .filter_map(|l| l.split('\t').next())
        .collect();

    let missing_in_golden: Vec<&&str> = scanned_names.difference(&golden_names).collect();
    let missing_in_source: Vec<&&str> = golden_names.difference(&scanned_names).collect();

    if !missing_in_golden.is_empty() {
        panic!(
            "coverage check: {} struct(s) in payloads.rs but NOT in golden:\n{}\n\
             Regenerate via `cargo test -p elai-orchestrator --test \
             payload_field_shape_lock regen_payload_field_shape_golden -- \
             --ignored --nocapture`.",
            missing_in_golden.len(),
            missing_in_golden.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
        );
    }
    if !missing_in_source.is_empty() {
        panic!(
            "coverage check: {} struct(s) in golden but NOT in payloads.rs:\n{}\n\
             The golden references structs that no longer exist in source.",
            missing_in_source.len(),
            missing_in_source.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
        );
    }
}

/// Regenerate the golden file in place. `#[ignore]` so it never runs in CI.
/// Invoke manually after a deliberate payload struct change:
///
///   cargo test -p elai-orchestrator --test payload_field_shape_lock \
///       regen_payload_field_shape_golden -- --ignored --nocapture
///
/// Review the resulting `git diff` BEFORE committing.
#[test]
#[ignore]
fn regen_payload_field_shape_golden() {
    let path = golden_path();
    let mut out = String::new();
    out.push_str("# payload_field_shape.golden -- pub struct *Payload field-set lock file\n");
    out.push_str("# Generated by: cargo test -p elai-orchestrator --test payload_field_shape_lock regen_payload_field_shape_golden -- --ignored --nocapture\n");
    out.push_str("# Format: one line per struct, ALPHABETICALLY sorted by struct name.\n");
    out.push_str("# Each line: <StructName>\\t<field1>:<type1>,<field2>:<type2>,...\n");
    out.push_str("# Fields are sorted alphabetically within each struct (field SET, not declaration order).\n");
    out.push_str("# The lock catches field rename / add / remove / type-text change.\n");
    out.push_str("# Nested type-shape inside field types is NOT locked (type text is opaque).\n");
    out.push_str("#\n");
    let body = runtime_body_lines();
    for line in &body {
        out.push_str(line);
        out.push('\n');
    }
    match fs::write(&path, &out) {
        Ok(_) => {}
        Err(e) => panic!(
            "regen_payload_field_shape_golden: failed to write {}: {}",
            path.display(),
            e
        ),
    }
    eprintln!(
        "regen_payload_field_shape_golden: wrote {} struct lines to {}",
        body.len(),
        path.display()
    );
}
