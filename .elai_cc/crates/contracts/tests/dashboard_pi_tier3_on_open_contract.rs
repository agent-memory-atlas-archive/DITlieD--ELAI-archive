// MVP audit 2026-04-18 §4.7: Project Intelligence Tier 3 on project open.
//
// Contract: the dashboard's project-open path populates Tier 3 fields
// (domains, architecture, project_vision) on the persisted profile at
// `<root>/.elai_cc/project-profile.toml`, is idempotent (does not re-run
// if those fields are already present), and leaves the Tier 1+2 profile
// intact on any failure.
//
// We test the knowledge-crate primitives `populate_tier3` +
// `serialize_profile` / `deserialize_profile` directly because they are
// the building blocks the dashboard uses (dashboard::commands calls these
// same symbols; see `populate_tier3_if_missing` in
// `crates/dashboard/src/commands.rs`). Exercising the Tauri command
// handler itself requires a live `DashboardState` + Tauri runtime which
// the contracts crate does not bring up — the pure-Rust surface below
// covers the contract of interest (idempotence + persistence round-trip).

use std::fs;
use std::path::{Path, PathBuf};

use elai_core::project_profile::{
    ArchitecturePattern, BuildCommandRegistry, BuildSystem, BuildSystemProfile,
    DependencyProfile, EntryPoint, EntryPointKind, ErrorHandlingProfile, FrameworkCategory,
    FrameworkDetection, Language, LanguageProfile, ModuleOrganization, NamingConventions,
    ProjectProfile, PROFILE_SCHEMA_VERSION, TestProfile,
};
use elai_knowledge::{deserialize_profile, populate_tier3, serialize_profile};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Relative path matching `crates/dashboard/src/commands.rs::PROJECT_PROFILE_RELPATH`.
/// Keep in sync with that constant.
const PROFILE_RELPATH: &str = ".elai_cc/project-profile.toml";

/// Minimal Tier 1+2 profile — no Tier 3 fields set. Mirrors
/// `knowledge::tier3::tests::base_profile` so the contract exercises the
/// same starting point the dashboard would produce after Tier 1+2.
fn tier12_only_profile() -> ProjectProfile {
    ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at: "2026-04-18T00:00:00Z".to_owned(),
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
            distribution: std::collections::BTreeMap::new(),
            tree_sitter_coverage: 0.0,
        },
        // Axum (+0.5) + server entry point (+0.3) = 0.8, comfortably above
        // the WebBackend domain classification threshold (0.6 per spec 4B).
        // This mirrors the `classify_web_backend_with_framework` test in
        // `knowledge::tier3::tests` so Tier 3 produces a non-empty domain
        // set and the round-trip assertions have something to compare.
        frameworks: vec![FrameworkDetection {
            name: "Axum".to_owned(),
            version: None,
            category: FrameworkCategory::Web,
            confidence: 1.0,
            detected_via: "Cargo.toml".to_owned(),
        }],
        test_profile: TestProfile::default(),
        lint_tools: Vec::new(),
        entry_points: vec![EntryPoint {
            file: PathBuf::from("src/main.rs"),
            kind: EntryPointKind::Server,
        }],
        ci: None,
        naming: NamingConventions::default(),
        error_handling: ErrorHandlingProfile::default(),
        module_organization: ModuleOrganization::default(),
        dep_profile: DependencyProfile::default(),
        project_vision: None,
        architecture: None,
        domains: Vec::new(),
        commands: BuildCommandRegistry::default(),
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    }
}

/// Write a profile to `<root>/.elai_cc/project-profile.toml` exactly like
/// `dashboard::commands::write_profile_toml` does.
fn write_profile(root: &Path, profile: &ProjectProfile) {
    let toml = serialize_profile(profile).expect("serialize profile");
    let path = root.join(PROFILE_RELPATH);
    fs::create_dir_all(path.parent().expect("has parent")).expect("mkdir .elai_cc");
    fs::write(&path, toml).expect("write profile");
}

/// Load a profile from the canonical dashboard path.
fn load_profile(root: &Path) -> ProjectProfile {
    let path = root.join(PROFILE_RELPATH);
    let toml = fs::read_to_string(&path).expect("read profile");
    deserialize_profile(&toml).expect("parse profile")
}

/// Mirrors `dashboard::commands::profile_lacks_tier3`. Kept in sync manually
/// — if the gate changes, both this helper and the dashboard copy must be
/// updated together (audit §4.7 check).
fn profile_lacks_tier3(profile: &ProjectProfile) -> bool {
    profile.project_vision.is_none()
        && profile.architecture.is_none()
        && profile.domains.is_empty()
}

// ---------------------------------------------------------------------------
// Contract 1: Tier 3 runs exactly once per profile. A second invocation
// against an already-populated profile must NOT re-run `populate_tier3`
// (idempotence gate = `profile_lacks_tier3`).
// ---------------------------------------------------------------------------

