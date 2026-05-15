// B-29 Phase 4 — Symbol graph build pipeline.
//
// Provides:
//   build_symbol_graph       — full build from a project root
//   update_changed_files_symbol_graph — incremental update or full rebuild
//   BuildSummary / UpdateSummary — return types
//   INCREMENTAL_SYMBOL_THRESHOLD — spec-locked constant (50)
//
// Design invariants:
//   - Sync only: no async fn, no tokio, no JoinSet.
//   - No .unwrap() / .expect() in non-test code.
//   - BTreeMap everywhere order matters for determinism.
//   - Opens its own rusqlite::Connection to dkg_db_path (caller has dropped
//     DkgEngine, releasing the WAL write-lock).
//   - If the symbol-graph tables are absent, returns Err immediately.
//   - Walks source files via ignore::WalkBuilder so .gitignore is respected.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::dkg::{
    ext_to_lang_id, BashGrammar, CGrammar, CSharpGrammar, CppGrammar, GoGrammar, JavaGrammar,
    JavaScriptGrammar, PhpGrammar, PythonGrammar, RubyGrammar, RustGrammar, ScalaGrammar,
    SvelteGrammar, SwiftGrammar, TsxGrammar, TypeScriptGrammar,
};
use crate::symbol_graph::{
    sharding::{all_name_shard_keys, name_shard_key, reverse_shard_bucket},
    store, types::{SymbolGraphFilePayload, SymbolGraphMeta, SymbolRef},
};
use crate::symbol_graph::extraction::{extract_symbols_and_calls, raw_calls_to_unresolved_edges};
use crate::symbol_graph::resolution::{build_file_dep_map, compute_unresolved_pct, resolve_call_sites};
use crate::KnowledgeError;
use crate::LanguageGrammar;

// ---------------------------------------------------------------------------
// Public constants
// ---------------------------------------------------------------------------

/// Number of changed+removed files at or below which an incremental update is
/// performed. Above this threshold a full rebuild is triggered instead.
///
/// Mirrors SocratiCode's INCREMENTAL_SYMBOL_THRESHOLD. Spec-locked: do not
/// change without updating all callers and the Phase 4 contract test.
pub const INCREMENTAL_SYMBOL_THRESHOLD: usize = 50;

/// Symbol graph schema version written to `symbol_graph_meta.schema_version`.
const SYMBOL_GRAPH_SCHEMA_VERSION: i64 = 3;

// ---------------------------------------------------------------------------
// Public return types
// ---------------------------------------------------------------------------

/// Summary of a full symbol-graph build.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildSummary {
    /// Number of source files indexed.
    pub files: usize,
    /// Total number of SymbolNode rows written.
    pub symbols: usize,
    /// Total number of SymbolEdge rows written.
    pub edges: usize,
    /// Percentage (0.0–100.0) of edges whose confidence is Unresolved.
    pub unresolved_pct: f64,
}

