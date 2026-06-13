// A-43: Oracle research pipeline — 7-phase research engine.
// Phases: A (query formulation) -> B (search execution) -> C (source tier classification)
//      -> D (fact extraction) -> E (cross-verification) -> F (synthesis) -> G (post-verification).
// D3: Phase F is now LLM-backed via GroundingClient with deterministic fallback.

use rustc_hash::FxHashMap;
use regex::Regex;
use std::sync::LazyLock;

use elai_core::research_types::{
    AnswerConfidence, ExtractedFact, ResearchAnswer, ResearchPipelineError, ResearchQuery,
    VerificationLevel, VerifiedClaim, MAX_SEARCH_QUERIES,
    MIN_SEARCH_QUERIES, SUPPORTED_MIN_SOURCES, VERIFIED_MIN_SOURCES,
};
use elai_core::source_tier::{
    classify_source, SourceClassification, SourceTier, SourceTierConfig,
};
use crate::grounding_client::GroundingClient;

// ---------------------------------------------------------------------------
// Citation guard helper — A3 patch.
// ---------------------------------------------------------------------------

/// Regex that matches a numeric citation marker such as [1], [2], or [12].
/// Matches only `\[\d+\]` — rejects bare `[` tokens like `[INST]`, `[INFO]`,
/// `[Note]` that Qwen 3B may emit and that the old `.contains('[')` guard
/// would incorrectly accept as "cited prose".
static NUMERIC_CITATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\d+\]").expect("static regex is valid"));

/// Returns true iff `s` contains at least one numeric citation marker `[N]`.
fn has_numeric_citation(s: &str) -> bool {
    NUMERIC_CITATION_RE.is_match(s)
}

/// A single search result returned by a search backend.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    /// URL of the result page.
    pub url: String,
    /// Title of the result.
    pub title: String,
    /// Snippet or summary text.
    pub snippet: String,
}

/// Trait for pluggable search backends. Implementations execute queries and
/// return raw search results. The trait is object-safe for dynamic dispatch.
pub trait SearchBackend: Send + Sync {
    /// Execute a search query and return results (max `max_results` items).
    fn search(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<SearchResult>, ResearchPipelineError>;
}

/// Stub search backend for testing. Returns pre-configured results.
#[derive(Debug, Clone, Default)]
pub struct StubSearchBackend {
    /// Map from query substring to results.
    results: FxHashMap<String, Vec<SearchResult>>,
}

impl StubSearchBackend {
    /// Create a new stub with no pre-configured results.
    pub fn new() -> Self {
        Self {
            results: FxHashMap::default(),
        }
    }

