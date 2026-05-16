//! Context artifacts pipeline — B-35 Phase A.
//!
//! Provides a parallel indexing pipeline alongside the code DKG/BM25/vector
//! pipeline. Artifacts are project-typed non-code knowledge: database schemas,
//! OpenAPI specs, k8s manifests, ADRs, domain glossaries, Terraform configs.
//!
//! Phase A scope: manifest loading, types, indexer, staleness detection,
//! SQLite store (`context_{project_id}.db`), in-memory cosine search.
//! Phase B will add HNSW + tantivy and SSB event variants.
//!
//! # Module tree
//!
//! ```
//! artifacts
//! ├── mod.rs         — re-exports; this file
//! ├── types.rs       — ArtifactSpec, ArtifactChunk, ArtifactIndexState, ArtifactStatus
//! ├── manifest.rs    — ArtifactManifest::load + ArtifactError
//! ├── staleness.rs   — detect_staleness, StaleResult
//! ├── store.rs       — ArtifactStore (SQLite context_*.db)
//! ├── indexer.rs     — ArtifactIndexer
//! └── search.rs      — artifact_search (cosine fallback)
//! ```

pub mod indexer;
pub mod manifest;
pub mod search;
pub mod stage;
pub mod staleness;
pub mod store;
pub mod types;

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use manifest::ArtifactError;
pub use stage::ArtifactStage;
pub use types::{
    ArtifactChunk, ArtifactIndexAllResult, ArtifactIndexState, ArtifactManifest, ArtifactSearchResult,
    ArtifactSpec, ArtifactStatus,
};
