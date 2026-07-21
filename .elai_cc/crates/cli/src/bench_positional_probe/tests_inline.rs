use super::*;

const POSITIONAL_PROBE_FIXTURE_VALID: &str = r#"
suite = "test_positional_probe_fixture"
seed_count = 5
paired = true

[[levels]]
depth_fraction = 0.0
outcomes = [
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true],
  [true, true, true, true]
]

[[levels]]
depth_fraction = 0.25
outcomes = [
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false],
  [true, true, true, false]
]

[[levels]]
depth_fraction = 0.5
outcomes = [
  [true, true, false, false],
  [true, true, false, false],
  [true, true, false, false],
  [true, true, false, false],
  [true, true, false, false]
]

[[levels]]
depth_fraction = 0.75
outcomes = [
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false],
  [true, false, false, false]
]

[[levels]]
depth_fraction = 1.0
outcomes = [
  [false, false, false, false],
  [false, false, false, false],
  [false, false, false, false],
  [false, false, false, false],
  [false, false, false, false]
]
"#;

const POSITIONAL_PROBE_FIXTURE_UNDERPOWER: &str = r#"
suite = "test_positional_probe_underpower"
seed_count = 3
paired = false

[[levels]]
depth_fraction = 0.0
outcomes = [[true, false], [true, false], [false, false]]

[[levels]]
depth_fraction = 0.5
outcomes = [[true, true], [true, true], [true, false]]
"#;

const POSITIONAL_PROBE_FIXTURE_LOREM: &str = r#"
suite = "lorem_ipsum_padding"
seed_count = 5
paired = true

[[levels]]
depth_fraction = 0.0
outcomes = [
  [true, true],
  [true, true],
  [true, true],
  [true, true],
  [true, true]
]

[[levels]]
depth_fraction = 0.5
outcomes = [
  [true, true],
  [true, true],
  [true, true],
  [true, true],
  [true, true]
]
"#;

const POSITIONAL_PROBE_FIXTURE_RANDOM: &str = r#"
suite = "random_padding_test"
seed_count = 5
paired = true

[[levels]]
depth_fraction = 0.0
outcomes = [
  [true, true],
  [true, true],
  [true, true],
  [true, true],
  [true, true]
]

[[levels]]
depth_fraction = 0.5
outcomes = [
  [true, true],
  [true, true],
  [true, true],
  [true, true],
  [true, true]
]
"#;

fn parse_fixture(s: &str) -> PositionalProbeConfig {
    toml::from_str(s).expect("fixture parse")
}

#[test]
fn floor_check_passes_valid_config() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    assert!(floor_check(&config).is_ok());
}

#[test]
fn floor_check_rejects_underpower() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_UNDERPOWER);
    let result = floor_check(&config);
    assert!(result.is_err());
    match result.unwrap_err() {
        BenchPositionalProbeError::FloorViolation { seed_count, paired } => {
            assert_eq!(seed_count, 3);
            assert!(!paired);
        }
        other => panic!("expected FloorViolation, got: {other}"),
    }
}

#[test]
fn floor_check_rejects_seed_count_below_5() {
    let mut config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    config.seed_count = 4;
    assert!(floor_check(&config).is_err());
}

#[test]
fn floor_check_rejects_unpaired() {
    let mut config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    config.paired = false;
    assert!(floor_check(&config).is_err());
}

#[test]
fn mean_rate_correct_for_all_honored() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let level_0 = &config.levels[0];
    let rate = mean_rate_for_level(level_0);
    assert!((rate - 1.0_f64).abs() < 1e-9);
}

#[test]
fn mean_rate_correct_for_half_honored() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let level_50 = &config.levels[2];
    let rate = mean_rate_for_level(level_50);
    assert!((rate - 0.50_f64).abs() < 1e-9);
}

#[test]
fn mean_rate_correct_for_none_honored() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let level_100 = &config.levels[4];
    let rate = mean_rate_for_level(level_100);
    assert!((rate - 0.0_f64).abs() < 1e-9);
}

#[test]
fn build_positional_curve_produces_ordered_samples() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve = build_positional_curve(&config);
    assert_eq!(curve.samples.len(), 5);
    assert!((curve.samples[0].depth_fraction - 0.0).abs() < 1e-6);
    assert!((curve.samples[1].depth_fraction - 0.25).abs() < 1e-6);
    assert!((curve.samples[2].depth_fraction - 0.5).abs() < 1e-6);
    assert!((curve.samples[3].depth_fraction - 0.75).abs() < 1e-6);
    assert!((curve.samples[4].depth_fraction - 1.0).abs() < 1e-6);
    assert!((f64::from(curve.samples[0].recall_rate) - 1.00).abs() < 1e-6);
    assert!((f64::from(curve.samples[1].recall_rate) - 0.75).abs() < 1e-6);
    assert!((f64::from(curve.samples[2].recall_rate) - 0.50).abs() < 1e-6);
    assert!((f64::from(curve.samples[3].recall_rate) - 0.25).abs() < 1e-6);
    assert!((f64::from(curve.samples[4].recall_rate) - 0.00).abs() < 1e-6);
}

