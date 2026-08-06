//! J-77 I11 — 6-metric perception-faithfulness done-gate harness (Tier-1).
//!
//! Scores perception pipeline outputs against a labelled fixture using six
//! deterministic, model-free metrics. Calls REAL production fns:
//!   * `perception::fuse_detections` — fusion + duplicate drop
//!   * `perception::filter_ghost_regions` — VLM ghost filtering
//!
//! Thresholds live here as named constants (never in the fixture JSON).
//! The harness PASSES on the shipped fixture and FAILS on falsified inputs.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::perception::{
    filter_ghost_regions, fuse_detections, ElementSource, FusionConfig, LiteralElement,
};

// ---------------------------------------------------------------------------
// Threshold constants (live here, NOT in the fixture JSON)
// ---------------------------------------------------------------------------

/// Metric 1: exact label match on legible GT text regions.
const OCR_EXACT_MATCH_FLOOR: f64 = 0.98;
/// Metric 2: recall @ IoU 0.5 for text blocks + major controls.
const REGION_GROUNDING_RECALL_IOU05_FLOOR: f64 = 0.95;
/// Metric 3: duplicate_region_rate = duplicate_drops / raw_len.
const DUPLICATE_REGION_RATE_CEILING: f64 = 0.01;
/// Metric 4: exact count accuracy for fixtures with <= 10 separated elements.
const EXACT_COUNT_ACCURACY_FLOOR: f64 = 0.90;
/// Metric 6: blind LLM QA lift must be strictly positive.
/// (Metric 5 is uncited_claims == 0, an equality check, no threshold const.)

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum PerceptionFaithfulnessError {
    /// The fixture file could not be read.
    #[error("perception-faithfulness: could not read fixture {path}: {source_msg}")]
    FixtureRead { path: String, source_msg: String },
    /// The fixture JSON could not be parsed.
    #[error("perception-faithfulness: could not parse fixture JSON: {source_msg}")]
    FixtureParse { source_msg: String },
}

// ---------------------------------------------------------------------------
// Fixture model (deserialized from JSON)
// ---------------------------------------------------------------------------

/// A ground-truth region label + bbox + kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GtRegion {
    label: String,
    bbox: [u32; 4],
    #[serde(default)]
    #[allow(dead_code)]
    kind: Option<String>,
}

/// Ground truth: labelled regions + expected count.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GroundTruth {
    regions: Vec<GtRegion>,
    count: u32,
}

/// A predicted detection (raw or VLM claim) with source + confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PredictedDetection {
    label: String,
    bbox: [u32; 4],
    source: String,
    #[serde(default = "default_confidence_f32")]
    confidence: f32,
}

fn default_confidence_f32() -> f32 {
    1.0
}

/// Predicted output: raw detections + VLM claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Predicted {
    raw_detections: Vec<PredictedDetection>,
    #[serde(default)]
    vlm_claims: Vec<PredictedDetection>,
}

/// QA scores from the fixture (no live model call).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QaScores {
    blind_llm_score: f64,
    ocr_only_baseline_score: f64,
}

/// The full fixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Fixture {
    #[allow(dead_code)]
    id: String,
    ground_truth: GroundTruth,
    predicted: Predicted,
    qa: QaScores,
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

/// Per-metric pass/fail + measured value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricResult {
    pub name: String,
    pub value: f64,
    pub pass: bool,
}

/// The 6-metric perception-faithfulness report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerceptionFaithfulnessReport {
    pub fixture_id: String,
    pub ocr_exact_match: MetricResult,
    pub region_grounding_recall_iou05: MetricResult,
    pub duplicate_region_rate: MetricResult,
    pub exact_count_accuracy: MetricResult,
    pub uncited_claims: MetricResult,
    pub blind_llm_qa_lift: MetricResult,
    pub overall_pass: bool,
}

// ---------------------------------------------------------------------------
// IoU helper (local pure fn)
// ---------------------------------------------------------------------------

