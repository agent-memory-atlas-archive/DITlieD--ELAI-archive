//! E-51: platform detection helpers for MlxAdapter and LlamaCppAdapter.
//!
//! `detect_mlx_capable()` — returns `NotApple` on non-Darwin, `AppleIntel`
//! on Darwin+x86, `AppleSilicon{..}` on Darwin+arm64 with chip info from
//! `sysctl`. No panics; errors produce conservative safe defaults.
//!
//! `detect_llamacpp_ep()` — returns the best execution provider via env
//! heuristics (no subprocess; subprocess-based detection deferred to E-51.1).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MLX capability
// ---------------------------------------------------------------------------

/// Apple chip family enum for MLX platform detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppleChipFamily {
    M1,
    M2,
    M3,
    M4,
    M5,
    /// Chip family present but not recognised by the current parser.
    Unknown,
}

/// MLX capability on the current host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MlxCapability {
    /// Not a Darwin platform — MLX unavailable.
    NotApple,
    /// Darwin+x86_64 (Intel Mac) — MLX unavailable (requires Metal 3 / arm64).
    AppleIntel,
    /// Darwin+arm64 — MLX available; chip family and GPU core count follow.
    AppleSilicon {
        chip_family: AppleChipFamily,
        gpu_cores: u8,
    },
}

/// Detect MLX capability on the current host.
///
/// Returns `NotApple` on non-macOS, `AppleIntel` on Intel Mac, and
/// `AppleSilicon { .. }` on arm64 Darwin with chip info read from `sysctl`.
/// All subprocess/IO errors produce conservative safe fallbacks — no panics.
#[must_use]
pub fn detect_mlx_capable() -> MlxCapability {
    #[cfg(not(target_os = "macos"))]
    {
        MlxCapability::NotApple
    }

    #[cfg(target_os = "macos")]
    {
        detect_mlx_macos()
    }
}

#[cfg(target_os = "macos")]
fn detect_mlx_macos() -> MlxCapability {
    if std::env::consts::ARCH != "aarch64" {
        return MlxCapability::AppleIntel;
    }

    // Read hw.model to identify chip family.
    let chip_family = read_sysctl("hw.model")
        .map(|model| parse_chip_family(&model))
        .unwrap_or(AppleChipFamily::Unknown);

    // Read GPU core count from hw.perflevel0.gpu_core_count.
    let gpu_cores: u8 = read_sysctl("hw.perflevel0.gpu_core_count")
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0);

    MlxCapability::AppleSilicon {
        chip_family,
        gpu_cores,
    }
}

#[cfg(target_os = "macos")]
fn read_sysctl(key: &str) -> Option<String> {
    let output = std::process::Command::new("sysctl")
        .arg("-n")
        .arg(key)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        tracing::warn!(key, "sysctl query failed");
        None
    }
}

fn parse_chip_family(hw_model: &str) -> AppleChipFamily {
    // hw.model examples: "Mac14,5" (M2 Max), "Mac15,6" (M3 Pro), etc.
    // Apple publishes chip_id via IOKit; hw.model alone isn't deterministic.
    // We supplement with a cpufamily or brand string lookup.
    let model_upper = hw_model.to_uppercase();
    if model_upper.contains("M5") {
        AppleChipFamily::M5
    } else if model_upper.contains("M4") {
        AppleChipFamily::M4
    } else if model_upper.contains("M3") {
        AppleChipFamily::M3
    } else if model_upper.contains("M2") {
        AppleChipFamily::M2
    } else if model_upper.contains("M1") || model_upper.contains("APPLE") {
        // Many early M1 models report "AppleM1" in brand string.
        AppleChipFamily::M1
    } else {
        // Attempt to identify via machdep.cpu.brand_string (supplementary).
        #[cfg(target_os = "macos")]
        {
            if let Some(brand) = read_sysctl("machdep.cpu.brand_string") {
                let brand_upper = brand.to_uppercase();
                if brand_upper.contains("M5") {
                    return AppleChipFamily::M5;
                } else if brand_upper.contains("M4") {
                    return AppleChipFamily::M4;
                } else if brand_upper.contains("M3") {
                    return AppleChipFamily::M3;
                } else if brand_upper.contains("M2") {
                    return AppleChipFamily::M2;
                } else if brand_upper.contains("M1") || brand_upper.contains("APPLE") {
                    return AppleChipFamily::M1;
                }
            }
        }
        AppleChipFamily::Unknown
    }
}

// ---------------------------------------------------------------------------
// LlamaCpp execution provider
// ---------------------------------------------------------------------------

/// Best llama.cpp execution provider on the current host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlamaCppEp {
    /// NVIDIA CUDA (CUDA_HOME or CUDA_VISIBLE_DEVICES detected).
    Cuda,
    /// AMD ROCm / HIP (ROCM_PATH detected).
    Hip,
    /// Apple Metal (macOS platform).
    Metal,
    /// Khronos Vulkan (VULKAN_SDK detected).
    Vulkan,
    /// CPU fallback — no accelerator detected.
    Cpu,
}

/// Detect the best llama.cpp execution provider via environment heuristics.
///
/// Priority: CUDA > HIP > Metal (macOS) > Vulkan > CPU.
/// Uses env-var heuristics only (no subprocess). Subprocess-based `--list-devices`
/// detection is deferred to E-51.1 (requires a pinned llama-server binary).
#[must_use]
pub fn detect_llamacpp_ep() -> LlamaCppEp {
    // CUDA: CUDA_HOME, CUDA_PATH, or CUDA_VISIBLE_DEVICES set.
    if std::env::var("CUDA_HOME").is_ok()
        || std::env::var("CUDA_PATH").is_ok()
        || std::env::var("CUDA_VISIBLE_DEVICES").is_ok()
    {
        return LlamaCppEp::Cuda;
    }

    // ROCm / HIP
    if std::env::var("ROCM_PATH").is_ok() || std::env::var("HIP_PATH").is_ok() {
        return LlamaCppEp::Hip;
    }

    // Apple Metal — macOS platform implies Metal support (M-series and later).
    #[cfg(target_os = "macos")]
    {
        return LlamaCppEp::Metal;
    }

    // Vulkan
    if std::env::var("VULKAN_SDK").is_ok() {
        return LlamaCppEp::Vulkan;
    }

    LlamaCppEp::Cpu
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_mlx_capable_no_panic() {
        // Must return a valid variant on any platform without panicking.
        let _cap = detect_mlx_capable();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_is_not_apple() {
        assert_eq!(detect_mlx_capable(), MlxCapability::NotApple);
    }

    #[test]
    fn detect_llamacpp_ep_no_panic() {
        let _ep = detect_llamacpp_ep();
    }

    #[test]
    fn parse_chip_family_m5() {
        assert_eq!(parse_chip_family("Mac16,5 M5"), AppleChipFamily::M5);
    }

    #[test]
    fn parse_chip_family_unknown_for_garbage() {
        assert_eq!(parse_chip_family("SomePC-123"), AppleChipFamily::Unknown);
    }
}
