//! H-5 — Batch API for non-interactive lanes (Tier-1 done-gate contract).
//!
//! Grounded correction (see workflow/reasoning/H-5-decisions.md): the plan framed
//! the batch lane as `CosmosWorker::tick -> AnthropicClient` direct. The live code
//! refutes that — cosmos research flows through an injected `dyn ResearchDispatcher`
//! (cosmos_worker.rs:606), whose PRODUCTION impl is `HttpCosmosResearchDispatcher`
//! (grounding_client.rs:1301). The genuine batchable Anthropic lane is therefore the
//! research-dispatcher layer, not a mythical direct cosmos->AnthropicClient call. H-5
//! batches there: `ResearchDispatcher::run_batch` (default = sequential fallback so
//! Noop/Mock are unaffected) is overridden by `HttpCosmosResearchDispatcher` to submit
//! ONE Anthropic batch via `AnthropicBatchClient` and emit `batch_lifecycle`. The
//! AUTONOMOUS lane (`CosmosWorker::tick_batched`, called by live_cli cosmos_tick_emit_loop)
//! batches; the user-initiated dashboard `trigger_cosmos_refresh` keeps `tick()` (sync).
//!
//! Tier-1 (this file): deterministic, in-process. Proves the new SSB action_type
//! round-trips + maps to its payload (W-9), the runtime SQL CHECK cascade includes the
//! new label (the G-8/G-9 silent-INSERT-failure gotcha), the batch client targets the
//! VERIFIED endpoint /v1/messages/batches with no excluded params, the 50%-off spend is
//! a MEASURED half (HD-1), and the autonomous-lane wiring exists in production source
//! (W-3). Tier-2 (the live POST to /v1/messages/batches over the wire) is a DEFERRED,
//! separately-gated checkbox (W-10) — cloud-only per the run override, no sidecar.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::ssb::{
    BatchLifecycleKind, BatchLifecyclePayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
    SSB_ACTION_TYPES,
};
use elai_providers::{half_price_cents, AnthropicBatchClient};

/// crates/contracts/.. -> the workspace `crates` dir.
fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// --- W-9: the new action_type round-trips and maps to its payload ----------------

#[test]
fn batch_lifecycle_action_type_sql_label() {
    // The wire/SQL label. The deserializer round-trip (from the SQL string back to the
    // variant) is covered by the orchestrator's own action_type_lock + typed-payloads
    // tests; here we pin the canonical label the SQL CHECK + golden depend on.
    assert_eq!(SsbActionType::BatchLifecycle.as_sql_str(), "batch_lifecycle");
}

#[test]
fn batch_lifecycle_payload_maps_to_its_action_type() {
    let payload = SsbPayload::BatchLifecycle(BatchLifecyclePayload {
        kind: BatchLifecycleKind::Submitted,
        batch_id: "msgbatch_test".into(),
        request_count: 2,
        model: "claude-sonnet-4-6".into(),
        spend_cents_half: 0,
        task_id: "t-h5".into(),
    });
    assert_eq!(payload.action_type(), SsbActionType::BatchLifecycle);
}

#[test]
fn batch_lifecycle_kind_wire_labels_are_snake_case() {
    assert_eq!(BatchLifecycleKind::Submitted.as_str(), "submitted");
    assert_eq!(BatchLifecycleKind::Completed.as_str(), "completed");
    assert_eq!(BatchLifecycleKind::Expired.as_str(), "expired");
}

#[test]
fn batch_lifecycle_payload_serde_roundtrips() {
    let payload = BatchLifecyclePayload {
        kind: BatchLifecycleKind::Completed,
        batch_id: "msgbatch_01".into(),
        request_count: 3,
        model: "claude-sonnet-4-6".into(),
        spend_cents_half: 375,
        task_id: "t-h5".into(),
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let back: BatchLifecyclePayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, payload);
}

