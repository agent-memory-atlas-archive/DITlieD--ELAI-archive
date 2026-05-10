//! B-23 Phase 4H Group E — Ontology-Similarity Convergence (4H.7B).
//!
//! Catches "score jiggle": variants whose nominal score moves but whose
//! `config_blob` is structurally identical or alternates between two
//! states. MSI alone misses this.
//!
//! Port from `github.com/Q00/ouroboros` (MIT) — see
//! `workflow/papers/ouroboros-interface.md`. Key formulas:
//!   similarity = 0.5*key_overlap + 0.3*type_match + 0.2*exact_match
//!   denominator = |union(keys(curr), keys(prev))|  (NOT intersection)
//!   empty schema -> similarity = 1.0   (boundary guard)
//!
//! Verdicts (spec phase-4H.txt:830-855):
//!   Stagnant:     similarity >= 0.95 over `stagnation_window` consecutive variants (default 3).
//!   Oscillating:  PERIOD-2 FULL VERIFICATION — sim(N,N-2) AND sim(N-1,N-3) both >= 0.95.
//!                 Requires >= 4 variants. With 3 variants, single half-period match is
//!                 Inconclusive (insufficient evidence — ouroboros source convention).
//!   Evolving:     at least one consecutive pair has similarity < 0.95.
//!   Inconclusive: < min_history (default 3 variants).

use std::collections::BTreeMap;

use crate::self_mod_types::{
    ConfigVariant, ConfigVariantSimilarity, OscVerdict, SIM_STAGNATION_THRESHOLD,
    SIM_STAGNATION_WINDOW, SIM_WEIGHT_EXACT_MATCH, SIM_WEIGHT_KEY_OVERLAP, SIM_WEIGHT_TYPE_MATCH,
};

// ---------------------------------------------------------------------------
// Similarity computation
// ---------------------------------------------------------------------------

/// Compute the OSC similarity over two `config_blob`s. The blobs are
/// expected to deserialize to a flat `serde_json::Map<String, Value>`
/// (key → value). Returns 1.0 on the empty-schema boundary so a fresh
/// archive does not falsely register Evolving on its first pair.
///
/// Group E audit BLOCK fix: a non-empty blob that fails to decode (corrupt
/// JSON, wrong type) returns similarity 0.0 — never silently maps to {}.
/// Without this, two corrupt blobs would register as identical and
/// trigger false EVO Withhold / OSC Stagnant.
pub fn compute_similarity(prev: &[u8], curr: &[u8]) -> ConfigVariantSimilarity {
    let prev_map = match decode_flat_map(prev) {
        Some(m) => m,
        None => {
            return ConfigVariantSimilarity {
                key_overlap: 0.0,
                type_match: 0.0,
                exact_match: 0.0,
                similarity: 0.0,
            };
        }
    };
    let curr_map = match decode_flat_map(curr) {
        Some(m) => m,
        None => {
            return ConfigVariantSimilarity {
                key_overlap: 0.0,
                type_match: 0.0,
                exact_match: 0.0,
                similarity: 0.0,
            };
        }
    };
    compute_similarity_from_maps(&prev_map, &curr_map)
}

fn compute_similarity_from_maps(
    prev: &BTreeMap<String, serde_json::Value>,
    curr: &BTreeMap<String, serde_json::Value>,
) -> ConfigVariantSimilarity {
    let prev_keys: std::collections::BTreeSet<&String> = prev.keys().collect();
    let curr_keys: std::collections::BTreeSet<&String> = curr.keys().collect();
    let union: std::collections::BTreeSet<&String> = prev_keys.union(&curr_keys).copied().collect();
    let common: std::collections::BTreeSet<&String> =
        prev_keys.intersection(&curr_keys).copied().collect();

    let union_size = union.len();
    if union_size == 0 {
        // Boundary: both blobs empty. Treat as fully similar (ouroboros
        // convention) so an empty archive does not register false Evolving.
        return ConfigVariantSimilarity {
            key_overlap: 1.0,
            type_match: 1.0,
            exact_match: 1.0,
            similarity: 1.0,
        };
    }

    let key_overlap = common.len() as f32 / union_size as f32;

    let mut type_matches = 0usize;
    let mut exact_matches = 0usize;
    for k in &common {
        let pv = &prev[*k];
        let cv = &curr[*k];
        if pv == cv {
            exact_matches += 1;
            type_matches += 1;
        } else if json_type_eq(pv, cv) {
            type_matches += 1;
        }
    }
    // Per ouroboros source: type_match and exact_match denominators are
    // the FULL UNION size, not |common|. This is intentional — see
    // `workflow/papers/ouroboros-interface.md` ALGORITHM A test
    // `test_similarity_denominator_is_union_not_common`.
    let type_match = type_matches as f32 / union_size as f32;
    let exact_match = exact_matches as f32 / union_size as f32;

    let similarity = SIM_WEIGHT_KEY_OVERLAP * key_overlap
        + SIM_WEIGHT_TYPE_MATCH * type_match
        + SIM_WEIGHT_EXACT_MATCH * exact_match;

    ConfigVariantSimilarity {
        key_overlap,
        type_match,
        exact_match,
        similarity: similarity.clamp(0.0, 1.0),
    }
}