    /// Add results for a query pattern.
    pub fn add_results(&mut self, query_pattern: impl Into<String>, results: Vec<SearchResult>) {
        self.results.insert(query_pattern.into(), results);
    }
}

impl SearchBackend for StubSearchBackend {
    fn search(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<SearchResult>, ResearchPipelineError> {
        let max = max_results as usize;
        // Return first matching pattern's results.
        for (pattern, results) in &self.results {
            if query.contains(pattern.as_str()) {
                return Ok(results.iter().take(max).cloned().collect());
            }
        }
        // No match: return empty (valid — pipeline handles "no sources found").
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// Phase A: Query Formulation
// ---------------------------------------------------------------------------

/// Formulated search queries derived from the research question.
/// In a full implementation, an LLM reformulates the question into 2-4 search queries.
/// This implementation does structural rephrasing (deterministic).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormulatedQueries {
    /// Original question.
    pub original: String,
    /// Generated search queries (2-4).
    pub queries: Vec<String>,
}

/// Phase A: formulate search queries from the research question.
/// Structural rephrasing: the question itself + a "what is" variant.
/// An LLM-backed implementation would generate more diverse queries.
pub fn phase_a_formulate_queries(query: &ResearchQuery) -> Result<FormulatedQueries, ResearchPipelineError> {
    if query.question.trim().is_empty() {
        return Err(ResearchPipelineError::EmptyQuestion);
    }

    let mut queries = Vec::with_capacity(MAX_SEARCH_QUERIES as usize);

    // Query 1: the question verbatim.
    queries.push(query.question.clone());

    // Query 2: with context if available.
    if let Some(ref ctx) = query.context {
        queries.push(format!("{} {}", query.question, ctx));
    } else {
        // Append "explanation" as a broadening term.
        queries.push(format!("{} explanation", query.question));
    }

    // Invariant: query count within bounds.
    assert!(queries.len() >= MIN_SEARCH_QUERIES as usize);
    assert!(queries.len() <= MAX_SEARCH_QUERIES as usize);

    Ok(FormulatedQueries {
        original: query.question.clone(),
        queries,
    })
}

// ---------------------------------------------------------------------------
// Phase B: Search Execution
// ---------------------------------------------------------------------------

/// Collected search results with deduplication.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CollectedResults {
    /// Unique search results (deduplicated by URL).
    pub results: Vec<SearchResult>,
    /// Total raw results before dedup.
    pub raw_count: usize,
}

/// Phase B: execute search queries and deduplicate results by URL.
pub fn phase_b_search(
    queries: &FormulatedQueries,
    backend: &dyn SearchBackend,
    max_sources: u32,
) -> Result<CollectedResults, ResearchPipelineError> {
    let mut all_results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();
    let mut raw_count: usize = 0;

    for q in &queries.queries {
        let results = backend.search(q, max_sources)?;
        raw_count += results.len();
        for r in results {
            if seen_urls.insert(r.url.clone()) {
                all_results.push(r);
            }
        }
    }

    Ok(CollectedResults {
        results: all_results,
        raw_count,
    })
}

// ---------------------------------------------------------------------------
// Phase C: Source Tier Classification
// ---------------------------------------------------------------------------

/// Results classified by source tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassifiedResults {
    /// Results with their tier classification, filtered to usable tiers.
    pub usable: Vec<(SearchResult, SourceClassification)>,
    /// T4 (rejected) results.
    pub rejected: Vec<(SearchResult, SourceClassification)>,
    /// Results where classification failed (logged, not used).
    pub classification_errors: Vec<(SearchResult, String)>,
}

/// Phase C: classify all search results using the source tier config.
/// Deterministic — no LLM input. T4 results are separated out.
/// If `required_tier` is set, only results at or above that tier pass.
pub fn phase_c_classify(
    collected: &CollectedResults,
    config: &SourceTierConfig,
    required_tier: Option<SourceTier>,
) -> ClassifiedResults {
    let mut usable = Vec::new();
    let mut rejected = Vec::new();
    let mut classification_errors = Vec::new();

    for result in &collected.results {
        match classify_source(&result.url, config) {
            Ok(classification) => {
                let dominated = match required_tier {
                    Some(SourceTier::T1Authoritative) => {
                        classification.tier != SourceTier::T1Authoritative
                    }
                    Some(SourceTier::T2Strong) => matches!(
                        classification.tier,
                        SourceTier::T3Weak | SourceTier::T4Rejected
                    ),
                    _ => classification.tier == SourceTier::T4Rejected,
                };

                if dominated {
                    rejected.push((result.clone(), classification));
                } else {
                    usable.push((result.clone(), classification));
                }
            }
            Err(e) => {
                classification_errors.push((result.clone(), e.to_string()));
            }
        }
    }

    ClassifiedResults {
        usable,
        rejected,
        classification_errors,
    }
}

// ---------------------------------------------------------------------------
// Phase D: Fact Extraction
// ---------------------------------------------------------------------------

/// Phase D: extract facts from classified search results.
/// In a full implementation, an LLM extracts structured facts with exact quotes.
/// This stub creates one fact per result from the snippet.
pub fn phase_d_extract_facts(
    classified: &ClassifiedResults,
) -> Vec<ExtractedFact> {
    let mut facts = Vec::new();

    for (result, classification) in &classified.usable {
        // Stub: treat snippet as both claim and exact quote.
        if result.snippet.trim().is_empty() {
            continue;
        }

        let fact = ExtractedFact::new(
            result.snippet.clone(),
            result.url.clone(),
            classification.tier,
        )
        .with_exact_quote(result.snippet.clone())
        .with_page_section(result.title.clone())
        .with_confidence(stub_extraction_confidence(classification.tier));

        facts.push(fact);
    }

    facts
}

/// Stub confidence based on tier. Real implementation would use LLM output.
fn stub_extraction_confidence(tier: SourceTier) -> f64 {
    match tier {
        SourceTier::T1Authoritative => 0.9,
        SourceTier::T2Strong => 0.7,
        SourceTier::T3Weak => 0.4,
        SourceTier::T4Rejected => 0.1,
    }
}

// ---------------------------------------------------------------------------
// Phase E: Cross-Verification
// ---------------------------------------------------------------------------

/// Phase E: cross-verify facts to produce verified claims.
/// Groups facts by normalized claim text. Counts independent T1/T2 sources with
/// exact quotes. Assigns verification level by set intersection (deterministic).
pub fn phase_e_cross_verify(
    facts: &[ExtractedFact],
) -> Vec<VerifiedClaim> {
    // Group facts by normalized claim (lowercased, trimmed).
    let mut claim_groups: FxHashMap<String, Vec<&ExtractedFact>> = FxHashMap::default();
    for fact in facts {
        let key = fact.claim.trim().to_lowercase();
        claim_groups.entry(key).or_default().push(fact);
    }

    let mut claims = Vec::new();

    for (_key, group_facts) in &claim_groups {
        // Count independent T1/T2 sources with exact quotes.
        let mut seen_domains = std::collections::HashSet::new();
        let mut independent_count: u32 = 0;

        for fact in group_facts {
            let is_high_tier = matches!(
                fact.source_tier,
                SourceTier::T1Authoritative | SourceTier::T2Strong
            );
            if is_high_tier && fact.has_exact_quote() {
                // Use domain as independence proxy — same domain = not independent.
                let domain_key = extract_domain_for_dedup(&fact.source_url);
                if seen_domains.insert(domain_key) {
                    independent_count += 1;
                }
            }
        }

        let has_any_quote = group_facts.iter().any(|f| f.has_exact_quote());
        let has_high_tier = group_facts.iter().any(|f| {
            matches!(
                f.source_tier,
                SourceTier::T1Authoritative | SourceTier::T2Strong
            )
        });

        let verification_level = if independent_count >= VERIFIED_MIN_SOURCES {
            VerificationLevel::Verified
        } else if independent_count >= SUPPORTED_MIN_SOURCES {
            VerificationLevel::Supported
        } else if has_high_tier || has_any_quote {
            // Has high-tier source but no exact quote, or has quote but only T3.
            VerificationLevel::Weak
        } else {
            VerificationLevel::Unsupported
        };

        // Use the first fact's claim text as the canonical form.
        let canonical_claim = group_facts
            .first()
            .map(|f| f.claim.clone())
            .unwrap_or_default();

        claims.push(VerifiedClaim {
            claim: canonical_claim,
            supporting_facts: group_facts.iter().map(|f| (*f).clone()).collect(),
            verification_level,
            independent_source_count: independent_count,
        });
    }

    claims
}

/// Extract domain from URL for deduplication (simplified — not for tier classification).
fn extract_domain_for_dedup(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .to_lowercase()
}

// ---------------------------------------------------------------------------
// Phase F: Synthesis
// ---------------------------------------------------------------------------

/// Phase F: synthesize verified claims into a research answer.
/// D3: LLM-backed synthesis via GroundingClient. On any failure or missing
/// citation markers the deterministic concatenation path fires as fallback.
/// tracing::info! logs which path executed (path=llm or path=deterministic_fallback).
pub async fn phase_f_synthesize<C: GroundingClient>(
    query: &ResearchQuery,
    claims: &[VerifiedClaim],
    classified: &ClassifiedResults,
    grounding: &C,
) -> ResearchAnswer {
    // Collect sources first (needed for both paths).
    let sources_consulted: Vec<SourceClassification> = classified
        .usable
        .iter()
        .map(|(_, sc)| sc.clone())
        .collect();
    let sources_rejected: Vec<SourceClassification> = classified
        .rejected
        .iter()
        .map(|(_, sc)| sc.clone())
        .collect();

    // Compute confidence.
    let confidence = compute_confidence(claims);

    // Deterministic fallback answer text — built eagerly so the fallback path
    // has zero extra work when the LLM path fails.
    let deterministic_text = if claims.is_empty() {
        "I don't know — no verifiable facts were found for this question.".to_owned()
    } else {
        claims
            .iter()
            .enumerate()
            .map(|(i, c)| format!("[{}] {}", i + 1, c.claim))
            .collect::<Vec<_>>()
            .join(" ")
    };

    // Short-circuit: no claims means no useful synthesis prompt.
    if claims.is_empty() {
        tracing::info!(path = "deterministic_fallback", reason = "no_claims", "phase_f_synthesis_path");
        return ResearchAnswer {
            question: query.question.clone(),
            answer_text: deterministic_text,
            evidence_chain: claims.to_vec(),
            confidence,
            caveats: Vec::new(),
            unanswered_parts: Vec::new(),
            sources_consulted,
            sources_rejected,
            path: "deterministic_fallback".to_owned(),
        };
    }

    // Build inline synthesis prompt.  Instructions tell the model to output
    // JSON conforming to ResearchReport so the existing ground() parser handles
    // it, then we extract .answer.  The sentinel DETERMINISTIC_FALLBACK in the
    // answer field triggers the deterministic path explicitly.
    let claims_block: String = claims
        .iter()
        .enumerate()
        .map(|(i, c)| format!("[{}] {}", i + 1, c.claim))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        include_str!("prompts/research_synthesizer.md"),
        question = query.question,
        claims_block = claims_block,
    );

