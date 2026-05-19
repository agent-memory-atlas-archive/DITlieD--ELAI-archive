//! Cosmos briefing compiler — B-42 Phase 2.7.
//!
//! Produces an `AVAILABLE_COSMOS` Markdown block injected into agent prompts
//! (designer / idea-framer / per-level-auditor). Filters by `ProjectClass`
//! and `BriefingTier`, scores by `tier_relevance`, and truncates to a token
//! budget.
//!
//! # Token budget
//!
//! The budget is a SOFT cap measured in characters as a token proxy
//! (≈1 token ≈ 4 chars for English Markdown). Default
//! [`DEFAULT_BRIEFING_BUDGET_TOKENS`] = 1500 tokens ≈ 6000 chars. When the
//! compiled body would exceed the cap, lower-`tier_relevance` artifacts are
//! dropped first; the per-artifact body itself is then truncated with a
//! `... [truncated, N bytes]` footer (deterministic — no `ShortCoder`
//! dependency since the knowledge crate's ShortCoder needs an ONNX model
//! loaded; the briefing path runs even when the model is absent).
//!
//! Future enhancement: when the host has a ShortCoder loaded, swap the
//! deterministic suffix for an actual summarisation. Tracked under
//! `workflow/reasoning/B-42-decisions.md` DECISION-SHORTCODER-FALLBACK.

use std::collections::BTreeMap;

use elai_knowledge::cosmos_worker::{
    floor_char_boundary, CosmosArtifact, CosmosError, CosmosStatus, CosmosWorker,
};
use thiserror::Error;
use tracing::warn;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Chars-per-token heuristic for budget conversion. English Markdown averages
/// ≈4 chars/token under cl100k/bpe-like tokenizers. Deliberately conservative.
pub const BUDGET_CHARS_PER_TOKEN: usize = 4;

/// Default `tier_relevance` filter threshold. Artifacts whose relevance for
/// the requested tier is below this are excluded from the briefing.
pub const DEFAULT_TIER_RELEVANCE_THRESHOLD: f32 = 0.4;

/// Suffix appended to truncated bodies (Markdown-safe).
pub const TRUNCATION_SUFFIX: &str = "\n\n... [truncated, {n} bytes]\n";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Briefing tier — coarse band describing the consumer of the briefing.
///
/// Mirrors `research_tree::DecompositionNodeKind` plus a top-level `Plan`
/// band. Tier names also appear as keys in `CosmosArtifactSpec.tier_relevance`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BriefingTier {
    Concepts,
    Phases,
    Steps,
    BabySteps,
    Plan,
}

impl BriefingTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            BriefingTier::Concepts => "concepts",
            BriefingTier::Phases => "phases",
            BriefingTier::Steps => "steps",
            BriefingTier::BabySteps => "baby_steps",
            BriefingTier::Plan => "plan",
        }
    }
}