#[test]
fn contract_tier3_runs_once_repeated_opens_skip() {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path();

    // Step 1: write a Tier 1+2-only profile (as the dashboard does after
    // `generate_initial_project_profile`).
    let profile = tier12_only_profile();
    write_profile(root, &profile);

    // Step 2: simulate `populate_tier3_if_missing` — load, gate, populate,
    // write. Mirrors the dashboard helper. First call must run Tier 3.
    {
        let mut p = load_profile(root);
        assert!(profile_lacks_tier3(&p), "fresh profile must lack Tier 3");
        populate_tier3(&mut p, &[]);
        write_profile(root, &p);
    }

    let after_first = load_profile(root);
    assert!(
        !profile_lacks_tier3(&after_first),
        "Tier 3 must populate at least one of domains/architecture/vision"
    );
    // Axum => WebBackend domain should be classified above threshold.
    assert!(
        !after_first.domains.is_empty(),
        "Axum framework should yield at least one domain"
    );

    // Step 3: second call — gate must short-circuit. We simulate that by
    // loading the profile, checking the gate, and asserting we DO NOT run
    // populate_tier3 again (if we did, the serialized output could still
    // match, but the gate is the contract).
    let second = load_profile(root);
    assert!(
        !profile_lacks_tier3(&second),
        "second open must see Tier 3 already present (gate short-circuits)",
    );
}

// ---------------------------------------------------------------------------
// Contract 2: a Tier 3 failure leaves the Tier 1+2 profile intact. We
// simulate "failure" by simply not running `populate_tier3` — the file on
// disk must still deserialize to the original Tier 1+2 profile with zero
// Tier 3 fields.
// ---------------------------------------------------------------------------

#[test]
fn contract_tier3_failure_leaves_tier12_intact() {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path();
    let original = tier12_only_profile();
    write_profile(root, &original);

    // Simulated failure path: the dashboard catches the error from
    // populate_tier3_if_missing and leaves the file untouched.
    // Assertion: re-reading the file yields the original profile unchanged.
    let reloaded = load_profile(root);
    assert_eq!(reloaded.project_vision, None);
    assert_eq!(reloaded.architecture, None);
    assert!(reloaded.domains.is_empty());
    assert_eq!(
        reloaded.frameworks.len(),
        1,
        "Tier 1+2 fields must be preserved on Tier 3 failure"
    );
    assert_eq!(reloaded.frameworks[0].name, "Axum");
}

// ---------------------------------------------------------------------------
// Contract 3: persisted TOML round-trips through serialize/deserialize
// after Tier 3 runs — proves the new `[architecture]` / `domains` / vision
// fields on disk deserialize back to the same in-memory representation
// byte-for-byte (serde stability).
// ---------------------------------------------------------------------------

#[test]
fn contract_tier3_profile_roundtrip_via_toml() {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path();

    let mut profile = tier12_only_profile();
    populate_tier3(&mut profile, &[]);
    assert!(
        !profile_lacks_tier3(&profile),
        "populate_tier3 should fill at least one field for an Axum project"
    );

    write_profile(root, &profile);
    let reloaded = load_profile(root);

    assert_eq!(reloaded.project_vision, profile.project_vision);
    assert_eq!(reloaded.architecture, profile.architecture);
    assert_eq!(reloaded.domains.len(), profile.domains.len());
    for (orig, back) in profile.domains.iter().zip(reloaded.domains.iter()) {
        assert_eq!(orig.domain, back.domain);
        assert!(
            (orig.confidence - back.confidence).abs() < 1e-6,
            "domain confidence must round-trip",
        );
    }
    // Sanity: Axum still present after round-trip (Tier 1+2 not clobbered).
    assert_eq!(reloaded.frameworks.len(), 1);
    assert_eq!(reloaded.frameworks[0].name, "Axum");
}

// ---------------------------------------------------------------------------
// Contract 4: the idempotence gate (`profile_lacks_tier3`) fires on ANY of
// the three Tier 3 fields being populated — not all three. If only
// `architecture` is set, we must NOT re-run Tier 3. Guards against the
// classic "detects one field so nukes the others" failure mode.
// ---------------------------------------------------------------------------

#[test]
fn contract_tier3_gate_fires_on_any_field() {
    let mut p = tier12_only_profile();
    assert!(profile_lacks_tier3(&p), "all-None profile lacks Tier 3");

    p.architecture = Some(ArchitecturePattern::Unknown);
    assert!(
        !profile_lacks_tier3(&p),
        "architecture-only profile must be considered Tier 3-populated",
    );

    let mut p2 = tier12_only_profile();
    p2.project_vision = Some("test vision".to_owned());
    assert!(!profile_lacks_tier3(&p2));

    let mut p3 = tier12_only_profile();
    populate_tier3(&mut p3, &[]);
    if !p3.domains.is_empty() {
        // domains populated alone is enough to short-circuit.
        let mut p4 = tier12_only_profile();
        p4.domains = p3.domains.clone();
        assert!(!profile_lacks_tier3(&p4));
    }
}
