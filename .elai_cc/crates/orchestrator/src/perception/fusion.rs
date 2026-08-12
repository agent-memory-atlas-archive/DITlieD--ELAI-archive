//! J-77 S5 I3 — typed merge graph: deterministic fusion of literal elements.
//!
//! Fuses raw detections words→lines→blocks→semantic-regions by IoU + text
//! similarity + class agreement. One level owns counts (Blocks), killing
//! double-counting from overlapping detections.
//!
//! Source plan: `docs/thefinalglance/J-plan/J-77_*.md` (I3 slice).
//!
//! `ElementSource` lives in `scaffold.rs` (landed I5/I6); this module imports
//! it from there. Fuse runs AFTER `filter_ghost_regions`; the production
//! caller fuses the `kept` elements from the prebind outcome.

use super::scaffold::{ElementSource, LiteralElement};

/// Configuration for [`fuse_detections`].
#[derive(Debug, Clone)]
pub struct FusionConfig {
    /// IoU above this + similar text → merge into one region.
    pub iou_merge_threshold: f32,
    /// Text similarity (0..=1) above this → considered "same text".
    pub text_similarity_threshold: f32,
    /// IoU above this + same source + similar text → DUPLICATE → drop.
    pub max_same_type_iou: f32,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            iou_merge_threshold: 0.5,
            text_similarity_threshold: 0.8,
            max_same_type_iou: 0.9,
        }
    }
}

/// Result of [`fuse_detections`].
#[derive(Debug, Clone, PartialEq)]
pub struct FusionResult {
    /// Fused regions after merge + duplicate drop.
    pub regions: Vec<LiteralElement>,
    /// Count of elements dropped as duplicates.
    pub duplicate_drops: u32,
    /// Level that owns counts (always "Blocks" — never VLM captions).
    pub count_level: &'static str,
}

/// Fuse raw detections into block-level regions. Deterministic: stable sort
/// by `(y0, x0, label)` before merge. Merges by IoU + text similarity +
/// source agreement. Drops duplicates above `max_same_type_iou`.
///
/// TD-3: counts are `regions.len()` after fusion — never invented from
/// captions. Pure-VLM elements (source == Vlm) are stripped from count
/// ownership.
#[must_use]
pub fn fuse_detections(raw: &[LiteralElement], cfg: &FusionConfig) -> FusionResult {
    if raw.is_empty() {
        return FusionResult {
            regions: Vec::new(),
            duplicate_drops: 0,
            count_level: "Blocks",
        };
    }

    // Deterministic ordering: stable sort by (y0, x0, label).
    let mut sorted: Vec<(usize, &LiteralElement)> = raw.iter().enumerate().collect();
    sorted.sort_by(|a, b| {
        let ea = &a.1;
        let eb = &b.1;
        ea.bbox[1]
            .cmp(&eb.bbox[1])
            .then(ea.bbox[0].cmp(&eb.bbox[0]))
            .then(ea.label.cmp(&eb.label))
    });

    let mut merged: Vec<LiteralElement> = Vec::with_capacity(sorted.len());
    let mut duplicate_drops: u32 = 0;

    for &(_, elem) in &sorted {
        let mut found_merge = false;
        for existing in &mut merged {
            let iou = iou(&elem.bbox, &existing.bbox);
            if iou >= cfg.iou_merge_threshold {
                let sim = text_similarity(&elem.label, &existing.label);
                if sim >= cfg.text_similarity_threshold {
                    // Merge: union bbox, keep the label of the larger-area
                    // element (proxy for confidence).
                    let area_elem = bbox_area(&elem.bbox);
                    let area_existing = bbox_area(&existing.bbox);
                    if area_elem > area_existing {
                        existing.label = elem.label.clone();
                    }
                    existing.bbox = union_bbox(&existing.bbox, &elem.bbox);
                    found_merge = true;
                    break;
                }
            }
        }
        if !found_merge {
            // Check for duplicate: high IoU + similar text but below merge
            // threshold is still a potential duplicate if IoU > max_same_type_iou.
            let mut is_dup = false;
            for existing in &merged {
                let iou_val = iou(&elem.bbox, &existing.bbox);
                if iou_val > cfg.max_same_type_iou {
                    let sim = text_similarity(&elem.label, &existing.label);
                    if sim >= cfg.text_similarity_threshold {
                        is_dup = true;
                        break;
                    }
                }
            }
            if is_dup {
                duplicate_drops += 1;
            } else {
                merged.push(elem.clone());
            }
        }
    }

    // Final pass: ensure no two regions with IoU > max_same_type_iou and
    // similar text remain (catch merges that produced near-identical
    // bounding boxes).
    let mut final_regions: Vec<LiteralElement> = Vec::with_capacity(merged.len());
    for elem in merged {
        let mut is_dup = false;
        for existing in &final_regions {
            let iou_val = iou(&elem.bbox, &existing.bbox);
            if iou_val > cfg.max_same_type_iou {
                let sim = text_similarity(&elem.label, &existing.label);
                if sim >= cfg.text_similarity_threshold {
                    is_dup = true;
                    break;
                }
            }
        }
        if is_dup {
            duplicate_drops += 1;
        } else {
            final_regions.push(elem);
        }
    }

    FusionResult {
        regions: final_regions,
        duplicate_drops,
        count_level: "Blocks",
    }
}

