// I-13: MemoryGuardEvent -- write-time memory guard contract.
//
// Pins the contract that the aho-corasick MemoryWriteScanner (CHOKEPOINT-1 and
// CHOKEPOINT-2) blocks invisible-Unicode and confusable-shell patterns with a
// hard Err (fail-closed, no log-and-continue), and that the MemoryGuardEvent SSB
// variants (PoisonBlocked / DualSerialized / TraceQueryExpanded) round-trip
// through SSB serde correctly.
//
// ARM-10 and ARM-11 verify the dual-serialization trait (to_ui_dict / to_llm_content)
// produces distinct outputs and that the DualSerialized SSB row lands in the real DB.
//
// Gate entry in wire-dark-allowlist.toml:
//   name = "ELAI_MEMORY_GUARD_ENABLE"
//   smoke_test = "crates/contracts/tests/i13_memory_guard_contract.rs::i13_chokepoint1_invisible_unicode_blocked"
//
// SCHEMA_VERSION 249, TOTAL_VARIANTS 558.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use std::pin::Pin;
use std::future::Future;

use elai_memory::{extractor::ExtractedFact, fact::FactType};
use elai_ipc_types::{FsmState, ToolResultPayload};
use elai_orchestrator::{
    memory_guard::{MemoryGuardVerdict, MemoryWriteScanner},
    ssb::{MemoryGuardEventKind, MemoryGuardEventPayload, SsbActionType, SsbConfig, SsbPayload},
    Ssb, SsbReader,
};
use elai_tools::{
    ipc_registry::{IpcToolRegistry, ToolError, ToolHandler},
    tool_result::DualSerializable,
    trace_query::TraceQuery,
};
use elai_core::clock::SeededClock;
use tempfile::TempDir;

/// Serialize env-var mutation tests so parallel test threads cannot race on
/// ELAI_MEMORY_GUARD_ENABLE (std::env is process-global, not thread-safe).
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Helper: build a minimal ExtractedFact for scanner tests.
fn cand(key: &str, val: serde_json::Value) -> ExtractedFact {
    ExtractedFact {
        fact_type: FactType::Preference,
        key: key.into(),
        value: val,
        confidence: 0.9,
        evidence: vec!["i13-contract".into()],
    }
}

// ---------------------------------------------------------------------------
// ARM-1: guard disabled by default (ELAI_MEMORY_GUARD_ENABLE not set)
// ---------------------------------------------------------------------------

#[test]
fn i13_guard_disabled_by_default() {
    let _g = ENV_LOCK.lock().unwrap();
    // ENV must NOT be set in the default test run; confirms the
    // experiment gate is not default-ON (wiring-enforcement R9).
    std::env::remove_var("ELAI_MEMORY_GUARD_ENABLE");
    // A payload that would be blocked if the guard were on must pass when disabled.
    let fact = cand("test_key", serde_json::json!("\u{200B} invisible"));
    let verdict = MemoryWriteScanner::scan(&fact).expect("scan must not Err when disabled");
    assert!(
        matches!(verdict, MemoryGuardVerdict::GuardDisabled),
        "expected GuardDisabled when ELAI_MEMORY_GUARD_ENABLE is not set, got {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// ARM-2: invisible Unicode blocked (CHOKEPOINT-1 gate)
// ---------------------------------------------------------------------------

#[test]
fn i13_chokepoint1_invisible_unicode_blocked() {
    let _g = ENV_LOCK.lock().unwrap();
    std::env::set_var("ELAI_MEMORY_GUARD_ENABLE", "1");
    // U+200B (ZERO-WIDTH SPACE) embedded in a memory key.
    let fact = cand("inject\u{200B}key", serde_json::json!("clean value"));
    let verdict = MemoryWriteScanner::scan(&fact).expect("scan must not Err");
    assert!(
        matches!(
            verdict,
            MemoryGuardVerdict::PoisonBlocked { ref pattern_class } if pattern_class.contains("invisible")
        ),
        "expected PoisonBlocked(invisible_unicode), got {verdict:?}"
    );
    std::env::remove_var("ELAI_MEMORY_GUARD_ENABLE");
}

// ---------------------------------------------------------------------------
// ARM-3: confusable-shell pattern blocked (Cyrillic homoglyph)
// ---------------------------------------------------------------------------

#[test]
fn i13_chokepoint1_confusable_shell_blocked() {
    let _g = ENV_LOCK.lock().unwrap();
    std::env::set_var("ELAI_MEMORY_GUARD_ENABLE", "1");
    // Greek sigma (U+03C3) + 'udo' -- confusable for "sudo".
    let fact = cand("cmd_key", serde_json::json!("\u{03C3}udo rm -rf /"));
    let verdict = MemoryWriteScanner::scan(&fact).expect("scan must not Err");
    assert!(
        matches!(
            verdict,
            MemoryGuardVerdict::PoisonBlocked { ref pattern_class } if pattern_class.contains("confusable")
        ),
        "expected PoisonBlocked(confusable_shell), got {verdict:?}"
    );
    std::env::remove_var("ELAI_MEMORY_GUARD_ENABLE");
}

// ---------------------------------------------------------------------------
// ARM-4: clean fact allowed when guard is enabled
// ---------------------------------------------------------------------------

#[test]
fn i13_clean_fact_allowed_when_guard_enabled() {
    let _g = ENV_LOCK.lock().unwrap();
    std::env::set_var("ELAI_MEMORY_GUARD_ENABLE", "1");
    let fact = cand("favorite_editor", serde_json::json!("neovim"));
    let verdict = MemoryWriteScanner::scan(&fact).expect("scan must not Err");
    assert!(
        matches!(verdict, MemoryGuardVerdict::Allow),
        "expected Allow for clean fact, got {verdict:?}"
    );
    std::env::remove_var("ELAI_MEMORY_GUARD_ENABLE");
}

// ---------------------------------------------------------------------------
// ARM-5: emoji in fact does not trigger guard (FP guard)
// ---------------------------------------------------------------------------

#[test]
fn i13_emoji_fact_not_blocked() {
    let _g = ENV_LOCK.lock().unwrap();
    std::env::set_var("ELAI_MEMORY_GUARD_ENABLE", "1");
    let fact = cand("emoji_key", serde_json::json!("i like \u{1F600} faces"));
    let verdict = MemoryWriteScanner::scan(&fact).expect("scan must not Err");
    assert!(
        matches!(verdict, MemoryGuardVerdict::Allow),
        "emoji should not trigger guard, got {verdict:?}"
    );
    std::env::remove_var("ELAI_MEMORY_GUARD_ENABLE");
}

// ---------------------------------------------------------------------------
// ARM-6: MemoryGuardEventPayload SSB round-trip (PoisonBlocked arm)
// ---------------------------------------------------------------------------

#[test]
fn i13_ssb_payload_poison_blocked_round_trip() {
    let payload = SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
        kind: MemoryGuardEventKind::PoisonBlocked,
        pattern_class: Some("invisible_unicode".into()),
        token_delta: None,
        byte_range: None,
        ts_ms: 1_700_000_001_000,
    });
    let json = serde_json::to_string(&payload).expect("serialize");
    let decoded: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(payload, decoded, "PoisonBlocked SSB payload must survive serde round-trip");
}

