// E-45 — Cross-language drift detector contract test.
//
// Locks the public surface added by this plan:
//   * elai_orchestrator::cross_lang_drift::DriftDetector (host-side detector)
//   * elai_orchestrator::cross_lang_drift::PairRegistry (TOML loader)
//   * elai_orchestrator::cross_lang_drift::{DriftKind, DriftItem, DriftPair, PairKind, Severity}
//   * 3 SSB action types round-trip via msgpack:
//       - SsbActionType::CrossLangDriftDetected + payload
//       - SsbActionType::CrossLangPairScanned + payload
//       - SsbActionType::SchemaVersionMismatch + payload
//   * .claude/tools/cross_lang_diff.py has --json + --baseline + severity rollup
//   * .elai_cc/python/elai_inference/schemas/ has Pydantic mirror (messages.py)
//   * 10th lockstep site updated: action_type_lock.rs TOTAL_VARIANTS bumped
//     391 -> 394 atomically with the 3 E-45 variants.
//   * SCHEMA_VERSION_PUBLIC floor at 137.
//   * WIRE-DARK avoidance: drift_cmd is the production caller anchor for
//     DriftDetector::detect_pairs (cli/src/drift_cmd.rs:run_detect ->
//     build_report -> DriftDetector::detect_pairs).
//
// Design grounding source: D-10 DECISION-8 (hand-authored TS mirror over
// ts-rs codegen) + .claude/tools/cross_lang_diff.py existing 6-pair harness.
// E-45 plan Tasks 1+2 (ts-rs codegen) intentionally DEFERRED — this contract
// holds the alternative substrate (host-side drift detector + Pydantic
// boundary + extended diff tool) that supersedes Tasks 1+2.

use std::fs;
use std::path::{Path, PathBuf};

use elai_orchestrator::cross_lang_drift::{
    DriftDetector, DriftItem, DriftKind, DriftPair, PairKind, PairRegistry, Severity,
};
use elai_orchestrator::ssb::{
    CrossLangDriftDetectedPayload, CrossLangPairScannedPayload, SchemaVersionMismatchPayload,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// (F1) SCHEMA_VERSION_PUBLIC floor: E-45 atomic-bumped to 137.
// ---------------------------------------------------------------------------

#[test]
fn e45_f1_schema_version_floor_is_137_or_higher() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 137,
        "E-45 atomic-bump puts SCHEMA_VERSION_PUBLIC at >= 137, found {SCHEMA_VERSION_PUBLIC}"
    );
}

// ---------------------------------------------------------------------------
// (F2) DriftDetector + DriftKind variants exist.
// ---------------------------------------------------------------------------

#[test]
fn e45_f2_drift_kind_variants_cover_design() {
    // Variant smoke: each enum constructor compiles. The detector
    // implementation maps file-IO drift signals to these arms; if a
    // variant were removed, this file fails to compile.
    let _added = DriftKind::TypeFieldAdded {
        field: "x".into(),
    };
    let _removed = DriftKind::TypeFieldRemoved {
        field: "x".into(),
    };
    let _renamed = DriftKind::TypeFieldRenamed {
        rust_name: "a".into(),
        mirror_name: "b".into(),
    };
    let _ver = DriftKind::SchemaVersionMismatch {
        rust_version: 137,
        mirror_version: 136,
    };
    let _missing = DriftKind::PairSideMissing {
        missing_side: PathBuf::from("nope"),
    };
}

#[test]
fn e45_f2b_drift_item_severity_mapping_is_stable() {
    // Critical: rename / version / missing.
    let ren = DriftItem::new(
        DriftKind::TypeFieldRenamed {
            rust_name: "a".into(),
            mirror_name: "b".into(),
        },
        PathBuf::from("x"),
    );
    assert_eq!(ren.severity, Severity::Critical);

    let ver = DriftItem::new(
        DriftKind::SchemaVersionMismatch {
            rust_version: 1,
            mirror_version: 2,
        },
        PathBuf::from("x"),
    );
    assert_eq!(ver.severity, Severity::Critical);

    // Warning: add / remove.
    let add = DriftItem::new(
        DriftKind::TypeFieldAdded {
            field: "f".into(),
        },
        PathBuf::from("x"),
    );
    assert_eq!(add.severity, Severity::Warning);
}

