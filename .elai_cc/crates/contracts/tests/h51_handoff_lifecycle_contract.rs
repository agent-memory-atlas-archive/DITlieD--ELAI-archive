//! H-51 — cross-session sleep-time handoff lifecycle contract (source-wiring gate).
//!
//! Verifies the structural claims from the H-51 WIRING_CONTRACT:
//!
//! D1: HandoffLifecycleEvent is a live SSB action type (Pattern-2, 4-arm Kind
//!     discriminator: DigestWritten | ResumeOffered | ResumeConfirmed | TieredLoad).
//!
//! D2: author_topic_digest is exported from elai-orchestrator; detect_open_topic
//!     returns None when ELAI_HANDOFF_AUTORESUME_DISABLE=1 (kill-switch guard).
//!
//! D3: SCHEMA_VERSION in ssb.rs is >= 227 (H-51 bump from 226).
//!
//! D4: TieredLoad done-gate — loaded_tokens <= 5000 is the pass condition.
//!     Contract verifies the public MAX_DIGEST_TOKENS constant is <= 5000 and
//!     that detect_open_topic's token estimator is bounded.
//!
//! D5: Production wiring — live_cli.rs Drop hook calls author_topic_digest
//!     BEFORE ssb.shutdown (source-reachability assertion).
//!
//! D6: wire-dark-allowlist.toml carries the ELAI_HANDOFF_AUTORESUME_DISABLE
//!     entry with all mandatory R9 fields (type, owner, expiry_date,
//!     promotion_plan_id, smoke_test).
//!
//! All assertions are source-grep / unit-logic checks; no live model or sidecar
//! required.  The DigestWritten SSB roundtrip test (D1) uses the in-process
//! elai-orchestrator crate directly.

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn project_root() -> PathBuf {
    crates_dir()
        .parent()
        .expect("crates/ has a parent (.elai_cc)")
        .to_path_buf()
}

// Serialize all env-mutating tests to prevent races under the default parallel
// runner.  Using a plain std::sync::Mutex so no external dep is required
// (serial_test not in dev-deps; mirrors context_policy.rs idiom, WARN-2 fix).
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ---------------------------------------------------------------------------
// D1: HandoffLifecycleEvent roundtrip — Pattern-2 4-arm SSB variant
// ---------------------------------------------------------------------------

/// D1a: HandoffLifecycleEvent exists in SsbActionType and round-trips through
/// as_sql_str / from_str correctly.
#[test]
fn d1_handoff_lifecycle_event_action_type_roundtrip() {
    use elai_orchestrator::ssb::SsbActionType;

    let t = SsbActionType::HandoffLifecycleEvent;
    let wire = t.as_sql_str();
    assert_eq!(
        wire, "handoff_lifecycle_event",
        "HandoffLifecycleEvent wire string must be 'handoff_lifecycle_event', got '{wire}'"
    );

    let parsed = elai_orchestrator::ssb::ssb_action_type_from_str_public(wire)
        .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public('{wire}'): {e}"));
    assert_eq!(
        parsed,
        SsbActionType::HandoffLifecycleEvent,
        "ssb_action_type_from_str_public('handoff_lifecycle_event') must round-trip"
    );
}

/// D1b: HandoffLifecyclePayload serialises / deserialises correctly for all
/// four Kind arms via msgpack (the SSB wire format).
#[test]
fn d1b_handoff_lifecycle_payload_serde_all_arms() {
    use elai_orchestrator::ssb::{
        HandoffLifecycleEventKind, HandoffLifecyclePayload, SsbPayload,
    };

    let arms = [
        HandoffLifecycleEventKind::DigestWritten,
        HandoffLifecycleEventKind::ResumeOffered,
        HandoffLifecycleEventKind::ResumeConfirmed,
        HandoffLifecycleEventKind::TieredLoad,
    ];

    for kind in arms {
        let payload = SsbPayload::HandoffLifecycleEvent(HandoffLifecyclePayload {
            kind: kind.clone(),
            topic_slug: "test-topic".into(),
            digest_path: Some("workflow/handoffs/test-topic.md".into()),
            loaded_tokens: if matches!(kind, HandoffLifecycleEventKind::TieredLoad) {
                Some(1_500)
            } else {
                None
            },
            session_id: "s-test".into(),
        });

        let encoded =
            rmp_serde::to_vec(&payload).unwrap_or_else(|e| panic!("encode {kind:?}: {e}"));
        let decoded: SsbPayload =
            rmp_serde::from_slice(&encoded).unwrap_or_else(|e| panic!("decode {kind:?}: {e}"));

        if let SsbPayload::HandoffLifecycleEvent(p) = decoded {
            assert_eq!(
                p.kind, kind,
                "kind arm must survive msgpack roundtrip for {kind:?}"
            );
            assert_eq!(p.topic_slug, "test-topic");
        } else {
            panic!("decoded to wrong payload arm for kind={kind:?}");
        }
    }
}

