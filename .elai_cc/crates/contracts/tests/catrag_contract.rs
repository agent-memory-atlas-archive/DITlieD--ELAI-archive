#![allow(unused_imports, dead_code)]
// Compile-time interface assertions for CatRag, CatRagOverlay, and phi().
// Type-level checks — if the API surface changes, this file fails to compile.

use std::error::Error;

use elai_knowledge::catrag::{phi, CatRag, CatRagConfig, CatRagOverlay};
use elai_knowledge::pagerank::DkgSnapshot;

// Verify phi signature: (u8) -> f64
fn _assert_phi_signature() {
    let _: fn(u8) -> f64 = phi;
}

// Verify CatRag::new signature: (CatRagConfig) -> Self
fn _assert_catrag_new_signature() {
    let _: fn(CatRagConfig) -> CatRag = CatRag::new;
}

// Verify compute_overlay signature: (&self, &str, &DkgSnapshot, &[(usize,f64)]) -> Result<CatRagOverlay, ...>
fn _assert_compute_overlay_signature(catrag: &CatRag) {
    let snapshot = DkgSnapshot {
        nodes: vec![],
        edges: vec![],
    };
    let scores: &[(usize, f64)] = &[];
    let _: Result<CatRagOverlay, Box<dyn Error>> = catrag.compute_overlay("q", &snapshot, scores);
}

// Verify CatRagOverlay fields are public
fn _assert_overlay_fields(overlay: CatRagOverlay) {
    let _: Vec<(usize, f64)> = overlay.reset_distribution;
    let _: Vec<(usize, usize, f64)> = overlay.edge_weight_deltas;
}

// Verify CatRagConfig fields are public and Default is implemented
fn _assert_config_fields() {
    let config = CatRagConfig::default();
    let _: f64 = config.damping;
    let _: f64 = config.epsilon;
    let _: f64 = config.beta;
    let _: usize = config.n_seed;
    let _: usize = config.k_edge;
    let _: f64 = config.min_graph_density;
}
