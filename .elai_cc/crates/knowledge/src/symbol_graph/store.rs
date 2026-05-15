// B-29 Phase 3 — persistence read/write layer for the symbol graph.
//
// All functions accept a `&Connection` (reads) or `&mut Connection` (writes).
// The caller is responsible for opening / closing the connection.
//
// Column names follow the spec contract (`*_msgpack`) even though the actual
// binary encoding is postcard, not MessagePack — the names are immutable
// SQLite contract.
//
// Write helpers are created here for Phase 4 (build_symbol_graph) even though
// Phase 3 only exercises the read path.  This avoids re-scoping when Phase 4
// wires in.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{Connection, OptionalExtension};

use crate::KnowledgeError;

use super::types::{SymbolGraphFilePayload, SymbolGraphMeta, SymbolNode, SymbolEdge, SymbolRef};

// ---------------------------------------------------------------------------
// Read helpers
// ---------------------------------------------------------------------------

/// Load the `symbol_graph_meta` row for `project_id`.
///
/// Returns `Ok(None)` when no row exists (project not yet indexed).
pub fn load_meta(
    conn: &Connection,
    project_id: &str,
) -> Result<Option<SymbolGraphMeta>, KnowledgeError> {
    let result = conn
        .query_row(
            "SELECT project_id, symbol_count, edge_count, file_count,
                    unresolved_edge_pct, built_at, schema_version
             FROM symbol_graph_meta
             WHERE project_id = ?1",
            rusqlite::params![project_id],
            |row| {
                Ok(SymbolGraphMeta {
                    project_id: row.get(0)?,
                    symbol_count: row.get(1)?,
                    edge_count: row.get(2)?,
                    file_count: row.get(3)?,
                    unresolved_edge_pct: row.get(4)?,
                    built_at: row.get(5)?,
                    schema_version: row.get(6)?,
                })
            },
        )
        .optional()?;
    Ok(result)
}

/// Load all entries for a single name-index shard identified by `shard_key`.
///
/// Returns an empty map when the shard has no rows.
pub fn load_name_shard(
    conn: &Connection,
    shard_key: char,
) -> Result<BTreeMap<String, Vec<SymbolRef>>, KnowledgeError> {
    let shard_str = shard_key.to_string();
    let mut stmt = conn.prepare(
        "SELECT symbol_name, refs_msgpack
         FROM symbol_name_index
         WHERE shard_key = ?1",
    )?;

    let mut map: BTreeMap<String, Vec<SymbolRef>> = BTreeMap::new();
    let rows = stmt.query_map(rusqlite::params![shard_str], |row| {
        let name: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((name, blob))
    })?;

    for row_result in rows {
        let (name, blob) = row_result?;
        let refs: Vec<SymbolRef> = postcard::from_bytes(&blob)
            .map_err(|e| KnowledgeError::Graph(format!("name shard decode error: {e}")))?;
        map.insert(name, refs);
    }

    Ok(map)
}

/// Load all entries for a single reverse-index shard identified by `shard_bucket`.
///
/// Returns an empty map when the bucket has no rows.
pub fn load_reverse_shard(
    conn: &Connection,
    shard_bucket: u8,
) -> Result<BTreeMap<String, BTreeSet<String>>, KnowledgeError> {
    let bucket_i64 = i64::from(shard_bucket);
    let mut stmt = conn.prepare(
        "SELECT callee_file, caller_files_msgpack
         FROM symbol_reverse_index
         WHERE shard_bucket = ?1",
    )?;

    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let rows = stmt.query_map(rusqlite::params![bucket_i64], |row| {
        let callee: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((callee, blob))
    })?;

    for row_result in rows {
        let (callee, blob) = row_result?;
        let callers_vec: Vec<String> = postcard::from_bytes(&blob)
            .map_err(|e| KnowledgeError::Graph(format!("reverse shard decode error: {e}")))?;
        let callers_set: BTreeSet<String> = callers_vec.into_iter().collect();
        map.insert(callee, callers_set);
    }

    Ok(map)
}

/// Load a per-file payload from `symbol_file_payload`.
///
/// Returns `Ok(None)` when the file has not been indexed.
pub fn load_file_payload(
    conn: &Connection,
    file: &str,
) -> Result<Option<SymbolGraphFilePayload>, KnowledgeError> {
    let result = conn
        .query_row(
            "SELECT file, language, content_hash, symbols_msgpack, edges_msgpack
             FROM symbol_file_payload
             WHERE file = ?1",
            rusqlite::params![file],
            |row| {
                let file_col: String = row.get(0)?;
                let language: String = row.get(1)?;
                let content_hash: String = row.get(2)?;
                let symbols_blob: Vec<u8> = row.get(3)?;
                let edges_blob: Vec<u8> = row.get(4)?;
                Ok((file_col, language, content_hash, symbols_blob, edges_blob))
            },
        )
        .optional()?;

    let Some((file_col, language, content_hash, symbols_blob, edges_blob)) = result else {
        return Ok(None);
    };

    let symbols: Vec<SymbolNode> = postcard::from_bytes(&symbols_blob)
        .map_err(|e| KnowledgeError::Graph(format!("symbols decode error for {file}: {e}")))?;
    let outgoing_calls: Vec<SymbolEdge> = postcard::from_bytes(&edges_blob)
        .map_err(|e| KnowledgeError::Graph(format!("edges decode error for {file}: {e}")))?;

    Ok(Some(SymbolGraphFilePayload {
        file: file_col,
        language,
        content_hash,
        symbols,
        outgoing_calls,
    }))
}

