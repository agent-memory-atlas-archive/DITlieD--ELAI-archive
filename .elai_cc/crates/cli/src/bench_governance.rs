//! I-33 -- `elai bench governance-probe` CLI driver.
//!
//! Deterministic fixture harness for the mechanical decision governance
//! primitives (CEFL / I6Q / commit-reveal). Exercises all three
//! `GovernancePrimitiveFired` Kind arms without invoking a live LLM.
//!
//! # What runs
//!
//! Three fixture arms emitting one `GovernancePrimitiveFired` SSB row each:
//!
//! 1. `framing-inversion` -- CeflRequest with 2 candidates; model-invented
//!    option rejected by schema (detected as not in enum); emits
//!    `CeflExternalized` row.
//! 2. `vacuous-rationale` -- I6qValidator rejects a sub-threshold rationale;
//!    emits `I6qRejected` row.
//! 3. `out-of-order-commit` -- CommitRevealRecord::verify_order detects a
//!    reversed row ID pair; emits `CommitRevealSealed` row with the mismatch
//!    recorded in details_json.
//!
//! # Flags
//!
//!   --seed <u64>   Seed for CommitRevealRecord (default 42).
//!
//! # Wiring
//!
//! producer  : `emit_governance_fired` (decision_governance.rs) -- the real
//!             governance emitter called from this fixture.
//! consumer  : action_type_lock golden + i33_governance_contract.
//! ssb rows  : `SsbActionType::GovernancePrimitiveFired` x 3 (one per arm).
//! off-author: decision_governance.rs emits; this fixture only drives it.
//!
//! Cross-lang pair: crates/dashboard-v2/src-tauri/src/commands_i33.rs
//!                  (cross-lang-skip: CLI bench is not a dashboard provider)

use std::sync::Arc;

use anyhow::Context;
use elai_core::clock::{ClockProvider, SeededClock};
use elai_orchestrator::decision_governance::{
    CeflRequest, CommitRevealRecord, I6qRejection, I6qValidator, emit_governance_fired,
};
use elai_orchestrator::ssb::{
    GovernancePrimitiveFiredKind, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    decode_ssb_payload,
};
use tracing::info;

// ---------------------------------------------------------------------------
// Temp-dir helper
// ---------------------------------------------------------------------------

fn make_temp_dir(prefix: &str) -> anyhow::Result<std::path::PathBuf> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("{prefix}_{pid}_{ts}"));
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("could not create temp dir {}", dir.display()))?;
    Ok(dir)
}

// ---------------------------------------------------------------------------
// CLI entry
// ---------------------------------------------------------------------------