// ---------------------------------------------------------------------------
// ARM-7: MemoryGuardEventPayload SSB round-trip (DualSerialized arm)
// ---------------------------------------------------------------------------

#[test]
fn i13_ssb_payload_dual_serialized_round_trip() {
    let payload = SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
        kind: MemoryGuardEventKind::DualSerialized,
        pattern_class: None,
        token_delta: Some(256),
        byte_range: None,
        ts_ms: 1_700_000_002_000,
    });
    let json = serde_json::to_string(&payload).expect("serialize");
    let decoded: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(payload, decoded, "DualSerialized SSB payload must survive serde round-trip");
}

// ---------------------------------------------------------------------------
// ARM-8: MemoryGuardEventPayload SSB round-trip (TraceQueryExpanded arm)
// ---------------------------------------------------------------------------

#[test]
fn i13_ssb_payload_trace_query_expanded_round_trip() {
    let payload = SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
        kind: MemoryGuardEventKind::TraceQueryExpanded,
        pattern_class: None,
        token_delta: None,
        byte_range: Some("crates/orchestrator/src/ssb.rs:100-200".into()),
        ts_ms: 1_700_000_003_000,
    });
    let json = serde_json::to_string(&payload).expect("serialize");
    let decoded: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(payload, decoded, "TraceQueryExpanded SSB payload must survive serde round-trip");
}

// ---------------------------------------------------------------------------
// ARM-9: TraceQuery expand -- integration check (uses elai-tools directly)
// ---------------------------------------------------------------------------

#[test]
fn i13_trace_query_expand_integration() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut f = NamedTempFile::new().expect("tempfile");
    for i in 1..=10 {
        writeln!(f, "line_{i}").expect("write");
    }

    let result = TraceQuery::expand(f.path(), 3, 5).expect("expand");
    assert_eq!(result.start_line, 3);
    assert_eq!(result.end_line, 5);
    assert_eq!(result.total_lines, 10);
    assert!(result.content.contains("line_3"), "content must include line_3");
    assert!(result.content.contains("line_4"), "content must include line_4");
    assert!(result.content.contains("line_5"), "content must include line_5");
    assert!(!result.content.contains("line_2"), "content must not include line_2");
    assert!(!result.content.contains("line_6"), "content must not include line_6");
}

// ---------------------------------------------------------------------------
// ARM-10: dual-serialization trait -- to_ui_dict and to_llm_content are distinct
// for large payloads (Concern C delivery verification)
// ---------------------------------------------------------------------------