// ---------------------------------------------------------------------------
// (F3) PairRegistry loads from cross-lang-pairs.toml.
// ---------------------------------------------------------------------------

#[test]
fn e45_f3_pair_registry_loads_canonical_toml() {
    // The canonical registry is at .elai_cc/cross-lang-pairs.toml relative
    // to the workflow root. The test runs from the .elai_cc workspace
    // (cargo test cwd), so we climb one level to find the registry.
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR -> .../crates/contracts. Climb to .elai_cc:
    let elai_cc = workspace
        .parent()
        .expect("contracts has a parent")
        .parent()
        .expect("crates has a parent");
    let registry_path = elai_cc.join("cross-lang-pairs.toml");
    assert!(
        registry_path.exists(),
        "canonical registry must exist at {}",
        registry_path.display()
    );

    let reg = PairRegistry::load_from(&registry_path).expect("registry loads");
    assert!(
        reg.len() >= 2,
        "canonical registry must declare >= 2 pairs; got {}",
        reg.len()
    );
    // Sidecar pair must be present (it is the load-bearing boundary).
    let has_sidecar = reg
        .pairs()
        .iter()
        .any(|p| p.kind == PairKind::SidecarOrchestratorSchemas);
    assert!(
        has_sidecar,
        "canonical registry must include sidecar_orchestrator_schemas pair"
    );
}

// ---------------------------------------------------------------------------
// (F4) 3 SSB variants round-trip via msgpack.
// ---------------------------------------------------------------------------

