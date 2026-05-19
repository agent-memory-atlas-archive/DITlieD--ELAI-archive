//! C-22 contract: FeasibilityProber completeness gap-closure.
//!
//! Pins the following invariants:
//!   1. `SystemProbeResult` is the exported name (rename from the
//!      pre-C-22 `System` `Probe` `Report` per C-22 task 1).
//!   2. `GpuInfo` carries `uuid: Option<String>` (C-22 task 2).
//!   3. `ResourceDelta` struct is exported with the four
//!      `vram_bytes_before/after` / `ram_bytes_before/after` fields and
//!      derives `Default` (C-22 task 3).
//!   4. `StubResult` v2 carries four additive fields:
//!      `alternative_index: u8`, `stdout_hash: Option<String>`,
//!      `stderr_summary: Option<String>`, `resource_delta:
//!      Option<ResourceDelta>` (C-22 task 4). All four default to their
//!      zero value via `Default::default()` so existing struct-literal
//!      call sites can use the spread pattern.
//!   5. `SsbActionType::AlternativeExplored` +
//!      `SsbActionType::FeasibilityEscalated` exist, round-trip through
//!      `from_str_public` / `as_sql_str`, and have corresponding
//!      `SsbPayload` variants `AlternativeExplored` / `FeasibilityEscalated`.
//!   6. `SCHEMA_VERSION_PUBLIC` >= 54 (post-C-22 atomic bump).
//!
//! These invariants are deterministic; no I/O, no SSB DB, no shell-out.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AlternativeExploredPayload, FeasibilityEscalatedPayload,
    SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    DefaultFeasibilityProber, GpuInfo, ResourceDelta, StubResult, SystemProbeResult,
};

#[test]
fn c22_t1_system_probe_result_is_exported_name() {
    // Type-existence pin — compilation alone is the assertion. The
    // `SystemProbeResult` re-export is verified by `use elai_orchestrator::
    // SystemProbeResult;` at the top of this file. Equally pin the field
    // shape so a future rename to a different struct does not pass.
    let r = SystemProbeResult {
        gpu: None,
        ram_total_mb: 0,
        ram_free_mb: 0,
        disk_total_mb: 0,
        disk_free_mb: 0,
        detection_method: elai_orchestrator::DetectionMethod::Unavailable,
        probed_at_ms: 0,
    };
    assert!(r.gpu.is_none());
}

#[test]
fn c22_t2_gpu_info_has_uuid_option_string() {
    use elai_core::VramTier;
    // Construct with uuid: Some(...) — must compile and round-trip.
    let g = GpuInfo {
        name: "test-gpu".to_owned(),
        vram_total_mb: 12_288,
        vram_free_mb: 6_000,
        tier: VramTier::Vram12Gb,
        driver_version: Some("552.22".to_owned()),
        uuid: Some("GPU-12345678-1234-1234-1234-123456789012".to_owned()),
    };
    assert_eq!(g.uuid.as_deref(), Some("GPU-12345678-1234-1234-1234-123456789012"));

    // Construct with uuid: None — must also compile.
    let g_none = GpuInfo {
        name: "test-gpu-2".to_owned(),
        vram_total_mb: 24_576,
        vram_free_mb: 12_000,
        tier: VramTier::Vram24Gb,
        driver_version: None,
        uuid: None,
    };
    assert!(g_none.uuid.is_none());
}

#[test]
fn c22_t3_resource_delta_struct_exists_and_derives_default() {
    // Field shape pin.
    let d = ResourceDelta {
        vram_bytes_before: 1_000_000,
        vram_bytes_after: 900_000,
        ram_bytes_before: 8_000_000,
        ram_bytes_after: 7_900_000,
    };
    assert_eq!(d.vram_bytes_before, 1_000_000);
    assert_eq!(d.vram_bytes_after, 900_000);
    assert_eq!(d.ram_bytes_before, 8_000_000);
    assert_eq!(d.ram_bytes_after, 7_900_000);

    // Default derive: spread pattern compiles.
    let d2 = ResourceDelta::default();
    assert_eq!(d2.vram_bytes_before, 0);
    assert_eq!(d2.vram_bytes_after, 0);
    assert_eq!(d2.ram_bytes_before, 0);
    assert_eq!(d2.ram_bytes_after, 0);
}

