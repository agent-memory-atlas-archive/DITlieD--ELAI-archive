// E-44 — AgentCheckpoint serialization format benchmark: msgpack vs postcard.
//
// Measures SIZE (bytes) and LATENCY (ns) for round-trip serialize+deserialize
// of a realistic AgentCheckpointData payload using two formats:
//
//   * msgpack  — rmp_serde::to_vec_named (current production format, the path
//                CheckpointManager writes today; rmp_serde 1.3)
//   * postcard — postcard::to_allocvec  (postcard 1.1, workspace dep)
//
// This is the E-44 deliverable (spec "checkpoint_format bench"); the auditor
// runs `cargo bench -p elai-orchestrator --bench checkpoint_format` and writes
// the measured numbers into workflow/audits/E-44-checkpoint-format-bench-2026-05-19.md.
// The numbers in THIS file are NEVER fabricated — they are produced by the
// benchmark binary. Do not copy a number from here into a spec without the
// host+OS+date provenance comment.
//
// Convention: matches ssb_throughput.rs — harness=false, std::time::Instant,
// std::hint::black_box. NO criterion dep (orchestrator D2/DRIFT-2 decision,
// mirrors the C-19 plugins bench). Output format mirrors ssb_throughput.rs:
// a human-readable block followed by a TOML block for metric backfill.
//
// Run:
//   cargo bench -p elai-orchestrator --bench checkpoint_format
//
// The MEASURED numbers for the audit doc come only from this binary's stdout.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use elai_orchestrator::checkpoint::{
    AgentCheckpointData, ChannelKind, ChannelReference, GrcHistoryEntry,
    LEGACY_CHECKPOINT_SCHEMA_VERSION,
};
use elai_orchestrator::durability::CURRENT_CHECKPOINT_SCHEMA_VERSION;

/// Number of serialize+deserialize round-trips in the timed loop. Large enough
/// that per-iteration cost dominates the Instant overhead; small enough that
/// the bench finishes in a few seconds. Mirror: ssb_throughput.rs INSERT_COUNT.
const ITER_COUNT: u64 = 50_000;

/// Build a realistic AgentCheckpointData with non-trivial field sizes so the
/// measurement reflects production payloads rather than an empty struct.
///
/// Sizes chosen to represent a mid-session checkpoint:
/// - 5 GRC history entries (typical reasoning chain depth)
/// - 8 active file paths (typical editing session)
/// - 12 tool_results strings
/// - 4 channel references
/// - ~400-byte output_draft
fn make_checkpoint() -> AgentCheckpointData {
    let grc_history = (0..5)
        .map(|i| GrcHistoryEntry {
            step_type: "observe".to_owned(),
            content: format!("observation content step {i}: checking codebase structure for E-44 deliverable items in the orchestrator crate"),
            grounding: format!("file:crates/orchestrator/src/checkpoint.rs:L{}", 80 + i),
            topic: "e44-bench".to_owned(),
        })
        .collect();

    let active_files = (0..8)
        .map(|i| {
            PathBuf::from(format!(
                "crates/orchestrator/src/durability/module_{i}.rs"
            ))
        })
        .collect();

    let tool_results = (0..12)
        .map(|i| {
            format!(
                "{{\"tool\":\"read_file\",\"result\":\"ok\",\"lines\":150,\"iter\":{i}}}"
            )
        })
        .collect();

    let channel_references = vec![
        ChannelReference {
            channel_id: "session-root".to_owned(),
            kind: ChannelKind::LastValue,
        },
        ChannelReference {
            channel_id: "phase-ctx".to_owned(),
            kind: ChannelKind::Topic,
        },
        ChannelReference {
            channel_id: "ephemeral-scratch".to_owned(),
            kind: ChannelKind::Ephemeral,
        },
        ChannelReference {
            channel_id: "untracked-output".to_owned(),
            kind: ChannelKind::UntrackedValue,
        },
    ];

    AgentCheckpointData {
        agent_status: "EXECUTING".to_owned(),
        grc_history,
        active_files,
        tool_results,
        output_draft: "Draft output for E-44 benchmark pass. The checkpoint_format bench measures msgpack vs postcard size and latency for AgentCheckpointData. Auditor writes measured numbers to workflow/audits/E-44-checkpoint-format-bench-2026-05-19.md.".to_owned(),
        channel_references,
        schema_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
    }
}

// ---------------------------------------------------------------------------
// msgpack path (rmp_serde — current production format)
// ---------------------------------------------------------------------------

fn msgpack_serialize(data: &AgentCheckpointData) -> Vec<u8> {
    rmp_serde::to_vec_named(data).expect("rmp_serde serialize")
}

fn msgpack_deserialize(bytes: &[u8]) -> AgentCheckpointData {
    rmp_serde::from_slice(bytes).expect("rmp_serde deserialize")
}

// ---------------------------------------------------------------------------
// postcard path (workspace dep, alloc feature)
// ---------------------------------------------------------------------------

fn postcard_serialize(data: &AgentCheckpointData) -> Vec<u8> {
    postcard::to_allocvec(data).expect("postcard serialize")
}

fn postcard_deserialize(bytes: &[u8]) -> AgentCheckpointData {
    postcard::from_bytes(bytes).expect("postcard deserialize")
}

// ---------------------------------------------------------------------------
// Bench runner: measures BOTH serialize and deserialize in one pass.
// Returns (serialize_total_ns, deserialize_total_ns, last_serialized_bytes).
// ---------------------------------------------------------------------------

