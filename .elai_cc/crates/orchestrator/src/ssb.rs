//! Shared State Board (SSB): inter-agent communication via SQLite WAL.
//!
//! Single-writer guarantee via `tokio::mpsc` channel. All writes serialized
//! through one dedicated task. Reads served from a `deadpool-sqlite` async connection pool.
//! Append-only within a session — no UPDATE or DELETE.
//!
//! A single [`Ssb`] instance is permanently bound to one `(project_id, session_id)` pair.
//! Multi-session scenarios require separate instances.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use rusqlite::types::Value;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::{broadcast, mpsc, oneshot};

use elai_core::clock::ClockProvider;
use elai_reasoning::chat_reflect::{ReflectEvent, ReflectEventSink};

// E-25 (SCHEMA_VERSION 172) -- claude-code-architecture catalog payload
// type used in `SsbPayload::ClaudeCodeArchitectureEvent`. The catalog enum
// + kind discriminator are imported inside the `tests` module where the
// sample-payload constructors live.
use crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventPayload;

// E-29 Tier-B (SCHEMA_VERSION 173) -- agentic-observability event payload
// type used in `SsbPayload::AgenticObservabilityEvent`. The payload + 4-
// arm kind discriminator live inline below (`AgenticObservabilityEventPayload`
// + `AgenticObservabilityEventKind`) since the observability subtree
// (crate::observability) consumes the payload to build the row but does
// not own the wire-format definition. Reuses crate::mast::MastMode so
// the MastFailureClassified arm carries the canonical 14-variant tag
// without duplicating the taxonomy.
use crate::mast::MastMode;


// === Tier-B Move 1: ssb.rs split into a module directory (behavior-preserving). ===
// Submodules live in src/ssb/; everything is re-exported here so every external
// path `elai_orchestrator::ssb::X` is unchanged. SCHEMA_VERSION stays in THIS file
// (literal const line) so the contract path-greps (e12, h51) remain green.
mod schema;
mod action_type;
mod payloads;
mod store;

pub use schema::*;
pub use action_type::*;
pub use payloads::*;
pub use store::*;

const SCHEMA_VERSION: i64 = 253;

/// Public re-export of [`SCHEMA_VERSION`] for contract / integration tests
/// outside the orchestrator crate. The const itself stays crate-private so
/// internal callers cannot drift from the canonical value.
pub const SCHEMA_VERSION_PUBLIC: i64 = SCHEMA_VERSION;

/// Maximum serialized payload size in bytes (1 MiB).
/// `write_payload()` rejects payloads exceeding this limit with `SsbError::PayloadTooLarge`.
const PAYLOAD_MAX_BYTES: usize = 1_048_576;

/// H-10 canvas: number of ops between automatic state checkpoints.
pub const CANVAS_CHECKPOINT_INTERVAL_OPS: u32 = 20;

/// H-10 canvas: maximum number of ops that may accumulate between checkpoints
/// before a forced checkpoint is triggered. Must be >= CANVAS_CHECKPOINT_INTERVAL_OPS.
pub const CANVAS_MAX_DELTA_WITHOUT_CHECKPOINT: u32 = 50;

/// F-10 / Slice A1 (SCHEMA_VERSION 200) -- kill-switch env var name for the
/// security-monotonicity iteration guard. Default-ON: the guard runs unless
/// this var is set to a truthy value. There is intentionally NO default-off
/// ENABLE flag (W-2 -- security gates must not be wire-dark by default).
pub const ELAI_SECURITY_MONOTONICITY_DISABLE_ENV: &str =
    "ELAI_SECURITY_MONOTONICITY_DISABLE";

/// Tranche K / B.2 Option-B -- non-web retry emit env gate. Wire-dark
/// default: when set to truthy ("1"/"true"/"yes"/"on"), enables a
/// SECOND `fix_and_rerun_until_green` call site in
/// `task_runner::drive_task` (inserted AFTER the web `is_web_project`
/// branch) that fires for non-web projects (where `is_web_project=false`
/// and the baby step did not pass). The web-side gate
/// (`ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE`) is unchanged. Both gates are
/// TRULY INDEPENDENT observability switches: per the W3 finding in the
/// Tranche K context package, gate selection is funneled through
/// [`crate::contract_coding::retry_emit::c50_emit_gate_truthy`], so
/// flipping only the non-web gate does NOT widen web-path emission, and
/// vice versa. Operators flip per-mode without coupling.
pub const ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV: &str =
    "ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE";

/// Tranche B-6 quad -- shared env-gate predicate. Returns `true` when the
/// named env var is set to one of `"1"` / `"true"` / `"yes"` / `"on"`
/// (case-insensitive). Mirrors the precedent set by other Pattern-2
/// substrates (E-47 LoraShipGate, E-40 SymbolGraph). Reads via
/// `std::env::var`; cheap enough for emit-hot-paths (microseconds).
#[must_use]
pub fn env_gate_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
                || trimmed.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Tranche B-6 quad -- shared blake3-prefix label hasher. Returns the
/// first 8 bytes of `blake3(label.as_bytes())` packed LE as `u64`.
/// Matches the prefix-hashing discipline used by C-45 / C-46 / C-47
/// payloads. blake3 (NOT blake2 / sha2) per CLAUDE.md HARD RULE.
#[must_use]
pub fn blake3_label_prefix(label: &str) -> u64 {
    let bytes = blake3::hash(label.as_bytes());
    let prefix: [u8; 8] = bytes.as_bytes()[..8].try_into().unwrap_or([0u8; 8]);
    u64::from_le_bytes(prefix)
}

/// Tranche B-6 quad -- system clock ms helper. Saturating-to-`i64::MAX`
/// on the `u128 -> i64` cast (year 292278994 + safe by construction);
/// `UNIX_EPOCH` shift errors collapse to `0` (clock set before 1970 is
/// advisory-only at the SSB ts boundary).
#[must_use]
pub fn system_time_ms_i64() -> i64 {
    // This helper IS the canonical timestamp source for ssb rows; the
    // ssb writer is by design where ClockProvider injection bottoms out.
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms_u128 = SystemTime::now() // entropy-allow: ssb-canonical-timestamp-source-saturating-on-failure
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    i64::try_from(ms_u128).unwrap_or(i64::MAX)
}


#[cfg(test)]
mod tests;
