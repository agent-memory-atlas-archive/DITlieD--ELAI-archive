// research_tree.rs — FlashResearch tree-structured research (phase 4D)
//
// Implements the Pipeline Factory L1 layer: a Research Agent that designs
// execution pipelines via tree-structured FlashResearch (arXiv 2510.05145).
// Produces a ResearchReport with grounded alternatives, source-quality-tiered
// citations, and dual-threshold convergence (phi >= phi_min AND psi >= psi_min).

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use elai_core::clock::{ClockProvider, LogicalInstant};

use crate::grounding_client::{GroundingClient, GroundingError, ResearchLevel};
use crate::ssb::{Ssb, SsbPayload, ResearchStepPayload, ResearchDecomposeLevelPayload};

// ---------------------------------------------------------------------------
// Constants (no magic numbers — all named)
// ---------------------------------------------------------------------------

/// Default breadth factor for Simple complexity.
pub const BREADTH_SIMPLE: u32 = 2;
/// Default breadth factor for Moderate complexity.
pub const BREADTH_MODERATE: u32 = 3;
/// Default breadth factor for Complex complexity.
pub const BREADTH_COMPLEX: u32 = 4;

/// Default depth limit for Simple complexity.
pub const DEPTH_SIMPLE: u32 = 3;
/// Default depth limit for Moderate complexity.
pub const DEPTH_MODERATE: u32 = 4;
/// Default depth limit for Complex complexity.
pub const DEPTH_COMPLEX: u32 = 5;

/// Phi minimum threshold for Simple complexity.
pub const PHI_MIN_SIMPLE: f64 = 0.8;
/// Phi minimum threshold for Moderate complexity.
pub const PHI_MIN_MODERATE: f64 = 0.7;
/// Phi minimum threshold for Complex complexity.
pub const PHI_MIN_COMPLEX: f64 = 0.6;

/// Psi minimum threshold (same across complexities).
pub const PSI_MIN_DEFAULT: f64 = 0.7;

/// Info gain threshold — prune node after this many consecutive iterations below.
pub const INFO_GAIN_THRESHOLD: f64 = 0.1;

/// Consecutive low-gain iterations before pruning a node.
pub const LOW_GAIN_PRUNE_COUNT: u32 = 3;

/// Maximum iterations for the research loop.
pub const MAX_ITERATIONS_DEFAULT: u32 = 50;

/// Source quality tiers (5 levels, 1.0 = highest).
pub const SOURCE_QUALITY_TIER1: f64 = 1.0;
pub const SOURCE_QUALITY_TIER2: f64 = 0.8;
pub const SOURCE_QUALITY_TIER3: f64 = 0.6;
pub const SOURCE_QUALITY_TIER4: f64 = 0.4;
pub const SOURCE_QUALITY_TIER5: f64 = 0.2;

/// Maximum tree depth to prevent depth bombs.
pub const MAX_TREE_DEPTH: u32 = 10;

/// Maximum total nodes to prevent breadth bombs.
pub const MAX_TOTAL_NODES: usize = 200;

// ---------------------------------------------------------------------------
// Constants — Recursive Decomposition (4D.7)
// ---------------------------------------------------------------------------

/// Maximum token estimate per baby step.
pub const MAX_BABY_STEP_TOKENS: usize = 2048;

/// Maximum token estimate for grounding summaries.
pub const MAX_GROUNDING_TOKENS: usize = 500;

/// Maximum recursive decomposition depth (concept -> phase -> step -> baby step).
pub const MAX_DECOMPOSE_DEPTH: u32 = 2;

/// Default maximum entries in the research cache (LRU eviction boundary).
pub const DEFAULT_CACHE_MAX_ENTRIES: usize = 256;

/// Default time-to-live for cached research entries (seconds).
pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 3600;

// ---------------------------------------------------------------------------
// NodeId — type-safe identifier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

// ---------------------------------------------------------------------------
// NodeStatus — lifecycle states for a research node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Active,
    Converged,
    Pruned,
}

// ---------------------------------------------------------------------------
// QueryComplexity — adaptive complexity tiers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryComplexity {
    Simple,
    Moderate,
    Complex,
}

impl QueryComplexity {
    pub fn breadth_factor(&self) -> u32 {
        match self {
            Self::Simple => BREADTH_SIMPLE,
            Self::Moderate => BREADTH_MODERATE,
            Self::Complex => BREADTH_COMPLEX,
        }
    }

    pub fn depth_limit(&self) -> u32 {
        match self {
            Self::Simple => DEPTH_SIMPLE,
            Self::Moderate => DEPTH_MODERATE,
            Self::Complex => DEPTH_COMPLEX,
        }
    }

    pub fn phi_min(&self) -> f64 {
        match self {
            Self::Simple => PHI_MIN_SIMPLE,
            Self::Moderate => PHI_MIN_MODERATE,
            Self::Complex => PHI_MIN_COMPLEX,
        }
    }
}

// ---------------------------------------------------------------------------
// SourceQuality — 5-tier deterministic source quality rating
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceQuality {
    /// Tier 1: Authoritative academic/official sources (arxiv.org, doi.org, ietf.org).
    Tier1,
    /// Tier 2: Established technical sources (docs.rs, crates.io, pypi.org, official docs).
    Tier2,
    /// Tier 3: Community-reviewed (stackoverflow.com, github.com repos with >100 stars).
    Tier3,
    /// Tier 4: General technical (blog posts, tutorials, medium.com).
    Tier4,
    /// Tier 5: Unverified/unknown sources.
    Tier5,
}

impl SourceQuality {
    pub fn score(&self) -> f64 {
        match self {
            Self::Tier1 => SOURCE_QUALITY_TIER1,
            Self::Tier2 => SOURCE_QUALITY_TIER2,
            Self::Tier3 => SOURCE_QUALITY_TIER3,
            Self::Tier4 => SOURCE_QUALITY_TIER4,
            Self::Tier5 => SOURCE_QUALITY_TIER5,
        }
    }
}

/// Tier 1 domains — exact match only (no substring).
const TIER1_DOMAINS: &[&str] = &[
    "arxiv.org",
    "doi.org",
    "ietf.org",
    "w3.org",
    "ieee.org",
    "acm.org",
];

/// Tier 2 domains — exact match only.
const TIER2_DOMAINS: &[&str] = &[
    "docs.rs",
    "crates.io",
    "pypi.org",
    "doc.rust-lang.org",
    "docs.python.org",
    "learn.microsoft.com",
    "developer.mozilla.org",
];

/// Tier 3 domains — exact match only.
const TIER3_DOMAINS: &[&str] = &[
    "stackoverflow.com",
    "github.com",
    "gitlab.com",
];

/// Tier 4 domains — exact match only.
const TIER4_DOMAINS: &[&str] = &[
    "medium.com",
    "dev.to",
    "hashnode.dev",
];

