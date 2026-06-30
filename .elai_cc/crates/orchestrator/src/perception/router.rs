//! J-1.T3 — hard-limit router table.
//!
//! Source plan: `docs/thefinalglance/J-plan/J-1_perception_gateway.md`
//! (lines 65-66). visual-diff -> pixel-diff path (B-24 Tier-1 veto reused,
//! NEVER a VLM — plan stub_ban: "NO VLM call on the visual-diff path
//! (Tier-1 veto is law)"); counting>10 elements / micro-text -> tiling OR
//! typed refusal (D-16 refusal target, never a guess); normal -> passthrough.
//!
//! Controller decisions (binding — see
//! `workflow/sessions/s-38068/context-package-J-1-T3.md`):
//!   1. `route()` stays SYNC — returns [`RouterDecision`] only. The async
//!      pixel-diff dispatch (`ImageValidator::validate().await`) runs at
//!      the gate site (`handle_infer_request`), NOT inside `route()`.
//!   2. Router gate goes INSIDE `perception_gateway_enabled()` block,
//!      BEFORE the scaffold call. `RouteToPixelDiff` / `TypedRefusal`
//!      short-circuit.
//!   3. Do NOT extend `RefusalReason` enum. `route()` returns
//!      [`RouterDecision::TypedRefusal`]; the gate site returns
//!      `RefusalDecision::Abstain` with `reason=None`.
//!   4. `classify_prompt()` is a deterministic keyword heuristic (no model
//!      call) — `route()` is model-free per the J-1 leg-1 contract.
//!   5. [`RouterInput`] extended with `image_path` + `reference_path`
//!      (`Option<PathBuf>`, serde-defaulted `None`).
//!
//! `route()` is WIRED at `production_agent_rpc.rs::handle_infer_request`
//! — no WIRE-DARK[J-1] marker, no allowlist row.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Default counting threshold (plan: "counting > 10 elements"). Named
/// constant — no magic number (anti-stale-training rule).
pub const DEFAULT_COUNTING_THRESHOLD: u32 = 10;

/// Router type — the hard-limit routing table discriminator. Derived from
/// the dispatch prompt via the deterministic keyword heuristic
/// [`classify_prompt`] (no model call — controller decision 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouterType {
    /// Visual-diff intent (compare two images pixel-by-pixel). Routes to
    /// the B-24 Tier-1 pixel-diff veto — NEVER a VLM (plan stub_ban).
    VisualDiff,
    /// Counting intent — count elements in the image. Carries the
    /// hard-limit threshold (default [`DEFAULT_COUNTING_THRESHOLD`]); at
    /// or above the threshold the router refuses (typed refusal, never a
    /// guess); below the threshold it routes to tiling.
    Counting { threshold: u32 },
    /// Micro-text intent — dense small text in the image. Routes to
    /// tiling (overlapping tiles + a global thumbnail, SRC-3 mechanism 3).
    MicroText,
    /// Normal intent — no hard-limit routing; passthrough to the model.
    Normal,
}

impl Default for RouterType {
    fn default() -> Self {
        RouterType::Normal
    }
}

/// Router input. The dispatch gate at `handle_infer_request` constructs
/// this from the prompt + the optional image / reference paths (controller
/// decision 5: `image_path` + `reference_path` are `Option<PathBuf>`,
/// serde-defaulted `None`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouterInput {
    pub router_type: RouterType,
    /// True when the prompt carries an image marker (reuses
    /// [`super::scaffold::prompt_carries_image`]).
    pub has_image: bool,
    /// Candidate image path for the pixel-diff arm. `None` -> the
    /// visual-diff arm degrades to passthrough (no file to diff).
    #[serde(default)]
    pub image_path: Option<PathBuf>,
    /// Baseline reference path for the pixel-diff arm. `None` -> the
    /// visual-diff arm degrades to passthrough (no baseline to diff
    /// against).
    #[serde(default)]
    pub reference_path: Option<PathBuf>,
}

impl RouterInput {
    /// Construct a [`RouterInput`] with `image_path` + `reference_path`
    /// set to `None` (the common case at the dispatch gate — the runtime
    /// InferRequest frame is text-only today).
    #[must_use]
    pub fn new(router_type: RouterType, has_image: bool) -> Self {
        Self {
            router_type,
            has_image,
            image_path: None,
            reference_path: None,
        }
    }
}

