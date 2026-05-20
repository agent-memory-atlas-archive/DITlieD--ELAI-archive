//! E-5 — user modeling + long-term memory.
//!
//! This crate is the storage substrate for the bi-temporal episodic memory
//! store described in `docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md`.
//! It collapses the Zep/Graphiti bi-temporal edge model (arXiv:2501.13956
//! §2.2.3) into a single `memory_facts` relational table, plus a Mem0-style
//! (arXiv:2504.19413) extract+update pipeline that produces ADD/UPDATE/DELETE/
//! NOOP tool calls. Per-fact `trust_score` decays monotonically with elapsed
//! time per fact-type-specific half-lives as the structural defense against
//! memory poisoning (arXiv:2601.05504).
//!
//! Scope of this initial vertical slice (called by orchestrator hooks +
//! Tauri commands `get_memory_facts` / `get_trust_score`):
//!
//! - [`fact`] — [`MemoryFact`] + [`FactType`] + [`Scope`] (serde + sqlite
//!   row-mapping)
//! - [`store`] — [`FactStore`] CRUD with bi-temporal queries
//! - [`extractor`] — pure-Rust [`extract_facts`] for the per-turn extraction
//!   producer (LLM dispatch lives in orchestrator's call site; this function
//!   accepts the already-parsed JSON envelope and returns fact candidates)
//! - [`retrieval`] — trust-and-provenance-filtered [`get_relevant_facts`]
//! - [`decay`] — pure decay-curve math + [`decay_tick`] batch helper
//!
//! Reserved for follow-ups (intentionally deferred from this slice per the
//! plan's PARTIAL / additive policy): full Mem0 ADD/UPDATE/DELETE/NOOP
//! dispatcher, two-stage write gate, scoped CLI subcommand surface,
//! contradiction adjudicator, MemoryDrawer rendering, decay scheduler task.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod core_block;
pub mod decay;
pub mod error;
pub mod extractor;
pub mod fact;
pub mod hippo_rag;
pub mod notes;
pub mod retrieval;
pub mod store;

pub use core_block::{CoreBlock, CoreBlockStore, CORE_BLOCK_MAX_BYTES};
pub use decay::{decay_tick, decayed_trust, default_decay_rate, DecayTickStats};
pub use error::MemoryError;
pub use extractor::{extract_facts, Conversation, ExtractedFact, Turn};
pub use fact::{FactType, MemoryFact, Scope};
pub use hippo_rag::{
    retrieve_notes, RetrievedNote, PPR_CONVERGENCE_TOL, PPR_DAMPING, PPR_MAX_ITERATIONS,
};
pub use notes::{MemoryNote, NoteStore};
pub use retrieval::{get_relevant_facts, RetrievalFilter};
pub use store::FactStore;
