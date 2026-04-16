// IPC module — real types in ipc_types, DST stub in dst_ipc submodule.

// Re-export real IPC types for use across the codebase.
pub use crate::ipc_types::*;

// ---------------------------------------------------------------------------
// DST stub — SimulatorRuntime-coupled types for deterministic simulation
// testing. Kept always-compiled because integration tests (external crates)
// need access. No production pipeline code should use these types.
// ---------------------------------------------------------------------------

pub mod dst_ipc {
    use std::collections::VecDeque;

    use crate::agent_pool::AgentRole;
    use crate::dst::SimulatorRuntime;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DstIpcMessage {
        AgentReady { role: AgentRole },
        AgentDead { role: AgentRole },
        ToolResult { data: Vec<u8> },
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum DstIpcError {
        #[error("connection lost to remote node")]
        ConnectionLost,
        #[error("message dropped due to network fault")]
        MessageDropped,
        #[error("remote node is dead")]
        NodeDead,
        #[error("no message available in receive queue")]
        NoMessage,
    }

    pub struct DstIpcGateway {
        pub(crate) node_id: u64,
        msg_queue: VecDeque<DstIpcMessage>,
    }

    impl DstIpcGateway {
        pub fn new(node_id: u64) -> Self {
            Self {
                node_id,
                msg_queue: VecDeque::new(),
            }
        }

        pub fn enqueue(&mut self, msg: DstIpcMessage) {
            self.msg_queue.push_back(msg);
        }

        pub fn send(
            &mut self,
            _msg: DstIpcMessage,
            sim: &mut SimulatorRuntime,
        ) -> Result<(), DstIpcError> {
            if sim.has_active_agent_crash() {
                return Err(DstIpcError::NodeDead);
            }
            if sim.has_active_partition(self.node_id, None).is_some() {
                return Err(DstIpcError::ConnectionLost);
            }
            Ok(())
        }

        pub fn receive(
            &mut self,
            sim: &mut SimulatorRuntime,
        ) -> Result<DstIpcMessage, DstIpcError> {
            if sim.has_active_agent_crash() {
                return Err(DstIpcError::NodeDead);
            }
            if sim.has_active_partition(self.node_id, None).is_some() {
                return Err(DstIpcError::ConnectionLost);
            }
            self.msg_queue.pop_front().ok_or(DstIpcError::NoMessage)
        }
    }
}
