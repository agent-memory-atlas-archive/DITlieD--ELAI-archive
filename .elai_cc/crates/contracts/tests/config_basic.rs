//! Contract: elai_cc config must parse elai_cc.toml and support
//! msgpack round-trip serialization.
//!
//! RED phase — compiles against existing core config types, fails
//! at runtime because elai_cc.toml is not in the discovery list.

use elai_core::ConfigLoader;

/// ConfigLoader.discover() must include an elai_cc.toml entry.
/// Currently it only discovers .elai.json and settings.json paths.
/// The assertion on source type will fail at runtime.
#[test]
fn config_discovers_elai_cc_toml() {
    let tmp = std::env::temp_dir().join("elai_cc_contracts_test");
    let loader = ConfigLoader::new(
        &tmp,
        tmp.join("config"),
    );

    let entries = loader.discover();

    // CONTRACT: at least one entry must have a path ending in "elai_cc.toml".
    let has_elai_cc_toml = entries
        .iter()
        .any(|e| e.path.to_string_lossy().ends_with("elai_cc.toml"));
    assert!(
        has_elai_cc_toml,
        "CONTRACT: ConfigLoader must discover elai_cc.toml — found: {:?}",
        entries.iter().map(|e| &e.path).collect::<Vec<_>>()
    );
}
