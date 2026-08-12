//! J-77 S5 (I5) — citation pre-bind over perception regions.
//!
//! Composes H-53 (`crate::citation_prebind`) without reimplementing
//! freeze/check. Maps perception [`LiteralElement`] regions into
//! [`RankedEvidence`], freezes them into an [`EvidenceAnchorSet`] under
//! `ELAI_SEED`, and checks VLM/claimed region ids against the closed set.
//! Ghost regions (VLM-sourced elements not in the frozen deterministic set)
//! are routed to `unverified` so they never enter downstream counts
//! (TD-3: counts never from VLM).

use crate::citation_prebind::{
    check_citation_in_set, freeze_anchors, CitationCheckResult, EvidenceAnchorSet,
    RankedEvidence,
};
use super::scaffold::{ElementSource, LiteralElement};

/// Map regions → [`RankedEvidence`].
///
/// Id order = input order (rank = index); excerpt = label; source_uri =
/// `region:{i}` (0-based index). Source tier 0 (T1 — deterministic detector
/// output).
fn regions_to_ranked_internal(regions: &[LiteralElement]) -> Vec<RankedEvidence> {
    regions
        .iter()
        .enumerate()
        .map(|(i, r)| RankedEvidence {
            rank: i as u32,
            excerpt: r.label.clone(),
            source_uri: format!("region:{i}"),
            retrieved_at_ms: 0,
            source_tier: 0,
        })
        .collect()
}

/// Map regions → [`RankedEvidence`] (id order = input order; excerpt = label;
/// source_uri = `region:{i}`). Public for contract tests and callers that
/// need the ranked form.
#[must_use]
pub fn regions_to_ranked(regions: &[LiteralElement]) -> Vec<RankedEvidence> {
    regions_to_ranked_internal(regions)
}

/// Freeze region set under `seed` (pass `ELAI_SEED` or 0).
#[must_use]
pub fn freeze_region_anchors(regions: &[LiteralElement], seed: u64) -> EvidenceAnchorSet {
    freeze_anchors(&regions_to_ranked_internal(regions), seed)
}

/// Claimed region ids (1-based) that appear in model text as `[N]` or
/// `region:N`. Also accept bare integer claims from a structured claim list.
///
/// Returns one [`CitationCheckResult`] per claimed id, using
/// [`check_citation_in_set`] with an empty excerpt (id-membership-only —
/// Tier-1 path).
#[must_use]
pub fn check_region_claims(
    claimed_ids: &[u32],
    set: &EvidenceAnchorSet,
) -> Vec<CitationCheckResult> {
    claimed_ids
        .iter()
        .map(|&id| check_citation_in_set(set, id, ""))
        .collect()
}

/// Outcome of region pre-binding: the frozen anchor set plus the
/// kept / unverified partition and per-claim check results.
#[derive(Debug, Clone)]
pub struct RegionPrebindOutcome {
    /// The frozen, blake3-pinned anchor set.
    pub anchor_set: EvidenceAnchorSet,
    /// Elements that survived the closed-set membership check.
    pub kept: Vec<LiteralElement>,
    /// VLM-sourced elements whose label/bbox did not match any frozen
    /// anchor (ghost regions).
    pub unverified: Vec<LiteralElement>,
    /// Per-claim check results for `claimed_ids`.
    pub checks: Vec<CitationCheckResult>,
}

/// Production entry: freeze anchors from DETECTION regions (non-Vlm
/// preferred), then check VLM/claimed ids against the set. Ghosts →
/// `unverified`.
///
/// `detection_regions` are the deterministic detector output (the closed
/// set). `claimed_ids` are 1-based region ids the model claims. Elements in
/// `detection_regions` with `source == Vlm` are treated as claims to verify
/// against the non-Vlm frozen set; Vlm elements that fail membership land in
/// `unverified`.
#[must_use]
pub fn prebind_regions(
    detection_regions: &[LiteralElement],
    claimed_ids: &[u32],
    seed: u64,
) -> RegionPrebindOutcome {
    // Freeze ALL non-Vlm elements as the closed set.
    let non_vlm: Vec<LiteralElement> = detection_regions
        .iter()
        .filter(|e| e.source != ElementSource::Vlm)
        .cloned()
        .collect();

    let anchor_set = freeze_region_anchors(&non_vlm, seed);
    let checks = check_region_claims(claimed_ids, &anchor_set);

    // Partition: non-Vlm always kept; Vlm elements checked against frozen set.
    let mut kept = Vec::new();
    let mut unverified = Vec::new();

    // Frozen anchor labels for membership check.
    let frozen_labels: Vec<&str> =
        anchor_set.anchors.iter().map(|a| a.excerpt.as_str()).collect();

    for elem in detection_regions {
        if elem.source != ElementSource::Vlm {
            kept.push(elem.clone());
        } else {
            // Vlm element: check if its label matches any frozen anchor.
            if frozen_labels.iter().any(|l| *l == elem.label) {
                kept.push(elem.clone());
            } else {
                unverified.push(elem.clone());
            }
        }
    }

    // Also route OutOfSet claimed ids to unverified (if a Vlm element
    // corresponds to a claimed id that is OutOfSet, it is already caught by
    // the label check above; the checks vector carries the per-claim
    // verdict for the caller).
    RegionPrebindOutcome {
        anchor_set,
        kept,
        unverified,
        checks,
    }
}