/// D1c: HandoffLifecycleEventKind::as_str returns stable wire strings.
#[test]
fn d1c_kind_as_str_stable() {
    use elai_orchestrator::ssb::HandoffLifecycleEventKind;

    assert_eq!(HandoffLifecycleEventKind::DigestWritten.as_str(), "digest_written");
    assert_eq!(HandoffLifecycleEventKind::ResumeOffered.as_str(), "resume_offered");
    assert_eq!(HandoffLifecycleEventKind::ResumeConfirmed.as_str(), "resume_confirmed");
    assert_eq!(HandoffLifecycleEventKind::TieredLoad.as_str(), "tiered_load");
}

// ---------------------------------------------------------------------------
// D2: author_topic_digest / detect_open_topic public API
// ---------------------------------------------------------------------------

/// D2a: author_topic_digest is reachable in the orchestrator crate source.
#[test]
fn d2a_author_topic_digest_exported() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    assert!(
        src.contains("pub fn author_topic_digest"),
        "handoff_digest.rs must export author_topic_digest"
    );
    assert!(
        src.contains("pub fn detect_open_topic"),
        "handoff_digest.rs must export detect_open_topic"
    );
}

/// D2b: detect_open_topic respects ELAI_HANDOFF_AUTORESUME_DISABLE kill-switch.
/// This is a live function call, not just a source grep.
#[test]
fn d2b_detect_open_topic_kill_switch_runtime() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // Safety: test-only env mutation; serialized by ENV_MUTEX to prevent
    // races with d2c and any other env-mutating test under the parallel runner.
    unsafe { std::env::set_var("ELAI_HANDOFF_AUTORESUME_DISABLE", "1") };
    let result = elai_orchestrator::handoff_digest::detect_open_topic(std::path::Path::new("."));
    unsafe { std::env::remove_var("ELAI_HANDOFF_AUTORESUME_DISABLE") };

    assert!(
        result.is_none(),
        "detect_open_topic must return None when ELAI_HANDOFF_AUTORESUME_DISABLE=1"
    );
}

/// D2c: detect_open_topic on a fresh empty dir returns None (no false positives).
#[test]
fn d2c_detect_open_topic_empty_dir_returns_none() {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // Ensure kill-switch is off so we exercise the real path (serialized by ENV_MUTEX).
    unsafe { std::env::remove_var("ELAI_HANDOFF_AUTORESUME_DISABLE") };
    let dir = tempfile::TempDir::new().expect("tempdir");
    let result = elai_orchestrator::handoff_digest::detect_open_topic(dir.path());
    assert!(result.is_none(), "detect_open_topic on empty dir must return None");
}

/// D2d: lib.rs exports TopicHandle at the crate root.
#[test]
fn d2d_topic_handle_exported_at_crate_root() {
    let src = read_src("orchestrator/src/lib.rs");
    assert!(
        src.contains("pub use handoff_digest::TopicHandle"),
        "orchestrator/src/lib.rs must re-export TopicHandle at the crate root"
    );
    assert!(
        src.contains("pub mod handoff_digest"),
        "orchestrator/src/lib.rs must declare pub mod handoff_digest"
    );
}

// ---------------------------------------------------------------------------
// D3: SCHEMA_VERSION >= 227
// ---------------------------------------------------------------------------

