// E-44 — Checkpointing durability contract test.
//
// Locks the public surface added by this plan:
//   * elai_orchestrator::durability::checkpoint_migrator (lazy schema migrator)
//   * elai_orchestrator::durability::idempotent_dispatcher (ToolDispatcher wrapper)
//   * elai_orchestrator::durability::compact_lock (fd-lock-backed CompactLock)
//   * elai_orchestrator::durability::continue_as_new (NEW wrapper around create_handoff)
//   * elai_orchestrator::durability::resume_validator (checkpoint validity check)
//   * elai_orchestrator::checkpoint::AgentCheckpointData.schema_version field
//   * 5 SSB action types round-trip via msgpack
//   * SCHEMA_VERSION_PUBLIC floor at 136
//   * WIRE-DARK avoidance: durability_cmd is the production caller anchor.
//
// Paper grounding DEFERRED per workflow/sessions/s-25120/mistakes.md
// cluster #8 (arXiv:2603.20625 / 2603.29231 / 2603.22744). No inline
// arXiv assertions in this file; the cite register is the documented
// errata path. The 3 papers are design-intent sources only.

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::checkpoint::{
    AgentCheckpointData, LEGACY_CHECKPOINT_SCHEMA_VERSION,
};
use elai_orchestrator::durability::{
    call_shape_hash, continue_as_new_if_needed, migrate_checkpoint, validate_checkpoint,
    CheckpointMigrator, CheckpointValidity, CompactLock, CompactLockError,
    ContinueAsNewError, ContinueAsNewOutcome, IdempotentDispatcher, MigratorError,
    CURRENT_CHECKPOINT_SCHEMA_VERSION, DEFAULT_EVENT_COUNT_CEILING,
    DEFAULT_PHASE_COUNT_CEILING,
};
use elai_orchestrator::ssb::{
    CheckpointMigratedPayload, CheckpointResumedPayload, CheckpointWrittenPayload,
    DurabilityLockBlockedPayload, IdempotentDispatchSkippedPayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// (F1) SCHEMA_VERSION_PUBLIC floor: E-44 atomic-bumped to 136.
#[test]
fn e44_f1_schema_version_floor_is_136_or_higher() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 136,
        "E-44 atomic-bump puts SCHEMA_VERSION_PUBLIC at >= 136, found {SCHEMA_VERSION_PUBLIC}"
    );
}

// (F2) AgentCheckpointData has the `schema_version` field with
// serde(default) backwards-compatibility.
#[test]
fn e44_f2_agent_checkpoint_data_has_schema_version_with_default() {
    let data = AgentCheckpointData {
        agent_status: "running".into(),
        grc_history: vec![],
        active_files: vec![],
        tool_results: vec![],
        output_draft: "".into(),
        channel_references: vec![],
        schema_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
    };
    assert_eq!(data.schema_version, CURRENT_CHECKPOINT_SCHEMA_VERSION);

    // Older blobs WITHOUT schema_version must deserialise at the
    // LEGACY default via serde(default). Construct a minimal blob
    // missing the field by serialising a parallel struct without it.
    #[derive(serde::Serialize)]
    struct LegacyBlob {
        agent_status: String,
        grc_history: Vec<()>,
        active_files: Vec<PathBuf>,
        tool_results: Vec<String>,
        output_draft: String,
        channel_references: Vec<()>,
    }
    let legacy = LegacyBlob {
        agent_status: "legacy".into(),
        grc_history: vec![],
        active_files: vec![],
        tool_results: vec![],
        output_draft: "".into(),
        channel_references: vec![],
    };
    let bytes = rmp_serde::to_vec_named(&legacy).expect("encode legacy blob");
    let decoded: AgentCheckpointData =
        rmp_serde::from_slice(&bytes).expect("decode legacy blob via serde(default)");
    assert_eq!(decoded.schema_version, LEGACY_CHECKPOINT_SCHEMA_VERSION);
    assert_eq!(decoded.agent_status, "legacy");
}

// (F3) CheckpointMigrator chain bridges LEGACY -> CURRENT.
#[test]
fn e44_f3_migrator_chain_bridges_legacy_to_current() {
    let m = CheckpointMigrator::with_defaults();
    let input = b"checkpoint-bytes".to_vec();
    let out = m
        .migrate(
            &input,
            LEGACY_CHECKPOINT_SCHEMA_VERSION,
            CURRENT_CHECKPOINT_SCHEMA_VERSION,
        )
        .expect("migrate ok");
    assert_eq!(out, input, "identity transform chain must passthrough");
}

