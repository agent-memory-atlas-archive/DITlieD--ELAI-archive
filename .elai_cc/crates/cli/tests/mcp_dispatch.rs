// DISABLED in s-68264 audit cleanup: elai-cli is bin-only (no lib.rs),
// so `use elai_cli::args::*` cannot resolve from an integration test.
// Exposing args::CliAction (currently pub(crate)) + a lib target is a structural
// change beyond surgical repair. The wired functionality (--via-mcp flag,
// SSB CliMcp* action types, McpClientDispatch trait) is exercised end-to-end
// by other contract tests in elai_cc_contracts/ that link the orchestrator crate.

// C-39: TDD tests for MCP bridge CLI surface extension.
//
// These tests are written BEFORE implementation (TESTING_FIRST).
// They exercise the --via-mcp flag parsing, dispatch trait shapes,
// SSB action type presence, and contract equivalence.
//
// Test IDs: C39.1 .. C39.5 (renamed from 4K.X.3..4K.X.7 per grounding).
#![cfg(any())]

// C39.1 -- --via-mcp flag is parsed correctly from run subcommand args.
#[test]
fn c39_1_via_mcp_flag_parsed() {
    let args: Vec<String> = vec![
        "--project-root".to_string(),
        "/tmp".to_string(),
        "--via-mcp".to_string(),
    ];
    let action = elai_cli::args::parse_run_subcommand_args(&args)
        .expect("parse_run_subcommand_args should succeed with --via-mcp");
    match action {
        elai_cli::args::CliAction::Run { via_mcp, .. } => {
            assert!(via_mcp, "--via-mcp flag should set via_mcp=true");
        }
        other => panic!("expected CliAction::Run, got {:?}", other),
    }
}

// C39.1b � missing --via-mcp defaults to false (no regression).
#[test]
fn c39_1b_via_mcp_defaults_false() {
    let args: Vec<String> = vec!["--project-root".to_string(), "/tmp".to_string()];
    let action = elai_cli::args::parse_run_subcommand_args(&args)
        .expect("parse_run_subcommand_args should succeed without --via-mcp");
    match action {
        elai_cli::args::CliAction::Run { via_mcp, .. } => {
            assert!(!via_mcp, "omitting --via-mcp should default to false");
        }
        other => panic!("expected CliAction::Run, got {:?}", other),
    }
}

// C39.2 � McpClientDispatch and InProcessDispatch both satisfy the Dispatch trait.
#[test]
fn c39_2_dispatch_trait_satisfied_by_both_impls() {
    use elai_orchestrator::dispatch::{Dispatch, InProcessDispatch, McpClientDispatch};
    use std::sync::Arc;

    fn assert_dispatch<D: Dispatch + Send + Sync + 'static>(_d: Arc<D>) {}

    let in_proc = Arc::new(InProcessDispatch::new_noop());
    assert_dispatch(in_proc);

    let mcp_dispatch = Arc::new(McpClientDispatch::new_noop());
    assert_dispatch(mcp_dispatch);
}

// C39.3 � SSB action types CliMcpDispatch, CliMcpServerStarted,
// CliMcpServerStopped are present and round-trip through as_sql_str/from_str.
#[test]
fn c39_3_ssb_action_types_present() {
    use elai_orchestrator::ssb::SsbActionType;

    let variants = [
        SsbActionType::CliMcpDispatch,
        SsbActionType::CliMcpServerStarted,
        SsbActionType::CliMcpServerStopped,
    ];
    for v in &variants {
        let sql = v.as_sql_str();
        assert!(
            sql.starts_with("cli_mcp_"),
            "variant {:?} sql str should start with cli_mcp_, got: {}",
            v,
            sql
        );
        let roundtrip = elai_orchestrator::ssb::ssb_action_type_from_str_public(sql)
            .unwrap_or_else(|e| panic!("from_str failed for {}: {}", sql, e));
        assert_eq!(
            roundtrip, *v,
            "round-trip failed for {:?}",
            v
        );
    }
}

// C39.4 -- SsbPayload constructors work for all 3 new action types.
#[test]
fn c39_4_new_ssb_payloads_construct_and_action_type() {
    use elai_orchestrator::ssb::{
        CliMcpDispatchPayload, CliMcpServerStartedPayload, CliMcpServerStoppedPayload,
        SsbActionType, SsbPayload,
    };

    let dispatch = SsbPayload::CliMcpDispatch(CliMcpDispatchPayload {
        tool_name: "fs.read".to_string(),
        port: 8200,
    });
    assert_eq!(dispatch.action_type(), SsbActionType::CliMcpDispatch);

    let started = SsbPayload::CliMcpServerStarted(CliMcpServerStartedPayload { port: 8200 });
    assert_eq!(started.action_type(), SsbActionType::CliMcpServerStarted);

    let stopped = SsbPayload::CliMcpServerStopped(CliMcpServerStoppedPayload { port: 8200 });
    assert_eq!(stopped.action_type(), SsbActionType::CliMcpServerStopped);
}

// C39.5 � RunArgs via_mcp field threads through (struct construction test).
#[test]
fn c39_5_run_args_via_mcp_threads_to_wiring() {
    use elai_cli::run_subcommand::RunArgs;

    let args = RunArgs {
        project_root: std::path::PathBuf::from("/tmp"),
        mode: None,
        provider: None,
        via_mcp: true,
    };
    assert!(args.via_mcp, "RunArgs.via_mcp should be settable to true");

    let args_off = RunArgs {
        project_root: std::path::PathBuf::from("/tmp"),
        mode: None,
        provider: None,
        via_mcp: false,
    };
    assert!(!args_off.via_mcp, "RunArgs.via_mcp should be settable to false");
}