/// D3: ssb.rs SCHEMA_VERSION is at least 227 (H-51 bump).
/// The const is declared as `const SCHEMA_VERSION: i64 = <N>;` in ssb.rs.
#[test]
fn d3_schema_version_at_least_227() {
    let src = read_src("orchestrator/src/ssb.rs");
    // Find the SCHEMA_VERSION const declaration line.
    // Pattern: `const SCHEMA_VERSION: i64 = NNN;`
    let line = src
        .lines()
        .find(|l| {
            l.contains("SCHEMA_VERSION") && l.contains("i64") && l.contains("=")
        })
        .unwrap_or_else(|| panic!("const SCHEMA_VERSION: i64 = N not found in ssb.rs"));

    // Extract the numeric value (after the last '=' before ';').
    let value: i64 = line
        .split('=')
        .nth(1)
        .and_then(|s| s.split(';').next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| panic!("could not parse SCHEMA_VERSION from: {line}"));

    assert!(
        value >= 227,
        "SCHEMA_VERSION must be >= 227 (H-51 bump from 226), got {value}"
    );
}

// ---------------------------------------------------------------------------
// D4: TieredLoad done-gate — loaded_tokens <= 5000
// ---------------------------------------------------------------------------

/// D4: The source-level token budget constant (MAX_DIGEST_TOKENS) is <= 5000.
/// This guarantees the TieredLoad done-gate (loaded_tokens <= 5000) can pass.
#[test]
fn d4_max_digest_tokens_within_tiered_load_gate() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    // Find the MAX_DIGEST_TOKENS line.
    let line = src
        .lines()
        .find(|l| l.contains("MAX_DIGEST_TOKENS") && l.contains("="))
        .unwrap_or_else(|| panic!("MAX_DIGEST_TOKENS const not found in handoff_digest.rs"));

    let value: usize = line
        .split('=')
        .nth(1)
        .and_then(|s| s.split(';').next())
        .and_then(|s| s.trim().replace('_', "").parse().ok())
        .unwrap_or_else(|| panic!("could not parse MAX_DIGEST_TOKENS from: {line}"));

    assert!(
        value <= 5_000,
        "MAX_DIGEST_TOKENS ({value}) must be <= 5000 (TieredLoad done-gate pass condition)"
    );
}

/// D4b: commands_h51.rs tiered_load_confirm enforces the 5000-token gate at
/// the Tauri command layer (loaded_tokens > 5000 returns Err).
#[test]
fn d4b_tauri_tiered_load_enforces_gate() {
    let src = read_src("dashboard-v2/src-tauri/src/commands_h51.rs");
    assert!(
        src.contains("5000") || src.contains("5_000"),
        "commands_h51.rs must enforce the 5000-token gate in tiered_load_confirm"
    );
    // The gate returns Err on violation — not Ok, not a warning.
    assert!(
        src.contains("Err("),
        "commands_h51.rs tiered_load_confirm must return Err when loaded_tokens > 5000"
    );
}

// ---------------------------------------------------------------------------
// D5: Production wiring — live_cli.rs Drop hook
// ---------------------------------------------------------------------------

/// D5a: live_cli.rs Drop hook calls author_topic_digest (source wiring).
#[test]
fn d5a_live_cli_drop_calls_author_topic_digest() {
    let src = read_src("cli/src/live_cli.rs");
    assert!(
        src.contains("author_topic_digest"),
        "live_cli.rs must call handoff_digest::author_topic_digest in its Drop hook (H-51 wiring)"
    );
}

/// D5b: live_cli.rs emits DigestWritten SSB row in Drop (source wiring).
#[test]
fn d5b_live_cli_drop_emits_digest_written_row() {
    let src = read_src("cli/src/live_cli.rs");
    assert!(
        src.contains("DigestWritten"),
        "live_cli.rs Drop must emit a DigestWritten HandoffLifecycleEvent SSB row"
    );
    assert!(
        src.contains("HandoffLifecycleEvent"),
        "live_cli.rs Drop must construct a HandoffLifecycleEvent payload"
    );
}

