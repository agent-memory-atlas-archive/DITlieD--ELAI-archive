use rustc_hash::FxHashMap;

use crate::types::{AgentGrcConfig, GrcBudget, GrcDefaults, GrcError, GrcState, GroundingLevel, VerificationMethod};

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
        // Per-role overrides merged with global defaults (field-by-field, not deep merge).
        // In phase 4A, these overrides come from core::Config parsing of elai_cc.toml
        // [agents.{role}.grc] tables. For now, known role definitions are hardcoded
        // per reasoning.txt section 7.
        let config = match role {
            "executor" => {
                AgentGrcConfig {
                    role: role.to_string(),
                    budget: GrcBudget::new(8, 1200),
                    required_steps: vec![
                        GrcState::Observe,
                        GrcState::Hypothesize,
                        GrcState::PlanAction,
                        GrcState::Act,
                        GrcState::Verify,
                    ],
                    grounding_level: global_defaults.grounding_level,
                    csq_required: true,
                    allowed_methods: global_defaults.allowed_methods.clone(),
                    transition_overrides: FxHashMap::default(),
                    require_scope_enumeration: true,
                }
            }
            "auditor" => {
                let mut overrides = FxHashMap::default();
                overrides.insert(GrcState::Hypothesize, vec![GrcState::Verify]);
                overrides.insert(GrcState::Verify, vec![GrcState::Verify, GrcState::Observe]);
                AgentGrcConfig {
                    role: role.to_string(),
                    budget: GrcBudget::new(10, 1200),
                    required_steps: vec![
                        GrcState::Observe,
                        GrcState::Hypothesize,
                        GrcState::Verify,
                        GrcState::Verify,
                    ],
                    grounding_level: GroundingLevel::Verified,
                    csq_required: true,
                    allowed_methods: global_defaults.allowed_methods.clone(),
                    transition_overrides: overrides,
                    require_scope_enumeration: false,
                }
            }
            "context_compiler" => {
                AgentGrcConfig {
                    role: role.to_string(),
                    budget: GrcBudget::new(3, 600),
                    required_steps: vec![GrcState::Observe],
                    grounding_level: GroundingLevel::High,
                    csq_required: false,
                    allowed_methods: vec![
                        VerificationMethod::FileCheck,
                        VerificationMethod::SourceCheck,
                    ],
                    transition_overrides: FxHashMap::default(),
                    require_scope_enumeration: false,
                }
            }
            "prompt_engineer" => {
                AgentGrcConfig {
                    role: role.to_string(),
                    budget: GrcBudget::new(5, 800),
                    required_steps: vec![
                        GrcState::Observe,
                        GrcState::Hypothesize,
                        GrcState::Converge,
                    ],
                    grounding_level: GroundingLevel::High,
                    csq_required: false,
                    allowed_methods: vec![
                        VerificationMethod::FileCheck,
                        VerificationMethod::SourceCheck,
                    ],
                    transition_overrides: FxHashMap::default(),
                    require_scope_enumeration: false,
                }
            }
            "memory_writer" => {
                AgentGrcConfig {
                    role: role.to_string(),
                    budget: GrcBudget::new(2, 400),
                    required_steps: vec![GrcState::Observe, GrcState::Converge],
                    grounding_level: global_defaults.grounding_level,
                    csq_required: false,
                    allowed_methods: vec![
                        VerificationMethod::FileCheck,
                        VerificationMethod::SourceCheck,
                    ],
                    transition_overrides: FxHashMap::default(),
                    require_scope_enumeration: false,
                }
            }
            "designer" => {
                let mut overrides = FxHashMap::default();
                overrides.insert(GrcState::PlanAction, vec![GrcState::Verify]);
                AgentGrcConfig {
                    role: role.to_string(),
                    budget: GrcBudget::new(8, 1200),
                    required_steps: vec![
                        GrcState::Observe,
                        GrcState::Hypothesize,
                        GrcState::PlanAction,
                        GrcState::Verify,
                        GrcState::Converge,
                    ],
                    grounding_level: GroundingLevel::High,
                    csq_required: true,
                    allowed_methods: global_defaults.allowed_methods.clone(),
                    transition_overrides: overrides,
                    require_scope_enumeration: true,
                }
            }
            // Unknown role: use defaults only (allows dynamic roles).
            _ => {
                AgentGrcConfig {
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
                    transition_overrides: FxHashMap::default(),
                    require_scope_enumeration: false,
                }
            }
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
    overrides: &FxHashMap<GrcState, Vec<GrcState>>,
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

// ---------------------------------------------------------------------------
// agent_grc_config_for_role: canonical per-role config table
// ---------------------------------------------------------------------------

/// Return the canonical `AgentGrcConfig` for a role name string.
///
/// This is the single source of truth for role configs that are **not** covered
/// by [`TomlConfigLoader`] (i.e., roles that exist in the agent pool but have
/// no TOML override path yet). Roles already handled by `TomlConfigLoader` are
/// re-exported here so callers can use one lookup function regardless of source.
///
/// Currently handles: `context_compiler`, `media_validator`.
/// For all other role names, returns `None` (caller falls back to
/// `AgentGrcConfig::default()` or `TomlConfigLoader::load_config`).
///
/// # B-5 wiring note
/// `agent_pool::default_grc_config` calls this for `ContextCompiler` and
/// `MediaValidator` arms, eliminating the divergent `..AgentGrcConfig::default()`
/// that previously yielded `required_steps=[]` (silently violating
/// `validate_config` rule (a)).
pub fn agent_grc_config_for_role(role: &str) -> Option<AgentGrcConfig> {
    match role {
        "context_compiler" => Some(AgentGrcConfig {
            role: role.to_string(),
            budget: GrcBudget::new(3, 600),
            // Observe-only: context compiler reads, never mutates.
            required_steps: vec![GrcState::Observe],
            grounding_level: GroundingLevel::High,
            csq_required: false,
            allowed_methods: vec![
                VerificationMethod::FileCheck,
                VerificationMethod::SourceCheck,
            ],
            transition_overrides: FxHashMap::default(),
            require_scope_enumeration: false,
        }),
        "media_validator" => {
            // Observe->Verify is not in the default GrcState transition graph
            // (Observe.allowed_next = [Hypothesize, Converge, Status]).
            // Add an additive transition override so validate_config rule (b)
            // passes if this config is ever run through TomlConfigLoader::validate_config.
            let mut overrides = FxHashMap::default();
            overrides.insert(GrcState::Observe, vec![GrcState::Verify]);
            Some(AgentGrcConfig {
                role: role.to_string(),
                budget: GrcBudget::new(5, 800),
                // Observe then Verify: validate media assets without mutating.
                required_steps: vec![GrcState::Observe, GrcState::Verify],
                grounding_level: GroundingLevel::High,
                csq_required: false,
                allowed_methods: vec![
                    VerificationMethod::FileCheck,
                    VerificationMethod::SourceCheck,
                    VerificationMethod::TestCheck,
                ],
                transition_overrides: overrides,
                require_scope_enumeration: false,
            })
        }
        // B-39 chat_reasoner: drives the REASONER stage (N best-of-N candidates).
        // GroundingLevel::High (Medium variant absent in types.rs); reasoner reads
        // code-just-read which IS the High definition. Required steps stop at
        // PlanAction because the Reasoner does not Verify — that's the auditor.
        "chat_reasoner" => Some(AgentGrcConfig {
            role: role.to_string(),
            budget: GrcBudget::new(4, 8192),
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
            ],
            grounding_level: GroundingLevel::High,
            csq_required: false,
            allowed_methods: vec![
                VerificationMethod::FileCheck,
                VerificationMethod::SourceCheck,
            ],
            transition_overrides: FxHashMap::default(),
            require_scope_enumeration: false,
        }),
        // B-39 chat_auditor: scores candidates with EXTERNAL grounding (Kamoi 2024
        // TACL: self-correction needs reliable feedback). Mirrors media_validator
        // for the Observe -> Verify additive override (not in default transitions).
        "chat_auditor" => {
            let mut overrides = FxHashMap::default();
            overrides.insert(GrcState::Observe, vec![GrcState::Verify]);
            Some(AgentGrcConfig {
                role: role.to_string(),
                budget: GrcBudget::new(4, 5120),
                required_steps: vec![GrcState::Observe, GrcState::Verify],
                grounding_level: GroundingLevel::High,
                csq_required: false,
                allowed_methods: vec![
                    VerificationMethod::FileCheck,
                    VerificationMethod::SourceCheck,
                    VerificationMethod::TestCheck,
                ],
                transition_overrides: overrides,
                require_scope_enumeration: false,
            })
        }
        _ => None,
    }
}
