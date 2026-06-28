use std::io::Write;

use tempfile::NamedTempFile;

use elai_reasoning::{
    BasicGrcEngine, GrcEngine, GrcError, MockFileReader, MockTestRunner,
    PlanGroundingResult, TestRunResult, VerificationMethod, VerificationResult,
};
use elai_reasoning::{LocalVerifier, NetworkVerifier, Verifier};

// =========================================================================
// T1. file_check_existing_file
// =========================================================================

#[test]
fn file_check_existing_file() {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    write!(tmp, "content").expect("write temp");
    let path = tmp.path().to_str().expect("path to str").to_string();

    let reader = MockFileReader::new().with_file(&path, "content");
    let runner = MockTestRunner::new();
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner));

    let result = verifier
        .verify(VerificationMethod::FileCheck, &path, "")
        .expect("verify should not error");

    assert!(result.passes, "existing file should pass FileCheck");
    assert!(
        result.evidence.contains(&path),
        "evidence should contain file path"
    );
}

// =========================================================================
// T2. file_check_nonexistent_file
// =========================================================================

#[test]
fn file_check_nonexistent_file() {
    let reader = MockFileReader::new();
    let runner = MockTestRunner::new();
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner));

    let result = verifier
        .verify(
            VerificationMethod::FileCheck,
            "nonexistent_file_xyz.rs",
            "",
        )
        .expect("verify should not error");

    assert!(!result.passes, "nonexistent file should fail FileCheck");
}

// =========================================================================
// T3. source_check_function_exists
// =========================================================================

#[test]
fn source_check_function_exists() {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    write!(tmp, "fn hello_world() {{}}\n").expect("write temp");
    let path = tmp.path().to_str().expect("path to str").to_string();

    let reader = MockFileReader::new().with_file(&path, "fn hello_world() {}\n");
    let runner = MockTestRunner::new();
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner));

    let result = verifier
        .verify(VerificationMethod::SourceCheck, "hello_world", &path)
        .expect("verify should not error");

    assert!(result.passes, "existing function should pass SourceCheck");
    assert!(
        result.evidence.contains("hello_world"),
        "evidence should contain function name"
    );
}

// =========================================================================
// T4. source_check_function_missing
// =========================================================================

#[test]
fn source_check_function_missing() {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    write!(tmp, "fn other_function() {{}}\n").expect("write temp");
    let path = tmp.path().to_str().expect("path to str").to_string();

    let reader = MockFileReader::new().with_file(&path, "fn other_function() {}\n");
    let runner = MockTestRunner::new();
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner));

    let result = verifier
        .verify(VerificationMethod::SourceCheck, "nonexistent_fn", &path)
        .expect("verify should not error");

    assert!(!result.passes, "missing function should fail SourceCheck");
}

// =========================================================================
// T5. test_check_passing_test (mock test runner, no subprocess)
// =========================================================================

#[test]
fn test_check_passing_test() {
    let reader = MockFileReader::new();
    let runner = MockTestRunner::new().with_result(
        "test_name",
        TestRunResult {
            passed: true,
            output: "ok".to_string(),
            duration_ms: 1,
        },
    );
    let verifier = LocalVerifier::new(Box::new(reader), Box::new(runner));

    let result = verifier
        .verify(VerificationMethod::TestCheck, "test_name", "test_module_path")
        .expect("verify should not error");

    assert!(result.passes, "passing test should pass TestCheck");
}

// =========================================================================
// T6. web_check_stub_returns_error
// =========================================================================

#[test]
fn web_check_stub_returns_error() {
    let verifier = NetworkVerifier::new();

    let result = verifier.verify(VerificationMethod::WebCheck, "some algorithm", "");

    assert!(result.is_err(), "WebCheck stub should return error");
    match result.unwrap_err() {
        GrcError::VerificationError { method, .. } => {
            assert_eq!(*method, VerificationMethod::WebCheck);
        }
        other => panic!("expected VerificationError, got: {:?}", other),
    }
}

// =========================================================================
// T7. api_check_stub_returns_error
// =========================================================================

#[test]
fn api_check_stub_returns_error() {
    let verifier = NetworkVerifier::new();

    let result = verifier.verify(VerificationMethod::ApiCheck, "some_api", "");

    assert!(result.is_err(), "ApiCheck stub should return error");
    match result.unwrap_err() {
        GrcError::VerificationError { method, .. } => {
            assert_eq!(*method, VerificationMethod::ApiCheck);
        }
        other => panic!("expected VerificationError, got: {:?}", other),
    }
}

// =========================================================================
// T8. api_registry_check_stub_returns_error
// =========================================================================

#[test]
fn api_registry_check_stub_returns_error() {
    let verifier = NetworkVerifier::new();

    let result = verifier.verify(VerificationMethod::ApiRegistryCheck, "some_registry", "");

    assert!(result.is_err(), "ApiRegistryCheck stub should return error");
    match result.unwrap_err() {
        GrcError::VerificationError { method, .. } => {
            assert_eq!(*method, VerificationMethod::ApiRegistryCheck);
        }
        other => panic!("expected VerificationError, got: {:?}", other),
    }
}

// =========================================================================
// T9. plan_grounding_sweep_blocks_ungrounded
// Calls BasicGrcEngine::ground_plan — currently unimplemented!() (TDD stub)
// =========================================================================

