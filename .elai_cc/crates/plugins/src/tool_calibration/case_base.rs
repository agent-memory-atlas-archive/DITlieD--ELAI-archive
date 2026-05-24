//! C-47: AggregatedCase + response classification.
//!
//! One [`AggregatedCase`] per `(tool_name, call_shape_hash)` aggregates
//! historical telemetry: total_calls, error_count, last response_kind,
//! complexity score, and the 5-dim [`FailureProfile`]. Aggregation
//! matches the CAST paper's case-base usage (arXiv:2605.15041 Section
//! 3.2) — signals extracted ACROSS the case base, NOT per-query
//! nearest-neighbor lookup.

use serde::{Deserialize, Serialize};

use super::complexity::{complexity_score, COMPLEXITY_FLOOR};
use super::failure_profile::{FailureDimension, FailureProfile};

/// One aggregated case for a `(tool_name, call_shape_hash)` pair. The
/// case_base stores at most [`super::registry::CASE_BASE_LRU_CAP`] of
/// these per tool via the LRU.
///
/// Note: omits `Eq` because `complexity_score` is `f32` (NaN-not-equal).
/// All Eq-requiring code paths use `call_shape_hash` for identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregatedCase {
    /// Logical tool identity (e.g. `"fs.read"`, `"git.commit"`).
    pub tool_name: String,
    /// blake3-derived `u64` over the canonical-sorted call-shape JSON
    /// key set. Identical shapes hash identically across sessions.
    pub call_shape_hash: u64,
    /// Number of historical calls observed for this (tool, shape) pair.
    /// Saturates at `u32::MAX`.
    pub total_calls: u32,
    /// Number of those calls that returned an error (any
    /// [`ResponseKind`] other than [`ResponseKind::Ok`]). Saturates at
    /// `u32::MAX`. Invariant: `error_count <= total_calls`.
    pub error_count: u32,
    /// 5-dim CAST failure profile, accumulated across all observed
    /// failures.
    pub failure_profile: FailureProfile,
    /// snake-case representation of the most recent
    /// [`ResponseKind`]. Lets the dashboard render "last result" without
    /// having to track the entire history.
    pub last_response_kind: String,
    /// Wall-clock ms timestamp of the most recent observation. Carried
    /// as `i64` for parity with SSB `ts_ms` columns.
    pub last_ts_ms: i64,
    /// Heuristic complexity score `H in [0,1]` (paper Section 3.2).
    /// Recomputed on every observation against the most recent
    /// call_shape JSON; stays in `[0, 1]` by construction.
    pub complexity_score: f32,
}

impl AggregatedCase {
    /// Construct a fresh case with zero observations. The first
    /// [`Self::observe`] call promotes it to a "seen" state.
    pub fn new(tool_name: String, call_shape_hash: u64) -> Self {
        Self {
            tool_name,
            call_shape_hash,
            total_calls: 0,
            error_count: 0,
            failure_profile: FailureProfile::default(),
            last_response_kind: ResponseKind::Ok.as_str().to_string(),
            last_ts_ms: 0,
            complexity_score: COMPLEXITY_FLOOR,
        }
    }

    /// Record one observation. Updates total_calls / error_count /
    /// failure_profile / last_response_kind / last_ts_ms /
    /// complexity_score. The aggregate is monotonic — never decreases.
    pub fn observe(
        &mut self,
        kind: ResponseKind,
        call_shape_json: &str,
        ts_ms: i64,
    ) {
        self.total_calls = self.total_calls.saturating_add(1);
        self.last_response_kind = kind.as_str().to_string();
        self.last_ts_ms = ts_ms;
        self.complexity_score = complexity_score(call_shape_json);
        if let Some(dim) = kind.failure_dimension() {
            self.error_count = self.error_count.saturating_add(1);
            self.failure_profile.record(dim);
        }
    }

    /// Error rate in `[0,1]`. Returns `0.0` when no observations exist
    /// (no signal to act on).
    pub fn error_rate(&self) -> f32 {
        if self.total_calls == 0 {
            return 0.0;
        }
        // Guarded division — total_calls is non-zero here.
        f32::from(u16::try_from(self.error_count.min(u32::from(u16::MAX))).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(self.total_calls.min(u32::from(u16::MAX))).unwrap_or(u16::MAX))
    }
}

/// Coarse response classification used by the case base. Maps onto the
/// CAST 5-dim failure profile via [`Self::failure_dimension`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseKind {
    /// Tool returned a normal (non-error) response.
    Ok,
    /// Caller invoked an unknown tool name (maps to `Name` dim).
    ErrToolNotFound,
    /// Argument key / type / value rejected up-front (maps to `Key`
    /// dim — the loudest sub-class of input-shape errors).
    ErrInvalidArgs,
    /// HTTP / network-level timeout — does NOT map to any CAST
    /// dimension; counted in `error_count` but excluded from
    /// failure_profile because it is not a shape-correctness signal.
    ErrTimeout,
    /// Server-side execution error (e.g. permission denied, internal
    /// state). Maps to `Value` dim (semantically wrong inputs / state
    /// preconditions).
    ErrServer,
    /// Catch-all (deserialization error, sanitization rejection, etc.).
    /// Counted in `error_count`; not mapped to any dim.
    ErrOther,
}

