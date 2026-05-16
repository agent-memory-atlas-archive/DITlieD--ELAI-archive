//! B-40 — Idea-Framer + Domain Research Presets + Critic Loop.
//!
//! Inserts a FRAMING substate before Phase 4D Recursive Decomposition that:
//!   1. Classifies the raw user prompt into one of 9 [`ProjectClass`] variants
//!      via a deterministic regex classifier (A-47 pattern; LLM fallback only
//!      on `Other` — at MVP we use regex-only to keep the classifier offline).
//!   2. Loads a `ResearchPreset` (named bundle of `ResearchStream`s) from
//!      `elai_cc.toml` and dispatches a multi-root FlashResearch tree
//!      (`ResearchTree::new_multi_root`).
//!   3. Spawns an [`AgentRole::IdeaFramer`] to compose a [`FramedIdea`] from
//!      the converged research streams.
//!   4. Runs a critic loop (max 3 rounds, RefineBench arXiv:2511.22173) where
//!      an [`AgentRole::IdeaCritic`] emits [`CriticFinding`]s across 10
//!      [`CriticCategory`] discriminants. BLOCK findings refuse to advance.
//!   5. Persists `framed-idea.toml`, `critic-rounds/round-N.toml`, and
//!      `tradeoff-responses.toml` under `workflow/sessions/{sid}/idea-framing/`.
//!
//! Spec refs:
//!   - docs/specs/guide/steps/phase-4D.txt L28-44 (Designer role config) —
//!     idea_framer mirrors Designer-shape GRC (Observe -> PlanAction -> Act
//!     -> Verify) with tighter budget 5/800.
//!   - docs/specs/guide/steps/phase-4D.txt L346-352 (L0 IDEA -> L1 CONCEPT) —
//!     FRAMING inserts BEFORE L0 so the IDEA passed to research_decompose is
//!     already framed + critic-converged.
//!   - arXiv:2510.05145 (FlashResearch) — multi-root tree dispatch via the
//!     existing `ResearchTree` (workflow/papers/flashresearch-interface.md).
//!   - arXiv:2511.22173 (RefineBench) — diminishing returns past round 2;
//!     MAX_CRITIC_ROUNDS=3 gives one safety margin.
//!   - arXiv:2409.04109 (Si et al. 2024 — substituted for the originally-cited
//!     arXiv:2407.21787 which was the wrong paper / "Large Language Monkeys").
//!     Rubric-based ablation: framing-present vs framing-absent (no specific
//!     effect-size gate). Documented in workflow/reasoning/B-40-decisions.md.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use elai_core::clock::ClockProvider;
// Note: AgentRole + NodeId + ConvergenceStatus are re-exported / referenced
// from sibling modules (lib.rs `pub use`) but the framer body itself does not
// directly construct any of them — multi-root dispatch uses the integer u64
// projection via NodeId.0, agent spawn is deferred to the production wire-up.
use crate::research_tree::{ResearchTree, ResearchTreeError};
use crate::ssb::{
    CriticRoundCompletePayload, IdeaFramedPayload, IdeaFramingStartedPayload,
    ProjectClassClassifiedPayload, Ssb, SsbActionType, SsbPayload, SsbWriteRequest,
    TradeoffCardEmittedPayload,
};

// ===========================================================================
// Constants
// ===========================================================================

/// Maximum critic rounds before BLOCK. RefineBench arXiv:2511.22173 reports
/// diminishing returns past round 2; 3 gives one safety margin without
/// runaway cost. See B-40-decisions.md DECISION-MAX-ROUNDS.
pub const MAX_CRITIC_ROUNDS: u32 = 3;

/// Exact count of [`CriticCategory`] variants. Enforced by
/// `tests/idea_framing_unit.rs::test_critic_category_exhaustive_match`.
/// Documented in B-40-decisions.md DECISION-10-CATEGORIES.
pub const NUM_CRITIC_CATEGORIES: usize = 10;

/// GRC budget call limit for the idea_framer agent. Tighter than Designer's
/// 8 because the framing scope is narrower (one FramedIdea, no PipelineSpec).
pub const IDEA_FRAMER_GRC_BUDGET_CALLS: u32 = 5;

/// GRC budget char limit for the idea_framer agent.
pub const IDEA_FRAMER_GRC_BUDGET_CHARS: u32 = 800;

/// stub_ban: the agent ID used when emitting SSB rows from the framer module
/// itself (orchestrator-driven, not agent-driven).
const FRAMER_AGENT_ID: &str = "idea_framer";

/// stub_ban: the agent ID used for critic-loop SSB rows.
const CRITIC_AGENT_ID: &str = "idea_critic";

// ===========================================================================
// ProjectClass — 9-variant project classification
// ===========================================================================

/// Project class detected from the raw user prompt. Drives which
/// [`ResearchPreset`] is dispatched. Source: B-40 plan L93-97.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectClass {
    ConsumerMobileApp,
    WebApp,
    SaaSDashboard,
    DesktopApp,
    GameMod,
    ResearchTool,
    LibraryOrSDK,
    CLITool,
    Other,
}

impl ProjectClass {
    /// snake_case discriminant string used in SSB payloads + TOML preset keys.
    pub fn as_snake_case(self) -> &'static str {
        match self {
            ProjectClass::ConsumerMobileApp => "consumer_mobile_app",
            ProjectClass::WebApp => "web_app",
            ProjectClass::SaaSDashboard => "saas_dashboard",
            ProjectClass::DesktopApp => "desktop_app",
            ProjectClass::GameMod => "game_mod",
            ProjectClass::ResearchTool => "research_tool",
            ProjectClass::LibraryOrSDK => "library_or_sdk",
            ProjectClass::CLITool => "cli_tool",
            ProjectClass::Other => "other",
        }
    }

    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "consumer_mobile_app" => Some(ProjectClass::ConsumerMobileApp),
            "web_app" => Some(ProjectClass::WebApp),
            "saas_dashboard" => Some(ProjectClass::SaaSDashboard),
            "desktop_app" => Some(ProjectClass::DesktopApp),
            "game_mod" => Some(ProjectClass::GameMod),
            "research_tool" => Some(ProjectClass::ResearchTool),
            "library_or_sdk" => Some(ProjectClass::LibraryOrSDK),
            "cli_tool" => Some(ProjectClass::CLITool),
            "other" => Some(ProjectClass::Other),
            _ => None,
        }
    }
}

