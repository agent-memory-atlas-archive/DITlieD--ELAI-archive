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

// === Tranche B-6 quad C-47 -- CaseBase emit helpers ===========================
//
// Three new entry-points that wrap [`AggregatedCase`] mutations with a
// matching [`CaseBaseSsbEmitter`] notification: `add_case`, `select_case`,
// `apply_calibration`. The emitter trait carries default no-op impls for
// these three methods (see `registry.rs::CaseBaseSsbEmitter`), so legacy
// emitters keep compiling during the one-release-window migration.
//
// These helpers DO NOT change the existing 3 emit_call_logged / emit_hint /
// emit_drift surfaces. Their callers gradually migrate to the new Pattern-2
// [`crate::ssb::ToolCalibrationEvent`] variant in C-47b.

use super::registry::CaseBaseSsbEmitter;

/// Outcome returned by [`apply_calibration`]. Stable wire labels for the
/// `details_json.outcome` field on the matching SSB row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationOutcome {
    /// Calibration applied successfully (hint adoption / sampling
    /// adjustment / etc).
    Applied,
    /// Calibration skipped because no matching case was found.
    NoCase,
    /// Calibration deliberately suppressed (hinter suppression flag,
    /// env-gate off, etc).
    Suppressed,
}

impl CalibrationOutcome {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::NoCase => "no_case",
            Self::Suppressed => "suppressed",
        }
    }
}

/// Tranche B-6 quad C-47 -- admit a fresh case + notify the emitter.
///
/// `case` is moved into the caller's case-base storage by `store_fn`; on
/// successful store the [`CaseBaseSsbEmitter::emit_case_added`] hook fires
/// with the canonical (tool_name, call_shape_hash, ts_ms) tuple. Wire-dark
/// unless `ELAI_C47_CASE_BASE_ENABLE=1` is set on the orchestrator-side
/// adapter.
///
/// `store_fn` returns `true` when the case was admitted (e.g. LRU still
/// had room) and `false` when it was rejected (e.g. duplicate). Emit only
/// fires on `true` so SSB rows track ACTUAL case-base growth.
pub fn add_case<F>(
    case: AggregatedCase,
    emitter: &dyn CaseBaseSsbEmitter,
    ts_ms: i64,
    store_fn: F,
) -> bool
where
    F: FnOnce(AggregatedCase) -> bool,
{
    let tool_name = case.tool_name.clone();
    let call_shape_hash = case.call_shape_hash;
    let admitted = store_fn(case);
    if admitted {
        // Drop emit errors silently: SSB writes are best-effort per the
        // existing `dispatch_payload` semantics in
        // tool_calibration_ssb_adapter.rs.
        let _ = emitter.emit_case_added(&tool_name, call_shape_hash, ts_ms);
    }
    admitted
}

/// Tranche B-6 quad C-47 -- look up a case in the base + notify the emitter.
///
/// `lookup_fn` returns the matched case (cloned out of LRU storage) when
/// the query hits, `None` otherwise. Emit fires only on a hit (the
/// dashboard tracks selection rate, not query rate).
pub fn select_case<F>(
    tool_name: &str,
    call_shape_hash: u64,
    emitter: &dyn CaseBaseSsbEmitter,
    ts_ms: i64,
    lookup_fn: F,
) -> Option<AggregatedCase>
where
    F: FnOnce(&str, u64) -> Option<AggregatedCase>,
{
    let hit = lookup_fn(tool_name, call_shape_hash);
    if hit.is_some() {
        let _ = emitter.emit_case_selected(tool_name, call_shape_hash, ts_ms);
    }
    hit
}

