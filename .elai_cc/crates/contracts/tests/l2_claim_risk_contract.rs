use std::sync::Arc;
use elai_orchestrator::claim::extract::{ByteSpan, ClaimKind, ClaimRef, ClaimSet, ExtractedClaim};
use elai_orchestrator::claim::risk::{
    expensive_predictors_for, score_claim_risk, score_claim_risk_with, ClaimRisk, PredictorKind,
    PredictorSignal, PredictorSource, RiskTier,
};

fn set(resolved: bool) -> ClaimSet {
    ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 3, end: 9 },
        refs: vec![ClaimRef { raw: "src/lib.rs".into(), resolved }],
    }])
}

struct Signals(Vec<PredictorSignal>);
impl PredictorSource for Signals {
    fn signals(&self, _claim: &ExtractedClaim) -> Vec<PredictorSignal> { self.0.clone() }
}

#[test]
fn l2_risk_converges_fired_predictors() {
    let source = Signals(vec![
        PredictorSignal { source: PredictorKind::RoundTrip, value: 0.7, fired: true },
        PredictorSignal { source: PredictorKind::Oscillation, value: 0.7, fired: true },
    ]);
    let risks = score_claim_risk_with(&set(true), &[&source]);
    assert!(matches!(risks[0].tier(), RiskTier::High | RiskTier::Critical));
}

#[test]
fn l2_risk_finite_tier_with_extreme_signal() {
    let source = Signals(vec![
        PredictorSignal { source: PredictorKind::RoundTrip, value: f64::MAX, fired: true },
    ]);
    assert_eq!(score_claim_risk_with(&set(true), &[&source])[0].tier(), RiskTier::Medium);
}

#[test]
fn l2_valid_tier_without_logprob_signal() {
    let risks = score_claim_risk(&set(true));
    assert_eq!(risks[0].tier(), RiskTier::Low);
    assert!(!risks[0].signals.iter().any(|s| s.source == PredictorKind::LogprobOptional));
}

#[test]
fn l2_counterfactual_signal_controls_causal_support() {
    let not_fired = Signals(vec![PredictorSignal { source: PredictorKind::Counterfactual, value: 1.0, fired: false }]);
    assert!(!score_claim_risk_with(&set(true), &[&not_fired])[0].causal_supported);
    let fired = Signals(vec![PredictorSignal { source: PredictorKind::Counterfactual, value: 1.0, fired: true }]);
    assert!(score_claim_risk_with(&set(true), &[&fired])[0].causal_supported);
}

#[test]
fn l2_low_cost_gate_is_empty() {
    assert!(expensive_predictors_for(RiskTier::Low).is_empty());
}

#[test]
fn l2_recorded_signals_recompute_same_tier() {
    let source = Signals(vec![
        PredictorSignal { source: PredictorKind::RoundTrip, value: 0.7, fired: true },
        PredictorSignal { source: PredictorKind::Oscillation, value: 0.7, fired: true },
    ]);
    let first = score_claim_risk_with(&set(true), &[&source]);
    let recorded = Signals(first[0].signals.clone());
    let second = score_claim_risk_with(&set(true), &[&recorded]);
    assert_eq!(first[0].tier(), second[0].tier());
}

#[test]
fn l2_claim_risk_public_type_is_usable() {
    fn accepts_risk(_: &ClaimRisk) {}
    let risk = score_claim_risk(&set(true)).remove(0);
    accepts_risk(&risk);
    assert_eq!(risk.tier(), RiskTier::Low);
}

use elai_orchestrator::agent_pool::{AgentPoolConfig, InProcessAgentPool, ModelPool};
use elai_orchestrator::ipc_accept::test_support::ScriptedAgentRpcServer;
use elai_orchestrator::ipc_gateway::IpcGateway;
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::sandbox::SelfModificationLocks;
use elai_orchestrator::task_runner::{AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutor};
use elai_orchestrator::ssb::{ClaimRiskScoredKind, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use elai_core::SeededRng;
use elai_core::clock::OsClock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;
use tracing_test::traced_test;

struct L2InfiniteVramPool;
impl ModelPool for L2InfiniteVramPool {
    fn available_vram(&self) -> u64 { u64::MAX }
}

fn l2_make_pool() -> Arc<AsyncMutex<InProcessAgentPool<SeededRng>>> {
    Arc::new(AsyncMutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(), SeededRng::new(0x1_2C1A_15EED), Arc::new(L2InfiniteVramPool),
    )))
}

