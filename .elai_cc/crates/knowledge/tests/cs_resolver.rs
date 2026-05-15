// B-30 Pass C: integration tests for CsNamespaceResolver.

use elai_knowledge::dkg::resolution::{CsNamespaceResolver, ImportResolver};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_file(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

#[test]
fn block_and_file_scoped_namespaces() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "src/Models/User.cs",
        "namespace App.Models { class User {} }");
    write_file(tmp.path(), "src/Services/Auth.cs",
        "namespace App.Services;");

    let r = CsNamespaceResolver::new(tmp.path());

    let user_path = r.resolve("App.Models").expect("App.Models (block ns)");
    assert!(user_path.to_string_lossy().contains("User.cs"),
        "expected User.cs, got: {user_path:?}");

    let auth_path = r.resolve("App.Services").expect("App.Services (file-scoped ns)");
    assert!(auth_path.to_string_lossy().contains("Auth.cs"),
        "expected Auth.cs, got: {auth_path:?}");
}

#[test]
fn strip_class_segment_fallback() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "src/Models/User.cs",
        "namespace App.Models { class User {} }");

    let r = CsNamespaceResolver::new(tmp.path());

    // "App.Models.User" -> strips "User", matches namespace "App.Models"
    let result = r.resolve("App.Models.User")
        .expect("App.Models.User should fall back to App.Models");
    assert!(result.to_string_lossy().contains("User.cs"),
        "expected User.cs, got: {result:?}");

    // Unknown namespace and no useful prefix -> None
    assert!(r.resolve("Unknown.Namespace.Cls").is_none());
}

#[test]
fn lexicographic_determinism_across_runs() {
    let tmp = TempDir::new().unwrap();
    // Two files with the same namespace.
    write_file(tmp.path(), "a/Alpha.cs", "namespace Shared.Util {}");
    write_file(tmp.path(), "b/Beta.cs",  "namespace Shared.Util {}");

    let r1 = CsNamespaceResolver::new(tmp.path());
    let r2 = CsNamespaceResolver::new(tmp.path());

    let p1 = r1.resolve("Shared.Util").expect("Shared.Util run 1");
    let p2 = r2.resolve("Shared.Util").expect("Shared.Util run 2");
    assert_eq!(p1, p2, "must be deterministic across runs");

    // Lexicographically, a/Alpha.cs < b/Beta.cs
    assert!(
        p1.to_string_lossy().contains("Alpha.cs"),
        "Alpha.cs should win (lex order), got: {p1:?}"
    );
}
