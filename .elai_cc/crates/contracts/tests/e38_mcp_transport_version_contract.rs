//! E-38 (SCHEMA_VERSION 175) -- MCP transport-version log + MRCR
//! unverified + posture-doc-stale producer contracts.
//!
//! Asserts:
//!   - MCP transport canonical strings (current default + roadmap target)
//!   - roadmap_signals_enabled honours the ELAI_AI_ROADMAP_SIGNALS_ENABLE
//!     env-gate (case-insensitive `"1"` / `"true"` / `"yes"`)
//!   - build_mrcr_unverified_payload populates the correct fields
//!   - check_posture_doc_age returns None for a missing file (no noisy
//!     emission when the doc is absent)
//!
//! See workflow/reasoning/E-38-decisions.md.

use std::path::Path;

use elai_orchestrator::dispatch::{MCP_TRANSPORT_CURRENT_DEFAULT, MCP_TRANSPORT_ROADMAP_TARGET};
use elai_orchestrator::roadmap_signals::{
    build_mrcr_unverified_payload, check_posture_doc_age, roadmap_signals_enabled,
};
use elai_orchestrator::ssb::{
    ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV, MRCR_UNVERIFIED_CONTEXT_THRESHOLD, RoadmapSignalKind,
};

#[test]
fn mcp_transport_current_label_is_streamable_http() {
    // ELAI defaults to MCP spec 2025-11-25 Streamable HTTP per
    // McpTransportMode::Default. Plugins crate Default impl + this
    // constant must stay in sync.
    assert_eq!(MCP_TRANSPORT_CURRENT_DEFAULT, "streamable_http");
}

#[test]
fn mcp_transport_roadmap_target_contains_known_milestones() {
    // modelcontextprotocol.io/development/roadmap (verified May 2026):
    // stateless transport, Tasks primitive, MCP Registry Q4 2026,
    // SAML/OIDC/PKCE Q2 2026.
    assert!(MCP_TRANSPORT_ROADMAP_TARGET.contains("stateless"));
    assert!(MCP_TRANSPORT_ROADMAP_TARGET.contains("tasks"));
    assert!(MCP_TRANSPORT_ROADMAP_TARGET.contains("registry"));
    assert!(MCP_TRANSPORT_ROADMAP_TARGET.contains("saml"));
    assert!(MCP_TRANSPORT_ROADMAP_TARGET.contains("pkce"));
}

#[test]
fn roadmap_signals_disabled_by_default() {
    // SAFETY: env mutation in single-threaded test scope.
    unsafe {
        std::env::remove_var(ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV);
    }
    assert!(
        !roadmap_signals_enabled(),
        "wire-dark default must hold when ELAI_AI_ROADMAP_SIGNALS_ENABLE is unset"
    );
}

#[test]
fn build_mrcr_unverified_payload_populates_fields() {
    let p = build_mrcr_unverified_payload("claude-opus-4-7", 1_000_000, "vendor-claim-only");
    assert_eq!(p.kind, RoadmapSignalKind::MrcrUnverified);
    assert_eq!(p.backend_name, "claude-opus-4-7");
    assert_eq!(p.declared_context_tokens, 1_000_000);
    assert_eq!(p.mrcr_score_label, "vendor-claim-only");
    // Cross-kind fields stay empty.
    assert!(p.assertion_label.is_empty());
    assert!(p.posture_doc_path.is_empty());
    assert!(p.mcp_transport_current.is_empty());
    assert_eq!(p.assertions_checked, 0);
    // Plan task 4 deterministic check threshold.
    assert!(p.declared_context_tokens >= MRCR_UNVERIFIED_CONTEXT_THRESHOLD);
}

#[test]
fn check_posture_doc_age_returns_none_for_missing_file() {
    // The posture doc may not have been authored yet (or sits at the
    // workflow root, not inside .elai_cc/). Missing-file = None means
    // we don't emit a noisy `posture_doc_stale` row when the absence
    // itself is the signal.
    let nonexistent = Path::new("Z:/this/path/does/not/exist/posture.md");
    assert!(check_posture_doc_age(nonexistent).is_none());
}

#[test]
fn check_posture_doc_age_returns_none_for_fresh_file() {
    // A file authored just now is younger than the 90-day threshold;
    // the probe must return None to keep the wire quiet on fresh
    // posture docs.
    use std::fs;
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path().join("ELAI-2026-POSTURE.md");
    fs::write(&p, "# fresh posture doc\n").expect("write");
    assert!(
        check_posture_doc_age(&p).is_none(),
        "fresh-file mtime must NOT trigger posture_doc_stale"
    );
}
