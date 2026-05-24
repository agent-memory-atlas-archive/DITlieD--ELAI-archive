//! Empire integration wire-format value objects.
//!
//! All types here are serde-friendly so they can cross both the FactoryLine
//! protocol (newline-delimited JSON over AF_UNIX / TCP) and the Tauri
//! command boundary. The 5 `EmpireEventKind` variants line up with the 5
//! SSB action types planned in B-25 (empire_connected, empire_disconnected,
//! empire_event_received, empire_capability_invoked, empire_repair_proposed)
//! so the bridge can dispatch a single typed enum into the lockstep when
//! Empire is live (EMPIRE-PENDING on the dispatcher side).
//!
//! NOTE on `ts-rs`: this workspace does NOT use the `ts-rs` derive
//! generator. Cross-boundary types stay plain `serde::{Serialize,
//! Deserialize}` and the TS counterparts are hand-authored in
//! `dashboard-v2/src/lib/api/empire.ts` to match. The mirror pattern is the
//! same one used by `media_validator` and `commands_e46`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// FactoryLine protocol (brain<->factory bus, src/core/bus.py reference)
// ---------------------------------------------------------------------------

/// One of the four FactoryLine bus verbs the brain side speaks.
///
/// Grounding: empire.txt section 1 + 4J.1 wire-format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactoryCmd {
    /// Initial registration of a brain with the factory.
    Register,
    /// Periodic heartbeat ping (configurable interval).
    Heartbeat,
    /// Request the factory's list of known brains.
    List,
    /// Brain asks the factory to push a refreshed view of state.
    RequestUpdate,
}

/// Brain-side FactoryLine command frame.
///
/// Wire format: newline-delimited JSON, exactly one `\n` per message (NOT
/// `\r\n`). `v` is the protocol version (currently 1 per Empire bus.py).
/// `Eq` is intentionally NOT derived because `ts: f64` cannot satisfy `Eq`;
/// callers that need value-equality use `PartialEq` (the only place this
/// matters is the unit tests below, which round-trip the same exact f64).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryLineCommand {
    pub cmd: FactoryCmd,
    pub v: u8,
    pub brain: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pid: Option<u32>,
    /// Floating-point UNIX timestamp (seconds) — matches Python `time.time()`.
    pub ts: f64,
}

/// FactoryLine response frame (factory -> brain).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryLineResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Brain manifest (replaces brain.json -- schema-validated)
// ---------------------------------------------------------------------------

/// Empire engine tier this brain is rated for.
///
/// Mirrors the engine hierarchy MKI -> MKII -> MKIII documented in
/// empire.txt section 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EngineTier {
    Mki,
    Mkii,
    Mkiii,
}

/// One hook binding inside a neuron's registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookBinding {
    pub kind: HookKind,
    pub pattern: String,
}

/// HookRegistry kinds documented in empire.txt section 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    Route,
    Event,
}

/// A single capability surfaced from a neuron to the brain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

/// A neuron (module) inside a brain manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeuronRegistration {
    pub neuron_id: String,
    pub role: String,
    pub hooks: Vec<HookBinding>,
}

// ---------------------------------------------------------------------------
// Factory line snapshot (read surface for the dashboard)
// ---------------------------------------------------------------------------

/// One line/brain row returned by the bridge for the EmpireBridgePanel.
///
/// While the bridge is `Disconnected`, this list is empty (graceful
/// degradation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryLine {
    pub brain_id: String,
    pub engine_tier: EngineTier,
    pub neuron_count: u32,
    pub last_heartbeat_ms: i64,
    pub status: String,
}

// ---------------------------------------------------------------------------
// SSB action-kind enum (host-side mirror of the 5 B-25 variants)
// ---------------------------------------------------------------------------

/// Empire-side event kind dispatched by `EmpireBridge::emit_action`.
///
/// These five variants line up 1:1 with the 5 SSB action types planned in
/// the B-25 WIRING_CONTRACT:
///   - `Connected`         -> `empire_connected`
///   - `Disconnected`      -> `empire_disconnected`
///   - `EventReceived`     -> `empire_event_received`
///   - `CapabilityInvoked` -> `empire_capability_invoked`
///   - `RepairProposed`    -> `empire_repair_proposed`
///
/// EMPIRE-PENDING: the actual SSB lockstep (9-tuple per variant: enum arm +
/// SQL CHECK + as_sql_str + from_str + SsbPayload variant + action_type
/// match + payload struct + test_payload + SCHEMA_VERSION bump from 97
/// -> 98) is gated on Empire deployment; the bridge holds this typed
/// enum so that future patch is a one-place dispatch flip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum EmpireEventKind {
    Connected {
        brain_id: String,
        protocol_version: u8,
    },
    Disconnected {
        brain_id: String,
        reason: String,
    },
    EventReceived {
        source: String,
        event_type: String,
        payload: serde_json::Value,
    },
    CapabilityInvoked {
        capability: String,
        caller: String,
        allowed: bool,
    },
    RepairProposed {
        issue: String,
        target_path: String,
        fix_kind: String,
    },
}

