use std::collections::HashMap;

use regex::Regex;

use crate::types::{
    AgentGrcConfig, GrcBudget, GrcError, GrcState, GroundingLevel, GroundingResult,
    PlanGroundingResult, ThinkResult, ThinkStep, TopicState, VerificationMethod, VerificationResult,
};
use crate::verification::{LocalVerifier, NoopTestRunner, StdFileReader, Verifier};

// ---------------------------------------------------------------------------
// GrcEngine trait: the contract for grounded reasoning
// ---------------------------------------------------------------------------

pub trait GrcEngine {
    // Phase 2A methods (implemented in BasicGrcEngine)
    fn think(&mut self, step: ThinkStep) -> Result<ThinkResult, GrcError>;
    fn set_budget(&mut self, topic: &str, max_calls: u32, max_chars: u32);
    fn get_budget(&self, topic: &str) -> Option<&GrcBudget>;
    fn reset_topic(&mut self, topic: &str);

    // Phase 2B: grounding verification (D6, D7 breaking changes from 2A)
    fn verify_claim(
        &self,
        claim: &str,
        source: &str,
        context: &str,
    ) -> Result<VerificationResult, GrcError>;
    fn ground_plan(&self, plan: &str) -> PlanGroundingResult;

    // Phase 2C: per-agent configuration routing
    /// Apply an agent-specific GRC configuration. Resets all topic state
    /// so leftover budget/steps from a previous config do not leak.
    fn with_agent_config(&mut self, config: AgentGrcConfig);
    /// Return the current agent config, if any.
    fn get_agent_config(&self) -> Option<&AgentGrcConfig>;
}

// ---------------------------------------------------------------------------
// BasicGrcEngine: phase 2A implementation
// ---------------------------------------------------------------------------

pub struct BasicGrcEngine {
    topics: HashMap<String, TopicState>,
    agent_config: Option<AgentGrcConfig>,
    verifier: Box<dyn Verifier>,
}

impl BasicGrcEngine {
    pub fn new() -> Self {
        let file_reader = Box::new(StdFileReader::new());
        let test_runner = Box::new(NoopTestRunner::new());
        Self {
            topics: HashMap::new(),
            agent_config: None,
            verifier: Box::new(LocalVerifier::new(file_reader, test_runner)),
        }
    }

    pub fn with_config(config: AgentGrcConfig) -> Self {
        let file_reader = Box::new(StdFileReader::new());
        let test_runner = Box::new(NoopTestRunner::new());
        Self {
            topics: HashMap::new(),
            agent_config: Some(config),
            verifier: Box::new(LocalVerifier::new(file_reader, test_runner)),
        }
    }

    pub fn with_verifier(verifier: Box<dyn Verifier>) -> Self {
        Self {
            topics: HashMap::new(),
            agent_config: None,
            verifier,
        }
    }

    /// Returns the effective require_scope_enumeration setting from agent config.
    fn require_scope_enumeration(&self) -> bool {
        self.agent_config
            .as_ref()
            .map_or(false, |c| c.require_scope_enumeration)
    }

