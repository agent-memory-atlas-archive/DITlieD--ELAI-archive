//! J-77 S5 I4 — canonical-frame transform ledger.
//!
//! One canonical pixel space at ingest. Every resize/crop/tile records its
//! transform relative to that frame. Boxes detected in a downstream op space
//! are reprojected back to canonical; inconsistent inverse transforms
//! (sx/sy ~ 0 or NaN) are rejected. RRA-style compose shape, NOT a new RRA
//! subsystem.
//!
//! Source plan: `docs/thefinalglance/J-plan/J-77_*` (S5 I4 slice).
//! Production caller: `production_agent_rpc.rs::handle_infer_request`
//! (the perception-gateway scaffold block, when gateway enabled).

use serde::{Deserialize, Serialize};

use super::scaffold::{ElementSource, LiteralElement};

/// One transform op recorded in the ledger. Maps from the parent space into
/// this op's space: `x' = x * sx + tx` (same for y).
///
/// `name` is `"identity"` | `"scale"` | `"crop"` | `"tile"` (informational;
/// the math is fully determined by sx/sy/tx/ty).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformOp {
    pub name: String,
    pub sx: f64,
    pub sy: f64,
    pub tx: f64,
    pub ty: f64,
}

impl TransformOp {
    /// Identity op (no transform).
    #[must_use]
    pub fn identity() -> Self {
        Self {
            name: "identity".to_string(),
            sx: 1.0,
            sy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Uniform scale op centered at the origin: `x' = x * s`.
    #[must_use]
    pub fn scale(s: f64) -> Self {
        Self {
            name: "scale".to_string(),
            sx: s,
            sy: s,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// True when the op is invertible (sx/sy non-zero, finite).
    #[must_use]
    pub fn invertible(&self) -> bool {
        self.sx.abs() > f64::EPSILON
            && self.sy.abs() > f64::EPSILON
            && self.sx.is_finite()
            && self.sy.is_finite()
            && self.tx.is_finite()
            && self.ty.is_finite()
    }

    /// Invert a single point coordinate pair from this op's space back to
    /// the parent space. Returns `None` when the op is not invertible.
    #[must_use]
    fn invert_point(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        if !self.invertible() {
            return None;
        }
        Some(((x - self.tx) / self.sx, (y - self.ty) / self.sy))
    }
}

/// The ledger: canonical frame dimensions + the ordered list of transform
/// ops applied since ingest. Ops compose left-to-right (op[0] maps
/// canonical -> space1, op[1] maps space1 -> space2, ...). Reprojection
/// inverts the chain from the last op back to canonical.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformLedger {
    pub canonical_w: u32,
    pub canonical_h: u32,
    pub ops: Vec<TransformOp>,
}

impl TransformLedger {
    /// Construct an empty ledger anchored to a canonical frame of
    /// `canonical_w` x `canonical_h` pixels.
    #[must_use]
    pub fn new(canonical_w: u32, canonical_h: u32) -> Self {
        Self {
            canonical_w,
            canonical_h,
            ops: Vec::new(),
        }
    }

    /// Record a transform op (appended to the chain).
    pub fn record(&mut self, op: TransformOp) {
        self.ops.push(op);
    }

    /// Compose all ops and map a box `[x0, y0, x1, y1]` in the LAST op space
    /// back to canonical pixel coordinates. Returns `None` when any op in the
    /// chain is not invertible (sx/sy ~ 0 or NaN), or when the reprojected
    /// coordinates are non-finite.
    ///
    /// The returned box is clamped to `[0, canonical_w]` x `[0, canonical_h]`
    /// and rounded to the nearest integer; a box that inverts to negative
    /// width or height (x0 > x1 after rounding) is rejected.
    #[must_use]
    pub fn reproject_to_canonical(&self, bbox: [u32; 4]) -> Option<[u32; 4]> {
        let mut x0 = f64::from(bbox[0]);
        let mut y0 = f64::from(bbox[1]);
        let mut x1 = f64::from(bbox[2]);
        let mut y1 = f64::from(bbox[3]);
        for op in self.ops.iter().rev() {
            let (nx0, ny0) = op.invert_point(x0, y0)?;
            let (nx1, ny1) = op.invert_point(x1, y1)?;
            x0 = nx0;
            y0 = ny0;
            x1 = nx1;
            y1 = ny1;
        }
        if !(x0.is_finite() && y0.is_finite() && x1.is_finite() && y1.is_finite()) {
            return None;
        }
        let cw = f64::from(self.canonical_w);
        let ch = f64::from(self.canonical_h);
        x0 = x0.clamp(0.0, cw);
        y0 = y0.clamp(0.0, ch);
        x1 = x1.clamp(0.0, cw);
        y1 = y1.clamp(0.0, ch);
        let ix0 = x0.round() as u32;
        let iy0 = y0.round() as u32;
        let ix1 = x1.round() as u32;
        let iy1 = y1.round() as u32;
        if ix0 > ix1 || iy0 > iy1 {
            return None;
        }
        Some([ix0, iy0, ix1, iy1])
    }
}

/// Reject elements whose reprojected box fails (non-invertible chain) or
/// leaves canonical bounds badly (inverted / degenerate box). Returns the
/// kept elements and a count of rejected elements.
#[must_use]
pub fn filter_consistent_boxes(
    ledger: &TransformLedger,
    elements: &[LiteralElement],
) -> (Vec<LiteralElement>, u32) {
    let mut kept = Vec::with_capacity(elements.len());
    let mut rejected: u32 = 0;
    for elem in elements {
        match ledger.reproject_to_canonical(elem.bbox) {
            Some(reprojected) => {
                kept.push(LiteralElement {
                    label: elem.label.clone(),
                    bbox: reprojected,
                    source: elem.source,
                    confidence: elem.confidence,
                    box_id: elem.box_id,
                    interactable: elem.interactable,
                });
            }
            None => {
                rejected += 1;
            }
        }
    }
    (kept, rejected)
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
    fn scale_then_reproject_recovers_original() {
        let mut ledger = TransformLedger::new(2000, 2000);
        ledger.record(TransformOp::scale(0.5));
        let recovered = ledger
            .reproject_to_canonical([100, 100, 300, 300])
            .expect("scale must be invertible");
        assert_eq!(recovered, [200, 200, 600, 600]);
    }

    #[test]
    fn identity_reproject_is_identity() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp::identity());
        let recovered = ledger
            .reproject_to_canonical([10, 20, 30, 40])
            .expect("identity must be invertible");
        assert_eq!(recovered, [10, 20, 30, 40]);
    }

    #[test]
    fn inconsistent_sx_zero_rejects() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp {
            name: "scale".to_string(),
            sx: 0.0,
            sy: 1.0,
            tx: 0.0,
            ty: 0.0,
        });
        assert!(ledger.reproject_to_canonical([10, 10, 20, 20]).is_none());
    }

    #[test]
    fn nan_scale_rejects() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp {
            name: "scale".to_string(),
            sx: f64::NAN,
            sy: 1.0,
            tx: 0.0,
            ty: 0.0,
        });
        assert!(ledger.reproject_to_canonical([10, 10, 20, 20]).is_none());
    }

    #[test]
    fn empty_ops_reproject_clamps_to_canonical() {
        let ledger = TransformLedger::new(1000, 1000);
        let recovered = ledger
            .reproject_to_canonical([10, 10, 20, 20])
            .expect("no ops = identity chain");
        assert_eq!(recovered, [10, 10, 20, 20]);
    }

    #[test]
    fn clamp_to_canonical_bounds() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp::scale(0.5));
        let recovered = ledger
            .reproject_to_canonical([600, 600, 1200, 1200])
            .expect("clamped box is valid");
        assert_eq!(recovered, [1000, 1000, 1000, 1000]);
    }

    #[test]
    fn inverted_box_after_reproject_rejects() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp {
            name: "crop".to_string(),
            sx: -1.0,
            sy: 1.0,
            tx: 100.0,
            ty: 0.0,
        });
        assert!(ledger.reproject_to_canonical([10, 10, 20, 20]).is_none());
    }

    #[test]
    fn filter_consistent_boxes_keeps_valid_rejects_bad() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp::scale(0.5));
        let elements = vec![
            elem("a", [100, 100, 300, 300]),
            elem("b", [0, 0, 0, 0]),
        ];
        let (kept, rejected) = filter_consistent_boxes(&ledger, &elements);
        assert_eq!(kept.len(), 2);
        assert_eq!(rejected, 0);
        assert_eq!(kept[0].bbox, [200, 200, 600, 600]);
        assert_eq!(kept[1].bbox, [0, 0, 0, 0]);
    }

    #[test]
    fn filter_consistent_boxes_rejects_non_invertible() {
        let mut ledger = TransformLedger::new(1000, 1000);
        ledger.record(TransformOp {
            name: "scale".to_string(),
            sx: 0.0,
            sy: 1.0,
            tx: 0.0,
            ty: 0.0,
        });
        let elements = vec![elem("a", [10, 10, 20, 20])];
        let (kept, rejected) = filter_consistent_boxes(&ledger, &elements);
        assert!(kept.is_empty());
        assert_eq!(rejected, 1);
    }

    #[test]
    fn compose_two_scales() {
        let mut ledger = TransformLedger::new(4000, 4000);
        ledger.record(TransformOp::scale(0.5));
        ledger.record(TransformOp::scale(0.5));
        let recovered = ledger
            .reproject_to_canonical([100, 100, 300, 300])
            .expect("both scales invertible");
        assert_eq!(recovered, [400, 400, 1200, 1200]);
    }
}
