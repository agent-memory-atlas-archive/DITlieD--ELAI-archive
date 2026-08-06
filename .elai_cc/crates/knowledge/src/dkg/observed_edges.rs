//! Runtime-observed DKG call edges (J-88 E2).
use serde::{Deserialize, Serialize};
use rusqlite::{params, OptionalExtension};

use super::DkgEngine;
use crate::KnowledgeError;

pub const EDGE_OBSERVED_CALLS: &str = "observed_calls";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeProvenance {
    pub source: String,
    pub source_seq: Option<i64>,
    pub observed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedCallEdge {
    pub caller_symbol: String,
    pub callee_symbol: String,
    pub caller_file: Option<String>,
    pub callee_file: Option<String>,
    pub provenance: EdgeProvenance,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ObservedEdgeIngestStats {
    pub edges_inserted: u64,
    pub edges_already_present: u64,
    pub edges_unresolved: u64,
    pub edges_rejected_no_provenance: u64,
}

impl DkgEngine {
    /// Widen-only ingest of runtime call observations. Static edges are never removed.
    pub fn ingest_observed_edges(
        &mut self,
        edges: &[ObservedCallEdge],
    ) -> Result<ObservedEdgeIngestStats, KnowledgeError> {
        let mut stats = ObservedEdgeIngestStats::default();
        for edge in edges {
            if edge.provenance.source.trim().is_empty() {
                stats.edges_rejected_no_provenance += 1;
                continue;
            }
            let source_id = find_node(&self.conn, &edge.caller_symbol, edge.caller_file.as_deref())?;
            let target_id = find_node(&self.conn, &edge.callee_symbol, edge.callee_file.as_deref())?;
            let (Some(source_id), Some(target_id)) = (source_id, target_id) else {
                stats.edges_unresolved += 1;
                continue;
            };
            let changed = self.conn.execute(
                "INSERT OR IGNORE INTO dkg_edges (source_id, target_id, edge_type) VALUES (?1, ?2, ?3)",
                params![source_id, target_id, EDGE_OBSERVED_CALLS],
            )?;
            if changed == 1 { stats.edges_inserted += 1; } else { stats.edges_already_present += 1; }
            self.conn.execute(
                "INSERT OR IGNORE INTO dkg_observed_edge_provenance (source_id, target_id, provenance_source, source_seq, observed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![source_id, target_id, edge.provenance.source, edge.provenance.source_seq, edge.provenance.observed_at],
            )?;
        }
        Ok(stats)
    }
}

fn find_node(
    conn: &rusqlite::Connection,
    name: &str,
    file: Option<&str>,
) -> Result<Option<i64>, rusqlite::Error> {
    match file {
        Some(file_path) => conn.query_row(
            "SELECT id FROM dkg_nodes WHERE name = ?1 AND file_path = ?2 LIMIT 1",
            params![name, file_path], |row| row.get(0)).optional(),
        None => conn.query_row(
            "SELECT id FROM dkg_nodes WHERE name = ?1 LIMIT 1",
            params![name], |row| row.get(0)).optional(),
    }
}

#[cfg(test)]
mod j88_observed_edges_tests {
    use super::*;
    use std::path::Path;
    use tempfile::NamedTempFile;

    fn engine() -> DkgEngine {
        let file = NamedTempFile::new().expect("temp db");
        let path = file.into_temp_path();
        // Keep the temporary path alive for the engine's lifetime.
        let engine = DkgEngine::new(Path::new(&path), None, 1).expect("engine");
        std::mem::forget(path);
        engine
    }

    fn add_nodes(engine: &DkgEngine) {
        for (id, name) in [(1, "do_call"), (2, "handle")] {
            engine.conn.execute("INSERT INTO dkg_nodes (id,file_path,name,kind,start_line,end_line) VALUES (?1,'test.rs',?2,'function',1,1)", params![id,name]).expect("node");
        }
    }

    fn edge() -> ObservedCallEdge {
        ObservedCallEdge { caller_symbol: "do_call".into(), callee_symbol: "handle".into(), caller_file: None, callee_file: None,
            provenance: EdgeProvenance { source: "coverage_trace".into(), source_seq: None, observed_at: 1 } }
    }

    #[test]
    fn ingest_observed_edge_widens_cone() { let mut e=engine(); add_nodes(&e); let s=e.ingest_observed_edges(&[edge()]).expect("ingest"); assert_eq!(s.edges_inserted,1); assert_eq!(e.conn.query_row("SELECT COUNT(*) FROM dkg_edges WHERE edge_type='observed_calls'",[],|r|r.get::<_,i64>(0)).unwrap(),1); assert_eq!(e.conn.query_row("SELECT COUNT(*) FROM dkg_observed_edge_provenance",[],|r|r.get::<_,i64>(0)).unwrap(),1); }
    #[test]
    fn ingest_observed_edge_idempotent() { let mut e=engine(); add_nodes(&e); e.ingest_observed_edges(&[edge()]).unwrap(); let s=e.ingest_observed_edges(&[edge()]).unwrap(); assert_eq!(s.edges_already_present,1); }
    #[test]
    fn ingest_rejects_edge_without_provenance() { let mut e=engine(); add_nodes(&e); let mut x=edge(); x.provenance.source=" ".into(); let s=e.ingest_observed_edges(&[x]).unwrap(); assert_eq!(s.edges_rejected_no_provenance,1); }
    #[test]
    fn ingest_skips_unresolved_endpoints() { let mut e=engine(); add_nodes(&e); let mut x=edge(); x.callee_symbol="missing".into(); let s=e.ingest_observed_edges(&[x]).unwrap(); assert_eq!(s.edges_unresolved,1); }
    #[test]
    fn ingest_does_not_shrink_static_cone() { let mut e=engine(); add_nodes(&e); e.conn.execute("INSERT INTO dkg_edges VALUES (1,2,'imports')",[]).unwrap(); e.ingest_observed_edges(&[edge()]).unwrap(); assert_eq!(e.conn.query_row("SELECT COUNT(*) FROM dkg_edges WHERE edge_type='imports'",[],|r|r.get::<_,i64>(0)).unwrap(),1); }
}