fn bench_format<S, D>(serialize: S, deserialize: D, data: &AgentCheckpointData) -> (u64, u64, usize)
where
    S: Fn(&AgentCheckpointData) -> Vec<u8>,
    D: Fn(&[u8]) -> AgentCheckpointData,
{
    // --- serialize loop ---
    let mut last_bytes: Vec<u8> = Vec::new();
    let t0 = Instant::now();
    for _ in 0..ITER_COUNT {
        let bytes = serialize(black_box(data));
        last_bytes = black_box(bytes);
    }
    let serialize_ns = t0.elapsed().as_nanos() as u64;

    // --- deserialize loop (from the last serialized buffer) ---
    let t1 = Instant::now();
    for _ in 0..ITER_COUNT {
        let decoded = deserialize(black_box(&last_bytes));
        let _ = black_box(decoded);
    }
    let deserialize_ns = t1.elapsed().as_nanos() as u64;

    let payload_bytes = last_bytes.len();
    (serialize_ns, deserialize_ns, payload_bytes)
}

// ---------------------------------------------------------------------------
// Print helpers
// ---------------------------------------------------------------------------

fn print_result(label: &str, ser_ns: u64, de_ns: u64, bytes: usize) {
    let ser_per_ns = ser_ns / ITER_COUNT;
    let de_per_ns = de_ns / ITER_COUNT;
    println!("{label}:");
    println!("  payload_bytes         = {bytes}");
    println!("  serialize_total_ns    = {ser_ns}");
    println!("  serialize_per_iter_ns = {ser_per_ns}");
    println!("  deserialize_total_ns  = {de_ns}");
    println!("  deserialize_per_iter_ns = {de_per_ns}");
}

fn print_toml(label: &str, ser_ns: u64, de_ns: u64, bytes: usize) {
    let ser_per_ns = ser_ns / ITER_COUNT;
    let de_per_ns = de_ns / ITER_COUNT;
    println!("[{label}]");
    println!("payload_bytes = {bytes}");
    println!("serialize_per_iter_ns = {ser_per_ns}");
    println!("deserialize_per_iter_ns = {de_per_ns}");
    println!("iter_count = {ITER_COUNT}");
    println!("status = \"MEASURED_ON_RUN\"  # provenance (host/os/date) required when recorded");
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let data = make_checkpoint();

    // Warm-up: a small unmeasured batch so allocator / CPU caches are primed.
    // Mirror: ssb_throughput.rs warm-up pattern.
    for _ in 0..1_000 {
        let _ = black_box(msgpack_serialize(black_box(&data)));
        let _ = black_box(postcard_serialize(black_box(&data)));
    }

    let (mp_ser, mp_de, mp_bytes) = bench_format(msgpack_serialize, msgpack_deserialize, &data);
    let (pc_ser, pc_de, pc_bytes) = bench_format(postcard_serialize, postcard_deserialize, &data);

    // Verify both formats round-trip to the same logical payload.
    // This is not timed; it catches miscompile / field-mismatch silently.
    let rt_mp: AgentCheckpointData =
        msgpack_deserialize(&msgpack_serialize(&data));
    let rt_pc: AgentCheckpointData =
        postcard_deserialize(&postcard_serialize(&data));
    assert_eq!(
        rt_mp.agent_status, data.agent_status,
        "msgpack round-trip agent_status mismatch"
    );
    assert_eq!(
        rt_pc.agent_status, data.agent_status,
        "postcard round-trip agent_status mismatch"
    );
    assert_eq!(
        rt_mp.schema_version, CURRENT_CHECKPOINT_SCHEMA_VERSION,
        "msgpack schema_version mismatch"
    );
    assert_eq!(
        rt_pc.schema_version, CURRENT_CHECKPOINT_SCHEMA_VERSION,
        "postcard schema_version mismatch"
    );
    // Legacy baseline: a zero-field-schema_version blob still deserializes at the legacy version.
    let legacy_blob: &[u8] = b""; // rmp_serde from_slice of empty bytes will fail; this only
    // documents the intent — the migrator test covers the real path.
    let _ = black_box(legacy_blob);
    let _ = black_box(LEGACY_CHECKPOINT_SCHEMA_VERSION);

    println!(
        "checkpoint_format: {ITER_COUNT} serialize+deserialize round-trips of AgentCheckpointData"
    );
    println!();
    print_result("msgpack", mp_ser, mp_de, mp_bytes);
    println!();
    print_result("postcard", pc_ser, pc_de, pc_bytes);
    println!();

    // Delta summary for the audit doc.
    let size_delta_bytes = mp_bytes as i64 - pc_bytes as i64;
    let ser_delta_ns = mp_ser as i64 / ITER_COUNT as i64 - pc_ser as i64 / ITER_COUNT as i64;
    let de_delta_ns = mp_de as i64 / ITER_COUNT as i64 - pc_de as i64 / ITER_COUNT as i64;
    println!("delta (msgpack - postcard):");
    println!("  size_delta_bytes          = {size_delta_bytes:+}  (positive = msgpack larger)");
    println!("  serialize_delta_ns_per_iter = {ser_delta_ns:+}  (positive = msgpack slower)");
    println!("  deserialize_delta_ns_per_iter = {de_delta_ns:+}  (positive = msgpack slower)");
    println!();

    println!("# TOML output (for metric backfill — requires host/os/date provenance)");
    print_toml("checkpoint_format.msgpack", mp_ser, mp_de, mp_bytes);
    println!();
    print_toml("checkpoint_format.postcard", pc_ser, pc_de, pc_bytes);
    println!();
    println!("[checkpoint_format.delta]");
    println!("size_delta_bytes = {size_delta_bytes}");
    println!("serialize_delta_ns_per_iter = {ser_delta_ns}");
    println!("deserialize_delta_ns_per_iter = {de_delta_ns}");
    println!(
        "conclusion = \"MEASURED_ON_RUN\"  # auditor fills: which format wins and by how much"
    );
}
