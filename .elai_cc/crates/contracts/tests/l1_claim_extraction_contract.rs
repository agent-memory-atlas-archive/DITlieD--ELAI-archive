use elai_orchestrator::claim::extract::{extract_claims, extract_claims_with, ClaimKind, ClaimRefResolver, ClaimSet};
use elai_orchestrator::claim::resolve::WorkspaceRefResolver;

struct KnownPath;
impl ClaimRefResolver for KnownPath { fn resolve_path(&self, raw: &str) -> bool { raw == "known.rs" } }

#[test]
fn typed_contract_and_unresolved_refs() {
    let file = extract_claims("See `crates/orchestrator/src/claim.rs`.");
    assert_eq!(file.count_of_kind(ClaimKind::File), 1);
    assert_eq!(file.claims()[0].refs[0].raw, "crates/orchestrator/src/claim.rs");
    let dep = extract_claims("Use `left-pad@9.9.9`.");
    assert_eq!(dep.count_of_kind(ClaimKind::Dependency), 1);
    assert!(!dep.claims()[0].refs[0].resolved);
}

#[test]
fn sentence_precedence_and_resolver() {
    let out = extract_claims("It causes breaks. It works!");
    assert_eq!(out.claims()[0].kind, ClaimKind::Causal);
    assert_eq!(out.claims()[1].kind, ClaimKind::Behavior);
    assert_eq!(extract_claims("`fn parse(x: &str) -> T`").claims()[0].kind, ClaimKind::Api);
    let resolved = extract_claims_with("`known.rs` `other.rs`", &KnownPath);
    assert!(resolved.claims()[0].refs[0].resolved);
    assert!(!resolved.claims()[1].refs[0].resolved);
}

#[test]
fn extractor_is_plain_synchronous_function() {
    // A plain function pointer taking only &str is a structural purity witness:
    // this proves the callable has no client, runtime, or handle parameter.
    let f: fn(&str) -> ClaimSet = extract_claims;
    assert!(f("").is_empty());
}


fn contract_scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("elai-l1-contract-{}-{}", std::process::id(), name))
}

#[test]
fn workspace_resolver_separates_real_from_fake_paths() {
    let root = contract_scratch("paths");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("crates/demo")).unwrap();
    std::fs::write(root.join("crates/demo/real.rs"), "").unwrap();
    let resolver = WorkspaceRefResolver::new(&root);
    let claims = extract_claims_with("`crates/demo/real.rs` `crates/demo/ghost.rs`", &resolver);
    assert!(claims.claims()[0].refs[0].resolved);
    assert!(!claims.claims()[1].refs[0].resolved);
    assert_eq!(claims.unresolved_ref_count(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn workspace_resolver_refuses_path_traversal() {
    let root = contract_scratch("traversal");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let resolver = WorkspaceRefResolver::new(&root);
    let claims = extract_claims_with("`../../etc/passwd`", &resolver);
    assert!(!claims.claims()[0].refs[0].resolved);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn workspace_resolver_reads_the_lockfile() {
    let root = contract_scratch("lockfile");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.lock"), "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n").unwrap();
    let resolver = WorkspaceRefResolver::new(&root);
    let claims = extract_claims_with("`serde@1.0.0` `no-such-crate-zzz@1.0.0`", &resolver);
    assert!(claims.claims()[0].refs[0].resolved);
    assert!(!claims.claims()[1].refs[0].resolved);
    let _ = std::fs::remove_dir_all(root);
}
