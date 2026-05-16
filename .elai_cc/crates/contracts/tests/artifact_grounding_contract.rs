//! Contract test: B-35 Phase B — VerificationMethod::ArtifactCheck + LocalVerifier
//! wiring via artifact_lookup callback.
//!
//! Verifies:
//!   (1) VerificationMethod has ArtifactCheck variant (compile-time assertion).
//!   (2) LocalVerifier::with_artifact_lookup builder exists.
//!   (3) verify_claim handles ArtifactCheck — no closure -> Err(MalformedGrounding).
//!   (4) verify_claim handles ArtifactCheck — closure returning true -> passes = true.
//!   (5) verify_claim handles ArtifactCheck — closure returning false -> passes = false.
//!
//! No real ArtifactStore is used; closures are mocks.

use std::sync::Arc;

use elai_reasoning::verification::{LocalVerifier, MockFileReader, MockTestRunner, Verifier};
use elai_reasoning::types::{GrcError, VerificationMethod};

// ---------------------------------------------------------------------------
// (1) Compile-time: ArtifactCheck variant exists with correct field names.
// ---------------------------------------------------------------------------

#[test]
fn artifact_check_variant_exists() {
    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "openapi_spec".to_string(),
        query: "list endpoints".to_string(),
    };
    // Confirm Debug + Clone + PartialEq are derived.
    let cloned = method.clone();
    assert_eq!(method, cloned);
    // Confirm requires_artifact_store predicate works.
    assert!(method.requires_artifact_store());
    // Confirm existing variants do NOT require artifact store.
    assert!(!VerificationMethod::FileCheck.requires_artifact_store());
    assert!(!VerificationMethod::WebCheck.requires_artifact_store());
}

// ---------------------------------------------------------------------------
// (2) & (3) No closure -> Err(MalformedGrounding) with "artifact_lookup not bound".
// ---------------------------------------------------------------------------

#[test]
fn artifact_check_no_closure_returns_err() {
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    );
    // No with_artifact_lookup call — closure is None.
    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "db_schema".to_string(),
        query: "users table".to_string(),
    };
    let result = verifier.verify(method, "some claim", "");
    match result {
        Err(GrcError::MalformedGrounding { reason, .. }) => {
            assert!(
                reason.contains("artifact_lookup not bound"),
                "expected 'artifact_lookup not bound' in reason, got: {reason}"
            );
        }
        other => panic!("expected Err(MalformedGrounding), got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (4) Closure returning true -> passes = true, Verified evidence.
// ---------------------------------------------------------------------------

#[test]
fn artifact_check_closure_true_returns_verified() {
    let lookup = Arc::new(|_artifact: &str, _query: &str| -> bool { true });
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    )
    .with_artifact_lookup(lookup);

    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "k8s_manifests".to_string(),
        query: "deployment replicas".to_string(),
    };
    let result = verifier.verify(method, "deployment has 3 replicas", "").expect("should not error");
    assert!(result.passes, "expected passes=true when lookup returns true");
    assert!(
        result.evidence.contains("k8s_manifests"),
        "expected artifact name in evidence"
    );
    // method_used should be ArtifactCheck
    assert!(
        matches!(result.method_used, VerificationMethod::ArtifactCheck { .. }),
        "expected method_used = ArtifactCheck, got {:?}", result.method_used
    );
}

// ---------------------------------------------------------------------------
// (5) Closure returning false -> passes = false, Unverified evidence.
// ---------------------------------------------------------------------------

#[test]
fn artifact_check_closure_false_returns_unverified() {
    let lookup = Arc::new(|_artifact: &str, _query: &str| -> bool { false });
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    )
    .with_artifact_lookup(lookup);

    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "openapi_spec".to_string(),
        query: "GET /users endpoint".to_string(),
    };
    let result = verifier.verify(method, "API has GET /users", "").expect("should not error");
    assert!(!result.passes, "expected passes=false when lookup returns false");
    assert!(
        result.evidence.contains("no match"),
        "expected 'no match' in evidence, got: {}", result.evidence
    );
}

// ---------------------------------------------------------------------------
// (6) Closure receives correct arguments.
// ---------------------------------------------------------------------------

#[test]
fn artifact_check_closure_receives_correct_args() {
    use std::sync::Mutex;
    let calls: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_clone = calls.clone();
    let lookup = Arc::new(move |artifact: &str, query: &str| -> bool {
        calls_clone.lock().unwrap().push((artifact.to_string(), query.to_string()));
        true
    });

    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    )
    .with_artifact_lookup(lookup);

    let method = VerificationMethod::ArtifactCheck {
        artifact_name: "terraform_configs".to_string(),
        query: "S3 bucket config".to_string(),
    };
    let _ = verifier.verify(method, "terraform defines S3 bucket", "");
    let recorded = calls.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].0, "terraform_configs");
    assert_eq!(recorded[0].1, "S3 bucket config");
}