#[test]
fn plan_grounding_sweep_blocks_ungrounded() {
    let engine = BasicGrcEngine::new();

    let plan = "Grounding: file:src/main.rs\nGrounding: file:nonexistent.rs";
    let result: PlanGroundingResult = engine.ground_plan(plan);

    assert!(!result.all_grounded, "plan with missing file should not be all_grounded");
    assert!(
        result.ungrounded.contains(&"file:nonexistent.rs".to_string()),
        "ungrounded should contain the missing claim"
    );
}

// =========================================================================
// T10. plan_grounding_sweep_passes_all_grounded
// Calls BasicGrcEngine::ground_plan — currently unimplemented!()
// =========================================================================

#[test]
fn plan_grounding_sweep_passes_all_grounded() {
    let engine = BasicGrcEngine::new();

    // Plan with only valid claims (actual verification deferred to 2B.4)
    let plan = "Grounding: file:Cargo.toml";
    let result: PlanGroundingResult = engine.ground_plan(plan);

    assert!(result.all_grounded, "plan with valid claims should be all_grounded");
    assert!(result.ungrounded.is_empty(), "ungrounded should be empty");
}

// =========================================================================
// T11. plan_grounding_sweep_empty_plan
// Calls BasicGrcEngine::ground_plan — currently unimplemented!()
// Empty plan must not panic or return UNGROUNDED (R-2B-3).
// =========================================================================

#[test]
fn plan_grounding_sweep_empty_plan() {
    let engine = BasicGrcEngine::new();

    let result: PlanGroundingResult = engine.ground_plan("");

    assert!(result.all_grounded, "empty plan is trivially grounded");
    assert!(result.ungrounded.is_empty());
}

// =========================================================================
// T12. uncertain_claim_auto_resolution
// Uses BasicGrcEngine::verify_claim — currently unimplemented!()
// A "training:some_claim" referencing an existing file should be promoted
// to Verified after FILE_CHECK confirms existence.
// =========================================================================

#[test]
fn uncertain_claim_auto_resolution() {
    let mut tmp = NamedTempFile::new().expect("create temp file");
    write!(tmp, "content").expect("write temp");
    let path = tmp.path().to_str().expect("path to str").to_string();

    let engine = BasicGrcEngine::new();

    // verify_claim should detect "training:" prefix, resolve via FileCheck
    let result: VerificationResult = engine
        .verify_claim(&format!("file:{}", path), "training", "")
        .expect("verify_claim should not error");

    assert!(
        result.passes,
        "existing file claim should be auto-resolved to passing"
    );
}

// =========================================================================
// T13. verification_method_requires_network
// Compile-time correctness check; good regression test.
// =========================================================================

#[test]
fn verification_method_requires_network() {
    assert!(
        VerificationMethod::WebCheck.requires_network(),
        "WebCheck requires network"
    );
    assert!(
        VerificationMethod::ApiCheck.requires_network(),
        "ApiCheck requires network"
    );
    assert!(
        VerificationMethod::ApiRegistryCheck.requires_network(),
        "ApiRegistryCheck requires network"
    );
    assert!(
        !VerificationMethod::FileCheck.requires_network(),
        "FileCheck does not require network"
    );
    assert!(
        !VerificationMethod::SourceCheck.requires_network(),
        "SourceCheck does not require network"
    );
    assert!(
        !VerificationMethod::TestCheck.requires_network(),
        "TestCheck does not require network"
    );
}

// =========================================================================
// T14. verifier_trait_object_safety
// Box<dyn Verifier> must compile — trait is object-safe.
// =========================================================================

#[test]
fn verifier_trait_object_safety() {
    let reader = MockFileReader::new();
    let runner = MockTestRunner::new();
    let _: Box<dyn Verifier> = Box::new(LocalVerifier::new(Box::new(reader), Box::new(runner)));
}

// =========================================================================
// T15. plan_sweep_adversarial_regex
// Calls BasicGrcEngine::ground_plan — currently unimplemented!()
// Regex must match grounding format context, not arbitrary occurrences
// inside comments or string literals.
// =========================================================================

#[test]
fn plan_sweep_adversarial_regex() {
    let engine = BasicGrcEngine::new();

    let plan = "// see file:fake_path for details\nGrounding: file:src/main.rs";
    let result: PlanGroundingResult = engine.ground_plan(plan);

    // Should extract only "file:src/main.rs" (the real claim), not "file:fake_path"
    let claim_strings: Vec<&str> = result.claims.iter().map(|c| c.claim.as_str()).collect();
    assert!(
        !claim_strings.contains(&"file:fake_path"),
        "should not extract file:fake_path from comment"
    );
    assert!(
        claim_strings.contains(&"file:src/main.rs") || result.ungrounded.contains(&"file:src/main.rs".to_string()),
        "should extract file:src/main.rs from Grounding: line"
    );
}

// =========================================================================
// T16. verify_claim_strips_prefix
// Calls BasicGrcEngine::verify_claim — currently unimplemented!()
// The "file:" prefix should be stripped before passing to the Verifier.
// =========================================================================

#[test]
fn verify_claim_strips_prefix() {
    let engine = BasicGrcEngine::new();

    // verify_claim("file:src/main.rs", ...) should strip "file:" prefix
    // and verify "src/main.rs" via FileCheck.
    let result: VerificationResult = engine
        .verify_claim("file:src/main.rs", "", "")
        .expect("verify_claim should not error");

    // The claim in the result should be the stripped path
    assert_eq!(
        result.claim, "src/main.rs",
        "verify_claim should strip 'file:' prefix"
    );
}
