use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::ScaffoldError;

pub const VAR_OPEN: &str = "{{";
pub const VAR_CLOSE: &str = "}}";

pub fn substitute_variables(
    input: &str,
    variables: &BTreeMap<String, String>,
) -> Result<String, ScaffoldError> {
    let bytes = input.as_bytes();
    let open = VAR_OPEN.as_bytes();
    let close = VAR_CLOSE.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i: usize = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(open) {
            let rest = &bytes[i + open.len()..];
            if let Some(end_rel) = find_subslice(rest, close) {
                let name_bytes = &rest[..end_rel];
                if is_ascii_identifier(name_bytes) {
                    let name = std::str::from_utf8(name_bytes)
                        .map_err(|_| ScaffoldError::VariableMissing(
                            String::from_utf8_lossy(name_bytes).into_owned(),
                        ))?;
                    let value = variables
                        .get(name)
                        .ok_or_else(|| ScaffoldError::VariableMissing(name.to_string()))?;
                    out.extend_from_slice(value.as_bytes());
                    i += open.len() + end_rel + close.len();
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    let text = String::from_utf8(out).map_err(|_| ScaffoldError::InvalidUtf8(PathBuf::new()))?;
    Ok(text)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    let mut i: usize = 0;
    while i <= last {
        if &haystack[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn is_ascii_identifier(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    for b in &bytes[1..] {
        if !(b.is_ascii_alphanumeric() || *b == b'_') {
            return false;
        }
    }
    true
}