#[test]
fn e45_f4_cross_lang_drift_detected_round_trips() {
    let payload = SsbPayload::CrossLangDriftDetected(CrossLangDriftDetectedPayload {
        pair_kind_hash: 0xDEAD_BEEF_0000_0000,
        drift_count: 3,
        severity_max: 3,
        ts_ms: 1_700_000_137_999,
    });
    assert_eq!(payload.action_type(), SsbActionType::CrossLangDriftDetected);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

#[test]
fn e45_f4b_cross_lang_pair_scanned_round_trips() {
    let payload = SsbPayload::CrossLangPairScanned(CrossLangPairScannedPayload {
        repo_root_hash: 0xCAFE_F00D_0000_0000,
        pair_count: 6,
        ts_ms: 1_700_000_137_001,
    });
    assert_eq!(payload.action_type(), SsbActionType::CrossLangPairScanned);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

#[test]
fn e45_f4c_schema_version_mismatch_round_trips() {
    let payload = SsbPayload::SchemaVersionMismatch(SchemaVersionMismatchPayload {
        rust_schema: 137,
        python_schema: 136,
        ts_ms: 1_700_000_137_002,
    });
    assert_eq!(payload.action_type(), SsbActionType::SchemaVersionMismatch);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

// ---------------------------------------------------------------------------
// (F5) cross_lang_diff.py extension: --json + --baseline flags present.
// ---------------------------------------------------------------------------

#[test]
fn e45_f5_cross_lang_diff_py_has_extension_flags() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // .../crates/contracts -> climb to workflow root (parent of .elai_cc).
    let workflow_root = workspace
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("contracts has a 3-up parent (workflow root)");
    let py_path = workflow_root.join(".claude").join("tools").join("cross_lang_diff.py");
    assert!(
        py_path.exists(),
        "cross_lang_diff.py must exist at {}",
        py_path.display()
    );
    let body = fs::read_to_string(&py_path).expect("read py");
    assert!(
        body.contains("--baseline"),
        "cross_lang_diff.py must expose --baseline flag (E-45 extension)"
    );
    assert!(
        body.contains("_classify_severity")
            || body.contains("severity"),
        "cross_lang_diff.py must include severity rollup (E-45 extension)"
    );
    // --json flag is the existing alias for the as_json arg; confirm it
    // emits a severity field in the JSON payload path.
    assert!(
        body.contains("\"severity\""),
        "cross_lang_diff.py --json must include severity in the report payload"
    );
}

// ---------------------------------------------------------------------------
// (F6) Sidecar Pydantic mirror present (presence-only test; running
//      Python is out-of-scope for the contracts crate).
// ---------------------------------------------------------------------------

#[test]
fn e45_f6_sidecar_pydantic_schemas_present() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let elai_cc = workspace
        .parent()
        .and_then(|p| p.parent())
        .expect("contracts has 2-up parent (.elai_cc)");
    let schemas_init =
        elai_cc.join("python").join("elai_inference").join("schemas").join("__init__.py");
    let messages = elai_cc
        .join("python")
        .join("elai_inference")
        .join("schemas")
        .join("messages.py");
    assert!(
        schemas_init.exists(),
        "schemas/__init__.py must exist at {}",
        schemas_init.display()
    );
    assert!(
        messages.exists(),
        "schemas/messages.py must exist at {}",
        messages.display()
    );
    let messages_body = fs::read_to_string(&messages).expect("read messages.py");
    assert!(
        messages_body.contains("MessagesRequest"),
        "messages.py must declare MessagesRequest"
    );
    assert!(
        messages_body.contains("SCHEMA_VERSION"),
        "messages.py must declare SCHEMA_VERSION mirror"
    );
}

// ---------------------------------------------------------------------------
// (F7) 10th lockstep site (TOTAL_VARIANTS) updated. We can't import the
//      const because it lives in the test crate's tests/ subdir, but we
//      can assert the source contains the post-E-45 count.
// ---------------------------------------------------------------------------

#[test]
fn e45_f7_action_type_lock_total_variants_updated() {
    // Tier-B Move 2 derived TOTAL_VARIANTS and relocated ALL_VARIANTS (now
    // SSB_ACTION_TYPES in ssb.rs), so the source-text reads of action_type_lock.rs
    // no longer apply. This was the sole exact-equality TOTAL_VARIANTS lock; it is
    // converted to a runtime >= 562 floor (562 = the live count that includes every
    // lockstep variant below) plus runtime registration checks for each variant.
    use elai_orchestrator::ssb::SSB_ACTION_TYPES;

    assert!(
        SSB_ACTION_TYPES.len() >= 562,
        "TOTAL_VARIANTS must be >= 562 (live registry count); got {}",
        SSB_ACTION_TYPES.len()
    );

    // Every lockstep variant below must be registered in SSB_ACTION_TYPES, asserted
    // via its stable wire tag (arm-shape-agnostic; tighter than a source grep).
    let registered = |tag: &str| SSB_ACTION_TYPES.iter().any(|v| v.as_sql_str() == tag);

    // H-23 same-seed rerun hash equality gate.
    assert!(registered("rerun_hash_equality"), "registry must include RerunHashEquality");
    // H-24 sim-to-real parity gate.
    assert!(registered("sim_real_parity"), "registry must include SimRealParity");
    // H-25 result manifest re-derivation gate.
    assert!(
        registered("result_manifest_verified"),
        "registry must include ResultManifestVerified"
    );
    // H-26 shadow-exec + financial-invariant MR gate.
    assert!(
        registered("differential_divergence"),
        "registry must include DifferentialDivergence"
    );
    // H-27 gate self-validation META gate.
    assert!(
        registered("gate_self_validation"),
        "registry must include GateSelfValidation"
    );
    // H-28 current-RF reward-tampering gate.
    assert!(registered("reward_tampering"), "registry must include RewardTampering");
    // H-19 reusable-holdout Thresholdout DP query budget.
    assert!(
        registered("holdout_budget_event"),
        "registry must include HoldoutBudgetEvent"
    );
    // The 3 E-45 variants.
    assert!(
        registered("cross_lang_drift_detected"),
        "registry must include CrossLangDriftDetected"
    );
    assert!(
        registered("cross_lang_pair_scanned"),
        "registry must include CrossLangPairScanned"
    );
    assert!(
        registered("schema_version_mismatch"),
        "registry must include SchemaVersionMismatch"
    );
    // I-35 temporal tool-sequence enforcement.
    assert!(
        registered("tool_sequence_verdict"),
        "registry must include ToolSequenceVerdict (I-35 temporal sequence enforcement)"
    );
}

// ---------------------------------------------------------------------------
// (F8) WIRE-DARK CHECK: drift_cmd has DriftDetector::detect_pairs caller.
// ---------------------------------------------------------------------------

#[test]
fn e45_f8_drift_cmd_anchors_detect_pairs_caller() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let elai_cc = workspace
        .parent()
        .and_then(|p| p.parent())
        .expect("contracts has 2-up parent (.elai_cc)");
    let drift_cmd = elai_cc.join("crates").join("cli").join("src").join("drift_cmd.rs");
    assert!(
        drift_cmd.exists(),
        "drift_cmd.rs must exist at {}",
        drift_cmd.display()
    );
    let body = fs::read_to_string(&drift_cmd).expect("read drift_cmd.rs");
    assert!(
        body.contains("DriftDetector::new")
            || body.contains("DriftDetector::new("),
        "drift_cmd.rs must construct DriftDetector (WIRE-DARK anchor)"
    );
    assert!(
        body.contains("detect_pairs"),
        "drift_cmd.rs must call DriftDetector::detect_pairs (WIRE-DARK anchor)"
    );
    assert!(
        body.contains("PairRegistry::load"),
        "drift_cmd.rs must call PairRegistry::load"
    );
}

