// E-52 Tier-B — LSP integration contract test.
//
// Locks the public surface added by the E-52 Tier-B ship:
//   * elai_orchestrator::symbol_graph::{LspMode, oracle_lookup_with_mode}
//     (Probe + FullHandshake modes; Probe is back-compat default).
//   * elai_orchestrator::symbol_graph::lsp_oracle::{parse_content_length,
//     read_frame, write_frame} — hand-rolled LSP base-protocol v3.17
//     Content-Length framing (no new crate deps; tokio io-util only).
//   * elai_knowledge::diagnostic::DiagnosticProbeRunStatus (DRIFT-14
//     closure: sibling enum surfacing Completed / TimedOut / ToolMissing
//     without breaking the locked Tier-A
//     diagnostic_envelope_contract.rs).
//   * 4 new SSB action types round-trip via msgpack:
//       - SsbActionType::DiagnosticProbeDispatched
//       - SsbActionType::DiagnosticProbeCompleted
//       - SsbActionType::DiagnosticProbeBlock
//       - SsbActionType::ScipRefreshCompleted
//   * SCHEMA_VERSION_PUBLIC floor at 139.
//   * Composition with the E-40 LspOracleConsulted variant (E-52
//     Tier-B REUSES the E-40 variant for oracle invocations; the 4 new
//     variants are exclusively about the diagnostic probe + SCIP refresh
//     surface).
//   * WIRE-DARK avoidance: scip_cmd `lsp-probe` sub-verb is the
//     production caller anchor for oracle_lookup_with_mode.
//
// Design source: LSP base protocol v3.17 (Microsoft-canonical;
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
// Cite-discipline: spec text is Microsoft-published, GROUNDED without
// arXiv escalation; the contract asserts wire-shape + framing parser
// behaviour + SSB round-trip + back-compat for legacy payloads, all
// without requiring a real rust-analyzer / pylsp on the host.

use elai_knowledge::diagnostic::{
    DiagnosticProbeOutcome, DiagnosticProbeRunStatus, DiagnosticSeverity, ProbeKind,
};
use elai_orchestrator::ssb::{
    DiagnosticProbeBlockPayload, DiagnosticProbeCompletedPayload,
    DiagnosticProbeDispatchedPayload, ScipRefreshCompletedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::symbol_graph::{
    oracle_lookup, oracle_lookup_with_mode, LspMode, LspOracleError, OracleAnswer, OracleKind,
};

// ---------------------------------------------------------------------------
// (F1) SCHEMA_VERSION_PUBLIC floor: E-52 Tier-B atomic-bumped to 139.
// ---------------------------------------------------------------------------

#[test]
fn e52_f1_schema_version_floor_is_139_or_higher() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 139,
        "E-52 Tier-B atomic-bump puts SCHEMA_VERSION_PUBLIC at >= 139, found {SCHEMA_VERSION_PUBLIC}"
    );
}

// ---------------------------------------------------------------------------
// (F2) DiagnosticProbeRunStatus sibling enum — DRIFT-14 closure.
// ---------------------------------------------------------------------------

#[test]
fn e52_f2_run_status_three_variants_completed_default() {
    // Default must be Completed (preserves Tier-A semantics: counters are
    // authoritative when run_status is unset on legacy payloads).
    assert_eq!(
        DiagnosticProbeRunStatus::default(),
        DiagnosticProbeRunStatus::Completed
    );
    // Only Completed is authoritative; TimedOut + ToolMissing are soft-
    // fail per the auditor anti-rubber-stamp rule.
    assert!(DiagnosticProbeRunStatus::Completed.is_authoritative());
    assert!(!DiagnosticProbeRunStatus::TimedOut.is_authoritative());
    assert!(!DiagnosticProbeRunStatus::ToolMissing.is_authoritative());
    // Wire tags are snake_case (LSP-canonical shorthand).
    assert_eq!(DiagnosticProbeRunStatus::Completed.as_str(), "completed");
    assert_eq!(DiagnosticProbeRunStatus::TimedOut.as_str(), "timed_out");
    assert_eq!(DiagnosticProbeRunStatus::ToolMissing.as_str(), "tool_missing");
}

