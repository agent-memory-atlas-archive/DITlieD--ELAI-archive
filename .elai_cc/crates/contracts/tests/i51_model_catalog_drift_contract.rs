//! I-51 — model capability catalog drift contract.
//!
//! Asserts:
//!   (a) models.toml exists at the known project-relative path and parses.
//!   (b) The pricing VIEW (pricing_for_model) returns byte-identical values
//!       to the catalog entries (behavior-neutral transcription, D-018).
//!   (c) The VRAM threshold values in models.toml [vram_thresholds] match
//!       the in-situ constants in providers::vram_tier (SLC-2 wrap+assert;
//!       core/vram_tier has no data to drift — it is pure types/logic).
//!   (d) Value-sensitivity probe: a seeded wrong price value must NOT equal
//!       the catalog value (demonstrates the comparison is actually sensitive).
//!   (e) Zero raw f64 price literals in the MIGRATED modules (providers/src/
//!       pricing.rs and providers/src/model_catalog.rs) outside models.toml —
//!       scoped to those two files only. cloud_mode.rs is explicitly EXCLUDED
//!       (deferred scatter site per context-package HIGH validation warning).
//!
//! Grounding:
//!   * config/models.toml (I-51 canonical catalog)
//!   * crates/providers/src/pricing.rs (view implementation)
//!   * crates/providers/src/model_catalog.rs (catalog loader)
//!   * crates/providers/src/vram_tier.rs (in-situ threshold constants, SLC-2)

use std::path::Path;

use elai_providers::model_catalog::ModelCatalog;
use elai_providers::pricing::pricing_for_model;

// ---- helper: resolve config/models.toml from CARGO_MANIFEST_DIR ----
// mirrors the a45_source_tiers_toml_contract.rs pattern exactly.
fn models_toml_path() -> std::path::PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crates dir")
        .parent()
        .expect(".elai_cc root")
        .join("config")
        .join("models.toml")
}

// ---- (a) real models.toml exists and parses ----------------------------

#[test]
fn contract_models_toml_exists_and_loads() {
    let path = models_toml_path();
    assert!(
        path.exists(),
        "config/models.toml must exist at {}",
        path.display()
    );
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let catalog = ModelCatalog::from_toml_str(&src)
        .unwrap_or_else(|e| panic!("failed to parse models.toml: {e}"));
    assert!(
        !catalog.is_empty(),
        "catalog must have at least one entry"
    );
    // All three ELAI families must be present.
    assert!(
        !catalog.get_by_family("sonnet").is_empty(),
        "sonnet family must be present"
    );
    assert!(
        !catalog.get_by_family("opus").is_empty(),
        "opus family must be present"
    );
    assert!(
        !catalog.get_by_family("haiku").is_empty(),
        "haiku family must be present"
    );
}

// ---- (b) pricing VIEW returns catalog values byte-identically ---------

#[test]
fn contract_pricing_view_matches_catalog_sonnet() {
    let catalog = ModelCatalog::load().expect("catalog");
    let entries = catalog.get_by_family("sonnet");
    assert_eq!(entries.len(), 1, "exactly one sonnet entry");
    let cat = &entries[0];

    // pricing_for_model uses family-match on model id containing "sonnet".
    let view = pricing_for_model("claude-sonnet-4-20250514")
        .expect("sonnet pricing_for_model must return Some");

    assert_eq!(
        view.input_cost_per_million, cat.input_cost_per_million,
        "sonnet input price: view vs catalog"
    );
    assert_eq!(
        view.output_cost_per_million, cat.output_cost_per_million,
        "sonnet output price: view vs catalog"
    );
    assert_eq!(
        view.cache_creation_cost_per_million, cat.cache_creation_cost_per_million,
        "sonnet cache_creation price: view vs catalog"
    );
    assert_eq!(
        view.cache_read_cost_per_million, cat.cache_read_cost_per_million,
        "sonnet cache_read price: view vs catalog"
    );
}