/// Summary of an incremental symbol-graph update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSummary {
    /// Number of files whose payloads were re-extracted and upserted.
    pub files_changed: usize,
    /// Number of files whose payloads were deleted.
    pub files_removed: usize,
    /// `true` when the change set exceeded `INCREMENTAL_SYMBOL_THRESHOLD` and
    /// a full rebuild was triggered instead.
    pub rebuild_triggered: bool,
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Build the full symbol graph for `project_id` from source files under
/// `project_root`, using the DKG database at `dkg_db_path` for import edges.
///
/// The caller MUST drop the `DkgEngine` instance before calling this function
/// to avoid WAL write-lock contention on `dkg_db_path`.
pub fn build_symbol_graph(
    project_id: &str,
    project_root: &Path,
    dkg_db_path: &Path,
    clock_now_ms: i64,
) -> Result<BuildSummary, KnowledgeError> {
    let mut conn = open_write_conn(dkg_db_path)?;

    // ------------------------------------------------------------------
    // 1. Collect source files under project_root (respects .gitignore).
    // ------------------------------------------------------------------
    let source_files = collect_source_files(project_root);

    // ------------------------------------------------------------------
    // 2. Extract symbols and calls per file.
    // ------------------------------------------------------------------
    let mut symbols_by_file: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut outgoing_calls_by_file: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let mut payloads: Vec<SymbolGraphFilePayload> = Vec::new();

    for (file_path, lang_id) in &source_files {
        let source = match std::fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    target = "symbol_graph",
                    file = %file_path.display(),
                    error = %e,
                    "skipping unreadable file"
                );
                continue;
            }
        };

        let grammar: Box<dyn LanguageGrammar> = match make_grammar(lang_id) {
            Some(g) => g,
            None => continue,
        };

        let file_str = file_path.to_string_lossy().into_owned();
        let extracted = extract_symbols_and_calls(&source, grammar.as_ref(), &file_str);

        let content_hash = blake3::hash(source.as_bytes()).to_hex().to_string();

        let unresolved_edges = raw_calls_to_unresolved_edges(&extracted.raw_calls);

        symbols_by_file.insert(file_str.clone(), extracted.symbols.clone());
        outgoing_calls_by_file.insert(file_str.clone(), unresolved_edges.clone());

        payloads.push(SymbolGraphFilePayload {
            file: file_str,
            language: lang_id.to_string(),
            content_hash,
            symbols: extracted.symbols,
            outgoing_calls: unresolved_edges,
        });
    }

    // ------------------------------------------------------------------
    // 3. Build file dependency map from DKG edges table.
    // ------------------------------------------------------------------
    let deps_by_file = build_file_dep_map(&conn)
        .unwrap_or_else(|e| {
            tracing::warn!(
                target = "symbol_graph",
                error = %e,
                "build_file_dep_map failed; continuing without import edges"
            );
            BTreeMap::new()
        });

    // ------------------------------------------------------------------
    // 4. Resolve call sites (mutates outgoing_calls_by_file in place).
    // ------------------------------------------------------------------
    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing_calls_by_file);

    // ------------------------------------------------------------------
    // 5. Compute metrics.
    // ------------------------------------------------------------------
    let unresolved_pct = compute_unresolved_pct(&outgoing_calls_by_file);

    let total_symbols: usize = symbols_by_file.values().map(|v| v.len()).sum();
    let total_edges: usize = outgoing_calls_by_file.values().map(|v| v.len()).sum();
    let file_count = payloads.len();

    // ------------------------------------------------------------------
    // 6. Persist: update payloads with resolved edges, then write.
    // ------------------------------------------------------------------
    for payload in &mut payloads {
        if let Some(resolved_edges) = outgoing_calls_by_file.get(&payload.file) {
            payload.outgoing_calls = resolved_edges.clone();
        }
        store::upsert_file_payload(&mut conn, payload)?;
    }

    // ------------------------------------------------------------------
    // 7. Build and persist name-index shards (27 shards).
    // ------------------------------------------------------------------
    let name_shards = build_name_shards(&symbols_by_file);
    for shard_key in all_name_shard_keys() {
        let entries = name_shards.get(&shard_key).cloned().unwrap_or_default();
        store::upsert_name_shard(&mut conn, shard_key, &entries)?;
    }

    // ------------------------------------------------------------------
    // 8. Build and persist reverse-index shards (256 buckets).
    // ------------------------------------------------------------------
    let reverse_shards = build_reverse_shards(&outgoing_calls_by_file);
    for bucket in 0u8..=u8::MAX {
        let entries = reverse_shards.get(&bucket).cloned().unwrap_or_default();
        store::upsert_reverse_shard(&mut conn, bucket, &entries)?;
    }

    // Handle bucket 255 — already covered by the loop above since 0u8..=255u8
    // iterates all 256 values. The loop uses u8::MAX (255) as the final value.

    // ------------------------------------------------------------------
    // 9. Persist symbol_graph_meta.
    // ------------------------------------------------------------------
    let meta = SymbolGraphMeta {
        project_id: project_id.to_string(),
        symbol_count: i64::try_from(total_symbols).unwrap_or(i64::MAX),
        edge_count: i64::try_from(total_edges).unwrap_or(i64::MAX),
        file_count: i64::try_from(file_count).unwrap_or(i64::MAX),
        unresolved_edge_pct: unresolved_pct,
        // Caller injects the timestamp (ClockProvider invariant — no raw
        // SystemTime::now() inside the library; matches DkgEngine::new pattern).
        built_at: clock_now_ms,
        schema_version: SYMBOL_GRAPH_SCHEMA_VERSION,
    };
    store::upsert_meta(&mut conn, &meta)?;

    tracing::info!(
        target = "symbol_graph",
        files = file_count,
        symbols = total_symbols,
        edges = total_edges,
        unresolved_pct = unresolved_pct,
        "Symbol graph persisted"
    );

    Ok(BuildSummary {
        files: file_count,
        symbols: total_symbols,
        edges: total_edges,
        unresolved_pct,
    })
}

