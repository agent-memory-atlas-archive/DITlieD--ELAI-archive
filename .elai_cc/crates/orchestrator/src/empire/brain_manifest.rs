//! BrainManifest — schema-validated replacement for Empire's brain.json.
//!
//! Per empire.txt section 6, Empire's brain.json has zero schema validation.
//! `BrainManifest::validate` enforces the rules called out in phase-4J
//! 4J.1 ADVERSARIAL: empty brain_id is rejected, empty neurons list is
//! rejected, duplicate neuron_ids are rejected. Validation is called
//! CLIENT-SIDE before the FactoryLine `Register` command is even sent.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::types::{Capability, EngineTier, NeuronRegistration};

/// Manifest sent from the brain to the factory on `Register`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrainManifest {
    pub brain_id: String,
    pub version: String,
    pub engine_tier: EngineTier,
    pub neurons: Vec<NeuronRegistration>,
    pub capabilities: Vec<Capability>,
    pub heartbeat_interval_ms: u64,
}

/// Failure modes of `BrainManifest::validate`.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum BrainManifestError {
    #[error("brain_id is empty")]
    EmptyBrainId,
    #[error("neurons list is empty")]
    EmptyNeurons,
    #[error("duplicate neuron_id: {0}")]
    DuplicateNeuron(String),
    #[error("heartbeat_interval_ms must be > 0")]
    HeartbeatNonZero,
}

impl BrainManifest {
    /// Validate the manifest against the four hard-rule schema checks.
    pub fn validate(&self) -> Result<(), BrainManifestError> {
        if self.brain_id.trim().is_empty() {
            return Err(BrainManifestError::EmptyBrainId);
        }
        if self.neurons.is_empty() {
            return Err(BrainManifestError::EmptyNeurons);
        }
        if self.heartbeat_interval_ms == 0 {
            return Err(BrainManifestError::HeartbeatNonZero);
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for n in &self.neurons {
            if !seen.insert(n.neuron_id.as_str()) {
                return Err(BrainManifestError::DuplicateNeuron(n.neuron_id.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_neuron(id: &str) -> NeuronRegistration {
        NeuronRegistration {
            neuron_id: id.to_string(),
            role: "stub".to_string(),
            hooks: Vec::new(),
        }
    }

    fn ok_manifest() -> BrainManifest {
        BrainManifest {
            brain_id: "elai_cc".to_string(),
            version: "0.1.0".to_string(),
            engine_tier: EngineTier::Mkii,
            neurons: vec![sample_neuron("n1"), sample_neuron("n2")],
            capabilities: Vec::new(),
            heartbeat_interval_ms: 5_000,
        }
    }

    #[test]
    fn valid_manifest_passes() {
        let m = ok_manifest();
        assert_eq!(m.validate(), Ok(()));
    }

    #[test]
    fn empty_brain_id_rejected() {
        let mut m = ok_manifest();
        m.brain_id = "".to_string();
        assert_eq!(m.validate(), Err(BrainManifestError::EmptyBrainId));
    }

    #[test]
    fn empty_neurons_rejected() {
        let mut m = ok_manifest();
        m.neurons.clear();
        assert_eq!(m.validate(), Err(BrainManifestError::EmptyNeurons));
    }

    #[test]
    fn duplicate_neuron_rejected() {
        let mut m = ok_manifest();
        m.neurons.push(sample_neuron("n1"));
        match m.validate() {
            Err(BrainManifestError::DuplicateNeuron(id)) => assert_eq!(id, "n1"),
            other => panic!("expected DuplicateNeuron, got {:?}", other),
        }
    }

    #[test]
    fn zero_heartbeat_rejected() {
        let mut m = ok_manifest();
        m.heartbeat_interval_ms = 0;
        assert_eq!(m.validate(), Err(BrainManifestError::HeartbeatNonZero));
    }
}