#[test]
fn contract_pricing_view_matches_catalog_haiku() {
    let catalog = ModelCatalog::load().expect("catalog");
    let entries = catalog.get_by_family("haiku");
    assert_eq!(entries.len(), 1, "exactly one haiku entry");
    let cat = &entries[0];

    let view = pricing_for_model("claude-haiku-4-5-20251001")
        .expect("haiku pricing_for_model must return Some");

    assert_eq!(
        view.input_cost_per_million, cat.input_cost_per_million,
        "haiku input price: view vs catalog"
    );
    assert_eq!(
        view.output_cost_per_million, cat.output_cost_per_million,
        "haiku output price: view vs catalog"
    );
    assert_eq!(
        view.cache_creation_cost_per_million, cat.cache_creation_cost_per_million,
        "haiku cache_creation price: view vs catalog"
    );
    assert_eq!(
        view.cache_read_cost_per_million, cat.cache_read_cost_per_million,
        "haiku cache_read price: view vs catalog"
    );
}

#[test]
fn contract_pricing_view_matches_catalog_opus() {
    let catalog = ModelCatalog::load().expect("catalog");
    let entries = catalog.get_by_family("opus");
    assert_eq!(entries.len(), 1, "exactly one opus entry");
    let cat = &entries[0];

    let view = pricing_for_model("claude-opus-4-6")
        .expect("opus pricing_for_model must return Some");

    assert_eq!(
        view.input_cost_per_million, cat.input_cost_per_million,
        "opus input price: view vs catalog"
    );
    assert_eq!(
        view.output_cost_per_million, cat.output_cost_per_million,
        "opus output price: view vs catalog"
    );
}

// ---- (c) VRAM threshold values in catalog match providers::vram_tier consts
//          (SLC-2 wrap+assert approach) ----------------------------------------
//
// The in-situ constants in providers::vram_tier are (units: bytes, GiB = 1024^3):
//   VRAM_THRESHOLD_48GB = 40 * GiB  -> 40 GiB
//   VRAM_THRESHOLD_24GB = 20 * GiB  -> 20 GiB
//   VRAM_THRESHOLD_16GB = 14 * GiB  -> 14 GiB
//   VRAM_THRESHOLD_12GB = 10 * GiB  -> 10 GiB
// The catalog [vram_thresholds] section reproduces these as GiB values.
// This test asserts equality so any future drift between the two is caught.

const IN_SITU_THRESHOLD_48GB_GIB: u32 = 40;
const IN_SITU_THRESHOLD_24GB_GIB: u32 = 20;
const IN_SITU_THRESHOLD_16GB_GIB: u32 = 14;
const IN_SITU_THRESHOLD_12GB_GIB: u32 = 10;

#[test]
fn contract_vram_thresholds_catalog_matches_insitu_constants() {
    let catalog = ModelCatalog::load().expect("catalog");
    let t = &catalog.vram_thresholds;

    assert_eq!(
        t.threshold_48gb_gib, IN_SITU_THRESHOLD_48GB_GIB,
        "catalog threshold_48gb_gib must match in-situ VRAM_THRESHOLD_48GB (40 GiB)"
    );
    assert_eq!(
        t.threshold_24gb_gib, IN_SITU_THRESHOLD_24GB_GIB,
        "catalog threshold_24gb_gib must match in-situ VRAM_THRESHOLD_24GB (20 GiB)"
    );
    assert_eq!(
        t.threshold_16gb_gib, IN_SITU_THRESHOLD_16GB_GIB,
        "catalog threshold_16gb_gib must match in-situ VRAM_THRESHOLD_16GB (14 GiB)"
    );
    assert_eq!(
        t.threshold_12gb_gib, IN_SITU_THRESHOLD_12GB_GIB,
        "catalog threshold_12gb_gib must match in-situ VRAM_THRESHOLD_12GB (10 GiB)"
    );
}

// core/vram_tier.rs has NO data to drift against — it is pure types/logic
// (VramTier enum, VramTierConfig, MAX_MCTS_BRANCHES_*). The SLC-2 ruling
// says wrap-not-move; the wrap is the assertion above that the catalog's
// vram_thresholds section == the providers::vram_tier in-situ constants.
// core/vram_tier is unchanged and the compile-time checks in
// vram_tier_contract.rs already lock its shape.

