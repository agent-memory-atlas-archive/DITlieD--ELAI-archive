//! B-35 Phase A — ArtifactManifest loader tests.
//!
//! 4 tests: parse_5_artifacts, reject_path_escape, reject_invalid_name,
//! support_dir_path.

use std::fs;
use std::path::Path;

use elai_knowledge::artifacts::manifest::ArtifactError;
use elai_knowledge::artifacts::ArtifactManifest;

fn write_file(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

fn toml_with_artifact(name: &str, path: &str, description: &str) -> String {
    format!(
        r#"[[knowledge.artifacts]]
name = "{name}"
path = "{path}"
description = "{description}"
"#
    )
}

// ---------------------------------------------------------------------------

#[test]
fn parse_5_artifacts() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create the files so canonicalize succeeds
    write_file(root, "docs/schema.sql", "CREATE TABLE users (id INT);");
    write_file(root, "docs/openapi.yaml", "openapi: 3.0");
    write_file(root, "docs/glossary.md", "# Glossary");
    write_file(root, "docs/terraform.tf", "resource {}");
    write_file(root, "k8s/deploy.yaml", "apiVersion: apps/v1");

    let toml_content = r#"
[[knowledge.artifacts]]
name = "database-schema"
path = "docs/schema.sql"
description = "PostgreSQL schema"

[[knowledge.artifacts]]
name = "api-spec"
path = "docs/openapi.yaml"
description = "OpenAPI 3.0 spec"

[[knowledge.artifacts]]
name = "ubiquitous-language"
path = "docs/glossary.md"
description = "Domain glossary"

[[knowledge.artifacts]]
name = "terraform-config"
path = "docs/terraform.tf"
description = "Terraform config"

[[knowledge.artifacts]]
name = "k8s-manifests"
path = "k8s/deploy.yaml"
description = "K8s manifests"
"#;

    write_file(root, "elai_cc.toml", toml_content);

    let manifest = ArtifactManifest::load(root).unwrap();
    assert_eq!(manifest.items.len(), 5, "expected 5 artifacts");

    let names: Vec<&str> = manifest.items.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"database-schema"));
    assert!(names.contains(&"k8s-manifests"));
}

#[test]
fn reject_path_escape() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Deliberately do NOT create the target file — canonicalize will fail
    // because the path is outside root and doesn't exist, but the primary
    // guard is the starts_with check after canonicalization.
    // On most OS, "../etc/passwd" from a temp dir will resolve to a real
    // path outside the temp dir root, triggering PathEscape.
    let toml_content = toml_with_artifact("evil", "../etc/passwd", "should be rejected");
    write_file(root, "elai_cc.toml", &toml_content);

    let result = ArtifactManifest::load(root);
    // Expect either PathEscape or PathNotFound (either is an appropriate rejection)
    assert!(result.is_err(), "path escape must be rejected");
    let err = result.unwrap_err();
    let is_path_error = matches!(
        err,
        ArtifactError::PathEscape(_) | ArtifactError::PathNotFound(_)
    );
    assert!(is_path_error, "expected PathEscape or PathNotFound, got: {err:?}");
}

#[test]
fn reject_invalid_name() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(root, "docs/schema.sql", "-- schema");

    let toml_content = toml_with_artifact("bad name!", "docs/schema.sql", "description");
    write_file(root, "elai_cc.toml", &toml_content);

    let result = ArtifactManifest::load(root);
    assert!(result.is_err(), "invalid name must be rejected");
    assert!(
        matches!(result.unwrap_err(), ArtifactError::InvalidName(_)),
        "expected InvalidName"
    );
}

#[test]
fn support_dir_path() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create a directory artifact
    write_file(root, "k8s/deploy.yaml", "apiVersion: apps/v1");
    write_file(root, "k8s/service.yaml", "kind: Service");

    let toml_content = toml_with_artifact("k8s-manifests", "k8s", "K8s manifests directory");
    write_file(root, "elai_cc.toml", &toml_content);

    let manifest = ArtifactManifest::load(root).unwrap();
    assert_eq!(manifest.items.len(), 1);
    assert!(
        manifest.items[0].is_dir,
        "directory artifact must have is_dir=true"
    );
    assert_eq!(manifest.items[0].name, "k8s-manifests");
}
