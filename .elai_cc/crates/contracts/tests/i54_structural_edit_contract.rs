//! I-54 (Track 20 Wave 3) — agent-callable structural (AST-aware) edit tool contract.
//!
//! Black-box done-gate for the 3-arm search/preview/apply structural-edit tool
//! over the SHIPPED tree-sitter grammars (one version source via
//! `elai_knowledge::TreeSitterAstEvaluator`; ast-grep dep BANNED, SLC-3). Pins:
//!   (C1) SSB bijection: StructuralEditApplied's 5 wire tags round-trip
//!        from_str <-> as_sql_str; SCHEMA_VERSION_PUBLIC >= 260.
//!   (C2) REAL-DB production path + STRUCTURAL change: drive
//!        translate_tool_request_for_test (the live agent dispatch chokepoint)
//!        preview -> apply on a TempDir Rust file; assert the resulting bytes are
//!        the AST-rewrite (a function renamed via a tree-sitter query, not a
//!        text/regex swap), and the StructuralEditApplied(Applied) row lands.
//!   (C3) STALENESS: preview, mutate the file, then apply -> the apply is REFUSED
//!        (no bytes written, the file is unchanged) and a
//!        StructuralEditApplied(RefusedStale) row lands (load-bearing R1).
//!   (C4) GATE: an apply whose target escapes the worktree root is REFUSED by the
//!        REUSED PreToolUse edit gates (enforce_fs_blast_radius FsEscape) BEFORE
//!        any byte lands -- the structural apply rides the same gates as TOOL_EDIT.
//!
//! Wiring: the tool is reachable ONLY via the production agent dispatch
//! (translate_tool_request); these contract callers drive that exact entrypoint
//! through the public test helper (a contracts-only caller does NOT satisfy R7,
//! but the helper forwards into the real translate_tool_request match — the same
//! code path a live worker hits).
//!
//! Files under test:
//!   - crates/orchestrator/src/structural_edit.rs (search/preview/apply + staleness)
//!   - crates/orchestrator/src/production_agent_rpc.rs (structural_* dispatch arms,
//!     the REUSED enforce_taint_sink_gate + enforce_fs_blast_radius + dispatch_edit)
//!   - crates/knowledge/src/ast_adapter.rs (AstMatch byte-span extension)
//!   - crates/orchestrator/src/ssb/{action_type,payloads,schema}.rs (StructuralEditApplied)

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::tool_dispatcher::ToolDispatcher;

use elai_ipc_types::{TOOL_STRUCTURAL_APPLY, TOOL_STRUCTURAL_PREVIEW, TOOL_STRUCTURAL_SEARCH};
use elai_orchestrator::production_agent_rpc::translate_tool_request_for_test;
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    StructuralEditAppliedKind, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use elai_tools::dispatcher::InProcessToolDispatcher;
use elai_tools::tiered_validation::{
    DefaultTieredValidator, DeterminismCommandRunner, IntegrationCommandRunner,
    IntegrationRunOutcome, SyntaxCommandRunner, TieredValidator,
};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tempfile::TempDir;

const PROJECT_ROOT_ENV: &str = "ELAI_PROJECT_ROOT";

// Serialize the env-mutating REAL-DB tests (C2/C3/C4) to prevent a process-global
// ELAI_PROJECT_ROOT race under the default parallel runner. Plain std::sync::Mutex
// (mirrors i52_checkpoint_tool_contract.rs ENV_MUTEX idiom; no external dep).
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i54-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

// A real ToolDispatcher whose dispatch_edit actually performs the 3-tier byte
// write (the structural apply arm routes its rewrites through this). Always-pass
// validators -- the structural apply does not invoke validation, only edit.
fn ok_outcome() -> IntegrationRunOutcome {
    IntegrationRunOutcome {
        passed: true,
        rules_fired: 0,
        stderr_snippet: None,
    }
}

struct AlwaysOkDeterminism;
impl DeterminismCommandRunner for AlwaysOkDeterminism {
    fn run_determinism<'a>(
        &'a self,
        _project_dir: &'a Path,
    ) -> Pin<Box<dyn std::future::Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async { ok_outcome() })
    }
}
struct AlwaysOkSyntax;
impl SyntaxCommandRunner for AlwaysOkSyntax {
    fn run_syntax<'a>(
        &'a self,
        _language: &'a str,
        _project_dir: &'a Path,
        _changed_files: &'a [PathBuf],
    ) -> Pin<Box<dyn std::future::Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async { ok_outcome() })
    }
}
struct AlwaysOkIntegration;
impl IntegrationCommandRunner for AlwaysOkIntegration {
    fn run_integration<'a>(
        &'a self,
        _level_label: &'a str,
        _project_dir: &'a Path,
    ) -> Pin<Box<dyn std::future::Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async { ok_outcome() })
    }
}