/// All 9 [`ProjectClass`] discriminants for iteration in tests + preset
/// loader.
pub const ALL_PROJECT_CLASSES: &[ProjectClass] = &[
    ProjectClass::ConsumerMobileApp,
    ProjectClass::WebApp,
    ProjectClass::SaaSDashboard,
    ProjectClass::DesktopApp,
    ProjectClass::GameMod,
    ProjectClass::ResearchTool,
    ProjectClass::LibraryOrSDK,
    ProjectClass::CLITool,
    ProjectClass::Other,
];

// ===========================================================================
// CriticCategory — exactly 10 categories
// ===========================================================================

/// Critic finding category. Exactly 10 — empirical sweet-spot per the
/// originating chat reference workflow; fewer misses C-MARKET / C-IP failures,
/// more produces overlapping double-counted findings.
///
/// snake_case serde discriminants (used in SSB rows + TOML artifacts):
///   STALE / SLOP / POLICY / FEASIBILITY / MARKET / SCOPE / CONTRADICTION /
///   DESIGN_HARD / VAGUE / MISSING
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriticCategory {
    /// C-STALE — research evidence is older than the cutoff threshold.
    Stale,
    /// C-SLOP — concept lacks specificity; LLM hallucination markers.
    Slop,
    /// C-POLICY — proposed approach violates platform / store / IP policy.
    Policy,
    /// C-FEASIBILITY — required capability not available on target stack.
    Feasibility,
    /// C-MARKET — no addressable market or saturated competitor space.
    Market,
    /// C-SCOPE — scope creep relative to the user's stated intent.
    Scope,
    /// C-CONTRADICTION — internal contradiction between key decisions.
    Contradiction,
    /// C-DESIGN-HARD — design constraint hardcoded without grounding.
    DesignHard,
    /// C-VAGUE — language too vague to drive a downstream Concept decomposition.
    Vague,
    /// C-MISSING — required FramedIdea field is empty or missing.
    Missing,
}

impl CriticCategory {
    pub fn as_snake_case(self) -> &'static str {
        match self {
            CriticCategory::Stale => "stale",
            CriticCategory::Slop => "slop",
            CriticCategory::Policy => "policy",
            CriticCategory::Feasibility => "feasibility",
            CriticCategory::Market => "market",
            CriticCategory::Scope => "scope",
            CriticCategory::Contradiction => "contradiction",
            CriticCategory::DesignHard => "design_hard",
            CriticCategory::Vague => "vague",
            CriticCategory::Missing => "missing",
        }
    }
}

/// All 10 critic-category discriminants for exhaustive iteration. The
/// `tests/idea_framing_unit.rs::test_critic_category_exhaustive_match` test
/// asserts this slice has exactly [`NUM_CRITIC_CATEGORIES`] entries — adding
/// a new variant requires bumping both.
pub const ALL_CRITIC_CATEGORIES: &[CriticCategory] = &[
    CriticCategory::Stale,
    CriticCategory::Slop,
    CriticCategory::Policy,
    CriticCategory::Feasibility,
    CriticCategory::Market,
    CriticCategory::Scope,
    CriticCategory::Contradiction,
    CriticCategory::DesignHard,
    CriticCategory::Vague,
    CriticCategory::Missing,
];

// ===========================================================================
// Severity — 3-variant finding severity
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// BLOCK — refuses to advance FRAMING -> PLANNING.
    Block,
    /// WARN — surfaced to the user but does not block.
    Warn,
    /// INFO — advisory; recorded but does not surface.
    Info,
}

impl Severity {
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Severity::Block => "block",
            Severity::Warn => "warn",
            Severity::Info => "info",
        }
    }
}

// ===========================================================================
// Source tier (for research preset min_source_tier)
// ===========================================================================

/// Source tier requirement for a research stream. Mirrors the public
/// [`crate::research_tree::SourceQuality`] tiers but uses string discriminants
/// for TOML preset declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SourceTier {
    T1,
    T2,
    T3,
    T4,
}

impl SourceTier {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceTier::T1 => "T1",
            SourceTier::T2 => "T2",
            SourceTier::T3 => "T3",
            SourceTier::T4 => "T4",
        }
    }
}

// ===========================================================================
// ResearchStream — one named research bundle in a preset
// ===========================================================================

/// One named research bundle in a [`ResearchPreset`]. Mapped 1:1 to a root
/// node in `ResearchTree::new_multi_root`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchStream {
    /// Stream name (e.g. "competitor_ux_teardown"). Used as the SSB
    /// row tag + dashboard label.
    pub name: String,
    /// Tera template with `{idea}` / `{class}` slots. Rendered at dispatch.
    pub query_template: String,
    /// Minimum source tier for a citation to count toward convergence.
    pub min_source_tier: SourceTier,
    /// Maximum sibling sub-questions for this stream (caps fan-out).
    pub parallelism_budget: u32,
    /// B-42 forward-dependency: cosmos artifact name to inject as prefix.
    /// At MVP this is a no-op (resolver returns `None`). Documented in
    /// B-40-decisions.md DECISION-COSMOS-FORWARD-DEP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cosmos_grounding: Option<String>,
    /// When true, this stream MUST converge OR the framer emits
    /// `degraded_streams`. When false, the stream is best-effort — skipped
    /// if the parallelism budget is exhausted.
    pub mandatory: bool,
}

