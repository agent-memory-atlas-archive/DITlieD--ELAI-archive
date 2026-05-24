//! E-18 — Tool reliability gate.
//!
//! `ToolReliabilityGate::check` is a pure function over a
//! [`super::types::ToolCallSnapshot`]: it returns a
//! [`super::types::ReliabilityVerdict`] (Pass / Reject / Spill) without
//! touching the network or the filesystem. The dispatcher fans out to
//! the spill subsystem only when the verdict is `Spill`.
//!
//! Composition order (locked, see module docs):
//! ```text
//!   AuthzPolicy (C-42)
//!   -> ToolGate (C-35)
//!   -> ToolCalibrationRegistry::consult (C-47)
//!   -> ToolReliabilityGate::check (E-18, this module)
//!   -> dispatch (plugins::mcp_bridge::http_call_tool)
//!   -> ToolCalibrationRegistry::record (C-47)
//! ```
//!
//! Decoupling: the gate does NOT take a `&CalibrationRegistry`. The
//! prior-art consultation happens INSIDE the plugins crate already
//! (C-47 `consult_calibration` is called from `mcp_bridge` before the
//! gate). We pass forward only the bits the gate genuinely needs to
//! make its three-way decision.

use super::types::{
    ReliabilityRejectReason, ReliabilityVerdict, SpillReason, ToolCallSnapshot,
    TOOL_RESULT_INLINE_CAP_BYTES,
};

/// Re-export of the inline cap so callers can budget without reaching
/// into `types::TOOL_RESULT_INLINE_CAP_BYTES` directly. Public API.
pub const RELIABILITY_GATE_INLINE_CAP: usize = TOOL_RESULT_INLINE_CAP_BYTES;

/// Optional inputs to the gate. Kept as a struct so adding future
/// signals (e.g. C-47 calibration hint, runtime feature flags) does
/// NOT break call sites.
#[derive(Debug, Clone, Default)]
pub struct ReliabilityGateInputs<'a> {
    /// When `Some(_)`, the gate consults D-11 package-registry verdict
    /// for package-manifest edits. The dispatcher resolves the actual
    /// [`crate::checker_registry::CheckerKind::RegistryCheck`] call
    /// upstream and passes the boolean result here so the gate stays
    /// synchronous (no async / subprocess from inside `check`).
    pub package_registry_allowed: Option<bool>,
    /// Optional path-allowlist enforced at grammar-mask time upstream.
    /// When the snapshot's args carry a `path` field that does NOT
    /// match the allowlist, the gate emits
    /// [`ReliabilityRejectReason::PathAllowlistViolation`]. Empty slice
    /// means "no allowlist" -> never reject on this axis.
    pub path_allowlist: &'a [String],
    /// Retry attempt count for this `(tool, args-hash)`. When > 0 the
    /// caller is asking the gate to consider this a re-dispatch; we
    /// surface it on `RetryBudgetExhausted` reject.
    pub attempt: u32,
    /// Hard cap on retries. Exceeding triggers
    /// [`ReliabilityRejectReason::RetryBudgetExhausted`]. Default 1 —
    /// matches plan §4 one-shot retry policy.
    pub retry_cap: u32,
}

impl<'a> ReliabilityGateInputs<'a> {
    /// Builder-style construction for tests / dispatchers that only
    /// need to set a subset of fields.
    pub fn new() -> Self {
        Self {
            package_registry_allowed: None,
            path_allowlist: &[],
            attempt: 0,
            retry_cap: 1,
        }
    }

    pub fn with_package_registry_allowed(mut self, ok: bool) -> Self {
        self.package_registry_allowed = Some(ok);
        self
    }

    pub fn with_path_allowlist(mut self, allowlist: &'a [String]) -> Self {
        self.path_allowlist = allowlist;
        self
    }

    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.attempt = attempt;
        self
    }

    pub fn with_retry_cap(mut self, cap: u32) -> Self {
        self.retry_cap = cap;
        self
    }
}

/// Pure reliability gate. See module docs for composition order.
#[derive(Debug, Clone, Copy, Default)]
pub struct ToolReliabilityGate;

impl ToolReliabilityGate {
    pub fn new() -> Self {
        Self
    }