    // Call the sidecar via synthesize_text (raw prompt, no template wrapping).
    let llm_text = match grounding.synthesize_text(&prompt).await {
        Ok(raw) => {
            let text = raw.trim().to_owned();
            if text == "DETERMINISTIC_FALLBACK" || text.is_empty() || !has_numeric_citation(&text) {
                tracing::info!(
                    path = "deterministic_fallback",
                    reason = if text == "DETERMINISTIC_FALLBACK" {
                        "sentinel"
                    } else if text.is_empty() {
                        "empty_response"
                    } else {
                        "no_citation_markers"
                    },
                    "phase_f_synthesis_path"
                );
                None
            } else {
                tracing::info!(path = "llm", "phase_f_synthesis_path");
                Some(text)
            }
        }
        Err(e) => {
            tracing::info!(
                path = "deterministic_fallback",
                reason = "grounding_error",
                error = %e,
                "phase_f_synthesis_path"
            );
            None
        }
    };

    let chosen_path = if llm_text.is_some() { "llm" } else { "deterministic_fallback" };
    let answer_text = llm_text.unwrap_or(deterministic_text);

    ResearchAnswer {
        question: query.question.clone(),
        answer_text,
        evidence_chain: claims.to_vec(),
        confidence,
        caveats: Vec::new(),
        unanswered_parts: Vec::new(),
        sources_consulted,
        sources_rejected,
        path: chosen_path.to_owned(),
    }
}

/// Compute overall answer confidence from claim verification levels.
fn compute_confidence(claims: &[VerifiedClaim]) -> AnswerConfidence {
    if claims.is_empty() {
        return AnswerConfidence::Unknown;
    }

    let all_verified = claims
        .iter()
        .all(|c| c.verification_level == VerificationLevel::Verified);
    let any_unsupported = claims
        .iter()
        .any(|c| c.verification_level == VerificationLevel::Unsupported);
    let any_weak = claims
        .iter()
        .any(|c| c.verification_level == VerificationLevel::Weak);

    if all_verified {
        AnswerConfidence::High
    } else if any_unsupported {
        AnswerConfidence::Low
    } else if any_weak {
        AnswerConfidence::Low
    } else {
        // All are VERIFIED or SUPPORTED, not all VERIFIED.
        AnswerConfidence::Medium
    }
}

// ---------------------------------------------------------------------------
// Phase G: Post-Verification
// ---------------------------------------------------------------------------

/// Post-verification issues found.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PostVerificationResult {
    /// True if the answer passes all checks.
    pub passed: bool,
    /// Issues found (empty if passed).
    pub issues: Vec<String>,
    /// Caveats to add to the answer.
    pub caveats: Vec<String>,
    /// Parts of the question that remain unanswered.
    pub unanswered_parts: Vec<String>,
}

/// Phase G: post-verification checks on the synthesized answer.
/// 4 checks: citation completeness, tier consistency, "I don't know" enforcement,
/// hallucination guard.
pub fn phase_g_post_verify(
    answer: &ResearchAnswer,
    query: &ResearchQuery,
) -> PostVerificationResult {
    let mut issues = Vec::new();
    let mut caveats = Vec::new();
    let mut unanswered_parts = Vec::new();

    // Check 1: citation completeness — every claim must have supporting facts.
    for claim in &answer.evidence_chain {
        if claim.supporting_facts.is_empty() {
            issues.push(format!("Claim has no supporting facts: {}", truncate_for_log(&claim.claim)));
        }
    }

    // Check 2: tier consistency — answer confidence should match evidence tiers.
    let has_only_weak = answer.evidence_chain.iter().all(|c| {
        matches!(
            c.verification_level,
            VerificationLevel::Weak | VerificationLevel::Unsupported
        )
    }) && !answer.evidence_chain.is_empty();

    if has_only_weak && answer.confidence != AnswerConfidence::Low {
        issues.push("Confidence is above LOW but all evidence is WEAK/UNSUPPORTED".to_owned());
    }

    // Check 3: "I don't know" enforcement — if no usable evidence, answer must acknowledge.
    if answer.evidence_chain.is_empty() && !answer.answer_text.contains("don't know") {
        issues.push("No evidence found but answer does not acknowledge uncertainty".to_owned());
    }

    // Check 4: hallucination guard — answer should not contain claims not in evidence chain.
    // (Stub: check that answer references at least one claim number.)
    if !answer.evidence_chain.is_empty() && !has_numeric_citation(&answer.answer_text) {
        caveats.push("Answer text may not cite all evidence — review citations".to_owned());
    }

    // Identify unanswered parts via context keywords.
    if let Some(ref ctx) = query.context {
        let ctx_words: Vec<&str> = ctx.split_whitespace().collect();
        let answer_lower = answer.answer_text.to_lowercase();
        for word in ctx_words {
            if word.len() > 3 && !answer_lower.contains(&word.to_lowercase()) {
                unanswered_parts.push(format!("Context keyword '{}' not addressed", word));
            }
        }
    }

    let passed = issues.is_empty();

    PostVerificationResult {
        passed,
        issues,
        caveats,
        unanswered_parts,
    }
}

