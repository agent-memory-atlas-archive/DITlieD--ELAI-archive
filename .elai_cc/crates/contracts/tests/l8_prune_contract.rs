use std::collections::HashSet;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::SeededRng;
use elai_orchestrator::agent_pool::{AgentPoolConfig, InProcessAgentPool, ModelPool};
use elai_orchestrator::ipc_accept::test_support::ScriptedAgentRpcServer;
use elai_orchestrator::ipc_gateway::IpcGateway;
use elai_orchestrator::pipeline_tree_navigator::BabyStepOutcome;
use elai_orchestrator::sandbox::SelfModificationLocks;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
use elai_orchestrator::task_runner::{AgentPoolBabyStepExecutor, BabyStepExecutionCtx, BabyStepExecutor};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;

use elai_orchestrator::claim::extract::{ByteSpan, ClaimKind, ClaimRef, ClaimSet, ExtractedClaim, UnresolvedResolver};
use elai_orchestrator::claim::prune::{content_key, detect_contradicted};

fn missing(span: ByteSpan) -> ExtractedClaim {
    ExtractedClaim {
        kind: ClaimKind::File,
        span,
        refs: vec![ClaimRef { raw: "l8_missing_path.rs".to_owned(), resolved: false }],
    }
}

#[test]
fn repeated_unresolved_claim_is_contradicted_but_first_pass_is_not() {
    let claim = missing(ByteSpan { start: 1, end: 3 });
    let set = ClaimSet::from_claims(vec![claim.clone()]);
    let resolver = UnresolvedResolver;
    assert!(detect_contradicted(&HashSet::new(), &set, &resolver).is_empty());
    let mut prior = HashSet::new();
    prior.insert(content_key(&claim));
    let result = detect_contradicted(&prior, &set, &resolver);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].kind, ClaimKind::File);
}

#[test]
fn content_key_is_span_independent() {
    let first = missing(ByteSpan { start: 1, end: 3 });
    let second = missing(ByteSpan { start: 101, end: 103 });
    assert_eq!(content_key(&first), content_key(&second));
}


struct L8InfiniteVramPool;
impl ModelPool for L8InfiniteVramPool {
    fn available_vram(&self) -> u64 { u64::MAX }
}

fn l8_pool() -> Arc<AsyncMutex<InProcessAgentPool<SeededRng>>> {
    Arc::new(AsyncMutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(), SeededRng::new(0x1_8C1A_15EED), Arc::new(L8InfiniteVramPool),
    )))
}

async fn l8_write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &elai_ipc_types::IpcMessageType) {
    let frame = elai_ipc_types::IpcFrame::encode(msg).expect("encode");
    writer.write_all(&frame.to_wire_bytes().expect("to_wire_bytes")).await.expect("write_all");
    writer.flush().await.expect("flush");
}

async fn l8_read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> elai_ipc_types::IpcMessageType {
    let mut header = [0u8; elai_ipc_types::FRAME_HEADER_SIZE];
    reader.read_exact(&mut header).await.expect("read header");
    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    let mut payload = vec![0u8; len];
    if len > 0 { reader.read_exact(&mut payload).await.expect("read payload"); }
    elai_ipc_types::IpcFrame::decode(&payload).expect("decode frame")
}

async fn l8_pass(
    executor: &AgentPoolBabyStepExecutor<SeededRng>,
    path_rx: tokio::sync::oneshot::Receiver<std::path::PathBuf>,
    content: &'static str,
    id: u64,
) -> BabyStepOutcome {
    let stub = tokio::spawn(async move {
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
        let _request = l8_read_frame(&mut stream).await;
        l8_write_frame(&mut stream, &elai_ipc_types::IpcMessageType::AgentResponse { content: content.to_owned(), artifacts: Vec::new() }).await;
    });
    let result = executor.execute(BabyStepExecutionCtx {
        baby_step_id: id, task_summary: "l8 prune contract seam".into(), escalation_message: None,
        mode_selection: None, session_credentials: std::collections::BTreeMap::new(),
    }).await;
    stub.await.expect("stub agent task panicked");
    result.outcome
}