#[test]
fn i13_dual_serialization_produces_distinct_outputs() {
    use elai_tools::tool_result::LLM_OUTPUT_CHAR_LIMIT;

    // Large payload: output exceeds LLM_OUTPUT_CHAR_LIMIT.
    let large_output = "x".repeat(LLM_OUTPUT_CHAR_LIMIT + 1_000);
    let payload = ToolResultPayload {
        output: large_output.clone(),
        diagnostics: vec!["test_diag".to_string()],
    };

    let ui = payload.to_ui_dict();
    let llm = payload.to_llm_content();

    // to_ui_dict must contain the full output.
    assert!(
        ui.contains(&large_output),
        "to_ui_dict must contain the full output"
    );

    // to_llm_content must be shorter (condensed).
    assert!(
        llm.len() < ui.len(),
        "to_llm_content must be shorter than to_ui_dict for large output"
    );

    // to_llm_content must contain the truncation notice.
    assert!(
        llm.contains("truncated"),
        "to_llm_content must contain truncation notice"
    );

    // to_llm_content must NOT contain the full large output.
    assert!(
        !llm.contains(&large_output),
        "to_llm_content must not contain the full large output"
    );

    // Diagnostics appear in both.
    assert!(ui.contains("test_diag"), "to_ui_dict must include diagnostics");
    assert!(llm.contains("test_diag"), "to_llm_content must include diagnostics");

    // Token delta is positive.
    let delta = elai_tools::tool_result::dual_serialize_delta(&payload);
    assert!(
        delta > 0,
        "token delta must be positive for large output, got {delta}"
    );

    // Small payload: no truncation, no delta.
    let small = ToolResultPayload {
        output: "short output".to_string(),
        diagnostics: vec![],
    };
    let small_delta = elai_tools::tool_result::dual_serialize_delta(&small);
    assert_eq!(small_delta, 0, "token delta must be 0 for short output");
}

// ---------------------------------------------------------------------------
// ARM-11: DualSerialized SSB DB-row assertion (WARN D2 resolution)
// -- drives IpcToolRegistry::dispatch() with a real SSB and asserts the
//    DualSerialized row lands in the REAL DB (query_by_type + COUNT).
// ---------------------------------------------------------------------------

/// Minimal ToolHandler that returns a large ToolResultPayload on any call.
struct LargePayloadHandler;

impl ToolHandler for LargePayloadHandler {
    fn execute<'a>(
        &'a self,
        _arguments: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>> {
        use elai_tools::tool_result::LLM_OUTPUT_CHAR_LIMIT;
        Box::pin(async move {
            Ok(ToolResultPayload {
                output: "y".repeat(LLM_OUTPUT_CHAR_LIMIT + 2_000),
                diagnostics: vec![],
            })
        })
    }

    fn tool_id(&self) -> &str {
        "i13_test_large_tool"
    }

    fn is_write_tool(&self) -> bool {
        false
    }
}

#[tokio::test]
async fn i13_dual_serialized_ssb_row_lands_in_real_db() {
    // 1. Create a real TempDir SSB database.
    let dir = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("i13_dual.db"),
        pool_size: Some(2),
        project_id: "i13_test".into(),
        session_id: "i13_s".into(),
    };
    let clock = Arc::new(SeededClock::with_epoch(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
    )) as Arc<dyn elai_core::ClockProvider>;
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));

    // 2. Create registry with SSB wired, register the large-payload handler.
    let mut registry = IpcToolRegistry::new().with_ssb(Arc::clone(&ssb));
    registry.register(Arc::new(LargePayloadHandler));

    // 3. Dispatch the tool (triggers DualSerialized emit in IpcToolRegistry::dispatch).
    let result = registry
        .dispatch("i13_test_large_tool", &[], FsmState::Executing)
        .await;
    assert!(result.is_ok(), "dispatch must succeed, got: {result:?}");

    // 4. Query the real SSB DB for MemoryGuardEvent rows.
    //    The DualSerialized arm must be present (COUNT > 0).
    // query_by_type uses action_type.as_sql_str() — any MemoryGuardEventKind matches
    // all MemoryGuardEvent rows (the kind is inside the payload, not the SQL column).
    let rows = ssb
        .query_by_type(SsbActionType::MemoryGuardEvent(MemoryGuardEventKind::DualSerialized))
        .expect("query_by_type must succeed");

    assert!(
        !rows.is_empty(),
        "DualSerialized SSB row must land in the real DB after IpcToolRegistry::dispatch"
    );

    // 5. Decode and verify the row carries kind=DualSerialized and token_delta > 0.
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode SsbPayload");
    match decoded {
        SsbPayload::MemoryGuardEvent(ref p) => {
            assert_eq!(
                p.kind,
                MemoryGuardEventKind::DualSerialized,
                "first row kind must be DualSerialized"
            );
            assert!(
                p.token_delta.unwrap_or(0) > 0,
                "token_delta must be positive, got {:?}",
                p.token_delta
            );
        }
        other => panic!(
            "expected SsbPayload::MemoryGuardEvent, got different variant: {other:?}"
        ),
    }
}
