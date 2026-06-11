// 3B.bench — SSB single-writer insert throughput micro-benchmark.
//
// Purpose: ground the "50K inserts/sec" SSB performance claim with a real,
// reproducible measured number. The claim is currently BLOCKED_MEASUREMENT —
// this harness EXISTS so the number can be produced, but the number itself is
// only valid once this bench is actually run on a named machine and the result
// recorded with the command + wall-clock + date (never fabricated; CLAUDE.md
// "NEVER fabricate a measured number").
//
// Convention: matches the orchestrator/knowledge/reasoning workspace precedent
// (orchestrator D2 / DRIFT-2; the C-19 plugins bench
// crates/plugins/benches/tiered_analysis_bench.rs): `harness = false` +
// std::hint::black_box + std::time::Instant, NO criterion dependency. The
// FINISH punch list named this a "criterion benchmark"; adding the criterion
// crate to the orchestrator dev-deps would violate the documented D2/DRIFT-2
// decision and CLAUDE.md "no new dependencies without confirmation". The
// no-harness path produces the same median/p99/throughput figures.
//
// What is measured: the END-TO-END durable write path —
// `Ssb::write_payload` -> typed redaction -> rmp_serde::to_vec_named ->
// single-writer mpsc hop -> write_loop INSERT -> oneshot seq confirmation.
// Each iteration `.await`s the seq, so the figure is the latency-bound
// single-producer throughput (the IPC sync contract guarantees durability
// before the caller proceeds — ssb.rs WARN-6). A real production deployment
// has many producers funneling into the one writer; this bench is the
// single-producer floor, which is the conservative (lower-bound) number for
// the 50K/sec claim.
//
// Run:
//   cargo bench -p elai-orchestrator --bench ssb_throughput
// Record the printed `throughput_inserts_per_sec` line (plus host CPU, OS,
// date) into metrics/ when grounding the 50K/sec claim. Until then the claim
// stays BLOCKED_MEASUREMENT.

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use elai_core::clock::{ClockProvider, SeededClock};
use elai_orchestrator::ssb::{FsmTransitionPayload, Ssb, SsbConfig, SsbPayload};

/// Number of rows written in the timed loop. Large enough that the per-row
/// fixed cost dominates over runtime/tempdir setup, small enough that a single
/// `cargo bench` invocation finishes in a few seconds on a dev box.
const INSERT_COUNT: u64 = 20_000;

/// Pool size for the read pool (irrelevant to the write path; kept small so the
/// bench process opens few connections). The write path uses a single dedicated
/// writer connection, not the pool.
const BENCH_POOL_SIZE: u32 = 2;

/// Build a fresh tempdir-backed SSB instance for the bench. Returns the SSB and
/// the TempDir guard (kept alive by the caller so the db file is not reaped
/// mid-run).
fn make_ssb(dir: &tempfile::TempDir) -> Ssb {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_throughput_bench.db"),
        pool_size: Some(BENCH_POOL_SIZE),
        project_id: "bench-project".to_owned(),
        session_id: "bench-session".to_owned(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    Ssb::new(cfg, clock).expect("bench ssb init")
}

/// One distinct payload per iteration so msgpack serialization does real work
/// every time (a constant payload could let the allocator/serializer warm a hot
/// path that does not reflect production variety). FsmTransitionPayload is the
/// smallest typed variant (3 short strings) so the figure isolates the writer
/// fixed cost rather than serialization of a large body.
fn payload_for(i: u64) -> SsbPayload {
    SsbPayload::FsmTransition(FsmTransitionPayload {
        from: "EXECUTING".to_owned(),
        to: "AUDITING".to_owned(),
        reason: format!("bench-iter-{i}"),
    })
}

/// Drive `INSERT_COUNT` durable writes through the single writer, awaiting each
/// seq confirmation, and return the wall-clock elapsed seconds.
async fn run_inserts(ssb: &Ssb) -> f64 {
    let start = Instant::now();
    for i in 0..INSERT_COUNT {
        let seq = ssb
            .write_payload("bench-agent", 0, payload_for(i), Vec::new())
            .await
            .expect("bench write_payload");
        // Touch the returned seq so the optimizer cannot elide the await.
        black_box(seq);
    }
    start.elapsed().as_secs_f64()
}

fn main() {
    // Dedicated multi-thread runtime: Ssb::new spawns the write_loop task, and
    // write_payload awaits a oneshot from it, so a current-thread runtime would
    // be sufficient but multi-thread mirrors the production tokio runtime.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("bench tokio runtime");

    let elapsed_secs = rt.block_on(async {
        let dir = tempfile::TempDir::new().expect("bench tempdir");
        let ssb = make_ssb(&dir);
        // Warm-up: a small unmeasured batch so the first WAL checkpoint / page
        // cache fill does not skew the timed loop.
        for i in 0..1_000u64 {
            let _ = ssb
                .write_payload("bench-agent", 0, payload_for(i), Vec::new())
                .await
                .expect("bench warm-up write");
        }
        let secs = run_inserts(&ssb).await;
        // Hold the dir + ssb alive until here.
        drop(ssb);
        drop(dir);
        secs
    });

    let per_insert_us = (elapsed_secs * 1_000_000.0) / (INSERT_COUNT as f64);
    let throughput = (INSERT_COUNT as f64) / elapsed_secs;

    // Human-readable + a TOML block for metric backfill (mirrors the C-19
    // plugins bench output convention). The number printed here is ONLY valid
    // when this bench was actually executed; do not transcribe it into a spec
    // without the host + date provenance.
    println!("ssb_throughput: {INSERT_COUNT} durable single-producer inserts");
    println!("  elapsed_secs            = {elapsed_secs:.4}");
    println!("  per_insert_us           = {per_insert_us:.3}");
    println!("  throughput_inserts_per_sec = {throughput:.1}");
    println!("# TOML output");
    println!("[ssb_throughput]");
    println!("insert_count = {INSERT_COUNT}");
    println!("elapsed_secs = {elapsed_secs:.4}");
    println!("per_insert_us = {per_insert_us:.3}");
    println!("throughput_inserts_per_sec = {throughput:.1}");
    println!("status = \"MEASURED_ON_RUN\"  # provenance (host/os/date) required when recorded");
}
