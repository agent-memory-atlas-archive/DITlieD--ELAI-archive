use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use elai_core::VramTier;
use elai_orchestrator::hardware_catalog::{
    CatalogLaneKind, HardwareModelCatalog, ModelCatalogEntry,
};
use elai_orchestrator::ssb::{
    HardwareModelCatalogEventKind, HardwareModelCatalogEventPayload, SsbPayload,
};
use elai_orchestrator::system_probe::{
    classify_vram_tier, DetectionMethod, GpuInfo, SystemProbeResult,
};
use serde::Deserialize;

const CATALOG_PATH_ENV: &str = "ELAI_HARDWARE_CATALOG_PATH";
const HW14_RESIDUE: &str = "HW-14: prior rejection was user-hardware-specific, not universal; this model is catalogued tier-conditionally and remains informational only.";

#[derive(Debug, Deserialize)]
struct SeedFile {
    models: Vec<SeedModel>,
}

#[derive(Debug, Deserialize)]
struct SeedModel {
    lane: String,
    model_id: String,
}

struct CatalogPathGuard(Option<std::ffi::OsString>);

impl CatalogPathGuard {
    fn install() -> Self {
        Self(std::env::var_os(CATALOG_PATH_ENV))
    }
}

impl Drop for CatalogPathGuard {
    fn drop(&mut self) {
        if let Some(value) = self.0.take() {
            std::env::set_var(CATALOG_PATH_ENV, value);
        } else {
            std::env::remove_var(CATALOG_PATH_ENV);
        }
    }
}

fn fixture(vram_mb: u32) -> SystemProbeResult {
    SystemProbeResult {
        gpu: (vram_mb > 0).then(|| GpuInfo {
            name: "fixture-gpu".to_owned(),
            vram_total_mb: vram_mb,
            vram_free_mb: vram_mb,
            tier: classify_vram_tier(vram_mb),
            driver_version: Some("fixture".to_owned()),
            uuid: None,
        }),
        ram_total_mb: 65_536,
        ram_free_mb: 49_152,
        disk_total_mb: 1_000_000,
        disk_free_mb: 750_000,
        detection_method: DetectionMethod::EnvFallback,
        probed_at_ms: 1_234_567,
    }
}

fn entry<'a>(catalog: &'a HardwareModelCatalog, model_id: &str) -> &'a ModelCatalogEntry {
    catalog
        .lanes
        .iter()
        .flat_map(|lane| lane.entries.iter())
        .find(|entry| entry.model_id == model_id)
        .unwrap_or_else(|| panic!("missing catalog entry {model_id}"))
}

fn eligible_ids(catalog: &HardwareModelCatalog, lane: CatalogLaneKind) -> BTreeSet<String> {
    catalog
        .lanes
        .iter()
        .find(|candidate| candidate.lane == lane)
        .expect("lane exists")
        .entries
        .iter()
        .filter(|entry| entry.eligible)
        .map(|entry| entry.model_id.clone())
        .collect()
}