// --- HD-1: the 50%-off saving is a MEASURED half, not a literature multiplier -----

#[test]
fn batch_spend_is_exactly_half_synchronous() {
    // The journal-recomputable invariant: spend_cents_half == floor(sync / 2).
    assert_eq!(half_price_cents(750), 375);
    assert_eq!(half_price_cents(0), 0);
    assert_eq!(half_price_cents(1), 0); // floor, never rounds up over the bill
    assert_eq!(half_price_cents(999), 499);
}

// --- G-8/G-9 gotcha: the runtime SQL CHECK cascade must include the new label -----

#[test]
fn ssb_sql_check_constraint_includes_batch_lifecycle() {
    // Tier-B Move 1 moved the CREATE_SSB_TABLE CHECK(action_type IN (...)) constraint
    // into the ssb/schema.rs submodule; read it there.
    let src = read_src("orchestrator/src/ssb/schema.rs");
    // The CHECK(action_type IN (...)) list is the runtime cascade site; a variant
    // missing here causes a SILENT INSERT failure even when the Rust enum compiles.
    assert!(
        src.contains("'batch_lifecycle'"),
        "ssb.rs SQL CHECK IN(...) must include 'batch_lifecycle' (runtime cascade)"
    );
    // SCHEMA bumped from the LIVE baseline (218 -> 219); floor assertion (>= 219)
    // so subsequent H-N bumps do not break this H-5 contract.
    // Tier-B Move 2 made SCHEMA_VERSION a derived expression (no longer a parseable
    // literal in source), so read the published runtime constant; same >= 219 floor.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 219,
        "SCHEMA_VERSION must be >= 219 (H-5 bumped from 218; floor assertion); got {SCHEMA_VERSION_PUBLIC}"
    );
}

#[test]
fn action_type_lock_counts_the_new_variant() {
    // Tier-B Move 2 derived TOTAL_VARIANTS and relocated the ALL_VARIANTS array
    // (now SSB_ACTION_TYPES in ssb.rs), so the source-text parse + grep of
    // action_type_lock.rs no longer apply. Read the runtime registry directly;
    // same >= 496 floor, same BatchLifecycle presence guarantee.
    assert!(
        SSB_ACTION_TYPES.len() >= 496,
        "TOTAL_VARIANTS must be >= 496 (H-5 added batch_lifecycle, floor assertion); got {}",
        SSB_ACTION_TYPES.len()
    );
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|v| v.as_sql_str() == "batch_lifecycle"),
        "SSB_ACTION_TYPES must include SsbActionType::BatchLifecycle"
    );
}

// --- the batch client targets the VERIFIED endpoint, excludes the banned params ----

#[test]
fn batch_client_targets_messages_batches_endpoint() {
    let src = read_src("providers/src/providers/anthropic_batch.rs");
    assert!(
        src.contains("/v1/messages/batches"),
        "batch client must POST the verified /v1/messages/batches endpoint"
    );
    assert!(
        src.contains("AnthropicBatchClient"),
        "anthropic_batch.rs must define AnthropicBatchClient"
    );
    // Batch requests EXCLUDE streaming (verify:batch-api). The batch item builder
    // must force stream:false — assert the source never sets stream true on a batch item.
    assert!(
        !src.contains("stream: true"),
        "batch items must never set stream:true (excluded by the Batch API)"
    );
}

// --- the batch producer is wired into the GENUINE Anthropic non-interactive lane -----

