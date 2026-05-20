//! E-1 — MemGPT-style [`CoreBlock`]: user-pinned permanent context that the
//! reflection loop edits over time and the chat prompt-builder always
//! injects (capped at ~1500 tokens, the MemGPT core budget per
//! arXiv:2310.08560 §3.2).
//!
//! Lives in its own `core_blocks` SQLite table (single-row-per-user-scope)
//! rather than piggybacking on E-5's `memory_facts` — the core block is
//! mutable in place (no bi-temporal supersede chain), so the relational
//! shape is fundamentally different from [`crate::fact::MemoryFact`].
//! It does, however, reuse the same rusqlite [`Connection`] discipline as
//! [`crate::store::FactStore`].

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::MemoryError;

/// Hard token-equivalent ceiling for the block body. MemGPT (arXiv:2310.08560
/// §3.2 Table 1) ships a 1.5K-token core; we cap on bytes as a coarse proxy
/// since the local Qwen tokenizer is not in scope at this layer. ~6000 bytes
/// = ~1500 tokens for English code-chat text.
pub const CORE_BLOCK_MAX_BYTES: usize = 6_000;

/// One persisted CoreBlock row. Singleton per `(user_id, scope_tag)` — the
/// reflection loop calls [`CoreBlockStore::upsert`] rather than supersede.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreBlock {
    /// User identifier (single-user installs use `"default"`).
    pub user_id: String,
    /// Scope tag — `"global"` for cross-project, `"project:<id>"` for a
    /// per-project block. Stored as a string so future scopes can land
    /// without a schema migration.
    pub scope_tag: String,
    /// Free-form text content (style + active goal + recurring opinions).
    /// Capped at [`CORE_BLOCK_MAX_BYTES`] on insert.
    pub content: String,
    /// ms-since-epoch of the most recent edit.
    pub updated_at_ms: u128,
    /// Total edit count — useful for the dashboard's "drift" indicator.
    pub edits: u64,
}

impl CoreBlock {
    /// Construct an empty block for `(user_id, scope_tag)` at `now_ms`.
    pub fn empty(user_id: impl Into<String>, scope_tag: impl Into<String>, now_ms: u128) -> Self {
        Self {
            user_id: user_id.into(),
            scope_tag: scope_tag.into(),
            content: String::new(),
            updated_at_ms: now_ms,
            edits: 0,
        }
    }

    /// Cheap validator — rejects oversized blocks at the boundary so the
    /// chat prompt-builder cannot accidentally emit >1.5K tokens.
    pub fn validate(&self) -> Result<(), MemoryError> {
        if self.user_id.is_empty() {
            return Err(MemoryError::Invalid("core_block user_id is empty".into()));
        }
        if self.scope_tag.is_empty() {
            return Err(MemoryError::Invalid(
                "core_block scope_tag is empty".into(),
            ));
        }
        if self.content.as_bytes().len() > CORE_BLOCK_MAX_BYTES {
            return Err(MemoryError::Invalid(format!(
                "core_block content {} bytes exceeds cap {}",
                self.content.as_bytes().len(),
                CORE_BLOCK_MAX_BYTES,
            )));
        }
        Ok(())
    }
}

const CREATE_CORE_BLOCK_SQL: &str = "\
CREATE TABLE IF NOT EXISTS core_blocks (
    user_id TEXT NOT NULL,
    scope_tag TEXT NOT NULL,
    content TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    edits INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, scope_tag)
);
CREATE INDEX IF NOT EXISTS idx_core_blocks_updated
    ON core_blocks(updated_at_ms);
";

/// rusqlite-backed CRUD facade. Mirrors the [`crate::store::FactStore`]
/// shape — each instance owns one `Connection`; callers needing concurrent
/// writes from multiple threads should wrap in a `Mutex` (sqlite handles
/// the file-level WAL lock).
pub struct CoreBlockStore {
    conn: Connection,
}

