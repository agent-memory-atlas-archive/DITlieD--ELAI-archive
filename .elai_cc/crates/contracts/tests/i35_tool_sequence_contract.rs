//! I-35 temporal tool-sequence enforcement contract.
//!
//! Tests the tool-sequence enforcement gate via the REAL evaluator and emitter,
//! verifying that ToolSequenceVerdict SSB rows land as off-author wiring-enforcement
//! R4 evidence, and that the gate is fail-closed on missing evidence (R1).
//!
//! Arms (all fail-closed, R1 missing-evidence = FAIL):
//!   (a) COMPLIANT-ALLOWED: history [src_edit, auditor_pass, git_push] satisfies R1;
//!       verdict must be Allowed and SSB row lands.
//!   (b) VIOLATION-HALT: history [src_edit, git_push] missing auditor_pass; R1 triggers
//!       HaltEnforced; SSB row lands.
//!   (c) REPLAY-DETERMINISM: same history + rules evaluated twice yields identical verdicts
//!       (GD-2 replay-determinism property).
//!   (d) DEGRADED-PREDICATE-HONESTY: documents and asserts the documented degraded behavior;
//!       R1 ENABLED but bench-injected (production push site deferred).
//!   (e) SCHEMA-LOCK: SCHEMA_VERSION_PUBLIC >= 242 AND action_type_lock.rs declares
//!       TOTAL_VARIANTS: usize = 551 AND "tool_sequence_verdict" round-trips via from_str.
//!
//! Spec references: docs/thefinalglance/I-plan/I-35_temporal_sequence_enforcement.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; .claude/rules/tracing-test-assert.md

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    SCHEMA_VERSION_PUBLIC, SsbActionType, SsbPayload, ToolSequenceVerdictKind,
    decode_ssb_payload, ssb_action_type_from_str_public,
};
use elai_orchestrator::tool_sequence::{
    HistoryEntry, SequenceRuleSet, ToolSequenceVerdict, emit_tool_sequence_verdict,
    evaluate_sequence_rules,
};

