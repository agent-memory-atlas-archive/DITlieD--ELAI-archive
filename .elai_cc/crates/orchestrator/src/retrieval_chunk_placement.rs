//! J-76 C6 — chunk-count cap + head/tail placement policy knobs.
//!
//! Extracted from `context_calibration` so that file stays under the 1000-line
//! maintainability ceiling. The symbols below are re-exported from
//! `context_calibration` via `lib.rs` for existing callers.

/// Default retrieval chunk cap when no measured curve supplies a tighter bound.
/// Conservative; not a frontier-model number (ID-5).
pub const DEFAULT_RETRIEVAL_CHUNK_CAP: usize = 8;

/// Derive a chunk cap from an optional fill-degradation curve knee.
/// - `Some(knee)` with `knee_fill_fraction` in `(0.0, 1.0]` =>
///   `max(3, floor(DEFAULT * knee))` style conservative clamp.
/// - `None` => `DEFAULT_RETRIEVAL_CHUNK_CAP`.
///
/// Pure; no I/O. The knee fraction is the fill at which outcome rate drops
/// most steeply (see [`crate::context_calibration::detect_knee`]); a lower
/// knee means the model degrades earlier, so we cap retrieval chunks more
/// tightly.
#[must_use]
pub fn retrieval_chunk_cap(knee_fill_fraction: Option<f64>) -> usize {
    match knee_fill_fraction {
        Some(knee) if knee > 0.0 && knee <= 1.0 => {
            let scaled = (DEFAULT_RETRIEVAL_CHUNK_CAP as f64 * knee).floor() as usize;
            scaled.max(3)
        }
        _ => DEFAULT_RETRIEVAL_CHUNK_CAP,
    }
}

