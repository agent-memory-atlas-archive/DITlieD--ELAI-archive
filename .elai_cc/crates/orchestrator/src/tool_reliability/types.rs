//! E-18 — Shared types for the reliability gate. Kept in a leaf module so
//! `gate.rs` / `spill.rs` / `retry.rs` can import the verdict + reject
//! enums without depending on each other.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Inline payload cap (bytes). Mirrors the plan's "8K tokens" using a
/// byte-count proxy: at 4 chars/token typical we land near 32 KiB, but we
/// use a conservative 8 KiB ceiling so spills happen earlier (matches the
/// long-context degradation curve from arxiv:2505.10570 LongFuncEval where
/// regression begins well before the model's nominal context window).
pub const TOOL_RESULT_INLINE_CAP_BYTES: usize = 8 * 1024;

/// Outcome of `ToolReliabilityGate::check`. Three terminal variants —
/// callers fan out to dispatch (Pass), an error path (Reject), or the
/// spill subsystem (Spill).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliabilityVerdict {
    /// The call is well-formed and within inline cap; dispatch proceeds.
    Pass,
    /// The call is rejected outright. The dispatcher MUST NOT execute
    /// the tool and SHOULD emit `ToolReliabilityGateRejected` SSB with
    /// the embedded reason.
    Reject { reason: ReliabilityRejectReason },
    /// The call body is too large for inline IPC. The dispatcher MUST
    /// route the payload through [`crate::tool_reliability::spill::spill_call`]
    /// and pass the returned [`crate::tool_reliability::spill::SpillRecord`]
    /// back to the caller as the IPC ToolResult `spill_ref` (when wire
    /// extension lands) or as an out-of-band SSB pointer.
    Spill { reason: SpillReason, payload_bytes: usize },
}

/// Why the gate refused. Kept narrow on purpose — operators read the
/// dashboard counters by variant; the `details` String carries any
/// non-enum specifics (tool name, decoder error, etc) for the SSB row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliabilityRejectReason {
    /// Args failed schema / fence pre-flight (E-35 DelimiterCollisionGuard
    /// fence-collision OR the tool registry refused the args shape).
    MalformedArgs { details: String },

    /// Edit / Bash targeted a package-manifest file (Cargo.toml /
    /// pyproject.toml / package.json) and D-11 RegistryCheck refused
    /// the referenced package (unknown / squat / cool-down window).
    /// E-18 composes D-11 here; the D-11 verdict is the source of truth.
    PackageRefRejected { manifest_path: String, pkg: String },

    /// Tool dispatch caller exceeded the retry budget. Carries the
    /// final attempt count so the SSB row + dashboard tile can render
    /// the escalation context.
    RetryBudgetExhausted { attempts: u32 },

    /// Path-allowlist pre-filter fired. The agent attempted to operate
    /// on a path outside the GroundingRequest allowlist. Carries the
    /// rejected path so the SSB row's hash anchors a deterministic
    /// blame trail.
    PathAllowlistViolation { path: String },
}

/// Why a payload spilled to disk instead of staying inline. We surface
/// the cause so the dashboard can distinguish "huge tool output"
/// (`OverInlineCap`) from "operator requested" (`ForcedByConfig`) when
/// triaging spill-frequency outliers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpillReason {
    /// Body bytes exceeded [`TOOL_RESULT_INLINE_CAP_BYTES`].
    OverInlineCap,
    /// Operator set `ELAI_TOOL_RELIABILITY_FORCE_SPILL=1`. Useful for
    /// flushing the spill subsystem in tests + bench harnesses.
    ForcedByConfig,
}

/// Minimal snapshot of a tool call passed into the gate. The gate
/// itself is decoupled from the C-35 `McpCallResult` / plugins types so
/// it can be re-used by orchestrator-side dispatch paths (which don't
/// pull the plugins crate) AND the plugins-side `mcp_bridge` path.
///
/// Fields are intentionally `String` / `serde_json::Value` rather than
/// references — the call must be cloneable into a spill record so we
/// own the snapshot for the lifetime of the IO write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallSnapshot {
    pub tool_name: String,
    pub args: serde_json::Value,
    /// Optional: the target manifest path when the call touches a
    /// package manifest (Cargo.toml / pyproject.toml / package.json).
    /// Drives [`ReliabilityRejectReason::PackageRefRejected`] composition
    /// with D-11.
    pub manifest_path: Option<String>,
    /// Optional: the package name extracted from the manifest diff,
    /// hashed and stored in the SSB row downstream.
    pub package_name: Option<String>,
}

impl ToolCallSnapshot {
    /// Convenience constructor for tests + dispatcher call sites.
    pub fn new(tool_name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            args,
            manifest_path: None,
            package_name: None,
        }
    }

    /// Builder-style setter for the manifest path.
    pub fn with_manifest_path(mut self, path: impl Into<String>) -> Self {
        self.manifest_path = Some(path.into());
        self
    }

    /// Builder-style setter for the package name (post-extraction).
    pub fn with_package_name(mut self, pkg: impl Into<String>) -> Self {
        self.package_name = Some(pkg.into());
        self
    }

    /// Approx body bytes — used by [`crate::tool_reliability::gate::ToolReliabilityGate::check`]
    /// to decide between Pass / Spill. Serializes via `serde_json::to_vec`
    /// so test harnesses get a deterministic byte count.
    pub fn approx_bytes(&self) -> usize {
        match serde_json::to_vec(&self.args) {
            Ok(v) => v.len() + self.tool_name.len(),
            Err(_) => self.tool_name.len(),
        }
    }
}

/// Side-channel passed back by `spill_call` so the dispatcher can plumb
/// the spill path into the IPC ToolResult and the SSB
/// `ToolCallSpilled` row. Kept here (instead of `spill.rs`) so types.rs
/// remains the canonical leaf import for downstream crates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpillManifest {
    pub path: PathBuf,
    pub payload_bytes: usize,
    pub blake3_hex: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn inline_cap_is_8kib() {
        assert_eq!(TOOL_RESULT_INLINE_CAP_BYTES, 8 * 1024);
    }

    #[test]
    fn snapshot_builder_chains() {
        let s = ToolCallSnapshot::new("edit", json!({"k":"v"}))
            .with_manifest_path("Cargo.toml")
            .with_package_name("tokio");
        assert_eq!(s.tool_name, "edit");
        assert_eq!(s.manifest_path.as_deref(), Some("Cargo.toml"));
        assert_eq!(s.package_name.as_deref(), Some("tokio"));
    }

    #[test]
    fn snapshot_approx_bytes_includes_name_and_args() {
        let s = ToolCallSnapshot::new("edit", json!({"path": "src/lib.rs"}));
        let b = s.approx_bytes();
        // {"path":"src/lib.rs"} is at least 20 bytes; plus "edit" name = 24+.
        assert!(b > 20, "approx_bytes too small: {b}");
    }

    #[test]
    fn verdict_serde_roundtrip() {
        let v = ReliabilityVerdict::Spill {
            reason: SpillReason::OverInlineCap,
            payload_bytes: 16384,
        };
        let s = serde_json::to_string(&v).expect("serialize");
        let back: ReliabilityVerdict = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(v, back);
    }

    #[test]
    fn reject_reason_serde_roundtrip() {
        let r = ReliabilityRejectReason::PackageRefRejected {
            manifest_path: "Cargo.toml".into(),
            pkg: "tokio".into(),
        };
        let s = serde_json::to_string(&r).expect("serialize");
        let back: ReliabilityRejectReason = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(r, back);
    }
}