/// IoU of two axis-aligned bboxes `[x0, y0, x1, y1]`.
fn iou(a: &[u32; 4], b: &[u32; 4]) -> f64 {
    let inter_x0 = a[0].max(b[0]);
    let inter_y0 = a[1].max(b[1]);
    let inter_x1 = a[2].min(b[2]);
    let inter_y1 = a[3].min(b[3]);

    let inter_w = if inter_x1 > inter_x0 { inter_x1 - inter_x0 } else { 0 };
    let inter_h = if inter_y1 > inter_y0 { inter_y1 - inter_y0 } else { 0 };
    let inter = (inter_w as f64) * (inter_h as f64);

    let area_a = bbox_area_f64(a);
    let area_b = bbox_area_f64(b);
    let union_val = area_a + area_b - inter;

    if union_val <= 0.0 {
        0.0
    } else {
        inter / union_val
    }
}

fn bbox_area_f64(b: &[u32; 4]) -> f64 {
    let w = if b[2] > b[0] { b[2] - b[0] } else { 0 };
    let h = if b[3] > b[1] { b[3] - b[1] } else { 0 };
    (w as f64) * (h as f64)
}

// ---------------------------------------------------------------------------
// Source string → ElementSource
// ---------------------------------------------------------------------------

fn parse_source(s: &str) -> ElementSource {
    match s.to_ascii_lowercase().as_str() {
        "ocr" => ElementSource::Ocr,
        "layout" => ElementSource::Layout,
        "detector" => ElementSource::Detector,
        "vlm" => ElementSource::Vlm,
        "merged" => ElementSource::Merged,
        _ => ElementSource::Ocr,
    }
}

// ---------------------------------------------------------------------------
// Core scoring
// ---------------------------------------------------------------------------