/// Cap ranked retrieval results to `max`, then place for lost-in-the-middle
/// mitigation:
/// 1. sort by score DESC (stable on `tie_id`)
/// 2. truncate to `max`
/// 3. head/tail pack: best -> index 0, 2nd-best -> last, 3rd -> index 1,
///    4th -> last-1, ... so top chunks sit at HEAD and TAIL, never middle-only.
///
/// Pure; no I/O. The `score` and `tie_id` closures let the caller project
/// `RetrievalResult` (or any ranked type) without a trait bound.
pub fn cap_and_place_retrieval_chunks<T>(
    chunks: Vec<T>,
    max: usize,
    score: impl Fn(&T) -> f64,
    tie_id: impl Fn(&T) -> &str,
) -> Vec<T> {
    if chunks.is_empty() || max == 0 {
        return Vec::new();
    }
    // 1. Sort by score DESC, stable on tie_id (chunk_id) for determinism.
    let mut indexed: Vec<(usize, T)> = chunks.into_iter().enumerate().collect();
    indexed.sort_by(|a, b| {
        let sa = score(&a.1);
        let sb = score(&b.1);
        sb.partial_cmp(&sa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| tie_id(&a.1).cmp(tie_id(&b.1)))
            .then_with(|| a.0.cmp(&b.0))
    });
    // 2. Truncate to max.
    indexed.truncate(max);
    // 3. Head/tail pack: interleave best into head positions, 2nd-best into
    //    tail, alternating. Build a Vec of slots then fill.
    let n = indexed.len();
    let mut placed: Vec<Option<T>> = (0..n).map(|_| None).collect();
    let mut head = 0usize;
    let mut tail = n; // exclusive end; we fill tail-1 then decrement
    let mut to_head = true;
    for (_, chunk) in indexed {
        if to_head {
            placed[head] = Some(chunk);
            head += 1;
        } else {
            tail -= 1;
            placed[tail] = Some(chunk);
        }
        to_head = !to_head;
    }
    placed.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::{
        cap_and_place_retrieval_chunks, retrieval_chunk_cap, DEFAULT_RETRIEVAL_CHUNK_CAP,
    };

    #[test]
    fn j76_c6_retrieval_chunk_cap_none_is_default() {
        assert_eq!(retrieval_chunk_cap(None), DEFAULT_RETRIEVAL_CHUNK_CAP);
    }

    #[test]
    fn j76_c6_retrieval_chunk_cap_with_knee_clamps_to_min_three() {
        // knee 0.1 => floor(8*0.1)=0 => clamped to 3.
        assert_eq!(retrieval_chunk_cap(Some(0.1)), 3);
    }

    #[test]
    fn j76_c6_retrieval_chunk_cap_with_knee_scales() {
        // knee 0.5 => floor(8*0.5)=4.
        assert_eq!(retrieval_chunk_cap(Some(0.5)), 4);
        // knee 1.0 => floor(8*1.0)=8.
        assert_eq!(retrieval_chunk_cap(Some(1.0)), 8);
    }

    #[test]
    fn j76_c6_retrieval_chunk_cap_rejects_out_of_range_knee() {
        // knee <= 0 or > 1 falls back to default (no pseudo-calibration).
        assert_eq!(retrieval_chunk_cap(Some(0.0)), DEFAULT_RETRIEVAL_CHUNK_CAP);
        assert_eq!(retrieval_chunk_cap(Some(-0.1)), DEFAULT_RETRIEVAL_CHUNK_CAP);
        assert_eq!(retrieval_chunk_cap(Some(1.5)), DEFAULT_RETRIEVAL_CHUNK_CAP);
    }

    #[derive(Debug, Clone)]
    struct Chunk {
        id: &'static str,
        score: f64,
    }

    #[test]
    fn j76_c6_cap_truncates_to_max_and_empty_stays_empty() {
        let chunks: Vec<Chunk> = Vec::new();
        let out = cap_and_place_retrieval_chunks(chunks, 5, |c| c.score, |c| c.id);
        assert!(out.is_empty(), "empty in -> empty out");

        let chunks: Vec<Chunk> = (0..10)
            .map(|i| Chunk { id: id_for(i), score: 1.0 - (i as f64) * 0.1 })
            .collect();
        let out = cap_and_place_retrieval_chunks(chunks, 5, |c| c.score, |c| c.id);
        assert_eq!(out.len(), 5, "truncated to max");
    }

    #[test]
    fn j76_c6_placement_head_tail_pack_order() {
        // scores [0.9,0.8,0.7,0.6,0.5] with max=5.
        // Sorted DESC: 0.9,0.8,0.7,0.6,0.5.
        // Head/tail pack: 0.9->idx0, 0.8->last(idx4), 0.7->idx1,
        //                 0.6->idx3, 0.5->idx2.
        // => [0.9, 0.7, 0.5, 0.6, 0.8]
        let chunks: Vec<Chunk> = vec![
            Chunk { id: "c0", score: 0.9 },
            Chunk { id: "c1", score: 0.8 },
            Chunk { id: "c2", score: 0.7 },
            Chunk { id: "c3", score: 0.6 },
            Chunk { id: "c4", score: 0.5 },
        ];
        let out = cap_and_place_retrieval_chunks(chunks, 5, |c| c.score, |c| c.id);
        assert_eq!(out.len(), 5);
        let scores: Vec<f64> = out.iter().map(|c| c.score).collect();
        assert_eq!(scores, vec![0.9, 0.7, 0.5, 0.6, 0.8]);
        // Head is best, tail is 2nd-best.
        assert!((out[0].score - 0.9).abs() < 1e-9);
        assert!((out[4].score - 0.8).abs() < 1e-9);
    }

    #[test]
    fn j76_c6_placement_stable_tie_on_chunk_id() {
        // Equal scores: tie breaks on chunk_id ASC then original index.
        let chunks: Vec<Chunk> = vec![
            Chunk { id: "z", score: 0.5 },
            Chunk { id: "a", score: 0.5 },
            Chunk { id: "m", score: 0.5 },
        ];
        let out = cap_and_place_retrieval_chunks(chunks, 3, |c| c.score, |c| c.id);
        // Sorted: a(0.5), m(0.5), z(0.5) by chunk_id.
        // Pack: a->idx0, m->last(idx2), z->idx1 => [a, z, m]
        assert_eq!(out[0].id, "a");
        assert_eq!(out[1].id, "z");
        assert_eq!(out[2].id, "m");
    }

    #[test]
    fn j76_c6_placement_max_zero_returns_empty() {
        let chunks: Vec<Chunk> = vec![Chunk { id: "c0", score: 0.9 }];
        let out = cap_and_place_retrieval_chunks(chunks, 0, |c| c.score, |c| c.id);
        assert!(out.is_empty());
    }

    #[test]
    fn j76_c6_placement_single_chunk() {
        let chunks: Vec<Chunk> = vec![Chunk { id: "c0", score: 0.9 }];
        let out = cap_and_place_retrieval_chunks(chunks, 5, |c| c.score, |c| c.id);
        assert_eq!(out.len(), 1);
        assert!((out[0].score - 0.9).abs() < 1e-9);
    }

    fn id_for(i: usize) -> &'static str {
        // Small static id table for the truncate test.
        const IDS: [&str; 10] = [
            "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9",
        ];
        IDS[i]
    }
}