#[test]
fn e52_f2b_run_status_serde_round_trip() {
    for status in [
        DiagnosticProbeRunStatus::Completed,
        DiagnosticProbeRunStatus::TimedOut,
        DiagnosticProbeRunStatus::ToolMissing,
    ] {
        let json = serde_json::to_string(&status).expect("serialize ok");
        let back: DiagnosticProbeRunStatus =
            serde_json::from_str(&json).expect("deserialize ok");
        assert_eq!(back, status);
    }
}

#[test]
fn e52_f2c_outcome_back_compat_legacy_payload_defaults_completed() {
    // Pre-E-52 Tier-B producers emit DiagnosticProbeOutcome WITHOUT a
    // `run_status` field. The serde(default) attribute on the new field
    // must keep these payloads parseable.
    let legacy_json = r#"{
        "kind": "pyright",
        "error_count": 0,
        "warning_count": 1,
        "info_count": 0,
        "hint_count": 0,
        "duration_ms": 250,
        "diagnostics": []
    }"#;
    let outcome: DiagnosticProbeOutcome =
        serde_json::from_str(legacy_json).expect("legacy payload must parse");
    assert_eq!(outcome.run_status, DiagnosticProbeRunStatus::Completed);
    assert!(outcome.is_clean());
    assert!(outcome.run_status.is_authoritative());
    assert_eq!(outcome.kind, ProbeKind::Pyright);
}

#[test]
fn e52_f2d_outcome_explicit_timeout_round_trip() {
    let outcome = DiagnosticProbeOutcome {
        kind: ProbeKind::RustAnalyzer,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        hint_count: 0,
        duration_ms: 30_000,
        diagnostics: vec![],
        run_status: DiagnosticProbeRunStatus::TimedOut,
    };
    let json = serde_json::to_string(&outcome).expect("serialize ok");
    let back: DiagnosticProbeOutcome = serde_json::from_str(&json).expect("deserialize ok");
    assert_eq!(back, outcome);
    // TimedOut counters MUST NOT be treated as a clean pass.
    assert!(back.is_clean()); // is_clean only checks error_count
    assert!(!back.run_status.is_authoritative()); // soft-fail per auditor
}

#[test]
fn e52_f2e_diagnostic_severity_uses_canonical_information_name() {
    // DRIFT-13 closure: shipped enum uses Information (LSP 1.x
    // canonical), not Info shorthand. New code MUST match.
    assert_eq!(DiagnosticSeverity::Information.as_str(), "information");
    assert_eq!(DiagnosticSeverity::Information.lsp_severity(), 3);
}

// ---------------------------------------------------------------------------
// (F3) LSP JSON-RPC frame parser.
// ---------------------------------------------------------------------------

#[test]
fn e52_f3_content_length_parser_accepts_canonical_header() {
    // The lsp_oracle module exposes parse_content_length as pub(crate)
    // so this contract test reaches it via the public read_frame /
    // write_frame round-trip (functionally equivalent — read_frame
    // calls parse_content_length internally).
    //
    // The unit tests inside lsp_oracle::tests cover parse_content_length
    // directly; this contract test covers the higher-level round-trip
    // via the public surface.
    //
    // For the framing parser we exercise the public path: an in-process
    // duplex round-trip of a canonical Content-Length-framed JSON-RPC
    // initialize request.
    use elai_orchestrator::symbol_graph::lsp_oracle::{read_frame, write_frame};
    use tokio::io::duplex;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt build");
    rt.block_on(async {
        let (mut a, mut b) = duplex(8192);
        let initialize = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": 12345,
                "rootUri": "file:///tmp/elai-test",
                "capabilities": {}
            }
        });
        write_frame(&mut a, &initialize).await.expect("write ok");
        let back = read_frame(&mut b).await.expect("read ok");
        assert_eq!(back, initialize);
    });
}

#[test]
fn e52_f3b_framing_parser_rejects_malformed_input() {
    use elai_orchestrator::symbol_graph::lsp_oracle::read_frame;
    use tokio::io::{duplex, AsyncWriteExt};
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt build");
    rt.block_on(async {
        // Missing Content-Length header.
        let (mut a, mut b) = duplex(1024);
        a.write_all(b"Content-Type: foo\r\n\r\n{}")
            .await
            .expect("write ok");
        drop(a);
        let err = read_frame(&mut b).await.expect_err("must error");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    });
    rt.block_on(async {
        // Truncated body (claims 100, supplies 3).
        let (mut a, mut b) = duplex(1024);
        a.write_all(b"Content-Length: 100\r\n\r\nABC")
            .await
            .expect("write ok");
        drop(a);
        let err = read_frame(&mut b).await.expect_err("must error");
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    });
}