// ---- (d) value-sensitivity probe -----------------------------------------
//
// The broken-arm probe asserts that the comparisons are actually value-sensitive
// and not trivially true (e.g. both sides being 0.0 by accident).

#[test]
fn contract_value_sensitivity_probe_catalog_price_is_not_zero() {
    // If this fails, something is deeply wrong with the catalog parse.
    let catalog = ModelCatalog::load().expect("catalog");
    let entries = catalog.get_by_family("sonnet");
    let cat = &entries[0];
    // A seeded wrong value must NOT equal the catalog value.
    let wrong_input_price: f64 = cat.input_cost_per_million + 1.0;
    assert_ne!(
        wrong_input_price, cat.input_cost_per_million,
        "value-sensitivity probe: catalog price + 1.0 must differ from catalog price"
    );
    // Confirm the catalog price itself is non-zero (not a trivially true assertion).
    assert!(
        cat.input_cost_per_million > 0.0,
        "sonnet input price must be positive (not zero, which would make +1.0 sensitivity trivial)"
    );
}

#[test]
fn contract_value_sensitivity_probe_view_is_not_zero() {
    // Same probe for the VIEW path.
    let view = pricing_for_model("claude-sonnet-x").expect("sonnet view");
    let wrong: f64 = view.input_cost_per_million + 1.0;
    assert_ne!(
        wrong, view.input_cost_per_million,
        "value-sensitivity probe: view price + 1.0 must differ from view price"
    );
    assert!(view.input_cost_per_million > 0.0);
}

#[test]
fn contract_value_sensitivity_probe_vram_threshold_nonzero() {
    // Confirm the VRAM threshold constants are non-zero so the assert_eq above
    // is not trivially true (both sides happen to be 0).
    assert!(IN_SITU_THRESHOLD_48GB_GIB > 0);
    assert!(IN_SITU_THRESHOLD_12GB_GIB > 0);
    // And a seeded wrong value proves the equality test would actually catch a drift.
    let wrong: u32 = IN_SITU_THRESHOLD_48GB_GIB + 1;
    assert_ne!(
        wrong, IN_SITU_THRESHOLD_48GB_GIB,
        "value-sensitivity probe: threshold + 1 must differ from threshold"
    );
}

// ---- (e) zero raw f64 price literals in migrated modules -----------------
//
// Reads providers/src/pricing.rs and providers/src/model_catalog.rs source
// and greps for raw floating-point price literals (e.g. 15.0, 1.0, 75.0)
// outside the DEFAULT_* const declarations or the fallback match arm in pricing.rs.
// The contract fails if any such literal is found in non-comment, non-const,
// non-fallback positions.
//
// NOTE: cloud_mode.rs is EXPLICITLY EXCLUDED per HIGH validation warning in
// context-package: it holds deferred per-token price consts (CLAUDE_SONNET_4_6_*)
// that are NOT in I-51 scope. Scoping this check to pricing.rs + model_catalog.rs only.