fn l8_executor(
    tmp: &tempfile::TempDir,
    ssb: Arc<Ssb>,
    path_tx: tokio::sync::oneshot::Sender<std::path::PathBuf>,
) -> AgentPoolBabyStepExecutor<SeededRng> {
    let socket_dir = tmp.path().join("ipc");
    std::fs::create_dir_all(&socket_dir).expect("create socket_dir");
    let gateway = Arc::new(AsyncMutex::new(IpcGateway::with_os_clock(socket_dir.clone(), SelfModificationLocks::new())));
    let rpc = Arc::new(ScriptedAgentRpcServer::new("l8-stub-infer-response"));
    AgentPoolBabyStepExecutor::new(l8_pool(), tmp.path().join("fake-agent-l8.exe"), socket_dir, gateway,
        rpc as Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static>)
        .with_timeout_ms(10_000).with_socket_path_notifier(path_tx).with_ssb_writer(ssb)
}

fn l8_ssb(tmp: &tempfile::TempDir) -> Arc<Ssb> {
    Arc::new(Ssb::new(SsbConfig { db_path: tmp.path().join("l8.db"), pool_size: Some(2),
        project_id: "l8-prune-contract".into(), session_id: "l8-prune-row".into() },
        Arc::new(OsClock)).expect("ssb open"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l8_real_db_quarantines_repeated_missing_claim_and_keeps_risk() {
    let tmp = tempfile::tempdir().expect("tempdir"); let ssb = l8_ssb(&tmp);
    let (tx1, rx1) = tokio::sync::oneshot::channel(); let executor = l8_executor(&tmp, ssb.clone(), tx1);
    let content = "The change fixes `src/l8_missing_claim.rs`.";
    assert!(matches!(l8_pass(&executor, rx1, content, 1).await, BabyStepOutcome::Passed));
    assert!(ssb.query_by_type(SsbActionType::StalenessQuarantineBroadcast).expect("query").is_empty());
    let (tx2, rx2) = tokio::sync::oneshot::channel(); let executor = executor.with_socket_path_notifier(tx2);
    assert!(matches!(l8_pass(&executor, rx2, content, 2).await, BabyStepOutcome::Passed));
    let rows = ssb.query_by_type(SsbActionType::StalenessQuarantineBroadcast).expect("query quarantine");
    assert_eq!(rows.len(), 1);
    let payload = rmp_serde::from_slice::<SsbPayload>(rows[0].payload.as_ref()).expect("decode quarantine payload");
    let SsbPayload::StalenessQuarantineBroadcast(payload) = payload else { panic!("wrong payload variant") };
    assert!(payload.fact_id.starts_with("claim:")); assert_eq!(payload.staleness_ratio, 1.0);
    assert_eq!(payload.dependents_demoted, 0); assert_eq!(payload.cosmos_artifacts_flagged, 0);
    assert_eq!(payload.dropped_count, 0); assert!(!payload.broadcast_id.is_empty());
    assert!(!ssb.query_by_type(SsbActionType::ClaimRiskScored).expect("query risk").is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn l8_real_db_existing_claim_is_never_quarantined() {
    let tmp = tempfile::tempdir().expect("tempdir"); let ssb = l8_ssb(&tmp);
    let (tx1, rx1) = tokio::sync::oneshot::channel(); let executor = l8_executor(&tmp, ssb.clone(), tx1);
    let content = "The change fixes `tests/l8_prune_contract.rs`.";
    assert!(matches!(l8_pass(&executor, rx1, content, 1).await, BabyStepOutcome::Passed));
    let (tx2, rx2) = tokio::sync::oneshot::channel(); let executor = executor.with_socket_path_notifier(tx2);
    assert!(matches!(l8_pass(&executor, rx2, content, 2).await, BabyStepOutcome::Passed));
    let risk_rows = ssb.query_by_type(SsbActionType::ClaimRiskScored).expect("query risk");
    let risk = rmp_serde::from_slice::<SsbPayload>(risk_rows.last().expect("risk row").payload.as_ref())
        .expect("decode risk payload");
    let SsbPayload::ClaimRiskScored(risk) = risk else { panic!("wrong payload variant") };
    assert!(risk.claim_count >= 1);
    assert_eq!(risk.unresolved_ref_count, 0);
    assert!(ssb.query_by_type(SsbActionType::StalenessQuarantineBroadcast).expect("query quarantine").is_empty());
}