// ---------------------------------------------------------------------------
// (F9) End-to-end smoke: synthetic registry + synthetic file pair yields
//      expected drift items. Locks the detector's behavior contract.
// ---------------------------------------------------------------------------

#[test]
fn e45_f9_synthetic_detector_smoke() {
    use tempfile::TempDir;
    let dir = TempDir::new().expect("tempdir");

    // Create a synthetic pair: rust has fields {a,b,c}, mirror has {a,b}.
    let rust_rel = "rust_src.rs";
    let mirror_rel = "mirror.py";
    fs::write(
        dir.path().join(rust_rel),
        "pub struct X { pub a: u32, pub b: u32, pub c: u32 }",
    )
    .expect("write rust");
    fs::write(
        dir.path().join(mirror_rel),
        "class X:\n    a: int\n    b: int",
    )
    .expect("write mirror");

    let registry_toml = format!(
        r#"
[[pairs]]
name = "sidecar_orchestrator_schemas"
rust_path = "{rust_rel}"
mirror_path = "{mirror_rel}"
"#
    );
    let registry_path = dir.path().join("cross-lang-pairs.toml");
    fs::write(&registry_path, registry_toml).expect("write toml");

    let reg = PairRegistry::load_from(&registry_path).expect("load reg");
    let detector = DriftDetector::new(&reg).expect("detector");
    let results = detector.detect_pairs(dir.path()).expect("detect");
    assert_eq!(results.len(), 1);

    let drift = &results[0].drift_items;
    let added = drift.iter().any(|d| {
        matches!(
            &d.kind,
            DriftKind::TypeFieldAdded { field } if field == "c"
        )
    });
    assert!(added, "expected TypeFieldAdded for 'c', got {drift:?}");
}

// ---------------------------------------------------------------------------
// (F10) cross-lang-pairs.toml registry is present at canonical path
//       and reachable from .elai_cc/.
// ---------------------------------------------------------------------------

#[test]
fn e45_f10_canonical_registry_path_present() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let elai_cc = workspace
        .parent()
        .and_then(|p| p.parent())
        .expect("contracts has 2-up parent (.elai_cc)");
    let canonical = elai_cc.join("cross-lang-pairs.toml");
    assert!(
        canonical.exists(),
        "canonical cross-lang-pairs.toml must exist at {} \
         (consumed by elai drift detect / list-pairs / baseline)",
        canonical.display()
    );
}

// ---------------------------------------------------------------------------
// (F11) DriftPair / Severity exposed at module root for downstream
//       consumers (CLI + future dashboard renderer).
// ---------------------------------------------------------------------------

#[test]
fn e45_f11_public_surface_at_module_root() {
    // Compile-time guarantee: each of these types/enums is re-exported
    // from `cross_lang_drift::*`. If any goes missing, this file
    // breaks the build.
    let _: Option<DriftPair> = None;
    let _: Option<DriftItem> = None;
    let _: Option<DriftKind> = None;
    let _: Option<PairKind> = None;
    let _: Option<Severity> = None;
}

// Silence dead-code warning for unused Path import on platforms where
// the test bodies don't all reach the import.
#[allow(dead_code)]
fn _path_anchor(_p: &Path) {}