// (F4) Migrator rejects downgrade attempts.
#[test]
fn e44_f4_migrator_rejects_downgrade() {
    let m = CheckpointMigrator::with_defaults();
    let err = m
        .migrate(b"x", CURRENT_CHECKPOINT_SCHEMA_VERSION, 40)
        .expect_err("downgrade must error");
    assert!(matches!(err, MigratorError::Downgrade { .. }));
}

// (F5) migrate_checkpoint convenience function uses the default
// registry and bridges to CURRENT.
#[test]
fn e44_f5_migrate_checkpoint_helper_targets_current() {
    let out = migrate_checkpoint(b"abc", LEGACY_CHECKPOINT_SCHEMA_VERSION).expect("ok");
    assert_eq!(out, b"abc");
}

// (F6) IdempotentDispatcher impls ToolDispatcher (compile-time
// contract). The assertion is via Arc<dyn ToolDispatcher> coercion.
#[test]
fn e44_f6_idempotent_dispatcher_impls_tool_dispatcher() {
    use elai_core::tool_dispatcher::ToolDispatcher;
    // Take a doc-test InProcessToolDispatcher via the cli/orchestrator
    // facade — but the contract only requires the trait impl to exist.
    // A compile-time check is sufficient: we assert IdempotentDispatcher
    // can be used as `dyn ToolDispatcher` via Arc coercion.
    fn _assert_impl<T: ToolDispatcher + ?Sized>() {}
    _assert_impl::<IdempotentDispatcher>();
    let _ = Arc::new as fn(IdempotentDispatcher) -> Arc<IdempotentDispatcher>;
}

// (F7) call_shape_hash separates methods (tag enforces distinction).
#[test]
fn e44_f7_call_shape_hash_separates_methods_and_bodies() {
    let body = b"identical";
    let h_edit = call_shape_hash("dispatch_edit", body);
    let h_validate = call_shape_hash("dispatch_validate", body);
    assert_ne!(
        h_edit, h_validate,
        "method tag must produce distinct hash for identical body"
    );
    let h_diff = call_shape_hash("dispatch_edit", b"different");
    assert_ne!(h_edit, h_diff, "different body must produce distinct hash");
}

// (F8) CompactLock.try_acquire returns a Result<LockGuard, ...>.
#[test]
fn e44_f8_compact_lock_try_acquire_returns_result() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lock = CompactLock::new(dir.path().join(".compact-lock"));
    let result = lock.try_acquire();
    match result {
        Ok(_guard) => {}
        Err(CompactLockError::Contended { .. }) => {}
        Err(CompactLockError::OpenFailed { .. }) => {
            panic!("fresh tempdir lock must not fail to open");
        }
    }
}

// (F9) continue_as_new_if_needed is a NEW wrapper, not a modification
// of create_handoff. Calling the wrapper does not touch SessionDag —
// it returns a pure decision over the supplied counts.
#[test]
fn e44_f9_continue_as_new_is_new_wrapper_not_create_handoff_mod() {
    // Below threshold: BelowThreshold variant.
    let outcome = continue_as_new_if_needed(
        "session-x",
        10,
        5,
        DEFAULT_EVENT_COUNT_CEILING,
        DEFAULT_PHASE_COUNT_CEILING,
    )
    .expect("ok");
    assert!(matches!(outcome, ContinueAsNewOutcome::BelowThreshold { .. }));

    // At threshold: Rollover variant carrying NewSessionHandle.
    let outcome2 = continue_as_new_if_needed(
        "session-x",
        DEFAULT_EVENT_COUNT_CEILING,
        5,
        DEFAULT_EVENT_COUNT_CEILING,
        DEFAULT_PHASE_COUNT_CEILING,
    )
    .expect("ok");
    assert!(matches!(outcome2, ContinueAsNewOutcome::Rollover(_)));

    // Zero ceilings error.
    let err = continue_as_new_if_needed("s", 0, 0, 0, 0).expect_err("zero ceilings");
    assert!(matches!(err, ContinueAsNewError::InvalidThresholds));
}

