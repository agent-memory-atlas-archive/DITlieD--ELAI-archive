// Session s-69820 / Phase A contract tests: dashboard-v2 website-creation
// pipeline. Covers the public surface introduced by the new
// scaffold::{fim, build_vars} modules (consumed by the dashboard-v2
// `DashboardScaffoldApplier` and the `fim_fill_section` Tauri command),
// plus shape assertions on shipped templates and the static-typing
// guarantee that DevServerHandle can live inside an
// `Arc<Mutex<BTreeMap<String, DevServerHandle>>>` registry.
//
// Layered against (must already exist in the workspace):
//   - scaffold::{apply_template_from_root, load_manifest,
//                TemplateManifest, ScaffoldError}
//   - elai_web_validator::DevServerHandle
// New surface verified here:
//   - scaffold::build_vars::{default_build_vars,
//                            SCAFFOLD_DEFAULT_PROJECT_NAME,
//                            SCAFFOLD_DEFAULT_AUTHOR,
//                            SCAFFOLD_DESCRIPTION_MAX_CHARS}
//   - scaffold::fim::{find_fim_section, splice_fim_section,
//                     build_qwen_fim_prompt, strip_fim_artifacts,
//                     FimSplice, FimError, QWEN_FIM_PREFIX,
//                     QWEN_FIM_SUFFIX, QWEN_FIM_MIDDLE, QWEN_FIM_PAD}
//
// All tests are pure unit tests — no network, no spawned processes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use elai_web_validator::DevServerHandle;
use scaffold::build_vars::{
    default_build_vars, SCAFFOLD_DEFAULT_AUTHOR, SCAFFOLD_DEFAULT_PROJECT_NAME,
    SCAFFOLD_DESCRIPTION_MAX_CHARS,
};
use scaffold::fim::{
    build_qwen_fim_prompt, find_fim_section, splice_fim_section, strip_fim_artifacts, FimError,
    QWEN_FIM_MIDDLE, QWEN_FIM_PAD, QWEN_FIM_PREFIX, QWEN_FIM_SUFFIX,
};
use scaffold::{apply_template_from_root, load_manifest, ScaffoldError, MANIFEST_FILENAME};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the workspace `templates/web/` directory from the contracts crate.
/// `CARGO_MANIFEST_DIR` for `crates/contracts` is …/.elai_cc/crates/contracts;
/// `../../templates/web` reaches …/.elai_cc/templates/web.
fn workspace_templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("templates")
        .join("web")
}

const SHIPPED_TEMPLATES: &[&str] = &["vite-svelte", "vite-react", "astro", "vanilla"];

const REQUIRED_VAR_KEYS: &[&str] = &[
    "project_name",
    "description",
    "author",
    "primary_hsl",
    "bg_hsl",
    "fg_hsl",
    "accent_hsl",
];

// ---------------------------------------------------------------------------
// 1. default_build_vars produces exactly the seven keys the templates expect.
// ---------------------------------------------------------------------------

#[test]
fn default_build_vars_produces_seven_required_keys() {
    let vars = default_build_vars("make me a react landing page", None, None, None);
    assert_eq!(vars.len(), REQUIRED_VAR_KEYS.len());
    for key in REQUIRED_VAR_KEYS {
        assert!(
            vars.contains_key(*key),
            "default_build_vars missing required key {key:?}; got keys {:?}",
            vars.keys().collect::<Vec<_>>()
        );
    }
    assert_eq!(vars["project_name"], SCAFFOLD_DEFAULT_PROJECT_NAME);
    assert_eq!(vars["author"], SCAFFOLD_DEFAULT_AUTHOR);
}

#[test]
fn default_build_vars_truncates_long_description_at_documented_cap() {
    let task = "x".repeat(SCAFFOLD_DESCRIPTION_MAX_CHARS * 4);
    let vars = default_build_vars(&task, None, None, None);
    let description = &vars["description"];
    assert!(
        description.chars().count() <= SCAFFOLD_DESCRIPTION_MAX_CHARS,
        "description exceeded SCAFFOLD_DESCRIPTION_MAX_CHARS: {} > {}",
        description.chars().count(),
        SCAFFOLD_DESCRIPTION_MAX_CHARS
    );
}

