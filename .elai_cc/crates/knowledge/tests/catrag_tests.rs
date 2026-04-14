use std::path::PathBuf;

use elai_knowledge::catrag::{phi, CatRag, CatRagConfig};
use elai_knowledge::pagerank::{DkgSnapshot, PageRankNodeInfo};
use elai_knowledge::SymbolKind;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_node(id: i64, name: &str) -> PageRankNodeInfo {
    PageRankNodeInfo {
        node_id: id,
        symbol_name: name.to_string(),
        file_path: PathBuf::from(format!("src/{name}.rs")),
        signature: format!("fn {name}()"),
        kind: SymbolKind::Function,
        ref_count: 1,
    }
}

/// 4 nodes, 2 non-self edges -> density = 2/12 ~ 0.167 > 0.01
fn dense_snapshot() -> DkgSnapshot {
    DkgSnapshot {
        nodes: vec![
            make_node(1, "alpha"),
            make_node(2, "beta"),
            make_node(3, "gamma"),
            make_node(4, "delta"),
        ],
        edges: vec![(0, 1, 1.0), (2, 3, 1.0)],
    }
}

/// 10 nodes, 0 edges -> density = 0 < 0.01
fn sparse_snapshot() -> DkgSnapshot {
    DkgSnapshot {
        nodes: (0..10).map(|i| make_node(i, &format!("node_{i}"))).collect(),
        edges: vec![],
    }
}

// ---------------------------------------------------------------------------
// (a) compute_overlay on dense graph panics (todo!())
// ---------------------------------------------------------------------------

#[test]
fn a_compute_overlay_dense_graph_hits_todo() {
    let catrag = CatRag::new(CatRagConfig::default());
    let snapshot = dense_snapshot();
    let scores: Vec<(usize, f64)> = vec![(0, 0.4), (1, 0.3), (2, 0.2), (3, 0.1)];
    let _ = catrag.compute_overlay("test query", &snapshot, &scores);
}

// ---------------------------------------------------------------------------
// (b) compute_overlay returns CatRagOverlay with non-empty fields (todo!())
// ---------------------------------------------------------------------------

#[test]
fn b_compute_overlay_returns_overlay_structure() {
    let catrag = CatRag::new(CatRagConfig::default());
    let snapshot = dense_snapshot();
    let scores: Vec<(usize, f64)> = vec![(0, 0.5), (1, 0.3), (2, 0.1), (3, 0.1)];
    let overlay = catrag.compute_overlay("rust async", &snapshot, &scores).unwrap();
    assert!(!overlay.reset_distribution.is_empty());
    assert!(!overlay.edge_weight_deltas.is_empty());
}

// ---------------------------------------------------------------------------
// (c) compute_overlay past density guard with custom config (todo!())
// ---------------------------------------------------------------------------

#[test]
fn c_compute_overlay_custom_config_past_density_guard() {
    let config = CatRagConfig {
        damping: 0.85,
        epsilon: 0.1,
        beta: 3.0,
        n_seed: 3,
        k_edge: 10,
        min_graph_density: 0.05,
    };
    let catrag = CatRag::new(config);
    let snapshot = dense_snapshot();
    let scores: Vec<(usize, f64)> = vec![(0, 0.25), (1, 0.25), (2, 0.25), (3, 0.25)];
    let _ = catrag.compute_overlay("graph traversal", &snapshot, &scores);
}

// ---------------------------------------------------------------------------
// (d) density guard returns empty overlay (PASSES immediately)
// ---------------------------------------------------------------------------

#[test]
fn d_density_guard_returns_empty_overlay() {
    let catrag = CatRag::new(CatRagConfig::default());
    let snapshot = sparse_snapshot();
    let scores: Vec<(usize, f64)> = (0..10).map(|i| (i, 0.1)).collect();
    let overlay = catrag.compute_overlay("anything", &snapshot, &scores).unwrap();
    assert!(overlay.reset_distribution.is_empty());
    assert!(overlay.edge_weight_deltas.is_empty());
}

// ---------------------------------------------------------------------------
// (e) phi() scores match Table 7 (PASSES immediately)
// ---------------------------------------------------------------------------

#[test]
fn e_phi_scores_match_table_7() {
    let expected: [(u8, f64); 11] = [
        (0, 0.0),
        (1, 0.0),
        (2, 0.0),
        (3, 0.0),
        (4, 0.2),
        (5, 0.25),
        (6, 0.3),
        (7, 2.0),
        (8, 2.5),
        (9, 3.0),
        (10, 5.0),
    ];
    for (score, want) in expected {
        let got = phi(score);
        assert!(
            (got - want).abs() < f64::EPSILON,
            "phi({score}) = {got}, expected {want}"
        );
    }
}

// ---------------------------------------------------------------------------
// (f) symbolic_anchor: lowercase-only query extracts 0 entities
// ---------------------------------------------------------------------------

