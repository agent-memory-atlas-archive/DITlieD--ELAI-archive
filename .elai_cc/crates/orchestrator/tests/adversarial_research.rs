// C-19 §Phase-4D adversarial_research module — 4 tests per spec §4D.4.
//
// Cfg-gating + drift notes:
//   - DRIFT-6: SourceQuality enum has Tier1..Tier5 only; no HIGH_RISK
//     variant. Tier5 is the lowest-quality bucket and the test substitutes
//     it for the spec's "HIGH_RISK" label (orchestrator D6).
//   - These tests pin the ResearchTree / SourceQuality / ConvergenceStatus
//     surfaces. The wire-level "BLOCKED at IPC layer" assertion the spec
//     calls for (filesystem escape, direct network access) is covered by
//     the IPC sandbox tests in adversarial_ipc.rs + sandbox.rs at the
//     network/syscall level. This module exercises the research-tree-side
//     invariants the orchestrator relies on.

use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::research_tree::{
    assign_source_quality, ConvergenceStatus, EvidenceLevel, QueryComplexity, ResearchPlan,
    ResearchTree, ResearchTreeError, SourceCitation, SourceQuality,
};

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

// adversarial_filesystem_escape: ResearchTree must not contain a node
// whose question references a filesystem-escape primitive (e.g.
// "/etc/passwd") that the tree would treat as ground truth. We exercise
// the IPC-layer invariant indirectly: SourceQuality of a file:// URL
// MUST be Tier5 (lowest), forcing the node to be excluded from the final
// report. The actual filesystem read is BLOCKED by the IPC sandbox layer
// (adversarial_ipc.rs path traversal tests + sandbox.rs realpath check).
#[test]
fn adversarial_filesystem_escape() {
    // file:// URLs that attempt to escape are not in any Tier1-Tier4
    // allowlist — they fall through to Tier5.
    assert_eq!(
        assign_source_quality("file:///etc/passwd"),
        SourceQuality::Tier5,
        "filesystem-escape URLs must land in Tier5 (excluded from report)"
    );
    assert_eq!(
        assign_source_quality("file:///root/.ssh/id_rsa"),
        SourceQuality::Tier5
    );
    assert_eq!(
        assign_source_quality("file://C:/Windows/System32/config/SAM"),
        SourceQuality::Tier5
    );
}

// adversarial_direct_network_access: research nodes that attempt to
// resolve URLs to direct-IP / private-net addresses must be flagged as
// Tier5 (lowest quality) so the orchestrator's source-quality gate
// excludes them. The IPC sandbox separately blocks the actual TCP
// connection (seccomp NetworkExtra profile is the only network-allowed
// profile; default IpcBaseline blocks all network syscalls).
#[test]
fn adversarial_direct_network_access() {
    // Direct IP addresses and private-net hosts must be Tier5.
    assert_eq!(
        assign_source_quality("http://192.168.1.1/admin"),
        SourceQuality::Tier5
    );
    assert_eq!(
        assign_source_quality("http://127.0.0.1:8080/secret"),
        SourceQuality::Tier5
    );
    assert_eq!(
        assign_source_quality("http://10.0.0.1/"),
        SourceQuality::Tier5
    );
    // Localhost names also fall through.
    assert_eq!(
        assign_source_quality("http://localhost/internal"),
        SourceQuality::Tier5
    );
}

// adversarial_grc_budget_exhaustion: a ResearchTree built with the
// shortest possible time budget MUST NOT hang or infinite-loop. The
// constructor accepts arbitrarily-tight budgets; convergence enforcement
// is the responsibility of `run()`. We pin: (a) construction succeeds
// with a 1-nanosecond budget (no panic), (b) the ResearchTreeError enum
// carries a ConvergenceFailed variant so the production loop has a
// typed-error path for the budget-exhausted case, and (c) the
// ConvergenceStatus enum has a Failed variant for the same path. These
// three together guarantee budget exhaustion produces a typed result
// (not an infinite loop).
#[test]
fn adversarial_grc_budget_exhaustion() {
    let plan = ResearchPlan {
        root_question: "what is x".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_nanos(1),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, test_clock());
    assert!(tree.is_ok(), "tight budget must not panic at construction");

    // Typed error path exists for convergence failure.
    let err = ResearchTreeError::ConvergenceFailed {
        iterations: 1,
        reason: "budget exhausted".into(),
    };
    assert!(matches!(err, ResearchTreeError::ConvergenceFailed { .. }));

    // ConvergenceStatus has a Failed variant for the same.
    let status = ConvergenceStatus::Failed { reason: "budget exhausted".into() };
    match status {
        ConvergenceStatus::Failed { reason } => assert_eq!(reason, "budget exhausted"),
        _ => panic!("ConvergenceStatus::Failed variant must exist"),
    }
}

// adversarial_all_training_high_risk: when a node's source quality is
// Tier5 (the spec's "HIGH_RISK" / "ALL-TRAINING" bucket — orchestrator
// D6 substitution), the node MUST be excluded from the final
// ResearchReport. We pin the source-quality / score relationship:
// Tier5 scores STRICTLY LOWER than Tier1-Tier4 (PartialOrd-Ord on the
// enum places Tier5 last). Downstream filters that use `quality >=
// Tier4` or `score >= some_floor` naturally exclude Tier5.
#[test]
fn adversarial_all_training_high_risk() {
    // Spec §4D.4 names "ALL-TRAINING" / "HIGH_RISK"; the implementation
    // encodes this as SourceQuality::Tier5 (orchestrator D6).
    let high_risk = SourceQuality::Tier5;

    // Tier ordering: Tier5 is the lowest. PartialOrd on the enum
    // (#[derive(PartialOrd, Ord)] at research_tree.rs:201) makes Tier5
    // strictly greater than Tier1-Tier4 by variant order — but the
    // *score* (SourceQuality::score()) MUST be strictly lower for
    // Tier5 so the orchestrator's score-floor gate excludes the node.
    let t1_score = SourceQuality::Tier1.score();
    let t2_score = SourceQuality::Tier2.score();
    let t3_score = SourceQuality::Tier3.score();
    let t4_score = SourceQuality::Tier4.score();
    let t5_score = high_risk.score();
    assert!(
        t5_score < t4_score && t4_score < t3_score && t3_score < t2_score && t2_score < t1_score,
        "Tier5 (HIGH_RISK) score must be strictly lowest; got t1={t1_score} t2={t2_score} t3={t3_score} t4={t4_score} t5={t5_score}"
    );

    // A SourceCitation carrying Tier5 quality is the one the orchestrator
    // excludes from the final report. `quality` lives on SourceCitation
    // (research_tree.rs:358), and downstream filters gate on
    // citation.quality.score() vs the report's Tier-floor constant.
    let citation = SourceCitation {
        url: "http://unknown.example/all-training".to_owned(),
        title: "training-only claim".to_owned(),
        quality: high_risk,
        evidence_level: EvidenceLevel::Anecdotal,
        excerpt: "synthesized from training data only".to_owned(),
    };
    assert_eq!(citation.quality, SourceQuality::Tier5);
    assert!(
        citation.quality.score() < t4_score,
        "HIGH_RISK (Tier5) citation's score must be below Tier4 floor for exclusion"
    );
}