/// Score a fixture (already parsed) against its ground truth using all 6
/// metrics. Calls `filter_ghost_regions` + `fuse_detections` (real production
/// fns). Deterministic, model-free.
///
/// `seed` is passed to `filter_ghost_regions` (anchor freezing).
#[must_use]
pub fn run_perception_faithfulness(fixture: &Fixture, seed: u64) -> PerceptionFaithfulnessReport {
    // Build the combined element list: raw_detections + vlm_claims.
    let raw_detections: Vec<LiteralElement> = fixture
        .predicted
        .raw_detections
        .iter()
        .map(|d| LiteralElement {
            label: d.label.clone(),
            bbox: d.bbox,
            source: parse_source(&d.source),
            confidence: d.confidence,
        })
        .collect();

    let vlm_claims: Vec<LiteralElement> = fixture
        .predicted
        .vlm_claims
        .iter()
        .map(|d| LiteralElement {
            label: d.label.clone(),
            bbox: d.bbox,
            source: parse_source(&d.source),
            confidence: d.confidence,
        })
        .collect();

    // Collect VLM claim labels for uncited_claims metric.
    let vlm_claim_labels: Vec<&str> = vlm_claims.iter().map(|e| e.label.as_str()).collect();

    // Combined set for filter_ghost_regions.
    let mut combined = raw_detections.clone();
    combined.extend(vlm_claims.clone());

    let raw_len = combined.len() as u32;

    // Metric 5: filter_ghost_regions → partition into kept + unverified.
    let prebind = filter_ghost_regions(&combined, seed);

    // uncited_claims = VLM claims that made it into `kept` (ghosts that were
    // NOT caught). A clean fixture routes all VLM ghosts to `unverified`.
    let uncited_count = prebind
        .kept
        .iter()
        .filter(|e| e.source == ElementSource::Vlm)
        .filter(|e| vlm_claim_labels.contains(&e.label.as_str()))
        .count() as u32;

    // Metric 3: fuse_detections on the kept set.
    let fusion = fuse_detections(&prebind.kept, &FusionConfig::default());
    let duplicate_drops = fusion.duplicate_drops;
    let duplicate_region_rate = if raw_len > 0 {
        f64::from(duplicate_drops) / f64::from(raw_len)
    } else {
        0.0
    };

    // Fused regions for metrics 1, 2, 4.
    let fused = &fusion.regions;

    // Metric 1: ocr_exact_match — exact label match on GT text regions.
    let gt_regions = &fixture.ground_truth.regions;
    let n_gt = gt_regions.len();
    let mut exact_matches = 0usize;
    for gt in gt_regions {
        if fused.iter().any(|f| f.label == gt.label) {
            exact_matches += 1;
        }
    }
    let ocr_exact_match = if n_gt > 0 {
        exact_matches as f64 / n_gt as f64
    } else {
        1.0
    };

    // Metric 2: region_grounding_recall_iou05 — recall @ IoU 0.5.
    let mut grounded = 0usize;
    for gt in gt_regions {
        if fused.iter().any(|f| iou(&f.bbox, &gt.bbox) >= 0.5) {
            grounded += 1;
        }
    }
    let region_grounding_recall = if n_gt > 0 {
        grounded as f64 / n_gt as f64
    } else {
        1.0
    };

    // Metric 4: exact_count_accuracy for fixtures with <= 10 separated elements.
    let gt_count = fixture.ground_truth.count;
    let predicted_count = fused.len() as u32;
    let exact_count_accuracy = if gt_count == predicted_count {
        1.0
    } else if gt_count > 0 {
        let diff = (gt_count as i64 - predicted_count as i64).unsigned_abs() as f64;
        1.0 - (diff / f64::from(gt_count))
    } else {
        0.0
    };

    // Metric 6: blind_llm_qa_lift from fixture JSON (no live model call).
    let blind_llm_qa_lift = fixture.qa.blind_llm_score - fixture.qa.ocr_only_baseline_score;

    // Build per-metric results with pass/fail.
    let ocr_result = MetricResult {
        name: "ocr_exact_match".to_string(),
        value: ocr_exact_match,
        pass: ocr_exact_match >= OCR_EXACT_MATCH_FLOOR,
    };
    let grounding_result = MetricResult {
        name: "region_grounding_recall_iou05".to_string(),
        value: region_grounding_recall,
        pass: region_grounding_recall >= REGION_GROUNDING_RECALL_IOU05_FLOOR,
    };
    let dup_result = MetricResult {
        name: "duplicate_region_rate".to_string(),
        value: duplicate_region_rate,
        pass: duplicate_region_rate < DUPLICATE_REGION_RATE_CEILING,
    };
    let count_result = MetricResult {
        name: "exact_count_accuracy".to_string(),
        value: exact_count_accuracy,
        pass: exact_count_accuracy >= EXACT_COUNT_ACCURACY_FLOOR,
    };
    let uncited_result = MetricResult {
        name: "uncited_claims".to_string(),
        value: f64::from(uncited_count),
        pass: uncited_count == 0,
    };
    let lift_result = MetricResult {
        name: "blind_llm_qa_lift".to_string(),
        value: blind_llm_qa_lift,
        pass: blind_llm_qa_lift > 0.0,
    };

    let overall_pass = ocr_result.pass
        && grounding_result.pass
        && dup_result.pass
        && count_result.pass
        && uncited_result.pass
        && lift_result.pass;

    info!(
        fixture_id = %fixture.id,
        overall_pass,
        ocr_exact_match,
        region_grounding_recall,
        duplicate_region_rate,
        exact_count_accuracy,
        uncited_count,
        blind_llm_qa_lift,
        "perception-faithfulness: 6-metric harness complete",
    );

    PerceptionFaithfulnessReport {
        fixture_id: fixture.id.clone(),
        ocr_exact_match: ocr_result,
        region_grounding_recall_iou05: grounding_result,
        duplicate_region_rate: dup_result,
        exact_count_accuracy: count_result,
        uncited_claims: uncited_result,
        blind_llm_qa_lift: lift_result,
        overall_pass,
    }
}

