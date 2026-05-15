// B-29 Phase 1 — public API surface contract.
//
// Locks the signatures of the new `knowledge::symbol_graph` module so refactors
// must intentionally update this file. Mirrors the WIRING_CONTRACT block in
// docs/thefinalglance/B-plan/B-29_symbol_level_impact_api.md.

use elai_knowledge::symbol_graph::sharding::{
    all_name_shard_keys, name_shard_key, reverse_shard_bucket, MAX_FLOW_DEPTH, MAX_IMPACT_DEPTH,
    SYMBOL_FILE_LRU_SIZE, SYMBOL_NAME_SHARDS, SYMBOL_REVERSE_SHARDS,
};
use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, EntryPoint, SymbolEdge, SymbolGraphFilePayload, SymbolGraphMeta,
    SymbolKind, SymbolNode, SymbolRef,
};

#[test]
fn b29_phase1_constants_match_plan() {
    assert_eq!(SYMBOL_NAME_SHARDS, 27);
    assert_eq!(SYMBOL_REVERSE_SHARDS, 256);
    assert_eq!(SYMBOL_FILE_LRU_SIZE, 500);
    assert_eq!(MAX_IMPACT_DEPTH, 10);
    assert_eq!(MAX_FLOW_DEPTH, 10);
}

#[test]
fn b29_phase1_name_shards_cover_a_to_z_plus_underscore() {
    let keys = all_name_shard_keys();
    assert_eq!(keys.len(), 27);
    let mut sorted = keys;
    sorted.sort();
    assert_eq!(sorted[0], '_');
    for (idx, ch) in ('a'..='z').enumerate() {
        assert_eq!(sorted[idx + 1], ch);
    }
}

#[test]
fn b29_phase1_name_shard_key_signature() {
    let _: char = name_shard_key("foo");
    let _: char = name_shard_key("");
    let _: char = name_shard_key("_x");
    let _: char = name_shard_key("Bar");
}

#[test]
fn b29_phase1_reverse_shard_bucket_signature() {
    let _: u8 = reverse_shard_bucket("crates/foo/src/lib.rs");
    assert_eq!(
        reverse_shard_bucket("a.rs"),
        reverse_shard_bucket("a.rs"),
        "deterministic"
    );
}

#[test]
fn b29_phase1_symbol_kind_has_nine_variants() {
    // Exhaustive match locks the variant set.
    fn _check(k: SymbolKind) -> u8 {
        match k {
            SymbolKind::Function => 0,
            SymbolKind::Method => 1,
            SymbolKind::Class => 2,
            SymbolKind::Struct => 3,
            SymbolKind::Trait => 4,
            SymbolKind::Module => 5,
            SymbolKind::Enum => 6,
            SymbolKind::Interface => 7,
            SymbolKind::Type => 8,
        }
    }
    let _ = _check;
}

#[test]
fn b29_phase1_confidence_has_four_variants() {
    fn _check(c: Confidence) -> u8 {
        match c {
            Confidence::Local => 0,
            Confidence::Unique => 1,
            Confidence::MultipleCandidates => 2,
            Confidence::Unresolved => 3,
        }
    }
    let _ = _check;
}

#[test]
fn b29_phase1_symbol_node_fields_lock() {
    let n = SymbolNode {
        id: "id".into(),
        name: "n".into(),
        qualified_name: "q::n".into(),
        kind: SymbolKind::Function,
        file: "f.rs".into(),
        line: 1,
        end_line: 2,
        language: "rust".into(),
    };
    assert_eq!(n.id, "id");
    assert_eq!(n.end_line - n.line, 1);
}

#[test]
fn b29_phase1_symbol_edge_fields_lock() {
    let e = SymbolEdge {
        caller_id: "c".into(),
        callee_name: "cn".into(),
        callee_candidates: vec!["x".into()],
        confidence: Confidence::Unique,
        call_site: CallSite {
            file: "f.rs".into(),
            line: 5,
        },
    };
    assert_eq!(e.callee_candidates.len(), 1);
    assert_eq!(e.confidence, Confidence::Unique);
}

#[test]
fn b29_phase1_symbol_ref_fields_lock() {
    let r = SymbolRef {
        file: "f.rs".into(),
        id: "f.rs::n".into(),
    };
    assert!(r.id.starts_with(&r.file));
}

#[test]
fn b29_phase1_entry_point_fields_lock() {
    let ep = EntryPoint {
        id: "main".into(),
        name: "main".into(),
        file: "main.rs".into(),
        line: Some(1),
        reason: "named main".into(),
    };
    assert_eq!(ep.line, Some(1));
}

#[test]
fn b29_phase1_meta_fields_lock_and_percent_scale() {
    let m = SymbolGraphMeta {
        project_id: "p".into(),
        symbol_count: 0,
        edge_count: 0,
        file_count: 0,
        unresolved_edge_pct: 12.0,
        built_at: 0,
        schema_version: 3,
    };
    // Contract: percentage scale (0.0–100.0), NOT a 0.0–1.0 fraction.
    // WIRING_CONTRACT stub_ban requires < 50.0; tests/fixtures use percentage values.
    assert!(m.unresolved_edge_pct >= 0.0 && m.unresolved_edge_pct <= 100.0);
    assert_eq!(m.schema_version, 3);
}

#[test]
fn b29_phase1_file_payload_fields_lock() {
    let p = SymbolGraphFilePayload {
        file: "f.rs".into(),
        language: "rust".into(),
        content_hash: "h".into(),
        symbols: vec![],
        outgoing_calls: vec![],
    };
    assert!(p.symbols.is_empty());
    assert!(p.outgoing_calls.is_empty());
}

#[test]
fn b29_phase1_postcard_is_the_encoder_not_msgpack() {
    // Spec contract: column blobs are postcard-encoded (workspace dep), NOT
    // rmp-serde. Re-encoding through postcard must succeed for all types.
    let m = SymbolGraphMeta {
        project_id: "p".into(),
        symbol_count: 5,
        edge_count: 3,
        file_count: 2,
        unresolved_edge_pct: 33.3,
        built_at: 1_700_000_000_000,
        schema_version: 3,
    };
    let bytes = postcard::to_allocvec(&m).expect("postcard encode");
    let back: SymbolGraphMeta = postcard::from_bytes(&bytes).expect("postcard decode");
    assert_eq!(m, back);
}