fn real_dispatcher() -> InProcessToolDispatcher {
    let v = DefaultTieredValidator::new(
        Arc::new(AlwaysOkDeterminism),
        Arc::new(AlwaysOkSyntax),
        Arc::new(AlwaysOkIntegration),
    );
    InProcessToolDispatcher::new(Arc::new(v) as Arc<dyn TieredValidator>)
}

// ---------------------------------------------------------------------------
// (C1) SSB bijection: the 5 StructuralEditApplied wire tags + SCHEMA floor.
// ---------------------------------------------------------------------------

#[test]
fn i54_c1_structural_edit_applied_bijection() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 260,
        "I-54 requires SCHEMA_VERSION >= 260, got {SCHEMA_VERSION_PUBLIC}"
    );

    let cases = [
        ("structural_edit_matched", StructuralEditAppliedKind::Matched),
        (
            "structural_edit_previewed",
            StructuralEditAppliedKind::Previewed,
        ),
        ("structural_edit_applied", StructuralEditAppliedKind::Applied),
        (
            "structural_edit_refused_stale",
            StructuralEditAppliedKind::RefusedStale,
        ),
        (
            "structural_edit_parse_failed",
            StructuralEditAppliedKind::ParseFailed,
        ),
    ];
    for (tag, kind) in cases {
        let at = ssb_action_type_from_str_public(tag)
            .unwrap_or_else(|_| panic!("from_str must accept '{tag}'"));
        assert_eq!(
            at,
            SsbActionType::StructuralEditApplied(kind),
            "from_str('{tag}') must map to StructuralEditApplied({kind:?})"
        );
        assert_eq!(
            at.as_sql_str(),
            tag,
            "as_sql_str round-trip for StructuralEditApplied({kind:?})"
        );
    }
}

