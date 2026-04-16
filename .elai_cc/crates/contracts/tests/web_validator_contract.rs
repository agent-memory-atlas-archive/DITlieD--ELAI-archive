// Contract tests for elai-web-validator public API + the cross-crate
// PreviewAvailablePayload contract emitted into the SSB by the orchestrator.

use std::path::{Path, PathBuf};

use elai_orchestrator::ssb::{PreviewAvailablePayload, SsbActionType, SsbPayload};
use elai_web_validator::{
    check_dev_server, promote_slot, start_dev_server, DevServerHandle, DevServerStatus,
    WebBuildResult, WebValidationTier, WebValidatorError, DEV_SERVER_CHECK_TIMEOUT_SECS,
    DEV_SERVER_POLL_INTERVAL_MS, DEV_SERVER_PROBE_HOST, DEV_SERVER_START_TIMEOUT_SECS,
    EXPECTED_HTML_CLOSING_TAG, EXPECTED_META_VIEWPORT_FRAGMENT, MAX_STDERR_BYTES,
    WEB_BUILD_TIMEOUT_SECS,
};

// ---------------------------------------------------------------------------
// Public constants — bumping these is a SemVer event. Tests pin the values.
// ---------------------------------------------------------------------------

#[test]
fn named_constants_are_stable() {
    assert_eq!(DEV_SERVER_START_TIMEOUT_SECS, 15);
    assert_eq!(DEV_SERVER_CHECK_TIMEOUT_SECS, 5);
    assert_eq!(DEV_SERVER_POLL_INTERVAL_MS, 250);
    assert_eq!(WEB_BUILD_TIMEOUT_SECS, 180);
    assert_eq!(MAX_STDERR_BYTES, 65536);
    assert_eq!(EXPECTED_HTML_CLOSING_TAG, "</html>");
    assert_eq!(EXPECTED_META_VIEWPORT_FRAGMENT, "name=\"viewport\"");
    assert_eq!(DEV_SERVER_PROBE_HOST, "127.0.0.1");
}

// ---------------------------------------------------------------------------
// Result type field contracts — guard against silent renames.
// ---------------------------------------------------------------------------

#[test]
fn dev_server_status_field_contract() {
    let s = DevServerStatus {
        ok: true,
        status_code: 200,
        has_html_close: true,
        has_meta_viewport: true,
        stderr_errors: vec!["e".into()],
    };
    assert!(s.ok);
    assert_eq!(s.status_code, 200);
    assert!(s.has_html_close);
    assert!(s.has_meta_viewport);
    assert_eq!(s.stderr_errors, vec!["e".to_owned()]);
}

#[test]
fn web_build_result_field_contract() {
    let r = WebBuildResult {
        success: true,
        exit_code: 0,
        output_dir: PathBuf::from("dist"),
        stderr_errors: Vec::new(),
    };
    assert!(r.success);
    assert_eq!(r.exit_code, 0);
    assert_eq!(r.output_dir, Path::new("dist"));
    assert!(r.stderr_errors.is_empty());
}

#[test]
fn web_validation_tier_variants_present() {
    let tiers = [
        WebValidationTier::WebBuild,
        WebValidationTier::WebDevServer,
        WebValidationTier::WebLint,
    ];
    assert_eq!(tiers.len(), 3);
}

// ---------------------------------------------------------------------------
// Error variant Display contracts.
// ---------------------------------------------------------------------------

#[test]
fn invalid_port_error_display_mentions_port() {
    let err = WebValidatorError::InvalidPort(0);
    assert!(format!("{err}").contains("port"));
}

#[test]
fn timeout_error_display_includes_seconds() {
    let err = WebValidatorError::Timeout {
        what: "build".into(),
        after_secs: 42,
    };
    assert!(format!("{err}").contains("42"));
    assert!(format!("{err}").contains("build"));
}

#[test]
fn premature_exit_error_distinguishes_from_timeout() {
    // PrematureExit must be a distinct variant so callers can render the
    // correct failure mode (the dev server died, it didn't fail to respond).
    let err = WebValidatorError::PrematureExit {
        command: "npm run dev".into(),
        port: 5173,
    };
    let s = format!("{err}");
    assert!(s.contains("exited"));
    assert!(s.contains("5173"));
    assert!(!matches!(err, WebValidatorError::Timeout { .. }));
}