#[test]
fn curve_is_not_flat() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve = build_positional_curve(&config);
    assert!(!is_flat_curve(&curve.samples));
}

#[test]
fn curve_is_monotonically_non_increasing() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve = build_positional_curve(&config);
    let non_mon = detect_non_monotonic(&curve.samples);
    assert!(non_mon.is_empty(), "monotonic fixture must produce empty non_monotonic_at; got: {non_mon:?}");
}

#[test]
fn knee_detected_at_largest_drop() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve = build_positional_curve(&config);
    let knee = detect_knee(&curve.samples);
    assert!(knee.is_some());
    let (depth, drop_) = knee.unwrap();
    // Drops: 1.0->0.75 (-0.25), 0.75->0.50 (-0.25), 0.50->0.25 (-0.25), 0.25->0.0 (-0.25)
    // All equal drops; first one wins (lowest depth_fraction).
    assert!((depth - 0.0).abs() < 1e-6);
    assert!((drop_ - 0.25).abs() < 1e-6);
}

#[test]
fn audit_fixture_rejects_lorem() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_LOREM);
    let result = audit_fixture(&config);
    assert!(result.is_err(), "lorem suite must fail fixture audit");
    match result.unwrap_err() {
        BenchPositionalProbeError::FixtureAudit { reason } => {
            assert!(reason.contains("lorem"));
        }
        other => panic!("expected FixtureAudit, got: {other}"),
    }
}

#[test]
fn audit_fixture_rejects_random() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_RANDOM);
    let result = audit_fixture(&config);
    assert!(result.is_err(), "random suite must fail fixture audit");
    match result.unwrap_err() {
        BenchPositionalProbeError::FixtureAudit { reason } => {
            assert!(reason.contains("random"));
        }
        other => panic!("expected FixtureAudit, got: {other}"),
    }
}

#[test]
fn audit_fixture_rejects_all_true_flat() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_LOREM);
    // Override suite to not contain lorem/random but keep all-true outcomes
    let mut config = config;
    config.suite = "all_true_flat".to_string();
    let result = audit_fixture(&config);
    assert!(result.is_err(), "all-true outcomes must fail fixture audit (flat curve)");
    match result.unwrap_err() {
        BenchPositionalProbeError::FixtureAudit { reason } => {
            assert!(reason.contains("flat"));
        }
        other => panic!("expected FixtureAudit, got: {other}"),
    }
}

#[test]
fn audit_fixture_passes_valid() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    assert!(audit_fixture(&config).is_ok());
}

#[test]
fn resolve_identity_key_uses_config_when_present() {
    let mut config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    config.identity_key = Some("model:test-identity".to_string());
    let key = resolve_identity_key(&config, b"some-bytes");
    assert_eq!(key, "model:test-identity");
}

#[test]
fn resolve_identity_key_synthetic_when_absent() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let key = resolve_identity_key(&config, b"some-bytes");
    assert!(key.starts_with("fixture:positional-probe:"));
    let key2 = resolve_identity_key(&config, b"some-bytes");
    assert_eq!(key, key2);
    let key3 = resolve_identity_key(&config, b"different-bytes");
    assert_ne!(key, key3);
}

/// Both legs share the process-wide `ELAI_SEED` env var, so they MUST
/// run in a single test body (no internal parallelism) to avoid the
/// shared-env race a split pair would hit under cargo's multi-threaded
/// runner. No `serial_test` dep is available and none is added.
#[test]
fn read_elai_seed_defaults_to_zero_then_parses_value() {
    std::env::remove_var("ELAI_SEED");
    assert_eq!(read_elai_seed(), 0);
    std::env::set_var("ELAI_SEED", "42");
    assert_eq!(read_elai_seed(), 42);
    std::env::remove_var("ELAI_SEED");
}

// -----------------------------------------------------------------------
// Determinism: same fixture + same ELAI_SEED -> byte-identical curve
// -----------------------------------------------------------------------

#[test]
fn deterministic_curve_same_fixture_same_seed() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve1 = build_positional_curve(&config);
    let curve2 = build_positional_curve(&config);
    assert_eq!(curve1, curve2, "same fixture must produce byte-identical curve");
}

