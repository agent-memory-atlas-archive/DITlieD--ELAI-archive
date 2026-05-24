//! E-1 — `memory_notes` table: A-MEM Zettelkasten-style conversational
//! notes (arXiv:2502.12110 §3.2). Distinct from E-5's structured
//! [`crate::fact::MemoryFact`] rows — notes carry free-form `body` text
//! plus `tags` / `links`, and are the substrate the HippoRAG-style
//! Personalized PageRank walks over.
//!
//! Schema mirrors the E-1 plan Task 1 column list collapsed to the
//! columns this in-scope slice consumes (the full bi-temporal layout +
//! `superseded_by` chain lives in E-5's `memory_facts`; notes use a
//! single `valid_to_ms`-NULL flag for the active-set hot path).

use std::path::Path;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::MemoryError;

/// One row of the `memory_notes` table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryNote {
    /// UUIDv4 hex (use [`MemoryNote::new_id`]).
    pub note_id: String,
    /// `"session"` | `"project"` | `"global"`.
    pub scope: String,
    /// Free-form tags extracted by the chat-reflect TRIAGE stage.
    pub tags: Vec<String>,
    /// Free-form keywords / entities (PPR seed input).
    pub keywords: Vec<String>,
    /// Optional human-readable context blob (one-paragraph summary).
    pub context: String,
    /// The note body — one chat-turn summary or one extracted fact.
    pub body: String,
    /// `note_id` references to other notes (Zettelkasten links — the
    /// adjacency the PPR walker uses).
    pub links: Vec<String>,
    /// `"note"` | `"summary"` | `"mistake"` | etc. (mirrors E-5 fact_type
    /// vocabulary so callers can union across the two tables when needed).
    pub fact_type: String,
    /// ms-since-epoch of row creation.
    pub created_at_ms: u128,
    /// ms-since-epoch of the most recent retrieval (PPR walker bumps this).
    pub last_accessed_ms: u128,
    /// Optional bi-temporal close timestamp. `None` = currently valid.
    pub valid_to_ms: Option<u128>,
    /// [0.0, 1.0] — initial trust, decayed by [`crate::decay`].
    pub trust_score: f32,
}

impl MemoryNote {
    /// Generate a fresh UUIDv4 hex string for `note_id`.
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Cheap invariant check before insert.
    pub fn validate(&self) -> Result<(), MemoryError> {
        if self.note_id.is_empty() {
            return Err(MemoryError::Invalid("note_id is empty".into()));
        }
        if self.body.is_empty() {
            return Err(MemoryError::Invalid("body is empty".into()));
        }
        if !(0.0..=1.0).contains(&self.trust_score) {
            return Err(MemoryError::Invalid(format!(
                "trust_score out of [0,1]: {}",
                self.trust_score
            )));
        }
        Ok(())
    }
}

const CREATE_NOTES_SQL: &str = "\
CREATE TABLE IF NOT EXISTS memory_notes (
    note_id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    tags TEXT NOT NULL,
    keywords TEXT NOT NULL,
    context TEXT NOT NULL,
    body TEXT NOT NULL,
    links TEXT NOT NULL,
    fact_type TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    last_accessed_ms INTEGER NOT NULL,
    valid_to_ms INTEGER,
    trust_score REAL NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_memory_notes_created
    ON memory_notes(created_at_ms);
CREATE INDEX IF NOT EXISTS idx_memory_notes_valid_now
    ON memory_notes(scope) WHERE valid_to_ms IS NULL;
";

/// rusqlite facade for the `memory_notes` table. Same one-`Connection`-
/// per-store discipline as [`crate::store::FactStore`].
pub struct NoteStore {
    conn: Connection,
}

