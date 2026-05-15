// B-30 Pass C: integration tests for JvmResolver.
//
// These tests run with real filesystem fixtures via tempfile::TempDir.
// No database required.

use elai_knowledge::dkg::resolution::{ImportResolver, JvmResolver};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_file(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

#[test]
fn empty_project_returns_none() {
    let tmp = TempDir::new().unwrap();
    let resolver = JvmResolver::new(tmp.path());
    assert!(resolver.resolve("com.example.Foo").is_none());
    assert!(resolver.resolve("").is_none());
}

#[test]
fn single_java_and_kotlin_file_resolves() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "src/main/java/com/example/Foo.java",
        "package com.example; class Foo {}");
    write_file(tmp.path(), "src/main/java/com/example/Bar.java",
        "package com.example; class Bar {}");
    write_file(tmp.path(), "src/main/kotlin/com/lib/Util.kt",
        "package com.lib");

    let resolver = JvmResolver::new(tmp.path());

    let foo = resolver.resolve("com.example.Foo")
        .expect("com.example.Foo should resolve");
    assert!(foo.to_string_lossy().contains("Foo.java"), "expected Foo.java path, got: {foo:?}");

    let bar = resolver.resolve("com.example.Bar")
        .expect("com.example.Bar should resolve");
    assert!(bar.to_string_lossy().contains("Bar.java"), "expected Bar.java path, got: {bar:?}");

    let util = resolver.resolve("com.lib.Util")
        .expect("com.lib.Util should resolve");
    assert!(util.to_string_lossy().contains("Util.kt"), "expected Util.kt path, got: {util:?}");

    // Unknown FQN returns None.
    assert!(resolver.resolve("com.example.Missing").is_none());
}

#[test]
fn wildcard_import_returns_none() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "src/main/java/com/example/Foo.java",
        "package com.example; class Foo {}");
    let resolver = JvmResolver::new(tmp.path());
    // Java wildcard
    assert!(resolver.resolve("com.example.*").is_none(), "wildcard should return None");
    // Kotlin/Scala wildcard
    assert!(resolver.resolve("com.example._").is_none(), "Kotlin wildcard should return None");
}