impl ResponseKind {
    /// Stable snake-case wire string used in `last_response_kind`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::ErrToolNotFound => "err_tool_not_found",
            Self::ErrInvalidArgs => "err_invalid_args",
            Self::ErrTimeout => "err_timeout",
            Self::ErrServer => "err_server",
            Self::ErrOther => "err_other",
        }
    }

    /// Returns the CAST failure dimension this response maps to, when
    /// any. `Ok`, `ErrTimeout`, `ErrOther` return `None`.
    pub fn failure_dimension(&self) -> Option<FailureDimension> {
        match self {
            Self::Ok => None,
            Self::ErrToolNotFound => Some(FailureDimension::Name),
            Self::ErrInvalidArgs => Some(FailureDimension::Key),
            Self::ErrServer => Some(FailureDimension::Value),
            Self::ErrTimeout | Self::ErrOther => None,
        }
    }
}

/// Classify an [`crate::mcp_bridge::McpCallResult`]-shaped outcome into
/// a [`ResponseKind`]. Synchronous heuristic per the C-47 plan rebase:
/// audit-loop integration is a follow-on extension; this seam reads
/// only `is_error` and an optional content hint.
///
/// `is_error` — value of `McpCallResult.is_error`.
/// `error_hint` — optional snake-case substring extracted from the
///                content for sub-classification (e.g. `"timeout"` →
///                `ErrTimeout`). Pass `None` when no hint is available.
pub fn classify_response(is_error: bool, error_hint: Option<&str>) -> ResponseKind {
    if !is_error {
        return ResponseKind::Ok;
    }
    match error_hint {
        Some(h) if h.contains("not_found") || h.contains("unknown_tool") => {
            ResponseKind::ErrToolNotFound
        }
        Some(h) if h.contains("invalid_args") || h.contains("invalid_argument") => {
            ResponseKind::ErrInvalidArgs
        }
        Some(h) if h.contains("timeout") => ResponseKind::ErrTimeout,
        Some(h) if h.contains("server") || h.contains("internal") => ResponseKind::ErrServer,
        _ => ResponseKind::ErrOther,
    }
}

/// Compute the canonical `call_shape_hash` for an arguments JSON
/// [`serde_json::Value`]. blake3-derived `u64` over the
/// alphabetically-sorted key list (top-level keys only — values are
/// excluded so different VALUES of the same shape collide as
/// intended).
pub fn call_shape_hash(args: &serde_json::Value) -> u64 {
    let mut keys: Vec<&str> = match args.as_object() {
        Some(obj) => obj.keys().map(|s| s.as_str()).collect(),
        None => Vec::new(),
    };
    keys.sort_unstable();
    let mut hasher = blake3::Hasher::new();
    for k in keys {
        hasher.update(k.as_bytes());
        hasher.update(b"\0");
    }
    // blake3 output is 32 bytes; take the first 8 as a u64 prefix.
    let bytes = hasher.finalize();
    let prefix = &bytes.as_bytes()[..8];
    u64::from_le_bytes(prefix.try_into().unwrap_or([0u8; 8]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn aggregated_case_observe_increments() {
        let mut c = AggregatedCase::new("fs.read".into(), 42);
        c.observe(ResponseKind::Ok, r#"{"path":"a"}"#, 1_000);
        c.observe(ResponseKind::ErrInvalidArgs, r#"{"path":"a"}"#, 2_000);
        assert_eq!(c.total_calls, 2);
        assert_eq!(c.error_count, 1);
        assert_eq!(c.failure_profile.key_err, 1);
        assert_eq!(c.last_response_kind, "err_invalid_args");
        assert_eq!(c.last_ts_ms, 2_000);
    }

    #[test]
    fn error_rate_zero_when_no_calls() {
        let c = AggregatedCase::new("fs.read".into(), 1);
        assert_eq!(c.error_rate(), 0.0);
    }

    #[test]
    fn call_shape_hash_is_key_order_independent() {
        let a = json!({"path": "x", "lines": 5});
        let b = json!({"lines": 5, "path": "x"});
        assert_eq!(call_shape_hash(&a), call_shape_hash(&b));
    }

    #[test]
    fn call_shape_hash_collides_on_same_keys_different_values() {
        // Intentional collision — shape == keyset, not values.
        let a = json!({"path": "a"});
        let b = json!({"path": "different_value"});
        assert_eq!(call_shape_hash(&a), call_shape_hash(&b));
    }

    #[test]
    fn classify_response_ok_when_not_error() {
        assert_eq!(classify_response(false, None), ResponseKind::Ok);
        assert_eq!(
            classify_response(false, Some("doesnt_matter")),
            ResponseKind::Ok
        );
    }

    #[test]
    fn classify_response_hint_routing() {
        assert_eq!(
            classify_response(true, Some("invalid_args: missing key 'path'")),
            ResponseKind::ErrInvalidArgs
        );
        assert_eq!(
            classify_response(true, Some("not_found")),
            ResponseKind::ErrToolNotFound
        );
        assert_eq!(
            classify_response(true, Some("timeout exceeded")),
            ResponseKind::ErrTimeout
        );
        assert_eq!(
            classify_response(true, Some("server error 500")),
            ResponseKind::ErrServer
        );
        assert_eq!(
            classify_response(true, Some("???")),
            ResponseKind::ErrOther
        );
    }
}