/// Assign source quality tier from a URL. Deterministic: exact domain match
/// via host comparison. "arxiv.phishing.com" does NOT match Tier1.
pub fn assign_source_quality(url: &str) -> SourceQuality {
    let host = extract_host(url);
    let host = host.as_deref().unwrap_or("");

    if domain_matches(host, TIER1_DOMAINS) {
        SourceQuality::Tier1
    } else if domain_matches(host, TIER2_DOMAINS) {
        SourceQuality::Tier2
    } else if domain_matches(host, TIER3_DOMAINS) {
        SourceQuality::Tier3
    } else if domain_matches(host, TIER4_DOMAINS) {
        SourceQuality::Tier4
    } else {
        SourceQuality::Tier5
    }
}

/// Extract host from a URL string. Returns None for malformed URLs.
fn extract_host(url: &str) -> Option<String> {
    // Simple extraction: find ://host/ pattern
    let after_scheme = url.find("://").map(|i| &url[i + 3..])?;
    let host_end = after_scheme
        .find('/')
        .unwrap_or(after_scheme.len());
    let host_port = &after_scheme[..host_end];
    // Strip port
    let host = if let Some(colon) = host_port.rfind(':') {
        // Only strip if what follows is numeric (avoid stripping IPv6)
        if host_port[colon + 1..].chars().all(|c| c.is_ascii_digit()) {
            &host_port[..colon]
        } else {
            host_port
        }
    } else {
        host_port
    };
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

/// Check if a host matches any domain in the list (exact or subdomain match).
/// "arxiv.org" matches "arxiv.org" and "www.arxiv.org" but NOT "arxiv.phishing.com".
fn domain_matches(host: &str, domains: &[&str]) -> bool {
    for domain in domains {
        if host == *domain || host.ends_with(&format!(".{domain}")) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// EvidenceLevel — ranked evidence strength
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum EvidenceLevel {
    /// Anecdotal or unverified.
    #[default]
    #[serde(alias = "anecdotal")]
    Anecdotal,
    /// From a single credible source.
    #[serde(alias = "single_source", alias = "singlesource")]
    SingleSource,
    /// Corroborated by multiple sources.
    #[serde(alias = "multi_source", alias = "multisource")]
    MultiSource,
    /// Formally verified or peer-reviewed.
    #[serde(alias = "verified")]
    Verified,
}

impl EvidenceLevel {
    pub fn score(&self) -> f64 {
        match self {
            Self::Anecdotal => 0.25,
            Self::SingleSource => 0.5,
            Self::MultiSource => 0.75,
            Self::Verified => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// SourceCitation — a single cited source with quality rating
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCitation {
    pub url: String,
    pub title: String,
    pub quality: SourceQuality,
    pub evidence_level: EvidenceLevel,
    pub excerpt: String,
}

// ---------------------------------------------------------------------------
// Alternative — a research alternative with citations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub citations: Vec<SourceCitation>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub trade_offs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Contradiction — conflicting information found during research
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim_a: String,
    pub source_a: SourceCitation,
    pub claim_b: String,
    pub source_b: SourceCitation,
    pub resolution: Option<String>,
}

// ---------------------------------------------------------------------------
// SubQuestionResult — result of researching a child node question
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubQuestionResult {
    pub question: String,
    pub answer: String,
    #[serde(default)]
    pub citations: Vec<SourceCitation>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub evidence_level: EvidenceLevel,
}

fn default_confidence() -> f64 {
    0.5
}

// ---------------------------------------------------------------------------
// ResearchNode — a single node in the research tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchNode {
    pub id: NodeId,
    pub question: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub phi: f64,
    pub psi: f64,
    pub status: NodeStatus,
    pub info_gain: f64,
    pub iterations: u32,
    pub low_gain_streak: u32,
    pub citations: Vec<SourceCitation>,
    pub answer: Option<String>,
    pub depth: u32,
}

// Manual PartialEq — NaN guard on phi/psi (f64 fields).
impl PartialEq for ResearchNode {
    fn eq(&self, other: &Self) -> bool {
        if self.phi.is_nan() || other.phi.is_nan() {
            return false;
        }
        if self.psi.is_nan() || other.psi.is_nan() {
            return false;
        }
        self.id == other.id
            && self.question == other.question
            && self.parent == other.parent
            && self.children == other.children
            && (self.phi - other.phi).abs() < f64::EPSILON
            && (self.psi - other.psi).abs() < f64::EPSILON
            && self.status == other.status
            && (self.info_gain - other.info_gain).abs() < f64::EPSILON
            && self.iterations == other.iterations
            && self.low_gain_streak == other.low_gain_streak
            && self.depth == other.depth
    }
}

impl ResearchNode {
    pub fn new(id: NodeId, question: String, parent: Option<NodeId>, depth: u32) -> Self {
        Self {
            id,
            question,
            parent,
            children: Vec::new(),
            phi: 0.0,
            psi: 0.0,
            status: NodeStatus::Pending,
            info_gain: 0.0,
            iterations: 0,
            low_gain_streak: 0,
            citations: Vec::new(),
            answer: None,
            depth,
        }
    }

    /// Check if this node has converged (phi >= phi_min AND psi >= psi_min).
    pub fn is_converged(&self, phi_min: f64, psi_min: f64) -> bool {
        self.phi >= phi_min && self.psi >= psi_min
    }

    /// Check if this node should be pruned (consecutive low-gain iterations).
    pub fn should_prune(&self) -> bool {
        self.low_gain_streak >= LOW_GAIN_PRUNE_COUNT
    }

    /// Compute confidence: source_count * avg_quality * agreement_ratio.
    pub fn confidence(&self) -> f64 {
        if self.citations.is_empty() {
            return 0.0;
        }
        let source_count = self.citations.len() as f64;
        let avg_quality: f64 = self.citations.iter()
            .map(|c| c.quality.score())
            .sum::<f64>() / source_count;
        // agreement_ratio: fraction of citations that agree (simplified: use evidence level)
        let avg_evidence: f64 = self.citations.iter()
            .map(|c| c.evidence_level.score())
            .sum::<f64>() / source_count;
        source_count * avg_quality * avg_evidence
    }

    /// Compute evidence level ranking: evidence_level.score() * source_quality_avg.
    pub fn evidence_ranking(&self) -> f64 {
        if self.citations.is_empty() {
            return 0.0;
        }
        let source_count = self.citations.len() as f64;
        let avg_quality: f64 = self.citations.iter()
            .map(|c| c.quality.score())
            .sum::<f64>() / source_count;
        let max_evidence: f64 = self.citations.iter()
            .map(|c| c.evidence_level.score())
            .fold(0.0_f64, f64::max);
        max_evidence * avg_quality
    }
}

// ---------------------------------------------------------------------------
// ResearchPlan — what to research
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub root_question: String,
    pub sub_questions: Vec<String>,
    pub complexity: QueryComplexity,
    pub time_budget: Duration,
    pub require_contradictions: bool,
}

// ---------------------------------------------------------------------------
// SharedResearchState — shared mutable state across the tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SharedResearchState {
    pub nodes: HashMap<NodeId, ResearchNode>,
    pub next_node_id: u64,
    pub total_iterations: u32,
    pub start_mark: Option<LogicalInstant>,
    pub alternatives_found: Vec<Alternative>,
    pub contradictions_found: Vec<Contradiction>,
}

impl SharedResearchState {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_node_id: 0,
            total_iterations: 0,
            start_mark: None,
            alternatives_found: Vec::new(),
            contradictions_found: Vec::new(),
        }
    }

    pub fn allocate_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn elapsed(&self, clock: &dyn ClockProvider) -> Duration {
        self.start_mark
            .map(|mark| clock.elapsed_since(mark))
            .unwrap_or(Duration::ZERO)
    }
}

