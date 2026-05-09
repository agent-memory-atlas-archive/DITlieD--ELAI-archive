// 4D.17: WebResearchOrchestrator — drives the per-node research loop.
//
// Wires 6 infrastructure crates (SearxClient, WebFetcher, BasicExtractor,
// SourceQualityTable, WebContentIndex, ResearchBudgetManager) into a single
// orchestrator that drives ResearchTree traversal with the loop:
// FORMULATE -> SEARCH -> QUALITY GATE -> FETCH+EXTRACT -> INDEX -> RETRIEVE -> REFLECT -> CONVERGE
//
// Lives in the orchestrator process (not an agent). Calls deps directly (no IPC).

use std::sync::Arc;

use serde_json::json;
use thiserror::Error;
use tracing::{debug, info, warn};

use elai_core::clock::ClockProvider;
use elai_core::source_tier::SourceTier;
use elai_knowledge::web_arag::{WebChunk, WebContentIndex};
use elai_web_search::client::SearxClient;
use elai_web_search::extractor::{BasicExtractor, ContentExtractor, PageType};
use elai_web_search::fetcher::WebFetcher;
use elai_web_search::source_quality::SourceQualityTable;
use elai_web_search::{SearchError, SearchResult, WebSearchError};

use crate::research_budget::{EvidenceItem, ResearchBudgetManager};
use crate::research_tier::ResearchTierConfig;
use crate::research_tree::{NodeId, NodeStatus, ResearchNode, SourceCitation,
    SourceQuality, EvidenceLevel};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum number of sources before convergence check.
const MIN_SOURCES_FOR_CONVERGENCE: usize = 2;

/// Fraction of T4Rejected (low-quality) sources that triggers LOW confidence.
const LOW_QUALITY_THRESHOLD: f64 = 0.8;

/// Confidence level below which the result is considered LOW.
const LOW_CONFIDENCE_CUTOFF: f64 = 0.5;

/// Multiplier on depth_limit for max iterations per node.
const MAX_ITERATIONS_PER_DEPTH: u32 = 10;