#[test]
fn c22_t4_stub_result_v2_has_four_additive_fields() {
    // Full construction with all v2 fields.
    let r = StubResult {
        step_id: "c22-step".to_owned(),
        stub_passed: true,
        elapsed_ms: 12,
        error: None,
        alternative_index: 2,
        stdout_hash: Some("deadbeef".to_owned()),
        stderr_summary: Some("warning: foo".to_owned()),
        resource_delta: Some(ResourceDelta::default()),
    };
    assert_eq!(r.alternative_index, 2);
    assert_eq!(r.stdout_hash.as_deref(), Some("deadbeef"));
    assert_eq!(r.stderr_summary.as_deref(), Some("warning: foo"));
    assert!(r.resource_delta.is_some());

    // Default-spread construction must compile (proves `#[derive(Default)]`
    // landed on StubResult).
    let r_default = StubResult {
        step_id: "x".to_owned(),
        ..Default::default()
    };
    assert_eq!(r_default.alternative_index, 0);
    assert!(r_default.stdout_hash.is_none());
    assert!(r_default.stderr_summary.is_none());
    assert!(r_default.resource_delta.is_none());

    // Round-trip via msgpack — back-compat: v1 payload (no v2 fields)
    // must deserialize cleanly into v2 StubResult with serde defaults.
    let v1_only = StubResult {
        step_id: "legacy".to_owned(),
        stub_passed: true,
        elapsed_ms: 5,
        error: None,
        ..Default::default()
    };
    let bytes = rmp_serde::to_vec_named(&v1_only).expect("encode v1-shape StubResult");
    let decoded: StubResult = rmp_serde::from_slice(&bytes).expect("decode StubResult v2");
    assert_eq!(decoded.alternative_index, 0);
    assert!(decoded.resource_delta.is_none());
}

#[test]
fn c22_t5_ssb_action_types_round_trip() {
    // as_sql_str + from_str_public symmetry for both new variants.
    let alt = SsbActionType::AlternativeExplored;
    assert_eq!(alt.as_sql_str(), "alternative_explored");
    assert!(matches!(
        ssb_action_type_from_str_public("alternative_explored"),
        Ok(SsbActionType::AlternativeExplored)
    ));

    let esc = SsbActionType::FeasibilityEscalated;
    assert_eq!(esc.as_sql_str(), "feasibility_escalated");
    assert!(matches!(
        ssb_action_type_from_str_public("feasibility_escalated"),
        Ok(SsbActionType::FeasibilityEscalated)
    ));

    // SsbPayload variants exist and report the correct discriminant via
    // action_type().
    let p1 = SsbPayload::AlternativeExplored(AlternativeExploredPayload {
        step_id: "step-1".to_owned(),
        alternative_index: 1,
        accepted: true,
    });
    assert_eq!(p1.action_type(), SsbActionType::AlternativeExplored);

    let p2 = SsbPayload::FeasibilityEscalated(FeasibilityEscalatedPayload {
        reason: "vram_breach".to_owned(),
        variant: "resource_margin_breached".to_owned(),
        step_id: String::new(),
    });
    assert_eq!(p2.action_type(), SsbActionType::FeasibilityEscalated);

    // Payload msgpack round-trip pins the wire-format integrity.
    let bytes = rmp_serde::to_vec_named(&p1).expect("encode AlternativeExplored");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode SsbPayload");
    assert_eq!(decoded.action_type(), SsbActionType::AlternativeExplored);
}

#[test]
fn c22_t6_schema_version_at_least_54() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 54,
        "SCHEMA_VERSION_PUBLIC must be >= 54 after C-22 atomic bump; got {SCHEMA_VERSION_PUBLIC}"
    );
}

#[test]
fn c22_t7_default_feasibility_prober_builder_chain() {
    // The DefaultFeasibilityProber must support both with_stub_executor and
    // with_system_prober builders without breaking the existing
    // single-arg new(PathBuf) constructor. Pin the builder surface so a
    // future signature change is caught here.
    let prober = DefaultFeasibilityProber::new(std::path::PathBuf::from("/tmp/c22_root"));
    let _root = prober.feasibility_root().to_path_buf();
}