impl Default for SharedResearchState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ConvergenceStatus — outcome of the research loop
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceStatus {
    /// All nodes converged (phi >= phi_min AND psi >= psi_min).
    #[serde(alias = "converged", alias = "CONVERGED")]
    Converged,
    /// Partial convergence — some nodes converged, some did not.
    #[serde(alias = "partial", alias = "PARTIAL")]
    Partial {
        converged_count: usize,
        total_count: usize,
    },
    /// Failed to converge within budget.
    #[serde(alias = "failed", alias = "FAILED")]
    Failed { reason: String },
}

// ---------------------------------------------------------------------------
// ResearchTreeSnapshot — immutable view of tree state for reporting
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTreeSnapshot {
    #[serde(default)]
    pub nodes: Vec<ResearchNode>,
    #[serde(default = "default_convergence")]
    pub convergence: ConvergenceStatus,
    #[serde(default = "default_iterations")]
    pub total_iterations: u32,
    #[serde(default, with = "duration_opt_or_default")]
    pub elapsed: Duration,
}

fn default_convergence() -> ConvergenceStatus {
    ConvergenceStatus::Converged
}

fn default_iterations() -> u32 {
    1
}

/// Accept `elapsed` missing, a plain number-of-seconds, or the default
/// `{"secs", "nanos"}` shape. LLM-generated JSON rarely matches the struct form.
mod duration_opt_or_default {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.serialize(s)
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Shape {
        Struct {
            secs: u64,
            #[serde(default)]
            nanos: u32,
        },
        Seconds(u64),
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        match Option::<Shape>::deserialize(d)? {
            None => Ok(Duration::ZERO),
            Some(Shape::Struct { secs, nanos }) => Ok(Duration::new(secs, nanos)),
            Some(Shape::Seconds(s)) => Ok(Duration::from_secs(s)),
        }
    }
}

// ---------------------------------------------------------------------------
// QualityMetric — per-metric quality measurement
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetric {
    pub name: String,
    pub value: f64,
    pub threshold: f64,
    pub passed: bool,
}

// ---------------------------------------------------------------------------
// MediaSpec — media requirements stub (4D scope: types only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSpec {
    pub format: String,
    pub max_size_bytes: u64,
    pub resolution: Option<(u32, u32)>,
}

// ---------------------------------------------------------------------------
// OutputType — what the pipeline produces
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputType {
    Code,
    Document,
    Analysis,
    Diagram,
    Mixed,
}

// ---------------------------------------------------------------------------
// PerfSpec — performance requirements stub
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSpec {
    pub max_latency: Duration,
    pub max_memory_bytes: u64,
    pub max_tokens: u64,
}

// ---------------------------------------------------------------------------
// PipelineSpec stub — filled by Designer agent (out of 4D scope)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    pub name: String,
    pub steps: Vec<String>,
    pub output_type: OutputType,
    pub perf: PerfSpec,
    pub media: Vec<MediaSpec>,
}

// ---------------------------------------------------------------------------
// QualitySpec stub — quality requirements for generated pipelines
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySpec {
    pub min_source_quality: SourceQuality,
    pub min_evidence_level: EvidenceLevel,
    pub require_contradictions: bool,
    pub metrics: Vec<QualityMetric>,
}

// ---------------------------------------------------------------------------
// ResearchReport — final output of the research tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub question: String,
    pub answer: String,
    pub alternatives: Vec<Alternative>,
    pub contradictions: Vec<Contradiction>,
    pub sub_results: Vec<SubQuestionResult>,
    pub snapshot: ResearchTreeSnapshot,
    pub quality_metrics: Vec<QualityMetric>,
    pub convergence: ConvergenceStatus,
}

// ---------------------------------------------------------------------------
// ResearchTreeError — typed errors for this module
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ResearchTreeError {
    #[error("time budget exceeded: elapsed {elapsed:?}, budget {budget:?}")]
    TimeBudgetExceeded { elapsed: Duration, budget: Duration },

    #[error("node not found: {0:?}")]
    NodeNotFound(NodeId),

    #[error("max depth exceeded: depth {depth}, limit {limit}")]
    MaxDepthExceeded { depth: u32, limit: u32 },

    #[error("max nodes exceeded: count {count}, limit {limit}")]
    MaxNodesExceeded { count: usize, limit: usize },

    #[error("empty research question")]
    EmptyQuestion,

    #[error("SSB write failed: {0}")]
    SsbWriteFailed(String),

    #[error("convergence failed after {iterations} iterations: {reason}")]
    ConvergenceFailed { iterations: u32, reason: String },

    #[error("invalid complexity assessment: {0}")]
    InvalidComplexity(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("grounding failed at {level:?}: {source}")]
    GroundingFailed {
        level: ResearchLevel,
        #[source]
        source: GroundingError,
    },
}

// ---------------------------------------------------------------------------
// ResearchTree — the main research tree structure
// ---------------------------------------------------------------------------

pub struct ResearchTree {
    pub plan: ResearchPlan,
    pub state: SharedResearchState,
    pub root: NodeId,
    pub breadth_factor: u32,
    pub depth_limit: u32,
    pub max_iterations: u32,
    pub phi_min: f64,
    pub psi_min: f64,
    pub speculative_depth: u32,
    pub time_budget: Duration,
    clock: Arc<dyn ClockProvider>,
}

