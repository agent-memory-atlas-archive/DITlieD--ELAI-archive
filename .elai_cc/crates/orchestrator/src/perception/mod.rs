//! J-1 — perception gateway: condition every inbound image before any model
//! sees it.
//!
//! Source plan: `docs/thefinalglance/J-plan/J-1_perception_gateway.md`.
//! Decisions: `workflow/reasoning/J-1-decisions.md` (T8).
//!
//! Submodules:
//!   * [`preprocess`] — `PreprocessSpec` + per-target-model sweet-spot edges
//!     (EXTERNAL SRC-3 / HD-1 — design rationale only; every shipped number
//!     is ELAI-measured). The Rust side orchestrates + records; heavy image
//!     ops land on the sidecar in T5 (legs 2/4, sidecar-gated W-10).
//!   * [`scaffold`] — two-call perception scaffold (call 1 =
//!     describe-literal-elements to structured JSON, call 2 = task model
//!     reasons over description+image). Fires ONLY on image-bearing
//!     dispatches; injected AFTER the last cache breakpoint (HD-2 — never
//!     busts the byte-stable prefix). [`scaffold::scaffold`] is WIRED at
//!     `production_agent_rpc.rs::handle_infer_request`.
//!   * [`router`] — hard-limit router table (visual-diff -> pixel-diff path
//!     via B-24 Tier-1 veto, NEVER a VLM; counting>10 / micro-text -> tiling
//!     or typed refusal via D-16; normal -> passthrough). [`router::route`]
//!     is WIRED at `production_agent_rpc.rs::handle_infer_request`.
//!
//! Wire-dark precedent: this module follows the
//! `ELAI_DOCUMENT_VISION_RAG_ENABLE` / `ELAI_PEER_SYSTEMS_ENABLE` /
//! `ELAI_SKILL_SYSTEM_ENABLE` default-OFF gate discipline (GD-4 — measure
//! before default-ON). The single env-var gate [`perception_gateway_enabled`]
//! covers every emit / dispatch site in this module; no per-fn env var.

pub mod citation_prebind_regions;
pub mod fusion;
pub mod layout_graph;
pub mod preprocess;
pub mod prompt_shapes;
pub mod router;
pub mod scaffold;
pub mod sweet_spot;
pub mod transform_ledger;
pub mod vision_ladder;

// Re-exports — the dispatch seam imports these directly.
pub use scaffold::{
    apply_schema_v2, coords_within_schema_v2, elements_pass_schema_v2, prompt_carries_image,
    scaffold, scaffold_with_call1, ElementSource, LiteralElement, SchemaV2Verdict, ScaffoldInput,
    ScaffoldOutcome, ScaffoldResult,
};
pub use citation_prebind_regions::{
    check_region_claims, filter_ghost_regions, freeze_region_anchors, prebind_regions,
    regions_to_ranked, RegionPrebindOutcome,
};
pub use router::{
    classify_prompt, route, route_outcome, RouteOutcome, RouterDecision, RouterInput, RouterType,
    MICRO_TEXT_CONFIDENCE_FLOOR, MULTI_IMAGE_MIN_COUNT,
};
pub use fusion::{fuse_detections, FusionConfig, FusionResult};
pub use layout_graph::{
    check_layout_consistency, emit_layout_graph, LayoutConsistencyReport, LayoutGraph,
};
pub use transform_ledger::{filter_consistent_boxes, TransformLedger, TransformOp};
// J-77 I10: demoted instruction-before-image ordering claim.
pub use prompt_shapes::{
    instruction_image_order_claim, instruction_image_order_label,
    InstructionImageOrderClaim, DEFAULT_INSTRUCTION_IMAGE_ORDER_CLAIM,
};

/// Environment-variable name that gates production dispatch through the
/// perception gateway. Wire-dark default: the scaffold + router are
/// silently bypassed (DegradedPassthrough) until the operator sets this to
/// a truthy value (`"1"` / `"true"` / `"yes"`).
///
/// Mirrors `ELAI_DOCUMENT_VISION_RAG_ENABLE` precedent
/// (`crates/orchestrator/src/document_vision_rag/mod.rs:31`).
pub const ELAI_PERCEPTION_GATEWAY_ENABLE_ENV: &str = "ELAI_PERCEPTION_GATEWAY_ENABLE";

/// True when the perception-gateway wire-dark gate is ON. Returns false
/// when unset / empty / non-truthy. The single gate the dispatch seam at
/// `production_agent_rpc.rs::handle_infer_request` checks before running
/// the router + scaffold.
#[must_use]
pub fn perception_gateway_enabled() -> bool {
    match std::env::var(ELAI_PERCEPTION_GATEWAY_ENABLE_ENV) {
        Ok(v) => {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes")
        }
        Err(_) => false,
    }
}

/// Companion negation for callers that prefer the "skip if disabled"
/// idiom. Mirrors `document_vision_rag_emit_disabled`.
// WIRE-DARK[J-1]
#[must_use]
pub fn perception_gateway_disabled() -> bool {
    !perception_gateway_enabled()
}

/// Stateless perception-gateway handle. Holds the [`PreprocessSpec`] the
/// sidecar preprocess step applies (T5 wires the live sidecar call; T1
/// ships the spec + the struct skeleton). The struct holds no lock across
/// `.await` — the dispatch gate constructs / clones it per-call (the
/// scaffold + router are pure functions over their inputs).
#[derive(Debug, Clone)]
pub struct PerceptionGateway {
    /// Per-target-model sweet-spot edges + JPEG / tile / thumbnail flags.
    pub preprocess_spec: preprocess::PreprocessSpec,
}

impl PerceptionGateway {
    /// Construct a gateway with a non-default [`PreprocessSpec`]. The
    /// dispatch seam currently uses the default spec (T5 wires the
    /// per-target-model selection from the active provider); this builder
    /// is the seam T5 will call.
    // WIRE-DARK[J-1]
    #[must_use]
    pub fn with_spec(preprocess_spec: preprocess::PreprocessSpec) -> Self {
        Self { preprocess_spec }
    }

    /// Construct a gateway with the default [`PreprocessSpec`] (Claude
    /// sweet-spot, JPEG q85, no tiling, no thumbnail). Used by the dispatch
    /// seam when no per-target-model spec is configured.
    #[must_use]
    pub fn default_spec() -> Self {
        Self {
            preprocess_spec: preprocess::PreprocessSpec::default(),
        }
    }
}

impl Default for PerceptionGateway {
    fn default() -> Self {
        Self::default_spec()
    }
}
