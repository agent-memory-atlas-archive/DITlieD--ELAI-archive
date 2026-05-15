// B-30 Pass D: deterministic first-match-wins test for JvmResolver (WARN 3 fix).
//
// When two files produce the same FQN (possible in multi-module Maven projects
// where moduleA and moduleB each have com.example.Foo), the globally
// lexicographically-earliest file path must win — regardless of WalkBuilder's
// per-directory traversal order.

use elai_knowledge::dkg::resolution::{ImportResolver, JvmResolver};
use std::path::Path;

fn write(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, content).unwrap();
}

#[test]
fn test_global_lex_first_match_wins() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path();

    // Two competing files with the same FQN.
    // "moduleA" < "moduleB" lexicographically so moduleA must win.
    write(
        project,
        "project/moduleA/src/main/java/com/example/Foo.java",
        "package com.example; class Foo {}",
    );
    write(
        project,
        "project/moduleB/src/main/java/com/example/Foo.java",
        "package com.example; class Foo {}",
    );

    let resolver = JvmResolver::new(project);
    let resolved = resolver
        .resolve("com.example.Foo")
        .expect("com.example.Foo must resolve");
    let resolved_str = resolved.to_string_lossy();

    assert!(
        resolved_str.contains("moduleA"),
        "expected moduleA (lex-first) to win, got: {}",
        resolved_str
    );
    assert!(
        !resolved_str.contains("moduleB"),
        "moduleB must not win over moduleA, got: {}",
        resolved_str
    );
}

#[test]
fn test_global_lex_deterministic_across_runs() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path();

    write(
        project,
        "alpha/src/main/java/com/lib/Bar.java",
        "package com.lib; class Bar {}",
    );
    write(
        project,
        "beta/src/main/java/com/lib/Bar.java",
        "package com.lib; class Bar {}",
    );

    // Build twice — result must be byte-identical.
    let r1 = JvmResolver::new(project);
    let r2 = JvmResolver::new(project);

    let p1 = r1.resolve("com.lib.Bar").expect("first resolve");
    let p2 = r2.resolve("com.lib.Bar").expect("second resolve");

    assert_eq!(
        p1, p2,
        "resolver must be deterministic across identical runs, got {p1:?} vs {p2:?}"
    );

    // "alpha" < "beta" — alpha wins.
    assert!(
        p1.to_string_lossy().contains("alpha"),
        "expected alpha/ (lex-first) to win, got: {:?}",
        p1
    );
}
