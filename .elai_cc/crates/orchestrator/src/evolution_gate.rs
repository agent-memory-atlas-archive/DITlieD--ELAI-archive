//! B-23 Phase 4H Group E — Evolution Gate (4H.7D).
//!
//! Catches "fake convergence" claims when consecutive `ConfigVariant`s are
//! byte-identical despite differing scores. Without EVO, MSI can read
//! IMPROVING from pure score noise on a frozen config.
//!
//! Port from `github.com/Q00/ouroboros` (MIT,
//! convergence.py:156-171, _count_evolved_generations + withholding clause).
//!
//! Compute (per PipelineMode):
//!   window = last N variants in mode (default N = MSI window = 10)
//!   evolved_count = pairs (window[i], window[i-1]) with similarity < 0.95
//!   latest_similarity = sim(window[-1], window[-2])  (1.0 when len < 2)
//!
//!   if evolved_count == 0 AND latest_similarity >= 0.95 -> Withhold
//!     (forces MSI verdict to STABILIZING for the mode)
//!   else -> Allow

use crate::self_mod_types::{ConfigVariant, EvolutionGateResult, SIM_STAGNATION_THRESHOLD};

use crate::osc::compute_similarity;

/// Compute the EVO gate over a window of `ConfigVariant`s ordered
/// **oldest first** (caller filters by pipeline_mode).
pub fn compute_evolution_gate(window: &[ConfigVariant]) -> EvolutionGateResult {
    let n = window.len();
    if n < 2 {
        return EvolutionGateResult::Allow { evolved_count: 0 };
    }

    let mut evolved_count = 0usize;
    for i in 1..n {
        let s = compute_similarity(&window[i - 1].config_blob, &window[i].config_blob).similarity;
        if s < SIM_STAGNATION_THRESHOLD {
            evolved_count += 1;
        }
    }
    let latest_similarity =
        compute_similarity(&window[n - 2].config_blob, &window[n - 1].config_blob).similarity;

    if evolved_count == 0 && latest_similarity >= SIM_STAGNATION_THRESHOLD {
        return EvolutionGateResult::Withhold {
            reason: "ontology never evolved".into(),
            identical_streak: n - 1,
        };
    }

    EvolutionGateResult::Allow { evolved_count }
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
    fn empty_window_allows() {
        let v = compute_evolution_gate(&[]);
        assert!(matches!(v, EvolutionGateResult::Allow { evolved_count: 0 }));
    }

    #[test]
    fn single_variant_allows() {
        let v = compute_evolution_gate(&[variant("v0", &serde_json::json!({"k": 1}))]);
        assert!(matches!(v, EvolutionGateResult::Allow { .. }));
    }

    #[test]
    fn five_identical_variants_withhold() {
        let blob = serde_json::json!({"k": 1, "j": "x"});
        let win: Vec<ConfigVariant> = (0..5).map(|i| variant(&format!("v{i}"), &blob)).collect();
        let v = compute_evolution_gate(&win);
        assert!(matches!(v, EvolutionGateResult::Withhold { .. }));
    }

    #[test]
    fn divergent_variants_allow() {
        let win: Vec<ConfigVariant> = (0..5)
            .map(|i| variant(&format!("v{i}"), &serde_json::json!({ "k": i })))
            .collect();
        let v = compute_evolution_gate(&win);
        match v {
            EvolutionGateResult::Allow { evolved_count } => {
                assert!(evolved_count > 0, "expected non-zero evolution");
            }
            other => panic!("expected Allow, got {other:?}"),
        }
    }

    #[test]
    fn mixed_three_identical_two_mutated_allow() {
        let blob_a = serde_json::json!({"k": 1, "j": "x"});
        let blob_b = serde_json::json!({"k": 1, "j": "y", "extra": 7});
        let win = vec![
            variant("v0", &blob_a),
            variant("v1", &blob_a),
            variant("v2", &blob_a),
            variant("v3", &blob_b),
            variant("v4", &blob_b),
        ];
        let v = compute_evolution_gate(&win);
        match v {
            EvolutionGateResult::Allow { evolved_count } => {
                assert_eq!(evolved_count, 1, "expected one evolved pair");
            }
            other => panic!("expected Allow, got {other:?}"),
        }
    }
}
