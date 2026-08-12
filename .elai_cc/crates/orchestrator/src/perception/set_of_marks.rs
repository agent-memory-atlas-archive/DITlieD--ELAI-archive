//! P-13 Set-of-Marks id indirection and source provenance.
//!
//! The harness assigns ids and owns geometry. Callers resolve model-selected
//! integer ids back to these harness-owned elements.

use serde::{Deserialize, Serialize};

use super::scaffold::{ElementSource, LiteralElement};

/// A numbered element and its harness-owned geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkedElement {
    /// One-based integer selected by the model.
    pub box_id: u32,
    /// The source-tagged element associated with this id.
    pub element: LiteralElement,
}

/// Trust class for a perception source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTrustLevel {
    /// The accessibility tree supplies exact action identity.
    Authoritative,
    /// OCR supplies text identity but does not authorize actions.
    TextDerived,
    /// The source can guide perception but cannot authorize actions.
    Advisory,
}

/// Build the model-facing numbered list from interactable elements.
///
/// Ids are assigned in input order, start at one, and are written into the
/// copied element. Non-interactable elements stay out of the candidate list.
#[must_use]
pub fn build_numbered_list(elements: &[LiteralElement]) -> Vec<MarkedElement> {
    elements
        .iter()
        .filter(|element| element.interactable)
        .enumerate()
        .map(|(index, element)| {
            let box_id = (index as u32).saturating_add(1);
            let mut marked_element = element.clone();
            marked_element.box_id = Some(box_id);
            MarkedElement {
                box_id,
                element: marked_element,
            }
        })
        .collect()
}

/// Resolve a model-selected integer id to a marked element.
#[must_use]
pub fn resolve_by_id(marked: &[MarkedElement], id: u32) -> Option<&MarkedElement> {
    if id == 0 {
        return None;
    }
    marked.iter().find(|element| element.box_id == id)
}

/// Classify a source according to its action trust level.
#[must_use]
pub fn trust_level(source: ElementSource) -> SourceTrustLevel {
    match source {
        ElementSource::AxtreeExact => SourceTrustLevel::Authoritative,
        ElementSource::OcrText | ElementSource::Ocr => SourceTrustLevel::TextDerived,
        ElementSource::VlmCaption
        | ElementSource::PixelYolo
        | ElementSource::Vlm
        | ElementSource::Layout
        | ElementSource::Detector
        | ElementSource::Merged => SourceTrustLevel::Advisory,
    }
}

/// Return whether a source can authorize an action-critical read.
#[must_use]
pub fn is_action_authoritative(source: ElementSource) -> bool {
    matches!(trust_level(source), SourceTrustLevel::Authoritative)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(label: &str, interactable: bool) -> LiteralElement {
        LiteralElement {
            label: label.to_string(),
            bbox: [0, 0, 10, 10],
            source: ElementSource::AxtreeExact,
            confidence: 1.0,
            box_id: None,
            interactable,
        }
    }

    #[test]
    fn empty_input_has_no_marks() {
        assert!(build_numbered_list(&[]).is_empty());
    }

    #[test]
    fn non_interactable_elements_are_filtered() {
        let elements = vec![element("hidden", false)];
        assert!(build_numbered_list(&elements).is_empty());
    }

    #[test]
    fn resolver_is_one_based() {
        let marked = build_numbered_list(&[element("one", true)]);
        assert!(resolve_by_id(&marked, 0).is_none());
        assert_eq!(resolve_by_id(&marked, 1).map(|item| item.box_id), Some(1));
        assert!(resolve_by_id(&marked, 2).is_none());
    }
}
