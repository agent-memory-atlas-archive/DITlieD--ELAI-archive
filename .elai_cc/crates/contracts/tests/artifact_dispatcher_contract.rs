//! Contract test: B-35 Phase B — dispatcher tool name constants + wire struct serde.
//!
//! Verifies:
//!   (1) TOOL_* constants exist in ipc-types (compile-time assertions).
//!   (2) ListArtifactsRequest / SearchArtifactsRequest / IndexArtifactsRequest /
//!       RemoveArtifactsRequest roundtrip through serde_json.
//!   (3) SearchArtifactsRequest optional fields serialize correctly.
//!   (4) DEFAULT_ARTIFACT_SEARCH_LIMIT constant exists and is non-zero.
//!
//! No real ArtifactStore, no real dispatcher — purely compile + serde contract.

use elai_ipc_types::{
    IndexArtifactsRequest, ListArtifactsRequest, RemoveArtifactsRequest, SearchArtifactsRequest,
    DEFAULT_ARTIFACT_SEARCH_LIMIT, TOOL_INDEX_ARTIFACTS, TOOL_LIST_ARTIFACTS,
    TOOL_REMOVE_ARTIFACTS, TOOL_SEARCH_ARTIFACTS,
};

// ---------------------------------------------------------------------------
// (1) Tool constants exist and are snake_case strings.
// ---------------------------------------------------------------------------

#[test]
fn tool_constants_exist_and_are_snake_case() {
    assert_eq!(TOOL_LIST_ARTIFACTS, "list_artifacts");
    assert_eq!(TOOL_INDEX_ARTIFACTS, "index_artifacts");
    assert_eq!(TOOL_SEARCH_ARTIFACTS, "search_artifacts");
    assert_eq!(TOOL_REMOVE_ARTIFACTS, "remove_artifacts");
}

// ---------------------------------------------------------------------------
// (2) DEFAULT_ARTIFACT_SEARCH_LIMIT is non-zero.
// ---------------------------------------------------------------------------

#[test]
fn default_artifact_search_limit_nonzero() {
    assert!(
        DEFAULT_ARTIFACT_SEARCH_LIMIT > 0,
        "DEFAULT_ARTIFACT_SEARCH_LIMIT must be > 0"
    );
}

// ---------------------------------------------------------------------------
// (3) ListArtifactsRequest serde roundtrip.
// ---------------------------------------------------------------------------

#[test]
fn list_artifacts_request_serde_roundtrip() {
    let req = ListArtifactsRequest {
        db_path: "/tmp/context_myproject.db".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let decoded: ListArtifactsRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.db_path, decoded.db_path);
}

// ---------------------------------------------------------------------------
// (4) SearchArtifactsRequest serde roundtrip — all fields.
// ---------------------------------------------------------------------------

#[test]
fn search_artifacts_request_serde_roundtrip_all_fields() {
    let req = SearchArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        query: "find users table".to_string(),
        artifact_name: Some("db_schema".to_string()),
        limit: Some(5),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let decoded: SearchArtifactsRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.db_path, decoded.db_path);
    assert_eq!(req.query, decoded.query);
    assert_eq!(req.artifact_name, decoded.artifact_name);
    assert_eq!(req.limit, decoded.limit);
}

// ---------------------------------------------------------------------------
// (5) SearchArtifactsRequest serde roundtrip — optional fields absent.
// ---------------------------------------------------------------------------

#[test]
fn search_artifacts_request_serde_roundtrip_optional_absent() {
    let req = SearchArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        query: "get endpoints".to_string(),
        artifact_name: None,
        limit: None,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let decoded: SearchArtifactsRequest = serde_json::from_str(&json).expect("deserialize");
    assert!(decoded.artifact_name.is_none());
    assert!(decoded.limit.is_none());
    // When limit is None, callers should use DEFAULT_ARTIFACT_SEARCH_LIMIT.
    let effective_limit = decoded.limit.unwrap_or(DEFAULT_ARTIFACT_SEARCH_LIMIT);
    assert_eq!(effective_limit, DEFAULT_ARTIFACT_SEARCH_LIMIT);
}

