//! C-2 contract test (Layer-4) — tree-node SSB emission.
//!
//! Source plan: docs/thefinalglance/C-plan/C-2_ssb_emission_orchestration.md
//! Wiring doctrine: .claude/rules/wiring-enforcement.md Layer-4 — drive the
//! REAL production emitter object and assert the off-agent SSB row lands in a
//! real SQLite-backed Ssb. Missing evidence = FAIL (a payload that never
//! reaches the writer is indistinguishable from a wire-dark producer, so each
//! test asserts on the bytes read back from the DB, NOT on a hand-built
//! payload).
//!
//! The production tree-node producer is
//! `elai_orchestrator::research_tree::SsbDecompositionEmitter`, the
//! `DecompositionTreeEmitter` the task_runner plugs into `research_decompose`
//! so every concept/phase/step/baby_step built surfaces as a
//! `concept_decomposition_node` SSB row. We drive its real `emit` path and
//! poll the DB until the fire-and-forget `tokio::spawn` write lands.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::research_tree::{
    DecompositionNodeKind, DecompositionNodePayload, DecompositionTreeEmitter,
    SsbDecompositionEmitter,
};
use elai_orchestrator::ssb::{
    ConceptDecompositionNodePayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};
use tempfile::TempDir;

const TREE_AGENT: &str = "c2-tree-node-contract";

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

fn open_writer(path: &std::path::Path) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "c2-tree".into(),
        session_id: "c2-tree-test".into(),
    };
    Arc::new(Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init"))
}

/// Poll the DB up to ~2s for at least `min` rows of `kind` to land (the
/// emitter writes via `tokio::spawn`, so the row is not synchronous with the
/// `emit` call). Returns the rows once present; panics on timeout (a timeout
/// is a FAIL — the producer never reached the writer).
async fn await_rows(
    ssb: &Ssb,
    kind: SsbActionType,
    min: usize,
) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    for _ in 0..200 {
        let rows = ssb.query_by_type(kind).expect("query_by_type");
        if rows.len() >= min {
            return rows;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "timed out waiting for >= {min} {kind:?} rows; the real emitter never \
         reached the SSB writer (Layer-4 FAIL: producer is wire-dark)"
    );
}

// ---------------------------------------------------------------------------
// (1) A single concept node emitted by the REAL producer lands as a real row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concept_node_emit_lands_real_ssb_row() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);
    let emitter = SsbDecompositionEmitter::new(Arc::clone(&ssb), TREE_AGENT, 0);

    emitter.emit(DecompositionNodePayload {
        node_kind: DecompositionNodeKind::Concept,
        node_id: 1,
        parent_id: 0,
        name: "auth-refactor".into(),
    });

    let rows = await_rows(&ssb, SsbActionType::ConceptDecompositionNode, 1).await;
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("deserialize ConceptDecompositionNode");
    match payload {
        SsbPayload::ConceptDecompositionNode(ConceptDecompositionNodePayload {
            node_kind,
            node_id,
            parent_id,
            name,
        }) => {
            assert_eq!(node_kind, "concept");
            assert_eq!(node_id, 1);
            assert_eq!(parent_id, 0);
            assert_eq!(name, "auth-refactor");
        }
        other => panic!("expected ConceptDecompositionNode, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (2) Every node KIND (concept/phase/step/baby_step) round-trips the kind tag.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn all_node_kinds_round_trip_kind_tag() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);
    let emitter = SsbDecompositionEmitter::new(Arc::clone(&ssb), TREE_AGENT, 0);

    let kinds = [
        (DecompositionNodeKind::Concept, "concept"),
        (DecompositionNodeKind::Phase, "phase"),
        (DecompositionNodeKind::Step, "step"),
        (DecompositionNodeKind::BabyStep, "baby_step"),
    ];
    for (i, (kind, _)) in kinds.iter().enumerate() {
        emitter.emit(DecompositionNodePayload {
            node_kind: *kind,
            node_id: (i + 1) as u64,
            parent_id: i as u64,
            name: format!("node-{i}"),
        });
    }

    let rows = await_rows(&ssb, SsbActionType::ConceptDecompositionNode, 4).await;
    let mut seen: Vec<String> = Vec::new();
    for row in &rows {
        if let Ok(SsbPayload::ConceptDecompositionNode(p)) = rmp_serde::from_slice(&row.payload) {
            seen.push(p.node_kind);
        }
    }
    for (_, tag) in kinds.iter() {
        assert!(
            seen.iter().any(|s| s == tag),
            "kind '{tag}' missing from emitted rows: {seen:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// (3) parent_id linkage is preserved on the wire (tree topology survives).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parent_linkage_preserved_on_wire() {
    let (_dir, path) = fresh_db();
    let ssb = open_writer(&path);
    let emitter = SsbDecompositionEmitter::new(Arc::clone(&ssb), TREE_AGENT, 0);

    // concept(10) -> phase(11) child of 10.
    emitter.emit(DecompositionNodePayload {
        node_kind: DecompositionNodeKind::Concept,
        node_id: 10,
        parent_id: 0,
        name: "root".into(),
    });
    emitter.emit(DecompositionNodePayload {
        node_kind: DecompositionNodeKind::Phase,
        node_id: 11,
        parent_id: 10,
        name: "child".into(),
    });

    let rows = await_rows(&ssb, SsbActionType::ConceptDecompositionNode, 2).await;
    let mut child_parent: Option<u64> = None;
    for row in &rows {
        if let Ok(SsbPayload::ConceptDecompositionNode(p)) = rmp_serde::from_slice(&row.payload) {
            if p.node_id == 11 {
                child_parent = Some(p.parent_id);
            }
        }
    }
    assert_eq!(
        child_parent,
        Some(10),
        "phase node 11 must record parent_id=10 (tree linkage lost on the wire)"
    );
}

// ---------------------------------------------------------------------------
// (4) action_type() <-> as_sql_str round-trip is stable for the tree variant.
// ---------------------------------------------------------------------------

#[test]
fn concept_decomposition_node_action_type_is_stable() {
    assert_eq!(
        SsbActionType::ConceptDecompositionNode.as_sql_str(),
        "concept_decomposition_node"
    );
    let payload = SsbPayload::ConceptDecompositionNode(ConceptDecompositionNodePayload {
        node_kind: "phase".into(),
        node_id: 7,
        parent_id: 3,
        name: "x".into(),
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::ConceptDecompositionNode
    );
}
