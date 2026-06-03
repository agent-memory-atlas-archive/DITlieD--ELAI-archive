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
//! E-5 session-2 additions (active scope per E-5-decisions.md): Mem0
//! ADD/UPDATE/DELETE/NOOP dispatcher ([`mem0_dispatcher`]), Stage-1
//! regex blocklist + sensitive-class skip-list ([`defense`]), heuristic
//! contradiction adjudicator ([`contradiction`]), tokio decay
//! scheduler ([`decay_scheduler`]).
//!
//! Deferred to E-5-followup (per E-5-decisions.md): LLM-backed
//! extractor + Stage-2 classifier, MemoryDrawer adjudication UI,
//! promotion gate (depends on Stage-2 classifier), per-user
//! `%USERPROFILE%\.elai_user\` tier (paths helpers shipped, FactStore
//! tiering deferred until per-user write paths exist).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod contradiction;
pub mod core_block;
pub mod decay;
pub mod decay_scheduler;
pub mod defense;
pub mod error;
pub mod evolution;
pub mod extractor;
pub mod fact;
pub mod hippo_rag;
pub mod mem0_dispatcher;
// F-12 Tier-2 — OnnxNliJudge over ort, behind the default-OFF `nli-onnx`
// feature so the default Tier-1 build compiles WITHOUT ort.
#[cfg(feature = "nli-onnx")]
pub mod nli_onnx;
pub mod notes;
pub mod quarantine_broadcast;
// F-12 Tier-1 — semantic contradiction pre-filter (deterministic local NLI
// judge). Fully offline; no ort in the default build.
pub mod semantic_contradiction;
pub mod retrieval;
pub mod store;
pub mod tools;
pub mod trust;
pub mod view;

pub use contradiction::{audit_conflicts, ConflictAction};
pub use core_block::{CoreBlock, CoreBlockStore, CORE_BLOCK_MAX_BYTES};
pub use decay::{decay_tick, decayed_trust, default_decay_rate, DecayTickStats};
pub use decay_scheduler::{
    spawn_decay_scheduler, DecayHandle, DecaySchedulerConfig, DecayTickObserver,
    DEFAULT_DECAY_TICK_INTERVAL, MIN_DECAY_TICK_INTERVAL,
};
pub use defense::{classify, is_blocked_payload, is_sensitive_skip, DefenseVerdict};
pub use error::MemoryError;
pub use mem0_dispatcher::{
    dispatch_batch, dispatch_one, DispatchContext, DispatchOp, TOMBSTONE_MARKER,
};
pub use evolution::{
    apply_diff, evolve_on_insert, ppr_seeded_top_k, AttributeDiff, AttributeDiffKind,
    MAX_NEIGHBOURS_PER_INSERT,
};
pub use extractor::{extract_facts, Conversation, ExtractedFact, Turn};
pub use fact::{FactType, MemoryFact, Scope};
pub use hippo_rag::{
    retrieve_notes, RetrievedNote, PPR_CONVERGENCE_TOL, PPR_DAMPING, PPR_MAX_ITERATIONS,
};
pub use notes::{MemoryNote, NoteStore};
pub use quarantine_broadcast::{
    collect_quarantined, QuarantineBroadcastQueue, QuarantineRecord,
    QUARANTINE_QUEUE_CAPACITY, STALENESS_QUARANTINE_THRESHOLD,
};
pub use retrieval::{get_relevant_facts, RetrievalFilter};
pub use semantic_contradiction::{
    adjudicate_semantic, collect_cross_key_candidates, pair_fingerprint, ContradictionCandidate,
    DeterministicNliJudge, NliJudge, NliLabel, NliVerdict, SemanticAction, SemanticConflictOutcome,
    SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
};
pub use store::FactStore;
pub use tools::{
    archival_search_tool_def, core_memory_replace_tool_def, ChatMemoryToolDef,
    ARCHIVAL_SEARCH_TOOL_NAME, CORE_MEMORY_REPLACE_TOOL_NAME,
};
pub use trust::{Provenance, TrustTier};
pub use view::{SafetyCriticalQueryType, TierCeiling};