/// A typed event the bridge surfaces both to the SSB writer and to the
/// dashboard read surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpireEvent {
    pub timestamp_ms: i64,
    pub kind: EmpireEventKind,
}

// ---------------------------------------------------------------------------
// ElaiNeuron — capability whitelist (exactly 4)
// ---------------------------------------------------------------------------

/// `ElaiNeuron` is elai_cc's neuron-side surface registered with Empire.
///
/// The capability whitelist is HARDCODED to exactly four entries
/// (task_submit, task_status, pipeline_result, health_check) per
/// phase-4J 4J.5 ACCEPTANCE. Internal capabilities (fsm_transition,
/// evidence_gate, rules_query, ssb_direct_write, config_modify,
/// agent_spawn) are NEVER added here — adversarial test in 4J.5 asserts
/// the count is exactly 4.
///
/// `tool_filter` reuses the existing locked `elai_ipc_types::ToolFilter`
/// per phase-4J-decisions.md F6 ("reuse existing locked type, do not invent
/// a new one"). The 4-capability whitelist on the `capabilities` field maps
/// 1:1 to the 4 `ToolId` entries in `tool_filter.allowed_tools`. The MCP
/// dispatcher consults `tool_filter.allows(&ToolId)` before routing any
/// inbound capability invocation so internal tools cannot leak through
/// even if a future code change adds a new `Capability` without updating
/// the filter (defence in depth — the filter is the authoritative gate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElaiNeuron {
    pub capabilities: Vec<Capability>,
    pub tool_filter: elai_ipc_types::ToolFilter,
}

impl ElaiNeuron {
    /// Construct the hardcoded 4-capability whitelist with empty schemas.
    /// Real schemas are wired by the orchestrator at construction time when
    /// the empire feature is live; this constructor exists so contract
    /// tests can assert the cardinality without an Empire round-trip.
    pub fn whitelist() -> Self {
        let names = [
            "task_submit",
            "task_status",
            "pipeline_result",
            "health_check",
        ];
        let capabilities: Vec<Capability> = names
            .iter()
            .map(|name| Capability {
                name: (*name).to_string(),
                description: format!("Empire-exposed capability: {name}"),
                input_schema: serde_json::Value::Object(Default::default()),
                output_schema: serde_json::Value::Object(Default::default()),
            })
            .collect();
        let tool_filter = elai_ipc_types::ToolFilter {
            allowed_tools: names
                .iter()
                .map(|n| elai_ipc_types::ToolId::new(*n))
                .collect(),
        };
        Self {
            capabilities,
            tool_filter,
        }
    }

    /// Authoritative capability gate — consulted by the MCP dispatcher
    /// before any inbound capability invocation is routed. Returns `false`
    /// for any capability not on the 4-entry whitelist, even if a
    /// `Capability` row claims it. The filter is the single source of
    /// truth; the dispatcher does not iterate `self.capabilities`.
    pub fn allows(&self, capability_name: &str) -> bool {
        self.tool_filter
            .allows(&elai_ipc_types::ToolId::new(capability_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_cmd_round_trip() {
        let cmd = FactoryLineCommand {
            cmd: FactoryCmd::Register,
            v: 1,
            brain: "elai_cc".to_string(),
            dir: Some(PathBuf::from("/var/empire/elai_cc")),
            pid: Some(12345),
            ts: 1747934400.0,
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        let back: FactoryLineCommand = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cmd, back);
    }

    #[test]
    fn elai_neuron_has_exactly_four_capabilities() {
        let n = ElaiNeuron::whitelist();
        assert_eq!(n.capabilities.len(), 4);
        let names: Vec<&str> = n.capabilities.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"task_submit"));
        assert!(names.contains(&"task_status"));
        assert!(names.contains(&"pipeline_result"));
        assert!(names.contains(&"health_check"));
    }

    #[test]
    fn elai_neuron_tool_filter_mirrors_whitelist() {
        let n = ElaiNeuron::whitelist();
        assert_eq!(n.tool_filter.allowed_tools.len(), 4);
        // The filter must allow every capability that is on the whitelist.
        assert!(n.allows("task_submit"));
        assert!(n.allows("task_status"));
        assert!(n.allows("pipeline_result"));
        assert!(n.allows("health_check"));
    }

    #[test]
    fn elai_neuron_tool_filter_denies_internal_capabilities() {
        let n = ElaiNeuron::whitelist();
        // Adversarial: every internal capability mentioned in the spec MUST
        // be rejected. If a future commit accidentally adds one of these to
        // ::whitelist(), this test fires before the build ships.
        for internal in [
            "fsm_transition",
            "evidence_gate",
            "rules_query",
            "ssb_direct_write",
            "config_modify",
            "agent_spawn",
        ] {
            assert!(!n.allows(internal), "internal {internal} must be denied");
        }
    }

    #[test]
    fn engine_tier_serializes_uppercase() {
        let s = serde_json::to_string(&EngineTier::Mkii).expect("serialize");
        assert_eq!(s, "\"MKII\"");
    }
}
