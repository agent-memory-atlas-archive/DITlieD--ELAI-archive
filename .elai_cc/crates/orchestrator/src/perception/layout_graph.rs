//! J-77 S5 I7 — perception layout graph emitter (DKG shape).
//!
//! Source plan: `docs/thefinalglance/J-plan/` (J-77 slice). Emits the
//! perception layout as a graph: nodes carry coords + z-order, edges encode
//! containment, reading order, and adjacency. Shape compatible with the DKG
//! mental model; does not require a live DKG SQLite write.
//!
//! [`emit_layout_graph`] is WIRED at
//! `production_agent_rpc.rs::handle_infer_request` (the ScaffoldEmitted arm
//! of the perception block), alongside [`check_layout_consistency`].

use serde::{Deserialize, Serialize};

use super::scaffold::{ElementSource, LiteralElement};

/// Containment margin (in normalized 0..=1000 coords). A child is "fully
/// inside" a parent only when it is inset by at least this much on every
/// side, preventing trivial same-bbox containment.
const CONTAIN_MARGIN: u32 = 1;

/// Gap threshold (in normalized 0..=1000 coords) below which two non-
/// overlapping bboxes are considered Adjacent.
const ADJACENT_GAP_THRESHOLD: u32 = 20;

/// Tolerance (in normalized 0..=1000 coords) for the reading-order
/// consistency check: a ReadingOrderNext edge from `a` to `b` is a gross
/// reversal when `b.y0 > a.y0 + READING_ORDER_REVERSAL_TOLERANCE` (i.e. the
/// next element in reading order starts well below the current one — the
/// chain should generally move downward, so a large downward jump is fine,
/// but a large *upward* jump (`b.y0` much less than `a.y0`) is the
/// violation). We flag edges where `b.y0 + tol < a.y0`.
const READING_ORDER_REVERSAL_TOLERANCE: u32 = 50;

/// One node in the layout graph. `id` is the 1-based index of the source
/// `LiteralElement`. `bbox` is `[x0, y0, x1, y1]` in normalized coords.
/// `z_order` is the element's paint order (0 = bottom; later elements
/// paint on top — derived from input order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutNode {
    pub id: u32,
    pub label: String,
    pub bbox: [u32; 4],
    pub z_order: u32,
}

/// Kind of edge between two layout nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutEdgeKind {
    /// `from` spatially contains `to` (child fully inside parent with
    /// margin).
    Contains,
    /// `to` immediately follows `from` in reading order (sorted by
    /// `(y0, x0)`).
    ReadingOrderNext,
    /// `from` and `to` are horizontal/vertical neighbors (gap within
    /// threshold, no overlap). Direction is from the earlier-sorted node
    /// to the later-sorted node.
    Adjacent,
}

/// One edge in the layout graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutEdge {
    pub from: u32,
    pub to: u32,
    pub kind: LayoutEdgeKind,
}

/// The full layout graph: nodes + edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutGraph {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
}

/// Deterministic consistency report for a [`LayoutGraph`].
#[derive(Debug, Clone)]
pub struct LayoutConsistencyReport {
    pub ok: bool,
    pub parent_contains_child_violations: u32,
    pub reading_order_cross_violations: u32,
    /// `true` when `nodes.len()` != the number of source elements the
    /// emitter was given. The emitter always sets this correctly; the
    /// checker re-derives it from the graph alone and flags a mismatch
    /// when node ids are not a dense 1..=n sequence.
    pub count_mismatch: bool,
}

