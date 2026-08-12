//! P-13 contract test — Set-of-Marks id indirection and provenance.
//!
//! These tests call the same builder and resolver that the production
//! perception gateway uses. The model-facing API is an integer id only.

use elai_orchestrator::perception::{
    build_numbered_list, is_action_authoritative, resolve_by_id, trust_level, ElementSource,
    LiteralElement, SourceTrustLevel,
};

fn element(label: &str, source: ElementSource, interactable: bool) -> LiteralElement {
    LiteralElement {
        label: label.to_string(),
        bbox: [10, 20, 30, 40],
        source,
        confidence: 1.0,
        box_id: None,
        interactable,
    }
}

#[test]
fn builder_assigns_sequential_ids_and_filters_non_interactable_elements() {
    let elements = vec![
        element("first", ElementSource::AxtreeExact, true),
        element("hidden", ElementSource::AxtreeExact, false),
        element("third", ElementSource::OcrText, true),
    ];

    let marked = build_numbered_list(&elements);

    assert_eq!(marked.len(), 2);
    assert_eq!(marked[0].box_id, 1);
    assert_eq!(marked[1].box_id, 2);
    assert_eq!(marked[0].element.box_id, Some(1));
    assert_eq!(marked[1].element.box_id, Some(2));
    assert_eq!(marked[0].element.label, "first");
    assert_eq!(marked[1].element.label, "third");
}

#[test]
fn resolver_accepts_integer_ids_and_rejects_invalid_ids() {
    let marked = build_numbered_list(&[
        element("first", ElementSource::AxtreeExact, true),
        element("second", ElementSource::OcrText, true),
    ]);

    assert_eq!(resolve_by_id(&marked, 0), None);
    assert_eq!(resolve_by_id(&marked, 3), None);
    assert_eq!(
        resolve_by_id(&marked, 2).map(|item| item.element.label.as_str()),
        Some("second")
    );
}

#[test]
fn source_tags_are_preserved_on_every_marked_element() {
    let marked = build_numbered_list(&[
        element("ax", ElementSource::AxtreeExact, true),
        element("ocr", ElementSource::OcrText, true),
        element("caption", ElementSource::VlmCaption, true),
        element("pixel", ElementSource::PixelYolo, true),
    ]);

    assert_eq!(
        marked
            .iter()
            .map(|item| item.element.source)
            .collect::<Vec<_>>(),
        vec![
            ElementSource::AxtreeExact,
            ElementSource::OcrText,
            ElementSource::VlmCaption,
            ElementSource::PixelYolo,
        ]
    );
}

#[test]
fn only_axtree_exact_is_action_authoritative() {
    let sources = [
        ElementSource::AxtreeExact,
        ElementSource::OcrText,
        ElementSource::VlmCaption,
        ElementSource::PixelYolo,
        ElementSource::Ocr,
        ElementSource::Layout,
        ElementSource::Detector,
        ElementSource::Vlm,
        ElementSource::Merged,
    ];

    for source in sources {
        assert_eq!(
            is_action_authoritative(source),
            source == ElementSource::AxtreeExact,
            "unexpected action authority for {source:?}"
        );
    }
}

#[test]
fn trust_hierarchy_keeps_text_above_advisory_sources() {
    assert_eq!(
        trust_level(ElementSource::AxtreeExact),
        SourceTrustLevel::Authoritative
    );
    assert_eq!(
        trust_level(ElementSource::OcrText),
        SourceTrustLevel::TextDerived
    );
    assert_eq!(trust_level(ElementSource::Ocr), SourceTrustLevel::TextDerived);
    assert_eq!(
        trust_level(ElementSource::Vlm),
        SourceTrustLevel::Advisory
    );
    assert_eq!(
        trust_level(ElementSource::PixelYolo),
        SourceTrustLevel::Advisory
    );
}

#[test]
fn literal_element_defaults_keep_old_json_compatible() {
    let element: LiteralElement =
        serde_json::from_str(r#"{"label":"old","bbox":[0,0,50,50]}"#)
            .expect("old JSON must deserialize");

    assert_eq!(element.box_id, None);
    assert!(element.interactable);
}
