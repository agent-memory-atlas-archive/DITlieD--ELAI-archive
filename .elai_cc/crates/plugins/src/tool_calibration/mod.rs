//! C-47: Case-Based Calibration of Tool-Use Telemetry.
//!
//! Paper grounding: arXiv:2605.15041 (CAST — Case-Based Calibration of
//! Adaptive Reasoning and Execution for LLM Tool Use). The paper extracts
//! AGGREGATE signals across the historical case base (complexity profile +
//! 5-dim failure profile), NOT per-query nearest-neighbor retrieval. ELAI's
//! C-47 follows that aggregate pattern: one [`AggregatedCase`] per (tool,
//! call_shape_hash) accumulates total_calls + error_count + per-dimension
//! failure counts; the hinter consults that aggregate to decide whether to
//! inject a corrective hint into the next prompt. See
//! workflow/reasoning/C-47-decisions.md.
//!
//! WIRING_CONTRACT (from C-47 plan, re-anchored after path drifts):
//!   produces: ToolCalibrationRegistry, AggregatedCase, FailureProfile,
//!             CaseBaseSsbEmitter, NoopCaseBaseEmitter,
//!             tool_calibration_emit_kinds (3 &'static str consts)
//!   consumes: every MCP tool invocation via DefaultMcpBridge::record_call_
//!             and ::consult_calibration; SSB event bus via the adapter
//!             trait at the orchestrator boundary.
//!   stub_ban: record_call MUST run AFTER tool returns (audit-trail
//!             invariant); LRU eviction MUST be deterministic
//!             (lru::LruCache.put semantics — least-recently-USED, NOT
//!             random); hint suppression MUST gate AT THE EMIT POINT
//!             (consult_calibration returns None before any SSB emit when
//!             the registry is None or threshold not met).
//!
//! Module layout:
//!   * [`case_base`] — [`AggregatedCase`] + [`ResponseKind`]
//!   * [`failure_profile`] — 5-dim FailureProfile per CAST paper
//!   * [`complexity`] — heuristic H in [0,1] complexity score
//!   * [`hinter`] — generate_hint(case, tool_name) -> Option<String>
//!   * [`registry`] — per-tool LRU<call_shape_hash, AggregatedCase>
//!
//! Anti-stale-training: uses `lru = "0.18"` (workspace dep) NOT a new sled
//! crate; uses `tokio::sync::RwLock` in the async bridge path NOT
//! `std::sync::Mutex`; uses `tracing` not `log`; uses `thiserror` not
//! `Box<dyn Error>` for errors.

pub mod case_base;
pub mod complexity;
pub mod failure_profile;
pub mod hinter;
pub mod registry;

pub use case_base::{call_shape_hash, classify_response, AggregatedCase, ResponseKind};
pub use complexity::{complexity_score, COMPLEXITY_FLOOR, COMPLEXITY_CEIL};
pub use failure_profile::FailureProfile;
pub use hinter::{generate_hint, HINT_ERROR_RATE_THRESHOLD, HINT_MIN_TOTAL_CALLS};
pub use registry::{
    CASE_BASE_LRU_CAP, CaseBaseSsbEmitter, NoopCaseBaseEmitter, ToolCalibrationRegistry,
    ToolCalibrationStatus, emit_kinds,
};