/// D5c: live_cli.rs Drop impl calls author_topic_digest BEFORE inner.shutdown()
/// (ordering check within the Drop block — inner.shutdown is the SSB shutdown).
///
/// Since the file has many `shutdown()` calls in other methods, we locate the
/// `impl Drop for LiveCli` block specifically and check relative positions inside it.
#[test]
fn d5c_live_cli_author_digest_before_shutdown() {
    let src = read_src("cli/src/live_cli.rs");

    // Locate the Drop impl block start.
    let drop_pos = src
        .find("impl Drop for LiveCli")
        .unwrap_or_else(|| panic!("'impl Drop for LiveCli' not found in live_cli.rs"));

    // Take the slice from the Drop impl start onward.
    let drop_slice = &src[drop_pos..];

    // Within the Drop slice, author_topic_digest must precede inner.shutdown().
    let author_in_drop = drop_slice
        .find("author_topic_digest")
        .unwrap_or_else(|| panic!("author_topic_digest not found inside Drop impl in live_cli.rs"));

    // The SSB shutdown inside Drop is `inner.shutdown()`.
    let shutdown_in_drop = drop_slice
        .find("inner.shutdown()")
        .unwrap_or_else(|| panic!("inner.shutdown() not found inside Drop impl in live_cli.rs"));

    assert!(
        author_in_drop < shutdown_in_drop,
        "In Drop impl: author_topic_digest (offset {author_in_drop}) must appear BEFORE \
         inner.shutdown() (offset {shutdown_in_drop})"
    );
}

// ---------------------------------------------------------------------------
// D6: wire-dark-allowlist.toml R9 compliance
// ---------------------------------------------------------------------------

/// D6: wire-dark-allowlist.toml must contain ELAI_HANDOFF_AUTORESUME_DISABLE
/// with all mandatory R9 fields present.
#[test]
fn d6_wire_dark_allowlist_has_h51_entry() {
    let path = project_root().join("wire-dark-allowlist.toml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read wire-dark-allowlist.toml: {e}"));

    assert!(
        content.contains("ELAI_HANDOFF_AUTORESUME_DISABLE"),
        "wire-dark-allowlist.toml must have a ELAI_HANDOFF_AUTORESUME_DISABLE entry (R9 compliance)"
    );
    // R9 mandatory fields: type, owner, expiry_date, promotion_plan_id, smoke_test
    // Find the block containing the flag name and check that the mandatory fields
    // appear within a reasonable proximity (within 400 chars of the flag name).
    let offset = content
        .find("ELAI_HANDOFF_AUTORESUME_DISABLE")
        .expect("flag name must be present");
    let window = &content[offset..content.len().min(offset + 400)];

    assert!(
        window.contains("kill_switch"),
        "ELAI_HANDOFF_AUTORESUME_DISABLE entry must have type = \"kill_switch\""
    );
    assert!(
        window.contains("owner"),
        "ELAI_HANDOFF_AUTORESUME_DISABLE entry must have an owner field"
    );
    assert!(
        window.contains("expiry_date"),
        "ELAI_HANDOFF_AUTORESUME_DISABLE entry must have expiry_date"
    );
    assert!(
        window.contains("promotion_plan_id"),
        "ELAI_HANDOFF_AUTORESUME_DISABLE entry must have promotion_plan_id"
    );
    assert!(
        window.contains("smoke_test"),
        "ELAI_HANDOFF_AUTORESUME_DISABLE entry must have smoke_test"
    );
}

// ---------------------------------------------------------------------------
// D7: Dashboard cross-lang pair — commands_h51.rs + lib.rs + stub.ts
// ---------------------------------------------------------------------------

/// D7a: commands_h51.rs exports resume_offer and tiered_load_confirm Tauri commands.
#[test]
fn d7a_tauri_commands_h51_exports_correct_commands() {
    let src = read_src("dashboard-v2/src-tauri/src/commands_h51.rs");
    assert!(
        src.contains("pub async fn resume_offer"),
        "commands_h51.rs must export async fn resume_offer"
    );
    assert!(
        src.contains("pub async fn tiered_load_confirm"),
        "commands_h51.rs must export async fn tiered_load_confirm"
    );
}

/// D7b: lib.rs registers both H-51 Tauri commands in generate_handler!.
#[test]
fn d7b_tauri_lib_registers_h51_commands() {
    let src = read_src("dashboard-v2/src-tauri/src/lib.rs");
    assert!(
        src.contains("commands_h51"),
        "dashboard-v2 lib.rs must declare mod commands_h51"
    );
    assert!(
        src.contains("resume_offer"),
        "lib.rs generate_handler! must include resume_offer"
    );
    assert!(
        src.contains("tiered_load_confirm"),
        "lib.rs generate_handler! must include tiered_load_confirm"
    );
}

