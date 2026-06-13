// I-14: deterministic context compression -- contract.
//
// Pins the CACHE-INVARIANT ECONOMICS MIDDLEWARE (Track 14) Tier-1 contract:
// the `ContextTransformer` shrinks large tool-result payloads with two
// deterministic strategies (AstFold tree-sitter body elision + LspSlice
// compiler-error slicing), journals every transform as a reversible
// `ContextTransformReceipt` SSB row (ID-3 JOURNALED-MIDDLEWARE), and is wired
// at the production tool-result byte seam (ipc_accept.rs run_agent_session ->
// AgentRpcServer::transform_tool_output, overridden in production_agent_rpc.rs).
//
// ARM coverage:
//   ARM-1  GD-2 determinism: same input bytes -> byte-identical output AND an
//          identical reverse_hash across two independent transform() runs.
//   ARM-2  real-DB-row receipt: drive ContextTransformer::transform() against a
//          REAL TempDir SQLite SSB, then query_by_type(ContextTransformReceipt)
//          and assert COUNT >= 1 + decode the row + verify reverse_hash ==
//          blake3(input) and before/after token counts. NOT a struct-only
//          round-trip (the I-5 / I-13 audits both caught struct-only fail-opens).
//   ARM-3  AstFold correctness: a Rust function payload folds (body elided,
//          signature kept); after_tokens < before_tokens; receipt.kind==AstFold.
//   ARM-4  LspSlice / ToolchainMissing: rust-analyzer is absent on the test lane
//          (lsp_oracle returns invoked:false), so LspSlice degrades to
//          skipped:true and returns the input verbatim -- no panic, no drop.
//   ARM-5  non-UTF-8 AstFold skip: binary tool output returns verbatim + skip.
//   ARM-6  receipt-presence (ID-3 / R1): a transform that ran emits exactly one
//          receipt row; zero rows after a transform is a FAIL (caught here).
//
// SCHEMA_VERSION 250, TOTAL_VARIANTS 559.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use elai_core::clock::SeededClock;
use elai_core::ClockProvider;
use elai_orchestrator::context_transform::{AstFoldTransform, ContextTransformer, LspSliceTransform};
use elai_orchestrator::ssb::{
    ContextTransformKind, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::{Ssb, SsbReader};
use tempfile::TempDir;

/// A small, valid Rust function payload the AstFold path can fold deterministically.
const RUST_FN_PAYLOAD: &[u8] =
    b"fn add(a: i32, b: i32) -> i32 {\n    let sum = a + b;\n    sum\n}\n";

/// Build a real TempDir-backed SSB with a deterministic seeded clock.
/// Returns the SSB handle and the owning TempDir (kept alive for the test).
fn real_ssb(tag: &str) -> (Arc<Ssb>, Arc<dyn ClockProvider>, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join(format!("{tag}.db")),
        pool_size: Some(2),
        project_id: format!("i14_{tag}"),
        session_id: "i14_s".into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::with_epoch(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
    ));
    let ssb = Arc::new(Ssb::new(cfg, Arc::clone(&clock)).expect("ssb init"));
    (ssb, clock, dir)
}

// ---------------------------------------------------------------------------
// ARM-1: GD-2 determinism -- byte-identical output + identical reverse_hash.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i14_transform_is_byte_identical_across_runs() {
    let (ssb_a, clock_a, _da) = real_ssb("det_a");
    let (ssb_b, clock_b, _db) = real_ssb("det_b");

    let t_a = ContextTransformer::new(Arc::clone(&ssb_a), Arc::clone(&clock_a));
    let t_b = ContextTransformer::new(Arc::clone(&ssb_b), Arc::clone(&clock_b));

    let first = t_a
        .transform(RUST_FN_PAYLOAD, 1)
        .await
        .expect("transform A must succeed");
    let second = t_b
        .transform(RUST_FN_PAYLOAD, 1)
        .await
        .expect("transform B must succeed");

    assert_eq!(
        first.bytes, second.bytes,
        "GD-2: same input bytes must yield byte-identical compressed output"
    );
    assert_eq!(
        first.reverse_hash, second.reverse_hash,
        "GD-2: reverse_hash must be identical across runs (deterministic blake3 of input)"
    );
    assert_eq!(
        first.kind, second.kind,
        "GD-2: strategy classification must be deterministic"
    );
    // The reverse_hash must actually be blake3 of the ORIGINAL input bytes.
    let expected = *blake3::hash(RUST_FN_PAYLOAD).as_bytes();
    assert_eq!(
        first.reverse_hash, expected,
        "reverse_hash must be the full 32-byte blake3 of the original bytes"
    );
}

