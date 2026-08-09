//! N-1 contract: profile-sink-writing bench handlers and the measurement spine
//! registry must describe exactly the same families.
//!
//! This source scan deliberately has a narrow uncovered residue: sinks written
//! through a type alias or a re-export under a different name are not detected;
//! sinks written outside `crates/cli/src/bench_*` are not detected; and a type
//! name inside a block comment or string literal over-detects (failing closed,
//! which is the safe direction).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const PROFILE_SINK_TYPES: [&str; 4] = [
    "ModelCapabilityProfile",
    "CalibrationArtifact",
    "FillDegradationCurve",
    "PositionalValueCurve",
];

fn handler_sources() -> Vec<(String, PathBuf)> {
    let handler_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut handlers = Vec::new();

    for entry in fs::read_dir(&handler_dir).expect("CLI source directory should be readable") {
        let entry = entry.expect("CLI source entry should be readable");
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if path.is_file() && file_name.starts_with("bench_") && path.extension().is_some_and(|ext| ext == "rs") {
            handlers.push((file_name, path));
        } else if path.is_dir() && file_name.starts_with("bench_") {
            let module = path.join("mod.rs");
            if module.is_file() {
                handlers.push((format!("{file_name}/mod.rs"), module));
            }
        }
    }
    handlers.sort_by(|left, right| left.0.cmp(&right.0));
    handlers
}

fn family_for(handler: &str) -> String {
    let stem = handler.strip_suffix("/mod.rs").unwrap_or(handler);
    let stem = stem.strip_suffix(".rs").unwrap_or(stem);
    stem.strip_prefix("bench_")
        .expect("bench handler must have bench_ prefix")
        .replace('_', "-")
}

fn sink_writing_families() -> Vec<(String, String)> {
    handler_sources()
        .into_iter()
        .filter_map(|(handler, path)| {
            let source = fs::read_to_string(&path).expect("bench handler should be readable");
            let writes_sink = source.lines().filter(|line| !line.trim_start().starts_with("//"))
                .any(|line| PROFILE_SINK_TYPES.iter().any(|type_name| line.contains(type_name)));
            writes_sink.then(|| (handler.clone(), family_for(&handler)))
        })
        .collect()
}

fn registry_families() -> BTreeSet<&'static str> {
    elai_orchestrator::measurement_spine::spine_entries()
        .iter()
        .map(|entry| entry.bench_family)
        .collect()
}

#[test]
fn handler_scan_is_not_vacuous() {
    let handlers: BTreeSet<_> = handler_sources().into_iter().map(|(name, _)| name).collect();
    assert!(!handlers.is_empty(), "bench handler scan found no handler sources");
    assert!(handlers.contains("bench_context_fill.rs"),
        "bench handler scan missed the known handler bench_context_fill.rs");
}

#[test]
fn every_profile_sink_handler_has_one_spine_row_and_no_row_is_ghost() {
    let sink_handlers = sink_writing_families();
    let registry = registry_families();

    for (handler, family) in &sink_handlers {
        assert!(registry.contains(family.as_str()),
            "sink-writing handler {handler} should have a measurement-spine row for family {family}");
    }

    let sink_families: BTreeSet<_> = sink_handlers.iter().map(|(_, family)| family.as_str()).collect();
    for family in registry {
        assert!(sink_families.contains(family),
            "measurement-spine row {family} has no sink-writing bench handler");
    }
}