/// Emit a [`LayoutGraph`] from a slice of [`LiteralElement`]s.
///
/// - Node `id` = 1-based index; `z_order` = 0-based index (input order).
/// - `Contains` edges: for every pair (a, b) where a's bbox fully contains
///   b's bbox with at least [`CONTAIN_MARGIN`] inset on every side, emit
///   `Contains { from: a.id, to: b.id }`. A parent is the *smallest* such
///   container (closest bbox by area) to avoid redundant transitive edges.
/// - `ReadingOrderNext` edges: sort node ids by `(y0, x0)` and chain
///   consecutive pairs.
/// - `Adjacent` edges: for every pair of non-overlapping bboxes whose gap
///   (horizontal or vertical) is within [`ADJACENT_GAP_THRESHOLD`], emit
///   `Adjacent { from, to }` with `from` < `to` by id.
#[must_use]
pub fn emit_layout_graph(elements: &[LiteralElement]) -> LayoutGraph {
    let nodes: Vec<LayoutNode> = elements
        .iter()
        .enumerate()
        .map(|(i, e)| LayoutNode {
            id: (i + 1) as u32,
            label: e.label.clone(),
            bbox: e.bbox,
            z_order: i as u32,
        })
        .collect();

    let mut edges: Vec<LayoutEdge> = Vec::new();

    // --- Contains edges (smallest container per child) ---
    for (child_idx, child) in nodes.iter().enumerate() {
        let mut best_parent: Option<usize> = None;
        let mut best_parent_area: u64 = u64::MAX;
        for (parent_idx, parent) in nodes.iter().enumerate() {
            if parent_idx == child_idx {
                continue;
            }
            if bbox_contains(&parent.bbox, &child.bbox) {
                let area = bbox_area(&parent.bbox);
                if area < best_parent_area {
                    best_parent_area = area;
                    best_parent = Some(parent_idx);
                }
            }
        }
        if let Some(pi) = best_parent {
            edges.push(LayoutEdge {
                from: nodes[pi].id,
                to: child.id,
                kind: LayoutEdgeKind::Contains,
            });
        }
    }

    // --- ReadingOrderNext edges (sort by y0, x0; chain consecutive) ---
    let mut order: Vec<usize> = (0..nodes.len()).collect();
    order.sort_by(|&a, &b| {
        nodes[a]
            .bbox[1]
            .cmp(&nodes[b].bbox[1])
            .then_with(|| nodes[a].bbox[0].cmp(&nodes[b].bbox[0]))
    });
    for w in order.windows(2) {
        edges.push(LayoutEdge {
            from: nodes[w[0]].id,
            to: nodes[w[1]].id,
            kind: LayoutEdgeKind::ReadingOrderNext,
        });
    }

    // --- Adjacent edges (non-overlapping, gap within threshold) ---
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let a = &nodes[i].bbox;
            let b = &nodes[j].bbox;
            if !bbox_overlaps(a, b) && adjacent_gap(a, b) <= ADJACENT_GAP_THRESHOLD {
                edges.push(LayoutEdge {
                    from: nodes[i].id,
                    to: nodes[j].id,
                    kind: LayoutEdgeKind::Adjacent,
                });
            }
        }
    }

    LayoutGraph { nodes, edges }
}