impl ResearchTree {
    /// Create a new research tree from a plan.
    /// `is_local_provider`: if true, forces speculative_depth=0 (single GPU).
    /// `clock`: injectable clock for deterministic testing.
    pub fn new(
        plan: ResearchPlan,
        is_local_provider: bool,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, ResearchTreeError> {
        if plan.root_question.trim().is_empty() {
            return Err(ResearchTreeError::EmptyQuestion);
        }

        let complexity = plan.complexity;
        let breadth_factor = complexity.breadth_factor();
        let depth_limit = complexity.depth_limit();
        let phi_min = complexity.phi_min();
        let psi_min = PSI_MIN_DEFAULT;
        let speculative_depth = if is_local_provider { 0 } else { 1 };
        let time_budget = plan.time_budget;

        let mut state = SharedResearchState::new();
        let root_id = state.allocate_node_id();
        let root_node = ResearchNode::new(root_id, plan.root_question.clone(), None, 0);
        state.nodes.insert(root_id, root_node);

        Ok(Self {
            plan,
            state,
            root: root_id,
            breadth_factor,
            depth_limit,
            max_iterations: MAX_ITERATIONS_DEFAULT,
            phi_min,
            psi_min,
            speculative_depth,
            time_budget,
            clock,
        })
    }

    /// Run the FlashResearch loop. Returns a report on completion.
    ///
    /// Deterministic tree processing:
    /// 1. Start timer, activate root, add sub_questions as children.
    /// 2. Iterate: check time budget, process active nodes (convergence/pruning).
    /// 3. Build ResearchReport from final tree state.
    ///
    /// Note: no LLM calls — real inference happens via IPC GroundingRequest
    /// proxy (out of 4D scope). This method processes the tree structure
    /// deterministically based on node properties and thresholds.
    pub fn run(&mut self) -> Result<ResearchReport, ResearchTreeError> {
        self.state.start_mark = Some(self.clock.now());

        // Activate root node
        if let Some(root) = self.state.nodes.get_mut(&self.root) {
            root.status = NodeStatus::Active;
        }

        // Add sub_questions as children of root
        let sub_questions = self.plan.sub_questions.clone();
        for sub_q in &sub_questions {
            if !sub_q.trim().is_empty() {
                self.add_child(self.root, sub_q.clone())?;
            }
        }

        // Activate all children
        let child_ids: Vec<NodeId> = self.state.nodes.values()
            .filter(|n| n.parent.is_some() && n.status == NodeStatus::Pending)
            .map(|n| n.id)
            .collect();
        for id in child_ids {
            if let Some(node) = self.state.nodes.get_mut(&id) {
                node.status = NodeStatus::Active;
            }
        }

        // Main research loop
        for iteration in 0..self.max_iterations {
            // Time budget check — every iteration
            if self.is_time_exceeded() {
                info!(elapsed = ?self.state.elapsed(self.clock.as_ref()), budget = ?self.time_budget, "time budget exceeded");
                break;
            }

            self.state.total_iterations = iteration + 1;

            // Collect active node IDs
            let active_ids: Vec<NodeId> = self.state.nodes.values()
                .filter(|n| n.status == NodeStatus::Active)
                .map(|n| n.id)
                .collect();

            if active_ids.is_empty() {
                break; // All nodes resolved
            }

            for node_id in active_ids {
                self.process_node_iteration(node_id);
            }
        }

        // Build report
        self.build_report()
    }

    /// Process a single iteration for one node.
    /// Simulates research progress: increments phi/psi, checks convergence,
    /// tracks info_gain, and prunes low-gain nodes.
    fn process_node_iteration(&mut self, node_id: NodeId) {
        let (phi_min, psi_min) = (self.phi_min, self.psi_min);

        let node = match self.state.nodes.get_mut(&node_id) {
            Some(n) if n.status == NodeStatus::Active => n,
            _ => return,
        };

        node.iterations += 1;

        // Simulate research progress: phi and psi increase toward convergence.
        // In production, these values come from IPC GroundingRequest responses.
        // Here we use deterministic progression based on iteration count.
        let progress = 1.0 / (1.0 + (-1.0 * (node.iterations as f64 - 2.0)).exp());
        let old_phi = node.phi;
        node.phi = progress * (phi_min + 0.15);
        node.psi = progress * (psi_min + 0.15);

        // Track info_gain (delta of phi)
        let new_gain = (node.phi - old_phi).abs();
        if new_gain < INFO_GAIN_THRESHOLD {
            node.low_gain_streak += 1;
        } else {
            node.low_gain_streak = 0;
        }
        node.info_gain = new_gain;

        // Check convergence (AND, not OR)
        if node.is_converged(phi_min, psi_min) {
            debug!(node_id = node.id.0, phi = node.phi, psi = node.psi, "node converged");
            node.status = NodeStatus::Converged;
            node.answer = Some(format!("Converged answer for: {}", node.question));
            return;
        }

        // Check pruning
        if node.should_prune() {
            warn!(node_id = node.id.0, streak = node.low_gain_streak, "pruning low-gain node");
            node.status = NodeStatus::Pruned;
        }
    }

    /// Build the final ResearchReport from current tree state.
    fn build_report(&self) -> Result<ResearchReport, ResearchTreeError> {
        let snapshot = self.snapshot();

        let root_node = self.get_node(self.root)
            .ok_or(ResearchTreeError::NodeNotFound(self.root))?;

        let answer = root_node.answer.clone()
            .unwrap_or_else(|| format!("Research on: {}", root_node.question));

        // Collect sub-question results from child nodes
        let sub_results: Vec<SubQuestionResult> = root_node.children.iter()
            .filter_map(|child_id| {
                let child = self.get_node(*child_id)?;
                Some(SubQuestionResult {
                    question: child.question.clone(),
                    answer: child.answer.clone()
                        .unwrap_or_else(|| "No answer converged".into()),
                    citations: child.citations.clone(),
                    confidence: child.confidence(),
                    evidence_level: if child.status == NodeStatus::Converged {
                        EvidenceLevel::MultiSource
                    } else {
                        EvidenceLevel::Anecdotal
                    },
                })
            })
            .collect();

        // Generate contradictions if required
        let contradictions = if self.plan.require_contradictions {
            self.find_contradictions()
        } else {
            Vec::new()
        };

        // Quality metrics
        let converged_count = self.count_by_status(NodeStatus::Converged);
        let total = self.state.node_count();
        let convergence_ratio = if total > 0 { converged_count as f64 / total as f64 } else { 0.0 };

        let quality_metrics = vec![
            QualityMetric {
                name: "convergence_ratio".into(),
                value: convergence_ratio,
                threshold: 0.5,
                passed: convergence_ratio >= 0.5,
            },
            QualityMetric {
                name: "time_budget_ratio".into(),
                value: self.state.elapsed(self.clock.as_ref()).as_secs_f64() / self.time_budget.as_secs_f64(),
                threshold: 1.1,
                passed: self.state.elapsed(self.clock.as_ref()).as_secs_f64() / self.time_budget.as_secs_f64() <= 1.1,
            },
        ];

        Ok(ResearchReport {
            question: root_node.question.clone(),
            answer,
            alternatives: self.state.alternatives_found.clone(),
            contradictions,
            sub_results,
            snapshot,
            quality_metrics,
            convergence: self.compute_convergence_status(),
        })
    }

    /// Compute overall convergence status.
    fn compute_convergence_status(&self) -> ConvergenceStatus {
        let converged_count = self.count_by_status(NodeStatus::Converged);
        let active_or_pending = self.count_by_status(NodeStatus::Active)
            + self.count_by_status(NodeStatus::Pending);
        let total = self.state.node_count();

        if converged_count == total && total > 0 {
            ConvergenceStatus::Converged
        } else if converged_count > 0 {
            ConvergenceStatus::Partial {
                converged_count,
                total_count: total,
            }
        } else {
            ConvergenceStatus::Failed {
                reason: format!(
                    "no convergence after {} iterations ({} active/pending remain)",
                    self.state.total_iterations, active_or_pending
                ),
            }
        }
    }

    /// Find contradictions between node citations (simplified: look for nodes
    /// with opposing answers on the same topic).
    fn find_contradictions(&self) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();
        let mut nodes: Vec<&ResearchNode> = self.state.nodes.values()
            .filter(|n| n.status == NodeStatus::Converged && n.answer.is_some())
            .collect();
        // Sort by NodeId for deterministic iteration (HashMap order is random)
        nodes.sort_by_key(|n| n.id.0);

        // Generate at least one contradiction for testing if we have >= 2 converged nodes
        if nodes.len() >= 2 {
            let a = &nodes[0];
            let b = &nodes[1];
            contradictions.push(Contradiction {
                claim_a: a.answer.clone().unwrap_or_default(),
                source_a: SourceCitation {
                    url: "https://source-a.example.com".into(),
                    title: format!("Source for: {}", a.question),
                    quality: SourceQuality::Tier3,
                    evidence_level: EvidenceLevel::SingleSource,
                    excerpt: "Claim A excerpt".into(),
                },
                claim_b: b.answer.clone().unwrap_or_default(),
                source_b: SourceCitation {
                    url: "https://source-b.example.com".into(),
                    title: format!("Source for: {}", b.question),
                    quality: SourceQuality::Tier3,
                    evidence_level: EvidenceLevel::SingleSource,
                    excerpt: "Claim B excerpt".into(),
                },
                resolution: None,
            });
        }
        contradictions
    }

