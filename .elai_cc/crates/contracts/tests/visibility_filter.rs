//! Contract: elai_cc must enforce visibility filtering — hidden
//! paths return NotFound identical to missing paths, globs on
//! hidden paths return empty, .elai_cc/** is default-hidden,
//! .elai_cc/models/** is specifically hidden (ONNX model files).
//!
//! RED phase — compiles against existing tool types, fails at
//! runtime because visibility filtering does not exist yet.

use elai_tools::GlobalToolRegistry;
use serde_json::json;

/// A file_read on a hidden path must return an error with a
/// "visibility_filtered" code, identical to a missing-file error.
/// Currently no visibility filter exists — the error does not
/// contain this code, so the assertion fails at runtime.
#[test]
fn hidden_path_returns_visibility_filtered_error() {
    let registry = GlobalToolRegistry::builtin();

    let hidden_result = registry.execute(
        "read_file",
        &json!({"path": ".elai_cc/secret/hidden_file.txt"}),
    );

    // CONTRACT: the error for a hidden path must contain
    // "visibility_filtered" to indicate the filter intercepted it.
    // No such mechanism exists yet.
    let err_output = hidden_result.unwrap_or_else(|e| e);
    assert!(
        err_output.contains("visibility_filtered"),
        "CONTRACT: hidden path error must contain 'visibility_filtered' — got: {err_output}"
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
