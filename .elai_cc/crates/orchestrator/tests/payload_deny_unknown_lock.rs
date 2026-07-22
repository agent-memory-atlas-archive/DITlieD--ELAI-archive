//! Coverage + behavioral lock for `#[serde(deny_unknown_fields)]` on every
//! SsbPayload variant payload struct.
//!
//! ENUM-DRIVEN: scans the `SsbPayload` enum in `ssb/payloads.rs` to collect
//! the set of payload type names, then walks every `.rs` under `src/` to
//! find each struct definition and verify `#[serde(deny_unknown_fields)]`
//! is present in the contiguous attr/doc block above it.
//!
//! J-22 T2: the field-shape golden (T1) locks field names/types; this lock
//! closes the unknown-field hole by ensuring every Payload struct carries
//! `#[serde(deny_unknown_fields)]` and that the attribute is live (an extra
//! JSON key is rejected at deserialization time).
//!
//! Two tests:
//!   - `payload_deny_unknown_coverage` -- every SsbPayload variant payload
//!     struct has `#[serde(deny_unknown_fields)]` in the contiguous
//!     attribute block above it; count > 500.
//!   - `payload_deny_unknown_rejects_extra_field` -- behavioral: deserialize
//!     a `ClaudeCodeArchitectureEventPayload` JSON with an extra
//!     `__j22_unknown__` key and assert Err.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn payloads_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ssb")
        .join("payloads.rs")
}

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// If the line is `    VariantName(TypeName),` return `Some(TypeName)`.
fn parse_enum_variant_payload(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("#[")
        || trimmed.starts_with("///")
    {
        return None;
    }
    let open = trimmed.find('(')?;
    let close = trimmed.rfind(')')?;
    if open >= close {
        return None;
    }
    let type_name = trimmed[open + 1..close].trim();
    if type_name.is_empty() {
        return None;
    }
    Some(type_name.to_string())
}

/// Extract the set of payload type names from `pub enum SsbPayload`.
fn collect_enum_payload_types(src: &str) -> BTreeSet<String> {
    let lines: Vec<&str> = src.lines().collect();
    let enum_start = lines
        .iter()
        .position(|l| l.trim_start().starts_with("pub enum SsbPayload"))
        .expect("pub enum SsbPayload not found in payloads.rs");

    let mut types: BTreeSet<String> = BTreeSet::new();
    let mut depth: i32 = 0;
    for line in &lines[enum_start..] {
        let trimmed = line.trim();
        if trimmed == "{" || trimmed.ends_with('{') {
            depth += trimmed.matches('{').count() as i32;
        }
        depth -= trimmed.matches('}').count() as i32;
        if depth <= 0 {
            break;
        }
        if let Some(type_name) = parse_enum_variant_payload(trimmed) {
            types.insert(type_name);
        }
    }
    types
}

/// Recursively collect all `.rs` files under `dir`.
fn collect_rs_files(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            out.push(path);
        }
    }
}

/// If the line is `pub struct Name {` or `struct Name {`, return `Some(Name)`.
fn parse_struct_header(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed
        .strip_prefix("pub struct ")
        .or_else(|| trimmed.strip_prefix("struct "))?;
    let brace_idx = rest.find('{')?;
    let name = rest[..brace_idx].trim();
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

/// For each enum-resolved payload type, find its struct definition file and
/// check that `#[serde(deny_unknown_fields)]` appears in the contiguous
/// attr/doc block above the struct header line.
#[test]
fn payload_deny_unknown_coverage() {
    let src = fs::read_to_string(payloads_path()).expect("read payloads.rs");
    let type_set = collect_enum_payload_types(&src);

    assert!(
        type_set.len() > 500,
        "enum-resolved payload type count {} is not > 500",
        type_set.len()
    );

    let root = src_root();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_rs_files(&root, &mut files);

    let mut missing: Vec<String> = Vec::new();
    let mut found_count: usize = 0;

    for file_path in &files {
        let file_src = match fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let lines: Vec<&str> = file_src.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if let Some(name) = parse_struct_header(line) {
                if !type_set.contains(&name) {
                    continue;
                }
                found_count += 1;
                let mut has_deny = false;
                for j in (0..i).rev() {
                    let prev = lines[j].trim();
                    if prev.is_empty() {
                        break;
                    }
                    if prev.starts_with("#[serde(deny_unknown_fields)]") {
                        has_deny = true;
                        break;
                    }
                    if prev.starts_with("#[") || prev.starts_with("///") || prev.starts_with("//") {
                        continue;
                    }
                    break;
                }
                if !has_deny {
                    missing.push(format!("{} (in {})", name, file_path.display()));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "{} SsbPayload variant payload structs missing #[serde(deny_unknown_fields)]: {}",
        missing.len(),
        missing.join(", ")
    );
}

#[test]
fn payload_deny_unknown_rejects_extra_field() {
    use elai_orchestrator::architecture::claude_code_arch::ClaudeCodeArchitectureEventPayload;

    let json = r#"{"kind":"compaction_ladder_tier_applied","primitive":"compaction_ladder","session_id":null,"details_json":{},"ts_ms":0,"__j22_unknown__":1}"#;
    let result: Result<ClaudeCodeArchitectureEventPayload, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "ClaudeCodeArchitectureEventPayload accepted an unknown field; deny_unknown_fields is not live"
    );

    let valid = r#"{"kind":"compaction_ladder_tier_applied","primitive":"compaction_ladder","session_id":null,"details_json":{},"ts_ms":0}"#;
    let ok: ClaudeCodeArchitectureEventPayload =
        serde_json::from_str(valid).expect("valid ClaudeCodeArchitectureEventPayload must deserialize");
    assert_eq!(ok.ts_ms, 0);
}