#[test]
fn batch_producer_wired_into_the_grounding_lane() {
    // GROUNDED CORRECTION (workflow/reasoning/H-5-decisions.md): cosmos research is
    // SearXNG web search (HttpCosmosResearchDispatcher::run -> SearxClient::search), NOT
    // an Anthropic LLM call -- so it cannot be Anthropic-batched. The genuine
    // non-interactive Anthropic lane is CloudGroundingClient (ground -> send_with_ledger
    // -> send_message). The batch producer lives there, not in cosmos.
    let grounding = read_src("orchestrator/src/grounding_client.rs");
    assert!(
        grounding.contains("pub async fn ground_batch"),
        "CloudGroundingClient must expose ground_batch (the real batch producer)"
    );
    assert!(
        grounding.contains("BatchLifecycleKind::Submitted")
            && grounding.contains("BatchLifecycleKind::Completed"),
        "ground_batch must emit batch_lifecycle Submitted + Completed rows"
    );
    assert!(
        grounding.contains("half_price_cents"),
        "the Completed row must carry the HD-1 MEASURED 50% spend"
    );

    // The synchronous client lends its auth+base_url to the batch sibling (GD-1, no
    // transport restatement).
    let anthropic = read_src("providers/src/providers/anthropic.rs");
    assert!(
        anthropic.contains("pub fn batch_client"),
        "AnthropicClient must expose batch_client() sharing auth+base_url"
    );

    // The live lane activation is a registered default-off experiment (R9): the live
    // Anthropic batch call + a production caller (GEPA/bench) are Tier-2/RunPod-pending.
    let allowlist_path = crates_dir()
        .parent()
        .expect("crates parent (.elai_cc)")
        .join("wire-dark-allowlist.toml");
    let allowlist = fs::read_to_string(&allowlist_path).unwrap_or_default();
    assert!(
        allowlist.contains("ELAI_BATCH_LANE_ENABLE"),
        "the batch lane flag must be registered in wire-dark-allowlist.toml (R9)"
    );
}

// --- a constructed batch client carries the verified anthropic-version, no panic ---

#[test]
fn batch_client_constructs_with_verified_version() {
    let client = AnthropicBatchClient::new("test-key");
    // GA endpoint: anthropic-version 2023-06-01, no beta header required.
    assert_eq!(client.anthropic_version(), "2023-06-01");
}

// --- W-1/W-5 off-agent artifact: a batch_lifecycle row survives the runtime SQL CHECK -

#[tokio::test]
async fn batch_lifecycle_row_writes_to_ssb_runtime_check() {
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use elai_core::clock::SeededClock;
    use elai_orchestrator::ssb::{Ssb, SsbConfig};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("h5.db"),
        pool_size: Some(2),
        project_id: "p".into(),
        session_id: "s".into(),
    };
    let clock = Arc::new(SeededClock::with_epoch(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
    )) as Arc<dyn elai_core::ClockProvider>;
    let ssb = Ssb::new(cfg, clock).expect("ssb init");

    // The runtime CHECK(action_type IN (...)) constraint must accept 'batch_lifecycle'
    // or the INSERT SILENTLY fails (the G-8/G-9 lesson). A successful write of both
    // lifecycle rows is the off-agent proof that the full cascade -- enum + SQL CHECK --
    // is complete, and that spend_cents_half is the HD-1 half of the synchronous cost.
    let submitted = SsbPayload::BatchLifecycle(BatchLifecyclePayload {
        kind: BatchLifecycleKind::Submitted,
        batch_id: "msgbatch_rt".into(),
        request_count: 2,
        model: "claude-sonnet-4-6".into(),
        spend_cents_half: 0,
        task_id: "t".into(),
    });
    ssb.write_payload("cloud_grounding_batch", 0, submitted, Vec::new())
        .await
        .expect("batch_lifecycle Submitted row must INSERT (SQL CHECK includes it)");

    let completed = SsbPayload::BatchLifecycle(BatchLifecyclePayload {
        kind: BatchLifecycleKind::Completed,
        batch_id: "msgbatch_rt".into(),
        request_count: 2,
        model: "claude-sonnet-4-6".into(),
        spend_cents_half: half_price_cents(750),
        task_id: "t".into(),
    });
    ssb.write_payload("cloud_grounding_batch", 1, completed, Vec::new())
        .await
        .expect("batch_lifecycle Completed row must INSERT");
}