// ---------------------------------------------------------------------------
// ARM-2 + ARM-6: real DB-row receipt (NOT a struct-only round-trip).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i14_receipt_row_lands_in_real_db() {
    let (ssb, clock, _dir) = real_ssb("receipt");
    let transformer = ContextTransformer::new(Arc::clone(&ssb), Arc::clone(&clock));

    // Drive the real transform path (this is what production_agent_rpc.rs's
    // transform_tool_output override calls). It emits exactly one receipt row.
    let result = transformer
        .transform(RUST_FN_PAYLOAD, 7)
        .await
        .expect("transform must succeed and journal a receipt");

    // ARM-2: query the REAL SSB DB. The 3-arm Kind lives inside the payload, not
    // the SQL column, so any ContextTransformKind discriminant matches every
    // ContextTransformReceipt row (query_by_type keys on action_type.as_sql_str()).
    let rows = ssb
        .query_by_type(SsbActionType::ContextTransformReceipt(
            ContextTransformKind::LspSlice,
        ))
        .expect("query_by_type must succeed");

    // ARM-6 (ID-3 / R1): a transform that ran MUST have left a receipt. Zero rows
    // here is the fail-open the ID-3 doctrine forbids; this assertion catches it.
    assert!(
        !rows.is_empty(),
        "ID-3: a transform that ran must journal at least one ContextTransformReceipt row in the real DB"
    );

    // Decode the row and verify it carries the real receipt fields.
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode SsbPayload from real DB row");
    match decoded {
        SsbPayload::ContextTransformReceipt(ref p) => {
            // reverse_hash on the row must be the hex blake3 of the original input,
            // and must equal the in-memory result's hash (round-trip integrity).
            let expected_hex = hex_lower(&blake3::hash(RUST_FN_PAYLOAD).as_bytes()[..]);
            assert_eq!(
                p.reverse_hash, expected_hex,
                "row.reverse_hash must be hex blake3 of the original bytes"
            );
            assert_eq!(
                hex_lower(&result.reverse_hash[..]),
                p.reverse_hash,
                "row hash must match the TransformResult hash (same transform)"
            );
            assert_eq!(
                p.before_tokens, result.before_tokens,
                "row before_tokens must match the result"
            );
            assert_eq!(
                p.after_tokens, result.after_tokens,
                "row after_tokens must match the result"
            );
            assert!(
                p.ts_ms > 0,
                "row ts_ms must be a real (seeded) timestamp, got {}",
                p.ts_ms
            );
            // The Rust-fn payload classifies to AstFold (no compiler-error markers).
            assert_eq!(
                p.kind,
                ContextTransformKind::AstFold,
                "a plain Rust fn payload must classify to AstFold"
            );
        }
        other => panic!("expected ContextTransformReceipt row, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// ARM-3: AstFold correctness -- body elided, signature kept, tokens shrink.
// ---------------------------------------------------------------------------

#[test]
fn i14_astfold_elides_body_keeps_signature() {
    // A larger function so the fold visibly reduces size.
    let src = b"fn big() -> i32 {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n    let d = 4;\n    a + b + c + d\n}\n";
    let out = AstFoldTransform.apply(src, "rust");
    assert!(!out.skipped, "valid Rust with a body must fold, not skip");
    let folded = String::from_utf8(out.bytes.clone()).expect("utf8");
    assert!(
        folded.contains("fn big() -> i32"),
        "signature must be preserved, got: {folded}"
    );
    assert!(
        folded.contains("/* elided */"),
        "body must be elided, got: {folded}"
    );
    assert!(
        !folded.contains("let a = 1"),
        "body interior must be removed, got: {folded}"
    );
    assert!(
        out.bytes.len() < src.len(),
        "folded output ({}) must be smaller than the input ({})",
        out.bytes.len(),
        src.len()
    );
}

// ---------------------------------------------------------------------------
// ARM-4: LspSlice degradation + slicing -- environment-robust.
//
// LspSlice's `skipped` value for a payload WITH a `--> ` line depends on whether
// rust-analyzer is on PATH (absent -> ToolchainMissing -> skip; present -> slice).
// So we do NOT assert a fixed `skipped` value for the with-arrow case (that would
// make the contract pass/fail by host toolchain). Instead:
//   (a) a payload with NO `--> ` line MUST skip deterministically (the arrow-row
//       search returns None and bails BEFORE any toolchain-dependent slicing) --
//       this is the host-independent ToolchainMissing-or-no-location degrade path;
//   (b) a real rustc panic WITH a `--> ` line MUST never panic, and whichever
//       branch runs (skip => verbatim; slice => a window) the result is no larger
//       than the input and, when sliced, retains the located error line.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i14_lspslice_skips_without_a_location_line() {
    // No "--> " arrow line: nothing to slice deterministically, so the transform
    // returns the input verbatim with skipped=true REGARDLESS of toolchain presence.
    let no_loc = b"warning: something happened\n   |\n   = note: no location arrow here\n";
    let out = LspSliceTransform.apply(no_loc).await;
    assert!(
        out.skipped,
        "LspSlice with no `--> ` location must skip deterministically, got skipped=false"
    );
    assert_eq!(
        out.bytes,
        no_loc.to_vec(),
        "a skipped LspSlice must return the input verbatim (no drop, no panic)"
    );
}

#[tokio::test]
async fn i14_lspslice_handles_panic_payload_either_branch() {
    // A realistic rustc panic with a --> location line and a long backtrace tail.
    let mut panic =
        String::from("error[E0308]: mismatched types\n  --> src/main.rs:42:5\n   |\n42 |     let x: u8 = \"oops\";\n   |                 ^^^^^^ expected `u8`, found `&str`\n\nstack backtrace:\n");
    for i in 0..200 {
        panic.push_str(&format!("  {i:>3}: frame_{i}::deep::call\n"));
    }
    let bytes = panic.into_bytes();
    let out = LspSliceTransform.apply(&bytes).await;

    // Host-independent invariant: never larger than the input.
    assert!(
        out.bytes.len() <= bytes.len(),
        "LspSlice output ({}) must never exceed input ({})",
        out.bytes.len(),
        bytes.len()
    );
    if out.skipped {
        // Toolchain absent: verbatim passthrough.
        assert_eq!(
            out.bytes, bytes,
            "a skipped LspSlice must return the input verbatim"
        );
    } else {
        // Toolchain present: a real slice. It must keep the located error line
        // and drop the long backtrace tail (window is bounded around the arrow).
        let sliced = String::from_utf8(out.bytes.clone()).expect("sliced output utf8");
        assert!(
            sliced.contains("--> src/main.rs:42:5"),
            "a non-skipped slice must retain the located error line, got: {sliced}"
        );
        assert!(
            out.bytes.len() < bytes.len(),
            "a non-skipped slice must shrink the 200-frame backtrace ({} -> {})",
            bytes.len(),
            out.bytes.len()
        );
    }
}

// ---------------------------------------------------------------------------
// ARM-5: non-UTF-8 AstFold skip -- binary tool output returns verbatim + skip.
// ---------------------------------------------------------------------------

#[test]
fn i14_astfold_skips_non_utf8() {
    let bytes = [0xff_u8, 0xfe, 0x00, 0x80, 0x81];
    let out = AstFoldTransform.apply(&bytes, "rust");
    assert!(out.skipped, "non-utf8 input must skip (cannot AST-parse)");
    assert_eq!(
        out.bytes,
        bytes.to_vec(),
        "a skipped non-utf8 transform must return the input verbatim"
    );
}

/// Lowercase hex-encode a byte slice (mirror of the production hex_encode; kept
/// local so the contract does not depend on a private fn).
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}
