// Contract tests for A-38: PI Incremental Profile Updates.
// Tests written BEFORE implementation (TDD).

use std::collections::HashMap;
use std::path::PathBuf;

use tempfile::TempDir;

use elai_core::project_profile::*;
use elai_knowledge::incremental_profile::{
    ConfigChangeSet, ProfileChangeDetector, BABY_STEP_REFRESH_N,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rust_profile() -> ProjectProfile {
    ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at: "2026-04-15T00:00:00Z".to_owned(),
        project_root: PathBuf::from("/test"),
        build_system: BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        },
        languages: LanguageProfile {
            primary: Language::Rust,
            secondary: Vec::new(),
            distribution: HashMap::new(),
            tree_sitter_coverage: 0.0,
        },
        frameworks: Vec::new(),
        test_profile: TestProfile::default(),
        lint_tools: Vec::new(),
        entry_points: Vec::new(),
        ci: None,
        naming: NamingConventions {
            functions: Convention::Strong("snake_case".to_owned()),
            types: Convention::Strong("PascalCase".to_owned()),
            constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
            modules: Convention::Unknown,
        },
        error_handling: ErrorHandlingProfile {
            pattern: ErrorPattern::ResultBased,
            propagation: PropagationStyle::QuestionMark,
            error_crate: Some("thiserror".to_owned()),
        },
        module_organization: ModuleOrganization {
            pattern: OrgPattern::Layered,
        },
        dep_profile: DependencyProfile::default(),
        project_vision: Some("Test project".to_owned()),
        architecture: Some(ArchitecturePattern::Layered),
        domains: vec![DomainClassification {
            domain: ProjectDomain::WebBackend,
            confidence: 0.9,
            signals: vec!["test".to_owned()],
        }],
        commands: BuildCommandRegistry {
            build: vec![BuildCommand {
                cmd: "cargo build".to_owned(),
                working_dir: PathBuf::from("."),
                env: HashMap::new(),
                timeout_s: 120,
                scope: CommandScope::Full,
                verified: false,
            }],
            test: vec![BuildCommand {
                cmd: "cargo test".to_owned(),
                working_dir: PathBuf::from("."),
                env: HashMap::new(),
                timeout_s: 120,
                scope: CommandScope::Full,
                verified: false,
            }],
            lint: Vec::new(),
            format: Vec::new(),
            run: Vec::new(),
            clean: Vec::new(),
        },
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    }
}

// ---------------------------------------------------------------------------
// Contract: baby_step_threshold_fires_at_n50
// ---------------------------------------------------------------------------

#[test]
fn baby_step_threshold_fires_at_n50() {
    let detector = ProfileChangeDetector::new(PathBuf::from("/test"), 0);
    // Should NOT fire at step 49.
    assert!(!detector.should_refresh_tier1(BABY_STEP_REFRESH_N - 1));
    // Should fire at step 50.
    assert!(detector.should_refresh_tier1(BABY_STEP_REFRESH_N));
    // Should fire at step 100 (2nd cycle).
    assert!(detector.should_refresh_tier1(BABY_STEP_REFRESH_N * 2));
    // Should NOT fire at step 51.
    assert!(!detector.should_refresh_tier1(BABY_STEP_REFRESH_N + 1));
}

// ---------------------------------------------------------------------------
// Contract: schema_version_mismatch_triggers_full_reprofile
// ---------------------------------------------------------------------------

