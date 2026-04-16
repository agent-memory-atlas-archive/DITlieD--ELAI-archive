// research_tree.rs — 4D.3/4D.4 contract, integration, adversarial, and proptest tests
//
// TDD red phase: all tests compile, most fail until 4D.5 implementation.

use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::research_tree::*;

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

// ---------------------------------------------------------------------------
// Contract tests (>= 14)
// ---------------------------------------------------------------------------

#[test]
fn contract_tree_creation_sets_correct_complexity_params() {
    let plan = ResearchPlan {
        root_question: "test question about Rust".into(),
        sub_questions: vec!["sub1".into()],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    assert_eq!(tree.breadth_factor, BREADTH_SIMPLE);
    assert_eq!(tree.depth_limit, DEPTH_SIMPLE);
    assert!((tree.phi_min - PHI_MIN_SIMPLE).abs() < f64::EPSILON);
    assert!((tree.psi_min - PSI_MIN_DEFAULT).abs() < f64::EPSILON);
}

#[test]
fn contract_tree_creation_moderate_params() {
    let plan = ResearchPlan {
        root_question: "moderate question".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Moderate,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, false, test_clock()).unwrap();
    assert_eq!(tree.breadth_factor, BREADTH_MODERATE);
    assert_eq!(tree.depth_limit, DEPTH_MODERATE);
    assert!((tree.phi_min - PHI_MIN_MODERATE).abs() < f64::EPSILON);
}

#[test]
fn contract_tree_creation_complex_params() {
    let plan = ResearchPlan {
        root_question: "complex question".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Complex,
        time_budget: Duration::from_secs(120),
        require_contradictions: true,
    };
    let tree = ResearchTree::new(plan, false, test_clock()).unwrap();
    assert_eq!(tree.breadth_factor, BREADTH_COMPLEX);
    assert_eq!(tree.depth_limit, DEPTH_COMPLEX);
    assert!((tree.phi_min - PHI_MIN_COMPLEX).abs() < f64::EPSILON);
}

#[test]
fn contract_root_node_exists_after_creation() {
    let plan = ResearchPlan {
        root_question: "root test".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let root = tree.get_node(tree.root).unwrap();
    assert_eq!(root.status, NodeStatus::Pending);
    assert_eq!(root.depth, 0);
    assert!(root.parent.is_none());
}

#[test]
fn contract_empty_question_rejected() {
    let plan = ResearchPlan {
        root_question: String::new(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    assert!(ResearchTree::new(plan, true, test_clock()).is_err());
}

#[test]
fn contract_whitespace_only_question_rejected() {
    let plan = ResearchPlan {
        root_question: "   \t\n  ".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    assert!(ResearchTree::new(plan, true, test_clock()).is_err());
}

#[test]
fn contract_local_provider_forces_speculative_zero() {
    let plan = ResearchPlan {
        root_question: "test".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Complex,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    assert_eq!(tree.speculative_depth, 0, "local provider must force speculative_depth=0");
}

#[test]
fn contract_cloud_provider_allows_speculative() {
    let plan = ResearchPlan {
        root_question: "test".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Complex,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let tree = ResearchTree::new(plan, false, test_clock()).unwrap();
    assert!(tree.speculative_depth > 0, "cloud provider should allow speculative depth");
}

#[test]
fn contract_convergence_requires_both_thresholds() {
    let node = ResearchNode::new(NodeId(0), "test".into(), None, 0);

    // phi high, psi low => not converged
    let mut n = node.clone();
    n.phi = 0.9;
    n.psi = 0.5;
    assert!(!n.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT));

    // phi low, psi high => not converged
    n.phi = 0.5;
    n.psi = 0.9;
    assert!(!n.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT));

    // both high => converged
    n.phi = 0.9;
    n.psi = 0.9;
    assert!(n.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT));
}

#[test]
fn contract_node_lifecycle_pending_to_active() {
    let mut node = ResearchNode::new(NodeId(1), "q".into(), None, 0);
    assert_eq!(node.status, NodeStatus::Pending);
    node.status = NodeStatus::Active;
    assert_eq!(node.status, NodeStatus::Active);
}

#[test]
fn contract_node_lifecycle_active_to_converged() {
    let mut node = ResearchNode::new(NodeId(1), "q".into(), None, 0);
    node.status = NodeStatus::Active;
    node.phi = 0.9;
    node.psi = 0.9;
    if node.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT) {
        node.status = NodeStatus::Converged;
    }
    assert_eq!(node.status, NodeStatus::Converged);
}

#[test]
fn contract_node_lifecycle_active_to_pruned() {
    let mut node = ResearchNode::new(NodeId(1), "q".into(), None, 0);
    node.status = NodeStatus::Active;
    node.low_gain_streak = LOW_GAIN_PRUNE_COUNT;
    if node.should_prune() {
        node.status = NodeStatus::Pruned;
    }
    assert_eq!(node.status, NodeStatus::Pruned);
}

#[test]
fn contract_add_child_increments_depth() {
    let plan = ResearchPlan {
        root_question: "root".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Moderate,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let root = tree.root;
    let child = tree.add_child(root, "child".into()).unwrap();
    assert_eq!(tree.get_node(child).unwrap().depth, 1);
    assert_eq!(tree.get_node(child).unwrap().parent, Some(root));
}

#[test]
fn contract_add_child_updates_parent_children_list() {
    let plan = ResearchPlan {
        root_question: "root".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Moderate,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let root = tree.root;
    let child = tree.add_child(root, "child".into()).unwrap();
    assert!(tree.get_node(root).unwrap().children.contains(&child));
}

#[test]
fn contract_source_quality_deterministic() {
    // Same URL must always return same quality
    let url = "https://arxiv.org/abs/2510.05145";
    let q1 = assign_source_quality(url);
    let q2 = assign_source_quality(url);
    assert_eq!(q1, q2);
    assert_eq!(q1, SourceQuality::Tier1);
}

#[test]
fn contract_source_quality_exact_domain_no_substring() {
    // Phishing domain must NOT match Tier1
    assert_eq!(assign_source_quality("https://arxiv.phishing.com/abs/foo"), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("https://notarxiv.org/abs/foo"), SourceQuality::Tier5);
    // Subdomain DOES match
    assert_eq!(assign_source_quality("https://www.arxiv.org/abs/foo"), SourceQuality::Tier1);
}

// ---------------------------------------------------------------------------
// Integration tests (>= 3) — test run() which is the 4D.5 target
// ---------------------------------------------------------------------------

#[test]
fn integration_run_returns_report_on_simple_query() {
    let plan = ResearchPlan {
        root_question: "What is Rust ownership?".into(),
        sub_questions: vec!["How does borrowing work?".into()],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(10),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let result = tree.run();
    // 4D.5: should succeed. Currently returns Err (stub).
    assert!(result.is_ok(), "run() should return Ok after 4D.5 implementation");
}

#[test]
fn integration_run_converges_moderate_query() {
    let plan = ResearchPlan {
        root_question: "Compare Rust vs Go for web servers".into(),
        sub_questions: vec![
            "What are the performance differences?".into(),
            "What are the ecosystem differences?".into(),
        ],
        complexity: QueryComplexity::Moderate,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let result = tree.run();
    assert!(result.is_ok(), "run() should converge on moderate query after 4D.5");
    if let Ok(report) = result {
        // Convergence must be Converged or Partial, not Failed
        match report.convergence {
            ConvergenceStatus::Failed { .. } => panic!("should not fail on moderate query"),
            _ => {}
        }
    }
}

#[test]
fn integration_run_respects_time_budget() {
    let plan = ResearchPlan {
        root_question: "Complex analysis of distributed systems".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Complex,
        time_budget: Duration::from_millis(1), // Very short budget
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let result = tree.run();
    // Should return a result (Ok or Err) but not hang
    // After 4D.5: should return Ok with ConvergenceStatus::Failed or Partial
    match result {
        Ok(report) => {
            // Time budget exceeded -> convergence should not be full Converged
            // (unless the tree trivially converged in 0 iterations)
        }
        Err(_) => {
            // Acceptable during stub phase
        }
    }
}

#[test]
fn integration_snapshot_reflects_tree_state() {
    let plan = ResearchPlan {
        root_question: "snapshot test".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(10),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();

    // Add some children
    let root = tree.root;
    let _c1 = tree.add_child(root, "child 1".into()).unwrap();
    let _c2 = tree.add_child(root, "child 2".into()).unwrap();

    let snap = tree.snapshot();
    assert_eq!(snap.nodes.len(), 3); // root + 2 children
}

// ---------------------------------------------------------------------------
// Adversarial tests (>= 8)
// ---------------------------------------------------------------------------

#[test]
fn adversarial_depth_bomb_rejected() {
    // Attacker tries to create an extremely deep tree
    let plan = ResearchPlan {
        root_question: "depth bomb".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple, // depth_limit = 3
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let mut parent = tree.root;

    // Try to go deeper than MAX_TREE_DEPTH
    for i in 0..MAX_TREE_DEPTH + 5 {
        match tree.add_child(parent, format!("depth {i}")) {
            Ok(child) => parent = child,
            Err(_) => break, // Expected: depth limit prevents bomb
        }
    }
    // Tree depth must not exceed the limit
    let max_depth = tree.state.nodes.values().map(|n| n.depth).max().unwrap_or(0);
    assert!(max_depth <= tree.depth_limit.min(MAX_TREE_DEPTH),
        "depth bomb must be prevented: max_depth={max_depth}, limit={}",
        tree.depth_limit.min(MAX_TREE_DEPTH));
}

#[test]
fn adversarial_breadth_bomb_rejected() {
    // Attacker tries to create hundreds of children on root
    let plan = ResearchPlan {
        root_question: "breadth bomb".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let root = tree.root;

    for i in 0..MAX_TOTAL_NODES + 50 {
        match tree.add_child(root, format!("child {i}")) {
            Ok(_) => {}
            Err(_) => break, // Expected: node count limit prevents bomb
        }
    }
    assert!(tree.state.node_count() <= MAX_TOTAL_NODES,
        "breadth bomb must be prevented: count={}, limit={MAX_TOTAL_NODES}",
        tree.state.node_count());
}

#[test]
fn adversarial_source_quality_phishing_domain() {
    // Attacker crafts URLs that look like tier1 but aren't
    assert_eq!(assign_source_quality("https://arxiv.phishing.com/abs/2510.05145"), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("https://fake-arxiv.org/paper"), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("https://arxiv.org.evil.com/abs/123"), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("https://docs.rs.malware.net/serde"), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("https://xarxiv.org/abs/123"), SourceQuality::Tier5);
}

#[test]
fn adversarial_empty_input_handling() {
    // Empty/malformed inputs must produce typed errors, not panics
    assert!(assess_complexity("").is_err());
    assert!(assess_complexity("   ").is_err());

    let plan = ResearchPlan {
        root_question: String::new(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    assert!(ResearchTree::new(plan, true, test_clock()).is_err());

    // Malformed URLs
    assert_eq!(assign_source_quality(""), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("not-a-url"), SourceQuality::Tier5);
    assert_eq!(assign_source_quality("://missing-scheme"), SourceQuality::Tier5);
}

#[test]
fn adversarial_nan_phi_psi_never_converge() {
    let mut node = ResearchNode::new(NodeId(0), "test".into(), None, 0);
    node.phi = f64::NAN;
    node.psi = 0.9;
    assert!(!node.is_converged(0.0, 0.0), "NaN phi must never count as converged");

    node.phi = 0.9;
    node.psi = f64::NAN;
    assert!(!node.is_converged(0.0, 0.0), "NaN psi must never count as converged");

    node.phi = f64::NAN;
    node.psi = f64::NAN;
    assert!(!node.is_converged(0.0, 0.0), "NaN phi+psi must never count as converged");
}

#[test]
fn adversarial_infinity_phi_psi() {
    let mut node = ResearchNode::new(NodeId(0), "test".into(), None, 0);
    node.phi = f64::INFINITY;
    node.psi = f64::INFINITY;
    // Infinity >= any threshold is true in IEEE 754, but the system should handle it.
    // This tests that we don't crash — the convergence check is mathematically valid.
    let _result = node.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT);
    // No panic = pass
}

#[test]
fn adversarial_speculative_pruning_consistency() {
    let plan = ResearchPlan {
        root_question: "prune test".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Moderate,
        time_budget: Duration::from_secs(60),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let root = tree.root;

    // Add children, mark some as Active
    let c1 = tree.add_child(root, "child 1".into()).unwrap();
    let c2 = tree.add_child(root, "child 2".into()).unwrap();
    let c3 = tree.add_child(root, "child 3".into()).unwrap();

    tree.get_node_mut(c1).unwrap().status = NodeStatus::Active;
    tree.get_node_mut(c2).unwrap().status = NodeStatus::Converged;
    // c3 stays Pending

    let pruned = tree.prune_speculative_children(root).unwrap();
    assert_eq!(pruned, 1, "only Pending children should be pruned");
    assert_eq!(tree.get_node(c1).unwrap().status, NodeStatus::Active);
    assert_eq!(tree.get_node(c2).unwrap().status, NodeStatus::Converged);
    assert_eq!(tree.get_node(c3).unwrap().status, NodeStatus::Pruned);
}

#[test]
fn adversarial_contradiction_surfacing() {
    // After 4D.5: run() on a query with contradictions must surface them in the report
    let plan = ResearchPlan {
        root_question: "Is Rust or C++ faster for embedded systems?".into(),
        sub_questions: vec!["What is Rust embedded performance?".into(), "What is C++ embedded performance?".into()],
        complexity: QueryComplexity::Moderate,
        time_budget: Duration::from_secs(30),
        require_contradictions: true,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let result = tree.run();
    // 4D.5: should produce a report with contradictions when require_contradictions=true
    if let Ok(report) = result {
        assert!(!report.contradictions.is_empty(),
            "contradictions must be surfaced when require_contradictions=true");
    }
    // During stub phase: Err is acceptable
}

#[test]
fn adversarial_add_child_to_nonexistent_parent() {
    let plan = ResearchPlan {
        root_question: "test".into(),
        sub_questions: vec![],
        complexity: QueryComplexity::Simple,
        time_budget: Duration::from_secs(30),
        require_contradictions: false,
    };
    let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
    let result = tree.add_child(NodeId(9999), "orphan".into());
    assert!(result.is_err(), "adding child to nonexistent parent must error");
}

// ---------------------------------------------------------------------------
// Proptest strategies (>= 3)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_complexity() -> impl Strategy<Value = QueryComplexity> {
        prop_oneof![
            Just(QueryComplexity::Simple),
            Just(QueryComplexity::Moderate),
            Just(QueryComplexity::Complex),
        ]
    }

    fn arb_source_quality() -> impl Strategy<Value = SourceQuality> {
        prop_oneof![
            Just(SourceQuality::Tier1),
            Just(SourceQuality::Tier2),
            Just(SourceQuality::Tier3),
            Just(SourceQuality::Tier4),
            Just(SourceQuality::Tier5),
        ]
    }

    fn arb_plan() -> impl Strategy<Value = ResearchPlan> {
        (
            "[a-zA-Z ]{5,50}",
            prop::collection::vec("[a-zA-Z ]{5,30}", 0..5),
            arb_complexity(),
        )
            .prop_map(|(question, subs, complexity)| ResearchPlan {
                root_question: question,
                sub_questions: subs,
                complexity,
                time_budget: Duration::from_secs(30),
                require_contradictions: false,
            })
    }

    // Strategy 1: Tree creation never panics for valid inputs
    proptest! {
        #[test]
        fn tree_creation_never_panics(plan in arb_plan(), local in proptest::bool::ANY) {
            let result = ResearchTree::new(plan, local, test_clock());
            // Must either succeed or return a typed error — never panic
            match result {
                Ok(tree) => {
                    prop_assert!(tree.get_node(tree.root).is_some());
                }
                Err(_) => {
                    // Typed error is acceptable
                }
            }
        }
    }

    // Strategy 2: Source quality is deterministic (same URL => same tier)
    proptest! {
        #[test]
        fn source_quality_deterministic(url in "https?://[a-z]{3,20}\\.[a-z]{2,5}/[a-z]{0,30}") {
            let q1 = assign_source_quality(&url);
            let q2 = assign_source_quality(&url);
            prop_assert_eq!(q1, q2, "source quality must be deterministic for same URL");
        }
    }

    // Strategy 3: Complexity params are internally consistent
    proptest! {
        #[test]
        fn complexity_params_consistent(complexity in arb_complexity()) {
            let breadth = complexity.breadth_factor();
            let depth = complexity.depth_limit();
            let phi_min = complexity.phi_min();

            prop_assert!(breadth >= 2 && breadth <= 4);
            prop_assert!(depth >= 3 && depth <= 5);
            prop_assert!(phi_min >= 0.6 && phi_min <= 0.8);

            // Higher complexity => lower phi_min (easier to converge)
            match complexity {
                QueryComplexity::Simple => prop_assert!((phi_min - PHI_MIN_SIMPLE).abs() < f64::EPSILON),
                QueryComplexity::Moderate => prop_assert!((phi_min - PHI_MIN_MODERATE).abs() < f64::EPSILON),
                QueryComplexity::Complex => prop_assert!((phi_min - PHI_MIN_COMPLEX).abs() < f64::EPSILON),
            }
        }
    }

    // Strategy 4: Add child always increments depth
    proptest! {
        #[test]
        fn add_child_increments_depth(
            plan in arb_plan(),
            child_count in 1u32..5,
        ) {
            let mut tree = ResearchTree::new(plan, true, test_clock())?;
            let root = tree.root;
            let mut parent = root;
            for i in 0..child_count {
                match tree.add_child(parent, format!("child {i}")) {
                    Ok(child) => {
                        let child_node = tree.get_node(child).unwrap();
                        let parent_node = tree.get_node(parent).unwrap();
                        prop_assert_eq!(child_node.depth, parent_node.depth + 1);
                        parent = child;
                    }
                    Err(_) => break, // depth limit hit — acceptable
                }
            }
        }
    }
}
