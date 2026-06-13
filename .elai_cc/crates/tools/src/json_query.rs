//! I-53: JSON tree query — path-navigate and bounded-render a JSON file.
//!
//! Ports the cap constants from gajae json-tree.ts (a TUI renderer for
//! in-memory values). This module is a FILE-QUERY tool: read a JSON file
//! from disk, navigate a dot-path, render a bounded subtree.
//!
//! Invariants:
//!   - Hard file size cap (`MAX_FILE_BYTES`) prevents OOM on multi-GB files.
//!   - Output line count cap (`max_lines`) prevents context flooding.
//!   - Depth cap (`max_depth`) prevents infinite recursion on deeply nested values.
//!   - Scalar values are truncated to `SCALAR_LEN_DEFAULT` characters.
//!   - A truncation marker is appended when any cap fires.

use std::path::Path;

use serde_json::Value;

// ---------------------------------------------------------------------------
// Constants (ported from gajae json-tree.ts)
// ---------------------------------------------------------------------------

/// Default max depth when rendering a collapsed subtree.
pub const MAX_DEPTH_DEFAULT: usize = 2;
/// Default max output lines (collapsed).
pub const MAX_LINES_DEFAULT: usize = 6;
/// Max output lines for an expanded render.
pub const MAX_LINES_EXPANDED: usize = 200;
/// Max characters shown for a scalar value before truncation.
pub const SCALAR_LEN_DEFAULT: usize = 60;
/// Max file size (bytes) we will read. Prevents OOM on huge files.
const MAX_FILE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB

// ---------------------------------------------------------------------------
// Dot-path navigation
// ---------------------------------------------------------------------------

