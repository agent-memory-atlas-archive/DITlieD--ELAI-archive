//! K-3 PR/CI evidence emitter contract.
//!
//! The fixture is entirely offline: it drives the pure renderer with an RRA
//! gate manifest and a multi-hunk review surface, then source-checks the real
//! CLI dispatch arm. No GitHub adapter or network call is exercised here.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::determinism::{DeterminismConfig, HardwareClass};
use elai_orchestrator::focus_tunnel::diff::{DiffBodyKind, DiffBodyLine, DiffHunk};
use elai_orchestrator::github_pr::evidence_emitter::render_pr_evidence;
use elai_orchestrator::review_anchor::{anchor_finding, AnchorKind};
use elai_orchestrator::review_surface::{
    hunk_id_for, FindingKind, HunkVouch, ReviewFinding, ReviewSurfaceScore, VouchKind,
};
use elai_orchestrator::rra::{build_rra, GateVerdictHash, RunArtifact};

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("contracts crate has a crates/ parent")
        .to_path_buf()
}

fn read_src(relative: &str) -> String {
    let path = crates_dir().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn fixture_artifact() -> RunArtifact {
    let pins = DeterminismConfig {
        hardware_class: HardwareClass::Cpu,
        seed: 73,
        blake3_cache_prefix: [0x4b; 8],
        system_fingerprint_pin: Some("k3-offline-fixture".to_string()),
    };
    let mut artifact = build_rra("k3-offline-session", &[], &pins);
    artifact.manifest = vec![
        GateVerdictHash {
            gate: "F-1".to_string(),
            action_type: "vibe_gate_verdict".to_string(),
            verdict_blake3: "1111222233334444".to_string(),
            ssb_event_seq: 41,
        },
        GateVerdictHash {
            gate: "F-4".to_string(),
            action_type: "test_integrity_event".to_string(),
            verdict_blake3: "aaaabbbbccccdddd".to_string(),
            ssb_event_seq: 42,
        },
    ];
    artifact
}

fn fixture_hunk(file_path: &str, old_start: usize, new_start: usize, added: &str) -> DiffHunk {
    DiffHunk {
        file_path: file_path.to_string(),
        old_start,
        old_count: 0,
        new_start,
        new_count: 1,
        context_lines: Vec::new(),
        added_lines: vec![added.to_string()],
        removed_lines: Vec::new(),
        body_order: vec![DiffBodyLine {
            kind: DiffBodyKind::Added,
            text: added.to_string(),
        }],
    }
}

fn fixture_review_surface() -> ReviewSurfaceScore {
    let vouched_diff = fixture_hunk("src/lib.rs", 10, 10, "let verified = true;");
    let unvouched_diff = fixture_hunk("src/unsafe.rs", 20, 20, "run_unverified();");
    let vouched_hunk_id = hunk_id_for(&vouched_diff);
    let unvouched_hunk_id = hunk_id_for(&unvouched_diff);
    let anchor = anchor_finding(
        "let verified = true;",
        std::slice::from_ref(&vouched_diff),
        Some(&vouched_hunk_id),
    );
    assert_eq!(anchor.kind, AnchorKind::Anchored);

    ReviewSurfaceScore {
        hunks: vec![
            HunkVouch {
                hunk_id: vouched_hunk_id,
                file_path: vouched_diff.file_path,
                vouched_by: vec![VouchKind::TestStdoutHash],
                lines_changed: 4,
                needs_attention: false,
            },
            HunkVouch {
                hunk_id: unvouched_hunk_id,
                file_path: unvouched_diff.file_path,
                vouched_by: Vec::new(),
                lines_changed: 3,
                needs_attention: true,
            },
        ],
        findings: vec![ReviewFinding {
            file_path: "src/lib.rs".to_string(),
            kind: FindingKind::TestFailed,
            snippet: "let verified = true;".to_string(),
            anchor: Some(anchor),
        }],
        dsr: 3.0 / 7.0,
        total_changed_lines: 7,
        attention_lines: 3,
        vouched_hunks: 1,
        unvouched_hunks: 1,
    }
}

#[test]
fn multi_hunk_fixture_renders_gate_and_vouch_bands() {
    let rendered = render_pr_evidence(&fixture_artifact(), &fixture_review_surface());

    assert!(rendered.contains("Check-run gate summary"));
    assert!(rendered.contains("1111222233334444"));
    assert!(rendered.contains("aaaabbbbccccdddd"));
    assert!(rendered.contains("VOUCHED — `src/lib.rs`"));
    assert!(rendered.contains("`test_stdout_hash`"));
    assert!(rendered.contains("resolver-owned new-side lines 10-10"));
}

#[test]
fn evidence_less_hunk_is_explicitly_unvouched() {
    let rendered = render_pr_evidence(&fixture_artifact(), &fixture_review_surface());

    assert!(rendered.contains("UNVOUCHED — `src/unsafe.rs`"));
    assert!(rendered.contains("UNVOUCHED — full reviewer attention required"));
    assert!(!rendered.contains("CLEAN — `src/unsafe.rs`"));
}

#[test]
fn emit_evidence_arm_rejects_clean_override_before_rendering() {
    let cli = read_src("cli/src/pr_review_cmd.rs");
    let arm_start = cli
        .find("PrReviewAction::EmitEvidence {\n            pr,")
        .expect("EmitEvidence dispatch arm exists");
    let arm_tail = &cli[arm_start..];
    let arm_end = arm_tail
        .find("PrReviewAction::SuggestSquash")
        .expect("next pure dispatch arm exists");
    let arm = &arm_tail[..arm_end];
    let rejection = arm
        .find("if post_unvouched_as_clean")
        .expect("unsafe clean override is checked");
    let render = arm
        .find("render_pr_evidence(&run_artifact, &review_surface)")
        .expect("production arm calls renderer");
    let compose = arm
        .find("compose_pr_body(\"{{evidence}}\", &ctx)")
        .expect("production arm feeds existing body seam");

    assert!(rejection < render, "override must fail before any render");
    assert!(render < compose, "rendered evidence must feed compose_pr_body");
    assert!(arm.contains("return Err(anyhow!("));
    assert!(arm.contains("\"posted\": false"));
    assert!(!arm.contains("post_review_comment(Arc::clone(&ssb), pr, &body"));
}

#[test]
fn emitter_remains_renderer_only_and_uses_resolver_owned_anchors() {
    let emitter = read_src("orchestrator/src/github_pr/evidence_emitter.rs");

    assert!(!emitter.contains("impl GithubPrAdapter"));
    assert!(!emitter.contains("GhCliAdapter"));
    assert!(!emitter.contains("reqwest"));
    assert!(!emitter.contains("octocrab"));
    assert!(emitter.contains("ReviewFinding::anchor"));
    assert!(emitter.contains("anchor.new_line_range"));
    assert!(emitter.contains("!hunk.vouched_by.is_empty() && !hunk.needs_attention"));
    assert!(!emitter.contains("finding.snippet.parse"));
}
