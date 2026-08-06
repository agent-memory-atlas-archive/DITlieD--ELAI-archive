//! J-8 T7 (VR-10 / VR-6) — prompt-shape library for perception upgrades.
//!
//! Exports the VR-10 component-inventory prompt template and the VR-6 MoAI
//! citation string. These are consumed by H-14's future perception-prompt
//! pipeline; this slice ships the library exports only (no production mount).

/// Field keys the VR-10 component-inventory template requires per component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentInventoryFields;

impl ComponentInventoryFields {
    pub const TYPE: &'static str = "type";
    pub const VERBATIM_TEXT: &'static str = "verbatim text";
    pub const HIERARCHY: &'static str = "hierarchy";
    pub const LAYOUT_CUES: &'static str = "layout cues";
}

/// Ordered field list for the component-inventory template (stable order for
/// H-14 consumers).
// WIRE-DARK[J-8]
#[must_use]
pub fn component_inventory_fields() -> &'static [&'static str] {
    &[
        ComponentInventoryFields::TYPE,
        ComponentInventoryFields::VERBATIM_TEXT,
        ComponentInventoryFields::HIERARCHY,
        ComponentInventoryFields::LAYOUT_CUES,
    ]
}

/// Full prompt-shape template body H-14 will consume. Instructs the model to
/// emit `{type, verbatim text, hierarchy, layout cues}` per component before
/// any codegen.
// WIRE-DARK[J-8]
#[must_use]
pub fn component_inventory_template() -> &'static str {
    "For each component in the image, emit an inventory entry with these fields \
before any codegen: {type, verbatim text, hierarchy, layout cues}. \
The type field names the component kind. The verbatim text field reproduces \
the exact text content. The hierarchy field records the parent/child position. \
The layout cues field captures spatial placement hints."
}

/// VR-6 MoAI citation string (design rationale only, not a capability claim).
pub const MOAI_CITATION: &str =
    "MoAI (KAIST, arXiv:2403.07508): verbalizing specialist-CV outputs into the VLM context";

// WIRE-DARK[J-8]
#[must_use]
pub fn moai_citation() -> &'static str {
    MOAI_CITATION
}

// J-77 I10 — demote the "instruction BEFORE image" assertion to unverified
// folklore. J-1 leg-3 asserted it as settled; the report found NO strong
// benchmark evidence it universally improves faithfulness. The current
// default content order remains `InstructionThenImage` as a HEURISTIC ONLY;
// the kept lever is the VR-2 schema constraint (`apply_schema_v2`), not the
// ordering. Measure if ordering matters before promoting it.

/// Status claim for the "instruction BEFORE image" content ordering.
///
/// J-77 I10: demoted from a settled assertion to `UnverifiedFolklore`. The
/// default content order remains `InstructionThenImage` as a heuristic only;
/// the VR-2 schema constraint (`apply_schema_v2`) is the kept lever. If a
/// `Settled` variant is added later, it MUST NOT be the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionImageOrderClaim {
    /// The "instruction BEFORE image" ordering is unverified folklore. It is
    /// the current default content order as a heuristic only, NOT a proven
    /// faithfulness improvement. Measure if it matters before promoting.
    UnverifiedFolklore,
}

/// Default status claim for the instruction/image content ordering
/// (`UnverifiedFolklore`).
pub const DEFAULT_INSTRUCTION_IMAGE_ORDER_CLAIM: InstructionImageOrderClaim =
    InstructionImageOrderClaim::UnverifiedFolklore;

/// Return the current status claim for the instruction/image content
/// ordering. J-77 I10: returns `UnverifiedFolklore` (the default).
#[must_use]
pub fn instruction_image_order_claim() -> InstructionImageOrderClaim {
    DEFAULT_INSTRUCTION_IMAGE_ORDER_CLAIM
}

/// Stable string label for the current instruction/image ordering claim.
/// J-77 I10: returns `"unverified_folklore"`. Consumed by the production
/// SSB emit site at `production_agent_rpc.rs` ScaffoldEmitted detail JSON.
#[must_use]
pub fn instruction_image_order_label() -> &'static str {
    match instruction_image_order_claim() {
        InstructionImageOrderClaim::UnverifiedFolklore => "unverified_folklore",
    }
}

#[cfg(test)]
mod j77_i10_tests {
    use super::*;

    #[test]
    fn claim_is_unverified_folklore() {
        assert_eq!(
            instruction_image_order_claim(),
            InstructionImageOrderClaim::UnverifiedFolklore
        );
    }

    #[test]
    fn label_is_exactly_unverified_folklore() {
        assert_eq!(instruction_image_order_label(), "unverified_folklore");
    }

    #[test]
    fn default_claim_is_unverified_folklore() {
        assert_eq!(
            DEFAULT_INSTRUCTION_IMAGE_ORDER_CLAIM,
            InstructionImageOrderClaim::UnverifiedFolklore
        );
    }
}