/// Deterministic consistency check for a [`LayoutGraph`].
///
/// - `parent_contains_child_violations`: for every `Contains` edge, verify
///   the parent bbox actually contains the child bbox (with margin). Each
///   failing edge increments the counter.
/// - `reading_order_cross_violations`: for every `ReadingOrderNext` edge
///   (a -> b), flag a gross reversal when `b.y0 + tol < a.y0` (the next
///   element in reading order starts well *above* the current one).
/// - `count_mismatch`: `true` when node ids are not a dense `1..=n`
///   sequence (the emitter guarantees this; the checker re-derives it).
/// - `ok`: `true` iff all counters are zero and no count mismatch.
#[must_use]
pub fn check_layout_consistency(g: &LayoutGraph) -> LayoutConsistencyReport {
    let mut parent_contains_child_violations: u32 = 0;
    let mut reading_order_cross_violations: u32 = 0;

    let id_to_bbox: std::collections::HashMap<u32, [u32; 4]> = g
        .nodes
        .iter()
        .map(|n| (n.id, n.bbox))
        .collect();

    for edge in &g.edges {
        match edge.kind {
            LayoutEdgeKind::Contains => {
                let (Some(parent_bbox), Some(child_bbox)) = (
                    id_to_bbox.get(&edge.from).copied(),
                    id_to_bbox.get(&edge.to).copied(),
                ) else {
                    parent_contains_child_violations += 1;
                    continue;
                };
                if !bbox_contains(&parent_bbox, &child_bbox) {
                    parent_contains_child_violations += 1;
                }
            }
            LayoutEdgeKind::ReadingOrderNext => {
                let (Some(from_bbox), Some(to_bbox)) = (
                    id_to_bbox.get(&edge.from).copied(),
                    id_to_bbox.get(&edge.to).copied(),
                ) else {
                    reading_order_cross_violations += 1;
                    continue;
                };
                if to_bbox[1] + READING_ORDER_REVERSAL_TOLERANCE < from_bbox[1] {
                    reading_order_cross_violations += 1;
                }
            }
            LayoutEdgeKind::Adjacent => {}
        }
    }

    let n = g.nodes.len() as u32;
    let count_mismatch = if n == 0 {
        false
    } else {
        let mut ids: Vec<u32> = g.nodes.iter().map(|node| node.id).collect();
        ids.sort_unstable();
        ids.iter().enumerate().any(|(i, &id)| id != (i as u32 + 1))
    };

    let ok = parent_contains_child_violations == 0
        && reading_order_cross_violations == 0
        && !count_mismatch;

    LayoutConsistencyReport {
        ok,
        parent_contains_child_violations,
        reading_order_cross_violations,
        count_mismatch,
    }
}

fn bbox_contains(outer: &[u32; 4], inner: &[u32; 4]) -> bool {
    outer[0] + CONTAIN_MARGIN <= inner[0]
        && outer[1] + CONTAIN_MARGIN <= inner[1]
        && inner[2] + CONTAIN_MARGIN <= outer[2]
        && inner[3] + CONTAIN_MARGIN <= outer[3]
}

fn bbox_area(b: &[u32; 4]) -> u64 {
    let w = (b[2] as u64).saturating_sub(b[0] as u64);
    let h = (b[3] as u64).saturating_sub(b[1] as u64);
    w.saturating_mul(h)
}

