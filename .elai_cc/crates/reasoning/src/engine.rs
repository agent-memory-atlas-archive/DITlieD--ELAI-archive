use std::collections::HashMap;

use crate::types::{
    AgentGrcConfig, GrcBudget, GrcError, GrcState, GroundingLevel, GroundingResult, ThinkResult,
    ThinkStep, TopicState, VerificationResult,
};

// ---------------------------------------------------------------------------
// GrcEngine trait: the contract for grounded reasoning
// ---------------------------------------------------------------------------

pub trait GrcEngine {
    // Phase 2A methods (implemented in BasicGrcEngine)
    fn think(&mut self, step: ThinkStep) -> Result<ThinkResult, GrcError>;
    fn set_budget(&mut self, topic: &str, max_calls: u32, max_chars: u32);
    fn get_budget(&self, topic: &str) -> Option<&GrcBudget>;
    fn reset_topic(&mut self, topic: &str);

    // Phase 2B stubs (present for trait stability)
    #[allow(unused)]
    fn verify_claim(&self, claim: &str, source: &str) -> Result<VerificationResult, GrcError>;
    #[allow(unused)]
    fn ground_plan(&self, plan: &str) -> Vec<GroundingResult>;
}

// ---------------------------------------------------------------------------
// BasicGrcEngine: phase 2A implementation
// ---------------------------------------------------------------------------

pub struct BasicGrcEngine {
    topics: HashMap<String, TopicState>,
    config: AgentGrcConfig,
}

impl BasicGrcEngine {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            config: AgentGrcConfig::default(),
        }
    }

    pub fn with_config(config: AgentGrcConfig) -> Self {
        Self {
            topics: HashMap::new(),
            config,
        }
    }

    fn get_or_create_topic(&mut self, topic: &str) -> &mut TopicState {
        self.topics
            .entry(topic.to_string())
            .or_insert_with(|| TopicState::new(GrcBudget::default()))
    }

    /// Parse grounding string into (source_type, reference) pairs.
    /// Valid types: "file", "grep", "test", "search", "tool", "training".
    /// Format: "type:reference" comma-separated.
    fn parse_grounding(raw: &str) -> Result<Vec<(String, String, GroundingLevel)>, GrcError> {
        let mut results = Vec::new();
        for citation in raw.split(',') {
            let citation = citation.trim();
            if citation.is_empty() {
                continue;
            }
            let colon_pos = citation.find(':').ok_or_else(|| GrcError::MalformedGrounding {
                raw: citation.to_string(),
                reason: "missing ':' separator — expected 'type:reference'".to_string(),
            })?;
            let source_type = &citation[..colon_pos];
            let reference = &citation[colon_pos + 1..];
            if reference.is_empty() {
                return Err(GrcError::MalformedGrounding {
                    raw: citation.to_string(),
                    reason: "empty reference after ':'".to_string(),
                });
            }
            let level = match source_type {
                "test" | "search" => GroundingLevel::Verified,
                "file" | "grep" | "tool" => GroundingLevel::High,
                "training" => GroundingLevel::Uncertain,
                _ => {
                    return Err(GrcError::MalformedGrounding {
                        raw: citation.to_string(),
                        reason: format!(
                            "unknown source type '{}' — valid: file, grep, test, search, tool, training",
                            source_type
                        ),
                    });
                }
            };
            results.push((source_type.to_string(), reference.to_string(), level));
        }
        if results.is_empty() {
            return Err(GrcError::MalformedGrounding {
                raw: raw.to_string(),
                reason: "no valid citations found".to_string(),
            });
        }
        Ok(results)
    }

    /// Determine overall grounding level from a set of citations.
    /// Returns the minimum (weakest) level across all citations.
    fn overall_grounding_level(citations: &[(String, String, GroundingLevel)]) -> GroundingLevel {
        citations
            .iter()
            .map(|(_, _, level)| *level)
            .min()
            .unwrap_or(GroundingLevel::Uncertain)
    }
}