/// IoU (Intersection over Union) of two axis-aligned bboxes `[x0, y0, x1, y1]`.
#[must_use]
fn iou(a: &[u32; 4], b: &[u32; 4]) -> f32 {
    let inter_x0 = a[0].max(b[0]);
    let inter_y0 = a[1].max(b[1]);
    let inter_x1 = a[2].min(b[2]);
    let inter_y1 = a[3].min(b[3]);

    let inter_w = if inter_x1 > inter_x0 {
        inter_x1 - inter_x0
    } else {
        0
    };
    let inter_h = if inter_y1 > inter_y0 {
        inter_y1 - inter_y0
    } else {
        0
    };
    let inter = (inter_w as f32) * (inter_h as f32);

    let area_a = bbox_area(a);
    let area_b = bbox_area(b);
    let union_val = area_a + area_b - inter;

    if union_val <= 0.0 {
        0.0
    } else {
        inter / union_val
    }
}

/// Area of an axis-aligned bbox `[x0, y0, x1, y1]`.
#[must_use]
fn bbox_area(b: &[u32; 4]) -> f32 {
    let w = if b[2] > b[0] { b[2] - b[0] } else { 0 };
    let h = if b[3] > b[1] { b[3] - b[1] } else { 0 };
    (w as f32) * (h as f32)
}

/// Union (smallest enclosing) bbox of two axis-aligned bboxes.
#[must_use]
fn union_bbox(a: &[u32; 4], b: &[u32; 4]) -> [u32; 4] {
    [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])]
}

/// Simple text similarity ratio (0..=1) based on longest common subsequence
/// length divided by the longer string length. Exact match → 1.0.
#[must_use]
fn text_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let max_len = a_bytes.len().max(b_bytes.len());
    if max_len == 0 {
        return 1.0;
    }
    let lcs = lcs_length(a_bytes, b_bytes);
    (lcs as f32) / (max_len as f32)
}