#[test]
fn output_dir_missing_display_includes_path() {
    let err = WebValidatorError::OutputDirMissing(PathBuf::from("/dist"));
    assert!(format!("{err}").contains("dist"));
}

#[test]
fn symlink_failed_display_includes_reason() {
    let err = WebValidatorError::SymlinkFailed {
        dst: PathBuf::from("/live"),
        reason: "permission denied".into(),
    };
    let s = format!("{err}");
    assert!(s.contains("permission denied"));
}

// ---------------------------------------------------------------------------
// Behavioral contracts — cheap, deterministic, no external processes.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn check_dev_server_rejects_port_zero_with_invalid_port_error() {
    let res = check_dev_server(0).await;
    assert!(matches!(res, Err(WebValidatorError::InvalidPort(0))));
}

#[tokio::test]
async fn start_dev_server_rejects_port_zero_with_invalid_port_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let res = start_dev_server(dir.path(), "true", 0).await;
    assert!(matches!(res, Err(WebValidatorError::InvalidPort(0))));
}

#[tokio::test]
async fn start_dev_server_rejects_empty_command_with_empty_command_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let res = start_dev_server(dir.path(), "   ", 8080).await;
    assert!(matches!(res, Err(WebValidatorError::EmptyCommand)));
}

#[test]
fn promote_slot_errors_when_draft_missing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let draft = dir.path().join("nope");
    let live = dir.path().join("live");
    let err = promote_slot(&draft, &live).expect_err("missing draft must error");
    assert!(matches!(err, WebValidatorError::IoError(_)));
}

#[test]
fn dev_server_handle_is_opaque_to_external_callers() {
    // Compile-time guarantee: DevServerHandle's pid/dir/child fields are private.
    // External callers may only observe via accessor methods.
    fn assert_accessors_compile(h: &DevServerHandle) {
        let _: u32 = h.pid();
        let _: u16 = h.port();
        let _: &Path = h.dir();
    }
    let _f = assert_accessors_compile;
}

// ---------------------------------------------------------------------------
// PreviewAvailablePayload spec contract (orchestrator emits, dashboard reads).
// Spec (mvp-implementation-order.txt WEB.3 line 487):
//   payload = { url: "http://localhost:{port}", step_id, slot: "live" }
// ---------------------------------------------------------------------------

#[test]
fn preview_available_payload_has_required_spec_fields() {
    // If any of `url`, `step_id`, or `slot` is renamed/removed, this fails to compile.
    let p = PreviewAvailablePayload {
        url: "http://localhost:5173".into(),
        step_id: "WEB-3-baby-7".into(),
        slot: "live".into(),
        agent_id: "previewer".into(),
        build_success: true,
        dev_server_ok: true,
    };
    assert!(p.url.starts_with("http://"));
    assert_eq!(p.slot, "live");
    assert!(!p.step_id.is_empty());
}

#[test]
fn preview_available_action_type_matches_payload_variant() {
    let p = SsbPayload::PreviewAvailable(PreviewAvailablePayload {
        url: "http://localhost:8080".into(),
        step_id: "s".into(),
        slot: "draft".into(),
        agent_id: "a".into(),
        build_success: true,
        dev_server_ok: false,
    });
    assert_eq!(p.action_type(), SsbActionType::PreviewAvailable);
}

#[test]
fn preview_available_round_trips_msgpack() {
    let original = PreviewAvailablePayload {
        url: "http://localhost:1234".into(),
        step_id: "step-x".into(),
        slot: "previous".into(),
        agent_id: "agent-y".into(),
        build_success: false,
        dev_server_ok: false,
    };
    let bytes = rmp_serde::to_vec_named(&original).expect("serialize");
    let decoded: PreviewAvailablePayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert_eq!(decoded, original);
}

#[test]
fn preview_available_serializes_to_json_with_spec_field_names() {
    let p = PreviewAvailablePayload {
        url: "http://localhost:5173".into(),
        step_id: "step-1".into(),
        slot: "live".into(),
        agent_id: "a".into(),
        build_success: true,
        dev_server_ok: true,
    };
    let json = serde_json::to_string(&p).expect("serialize");
    // Spec-mandated field names must appear verbatim for dashboard consumer.
    assert!(json.contains("\"url\""));
    assert!(json.contains("\"step_id\""));
    assert!(json.contains("\"slot\""));
    assert!(json.contains("http://localhost:5173"));
}