impl Default for BasicGrcEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GrcEngine for BasicGrcEngine {
    fn think(&mut self, step: ThinkStep) -> Result<ThinkResult, GrcError> {
        let topic_key = step.topic.clone();

        // Ensure topic exists with default budget if not yet configured.
        self.get_or_create_topic(&topic_key);

        let state = self.topics.get_mut(&topic_key).expect("just inserted");
        let mut warnings = Vec::new();

        // --- State transition validation ---
        if step.step_type == GrcState::Status {
            // Status is free: no transition check, no budget, no grounding.
            state.steps.push(step.clone());
            return Ok(ThinkResult {
                step,
                budget_remaining: state.budget,
                grounding_level: None,
                warnings,
            });
        }

        // First non-Status/non-Converge step must be Observe.
        if step.step_type != GrcState::Converge {
            match state.last_state {
                None => {
                    if step.step_type != GrcState::Observe {
                        return Err(GrcError::InvalidTransition {
                            from: None,
                            to: step.step_type,
                            allowed: vec![GrcState::Observe, GrcState::Converge, GrcState::Status],
                        });
                    }
                }
                Some(last) => {
                    let allowed = last.allowed_next();
                    if !allowed.contains(&step.step_type) {
                        return Err(GrcError::InvalidTransition {
                            from: Some(last),
                            to: step.step_type,
                            allowed: allowed.to_vec(),
                        });
                    }
                }
            }
        }
        // Converge is allowed from any state (including as first step).

        // --- Budget enforcement (check-then-increment) ---
        if state.budget.is_exhausted() {
            return Err(GrcError::BudgetExhausted {
                topic: topic_key,
                calls_used: state.budget.calls_used,
                max_calls: state.budget.max_calls,
            });
        }

        let content_len = step.content.len() as u32;
        if state.budget.chars_used.saturating_add(content_len) > state.budget.max_chars {
            return Err(GrcError::BudgetExhausted {
                topic: topic_key,
                calls_used: state.budget.calls_used,
                max_calls: state.budget.max_calls,
            });
        }

        // Must-converge check: if remaining_calls <= 1, only Converge is allowed.
        if state.budget.must_converge() && step.step_type != GrcState::Converge {
            return Err(GrcError::BudgetExhausted {
                topic: topic_key,
                calls_used: state.budget.calls_used,
                max_calls: state.budget.max_calls,
            });
        }

        // --- Grounding validation ---
        let mut grounding_level = None;

        if step.step_type.requires_grounding() {
            match &step.grounding {
                None => {
                    return Err(GrcError::UngroundedStep {
                        step_type: step.step_type,
                        topic: topic_key,
                    });
                }
                Some(raw) => {
                    let citations = Self::parse_grounding(raw)?;
                    let level = Self::overall_grounding_level(&citations);
                    if level == GroundingLevel::Uncertain {
                        warnings.push(format!(
                            "UNCERTAIN grounding on {:?} step in topic '{}' — should be resolved before acting",
                            step.step_type, topic_key
                        ));
                    }
                    for (stype, reference, clevel) in &citations {
                        state
                            .grounding_log
                            .push((format!("{}:{}", stype, reference), *clevel));
                    }
                    grounding_level = Some(level);
                }
            }
        }

        // --- Scope enumeration validation ---
        if self.config.require_scope_enumeration {
            if step.step_type == GrcState::Observe {
                match &step.scope {
                    Some(scope_items) => {
                        state.declared_scope = Some(scope_items.clone());
                    }
                    None => {
                        warnings.push(format!(
                            "scope enumeration required but not provided in Observe step for topic '{}'",
                            topic_key
                        ));
                    }
                }
            }

            if step.step_type == GrcState::PlanAction {
                if let Some(declared) = &state.declared_scope {
                    let content_lower = step.content.to_lowercase();
                    for item in declared {
                        if !content_lower.contains(&item.to_lowercase()) {
                            warnings.push(format!(
                                "scope item '{}' declared in Observe but not addressed in PlanAction for topic '{}'",
                                item, topic_key
                            ));
                        }
                    }
                }
            }
        }

        // --- Budget increment (after all checks pass) ---
        state.budget.calls_used = state.budget.calls_used.saturating_add(1);
        state.budget.chars_used = state.budget.chars_used.saturating_add(content_len);

        // --- Update state ---
        state.last_state = Some(step.step_type);
        state.steps.push(step.clone());
        let budget_remaining = state.budget;

        Ok(ThinkResult {
            step,
            budget_remaining,
            grounding_level,
            warnings,
        })
    }

    fn set_budget(&mut self, topic: &str, max_calls: u32, max_chars: u32) {
        let state = self
            .topics
            .entry(topic.to_string())
            .or_insert_with(|| TopicState::new(GrcBudget::new(max_calls, max_chars)));
        state.budget = GrcBudget::new(max_calls, max_chars);
    }

    fn get_budget(&self, topic: &str) -> Option<&GrcBudget> {
        self.topics.get(topic).map(|s| &s.budget)
    }

    fn reset_topic(&mut self, topic: &str) {
        self.topics.remove(topic);
    }

    // Phase 2B stubs
    fn verify_claim(&self, _claim: &str, _source: &str) -> Result<VerificationResult, GrcError> {
        unimplemented!("verify_claim is phase 2B — not yet implemented")
    }

    fn ground_plan(&self, _plan: &str) -> Vec<GroundingResult> {
        unimplemented!("ground_plan is phase 2B — not yet implemented")
    }
}