    /// Three-way verdict on the snapshot. Logic order:
    ///
    /// 1. Retry-budget check — escalates first because re-dispatching a
    ///    known-failing call is a waste of every downstream check.
    /// 2. Path-allowlist check — refuse before even hashing args.
    /// 3. Package-registry verdict (when set) — D-11 consult result.
    /// 4. Inline-cap check — spill if the args body itself is over cap
    ///    OR force-spill is enabled. (The post-dispatch result spill
    ///    happens AFTER tool exec; see [`super::spill::spill_call`].)
    /// 5. Default: Pass.
    pub fn check(&self, call: &ToolCallSnapshot, inputs: &ReliabilityGateInputs<'_>) -> ReliabilityVerdict {
        // 1. Retry budget — strict > on attempt vs cap so attempt=cap is
        //    still the final allowed try; cap+1 = exhausted.
        if inputs.attempt > inputs.retry_cap {
            return ReliabilityVerdict::Reject {
                reason: ReliabilityRejectReason::RetryBudgetExhausted {
                    attempts: inputs.attempt,
                },
            };
        }

        // 2. Path-allowlist.
        if !inputs.path_allowlist.is_empty() {
            if let Some(path) = extract_path_arg(&call.args) {
                if !inputs.path_allowlist.iter().any(|p| p == &path) {
                    return ReliabilityVerdict::Reject {
                        reason: ReliabilityRejectReason::PathAllowlistViolation { path },
                    };
                }
            }
        }

        // 3. D-11 package-registry verdict (when the manifest path field
        //    is set on the snapshot AND inputs.package_registry_allowed
        //    is explicitly Some(false)).
        if let Some(false) = inputs.package_registry_allowed {
            if let Some(path) = call.manifest_path.as_deref() {
                let pkg = call.package_name.clone().unwrap_or_else(|| "<unknown>".into());
                return ReliabilityVerdict::Reject {
                    reason: ReliabilityRejectReason::PackageRefRejected {
                        manifest_path: path.into(),
                        pkg,
                    },
                };
            }
        }

        // 4. Inline-cap check. Force-spill env wins regardless of size.
        let bytes = call.approx_bytes();
        if force_spill_enabled() {
            return ReliabilityVerdict::Spill {
                reason: SpillReason::ForcedByConfig,
                payload_bytes: bytes,
            };
        }
        if bytes > TOOL_RESULT_INLINE_CAP_BYTES {
            return ReliabilityVerdict::Spill {
                reason: SpillReason::OverInlineCap,
                payload_bytes: bytes,
            };
        }

        ReliabilityVerdict::Pass
    }
}

/// Extract a path-like arg from the call args for path-allowlist
/// enforcement. We look at `path` (canonical), falling back to
/// `file_path` for sibling tools that use that name. Returns `None`
/// when the args don't carry either field — the allowlist check is
/// skipped in that case (the caller is responsible for tools that
/// take paths via a different name).
fn extract_path_arg(args: &serde_json::Value) -> Option<String> {
    let map = args.as_object()?;
    if let Some(v) = map.get("path").and_then(|v| v.as_str()) {
        return Some(v.to_string());
    }
    if let Some(v) = map.get("file_path").and_then(|v| v.as_str()) {
        return Some(v.to_string());
    }
    None
}

/// Env-gated force-spill toggle. Mirrors C-35's
/// `ELAI_TOOL_GATE_OFF_ENV` convention. Anything truthy = spill every
/// call. Used by test harnesses + the bench runner to flush the spill
/// subsystem; production unsets it.
const FORCE_SPILL_ENV: &str = "ELAI_TOOL_RELIABILITY_FORCE_SPILL";

