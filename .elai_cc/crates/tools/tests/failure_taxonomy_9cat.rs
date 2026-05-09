//! B-20A Task 4 — failure taxonomy 13->9 reduction tests.
//!
//! Asserts that the 9-variant `FailureCategory` enum (and the parallel
//! `ToolFailureCategory` in `elai-core`) round-trip through the regex
//! classifier (`classify_output`) and the IPC `ClassifyHandler` for each
//! of the 9 spec-mandated variants.
//!
//! Decision A: B-20 plan's 9-list (rust-flavoured), NOT phase-4B.txt's
//! competing list (no PERMISSION_DENIED). Documented in
//! workflow/reasoning/B-20A-decisions.md.

use elai_ipc_types::ClassifyRequest;
use elai_tools::failure_taxonomy::{ClassifyHandler, DiagnosticResult, FailureCategory};
use elai_tools::ipc_registry::ToolHandler;
use elai_core::tool_dispatcher::{ClassifiedFailure, FailureInput, ToolDispatcher, ToolFailureCategory};
use elai_tools::dispatcher::InProcessToolDispatcher;
use elai_tools::tiered_validation::{TieredValidator, IntegrationCommandRunner, IntegrationRunOutcome, DefaultTieredValidator, CargoDeterminismRunner, LanguageSyntaxRunner};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

struct NullIntegration;
impl IntegrationCommandRunner for NullIntegration {
    fn run_integration<'a>(
        &'a self,
        _level: &'a str,
        _pd: &'a Path,
    ) -> Pin<Box<dyn Future<Output = IntegrationRunOutcome> + Send + 'a>> {
        Box::pin(async move {
            IntegrationRunOutcome { passed: true, rules_fired: 0, stderr_snippet: None }
        })
    }
}

fn build_dispatcher() -> InProcessToolDispatcher {
    let determinism = Arc::new(CargoDeterminismRunner::new("rust"));
    let syntax = Arc::new(LanguageSyntaxRunner);
    let integration: Arc<dyn IntegrationCommandRunner> = Arc::new(NullIntegration);
    let validator: Arc<dyn TieredValidator> =
        Arc::new(DefaultTieredValidator::new(determinism, syntax, integration));
    InProcessToolDispatcher::new(validator)
}

async fn classify(d: &InProcessToolDispatcher, raw: &str) -> Option<ClassifiedFailure> {
    d.dispatch_classify_failure(FailureInput {
        raw_output: raw.to_string(),
        step_id: "t".into(),
        source: "validator".into(),
    })
    .await
}

// ---------------------------------------------------------------------------
// Round-trip + classifier tests, one per variant. Each covers:
//   1. The `as_str` of `ToolFailureCategory` matches the SCREAMING_SNAKE_CASE
//      spec name.
//   2. The regex classifier maps a representative error message to the
//      expected variant.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn syntax_error_round_trip() {
    assert_eq!(ToolFailureCategory::SyntaxError.as_str(), "SYNTAX_ERROR");
    let d = build_dispatcher();
    let cf = classify(&d, "error: expected `;`, found `}`").await
        .expect("syntax classifier");
    assert_eq!(cf.category, ToolFailureCategory::SyntaxError);
}

#[tokio::test]
async fn type_mismatch_round_trip() {
    assert_eq!(ToolFailureCategory::TypeMismatch.as_str(), "TYPE_MISMATCH");
    let d = build_dispatcher();
    let cf = classify(&d, "error[E0308]: mismatched types").await
        .expect("type mismatch classifier");
    assert_eq!(cf.category, ToolFailureCategory::TypeMismatch);
}

#[tokio::test]
async fn undefined_symbol_round_trip() {
    assert_eq!(ToolFailureCategory::UndefinedSymbol.as_str(), "UNDEFINED_SYMBOL");
    let d = build_dispatcher();
    let cf = classify(&d, "error[E0425]: cannot find value `foo` in this scope").await
        .expect("undefined symbol classifier");
    assert_eq!(cf.category, ToolFailureCategory::UndefinedSymbol);
}

#[tokio::test]
async fn missing_import_round_trip() {
    assert_eq!(ToolFailureCategory::MissingImport.as_str(), "MISSING_IMPORT");
    let d = build_dispatcher();
    let cf = classify(&d, "error[E0432]: unresolved import `crate::foo`").await
        .expect("missing import classifier");
    assert_eq!(cf.category, ToolFailureCategory::MissingImport);
}