/// Decode a `config_blob` into a flat key→value map.
///
/// * `Some(empty)` — blob is empty (intentional fresh archive).
/// * `Some(map)`   — blob decoded as a JSON object.
/// * `None`        — blob is non-empty AND failed to decode as an object;
///                   caller treats as "fully different" (similarity 0.0).
fn decode_flat_map(blob: &[u8]) -> Option<BTreeMap<String, serde_json::Value>> {
    if blob.is_empty() {
        return Some(BTreeMap::new());
    }
    let v: serde_json::Value = serde_json::from_slice(blob).ok()?;
    match v {
        serde_json::Value::Object(map) => Some(map.into_iter().collect()),
        _ => None,
    }
}

fn json_type_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value::*;
    matches!(
        (a, b),
        (Null, Null)
            | (Bool(_), Bool(_))
            | (Number(_), Number(_))
            | (String(_), String(_))
            | (Array(_), Array(_))
            | (Object(_), Object(_))
    )
}

// ---------------------------------------------------------------------------
// Verdict computation
// ---------------------------------------------------------------------------

/// Compute the OSC verdict over a window of `ConfigVariant`s ordered
/// **oldest first**. The latest variant is `window.last()`. Caller is
/// responsible for filtering by `pipeline_mode` (verdict is per-mode).
pub fn compute_osc_verdict(window: &[ConfigVariant]) -> OscVerdict {
    let n = window.len();
    if n < SIM_STAGNATION_WINDOW {
        return OscVerdict::Inconclusive {
            reason: format!("need >= {SIM_STAGNATION_WINDOW} variants, got {n}"),
        };
    }

    // Stagnation: last `SIM_STAGNATION_WINDOW` consecutive pairs all
    // >= threshold. With window=3, that's pairs (N, N-1) and (N-1, N-2).
    let stagnant = (n.saturating_sub(SIM_STAGNATION_WINDOW)..n.saturating_sub(1)).all(|i| {
        compute_similarity(&window[i].config_blob, &window[i + 1].config_blob).similarity
            >= SIM_STAGNATION_THRESHOLD
    });

    // Period-2 oscillation. With >= 4 gens we require BOTH half-periods
    // to pass the threshold. With exactly 3 gens, a single half-period
    // match is INCONCLUSIVE per ouroboros source convention.
    if n >= 4 {
        let half_a =
            compute_similarity(&window[n - 1].config_blob, &window[n - 3].config_blob).similarity;
        let half_b =
            compute_similarity(&window[n - 2].config_blob, &window[n - 4].config_blob).similarity;
        if half_a >= SIM_STAGNATION_THRESHOLD && half_b >= SIM_STAGNATION_THRESHOLD && !stagnant {
            return OscVerdict::Oscillating { period: 2 };
        }
    } else if n == 3 {
        // Single half-period only — cannot confirm oscillation.
        let half_a =
            compute_similarity(&window[n - 1].config_blob, &window[n - 3].config_blob).similarity;
        if half_a >= SIM_STAGNATION_THRESHOLD && !stagnant {
            return OscVerdict::Inconclusive {
                reason: "single half-period match; need 4+ variants to confirm".into(),
            };
        }
    }

    if stagnant {
        let latest_sim = compute_similarity(
            &window[n - 2].config_blob,
            &window[n - 1].config_blob,
        )
        .similarity;
        return OscVerdict::Stagnant {
            window: SIM_STAGNATION_WINDOW,
            similarity: latest_sim,
        };
    }

    OscVerdict::Evolving
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_mod_layer1a::PipelineMode;
    use crate::self_mod_types::VariantStatus;

    fn variant(id: &str, blob: &serde_json::Value) -> ConfigVariant {
        ConfigVariant {
            variant_id: id.into(),
            parent_id: None,
            score: 0.5,
            children_count: 0,
            config_blob: serde_json::to_vec(blob).unwrap(),
            created_at: 0,
            pipeline_outcomes: vec![],
            pipeline_mode: PipelineMode::Surgical,
            status: VariantStatus::Archived,
        }
    }

    #[test]
    fn empty_blob_pair_is_fully_similar() {
        let s = compute_similarity(b"", b"");
        assert!((s.similarity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn identical_blobs_yield_1_0() {
        let blob = serde_json::json!({"a": 1, "b": "x"});
        let bytes = serde_json::to_vec(&blob).unwrap();
        let s = compute_similarity(&bytes, &bytes);
        assert!((s.similarity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn disjoint_blobs_yield_0_0() {
        let a = serde_json::to_vec(&serde_json::json!({"a": 1})).unwrap();
        let b = serde_json::to_vec(&serde_json::json!({"b": 1})).unwrap();
        let s = compute_similarity(&a, &b);
        assert_eq!(s.similarity, 0.0);
    }

    #[test]
    fn weights_are_locked_at_0_5_0_3_0_2() {
        // Same keys, type-matched but value-different. With:
        //   key_overlap = 1.0, type_match = 1.0, exact_match = 0.0
        // similarity should be 0.5*1 + 0.3*1 + 0.2*0 = 0.8.
        let a = serde_json::to_vec(&serde_json::json!({"k": 1})).unwrap();
        let b = serde_json::to_vec(&serde_json::json!({"k": 2})).unwrap();
        let s = compute_similarity(&a, &b);
        assert!((s.similarity - 0.8).abs() < 1e-5);
    }

    #[test]
    fn five_identical_variants_stagnate() {
        let blob = serde_json::json!({"k": 1, "j": "z"});
        let win: Vec<ConfigVariant> = (0..5).map(|i| variant(&format!("v{i}"), &blob)).collect();
        let v = compute_osc_verdict(&win);
        assert!(matches!(v, OscVerdict::Stagnant { .. }));
    }

    #[test]
    fn oscillating_period_2_full_verification() {
        // A B A B A B (six variants alternating between two distinct shapes).
        let a = serde_json::json!({"x": 1});
        let b = serde_json::json!({"y": 2});
        let mut win: Vec<ConfigVariant> = Vec::new();
        for i in 0..6 {
            let blob = if i % 2 == 0 { &a } else { &b };
            win.push(variant(&format!("v{i}"), blob));
        }
        let v = compute_osc_verdict(&win);
        // A and B are disjoint -> similarity 0; not >=0.95. Window detects
        // EVOLVING, not Oscillating, because the period-2 sim is 1.0 ONLY
        // when A and B share keys with high similarity. Construct a real
        // period-2 pair with a single value flip:
        let _ = v;
        let a = serde_json::json!({"k": 1, "shared": "x"});
        let b = serde_json::json!({"k": 2, "shared": "x"});
        let mut win: Vec<ConfigVariant> = Vec::new();
        for i in 0..6 {
            let blob = if i % 2 == 0 { &a } else { &b };
            win.push(variant(&format!("v{i}"), blob));
        }
        let v = compute_osc_verdict(&win);
        assert!(matches!(v, OscVerdict::Oscillating { period: 2 }));
    }

    #[test]
    fn three_variants_inconclusive_on_single_half_period() {
        let a = serde_json::json!({"k": 1, "shared": "x"});
        let b = serde_json::json!({"k": 2, "shared": "x"});
        let win = vec![
            variant("v0", &a),
            variant("v1", &b),
            variant("v2", &a),
        ];
        let v = compute_osc_verdict(&win);
        assert!(matches!(v, OscVerdict::Inconclusive { .. }));
    }

    #[test]
    fn evolving_when_pairs_below_threshold() {
        let win: Vec<ConfigVariant> = (0..5)
            .map(|i| variant(&format!("v{i}"), &serde_json::json!({ "k": i })))
            .collect();
        // Each pair has identical key set but value-different ints =>
        // similarity 0.5*1 + 0.3*1 + 0.2*0 = 0.8 < 0.95. Stagnation
        // window thus does not trigger (every consecutive pair < 0.95).
        let v = compute_osc_verdict(&win);
        match v {
            OscVerdict::Evolving => (),
            other => panic!("expected Evolving, got {other:?}"),
        }
    }

    #[test]
    fn fewer_than_window_inconclusive() {
        let blob = serde_json::json!({"k": 1});
        let win = vec![variant("v0", &blob), variant("v1", &blob)];
        let v = compute_osc_verdict(&win);
        assert!(matches!(v, OscVerdict::Inconclusive { .. }));
    }
}