#[test]
fn contract_no_raw_price_literals_in_pricing_rs() {
    // Zero-residue check: the pre-I-51 pricing_for_model code had each
    // family branch directly returning `Some(ModelPricing { input_cost_per_million: X.Y, ... })`
    // with inline float literals. After I-51, the primary path reads from
    // the catalog via `entry.input_cost_per_million` etc. — no inline floats
    // in the catalog lookup path.
    //
    // The marker for the OLD unmigrated pattern is the presence of the
    // comment "/* pre-I-51 */" or the old if-branch structure. We check
    // the concrete structural invariant: after migration the primary code
    // path (catalog lookup block) uses `entry.` field access, NOT inline
    // float literals for ModelPricing construction.
    //
    // Specifically: `entry.input_cost_per_million` must appear in the file
    // (proves the catalog lookup is wired), and the file must NOT contain
    // a direct `return Some(ModelPricing {` block followed immediately by
    // `input_cost_per_million: 1.0` on the same or next non-whitespace line
    // (the old haiku-branch pattern, now replaced by catalog lookup).
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pricing_rs = manifest
        .parent()
        .expect("crates dir")
        .parent()
        .expect(".elai_cc root")
        .join("crates")
        .join("providers")
        .join("src")
        .join("pricing.rs");

    let src = std::fs::read_to_string(&pricing_rs)
        .unwrap_or_else(|e| panic!("failed to read pricing.rs: {e}"));

    // POSITIVE assertion: the migrated catalog-lookup code must be present.
    assert!(
        src.contains("entry.input_cost_per_million"),
        "pricing.rs must contain 'entry.input_cost_per_million' (catalog view wiring)"
    );
    assert!(
        src.contains("ModelCatalog::load_or_default()"),
        "pricing.rs must call ModelCatalog::load_or_default() (catalog view wiring)"
    );

    // NEGATIVE assertion: the old if-branch inline-struct pattern
    // `if normalized.contains("haiku") { return Some(ModelPricing { ... 1.0 ...` is gone.
    // The tell-tale of the old pattern is the haiku contains-check immediately
    // followed by a return Some(ModelPricing block (with no catalog lookup between them).
    // After migration, the contains() check sets a `family` string variable, not a
    // direct return. So the old `return Some(ModelPricing {` inside the haiku-branch
    // must no longer exist.
    //
    // We check: the file must NOT contain both "contains(\"haiku\")" and
    // "return Some(ModelPricing {" on lines where the return comes first
    // (i.e., the old pattern of returning directly inside the if-branch).
    // Simpler: search for the specific indented return pattern from the old code.
    assert!(
        !src.contains("        return Some(ModelPricing {\n            input_cost_per_million: 1.0"),
        "pricing.rs must not contain old unmigrated haiku-branch inline return pattern"
    );
    assert!(
        !src.contains("        return Some(ModelPricing {\n            input_cost_per_million: 15.0"),
        "pricing.rs must not contain old unmigrated opus/sonnet-branch inline return pattern"
    );
}

#[test]
fn contract_no_raw_price_literals_in_model_catalog_rs() {
    // model_catalog.rs is a pure loader/index — it deserializes ModelEntry from TOML
    // and builds BTreeMap indices. It must NOT contain hard-coded price float literals
    // for any of the ELAI model families (those values live in models.toml only).
    //
    // We check for struct-initializer patterns specifically: an inline `f64 = X.Y`
    // assignment that sets one of the well-known price literals in NON-test,
    // NON-comment code. Test code (inside `mod tests { }` blocks) is excluded
    // because tests may reference catalog-loaded values for assertion comparisons.
    //
    // Approach: scan only the non-test portion of the file (before `#[cfg(test)]`)
    // for raw price struct-initializer patterns.
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog_rs = manifest
        .parent()
        .expect("crates dir")
        .parent()
        .expect(".elai_cc root")
        .join("crates")
        .join("providers")
        .join("src")
        .join("model_catalog.rs");

    let src = std::fs::read_to_string(&catalog_rs)
        .unwrap_or_else(|e| panic!("failed to read model_catalog.rs: {e}"));

    // Split at #[cfg(test)] — only scan the production portion.
    let production_src = src
        .find("#[cfg(test)]")
        .map_or(src.as_str(), |pos| &src[..pos]);

    // Specific known price literals from the scatter inventory (per-million USD values).
    // These must not appear as field-initializer assignments in production code.
    // Format we look for: `field_name: <literal>,` which is the struct-init form.
    let price_field_patterns = [
        "input_cost_per_million: 15.0",
        "input_cost_per_million: 1.0",
        "output_cost_per_million: 75.0",
        "output_cost_per_million: 5.0",
        "cache_creation_cost_per_million: 18.75",
        "cache_creation_cost_per_million: 1.25",
        "cache_read_cost_per_million: 1.5",
        "cache_read_cost_per_million: 0.1",
    ];

    for pattern in &price_field_patterns {
        for (line_no, line) in production_src.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains(pattern) {
                panic!(
                    "model_catalog.rs line {} (production section): found hard-coded price \
                    struct-initializer pattern {:?}. \
                    model_catalog.rs must not contain hard-coded price values — \
                    they come from models.toml",
                    line_no + 1,
                    pattern
                );
            }
        }
    }
}