// ---------------------------------------------------------------------------
// (F4) LspMode — Probe is default; FullHandshake is opt-in.
// ---------------------------------------------------------------------------

#[test]
fn e52_f4_lsp_mode_default_is_probe() {
    // Back-compat: any caller that does not opt in to FullHandshake
    // stays on the E-40 --version smoke probe.
    assert_eq!(LspMode::default(), LspMode::Probe);
    assert_eq!(LspMode::Probe.as_str(), "probe");
    assert_eq!(LspMode::FullHandshake.as_str(), "full_handshake");
}

#[test]
fn e52_f4b_lsp_mode_probe_back_compat_runpod_pending_shape() {
    // The E-40 contract pins the RUNPOD-PENDING synthetic-answer
    // shape: invoked=false, binary="". Probe mode MUST preserve this
    // when the toolchain is absent (CI lane).
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt build");
    rt.block_on(async {
        let answer = oracle_lookup_with_mode("std::option::Option", "rust", LspMode::Probe)
            .await;
        match answer {
            Ok(OracleAnswer { invoked: false, ref binary, .. }) => {
                assert!(binary.is_empty(), "synthetic answer must have empty binary");
            }
            Ok(OracleAnswer { invoked: true, .. }) => {
                // Toolchain happened to be present; that is OK on dev hosts.
            }
            Err(LspOracleError::Subprocess { .. }) => {
                // Toolchain found but subprocess errored; acceptable.
            }
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
    });
}

#[test]
fn e52_f4c_lsp_mode_full_handshake_honours_runpod_pending() {
    // FullHandshake mode MUST honour the same RUNPOD-PENDING contract
    // as Probe when the toolchain is absent.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt build");
    rt.block_on(async {
        let answer = oracle_lookup_with_mode(
            "std::option::Option",
            "rust",
            LspMode::FullHandshake,
        )
        .await;
        match answer {
            Ok(OracleAnswer { invoked: false, ref binary, .. }) => {
                assert!(binary.is_empty(), "synthetic answer must have empty binary");
            }
            Ok(OracleAnswer { invoked: true, .. }) => {
                // Real toolchain was on PATH and answered the handshake.
                // Acceptable on dev hosts with rust-analyzer installed.
            }
            Err(LspOracleError::Subprocess { .. }) => {
                // Toolchain found but handshake errored / timed out; acceptable.
            }
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
    });
}

#[test]
fn e52_f4d_oracle_lookup_default_preserves_e40_signature() {
    // The default-mode entry point (oracle_lookup without explicit mode)
    // MUST remain callable for E-40 callers that don't know about
    // LspMode. EmptySymbol contract from E-40 is preserved.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt build");
    rt.block_on(async {
        let result = oracle_lookup("", "rust").await;
        assert!(matches!(result, Err(LspOracleError::EmptySymbol)));
    });
}

#[test]
fn e52_f4e_oracle_kind_back_compat_unchanged() {
    // E-40 OracleKind taxonomy + slug aliases unchanged by E-52 Tier-B.
    assert_eq!(OracleKind::from_slug("rust"), Some(OracleKind::Rust));
    assert_eq!(OracleKind::from_slug("rs"), Some(OracleKind::Rust));
    assert_eq!(OracleKind::from_slug("python"), Some(OracleKind::Python));
    assert_eq!(OracleKind::from_slug("ts"), Some(OracleKind::TypeScript));
    assert_eq!(OracleKind::from_slug("tsx"), Some(OracleKind::TypeScript));
    assert_eq!(OracleKind::from_slug("go"), Some(OracleKind::Go));
    assert_eq!(OracleKind::from_slug("cobol"), None);
    assert_eq!(OracleKind::Rust.canonical_binary(), "rust-analyzer");
    assert_eq!(OracleKind::Python.canonical_binary(), "pylsp");
    assert_eq!(
        OracleKind::TypeScript.canonical_binary(),
        "typescript-language-server"
    );
    assert_eq!(OracleKind::Go.canonical_binary(), "gopls");
}

// ---------------------------------------------------------------------------
// (F5) 4 new SSB action types — discriminator-preserving round-trip.
// ---------------------------------------------------------------------------

#[test]
fn e52_f5_diagnostic_probe_dispatched_round_trip() {
    let payload = SsbPayload::DiagnosticProbeDispatched(DiagnosticProbeDispatchedPayload {
        probe_kind_hash: 0xE052_0001_0000_0000,
        language_hash: 0xE052_0001_0000_0001,
        file_count: 3,
        ts_ms: 1_700_000_139_001,
    });
    assert_eq!(payload.action_type(), SsbActionType::DiagnosticProbeDispatched);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode ok");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode ok");
    assert_eq!(back.action_type(), SsbActionType::DiagnosticProbeDispatched);
}

#[test]
fn e52_f5b_diagnostic_probe_completed_round_trip() {
    let payload = SsbPayload::DiagnosticProbeCompleted(DiagnosticProbeCompletedPayload {
        probe_kind_hash: 0xE052_0002_0000_0000,
        run_status_hash: 0xE052_0002_0000_0001,
        error_count: 0,
        warning_count: 2,
        duration_ms: 420,
        ts_ms: 1_700_000_139_002,
    });
    assert_eq!(payload.action_type(), SsbActionType::DiagnosticProbeCompleted);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode ok");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode ok");
    assert_eq!(back.action_type(), SsbActionType::DiagnosticProbeCompleted);
}

