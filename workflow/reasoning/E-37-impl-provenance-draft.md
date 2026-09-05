E-37 Implementation Draft: provenance.rs
Session: s-1233230 | Date: 2026-05-21
Copy to .elai_cc/crates/safety/src/provenance.rs on resume

CRITICAL: Named DataProvenance (not Provenance) to avoid collision with memory::Provenance (C-48)
CRITICAL: Named CapGateDecision (not GateDecision) to avoid collision with 3 existing types

```rust
//! Data-plane provenance tagging for ELAI agent context values.
//!
//! Every value that flows into agent argv gets a `Provenanced<T>` wrapper.
//! The tagging is METADATA only — it cannot be forged by content (matches
//! CaMeL formalism arxiv:2503.18813). The capability gate (capability_gate.rs)
//! rejects destructive tool calls where any argv carries WebTier* or McpTool provenance.
//!
//! NAMING: This type is `DataProvenance` (not `Provenance`) to avoid collision
//! with `elai_memory::Provenance` (C-48, crates/memory/src/trust.rs:62).

use std::fmt;

/// Provenance of a value entering agent context.
///
/// Variants are ordered from most-trusted (User, System) to least-trusted (WebTier*).
/// The capability gate rejects tool calls where any argv carries WebTier* or McpTool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataProvenance {
    /// Value originated from the authenticated user (chat input, confirmed file path).
    User,
    /// Value originated from ELAI system code (process args, internal constants).
    System,
    /// Web content from a tier-A source (allowlisted search engine result summary).
    WebTierA,
    /// Web content from a tier-B source (allowlisted page fetch, parsed structured data).
    WebTierB,
    /// Web content from a tier-C source (arbitrary fetched page body, untrusted HTML).
    WebTierC,
    /// Output from an MCP tool call.
    McpTool {
        server_url: String,
        tool_name: String,
        /// blake3 hash of the tool description at call time (for drift detection).
        desc_blake3: [u8; 32],
    },
    /// Content from a user file upload.
    Upload { source_path: String },
    /// Output from a sandboxed subprocess.
    SandboxOutput { sandbox_id: String },
    /// Value retrieved from ELAI memory store.
    Memory { source_key: String },
}

impl DataProvenance {
    /// Returns true if this provenance requires capability gate enforcement.
    /// Web and MCP sources are untrusted; User and System are trusted.
    pub fn requires_gate(&self) -> bool {
        matches!(
            self,
            DataProvenance::WebTierA
                | DataProvenance::WebTierB
                | DataProvenance::WebTierC
                | DataProvenance::McpTool { .. }
                | DataProvenance::Upload { .. }
                | DataProvenance::SandboxOutput { .. }
        )
    }

    /// Returns true if this value can appear in destructive tool argv.
    pub fn is_gate_safe(&self) -> bool {
        matches!(self, DataProvenance::User | DataProvenance::System | DataProvenance::Memory { .. })
    }
}

impl fmt::Display for DataProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataProvenance::User => write!(f, "User"),
            DataProvenance::System => write!(f, "System"),
            DataProvenance::WebTierA => write!(f, "WebTierA"),
            DataProvenance::WebTierB => write!(f, "WebTierB"),
            DataProvenance::WebTierC => write!(f, "WebTierC"),
            DataProvenance::McpTool { server_url, tool_name, .. } =>
                write!(f, "McpTool({server_url}/{tool_name})"),
            DataProvenance::Upload { source_path } => write!(f, "Upload({source_path})"),
            DataProvenance::SandboxOutput { sandbox_id } => write!(f, "SandboxOutput({sandbox_id})"),
            DataProvenance::Memory { source_key } => write!(f, "Memory({source_key})"),
        }
    }
}

/// A value with its data-plane provenance tag attached.
///
/// The provenance is METADATA — it lives outside the value string and
/// cannot be overwritten by the value's content.
#[derive(Debug, Clone)]
pub struct Provenanced<T> {
    pub value: T,
    pub provenance: DataProvenance,
}

impl<T> Provenanced<T> {
    pub fn new(value: T, provenance: DataProvenance) -> Self {
        Self { value, provenance }
    }

    pub fn user(value: T) -> Self {
        Self::new(value, DataProvenance::User)
    }

    pub fn system(value: T) -> Self {
        Self::new(value, DataProvenance::System)
    }

    pub fn web_tier_c(value: T) -> Self {
        Self::new(value, DataProvenance::WebTierC)
    }

    pub fn requires_gate(&self) -> bool {
        self.provenance.requires_gate()
    }
}

impl<T: fmt::Display> fmt::Display for Provenanced<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[prov={}]", self.value, self.provenance)
    }
}

/// Decision from the capability gate.
///
/// NAMING: CapGateDecision (not GateDecision) to avoid collision with:
/// - orchestrator::gate::GateDecision { Approve, Reject, Modify, Redirect }
/// - tools::action_classifier::GateDecision { Allow, ConfirmationRequired, Block }
/// - orchestrator::ssb::GateDecisionPayload
#[derive(Debug, Clone, PartialEq)]
pub enum CapGateDecision {
    Allow,
    Reject(String),
}
```
