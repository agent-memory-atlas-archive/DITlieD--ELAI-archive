//! IPC fault simulation tests — TDD red phase (3I.6).
//!
//! Defines the interface contract for IpcGateway under fault injection.
//! All 4 tests compile and run but are #[ignore] because SimulatorRuntime::inject_fault
//! is a no-op stub — 3I.8 fills in fault-aware behavior and removes #[ignore].

use std::time::Duration;

use elai_orchestrator::agent_pool::AgentRole;
use elai_orchestrator::dst::{SimFault, SimulatorRuntime};
use elai_orchestrator::ipc::{IpcError, IpcGateway, IpcMessage};

/// Inject a NetworkPartition fault, then assert that send returns a
/// fault-specific error (ConnectionLost or MessageDropped, not NotImplemented).
#[test]
#[ignore = "3I.8: inject_fault is a no-op stub, send always returns NotImplemented"]
fn ipc_packet_loss() {
    let mut sim = SimulatorRuntime::new(42);
    sim.inject_fault(SimFault::NetworkPartition {
        duration: Duration::from_millis(500),
    });

    let mut gw = IpcGateway::new(1);
    let result = gw.send(IpcMessage::ToolResult {
        data: vec![1, 2, 3],
    });

    assert!(result.is_err());
    let err = result.unwrap_err();
    // After 3I.8: err should be MessageDropped or ConnectionLost, not NotImplemented.
    assert!(
        err == IpcError::MessageDropped || err == IpcError::ConnectionLost,
        "expected fault-specific error, got: {err}",
    );
}

/// Inject asymmetric fault: one direction blocked, the other open.
/// Forward send should fail; a separate gateway (reverse direction) should succeed.
/// NOTE: SimFault::NetworkPartition currently has no directional fields (from_node/to_node).
/// 3I.8 must extend SimFault with directional routing before this test can pass.
#[test]
#[ignore = "3I.8: inject_fault is a no-op stub, asymmetric faults not modeled yet"]
fn ipc_asymmetric() {
    let mut sim = SimulatorRuntime::new(99);
    // Partition node 1 -> node 2 direction only.
    // Requires 3I.8 to add from_node/to_node fields to SimFault::NetworkPartition.
    sim.inject_fault(SimFault::NetworkPartition {
        duration: Duration::from_secs(1),
    });

    let mut forward = IpcGateway::new(1);
    let mut reverse = IpcGateway::new(2);

    let fwd_result = forward.send(IpcMessage::AgentReady {
        role: AgentRole::Executor,
    });
    let rev_result = reverse.send(IpcMessage::AgentReady {
        role: AgentRole::Auditor,
    });

    // After 3I.8: forward fails (partitioned direction), reverse succeeds.
    assert!(fwd_result.is_err());
    assert!(
        fwd_result.unwrap_err() == IpcError::ConnectionLost,
        "forward direction should be blocked",
    );
    assert!(rev_result.is_ok(), "reverse direction should be open");
}

/// Inject AgentCrash fault, then assert the gateway detects the dead node
/// and SSB row count remains 0 (no orphaned rows from the crash).
#[test]
#[ignore = "3I.8: inject_fault is a no-op stub, crash detection not implemented"]
fn ipc_agent_crash() {
    let mut sim = SimulatorRuntime::new(7);
    sim.inject_fault(SimFault::AgentCrash {
        role: AgentRole::Researcher,
    });

    let mut gw = IpcGateway::new(1);
    let result = gw.receive();

    // After 3I.8: receive should return Err(NodeDead) when the remote agent is crashed.
    assert!(result.is_err());
    assert!(
        result.unwrap_err() == IpcError::NodeDead,
        "should detect dead remote node",
    );

    // SSB should have 0 orphaned rows from the crash.
    let snap = sim.snapshot_state();
    assert_eq!(snap.ssb_row_count, 0, "crash must not leave orphaned SSB rows");
}

/// Inject 5 consecutive NetworkPartition faults (burst), then clear.
/// The 6th send should succeed, verifying the gateway recovers after faults expire.
#[test]
#[ignore = "3I.8: inject_fault is a no-op stub, burst recovery not implemented"]
fn ipc_burst_loss() {
    let mut sim = SimulatorRuntime::new(256);

    // Inject 5 short-lived partitions (burst).
    for _ in 0..5 {
        sim.inject_fault(SimFault::NetworkPartition {
            duration: Duration::from_millis(10),
        });
    }

    let mut gw = IpcGateway::new(1);

    // During burst: sends should fail.
    let burst_result = gw.send(IpcMessage::ToolResult { data: vec![0] });
    assert!(burst_result.is_err(), "send during burst should fail");

    // Advance time past all fault durations to clear the burst.
    sim.advance_time(Duration::from_millis(100));

    // After burst: send should succeed (recovery).
    let recovery_result = gw.send(IpcMessage::AgentReady {
        role: AgentRole::Executor,
    });
    assert!(
        recovery_result.is_ok(),
        "send after burst clearance should succeed, got: {:?}",
        recovery_result,
    );
}
