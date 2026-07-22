//! Coverage + behavioral lock for `#[serde(deny_unknown_fields)]` on every
//! `pub struct *Payload` in `ssb/payloads.rs`.
//!
//! J-22 T2: the field-shape golden (T1) locks field names/types; this lock
//! closes the unknown-field hole by ensuring every Payload struct carries
//! `#[serde(deny_unknown_fields)]` and that the attribute is live (an extra
//! JSON key is rejected at deserialization time).
//!
//! Two tests:
//!   - `payload_deny_unknown_coverage` -- every `pub struct \w+Payload` has
//!     `#[serde(deny_unknown_fields)]` in the contiguous attribute block
//!     above it; count > 500.
//!   - `payload_deny_unknown_rejects_extra_field` -- behavioral: deserialize
//!     a `FileWritePayload` JSON with an extra `__j22_unknown__` key and
//!     assert Err.

use std::fs;
use std::path::PathBuf;

fn payloads_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ssb")
        .join("payloads.rs")
}

#[test]
fn payload_deny_unknown_coverage() {
    let src = fs::read_to_string(payloads_path()).expect("read payloads.rs");
    let lines: Vec<&str> = src.lines().collect();

    let mut payload_structs: Vec<(usize, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("pub struct ") {
            if let Some(name_end) = rest.find(|c: char| !c.is_alphanumeric() && c != '_') {
                let name = &rest[..name_end];
                if name.ends_with("Payload") {
                    payload_structs.push((i, name.to_string()));
                }
            }
        }
    }

    assert!(
        payload_structs.len() > 500,
        "payload struct count {} is not > 500",
        payload_structs.len()
    );

    let mut missing: Vec<String> = Vec::new();
    for (line_idx, name) in &payload_structs {
        let mut found = false;
        for j in (0..*line_idx).rev() {
            let prev = lines[j].trim();
            if prev.is_empty() {
                break;
            }
            if prev.starts_with("#[serde(deny_unknown_fields)]") {
                found = true;
                break;
            }
            if prev.starts_with("#[") || prev.starts_with("///") || prev.starts_with("//") {
                continue;
            }
            break;
        }
        if !found {
            missing.push(name.clone());
        }
    }

    assert!(
        missing.is_empty(),
        "{} Payload structs missing #[serde(deny_unknown_fields)]: {}",
        missing.len(),
        missing.join(", ")
    );
}

#[test]
fn payload_deny_unknown_rejects_extra_field() {
    use elai_orchestrator::ssb::FileWritePayload;

    let json = r#"{"path":"/x","size_bytes":0,"hash":"abc","__j22_unknown__":1}"#;
    let result: Result<FileWritePayload, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "FileWritePayload accepted an unknown field; deny_unknown_fields is not live"
    );

    let valid = r#"{"path":"/x","size_bytes":0,"hash":"abc"}"#;
    let ok: FileWritePayload =
        serde_json::from_str(valid).expect("valid FileWritePayload must deserialize");
    assert_eq!(ok.path, "/x");
}
