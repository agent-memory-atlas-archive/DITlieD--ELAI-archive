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