/// Incrementally update the symbol graph for files that changed or were removed.
///
/// If `changed_paths.len() + removed_paths.len() > INCREMENTAL_SYMBOL_THRESHOLD`,
/// falls back to a full `build_symbol_graph` call.
pub fn update_changed_files_symbol_graph(
    project_id: &str,
    changed_paths: &[PathBuf],
    removed_paths: &[PathBuf],
    dkg_db_path: &Path,
    project_root: &Path,
    clock_now_ms: i64,
) -> Result<UpdateSummary, KnowledgeError> {
    let total_changes = changed_paths.len() + removed_paths.len();

    // ------------------------------------------------------------------
    // Fast path: too many changes — full rebuild.
    // ------------------------------------------------------------------
    if total_changes > INCREMENTAL_SYMBOL_THRESHOLD {
        let _summary = build_symbol_graph(project_id, project_root, dkg_db_path, clock_now_ms)?;
        return Ok(UpdateSummary {
            files_changed: changed_paths.len(),
            files_removed: removed_paths.len(),
            rebuild_triggered: true,
        });
    }

    // ------------------------------------------------------------------
    // Incremental path.
    // ------------------------------------------------------------------
    let mut conn = open_write_conn(dkg_db_path)?;

    // Get global deps map (needed for resolution of changed files).
    let deps_by_file = build_file_dep_map(&conn).unwrap_or_else(|e| {
        tracing::warn!(
            target = "symbol_graph",
            error = %e,
            "build_file_dep_map failed in incremental update"
        );
        BTreeMap::new()
    });

    // Load existing symbols for ALL files (needed to resolve cross-file calls).
    let all_files = store::list_all_files(&conn)?;
    let mut symbols_by_file: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for file in &all_files {
        if let Some(payload) = store::load_file_payload(&conn, file)? {
            symbols_by_file.insert(file.clone(), payload.symbols);
        }
    }

    // ------------------------------------------------------------------
    // Process changed files: re-extract and upsert.
    // ------------------------------------------------------------------
    let mut files_changed = 0usize;
    for path in changed_paths {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    target = "symbol_graph",
                    file = %path.display(),
                    error = %e,
                    "incremental: skipping unreadable changed file"
                );
                continue;
            }
        };

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang_id = match ext_to_lang_id(ext) {
            Some(id) => id,
            None => continue,
        };
        let grammar: Box<dyn LanguageGrammar> = match make_grammar(lang_id) {
            Some(g) => g,
            None => continue,
        };

        let file_str = path.to_string_lossy().into_owned();
        let extracted = extract_symbols_and_calls(&source, grammar.as_ref(), &file_str);
        let content_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
        let unresolved_edges = raw_calls_to_unresolved_edges(&extracted.raw_calls);

        // Update global symbol map for resolution.
        symbols_by_file.insert(file_str.clone(), extracted.symbols.clone());

        // Resolve just this file's calls using the global symbol map.
        let mut calls_map: BTreeMap<String, Vec<_>> = BTreeMap::new();
        calls_map.insert(file_str.clone(), unresolved_edges);
        resolve_call_sites(&deps_by_file, &symbols_by_file, &mut calls_map);
        let resolved_edges = calls_map.remove(&file_str).unwrap_or_default();

        let payload = SymbolGraphFilePayload {
            file: file_str,
            language: lang_id.to_string(),
            content_hash,
            symbols: extracted.symbols,
            outgoing_calls: resolved_edges,
        };
        store::upsert_file_payload(&mut conn, &payload)?;
        files_changed += 1;
    }

    // ------------------------------------------------------------------
    // Collect name shard keys touched by removed files BEFORE deleting them
    // from the in-memory map; otherwise their symbol names disappear and the
    // shard never gets rebuilt — leaving stale entries on disk.
    // ------------------------------------------------------------------
    let mut touched_name_keys: BTreeSet<char> = BTreeSet::new();
    for path in changed_paths.iter().chain(removed_paths.iter()) {
        let file_str = path.to_string_lossy().into_owned();
        if let Some(syms) = symbols_by_file.get(&file_str) {
            for sym in syms {
                touched_name_keys.insert(name_shard_key(&sym.name));
            }
        }
    }

    // ------------------------------------------------------------------
    // Process removed files: delete payload rows.
    // ------------------------------------------------------------------
    let mut files_removed = 0usize;
    for path in removed_paths {
        let file_str = path.to_string_lossy().into_owned();
        store::delete_file_payload(&mut conn, &file_str)?;
        symbols_by_file.remove(&file_str);
        files_removed += 1;
    }

    // ------------------------------------------------------------------
    // Rebuild affected shards and meta.
    // ------------------------------------------------------------------
    // For simplicity and correctness, rebuild the affected name shards and
    // reverse shards from the full in-memory symbol map (which was loaded at
    // the start). This avoids partial-shard corruption on concurrent updates.
    let name_shards = build_name_shards(&symbols_by_file);
    for key in touched_name_keys {
        let entries = name_shards.get(&key).cloned().unwrap_or_default();
        store::upsert_name_shard(&mut conn, key, &entries)?;
    }

    // Rebuild reverse shards for touched callee files.
    // We need a full outgoing_calls_by_file to rebuild them correctly.
    let mut outgoing_calls_by_file: BTreeMap<String, Vec<_>> = BTreeMap::new();
    let current_files = store::list_all_files(&conn)?;
    for file in &current_files {
        if let Some(payload) = store::load_file_payload(&conn, file)? {
            outgoing_calls_by_file.insert(file.clone(), payload.outgoing_calls);
        }
    }
    let reverse_shards = build_reverse_shards(&outgoing_calls_by_file);

    // Find which reverse buckets are touched.
    let mut touched_buckets: BTreeSet<u8> = BTreeSet::new();
    for path in changed_paths.iter().chain(removed_paths.iter()) {
        let file_str = path.to_string_lossy().into_owned();
        touched_buckets.insert(reverse_shard_bucket(&file_str));
    }
    for bucket in touched_buckets {
        let entries = reverse_shards.get(&bucket).cloned().unwrap_or_default();
        store::upsert_reverse_shard(&mut conn, bucket, &entries)?;
    }

    // Update meta with new totals.
    let total_symbols: usize = symbols_by_file.values().map(|v| v.len()).sum();
    let total_edges: usize = outgoing_calls_by_file.values().map(|v| v.len()).sum();
    let unresolved_pct = compute_unresolved_pct(&outgoing_calls_by_file);
    let meta = SymbolGraphMeta {
        project_id: project_id.to_string(),
        symbol_count: i64::try_from(total_symbols).unwrap_or(i64::MAX),
        edge_count: i64::try_from(total_edges).unwrap_or(i64::MAX),
        file_count: i64::try_from(current_files.len()).unwrap_or(i64::MAX),
        unresolved_edge_pct: unresolved_pct,
        built_at: clock_now_ms,
        schema_version: SYMBOL_GRAPH_SCHEMA_VERSION,
    };
    store::upsert_meta(&mut conn, &meta)?;

    Ok(UpdateSummary {
        files_changed,
        files_removed,
        rebuild_triggered: false,
    })
}

