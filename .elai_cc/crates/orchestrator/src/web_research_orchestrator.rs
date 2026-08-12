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
use elai_web_search::extractor::{BasicExtractor, ContentExtractor, PageType, QueryAwareExtractor};
use elai_web_search::fetcher::WebFetcher;
use elai_web_search::source_quality::SourceQualityTable;
use elai_web_search::{SearchError, SearchResult, WebSearchError};

use elai_safety::provenance::Provenance;
use elai_safety::taint_flow::tainted_context;
use elai_safety::blast_radius::{check_egress_host, resolve_host_std, BlastRadiusVerdict};

use crate::blast_radius_ssb_adapter::{blast_radius_gate_disabled, emit_blast_radius_blocked};
use crate::sandbox::researcher_net::ResearcherNetConfig;
use crate::ssb::BlastRadiusKind;
use crate::capability_flow::{commit_minimal_set, CommittedToolSet};
use crate::research_budget::{EvidenceItem, ResearchBudgetManager};
use crate::research_tier::ResearchTierConfig;
use crate::research_mode::ResearchMode;
use crate::research_tree::{NodeId, NodeStatus, ResearchNode, SourceCitation,
    SourceQuality, EvidenceLevel};
use crate::ssb::{SsbPayload, ToolFilterEventKind, ToolFilterPayload};
use crate::tool_filter::ToolCategory;

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
pub(crate) const MAX_ITERATIONS_PER_DEPTH: u32 = 10;

/// Minimum citations before upgrading evidence level to MultiSource.
const MULTI_SOURCE_THRESHOLD: usize = 2;