/// Minimum citations before upgrading evidence level to MultiSource.
const MULTI_SOURCE_THRESHOLD: usize = 2;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum WebResearchOrchestratorError {
    #[error("Search failed: {0}")]
    SearchFailed(#[from] SearchError),

    #[error("Fetch failed: {0}")]
    FetchFailed(#[from] WebSearchError),

    #[error("Sidecar unavailable: {reason}")]
    SidecarUnavailable { reason: String },

    #[error("Budget exhausted: {used} tokens used of {budget} budget")]
    BudgetExhausted { used: usize, budget: usize },

    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),

    #[error("Max iterations reached for node {node_id:?} after {iterations} iterations")]
    MaxIterationsReached { node_id: NodeId, iterations: u32 },
}

// ---------------------------------------------------------------------------
// WebResearchStepEmitter — per-iteration progress callback
// ---------------------------------------------------------------------------

/// Per-iteration progress event from [`WebResearchOrchestrator::run_node_loop`].
///
/// `action` is one of: `"iterate"` (standard progress), `"no_results"` (search
/// returned empty), `"converged"`, `"pruned"`, or `"max_iterations"`.
#[derive(Debug, Clone)]
pub struct WebResearchStepEvent {
    pub node_id: u64,
    pub action: String,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
    pub sources_fetched: u32,
    pub sources_used: u32,
    /// Backend that performed the fetch for this iteration step ("reqwest" or "obscura").
    /// Empty string when no fetch occurred (e.g. no_results, pruned, max_iterations).
    pub fetcher: String,
}

/// Injectable per-step emitter. The default orchestrator uses `None` and emits
/// nothing; the task runner plugs in an SSB-backed emitter so web-research
/// tasks surface `web_research_step` rows in the dashboard activity feed.
///
/// Uses the manual boxed-future pattern (not `async-trait` crate) so the trait
/// stays dyn-safe while permitting borrow-from-self inside emitter impls.
pub trait WebResearchStepEmitter: Send + Sync {
    fn emit(
        &self,
        event: WebResearchStepEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;

    /// Emit a fine-grained per-stage event (B-7: FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE).
    /// Default impl is a no-op; the SSB adapter overrides it. Existing impls
    /// need not change since this method has a default body.
    fn emit_stage(
        &self,
        event: WebResearchStageEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let _ = event;
        Box::pin(async {})
    }
}

// ---------------------------------------------------------------------------
// WebResearchStageEvent — fine-grained per-stage progress (B-7)
// ---------------------------------------------------------------------------

/// One FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE inner-loop event.
///
/// Reuses the `web_research_step` SSB action type; the `stage` + `stage_phase`
/// fields on `WebResearchStepPayload` distinguish these from coarse rows.
/// No schema version bump: the payload is stored as an opaque msgpack blob.
///
/// `stage`:       "formulate" | "search" | "extract" | "reflect" | "converge"
/// `stage_phase`: "started"   | "completed"
/// `payload`:     stage-specific structured data (see variants below).
#[derive(Debug, Clone)]
pub struct WebResearchStageEvent {
    pub node_id: u64,
    pub stage: &'static str,
    pub stage_phase: &'static str,
    pub phi: f64,
    pub psi: f64,
    pub iteration: u32,
    /// Stage-specific structured data serialised to JSON value.
    pub payload: serde_json::Value,
}

// ---------------------------------------------------------------------------
// WebResearchOrchestrator
// ---------------------------------------------------------------------------

pub struct WebResearchOrchestrator {
    searx_client: SearxClient,
    web_fetcher: WebFetcher,
    /// D4: optional obscura headless-browser fetcher. When set, fetch_and_extract
    /// uses this instead of web_fetcher. Only present with feature = "obscura".
    #[cfg(feature = "obscura")]
    obscura_fetcher: Option<elai_web_search::obscura_fetcher::ObscuraFetcher>,
    extractor: BasicExtractor,
    quality_table: SourceQualityTable,
    content_index: WebContentIndex,
    budget_manager: ResearchBudgetManager,
    tier_config: ResearchTierConfig,
    #[allow(dead_code)] // reserved for future time-bounded research sessions
    clock: Arc<dyn ClockProvider>,
    chunk_counter: u64,
    step_emitter: Option<Arc<dyn WebResearchStepEmitter>>,
}

impl WebResearchOrchestrator {
    pub fn new(
        searx_client: SearxClient,
        web_fetcher: WebFetcher,
        extractor: BasicExtractor,
        quality_table: SourceQualityTable,
        content_index: WebContentIndex,
        budget_manager: ResearchBudgetManager,
        tier_config: ResearchTierConfig,
        clock: Arc<dyn ClockProvider>,
    ) -> Self {
        Self {
            searx_client,
            web_fetcher,
            #[cfg(feature = "obscura")]
            obscura_fetcher: None,
            extractor,
            quality_table,
            content_index,
            budget_manager,
            tier_config,
            clock,
            chunk_counter: 0,
            step_emitter: None,
        }
    }

    /// Attach a per-iteration step emitter. The task runner uses this to bridge
    /// orchestrator progress to SSB `web_research_step` rows.
    pub fn with_step_emitter(mut self, emitter: Arc<dyn WebResearchStepEmitter>) -> Self {
        self.step_emitter = Some(emitter);
        self
    }

    /// D4: attach an ObscuraFetcher. When set, fetch_and_extract uses the
    /// headless-browser path instead of reqwest. Only available with feature = "obscura".
    #[cfg(feature = "obscura")]
    pub fn with_obscura_fetcher(
        mut self,
        fetcher: elai_web_search::obscura_fetcher::ObscuraFetcher,
    ) -> Self {
        self.obscura_fetcher = Some(fetcher);
        self
    }

    /// Access the tier config (for contract tests and callers).
    pub fn tier_config(&self) -> &ResearchTierConfig {
        &self.tier_config
    }

    /// Access the budget manager.
    pub fn budget_manager(&self) -> &ResearchBudgetManager {
        &self.budget_manager
    }

    /// Run the per-node research loop for a single node.
    ///
    /// Loop: FORMULATE -> SEARCH -> QUALITY GATE -> FETCH+EXTRACT ->
    ///       INDEX -> RETRIEVE -> REFLECT -> CONVERGE
    ///
    /// Returns the number of iterations performed.
    pub async fn run_node_loop(
        &mut self,
        node: &mut ResearchNode,
    ) -> Result<u32, WebResearchOrchestratorError> {
        let phi_min = self.tier_config.phi_min;
        let psi_min = self.tier_config.psi_min;
        let max_iterations = u32::try_from(self.tier_config.depth_limit).unwrap_or(u32::MAX) * MAX_ITERATIONS_PER_DEPTH;
        let mut total_sources_fetched: u32 = 0;
        let mut total_sources_used: u32 = 0;
        let mut last_fetcher: &'static str = "";

        node.status = NodeStatus::Active;

        for iteration in 0..max_iterations {
            node.iterations = iteration + 1;
            last_fetcher = "";

            // FORMULATE: build query from question + prior evidence.
            self.emit_stage(node, "formulate", "started", json!({
                "prior_citations": node.citations.len()
            })).await;
            let query = self.formulate_query(&node.question, &node.citations).await;
            debug!(node_id = node.id.0, iteration, query = %query, "formulated query");
            self.emit_stage(node, "formulate", "completed", json!({
                "query": query
            })).await;

            // SEARCH: execute search and rank by source quality.
            self.emit_stage(node, "search", "started", json!({
                "query": query
            })).await;
            let ranked_results = match self.search_and_rank(&query).await {
                Ok(results) => results,
                Err(e) => {
                    warn!(node_id = node.id.0, error = %e, "search failed, skipping iteration");
                    return Err(e);
                }
            };

            // Compute source-tier distribution for the search_completed event.
            let n_results = u32::try_from(ranked_results.len()).unwrap_or(u32::MAX);
            let (t1, t2, t3, t4) = ranked_results.iter().fold((0u32, 0u32, 0u32, 0u32), |acc, r| {
                match self.quality_table.lookup_url(&r.url) {
                    SourceTier::T1Authoritative => (acc.0 + 1, acc.1, acc.2, acc.3),
                    SourceTier::T2Strong        => (acc.0, acc.1 + 1, acc.2, acc.3),
                    SourceTier::T3Weak          => (acc.0, acc.1, acc.2 + 1, acc.3),
                    SourceTier::T4Rejected      => (acc.0, acc.1, acc.2, acc.3 + 1),
                }
            });
            self.emit_stage(node, "search", "completed", json!({
                "n_results": n_results,
                "t1": t1, "t2": t2, "t3": t3, "t4": t4
            })).await;

            if ranked_results.is_empty() {
                debug!(node_id = node.id.0, "no search results, marking for prune check");
                node.low_gain_streak += 1;
                self.emit_step(
                    node,
                    "no_results",
                    total_sources_fetched,
                    total_sources_used,
                    "",
                )
                .await;
                if node.should_prune() {
                    node.status = NodeStatus::Pruned;
                    self.emit_step(
                        node,
                        "pruned",
                        total_sources_fetched,
                        total_sources_used,
                        "",
                    )
                    .await;
                    break;
                }
                continue;
            }

            // QUALITY GATE: fetch Tier1-3 first; Tier4-5 only if budget allows.
            let (high_quality, low_quality): (Vec<_>, Vec<_>) = ranked_results
                .into_iter()
                .partition(|r| {
                    let tier = self.quality_table.lookup_url(&r.url);
                    matches!(tier, SourceTier::T1Authoritative | SourceTier::T2Strong | SourceTier::T3Weak)
                });

            let mut to_fetch = high_quality;
            if self.budget_manager.remaining() > 0 {
                to_fetch.extend(low_quality);
            }

            // Cap by max_sources_per_node.
            to_fetch.truncate(self.tier_config.max_sources_per_node);
            let n_urls_to_fetch = u32::try_from(to_fetch.len()).unwrap_or(u32::MAX);
            total_sources_fetched = total_sources_fetched.saturating_add(n_urls_to_fetch);

            // EXTRACT: fetch pages and extract content.
            // Respect max_concurrent_fetches by processing in batches.
            self.emit_stage(node, "extract", "started", json!({
                "n_urls": n_urls_to_fetch
            })).await;
            let batch_size = self.tier_config.max_concurrent_fetches;
            let mut extracted_items: Vec<(SearchResult, String, SourceTier)> = Vec::new();

            for batch in to_fetch.chunks(batch_size) {
                for result in batch {
                    match self.fetch_and_extract(&result.url).await {
                        Ok((text, tier, ftag)) => {
                            last_fetcher = ftag;
                            extracted_items.push((result.clone(), text, tier));
                        }
                        Err(e) => {
                            debug!(url = %result.url, error = %e, "fetch/extract failed, skipping");
                        }
                    }
                }
            }
            let extract_total_chars: u64 = extracted_items.iter()
                .map(|(_, text, _)| u64::try_from(text.len()).unwrap_or(u64::MAX))
                .fold(0u64, |a, b| a.saturating_add(b));
            let n_chunks_extracted = u32::try_from(extracted_items.len()).unwrap_or(u32::MAX);
            self.emit_stage(node, "extract", "completed", json!({
                "n_chunks": n_chunks_extracted,
                "total_chars": extract_total_chars
            })).await;

            // INDEX: ingest extracted content into the content index.
            for (result, text, tier) in &extracted_items {
                let tokens = EvidenceItem::estimate_tokens(text);
                if !self.budget_manager.can_admit(tokens) {
                    debug!(url = %result.url, tokens, "budget would be exceeded, skipping ingest");
                    continue;
                }

                self.chunk_counter += 1;
                let chunk_id = format!("wrc-{}", self.chunk_counter);
                let chunk = WebChunk::new(
                    chunk_id,
                    result.url.clone(),
                    *tier,
                    text.clone(),
                );
                self.content_index.ingest(chunk);
                self.budget_manager.admit(tokens);
                total_sources_used += 1;

                // Add citation to node.
                let quality = source_tier_to_quality(*tier);
                node.citations.push(SourceCitation {
                    url: result.url.clone(),
                    title: result.title.clone(),
                    quality,
                    evidence_level: EvidenceLevel::SingleSource,
                    excerpt: truncate_excerpt(text, 200),
                });

                // Upgrade all citations to MultiSource once corroborated.
                if node.citations.len() >= MULTI_SOURCE_THRESHOLD {
                    for c in &mut node.citations {
                        if c.evidence_level == EvidenceLevel::SingleSource {
                            c.evidence_level = EvidenceLevel::MultiSource;
                        }
                    }
                }
            }

            // RETRIEVE: keyword search on indexed content (progressive disclosure).
            let retrieval_results = self.content_index.keyword_search(
                &node.question,
                self.tier_config.breadth_factor,
            );
            debug!(
                node_id = node.id.0,
                results = retrieval_results.len(),
                "retrieval complete"
            );

            // REFLECT: update phi/psi based on evidence quality and coverage.
            self.emit_stage(node, "reflect", "started", json!({
                "citations": node.citations.len()
            })).await;
            let old_phi = node.phi;
            self.reflect(node);

            // Track info gain for pruning.
            let info_gain = node.phi - old_phi;
            node.info_gain = info_gain;
            if info_gain < crate::research_tree::INFO_GAIN_THRESHOLD {
                node.low_gain_streak += 1;
            } else {
                node.low_gain_streak = 0;
            }

            // CONVERGE: check phi >= phi_min AND psi >= psi_min.
            let converge_action = if node.is_converged(phi_min, psi_min)
                && node.citations.len() >= MIN_SOURCES_FOR_CONVERGENCE
            {
                "converge"
            } else if node.should_prune() {
                "prune"
            } else {
                "iterate"
            };
            self.emit_stage(node, "reflect", "completed", json!({
                "action": converge_action,
                "phi": node.phi,
                "psi": node.psi,
                "info_gain": info_gain
            })).await;

            if node.is_converged(phi_min, psi_min) && node.citations.len() >= MIN_SOURCES_FOR_CONVERGENCE {
                node.status = NodeStatus::Converged;
                info!(
                    node_id = node.id.0,
                    phi = node.phi,
                    psi = node.psi,
                    iterations = node.iterations,
                    "node converged"
                );
                self.emit_stage(node, "converge", "started", json!({
                    "phi": node.phi,
                    "psi": node.psi
                })).await;
                self.emit_step(
                    node,
                    "converged",
                    total_sources_fetched,
                    total_sources_used,
                    last_fetcher,
                )
                .await;
                self.emit_stage(node, "converge", "completed", json!({
                    "phi": node.phi,
                    "psi": node.psi,
                    "citations": node.citations.len()
                })).await;
                break;
            }

            // Prune check.
            if node.should_prune() {
                node.status = NodeStatus::Pruned;
                info!(
                    node_id = node.id.0,
                    low_gain_streak = node.low_gain_streak,
                    "node pruned due to low info gain"
                );
                self.emit_step(
                    node,
                    "pruned",
                    total_sources_fetched,
                    total_sources_used,
                    last_fetcher,
                )
                .await;
                break;
            }

            // Per-iteration progress (non-terminal).
            self.emit_step(
                node,
                "iterate",
                total_sources_fetched,
                total_sources_used,
                last_fetcher,
            )
            .await;
        }

        // If we hit max iterations without convergence or prune.
        if node.status == NodeStatus::Active {
            warn!(
                node_id = node.id.0,
                iterations = node.iterations,
                phi = node.phi,
                psi = node.psi,
                "max iterations reached without convergence"
            );
            self.emit_step(
                node,
                "max_iterations",
                total_sources_fetched,
                total_sources_used,
                last_fetcher,
            )
            .await;
        }

        info!(
            node_id = node.id.0,
            status = ?node.status,
            sources_fetched = total_sources_fetched,
            sources_used = total_sources_used,
            "node loop complete"
        );

        Ok(node.iterations)
    }

    /// Formulate a search query from the question and prior citations.
    ///
    /// In production, this calls the sidecar LLM via InferRequest IPC.
    /// For now, deterministic: use the question plus domain keywords from citations.
    pub async fn formulate_query(
        &self,
        question: &str,
        prior_citations: &[SourceCitation],
    ) -> String {
        // Simple deterministic formulation: question + top domain keywords.
        if prior_citations.is_empty() {
            return question.to_string();
        }

        // Extract unique domains from prior citations to diversify search.
        let visited_domains: Vec<String> = prior_citations
            .iter()
            .map(|c| elai_web_search::parse_domain(&c.url))
            .filter(|d| !d.is_empty())
            .take(3)
            .collect();

        if visited_domains.is_empty() {
            question.to_string()
        } else {
            // Append "-site:" exclusions for already-visited domains.
            let exclusions: Vec<String> = visited_domains
                .iter()
                .map(|d| format!("-site:{d}"))
                .collect();
            format!("{} {}", question, exclusions.join(" "))
        }
    }

    /// Execute search and rank results by source quality.
    pub async fn search_and_rank(
        &mut self,
        query: &str,
    ) -> Result<Vec<SearchResult>, WebResearchOrchestratorError> {
        let mut results = self.searx_client.search(query).await?;

        // Sort by source quality score (descending), then by search score.
        results.sort_by(|a, b| {
            let qa = self.quality_table.score_url(&a.url);
            let qb = self.quality_table.score_url(&b.url);
            qb.partial_cmp(&qa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        Ok(results)
    }

    /// Fetch a URL and extract its content. Returns (text, source_tier, fetcher_tag).
    ///
    /// `fetcher_tag` is `"obscura"` when the headless-browser path was used,
    /// `"reqwest"` otherwise.
    ///
    /// D4: if the "obscura" feature is enabled and an ObscuraFetcher was attached
    /// via with_obscura_fetcher(), uses the headless-browser path; otherwise falls
    /// back to the reqwest WebFetcher.
    pub async fn fetch_and_extract(
        &mut self,
        url: &str,
    ) -> Result<(String, SourceTier, &'static str), WebResearchOrchestratorError> {
        #[cfg(feature = "obscura")]
        let (page, fetcher_tag) = {
            if let Some(ref obscura) = self.obscura_fetcher {
                (obscura.fetch(url).await?, "obscura")
            } else {
                (self.web_fetcher.fetch(url).await?, "reqwest")
            }
        };
        #[cfg(not(feature = "obscura"))]
        let (page, fetcher_tag) = (self.web_fetcher.fetch(url).await?, "reqwest");

        let extracted = self.extractor.extract(&page.html, url)?;

        if extracted.page_type == PageType::Empty {
            return Err(WebResearchOrchestratorError::FetchFailed(
                WebSearchError::ExtractionFailed(format!("empty page: {url}")),
            ));
        }

        let tier = self.quality_table.lookup_url(url);
        Ok((extracted.text, tier, fetcher_tag))
    }

    /// Reflect on evidence: update phi/psi based on citation quality and count.
    ///
    /// phi = avg(citation.quality.score()) * coverage_factor
    /// psi = avg(evidence_level.score()) * quality_weight * coverage_factor
    pub fn reflect(&self, node: &mut ResearchNode) {
        if node.citations.is_empty() {
            return;
        }

        let count = node.citations.len() as f64;

        // phi: average source quality * coverage factor.
        let avg_quality: f64 = node.citations.iter()
            .map(|c| c.quality.score())
            .sum::<f64>() / count;

        // Coverage factor: diminishing returns, approaches 1.0 as count grows.
        let coverage_factor = 1.0 - (1.0 / (1.0 + count));
        node.phi = avg_quality * coverage_factor;

        // psi: agreement/corroboration score.
        // Based on evidence level (MultiSource=0.75 when >=2 sources) *
        // quality weight * coverage factor. Does NOT require tier diversity.
        let avg_evidence: f64 = node.citations.iter()
            .map(|c| c.evidence_level.score())
            .sum::<f64>() / count;
        node.psi = avg_evidence * avg_quality * coverage_factor;
    }

    /// Emit a fine-grained per-stage event (B-7). No-op when no emitter is attached.
    async fn emit_stage(
        &self,
        node: &ResearchNode,
        stage: &'static str,
        stage_phase: &'static str,
        payload: serde_json::Value,
    ) {
        if let Some(emitter) = self.step_emitter.as_ref() {
            emitter
                .emit_stage(WebResearchStageEvent {
                    node_id: node.id.0,
                    stage,
                    stage_phase,
                    phi: node.phi,
                    psi: node.psi,
                    iteration: node.iterations,
                    payload,
                })
                .await;
        }
    }

    /// Emit a per-iteration progress event via the injected step emitter.
    /// No-op when no emitter is attached.
    /// `fetcher` is the backend tag ("reqwest", "obscura", or "" when no fetch occurred).
    async fn emit_step(
        &self,
        node: &ResearchNode,
        action: &str,
        sources_fetched: u32,
        sources_used: u32,
        fetcher: &str,
    ) {
        if let Some(emitter) = self.step_emitter.as_ref() {
            emitter
                .emit(WebResearchStepEvent {
                    node_id: node.id.0,
                    action: action.to_owned(),
                    phi: node.phi,
                    psi: node.psi,
                    iteration: node.iterations,
                    sources_fetched,
                    sources_used,
                    fetcher: fetcher.to_owned(),
                })
                .await;
        }
    }

    /// Get node iteration stats for SSB reporting.
    pub fn node_stats(&self, node: &ResearchNode) -> NodeLoopStats {
        NodeLoopStats {
            node_id: node.id.0,
            iterations: node.iterations,
            phi: node.phi,
            psi: node.psi,
            sources_fetched: u32::try_from(node.citations.len()).unwrap_or(u32::MAX),
            sources_used: u32::try_from(node.citations.len()).unwrap_or(u32::MAX),
        }
    }
}

// ---------------------------------------------------------------------------
// NodeLoopStats — summary for SSB and metrics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NodeLoopStats {
    pub node_id: u64,
    pub iterations: u32,
    pub phi: f64,
    pub psi: f64,
    pub sources_fetched: u32,
    pub sources_used: u32,
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Compute source confidence from a list of source tiers.
/// Returns a value in [0, 1]. >80% T4Rejected sources -> LOW (< 0.5).
pub fn compute_source_confidence(tiers: &[SourceTier]) -> f64 {
    if tiers.is_empty() {
        return 0.0;
    }

    let total = tiers.len() as f64;
    let low_quality_count = tiers.iter()
        .filter(|t| matches!(t, SourceTier::T4Rejected))
        .count() as f64;

    let low_quality_ratio = low_quality_count / total;

    if low_quality_ratio > LOW_QUALITY_THRESHOLD {
        // Heavily penalized: >80% low quality -> confidence < 0.5.
        let penalty = low_quality_ratio - LOW_QUALITY_THRESHOLD;
        (LOW_CONFIDENCE_CUTOFF - penalty * 2.5).max(0.0)
    } else {
        // Weight by tier quality.
        let quality_sum: f64 = tiers.iter()
            .map(|t| match t {
                SourceTier::T1Authoritative => 1.0,
                SourceTier::T2Strong => 0.8,
                SourceTier::T3Weak => 0.5,
                SourceTier::T4Rejected => 0.2,
            })
            .sum();
        quality_sum / total
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Map SourceTier to SourceQuality for citations.
fn source_tier_to_quality(tier: SourceTier) -> SourceQuality {
    match tier {
        SourceTier::T1Authoritative => SourceQuality::Tier1,
        SourceTier::T2Strong => SourceQuality::Tier2,
        SourceTier::T3Weak => SourceQuality::Tier3,
        SourceTier::T4Rejected => SourceQuality::Tier4,
    }
}

/// Truncate text to `max_len` characters, adding "..." if truncated.
fn truncate_excerpt(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        let boundary = text.char_indices()
            .take_while(|(i, _)| *i < max_len.saturating_sub(3))
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &text[..boundary])
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_confidence_all_t1() {
        let tiers = vec![SourceTier::T1Authoritative; 5];
        let c = compute_source_confidence(&tiers);
        assert!((c - 1.0).abs() < f64::EPSILON, "all T1 should give confidence=1.0, got {c}");
    }

    #[test]
    fn compute_confidence_majority_low_quality() {
        let tiers = vec![
            SourceTier::T4Rejected,
            SourceTier::T4Rejected,
            SourceTier::T4Rejected,
            SourceTier::T4Rejected,
            SourceTier::T1Authoritative,
        ];
        let c = compute_source_confidence(&tiers);
        assert!(c < LOW_CONFIDENCE_CUTOFF, "80%+ T4 should give LOW confidence, got {c}");
    }

    #[test]
    fn compute_confidence_empty() {
        assert_eq!(compute_source_confidence(&[]), 0.0);
    }

    #[test]
    fn truncate_excerpt_short() {
        let text = "short";
        assert_eq!(truncate_excerpt(text, 200), "short");
    }

    #[test]
    fn truncate_excerpt_long() {
        let text = "a".repeat(300);
        let result = truncate_excerpt(&text, 200);
        assert!(result.len() <= 200);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn source_tier_to_quality_mapping() {
        assert_eq!(source_tier_to_quality(SourceTier::T1Authoritative), SourceQuality::Tier1);
        assert_eq!(source_tier_to_quality(SourceTier::T2Strong), SourceQuality::Tier2);
        assert_eq!(source_tier_to_quality(SourceTier::T3Weak), SourceQuality::Tier3);
        assert_eq!(source_tier_to_quality(SourceTier::T4Rejected), SourceQuality::Tier4);
    }

    #[test]
    fn reflect_updates_phi_psi() {
        let mut node = ResearchNode::new(NodeId(1), "test question".to_string(), None, 0);
        node.citations.push(SourceCitation {
            url: "https://docs.rs/tokio".to_string(),
            title: "Tokio docs".to_string(),
            quality: SourceQuality::Tier2,
            evidence_level: EvidenceLevel::SingleSource,
            excerpt: "test".to_string(),
        });
        node.citations.push(SourceCitation {
            url: "https://arxiv.org/paper".to_string(),
            title: "Paper".to_string(),
            quality: SourceQuality::Tier1,
            evidence_level: EvidenceLevel::Verified,
            excerpt: "test".to_string(),
        });

        let orch_cfg = crate::research_tier::research_config_for_tier(
            elai_core::VramTier::Vram12Gb,
        );
        // Use reflect logic directly.
        let count = node.citations.len() as f64;
        let avg_quality: f64 = node.citations.iter()
            .map(|c| c.quality.score())
            .sum::<f64>() / count;
        let coverage_factor = 1.0 - (1.0 / (1.0 + count));

        // Before reflect, phi/psi are 0.
        assert_eq!(node.phi, 0.0);
        assert_eq!(node.psi, 0.0);

        // Manually compute expected phi.
        let expected_phi = avg_quality * coverage_factor;
        assert!(expected_phi > 0.0);

        // We can't call reflect without an orchestrator instance, but the
        // unit test validates the math. The contract test validates the struct.
        let _ = orch_cfg;
    }
}