// ---------------------------------------------------------------------------
// Private — file walking
// ---------------------------------------------------------------------------

/// Walk `project_root` respecting .gitignore and collect (path, lang_id) pairs.
fn collect_source_files(project_root: &Path) -> Vec<(PathBuf, &'static str)> {
    let mut result: Vec<(PathBuf, &'static str)> = Vec::new();

    let walker = ignore::WalkBuilder::new(project_root)
        // .hidden(true) skips entries starting with '.' — so the indexer
        // does not descend into .cargo/.svelte-kit/.next/.git/etc that may
        // contain generated source files we explicitly do not want indexed.
        .hidden(true)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .build();

    for entry in walker.flatten() {
        let path = entry.into_path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if let Some(lang_id) = ext_to_lang_id(ext) {
            result.push((path, lang_id));
        }
    }

    // Sort for deterministic ordering.
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

// ---------------------------------------------------------------------------
// Private — grammar factory
// ---------------------------------------------------------------------------

/// Instantiate the grammar for `lang_id` as a boxed trait object.
///
/// Returns `None` for any language id not registered in DkgEngine. Grammar
/// instantiation is lightweight (unit structs); no tree-sitter parser is
/// created here.
fn make_grammar(lang_id: &str) -> Option<Box<dyn LanguageGrammar>> {
    match lang_id {
        "rust" => Some(Box::new(RustGrammar)),
        "python" => Some(Box::new(PythonGrammar)),
        "go" => Some(Box::new(GoGrammar)),
        "typescript" => Some(Box::new(TypeScriptGrammar)),
        "tsx" => Some(Box::new(TsxGrammar)),
        "svelte" => Some(Box::new(SvelteGrammar)),
        "java" => Some(Box::new(JavaGrammar)),
        "csharp" => Some(Box::new(CSharpGrammar)),
        "ruby" => Some(Box::new(RubyGrammar)),
        "javascript" => Some(Box::new(JavaScriptGrammar)),
        "c" => Some(Box::new(CGrammar)),
        "cpp" => Some(Box::new(CppGrammar)),
        "scala" => Some(Box::new(ScalaGrammar)),
        "php" => Some(Box::new(PhpGrammar)),
        "swift" => Some(Box::new(SwiftGrammar)),
        "bash" => Some(Box::new(BashGrammar)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Private — shard builders
// ---------------------------------------------------------------------------

/// Build the 27 name-index shards from `symbols_by_file`.
///
/// Returns `BTreeMap<shard_key, BTreeMap<symbol_name, Vec<SymbolRef>>>`.
fn build_name_shards(
    symbols_by_file: &BTreeMap<String, Vec<crate::symbol_graph::types::SymbolNode>>,
) -> BTreeMap<char, BTreeMap<String, Vec<SymbolRef>>> {
    let mut shards: BTreeMap<char, BTreeMap<String, Vec<SymbolRef>>> = BTreeMap::new();

    for (file, symbols) in symbols_by_file {
        for sym in symbols {
            let key = name_shard_key(&sym.name);
            let shard = shards.entry(key).or_default();
            let refs = shard.entry(sym.name.clone()).or_default();
            refs.push(SymbolRef {
                file: file.clone(),
                id: sym.id.clone(),
            });
        }
    }

    // Sort ref vectors for determinism.
    for shard in shards.values_mut() {
        for refs in shard.values_mut() {
            refs.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    shards
}

/// Build the 256 reverse-index shards from `outgoing_calls_by_file`.
///
/// Returns `BTreeMap<bucket, BTreeMap<callee_file, BTreeSet<caller_file>>>`.
fn build_reverse_shards(
    outgoing_calls_by_file: &BTreeMap<String, Vec<crate::symbol_graph::types::SymbolEdge>>,
) -> BTreeMap<u8, BTreeMap<String, BTreeSet<String>>> {
    let mut shards: BTreeMap<u8, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();

    for (caller_file, edges) in outgoing_calls_by_file {
        for edge in edges {
            // Use callee_candidates to determine callee files.
            for callee_id in &edge.callee_candidates {
                // callee_id format: "<file>::<name>#L<line>"
                if let Some(callee_file) = callee_id.split("::").next() {
                    if callee_file != caller_file {
                        let bucket = reverse_shard_bucket(callee_file);
                        let shard = shards.entry(bucket).or_default();
                        let callers = shard.entry(callee_file.to_string()).or_default();
                        callers.insert(caller_file.clone());
                    }
                }
            }
        }
    }

    shards
}

// ---------------------------------------------------------------------------
// Private — connection helper
// ---------------------------------------------------------------------------

/// Open `dkg_db_path` for read-write with WAL journal mode.
fn open_write_conn(dkg_db_path: &Path) -> Result<Connection, KnowledgeError> {
    let conn = Connection::open_with_flags(
        dkg_db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    Ok(conn)
}