/// Router decision — the per-variant specialist dispatch verdict. NO
/// `RouteToVlm` variant exists (plan stub_ban encoded in the type surface
/// — the visual-diff arm can ONLY return [`RouterDecision::RouteToPixelDiff`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouterDecision {
    /// Normal intent — passthrough to the model (no specialist dispatch).
    Passthrough,
    /// Visual-diff intent — route to the B-24 Tier-1 pixel-diff veto
    /// (NEVER a VLM). The gate site builds a `MediaValidationRequest`,
    /// calls `ImageValidator::validate().await`, reads `Tier1Result.pass`.
    RouteToPixelDiff,
    /// Counting-below-threshold / micro-text intent — route to tiling
    /// (overlapping tiles + a global thumbnail). T5 wires the live tiler.
    RouteToTiling,
    /// Counting-at-or-above-threshold intent — typed refusal (D-16
    /// refusal target, never a guess). The gate site returns
    /// `RefusalDecision::Abstain` with `reason=None` (controller decision
    /// 3 — do NOT extend `RefusalReason`).
    TypedRefusal,
    /// Router could not classify / specialist dispatch failed — the raw
    /// image forwards UNCONDITIONED with an explicit marker (never a
    /// silent fallthrough, plan stub_ban).
    DegradedPassthrough,
}

/// Keyword markers for the deterministic prompt classifier (controller
/// decision 4 — no model call). Kept lowercase; the classifier lowercases
/// the prompt before matching.
const VISUAL_DIFF_KEYWORDS: &[&str] = &[
    "visual diff",
    "visual-diff",
    "pixel diff",
    "pixel-diff",
    "image diff",
    "image-diff",
    "compare images",
    "compare the images",
    "diff the images",
    "diff these images",
    "spot the difference",
    "find the difference",
];

/// Keyword markers for the counting intent.
const COUNTING_KEYWORDS: &[&str] = &[
    "count the",
    "how many",
    "number of",
    "count elements",
    "count objects",
];

/// Keyword markers for the micro-text intent.
const MICRO_TEXT_KEYWORDS: &[&str] = &[
    "micro text",
    "micro-text",
    "small text",
    "dense text",
    "fine text",
    "tiny text",
    "read the text in",
    "ocr",
];

/// Deterministic keyword classifier — maps a prompt string to a
/// [`RouterType`] (controller decision 4 — no model call). The classifier
/// is intentionally conservative: it returns [`RouterType::Normal`] when
/// no keyword matches, so a mis-classified prompt passthroughs to the
/// model rather than mis-routing to a specialist arm.
#[must_use]
pub fn classify_prompt(prompt: &str) -> RouterType {
    let lower = prompt.to_ascii_lowercase();
    if VISUAL_DIFF_KEYWORDS.iter().any(|k| lower.contains(k)) {
        return RouterType::VisualDiff;
    }
    if MICRO_TEXT_KEYWORDS.iter().any(|k| lower.contains(k)) {
        return RouterType::MicroText;
    }
    if COUNTING_KEYWORDS.iter().any(|k| lower.contains(k)) {
        return RouterType::Counting {
            threshold: DEFAULT_COUNTING_THRESHOLD,
        };
    }
    RouterType::Normal
}

/// Run the hard-limit router on `input`. SYNC — returns a
/// [`RouterDecision`] only (controller decision 1). The async pixel-diff
/// dispatch (`ImageValidator::validate().await`) runs at the gate site
/// (`handle_infer_request`), NOT here.
///
/// Per-variant logic:
///   * [`RouterType::VisualDiff`] -> [`RouterDecision::RouteToPixelDiff`]
///     when both `image_path` + `reference_path` are present; else
///     [`RouterDecision::DegradedPassthrough`] (no file to diff — never a
///     VLM, never a silent fallthrough).
///   * [`RouterType::Counting { threshold }`] -> [`RouterDecision::TypedRefusal`]
///     when `threshold >= DEFAULT_COUNTING_THRESHOLD` (counting > 10 ->
///     typed refusal, D-16); else [`RouterDecision::RouteToTiling`].
///   * [`RouterType::MicroText`] -> [`RouterDecision::RouteToTiling`].
///   * [`RouterType::Normal`] -> [`RouterDecision::Passthrough`].
#[must_use]
pub fn route(input: &RouterInput) -> RouterDecision {
    match input.router_type {
        RouterType::VisualDiff => {
            if input.image_path.is_some() && input.reference_path.is_some() {
                RouterDecision::RouteToPixelDiff
            } else {
                // Missing paths — degrade to passthrough (never a VLM,
                // never a silent fallthrough — plan stub_ban).
                RouterDecision::DegradedPassthrough
            }
        }
        RouterType::Counting { threshold } => {
            if threshold >= DEFAULT_COUNTING_THRESHOLD {
                RouterDecision::TypedRefusal
            } else {
                RouterDecision::RouteToTiling
            }
        }
        RouterType::MicroText => RouterDecision::RouteToTiling,
        RouterType::Normal => RouterDecision::Passthrough,
    }
}
