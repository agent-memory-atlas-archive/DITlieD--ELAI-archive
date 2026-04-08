use std::collections::VecDeque;

use crate::agent_pool::AgentRole;
use crate::dst::SimulatorRuntime;
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
    #[error("connection lost to remote node")]
    ConnectionLost,
    #[error("message dropped due to network fault")]
    MessageDropped,
    #[error("remote node is dead")]
    NodeDead,
    #[error("no message available in receive queue")]
    NoMessage,
}

// ---------------------------------------------------------------------------
// IpcGateway — stub gateway for inter-agent communication
// ---------------------------------------------------------------------------

pub struct IpcGateway {
    pub(crate) node_id: u64,
    msg_queue: VecDeque<IpcMessage>,
}

impl IpcGateway {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            msg_queue: VecDeque::new(),
        }
    }

    /// Enqueue a message for later retrieval via `receive`.
    pub fn enqueue(&mut self, msg: IpcMessage) {
        self.msg_queue.push_back(msg);
    }

    /// Send a message through this gateway. Checks the simulator's fault queue
    /// for active `NetworkPartition` faults matching this node as sender.
    pub fn send(
        &mut self,
        _msg: IpcMessage,
        sim: &mut SimulatorRuntime,
    ) -> Result<(), IpcError> {
        if sim.has_active_agent_crash() {
            return Err(IpcError::NodeDead);
        }
        if sim.has_active_partition(self.node_id, None).is_some() {
            return Err(IpcError::ConnectionLost);
        }
        Ok(())
    }

    /// Receive a message from the internal queue. Checks the simulator's fault
    /// queue for active `AgentCrash` or `NetworkPartition` faults first.
    pub fn receive(&mut self, sim: &mut SimulatorRuntime) -> Result<IpcMessage, IpcError> {
        if sim.has_active_agent_crash() {
            return Err(IpcError::NodeDead);
        }
        if sim.has_active_partition(self.node_id, None).is_some() {
            return Err(IpcError::ConnectionLost);
        }
        self.msg_queue.pop_front().ok_or(IpcError::NoMessage)
    }
}