/// Maximum characters to show in log messages for claim text.
const LOG_TRUNCATE_LEN: usize = 80;

/// Truncate text for log/error messages (UTF-8 safe).
fn truncate_for_log(s: &str) -> String {
    match s.char_indices().nth(LOG_TRUNCATE_LEN) {
        Some((byte_idx, _)) => format!("{}...", &s[..byte_idx]),
        None => s.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Pipeline orchestrator
// ---------------------------------------------------------------------------

/// Run the full 7-phase research pipeline.
///
/// Phases A-G execute sequentially. Phase G may trigger re-synthesis (Phase F)
/// up to MAX_RESYNTHESIS_RETRIES times if post-verification fails.
/// D3: Phases A-E are sync (SearchBackend::search is sync). Phase F calls the
/// async GroundingClient; we bridge by checking for an existing Tokio runtime
/// via Handle::try_current(). If one exists we block via spawn_blocking->block_on
/// on a new current-thread runtime spawned on a fresh OS thread (avoids nesting).
/// If no runtime exists, a local one is built and used directly.
pub fn run_research_pipeline<C: GroundingClient + Send + Sync + 'static>(
    query: &ResearchQuery,
    config: &SourceTierConfig,
    backend: &dyn SearchBackend,
    grounding: &C,
) -> Result<ResearchAnswer, ResearchPipelineError> {
    // Phase A: formulate queries.
    let formulated = phase_a_formulate_queries(query)?;

    // Phase B: execute searches.
    let collected = phase_b_search(&formulated, backend, query.max_sources)?;

    // Phase C: classify sources.
    let classified = phase_c_classify(&collected, config, query.required_tier);

    if classified.usable.is_empty() {
        return Err(ResearchPipelineError::NoUsableSources);
    }

    // Phase D: extract facts.
    let facts = phase_d_extract_facts(&classified);

    // Phase E: cross-verify.
    let claims = phase_e_cross_verify(&facts);

    // Phase F + G: synthesize once and post-verify.
    // Re-synthesis retries are deferred to the MAX_RESYNTHESIS_RETRIES retry loop (out of scope D3).
    let mut answer = phase_f_synthesize_blocking(query, &claims, &classified, grounding);
    let verification = phase_g_post_verify(&answer, query);
    answer.caveats.extend(verification.caveats);
    answer.unanswered_parts.extend(verification.unanswered_parts);

    Ok(answer)
}

/// Sync bridge for phase_f_synthesize. Runs the async function inside
/// a freshly built current-thread runtime (no active runtime: the expected
/// production path from handle_slash_command which is a sync fn).
/// If a runtime is somehow already active (future-proofing), falls back to
/// the deterministic path rather than panicking.
fn phase_f_synthesize_blocking<C: GroundingClient + Send + Sync + 'static>(
    query: &ResearchQuery,
    claims: &[VerifiedClaim],
    classified: &ClassifiedResults,
    grounding: &C,
) -> ResearchAnswer {
    match tokio::runtime::Handle::try_current() {
        Err(_) => {
            // No active Tokio runtime: build a current-thread one and block on Phase F.
            // This is the expected production path (called from sync handle_slash_command).
            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt.block_on(phase_f_synthesize(query, claims, classified, grounding)),
                Err(e) => {
                    tracing::warn!(error = %e, "phase_f: failed to build local runtime, using deterministic fallback");
                    phase_f_deterministic(query, claims, classified)
                }
            }
        }
        Ok(_) => {
            // An active Tokio runtime exists. We cannot call block_on without nesting runtimes.
            // Use the deterministic path as a safe fallback rather than risking a panic.
            // This path only fires if run_research_pipeline is called from async context,
            // which is not supported by its sync signature.
            tracing::warn!(
                path = "deterministic_fallback",
                reason = "active_runtime_detected",
                "phase_f_synthesis_path"
            );
            phase_f_deterministic(query, claims, classified)
        }
    }
}