/// CLI entry -- builds a tokio runtime, runs the fixture harness, exits 0 on
/// a clean run or 1 on any infrastructure failure.
pub(crate) fn run(seed: u64) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("tokio runtime build failed: {e}"))?;

    let outcome = rt.block_on(async move { run_async(seed).await });

    match outcome {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("GOVERNANCE PROBE ABORT: {e}");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Async core
// ---------------------------------------------------------------------------

async fn run_async(seed: u64) -> anyhow::Result<()> {
    let tmp_dir = make_temp_dir("bench_governance")?;
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: tmp_dir.join("bench_governance.sqlite"),
                pool_size: Some(2),
                project_id: "bench-governance".into(),
                session_id: "s-bench-governance".into(),
            },
            Arc::clone(&clock),
        )
        .context("could not initialise SSB for bench governance")?,
    );

    let mut rows_emitted: Vec<GovernancePrimitiveFiredKind> = Vec::new();

    // -------------------------------------------------------------------------
    // Arm 1: framing-inversion
    // CeflRequest with two candidates; the CEFL framing is applied and a
    // CeflExternalized row is emitted to prove the schema is in flight.
    // -------------------------------------------------------------------------
    info!("governance-probe: arm 1 (framing-inversion)");
    let cefl = CeflRequest::new(vec!["approve".to_owned(), "reject".to_owned()])
        .expect("non-empty candidates");
    let augmented = cefl.apply("Should the agent proceed?");
    // Verify the framing was injected.
    anyhow::ensure!(
        augmented.contains("approve") && augmented.contains("reject"),
        "CEFL framing missing expected candidates in augmented prompt"
    );
    // Verify invented option "maybe" is not in the schema.
    let schema = cefl.schema_json();
    anyhow::ensure!(
        !schema.contains("maybe"),
        "schema should not contain invented option 'maybe'"
    );
    let details = format!(
        r#"{{"candidate_count":2,"arm":"framing-inversion","seed":{seed}}}"#
    );
    emit_governance_fired(
        Some(&ssb),
        GovernancePrimitiveFiredKind::CeflExternalized,
        Some("s-bench-governance"),
        &details,
        &clock,
    )
    .await
    .context("arm 1: emit CeflExternalized")?;
    rows_emitted.push(GovernancePrimitiveFiredKind::CeflExternalized);
    info!("governance-probe: arm 1 passed -- CeflExternalized row emitted");

    // -------------------------------------------------------------------------
    // Arm 2: vacuous-rationale
    // I6qValidator rejects a sub-threshold rationale; I6qRejected row emitted.
    // -------------------------------------------------------------------------
    info!("governance-probe: arm 2 (vacuous-rationale)");
    let i6q = I6qValidator::default();
    let vacuous = "yes yes yes yes yes yes yes yes yes yes yes";
    let rejection = i6q
        .validate(vacuous)
        .expect_err("short vacuous rationale should be rejected");
    anyhow::ensure!(
        matches!(rejection, I6qRejection::TooVacuous { .. }),
        "expected TooVacuous, got: {rejection:?}"
    );
    let details = format!(
        r#"{{"rejection":"{msg}","arm":"vacuous-rationale","seed":{seed}}}"#,
        msg = rejection.to_string().replace('"', "\\\""),
    );
    emit_governance_fired(
        Some(&ssb),
        GovernancePrimitiveFiredKind::I6qRejected,
        Some("s-bench-governance"),
        &details,
        &clock,
    )
    .await
    .context("arm 2: emit I6qRejected")?;
    rows_emitted.push(GovernancePrimitiveFiredKind::I6qRejected);
    info!("governance-probe: arm 2 passed -- I6qRejected row emitted");

    // -------------------------------------------------------------------------
    // Arm 3: out-of-order-commit
    // CommitRevealRecord::verify_order detects reversed row IDs; emits
    // CommitRevealSealed row with the out-of-order signal in details_json.
    // -------------------------------------------------------------------------
    info!("governance-probe: arm 3 (out-of-order-commit)");
    let record = CommitRevealRecord {
        seed,
        choice_index: 1,
    };
    let hash = record.commit_hash();
    let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    // Simulate reversed IDs: reveal wrote before commit.
    let commit_row_id: i64 = 50;
    let reveal_row_id: i64 = 30; // < commit_row_id -> out-of-order
    let order_err = CommitRevealRecord::verify_order(commit_row_id, reveal_row_id)
        .expect_err("reversed IDs should fail verify_order");
    let details = format!(
        r#"{{"commit_row_id":{commit_row_id},"reveal_row_id":{reveal_row_id},"verdict":"out_of_order","commit_hash_prefix":"{prefix}","arm":"out-of-order-commit","seed":{seed}}}"#,
        commit_row_id = commit_row_id,
        reveal_row_id = reveal_row_id,
        prefix = &hash_hex[..16],
        seed = seed,
    );
    emit_governance_fired(
        Some(&ssb),
        GovernancePrimitiveFiredKind::CommitRevealSealed,
        Some("s-bench-governance"),
        &details,
        &clock,
    )
    .await
    .context("arm 3: emit CommitRevealSealed")?;
    rows_emitted.push(GovernancePrimitiveFiredKind::CommitRevealSealed);
    info!(
        "governance-probe: arm 3 passed -- CommitRevealSealed row emitted (order_err: {order_err})"
    );

    // -------------------------------------------------------------------------
    // Verify all 3 rows landed in SSB (off-author artifact, R4)
    // query_by_type matches on the wire tag (governance_primitive_fired) then we
    // kind-discriminate by decoding each payload.
    // -------------------------------------------------------------------------
    let raw_rows = ssb
        .query_by_type(SsbActionType::GovernancePrimitiveFired(
            GovernancePrimitiveFiredKind::CeflExternalized,
        ))
        .context("SSB read: GovernancePrimitiveFired rows")?;
    anyhow::ensure!(
        raw_rows.len() == 3,
        "expected 3 GovernancePrimitiveFired rows, found {}",
        raw_rows.len()
    );

    // Kind-discriminate via rmp_serde decode.
    let mut cefl_count = 0usize;
    let mut i6q_count = 0usize;
    let mut seal_count = 0usize;
    for row in &raw_rows {
        let payload = decode_ssb_payload(&row.payload)
            .context("decode GovernancePrimitiveFired payload")?;
        if let SsbPayload::GovernancePrimitiveFired(p) = payload {
            match p.kind {
                GovernancePrimitiveFiredKind::CeflExternalized => cefl_count += 1,
                GovernancePrimitiveFiredKind::I6qRejected => i6q_count += 1,
                GovernancePrimitiveFiredKind::CommitRevealSealed => seal_count += 1,
            }
        }
    }
    anyhow::ensure!(cefl_count == 1, "expected 1 CeflExternalized row, got {cefl_count}");
    anyhow::ensure!(i6q_count == 1, "expected 1 I6qRejected row, got {i6q_count}");
    anyhow::ensure!(seal_count == 1, "expected 1 CommitRevealSealed row, got {seal_count}");

    info!(
        "governance-probe: all 3 arms passed -- {} GovernancePrimitiveFired rows verified \
         (cefl={cefl_count}, i6q={i6q_count}, seal={seal_count})",
        raw_rows.len()
    );
    Ok(())
}
