use std::collections::HashMap;

use crate::types::{AgentGrcConfig, GrcDefaults, GrcError, GrcState, VerificationMethod};

// ---------------------------------------------------------------------------
// AgentGrcConfigLoader: trait for loading per-role GRC configuration
// ---------------------------------------------------------------------------

pub trait AgentGrcConfigLoader {
    /// Load GRC configuration for a given agent role, merging with global defaults.
    /// Per-role values override globals field-by-field (not deep merge).
    /// Missing per-role fields inherit from global defaults.
    fn load_config(
        &self,
        role: &str,
        global_defaults: &GrcDefaults,
    ) -> Result<AgentGrcConfig, GrcError>;

    /// Validate an AgentGrcConfig after construction.
    /// Rules (D2 from phase-2C spec):
    ///   (a) required_steps is non-empty
    ///   (b) required_steps forms a valid chain through default + overridden transitions
    ///   (c) budget.max_calls >= required_steps.len()
    ///   (d) csq_required implies at least one Verify in required_steps
    ///   (e) allowed_methods is a subset of all known VerificationMethod variants
    fn validate_config(&self, config: &AgentGrcConfig) -> Result<(), GrcError>;
}

// ---------------------------------------------------------------------------
// TomlConfigLoader: stub implementation reading from core::Config
// Config parsing lives in core/ — reasoning/ receives typed structs, not raw TOML.
// ---------------------------------------------------------------------------

pub struct TomlConfigLoader;

impl TomlConfigLoader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TomlConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentGrcConfigLoader for TomlConfigLoader {
    fn load_config(
        &self,
        role: &str,
        global_defaults: &GrcDefaults,
    ) -> Result<AgentGrcConfig, GrcError> {
        // Stub: returns defaults merged with role name only.
        // Full implementation in phase 2C.4 (I5) will read from core::Config.
        let config = AgentGrcConfig {
            role: role.to_string(),
            budget: global_defaults.budget,
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ],
            grounding_level: global_defaults.grounding_level,
            csq_required: global_defaults.csq_required,
            allowed_methods: global_defaults.allowed_methods.clone(),
            transition_overrides: HashMap::new(),
            require_scope_enumeration: false,
        };
        self.validate_config(&config)?;
        Ok(config)
    }

    fn validate_config(&self, config: &AgentGrcConfig) -> Result<(), GrcError> {
        // (a) required_steps non-empty
        if config.required_steps.is_empty() {
            return Err(GrcError::ConfigValidation {
                role: config.role.clone(),
                reason: "required_steps must not be empty".to_string(),
            });
        }

        // (b) required_steps forms a valid chain through merged transitions
        validate_step_chain(config)?;

        // (c) budget.max_calls >= required_steps.len()
        if (config.budget.max_calls as usize) < config.required_steps.len() {
            return Err(GrcError::ConfigValidation {
                role: config.role.clone(),
                reason: format!(
                    "budget max_calls ({}) < required_steps length ({})",
                    config.budget.max_calls,
                    config.required_steps.len()
                ),
            });
        }

        // (d) csq_required implies at least one Verify in required_steps
        if config.csq_required
            && !config
                .required_steps
                .iter()
                .any(|s| *s == GrcState::Verify)
        {
            return Err(GrcError::ConfigValidation {
                role: config.role.clone(),
                reason: "csq_required is true but required_steps has no Verify step".to_string(),
            });
        }

        // (e) allowed_methods subset of all known variants
        let all_known = all_verification_methods();
        for method in &config.allowed_methods {
            if !all_known.contains(method) {
                return Err(GrcError::ConfigValidation {
                    role: config.role.clone(),
                    reason: format!("unknown verification method: {:?}", method),
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Compute the effective allowed transitions from a state, merging defaults
/// with the config's transition_overrides (additive-only union).
pub fn effective_allowed_next(
    state: GrcState,
    overrides: &HashMap<GrcState, Vec<GrcState>>,
) -> Vec<GrcState> {
    let base = state.allowed_next();
    let mut merged: Vec<GrcState> = base.to_vec();
    if let Some(extras) = overrides.get(&state) {
        for extra in extras {
            if !merged.contains(extra) {
                merged.push(*extra);
            }
        }
    }
    merged
}

/// Validate that required_steps forms a traversable chain through merged transitions.
/// The first step must be reachable from None (i.e., must be Observe or Converge).
/// Each subsequent step must be in effective_allowed_next of the previous.
fn validate_step_chain(config: &AgentGrcConfig) -> Result<(), GrcError> {
    let steps = &config.required_steps;
    if steps.is_empty() {
        return Ok(());
    }

    // First step must be Observe (only valid first non-Status step) or Converge.
    let first = steps[0];
    if first != GrcState::Observe && first != GrcState::Converge {
        return Err(GrcError::ConfigValidation {
            role: config.role.clone(),
            reason: format!(
                "required_steps must begin with Observe or Converge, got {:?}",
                first
            ),
        });
    }

    // Each subsequent step must be reachable from previous through merged transitions.
    for window in steps.windows(2) {
        let from = window[0];
        let to = window[1];
        let allowed = effective_allowed_next(from, &config.transition_overrides);
        if !allowed.contains(&to) {
            return Err(GrcError::ConfigValidation {
                role: config.role.clone(),
                reason: format!(
                    "invalid step chain: {:?} -> {:?} not in allowed transitions {:?}",
                    from, to, allowed
                ),
            });
        }
    }

    Ok(())
}

/// All known VerificationMethod variants.
fn all_verification_methods() -> Vec<VerificationMethod> {
    vec![
        VerificationMethod::FileCheck,
        VerificationMethod::SourceCheck,
        VerificationMethod::TestCheck,
        VerificationMethod::WebCheck,
        VerificationMethod::ApiCheck,
        VerificationMethod::ApiRegistryCheck,
    ]
}