// (F10) validate_checkpoint produces the documented variants.
#[test]
fn e44_f10_validate_checkpoint_variants() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Empty dir -> PartialFiles (every file missing).
    let v = validate_checkpoint(dir.path(), None).expect("ok");
    assert!(matches!(v, CheckpointValidity::PartialFiles { .. }));
}

// (F11) 5 SSB variants round-trip via msgpack.
#[test]
fn e44_f11_ssb_variants_round_trip() {
    let payloads: Vec<SsbPayload> = vec![
        SsbPayload::CheckpointWritten(CheckpointWrittenPayload {
            session_id_hash: 0xAA,
            schema_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
            file_count: 5,
            ts_ms: 1,
        }),
        SsbPayload::CheckpointResumed(CheckpointResumedPayload {
            session_id_hash: 0xBB,
            schema_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
            ts_ms: 2,
        }),
        SsbPayload::CheckpointMigrated(CheckpointMigratedPayload {
            from_version: 40,
            to_version: CURRENT_CHECKPOINT_SCHEMA_VERSION,
            transform_count: 96,
            ts_ms: 3,
        }),
        SsbPayload::IdempotentDispatchSkipped(IdempotentDispatchSkippedPayload {
            call_shape_hash: 0xCC,
            cached_result_hash: 0xDD,
            ts_ms: 4,
        }),
        SsbPayload::DurabilityLockBlocked(DurabilityLockBlockedPayload {
            lock_path_hash: 0xEE,
            holder_hash: 0xFF,
            ts_ms: 5,
        }),
    ];
    for p in payloads {
        let bytes = rmp_serde::to_vec_named(&p).expect("encode");
        let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(p.action_type(), back.action_type());
    }
}

// (F12) Every new SSB action_type round-trips via from_str / as_sql_str.
#[test]
fn e44_f12_ssb_action_type_round_trip() {
    use elai_orchestrator::ssb::ssb_action_type_from_str_public;
    let cases = [
        ("checkpoint_written", SsbActionType::CheckpointWritten),
        ("checkpoint_resumed", SsbActionType::CheckpointResumed),
        ("checkpoint_migrated", SsbActionType::CheckpointMigrated),
        (
            "idempotent_dispatch_skipped",
            SsbActionType::IdempotentDispatchSkipped,
        ),
        ("durability_lock_blocked", SsbActionType::DurabilityLockBlocked),
    ];
    for (s, expected) in cases {
        let parsed = ssb_action_type_from_str_public(s).expect("parse");
        assert_eq!(parsed, expected);
        assert_eq!(parsed.as_sql_str(), s);
    }
}

// (F13) WIRE-DARK avoidance: the durability_cmd module is the
// documented production caller anchor for every public function in
// `durability::*`. This is enforced by a string-search assertion
// against the CLI source.
//
// Each of the 5 primitives MUST appear at least once in
// `crates/cli/src/durability_cmd.rs`.
#[test]
fn e44_f13_wire_dark_avoidance_durability_cmd_calls_each_primitive() {
    let cli_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("contracts parent")
        .join("cli")
        .join("src")
        .join("durability_cmd.rs");
    let src = std::fs::read_to_string(&cli_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", cli_path.display()));
    for needle in &[
        "validate_checkpoint",
        "migrate_checkpoint",
        "continue_as_new_if_needed",
        "CompactLock",
        "IdempotentDispatcher",
    ] {
        assert!(
            src.contains(needle),
            "durability_cmd.rs must reference `{needle}` to anchor production wiring \
             (WIRE-DARK avoidance per E-41 precedent). File: {}",
            cli_path.display()
        );
    }
}

// (F14) Determinism: identical inputs to continue_as_new produce
// identical NewSessionHandle (blake3 archive hash is deterministic).
#[test]
fn e44_f14_continue_as_new_deterministic() {
    let a = continue_as_new_if_needed("sid", 500, 5, 500, 100).expect("ok");
    let b = continue_as_new_if_needed("sid", 500, 5, 500, 100).expect("ok");
    assert_eq!(a, b, "identical inputs must produce identical outcomes");
    let c = continue_as_new_if_needed("other", 500, 5, 500, 100).expect("ok");
    assert_ne!(a, c, "session id must affect the hash");
}