// ---------------------------------------------------------------------------
// (6) IndexArtifactsRequest serde roundtrip.
// ---------------------------------------------------------------------------

#[test]
fn index_artifacts_request_serde_roundtrip() {
    let req = IndexArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        artifact_name: "openapi_spec".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let decoded: IndexArtifactsRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.db_path, decoded.db_path);
    assert_eq!(req.artifact_name, decoded.artifact_name);
}

// ---------------------------------------------------------------------------
// (7) RemoveArtifactsRequest serde roundtrip.
// ---------------------------------------------------------------------------

#[test]
fn remove_artifacts_request_serde_roundtrip() {
    let req = RemoveArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        artifact_name: "k8s_manifests".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let decoded: RemoveArtifactsRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(req.db_path, decoded.db_path);
    assert_eq!(req.artifact_name, decoded.artifact_name);
}

// ---------------------------------------------------------------------------
// Postcard wire-format roundtrip tests (one per struct).
//
// The IPC dispatcher uses postcard encoding for all tool arguments
// (confirmed at production_agent_rpc.rs: postcard::from_bytes calls).
// serde_json and postcard have divergent encoding for Option<usize> and usize;
// these tests catch encoding drift that the serde_json tests above would miss.
// ---------------------------------------------------------------------------

#[test]
fn list_artifacts_request_postcard_roundtrip() {
    let req = ListArtifactsRequest {
        db_path: "/tmp/context_myproject.db".to_string(),
    };
    let encoded = postcard::to_allocvec(&req).expect("postcard serialize");
    let decoded: ListArtifactsRequest = postcard::from_bytes(&encoded).expect("postcard deserialize");
    assert_eq!(req.db_path, decoded.db_path);
}

#[test]
fn search_artifacts_request_postcard_roundtrip_all_fields() {
    let req = SearchArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        query: "find users table".to_string(),
        artifact_name: Some("db_schema".to_string()),
        limit: Some(5),
    };
    let encoded = postcard::to_allocvec(&req).expect("postcard serialize");
    let decoded: SearchArtifactsRequest = postcard::from_bytes(&encoded).expect("postcard deserialize");
    assert_eq!(req.db_path, decoded.db_path);
    assert_eq!(req.query, decoded.query);
    assert_eq!(req.artifact_name, decoded.artifact_name);
    assert_eq!(req.limit, decoded.limit);
}

#[test]
fn search_artifacts_request_postcard_roundtrip_optional_absent() {
    let req = SearchArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        query: "get endpoints".to_string(),
        artifact_name: None,
        limit: None,
    };
    let encoded = postcard::to_allocvec(&req).expect("postcard serialize");
    let decoded: SearchArtifactsRequest = postcard::from_bytes(&encoded).expect("postcard deserialize");
    assert!(decoded.artifact_name.is_none());
    assert!(decoded.limit.is_none());
}

#[test]
fn index_artifacts_request_postcard_roundtrip() {
    let req = IndexArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        artifact_name: "openapi_spec".to_string(),
    };
    let encoded = postcard::to_allocvec(&req).expect("postcard serialize");
    let decoded: IndexArtifactsRequest = postcard::from_bytes(&encoded).expect("postcard deserialize");
    assert_eq!(req.db_path, decoded.db_path);
    assert_eq!(req.artifact_name, decoded.artifact_name);
}

#[test]
fn remove_artifacts_request_postcard_roundtrip() {
    let req = RemoveArtifactsRequest {
        db_path: "/tmp/context_proj.db".to_string(),
        artifact_name: "k8s_manifests".to_string(),
    };
    let encoded = postcard::to_allocvec(&req).expect("postcard serialize");
    let decoded: RemoveArtifactsRequest = postcard::from_bytes(&encoded).expect("postcard deserialize");
    assert_eq!(req.db_path, decoded.db_path);
    assert_eq!(req.artifact_name, decoded.artifact_name);
}