#[derive(Debug, Error)]
pub enum BriefingError {
    #[error("cosmos error: {0}")]
    Cosmos(#[from] CosmosError),

    #[error("budget too small to include any artifact ({budget_chars} chars)")]
    BudgetTooSmall { budget_chars: usize },
}

// ---------------------------------------------------------------------------
// Compile entrypoint
// ---------------------------------------------------------------------------

/// Compile a cosmos briefing for the given `project_class` (snake_case to
/// match `ProjectClass` serde) at the given `tier`.
///
/// Returns the Markdown block to splice into the agent prompt. When no
/// artifacts qualify, returns an empty string (NOT an error — callers can
/// no-op gracefully).
pub fn compile_cosmos_briefing(
    worker: &CosmosWorker,
    project_class: &str,
    tier: BriefingTier,
    tier_relevance_threshold: f32,
    budget_tokens: usize,
) -> Result<String, BriefingError> {
    let budget_chars = budget_tokens.saturating_mul(BUDGET_CHARS_PER_TOKEN);
    if budget_chars < 256 {
        return Err(BriefingError::BudgetTooSmall { budget_chars });
    }

    // Score each manifest entry by tier_relevance for the requested tier.
    let tier_key = tier.as_str();
    let mut candidates: Vec<(String, f32)> = Vec::new();
    for spec in &worker.manifest().artifacts {
        if !spec.project_classes.iter().any(|c| c == project_class) {
            continue;
        }
        let score = spec
            .tier_relevance
            .get(tier_key)
            .copied()
            .unwrap_or(0.0);
        if score < tier_relevance_threshold {
            continue;
        }
        candidates.push((spec.name.clone(), score));
    }
    // Highest-relevance first; ties broken by name for determinism.
    candidates.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    if candidates.is_empty() {
        return Ok(String::new());
    }

    let header = format!(
        "## AVAILABLE_COSMOS (project_class={}, tier={})\n",
        project_class,
        tier.as_str()
    );
    let mut out = String::with_capacity(budget_chars.min(8192));
    out.push_str(&header);

    let mut remaining: i64 = budget_chars as i64 - out.len() as i64;
    let mut included = 0usize;

    for (name, score) in &candidates {
        if remaining < 256 {
            break;
        }
        let artifact = match worker.cosmos_get(name) {
            Ok(a) => a,
            Err(CosmosError::MissingBody(_)) => continue,
            Err(e) => {
                warn!(
                    target: "cosmos",
                    artifact = %name,
                    error = %e,
                    "cosmos artifact unreadable in briefing — skipped",
                );
                continue;
            }
        };
        let block = render_artifact_block(&artifact, *score, remaining as usize);
        if (block.len() as i64) > remaining {
            // Final artifact, truncate aggressively rather than overflow.
            // Snap DOWN to a char boundary so multibyte UTF-8 (curly quotes,
            // em-dashes, CJK from web research) does not panic.
            let original_block_len = block.len();
            let raw_cut = remaining.saturating_sub(64).max(0) as usize;
            let cut = floor_char_boundary(&block, raw_cut);
            let mut tail = block;
            tail.truncate(cut);
            tail.push_str(&truncation_suffix(block_len_overflow(
                original_block_len,
                tail.len(),
            )));
            out.push_str(&tail);
            included += 1;
            break;
        }
        out.push_str(&block);
        remaining -= block.len() as i64;
        included += 1;
    }

    if included == 0 {
        return Ok(String::new());
    }
    Ok(out)
}

fn render_artifact_block(
    artifact: &CosmosArtifact,
    score: f32,
    remaining: usize,
) -> String {
    let status = artifact.status;
    let status_str = status.as_str();
    let header = format!(
        "\n### {name} (status={status}, refreshed_at={ts}, tier_relevance={score:.2})\n",
        name = artifact.spec.name,
        status = status_str,
        ts = artifact.refreshed_at,
        score = score,
    );
    // Reserve ~64 chars for header padding/safety.
    let body_budget = remaining.saturating_sub(header.len() + 64);
    let body = if artifact.body.len() > body_budget && body_budget > 0 {
        // Snap DOWN to a char boundary; raw byte-slice panics mid-codepoint.
        let cut = floor_char_boundary(&artifact.body, body_budget);
        let mut b = artifact.body[..cut].to_string();
        b.push_str(&truncation_suffix(artifact.body.len() - cut));
        b
    } else {
        artifact.body.clone()
    };
    let mut out = String::with_capacity(header.len() + body.len() + 8);
    out.push_str(&header);
    out.push_str(&body);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if matches!(status, CosmosStatus::Decayed) {
        out.push_str("> WARN: cosmos artifact is DECAYED; treat as stale background.\n");
    }
    out
}

fn truncation_suffix(removed: usize) -> String {
    TRUNCATION_SUFFIX.replace("{n}", &removed.to_string())
}

/// Byte count removed when an artifact block was truncated. The suffix line
/// `"... [truncated, N bytes]"` was reporting 0 for every truncation event
/// (B-42 audit BLOCK-2: dead stub). Now returns the real delta between the
/// pre-truncation block length and the post-truncation `String::len`.
fn block_len_overflow(original: usize, truncated: usize) -> usize {
    original.saturating_sub(truncated)
}

/// Convenience: build a per-tier map suitable for callers that want to bind
/// a single closure that resolves "give me the briefing for tier X".
pub fn precompile_all_tiers(
    worker: &CosmosWorker,
    project_class: &str,
    tier_relevance_threshold: f32,
    budget_tokens: usize,
) -> Result<BTreeMap<&'static str, String>, BriefingError> {
    let mut out = BTreeMap::new();
    for t in [
        BriefingTier::Concepts,
        BriefingTier::Phases,
        BriefingTier::Steps,
        BriefingTier::BabySteps,
        BriefingTier::Plan,
    ] {
        let body = compile_cosmos_briefing(
            worker,
            project_class,
            t,
            tier_relevance_threshold,
            budget_tokens,
        )?;
        out.insert(t.as_str(), body);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_knowledge::cosmos_worker::{
        CosmosManifest, CosmosWorker, FixedClock, MockResearch, DEFAULT_BRIEFING_BUDGET_TOKENS,
    };
    use std::sync::Arc;

    async fn make_worker(body: &str) -> (tempfile::TempDir, CosmosWorker) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let toml = r#"
[[artifacts]]
name = "ios_hig_summary"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "monthly"
research_query = "Apple Human Interface Guidelines"
tier_relevance = { concepts = 0.9, phases = 0.7, steps = 0.4 }

[[artifacts]]
name = "app_store_policies"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "daily"
research_query = "App Store policies"
tier_relevance = { phases = 0.9, steps = 0.6 }

[[artifacts]]
name = "web_a11y_wcag"
project_classes = ["web_app"]
refresh_cadence = "quarterly"
research_query = "WCAG 2.2"
tier_relevance = { concepts = 0.8, steps = 0.5 }
"#;
        let manifest = CosmosManifest::from_toml_str(toml).expect("manifest");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new(body, "T1"));
        let worker = CosmosWorker::new(tmp.path().to_path_buf(), manifest, clock, research)
            .expect("worker");
        let r = worker.tick().await;
        assert_eq!(r.refreshed.len(), 3);
        (tmp, worker)
    }

    #[tokio::test]
    async fn briefing_filters_by_project_class_and_tier() {
        let (_tmp, w) = make_worker("# body content for relevant slice").await;
        let out = compile_cosmos_briefing(
            &w,
            "consumer_mobile_app",
            BriefingTier::Concepts,
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            DEFAULT_BRIEFING_BUDGET_TOKENS,
        )
        .expect("briefing");
        assert!(out.contains("ios_hig_summary"), "{out}");
        assert!(!out.contains("web_a11y_wcag"), "wrong class included");
        // app_store_policies has no `concepts` key -> default 0.0 -> excluded.
        assert!(!out.contains("app_store_policies"), "{out}");
    }

    #[tokio::test]
    async fn briefing_empty_when_no_match() {
        let (_tmp, w) = make_worker("# body").await;
        let out = compile_cosmos_briefing(
            &w,
            "research_tool",
            BriefingTier::Concepts,
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            DEFAULT_BRIEFING_BUDGET_TOKENS,
        )
        .expect("briefing");
        assert_eq!(out, "");
    }

    #[tokio::test]
    async fn briefing_respects_token_budget() {
        let big_body = "x".repeat(4096);
        let (_tmp, w) = make_worker(&big_body).await;
        // 200 tokens ≈ 800 chars — only header + maybe partial body fits.
        let out = compile_cosmos_briefing(
            &w,
            "consumer_mobile_app",
            BriefingTier::Phases,
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            200,
        )
        .expect("briefing");
        assert!(out.len() <= 200 * BUDGET_CHARS_PER_TOKEN + 256, "len={}", out.len());
    }

    #[tokio::test]
    async fn briefing_too_small_budget_errors() {
        let (_tmp, w) = make_worker("# body").await;
        let r = compile_cosmos_briefing(
            &w,
            "consumer_mobile_app",
            BriefingTier::Concepts,
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            10,
        );
        assert!(matches!(r, Err(BriefingError::BudgetTooSmall { .. })));
    }

    #[tokio::test]
    async fn truncation_suffix_reports_nonzero_byte_count() {
        // Body large enough to force per-artifact truncation under a tight
        // budget. The suffix `... [truncated, N bytes]` MUST report a
        // non-zero N (B-42 audit BLOCK-2: previously hard-coded 0).
        let big_body = "x".repeat(8192);
        let (_tmp, w) = make_worker(&big_body).await;
        let out = compile_cosmos_briefing(
            &w,
            "consumer_mobile_app",
            BriefingTier::Phases,
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            200,
        )
        .expect("briefing");
        assert!(out.contains("[truncated,"), "no truncation suffix in {out}");
        assert!(
            !out.contains("[truncated, 0 bytes]"),
            "block_len_overflow regression: suffix still reports 0 bytes\n{out}"
        );
    }

    #[tokio::test]
    async fn briefing_does_not_panic_on_multibyte_body() {
        // Multibyte body: each "日本語" = 9 bytes. Tight budget forces
        // truncation. Naive byte-index truncate (pre-fix) panics on a
        // mid-codepoint cut; floor_char_boundary must snap DOWN.
        let big_body = "日本語".repeat(1024);
        let (_tmp, w) = make_worker(&big_body).await;
        let out = compile_cosmos_briefing(
            &w,
            "consumer_mobile_app",
            BriefingTier::Phases,
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            200,
        )
        .expect("briefing");
        // Must produce valid UTF-8 (String construction would have panicked
        // otherwise) and must include the truncation suffix.
        assert!(out.contains("[truncated,"), "{out}");
    }

    #[tokio::test]
    async fn precompile_all_tiers_returns_five_entries() {
        let (_tmp, w) = make_worker("# body").await;
        let map = precompile_all_tiers(
            &w,
            "consumer_mobile_app",
            DEFAULT_TIER_RELEVANCE_THRESHOLD,
            DEFAULT_BRIEFING_BUDGET_TOKENS,
        )
        .expect("precompile");
        assert_eq!(map.len(), 5);
        assert!(map.contains_key("concepts"));
        assert!(map.contains_key("phases"));
        assert!(map.contains_key("steps"));
        assert!(map.contains_key("baby_steps"));
        assert!(map.contains_key("plan"));
    }
}