// ---------------------------------------------------------------------------
// (C2) REAL-DB happy path: search -> preview -> apply renames a Rust fn
//      STRUCTURALLY; the Applied row lands and the bytes are the AST-rewrite.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i54_c2_preview_then_apply_structural_rename() {
    let _env = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    let proj_dir = TempDir::new().expect("project tempdir");
    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let session = "i54-c2-session";
    // The gate's worktree root IS the project dir, so an in-root edit is allowed.
    std::env::set_var(PROJECT_ROOT_ENV, proj_dir.path());

    let ssb = test_ssb(&ssb_dir, session);
    let dispatcher = real_dispatcher();
    let clock = test_clock();

    // A Rust file with exactly one function named `old_name` (unique node text).
    let original = "fn old_name() -> i32 {\n    42\n}\n";
    let file = proj_dir.path().join("rename_me.rs");
    std::fs::write(&file, original).expect("write source");
    let file_str = file.to_string_lossy().to_string();

    // SEARCH: confirm the query matches the function name node.
    let search_args = serde_json::json!({
        "language": "rust",
        "source": original,
        "query": "(function_item name: (identifier) @name)"
    });
    let search_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_SEARCH,
        &serde_json::to_vec(&search_args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("structural_search must succeed");
    let search_json: serde_json::Value = serde_json::from_slice(&search_res).unwrap();
    assert_eq!(
        search_json["match_count"],
        serde_json::json!(1),
        "exactly one function name should match"
    );
    // Matched row lands.
    assert!(
        !ssb.query_by_type(SsbActionType::StructuralEditApplied(
            StructuralEditAppliedKind::Matched
        ))
        .expect("query Matched")
        .is_empty(),
        "StructuralEditApplied(Matched) row must land (R1 missing=FAIL)"
    );

    // PREVIEW: render the rewrite (rename old_name -> new_name) without writing.
    let preview_args = serde_json::json!({
        "language": "rust",
        "file": file_str,
        "query": "(function_item name: (identifier) @name)",
        "rewrite_template": "new_name",
        "whole_node_capture": "name"
    });
    let preview_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_PREVIEW,
        &serde_json::to_vec(&preview_args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("structural_preview must succeed");
    let preview_json: serde_json::Value = serde_json::from_slice(&preview_res).unwrap();
    assert_eq!(preview_json["match_count"], serde_json::json!(1));
    // No bytes written by preview.
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        original,
        "preview must NOT write any bytes"
    );
    assert!(
        !ssb.query_by_type(SsbActionType::StructuralEditApplied(
            StructuralEditAppliedKind::Previewed
        ))
        .expect("query Previewed")
        .is_empty(),
        "StructuralEditApplied(Previewed) row must land"
    );
    // The preview hands back the whole-file staleness anchor the apply requires.
    let source_hash = preview_json["source_hash"]
        .as_str()
        .expect("preview must return a source_hash anchor")
        .to_string();

    // APPLY: the rename actually lands as a STRUCTURAL edit. The agent passes the
    // preview's source_hash back as expected_source_hash (the file is unchanged).
    let mut apply_args = preview_args.clone();
    apply_args["expected_source_hash"] = serde_json::json!(source_hash);
    let apply_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_APPLY,
        &serde_json::to_vec(&apply_args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("structural_apply must succeed");
    let apply_json: serde_json::Value = serde_json::from_slice(&apply_res).unwrap();
    assert_eq!(apply_json["applied"], serde_json::json!(true));

    // The resulting bytes are the AST-rewrite: the function identifier renamed,
    // body byte-for-byte preserved (a structural rename, not a text smear).
    let after = std::fs::read_to_string(&file).unwrap();
    assert_eq!(
        after, "fn new_name() -> i32 {\n    42\n}\n",
        "apply must produce the exact AST-rewrite (fn name renamed, body intact)"
    );

    // Applied row lands.
    assert!(
        !ssb.query_by_type(SsbActionType::StructuralEditApplied(
            StructuralEditAppliedKind::Applied
        ))
        .expect("query Applied")
        .is_empty(),
        "StructuralEditApplied(Applied) row must land (R1 missing=FAIL)"
    );

    std::env::remove_var(PROJECT_ROOT_ENV);
}

// ---------------------------------------------------------------------------
// (C3) STALENESS: preview, mutate the file, apply -> REFUSED, no corruption,
//      RefusedStale row lands (the load-bearing safety observable, R1).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i54_c3_apply_refuses_stale_after_external_change() {
    let _env = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    let proj_dir = TempDir::new().expect("project tempdir");
    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let session = "i54-c3-session";
    std::env::set_var(PROJECT_ROOT_ENV, proj_dir.path());

    let ssb = test_ssb(&ssb_dir, session);
    let dispatcher = real_dispatcher();
    let clock = test_clock();

    let original = "fn old_name() -> i32 {\n    1\n}\n";
    let file = proj_dir.path().join("stale.rs");
    std::fs::write(&file, original).expect("write source");
    let file_str = file.to_string_lossy().to_string();

    let args = serde_json::json!({
        "language": "rust",
        "file": file_str,
        "query": "(function_item name: (identifier) @name)",
        "rewrite_template": "new_name",
        "whole_node_capture": "name"
    });

    // PREVIEW first (so the agent has seen the match set) and CAPTURE the
    // whole-file staleness anchor the apply will check against.
    let preview_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_PREVIEW,
        &serde_json::to_vec(&args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("preview ok");
    let preview_json: serde_json::Value = serde_json::from_slice(&preview_res).unwrap();
    let stale_hash = preview_json["source_hash"]
        .as_str()
        .expect("preview must return a source_hash anchor")
        .to_string();

    // MUTATE the file externally. A header is prepended above the function: the
    // matched node text `old_name` is UNCHANGED, so an in-node re-hash alone would
    // NOT detect this -- only the whole-file source_hash anchor does. This is
    // exactly the "changed since the preview" case the guard must catch.
    let mutated = "// header added externally\nfn old_name() -> i32 {\n    1\n}\n";
    std::fs::write(&file, mutated).expect("external mutation");

    // APPLY drives the REAL production dispatch with the now-STALE source_hash.
    // It MUST be refused by the production seam (not a bypass): the apply re-reads
    // the file, sees the hash no longer matches the previewed anchor, and refuses
    // with RefusedStale BEFORE any dispatch_edit.
    let mut apply_args = args.clone();
    apply_args["expected_source_hash"] = serde_json::json!(stale_hash);
    let apply_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_APPLY,
        &serde_json::to_vec(&apply_args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await;
    assert!(
        apply_res.is_err(),
        "a structural_apply against a file changed since the preview MUST be \
         refused by the production dispatch arm (RefusedStale), not applied"
    );
    let err_msg = format!("{:?}", apply_res.unwrap_err());
    assert!(
        err_msg.contains("changed since preview") || err_msg.contains("stale"),
        "the refusal must be the RefusedStale guard, got: {err_msg}"
    );

    // No overwrite: the file is byte-for-byte the externally-mutated content (the
    // apply wrote nothing -- no partial structural smear against a tree the agent
    // never previewed).
    let after = std::fs::read_to_string(&file).unwrap();
    assert_eq!(
        after, mutated,
        "a refused stale apply must write ZERO bytes (file == external mutation)"
    );

    // The RefusedStale SSB row landed via the PRODUCTION seam (R1 missing=FAIL,
    // R4 measure the real objective: this is the dispatch-arm emit, not a proxy).
    assert!(
        !ssb.query_by_type(SsbActionType::StructuralEditApplied(
            StructuralEditAppliedKind::RefusedStale
        ))
        .expect("query RefusedStale")
        .is_empty(),
        "StructuralEditApplied(RefusedStale) row must land from the production apply arm"
    );
    // And NO Applied row for the refused edit.
    assert!(
        ssb.query_by_type(SsbActionType::StructuralEditApplied(
            StructuralEditAppliedKind::Applied
        ))
        .expect("query Applied")
        .is_empty(),
        "a refused stale apply must NOT emit an Applied row"
    );

    std::env::remove_var(PROJECT_ROOT_ENV);
}

// ---------------------------------------------------------------------------
// (C4) GATE: an apply whose target ESCAPES the worktree root is REFUSED by the
//      REUSED enforce_fs_blast_radius (FsEscape) BEFORE any byte lands.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i54_c4_apply_refused_by_reused_blast_radius_gate() {
    let _env = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    let worktree_dir = TempDir::new().expect("worktree tempdir");
    let outside_dir = TempDir::new().expect("outside tempdir");
    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let session = "i54-c4-session";
    // The worktree root is `worktree_dir`, but the edit target lives in a
    // DIFFERENT TempDir (`outside_dir`) -> an FS escape the gate must refuse.
    std::env::set_var(PROJECT_ROOT_ENV, worktree_dir.path());

    let ssb = test_ssb(&ssb_dir, session);
    let dispatcher = real_dispatcher();
    let clock = test_clock();

    let original = "fn old_name() -> i32 {\n    7\n}\n";
    let file = outside_dir.path().join("escape.rs");
    std::fs::write(&file, original).expect("write outside source");
    let file_str = file.to_string_lossy().to_string();

    let args = serde_json::json!({
        "language": "rust",
        "file": file_str,
        "query": "(function_item name: (identifier) @name)",
        "rewrite_template": "new_name",
        "whole_node_capture": "name"
    });

    // PREVIEW to obtain the (matching) source_hash so the apply CLEARS the
    // staleness guard and reaches the FsEscape gate -- otherwise the apply would
    // refuse on a missing/stale hash and never exercise the blast-radius leg this
    // contract is about.
    let preview_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_PREVIEW,
        &serde_json::to_vec(&args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await
    .expect("preview ok (reads are not blast-radius gated)");
    let preview_json: serde_json::Value = serde_json::from_slice(&preview_res).unwrap();
    let source_hash = preview_json["source_hash"]
        .as_str()
        .expect("preview must return a source_hash anchor")
        .to_string();
    let mut apply_args = args.clone();
    apply_args["expected_source_hash"] = serde_json::json!(source_hash);

    let apply_res = translate_tool_request_for_test(
        TOOL_STRUCTURAL_APPLY,
        &serde_json::to_vec(&apply_args).unwrap(),
        &dispatcher,
        Some(&ssb),
        None,
        &clock,
    )
    .await;
    assert!(
        apply_res.is_err(),
        "an apply whose target escapes the worktree root MUST be refused by the \
         reused enforce_fs_blast_radius gate (FsEscape), not applied"
    );

    // The outside file must be byte-for-byte unchanged (the gate ran BEFORE any
    // dispatch_edit write).
    let after = std::fs::read_to_string(&file).unwrap();
    assert_eq!(
        after, original,
        "the gate must refuse BEFORE any byte lands (no partial write outside root)"
    );

    // No Applied row should have landed for this refused edit.
    let applied = ssb
        .query_by_type(SsbActionType::StructuralEditApplied(
            StructuralEditAppliedKind::Applied,
        ))
        .expect("query Applied");
    assert!(
        applied.is_empty(),
        "a gate-refused apply must NOT emit an Applied row"
    );

    // Sanity: the decode of any structural row that DID land round-trips.
    for kind in [
        StructuralEditAppliedKind::Matched,
        StructuralEditAppliedKind::Previewed,
        StructuralEditAppliedKind::RefusedStale,
    ] {
        for row in ssb
            .query_by_type(SsbActionType::StructuralEditApplied(kind))
            .expect("query rows")
        {
            match decode_ssb_payload(&row.payload).expect("decode payload") {
                SsbPayload::StructuralEditApplied(p) => assert_eq!(p.kind, kind),
                other => panic!("expected StructuralEditApplied, got {:?}", other.action_type()),
            }
        }
    }

    std::env::remove_var(PROJECT_ROOT_ENV);
}