// ===========================================================================
// ResearchPresets — registry loaded from elai_cc.toml
// ===========================================================================

/// Wrapper for a single ProjectClass's research-preset block. Required so the
/// TOML `[[research_presets.<class>.streams]]` array-of-tables shape
/// deserializes into `{ streams: Vec<ResearchStream> }` under each class key.
/// Without this wrapper serde sees a map under the class key and refuses to
/// coerce it to a bare `Vec<ResearchStream>` (B-40 BLOCK-1 audit fix).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassPreset {
    #[serde(default)]
    pub streams: Vec<ResearchStream>,
}

/// Top-level [research_presets.*] block from elai_cc.toml. Keyed by
/// `ProjectClass::as_snake_case`. Each value is a [`ClassPreset`] whose
/// `streams` field carries the ordered stream list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchPresets {
    /// snake_case ProjectClass name -> { streams: [...] }.
    /// e.g. `"consumer_mobile_app"` -> `ClassPreset { streams: vec![...] }`.
    #[serde(default, flatten)]
    pub presets: std::collections::BTreeMap<String, ClassPreset>,
}

impl ResearchPresets {
    /// Construct an empty preset registry (test default).
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up the preset stream list for a given [`ProjectClass`].
    pub fn get(&self, class: ProjectClass) -> Option<&Vec<ResearchStream>> {
        self.presets.get(class.as_snake_case()).map(|cp| &cp.streams)
    }
}

/// Load research preset streams for a class. Returns an empty Vec if no
/// preset is registered (caller falls back to single-root dispatch).
pub fn load_research_preset(
    class: ProjectClass,
    presets: &ResearchPresets,
) -> Vec<ResearchStream> {
    presets
        .get(class)
        .cloned()
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Production config loader (B-40 BLOCK-1 audit fix)
// ---------------------------------------------------------------------------

/// Top-level TOML wrapper used by [`parse_research_presets_from_toml`].
///
/// We deserialize only the `[research_presets]` table from `elai_cc.toml`
/// and ignore the rest. `#[serde(default)]` makes the section optional so
/// callers that read a config without the block fall through to the empty
/// registry (and FRAMING falls back to single-root dispatch).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ElaiTomlResearchPresetsSlice {
    research_presets: ResearchPresets,
}

/// Parse `[research_presets]` out of an `elai_cc.toml` source string.
///
/// Returns the empty registry when the section is absent (graceful default).
/// Surfaces the underlying [`toml::de::Error`] when the section is present
/// but malformed — production callers must decide whether to log-and-default
/// or bubble the error up.
pub fn parse_research_presets_from_toml(raw: &str) -> Result<ResearchPresets, toml::de::Error> {
    let slice: ElaiTomlResearchPresetsSlice = toml::from_str(raw)?;
    Ok(slice.research_presets)
}

/// Load `[research_presets]` from `<project_root>/elai_cc.toml`.
///
/// Mirrors the CLI's `load_elai_config` semantics: missing file or parse
/// failure yields `ResearchPresets::default()` (with a tracing warning on
/// parse failure). Used by orchestrator wiring that needs presets without
/// taking a circular dependency on the CLI crate.
pub fn load_research_presets_from_project_root(project_root: &std::path::Path) -> ResearchPresets {
    let config_path = project_root.join("elai_cc.toml");
    match std::fs::read_to_string(&config_path) {
        Ok(raw) => match parse_research_presets_from_toml(&raw) {
            Ok(presets) => presets,
            Err(e) => {
                tracing::warn!(
                    path = %config_path.display(),
                    error = %e,
                    "failed to parse [research_presets] in elai_cc.toml — using empty registry"
                );
                ResearchPresets::default()
            }
        },
        Err(_) => {
            tracing::debug!(
                path = %config_path.display(),
                "elai_cc.toml not found — FRAMING falls back to single-root dispatch"
            );
            ResearchPresets::default()
        }
    }
}