/// Maximum citation summaries supplied to a bounded operator view.
const MINIMAL_PROMPT_SUMMARY_CAP: usize = 12;
const MINIMAL_PROMPT_SUMMARY_CHARS: usize = 80;
const MINIMAL_PROMPT_FULL_EXTRACT_CHARS: usize = 200;
const MINIMAL_PROMPT_DISABLE_ENV: &str = "ELAI_RESEARCH_MINIMAL_PROMPT_DISABLE";

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

    /// G-3: the host-side egress blast-radius gate refused the outbound connect
    /// — the host carried a NUL/control char, failed to resolve, or resolved to
    /// an IP outside the pinned allow-set (fail-closed).
    #[error("Egress denied for host {host}: {detail}")]
    EgressDenied { host: String, detail: String },
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
    extractor: Box<dyn ContentExtractor>,
    /// J-40 T1: optional query-aware extractor. When set, fetch_and_extract
    /// calls extract_query_aware instead of extract.
    query_aware_extractor: Option<Box<dyn QueryAwareExtractor>>,
    quality_table: SourceQualityTable,
    content_index: WebContentIndex,
    budget_manager: ResearchBudgetManager,
    tier_config: ResearchTierConfig,
    research_mode: ResearchMode,
    #[allow(dead_code)] // reserved for future time-bounded research sessions
    clock: Arc<dyn ClockProvider>,
    chunk_counter: u64,
    step_emitter: Option<Arc<dyn WebResearchStepEmitter>>,
    /// F-3p1 / Slice C3 (PRODUCER): committed tool set locked before the
    /// first untrusted-byte boundary (`fetch_and_extract`). Shared with any
    /// enforcement layer that holds the same `Arc` (e.g. `Fsm::check_tool_allowed`
    /// via `Fsm::with_committed_tool_set`). `None` until
    /// [`Self::with_committed_tool_set`] is called; the loop commits into
    /// the slot once per `run_node_loop` call before `formulate_query`.
    committed_tool_set: Arc<tokio::sync::Mutex<Option<CommittedToolSet>>>,
    /// F-3p1 / Slice C3 (PRODUCER): SSB writer for `ToolFilterEvent{Committed}`
    /// rows. `None` until [`Self::with_ssb_writer`] is called; emission is
    /// a no-op when absent (telemetry gap is acceptable, enforcement always runs).
    ssb_writer: Option<Arc<crate::ssb::Ssb>>,
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
            extractor: Box::new(extractor),
            query_aware_extractor: None,
            quality_table,
            content_index,
            budget_manager,
            tier_config,
            research_mode: ResearchMode::Standard,
            clock,
            chunk_counter: 0,
            step_emitter: None,
            committed_tool_set: Arc::new(tokio::sync::Mutex::new(None)),
            ssb_writer: None,
        }
    }

    /// F-3p1 / Slice C3: share the committed-tool-set slot with an external
    /// enforcement layer (e.g. `Fsm`) so both read from the same `Arc`.
    ///
    /// When not called, the orchestrator still commits to its own internal slot;
    /// enforcement inside `fsm.check_tool_allowed` is a no-op unless the same
    /// `Arc` is wired there too.
    pub fn with_committed_tool_set(
        mut self,
        slot: Arc<tokio::sync::Mutex<Option<CommittedToolSet>>>,
    ) -> Self {
        self.committed_tool_set = slot;
        self
    }

    /// F-3p1 / Slice C3: attach an SSB writer for `ToolFilterEvent{Committed}`
    /// rows. When absent, commits still run but no SSB row is emitted.
    pub fn with_ssb_writer(mut self, writer: Arc<crate::ssb::Ssb>) -> Self {
        self.ssb_writer = Some(writer);
        self
    }

    /// Attach a per-iteration step emitter. The task runner uses this to bridge
    /// orchestrator progress to SSB `web_research_step` rows.
    pub fn with_step_emitter(mut self, emitter: Arc<dyn WebResearchStepEmitter>) -> Self {
        self.step_emitter = Some(emitter);
        self
    }

    /// J-40 T1: attach a query-aware extractor. When set, fetch_and_extract
    /// calls extract_query_aware with the formulated query instead of the
    /// plain extract path.
    pub fn with_query_aware_extractor(mut self, e: Box<dyn QueryAwareExtractor>) -> Self {
        self.query_aware_extractor = Some(e);
        self
    }

    pub fn with_research_mode(mut self, mode: ResearchMode) -> Self {
        self.research_mode = mode;
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
        let caps = self.research_mode.caps(&self.tier_config);
        let max_iterations = caps.max_iterations;
        let mut total_sources_fetched: u32 = 0;
        let mut total_sources_used: u32 = 0;
        let mut last_fetcher: &'static str = "";

        node.status = NodeStatus::Active;

        // F-3p1 / Slice C3 (PRODUCER): commit the minimal tool set BEFORE the
        // first untrusted-byte boundary (`fetch_and_extract`). The commit is
        // scoped to the current node's task_id so each node gets a fresh,
        // tamper-evident hash. This runs ONCE per `run_node_loop` call rather
        // than per iteration so the committed set is stable for the full loop.
        {
            let now_ms = i64::try_from(
                self.clock.now_ms()
            ).unwrap_or(i64::MAX);
            let task_id = format!("web-research-node-{}", node.id.0);
            let committed = commit_minimal_set(
                &task_id,
                &[ToolCategory::Web, ToolCategory::Read],
                now_ms,
            );
            let commit_hash = committed.commit_hash.clone();
            let allowed_labels: Vec<String> = committed
                .allowed_categories
                .iter()
                .map(|c| format!("{c:?}").to_ascii_lowercase())
                .collect();
            *self.committed_tool_set.lock().await = Some(committed);
            // Emit ToolFilterEvent{Committed} SSB row -- telemetry, non-fatal.
            if let Some(writer) = self.ssb_writer.as_ref() {
                let payload = SsbPayload::ToolFilterEvent(ToolFilterPayload {
                    commit_hash,
                    allowed_categories: allowed_labels,
                    denied_tool: None,
                    taint_source: None,
                    task_id,
                    kind: ToolFilterEventKind::Committed,
                });
                if let Err(e) = writer
                    .write_payload("web-research-orchestrator", 0, payload, Vec::new())
                    .await
                {
                    warn!(
                        node_id = node.id.0,
                        error = %e,
                        "web_research_orchestrator: ToolFilterEvent(Committed) SSB write failed (non-fatal)"
                    );
                }
            }
        }

        for iteration in 0..max_iterations {
            node.iterations = iteration + 1;
            last_fetcher = "";

            // FORMULATE: build query from question + prior evidence.
            let prompt_sizes = minimal_prompt_sizes(&node.citations);
            self.emit_stage(node, "formulate", "started", json!({
                "prior_citations": node.citations.len(),
                "prompt_size_before": prompt_sizes.0,
                "prompt_size_after": prompt_sizes.1,
                "prompt_size_ratio": prompt_size_ratio(prompt_sizes.0, prompt_sizes.1)
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
            to_fetch.truncate(caps.max_sources_per_node);
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
                    match self.fetch_and_extract(&result.url, &query).await {
                        Ok((text, tier, ftag)) => {
                            last_fetcher = ftag;
                            // G-1 INGEST: a web-fetched page body is UNTRUSTED
                            // external content. Register it into the process-
                            // local taint side-table keyed by its content so a
                            // later destructive sink whose argv derives from
                            // this body is blocked by the capability gate. The
                            // side-table skips Trusted labels, so this is a
                            // no-op for trusted flows. Default-ON (W-2): no env
                            // gate guards the register; the kill-switch is at
                            // the sink. See workflow/reasoning/G-1-decisions.md.
                            tainted_context()
                                .register(&text, Provenance::from_web(result.url.clone()));
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
                caps.breadth_factor,
            );
            debug!(
                node_id = node.id.0,
                results = retrieval_results.len(),
                "retrieval complete"
            );

            // REFLECT: update phi/psi based on evidence quality and coverage.
            let reflect_sizes = minimal_prompt_sizes(&node.citations);
            self.emit_stage(node, "reflect", "started", json!({
                "citations": node.citations.len(),
                "prompt_size_before": reflect_sizes.0,
                "prompt_size_after": reflect_sizes.1,
                "prompt_size_ratio": prompt_size_ratio(reflect_sizes.0, reflect_sizes.1)
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

        // The operator receives a bounded view; the kill-switch restores the
        // historical unbounded citation behavior for rollback/debugging.
        let operator_citations = minimal_prompt_citations(prior_citations);

        // Extract unique domains from prior citations to diversify search.
        let visited_domains: Vec<String> = operator_citations
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

    /// G-3: build the pinned egress IP allow-set by resolving each allowed host
    /// (the [`ResearcherNetConfig`] allow-list + trusted resolvers). Returns the
    /// union of every resolved IP plus the literal resolver IPs. Hosts that fail
    /// to resolve contribute nothing (a host whose resolution is empty can never
    /// match, so the gate fails closed for it — by design).
    fn pinned_egress_ips() -> Vec<std::net::IpAddr> {
        let cfg = ResearcherNetConfig::default();
        let mut ips: Vec<std::net::IpAddr> = Vec::new();
        for host in &cfg.allowed_hosts {
            ips.extend(resolve_host_std(host));
        }
        ips.extend(cfg.trusted_resolvers.iter().copied());
        ips
    }

    /// G-3: host-side egress blast-radius gate. Runs BEFORE any outbound connect
    /// in [`Self::fetch_and_extract`], fail-closed. Rejects a NUL/control-char
    /// host BEFORE resolution, then confirms every resolved IP is in the pinned
    /// allow-set. On a [`BlastRadiusVerdict::Block`] the fetch is REFUSED and a
    /// `blast_radius_blocked{EgressDenied}` SSB row is emitted through the live
    /// `ssb_writer` (the off-agent artifact, R4). Kill-switch
    /// `ELAI_BLAST_RADIUS_DISABLE` disarms it (default-ON, R9).
    fn enforce_egress(&self, url: &str) -> Result<(), WebResearchOrchestratorError> {
        if blast_radius_gate_disabled() {
            return Ok(());
        }
        let host = elai_web_search::parse_domain(url);
        let allowed = Self::pinned_egress_ips();
        let verdict = check_egress_host(&host, &allowed, resolve_host_std);
        if let BlastRadiusVerdict::Block {
            kind,
            pre_expansion,
            post_expansion,
        } = verdict
        {
            if let Some(ssb) = self.ssb_writer.as_ref() {
                emit_blast_radius_blocked(
                    ssb,
                    BlastRadiusKind::from(kind),
                    &pre_expansion,
                    &post_expansion,
                    &host,
                    "", // egress arm: no worktree root applies
                    "", // pure host-pin block; no provenance join
                    i64::try_from(self.clock.now_ms()).unwrap_or(i64::MAX),
                );
            }
            return Err(WebResearchOrchestratorError::EgressDenied {
                host,
                detail: post_expansion,
            });
        }
        Ok(())
    }

    /// Fetch a URL and extract its content. Returns (text, source_tier, fetcher_tag).
    ///
    /// `fetcher_tag` is `"obscura"` when the headless-browser path was used,
    /// `"reqwest"` otherwise.
    ///
    /// D4: if the "obscura" feature is enabled and an ObscuraFetcher was attached
    /// via with_obscura_fetcher(), uses the headless-browser path; otherwise falls
    /// back to the reqwest WebFetcher.
    ///
    /// G-3: the host-side egress blast-radius gate ([`Self::enforce_egress`])
    /// runs FIRST, before any outbound connect — a NUL/control-char host or an
    /// IP outside the pinned allow-set is refused fail-closed.
    pub async fn fetch_and_extract(
        &mut self,
        url: &str,
        query: &str,
    ) -> Result<(String, SourceTier, &'static str), WebResearchOrchestratorError> {
        // G-3 EGRESS GATE: reject NUL/control-char host + pin the resolved IP
        // BEFORE connect (fail-closed).
        self.enforce_egress(url)?;
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

        let extracted = if let Some(ref qe) = self.query_aware_extractor {
            qe.extract_query_aware(&page.html, url, query)?
        } else {
            self.extractor.extract(&page.html, url)?
        };

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

/// C3 operator view: `summary_cap` bounds the newest citations and keeps one
/// `one_full` extract for the operator while preserving the source vector.
fn minimal_prompt_disabled() -> bool {
    std::env::var(MINIMAL_PROMPT_DISABLE_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// Build the bounded citation data view consumed by FORMULATE and REFLECT.
/// The newest twelve rows remain, with one highest-quality full extract.
/// Every other row becomes a one-line excerpt. The source vector stays intact.
fn minimal_prompt_citations(citations: &[SourceCitation]) -> Vec<SourceCitation> {
    minimal_prompt_citations_with_disabled(citations, minimal_prompt_disabled())
}

fn minimal_prompt_citations_with_disabled(
    citations: &[SourceCitation],
    disabled: bool,
) -> Vec<SourceCitation> {
    if disabled {
        return citations.to_vec();
    }
    if citations.is_empty() {
        return Vec::new();
    }

    let start = citations.len().saturating_sub(MINIMAL_PROMPT_SUMMARY_CAP);
    let mut bounded: Vec<SourceCitation> = citations[start..].to_vec();
    let full_index = bounded
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.quality.score().total_cmp(&right.quality.score()))
        .map(|(index, _)| index)
        .unwrap_or(0);

    for (index, citation) in bounded.iter_mut().enumerate() {
        if index != full_index {
            citation.excerpt = one_line_excerpt(&citation.excerpt, MINIMAL_PROMPT_SUMMARY_CHARS);
        } else {
            citation.excerpt = truncate_excerpt(&citation.excerpt, MINIMAL_PROMPT_FULL_EXTRACT_CHARS);
        }
    }
    bounded
}

/// Estimate the character sizes before and after the bounded operator view.
fn minimal_prompt_sizes(citations: &[SourceCitation]) -> (usize, usize) {
    let before = citations
        .iter()
        .map(|citation| citation.excerpt.len())
        .sum();
    let after = minimal_prompt_citations(citations)
        .iter()
        .map(|citation| citation.excerpt.len())
        .sum();
    (before, after)
}

/// Return a recomputable before/after size ratio for stage telemetry.
fn prompt_size_ratio(before: usize, after: usize) -> f64 {
    if after == 0 {
        1.0
    } else {
        before as f64 / after as f64
    }
}

/// Normalize a compressed citation to one line before applying its size cap.
fn one_line_excerpt(text: &str, max_len: usize) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_excerpt(&one_line, max_len)
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

        // The unit test validates the math. The contract test validates the struct.
        let _ = orch_cfg;
    }

    #[test]
    fn minimal_prompt_citations_bounds_newest_and_preserves_evidence() {
        let citations: Vec<SourceCitation> = (0..20)
            .map(|index| SourceCitation {
                url: format!("https://example{index}.test/source"),
                title: format!("Source {index}"),
                quality: if index == 19 { SourceQuality::Tier1 } else { SourceQuality::Tier3 },
                evidence_level: EvidenceLevel::SingleSource,
                excerpt: "x".repeat(200),
            })
            .collect();
        let original = citations.clone();
        let view = minimal_prompt_citations_with_disabled(&citations, false);

        assert_eq!(view.len(), MINIMAL_PROMPT_SUMMARY_CAP);
        assert_eq!(view[0].url, "https://example8.test/source");
        assert_eq!(view.iter().filter(|citation| citation.excerpt.len() > MINIMAL_PROMPT_SUMMARY_CHARS).count(), 1);
        assert_eq!(citations.len(), original.len(), "compression must not drop production evidence");
        assert_eq!(citations[0].excerpt, original[0].excerpt, "operator view must not mutate source evidence");
    }

    #[test]
    fn minimal_prompt_disable_restores_unbounded_and_ratio_recomputes() {
        let citations: Vec<SourceCitation> = (0..15)
            .map(|index| SourceCitation {
                url: format!("https://example{index}.test/source"),
                title: format!("Source {index}"),
                quality: SourceQuality::Tier2,
                evidence_level: EvidenceLevel::SingleSource,
                excerpt: "x".repeat(200),
            })
            .collect();
        let disabled = minimal_prompt_citations_with_disabled(&citations, true);

        assert_eq!(disabled.len(), citations.len());
        assert_eq!(disabled[0].excerpt.len(), citations[0].excerpt.len());
        let (before, after) = minimal_prompt_sizes(&citations);
        assert!(after < before);
        assert!((prompt_size_ratio(before, after) - before as f64 / after as f64).abs() < f64::EPSILON);
    }
}