impl NoteStore {
    /// Open (or create) the sqlite file and migrate `memory_notes`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        if let Some(parent) = path.as_ref().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path.as_ref())?;
        conn.execute_batch(CREATE_NOTES_SQL)?;
        Ok(Self { conn })
    }

    /// Open an in-memory store (tests / transient retrieval).
    pub fn open_in_memory() -> Result<Self, MemoryError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(CREATE_NOTES_SQL)?;
        Ok(Self { conn })
    }

    /// Insert one note row.
    pub fn insert(&self, note: &MemoryNote) -> Result<(), MemoryError> {
        note.validate()?;
        let tags_json = serde_json::to_string(&note.tags)?;
        let keywords_json = serde_json::to_string(&note.keywords)?;
        let links_json = serde_json::to_string(&note.links)?;
        self.conn.execute(
            "INSERT INTO memory_notes (
                note_id, scope, tags, keywords, context, body, links,
                fact_type, created_at_ms, last_accessed_ms, valid_to_ms,
                trust_score
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12
            )",
            params![
                note.note_id,
                note.scope,
                tags_json,
                keywords_json,
                note.context,
                note.body,
                links_json,
                note.fact_type,
                note.created_at_ms as i64,
                note.last_accessed_ms as i64,
                note.valid_to_ms.map(|v| v as i64),
                note.trust_score,
            ],
        )?;
        Ok(())
    }

    /// List currently-valid notes (`valid_to_ms IS NULL`), most-recent
    /// first, capped at `limit`. Used by the dashboard + chat-reflect
    /// PRE-stage.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<MemoryNote>, MemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT note_id, scope, tags, keywords, context, body, links,
                    fact_type, created_at_ms, last_accessed_ms, valid_to_ms,
                    trust_score
             FROM memory_notes
             WHERE valid_to_ms IS NULL
             ORDER BY created_at_ms DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_note)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r??);
        }
        Ok(out)
    }

    /// Fetch by primary key.
    pub fn get(&self, note_id: &str) -> Result<Option<MemoryNote>, MemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT note_id, scope, tags, keywords, context, body, links,
                    fact_type, created_at_ms, last_accessed_ms, valid_to_ms,
                    trust_score
             FROM memory_notes WHERE note_id = ?1",
        )?;
        let mut rows = stmt.query(params![note_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_note(row)??)),
            None => Ok(None),
        }
    }

    /// Overwrite a note's mutable attributes (tags, links, context,
    /// last_accessed_ms, valid_to_ms, trust_score). Body is also
    /// rewritten — the A-MEM evolution loop only edits non-body
    /// attributes by convention, but the column is overwritten so the
    /// caller can keep this surface single-purpose. Caller is
    /// responsible for emitting `MemoryNoteEvolved` on SSB.
    pub fn replace_attributes(&self, note: &MemoryNote) -> Result<(), MemoryError> {
        note.validate()?;
        let tags_json = serde_json::to_string(&note.tags)?;
        let keywords_json = serde_json::to_string(&note.keywords)?;
        let links_json = serde_json::to_string(&note.links)?;
        let n = self.conn.execute(
            "UPDATE memory_notes SET
                scope = ?2,
                tags = ?3,
                keywords = ?4,
                context = ?5,
                body = ?6,
                links = ?7,
                fact_type = ?8,
                last_accessed_ms = ?9,
                valid_to_ms = ?10,
                trust_score = ?11
             WHERE note_id = ?1",
            params![
                note.note_id,
                note.scope,
                tags_json,
                keywords_json,
                note.context,
                note.body,
                links_json,
                note.fact_type,
                note.last_accessed_ms as i64,
                note.valid_to_ms.map(|v| v as i64),
                note.trust_score,
            ],
        )?;
        if n == 0 {
            return Err(MemoryError::NotFound(note.note_id.clone()));
        }
        Ok(())
    }

    /// Bump `last_accessed_ms` for a single note — called by the PPR
    /// walker on every retrieved row to feed the decay curve.
    pub fn touch(&self, note_id: &str, now_ms: u128) -> Result<(), MemoryError> {
        let n = self.conn.execute(
            "UPDATE memory_notes SET last_accessed_ms = ?2 WHERE note_id = ?1",
            params![note_id, now_ms as i64],
        )?;
        if n == 0 {
            return Err(MemoryError::NotFound(note_id.to_owned()));
        }
        Ok(())
    }

    /// Count of currently-valid rows. Used by the Tauri command for the
    /// dashboard drawer header.
    pub fn count_current(&self) -> Result<usize, MemoryError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_notes WHERE valid_to_ms IS NULL",
            [],
            |r| r.get(0),
        )?;
        Ok(n.max(0) as usize)
    }

    /// Borrow the underlying connection — reserved for future
    /// extensions (e.g. an out-of-crate adjacency reader that needs
    /// direct rusqlite access). Currently unused by [`crate::hippo_rag`]
    /// which goes through [`NoteStore::list_recent`].
    #[allow(dead_code)]
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