fn l2_make_gateway(socket_dir: &std::path::Path) -> Arc<AsyncMutex<IpcGateway>> {
    Arc::new(AsyncMutex::new(IpcGateway::with_os_clock(socket_dir.to_path_buf(), SelfModificationLocks::new())))
}

async fn l2_write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &elai_ipc_types::IpcMessageType) {
    let frame = elai_ipc_types::IpcFrame::encode(msg).expect("encode");
    let wire = frame.to_wire_bytes().expect("to_wire_bytes");
    writer.write_all(&wire).await.expect("write_all");
    writer.flush().await.expect("flush");
}

async fn l2_read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> elai_ipc_types::IpcMessageType {
    let mut header = [0u8; elai_ipc_types::FRAME_HEADER_SIZE];
    reader.read_exact(&mut header).await.expect("read header");
    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    let mut payload = vec![0u8; len];
    if len > 0 { reader.read_exact(&mut payload).await.expect("read payload"); }
    elai_ipc_types::IpcFrame::decode(&payload).expect("decode frame")
}

#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l2_executor_claim_risk_logs_per_tier() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_dir = tmp.path().join("ipc");
    std::fs::create_dir_all(&socket_dir).expect("create socket_dir");
    let pool = l2_make_pool();
    let gateway = l2_make_gateway(&socket_dir);
    let rpc_server = Arc::new(ScriptedAgentRpcServer::new("l2-stub-infer-response"));
    let (path_tx, path_rx) = tokio::sync::oneshot::channel::<std::path::PathBuf>();

    let stub_agent = tokio::spawn(async move {
        let socket_path = path_rx.await.expect("path_rx: sender dropped");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        #[cfg(unix)]
        let mut stream = tokio::net::UnixStream::connect(&socket_path).await.expect("stub connect");
        #[cfg(windows)]
        let mut stream = {
            let stem = socket_path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "elai-ipc".to_string());
            let pipe_name = format!(r"\\.\pipe\elai-{stem}");
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
            loop {
                match tokio::net::windows::named_pipe::ClientOptions::new().open(&pipe_name) {
                    Ok(s) => break s,
                    Err(_) if tokio::time::Instant::now() < deadline => tokio::time::sleep(std::time::Duration::from_millis(25)).await,
                    Err(e) => panic!("stub connect: {e}"),
                }
            }
        };
        let _request = l2_read_frame(&mut stream).await;
        l2_write_frame(&mut stream, &elai_ipc_types::IpcMessageType::AgentResponse { content: "The change fixes `src/l2_missing_claim_risk.rs`.".to_string(), artifacts: Vec::new() }).await;
    });

    let executor = AgentPoolBabyStepExecutor::new(pool, tmp.path().join("fake-agent-l2.exe"), socket_dir, gateway, rpc_server as Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>)
        .with_timeout_ms(10_000).with_socket_path_notifier(path_tx);
    let result = executor.execute(BabyStepExecutionCtx { baby_step_id: 1, task_summary: "l2 claim risk seam".into(), escalation_message: None, mode_selection: None, session_credentials: std::collections::BTreeMap::new() }).await;
    assert!(matches!(result.outcome, BabyStepOutcome::Passed), "expected Passed, got: {:?}", result.outcome);
    stub_agent.await.expect("stub agent task panicked");
    assert!(logs_contain("risk_medium"), "production seam must emit risk_medium per-tier field");
}