/// Tranche B-6 quad C-47 -- apply a calibration decision + notify the
/// emitter.
///
/// `apply_fn` performs the side-effecting calibration step (e.g. adjust
/// downstream sampling rate, inject a hint). The returned
/// [`CalibrationOutcome`] is mirrored into the SSB row via the
/// `emit_calibration_applied` hook ONLY on
/// [`CalibrationOutcome::Applied`]; `NoCase` / `Suppressed` paths skip
/// emission to avoid noise.
pub fn apply_calibration<F>(
    tool_name: &str,
    call_shape_hash: u64,
    emitter: &dyn CaseBaseSsbEmitter,
    ts_ms: i64,
    apply_fn: F,
) -> CalibrationOutcome
where
    F: FnOnce() -> CalibrationOutcome,
{
    let outcome = apply_fn();
    if matches!(outcome, CalibrationOutcome::Applied) {
        let _ = emitter.emit_calibration_applied(tool_name, call_shape_hash, ts_ms);
    }
    outcome
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

    // === Tranche B-6 quad C-47: add_case / select_case / apply_calibration ===

    use std::sync::Mutex;

    struct CapturingEmitter {
        adds: Mutex<Vec<(String, u64)>>,
        selects: Mutex<Vec<(String, u64)>>,
        applies: Mutex<Vec<(String, u64)>>,
    }

    impl CapturingEmitter {
        fn new() -> Self {
            Self {
                adds: Mutex::new(Vec::new()),
                selects: Mutex::new(Vec::new()),
                applies: Mutex::new(Vec::new()),
            }
        }
    }

    impl crate::tool_calibration::registry::CaseBaseSsbEmitter for CapturingEmitter {
        fn emit_call_logged(
            &self,
            _t: &str,
            _s: u64,
            _k: &str,
            _ok: bool,
            _ts: i64,
        ) -> Result<(), String> {
            Ok(())
        }
        fn emit_hint(&self, _t: &str, _s: u64, _h: &str, _ts: i64) -> Result<(), String> {
            Ok(())
        }
        fn emit_drift(&self, _t: &str, _r: f32, _n: u32, _ts: i64) -> Result<(), String> {
            Ok(())
        }
        fn emit_case_added(&self, t: &str, s: u64, _ts: i64) -> Result<(), String> {
            self.adds.lock().expect("lock").push((t.to_string(), s));
            Ok(())
        }
        fn emit_case_selected(&self, t: &str, s: u64, _ts: i64) -> Result<(), String> {
            self.selects.lock().expect("lock").push((t.to_string(), s));
            Ok(())
        }
        fn emit_calibration_applied(&self, t: &str, s: u64, _ts: i64) -> Result<(), String> {
            self.applies.lock().expect("lock").push((t.to_string(), s));
            Ok(())
        }
    }

    #[test]
    fn add_case_emits_on_admitted_only() {
        let em = CapturingEmitter::new();
        let c1 = AggregatedCase::new("fs.read".into(), 42);
        let admitted = add_case(c1, &em, 1_000, |_| true);
        assert!(admitted);
        assert_eq!(em.adds.lock().expect("lock").len(), 1);

        let c2 = AggregatedCase::new("fs.read".into(), 43);
        let rejected = add_case(c2, &em, 2_000, |_| false);
        assert!(!rejected);
        // Only 1 emit total -- rejected case did NOT fire.
        assert_eq!(em.adds.lock().expect("lock").len(), 1);
    }

    #[test]
    fn select_case_emits_on_hit_only() {
        let em = CapturingEmitter::new();
        let hit = select_case("fs.read", 42, &em, 1_000, |_, _| {
            Some(AggregatedCase::new("fs.read".into(), 42))
        });
        assert!(hit.is_some());
        assert_eq!(em.selects.lock().expect("lock").len(), 1);

        let miss = select_case("fs.read", 99, &em, 2_000, |_, _| None);
        assert!(miss.is_none());
        assert_eq!(em.selects.lock().expect("lock").len(), 1);
    }

    #[test]
    fn apply_calibration_emits_on_applied_only() {
        let em = CapturingEmitter::new();
        let o = apply_calibration("fs.read", 42, &em, 1_000, || CalibrationOutcome::Applied);
        assert_eq!(o, CalibrationOutcome::Applied);
        assert_eq!(em.applies.lock().expect("lock").len(), 1);

        let o2 = apply_calibration("fs.read", 42, &em, 2_000, || CalibrationOutcome::NoCase);
        assert_eq!(o2, CalibrationOutcome::NoCase);
        let o3 = apply_calibration("fs.read", 42, &em, 3_000, || {
            CalibrationOutcome::Suppressed
        });
        assert_eq!(o3, CalibrationOutcome::Suppressed);
        // NoCase + Suppressed both skip emit.
        assert_eq!(em.applies.lock().expect("lock").len(), 1);
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
