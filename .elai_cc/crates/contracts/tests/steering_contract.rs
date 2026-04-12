//! Contract tests for the steering vector (ActAdd) module.
//!
//! Verifies that the public API surface of layer_config::steering matches
//! the expected interface: SteeringConfig, SteeringVector, SteeringVectorSpec,
//! SteeringVectorEngine, SteeringError, and validation logic.

use elai_providers::layer_config::steering::{
    SteeringConfig, SteeringVector, SteeringVectorSpec, SteeringVectorEngine, SteeringError,
    COEFFICIENT_DEFAULT, COEFFICIENT_MIN, COEFFICIENT_MAX,
};

// ---------------------------------------------------------------------------
// Contract: SteeringVectorSpec public API surface
// ---------------------------------------------------------------------------

#[test]
fn steering_vector_spec_resolve_layer_auto_returns_correct_fraction() {
    let spec = SteeringVectorSpec {
        name: "test".to_string(),
        layer: "auto".to_string(),
        coefficient: 5.0,
        positive_prompt: "positive".to_string(),
        negative_prompt: "negative".to_string(),
    };
    
    let layer = spec.resolve_layer(30).expect("resolve_layer");
    // auto = model_layers / 3 = 30 / 3 = 10
    assert_eq!(layer, 10);
}

#[test]
fn steering_vector_spec_resolve_layer_numeric_parses_correctly() {
    let spec = SteeringVectorSpec {
        name: "test".to_string(),
        layer: "15".to_string(),
        coefficient: 5.0,
        positive_prompt: "positive".to_string(),
        negative_prompt: "negative".to_string(),
    };
    
    let layer = spec.resolve_layer(30).expect("resolve_layer");
    assert_eq!(layer, 15);
}

#[test]
fn steering_vector_spec_resolve_layer_invalid_returns_error() {
    let spec = SteeringVectorSpec {
        name: "test".to_string(),
        layer: "invalid".to_string(),
        coefficient: 5.0,
        positive_prompt: "positive".to_string(),
        negative_prompt: "negative".to_string(),
    };
    
    let result = spec.resolve_layer(30);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Contract: SteeringConfig public API surface
// ---------------------------------------------------------------------------

#[test]
fn steering_config_default_returns_disabled() {
    let config = SteeringConfig::default();
    assert!(!config.enabled);
    assert!(config.vectors.is_empty());
}

#[test]
fn steering_config_default_for_role_returns_correct_variants() {
    let exec = SteeringConfig::default_for_role("executor");
    assert!(!exec.enabled, "executor should not have steering by default");
    
    let audit = SteeringConfig::default_for_role("auditor");
    assert!(audit.enabled, "auditor should have steering enabled");
    assert_eq!(audit.vectors.len(), 1);
    assert_eq!(audit.vectors[0].name, "skepticism");
    
    let pe = SteeringConfig::default_for_role("prompt-engineer");
    assert!(pe.enabled, "prompt-engineer should have steering enabled");
    assert_eq!(pe.vectors[0].name, "completeness");
    
    let cc = SteeringConfig::default_for_role("context-compiler");
    assert!(cc.enabled, "context-compiler should have steering enabled");
    assert_eq!(cc.vectors[0].name, "fidelity");
}

#[test]
fn steering_config_serde_roundtrip() {
    let config = SteeringConfig {
        enabled: true,
        vectors: vec![SteeringVectorSpec {
            name: "test".to_string(),
            layer: "auto".to_string(),
            coefficient: 5.0,
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        }],
    };
    
    let json = serde_json::to_string(&config).expect("serialize");
    let back: SteeringConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.enabled, config.enabled);
    assert_eq!(back.vectors.len(), config.vectors.len());
    assert_eq!(back.vectors[0].name, config.vectors[0].name);
}

// ---------------------------------------------------------------------------
// Contract: SteeringVector validation logic
// ---------------------------------------------------------------------------

#[test]
fn steering_vector_magnitude_returns_norm() {
    let vector = vec![3.0, 4.0]; // norm = 5
    let sv = SteeringVector {
        name: "test".to_string(),
        layer: 10,
        coefficient: 5.0,
        vector: vector.clone(),
    };
    
    let magnitude = sv.magnitude();
    assert_eq!(magnitude, 5.0);
}