#[test]
fn l2_cost_gate_consult_set_is_recorded_on_the_risk() {
    let resolved = score_claim_risk(&set(true));
    assert!(resolved[0].consult.is_empty());

    let unresolved = score_claim_risk(&set(false));
    assert!(!unresolved[0].consult.is_empty());
    assert!(unresolved[0].consult.contains(&PredictorKind::RoundTrip));

    let expensive = Signals(vec![
        PredictorSignal { source: PredictorKind::RoundTrip, value: 0.7, fired: true },
        PredictorSignal { source: PredictorKind::Oscillation, value: 0.7, fired: true },
    ]);
    let injected = score_claim_risk_with(&set(true), &[&expensive]);
    assert!(injected[0].consult.is_empty());
}


#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l2_executor_emits_claim_risk_scored_row() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let socket_dir = tmp.path().join("ipc");
    std::fs::create_dir_all(&socket_dir).expect("create socket_dir");
    let pool = l2_make_pool();
    let gateway = l2_make_gateway(&socket_dir);
    let rpc_server = Arc::new(ScriptedAgentRpcServer::new("l2-stub-infer-response"));
    let (path_tx, path_rx) = tokio::sync::oneshot::channel::<std::path::PathBuf>();

    let stub_agent = tokio::spawn(async move {
        let socket_path = path_rx.await.expect("path_rx: sender dropped");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        #[cfg(unix)]
        let mut stream = tokio::net::UnixStream::connect(&socket_path).await.expect("stub connect");
        #[cfg(windows)]
        let mut stream = {
            let stem = socket_path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "elai-ipc".to_string());
            let pipe_name = format!(r"\\.\pipe\elai-{stem}");
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
            loop {
                match tokio::net::windows::named_pipe::ClientOptions::new().open(&pipe_name) {
                    Ok(s) => break s,
                    Err(_) if tokio::time::Instant::now() < deadline => tokio::time::sleep(std::time::Duration::from_millis(25)).await,
                    Err(e) => panic!("stub connect: {e}"),
                }
            }
        };
        let _request = l2_read_frame(&mut stream).await;
        l2_write_frame(&mut stream, &elai_ipc_types::IpcMessageType::AgentResponse {
            content: "The change fixes `src/l2_missing_claim_risk.rs`.".to_string(),
            artifacts: Vec::new(),
        }).await;
    });

    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(SsbConfig {
        db_path: tmp.path().join("l2-claim-risk-ssb.db"),
        pool_size: Some(2),
        project_id: "l2-claim-risk-contract".into(),
        session_id: "l2-claim-risk-row".into(),
    }, clock).expect("ssb open"));
    let executor = AgentPoolBabyStepExecutor::new(
        pool,
        tmp.path().join("fake-agent-l2.exe"),
        socket_dir,
        gateway,
        rpc_server as Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>,
    ).with_timeout_ms(10_000).with_socket_path_notifier(path_tx).with_ssb_writer(ssb.clone());
    let result = executor.execute(BabyStepExecutionCtx {
        baby_step_id: 1,
        task_summary: "l2 claim risk seam".into(),
        escalation_message: None,
        mode_selection: None,
        session_credentials: std::collections::BTreeMap::new(),
    }).await;
    assert!(matches!(result.outcome, BabyStepOutcome::Passed), "expected Passed, got: {:?}", result.outcome);
    stub_agent.await.expect("stub agent task panicked");

    let rows = ssb.query_by_type(SsbActionType::ClaimRiskScored).expect("query claim risk rows");
    assert_eq!(rows.len(), 1);
    let payload = rmp_serde::from_slice::<SsbPayload>(rows[0].payload.as_ref()).expect("decode claim risk payload");
    let SsbPayload::ClaimRiskScored(payload) = payload else { panic!("wrong payload variant") };
    assert_eq!(payload.kind, ClaimRiskScoredKind::MediumRisk);
    assert!(payload.medium_count >= 1);
    assert!(payload.unresolved_ref_count >= 1);
    assert!(!payload.claim_digests.is_empty());
    assert_eq!(payload.claim_digests.len(), payload.claim_count);
}