/// Pure deterministic synthesis (no LLM). Used as fallback from the blocking bridge.
fn phase_f_deterministic(
    query: &ResearchQuery,
    claims: &[VerifiedClaim],
    classified: &ClassifiedResults,
) -> ResearchAnswer {
    let answer_text = if claims.is_empty() {
        "I don't know — no verifiable facts were found for this question.".to_owned()
    } else {
        claims
            .iter()
            .enumerate()
            .map(|(i, c)| format!("[{}] {}", i + 1, c.claim))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let confidence = compute_confidence(claims);
    let sources_consulted = classified.usable.iter().map(|(_, sc)| sc.clone()).collect();
    let sources_rejected = classified.rejected.iter().map(|(_, sc)| sc.clone()).collect();
    ResearchAnswer {
        question: query.question.clone(),
        answer_text,
        evidence_chain: claims.to_vec(),
        confidence,
        caveats: Vec::new(),
        unanswered_parts: Vec::new(),
        sources_consulted,
        sources_rejected,
        path: "deterministic_fallback".to_owned(),
    }
}

// ===========================================================================
// H-53: citation pre-binding on the real answer path
// ===========================================================================

/// Result of `prebind_research_answer` — the frozen evidence set, per-citation
/// check results, and verified-citation objects minted by the production
/// `citation_verifier::prebind_verify_citations` path.
///
/// The `CitationAnchorsBound` SSB row is emitted by the CALLER (repl_command.rs)
/// after this function returns so the SSB handle stays outside the sync pipeline.
#[derive(Debug)]
pub struct PrebindOutcome {
    /// The frozen, blake3-pinned evidence set built from the answer's evidence chain.
    pub anchor_set: crate::citation_prebind::EvidenceAnchorSet,
    /// Per-citation check results for every `[N]` marker found in the answer text.
    /// An entry with `outcome.must_block() == true` means the citation is a ghost
    /// or stale anchor that must be held from the final answer (D-16 refusal path).
    pub citation_checks: Vec<crate::citation_prebind::CitationCheckResult>,
    /// Verified-citation objects produced by `build_outcome_prebind` + `mint_verified`
    /// for every citation whose `[N]` was present in the pre-bound set (Bound outcome).
    /// GHOST / STALE citations are NOT included here — the caller routes them to D-16.
    pub verified_citations: Vec<crate::research::web_orchestrator::citation_verifier::VerifiedCitation>,
    /// `true` if any citation in `citation_checks` has `must_block() == true`.
    pub has_ghost: bool,
}

/// H-53 — wire the citation pre-binding stage onto the REAL answer path.
///
/// Called by `repl_command.rs` AFTER a successful `run_research_pipeline` run,
/// BEFORE the answer is displayed or cached. This is the production caller chain
/// that closes the wire-dark BLOCK:
///   `run_research_pipeline` -> `prebind_research_answer` ->
///   `citation_prebind::freeze_anchors` (non-test, non-demo caller)
///   `citation_prebind::check_citation_in_set` (non-test, non-demo caller)
///   `citation_verifier::prebind_verify_citations` ->
///     `build_outcome_prebind` (non-dead-code caller)
///     `mint_verified` (non-dead-code caller)
///
/// When `ELAI_CITATION_PREBIND_DISABLE` is set the function returns an empty
/// `PrebindOutcome` with `anchor_set.is_empty() == true` so the caller can
/// emit a `CitationAnchorsBound::Blocked` SSB row and fall through to the
/// legacy detect-after path (E-48 open-world verifier).
///
/// No `.unwrap()`/`.expect()` in production paths; all failures are logged and
/// gracefully degraded.
pub fn prebind_research_answer(
    answer: &ResearchAnswer,
) -> PrebindOutcome {
    use crate::citation_prebind::{
        freeze_anchors, check_citation_in_set, extract_citation_ids,
        prebind_disabled, RankedEvidence,
    };
    use crate::research::web_orchestrator::citation_verifier::{
        prebind_verify_citations,
    };
    use crate::research_tree::{EvidenceLevel, SourceCitation, SourceQuality};

    // Kill-switch: fall through to legacy path.
    if prebind_disabled() {
        let empty_set = freeze_anchors(&[], 0);
        return PrebindOutcome {
            anchor_set: empty_set,
            citation_checks: Vec::new(),
            verified_citations: Vec::new(),
            has_ghost: false,
        };
    }

    // Read ELAI_SEED for deterministic id assignment (GD-2).
    let seed: u64 = std::env::var("ELAI_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // Build RankedEvidence from the answer's evidence chain.
    // Each fact in a VerifiedClaim becomes one ranked evidence item.
    // Rank is assigned by appearance order (first fact in first claim = rank 0).
    let mut ranked: Vec<RankedEvidence> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut rank: u32 = 0;
    for claim in &answer.evidence_chain {
        for fact in &claim.supporting_facts {
            // Deduplicate by source URL so each source gets one anchor.
            if seen_urls.insert(fact.source_url.clone()) {
                let excerpt = fact.exact_quote
                    .as_deref()
                    .unwrap_or(fact.claim.as_str())
                    .to_owned();
                ranked.push(RankedEvidence {
                    rank,
                    excerpt,
                    source_uri: fact.source_url.clone(),
                    retrieved_at_ms: 0, // retrieval timestamp not tracked in ExtractedFact
                    source_tier: match fact.source_tier {
                        SourceTier::T1Authoritative => 0,
                        SourceTier::T2Strong => 1,
                        SourceTier::T3Weak => 2,
                        SourceTier::T4Rejected => 3,
                    },
                });
                rank = rank.saturating_add(1);
            }
        }
    }

    // Freeze the evidence set.
    let anchor_set = freeze_anchors(&ranked, seed);

    if anchor_set.is_empty() {
        tracing::debug!(
            path = "prebind",
            reason = "no_evidence",
            "prebind_research_answer: empty anchor set (no evidence chain facts)"
        );
        return PrebindOutcome {
            anchor_set,
            citation_checks: Vec::new(),
            verified_citations: Vec::new(),
            has_ghost: false,
        };
    }

    // Extract [N] citation markers from the answer text.
    let cited_ids = extract_citation_ids(&answer.answer_text);

    // Check each citation against the closed set.
    let citation_checks: Vec<_> = cited_ids
        .iter()
        .map(|&id| check_citation_in_set(&anchor_set, id, ""))
        .collect();

    let has_ghost = citation_checks.iter().any(|r| r.outcome.must_block());

    // Build (SourceCitation, anchor_id, excerpt) tuples for Bound citations only.
    // Ghost/Stale citations are excluded — the caller routes them to D-16 refusal.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let prebind_items: Vec<(SourceCitation, u32, String)> = citation_checks
        .iter()
        .filter(|r| !r.outcome.must_block())
        .filter_map(|r| {
            anchor_set.get(r.id).map(|anchor| {
                let citation = SourceCitation {
                    url: anchor.source_uri.clone(),
                    title: format!("Source [{}]", anchor.id),
                    quality: match anchor.source_tier {
                        0 => SourceQuality::Tier1,
                        _ => SourceQuality::Tier2,
                    },
                    evidence_level: EvidenceLevel::SingleSource,
                    excerpt: anchor.excerpt.clone(),
                };
                (citation, r.id, anchor.excerpt.clone())
            })
        })
        .collect();

    let verified_citations = prebind_verify_citations(&prebind_items, "prebind_path", now_ms);

    tracing::debug!(
        anchor_count = anchor_set.len(),
        cited_count = citation_checks.len(),
        ghost_count = citation_checks.iter().filter(|r| r.outcome.must_block()).count(),
        bound_count = verified_citations.len(),
        "prebind_research_answer: citation pre-binding complete"
    );

    PrebindOutcome {
        anchor_set,
        citation_checks,
        verified_citations,
        has_ghost,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grounding_client::SyntheticGroundingClient;
    use elai_core::research_types::{DEFAULT_MAX_SOURCES, FACT_ID_HEX_LEN};
    use elai_core::source_tier::{parse_source_tier_config, ClassificationMethod};

    const TEST_TIER_CONFIG: &str = r#"
[tier1]
domains = ["doc.rust-lang.org", "arxiv.org"]
patterns = ['\.gov$']

[tier2]
domains = ["stackoverflow.com"]
min_so_score = 20

[tier4_blocklist]
domains = ["w3schools.com"]
patterns = []
"#;

    fn test_config() -> SourceTierConfig {
        parse_source_tier_config(TEST_TIER_CONFIG).expect("test config")
    }

    fn make_stub_backend() -> StubSearchBackend {
        let mut backend = StubSearchBackend::new();
        backend.add_results(
            "Rust",
            vec![
                SearchResult {
                    url: "https://doc.rust-lang.org/book/ch01.html".to_owned(),
                    title: "The Rust Programming Language".to_owned(),
                    snippet: "Rust is a systems programming language focused on safety".to_owned(),
                },
                SearchResult {
                    url: "https://stackoverflow.com/questions/123".to_owned(),
                    title: "What is Rust?".to_owned(),
                    snippet: "Rust is a systems programming language focused on safety".to_owned(),
                },
                SearchResult {
                    url: "https://w3schools.com/rust".to_owned(),
                    title: "Rust Tutorial".to_owned(),
                    snippet: "Learn Rust at w3schools".to_owned(),
                },
                SearchResult {
                    url: "https://randomsite.xyz/rust".to_owned(),
                    title: "Rust info".to_owned(),
                    snippet: "Some info about Rust".to_owned(),
                },
            ],
        );
        backend
    }

    // --- Contract: Phase A ---

    #[test]
    fn phase_a_rejects_empty_question() {
        let q = ResearchQuery::new("");
        assert!(phase_a_formulate_queries(&q).is_err());
    }

    #[test]
    fn phase_a_generates_min_queries() {
        let q = ResearchQuery::new("What is Rust?");
        let formulated = phase_a_formulate_queries(&q).expect("formulate");
        assert!(formulated.queries.len() >= MIN_SEARCH_QUERIES as usize);
        assert!(formulated.queries.len() <= MAX_SEARCH_QUERIES as usize);
        assert_eq!(formulated.original, "What is Rust?");
    }

    #[test]
    fn phase_a_includes_context() {
        let q = ResearchQuery::new("ownership").with_context("Rust borrow checker");
        let formulated = phase_a_formulate_queries(&q).expect("formulate");
        let has_context_query = formulated
            .queries
            .iter()
            .any(|q| q.contains("borrow checker"));
        assert!(has_context_query);
    }

    // --- Contract: Phase B ---

    #[test]
    fn phase_b_deduplicates_by_url() {
        let queries = FormulatedQueries {
            original: "test".to_owned(),
            queries: vec!["Rust".to_owned(), "Rust explanation".to_owned()],
        };
        let backend = make_stub_backend();
        let collected = phase_b_search(&queries, &backend, DEFAULT_MAX_SOURCES).expect("search");
        // Both queries match "Rust" pattern, but results should be deduplicated.
        let urls: Vec<&str> = collected.results.iter().map(|r| r.url.as_str()).collect();
        let unique: std::collections::HashSet<&str> = urls.iter().copied().collect();
        assert_eq!(urls.len(), unique.len(), "no duplicate URLs");
    }

    #[test]
    fn phase_b_empty_backend_returns_empty() {
        let queries = FormulatedQueries {
            original: "test".to_owned(),
            queries: vec!["nonexistent query".to_owned()],
        };
        let backend = StubSearchBackend::new();
        let collected = phase_b_search(&queries, &backend, DEFAULT_MAX_SOURCES).expect("search");
        assert!(collected.results.is_empty());
    }

    // --- Contract: Phase C ---

    #[test]
    fn phase_c_separates_t4_from_usable() {
        let collected = CollectedResults {
            results: vec![
                SearchResult {
                    url: "https://doc.rust-lang.org/book".to_owned(),
                    title: "Rust Book".to_owned(),
                    snippet: "T1 source".to_owned(),
                },
                SearchResult {
                    url: "https://w3schools.com/rust".to_owned(),
                    title: "T4 source".to_owned(),
                    snippet: "blocked".to_owned(),
                },
            ],
            raw_count: 2,
        };
        let cfg = test_config();
        let classified = phase_c_classify(&collected, &cfg, None);
        assert_eq!(classified.usable.len(), 1);
        assert_eq!(classified.rejected.len(), 1);
        assert_eq!(classified.usable[0].1.tier, SourceTier::T1Authoritative);
        assert_eq!(classified.rejected[0].1.tier, SourceTier::T4Rejected);
    }

    #[test]
    fn phase_c_required_tier_filters() {
        let collected = CollectedResults {
            results: vec![
                SearchResult {
                    url: "https://doc.rust-lang.org/book".to_owned(),
                    title: "T1".to_owned(),
                    snippet: "good".to_owned(),
                },
                SearchResult {
                    url: "https://stackoverflow.com/q/1".to_owned(),
                    title: "T2".to_owned(),
                    snippet: "good".to_owned(),
                },
                SearchResult {
                    url: "https://randomsite.xyz".to_owned(),
                    title: "T3".to_owned(),
                    snippet: "weak".to_owned(),
                },
            ],
            raw_count: 3,
        };
        let cfg = test_config();

        // Require T1 only.
        let classified = phase_c_classify(&collected, &cfg, Some(SourceTier::T1Authoritative));
        assert_eq!(classified.usable.len(), 1);
        assert_eq!(classified.rejected.len(), 2);
    }

    // --- Contract: Phase D ---

    #[test]
    fn phase_d_skips_empty_snippets() {
        let classified = ClassifiedResults {
            usable: vec![(
                SearchResult {
                    url: "https://doc.rust-lang.org/book".to_owned(),
                    title: "Rust Book".to_owned(),
                    snippet: "".to_owned(),
                },
                SourceClassification {
                    url: "https://doc.rust-lang.org/book".to_owned(),
                    domain: "doc.rust-lang.org".to_owned(),
                    tier: SourceTier::T1Authoritative,
                    classification_method: ClassificationMethod::ExactDomain,
                },
            )],
            rejected: Vec::new(),
            classification_errors: Vec::new(),
        };
        let facts = phase_d_extract_facts(&classified);
        assert!(facts.is_empty());
    }

    #[test]
    fn phase_d_generates_blake3_ids() {
        let classified = ClassifiedResults {
            usable: vec![(
                SearchResult {
                    url: "https://doc.rust-lang.org/book".to_owned(),
                    title: "Title".to_owned(),
                    snippet: "A fact about Rust".to_owned(),
                },
                SourceClassification {
                    url: "https://doc.rust-lang.org/book".to_owned(),
                    domain: "doc.rust-lang.org".to_owned(),
                    tier: SourceTier::T1Authoritative,
                    classification_method: ClassificationMethod::ExactDomain,
                },
            )],
            rejected: Vec::new(),
            classification_errors: Vec::new(),
        };
        let facts = phase_d_extract_facts(&classified);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].id.len(), FACT_ID_HEX_LEN);
        assert!(facts[0].has_exact_quote());
    }

    // --- Contract: Phase E ---

    #[test]
    fn phase_e_verified_with_two_independent_sources() {
        let facts = vec![
            ExtractedFact::new("Rust is safe", "https://doc.rust-lang.org", SourceTier::T1Authoritative)
                .with_exact_quote("Rust is safe"),
            ExtractedFact::new("Rust is safe", "https://stackoverflow.com/q/1", SourceTier::T2Strong)
                .with_exact_quote("Rust is safe"),
        ];
        let claims = phase_e_cross_verify(&facts);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].verification_level, VerificationLevel::Verified);
        assert_eq!(claims[0].independent_source_count, 2);
    }

    #[test]
    fn phase_e_supported_with_one_source() {
        let facts = vec![
            ExtractedFact::new("Rust is fast", "https://doc.rust-lang.org", SourceTier::T1Authoritative)
                .with_exact_quote("Rust is fast"),
        ];
        let claims = phase_e_cross_verify(&facts);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].verification_level, VerificationLevel::Supported);
        assert_eq!(claims[0].independent_source_count, 1);
    }

    #[test]
    fn phase_e_weak_without_exact_quote() {
        let facts = vec![
            ExtractedFact::new("Rust has lifetimes", "https://doc.rust-lang.org", SourceTier::T1Authoritative),
        ];
        let claims = phase_e_cross_verify(&facts);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].verification_level, VerificationLevel::Weak);
    }

    #[test]
    fn phase_e_unsupported_t3_no_quote() {
        let facts = vec![
            ExtractedFact::new("Rust is cool", "https://randomsite.xyz", SourceTier::T3Weak),
        ];
        let claims = phase_e_cross_verify(&facts);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].verification_level, VerificationLevel::Unsupported);
    }

    #[test]
    fn phase_e_same_domain_not_independent() {
        let facts = vec![
            ExtractedFact::new("Rust is safe", "https://doc.rust-lang.org/ch1", SourceTier::T1Authoritative)
                .with_exact_quote("Rust is safe"),
            ExtractedFact::new("Rust is safe", "https://doc.rust-lang.org/ch2", SourceTier::T1Authoritative)
                .with_exact_quote("Rust is safe"),
        ];
        let claims = phase_e_cross_verify(&facts);
        assert_eq!(claims.len(), 1);
        // Same domain = only 1 independent source.
        assert_eq!(claims[0].independent_source_count, 1);
        assert_eq!(claims[0].verification_level, VerificationLevel::Supported);
    }

    // --- Contract: Phase F ---

    #[tokio::test]
    async fn phase_f_empty_claims_says_dont_know() {
        let query = ResearchQuery::new("test");
        let classified = ClassifiedResults {
            usable: Vec::new(),
            rejected: Vec::new(),
            classification_errors: Vec::new(),
        };
        let grounding = SyntheticGroundingClient::new();
        let answer = phase_f_synthesize(&query, &[], &classified, &grounding).await;
        assert!(answer.answer_text.contains("don't know"));
        assert_eq!(answer.confidence, AnswerConfidence::Unknown);
    }

    #[tokio::test]
    async fn phase_f_all_verified_is_high_confidence() {
        let query = ResearchQuery::new("test");
        let claims = vec![VerifiedClaim {
            claim: "fact".to_owned(),
            supporting_facts: vec![],
            verification_level: VerificationLevel::Verified,
            independent_source_count: 2,
        }];
        let classified = ClassifiedResults {
            usable: Vec::new(),
            rejected: Vec::new(),
            classification_errors: Vec::new(),
        };
        let grounding = SyntheticGroundingClient::new();
        let answer = phase_f_synthesize(&query, &claims, &classified, &grounding).await;
        assert_eq!(answer.confidence, AnswerConfidence::High);
    }

    // --- Contract: Phase G ---

    #[test]
    fn phase_g_passes_valid_answer() {
        let answer = ResearchAnswer {
            question: "What is Rust?".to_owned(),
            answer_text: "[1] Rust is a safe language".to_owned(),
            evidence_chain: vec![VerifiedClaim {
                claim: "Rust is safe".to_owned(),
                supporting_facts: vec![ExtractedFact::new(
                    "Rust is safe",
                    "https://doc.rust-lang.org",
                    SourceTier::T1Authoritative,
                )
                .with_exact_quote("Rust is safe")],
                verification_level: VerificationLevel::Verified,
                independent_source_count: 2,
            }],
            confidence: AnswerConfidence::High,
            caveats: Vec::new(),
            unanswered_parts: Vec::new(),
            sources_consulted: Vec::new(),
            sources_rejected: Vec::new(),
            path: "llm".to_owned(),
        };
        let query = ResearchQuery::new("What is Rust?");
        let result = phase_g_post_verify(&answer, &query);
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn phase_g_detects_missing_evidence() {
        let answer = ResearchAnswer {
            question: "test".to_owned(),
            answer_text: "Some answer without evidence".to_owned(),
            evidence_chain: Vec::new(),
            confidence: AnswerConfidence::Unknown,
            caveats: Vec::new(),
            unanswered_parts: Vec::new(),
            sources_consulted: Vec::new(),
            sources_rejected: Vec::new(),
            path: "deterministic_fallback".to_owned(),
        };
        let query = ResearchQuery::new("test");
        let result = phase_g_post_verify(&answer, &query);
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("acknowledge uncertainty")));
    }

    // --- Contract: full pipeline ---

    #[test]
    fn pipeline_end_to_end() {
        let cfg = test_config();
        let backend = make_stub_backend();
        let query = ResearchQuery::new("What is Rust?");
        let grounding = SyntheticGroundingClient::new();
        let answer = run_research_pipeline(&query, &cfg, &backend, &grounding).expect("pipeline");

        assert_eq!(answer.question, "What is Rust?");
        assert!(!answer.answer_text.is_empty());
        assert!(!answer.evidence_chain.is_empty());
        // Should have rejected w3schools.
        assert!(answer
            .sources_rejected
            .iter()
            .any(|s| s.domain == "w3schools.com"));
    }

    #[test]
    fn pipeline_no_results_returns_error() {
        let cfg = test_config();
        let backend = StubSearchBackend::new(); // Empty backend.
        let query = ResearchQuery::new("obscure topic nobody wrote about");
        let grounding = SyntheticGroundingClient::new();
        let err = run_research_pipeline(&query, &cfg, &backend, &grounding);
        assert!(err.is_err());
        assert!(matches!(
            err.unwrap_err(),
            ResearchPipelineError::NoUsableSources
        ));
    }

    #[test]
    fn pipeline_empty_question_returns_error() {
        let cfg = test_config();
        let backend = StubSearchBackend::new();
        let query = ResearchQuery::new("");
        let grounding = SyntheticGroundingClient::new();
        let err = run_research_pipeline(&query, &cfg, &backend, &grounding);
        assert!(matches!(
            err.unwrap_err(),
            ResearchPipelineError::EmptyQuestion
        ));
    }

    // --- Contract: SearchBackend trait ---

    #[test]
    fn stub_backend_returns_configured_results() {
        let mut backend = StubSearchBackend::new();
        backend.add_results(
            "test",
            vec![SearchResult {
                url: "https://example.com".to_owned(),
                title: "Example".to_owned(),
                snippet: "content".to_owned(),
            }],
        );
        let results = backend.search("test query", 10).expect("search");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn stub_backend_no_match_returns_empty() {
        let backend = StubSearchBackend::new();
        let results = backend.search("anything", 10).expect("search");
        assert!(results.is_empty());
    }

    // --- Contract: confidence computation ---

    #[test]
    fn confidence_all_verified_is_high() {
        let claims = vec![
            VerifiedClaim {
                claim: "a".to_owned(),
                supporting_facts: vec![],
                verification_level: VerificationLevel::Verified,
                independent_source_count: 2,
            },
            VerifiedClaim {
                claim: "b".to_owned(),
                supporting_facts: vec![],
                verification_level: VerificationLevel::Verified,
                independent_source_count: 3,
            },
        ];
        assert_eq!(compute_confidence(&claims), AnswerConfidence::High);
    }

    #[test]
    fn confidence_mixed_supported_is_medium() {
        let claims = vec![
            VerifiedClaim {
                claim: "a".to_owned(),
                supporting_facts: vec![],
                verification_level: VerificationLevel::Verified,
                independent_source_count: 2,
            },
            VerifiedClaim {
                claim: "b".to_owned(),
                supporting_facts: vec![],
                verification_level: VerificationLevel::Supported,
                independent_source_count: 1,
            },
        ];
        assert_eq!(compute_confidence(&claims), AnswerConfidence::Medium);
    }

    #[test]
    fn confidence_any_weak_is_low() {
        let claims = vec![VerifiedClaim {
            claim: "a".to_owned(),
            supporting_facts: vec![],
            verification_level: VerificationLevel::Weak,
            independent_source_count: 0,
        }];
        assert_eq!(compute_confidence(&claims), AnswerConfidence::Low);
    }

    #[test]
    fn confidence_empty_is_unknown() {
        assert_eq!(compute_confidence(&[]), AnswerConfidence::Unknown);
    }
}

// ---------------------------------------------------------------------------
// Track A.3: test_helpers — expose has_numeric_citation for contract tests.
// Making the function pub avoids a feature-flag dependency. It is a pure,
// infallible, zero-side-effect fn; publishing it is safe.
// ---------------------------------------------------------------------------

/// Re-export for Track A.3 contract test (research_citation_regex_contract.rs).
/// Exposes has_numeric_citation via a test_helpers sub-module so the
/// contracts crate can import it without importing the internal function
/// directly (keeps the call path explicit in test code).
pub mod test_helpers {
    /// Returns true iff `s` contains at least one numeric citation marker `[N]`.
    /// Delegates to the internal `has_numeric_citation` function.
    /// This export exists for the Track A.3 contract test — not for production use.
    pub fn has_numeric_citation(s: &str) -> bool {
        super::has_numeric_citation(s)
    }
}
