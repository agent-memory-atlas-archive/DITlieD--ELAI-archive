//! Contract tests for the C-pack (knowledge crate polish).
//!
//! Validates:
//!   C1 — Svelte grammar registered in DkgEngine (`ext_to_lang_id("svelte")`
//!        returns `Some(_)`).
//!   C2 — `IndexManifest` round-trips through JSON in a stable order
//!        (BTreeMap-backed).
//!   C3 — `pick_strategy` returns the expected `ChunkStrategy` variant for a
//!        small input table.
//!
//! All three modules are new in this phase. Tests are deliberately minimal:
//! we assert public surface and round-trip stability, not algorithmic depth.

use std::path::PathBuf;

use elai_knowledge::chunk_strategy::{pick_strategy, ChunkStrategy};
use elai_knowledge::dkg::ext_to_lang_id;
use elai_knowledge::index_manifest::{
    IndexFileEntry, IndexFileStatus, IndexManifest,
};

// ---------------------------------------------------------------------------
// C1 — Svelte grammar registration
// ---------------------------------------------------------------------------

#[test]
fn svelte_extractor_registered() {
    // The C-pack contract requires `.svelte` files to map to a registered
    // language id. Symbol extraction depth is out of scope here — the
    // dkg_multilang_contract covers per-language extraction for the
    // languages that ship with full grammars. For Svelte we only require
    // that the extension is recognised by `ext_to_lang_id`.
    let lang = ext_to_lang_id("svelte");
    assert!(
        lang.is_some(),
        "ext_to_lang_id(\"svelte\") must return Some(_) once SvelteGrammar is registered"
    );
    assert_eq!(
        lang,
        Some("svelte"),
        "ext_to_lang_id(\"svelte\") should resolve to the canonical language id \"svelte\""
    );
}

// ---------------------------------------------------------------------------
// C2 — IndexManifest JSON round-trip with BTreeMap order stability
// ---------------------------------------------------------------------------

#[test]
fn index_manifest_roundtrips_via_json() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let manifest_path = tmp.path().join("manifest.json");

    let rust_entry = IndexFileEntry {
        path: PathBuf::from("crates/core/src/lib.rs"),
        mtime_secs: 1_700_000_000,
        content_blake3: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        status: IndexFileStatus::Indexed,
    };
    let python_entry = IndexFileEntry {
        path: PathBuf::from("python/elai_inference/server.py"),
        mtime_secs: 1_700_000_100,
        content_blake3: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .to_string(),
        status: IndexFileStatus::Pending,
    };
    let svelte_entry = IndexFileEntry {
        path: PathBuf::from("crates/dashboard-v2/src/routes/+page.svelte"),
        mtime_secs: 1_700_000_200,
        content_blake3: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            .to_string(),
        status: IndexFileStatus::Failed {
            reason: "parse error".to_string(),
        },
    };

    let mut original = IndexManifest::default();
    original
        .files
        .insert(rust_entry.path.clone(), rust_entry.clone());
    original
        .files
        .insert(python_entry.path.clone(), python_entry.clone());
    original
        .files
        .insert(svelte_entry.path.clone(), svelte_entry.clone());

    original.save(&manifest_path).expect("save manifest");
    let loaded = IndexManifest::load(&manifest_path).expect("load manifest");

    assert_eq!(loaded.version, original.version);
    assert_eq!(loaded.files.len(), 3);
    assert_eq!(loaded.files.get(&rust_entry.path), Some(&rust_entry));
    assert_eq!(loaded.files.get(&python_entry.path), Some(&python_entry));
    assert_eq!(loaded.files.get(&svelte_entry.path), Some(&svelte_entry));

    // BTreeMap iteration order is deterministic; loaded must match original.
    let original_keys: Vec<&PathBuf> = original.files.keys().collect();
    let loaded_keys: Vec<&PathBuf> = loaded.files.keys().collect();
    assert_eq!(
        original_keys, loaded_keys,
        "BTreeMap key order must survive JSON round-trip"
    );
}

#[test]
fn index_manifest_load_missing_returns_default() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let missing = tmp.path().join("does_not_exist.json");
    let manifest = IndexManifest::load(&missing)
        .expect("missing manifest must produce Default::default(), not error");
    assert!(manifest.files.is_empty());
}

// ---------------------------------------------------------------------------
// C3 — pick_strategy variant table
// ---------------------------------------------------------------------------

#[test]
fn pick_strategy_returns_expected_variants() {
    let cases: &[(&str, u64, ChunkStrategy)] = &[
        ("rust", 100, ChunkStrategy::ASTSemantic),
        ("md", 100, ChunkStrategy::SlidingWindow),
        ("markdown", 100, ChunkStrategy::SlidingWindow),
        ("rust", 100_000, ChunkStrategy::HierarchicalSummary),
        ("unknown", 100, ChunkStrategy::LineBlock),
        ("python", 200, ChunkStrategy::ASTSemantic),
        ("svelte", 200, ChunkStrategy::ASTSemantic),
        ("txt", 5_000, ChunkStrategy::SlidingWindow),
        ("python", 60_000, ChunkStrategy::HierarchicalSummary),
    ];
    for (lang, size, expected) in cases {
        let got = pick_strategy(lang, *size);
        assert_eq!(
            got, *expected,
            "pick_strategy(\"{lang}\", {size}) should be {expected:?}, got {got:?}"
        );
    }
}