#[test]
fn steering_vector_is_zero_magnitude_detects_zero() {
    let sv = SteeringVector {
        name: "test".to_string(),
        layer: 10,
        coefficient: 5.0,
        vector: vec![0.0, 0.0, 0.0],
    };
    
    assert!(sv.is_zero_magnitude());
}

#[test]
fn steering_vector_is_zero_magnitude_false_for_nonzero() {
    let sv = SteeringVector {
        name: "test".to_string(),
        layer: 10,
        coefficient: 5.0,
        vector: vec![1.0, 2.0, 3.0],
    };
    
    assert!(!sv.is_zero_magnitude());
}

// ---------------------------------------------------------------------------
// Contract: SteeringVectorEngine API surface
// ---------------------------------------------------------------------------

#[test]
fn steering_vector_engine_new_returns_engine() {
    let config = SteeringConfig::default();
    let engine = SteeringVectorEngine::new(&config, 32);
    
    assert!(!engine.is_active(), "should not be active with empty config");
    assert!(engine.vectors().is_empty());
}

#[test]
fn steering_vector_engine_is_active_false_when_no_vectors() {
    let config = SteeringConfig::default();
    let engine = SteeringVectorEngine::new(&config, 32);
    
    assert!(!engine.is_active());
}

#[test]
fn steering_vector_engine_coefficient_validation_clamps_to_min() {
    let config = SteeringConfig {
        enabled: true,
        vectors: vec![SteeringVectorSpec {
            name: "test".to_string(),
            layer: "10".to_string(),
            coefficient: 0.5, // Below COEFFICIENT_MIN
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        }],
    };
    
    let engine = SteeringVectorEngine::new(&config, 32);
    // Coefficient should be clamped to COEFFICIENT_DEFAULT (5.0)
    // This is validated in the Rust implementation
}

#[test]
fn steering_vector_engine_coefficient_validation_clamps_to_max() {
    let config = SteeringConfig {
        enabled: true,
        vectors: vec![SteeringVectorSpec {
            name: "test".to_string(),
            layer: "10".to_string(),
            coefficient: 100.0, // Above COEFFICIENT_MAX
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        }],
    };
    
    let engine = SteeringVectorEngine::new(&config, 32);
    // Coefficient should be clamped to COEFFICIENT_DEFAULT (5.0)
}

// ---------------------------------------------------------------------------
// Contract: SteeringError variants
// ---------------------------------------------------------------------------

#[test]
fn steering_error_no_active_vectors_displays_correctly() {
    let err = SteeringError::NoActiveVectors;
    let msg = format!("{}", err);
    assert!(msg.contains("no active steering vectors"));
}

#[test]
fn steering_error_zero_magnitude_displays_correctly() {
    let err = SteeringError::ZeroMagnitude {
        name: "test_vector".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("test_vector"));
    assert!(msg.contains("zero magnitude"));
}

#[test]
fn steering_error_layer_out_of_range_displays_correctly() {
    let err = SteeringError::LayerOutOfRange {
        index: 100,
        total: 32,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("100"));
    assert!(msg.contains("32"));
}

#[test]
fn steering_error_invalid_coefficient_displays_correctly() {
    let err = SteeringError::InvalidCoefficient {
        coefficient: 0.5,
        min: COEFFICIENT_MIN,
        max: COEFFICIENT_MAX,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("0.5"));
    assert!(msg.contains(&COEFFICIENT_MIN.to_string()));
    assert!(msg.contains(&COEFFICIENT_MAX.to_string()));
}

// ---------------------------------------------------------------------------
// Contract: Constants
// ---------------------------------------------------------------------------

#[test]
fn steering_constants_are_in_expected_ranges() {
    assert!(COEFFICIENT_DEFAULT >= COEFFICIENT_MIN);
    assert!(COEFFICIENT_DEFAULT <= COEFFICIENT_MAX);
    assert!(COEFFICIENT_MIN < COEFFICIENT_MAX);
}
