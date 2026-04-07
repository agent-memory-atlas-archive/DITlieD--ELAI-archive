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
#[should_panic(expected = "not yet implemented")]
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
#[should_panic(expected = "not yet implemented")]
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
#[should_panic(expected = "not yet implemented")]
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
