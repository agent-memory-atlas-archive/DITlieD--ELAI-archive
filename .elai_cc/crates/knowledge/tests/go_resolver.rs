// B-30 Pass C: integration tests for GoModuleResolver.

use elai_knowledge::dkg::resolution::{GoModuleResolver, ImportResolver};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_file(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

#[test]
fn resolves_subpackage_under_module_prefix() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("go.mod"),
        "module github.com/myorg/myrepo\n\ngo 1.21\n").unwrap();
    write_file(tmp.path(), "pkg/service/main.go", "package service");

    let r = GoModuleResolver::new(tmp.path());
    assert_eq!(r.module_prefix(), "github.com/myorg/myrepo");

    let resolved = r.resolve("github.com/myorg/myrepo/pkg/service")
        .expect("should resolve subpackage");
    // Result is a directory path.
    let s = resolved.to_string_lossy();
    assert!(s.contains("pkg") && (s.contains("service") || s.contains("service")),
        "expected pkg/service in path, got: {resolved:?}");
}

#[test]
fn stdlib_and_external_return_none() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("go.mod"),
        "module github.com/myorg/myrepo\n\ngo 1.21\n").unwrap();

    let r = GoModuleResolver::new(tmp.path());
    assert!(r.resolve("fmt").is_none(), "stdlib 'fmt' should return None");
    assert!(r.resolve("os").is_none(),  "stdlib 'os' should return None");
    assert!(r.resolve("github.com/other/repo/pkg").is_none(),
        "external module should return None");
}

#[test]
fn no_go_mod_yields_empty_resolver() {
    let tmp = TempDir::new().unwrap();
    let r = GoModuleResolver::new(tmp.path());
    assert_eq!(r.module_prefix(), "");
    assert!(r.resolve("github.com/myorg/myrepo/pkg").is_none());
    assert!(r.resolve("fmt").is_none());
}
