//! Contract: elai_cc must enforce visibility filtering — hidden
//! paths return NotFound identical to missing paths, globs on
//! hidden paths return empty, .elai_cc/** is default-hidden,
//! .elai_cc/models/** is specifically hidden (ONNX model files).
//!
//! B-18A Task 13 R31 flip: error envelope now mirrors std::io::ErrorKind::NotFound
//! (error_code "not_found", message starts with "No such file or directory:")
//! to make hidden-vs-missing indistinguishable to the agent (tools.txt:264-266).

use elai_tools::GlobalToolRegistry;
use serde_json::json;

/// A file_read on a hidden path must return an error with the same envelope
/// std::io::ErrorKind::NotFound would produce. The error_code is "not_found"
/// and the message begins with "No such file or directory:" — identical to a
/// genuinely missing file. R31 (tools.txt:264-266) requires this so the agent
/// cannot distinguish hidden from missing via timing or string matching.
#[test]
fn hidden_path_returns_not_found_error() {
    let registry = GlobalToolRegistry::builtin();

    let hidden_result = registry.execute(
        "read_file",
        &json!({"path": ".elai_cc/secret/hidden_file.txt"}),
    );

    let err_output = hidden_result.unwrap_or_else(|e| e);
    assert!(
        err_output.contains("\"error_code\":\"not_found\""),
        "CONTRACT R31: hidden path error must carry error_code=\"not_found\" — got: {err_output}"
    );
    assert!(
        err_output.contains("No such file or directory:"),
        "CONTRACT R31: hidden path error must look like std::io::ErrorKind::NotFound display — got: {err_output}"
    );
    assert!(
        !err_output.contains("visibility_filtered"),
        "CONTRACT R31: error must NOT leak that the visibility filter fired — got: {err_output}"
    );
}

/// Glob on a hidden path must return an empty result set.
#[test]
fn glob_on_hidden_path_returns_empty() {
    let registry = GlobalToolRegistry::builtin();

    let result = registry.execute(
        "glob_search",
        &json!({"pattern": ".elai_cc/**/*"}),
    );

    // CONTRACT: glob on hidden path must return empty JSON array.
    let output = result.unwrap_or_else(|e| e);
    let parsed: serde_json::Value = serde_json::from_str(&output)
        .unwrap_or_else(|_| serde_json::Value::Null);

    assert!(
        parsed.is_array() && parsed.as_array().unwrap().is_empty(),
        "CONTRACT: glob on hidden .elai_cc/** must return empty array — got: {output}"
    );
}

/// write_file on a hidden path must mirror the std::io::ErrorKind::NotFound
/// envelope (error_code "not_found", message starts "No such file or
/// directory:"). R31 forbids leaking the visibility filter via timing or
/// error-string differences.
#[test]
fn write_file_hidden_path_returns_not_found_error() {
    let registry = GlobalToolRegistry::builtin();

    let result = registry.execute(
        "write_file",
        &json!({"path": ".elai_cc/secret/forbidden.txt", "content": "x"}),
    );

    let err_output = result.unwrap_or_else(|e| e);
    assert!(
        err_output.contains("\"error_code\":\"not_found\""),
        "CONTRACT R31: write_file hidden path must carry error_code=\"not_found\" — got: {err_output}"
    );
    assert!(
        err_output.contains("No such file or directory:"),
        "CONTRACT R31: write_file hidden path must look like NotFound display — got: {err_output}"
    );
    assert!(
        !err_output.contains("visibility_filtered"),
        "CONTRACT R31: error must NOT leak that the visibility filter fired — got: {err_output}"
    );
}

/// edit_file on a hidden path must mirror the std::io::ErrorKind::NotFound
/// envelope identically to write_file and read_file.
#[test]
fn edit_file_hidden_path_returns_not_found_error() {
    let registry = GlobalToolRegistry::builtin();

    let result = registry.execute(
        "edit_file",
        &json!({
            "path": ".elai_cc/secret/forbidden.txt",
            "old_string": "a",
            "new_string": "b",
        }),
    );

    let err_output = result.unwrap_or_else(|e| e);
    assert!(
        err_output.contains("\"error_code\":\"not_found\""),
        "CONTRACT R31: edit_file hidden path must carry error_code=\"not_found\" — got: {err_output}"
    );
    assert!(
        err_output.contains("No such file or directory:"),
        "CONTRACT R31: edit_file hidden path must look like NotFound display — got: {err_output}"
    );
    assert!(
        !err_output.contains("visibility_filtered"),
        "CONTRACT R31: error must NOT leak that the visibility filter fired — got: {err_output}"
    );
}

/// grep_search with a hidden `path` field must mirror the NotFound envelope.
/// This locks the existing path-branch behavior so cycle-2 fixes do not
/// regress it.
#[test]
fn grep_search_hidden_path_returns_not_found_error() {
    let registry = GlobalToolRegistry::builtin();

    let result = registry.execute(
        "grep_search",
        &json!({
            "pattern": "anything",
            "path": ".elai_cc/sessions",
        }),
    );

    let err_output = result.unwrap_or_else(|e| e);
    assert!(
        err_output.contains("\"error_code\":\"not_found\""),
        "CONTRACT R31: grep_search hidden path must carry error_code=\"not_found\" — got: {err_output}"
    );
    assert!(
        err_output.contains("No such file or directory:"),
        "CONTRACT R31: grep_search hidden path must look like NotFound display — got: {err_output}"
    );
    assert!(
        !err_output.contains("visibility_filtered"),
        "CONTRACT R31: error must NOT leak that the visibility filter fired — got: {err_output}"
    );
}

/// grep_search bypass: an agent can pass path="." (allowed) with
/// glob=".elai_cc/**" (hidden). Cycle-1 only checked `path`. Cycle-2 must
/// also reject the hidden glob with the same NotFound envelope so the bypass
/// is closed.
#[test]
fn grep_search_hidden_glob_returns_not_found_error() {
    let registry = GlobalToolRegistry::builtin();

    let result = registry.execute(
        "grep_search",
        &json!({
            "pattern": "anything",
            "path": ".",
            "glob": ".elai_cc/**",
        }),
    );

    let err_output = result.unwrap_or_else(|e| e);
    assert!(
        err_output.contains("\"error_code\":\"not_found\""),
        "CONTRACT R31: grep_search hidden glob must carry error_code=\"not_found\" — got: {err_output}"
    );
    assert!(
        err_output.contains("No such file or directory:"),
        "CONTRACT R31: grep_search hidden glob must look like NotFound display — got: {err_output}"
    );
    assert!(
        !err_output.contains("visibility_filtered"),
        "CONTRACT R31: error must NOT leak that the visibility filter fired — got: {err_output}"
    );
}

/// .elai_cc/** must be in default hidden_paths.
#[test]
fn elai_cc_dir_in_default_hidden_paths() {
    let registry = GlobalToolRegistry::builtin();

    // CONTRACT: a "list_hidden_paths" tool or equivalent must exist
    // and include ".elai_cc/**". No such tool exists yet.
    let result = registry.execute("list_hidden_paths", &json!({}));
    assert!(
        result.is_ok(),
        "CONTRACT: list_hidden_paths tool must exist — got: {:?}",
        result.err()
    );

    let output = result.unwrap();
    let paths: Vec<String> = serde_json::from_str(&output)
        .expect("list_hidden_paths must return a JSON array of strings");
    assert!(
        paths.iter().any(|p| p.contains(".elai_cc")),
        "CONTRACT: hidden paths must include .elai_cc pattern — got: {paths:?}"
    );
}
