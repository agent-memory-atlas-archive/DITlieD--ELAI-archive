// A-37: VRAM tier detection — wraps discover_gpus() to classify hardware tier.

use elai_core::VramTier;

/// VRAM thresholds (bytes) for tier classification.
/// Each tier boundary is the minimum total VRAM to qualify for that tier.
const VRAM_THRESHOLD_48GB: u64 = 40 * 1024 * 1024 * 1024; // 40 GiB
const VRAM_THRESHOLD_24GB: u64 = 20 * 1024 * 1024 * 1024; // 20 GiB
const VRAM_THRESHOLD_16GB: u64 = 14 * 1024 * 1024 * 1024; // 14 GiB
const VRAM_THRESHOLD_12GB: u64 = 10 * 1024 * 1024 * 1024; // 10 GiB
// Below 10 GiB = 8GB tier

/// Environment variable override for VRAM tier (fallback when NVML unavailable).
/// Set to "8", "12", "16", "24", "48", or "cloud".
const VRAM_OVERRIDE_ENV: &str = "ELAI_VRAM_OVERRIDE";

/// Error from VRAM tier detection.
#[derive(Debug, thiserror::Error)]
pub enum VramDetectionError {
    #[error("GPU discovery failed: {0}")]
    Discovery(#[from] crate::model_placer::GpuDiscoveryError),

    #[error("invalid ELAI_VRAM_OVERRIDE value: {0}")]
    InvalidOverride(String),
}

/// Classify total VRAM bytes into a tier.
fn classify_vram_bytes(total_bytes: u64) -> VramTier {
    if total_bytes >= VRAM_THRESHOLD_48GB {
        VramTier::Vram48Gb
    } else if total_bytes >= VRAM_THRESHOLD_24GB {
        VramTier::Vram24Gb
    } else if total_bytes >= VRAM_THRESHOLD_16GB {
        VramTier::Vram16Gb
    } else if total_bytes >= VRAM_THRESHOLD_12GB {
        VramTier::Vram12Gb
    } else {
        VramTier::Vram8Gb
    }
}

/// Parse the ELAI_VRAM_OVERRIDE env var into a VramTier.
fn parse_vram_override(value: &str) -> Result<VramTier, VramDetectionError> {
    match value.trim() {
        "8" => Ok(VramTier::Vram8Gb),
        "12" => Ok(VramTier::Vram12Gb),
        "16" => Ok(VramTier::Vram16Gb),
        "24" => Ok(VramTier::Vram24Gb),
        "48" => Ok(VramTier::Vram48Gb),
        "cloud" => Ok(VramTier::Cloud),
        other => Err(VramDetectionError::InvalidOverride(other.to_owned())),
    }
}

/// Detect the VRAM tier for this machine.
///
/// Detection order:
/// 1. `ELAI_VRAM_OVERRIDE` env var (explicit user override)
/// 2. NVML via `discover_gpus()` (requires feature="vllm" and NVIDIA driver)
/// 3. Conservative fallback: `Vram8Gb` (lowest tier = safest capacity)
///
/// On machines without NVML (no NVIDIA GPU or vllm feature disabled), the function
/// returns `Vram8Gb` unless the user sets `ELAI_VRAM_OVERRIDE`.
pub fn detect_vram_tier() -> Result<VramTier, VramDetectionError> {
    // 1. Check env override first
    if let Ok(val) = std::env::var(VRAM_OVERRIDE_ENV) {
        let tier = parse_vram_override(&val)?;
        tracing::info!(tier = ?tier, source = "env_override", "VRAM tier from {VRAM_OVERRIDE_ENV}");
        return Ok(tier);
    }

    // 2. Try NVML discovery
    match crate::discover_gpus() {
        Ok(gpus) if !gpus.is_empty() => {
            // Use the GPU with the most total VRAM (multi-GPU: pick max)
            let max_vram = gpus.iter().map(|g| g.total_vram_bytes).max().unwrap_or(0);
            let tier = classify_vram_bytes(max_vram);
            tracing::info!(
                tier = ?tier,
                max_vram_bytes = max_vram,
                gpu_count = gpus.len(),
                source = "nvml",
                "VRAM tier detected via NVML"
            );
            Ok(tier)
        }
        Ok(_) => {
            // No GPUs found
            tracing::warn!("No GPUs discovered, falling back to Vram8Gb");
            Ok(VramTier::Vram8Gb)
        }
        Err(e) => {
            tracing::warn!(error = %e, "GPU discovery failed, falling back to Vram8Gb");
            Ok(VramTier::Vram8Gb)
        }
    }
}

/// Bytes-per-gibibyte for GB conversion in observational consumers (RouteDecisionEvent
/// cloud_mode emit). Named constant per CLAUDE.md anti-magic-number rule.
const BYTES_PER_GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Detect the raw VRAM available in GiB on this machine (f32 GiB).
///
/// Used by observational consumers (cloud_mode::emit_route_decision_log) that
/// need a numeric VRAM measurement rather than the discrete `VramTier` classification.
///
/// Detection order:
/// 1. NVML via `discover_gpus()` — sums total bytes across the single GPU with the
///    most VRAM (matches `detect_vram_tier`'s multi-GPU policy: pick max).
/// 2. Conservative fallback: `0.0` when no GPUs are visible or NVML errors. This
///    matches the prior hardcoded placeholder and the doctor.rs graceful-fallback
///    pattern (never bubble GPU-discovery errors past the observational layer).
///
/// Returns f32 GiB. Never panics.
pub fn detect_vram_gb() -> f32 {
    match crate::discover_gpus() {
        Ok(gpus) if !gpus.is_empty() => {
            let max_vram = gpus.iter().map(|g| g.total_vram_bytes).max().unwrap_or(0);
            // u64 -> f64 -> f32: u64::MAX in GiB is ~1.7e10, well within f32 range;
            // f64 intermediate preserves the integer GiB count exactly for any
            // realistic GPU VRAM (<1 EiB).
            let gb_f64 = (max_vram as f64) / BYTES_PER_GIB;
            gb_f64 as f32
        }
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_small_vram() {
        assert_eq!(classify_vram_bytes(6 * 1024 * 1024 * 1024), VramTier::Vram8Gb);
    }

    #[test]
    fn classify_12gb_vram() {
        assert_eq!(classify_vram_bytes(12 * 1024 * 1024 * 1024), VramTier::Vram12Gb);
    }

    #[test]
    fn classify_16gb_vram() {
        assert_eq!(classify_vram_bytes(16 * 1024 * 1024 * 1024), VramTier::Vram16Gb);
    }

    #[test]
    fn classify_24gb_vram() {
        assert_eq!(classify_vram_bytes(24 * 1024 * 1024 * 1024), VramTier::Vram24Gb);
    }

    #[test]
    fn classify_48gb_vram() {
        assert_eq!(classify_vram_bytes(48 * 1024 * 1024 * 1024), VramTier::Vram48Gb);
    }

    #[test]
    fn parse_override_valid() {
        assert_eq!(parse_vram_override("12").unwrap(), VramTier::Vram12Gb);
        assert_eq!(parse_vram_override("cloud").unwrap(), VramTier::Cloud);
        assert_eq!(parse_vram_override(" 24 ").unwrap(), VramTier::Vram24Gb);
    }

    #[test]
    fn parse_override_invalid() {
        assert!(parse_vram_override("99").is_err());
        assert!(parse_vram_override("").is_err());
    }

    // Cold-wiring (s-81344): the existing classify_* tests use round VRAM
    // values comfortably inside each tier; this locks the EXACT `>=` threshold
    // boundaries (40/20/14/10 GiB) so a future off-by-one in a comparison
    // operator is caught.
    #[test]
    fn classify_threshold_boundaries_are_inclusive() {
        const GIB: u64 = 1024 * 1024 * 1024;
        // Each threshold is the inclusive lower bound for its tier; one byte
        // below drops to the next lower tier.
        assert_eq!(classify_vram_bytes(40 * GIB), VramTier::Vram48Gb);
        assert_eq!(classify_vram_bytes(40 * GIB - 1), VramTier::Vram24Gb);
        assert_eq!(classify_vram_bytes(20 * GIB), VramTier::Vram24Gb);
        assert_eq!(classify_vram_bytes(20 * GIB - 1), VramTier::Vram16Gb);
        assert_eq!(classify_vram_bytes(14 * GIB), VramTier::Vram16Gb);
        assert_eq!(classify_vram_bytes(14 * GIB - 1), VramTier::Vram12Gb);
        assert_eq!(classify_vram_bytes(10 * GIB), VramTier::Vram12Gb);
        assert_eq!(classify_vram_bytes(10 * GIB - 1), VramTier::Vram8Gb);
    }
}
