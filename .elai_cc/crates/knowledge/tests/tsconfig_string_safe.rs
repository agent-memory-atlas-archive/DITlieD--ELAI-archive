// B-30 Pass D: tests for string-safe comment stripping in tsconfig.rs (WARN 2 fix).
//
// The state-machine strip_comments must NOT strip content inside JSON string
// literals even when those strings contain `//` (e.g. URLs).

use elai_knowledge::dkg::resolution::{ImportResolver, TsPathAliasResolver};
use std::path::Path;

fn write(dir: &Path, rel: &str, content: &str) {
    let full = dir.join(rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, content).unwrap();
}

#[test]
fn test_url_in_string_not_stripped() {
    // A tsconfig whose string value contains a `//` URL.
    // The old regex would strip from `//` to end-of-line, corrupting the JSON.
    let tmp = tempfile::TempDir::new().unwrap();
    write(
        tmp.path(),
        "tsconfig.json",
        r#"{
  "description": "See https://example.com/api/v1 for details",
  "compilerOptions": {
    "paths": {
      "@api/*": ["src/api/*"]
    }
  }
} // trailing line comment
"#,
    );
    // If strip_comments is correct, serde_json parses successfully and the
    // @api/* alias resolves.
    let resolver = TsPathAliasResolver::new(tmp.path());
    let result = resolver.resolve("@api/foo");
    assert!(
        result.is_some(),
        "expected @api/foo to resolve — strip_comments must preserve the string value containing '//', got None"
    );
}

#[test]
fn test_trailing_line_comment_removed() {
    // A tsconfig with a trailing `// comment` after the closing brace.
    // The comment must be removed (not the string content before it).
    let tmp = tempfile::TempDir::new().unwrap();
    write(
        tmp.path(),
        "tsconfig.json",
        r#"{ // project config
  "compilerOptions": {
    "paths": {
      "@lib/*": ["src/lib/*"] /* wildcard alias */
    }
  }
}"#,
    );
    let resolver = TsPathAliasResolver::new(tmp.path());
    assert!(
        resolver.resolve("@lib/utils").is_some(),
        "trailing comments must not prevent alias loading"
    );
}

#[test]
fn test_escaped_quote_inside_string() {
    // A string containing an escaped quote followed by `//` — the `//` is
    // still inside the string and must not be treated as a line comment.
    let tmp = tempfile::TempDir::new().unwrap();
    write(
        tmp.path(),
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "paths": {
      "@x/*": ["src/x/*"]
    }
  },
  "note": "say \"hi\" //not a comment"
}"#,
    );
    let resolver = TsPathAliasResolver::new(tmp.path());
    assert!(
        resolver.resolve("@x/foo").is_some(),
        "escaped-quote inside string must not corrupt JSON parsing"
    );
}