/// Navigate into `root` along a dot-separated path string.
/// Returns `None` if any segment is missing or the path is empty.
///
/// Array indices are supported as numeric segments: `a.0.b`.
fn navigate<'a>(root: &'a Value, dot_path: &str) -> Option<&'a Value> {
    let mut current = root;
    for segment in dot_path.split('.') {
        if segment.is_empty() {
            continue;
        }
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            Value::Array(arr) => {
                let idx: usize = segment.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

// ---------------------------------------------------------------------------
// Bounded subtree render
// ---------------------------------------------------------------------------

/// State threaded through the recursive render to track output limits.
struct RenderState<'a> {
    lines: &'a mut Vec<String>,
    max_lines: usize,
    max_depth: usize,
    scalar_len: usize,
    truncated: bool,
}

impl<'a> RenderState<'a> {
    fn push(&mut self, line: String) -> bool {
        if self.lines.len() >= self.max_lines {
            self.truncated = true;
            return false;
        }
        self.lines.push(line);
        true
    }
}

fn render_value(val: &Value, indent: usize, depth: usize, state: &mut RenderState<'_>) {
    if state.lines.len() >= state.max_lines {
        state.truncated = true;
        return;
    }

    let prefix = "  ".repeat(indent);

    if depth > state.max_depth {
        state.push(format!("{prefix}..."));
        state.truncated = true;
        return;
    }

    match val {
        Value::Null => {
            state.push(format!("{prefix}null"));
        }
        Value::Bool(b) => {
            state.push(format!("{prefix}{b}"));
        }
        Value::Number(n) => {
            state.push(format!("{prefix}{n}"));
        }
        Value::String(s) => {
            let truncated_s = if s.len() > state.scalar_len {
                format!("\"{}...\"", &s[..state.scalar_len])
            } else {
                format!("\"{s}\"")
            };
            state.push(format!("{prefix}{truncated_s}"));
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                state.push(format!("{prefix}[]"));
                return;
            }
            state.push(format!("{prefix}["));
            for (i, item) in arr.iter().enumerate() {
                render_value(item, indent + 1, depth + 1, state);
                if state.lines.len() >= state.max_lines {
                    state.truncated = true;
                    break;
                }
                // Add comma hint on all but last, if we have budget.
                if i + 1 < arr.len() && state.lines.len() < state.max_lines {
                    if let Some(last) = state.lines.last_mut() {
                        last.push(',');
                    }
                }
            }
            if state.lines.len() < state.max_lines {
                state.push(format!("{prefix}]"));
            }
        }
        Value::Object(map) => {
            if map.is_empty() {
                state.push(format!("{prefix}{{}}"));
                return;
            }
            state.push(format!("{prefix}{{"));
            let len = map.len();
            for (i, (key, value)) in map.iter().enumerate() {
                // Key line.
                if state.lines.len() >= state.max_lines {
                    state.truncated = true;
                    break;
                }
                let key_prefix = "  ".repeat(indent + 1);
                // For simple scalar values, render key: value on one line.
                match value {
                    Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                        let val_str = match value {
                            Value::Null => "null".to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => {
                                if s.len() > state.scalar_len {
                                    format!("\"{}...\"", &s[..state.scalar_len])
                                } else {
                                    format!("\"{s}\"")
                                }
                            }
                            _ => unreachable!(),
                        };
                        let comma = if i + 1 < len { "," } else { "" };
                        state.push(format!("{key_prefix}\"{key}\": {val_str}{comma}"));
                    }
                    _ => {
                        // Nested value: key on own line, value rendered below.
                        state.push(format!("{key_prefix}\"{key}\":"));
                        render_value(value, indent + 1, depth + 1, state);
                        if i + 1 < len {
                            if let Some(last) = state.lines.last_mut() {
                                if !last.trim_end().ends_with(',') {
                                    last.push(',');
                                }
                            }
                        }
                    }
                }
            }
            if state.lines.len() < state.max_lines {
                state.push(format!("{prefix}}}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Query a JSON file with an optional dot-path and render a bounded subtree.
///
/// - `file_path`: path to the JSON file (max `MAX_FILE_BYTES`).
/// - `dot_path`: optional `"a.b.c"` navigation path; `None` = render from root.
/// - `max_depth`: depth cap for rendering (default `MAX_DEPTH_DEFAULT`).
/// - `max_lines`: output line cap (default `MAX_LINES_DEFAULT`).
///
/// Returns a rendered string with a truncation marker appended when any cap fires.
pub fn query_json_tree(
    file_path: &Path,
    dot_path: Option<&str>,
    max_depth: usize,
    max_lines: usize,
) -> Result<String, String> {
    // Read file with size cap.
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| format!("json_tree: cannot stat '{}': {e}", file_path.display()))?;
    if metadata.len() as usize > MAX_FILE_BYTES {
        return Err(format!(
            "json_tree: file '{}' is {} bytes (limit: {MAX_FILE_BYTES}); \
             use a more specific dot_path to narrow scope",
            file_path.display(),
            metadata.len(),
        ));
    }

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("json_tree: read '{}' failed: {e}", file_path.display()))?;

    let root: Value = serde_json::from_str(&content)
        .map_err(|e| format!("json_tree: parse '{}' failed: {e}", file_path.display()))?;

    // Navigate to the requested subnode.
    let node: &Value = match dot_path {
        Some(p) if !p.is_empty() => navigate(&root, p).ok_or_else(|| {
            format!(
                "json_tree: path '{p}' not found in '{}'",
                file_path.display()
            )
        })?,
        _ => &root,
    };

    // Render.
    let effective_max_lines = max_lines.max(1);
    let effective_max_depth = max_depth;
    let mut lines: Vec<String> = Vec::with_capacity(effective_max_lines + 1);
    let mut state = RenderState {
        lines: &mut lines,
        max_lines: effective_max_lines,
        max_depth: effective_max_depth,
        scalar_len: SCALAR_LEN_DEFAULT,
        truncated: false,
    };

    render_value(node, 0, 0, &mut state);

    // Drop `state` to release the mutable borrow of `lines` before calling `join`.
    let truncated = state.truncated;
    drop(state);

    let mut output = lines.join("\n");
    if truncated {
        output.push_str(&format!(
            "\n<truncated: output capped at {effective_max_lines} lines or depth {effective_max_depth}>"
        ));
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_json(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).expect("create");
        f.write_all(content.as_bytes()).expect("write");
        path
    }

    #[test]
    fn query_root_flat_object() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_json(&dir, "flat.json", r#"{"a": 1, "b": "hello", "c": true}"#);
        let result = query_json_tree(&path, None, MAX_DEPTH_DEFAULT, MAX_LINES_EXPANDED)
            .expect("query");
        assert!(result.contains("\"a\""), "should contain key 'a'");
        assert!(result.contains("\"b\""), "should contain key 'b'");
    }

    #[test]
    fn query_dot_path_navigation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_json(
            &dir,
            "nested.json",
            r#"{"outer": {"inner": {"value": 42}}}"#,
        );
        let result =
            query_json_tree(&path, Some("outer.inner"), MAX_DEPTH_DEFAULT, MAX_LINES_EXPANDED)
                .expect("query");
        assert!(result.contains("42"), "should navigate to inner and show 42");
    }

    #[test]
    fn query_array_index_navigation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_json(&dir, "arr.json", r#"{"items": [10, 20, 30]}"#);
        let result =
            query_json_tree(&path, Some("items.1"), MAX_DEPTH_DEFAULT, MAX_LINES_EXPANDED)
                .expect("query");
        assert!(result.contains("20"), "items[1] should be 20");
    }

    #[test]
    fn query_line_cap_fires_truncation_marker() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Build a JSON with many keys.
        let mut obj = String::from("{");
        for i in 0..50 {
            obj.push_str(&format!("\"key{i}\": {i}"));
            if i < 49 {
                obj.push(',');
            }
        }
        obj.push('}');
        let path = write_json(&dir, "big.json", &obj);

        // Use collapsed default: MAX_LINES_DEFAULT = 6.
        let result =
            query_json_tree(&path, None, MAX_DEPTH_DEFAULT, MAX_LINES_DEFAULT).expect("query");
        let lines: Vec<&str> = result.lines().collect();
        // Should have <= MAX_LINES_DEFAULT + 1 (the truncation marker line).
        assert!(
            lines.len() <= MAX_LINES_DEFAULT + 2,
            "collapsed output should be capped; got {} lines:\n{result}",
            lines.len()
        );
        assert!(
            result.contains("truncated"),
            "truncation marker must be present; got: {result}"
        );
    }

    #[test]
    fn query_depth_cap_fires_truncation_marker() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Build deeply nested JSON (depth 10).
        let mut json = String::new();
        for _ in 0..10 {
            json.push_str("{\"x\":");
        }
        json.push_str("42");
        for _ in 0..10 {
            json.push('}');
        }
        let path = write_json(&dir, "deep.json", &json);

        let result =
            query_json_tree(&path, None, 2, MAX_LINES_EXPANDED).expect("query");
        assert!(
            result.contains("...") || result.contains("truncated"),
            "depth cap should produce '...' or truncation marker; got:\n{result}"
        );
    }

    #[test]
    fn query_missing_dot_path_returns_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_json(&dir, "obj.json", r#"{"a": 1}"#);
        let result = query_json_tree(&path, Some("nonexistent.key"), MAX_DEPTH_DEFAULT, MAX_LINES_DEFAULT);
        assert!(result.is_err(), "missing path should return Err");
        let err = result.unwrap_err();
        assert!(err.contains("not found"), "error should say 'not found'; got: {err}");
    }

    #[test]
    fn query_invalid_json_returns_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = write_json(&dir, "bad.json", "not json {{{ ~~~");
        let result = query_json_tree(&path, None, MAX_DEPTH_DEFAULT, MAX_LINES_DEFAULT);
        assert!(result.is_err(), "invalid JSON should return Err");
    }
}