// ===========================================================================
// FramedIdea + supporting types
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamSummary {
    pub stream_name: String,
    /// Root NodeId for this stream in the multi-root ResearchTree.
    pub stream_root: u64,
    pub convergence: String,
    /// True when the stream converged (or `degraded_streams` records the gap).
    pub converged: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchSummary {
    pub streams: Vec<StreamSummary>,
    /// Names of mandatory streams that failed to converge — surfaced to the
    /// user as a transparency gap (mandatory streams cannot be silently
    /// skipped; if missing the framer documents the absence).
    pub degraded_streams: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyDecision {
    pub name: String,
    pub rationale: String,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tradeoff {
    pub name: String,
    pub options: Vec<String>,
    pub default: String,
    pub user_facing_question: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TradeoffCard {
    pub tradeoffs: Vec<Tradeoff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticFinding {
    pub category: CriticCategory,
    pub severity: Severity,
    /// NodeId in the multi-root ResearchTree the finding is evidenced by.
    /// None when the finding is purely structural (e.g. missing field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<u64>,
    pub proposed_fix: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriticRound {
    pub round_index: u32,
    pub findings: Vec<CriticFinding>,
}

impl CriticRound {
    pub fn block_count(&self) -> u32 {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Block)
            .count() as u32
    }

    pub fn warn_count(&self) -> u32 {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warn)
            .count() as u32
    }

    pub fn info_count(&self) -> u32 {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count() as u32
    }

    /// Per-category finding count for SSB emission. Always emits an entry for
    /// each of the 10 [`CriticCategory`] discriminants (zero count when no
    /// findings in that bucket).
    pub fn category_counts(&self) -> Vec<(String, u32)> {
        ALL_CRITIC_CATEGORIES
            .iter()
            .map(|cat| {
                let count = self
                    .findings
                    .iter()
                    .filter(|f| f.category == *cat)
                    .count() as u32;
                (cat.as_snake_case().to_owned(), count)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FramedIdea {
    pub project_class: ProjectClass,
    pub raw_prompt: String,
    pub research_summary: ResearchSummary,
    pub key_decisions: Vec<KeyDecision>,
    pub known_unknowns: Vec<String>,
    pub tradeoff_card: TradeoffCard,
    /// Critic rounds run so far (1..=MAX_CRITIC_ROUNDS).
    #[serde(default)]
    pub critic_rounds: Vec<CriticRound>,
}

impl FramedIdea {
    /// True when no BLOCK findings remain in the latest critic round.
    pub fn is_converged(&self) -> bool {
        match self.critic_rounds.last() {
            Some(round) => round.block_count() == 0,
            None => false,
        }
    }
}

// ===========================================================================
// Errors
// ===========================================================================

#[derive(Debug, Error)]
pub enum IdeaFramingError {
    #[error("empty user prompt")]
    EmptyPrompt,

    #[error("research tree error: {0}")]
    ResearchTree(#[from] ResearchTreeError),

    #[error("io error writing artifact: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml serialization failed: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("ssb error: {0}")]
    Ssb(#[from] crate::ssb::SsbError),

    #[error("msgpack encode error: {0}")]
    MsgpackEncode(#[from] rmp_serde::encode::Error),

    #[error("max critic rounds reached with {block_count} unresolved BLOCK finding(s)")]
    MaxRoundsReachedWithBlocks { block_count: u32 },

    #[error("project class unclassified")]
    ProjectClassUnclassified,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

// ===========================================================================
// Project class detection
// ===========================================================================

/// Deterministic regex-based classifier. A-47 pattern: keyword + intent
/// regex first, falls back to `Other`. LLM fallback is deferred to a future
/// patch — at MVP we keep the classifier offline so latency stays < 10ms
/// for the 90% common case. Documented in B-40-decisions.md
/// DECISION-REGEX-CLASSIFIER.
///
/// Returns the matched class plus the (lowercased) matched pattern string
/// for SSB transparency. `(ProjectClass::Other, None)` when no rule fires.
pub fn classify_project_class_with_pattern(
    prompt: &str,
) -> (ProjectClass, Option<String>) {
    let lower = prompt.trim().to_lowercase();
    if lower.is_empty() {
        return (ProjectClass::Other, None);
    }

    // Order matters: more specific patterns first to avoid early shadowing.
    // Each rule is a (pattern, ProjectClass) pair — the FIRST hit wins.
    let rules: &[(&[&str], ProjectClass)] = &[
        // GameMod — explicit mod / game / mod-tool keywords.
        (
            &["game mod", "mod for", "minecraft mod", "skyrim mod",
              "factorio mod", "rimworld mod", "modding", "mod-loader",
              "mod loader"],
            ProjectClass::GameMod,
        ),
        // ResearchTool — explicit research / academic intent.
        (
            &["research tool", "paper analysis", "literature review",
              "academic tool", "arxiv", "dataset analysis",
              "benchmark suite", "experiment runner"],
            ProjectClass::ResearchTool,
        ),
        // SaaSDashboard — analytics/dashboards/cohort/SaaS markers.
        (
            &["saas", "dashboard", "analytics", "cohort", "metrics dashboard",
              "admin panel", "b2b platform", "subscription platform",
              "mailchimp competitor", "looker", "tableau"],
            ProjectClass::SaaSDashboard,
        ),
        // ConsumerMobileApp — mobile / ios / android / app store markers.
        (
            &["mobile app", "ios app", "android app", "iphone app",
              "react native", "flutter app", "tiktok competitor",
              "instagram competitor", "duolingo competitor",
              "picturethis", "app like", "consumer app"],
            ProjectClass::ConsumerMobileApp,
        ),
        // WebApp — web / browser / nextjs / svelte / vite markers.
        (
            &["web app", "webapp", "website", "next.js app", "nextjs app",
              "react app", "svelte app", "vue app", "browser game",
              "html5 game", "static site", "landing page"],
            ProjectClass::WebApp,
        ),
        // DesktopApp — electron / tauri / native desktop.
        (
            &["desktop app", "electron app", "tauri app", "native gui",
              "qt app", "wpf app", "winforms"],
            ProjectClass::DesktopApp,
        ),
        // LibraryOrSDK — library / crate / sdk / package keywords.
        (
            &["library", "sdk", "rust crate", "npm package",
              "python package", "open-source library",
              "api client", "wrapper for"],
            ProjectClass::LibraryOrSDK,
        ),
        // CLITool — cli / command-line / shell-utility markers.
        (
            &["cli tool", "command-line tool", "command line tool",
              "shell utility", "bash script", "rust cli", "go cli",
              "terminal tool"],
            ProjectClass::CLITool,
        ),
    ];

    for (patterns, class) in rules {
        for p in *patterns {
            if lower.contains(p) {
                return (*class, Some((*p).to_owned()));
            }
        }
    }

    (ProjectClass::Other, None)
}

/// Convenience wrapper returning only the class (drops the matched pattern).
pub fn classify_project_class(prompt: &str) -> ProjectClass {
    classify_project_class_with_pattern(prompt).0
}

// ===========================================================================
// Multi-root research dispatch
// ===========================================================================

/// Dispatch the multi-root research tree for a preset. Returns a
/// [`ResearchSummary`] with per-stream convergence + `degraded_streams` when
/// mandatory streams failed to converge.
///
/// `clock` is the injected ClockProvider per the entropy quarantine rule.
/// At MVP this performs a deterministic-only run (no live LLM calls — the
/// underlying `ResearchTree::run` simulates progress per node iteration).
/// Real grounding lands when this fn is wired into the live IPC GroundingRequest
/// path.
pub fn dispatch_multi_root_research(
    streams: Vec<ResearchStream>,
    clock: Arc<dyn ClockProvider>,
) -> Result<ResearchSummary, IdeaFramingError> {
    if streams.is_empty() {
        return Ok(ResearchSummary {
            streams: Vec::new(),
            degraded_streams: Vec::new(),
        });
    }

    let mut tree = ResearchTree::new_multi_root(streams.clone(), clock)?;
    let _report = tree.run()?;

    let snapshot = tree.snapshot();
    let mut summaries: Vec<StreamSummary> = Vec::with_capacity(streams.len());
    let mut degraded: Vec<String> = Vec::new();

    for (idx, stream) in streams.iter().enumerate() {
        let root_id = tree
            .stream_root_id(idx)
            .map(|n| n.0)
            .unwrap_or(0);
        // A stream is considered "converged" when its root node has
        // status Converged in the snapshot.
        let converged = snapshot
            .nodes
            .iter()
            .find(|n| n.id.0 == root_id)
            .map(|n| {
                matches!(
                    n.status,
                    crate::research_tree::NodeStatus::Converged
                )
            })
            .unwrap_or(false);
        let convergence_str = if converged {
            "converged".to_owned()
        } else {
            "degraded".to_owned()
        };
        if stream.mandatory && !converged {
            degraded.push(stream.name.clone());
        }
        summaries.push(StreamSummary {
            stream_name: stream.name.clone(),
            stream_root: root_id,
            convergence: convergence_str,
            converged,
        });
    }

    Ok(ResearchSummary {
        streams: summaries,
        degraded_streams: degraded,
    })
}

// ===========================================================================
// Critic loop
// ===========================================================================

/// Run up to [`MAX_CRITIC_ROUNDS`] critic rounds over a [`FramedIdea`].
/// Returns the number of rounds actually executed (1..=MAX_CRITIC_ROUNDS).
///
/// Per-round semantics:
///   1. Run the (deterministic at MVP) critic over the current FramedIdea.
///   2. Collect findings; if zero BLOCK -> converge, exit.
///   3. If round < MAX_CRITIC_ROUNDS -> mutate FramedIdea via
///      `apply_findings_as_rescope` (records known_unknowns from BLOCK
///      categories so the next round has different inputs).
///   4. If round == MAX_CRITIC_ROUNDS AND BLOCKs remain -> return
///      [`IdeaFramingError::MaxRoundsReachedWithBlocks`]. Caller emits an
///      `idea_framing_block` SSB row and refuses to advance.
///
/// SSB: emits one [`SsbActionType::CriticRoundComplete`] row per round.
pub async fn run_critic_loop(
    framed: &mut FramedIdea,
    max_rounds: u32,
    ssb: Option<&Arc<Ssb>>,
    session_id: &str,
) -> Result<u32, IdeaFramingError> {
    let cap = max_rounds.min(MAX_CRITIC_ROUNDS).max(1);

    let mut rounds_used = 0u32;
    for round_idx in 1..=cap {
        rounds_used = round_idx;

        let findings = run_one_critic_round_deterministic(framed);
        let round = CriticRound {
            round_index: round_idx,
            findings,
        };

        let block_count = round.block_count();
        let warn_count = round.warn_count();
        let info_count = round.info_count();
        let converged = block_count == 0;
        let category_counts = round.category_counts();

        // Emit SSB row before mutating framed (so the row reflects the
        // round we just produced, not the next-round state).
        if let Some(writer) = ssb {
            let payload = SsbPayload::CriticRoundComplete(CriticRoundCompletePayload {
                session_id: session_id.to_owned(),
                round_index: round_idx,
                block_count,
                warn_count,
                info_count,
                converged,
                category_counts,
            });
            let _ = write_ssb_row(
                writer,
                SsbActionType::CriticRoundComplete,
                CRITIC_AGENT_ID,
                payload,
            )
            .await;
        }

        framed.critic_rounds.push(round);

        if converged {
            info!(
                round = round_idx,
                warn_count, info_count,
                "critic loop converged with zero BLOCK findings"
            );
            return Ok(rounds_used);
        }

        if round_idx < cap {
            // Feed findings back into FramedIdea so the next round has
            // different inputs to evaluate. Deterministic at MVP: record
            // each BLOCK category in known_unknowns + clear any KeyDecision
            // whose name appears in a Contradiction finding's proposed_fix.
            apply_findings_as_rescope(framed);
        }
    }

    // cap reached with BLOCKs still present
    let last_block_count = framed
        .critic_rounds
        .last()
        .map(|r| r.block_count())
        .unwrap_or(0);
    if last_block_count > 0 {
        warn!(
            rounds_used,
            block_count = last_block_count,
            "critic loop exhausted MAX_CRITIC_ROUNDS with unresolved BLOCKs"
        );
        return Err(IdeaFramingError::MaxRoundsReachedWithBlocks {
            block_count: last_block_count,
        });
    }
    Ok(rounds_used)
}

/// Deterministic per-round critic (no LLM at MVP). Inspects the FramedIdea
/// for structural gaps + obvious contradictions + missing mandatory streams
/// and emits CriticFindings. Real LLM-driven critique lands when the agent
/// dispatch path is wired in via [`AgentRole::IdeaCritic`].
fn run_one_critic_round_deterministic(framed: &FramedIdea) -> Vec<CriticFinding> {
    let mut findings: Vec<CriticFinding> = Vec::new();

    // C-MISSING — required FramedIdea fields empty.
    if framed.raw_prompt.trim().is_empty() {
        findings.push(CriticFinding {
            category: CriticCategory::Missing,
            severity: Severity::Block,
            evidence_ref: None,
            proposed_fix: "raw_prompt is empty — restart framing with non-empty prompt".to_owned(),
        });
    }
    if framed.key_decisions.is_empty() {
        findings.push(CriticFinding {
            category: CriticCategory::Missing,
            severity: Severity::Warn,
            evidence_ref: None,
            proposed_fix: "key_decisions list is empty — framer should record at least one architectural decision".to_owned(),
        });
    }
    // C-MARKET / C-FEASIBILITY — mandatory streams degraded indicate the
    // framer is operating on an incomplete grounding base.
    if !framed.research_summary.degraded_streams.is_empty() {
        for stream_name in &framed.research_summary.degraded_streams {
            findings.push(CriticFinding {
                category: CriticCategory::Market,
                severity: Severity::Warn,
                evidence_ref: None,
                proposed_fix: format!(
                    "mandatory research stream '{}' failed to converge — framer must document the gap before advancing",
                    stream_name
                ),
            });
        }
    }
    // C-VAGUE — prompt too short.
    if framed.raw_prompt.trim().len() < 12 {
        findings.push(CriticFinding {
            category: CriticCategory::Vague,
            severity: Severity::Info,
            evidence_ref: None,
            proposed_fix: "prompt is very short — consider asking the user for elaboration via tradeoff card".to_owned(),
        });
    }
    // C-SLOP — Other class indicates the classifier could not anchor.
    if framed.project_class == ProjectClass::Other {
        findings.push(CriticFinding {
            category: CriticCategory::Slop,
            severity: Severity::Warn,
            evidence_ref: None,
            proposed_fix: "ProjectClass=Other — classifier could not anchor; framer should ask the user to clarify the project class".to_owned(),
        });
    }

    findings
}

/// Mutate the FramedIdea to record BLOCK category names in known_unknowns so
/// the next critic round has different inputs to evaluate. Deterministic at
/// MVP — real rescope-from-critic LLM dispatch lands when idea_framer is wired
/// in via `AgentRole::IdeaFramer`.
fn apply_findings_as_rescope(framed: &mut FramedIdea) {
    if let Some(round) = framed.critic_rounds.last() {
        for finding in &round.findings {
            if finding.severity == Severity::Block {
                let note = format!(
                    "[critic round {} BLOCK / {}]: {}",
                    round.round_index,
                    finding.category.as_snake_case(),
                    finding.proposed_fix
                );
                if !framed.known_unknowns.contains(&note) {
                    framed.known_unknowns.push(note);
                }
            }
        }
    }
}

// ===========================================================================
// Top-level orchestrator
// ===========================================================================

/// Top-level entry — given a raw user prompt + research presets + session
/// scratch dir, run the full FRAMING flow:
///   1. Emit IdeaFramingStarted.
///   2. Classify project class + emit ProjectClassClassified.
///   3. Dispatch multi-root research per preset.
///   4. Build initial FramedIdea (deterministic at MVP; LLM later).
///   5. Run critic loop; on convergence emit IdeaFramed.
///   6. Emit TradeoffCardEmitted.
///   7. Persist framed-idea.toml + critic-rounds/round-N.toml +
///      tradeoff-responses.toml under
///      `workflow/sessions/{sid}/idea-framing/`.
///
/// `ssb` is optional — passing `None` skips SSB emission (used by unit tests).
pub async fn frame_idea(
    prompt: &str,
    presets: &ResearchPresets,
    session_dir: &std::path::Path,
    session_id: &str,
    ssb: Option<&Arc<Ssb>>,
    clock: Arc<dyn ClockProvider>,
) -> Result<FramedIdea, IdeaFramingError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(IdeaFramingError::EmptyPrompt);
    }

    let started_at_ms = Ssb::now_ms_via_clock(&clock);

    // SSB: idea_framing_started
    if let Some(writer) = ssb {
        let payload = SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
            session_id: session_id.to_owned(),
            prompt: trimmed.to_owned(),
            started_at_ms,
        });
        let _ = write_ssb_row(
            writer,
            SsbActionType::IdeaFramingStarted,
            FRAMER_AGENT_ID,
            payload,
        )
        .await;
    }

    // Classification
    let (class, matched) = classify_project_class_with_pattern(trimmed);
    debug!(?class, matched=?matched, "classified project class");
    if let Some(writer) = ssb {
        let payload = SsbPayload::ProjectClassClassified(ProjectClassClassifiedPayload {
            session_id: session_id.to_owned(),
            project_class: class.as_snake_case().to_owned(),
            matched_pattern: matched,
        });
        let _ = write_ssb_row(
            writer,
            SsbActionType::ProjectClassClassified,
            FRAMER_AGENT_ID,
            payload,
        )
        .await;
    }

    // Research dispatch
    let streams = load_research_preset(class, presets);
    let research_summary = dispatch_multi_root_research(streams, Arc::clone(&clock))?;

    // Build initial framed idea
    let tradeoff_card = build_initial_tradeoff_card(class, trimmed);
    let mut framed = FramedIdea {
        project_class: class,
        raw_prompt: trimmed.to_owned(),
        research_summary,
        key_decisions: build_initial_key_decisions(class),
        known_unknowns: Vec::new(),
        tradeoff_card: tradeoff_card.clone(),
        critic_rounds: Vec::new(),
    };

    // Critic loop. stub_ban: never short-circuit max_rounds — if zero
    // findings in round 1 the loop still records the round and exits via
    // the `converged` early-return at the bottom of each iteration. That
    // counts as one round used, NOT skip-the-loop.
    let rounds_used = match run_critic_loop(
        &mut framed,
        MAX_CRITIC_ROUNDS,
        ssb,
        session_id,
    )
    .await
    {
        Ok(n) => n,
        Err(IdeaFramingError::MaxRoundsReachedWithBlocks { block_count }) => {
            // Persist the framed-idea with `converged=false` artifact for
            // forensics, then surface the error to the caller. Dashboard
            // surfaces this via the IdeaFramed payload's `converged: false`.
            let artifact_path = persist_artifacts(&framed, session_dir, false)?;
            if let Some(writer) = ssb {
                let payload = SsbPayload::IdeaFramed(IdeaFramedPayload {
                    session_id: session_id.to_owned(),
                    artifact_path,
                    project_class: class.as_snake_case().to_owned(),
                    rounds_used: MAX_CRITIC_ROUNDS,
                    tradeoff_count: framed.tradeoff_card.tradeoffs.len() as u32,
                    converged: false,
                });
                let _ = write_ssb_row(
                    writer,
                    SsbActionType::IdeaFramed,
                    FRAMER_AGENT_ID,
                    payload,
                )
                .await;
            }
            return Err(IdeaFramingError::MaxRoundsReachedWithBlocks { block_count });
        }
        Err(e) => return Err(e),
    };

    // Persist artifacts.
    let artifact_path = persist_artifacts(&framed, session_dir, true)?;

    // Emit IdeaFramed + TradeoffCardEmitted.
    if let Some(writer) = ssb {
        let payload = SsbPayload::IdeaFramed(IdeaFramedPayload {
            session_id: session_id.to_owned(),
            artifact_path,
            project_class: class.as_snake_case().to_owned(),
            rounds_used,
            tradeoff_count: framed.tradeoff_card.tradeoffs.len() as u32,
            converged: true,
        });
        let _ = write_ssb_row(
            writer,
            SsbActionType::IdeaFramed,
            FRAMER_AGENT_ID,
            payload,
        )
        .await;

        let names: Vec<String> = framed
            .tradeoff_card
            .tradeoffs
            .iter()
            .map(|t| t.name.clone())
            .collect();
        let counts: Vec<u32> = framed
            .tradeoff_card
            .tradeoffs
            .iter()
            .map(|t| t.options.len() as u32)
            .collect();
        let payload = SsbPayload::TradeoffCardEmitted(TradeoffCardEmittedPayload {
            session_id: session_id.to_owned(),
            tradeoff_names: names,
            option_counts: counts,
        });
        let _ = write_ssb_row(
            writer,
            SsbActionType::TradeoffCardEmitted,
            FRAMER_AGENT_ID,
            payload,
        )
        .await;
    }

    Ok(framed)
}

/// Persist framed-idea.toml + critic-rounds/round-N.toml + an empty
/// tradeoff-responses.toml stub. Returns the absolute path to
/// framed-idea.toml.
fn persist_artifacts(
    framed: &FramedIdea,
    session_dir: &std::path::Path,
    _converged: bool,
) -> Result<String, IdeaFramingError> {
    let idea_dir: PathBuf = session_dir.join("idea-framing");
    std::fs::create_dir_all(&idea_dir)?;

    let framed_path = idea_dir.join("framed-idea.toml");
    let framed_toml = toml::to_string_pretty(framed)?;
    std::fs::write(&framed_path, framed_toml)?;

    let rounds_dir = idea_dir.join("critic-rounds");
    std::fs::create_dir_all(&rounds_dir)?;
    for round in &framed.critic_rounds {
        let path = rounds_dir.join(format!("round-{}.toml", round.round_index));
        let round_toml = toml::to_string_pretty(round)?;
        std::fs::write(&path, round_toml)?;
    }

    // Empty tradeoff-responses.toml is created so the dashboard can write
    // user responses via submit_tradeoff_responses without an io::NotFound.
    let responses_path = idea_dir.join("tradeoff-responses.toml");
    if !responses_path.exists() {
        std::fs::write(&responses_path, "# user tradeoff responses (one entry per Tradeoff.name)\n")?;
    }

    Ok(framed_path.to_string_lossy().into_owned())
}

// ===========================================================================
// Initial-state builders (deterministic at MVP — LLM lands later)
// ===========================================================================

fn build_initial_tradeoff_card(class: ProjectClass, prompt: &str) -> TradeoffCard {
    // Two baseline tradeoffs always present so DETERMINISTIC_CHECK's
    // "tradeoff_count >= 2" gate passes even before the LLM-driven framer
    // adds class-specific tradeoffs.
    let mut tradeoffs = vec![
        Tradeoff {
            name: "build_or_buy".to_owned(),
            options: vec!["build_in_house".to_owned(), "compose_existing".to_owned()],
            default: "compose_existing".to_owned(),
            user_facing_question: format!(
                "For '{}': build everything in-house, or compose existing components?",
                truncate(prompt, 80)
            ),
        },
        Tradeoff {
            name: "scope_breadth".to_owned(),
            options: vec!["focused_mvp".to_owned(), "feature_complete".to_owned()],
            default: "focused_mvp".to_owned(),
            user_facing_question: "Ship a focused MVP first, or aim for feature-complete?".to_owned(),
        },
    ];

    // Class-specific tradeoffs.
    match class {
        ProjectClass::ConsumerMobileApp => {
            tradeoffs.push(Tradeoff {
                name: "platform_target".to_owned(),
                options: vec!["ios_first".to_owned(), "android_first".to_owned(), "cross_platform".to_owned()],
                default: "cross_platform".to_owned(),
                user_facing_question: "Which platform does the MVP target?".to_owned(),
            });
            tradeoffs.push(Tradeoff {
                name: "monetization".to_owned(),
                options: vec!["free_ad_supported".to_owned(), "freemium".to_owned(), "paid".to_owned()],
                default: "freemium".to_owned(),
                user_facing_question: "What monetization model?".to_owned(),
            });
        }
        ProjectClass::WebApp => {
            tradeoffs.push(Tradeoff {
                name: "rendering_strategy".to_owned(),
                options: vec!["ssr".to_owned(), "ssg".to_owned(), "client_only_spa".to_owned()],
                default: "ssr".to_owned(),
                user_facing_question: "Server-side rendering, static generation, or client-only SPA?".to_owned(),
            });
        }
        ProjectClass::SaaSDashboard => {
            tradeoffs.push(Tradeoff {
                name: "multi_tenancy".to_owned(),
                options: vec!["shared_db".to_owned(), "isolated_db_per_tenant".to_owned()],
                default: "shared_db".to_owned(),
                user_facing_question: "Multi-tenant data: shared DB or per-tenant isolation?".to_owned(),
            });
        }
        ProjectClass::GameMod => {
            tradeoffs.push(Tradeoff {
                name: "compatibility_target".to_owned(),
                options: vec!["latest_only".to_owned(), "lts_branch".to_owned(), "broad_compat".to_owned()],
                default: "latest_only".to_owned(),
                user_facing_question: "Which game version does the mod target?".to_owned(),
            });
        }
        _ => {}
    }

    TradeoffCard { tradeoffs }
}

fn build_initial_key_decisions(class: ProjectClass) -> Vec<KeyDecision> {
    // One baseline decision so critic doesn't immediately C-MISSING on
    // key_decisions.is_empty. Class-specific decisions are appended.
    let mut decisions = vec![KeyDecision {
        name: "project_class".to_owned(),
        rationale: format!(
            "Classified by deterministic regex classifier as {}",
            class.as_snake_case()
        ),
        source_refs: vec!["idea_framing::classify_project_class".to_owned()],
    }];

    // Class-specific stub decisions — overwritten by the LLM-driven
    // framer when wired in.
    match class {
        ProjectClass::ConsumerMobileApp => {
            decisions.push(KeyDecision {
                name: "ux_pattern".to_owned(),
                rationale: "consumer mobile apps follow Material/HIG conventions".to_owned(),
                source_refs: vec!["preset:consumer_mobile_app/ios_android_guidelines".to_owned()],
            });
        }
        ProjectClass::SaaSDashboard => {
            decisions.push(KeyDecision {
                name: "data_model".to_owned(),
                rationale: "SaaS dashboards centre on tenant-scoped time-series + cohort analytics".to_owned(),
                source_refs: vec!["preset:saas_dashboard/analytics_patterns".to_owned()],
            });
        }
        _ => {}
    }

    decisions
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

// ===========================================================================
// SSB write helper
// ===========================================================================

async fn write_ssb_row(
    writer: &Arc<Ssb>,
    action_type: SsbActionType,
    agent_id: &str,
    payload: SsbPayload,
) -> Result<i64, IdeaFramingError> {
    let bytes = rmp_serde::to_vec_named(&payload)?;
    let req = SsbWriteRequest {
        action_type,
        agent_id: agent_id.to_owned(),
        superstep: 0,
        payload: bytes,
        files: Vec::new(),
    };
    let seq = writer.write(req).await?;
    Ok(seq)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_class_all_9_variants() {
        assert_eq!(ALL_PROJECT_CLASSES.len(), 9);
    }

    #[test]
    fn critic_category_count_is_10() {
        assert_eq!(ALL_CRITIC_CATEGORIES.len(), NUM_CRITIC_CATEGORIES);
        assert_eq!(NUM_CRITIC_CATEGORIES, 10);
    }

    #[test]
    fn classify_consumer_mobile_app_via_picturethis() {
        let (class, matched) = classify_project_class_with_pattern("make an app like PictureThis");
        assert_eq!(class, ProjectClass::ConsumerMobileApp);
        assert!(matched.is_some());
    }

    #[test]
    fn classify_saas_dashboard_via_mailchimp_competitor() {
        let class = classify_project_class("build a mailchimp competitor with cohort analytics");
        assert_eq!(class, ProjectClass::SaaSDashboard);
    }

    #[test]
    fn classify_game_mod_via_minecraft() {
        let class = classify_project_class("Minecraft mod for redstone automation");
        assert_eq!(class, ProjectClass::GameMod);
    }

    #[test]
    fn classify_other_when_no_match() {
        let class = classify_project_class("hello world");
        assert_eq!(class, ProjectClass::Other);
    }

    #[test]
    fn classify_empty_prompt_is_other() {
        let class = classify_project_class("   ");
        assert_eq!(class, ProjectClass::Other);
    }

    #[test]
    fn project_class_snake_case_roundtrip() {
        for class in ALL_PROJECT_CLASSES {
            let s = class.as_snake_case();
            assert_eq!(ProjectClass::from_snake_case(s), Some(*class));
        }
    }

    #[test]
    fn critic_round_block_count() {
        let round = CriticRound {
            round_index: 1,
            findings: vec![
                CriticFinding {
                    category: CriticCategory::Missing,
                    severity: Severity::Block,
                    evidence_ref: None,
                    proposed_fix: "fix1".to_owned(),
                },
                CriticFinding {
                    category: CriticCategory::Vague,
                    severity: Severity::Warn,
                    evidence_ref: None,
                    proposed_fix: "fix2".to_owned(),
                },
                CriticFinding {
                    category: CriticCategory::Missing,
                    severity: Severity::Block,
                    evidence_ref: None,
                    proposed_fix: "fix3".to_owned(),
                },
            ],
        };
        assert_eq!(round.block_count(), 2);
        assert_eq!(round.warn_count(), 1);
        assert_eq!(round.info_count(), 0);
    }

    #[test]
    fn critic_round_category_counts_emits_all_10() {
        let round = CriticRound {
            round_index: 1,
            findings: Vec::new(),
        };
        let cc = round.category_counts();
        assert_eq!(cc.len(), NUM_CRITIC_CATEGORIES);
        for (_, count) in &cc {
            assert_eq!(*count, 0);
        }
    }
}