/// Load a fixture from a JSON file and score it.
///
/// This is the primary entry point for CLI callers. Reads the fixture JSON,
/// deserializes it, and delegates to [`run_perception_faithfulness`].
pub fn run_perception_faithfulness_on_fixture(
    path: &Path,
) -> Result<PerceptionFaithfulnessReport, PerceptionFaithfulnessError> {
    let bytes = std::fs::read(path).map_err(|e| PerceptionFaithfulnessError::FixtureRead {
        path: path.display().to_string(),
        source_msg: e.to_string(),
    })?;
    let fixture: Fixture = serde_json::from_slice(&bytes).map_err(|e| {
        PerceptionFaithfulnessError::FixtureParse {
            source_msg: e.to_string(),
        }
    })?;
    Ok(run_perception_faithfulness(&fixture, 0))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_JSON: &str = include_str!("../../../../data/perception/fixtures/j77_6metric_fixture.json");

    fn load_fixture() -> Fixture {
        serde_json::from_str(FIXTURE_JSON).expect("fixture must parse")
    }

    // (1) labelled fixture PASSES all 6 metrics.
    #[test]
    fn labelled_fixture_passes_all_six() {
        let fixture = load_fixture();
        let report = run_perception_faithfulness(&fixture, 0);
        assert!(report.overall_pass, "fixture must PASS all 6 metrics as shipped: {report:?}");
        assert!(report.ocr_exact_match.pass, "ocr_exact_match must pass");
        assert!(
            report.region_grounding_recall_iou05.pass,
            "region_grounding_recall must pass"
        );
        assert!(report.duplicate_region_rate.pass, "duplicate_region_rate must pass");
        assert!(report.exact_count_accuracy.pass, "exact_count_accuracy must pass");
        assert!(report.uncited_claims.pass, "uncited_claims must pass (== 0)");
        assert!(report.blind_llm_qa_lift.pass, "blind_llm_qa_lift must pass (> 0)");
    }

    // (2) FALSIFICATION: add many exact duplicates → duplicate_rate high → FAIL.
    #[test]
    fn falsification_high_duplicate_rate_fails() {
        let mut fixture = load_fixture();
        // Add 10 exact duplicates of the "OK" detection to spike duplicate_rate.
        for _ in 0..10 {
            fixture.predicted.raw_detections.push(PredictedDetection {
                label: "OK".to_string(),
                bbox: [10, 10, 100, 40],
                source: "Ocr".to_string(),
                confidence: 1.0,
            });
        }
        let report = run_perception_faithfulness(&fixture, 0);
        assert!(
            !report.duplicate_region_rate.pass,
            "high duplicate rate must FAIL: got {}",
            report.duplicate_region_rate.value
        );
        assert!(!report.overall_pass, "overall must FAIL on high duplicate rate");
    }

    // (2b) FALSIFICATION: VLM ghost label matches a GT label → uncited > 0 → FAIL.
    #[test]
    fn falsification_uncited_vlm_claim_fails() {
        let mut fixture = load_fixture();
        // Change the VLM ghost's label to match a GT label → it stays in kept.
        fixture.predicted.vlm_claims[0].label = "OK".to_string();
        let report = run_perception_faithfulness(&fixture, 0);
        assert!(
            !report.uncited_claims.pass,
            "uncited VLM claim in kept must FAIL: got {}",
            report.uncited_claims.value
        );
        assert!(!report.overall_pass, "overall must FAIL on uncited claims");
    }

    // (3) FALSIFICATION: drop a GT region from predictions → grounding recall fails.
    #[test]
    fn falsification_dropped_gt_region_fails() {
        let mut fixture = load_fixture();
        // Remove the "Submit" detection so it's missing from predictions.
        fixture
            .predicted
            .raw_detections
            .retain(|d| d.label != "Submit");
        let report = run_perception_faithfulness(&fixture, 0);
        assert!(
            !report.region_grounding_recall_iou05.pass,
            "dropped GT region must FAIL grounding recall: got {}",
            report.region_grounding_recall_iou05.value
        );
        assert!(!report.overall_pass, "overall must FAIL on dropped GT region");
    }

    // (4) run_perception_faithfulness_on_fixture loads from disk.
    #[test]
    fn on_fixture_loads_and_passes() {
        let path = Path::new(".elai_cc/data/perception/fixtures/j77_6metric_fixture.json");
        let report = run_perception_faithfulness_on_fixture(path)
            .expect("fixture must load and score");
        assert!(report.overall_pass, "on-fixture run must PASS: {report:?}");
    }

    // (5) IoU helper correctness.
    #[test]
    fn iou_identical_boxes() {
        let val = iou(&[0, 0, 10, 10], &[0, 0, 10, 10]);
        assert!((val - 1.0).abs() < 1e-9);
    }

    #[test]
    fn iou_disjoint_boxes() {
        let val = iou(&[0, 0, 10, 10], &[100, 100, 110, 110]);
        assert_eq!(val, 0.0);
    }
}