// ---------------------------------------------------------------------------
// (a) COMPLIANT-ALLOWED: history satisfies R1, SSB row lands.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i35_a_compliant_allowed_emits_ssb_row() {
    use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "i35-contract".into(),
                session_id: "i35-a".into(),
            },
            clock.clone(),
        )
        .expect("ssb open"),
    );

    let rules = SequenceRuleSet::load_builtin();

    // History: src_edit -> auditor_pass -> git_push
    // R1 (git_push requires auditor_pass since src_edit): satisfied.
    let history = vec![
        HistoryEntry {
            tool_class: "src_edit".to_owned(),
            ts_ms: 1_000,
            outcome: "allowed".to_owned(),
        },
        HistoryEntry {
            tool_class: "auditor_pass".to_owned(),
            ts_ms: 1_001,
            outcome: "allowed".to_owned(),
        },
        HistoryEntry {
            tool_class: "git_push".to_owned(),
            ts_ms: 1_002,
            outcome: "allowed".to_owned(),
        },
    ];

    let verdict = evaluate_sequence_rules(&rules, "git_push", &history);
    assert_eq!(
        verdict,
        ToolSequenceVerdict::Allowed,
        "compliant history must yield Allowed; got {verdict:?}"
    );

    // Emit SSB row (R4: off-author artifact).
    emit_tool_sequence_verdict(Some(&ssb), &verdict, "git_push", Some("i35-a"))
        .await
        .expect("emit Allowed row must succeed");

    // R1: off-author artifact must be present (missing = FAIL).
    let rows = ssb
        .query_by_type(SsbActionType::ToolSequenceVerdict(
            ToolSequenceVerdictKind::Allowed,
        ))
        .expect("query ToolSequenceVerdict rows");
    assert_eq!(rows.len(), 1, "exactly 1 ToolSequenceVerdict row must land; got {}", rows.len());

    let payload = decode_ssb_payload(&rows[0].payload).expect("decode payload");
    match payload {
        SsbPayload::ToolSequenceVerdict(p) => {
            assert_eq!(
                p.kind,
                ToolSequenceVerdictKind::Allowed,
                "payload kind must be Allowed; got {:?}",
                p.kind
            );
            assert_eq!(
                p.tool_class, "git_push",
                "tool_class must be 'git_push'; got '{}'",
                p.tool_class
            );
        }
        other => panic!("row must be ToolSequenceVerdict Allowed; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (b) VIOLATION-HALT: missing auditor_pass triggers HaltEnforced, row lands.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i35_b_violation_halt_emits_ssb_row() {
    use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "i35-contract".into(),
                session_id: "i35-b".into(),
            },
            clock.clone(),
        )
        .expect("ssb open"),
    );

    let rules = SequenceRuleSet::load_builtin();

    // History: src_edit -> git_push (no auditor_pass)
    // R1: unsatisfied -> HaltEnforced (fail-closed).
    let history = vec![
        HistoryEntry {
            tool_class: "src_edit".to_owned(),
            ts_ms: 2_000,
            outcome: "allowed".to_owned(),
        },
        HistoryEntry {
            tool_class: "git_push".to_owned(),
            ts_ms: 2_001,
            outcome: "allowed".to_owned(),
        },
    ];

    let verdict = evaluate_sequence_rules(&rules, "git_push", &history);

    // R1: missing evidence = HaltEnforced (not SKIP, not WARN).
    match &verdict {
        ToolSequenceVerdict::HaltEnforced { rule_id, .. } => {
            assert_eq!(
                rule_id, "R1-git-push-requires-auditor-pass",
                "rule_id must identify R1; got '{rule_id}'"
            );
        }
        other => panic!(
            "violation history must yield HaltEnforced; got {other:?}"
        ),
    }

    // Emit SSB row (R4: off-author artifact).
    emit_tool_sequence_verdict(Some(&ssb), &verdict, "git_push", Some("i35-b"))
        .await
        .expect("emit HaltEnforced row must succeed");

    let rows = ssb
        .query_by_type(SsbActionType::ToolSequenceVerdict(
            ToolSequenceVerdictKind::Allowed,
        ))
        .expect("query ToolSequenceVerdict rows");
    assert_eq!(rows.len(), 1, "exactly 1 ToolSequenceVerdict row must land; got {}", rows.len());

    let payload = decode_ssb_payload(&rows[0].payload).expect("decode payload");
    match payload {
        SsbPayload::ToolSequenceVerdict(p) => {
            assert_eq!(
                p.kind,
                ToolSequenceVerdictKind::HaltEnforced,
                "payload kind must be HaltEnforced; got {:?}",
                p.kind
            );
        }
        other => panic!("row must be ToolSequenceVerdict HaltEnforced; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (c) REPLAY-DETERMINISM (GD-2): same history + rules -> identical verdicts.
// ---------------------------------------------------------------------------

#[test]
fn i35_c_replay_determinism() {
    let rules = SequenceRuleSet::load_builtin();

    let history = vec![
        HistoryEntry {
            tool_class: "src_edit".to_owned(),
            ts_ms: 3_000,
            outcome: "allowed".to_owned(),
        },
        HistoryEntry {
            tool_class: "auditor_pass".to_owned(),
            ts_ms: 3_001,
            outcome: "allowed".to_owned(),
        },
        HistoryEntry {
            tool_class: "git_push".to_owned(),
            ts_ms: 3_002,
            outcome: "allowed".to_owned(),
        },
    ];

    // Evaluate twice: GD-2 requires identical results.
    let v1 = evaluate_sequence_rules(&rules, "git_push", &history);
    let v2 = evaluate_sequence_rules(&rules, "git_push", &history);
    assert_eq!(
        v1, v2,
        "GD-2 replay-determinism: two evaluations of identical history+rules must be identical; \
         v1={v1:?}, v2={v2:?}"
    );
    assert_eq!(
        v1,
        ToolSequenceVerdict::Allowed,
        "compliant replay must yield Allowed; got {v1:?}"
    );

    // Violation case is also deterministic.
    let history_viol = vec![
        HistoryEntry {
            tool_class: "src_edit".to_owned(),
            ts_ms: 3_100,
            outcome: "allowed".to_owned(),
        },
        HistoryEntry {
            tool_class: "git_push".to_owned(),
            ts_ms: 3_101,
            outcome: "allowed".to_owned(),
        },
    ];
    let v3 = evaluate_sequence_rules(&rules, "git_push", &history_viol);
    let v4 = evaluate_sequence_rules(&rules, "git_push", &history_viol);
    assert_eq!(
        v3, v4,
        "GD-2: violation history must also be deterministic; v3={v3:?}, v4={v4:?}"
    );
    assert!(
        matches!(v3, ToolSequenceVerdict::HaltEnforced { .. }),
        "violation replay must yield HaltEnforced; got {v3:?}"
    );
}

// ---------------------------------------------------------------------------
// (d) DEGRADED-PREDICATE-HONESTY: documents R1/R2 as bench-injected (degraded).
// ---------------------------------------------------------------------------

#[test]
fn i35_d_degraded_predicate_honesty() {
    let rules = SequenceRuleSet::load_builtin();

    // Assert: R1 is ENABLED (not disabled) so it fires when evidence is present.
    let r1 = rules
        .rules
        .iter()
        .find(|r| r.id == "R1-git-push-requires-auditor-pass")
        .expect("R1 must be present in the builtin rule set");
    assert!(
        r1.enabled,
        "R1 must be ENABLED (degraded, not disabled); disabled would be fail-open"
    );

    // Assert: R3 is DISABLED (production push site not wired; disabled = safer).
    let r3 = rules
        .rules
        .iter()
        .find(|r| r.id == "R3-git-commit-requires-test-run-pass")
        .expect("R3 must be present in the builtin rule set");
    assert!(
        !r3.enabled,
        "R3 must be DISABLED (no production test-runner push site; disabled > always-passing)"
    );

    // Assert: an empty history for a git_push trigger (no src_edit, no auditor_pass)
    // produces Allowed (R1 window opens at src_edit; absent src_edit = no window open).
    let empty_history: Vec<HistoryEntry> = Vec::new();
    let verdict_empty = evaluate_sequence_rules(&rules, "git_push", &empty_history);
    assert_eq!(
        verdict_empty,
        ToolSequenceVerdict::Allowed,
        "empty history for git_push (no src_edit in session) must yield Allowed \
         (predicate window has not opened); got {verdict_empty:?}"
    );

    // Assert: degraded behavior is documented in predicate_honesty field.
    assert_eq!(
        r1.predicate_honesty, "degraded",
        "R1 predicate_honesty must be 'degraded' (bench-injected auditor_pass; \
         production push site deferred)"
    );
}

// ---------------------------------------------------------------------------
// (e) SCHEMA-LOCK: version pins for the I-35 bump.
// ---------------------------------------------------------------------------

#[test]
fn i35_e_schema_lock() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 242,
        "SCHEMA_VERSION_PUBLIC must be >= 242 (I-35 floor: ToolSequenceVerdict added); \
         got {SCHEMA_VERSION_PUBLIC}"
    );

    use std::fs;
    let lock_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| {
            root.join("crates")
                .join("orchestrator")
                .join("tests")
                .join("action_type_lock.rs")
        })
        .expect("resolve action_type_lock.rs path");
    let body = fs::read_to_string(&lock_rs).expect("read action_type_lock.rs");
    // Floor check (A117): parse-then-floor avoids stale-anchor cascade on future TV bumps.
    const I35_TV_FLOOR: usize = 551;
    let tv_ok = {
        let prefix = "const TOTAL_VARIANTS: usize = ";
        body.lines().any(|line| {
            if let Some(rest) = line.trim().strip_prefix(prefix) {
                rest.trim_end_matches(';')
                    .parse::<usize>()
                    .map(|n| n >= I35_TV_FLOOR)
                    .unwrap_or(false)
            } else {
                false
            }
        })
    };
    assert!(
        tv_ok,
        "action_type_lock.rs must declare TOTAL_VARIANTS: usize >= {I35_TV_FLOOR} (I-35 floor); \
         file: {}",
        lock_rs.display()
    );
    assert!(
        body.contains("SsbActionType::ToolSequenceVerdict"),
        "ALL_VARIANTS must include ToolSequenceVerdict; file: {}",
        lock_rs.display()
    );

    // Wire-tag round-trip via public from_str.
    let at = ssb_action_type_from_str_public("tool_sequence_verdict")
        .expect("from_str tool_sequence_verdict must succeed");
    assert_eq!(
        at.as_sql_str(),
        "tool_sequence_verdict",
        "wire tag must round-trip as_sql_str -> from_str -> as_sql_str"
    );
}