#[test]
fn default_build_vars_uses_caller_overrides_when_provided() {
    let vars = default_build_vars(
        "ignored",
        Some("Acme Site"),
        Some("Marketing landing page"),
        Some("alice"),
    );
    assert_eq!(vars["project_name"], "Acme Site");
    assert_eq!(vars["description"], "Marketing landing page");
    assert_eq!(vars["author"], "alice");
}

#[test]
fn default_build_vars_falls_back_to_default_project_name_for_blank_task() {
    let vars = default_build_vars("   ", None, None, None);
    assert_eq!(vars["project_name"], SCAFFOLD_DEFAULT_PROJECT_NAME);
    assert_eq!(vars["description"], SCAFFOLD_DEFAULT_PROJECT_NAME);
}

// ---------------------------------------------------------------------------
// 2. Shipped manifest is parseable and complete.
// ---------------------------------------------------------------------------

#[test]
fn shipped_manifest_lists_all_four_documented_templates() {
    let root = workspace_templates_root();
    let manifest_path = root.join(MANIFEST_FILENAME);
    let manifest = load_manifest(&manifest_path).expect("manifest must load");
    let names: Vec<&str> = manifest
        .templates
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    for expected in SHIPPED_TEMPLATES {
        assert!(
            names.contains(expected),
            "manifest missing shipped template {expected:?}; got {names:?}"
        );
    }
}

#[test]
fn shipped_manifest_pins_section_taxonomy_for_every_template() {
    let root = workspace_templates_root();
    let manifest =
        load_manifest(&root.join(MANIFEST_FILENAME)).expect("manifest must load");
    let required_sections = ["Hero", "Features", "Cta", "Footer", "Gallery"];
    for entry in &manifest.templates {
        for section in &required_sections {
            assert!(
                entry.sections.iter().any(|s| s == section),
                "template {:?} missing section {section:?}; sections = {:?}",
                entry.name,
                entry.sections
            );
        }
    }
}