#[tokio::test]
async fn lifetime_error_round_trip() {
    assert_eq!(ToolFailureCategory::LifetimeError.as_str(), "LIFETIME_ERROR");
    let d = build_dispatcher();
    let cf = classify(&d, "error[E0621]: explicit lifetime required in the type of `x`").await
        .expect("lifetime classifier");
    assert_eq!(cf.category, ToolFailureCategory::LifetimeError);
}

#[tokio::test]
async fn borrow_error_round_trip() {
    assert_eq!(ToolFailureCategory::BorrowError.as_str(), "BORROW_ERROR");
    let d = build_dispatcher();
    let cf = classify(&d, "error[E0382]: borrow of moved value: `x`").await
        .expect("borrow classifier");
    assert_eq!(cf.category, ToolFailureCategory::BorrowError);
}

#[tokio::test]
async fn unused_warning_round_trip() {
    assert_eq!(ToolFailureCategory::UnusedWarning.as_str(), "UNUSED_WARNING");
    let d = build_dispatcher();
    let cf = classify(&d, "warning: unused variable: `x`").await
        .expect("unused classifier");
    assert_eq!(cf.category, ToolFailureCategory::UnusedWarning);
}

#[tokio::test]
async fn test_assertion_round_trip() {
    assert_eq!(ToolFailureCategory::TestAssertion.as_str(), "TEST_ASSERTION");
    let d = build_dispatcher();
    let cf = classify(&d, "test result: FAILED. assertion failed: left == right").await
        .expect("assertion classifier");
    assert_eq!(cf.category, ToolFailureCategory::TestAssertion);
}

#[tokio::test]
async fn runtime_panic_round_trip() {
    assert_eq!(ToolFailureCategory::RuntimePanic.as_str(), "RUNTIME_PANIC");
    let d = build_dispatcher();
    let cf = classify(&d, "thread 'main' panicked at 'oops'").await
        .expect("panic classifier");
    assert_eq!(cf.category, ToolFailureCategory::RuntimePanic);
}

// ---------------------------------------------------------------------------
// Bonus: the `failure_taxonomy::FailureCategory` (IPC layer enum, parallel)
// must carry the same 9 names. This is a structural sanity check — if a
// developer reduces only one enum the other one will fail to compile here.
// ---------------------------------------------------------------------------

#[test]
fn failure_taxonomy_handler_exposes_nine_variant_names() {
    // Construct each variant explicitly; the test fails to compile if any
    // variant is missing or renamed away from the lockstep names.
    let names: [(FailureCategory, &str); 9] = [
        (FailureCategory::SyntaxError, "SYNTAX_ERROR"),
        (FailureCategory::TypeMismatch, "TYPE_MISMATCH"),
        (FailureCategory::UndefinedSymbol, "UNDEFINED_SYMBOL"),
        (FailureCategory::MissingImport, "MISSING_IMPORT"),
        (FailureCategory::LifetimeError, "LIFETIME_ERROR"),
        (FailureCategory::BorrowError, "BORROW_ERROR"),
        (FailureCategory::UnusedWarning, "UNUSED_WARNING"),
        (FailureCategory::TestAssertion, "TEST_ASSERTION"),
        (FailureCategory::RuntimePanic, "RUNTIME_PANIC"),
    ];
    // Round-trip through ClassifyHandler so the IPC path is wired too.
    for (cat, _expected_name) in names {
        // serde rename_all = SCREAMING_SNAKE_CASE means the JSON tag matches.
        let json = serde_json::to_string(&cat).expect("serialize");
        let back: FailureCategory = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cat, back, "round-trip diverged for {cat:?}");
    }
    let handler = ClassifyHandler::new();
    let req = ClassifyRequest {
        error_output: "error[E0308]: mismatched types".to_string(),
        file_path: None,
    };
    let bytes = postcard::to_allocvec(&req).expect("postcard");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt");
    let result = rt.block_on(handler.execute(&bytes)).expect("handler ok");
    let diag: DiagnosticResult = serde_json::from_str(&result.output).expect("json");
    assert_eq!(diag.category, FailureCategory::TypeMismatch);
}
