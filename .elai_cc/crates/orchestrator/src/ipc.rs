use crate::agent_pool::AgentRole;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// IpcMessage — messages exchanged between agents over the IPC channel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcMessage {
    AgentReady { role: AgentRole },
    AgentDead { role: AgentRole },
    ToolResult { data: Vec<u8> },
}

// ---------------------------------------------------------------------------
// IpcError — typed errors for IPC operations (thiserror)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IpcError {
    #[error("IPC operation not implemented (stub — see 3I.8)")]
    NotImplemented,
    #[error("connection lost to remote node")]
    ConnectionLost,
    #[error("message dropped due to network fault")]
    MessageDropped,
    #[error("remote node is dead")]
    NodeDead,
}

// ---------------------------------------------------------------------------
// IpcGateway — stub gateway for inter-agent communication
// ---------------------------------------------------------------------------

pub struct IpcGateway {
    pub(crate) node_id: u64,
}

impl IpcGateway {
    pub fn new(node_id: u64) -> Self {
        Self { node_id }
    }

    pub fn send(&mut self, _msg: IpcMessage) -> Result<(), IpcError> {
        Err(IpcError::NotImplemented)
    }

    pub fn receive(&mut self) -> Result<IpcMessage, IpcError> {
        Err(IpcError::NotImplemented)
    }
}