fn bbox_overlaps(a: &[u32; 4], b: &[u32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

fn adjacent_gap(a: &[u32; 4], b: &[u32; 4]) -> u32 {
    let h_gap = if a[2] <= b[0] {
        b[0] - a[2]
    } else if b[2] <= a[0] {
        a[0] - b[2]
    } else {
        u32::MAX
    };
    let v_gap = if a[3] <= b[1] {
        b[1] - a[3]
    } else if b[3] <= a[1] {
        a[1] - b[3]
    } else {
        u32::MAX
    };
    if h_gap == u32::MAX && v_gap == u32::MAX {
        u32::MAX
    } else if h_gap == u32::MAX {
        v_gap
    } else if v_gap == u32::MAX {
        h_gap
    } else {
        u32::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(label: &str, bbox: [u32; 4]) -> LiteralElement {
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
    fn empty_elements_emit_empty_graph() {
        let g = emit_layout_graph(&[]);
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
        let r = check_layout_consistency(&g);
        assert!(r.ok);
        assert!(!r.count_mismatch);
    }

    #[test]
    fn single_element_has_no_edges() {
        let g = emit_layout_graph(&[el("a", [10, 10, 100, 100])]);
        assert_eq!(g.nodes.len(), 1);
        assert!(g.edges.is_empty());
        let r = check_layout_consistency(&g);
        assert!(r.ok);
    }

    #[test]
    fn contain_edge_emitted_for_nested_bbox() {
        let elements = vec![
            el("outer", [0, 0, 500, 500]),
            el("inner", [50, 50, 100, 100]),
        ];
        let g = emit_layout_graph(&elements);
        let contains: Vec<&LayoutEdge> = g
            .edges
            .iter()
            .filter(|e| matches!(e.kind, LayoutEdgeKind::Contains))
            .collect();
        assert_eq!(contains.len(), 1);
        assert_eq!(contains[0].from, 1);
        assert_eq!(contains[0].to, 2);
        let r = check_layout_consistency(&g);
        assert!(r.ok);
        assert_eq!(r.parent_contains_child_violations, 0);
    }

    #[test]
    fn contain_violation_detected_by_checker() {
        let g = LayoutGraph {
            nodes: vec![
                LayoutNode {
                    id: 1,
                    label: "a".into(),
                    bbox: [0, 0, 100, 100],
                    z_order: 0,
                },
                LayoutNode {
                    id: 2,
                    label: "b".into(),
                    bbox: [200, 200, 300, 300],
                    z_order: 1,
                },
            ],
            edges: vec![LayoutEdge {
                from: 1,
                to: 2,
                kind: LayoutEdgeKind::Contains,
            }],
        };
        let r = check_layout_consistency(&g);
        assert!(!r.ok);
        assert_eq!(r.parent_contains_child_violations, 1);
    }

    #[test]
    fn reading_order_chain_sorted_by_y0_x0() {
        let elements = vec![
            el("a", [100, 200, 200, 300]),
            el("b", [100, 0, 200, 100]),
        ];
        let g = emit_layout_graph(&elements);
        let ro: Vec<&LayoutEdge> = g
            .edges
            .iter()
            .filter(|e| matches!(e.kind, LayoutEdgeKind::ReadingOrderNext))
            .collect();
        assert_eq!(ro.len(), 1);
        assert_eq!(ro[0].from, 2);
        assert_eq!(ro[0].to, 1);
        let r = check_layout_consistency(&g);
        assert!(r.ok);
        assert_eq!(r.reading_order_cross_violations, 0);
    }

    #[test]
    fn reading_order_gross_reversal_flagged() {
        let g = LayoutGraph {
            nodes: vec![
                LayoutNode {
                    id: 1,
                    label: "low".into(),
                    bbox: [0, 500, 100, 600],
                    z_order: 0,
                },
                LayoutNode {
                    id: 2,
                    label: "high".into(),
                    bbox: [0, 0, 100, 100],
                    z_order: 1,
                },
            ],
            edges: vec![LayoutEdge {
                from: 1,
                to: 2,
                kind: LayoutEdgeKind::ReadingOrderNext,
            }],
        };
        let r = check_layout_consistency(&g);
        assert!(!r.ok);
        assert_eq!(r.reading_order_cross_violations, 1);
    }

    #[test]
    fn count_mismatch_detected() {
        let g = LayoutGraph {
            nodes: vec![
                LayoutNode {
                    id: 1,
                    label: "a".into(),
                    bbox: [0, 0, 10, 10],
                    z_order: 0,
                },
                LayoutNode {
                    id: 3,
                    label: "b".into(),
                    bbox: [20, 20, 30, 30],
                    z_order: 1,
                },
            ],
            edges: vec![],
        };
        let r = check_layout_consistency(&g);
        assert!(!r.ok);
        assert!(r.count_mismatch);
    }

    #[test]
    fn adjacent_edge_for_close_non_overlapping_bboxes() {
        let elements = vec![
            el("left", [0, 0, 100, 100]),
            el("right", [110, 0, 200, 100]),
        ];
        let g = emit_layout_graph(&elements);
        let adj: Vec<&LayoutEdge> = g
            .edges
            .iter()
            .filter(|e| matches!(e.kind, LayoutEdgeKind::Adjacent))
            .collect();
        assert_eq!(adj.len(), 1);
        assert_eq!(adj[0].from, 1);
        assert_eq!(adj[0].to, 2);
    }

    #[test]
    fn z_order_derived_from_input_order() {
        let elements = vec![
            el("first", [0, 0, 50, 50]),
            el("second", [60, 60, 100, 100]),
        ];
        let g = emit_layout_graph(&elements);
        assert_eq!(g.nodes[0].z_order, 0);
        assert_eq!(g.nodes[1].z_order, 1);
    }
}