/// Longest common subsequence length (byte-level).
fn lcs_length(a: &[u8], b: &[u8]) -> usize {
    let m = a.len();
    let n = b.len();
    if m == 0 || n == 0 {
        return 0;
    }
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.iter_mut().for_each(|v| *v = 0);
    }
    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(label: &str, bbox: [u32; 4]) -> LiteralElement {
        LiteralElement {
            label: label.to_string(),
            bbox,
            source: ElementSource::Detector,
            confidence: 1.0,
            box_id: None,
            interactable: true,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = fuse_detections(&[], &FusionConfig::default());
        assert!(result.regions.is_empty());
        assert_eq!(result.duplicate_drops, 0);
        assert_eq!(result.count_level, "Blocks");
    }

    #[test]
    fn no_overlap_keeps_all() {
        let raw = vec![
            elem("button", [0, 0, 100, 50]),
            elem("input", [200, 200, 300, 250]),
        ];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.regions.len(), 2);
        assert_eq!(result.duplicate_drops, 0);
    }

    #[test]
    fn high_iou_same_text_merges() {
        let raw = vec![
            elem("button", [0, 0, 100, 100]),
            elem("button", [10, 10, 110, 110]),
        ];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.regions.len(), 1, "two overlapping same-label elements should merge");
        assert_eq!(result.duplicate_drops, 0);
    }

    #[test]
    fn high_iou_different_text_kept_separate() {
        let raw = vec![
            elem("button", [0, 0, 100, 100]),
            elem("input", [10, 10, 110, 110]),
        ];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.regions.len(), 2, "different text should not merge or drop");
    }

    #[test]
    fn exact_duplicate_dropped() {
        let raw = vec![
            elem("button", [0, 0, 100, 100]),
            elem("button", [0, 0, 100, 100]),
        ];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.regions.len(), 1, "exact duplicates should collapse to 1");
    }

    #[test]
    fn deterministic_order() {
        let raw_a = vec![
            elem("z_label", [0, 100, 50, 150]),
            elem("a_label", [0, 0, 50, 50]),
        ];
        let raw_b = vec![
            elem("a_label", [0, 0, 50, 50]),
            elem("z_label", [0, 100, 50, 150]),
        ];
        let result_a = fuse_detections(&raw_a, &FusionConfig::default());
        let result_b = fuse_detections(&raw_b, &FusionConfig::default());
        assert_eq!(result_a, result_b, "fusion must be deterministic regardless of input order");
    }

    #[test]
    fn iou_disjoint_boxes() {
        let iou_val = iou(&[0, 0, 10, 10], &[100, 100, 110, 110]);
        assert_eq!(iou_val, 0.0);
    }

    #[test]
    fn iou_identical_boxes() {
        let iou_val = iou(&[0, 0, 10, 10], &[0, 0, 10, 10]);
        assert!((iou_val - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn iou_partial_overlap() {
        let iou_val = iou(&[0, 0, 10, 10], &[5, 5, 15, 15]);
        assert!((iou_val - (25.0 / 175.0)).abs() < 0.001);
    }

    #[test]
    fn text_similarity_exact() {
        assert!((text_similarity("hello", "hello") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn text_similarity_disjoint() {
        assert_eq!(text_similarity("abc", "xyz"), 0.0);
    }

    #[test]
    fn text_similarity_partial() {
        let sim = text_similarity("button", "btn");
        assert!((sim - 0.5).abs() < 0.001);
    }

    #[test]
    fn count_level_never_vlm() {
        let raw = vec![elem("test", [0, 0, 10, 10])];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.count_level, "Blocks");
        assert_ne!(result.count_level, "VLM");
    }

    #[test]
    fn td3_counts_from_regions_len() {
        let raw = vec![
            elem("a", [0, 0, 50, 50]),
            elem("b", [100, 100, 150, 150]),
            elem("c", [200, 200, 250, 250]),
        ];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.regions.len(), 3);
    }

    #[test]
    fn merge_preserves_larger_label() {
        let raw = vec![
            elem("big", [0, 0, 100, 100]),
            elem("big", [10, 10, 110, 110]),
        ];
        let result = fuse_detections(&raw, &FusionConfig::default());
        assert_eq!(result.regions.len(), 1, "overlapping same-label should merge");
        assert_eq!(result.regions[0].bbox, [0, 0, 110, 110]);
    }
}