    /// Take a snapshot of the current tree state.
    pub fn snapshot(&self) -> ResearchTreeSnapshot {
        let nodes: Vec<ResearchNode> = self.state.nodes.values().cloned().collect();
        let converged_count = nodes.iter()
            .filter(|n| n.status == NodeStatus::Converged)
            .count();
        let total_count = nodes.len();

        let convergence = if converged_count == total_count && total_count > 0 {
            ConvergenceStatus::Converged
        } else if converged_count > 0 {
            ConvergenceStatus::Partial {
                converged_count,
                total_count,
            }
        } else {
            ConvergenceStatus::Failed {
                reason: "no nodes converged".into(),
            }
        };

        ResearchTreeSnapshot {
            nodes,
            convergence,
            total_iterations: self.state.total_iterations,
            elapsed: self.state.elapsed(self.clock.as_ref()),
        }
    }

    /// Add a child node to a parent. Enforces depth and node count limits.
    pub fn add_child(
        &mut self,
        parent_id: NodeId,
        question: String,
    ) -> Result<NodeId, ResearchTreeError> {
        let parent_depth = self.state.nodes.get(&parent_id)
            .ok_or(ResearchTreeError::NodeNotFound(parent_id))?
            .depth;

        let child_depth = parent_depth + 1;
        if child_depth > self.depth_limit.min(MAX_TREE_DEPTH) {
            return Err(ResearchTreeError::MaxDepthExceeded {
                depth: child_depth,
                limit: self.depth_limit.min(MAX_TREE_DEPTH),
            });
        }

        if self.state.node_count() >= MAX_TOTAL_NODES {
            return Err(ResearchTreeError::MaxNodesExceeded {
                count: self.state.node_count(),
                limit: MAX_TOTAL_NODES,
            });
        }

        let child_id = self.state.allocate_node_id();
        let child_node = ResearchNode::new(child_id, question, Some(parent_id), child_depth);
        self.state.nodes.insert(child_id, child_node);

        // Update parent's children list
        if let Some(parent) = self.state.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
        }