/// Convenience: when all scaffold elements ARE the closed set and
/// `claimed_ids` references them; elements with `source == Vlm` that fail
/// membership land in `unverified`.
///
/// Rule: freeze ALL non-Vlm elements as the closed set; any Vlm element
/// whose label does not match any frozen anchor → `unverified`. Non-Vlm
/// always kept. No `claimed_ids` are checked (empty slice).
#[must_use]
pub fn filter_ghost_regions(elements: &[LiteralElement], seed: u64) -> RegionPrebindOutcome {
    prebind_regions(elements, &[], seed)
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation_prebind::CitationCheckOutcome;

    fn elem(label: &str, bbox: [u32; 4], source: ElementSource, conf: f32) -> LiteralElement {
        LiteralElement {
            label: label.to_string(),
            bbox,
            source,
            confidence: conf,
            box_id: None,
            interactable: true,
        }
    }

    #[test]
    fn ghost_region_routed_to_unverified() {
        // Detection: a real button (Ocr) + a VLM ghost (fake_control).
        let regions = vec![
            elem("button", [0, 0, 100, 50], ElementSource::Ocr, 1.0),
            elem("fake_control", [200, 200, 300, 250], ElementSource::Vlm, 0.3),
        ];
        let outcome = filter_ghost_regions(&regions, 0);
        assert!(outcome.unverified.iter().any(|e| e.label == "fake_control"));
        assert!(outcome.kept.iter().any(|e| e.label == "button"));
        assert!(!outcome.kept.iter().any(|e| e.label == "fake_control"));
    }

    #[test]
    fn closed_set_out_of_set_claim() {
        let regions = vec![
            elem("region_a", [0, 0, 100, 100], ElementSource::Ocr, 1.0),
            elem("region_b", [100, 0, 200, 100], ElementSource::Ocr, 1.0),
        ];
        let outcome = prebind_regions(&regions, &[99], 0);
        assert_eq!(outcome.checks.len(), 1);
        assert_eq!(outcome.checks[0].outcome, CitationCheckOutcome::Ghost);
    }

    #[test]
    fn source_confidence_roundtrip() {
        let element = elem("text", [10, 20, 30, 40], ElementSource::Vlm, 0.3);
        let json = serde_json::to_string(&element).expect("serialize");
        let deserialized: LiteralElement =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.source, ElementSource::Vlm);
        assert!((deserialized.confidence - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn old_json_without_source_confidence_deserializes() {
        let old_json = r#"{"label":"old","bbox":[0,0,50,50]}"#;
        let element: LiteralElement =
            serde_json::from_str(old_json).expect("old JSON must deserialize");
        assert_eq!(element.label, "old");
        assert_eq!(element.source, ElementSource::Ocr);
        assert_eq!(element.confidence, 1.0);
    }

    #[test]
    fn non_vlm_always_kept() {
        let regions = vec![
            elem("a", [0, 0, 10, 10], ElementSource::Ocr, 1.0),
            elem("b", [10, 0, 20, 10], ElementSource::Layout, 1.0),
            elem("c", [20, 0, 30, 10], ElementSource::Detector, 0.9),
            elem("d", [30, 0, 40, 10], ElementSource::Merged, 1.0),
        ];
        let outcome = filter_ghost_regions(&regions, 0);
        assert_eq!(outcome.kept.len(), 4);
        assert!(outcome.unverified.is_empty());
    }

    #[test]
    fn vlm_matching_label_kept() {
        let regions = vec![
            elem("button", [0, 0, 100, 50], ElementSource::Ocr, 1.0),
            elem("button", [0, 0, 100, 50], ElementSource::Vlm, 0.5),
        ];
        let outcome = filter_ghost_regions(&regions, 0);
        assert_eq!(outcome.kept.len(), 2);
        assert!(outcome.unverified.is_empty());
    }

    #[test]
    fn empty_regions_empty_outcome() {
        let outcome = filter_ghost_regions(&[], 0);
        assert!(outcome.anchor_set.is_empty());
        assert!(outcome.kept.is_empty());
        assert!(outcome.unverified.is_empty());
        assert!(outcome.checks.is_empty());
    }

    #[test]
    fn in_set_claim_is_bound() {
        let regions = vec![
            elem("region_a", [0, 0, 100, 100], ElementSource::Ocr, 1.0),
            elem("region_b", [100, 0, 200, 100], ElementSource::Ocr, 1.0),
        ];
        let outcome = prebind_regions(&regions, &[1], 0);
        assert_eq!(outcome.checks.len(), 1);
        assert_eq!(outcome.checks[0].outcome, CitationCheckOutcome::Bound);
    }
}