#[test]
fn e52_f5c_diagnostic_probe_block_round_trip() {
    let payload = SsbPayload::DiagnosticProbeBlock(DiagnosticProbeBlockPayload {
        probe_kind_hash: 0xE052_0003_0000_0000,
        error_count: 5,
        first_path_hash: 0xE052_0003_0000_0001,
        ts_ms: 1_700_000_139_003,
    });
    assert_eq!(payload.action_type(), SsbActionType::DiagnosticProbeBlock);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode ok");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode ok");
    assert_eq!(back.action_type(), SsbActionType::DiagnosticProbeBlock);
}

#[test]
fn e52_f5d_scip_refresh_completed_round_trip() {
    let payload = SsbPayload::ScipRefreshCompleted(ScipRefreshCompletedPayload {
        shard_count: 7,
        crate_count: 4,
        duration_ms: 12_500,
        ts_ms: 1_700_000_139_004,
    });
    assert_eq!(payload.action_type(), SsbActionType::ScipRefreshCompleted);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode ok");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode ok");
    assert_eq!(back.action_type(), SsbActionType::ScipRefreshCompleted);
}

#[test]
fn e52_f5e_all_four_wire_tags_present() {
    // Wire tags follow snake_case convention.
    assert_eq!(
        SsbActionType::DiagnosticProbeDispatched.as_sql_str(),
        "diagnostic_probe_dispatched"
    );
    assert_eq!(
        SsbActionType::DiagnosticProbeCompleted.as_sql_str(),
        "diagnostic_probe_completed"
    );
    assert_eq!(
        SsbActionType::DiagnosticProbeBlock.as_sql_str(),
        "diagnostic_probe_block"
    );
    assert_eq!(
        SsbActionType::ScipRefreshCompleted.as_sql_str(),
        "scip_refresh_completed"
    );
}

// ---------------------------------------------------------------------------
// (F6) Composition with E-40 LspOracleConsulted — reuse, not re-implement.
// ---------------------------------------------------------------------------

#[test]
fn e52_f6_e40_lsp_oracle_consulted_variant_still_resolvable() {
    // The E-52 Tier-B ship does NOT add a duplicate LSP-invocation
    // beacon — it reuses the E-40 LspOracleConsulted variant. The
    // 4 new variants are exclusively about diagnostic-probe + SCIP
    // refresh observables. This test pins that composition by
    // asserting the E-40 variant remains in the enum with its
    // canonical wire tag.
    assert_eq!(
        SsbActionType::LspOracleConsulted.as_sql_str(),
        "lsp_oracle_consulted"
    );
}
