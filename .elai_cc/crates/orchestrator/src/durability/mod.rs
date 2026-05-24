//! E-44 — Durability layer additions for ELAI's SessionDag + AgentCheckpoint
//! infrastructure (B-9 SHIPPED extension surface).
//!
//! This module hosts the durability primitives that compose on top of the
//! existing `session_dag`, `checkpoint`, and `tool_dispatcher` surfaces
//! without modifying their contracts:
//!
//! * [`checkpoint_migrator`] — lazy AgentCheckpoint schema migrator chaining
//!   versioned transforms at READ side only (never mutates on-disk files).
//! * [`idempotent_dispatcher`] — `ToolDispatcher` wrapper that dedupes
//!   replayed calls by deterministic call-shape hash inside a session.
//! * [`compact_lock`] — fd-lock-backed cross-process file lock so the
//!   PreCompact hook + 85% token compact path do not race on the same
//!   `SESSION_HANDOFF`.
//! * [`continue_as_new`] — `continue_as_new_if_needed` wrapper around
//!   `SessionDag::create_handoff` that snapshots minimal carry-over state
//!   and starts a new root when event / phase ceilings are exceeded.
//!   The wrapper preserves B-9's `create_handoff` signature (additive
//!   surface only).
//! * [`resume_validator`] — checkpoint signature + schema + integrity
//!   validation before a resume attempt.
//!
//! Cite-deferred (8th cluster in mistakes.md): arXiv:2603.20625
//! (ACRFence), arXiv:2603.29231 (Reliability Decay Curve), and
//! arXiv:2603.22744 (3-bucket error taxonomy) are flagged VERIFY in the
//! E-44 plan body. Per memory feedback_paper_citation_verify, the IDs
//! are deferred until WebSearch verification lands; the modules in this
//! folder ship as DESIGN-INTENT only without inline paper anchors. See
//! `workflow/sessions/s-25120/mistakes.md` cluster #8.

pub mod checkpoint_migrator;
pub mod compact_lock;
pub mod continue_as_new;
pub mod idempotent_dispatcher;
pub mod resume_validator;

pub use checkpoint_migrator::{
    migrate_checkpoint, CheckpointMigrator, MigratorError, TransformFn,
    CURRENT_CHECKPOINT_SCHEMA_VERSION,
};
pub use compact_lock::{CompactLock, CompactLockError, LockGuard};
pub use continue_as_new::{
    continue_as_new_if_needed, ContinueAsNewError, ContinueAsNewOutcome, NewSessionHandle,
    DEFAULT_EVENT_COUNT_CEILING, DEFAULT_PHASE_COUNT_CEILING,
};
pub use idempotent_dispatcher::{
    call_shape_hash, CachedToolResult, IdempotencyError, IdempotentDispatcher,
};
pub use resume_validator::{validate_checkpoint, CheckpointValidity, ResumeValidatorError};
