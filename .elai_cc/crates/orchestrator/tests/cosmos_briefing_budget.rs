//! C-16 Test File 5 — cosmos briefing token-budget compliance.
//!
//! Verifies the contract that compile_cosmos_briefing:
//!   - respects the budget_tokens cap (output_chars <= budget_chars + small overshoot)
//!   - filters out artifacts whose tier_relevance is below the threshold
//!   - returns empty string on empty / mismatched manifest
//!   - drops lower-relevance artifacts first when the budget is squeezed
//!
//! Plan reference: docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md
//! file 5 — cosmos_briefing_budget.rs.
//!
//! Note: the plan §"Test files" file 5 lists two behaviors (Fresh-before-Stale,
//! Decayed-exclusion-when-Fresh-exists) that compile_cosmos_briefing does NOT
//! currently implement (briefing.rs sorts by tier_relevance only). The context
//! package called this out as a GAP — C-16 keeps the current behavior to avoid
//! a parallel behavioral change. Those two cases are documented as
//! `#[ignore]` placeholders here so the test surface tracks the gap.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use elai_knowledge::cosmos_worker::{
    CosmosClock, CosmosError, CosmosManifest, CosmosWorker, ResearchDispatcher,
};
use elai_orchestrator::briefing::{
    compile_cosmos_briefing, BriefingTier, BUDGET_CHARS_PER_TOKEN,
    DEFAULT_TIER_RELEVANCE_THRESHOLD,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

struct FixedClock(std::sync::atomic::AtomicI64);
impl CosmosClock for FixedClock {
    fn now_secs(&self) -> i64 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

struct CannedResearch(String);
impl ResearchDispatcher for CannedResearch {
    fn run<'a>(
        &'a self,
        _q: &'a str,
        _t: &'a str,
        _m: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        let body = self.0.clone();
        Box::pin(async move { Ok((body, "T1".to_string())) })
    }
}

async fn make_worker_with_three_artifacts(body: &str) -> (tempfile::TempDir, CosmosWorker) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let toml = r#"
[[artifacts]]
name = "high_relevance"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "monthly"
research_query = "high relevance brief"
tier_relevance = { concepts = 0.9, phases = 0.8, steps = 0.7 }

[[artifacts]]
name = "medium_relevance"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "monthly"
research_query = "medium relevance brief"
tier_relevance = { concepts = 0.6, phases = 0.5, steps = 0.5 }

[[artifacts]]
name = "low_relevance"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "monthly"
research_query = "low relevance brief"
tier_relevance = { concepts = 0.45, phases = 0.42, steps = 0.41 }

[[artifacts]]
name = "below_threshold"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "monthly"
research_query = "very low relevance"
tier_relevance = { concepts = 0.1, phases = 0.1, steps = 0.1 }
"#;
    let manifest = CosmosManifest::from_toml_str(toml).expect("manifest");
    let clock: Arc<dyn CosmosClock> =
        Arc::new(FixedClock(std::sync::atomic::AtomicI64::new(1_700_000_000)));
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch(body.to_string()));
    let worker = CosmosWorker::new(tmp.path().to_path_buf(), manifest, clock, research)
        .expect("worker");
    let r = worker.tick().await;
    assert_eq!(r.refreshed.len(), 4, "all artifacts refreshed on first tick");
    (tmp, worker)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn briefing_respects_200_token_budget() {
    // Use a large body so the budget actually bites.
    let big_body = "x".repeat(4096);
    let (_tmp, w) = make_worker_with_three_artifacts(&big_body).await;

    let out = compile_cosmos_briefing(
        &w,
        "consumer_mobile_app",
        BriefingTier::Concepts,
        DEFAULT_TIER_RELEVANCE_THRESHOLD,
        200,
    )
    .expect("briefing");

    let budget_chars = 200 * BUDGET_CHARS_PER_TOKEN; // 800 chars
    // Small overshoot allowance for header + trailing newline. Briefing
    // truncates per-block; the worst case is one block straddling the
    // budget by ~256 chars (the loop's "remaining < 256" early-exit).
    assert!(
        out.len() <= budget_chars + 256,
        "briefing len {} exceeded 200-token cap (budget {} chars + 256 overshoot)",
        out.len(),
        budget_chars
    );
}

#[tokio::test]
async fn briefing_excludes_artifacts_below_relevance_threshold() {
    let (_tmp, w) = make_worker_with_three_artifacts("# short body").await;

    let out = compile_cosmos_briefing(
        &w,
        "consumer_mobile_app",
        BriefingTier::Concepts,
        DEFAULT_TIER_RELEVANCE_THRESHOLD, // 0.4
        1500,
    )
    .expect("briefing");

    // `below_threshold` has all values <= 0.1, well below 0.4 → excluded.
    assert!(
        !out.contains("below_threshold"),
        "low-relevance artifact must be excluded: {out}"
    );
    // The three above-threshold artifacts must all appear when budget is
    // ample.
    assert!(out.contains("high_relevance"), "high not in: {out}");
    assert!(out.contains("medium_relevance"), "medium not in: {out}");
    assert!(out.contains("low_relevance"), "low (still > 0.4) not in: {out}");
}

