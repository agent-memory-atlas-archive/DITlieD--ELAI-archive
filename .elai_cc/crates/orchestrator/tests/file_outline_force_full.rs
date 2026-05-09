//! B-20A Task 7 — file outline force_full + 50% budget guard tests.
//!
//! Decision D: caller-side enforcement. The `ToolDispatcher` trait stays
//! unchanged. `InProcessToolDispatcher::dispatch_file_read_with_force_full`
//! is a new inherent method (not on the trait) that takes `force_full: bool`.
//! `enforce_file_read_budget_guard` (`elai_orchestrator::task_runner`) is
//! the caller-side helper that returns a hint when the requested full read
//! would exceed 50% of the remaining context budget.
//!
//! Tests:
//!   1. >300 lines + force_full=false  -> outline only (no full content)
//!   2. >300 lines + force_full=true   -> full content returned
//!   3. enforce_budget guard fires when (total_bytes * 4) > 0.5 * budget

use std::io::Write;
use std::sync::Arc;

use elai_orchestrator::task_runner::{
    enforce_file_read_budget_guard, FileReadBudgetVerdict,
};
use elai_tools::dispatcher::InProcessToolDispatcher;
use elai_tools::tiered_validation::{
    CargoDeterminismRunner, DefaultTieredValidator, IntegrationCommandRunner,
    IntegrationRunOutcome, LanguageSyntaxRunner, TieredValidator,
};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use tempfile::NamedTempFile;

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

#[tokio::test]
async fn outline_default_for_files_over_300_lines() {
    let mut f = NamedTempFile::new().unwrap();
    for i in 0..400 {
        writeln!(f, "fn generated_{i}() {{}}").unwrap();
    }
    f.flush().unwrap();
    let path = f.path().to_path_buf();
    let d = build_dispatcher();
    let view = d
        .dispatch_file_read_with_force_full(path, 300, false)
        .await
        .expect("ok");
    assert!(view.outline_only, "outline should be on by default for 400-line file");
    assert!(view.full_content.is_none(), "no full content when outline_only");
    assert_eq!(view.signatures.len(), 400);
}

#[tokio::test]
async fn force_full_returns_full_content_above_threshold() {
    let mut f = NamedTempFile::new().unwrap();
    for i in 0..400 {
        writeln!(f, "fn generated_{i}() {{}}").unwrap();
    }
    f.flush().unwrap();
    let path = f.path().to_path_buf();
    let d = build_dispatcher();
    let view = d
        .dispatch_file_read_with_force_full(path, 300, true)
        .await
        .expect("ok");
    assert!(!view.outline_only, "force_full overrides outline mode");
    assert!(view.full_content.is_some(), "full content must be returned");
    assert!(
        view.full_content.as_deref().unwrap().contains("fn generated_399"),
        "all lines must round-trip"
    );
}

#[test]
fn enforce_budget_guard_fires_when_full_read_exceeds_50_percent() {
    // Suppose file is 50_000 bytes (~12_500 tokens at 4 bytes/token), and the
    // remaining budget is 20_000 tokens. 12_500 > 0.5 * 20_000 = 10_000 ⇒
    // the guard MUST trigger.
    let verdict =
        enforce_file_read_budget_guard(50_000, 20_000);
    match verdict {
        FileReadBudgetVerdict::Block { hint, budget_remaining } => {
            assert_eq!(budget_remaining, 20_000);
            assert!(
                hint.contains("50%") || hint.contains("budget"),
                "hint must mention budget; got {hint:?}"
            );
        }
        other => panic!("expected Block verdict, got {other:?}"),
    }

    // Allow path: small file, ample budget.
    let verdict = enforce_file_read_budget_guard(2_000, 20_000);
    assert!(matches!(verdict, FileReadBudgetVerdict::Allow));
}
