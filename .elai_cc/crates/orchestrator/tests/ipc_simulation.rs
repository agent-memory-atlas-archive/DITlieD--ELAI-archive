//! IPC fault simulation tests — green phase (3I.8).
//!
//! Validates DST IpcGateway behavior under fault injection via SimulatorRuntime.
//! These use the DST-specific types (DstIpc*), not the real IPC wire types.

use std::time::Duration;

use elai_orchestrator::agent_pool::AgentRole;
use elai_orchestrator::dst::{SimFault, SimulatorRuntime};
use elai_orchestrator::ipc::dst_ipc::{DstIpcError, DstIpcGateway, DstIpcMessage};

// Type aliases to minimize diff noise in test bodies.
type IpcMessage = DstIpcMessage;
type IpcGateway = DstIpcGateway;
type IpcError = DstIpcError;

/// Inject a broadcast NetworkPartition fault, then assert that send returns
/// a fault-specific error (ConnectionLost, not NotImplemented).
#[test]
fn ipc_packet_loss() {
    let mut sim = SimulatorRuntime::new(42);
    sim.inject_fault(SimFault::NetworkPartition {
        duration: Duration::from_millis(500),
        from_node: None,
        to_node: None,
    });

    let mut gw = IpcGateway::new(1);
    let result = gw.send(
        IpcMessage::ToolResult {
            data: vec![1, 2, 3],
        },
        &mut sim,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err == IpcError::MessageDropped || err == IpcError::ConnectionLost,
        "expected fault-specific error, got: {err}",
    );
}

/// Inject asymmetric fault: node 1 -> node 2 blocked, reverse open.
/// Forward send should fail; reverse gateway should succeed.
#[test]
fn ipc_asymmetric() {
    let mut sim = SimulatorRuntime::new(99);
    sim.inject_fault(SimFault::NetworkPartition {
        duration: Duration::from_secs(1),
        from_node: Some(1),
        to_node: Some(2),
    });

    let mut forward = IpcGateway::new(1);
    let mut reverse = IpcGateway::new(2);
    let mut bystander = IpcGateway::new(3);

    let fwd_result = forward.send(
        IpcMessage::AgentReady {
            role: AgentRole::Executor,
        },
        &mut sim,
    );
    let rev_result = reverse.send(
        IpcMessage::AgentReady {
            role: AgentRole::Auditor,
        },
        &mut sim,
    );
    let bystander_result = bystander.send(
        IpcMessage::AgentReady {
            role: AgentRole::Researcher,
        },
        &mut sim,
    );

    assert!(fwd_result.is_err());
    assert!(
        fwd_result.unwrap_err() == IpcError::ConnectionLost,
        "forward direction should be blocked",
    );
    assert!(rev_result.is_ok(), "reverse direction should be open");
    assert!(
        bystander_result.is_ok(),
        "bystander node 3 should not be affected by 1->2 partition",
    );
}

/// Inject AgentCrash fault, then assert the gateway detects the dead node
/// and SSB row count remains 0 (no orphaned rows from the crash).
#[test]
fn ipc_agent_crash() {
    let mut sim = SimulatorRuntime::new(7);
    sim.inject_fault(SimFault::AgentCrash {
        role: AgentRole::Researcher,
    });

    let mut gw = IpcGateway::new(1);
    let result = gw.receive(&mut sim);

    assert!(result.is_err());
    assert!(
        result.unwrap_err() == IpcError::NodeDead,
        "should detect dead remote node",
    );

    let snap = sim.snapshot_state();
    assert_eq!(snap.ssb_row_count, 0, "crash must not leave orphaned SSB rows");
}

/// Inject 5 consecutive NetworkPartition faults (burst), then clear.
/// The 6th send should succeed, verifying the gateway recovers after faults expire.
#[test]
fn ipc_burst_loss() {
    let mut sim = SimulatorRuntime::new(256);

    for _ in 0..5 {
        sim.inject_fault(SimFault::NetworkPartition {
            duration: Duration::from_millis(10),
            from_node: None,
            to_node: None,
        });
    }

    let mut gw = IpcGateway::new(1);

    let burst_result = gw.send(IpcMessage::ToolResult { data: vec![0] }, &mut sim);
    assert!(burst_result.is_err(), "send during burst should fail");

    sim.advance_time(Duration::from_millis(100));

    let recovery_result = gw.send(
        IpcMessage::AgentReady {
            role: AgentRole::Executor,
        },
        &mut sim,
    );
    assert!(
        recovery_result.is_ok(),
        "send after burst clearance should succeed, got: {:?}",
        recovery_result,
    );
}
