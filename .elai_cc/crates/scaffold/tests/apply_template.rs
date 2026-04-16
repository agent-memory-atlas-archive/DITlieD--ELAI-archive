use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use scaffold::{
    apply_template_from_root, load_manifest, substitute_variables, ScaffoldError,
    DEFAULT_BG_HSL, DEFAULT_FG_HSL, DEFAULT_PRIMARY_HSL,
};

fn default_vars() -> BTreeMap<String, String> {
    let mut v = BTreeMap::new();
    v.insert("project_name".into(), "elai-site".into());
    v.insert("description".into(), "An ELAI-generated site".into());
    v.insert("author".into(), "ELAI".into());
    v.insert("primary_hsl".into(), DEFAULT_PRIMARY_HSL.into());
    v.insert("bg_hsl".into(), DEFAULT_BG_HSL.into());
    v.insert("fg_hsl".into(), DEFAULT_FG_HSL.into());
    v.insert("accent_hsl".into(), "265 89% 78%".into());
    v
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn build_fixture_root(root: &Path) {
    let tpl = root.join("fixture-template");
    write(&tpl.join("package.json"), r#"{"name":"{{project_name}}","version":"0.0.1"}"#);
    write(
        &tpl.join("src/app.css"),
        ":root { --primary: {{primary_hsl}}; --bg: {{bg_hsl}}; --fg: {{fg_hsl}}; }",
    );
    write(
        &tpl.join("src/sections/Hero.svelte"),
        "<!-- FIM_PREFIX -->\n<h1>{{project_name}}</h1>\n<!-- FIM_SUFFIX -->\n",
    );
    let logo = tpl.join("assets/logo.bin");
    fs::create_dir_all(logo.parent().unwrap()).unwrap();
    fs::write(&logo, [0u8, 159, 146, 150, 255, 0, 128]).unwrap();
}

#[test]
fn unknown_template_returns_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    build_fixture_root(tmp.path());
    let target = tmp.path().join("out-unknown");
    let err = apply_template_from_root(
        tmp.path(),
        "does-not-exist",
        &target,
        &default_vars(),
    )
    .expect_err("should error");
    match err {
        ScaffoldError::TemplateNotFound(name) => assert_eq!(name, "does-not-exist"),
        other => panic!("expected TemplateNotFound, got {other:?}"),
    }
}

#[test]
fn apply_fixture_template_produces_expected_files() {
    let tmp = tempfile::tempdir().unwrap();
    build_fixture_root(tmp.path());
    let target = tmp.path().join("out-ok");
    apply_template_from_root(tmp.path(), "fixture-template", &target, &default_vars())
        .expect("apply_template should succeed");
    let mut produced: Vec<PathBuf> = Vec::new();
    collect(&target, &target, &mut produced);
    produced.sort();
    let expected: Vec<PathBuf> = vec![
        PathBuf::from("assets"),
        PathBuf::from("assets/logo.bin"),
        PathBuf::from("package.json"),
        PathBuf::from("src"),
        PathBuf::from("src/app.css"),
        PathBuf::from("src/sections"),
        PathBuf::from("src/sections/Hero.svelte"),
    ];
    assert_eq!(produced, expected);

    let pkg = fs::read_to_string(target.join("package.json")).unwrap();
    assert_eq!(pkg, r#"{"name":"elai-site","version":"0.0.1"}"#);
    let css = fs::read_to_string(target.join("src/app.css")).unwrap();
    assert!(css.contains("--primary: 162 68% 51%"));
    assert!(css.contains("--bg: 220 13% 7%"));
    assert!(css.contains("--fg: 0 0% 95%"));
    let bin = fs::read(target.join("assets/logo.bin")).unwrap();
    assert_eq!(bin, vec![0u8, 159, 146, 150, 255, 0, 128]);
}

#[test]
fn substitute_replaces_primary_hsl_byte_exact() {
    let input = "--primary: {{primary_hsl}};";
    let mut v = BTreeMap::new();
    v.insert("primary_hsl".into(), "162 68% 51%".into());
    let out = substitute_variables(input, &v).unwrap();
    assert_eq!(out, "--primary: 162 68% 51%;");
}

#[test]
fn substitute_missing_variable_returns_error() {
    let input = "{{primary_hsl}}";
    let v: BTreeMap<String, String> = BTreeMap::new();
    let err = substitute_variables(input, &v).expect_err("should error");
    match err {
        ScaffoldError::VariableMissing(name) => assert_eq!(name, "primary_hsl"),
        other => panic!("expected VariableMissing, got {other:?}"),
    }
}

#[test]
fn apply_template_is_deterministic_byte_identical() {
    let tmp = tempfile::tempdir().unwrap();
    build_fixture_root(tmp.path());
    let a = tmp.path().join("run-a");
    let b = tmp.path().join("run-b");
    apply_template_from_root(tmp.path(), "fixture-template", &a, &default_vars()).unwrap();
    apply_template_from_root(tmp.path(), "fixture-template", &b, &default_vars()).unwrap();
    let hash_a = hash_tree(&a);
    let hash_b = hash_tree(&b);
    assert_eq!(hash_a, hash_b);
}

#[test]
fn load_manifest_parses_all_four_templates() {
    let manifest_path = repo_templates_root().join("manifest.toml");
    if !manifest_path.exists() {
        eprintln!("skipping: manifest missing at {}", manifest_path.display());
        return;
    }
    let manifest = load_manifest(&manifest_path).expect("parse manifest");
    assert!(manifest.templates.len() >= 4, "expected >=4 templates");
    let names: Vec<String> = manifest.templates.iter().map(|t| t.name.clone()).collect();
    for want in ["vite-svelte", "vite-react", "astro", "vanilla"] {
        assert!(names.iter().any(|n| n == want), "missing template: {want}");
    }
    for entry in &manifest.templates {
        assert!(!entry.sections.is_empty(), "{} has empty sections", entry.name);
        assert_eq!(entry.theme_system, "css-variables-hsl", "{} theme_system mismatch", entry.name);
    }
}

#[test]
fn substitute_preserves_utf8_values() {
    let input = "prefix {{name}} suffix";
    let mut v = BTreeMap::new();
    v.insert("name".into(), "\u{00E9}\u{0301}\u{65E5}\u{672C}".into());
    let out = substitute_variables(input, &v).unwrap();
    assert_eq!(out, "prefix \u{00E9}\u{0301}\u{65E5}\u{672C} suffix");
}

#[test]
fn apply_template_refuses_non_empty_target() {
    let tmp = tempfile::tempdir().unwrap();
    build_fixture_root(tmp.path());
    let target = tmp.path().join("out-busy");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("pre-existing.txt"), b"busy").unwrap();
    let err = apply_template_from_root(
        tmp.path(),
        "fixture-template",
        &target,
        &default_vars(),
    )
    .expect_err("should error");
    match err {
        ScaffoldError::TargetDirNotEmpty(path) => assert_eq!(path, target),
        other => panic!("expected TargetDirNotEmpty, got {other:?}"),
    }
}

fn repo_templates_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("templates")
        .join("web")
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap().to_path_buf();
        if entry.file_type().unwrap().is_dir() {
            out.push(rel.clone());
            collect(root, &path, out);
        } else {
            out.push(rel);
        }
    }
}

fn hash_tree(root: &Path) -> BTreeMap<PathBuf, blake3::Hash> {
    let mut entries: Vec<PathBuf> = Vec::new();
    collect(root, root, &mut entries);
    entries.sort();
    let mut map: BTreeMap<PathBuf, blake3::Hash> = BTreeMap::new();
    for rel in entries {
        let abs = root.join(&rel);
        if abs.is_dir() {
            map.insert(rel, blake3::Hash::from_bytes([0u8; 32]));
        } else {
            let bytes = fs::read(&abs).unwrap();
            map.insert(rel, blake3::hash(&bytes));
        }
    }
    map
}
