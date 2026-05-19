// C-18 B-35 backfill — VerificationMethod::ArtifactCheck dispatch contract.
//
// Three test functions cover the three branches of the `ArtifactCheck` arm
// of `LocalVerifier::verify`:
//
//   1. test_artifact_check_verifies          : lookup returns true  -> Ok(passes=true)
//   2. test_artifact_check_unverified        : lookup returns false -> Ok(passes=false)
//   3. test_artifact_check_malformed_grounding: no lookup bound     -> Err(MalformedGrounding)
//
// The plan's wording ("ArtifactStore backed by an in-memory SQLite db") was
// imprecise — `LocalVerifier` doesn't take an `ArtifactStore` directly. It
// holds an `Option<Arc<dyn Fn(&str, &str) -> bool + Send + Sync>>` closure
// that the orchestrator wires to `artifact_search` in production. Tests use
// a small in-test closure (no SQLite needed, the contract is the dispatch
// branch, not the store wiring — that's covered separately in
// crates/orchestrator/tests/artifact_search_integration.rs).

use std::sync::Arc;

use elai_reasoning::{
    GrcError, LocalVerifier, MockFileReader, MockTestRunner, VerificationMethod, Verifier,
};

// ---------------------------------------------------------------------------
// 1. Verified branch: lookup returns true.
// ---------------------------------------------------------------------------

#[test]
fn test_artifact_check_verifies() {
    let reader = MockFileReader::new();
    let runner = MockTestRunner::new();
    // Closure that always returns true for the artifact we ask for, false
    // otherwise. Locks the (artifact_name, query) -> bool contract.
    let lookup: Arc<dyn Fn(&str, &str) -> bool + Send + Sync> = Arc::new(|name, query| {
        name == "design_doc" && query == "feature-flag rollout"
    });
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner))
        .with_artifact_lookup(lookup);

    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "design_doc".to_string(),
        query: "feature-flag rollout".to_string(),
    };
    let result = verifier
        .verify(method, "claim", "context")
        .expect("ArtifactCheck must return Ok when lookup is bound");

    assert!(
        result.passes,
        "passes must be true when lookup closure returns true (got passes=false, evidence={:?})",
        result.evidence,
    );
    assert!(
        result.evidence.contains("design_doc"),
        "evidence must reference the artifact name; got: {:?}",
        result.evidence,
    );
}

// ---------------------------------------------------------------------------
// 2. Unverified branch: lookup returns false (empty / no-match result).
// ---------------------------------------------------------------------------

#[test]
fn test_artifact_check_unverified() {
    let reader = MockFileReader::new();
    let runner = MockTestRunner::new();
    // Closure that always returns false — models "search returned empty"
    // (the plan's wording).
    let lookup: Arc<dyn Fn(&str, &str) -> bool + Send + Sync> = Arc::new(|_, _| false);
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner))
        .with_artifact_lookup(lookup);

    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "design_doc".to_string(),
        query: "nonexistent topic".to_string(),
    };
    let result = verifier
        .verify(method, "claim", "context")
        .expect("ArtifactCheck must return Ok when lookup is bound (even if no match)");

    assert!(
        !result.passes,
        "passes must be false when lookup closure returns false (got passes=true, evidence={:?})",
        result.evidence,
    );
    assert!(
        result.evidence.contains("no match"),
        "evidence must explain the absence of a match; got: {:?}",
        result.evidence,
    );
}

// ---------------------------------------------------------------------------
// 3. MalformedGrounding branch: artifact_lookup not bound.
// ---------------------------------------------------------------------------
//
// The plan's wording was "artifact_name not in manifest -> MalformedGrounding".
// Reading the production dispatch at
// crates/reasoning/src/verification.rs:402, the
// MalformedGrounding error is emitted only when `artifact_lookup` is `None`
// (i.e. the closure was never bound at construction).  The plan's
// "not in manifest" framing is the orchestrator's responsibility — the
// reasoning crate doesn't know about manifests, it knows about closures.
// We assert the actually-reachable error path.

#[test]
fn test_artifact_check_malformed_grounding() {
    let reader = MockFileReader::new();
    let runner = MockTestRunner::new();
    // No `.with_artifact_lookup(...)` — `artifact_lookup` stays `None`.
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner));

    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "design_doc".to_string(),
        query: "any query".to_string(),
    };
    let err = verifier
        .verify(method, "claim", "context")
        .expect_err("ArtifactCheck must return Err(MalformedGrounding) when lookup is unbound");

    match err {
        GrcError::MalformedGrounding { raw, reason } => {
            assert!(
                raw.contains("design_doc") && raw.contains("any query"),
                "MalformedGrounding.raw must embed artifact_name + query; got raw={:?}",
                raw,
            );
            assert!(
                reason.contains("artifact_lookup") && reason.contains("bound"),
                "MalformedGrounding.reason must mention the unbound artifact_lookup; got reason={:?}",
                reason,
            );
        }
        other => panic!(
            "expected GrcError::MalformedGrounding for unbound artifact_lookup; got {:?}",
            other,
        ),
    }
}