#[test]
fn shipped_assets_json_parses_for_every_template() {
    let root = workspace_templates_root();
    for name in SHIPPED_TEMPLATES {
        let assets_path = root.join(name).join("assets").join("assets.json");
        let bytes = std::fs::read(&assets_path)
            .unwrap_or_else(|e| panic!("read assets.json for {name}: {e}"));
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|e| panic!("parse assets.json for {name}: {e}"));
        assert!(
            value.get("logo").is_some(),
            "{name}/assets.json missing logo"
        );
        assert!(
            value.get("favicon").is_some(),
            "{name}/assets.json missing favicon"
        );
        assert!(
            value.get("images").is_some(),
            "{name}/assets.json missing images"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. apply_template rejects unknown templates.
// ---------------------------------------------------------------------------

#[test]
fn apply_template_returns_template_not_found_for_unknown_name() {
    let root = workspace_templates_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path().join("dest");
    let vars: BTreeMap<String, String> = BTreeMap::new();
    let result = apply_template_from_root(&root, "definitely-not-a-template", &target, &vars);
    match result {
        Err(ScaffoldError::TemplateNotFound(name)) => {
            assert_eq!(name, "definitely-not-a-template");
        }
        other => panic!("expected TemplateNotFound, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 4. FIM marker discovery: HTML comment variant.
// ---------------------------------------------------------------------------

#[test]
fn fim_finder_locates_html_comment_markers() {
    let body = "\
<!-- FIM_PREFIX: header -->
<section>
<!-- FIM_FILL_START -->
<h1>placeholder</h1>
<!-- FIM_FILL_END -->
</section>
<!-- FIM_SUFFIX: footer -->
";
    let splice = find_fim_section(body).expect("markers present");
    assert!(splice.before.contains("<!-- FIM_FILL_START -->"));
    assert!(splice.after.contains("<!-- FIM_FILL_END -->"));
    assert_eq!(splice.current_middle.trim(), "<h1>placeholder</h1>");
}

#[test]
fn fim_finder_locates_jsx_block_comment_markers() {
    let body = "\
{/* FIM_PREFIX */}
<div>
{/* FIM_FILL_START */}
old
{/* FIM_FILL_END */}
</div>
{/* FIM_SUFFIX */}
";
    let splice = find_fim_section(body).expect("markers present");
    assert!(splice.before.contains("FIM_FILL_START"));
    assert!(splice.after.contains("FIM_FILL_END"));
    assert_eq!(splice.current_middle.trim(), "old");
}

#[test]
fn fim_finder_errors_on_missing_start_marker() {
    let body = "<section>no markers here</section>\n";
    match find_fim_section(body) {
        Err(FimError::MissingStartMarker) => {}
        other => panic!("expected MissingStartMarker, got {other:?}"),
    }
}

#[test]
fn fim_finder_errors_on_missing_end_marker() {
    let body = "before\n<!-- FIM_FILL_START -->\nopen\n";
    match find_fim_section(body) {
        Err(FimError::MissingEndMarker) => {}
        other => panic!("expected MissingEndMarker, got {other:?}"),
    }
}

#[test]
fn fim_finder_errors_when_end_precedes_start() {
    let body = "<!-- FIM_FILL_END -->\nstuff\n<!-- FIM_FILL_START -->\n";
    match find_fim_section(body) {
        Err(FimError::MarkersOutOfOrder) => {}
        other => panic!("expected MarkersOutOfOrder, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 5. FIM splice replaces only the middle, leaves prefix/suffix intact.
// ---------------------------------------------------------------------------

#[test]
fn fim_splice_replaces_only_text_between_markers() {
    let body = "\
header
<!-- FIM_FILL_START -->
old content
<!-- FIM_FILL_END -->
footer
";
    let splice = find_fim_section(body).expect("markers present");
    let merged = splice_fim_section(&splice, "shiny new content");
    assert!(merged.contains("header\n"));
    assert!(merged.contains("footer\n"));
    assert!(merged.contains("shiny new content"));
    assert!(!merged.contains("old content"));
    // Markers themselves must survive the splice round-trip.
    assert!(merged.contains("<!-- FIM_FILL_START -->"));
    assert!(merged.contains("<!-- FIM_FILL_END -->"));
}

// ---------------------------------------------------------------------------
// 6. Qwen FIM prompt builder formats the special-token sandwich correctly.
// ---------------------------------------------------------------------------

#[test]
fn qwen_fim_prompt_orders_prefix_suffix_middle() {
    let body = "\
header
<!-- FIM_FILL_START -->
to fill
<!-- FIM_FILL_END -->
footer
";
    let splice = find_fim_section(body).expect("markers present");
    let prompt = build_qwen_fim_prompt(&splice, "section hint");
    let prefix_idx = prompt.find(QWEN_FIM_PREFIX).expect("contains fim_prefix");
    let suffix_idx = prompt.find(QWEN_FIM_SUFFIX).expect("contains fim_suffix");
    let middle_idx = prompt.find(QWEN_FIM_MIDDLE).expect("contains fim_middle");
    assert!(prefix_idx < suffix_idx, "fim_prefix must precede fim_suffix");
    assert!(suffix_idx < middle_idx, "fim_suffix must precede fim_middle");
}

// ---------------------------------------------------------------------------
// 7. Artifact stripping removes Qwen FIM pad / endoftext leakage.
// ---------------------------------------------------------------------------

#[test]
fn strip_fim_artifacts_removes_pad_and_endoftext_tokens() {
    let raw = format!("real content {QWEN_FIM_PAD} extra <|endoftext|>");
    let cleaned = strip_fim_artifacts(&raw);
    assert!(!cleaned.contains(QWEN_FIM_PAD));
    assert!(!cleaned.contains("<|endoftext|>"));
    assert!(cleaned.contains("real content"));
}

// ---------------------------------------------------------------------------
// 8. DevServerHandle satisfies the registry shape (Send + 'static).
// ---------------------------------------------------------------------------

#[test]
fn dev_server_handle_fits_into_registry_type() {
    fn assert_send_static<T: Send + 'static>() {}
    assert_send_static::<DevServerHandle>();
    // Compile-time: this typedef is what dashboard-v2's AppState will own.
    type DevServerRegistry = Arc<Mutex<BTreeMap<String, DevServerHandle>>>;
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DevServerRegistry>();
}