    /// Parse a claim string with "type:reference" format into (VerificationMethod, stripped_ref).
    /// The "type:" prefix is ALWAYS stripped before dispatching to the Verifier.
    fn parse_claim_prefix(claim: &str) -> (VerificationMethod, &str) {
        if let Some(rest) = claim.strip_prefix("file:") {
            (VerificationMethod::FileCheck, rest)
        } else if let Some(rest) = claim.strip_prefix("grep:") {
            (VerificationMethod::FileCheck, rest)
        } else if let Some(rest) = claim.strip_prefix("test:") {
            (VerificationMethod::TestCheck, rest)
        } else if let Some(rest) = claim.strip_prefix("search:") {
            (VerificationMethod::WebCheck, rest)
        } else if let Some(rest) = claim.strip_prefix("tool:") {
            (VerificationMethod::SourceCheck, rest)
        } else if let Some(rest) = claim.strip_prefix("training:") {
            // training: claims have no direct verification method; attempt FileCheck
            (VerificationMethod::FileCheck, rest)
        } else {
            // No recognized prefix — treat as FileCheck with the full claim
            (VerificationMethod::FileCheck, claim)
        }
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
        // Capture config-derived values before borrowing self.topics mutably.
        let scope_enumeration_required = self.require_scope_enumeration();

        // Ensure topic exists with default budget if not yet configured.
        let state = self
            .topics
            .entry(topic_key.clone())
            .or_insert_with(|| TopicState::new(GrcBudget::default()));
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
        if scope_enumeration_required {
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

    fn verify_claim(
        &self,
        claim: &str,
        source: &str,
        context: &str,
    ) -> Result<VerificationResult, GrcError> {
        let (method, stripped) = Self::parse_claim_prefix(claim);
        let ctx = if !context.is_empty() {
            context
        } else if !source.is_empty() {
            source
        } else {
            ""
        };
        let mut result = self.verifier.verify(method, stripped, ctx)?;
        // Ensure the claim in the result is the stripped reference, not the raw prefix form.
        result.claim = stripped.to_string();
        Ok(result)
    }

    fn ground_plan(&self, plan: &str) -> PlanGroundingResult {
        if plan.trim().is_empty() {
            return PlanGroundingResult {
                claims: Vec::new(),
                all_grounded: true,
                ungrounded: Vec::new(),
                verify_needed: Vec::new(),
            };
        }

        let claim_re = Regex::new(r"(?m)^(?:\s*Grounding:\s*)((file|grep|test|search|tool|training):[^\s]+)")
            .expect("claim regex is valid");

        let mut claims = Vec::new();
        let mut ungrounded = Vec::new();
        let mut verify_needed = Vec::new();

        for cap in claim_re.captures_iter(plan) {
            let raw_claim = cap[1].to_string();
            let source_type = cap[2].to_string();

            if source_type == "training" {
                // Attempt auto-resolution: try FileCheck or SourceCheck
                let (method, stripped) = Self::parse_claim_prefix(&raw_claim);
                let auto_result = self.verifier.verify(method, stripped, "");
                match auto_result {
                    Ok(vr) if vr.passes => {
                        claims.push(GroundingResult {
                            claim: raw_claim,
                            method: VerificationMethod::FileCheck,
                            level: GroundingLevel::Verified,
                            source: source_type,
                            verified_at: 0,
                        });
                    }
                    _ => {
                        verify_needed.push(raw_claim.clone());
                        ungrounded.push(raw_claim.clone());
                        claims.push(GroundingResult {
                            claim: raw_claim,
                            method: VerificationMethod::FileCheck,
                            level: GroundingLevel::Uncertain,
                            source: source_type,
                            verified_at: 0,
                        });
                    }
                }
                continue;
            }

            let result = self.verify_claim(&raw_claim, &source_type, "");
            match result {
                Ok(vr) => {
                    let level = if vr.passes {
                        GroundingLevel::Verified
                    } else {
                        ungrounded.push(raw_claim.clone());
                        GroundingLevel::Uncertain
                    };
                    claims.push(GroundingResult {
                        claim: raw_claim,
                        method: vr.method_used,
                        level,
                        source: source_type,
                        verified_at: 0,
                    });
                }
                Err(_) => {
                    ungrounded.push(raw_claim.clone());
                    let (method, _) = Self::parse_claim_prefix(&raw_claim);
                    claims.push(GroundingResult {
                        claim: raw_claim,
                        method,
                        level: GroundingLevel::Uncertain,
                        source: source_type,
                        verified_at: 0,
                    });
                }
            }
        }

        let all_grounded = ungrounded.is_empty();
        PlanGroundingResult {
            claims,
            all_grounded,
            ungrounded,
            verify_needed,
        }
    }

    fn with_agent_config(&mut self, config: AgentGrcConfig) {
        // Reset all topic state so leftover budget/steps from previous config don't leak.
        self.topics.clear();
        self.agent_config = Some(config);
    }

    fn get_agent_config(&self) -> Option<&AgentGrcConfig> {
        self.agent_config.as_ref()
    }
}