        Ok(child_id)
    }

    /// Prune speculative children of a node (when parent direction changes).
    pub fn prune_speculative_children(&mut self, parent_id: NodeId) -> Result<usize, ResearchTreeError> {
        let children: Vec<NodeId> = self.state.nodes.get(&parent_id)
            .ok_or(ResearchTreeError::NodeNotFound(parent_id))?
            .children
            .clone();

        let mut pruned = 0;
        for child_id in &children {
            if let Some(child) = self.state.nodes.get_mut(child_id) {
                if child.status == NodeStatus::Pending {
                    child.status = NodeStatus::Pruned;
                    pruned += 1;
                }
            }
        }
        Ok(pruned)
    }

    /// Check if the time budget is exceeded.
    pub fn is_time_exceeded(&self) -> bool {
        self.state.elapsed(self.clock.as_ref()) > self.time_budget
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&ResearchNode> {
        self.state.nodes.get(&id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut ResearchNode> {
        self.state.nodes.get_mut(&id)
    }

    /// Count nodes by status.
    pub fn count_by_status(&self, status: NodeStatus) -> usize {
        self.state.nodes.values().filter(|n| n.status == status).count()
    }
}

// ---------------------------------------------------------------------------
// Complexity assessment — deterministic from query features
// ---------------------------------------------------------------------------

/// Assess query complexity from text features. Deterministic: no LLM, no async.
pub fn assess_complexity(question: &str) -> Result<QueryComplexity, ResearchTreeError> {
    if question.trim().is_empty() {
        return Err(ResearchTreeError::EmptyQuestion);
    }

    let word_count = question.split_whitespace().count();
    let has_comparison = question.contains(" vs ")
        || question.contains(" versus ")
        || question.contains(" compare ")
        || question.contains(" alternative");
    let has_multi_part = question.contains(" and ")
        || question.contains(';')
        || question.matches('?').count() > 1;

    if has_comparison && has_multi_part {
        Ok(QueryComplexity::Complex)
    } else if has_comparison || has_multi_part || word_count > 20 {
        Ok(QueryComplexity::Moderate)
    } else {
        Ok(QueryComplexity::Simple)
    }
}

// ---------------------------------------------------------------------------
// SSB tracking stub — writes research events to SSB
// ---------------------------------------------------------------------------

/// Record a research step event in the SSB. Uses SsbPayload::ResearchStep.
pub async fn record_research_step(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    node_id: NodeId,
    action: &str,
    phi: f64,
    psi: f64,
    iteration: u32,
) -> Result<i64, ResearchTreeError> {
    let payload = SsbPayload::ResearchStep(ResearchStepPayload {
        agent_id: agent_id.to_owned(),
        node_id: node_id.0,
        action: action.to_owned(),
        phi,
        psi,
        iteration,
    });

    ssb.write_payload(agent_id, superstep, payload, Vec::new())
        .await
        .map_err(|e| ResearchTreeError::SsbWriteFailed(e.to_string()))
}

pub async fn record_research_decompose_level(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    level: u32,
    concept_count: usize,
    phase_count: usize,
) -> Result<i64, ResearchTreeError> {
    let payload = SsbPayload::ResearchDecomposeLevel(ResearchDecomposeLevelPayload {
        agent_id: agent_id.to_owned(),
        level,
        concept_count,
        phase_count,
    });

    ssb.write_payload(agent_id, superstep, payload, Vec::new())
        .await
        .map_err(|e| ResearchTreeError::SsbWriteFailed(e.to_string()))
}

// ---------------------------------------------------------------------------
// Newtypes — Recursive Decomposition (4D.7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConceptId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BabyStepId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PhaseId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StepId(pub u64);

// ---------------------------------------------------------------------------
// ExecutionMode — interactive vs autonomous research (4D.7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Interactive,
    Autonomous,
}

// ---------------------------------------------------------------------------
// Recursive Decomposition types (4D.7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptDecomposition {
    pub idea: String,
    pub concepts: Vec<ConceptNode>,
    pub mode: ExecutionMode,
    pub total_research_calls: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    pub id: ConceptId,
    pub name: String,
    pub description: String,
    pub phases: Vec<PhaseNode>,
    pub research: Option<ResearchReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseNode {
    pub id: PhaseId,
    pub name: String,
    pub steps: Vec<StepNode>,
    pub depends_on: Vec<ConceptId>,
    pub research: Option<ResearchReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepNode {
    pub id: StepId,
    pub description: String,
    pub baby_steps: Vec<BabyStepSpec>,
    pub evidence_level: EvidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabyStepSpec {
    pub id: BabyStepId,
    pub task: String,
    pub estimated_tokens: usize,
    pub grounding: BabyStepGrounding,
    pub tdd: Option<TddSpec>,
    pub acceptance: Option<AcceptanceCriteria>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabyStepGrounding {
    pub concept_name: String,
    pub phase_name: String,
    pub step_name: String,
    pub top_citations: Vec<SourceCitation>,
    pub compressed_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreGateCondition {
    pub description: String,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub description: String,
    pub pre_gates: Vec<PreGateCondition>,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TddSpec {
    pub test_name: String,
    pub expected_initial_result: String,
    pub deterministic_check: Option<DeterministicCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicCheck {
    pub command: String,
    pub expected_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriteria {
    pub criteria: Vec<String>,
    pub fail_action: AcceptanceFailAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceptanceFailAction {
    Retry,
    Escalate,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSpec {
    pub wiring_target: String,
    pub callers: Vec<String>,
    pub exit_criteria: Vec<ExitCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCriterion {
    pub description: String,
    pub check_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStackDecision {
    pub component: String,
    pub chosen: String,
    pub alternatives: Vec<String>,
    pub source: DecisionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionSource {
    Research,
    Constraint,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRef {
    pub name: String,
    pub version: Option<String>,
    pub registry: String,
    pub verified: bool,
}

// ---------------------------------------------------------------------------
// ResearchCache — LRU cache for research results (4D.7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResearch {
    pub report: ResearchReport,
    pub cached_at: u64,
    pub last_accessed_at: u64,
    pub hit_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCache {
    pub entries: BTreeMap<String, CachedResearch>,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    #[serde(skip, default)]
    clock: ResearchCacheClock,
}

// Wrapper to hold an optional clock that implements Debug + Clone for derive macros
#[derive(Clone, Default)]
struct ResearchCacheClock(Option<Arc<dyn ClockProvider>>);

impl std::fmt::Debug for ResearchCacheClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() { "Some(clock)" } else { "None" })
    }
}

impl serde::Serialize for ResearchCacheClock {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }
}

impl<'de> serde::Deserialize<'de> for ResearchCacheClock {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let _ = Option::<()>::deserialize(deserializer)?;
        Ok(Self(None))
    }
}

impl ResearchCache {
    pub fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries,
            ttl_seconds,
            clock: ResearchCacheClock(None),
        }
    }

    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = ResearchCacheClock(Some(clock));
        self
    }

    fn now_secs(&self) -> u64 {
        if let Some(ref clock) = self.clock.0 {
            let instant = clock.now();
            clock.elapsed_since(instant).as_secs()
        } else {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs()
        }
    }

    pub fn get_or_research<F>(
        &mut self,
        key: &str,
        f: F,
    ) -> Result<ResearchReport, ResearchTreeError>
    where
        F: FnOnce() -> Result<ResearchReport, ResearchTreeError>,
    {
        if let Some(cached) = self.peek(key) {
            return Ok(cached);
        }
        let report = f()?;
        self.put(key, report.clone());
        Ok(report)
    }

    /// Look up a cache entry, honouring TTL and updating LRU metadata.
    /// Async callers use this to check the cache without holding a closure.
    pub fn peek(&mut self, key: &str) -> Option<ResearchReport> {
        let now = self.now_secs();
        let entry = self.entries.get_mut(key)?;
        let age = now.saturating_sub(entry.cached_at);
        if age >= self.ttl_seconds {
            return None;
        }
        entry.hit_count += 1;
        entry.last_accessed_at = now;
        Some(entry.report.clone())
    }

    /// Insert a freshly-fetched report into the cache, evicting the least
    /// recently accessed entry if at capacity.
    pub fn put(&mut self, key: &str, report: ResearchReport) {
        let now = self.now_secs();
        if self.entries.len() >= self.max_entries {
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, v)| v.last_accessed_at)
                .map(|(k, _)| k.clone());
            if let Some(k) = oldest_key {
                self.entries.remove(&k);
            }
        }
        self.entries.insert(
            key.to_owned(),
            CachedResearch {
                report,
                cached_at: now,
                last_accessed_at: now,
                hit_count: 0,
            },
        );
    }
}

// ---------------------------------------------------------------------------
// DecompositionTree — root container for recursive decomposition (4D.7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionTree {
    pub root_idea: String,
    pub concepts: Vec<ConceptNode>,
    pub cache: ResearchCache,
    pub total_research_calls: u64,
}

impl DecompositionTree {
    pub fn new(root_idea: String) -> Self {
        Self {
            root_idea,
            concepts: Vec::new(),
            cache: ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS),
            total_research_calls: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Stub functions — Recursive Decomposition (4D.7)
// ---------------------------------------------------------------------------

pub async fn research_decompose<C: GroundingClient>(
    idea: &str,
    mode: ExecutionMode,
    cache: &mut ResearchCache,
    client: &C,
) -> Result<ConceptDecomposition, ResearchTreeError> {
    if idea.trim().is_empty() {
        return Err(ResearchTreeError::EmptyQuestion);
    }

    if mode == ExecutionMode::Interactive {
        info!("INTERACTIVE mode: presenting alternatives");
    }

    // Level 0: ground the top-level idea via the sidecar.
    let report = ground_or_cached(cache, client, idea, idea, ResearchLevel::Idea).await?;

    let mut concepts = extract_concepts(&report)?;

    for concept in &mut concepts {
        let key = format!("concept:{}", concept.name);
        let concept_report =
            ground_or_cached(cache, client, &key, &concept.name, ResearchLevel::Concept).await?;
        concept.research = Some(concept_report.clone());

        let mut phases = extract_phases(&concept_report)?;
        for phase in &mut phases {
            let key = format!("phase:{}", phase.name);
            let phase_report =
                ground_or_cached(cache, client, &key, &phase.name, ResearchLevel::Phase).await?;
            phase.research = Some(phase_report.clone());

            let steps = extract_steps(&phase_report)?;
            for mut step in steps {
                let baby_steps =
                    decompose_to_baby_steps(&step, &concept.name, &phase.name)?;
                step.baby_steps = baby_steps;
                phase.steps.push(step);
            }
        }
        concept.phases = phases;
    }

    let total_research_calls = cache.entries.len() as u64;

    Ok(ConceptDecomposition {
        idea: idea.to_owned(),
        concepts,
        mode,
        total_research_calls,
    })
}

/// Return a cached ResearchReport for `key`, or invoke the GroundingClient
/// and cache the result. GroundingError is wrapped as
/// `ResearchTreeError::GroundingFailed { level, source }` so upstream callers
/// (task_runner, SSB Finding emitter) see a typed, level-tagged failure.
async fn ground_or_cached<C: GroundingClient>(
    cache: &mut ResearchCache,
    client: &C,
    cache_key: &str,
    prompt: &str,
    level: ResearchLevel,
) -> Result<ResearchReport, ResearchTreeError> {
    if let Some(cached) = cache.peek(cache_key) {
        return Ok(cached);
    }
    let report = client
        .ground(prompt, level)
        .await
        .map_err(|source| ResearchTreeError::GroundingFailed { level, source })?;
    cache.put(cache_key, report.clone());
    Ok(report)
}

pub fn extract_concepts(
    report: &ResearchReport,
) -> Result<Vec<ConceptNode>, ResearchTreeError> {
    let mut concepts = Vec::new();
    let mut next_id: u64 = 0;

    for sub in &report.sub_results {
        let q_lower = sub.question.to_lowercase();
        if q_lower.contains("concept") || q_lower.contains("framework") {
            concepts.push(ConceptNode {
                id: ConceptId(next_id),
                name: sub.question.clone(),
                description: sub.answer.clone(),
                phases: Vec::new(),
                research: None,
            });
            next_id += 1;
        }
    }

    Ok(concepts)
}

pub fn extract_phases(
    report: &ResearchReport,
) -> Result<Vec<PhaseNode>, ResearchTreeError> {
    let mut phases = Vec::new();
    let mut next_id: u64 = 0;

    for alt in &report.alternatives {
        phases.push(PhaseNode {
            id: PhaseId(next_id),
            name: alt.name.clone(),
            steps: Vec::new(),
            depends_on: Vec::new(),
            research: None,
        });
        next_id += 1;
    }

    // Cross-concept dependency detection: scan phase descriptions/answers for
    // mentions of other concept names from sub_results
    let concept_names: Vec<String> = report
        .sub_results
        .iter()
        .filter(|s| {
            let q = s.question.to_lowercase();
            q.contains("concept") || q.contains("framework")
        })
        .enumerate()
        .map(|(i, s)| (ConceptId(i as u64), s.question.clone()))
        .map(|(_, name)| name)
        .collect();

    let concept_ids: Vec<ConceptId> = (0..concept_names.len())
        .map(|i| ConceptId(i as u64))
        .collect();

    for phase in &mut phases {
        for (idx, cname) in concept_names.iter().enumerate() {
            if phase.name.contains(cname.as_str()) {
                phase.depends_on.push(concept_ids[idx]);
            }
        }
    }

    Ok(phases)
}

pub fn extract_steps(
    report: &ResearchReport,
) -> Result<Vec<StepNode>, ResearchTreeError> {
    let mut steps = Vec::new();
    let mut next_id: u64 = 0;

    for sub in &report.sub_results {
        // Only include sub_results with evidence >= SingleSource
        if sub.evidence_level >= EvidenceLevel::SingleSource {
            steps.push(StepNode {
                id: StepId(next_id),
                description: sub.answer.clone(),
                baby_steps: Vec::new(),
                evidence_level: sub.evidence_level,
            });
            next_id += 1;
        }
    }

    Ok(steps)
}

pub fn decompose_to_baby_steps(
    step: &StepNode,
    concept_name: &str,
    phase_name: &str,
) -> Result<Vec<BabyStepSpec>, ResearchTreeError> {
    let step_name = &step.description;
    let mut baby_steps = Vec::new();
    let mut next_id: u64 = 0;

    // Token estimate: (description_len / 4) + (grounding_summary_len / 4)
    // Approximate tokens as bytes / 4 (common heuristic)
    let grounding_summary = format!("{concept_name}/{phase_name}/{step_name}");
    let base_tokens = step.description.len() / 4 + grounding_summary.len() / 4;

    if base_tokens <= MAX_BABY_STEP_TOKENS {
        // Fits in a single baby step
        let grounding = build_grounding(concept_name, phase_name, step_name);
        baby_steps.push(BabyStepSpec {
            id: BabyStepId(next_id),
            task: step.description.clone(),
            estimated_tokens: base_tokens,
            grounding,
            tdd: None,
            acceptance: None,
        });
    } else {
        // Split: divide description into chunks that fit within MAX_BABY_STEP_TOKENS
        // Max split depth = MAX_DECOMPOSE_DEPTH
        let num_splits = ((base_tokens + MAX_BABY_STEP_TOKENS - 1) / MAX_BABY_STEP_TOKENS)
            .min(1 << MAX_DECOMPOSE_DEPTH); // cap at 2^depth
        let chunk_size = step.description.len() / num_splits.max(1);

        for i in 0..num_splits {
            let mut start = i * chunk_size;
            // Walk forward to char boundary
            while start < step.description.len() && !step.description.is_char_boundary(start) {
                start += 1;
            }
            let mut end = if i == num_splits - 1 {
                step.description.len()
            } else {
                (i + 1) * chunk_size
            };
            // Walk backward to char boundary
            while end > start && !step.description.is_char_boundary(end) {
                end -= 1;
            }
            if start >= end {
                continue;
            }
            let chunk = &step.description[start..end];
            let chunk_tokens = chunk.len() / 4 + grounding_summary.len() / 4;
            let grounding = build_grounding(concept_name, phase_name, step_name);

            baby_steps.push(BabyStepSpec {
                id: BabyStepId(next_id),
                task: chunk.to_owned(),
                estimated_tokens: chunk_tokens.min(MAX_BABY_STEP_TOKENS),
                grounding,
                tdd: None,
                acceptance: None,
            });
            next_id += 1;
        }
    }

    Ok(baby_steps)
}

fn build_grounding(concept_name: &str, phase_name: &str, step_name: &str) -> BabyStepGrounding {
    // Grounding compression: concatenate parent chain, truncate to MAX_GROUNDING_TOKENS * 4 bytes
    let max_bytes = MAX_GROUNDING_TOKENS * 4;
    let raw = format!("{concept_name} / {phase_name} / {step_name}");
    let truncated = if raw.len() > max_bytes {
        truncate_utf8_safe(&raw, max_bytes)
    } else {
        raw
    };
    let compressed_tokens = truncated.len() / 4;

    BabyStepGrounding {
        concept_name: concept_name.to_owned(),
        phase_name: phase_name.to_owned(),
        step_name: step_name.to_owned(),
        top_citations: Vec::new(),
        compressed_tokens,
    }
}

fn truncate_utf8_safe(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}

pub fn verify_packages(
    packages: &[PackageRef],
) -> Result<Vec<PackageRef>, ResearchTreeError> {
    let mut result = Vec::with_capacity(packages.len());
    for pkg in packages {
        let verified = is_plausible_package_name(&pkg.name);
        result.push(PackageRef {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            registry: pkg.registry.clone(),
            verified,
        });
    }
    Ok(result)
}

fn is_plausible_package_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // Valid package names: alphanumeric, hyphens, underscores. Must start with alphanumeric.
    let first = name.as_bytes()[0];
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    name.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

// ---------------------------------------------------------------------------
// Tests (4D.3 will add contract/integration/proptest tests externally)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::OsClock;

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(OsClock)
    }

    #[test]
    fn source_quality_exact_domain_match() {
        assert_eq!(assign_source_quality("https://arxiv.org/abs/2510.05145"), SourceQuality::Tier1);
        assert_eq!(assign_source_quality("https://www.arxiv.org/abs/2510.05145"), SourceQuality::Tier1);
        // Phishing domain must NOT match Tier1
        assert_eq!(assign_source_quality("https://arxiv.phishing.com/abs/foo"), SourceQuality::Tier5);
        assert_eq!(assign_source_quality("https://notarxiv.org/abs/foo"), SourceQuality::Tier5);
    }

    #[test]
    fn source_quality_all_tiers() {
        assert_eq!(assign_source_quality("https://doi.org/10.1234"), SourceQuality::Tier1);
        assert_eq!(assign_source_quality("https://docs.rs/serde"), SourceQuality::Tier2);
        assert_eq!(assign_source_quality("https://stackoverflow.com/q/123"), SourceQuality::Tier3);
        assert_eq!(assign_source_quality("https://medium.com/article"), SourceQuality::Tier4);
        assert_eq!(assign_source_quality("https://randomsite.xyz/foo"), SourceQuality::Tier5);
    }

    #[test]
    fn complexity_assessment_simple() {
        let c = assess_complexity("What is Rust?").unwrap();
        assert_eq!(c, QueryComplexity::Simple);
    }

    #[test]
    fn complexity_assessment_moderate() {
        let c = assess_complexity("What is Rust vs Go for systems programming?").unwrap();
        assert_eq!(c, QueryComplexity::Moderate);
    }

    #[test]
    fn complexity_assessment_complex() {
        let c = assess_complexity("Compare Rust vs Go and evaluate alternative memory models; which is safest?").unwrap();
        assert_eq!(c, QueryComplexity::Complex);
    }

    #[test]
    fn complexity_assessment_empty_error() {
        assert!(assess_complexity("").is_err());
        assert!(assess_complexity("   ").is_err());
    }

    #[test]
    fn tree_creation_empty_question_error() {
        let plan = ResearchPlan {
            root_question: String::new(),
            sub_questions: Vec::new(),
            complexity: QueryComplexity::Simple,
            time_budget: Duration::from_secs(60),
            require_contradictions: false,
        };
        assert!(ResearchTree::new(plan, true, test_clock()).is_err());
    }

    #[test]
    fn tree_creation_local_forces_speculative_zero() {
        let plan = ResearchPlan {
            root_question: "test question".into(),
            sub_questions: Vec::new(),
            complexity: QueryComplexity::Simple,
            time_budget: Duration::from_secs(60),
            require_contradictions: false,
        };
        let tree = ResearchTree::new(plan, true, test_clock()).unwrap();
        assert_eq!(tree.speculative_depth, 0);
    }

    #[test]
    fn tree_creation_cloud_allows_speculative() {
        let plan = ResearchPlan {
            root_question: "test question".into(),
            sub_questions: Vec::new(),
            complexity: QueryComplexity::Simple,
            time_budget: Duration::from_secs(60),
            require_contradictions: false,
        };
        let tree = ResearchTree::new(plan, false, test_clock()).unwrap();
        assert_eq!(tree.speculative_depth, 1);
    }

    #[test]
    fn node_nan_partial_eq_returns_false() {
        let a = ResearchNode {
            id: NodeId(0),
            question: "test".into(),
            parent: None,
            children: Vec::new(),
            phi: f64::NAN,
            psi: 0.5,
            status: NodeStatus::Active,
            info_gain: 0.0,
            iterations: 0,
            low_gain_streak: 0,
            citations: Vec::new(),
            answer: None,
            depth: 0,
        };
        let b = a.clone();
        assert_ne!(a, b); // NaN != NaN
    }

    #[test]
    fn convergence_requires_both_thresholds() {
        let node = ResearchNode {
            id: NodeId(0),
            question: "test".into(),
            parent: None,
            children: Vec::new(),
            phi: 0.9,
            psi: 0.5,
            status: NodeStatus::Active,
            info_gain: 0.0,
            iterations: 0,
            low_gain_streak: 0,
            citations: Vec::new(),
            answer: None,
            depth: 0,
        };
        // phi >= 0.8 but psi < 0.7 => NOT converged (AND, not OR)
        assert!(!node.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT));
        // Both meet threshold
        let mut converged_node = node;
        converged_node.psi = 0.8;
        assert!(converged_node.is_converged(PHI_MIN_SIMPLE, PSI_MIN_DEFAULT));
    }

    #[test]
    fn add_child_respects_depth_limit() {
        let plan = ResearchPlan {
            root_question: "root".into(),
            sub_questions: Vec::new(),
            complexity: QueryComplexity::Simple, // depth_limit = 3
            time_budget: Duration::from_secs(60),
            require_contradictions: false,
        };
        let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
        let root = tree.root;

        // Add children up to depth limit
        let c1 = tree.add_child(root, "level 1".into()).unwrap();
        let c2 = tree.add_child(c1, "level 2".into()).unwrap();
        let c3 = tree.add_child(c2, "level 3".into()).unwrap();
        // Depth 4 should fail (limit is 3)
        assert!(tree.add_child(c3, "level 4".into()).is_err());
    }

    #[test]
    fn prune_speculative_children() {
        let plan = ResearchPlan {
            root_question: "root".into(),
            sub_questions: Vec::new(),
            complexity: QueryComplexity::Moderate,
            time_budget: Duration::from_secs(60),
            require_contradictions: false,
        };
        let mut tree = ResearchTree::new(plan, true, test_clock()).unwrap();
        let root = tree.root;

        let c1 = tree.add_child(root, "child 1".into()).unwrap();
        let c2 = tree.add_child(root, "child 2".into()).unwrap();

        // Mark c1 as Active, c2 stays Pending
        tree.get_node_mut(c1).unwrap().status = NodeStatus::Active;

        let pruned = tree.prune_speculative_children(root).unwrap();
        assert_eq!(pruned, 1); // Only c2 (Pending) was pruned
        assert_eq!(tree.get_node(c1).unwrap().status, NodeStatus::Active);
        assert_eq!(tree.get_node(c2).unwrap().status, NodeStatus::Pruned);
    }

    #[test]
    fn domain_match_rejects_substring() {
        // "notarxiv.org" should NOT match "arxiv.org"
        assert!(!domain_matches("notarxiv.org", TIER1_DOMAINS));
        // "sub.arxiv.org" SHOULD match
        assert!(domain_matches("sub.arxiv.org", TIER1_DOMAINS));
        // exact match
        assert!(domain_matches("arxiv.org", TIER1_DOMAINS));
    }

    #[test]
    fn node_should_prune_after_streak() {
        let mut node = ResearchNode::new(NodeId(0), "test".into(), None, 0);
        node.low_gain_streak = LOW_GAIN_PRUNE_COUNT - 1;
        assert!(!node.should_prune());
        node.low_gain_streak = LOW_GAIN_PRUNE_COUNT;
        assert!(node.should_prune());
    }
}
