use elai_orchestrator::claim::extract::{extract_claims, extract_claims_with, ClaimKind, ClaimRefResolver, ClaimSet};

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