#[test]
fn f_symbolic_anchor_no_entities_from_lowercase_query() {
    let catrag = CatRag::new(CatRagConfig::default());
    let mut snapshot = dense_snapshot();
    // Add a CamelCase node that should NOT be matched by a lowercase query
    snapshot.nodes.push(make_node(5, "HttpClient"));
    snapshot.edges.push((0, 4, 1.0));
    let scores: Vec<(usize, f64)> = vec![(0, 0.3), (1, 0.3), (2, 0.2), (3, 0.1), (4, 0.1)];

    let overlay = catrag
        .compute_overlay("how does the system work", &snapshot, &scores)
        .unwrap();

    // "how does the system work" has no CamelCase or UPPER_SNAKE tokens.
    // Symbolic anchoring should contribute zero entity-based entries.
    // All reset_distribution entries should come from PPR scores only.
    // Specifically, "HttpClient" node should NOT get an entity anchor weight.
    for &(idx, _weight) in &overlay.reset_distribution {
        // Node 4 ("HttpClient") should only appear if it has a PPR score,
        // not from entity anchoring of a lowercase query.
        if idx == 4 {
            // Weight should be purely from PPR, not boosted by entity anchoring.
            // With epsilon=0.2 and pr_score=0.1: PPR contribution = 0.1 * 0.2 = 0.02
            // Entity anchor would add epsilon * 1/ref_count = 0.2 * 1.0 = 0.2
            // So if entity anchoring fired, weight would be >> 0.02 relative to total.
            // We verify the node's weight is proportional to its PPR score only.
            let node0_entry = overlay.reset_distribution.iter().find(|e| e.0 == 0);
            if let Some(&(_, w0)) = node0_entry {
                // node0 has pr=0.3, node4 has pr=0.1, ratio should be ~3:1
                // If entity anchoring fired on node4, ratio would be much smaller
                assert!(
                    w0 / _weight > 2.0,
                    "HttpClient node weight too high relative to alpha — entity anchoring may have fired on lowercase query"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// (g) key_fact_enhance: verified edges get boosted, non-verified do not
// ---------------------------------------------------------------------------

#[test]
fn g_key_fact_enhance_boosts_verified_edges() {
    // Build a snapshot where some edges match query tokens (verified) and some don't
    let nodes = vec![
        make_node(1, "AsyncRuntime"),   // 0: matches "async" token
        make_node(2, "runtime_core"),   // 1: matches "runtime" token
        make_node(3, "unrelated_util"), // 2: no match
        make_node(4, "helper"),         // 3: no match
    ];
    let edges = vec![
        (0, 1, 1.0), // AsyncRuntime -> runtime_core (both match)
        (0, 2, 1.0), // AsyncRuntime -> unrelated_util
        (2, 3, 1.0), // unrelated_util -> helper (src doesn't match)
    ];
    let snapshot = DkgSnapshot { nodes, edges };

    let config = CatRagConfig {
        min_graph_density: 0.0, // disable density guard
        ..CatRagConfig::default()
    };
    let catrag = CatRag::new(config);
    let scores: Vec<(usize, f64)> = vec![(0, 0.5), (1, 0.3), (2, 0.1), (3, 0.1)];

    // Query with CamelCase entity "AsyncRuntime" so symbolic anchoring fires
    let overlay = catrag
        .compute_overlay("AsyncRuntime handler", &snapshot, &scores)
        .unwrap();

    // Key-fact enhancement should boost edges from nodes matching query tokens.
    // Edge (0,1) from "AsyncRuntime" (matches "asyncruntime"/"async"/"runtime" tokens)
    // should be boosted. Edge (2,3) from "unrelated_util" should NOT be boosted.
    // Edge from node 0 ("AsyncRuntime") should be boosted (matches query tokens)
    let boosted_entry = overlay
        .edge_weight_deltas
        .iter()
        .find(|&&(src, tgt, _)| src == 0 && tgt == 1);
    assert!(
        boosted_entry.is_some(),
        "Expected edge (0,1) from AsyncRuntime in deltas, got: {:?}",
        overlay.edge_weight_deltas
    );
    let boosted_weight = boosted_entry.unwrap().2;
    // phi(8)=2.5 is the base weight for a matched edge (score 5+3=8).
    // After key_fact_enhance: 2.5 * (1 + 2.5) = 8.75
    assert!(
        boosted_weight > 2.5,
        "Expected boosted weight > 2.5 (phi(8)), got {boosted_weight}"
    );

    // Verify at least one edge exists that was NOT boosted by key_fact_enhance.
    // Edge (2,3) from "unrelated_util" won't be in t_seed (not anchored, not top-N_seed)
    // so it should not appear in deltas at all, OR appear at base phi weight.
    // At minimum, not ALL edges should have the same boosted weight.
    let non_boosted_exists = overlay
        .edge_weight_deltas
        .iter()
        .any(|&(_, _, w)| (w - boosted_weight).abs() > f64::EPSILON);
    assert!(
        non_boosted_exists || overlay.edge_weight_deltas.len() == 1,
        "All edges have identical weight — key_fact_enhance may be boosting indiscriminately: {:?}",
        overlay.edge_weight_deltas
    );
}
