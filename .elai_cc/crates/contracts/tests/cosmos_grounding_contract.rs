//! Contract test: B-42 Phase 2 — VerificationMethod::CosmosCheck +
//! LocalVerifier wiring via cosmos_lookup callback.
//!
//! Verifies:
//!   (1) VerificationMethod has CosmosCheck variant (compile-time assertion).
//!   (2) requires_cosmos_store() predicate distinguishes CosmosCheck.
//!   (3) LocalVerifier::with_cosmos_lookup builder exists.
//!   (4) verify_claim handles CosmosCheck — no closure -> Err(MalformedGrounding).
//!   (5) verify_claim handles CosmosCheck — closure returning true -> passes = true.
//!   (6) verify_claim handles CosmosCheck — closure returning false -> passes = false.
//!   (7) supports() reports CosmosCheck as supported even without closure
//!       (matches ArtifactCheck precedent — runtime concern, not capability).
//!
//! No real cosmos store is used; closures are mocks.

use std::sync::Arc;

use elai_reasoning::types::{GrcError, VerificationMethod};
use elai_reasoning::verification::{LocalVerifier, MockFileReader, MockTestRunner, Verifier};

// ---------------------------------------------------------------------------
// (1) + (2) Variant exists + requires_cosmos_store predicate.
// ---------------------------------------------------------------------------

#[test]
fn cosmos_check_variant_exists() {
    let method = VerificationMethod::CosmosCheck {
        artifact_name: "ios_hig_summary".to_string(),
        query: "navigation bar patterns".to_string(),
        claim_category: "design".to_string(),
    };
    let cloned = method.clone();
    assert_eq!(method, cloned);
    assert!(method.requires_cosmos_store());
    assert!(!VerificationMethod::FileCheck.requires_cosmos_store());
    assert!(!VerificationMethod::WebCheck.requires_cosmos_store());
    // ArtifactCheck (B-35) is distinct from CosmosCheck (B-42).
    assert!(!VerificationMethod::ArtifactCheck {
        artifact_name: String::new(),
        query: String::new(),
    }
    .requires_cosmos_store());
}

// ---------------------------------------------------------------------------
// (3) + (4) No closure -> Err(MalformedGrounding).
// ---------------------------------------------------------------------------

#[test]
fn cosmos_check_no_closure_returns_err() {
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    );
    let method = VerificationMethod::CosmosCheck {
        artifact_name: "app_store_policies".to_string(),
        query: "subscription guidelines".to_string(),
        claim_category: "app_store".to_string(),
    };
    let result = verifier.verify(method, "claim about App Store rules", "");
    match result {
        Err(GrcError::MalformedGrounding { reason, .. }) => {
            assert!(
                reason.contains("cosmos_lookup not bound"),
                "expected 'cosmos_lookup not bound' in reason, got: {reason}"
            );
        }
        other => panic!(
            "expected Err(MalformedGrounding) when cosmos_lookup is None, got: {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// (5) Closure returning true -> passes = true.
// ---------------------------------------------------------------------------

#[test]
fn cosmos_check_closure_true_passes() {
    let captured: Arc<std::sync::Mutex<Vec<(String, String, String)>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
    let lookup = Arc::new(move |name: &str, query: &str, category: &str| -> bool {
        if let Ok(mut g) = captured_clone.lock() {
            g.push((name.to_string(), query.to_string(), category.to_string()));
        }
        true
    });
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    )
    .with_cosmos_lookup(lookup);
    let method = VerificationMethod::CosmosCheck {
        artifact_name: "ios_hig_summary".to_string(),
        query: "navigation patterns".to_string(),
        claim_category: "design".to_string(),
    };
    let result = verifier
        .verify(method, "iOS design claim", "")
        .expect("ok");
    assert!(result.passes, "expected passes=true, got {:?}", result);
    assert!(result.evidence.contains("ios_hig_summary"));
    assert!(result.evidence.contains("design"));
    let calls = captured.lock().expect("lock");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "ios_hig_summary");
    assert_eq!(calls[0].1, "navigation patterns");
    assert_eq!(calls[0].2, "design");
}

// ---------------------------------------------------------------------------
// (6) Closure returning false -> passes = false + evidence string.
// ---------------------------------------------------------------------------

#[test]
fn cosmos_check_closure_false_does_not_pass() {
    let lookup =
        Arc::new(|_n: &str, _q: &str, _c: &str| -> bool { false });
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    )
    .with_cosmos_lookup(lookup);
    let method = VerificationMethod::CosmosCheck {
        artifact_name: "web_a11y_wcag".to_string(),
        query: "color contrast rule 1.4.3".to_string(),
        claim_category: "ux_convention".to_string(),
    };
    let result = verifier
        .verify(method, "WCAG 1.4.3 claim", "")
        .expect("ok");
    assert!(!result.passes);
    assert!(result.evidence.contains("no match"));
}

// ---------------------------------------------------------------------------
// (7) supports() reports CosmosCheck as supported.
// ---------------------------------------------------------------------------

#[test]
fn supports_reports_cosmos_check() {
    let verifier = LocalVerifier::new(
        Box::new(MockFileReader::new()),
        Box::new(MockTestRunner::new()),
    );
    let probe = VerificationMethod::CosmosCheck {
        artifact_name: String::new(),
        query: String::new(),
        claim_category: String::new(),
    };
    assert!(
        verifier.supports(&probe),
        "LocalVerifier::supports() must report CosmosCheck regardless of closure binding"
    );
    // ArtifactCheck precedent: still supported.
    let probe_a = VerificationMethod::ArtifactCheck {
        artifact_name: String::new(),
        query: String::new(),
    };
    assert!(verifier.supports(&probe_a));
}
