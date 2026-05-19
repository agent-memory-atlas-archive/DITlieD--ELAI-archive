//! DKG-backed implementation of `DkgQuerier`.
//!
//! Opens a read-only SQLite connection to the DKG database for each query.
//! BFS traversal on `dkg_edges` with cycle detection via `HashSet<i64>`.

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, params};

use elai_rules::predicate::{
    DkgDataFlowEdgeRef, DkgEdgeRef, DkgError, DkgNodeRef, DkgQuerier, DkgSubgraphSnapshot,
    ReachabilityMatch, ReachabilityParams,
};

/// Queries the DKG (dependency knowledge graph) for reachability paths.
///
/// Holds only the path to the DKG SQLite database. Each query opens a fresh
/// read-only connection so there is no contention with `DkgEngine`'s writer.
pub struct DkgQuerierAdapter {
    db_path: PathBuf,
}

impl DkgQuerierAdapter {
    /// Create a new adapter pointing at `db_path`.
    ///
    /// Returns `Err` if the database file does not exist or cannot be opened
    /// read-only. The caller should fall back to `EmptyDkgQuerier` on error.
    pub fn new(db_path: &Path) -> Result<Self, DkgError> {
        // Verify the db is openable before committing to this adapter.
        let _conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| DkgError::QueryFailed(format!("cannot open DKG db: {e}")))?;
        Ok(Self {
            db_path: db_path.to_path_buf(),
        })
    }

    fn open_readonly(&self) -> Result<Connection, DkgError> {
        Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| DkgError::QueryFailed(format!("open read-only: {e}")))
    }
}

#[derive(Debug)]
struct BfsNode {
    node_id: i64,
    path: Vec<String>,
    edge_labels: Vec<String>,
    depth: u8,
}