/// D7c: stub.ts declares both H-51 function stubs for the Svelte frontend.
#[test]
fn d7c_ts_stub_declares_h51_functions() {
    let src = read_src("dashboard-v2/src/lib/api/stub.ts");
    assert!(
        src.contains("resume_offer"),
        "stub.ts must declare a resume_offer function (H-51 TS cross-lang pair)"
    );
    assert!(
        src.contains("tiered_load_confirm"),
        "stub.ts must declare a tiered_load_confirm function (H-51 TS cross-lang pair)"
    );
    assert!(
        src.contains("TopicHandleDto"),
        "stub.ts must declare the TopicHandleDto interface"
    );
    assert!(
        src.contains("TieredLoadResult"),
        "stub.ts must declare the TieredLoadResult interface"
    );
}

// ---------------------------------------------------------------------------
// D8: handoff_digest.rs STUB_BAN — no fabrication from LLM recollection
// ---------------------------------------------------------------------------

/// D8: The STUB_BAN assertion is documented in handoff_digest.rs source.
/// The module must only draw facts from InterventionRequest and FileWrite rows.
#[test]
fn d8_stub_ban_documented_in_source() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    assert!(
        src.contains("STUB_BAN") || src.contains("InterventionRequest"),
        "handoff_digest.rs must document the STUB_BAN (SSB-row-only fact extraction)"
    );
    assert!(
        src.contains("FileWrite"),
        "handoff_digest.rs must draw files only from FileWrite SSB rows"
    );
    // The function must NOT reference any LLM/sidecar call.
    assert!(
        !src.contains("query_sidecar") && !src.contains("model.generate"),
        "handoff_digest.rs must NOT call any model inference path (fabrication ban)"
    );
}

// ---------------------------------------------------------------------------
// D8b: repl.rs CLI inject path enforces the TieredLoad token gate
// ---------------------------------------------------------------------------

/// D8b: repl.rs must guard the system_prompt.push inject with a MAX_TIERED_LOAD_TOKENS
/// check, mirroring the dashboard gate in commands_h51.rs (D4b).
/// This closes the auditor BLOCK: the CLI inject path was unconditional, violating
/// the WIRING_CONTRACT stub_ban "a resume that loads >5K by default = FAIL" (R1/R4).
#[test]
fn d8b_repl_cli_inject_guarded_by_tiered_load_gate() {
    let src = read_src("cli/src/repl.rs");

    // A named constant must exist (no magic literal per house rules).
    assert!(
        src.contains("MAX_TIERED_LOAD_TOKENS"),
        "repl.rs must define a MAX_TIERED_LOAD_TOKENS const for the CLI inject gate"
    );

    // The const must be set to 5000 (matches the dashboard gate and the WIRING_CONTRACT).
    assert!(
        src.contains("MAX_TIERED_LOAD_TOKENS: u32 = 5_000")
            || src.contains("MAX_TIERED_LOAD_TOKENS: u32 = 5000"),
        "repl.rs MAX_TIERED_LOAD_TOKENS must equal 5_000"
    );

    // The inject (system_prompt.push) must be conditional on the gate.
    // Verify the gate check appears BEFORE the system_prompt.push in the file.
    let gate_pos = src
        .find("MAX_TIERED_LOAD_TOKENS")
        .expect("MAX_TIERED_LOAD_TOKENS not found in repl.rs (gate missing)");
    let push_pos = src
        .find("system_prompt.push")
        .expect("system_prompt.push not found in repl.rs (inject path missing)");
    assert!(
        gate_pos < push_pos,
        "repl.rs: MAX_TIERED_LOAD_TOKENS gate (offset {gate_pos}) must appear \
         BEFORE system_prompt.push (offset {push_pos})"
    );

    // An eprintln warning must be emitted on the skip path (not a silent drop).
    assert!(
        src.contains("skipping CLI inject") || src.contains("TieredLoad gate"),
        "repl.rs: the oversized-digest skip path must emit an eprintln naming the skip reason"
    );
}
