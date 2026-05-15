// B-30 Pass C: integration tests for TsPathAliasResolver.

use elai_knowledge::dkg::resolution::{ImportResolver, TsPathAliasResolver};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_file(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

#[test]
fn simple_wildcard_alias() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "tsconfig.json", r#"
    {
        "compilerOptions": {
            "paths": {
                "@app/*": ["src/app/*"],
                "@lib/*": ["src/lib/*"]
            }
        }
    }
    "#);

    let r = TsPathAliasResolver::new(tmp.path());

    let app_foo = r.resolve("@app/foo").expect("@app/foo should resolve");
    assert!(app_foo.to_string_lossy().contains("foo"),
        "expected foo in path, got: {app_foo:?}");

    let lib_bar = r.resolve("@lib/bar").expect("@lib/bar should resolve");
    assert!(lib_bar.to_string_lossy().contains("bar"),
        "expected bar in path, got: {lib_bar:?}");

    // Unknown alias.
    assert!(r.resolve("@unknown/x").is_none());
}

#[test]
fn extends_chain_child_and_parent_aliases_merged() {
    let tmp = TempDir::new().unwrap();

    // Parent config has @lib/*.
    write_file(tmp.path(), "tsconfig.base.json", r#"
    {
        "compilerOptions": {
            "paths": {
                "@lib/*": ["src/lib/*"]
            }
        }
    }
    "#);

    // Child extends parent, adds @app/*. Also has inline // comment.
    write_file(tmp.path(), "tsconfig.json", r#"
    { // child project config
        "extends": "./tsconfig.base.json",
        "compilerOptions": {
            "paths": {
                "@app/*": ["src/app/*"]
            }
        }
    }
    "#);

    let r = TsPathAliasResolver::new(tmp.path());

    // Both child and parent aliases must be present.
    assert!(r.resolve("@app/foo").is_some(), "@app/foo should resolve via child");
    assert!(r.resolve("@lib/bar").is_some(), "@lib/bar should resolve via parent");
}

#[test]
fn cycle_in_extends_does_not_infinite_loop() {
    let tmp = TempDir::new().unwrap();

    // Self-referential extends.
    write_file(tmp.path(), "tsconfig.cycle.json", r#"
    {
        "extends": "./tsconfig.cycle.json",
        "compilerOptions": {
            "paths": { "@cycle/*": ["src/cycle/*"] }
        }
    }
    "#);

    // Wrapper that kicks off the cycle.
    write_file(tmp.path(), "tsconfig.json", r#"
    {
        "extends": "./tsconfig.cycle.json"
    }
    "#);

    // Must not hang, stack-overflow, or panic.
    let r = TsPathAliasResolver::new(tmp.path());
    // @cycle/* should have been loaded from tsconfig.cycle.json before cycle detected.
    let _ = r.resolve("@cycle/foo");
    // No assertion on the result — just must terminate.
}