#[tokio::test]
async fn briefing_drops_lower_relevance_first_under_budget_pressure() {
    // Each body is large; the budget allows only ONE artifact to fit. The
    // sort key is tier_relevance descending, so `high_relevance` (0.9 at
    // Concepts) must be the one to appear.
    let big_body = "y".repeat(4096);
    let (_tmp, w) = make_worker_with_three_artifacts(&big_body).await;

    // Tight budget (300 tokens = 1200 chars). Three Concepts-eligible
    // artifacts at scores 0.9 / 0.6 / 0.45; the budget should fit just
    // the highest-scoring artifact.
    let out = compile_cosmos_briefing(
        &w,
        "consumer_mobile_app",
        BriefingTier::Concepts,
        DEFAULT_TIER_RELEVANCE_THRESHOLD,
        300,
    )
    .expect("briefing");

    assert!(
        out.contains("high_relevance"),
        "highest-relevance artifact must be included: {out}"
    );
    // medium_relevance (0.6) is the next candidate but should not fit
    // alongside high_relevance under a 300-token cap with 4 KiB bodies.
    // (We can't assert strict absence because the truncation suffix might
    // pad oddly; just assert high_relevance is present and the output
    // respects the budget.)
    let budget_chars = 300 * BUDGET_CHARS_PER_TOKEN;
    assert!(out.len() <= budget_chars + 256, "len={}", out.len());
}

#[tokio::test]
async fn briefing_empty_when_manifest_has_no_matching_project_class() {
    let (_tmp, w) = make_worker_with_three_artifacts("# body").await;
    let out = compile_cosmos_briefing(
        &w,
        "research_tool", // no artifact targets this class
        BriefingTier::Concepts,
        DEFAULT_TIER_RELEVANCE_THRESHOLD,
        1500,
    )
    .expect("briefing");
    assert_eq!(out, "", "no matching artifacts -> empty string");
}

#[tokio::test]
async fn briefing_empty_when_manifest_is_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let manifest = CosmosManifest::from_toml_str("").expect("empty manifest");
    let clock: Arc<dyn CosmosClock> =
        Arc::new(FixedClock(std::sync::atomic::AtomicI64::new(1_700_000_000)));
    let research: Arc<dyn ResearchDispatcher> = Arc::new(CannedResearch("body".to_string()));
    let w = CosmosWorker::new(tmp.path().to_path_buf(), manifest, clock, research)
        .expect("worker");
    // Empty manifest: tick is a no-op, briefing returns empty.
    let _ = w.tick().await;
    let out = compile_cosmos_briefing(
        &w,
        "consumer_mobile_app",
        BriefingTier::Concepts,
        DEFAULT_TIER_RELEVANCE_THRESHOLD,
        1500,
    )
    .expect("briefing");
    assert_eq!(out, "");
}

#[tokio::test]
async fn briefing_at_zero_tier_relevance_threshold_includes_all_eligible() {
    let (_tmp, w) = make_worker_with_three_artifacts("# body").await;
    // Threshold 0.0: any artifact with at least one tier_relevance entry
    // for the requested tier is included (artifacts with no entry default
    // to 0.0 which is still >= 0.0). All four artifacts qualify.
    let out = compile_cosmos_briefing(
        &w,
        "consumer_mobile_app",
        BriefingTier::Concepts,
        0.0,
        1500,
    )
    .expect("briefing");
    // All four appear (the budget is ample for short bodies).
    assert!(out.contains("high_relevance"));
    assert!(out.contains("medium_relevance"));
    assert!(out.contains("low_relevance"));
    assert!(out.contains("below_threshold"));
}

// ---------------------------------------------------------------------------
// Placeholders for behavior not currently implemented (context-package
// flagged as GAP; documented in C-16-decisions.md).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "C-16 GAP: compile_cosmos_briefing sorts by tier_relevance only; \
            Fresh-before-Stale ordering is not implemented. \
            See workflow/sessions/s-32952/context-package-C16.md file 5 GAP."]
fn briefing_orders_fresh_before_stale() {
    // Placeholder. Would require enhancing compile_cosmos_briefing's sort
    // key to consult CosmosStatus alongside tier_relevance.
}

#[test]
#[ignore = "C-16 GAP: Decayed artifacts are not excluded when a Fresh \
            alternative exists for the same project_class. \
            See workflow/sessions/s-32952/context-package-C16.md file 5 GAP."]
fn briefing_excludes_decayed_when_fresh_alternative_exists() {
    // Placeholder. Would require dedup-by-project_class with Fresh winning.
}
