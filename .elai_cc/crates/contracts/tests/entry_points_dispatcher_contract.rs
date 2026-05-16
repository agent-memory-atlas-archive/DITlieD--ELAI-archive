// B-34 contract: entry_points dispatcher tool.
//
// Verifies:
// 1. TOOL_ENTRY_POINTS constant is defined and equals "entry_points".
// 2. EntryPointsRequest serde + postcard roundtrip (include_tests default = true).
// 3. EntryPointsResponse / EntryPointDto / EntryPointSummaryDto serde round-trip.
// 4. JSON field names are snake_case (stable wire format).

use elai_ipc_types::{EntryPointDto, EntryPointSummaryDto, EntryPointsRequest, EntryPointsResponse, TOOL_ENTRY_POINTS};
use std::collections::BTreeMap;

#[test]
fn tool_entry_points_constant_value() {
    assert_eq!(TOOL_ENTRY_POINTS, "entry_points");
}

#[test]
fn entry_points_request_postcard_roundtrip() {
    let req = EntryPointsRequest {
        project_root: "/workspace/myproject".to_string(),
        include_tests: true,
    };
    let encoded = postcard::to_allocvec(&req).expect("encode");
    let decoded: EntryPointsRequest = postcard::from_bytes(&encoded).expect("decode");
    assert_eq!(decoded.project_root, req.project_root);
    assert_eq!(decoded.include_tests, req.include_tests);
}

#[test]
fn entry_points_request_default_include_tests_true() {
    // Verify that serde default for include_tests is true.
    let json = r#"{"project_root":"/proj"}"#;
    let req: EntryPointsRequest = serde_json::from_str(json).expect("json decode");
    assert!(req.include_tests, "include_tests must default to true when absent from JSON");
}

#[test]
fn entry_points_response_json_roundtrip() {
    let mut by_reason = BTreeMap::new();
    by_reason.insert("orphan".to_string(), 2u32);
    by_reason.insert("well_known:main".to_string(), 1u32);
    let resp = EntryPointsResponse {
        entries: vec![
            EntryPointDto {
                id: "src/main.rs".to_string(),
                name: "src/main.rs".to_string(),
                file: "src/main.rs".to_string(),
                line: None,
                reason: "orphan".to_string(),
                reason_detail: None,
            },
            EntryPointDto {
                id: "src/main.rs::main".to_string(),
                name: "main".to_string(),
                file: "src/main.rs".to_string(),
                line: Some(1),
                reason: "well_known".to_string(),
                reason_detail: Some("main".to_string()),
            },
        ],
        summary: EntryPointSummaryDto {
            total: 2,
            by_reason,
        },
    };

    let json = serde_json::to_string(&resp).expect("serialize");
    let decoded: EntryPointsResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.summary.total, 2);
    assert_eq!(decoded.entries.len(), 2);
    assert_eq!(decoded.entries[0].reason, "orphan");
    assert_eq!(decoded.entries[1].reason_detail, Some("main".to_string()));
}

#[test]
fn entry_points_response_json_has_snake_case_fields() {
    let resp = EntryPointsResponse {
        entries: vec![EntryPointDto {
            id: "x".to_string(),
            name: "x".to_string(),
            file: "x.rs".to_string(),
            line: Some(42),
            reason: "test".to_string(),
            reason_detail: None,
        }],
        summary: EntryPointSummaryDto {
            total: 1,
            by_reason: BTreeMap::new(),
        },
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    // Verify snake_case field names are present.
    assert!(json.contains("\"by_reason\""), "by_reason must be snake_case in JSON");
    assert!(json.contains("\"reason_detail\""), "reason_detail must be in JSON");
    assert!(!json.contains("byReason"), "camelCase must not appear in JSON");
}
