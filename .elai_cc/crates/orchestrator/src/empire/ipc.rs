//! Empire IPC engine-protocol types (module<->engine bus,
//! src/lib/ipc.py reference).
//!
//! IMPORTANT NAMESPACE NOTE
//!
//! The existing internal type `elai_ipc_types::IpcMessageType` (defined in
//! `crates/ipc-types/src/lib.rs`) carries elai_cc's INTERNAL orchestrator<->
//! agent IPC verbs over a postcard, length-prefixed wire. It is consumed by
//! 10+ call sites (feasibility_probe, task_runner, ipc_accept, agent_role,
//! contract tests) and any rename would be a P0 incident. The Empire IPC
//! engine-protocol verbs are a completely different concept (newline-JSON
//! over AF_UNIX, brain<->engine bus). They live HERE under
//! `orchestrator::empire::ipc::EmpireIpcMessageType` to avoid shadowing the
//! existing locked enum. Re-exported at `empire::EmpireIpcMessageType` for
//! convenience.

use serde::{Deserialize, Serialize};

/// One of the nine Empire IPC engine-protocol verbs.
///
/// Grounding: empire.txt section 1 + 4J.1 wire-format types + engine handler
/// types (MKI handles REGISTER_HOOK / EMIT / QUERY_HOOKS / SUBSCRIBE /
/// UNSUBSCRIBE / BUS_CREATE / BUS_JOIN / BUS_LEAVE / BUS_LIST).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmpireIpcMessageType {
    RegisterHook,
    Emit,
    QueryHooks,
    Subscribe,
    Unsubscribe,
    BusCreate,
    BusJoin,
    BusLeave,
    BusList,
}

/// An Empire IPC engine-protocol message envelope (mirrors the Python
/// `make_event()` shape from src/lib/ipc.py).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpireIpcMessage {
    pub msg_type: EmpireIpcMessageType,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub mac: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ip: Option<String>,
}

/// Empire IPC engine-protocol response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpireIpcResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_nine_variants_round_trip() {
        let variants = [
            EmpireIpcMessageType::RegisterHook,
            EmpireIpcMessageType::Emit,
            EmpireIpcMessageType::QueryHooks,
            EmpireIpcMessageType::Subscribe,
            EmpireIpcMessageType::Unsubscribe,
            EmpireIpcMessageType::BusCreate,
            EmpireIpcMessageType::BusJoin,
            EmpireIpcMessageType::BusLeave,
            EmpireIpcMessageType::BusList,
        ];
        for v in variants {
            let s = serde_json::to_string(&v).expect("ser");
            let back: EmpireIpcMessageType = serde_json::from_str(&s).expect("de");
            assert_eq!(v, back);
        }
    }

    #[test]
    fn message_round_trip_preserves_mac_and_ip() {
        let msg = EmpireIpcMessage {
            msg_type: EmpireIpcMessageType::Emit,
            agent: "elai_cc".to_string(),
            channel: Some("factory:deploy".to_string()),
            payload: Some(serde_json::json!({"k": "v"})),
            timestamp: Some(1747934400),
            mac: Some("aa:bb:cc:dd:ee:ff".to_string()),
            ip: Some("10.0.0.5".to_string()),
        };
        let s = serde_json::to_string(&msg).expect("ser");
        let back: EmpireIpcMessage = serde_json::from_str(&s).expect("de");
        assert_eq!(msg, back);
    }
}