/// Return all distinct file paths present in `symbol_file_payload`.
pub fn list_all_files(conn: &Connection) -> Result<Vec<String>, KnowledgeError> {
    let mut stmt = conn.prepare("SELECT DISTINCT file FROM symbol_file_payload ORDER BY file")?;
    let files: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(files)
}

// ---------------------------------------------------------------------------
// Write helpers (needed by Phase 4 build_symbol_graph)
// ---------------------------------------------------------------------------

/// Insert or replace the `symbol_graph_meta` row for this project.
pub fn upsert_meta(
    conn: &mut Connection,
    meta: &SymbolGraphMeta,
) -> Result<(), KnowledgeError> {
    conn.execute(
        "INSERT OR REPLACE INTO symbol_graph_meta
             (project_id, symbol_count, edge_count, file_count,
              unresolved_edge_pct, built_at, schema_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            meta.project_id,
            meta.symbol_count,
            meta.edge_count,
            meta.file_count,
            meta.unresolved_edge_pct,
            meta.built_at,
            meta.schema_version,
        ],
    )?;
    Ok(())
}

/// Replace all name-index entries for `shard_key` with `entries`.
///
/// Transactional: DELETE all existing rows for the shard, then INSERT new ones.
pub fn upsert_name_shard(
    conn: &mut Connection,
    shard_key: char,
    entries: &BTreeMap<String, Vec<SymbolRef>>,
) -> Result<(), KnowledgeError> {
    let shard_str = shard_key.to_string();
    let tx = conn.transaction()?;

    tx.execute(
        "DELETE FROM symbol_name_index WHERE shard_key = ?1",
        rusqlite::params![shard_str],
    )?;

    for (name, refs) in entries {
        let blob = postcard::to_allocvec(refs)
            .map_err(|e| KnowledgeError::Graph(format!("name shard encode error: {e}")))?;
        tx.execute(
            "INSERT INTO symbol_name_index (shard_key, symbol_name, refs_msgpack)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![shard_str, name, blob],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Replace all reverse-index entries for `shard_bucket` with `entries`.
///
/// Transactional: DELETE all existing rows for the bucket, then INSERT new ones.
pub fn upsert_reverse_shard(
    conn: &mut Connection,
    shard_bucket: u8,
    entries: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), KnowledgeError> {
    let bucket_i64 = i64::from(shard_bucket);
    let tx = conn.transaction()?;

    tx.execute(
        "DELETE FROM symbol_reverse_index WHERE shard_bucket = ?1",
        rusqlite::params![bucket_i64],
    )?;

    for (callee_file, caller_set) in entries {
        let callers_vec: Vec<&String> = caller_set.iter().collect();
        let blob = postcard::to_allocvec(&callers_vec)
            .map_err(|e| KnowledgeError::Graph(format!("reverse shard encode error: {e}")))?;
        tx.execute(
            "INSERT INTO symbol_reverse_index (shard_bucket, callee_file, caller_files_msgpack)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![bucket_i64, callee_file, blob],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Insert or replace the per-file payload for `payload.file`.
pub fn upsert_file_payload(
    conn: &mut Connection,
    payload: &SymbolGraphFilePayload,
) -> Result<(), KnowledgeError> {
    let symbols_blob = postcard::to_allocvec(&payload.symbols)
        .map_err(|e| KnowledgeError::Graph(format!("symbols encode error: {e}")))?;
    let edges_blob = postcard::to_allocvec(&payload.outgoing_calls)
        .map_err(|e| KnowledgeError::Graph(format!("edges encode error: {e}")))?;

    conn.execute(
        "INSERT OR REPLACE INTO symbol_file_payload
             (file, language, content_hash, symbols_msgpack, edges_msgpack)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            payload.file,
            payload.language,
            payload.content_hash,
            symbols_blob,
            edges_blob,
        ],
    )?;
    Ok(())
}

/// Delete the per-file payload row for `file`.
pub fn delete_file_payload(
    conn: &mut Connection,
    file: &str,
) -> Result<(), KnowledgeError> {
    conn.execute(
        "DELETE FROM symbol_file_payload WHERE file = ?1",
        rusqlite::params![file],
    )?;
    Ok(())
}