#[test]
fn j9_hardware_catalog_fixture_matrix_and_contract() {
    std::env::remove_var(CATALOG_PATH_ENV);
    let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/hardware_catalog/model_catalog.toml");
    let seed_text = std::fs::read_to_string(&catalog_path).expect("read model catalog seed");
    let seed: SeedFile = toml::from_str(&seed_text).expect("parse model catalog seed");
    let expected_by_lane: BTreeMap<String, BTreeSet<String>> = seed.models.into_iter().fold(
        BTreeMap::new(),
        |mut lanes, model| {
            lanes
                .entry(model.lane)
                .or_default()
                .insert(model.model_id);
            lanes
        },
    );

    let fixture_matrix = [
        (8_192, VramTier::Vram8Gb, [3, 3, 2, 4]),
        (12_288, VramTier::Vram12Gb, [3, 4, 2, 4]),
        (16_384, VramTier::Vram16Gb, [3, 4, 2, 4]),
        (24_576, VramTier::Vram24Gb, [5, 6, 2, 4]),
        (49_152, VramTier::Vram48Gb, [5, 9, 2, 4]),
    ];
    let apple_silicon = cfg!(all(target_os = "macos", target_arch = "aarch64"));

    for (vram_mb, expected_tier, mut expected_counts) in fixture_matrix {
        if apple_silicon {
            expected_counts[1] += 1;
        }
        let probe = fixture(vram_mb);
        let catalog = HardwareModelCatalog::build(&probe);
        assert_eq!(catalog.tier, expected_tier);
        assert_eq!(catalog.probe, probe);
        assert!(catalog.lanes_unavailable.is_none());
        assert_eq!(catalog.lanes.len(), 4);
        assert!(!catalog.seed_blake3.is_empty());
        assert_eq!(catalog, HardwareModelCatalog::build(&probe));

        for lane in &catalog.lanes {
            let actual: BTreeSet<String> = lane
                .entries
                .iter()
                .map(|entry| entry.model_id.clone())
                .collect();
            assert_eq!(actual, expected_by_lane[lane.lane.as_str()]);
            assert!(lane
                .entries
                .windows(2)
                .all(|pair| pair[0].eligible >= pair[1].eligible));
            for row in &lane.entries {
                if !row.eligible {
                    assert!(matches!(
                        row.reason.as_deref(),
                        Some("vram" | "backend_missing" | "platform")
                    ));
                }
                assert!(!row.source_tag.is_empty());
                for fit in &row.fits {
                    if !fit.eligible {
                        assert!(matches!(
                            fit.reason.as_deref(),
                            Some("vram" | "backend_missing" | "platform")
                        ));
                    }
                }
            }
        }

        let actual_counts = [
            eligible_ids(&catalog, CatalogLaneKind::Coding).len(),
            eligible_ids(&catalog, CatalogLaneKind::Vision).len(),
            eligible_ids(&catalog, CatalogLaneKind::Ocr).len(),
            eligible_ids(&catalog, CatalogLaneKind::EmbeddingReranker).len(),
        ];
        assert_eq!(actual_counts, expected_counts.map(|count| count as usize));
    }

    let twelve = HardwareModelCatalog::build(&fixture(12_288));
    assert!(!entry(&twelve, "Gemma-4").eligible);
    assert!(!entry(&twelve, "Qwen3-32B").eligible);
    assert_eq!(entry(&twelve, "Gemma-4").reason.as_deref(), Some("vram"));
    assert_eq!(
        entry(&twelve, "dots.ocr").reason.as_deref(),
        Some("backend_missing")
    );
    assert_eq!(
        entry(&twelve, "Qwen3-VL-Embedding").reason.as_deref(),
        Some("backend_missing")
    );

    let forty_eight = HardwareModelCatalog::build(&fixture(49_152));
    for model_id in ["Gemma-4", "Qwen3-32B"] {
        let row = entry(&forty_eight, model_id);
        assert!(row.eligible, "{model_id} must be tier-conditionally eligible");
        assert_eq!(row.residue_note.as_deref(), Some(HW14_RESIDUE));
    }

    let zero = HardwareModelCatalog::build(&fixture(0));
    assert_eq!(zero.tier, VramTier::Vram8Gb);
    assert!(zero
        .residue_notes
        .iter()
        .any(|note| note.contains("remote API rows only")));
    assert!(zero
        .lanes
        .iter()
        .all(|lane| lane.entries.iter().any(|row| row.eligible)));
    assert!(zero
        .lanes
        .iter()
        .flat_map(|lane| lane.entries.iter())
        .filter(|row| row.eligible)
        .all(|row| row.backend.to_string() == "api"));
    assert!(zero
        .lanes
        .iter()
        .flat_map(|lane| lane.entries.iter())
        .filter(|row| row.backend.to_string() != "api")
        .all(|row| !row.eligible && row.reason.is_some()));

    let cloud_mode = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../orchestrator/src/cloud_mode.rs"),
    )
    .expect("read cloud_mode.rs");
    let escalation_policy = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../providers/src/runtime/escalation_policy.rs"),
    )
    .expect("read escalation_policy.rs");
    assert!(!cloud_mode.contains("hardware_catalog"));
    assert!(!escalation_policy.contains("hardware_catalog"));

    let wire_payload = SsbPayload::HardwareModelCatalogEvent(
        HardwareModelCatalogEventPayload {
            kind: HardwareModelCatalogEventKind::CatalogBuilt,
            detected_tier: "48GB".to_owned(),
            vram_total_mb: 49_152,
            seed_blake3: forty_eight.seed_blake3.clone(),
            lanes_unavailable: None,
            eligible_count: 0,
            lanes: Vec::new(),
            residue_notes: forty_eight.residue_notes.clone(),
        },
    );
    assert_eq!(
        wire_payload.action_type().as_sql_str(),
        "hardware_model_catalog_event"
    );
    let wire_bytes = rmp_serde::to_vec_named(&wire_payload).expect("serialize catalog event");
    let decoded: SsbPayload = rmp_serde::from_slice(&wire_bytes).expect("decode catalog event");
    assert_eq!(decoded, wire_payload);

    let _guard = CatalogPathGuard::install();
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing-model-catalog.toml");
    std::env::set_var(CATALOG_PATH_ENV, &missing);
    let missing_catalog = HardwareModelCatalog::build(&fixture(12_288));
    assert!(missing_catalog.lanes.is_empty());
    assert!(missing_catalog
        .lanes_unavailable
        .as_deref()
        .is_some_and(|reason| reason.starts_with("catalog_read:")));

    let corrupt = temp.path().join("corrupt-model-catalog.toml");
    std::fs::write(&corrupt, "[[models]\ninvalid").expect("write corrupt seed");
    std::env::set_var(CATALOG_PATH_ENV, &corrupt);
    let corrupt_catalog = HardwareModelCatalog::build(&fixture(12_288));
    assert!(corrupt_catalog.lanes.is_empty());
    assert!(corrupt_catalog
        .lanes_unavailable
        .as_deref()
        .is_some_and(|reason| reason.starts_with("catalog_parse:")));
}