impl CoreBlockStore {
    /// Open (or create) the sqlite file and migrate the `core_blocks` table.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        if let Some(parent) = path.as_ref().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path.as_ref())?;
        conn.execute_batch(CREATE_CORE_BLOCK_SQL)?;
        Ok(Self { conn })
    }

    /// Open an in-memory store — used by tests and transient pipelines.
    pub fn open_in_memory() -> Result<Self, MemoryError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(CREATE_CORE_BLOCK_SQL)?;
        Ok(Self { conn })
    }

    /// Read the current block for `(user_id, scope_tag)`. Returns `None`
    /// when no row exists — callers use [`CoreBlock::empty`] to bootstrap.
    pub fn get(
        &self,
        user_id: &str,
        scope_tag: &str,
    ) -> Result<Option<CoreBlock>, MemoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, scope_tag, content, updated_at_ms, edits
             FROM core_blocks
             WHERE user_id = ?1 AND scope_tag = ?2",
        )?;
        let row = stmt
            .query_row(params![user_id, scope_tag], |r| {
                Ok(CoreBlock {
                    user_id: r.get::<_, String>(0)?,
                    scope_tag: r.get::<_, String>(1)?,
                    content: r.get::<_, String>(2)?,
                    updated_at_ms: r.get::<_, i64>(3)? as u128,
                    edits: r.get::<_, i64>(4)? as u64,
                })
            })
            .optional()?;
        Ok(row)
    }

    /// Replace the entire body for `(user_id, scope_tag)`. Increments
    /// `edits`. Atomic via `INSERT ... ON CONFLICT DO UPDATE`.
    ///
    /// Per the MemGPT contract (arXiv:2310.08560 §3.2) `core_memory_replace`
    /// is the only writer; partial section edits route through `replace`
    /// with the caller pre-composing the merged body. This keeps the
    /// underlying row a single mutable string + lets the reflection loop
    /// own section ordering / merging logic.
    pub fn upsert(&self, block: &CoreBlock) -> Result<(), MemoryError> {
        block.validate()?;
        self.conn.execute(
            "INSERT INTO core_blocks (user_id, scope_tag, content, updated_at_ms, edits)
             VALUES (?1, ?2, ?3, ?4, 1)
             ON CONFLICT(user_id, scope_tag) DO UPDATE SET
                content = excluded.content,
                updated_at_ms = excluded.updated_at_ms,
                edits = core_blocks.edits + 1",
            params![
                block.user_id,
                block.scope_tag,
                block.content,
                block.updated_at_ms as i64,
            ],
        )?;
        Ok(())
    }

    /// Append `delta` to the existing block body, truncating from the head
    /// if the result would exceed [`CORE_BLOCK_MAX_BYTES`]. This is the
    /// reflection loop's default writer when no `core_memory_replace` tool
    /// call has fired — newer notes push older content out.
    pub fn append(
        &self,
        user_id: &str,
        scope_tag: &str,
        delta: &str,
        now_ms: u128,
    ) -> Result<CoreBlock, MemoryError> {
        let current = self
            .get(user_id, scope_tag)?
            .unwrap_or_else(|| CoreBlock::empty(user_id, scope_tag, now_ms));

        let mut merged = if current.content.is_empty() {
            String::new()
        } else {
            let mut s = current.content.clone();
            s.push('\n');
            s
        };
        merged.push_str(delta);

        // Head-truncate if oversize. Cut on a char boundary so we never
        // produce invalid UTF-8.
        if merged.as_bytes().len() > CORE_BLOCK_MAX_BYTES {
            let overflow = merged.as_bytes().len() - CORE_BLOCK_MAX_BYTES;
            let mut cut = overflow.min(merged.len());
            while cut < merged.len() && !merged.is_char_boundary(cut) {
                cut += 1;
            }
            merged = merged.split_off(cut);
        }

        let updated = CoreBlock {
            user_id: current.user_id,
            scope_tag: current.scope_tag,
            content: merged,
            updated_at_ms: now_ms,
            edits: current.edits.saturating_add(1),
        };
        self.upsert(&updated)?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_round_trip() {
        let store = CoreBlockStore::open_in_memory().expect("store");
        let got = store.get("default", "global").expect("get");
        assert!(got.is_none());
    }

    #[test]
    fn upsert_then_read() {
        let store = CoreBlockStore::open_in_memory().expect("store");
        let cb = CoreBlock {
            user_id: "default".into(),
            scope_tag: "global".into(),
            content: "prefers blake3, lowercase i, hates emojis".into(),
            updated_at_ms: 1_700_000_000_000,
            edits: 0,
        };
        store.upsert(&cb).expect("upsert");
        let got = store.get("default", "global").expect("get").expect("some");
        assert_eq!(got.content, cb.content);
        // edits starts at 1 after the first insert (the ON CONFLICT path
        // increments; first insert writes 1 explicitly).
        assert_eq!(got.edits, 1);
    }

    #[test]
    fn upsert_increments_edits() {
        let store = CoreBlockStore::open_in_memory().expect("store");
        let cb = CoreBlock::empty("default", "global", 1);
        store.upsert(&cb).expect("first");
        store.upsert(&cb).expect("second");
        store.upsert(&cb).expect("third");
        let got = store.get("default", "global").expect("get").expect("some");
        assert_eq!(got.edits, 3);
    }

    #[test]
    fn append_head_truncates_on_overflow() {
        let store = CoreBlockStore::open_in_memory().expect("store");
        let big = "x".repeat(CORE_BLOCK_MAX_BYTES);
        let updated = store.append("default", "global", &big, 1).expect("append1");
        assert!(updated.content.as_bytes().len() <= CORE_BLOCK_MAX_BYTES);

        // Push more bytes — the old head must be cut.
        let more = "y".repeat(2000);
        let after = store
            .append("default", "global", &more, 2)
            .expect("append2");
        assert!(after.content.as_bytes().len() <= CORE_BLOCK_MAX_BYTES);
        assert!(after.content.ends_with("yyyy"));
    }

    #[test]
    fn validate_rejects_oversize() {
        let cb = CoreBlock {
            user_id: "u".into(),
            scope_tag: "global".into(),
            content: "z".repeat(CORE_BLOCK_MAX_BYTES + 1),
            updated_at_ms: 0,
            edits: 0,
        };
        assert!(cb.validate().is_err());
    }
}