#[test]
fn schema_version_mismatch_triggers_full_reprofile() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Write a profile with schema_version=999 (mismatched).
    let mut profile = rust_profile();
    profile.project_root = root.to_path_buf();
    profile.schema_version = 999;
    let toml_str = elai_knowledge::serialize_profile(&profile).unwrap();
    let profile_path = root.join("project-profile.toml");
    std::fs::write(&profile_path, &toml_str).unwrap();

    // Also create a Cargo.toml so detect_project_profile can work.
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();

    let result = elai_knowledge::incremental_profile::load_or_detect_profile(root);
    assert!(result.is_ok());
    let loaded = result.unwrap();
    // Should have re-profiled with current schema version, not 999.
    assert_eq!(loaded.schema_version, PROFILE_SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// Contract: config_mtime_change_detected
// ---------------------------------------------------------------------------

#[test]
fn config_mtime_change_detected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a Cargo.toml.
    std::fs::write(root.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();

    // Detector with last_profiled_at = 0 should detect the file as changed.
    let detector = ProfileChangeDetector::new(root.to_path_buf(), 0);
    let changes = detector.check_config_changes();
    assert!(changes.is_ok());
    let changeset = changes.unwrap();
    assert!(!changeset.changed_config_files.is_empty());
}

// ---------------------------------------------------------------------------
// Contract: no_change_returns_reuse
// ---------------------------------------------------------------------------

#[test]
fn no_change_returns_reuse() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a Cargo.toml.
    std::fs::write(root.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();

    // Set last_profiled_at to far future (year 2100 in unix seconds).
    let far_future = 4_102_444_800_u64;
    let detector = ProfileChangeDetector::new(root.to_path_buf(), far_future);
    let changes = detector.check_config_changes();
    assert!(changes.is_ok());
    let changeset = changes.unwrap();
    assert!(changeset.changed_config_files.is_empty());
}

// ---------------------------------------------------------------------------
// Contract: tier3_gate_new_framework_detected
// ---------------------------------------------------------------------------

#[test]
fn tier3_gate_new_framework_detected() {
    let detector = ProfileChangeDetector::new(PathBuf::from("/test"), 0);
    let changeset = ConfigChangeSet {
        changed_config_files: vec![PathBuf::from("Cargo.toml")],
        new_framework_detected: true,
        language_mix_shifted: false,
    };
    assert!(detector.should_refresh_tier3(&changeset));
}

// ---------------------------------------------------------------------------
// Contract: tier3_gate_language_mix_shift_gt10pct
// ---------------------------------------------------------------------------

#[test]
fn tier3_gate_language_mix_shift_gt10pct() {
    let detector = ProfileChangeDetector::new(PathBuf::from("/test"), 0);
    let changeset = ConfigChangeSet {
        changed_config_files: vec![PathBuf::from("package.json")],
        new_framework_detected: false,
        language_mix_shifted: true,
    };
    assert!(detector.should_refresh_tier3(&changeset));
}

// ---------------------------------------------------------------------------
// Contract: tier3_gate_no_significant_change_skips
// ---------------------------------------------------------------------------

#[test]
fn tier3_gate_no_significant_change_skips() {
    let detector = ProfileChangeDetector::new(PathBuf::from("/test"), 0);
    let changeset = ConfigChangeSet {
        changed_config_files: vec![PathBuf::from("Cargo.toml")],
        new_framework_detected: false,
        language_mix_shifted: false,
    };
    assert!(!detector.should_refresh_tier3(&changeset));
}

// ---------------------------------------------------------------------------
// Contract: incremental_refresh_updates_profile_fields
// ---------------------------------------------------------------------------

#[test]
fn incremental_refresh_updates_profile_fields() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create Cargo.toml so Tier 1 detection finds something.
    std::fs::write(root.join("Cargo.toml"), "[package]\nname=\"test\"\nversion=\"0.1.0\"\nedition=\"2021\"\n").unwrap();

    let mut profile = rust_profile();
    profile.project_root = root.to_path_buf();

    // Detector with last_profiled_at = 0 should detect changes and refresh.
    let mut detector = ProfileChangeDetector::new(root.to_path_buf(), 0);
    let result = elai_knowledge::incremental_profile::incremental_tier1_refresh(
        root, &mut detector, &mut profile,
    );
    assert!(result.is_ok());
    // Should return Ok(true) since config files changed (mtime > 0).
    assert!(result.unwrap());
}
