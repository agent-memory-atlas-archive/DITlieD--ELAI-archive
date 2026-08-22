//! J-104: overlay-first retrieval seam.
//!
//! The store lives in `elai_orchestrator::index_overlay`. This trait is what
//! `QueryPipeline` consults so a just-edited symbol is visible (and a
//! just-deleted symbol is gone) before any batch re-index.

/// One overlay verdict for a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayHit {
    /// Just-added or just-changed symbol. Authoritative over the base index.
    Live {
        symbol: String,
        file: String,
        snippet: String,
    },
    /// Just-deleted or renamed-away symbol. Shadows any base hit.
    Tombstone { symbol: String, file: String },
}

/// Read-side overlay consulted BEFORE BM25 / vector / DKG results.
pub trait LiveIndexOverlay: Send + Sync {
    fn hits_for_query(&self, query: &str) -> Vec<OverlayHit>;
}
