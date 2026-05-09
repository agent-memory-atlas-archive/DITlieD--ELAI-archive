// A-38: Incremental profile update system.
// Change detection via mtime checks, per-tier incremental refresh,
// schema_version tracking, baby step counter for Tier 1 heartbeat.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use elai_core::project_profile::{ProjectProfile, ProjectProfileError, PROFILE_SCHEMA_VERSION};
use elai_core::{ClockProvider as _, OsClock};

use crate::project_profile::{deserialize_profile, detect_project_profile};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of baby steps between lightweight Tier 1 refreshes.
pub const BABY_STEP_REFRESH_N: u32 = 50;

/// Tier 3 gate: minimum language distribution shift percentage to trigger re-vision.
pub const TIER3_LANGUAGE_SHIFT_PCT: u32 = 10;

/// Profile filename within the project data directory.
const PROFILE_FILENAME: &str = "project-profile.toml";

/// Config files checked for mtime changes during incremental detection.
/// These are the build system / dependency config files that indicate
/// project structure may have changed.
const TIER1_CONFIG_GLOBS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "setup.py",
    "go.mod",
    "CMakeLists.txt",
    "Makefile",
    "build.gradle",
    "build.gradle.kts",
    "pom.xml",
    "mix.exs",
    "Gemfile",
    "composer.json",
    "deno.json",
    "bun.lockb",
    ".github/workflows",
    ".gitlab-ci.yml",
    "Jenkinsfile",
];

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from incremental profile detection.
#[derive(Debug, thiserror::Error)]
pub enum ProfileDetectError {
    #[error("IO error during change detection: {0}")]
    Io(#[from] std::io::Error),

    #[error("profile detection failed: {0}")]
    Detection(#[from] ProjectProfileError),

    #[error("{0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// ConfigChangeSet
// ---------------------------------------------------------------------------

/// Result of checking config file mtimes against last profiling timestamp.
#[derive(Debug, Clone)]
pub struct ConfigChangeSet {
    /// Config files whose mtime is newer than last_profiled_at.
    pub changed_config_files: Vec<PathBuf>,
    /// True if a new framework config file appeared (e.g. new Cargo.toml in a subdir).
    pub new_framework_detected: bool,
    /// True if language distribution shifted more than TIER3_LANGUAGE_SHIFT_PCT.
    pub language_mix_shifted: bool,
}

impl ConfigChangeSet {
    /// True if any config file changed.
    pub fn has_changes(&self) -> bool {
        !self.changed_config_files.is_empty()
    }
}

// ---------------------------------------------------------------------------
// ProfileChangeDetector
// ---------------------------------------------------------------------------

/// Detects config file changes since last profiling run.
/// Created per-refresh call, not persisted across turns.
#[derive(Debug)]
pub struct ProfileChangeDetector {
    project_root: PathBuf,
    /// Unix timestamp (seconds) of the last successful profiling run.
    /// 0 on first run, which treats all files as changed.
    pub last_profiled_at: u64,
}

impl ProfileChangeDetector {
    pub fn new(project_root: PathBuf, last_profiled_at: u64) -> Self {
        Self {
            project_root,
            last_profiled_at,
        }
    }

    /// Check config files for mtime changes since last profiling.
    /// Returns a changeset describing what changed.
    pub fn check_config_changes(&self) -> Result<ConfigChangeSet, ProfileDetectError> {
        let mut changed = Vec::new();

        for &glob_pattern in TIER1_CONFIG_GLOBS {
            let candidate = self.project_root.join(glob_pattern);
            match std::fs::metadata(&candidate) {
                Ok(meta) => {
                    let mtime_secs = meta
                        .modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if mtime_secs > self.last_profiled_at {
                        changed.push(candidate);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // File does not exist — not an error, just skip.
                }
                Err(e) => {
                    return Err(ProfileDetectError::Io(e));
                }
            }
        }

        Ok(ConfigChangeSet {
            changed_config_files: changed,
            new_framework_detected: false,
            language_mix_shifted: false,
        })
    }

    /// Whether baby step counter indicates a Tier 1 refresh is due.
    /// Fires when baby_step_count is a non-zero multiple of BABY_STEP_REFRESH_N.
    pub fn should_refresh_tier1(&self, baby_step_count: u32) -> bool {
        baby_step_count > 0 && baby_step_count % BABY_STEP_REFRESH_N == 0
    }

    /// Whether a Tier 3 (vision/architecture) refresh is warranted.
    /// Fires on new framework detection OR significant language mix shift.
    pub fn should_refresh_tier3(&self, change: &ConfigChangeSet) -> bool {
        change.new_framework_detected || change.language_mix_shifted
    }
}

// ---------------------------------------------------------------------------
// Incremental Tier 1 refresh
// ---------------------------------------------------------------------------

/// Result of an incremental Tier 1 refresh, including Tier 3 gate signals.
#[derive(Debug, Clone)]
pub struct RefreshResult {
    /// True if the profile was modified by the refresh.
    pub modified: bool,
    /// Tier 3 gate: true if a new framework was detected that wasn't in the old profile.
    pub new_framework_detected: bool,
    /// Tier 3 gate: true if the language distribution shifted by more than TIER3_LANGUAGE_SHIFT_PCT.
    pub language_mix_shifted: bool,
}

/// Perform an incremental Tier 1 refresh on the given profile.
///
/// Checks config file mtimes via the detector. If changes found, re-runs
/// Tier 1 detection on the project root and merges results into `current`.
///
/// Returns a `RefreshResult` indicating whether the profile was modified and
/// whether Tier 3 gate signals fired.
pub fn incremental_tier1_refresh(
    root: &Path,
    detector: &mut ProfileChangeDetector,
    current: &mut ProjectProfile,
) -> Result<bool, ProfileDetectError> {
    let changes = detector.check_config_changes()?;
    if !changes.has_changes() {
        return Ok(false);
    }

    // Snapshot old state for Tier 3 gate comparison.
    let old_framework_count = current.frameworks.len();
    let old_primary_lang = current.languages.primary.clone();
    let old_distribution: std::collections::BTreeMap<_, _> = current.languages.distribution.clone();

    // Re-run full Tier 1 detection and merge into current profile.
    let fresh = detect_project_profile(root)?;

    // Check Tier 3 gate: new framework detected?
    let new_framework_detected = fresh.frameworks.len() > old_framework_count;

    // Check Tier 3 gate: language mix shifted significantly?
    let language_mix_shifted = check_language_shift(
        &old_distribution,
        &fresh.languages.distribution,
        TIER3_LANGUAGE_SHIFT_PCT,
    ) || old_primary_lang != fresh.languages.primary;

    // Merge Tier 1 fields only — preserve Tier 2/3 data.
    current.build_system = fresh.build_system;
    current.languages = fresh.languages;
    current.frameworks = fresh.frameworks;
    current.test_profile = fresh.test_profile;
    current.lint_tools = fresh.lint_tools;
    current.entry_points = fresh.entry_points;
    current.ci = fresh.ci;
    current.commands = fresh.commands;

    // Update generated_at to current time.
    let now_secs = OsClock
        .system_now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    current.generated_at = format_unix_timestamp(now_secs);
    detector.last_profiled_at = now_secs;

    // Log Tier 3 gate signals for downstream consumers.
    if new_framework_detected {
        tracing::info!("incremental refresh: new framework detected, Tier 3 gate open");
    }
    if language_mix_shifted {
        tracing::info!("incremental refresh: language mix shifted >{TIER3_LANGUAGE_SHIFT_PCT}%, Tier 3 gate open");
    }

    Ok(true)
}

/// Check if language distribution shifted by more than `threshold_pct` percent
/// in any language between old and new distributions. Distribution maps store
/// file counts per language; we convert to percentages for comparison.
fn check_language_shift(
    old: &std::collections::BTreeMap<elai_core::project_profile::Language, usize>,
    new: &std::collections::BTreeMap<elai_core::project_profile::Language, usize>,
    threshold_pct: u32,
) -> bool {
    let threshold = f64::from(threshold_pct);
    let old_total: usize = old.values().sum();
    let new_total: usize = new.values().sum();

    if old_total == 0 && new_total == 0 {
        return false;
    }

    let mut all_langs: std::collections::HashSet<_> = old.keys().collect();
    all_langs.extend(new.keys());

    for lang in all_langs {
        let old_pct = if old_total > 0 {
            (*old.get(lang).unwrap_or(&0) as f64 / old_total as f64) * 100.0
        } else {
            0.0
        };
        let new_pct = if new_total > 0 {
            (*new.get(lang).unwrap_or(&0) as f64 / new_total as f64) * 100.0
        } else {
            0.0
        };
        if (new_pct - old_pct).abs() > threshold {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// load_or_detect_profile
// ---------------------------------------------------------------------------

/// Load a cached profile from project-profile.toml, or detect from scratch.
///
/// Checks schema_version: if mismatched against PROFILE_SCHEMA_VERSION,
/// falls back to full detect_project_profile.
/// If the cached file doesn't exist, runs full detection.
pub fn load_or_detect_profile(project_dir: &Path) -> Result<ProjectProfile, ProjectProfileError> {
    let profile_path = project_dir.join(PROFILE_FILENAME);

    // Avoid TOCTOU: read directly and handle NotFound as fallthrough.
    match std::fs::read_to_string(&profile_path) {
        Ok(contents) => {
            match deserialize_profile(&contents) {
                Ok(profile) if profile.schema_version == PROFILE_SCHEMA_VERSION => {
                    return Ok(profile);
                }
                Ok(_) => {
                    // Schema version mismatch — full re-profile.
                }
                Err(_) => {
                    // Parse failure — full re-profile.
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File absent — fall through to full detection.
        }
        Err(e) => {
            return Err(ProjectProfileError::Io(e));
        }
    }

    detect_project_profile(project_dir)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a unix timestamp as an ISO 8601 string (UTC).
fn format_unix_timestamp(secs: u64) -> String {
    // Simple formatting without external deps.
    // Produces "YYYY-MM-DDTHH:MM:SSZ" format.
    const SECS_PER_MIN: u64 = 60;
    const SECS_PER_HOUR: u64 = 3600;
    const SECS_PER_DAY: u64 = 86400;

    let days = secs / SECS_PER_DAY;
    let time_of_day = secs % SECS_PER_DAY;
    let hours = time_of_day / SECS_PER_HOUR;
    let minutes = (time_of_day % SECS_PER_HOUR) / SECS_PER_MIN;
    let seconds = time_of_day % SECS_PER_MIN;

    // Convert days since epoch to y/m/d (civil calendar).
    let (year, month, day) = days_to_civil(days);

    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

/// Convert days since Unix epoch to (year, month, day).
/// Positive-epoch-only variant of the civil_from_days algorithm.
/// Only valid for dates after 1970-01-01 (input >= 0).
fn days_to_civil(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baby_step_threshold() {
        let det = ProfileChangeDetector::new(PathBuf::from("/x"), 0);
        assert!(!det.should_refresh_tier1(0));
        assert!(!det.should_refresh_tier1(1));
        assert!(!det.should_refresh_tier1(49));
        assert!(det.should_refresh_tier1(50));
        assert!(det.should_refresh_tier1(100));
    }

    #[test]
    fn test_tier3_gate() {
        let det = ProfileChangeDetector::new(PathBuf::from("/x"), 0);
        let no_change = ConfigChangeSet {
            changed_config_files: vec![],
            new_framework_detected: false,
            language_mix_shifted: false,
        };
        assert!(!det.should_refresh_tier3(&no_change));

        let framework = ConfigChangeSet {
            changed_config_files: vec![],
            new_framework_detected: true,
            language_mix_shifted: false,
        };
        assert!(det.should_refresh_tier3(&framework));
    }

    #[test]
    fn test_format_unix_timestamp() {
        // 2026-04-17 00:00:00 UTC = 1776384000
        let ts = format_unix_timestamp(1_776_384_000);
        assert!(ts.starts_with("2026-04-17"), "got: {ts}");
        assert!(ts.ends_with('Z'));
    }

    #[test]
    fn test_config_globs_not_empty() {
        assert!(!TIER1_CONFIG_GLOBS.is_empty());
        assert!(TIER1_CONFIG_GLOBS.contains(&"Cargo.toml"));
        assert!(TIER1_CONFIG_GLOBS.contains(&"package.json"));
    }
}