fn force_spill_enabled() -> bool {
    matches!(
        std::env::var(FORCE_SPILL_ENV)
            .ok()
            .as_deref()
            .map(|s| s.trim().to_ascii_lowercase()),
        Some(ref v) if v == "1" || v == "true" || v == "yes"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pass_when_default_inputs_and_small_args() {
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "src/lib.rs"}));
        let inputs = ReliabilityGateInputs::new();
        let v = gate.check(&call, &inputs);
        assert_eq!(v, ReliabilityVerdict::Pass);
    }

    #[test]
    fn reject_retry_budget_exhausted() {
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "src/lib.rs"}));
        let inputs = ReliabilityGateInputs::new().with_attempt(2).with_retry_cap(1);
        match gate.check(&call, &inputs) {
            ReliabilityVerdict::Reject {
                reason: ReliabilityRejectReason::RetryBudgetExhausted { attempts },
            } => assert_eq!(attempts, 2),
            other => panic!("expected RetryBudgetExhausted reject, got {other:?}"),
        }
    }

    #[test]
    fn reject_path_allowlist_violation() {
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "/tmp/secret"}));
        let allow = vec!["src/lib.rs".to_string()];
        let inputs = ReliabilityGateInputs::new().with_path_allowlist(&allow);
        match gate.check(&call, &inputs) {
            ReliabilityVerdict::Reject {
                reason: ReliabilityRejectReason::PathAllowlistViolation { path },
            } => assert_eq!(path, "/tmp/secret"),
            other => panic!("expected PathAllowlistViolation, got {other:?}"),
        }
    }

    #[test]
    fn pass_when_path_in_allowlist() {
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "src/lib.rs"}));
        let allow = vec!["src/lib.rs".to_string(), "src/main.rs".to_string()];
        let inputs = ReliabilityGateInputs::new().with_path_allowlist(&allow);
        let v = gate.check(&call, &inputs);
        assert_eq!(v, ReliabilityVerdict::Pass);
    }

    #[test]
    fn reject_package_registry_when_explicitly_disallowed() {
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "Cargo.toml"}))
            .with_manifest_path("Cargo.toml")
            .with_package_name("tokyo-async-future-v3");
        let inputs = ReliabilityGateInputs::new().with_package_registry_allowed(false);
        match gate.check(&call, &inputs) {
            ReliabilityVerdict::Reject {
                reason: ReliabilityRejectReason::PackageRefRejected { manifest_path, pkg },
            } => {
                assert_eq!(manifest_path, "Cargo.toml");
                assert_eq!(pkg, "tokyo-async-future-v3");
            }
            other => panic!("expected PackageRefRejected, got {other:?}"),
        }
    }

    #[test]
    fn pass_when_package_registry_allowed() {
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "Cargo.toml"}))
            .with_manifest_path("Cargo.toml")
            .with_package_name("tokio");
        let inputs = ReliabilityGateInputs::new().with_package_registry_allowed(true);
        let v = gate.check(&call, &inputs);
        assert_eq!(v, ReliabilityVerdict::Pass);
    }

    #[test]
    fn spill_when_args_over_cap() {
        let gate = ToolReliabilityGate::new();
        let big = "x".repeat(TOOL_RESULT_INLINE_CAP_BYTES + 128);
        let call = ToolCallSnapshot::new("edit", json!({"path": "src/lib.rs", "body": big}));
        let inputs = ReliabilityGateInputs::new();
        match gate.check(&call, &inputs) {
            ReliabilityVerdict::Spill {
                reason: SpillReason::OverInlineCap,
                payload_bytes,
            } => assert!(payload_bytes > TOOL_RESULT_INLINE_CAP_BYTES),
            other => panic!("expected over-inline-cap Spill, got {other:?}"),
        }
    }

    #[test]
    fn extract_path_arg_handles_both_field_names() {
        assert_eq!(
            extract_path_arg(&json!({"path": "a"})),
            Some("a".to_string())
        );
        assert_eq!(
            extract_path_arg(&json!({"file_path": "b"})),
            Some("b".to_string())
        );
        assert_eq!(extract_path_arg(&json!({"other": "c"})), None);
        assert_eq!(extract_path_arg(&json!(42)), None);
    }

    #[test]
    fn composition_order_retry_beats_path_allowlist() {
        // Retry-budget reject takes precedence over path-allowlist violation —
        // the gate should not spend allowlist-check cycles on an exhausted
        // retry.
        let gate = ToolReliabilityGate::new();
        let call = ToolCallSnapshot::new("edit", json!({"path": "/tmp/not-allowed"}));
        let allow = vec!["src/lib.rs".to_string()];
        let inputs = ReliabilityGateInputs::new()
            .with_attempt(5)
            .with_retry_cap(1)
            .with_path_allowlist(&allow);
        match gate.check(&call, &inputs) {
            ReliabilityVerdict::Reject {
                reason: ReliabilityRejectReason::RetryBudgetExhausted { .. },
            } => {}
            other => panic!("expected retry-budget reject ahead of allowlist, got {other:?}"),
        }
    }
}
