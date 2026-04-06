//! Contract: elai_cc ToolRegistry can register and execute a
//! file_read tool, returning a structured ToolResult.
//!
//! RED phase — compiles against existing tools types, fails at
//! runtime because the new execute->ToolResult contract is absent.

use elai_tools::GlobalToolRegistry;
use serde_json::json;

/// The tool registry must be able to execute file_read and return
/// a result containing the file contents. We call execute() with
/// a path that does not exist — the current implementation returns
/// Err with a plain string. The CONTRACT requires the error to be
/// a structured JSON object with an "error_code" field.
#[test]
fn tool_registry_execute_returns_structured_result() {
    let registry = GlobalToolRegistry::builtin();

    // Execute file_read on a nonexistent file.
    let result = registry.execute(
        "read_file",
        &json!({"path": "/nonexistent/contract_test_sentinel.txt"}),
    );

    // The current implementation returns either Ok(string) or Err(string).
    // CONTRACT: the error must be structured JSON with an "error_code" field.
    // Parse whatever we get and look for the contract shape.
    let output = match &result {
        Ok(s) => s.clone(),
        Err(e) => e.clone(),
    };

    let parsed: serde_json::Value = serde_json::from_str(&output)
        .expect("CONTRACT: tool output must be valid JSON");
    assert!(
        parsed.get("error_code").is_some(),
        "CONTRACT: tool error must contain error_code field — got: {parsed}"
    );
}
