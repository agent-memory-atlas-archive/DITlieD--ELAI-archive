use std::borrow::Cow;
use std::path::PathBuf;

use elai_core::VramTier;
use serde::Deserialize;

use crate::agent_pool::ModelPool;
use crate::context_budget::ModelCapabilityProfile;
use crate::dispatcher::BackendMode;
use crate::system_probe::{classify_vram_tier, SystemProbeResult};

const CATALOG_PATH_ENV: &str = "ELAI_HARDWARE_CATALOG_PATH";
const EMBEDDED_MODEL_CATALOG: &str =
    include_str!("../../../data/hardware_catalog/model_catalog.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogLaneKind {
    Coding,
    Vision,
    Ocr,
    EmbeddingReranker,
}

impl CatalogLaneKind {
    const ORDERED: [Self; 4] = [
        Self::Coding,
        Self::Vision,
        Self::Ocr,
        Self::EmbeddingReranker,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Coding => "coding",
            Self::Vision => "vision",
            Self::Ocr => "ocr",
            Self::EmbeddingReranker => "embedding_reranker",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantizationFit {
    pub quantization: String,
    pub min_vram_mb: u32,
    pub eligible: bool,
    pub reason: Option<String>,
    pub fit_score: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCatalogEntry {
    pub lane: CatalogLaneKind,
    pub model_id: String,
    pub backend: BackendMode,
    pub fits: Vec<QuantizationFit>,
    pub platform_tags: Vec<String>,
    pub source_tag: String,
    pub residue_note: Option<String>,
    pub eligible: bool,
    pub reason: Option<String>,
    pub fit_score: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogLane {
    pub lane: CatalogLaneKind,
    pub entries: Vec<ModelCatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareModelCatalog {
    pub probe: SystemProbeResult,
    pub tier: VramTier,
    pub lanes: Vec<CatalogLane>,
    pub lanes_unavailable: Option<String>,
    pub residue_notes: Vec<String>,
    pub seed_blake3: String,
}

#[derive(Debug, Deserialize)]
struct SeedCatalog {
    #[serde(default)]
    models: Vec<SeedEntry>,
}

#[derive(Debug, Deserialize)]
struct SeedEntry {
    lane: CatalogLaneKind,
    model_id: String,
    backend: BackendMode,
    #[serde(default = "default_backend_available")]
    backend_available: bool,
    #[serde(default)]
    footprints: Vec<SeedFootprint>,
    #[serde(default)]
    platform_tags: Vec<String>,
    source_tag: String,
    residue_note: Option<String>,
    #[serde(default)]
    capability_profile: Option<String>,
    fit_priority: u16,
}

#[derive(Debug, Deserialize)]
struct SeedFootprint {
    quantization: String,
    min_vram_mb: u32,
}

fn default_backend_available() -> bool {
    true
}

struct ProbeModelPool {
    available_vram_bytes: u64,
}

impl ModelPool for ProbeModelPool {
    fn available_vram(&self) -> u64 {
        self.available_vram_bytes
    }
}

impl HardwareModelCatalog {
    pub fn build(probe: &SystemProbeResult) -> Self {
        let vram_total_mb = probe
            .gpu
            .as_ref()
            .map(|gpu| gpu.vram_total_mb)
            .unwrap_or(0);
        let tier = classify_vram_tier(vram_total_mb);
        let mut residue_notes = vec![
            "VRAM estimates exclude runtime-specific KV-cache and context headroom.".to_owned(),
            "Multi-GPU and CPU-offload configurations are not yet modeled; the primary GPU is used."
                .to_owned(),
        ];
        if vram_total_mb == 0 {
            residue_notes.push(
                "No GPU VRAM was reported; eligibility degrades to remote API rows only."
                    .to_owned(),
            );
        }

        let seed_text = match load_seed_text() {
            Ok(seed_text) => seed_text,
            Err(reason) => {
                return Self {
                    probe: probe.clone(),
                    tier,
                    lanes: Vec::new(),
                    lanes_unavailable: Some(reason),
                    residue_notes,
                    seed_blake3: String::new(),
                };
            }
        };
        let seed_blake3 = blake3::hash(seed_text.as_bytes()).to_hex().to_string();
        let seed: SeedCatalog = match toml::from_str(seed_text.as_ref()) {
            Ok(seed) => seed,
            Err(error) => {
                return Self {
                    probe: probe.clone(),
                    tier,
                    lanes: Vec::new(),
                    lanes_unavailable: Some(format!("catalog_parse:{error}")),
                    residue_notes,
                    seed_blake3,
                };
            }
        };

        let pool = ProbeModelPool {
            available_vram_bytes: u64::from(vram_total_mb).saturating_mul(1024 * 1024),
        };
        let mut lanes = Vec::with_capacity(CatalogLaneKind::ORDERED.len());
        for lane_kind in CatalogLaneKind::ORDERED {
            let mut entries: Vec<ModelCatalogEntry> = seed
                .models
                .iter()
                .filter(|entry| entry.lane == lane_kind)
                .map(|entry| resolve_entry(entry, &pool, vram_total_mb))
                .collect();
            entries.sort_by(|left, right| {
                right
                    .eligible
                    .cmp(&left.eligible)
                    .then_with(|| right.fit_score.cmp(&left.fit_score))
                    .then_with(|| left.model_id.cmp(&right.model_id))
            });
            lanes.push(CatalogLane {
                lane: lane_kind,
                entries,
            });
        }

        Self {
            probe: probe.clone(),
            tier,
            lanes,
            lanes_unavailable: None,
            residue_notes,
            seed_blake3,
        }
    }
}

fn load_seed_text() -> Result<Cow<'static, str>, String> {
    if let Some(path) = std::env::var_os(CATALOG_PATH_ENV) {
        let path = PathBuf::from(path);
        return std::fs::read_to_string(&path)
            .map(Cow::Owned)
            .map_err(|error| format!("catalog_read:{}:{error}", path.display()));
    }

    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/hardware_catalog/model_catalog.toml");
    match std::fs::read_to_string(&source_path) {
        Ok(contents) => Ok(Cow::Owned(contents)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(Cow::Borrowed(EMBEDDED_MODEL_CATALOG))
        }
        Err(error) => Err(format!("catalog_read:{}:{error}", source_path.display())),
    }
}

fn resolve_entry(
    seed: &SeedEntry,
    pool: &ProbeModelPool,
    available_vram_mb: u32,
) -> ModelCatalogEntry {
    let platform_supported = platform_supported(&seed.platform_tags);
    let profile_floor_mb = seed
        .capability_profile
        .as_deref()
        .and_then(capability_profile)
        .map(|profile| u32::try_from(profile.model_vram_mb).unwrap_or(u32::MAX))
        .unwrap_or(0);
    let source_confidence = if seed.source_tag.starts_with("[MEASURED]") {
        100_u16
    } else {
        70_u16
    };

    let mut fits = Vec::with_capacity(seed.footprints.len());
    for footprint in &seed.footprints {
        let min_vram_mb = footprint.min_vram_mb.max(profile_floor_mb);
        let reason = if !platform_supported {
            Some("platform".to_owned())
        } else if !seed.backend_available {
            Some("backend_missing".to_owned())
        } else if seed.backend == BackendMode::Api {
            None
        } else {
            let bytes = u64::from(min_vram_mb).saturating_mul(1024 * 1024);
            (!pool.can_fit_llm(bytes)).then(|| "vram".to_owned())
        };
        let eligible = reason.is_none();
        let headroom = if eligible && seed.backend != BackendMode::Api && available_vram_mb > 0 {
            let spare = available_vram_mb.saturating_sub(min_vram_mb);
            ((u64::from(spare) * 100) / u64::from(available_vram_mb)) as u16
        } else if eligible {
            100
        } else {
            0
        };
        let fit_score = if eligible {
            ((u32::from(seed.fit_priority.min(100)) * 65
                + u32::from(headroom.min(100)) * 25
                + u32::from(source_confidence) * 10)
                / 10) as u16
        } else {
            0
        };
        fits.push(QuantizationFit {
            quantization: footprint.quantization.clone(),
            min_vram_mb,
            eligible,
            reason,
            fit_score,
        });
    }
    fits.sort_by(|left, right| {
        right
            .eligible
            .cmp(&left.eligible)
            .then_with(|| right.fit_score.cmp(&left.fit_score))
            .then_with(|| left.min_vram_mb.cmp(&right.min_vram_mb))
            .then_with(|| left.quantization.cmp(&right.quantization))
    });
    let eligible = fits.iter().any(|fit| fit.eligible);
    let fit_score = fits.iter().map(|fit| fit.fit_score).max().unwrap_or(0);
    let reason = if eligible {
        None
    } else if !platform_supported {
        Some("platform".to_owned())
    } else if !seed.backend_available {
        Some("backend_missing".to_owned())
    } else {
        Some("vram".to_owned())
    };

    ModelCatalogEntry {
        lane: seed.lane,
        model_id: seed.model_id.clone(),
        backend: seed.backend,
        fits,
        platform_tags: seed.platform_tags.clone(),
        source_tag: seed.source_tag.clone(),
        residue_note: seed.residue_note.clone(),
        eligible,
        reason,
        fit_score,
    }
}

fn platform_supported(tags: &[String]) -> bool {
    tags.is_empty()
        || tags.iter().any(|tag| match tag.as_str() {
            "apple_silicon" => cfg!(all(target_os = "macos", target_arch = "aarch64")),
            "linux" => cfg!(target_os = "linux"),
            "windows" => cfg!(target_os = "windows"),
            "macos" => cfg!(target_os = "macos"),
            _ => false,
        })
}

fn capability_profile(name: &str) -> Option<ModelCapabilityProfile> {
    match name {
        "micro_8gb" => Some(ModelCapabilityProfile::micro_8gb()),
        "micro_3b_12gb" => Some(ModelCapabilityProfile::micro_3b_12gb()),
        "micro_7b_12gb" => Some(ModelCapabilityProfile::micro_7b_12gb()),
        "large_32b_48gb" => Some(ModelCapabilityProfile::large_32b_48gb()),
        "cloud_200k" => Some(ModelCapabilityProfile::cloud_200k()),
        _ => None,
    }
}