impl DkgQuerier for DkgQuerierAdapter {
    fn query_reachability(
        &self,
        params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        self.validate_params(params)?;

        let conn = self.open_readonly()?;

        // Find source nodes matching source_pattern (LIKE match on name).
        let mut source_stmt = conn
            .prepare("SELECT id, name FROM dkg_nodes WHERE name LIKE ?1")
            .map_err(|e| DkgError::QueryFailed(format!("prepare source: {e}")))?;

        let source_nodes: Vec<(i64, String)> = source_stmt
            .query_map(params![&params.source_pattern], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| DkgError::QueryFailed(format!("query source: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DkgError::QueryFailed(format!("row error (source): {e}")))?;

        // Build edge type filter clause.
        let edge_filter = if params.edge_types.is_empty() {
            String::new()
        } else {
            let placeholders: Vec<String> = params
                .edge_types
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect();
            format!(" AND edge_type IN ({})", placeholders.join(", "))
        };

        let edge_sql = format!(
            "SELECT target_id, edge_type FROM dkg_edges WHERE source_id = ?1{}",
            edge_filter
        );

        let mut results = Vec::new();
        let mut edge_stmt = conn
            .prepare(&edge_sql)
            .map_err(|e| DkgError::QueryFailed(format!("prepare edge: {e}")))?;

        for (source_id, source_name) in &source_nodes {
            let mut visited = HashSet::new();
            visited.insert(*source_id);

            let mut queue = VecDeque::new();
            queue.push_back(BfsNode {
                node_id: *source_id,
                path: vec![source_name.clone()],
                edge_labels: Vec::new(),
                depth: 0,
            });

            while let Some(current) = queue.pop_front() {
                if current.depth >= params.max_depth {
                    continue;
                }

                // Build params: source_id + optional edge_type filters.
                let mut edge_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
                edge_params.push(Box::new(current.node_id));
                for et in &params.edge_types {
                    edge_params.push(Box::new(et.clone()));
                }

                let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                    edge_params.iter().map(|p| p.as_ref()).collect();

                let neighbors: Vec<(i64, String)> = edge_stmt
                    .query_map(param_refs.as_slice(), |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    })
                    .map_err(|e| DkgError::QueryFailed(format!("query edges: {e}")))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| DkgError::QueryFailed(format!("row error (edges): {e}")))?;

                for (target_id, edge_type) in neighbors {
                    if !visited.insert(target_id) {
                        continue; // cycle guard
                    }

                    // Resolve target node name.
                    let target_name: String = conn
                        .query_row(
                            "SELECT name FROM dkg_nodes WHERE id = ?1",
                            params![target_id],
                            |row| row.get(0),
                        )
                        .map_err(|e| DkgError::QueryFailed(format!("resolve node: {e}")))?;

                    let mut new_path = current.path.clone();
                    new_path.push(target_name.clone());

                    let mut new_labels = current.edge_labels.clone();
                    new_labels.push(edge_type);

                    // Check if target matches sink_pattern.
                    let is_sink = match_like(&target_name, &params.sink_pattern);

                    // Check must_traverse constraint if present.
                    let traverses_required = params
                        .must_traverse
                        .as_ref()
                        .map_or(true, |req| new_path.iter().any(|n| match_like(n, req)));

                    // Collect matching paths for both must_not_exist and normal cases.
                    // For must_not_exist, the caller uses non-empty results to detect
                    // that a forbidden path exists.
                    if is_sink && traverses_required {
                        results.push(ReachabilityMatch {
                            path: new_path.clone(),
                            edge_labels: new_labels.clone(),
                        });
                    }

                    queue.push_back(BfsNode {
                        node_id: target_id,
                        path: new_path,
                        edge_labels: new_labels,
                        depth: current.depth + 1,
                    });
                }
            }
        }

        Ok(results)
    }

    fn validate_params(&self, params: &ReachabilityParams) -> Result<(), DkgError> {
        if params.source_pattern.is_empty() {
            return Err(DkgError::InvalidParams(
                "source_pattern must not be empty".to_string(),
            ));
        }
        if params.sink_pattern.is_empty() {
            return Err(DkgError::InvalidParams(
                "sink_pattern must not be empty".to_string(),
            ));
        }
        if params.max_depth > 20 {
            return Err(DkgError::InvalidParams(format!(
                "max_depth {} exceeds limit of 20",
                params.max_depth
            )));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // C-30: production overrides for the four new DKG-query trait methods.
    // -----------------------------------------------------------------------

    fn query_callers(
        &self,
        target_symbol: &str,
        max_depth: u8,
    ) -> Result<Vec<DkgNodeRef>, DkgError> {
        if target_symbol.trim().is_empty() {
            return Err(DkgError::InvalidParams(
                "target_symbol must not be empty".to_string(),
            ));
        }
        let depth = max_depth.min(MAX_C30_DEPTH);
        let conn = self.open_readonly()?;
        // Inbound CALLS edges. DKG edge_type strings may be lowercase or
        // uppercase per builder — accept both (DRIFT-12 normalisation).
        let edge_filter = "edge_type IN ('CALLS', 'calls')";
        let target_id = lookup_node_id(&conn, target_symbol)?;
        let Some(target_id) = target_id else {
            return Ok(Vec::new());
        };
        let visited = bfs_neighbors(
            &conn,
            target_id,
            depth,
            BfsDirection::Inbound,
            edge_filter,
        )?;
        let mut nodes = Vec::with_capacity(visited.len());
        for id in visited {
            if let Some(n) = fetch_node_by_id(&conn, id)? {
                nodes.push(n);
            }
        }
        Ok(nodes)
    }

    fn query_dependents(
        &self,
        target_symbol: &str,
        max_depth: u8,
    ) -> Result<Vec<DkgNodeRef>, DkgError> {
        if target_symbol.trim().is_empty() {
            return Err(DkgError::InvalidParams(
                "target_symbol must not be empty".to_string(),
            ));
        }
        let depth = max_depth.min(MAX_C30_DEPTH);
        let conn = self.open_readonly()?;
        // Outbound IMPORTS / USES (both case variants — see DRIFT-12).
        let edge_filter =
            "edge_type IN ('IMPORTS', 'imports', 'USES', 'uses')";
        let target_id = lookup_node_id(&conn, target_symbol)?;
        let Some(target_id) = target_id else {
            return Ok(Vec::new());
        };
        let visited = bfs_neighbors(
            &conn,
            target_id,
            depth,
            BfsDirection::Outbound,
            edge_filter,
        )?;
        let mut nodes = Vec::with_capacity(visited.len());
        for id in visited {
            if let Some(n) = fetch_node_by_id(&conn, id)? {
                nodes.push(n);
            }
        }
        Ok(nodes)
    }

    fn query_data_flow(
        &self,
        target_symbol: &str,
        max_depth: u8,
    ) -> Result<Vec<DkgDataFlowEdgeRef>, DkgError> {
        if target_symbol.trim().is_empty() {
            return Err(DkgError::InvalidParams(
                "target_symbol must not be empty".to_string(),
            ));
        }
        let depth = max_depth.min(MAX_C30_DEPTH);
        let conn = self.open_readonly()?;
        let target_id = lookup_node_id(&conn, target_symbol)?;
        let Some(target_id) = target_id else {
            return Ok(Vec::new());
        };

        // BFS over dkg_data_flow edges (both directions). Each visited id
        // contributes outbound edges to the result.
        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(target_id);
        let mut queue: VecDeque<(i64, u8)> = VecDeque::new();
        queue.push_back((target_id, 0));
        let mut edges_out: Vec<DkgDataFlowEdgeRef> = Vec::new();

        while let Some((node_id, hop)) = queue.pop_front() {
            if hop >= depth {
                continue;
            }
            let mut stmt = conn
                .prepare(
                    "SELECT source_node_id, target_node_id, variable_name, flow_type \
                     FROM dkg_data_flow \
                     WHERE source_node_id = ?1 OR target_node_id = ?1",
                )
                .map_err(|e| DkgError::QueryFailed(format!("prepare data_flow: {e}")))?;
            let rows = stmt
                .query_map(params![node_id], |row| {
                    let src: i64 = row.get(0)?;
                    let tgt: i64 = row.get(1)?;
                    let var: String = row.get(2)?;
                    let ft: String = row.get(3)?;
                    Ok((src, tgt, var, ft))
                })
                .map_err(|e| DkgError::QueryFailed(format!("query data_flow: {e}")))?;
            for row in rows {
                let (src, tgt, var, ft) = row.map_err(|e| {
                    DkgError::QueryFailed(format!("row error (data_flow): {e}"))
                })?;
                let source_name = name_for_id(&conn, src).unwrap_or_default();
                let target_name = name_for_id(&conn, tgt).unwrap_or_default();
                if source_name.is_empty() && target_name.is_empty() {
                    continue;
                }
                edges_out.push(DkgDataFlowEdgeRef {
                    source: source_name,
                    target: target_name,
                    variable: var,
                    flow_type: ft,
                });
                let next = if src == node_id { tgt } else { src };
                if visited.insert(next) {
                    queue.push_back((next, hop + 1));
                }
            }
        }
        Ok(edges_out)
    }

    fn query_subgraph(
        &self,
        root_symbol: &str,
        depth: u8,
    ) -> Result<DkgSubgraphSnapshot, DkgError> {
        if root_symbol.trim().is_empty() {
            return Err(DkgError::InvalidParams(
                "root_symbol must not be empty".to_string(),
            ));
        }
        let depth = depth.min(MAX_C30_DEPTH);
        let conn = self.open_readonly()?;
        let root_id = lookup_node_id(&conn, root_symbol)?;
        let Some(root_id) = root_id else {
            return Ok(DkgSubgraphSnapshot {
                root: root_symbol.to_string(),
                depth,
                nodes: Vec::new(),
                edges: Vec::new(),
            });
        };

        let mut visited: HashSet<i64> = HashSet::new();
        visited.insert(root_id);
        let mut queue: VecDeque<(i64, u8)> = VecDeque::new();
        queue.push_back((root_id, 0));

        let mut nodes_out: Vec<DkgNodeRef> = Vec::new();
        let mut edges_out: Vec<DkgEdgeRef> = Vec::new();

        // Root node first.
        if let Some(n) = fetch_node_by_id(&conn, root_id)? {
            nodes_out.push(n);
        }

        while let Some((node_id, hop)) = queue.pop_front() {
            if hop >= depth {
                continue;
            }
            // Outbound edges.
            let mut stmt = conn
                .prepare(
                    "SELECT target_id, edge_type FROM dkg_edges WHERE source_id = ?1",
                )
                .map_err(|e| DkgError::QueryFailed(format!("prepare subgraph out: {e}")))?;
            let rows = stmt
                .query_map(params![node_id], |row| {
                    let target_id: i64 = row.get(0)?;
                    let edge_type: String = row.get(1)?;
                    Ok((target_id, edge_type))
                })
                .map_err(|e| DkgError::QueryFailed(format!("query subgraph out: {e}")))?;
            let source_name = name_for_id(&conn, node_id).unwrap_or_default();
            for row in rows {
                let (target_id, edge_type) = row.map_err(|e| {
                    DkgError::QueryFailed(format!("row error (subgraph out): {e}"))
                })?;
                let target_name = name_for_id(&conn, target_id).unwrap_or_default();
                edges_out.push(DkgEdgeRef {
                    source: source_name.clone(),
                    target: target_name,
                    edge_type,
                });
                if visited.insert(target_id) {
                    if let Some(n) = fetch_node_by_id(&conn, target_id)? {
                        nodes_out.push(n);
                    }
                    queue.push_back((target_id, hop + 1));
                }
            }
        }
        Ok(DkgSubgraphSnapshot {
            root: root_symbol.to_string(),
            depth,
            nodes: nodes_out,
            edges: edges_out,
        })
    }
}

// ---------------------------------------------------------------------------
// C-30 internal helpers
// ---------------------------------------------------------------------------

const MAX_C30_DEPTH: u8 = 8;

enum BfsDirection {
    Inbound,
    Outbound,
}

fn lookup_node_id(conn: &Connection, name: &str) -> Result<Option<i64>, DkgError> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM dkg_nodes WHERE name = ?1 LIMIT 1",
            params![name],
            |row| row.get(0),
        )
        .ok();
    Ok(id)
}

fn name_for_id(conn: &Connection, id: i64) -> Option<String> {
    conn.query_row(
        "SELECT name FROM dkg_nodes WHERE id = ?1",
        params![id],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

fn fetch_node_by_id(conn: &Connection, id: i64) -> Result<Option<DkgNodeRef>, DkgError> {
    let row = conn
        .query_row(
            "SELECT name, file_path, kind, start_line, end_line \
             FROM dkg_nodes WHERE id = ?1",
            params![id],
            |row| {
                let name: String = row.get(0)?;
                let file_path: String = row.get(1)?;
                let kind: String = row.get(2)?;
                let start_line: i64 = row.get(3)?;
                let end_line: i64 = row.get(4)?;
                Ok((name, file_path, kind, start_line, end_line))
            },
        )
        .ok();
    Ok(row.map(|(name, file_path, kind, start_line, end_line)| DkgNodeRef {
        name,
        file_path,
        kind,
        start_line: u32::try_from(start_line.max(0)).unwrap_or(0),
        end_line: u32::try_from(end_line.max(0)).unwrap_or(0),
    }))
}

/// BFS over `dkg_edges` from `start_id` up to `max_depth`. `edge_filter`
/// is the SQL WHERE-clause fragment that selects the relevant edge_type
/// strings (e.g. "edge_type IN ('CALLS', 'calls')"). `direction` selects
/// inbound (target = current, source = neighbour) vs outbound (source =
/// current, target = neighbour). Returns the set of visited node ids
/// (excluding `start_id`).
fn bfs_neighbors(
    conn: &Connection,
    start_id: i64,
    max_depth: u8,
    direction: BfsDirection,
    edge_filter: &str,
) -> Result<Vec<i64>, DkgError> {
    let (select_col, where_col) = match direction {
        BfsDirection::Inbound => ("source_id", "target_id"),
        BfsDirection::Outbound => ("target_id", "source_id"),
    };
    let sql = format!(
        "SELECT {select_col} FROM dkg_edges WHERE {where_col} = ?1 AND {edge_filter}",
    );
    let mut visited: HashSet<i64> = HashSet::new();
    visited.insert(start_id);
    let mut queue: VecDeque<(i64, u8)> = VecDeque::new();
    queue.push_back((start_id, 0));
    let mut out: Vec<i64> = Vec::new();
    while let Some((node_id, hop)) = queue.pop_front() {
        if hop >= max_depth {
            continue;
        }
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DkgError::QueryFailed(format!("prepare bfs: {e}")))?;
        let rows = stmt
            .query_map(params![node_id], |row| row.get::<_, i64>(0))
            .map_err(|e| DkgError::QueryFailed(format!("query bfs: {e}")))?;
        for row in rows {
            let next = row
                .map_err(|e| DkgError::QueryFailed(format!("row error (bfs): {e}")))?;
            if visited.insert(next) {
                out.push(next);
                queue.push_back((next, hop + 1));
            }
        }
    }
    Ok(out)
}

/// SQL LIKE-style pattern matching (% = any, _ = single char).
/// Uses iterative DP to avoid exponential recursion on patterns with many % wildcards.
fn match_like(value: &str, pattern: &str) -> bool {
    let val = value.as_bytes();
    let pat = pattern.as_bytes();
    let n = val.len();
    let m = pat.len();

    // dp[j] = true if pat[0..j] matches val[0..i] (rolling over i).
    let mut dp = vec![false; m + 1];
    dp[0] = true;

    // Initialize: leading % characters match empty string.
    for j in 0..m {
        if pat[j] == b'%' {
            dp[j + 1] = dp[j];
        }
    }

    for i in 0..n {
        let mut new_dp = vec![false; m + 1];
        // new_dp[0] = false: empty pattern cannot match non-empty prefix.
        for j in 0..m {
            match pat[j] {
                b'%' => {
                    // % matches zero chars (new_dp[j]) or one more char (dp[j + 1]).
                    new_dp[j + 1] = new_dp[j] || dp[j + 1];
                }
                b'_' => {
                    // _ matches exactly one character.
                    new_dp[j + 1] = dp[j];
                }
                c => {
                    new_dp[j + 1] = dp[j] && val[i] == c;
                }
            }
        }
        dp = new_dp;
    }

    dp[m]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_source() {
        let params = ReachabilityParams {
            source_pattern: String::new(),
            sink_pattern: "sink%".to_string(),
            must_traverse: None,
            must_not_exist: false,
            max_depth: 5,
            edge_types: vec![],
        };
        // We need a valid adapter to call validate_params, but since it's
        // a trait method we test the validation logic directly.
        let result = validate_params_standalone(&params);
        assert!(matches!(result, Err(DkgError::InvalidParams(_))));
    }

    #[test]
    fn validate_rejects_empty_sink() {
        let params = ReachabilityParams {
            source_pattern: "src%".to_string(),
            sink_pattern: String::new(),
            must_traverse: None,
            must_not_exist: false,
            max_depth: 5,
            edge_types: vec![],
        };
        let result = validate_params_standalone(&params);
        assert!(matches!(result, Err(DkgError::InvalidParams(_))));
    }

    #[test]
    fn validate_rejects_depth_over_20() {
        let params = ReachabilityParams {
            source_pattern: "src%".to_string(),
            sink_pattern: "sink%".to_string(),
            must_traverse: None,
            must_not_exist: false,
            max_depth: 21,
            edge_types: vec![],
        };
        let result = validate_params_standalone(&params);
        assert!(matches!(result, Err(DkgError::InvalidParams(_))));
    }

    #[test]
    fn validate_accepts_valid_params() {
        let params = ReachabilityParams {
            source_pattern: "src%".to_string(),
            sink_pattern: "sink%".to_string(),
            must_traverse: None,
            must_not_exist: false,
            max_depth: 10,
            edge_types: vec![],
        };
        let result = validate_params_standalone(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn like_match_basics() {
        assert!(match_like("hello", "hello"));
        assert!(match_like("hello", "%"));
        assert!(match_like("hello", "h%"));
        assert!(match_like("hello", "%lo"));
        assert!(match_like("hello", "h_llo"));
        assert!(!match_like("hello", "world"));
        assert!(!match_like("hello", "h_lo"));
    }

    /// Standalone validation for tests that don't need a real DB connection.
    fn validate_params_standalone(params: &ReachabilityParams) -> Result<(), DkgError> {
        if params.source_pattern.is_empty() {
            return Err(DkgError::InvalidParams(
                "source_pattern must not be empty".to_string(),
            ));
        }
        if params.sink_pattern.is_empty() {
            return Err(DkgError::InvalidParams(
                "sink_pattern must not be empty".to_string(),
            ));
        }
        if params.max_depth > 20 {
            return Err(DkgError::InvalidParams(format!(
                "max_depth {} exceeds limit of 20",
                params.max_depth
            )));
        }
        Ok(())
    }
}
