//! B-20A Task 6 — edit similarity hint tests.
//!
//! Decision C: hand-rolled Levenshtein + ratio (no strsim/levenshtein crate).
//! Threshold: emit hint only when ratio > 0.5.
//!
//! Wire-up: `dispatch_edit` in `InProcessToolDispatcher` returns
//! `EditOutcome.similarity_hint = Some(SimilarityHint{...})` when every tier
//! fails (3-tier matcher returns `EditMatchTier::NotMatched`) AND the best
//! line in the source has Levenshtein-ratio > 0.5 against `old_text`.

use elai_core::tool_dispatcher::{EditMatchTier, EditRequest, ToolDispatcher};
use elai_tools::dispatcher::InProcessToolDispatcher;
use elai_tools::string_similarity::{levenshtein, similarity_ratio};
use elai_tools::tiered_validation::{
    CargoDeterminismRunner, DefaultTieredValidator, IntegrationCommandRunner,
    IntegrationRunOutcome, LanguageSyntaxRunner, TieredValidator,
};
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
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

#[test]
fn levenshtein_ratio_mathematical_correctness() {
    // Trivial cases.
    assert_eq!(levenshtein("", ""), 0);
    assert_eq!(levenshtein("abc", ""), 3);
    assert_eq!(levenshtein("", "abc"), 3);
    assert_eq!(levenshtein("kitten", "sitting"), 3); // canonical example
    assert_eq!(levenshtein("flaw", "lawn"), 2);
    // Ratio: identical -> 1.0; disjoint -> 0.0
    assert!((similarity_ratio("hello", "hello") - 1.0).abs() < f32::EPSILON);
    assert!((similarity_ratio("", "") - 1.0).abs() < f32::EPSILON);
    let r = similarity_ratio("kitten", "sitting");
    // 1 - 3/7 ~= 0.5714
    assert!(r > 0.55 && r < 0.60, "expected ~0.57 ratio, got {r}");
}

#[tokio::test]
async fn near_match_emits_hint_above_threshold() {
    // File contains a line very similar to old_text but not byte-identical.
    let mut f = NamedTempFile::new().unwrap();
    write!(
        f,
        "fn alpha() {{}}\n\
         fn beta(x: u32) -> u32 {{ x + 1 }}\n\
         fn gamma() {{}}\n"
    )
    .unwrap();
    f.flush().unwrap();
    let path = f.path().to_path_buf();
    let d = build_dispatcher();
    // old_text: nearly identical to the `beta` line but with different return.
    let req = EditRequest {
        file_path: path.clone(),
        old_text: "fn beta(x: u32) -> u64 { x + 2 }".to_string(),
        new_text: "fn beta(x: u32) -> u64 { x + 99 }".to_string(),
    };
    let out = d.dispatch_edit(req).await.expect("dispatch ok");
    assert_eq!(out.tier, EditMatchTier::NotMatched);
    let hint = out
        .similarity_hint
        .as_ref()
        .expect("near-match must produce hint above 0.5 ratio");
    assert!(
        hint.ratio() > 0.5,
        "ratio {} should be above 0.5 threshold",
        hint.ratio()
    );
    assert!(
        hint.nearest_text.contains("beta"),
        "nearest text should be the beta line, got {:?}",
        hint.nearest_text
    );
    assert!(hint.line_no >= 1 && hint.line_no <= 3);
}

#[tokio::test]
async fn dissimilar_emits_no_hint_below_threshold() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "alpha\nbeta\ngamma\n").unwrap();
    f.flush().unwrap();
    let path = f.path().to_path_buf();
    let d = build_dispatcher();
    let req = EditRequest {
        file_path: path.clone(),
        // Drastically different — every line is below 0.5 ratio.
        old_text: "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq".to_string(),
        new_text: "irrelevant".to_string(),
    };
    let out = d.dispatch_edit(req).await.expect("dispatch ok");
    assert_eq!(out.tier, EditMatchTier::NotMatched);
    assert!(
        out.similarity_hint.is_none(),
        "below-threshold ratio must NOT emit hint, got {:?}",
        out.similarity_hint
    );
}

#[tokio::test]
async fn exact_match_no_hint() {
    // Successful exact-tier match must NOT carry a similarity_hint.
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "hello world").unwrap();
    f.flush().unwrap();
    let path = f.path().to_path_buf();
    let d = build_dispatcher();
    let req = EditRequest {
        file_path: path.clone(),
        old_text: "world".to_string(),
        new_text: "Rust".to_string(),
    };
    let out = d.dispatch_edit(req).await.expect("dispatch ok");
    assert_eq!(out.tier, EditMatchTier::Exact);
    assert!(out.similarity_hint.is_none());
}
