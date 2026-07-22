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

/// Minimum image count that triggers tiled-composite routing (VR-4: N>1
/// evidence images escalate to tiled-composite).
pub const MULTI_IMAGE_MIN_COUNT: u32 = 2;

/// Confidence floor for micro-text routing (VR-5). When a confidence score
/// is present and below this floor, the router returns typed refusal
/// instead of tiling.
pub const MICRO_TEXT_CONFIDENCE_FLOOR: f32 = 0.5;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Number of evidence images in the frame (VR-4). 0 = unknown / absent;
    /// >= [`MULTI_IMAGE_MIN_COUNT`] triggers tiled-composite routing.
    #[serde(default = "default_image_count")]
    pub image_count: u32,
    /// Optional confidence score for micro-text routing (VR-5). When
    /// `Some(c)` and `c < MICRO_TEXT_CONFIDENCE_FLOOR`, the router
    /// returns typed refusal instead of tiling. `None` = no confidence
    /// signal available, tiling proceeds.
    #[serde(default)]
    pub micro_text_confidence: Option<f32>,
}

fn default_image_count() -> u32 {
    0
}

impl RouterInput {
    /// Construct a [`RouterInput`] with `image_path` + `reference_path`
    /// set to `None` and `image_count` set to 0 (the common case at the
    /// dispatch gate — the runtime InferRequest frame is text-only today).
    #[must_use]
    pub fn new(router_type: RouterType, has_image: bool) -> Self {
        Self {
            router_type,
            has_image,
            image_path: None,
            reference_path: None,
            image_count: 0,
            micro_text_confidence: None,
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
    /// Multi-image N>1 intent (VR-4) — route to tiled-composite. The
    /// gate site emits the routing Kind + GD-5 residue; live B-24 Mode B
    /// dispatch is NOT wired this slice.
    RouteToTiledComposite,
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

/// Shared routing outcome — the decision plus a stable routing Kind string
/// and an optional GD-5 honest residue note. Contracts and the production
/// gate call [`route_outcome`] (the SAME fn) so there is one source of truth
/// for the routing decision + Kind + residue.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteOutcome {
    pub decision: RouterDecision,
    pub kind: &'static str,
    pub residue: Option<&'static str>,
}

/// Run the hard-limit router on `input` and return the decision + routing
/// Kind + GD-5 residue. SYNC — model-free (controller decision 4).
///
/// Priority (deterministic):
///   a. `VisualDiff` — paths present -> `RouteToPixelDiff`; else
///      `DegradedPassthrough`. residue None.
///   b. `image_count >= MULTI_IMAGE_MIN_COUNT` -> `RouteToTiledComposite`,
///      kind `"tiled_composite"`, residue naming VR-14 + Mode B not wired.
///      Multi-image wins over Counting / MicroText / Normal.
///   c. `Counting { threshold }` — threshold >= DEFAULT -> `TypedRefusal`;
///      else `RouteToTiling`. residue None.
///   d. `MicroText` — confidence Some(c) && c < floor -> `TypedRefusal`
///      (VR-5); else `RouteToTiling`. residue names VR-5.
///   e. `Normal` -> `Passthrough`. residue None.
#[must_use]
pub fn route_outcome(input: &RouterInput) -> RouteOutcome {
    match input.router_type {
        RouterType::VisualDiff => {
            if input.image_path.is_some() && input.reference_path.is_some() {
                RouteOutcome {
                    decision: RouterDecision::RouteToPixelDiff,
                    kind: "pixel_diff",
                    residue: None,
                }
            } else {
                RouteOutcome {
                    decision: RouterDecision::DegradedPassthrough,
                    kind: "degraded_passthrough",
                    residue: None,
                }
            }
        }
        _ if input.image_count >= MULTI_IMAGE_MIN_COUNT => RouteOutcome {
            decision: RouterDecision::RouteToTiledComposite,
            kind: "tiled_composite",
            residue: Some(
                "[DESIGN] multi-image N>1 escalates to tiled-composite (VR-14); B-24 Mode B live dispatch not wired this slice",
            ),
        },
        RouterType::Counting { threshold } => {
            if threshold >= DEFAULT_COUNTING_THRESHOLD {
                RouteOutcome {
                    decision: RouterDecision::TypedRefusal,
                    kind: "counting_typed_refusal",
                    residue: None,
                }
            } else {
                RouteOutcome {
                    decision: RouterDecision::RouteToTiling,
                    kind: "counting_tiling",
                    residue: None,
                }
            }
        }
        RouterType::MicroText => {
            if let Some(c) = input.micro_text_confidence {
                if c < MICRO_TEXT_CONFIDENCE_FLOOR {
                    return RouteOutcome {
                        decision: RouterDecision::TypedRefusal,
                        kind: "micro_text_typed_refusal",
                        residue: Some(
                            "[DESIGN] micro-text confidence below floor \u{2192} typed refusal (VR-5); no guess",
                        ),
                    };
                }
            }
            RouteOutcome {
                decision: RouterDecision::RouteToTiling,
                kind: "micro_text_tiling",
                residue: Some(
                    "[DESIGN] micro-text \u{2192} tiling (VR-5); confidence absent or >= floor",
                ),
            }
        }
        RouterType::Normal => RouteOutcome {
            decision: RouterDecision::Passthrough,
            kind: "passthrough",
            residue: None,
        },
    }
}

/// Thin wrapper over [`route_outcome`] returning only the decision. Kept
/// for backward compatibility with existing j1 callers.
#[must_use]
pub fn route(input: &RouterInput) -> RouterDecision {
    route_outcome(input).decision
}