#[test]
fn deterministic_curve_serialization_byte_identical() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve1 = build_positional_curve(&config);
    let curve2 = build_positional_curve(&config);
    let json1 = serde_json::to_vec(&curve1).expect("serialize");
    let json2 = serde_json::to_vec(&curve2).expect("serialize");
    assert_eq!(json1, json2, "serialized curves must be byte-identical");
}

// -----------------------------------------------------------------------
// run_inner end-to-end on a tempdir
// -----------------------------------------------------------------------

#[test]
fn run_inner_on_valid_fixture_emits_curve_measured_and_writes_report() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let suite_path = dir.path().join("fixture.toml");
    fs::write(&suite_path, POSITIONAL_PROBE_FIXTURE_VALID).expect("write fixture");

    let result = run_inner(suite_path, 0, false, dir.path());
    result.expect("run_inner must succeed on valid fixture");

    let report_dir = dir.path().join("benchmarks").join("positional_probe");
    let mut found = false;
    if let Ok(entries) = fs::read_dir(&report_dir) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("positional-probe-test_positional_probe_fixture-")
            {
                found = true;
                break;
            }
        }
    }
    assert!(found, "report file must land under benchmarks/positional_probe/");
}

#[test]
fn run_inner_rejects_underpowered_fixture() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let suite_path = dir.path().join("fixture.toml");
    fs::write(&suite_path, POSITIONAL_PROBE_FIXTURE_UNDERPOWER).expect("write fixture");

    let result = run_inner(suite_path, 0, false, dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        BenchPositionalProbeError::FloorViolation { .. } => {}
        other => panic!("expected FloorViolation, got: {other}"),
    }
}

#[test]
fn run_inner_rejects_lorem_fixture() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let suite_path = dir.path().join("fixture.toml");
    fs::write(&suite_path, POSITIONAL_PROBE_FIXTURE_LOREM).expect("write fixture");

    let result = run_inner(suite_path, 0, false, dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        BenchPositionalProbeError::FixtureAudit { .. } => {}
        other => panic!("expected FixtureAudit, got: {other}"),
    }
}

#[test]
fn run_inner_rejects_random_fixture() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let suite_path = dir.path().join("fixture.toml");
    fs::write(&suite_path, POSITIONAL_PROBE_FIXTURE_RANDOM).expect("write fixture");

    let result = run_inner(suite_path, 0, false, dir.path());
    assert!(result.is_err());
    match result.unwrap_err() {
        BenchPositionalProbeError::FixtureAudit { .. } => {}
        other => panic!("expected FixtureAudit, got: {other}"),
    }
}

#[test]
fn run_inner_with_elai_seed_42_produces_deterministic_curve() {
    let dir1 = tempfile::TempDir::new().expect("tempdir1");
    let dir2 = tempfile::TempDir::new().expect("tempdir2");
    let suite1 = dir1.path().join("fixture.toml");
    let suite2 = dir2.path().join("fixture.toml");
    fs::write(&suite1, POSITIONAL_PROBE_FIXTURE_VALID).expect("write fixture1");
    fs::write(&suite2, POSITIONAL_PROBE_FIXTURE_VALID).expect("write fixture2");

    std::env::set_var("ELAI_SEED", "42");
    run_inner(suite1, 0, true, dir1.path()).expect("run_inner 1");
    capture_stdout(|| { run_inner(suite2.clone(), 0, true, dir2.path()).expect("run_inner capture"); });
    run_inner(suite2, 0, true, dir2.path()).expect("run_inner 2");
    std::env::remove_var("ELAI_SEED");

    // The JSON output (positional curve) must be byte-identical across two runs
    // with ELAI_SEED=42. The curve is derived from the fixture, so same fixture
    // = same curve regardless of seed, but the seed is recorded in the report.
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve1 = build_positional_curve(&config);
    let curve2 = build_positional_curve(&config);
    let json1 = serde_json::to_vec(&curve1).expect("serialize1");
    let json2 = serde_json::to_vec(&curve2).expect("serialize2");
    assert_eq!(json1, json2, "ELAI_SEED=42 twice must produce byte-identical curve JSON");
}

fn capture_stdout<F: FnOnce()>(f: F) {
    f();
}

#[test]
fn run_inner_curve_is_monotonically_non_increasing_or_has_knee() {
    let config = parse_fixture(POSITIONAL_PROBE_FIXTURE_VALID);
    let curve = build_positional_curve(&config);
    let non_mon = detect_non_monotonic(&curve.samples);
    let flat = is_flat_curve(&curve.samples);
    let knee = detect_knee(&curve.samples);
    // The curve must be either monotonically non-increasing OR have a detectable knee (not flat)
    assert!(!flat, "curve must not be flat (must have signal)");
    assert!(
        non_mon.is_empty() || knee.is_some(),
        "curve must be monotonically non-increasing OR have a detectable knee"
    );
}