fn row_to_note(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<Result<MemoryNote, MemoryError>> {
    let note_id: String = row.get(0)?;
    let scope: String = row.get(1)?;
    let tags_text: String = row.get(2)?;
    let keywords_text: String = row.get(3)?;
    let context: String = row.get(4)?;
    let body: String = row.get(5)?;
    let links_text: String = row.get(6)?;
    let fact_type: String = row.get(7)?;
    let created_at: i64 = row.get(8)?;
    let last_accessed: i64 = row.get(9)?;
    let valid_to: Option<i64> = row.get(10)?;
    let trust_score: f64 = row.get(11)?;

    let tags: Vec<String> = match serde_json::from_str(&tags_text) {
        Ok(v) => v,
        Err(e) => return Ok(Err(MemoryError::from(e))),
    };
    let keywords: Vec<String> = match serde_json::from_str(&keywords_text) {
        Ok(v) => v,
        Err(e) => return Ok(Err(MemoryError::from(e))),
    };
    let links: Vec<String> = match serde_json::from_str(&links_text) {
        Ok(v) => v,
        Err(e) => return Ok(Err(MemoryError::from(e))),
    };

    Ok(Ok(MemoryNote {
        note_id,
        scope,
        tags,
        keywords,
        context,
        body,
        links,
        fact_type,
        created_at_ms: created_at as u128,
        last_accessed_ms: last_accessed as u128,
        valid_to_ms: valid_to.map(|v| v as u128),
        trust_score: trust_score as f32,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_note(id: &str, body: &str, tags: &[&str], links: &[&str]) -> MemoryNote {
        MemoryNote {
            note_id: id.into(),
            scope: "session".into(),
            tags: tags.iter().map(|s| (*s).into()).collect(),
            keywords: vec!["blake3".into()],
            context: "ctx".into(),
            body: body.into(),
            links: links.iter().map(|s| (*s).into()).collect(),
            fact_type: "note".into(),
            created_at_ms: 1_700_000_000_000,
            last_accessed_ms: 1_700_000_000_000,
            valid_to_ms: None,
            trust_score: 0.8,
        }
    }

    #[test]
    fn insert_get_round_trip() {
        let store = NoteStore::open_in_memory().expect("store");
        let n = sample_note("a", "i prefer blake3", &["hash", "blake3"], &[]);
        store.insert(&n).expect("insert");
        let got = store.get("a").expect("get").expect("some");
        assert_eq!(got.body, n.body);
        assert_eq!(got.tags, vec!["hash".to_string(), "blake3".to_string()]);
    }

    #[test]
    fn list_recent_descending() {
        let store = NoteStore::open_in_memory().expect("store");
        let mut a = sample_note("a", "first", &["t"], &[]);
        a.created_at_ms = 1;
        let mut b = sample_note("b", "second", &["t"], &[]);
        b.created_at_ms = 2;
        store.insert(&a).expect("a");
        store.insert(&b).expect("b");
        let recent = store.list_recent(10).expect("list");
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].note_id, "b");
    }

    #[test]
    fn touch_bumps_last_accessed() {
        let store = NoteStore::open_in_memory().expect("store");
        let n = sample_note("a", "body", &["t"], &[]);
        store.insert(&n).expect("insert");
        store.touch("a", 99_999_999).expect("touch");
        let got = store.get("a").expect("get").expect("some");
        assert_eq!(got.last_accessed_ms, 99_999_999);
    }

    #[test]
    fn count_current_excludes_closed() {
        let store = NoteStore::open_in_memory().expect("store");
        let a = sample_note("a", "x", &[], &[]);
        let mut b = sample_note("b", "y", &[], &[]);
        b.valid_to_ms = Some(1_700_000_500_000);
        store.insert(&a).expect("a");
        store.insert(&b).expect("b");
        assert_eq!(store.count_current().expect("count"), 1);
    }

    #[test]
    fn validate_rejects_empty_body() {
        let mut n = sample_note("a", "x", &[], &[]);
        n.body = String::new();
        assert!(n.validate().is_err());
    }
}
