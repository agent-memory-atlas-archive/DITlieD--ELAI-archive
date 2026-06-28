use rustc_hash::FxHashMap;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::effective_allowed_next;
use crate::types::{
    AgentGrcConfig, CsqCheck, CsqResult, CsqSeverity, FeasibilityMethod, FeasibilityOutcome,
    GrcBudget, GrcError, GrcState, GroundingLevel, GroundingResult, InlineFeasibilityResult,
    InlineFeasibilityState, PlanGroundingResult, ThinkResult, ThinkStep, TopicState,
    VerificationMethod, VerificationResult,
};
use crate::verification::{LocalVerifier, NoopTestRunner, StdFileReader, Verifier};

static CLAIM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(?:\s*Grounding:\s*)((file|grep|test|search|tool|training):[^\s]+)")
        .expect("CLAIM_RE: literal pattern, must compile")
});

/// Finding marker prefix for anti-rubber-stamp enforcement.
/// A finding is any Verify step whose content starts with this prefix
/// (case-sensitive, after trimming leading whitespace).
/// Internal protocol between GRC engine and auditor prompt.
const FINDING_PREFIX: &str = "FINDING:";

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

    // Phase 2D: CSQ and inline feasibility
    /// Generate counterfactual safety questions for a function.
    /// sensitive_ops: list of sensitive API calls found by the caller (e.g., "std::fs::write").
    /// Empty sensitive_ops -> empty checks (no CSQ needed).
    fn generate_csq(
        &self,
        function_id: &str,
        sensitive_ops: &[String],
    ) -> Result<CsqResult, GrcError>;

    /// Check inline feasibility of a claim during EXECUTING.
    /// Returns Err(BudgetExhausted) if max 2 checks already used.
    /// &mut self: mutates InlineFeasibilityState counter.
    fn check_inline_feasibility(
        &mut self,
        claim: &str,
        sources: &[String],
    ) -> Result<InlineFeasibilityResult, GrcError>;
}

// ---------------------------------------------------------------------------
// BasicGrcEngine: phase 2A implementation
// ---------------------------------------------------------------------------

pub struct BasicGrcEngine {
    topics: FxHashMap<String, TopicState>,
    agent_config: Option<AgentGrcConfig>,
    verifier: Box<dyn Verifier>,
    pub inline_feasibility_state: InlineFeasibilityState,
}

impl BasicGrcEngine {
    pub fn new() -> Self {
        let file_reader = Box::new(StdFileReader::new());
        let test_runner = Box::new(NoopTestRunner::new());
        Self {
            topics: FxHashMap::default(),
            agent_config: None,
            verifier: Box::new(LocalVerifier::new(file_reader, test_runner)),
            inline_feasibility_state: InlineFeasibilityState::default(),
        }
    }

    pub fn with_config(config: AgentGrcConfig) -> Self {
        let file_reader = Box::new(StdFileReader::new());
        let test_runner = Box::new(NoopTestRunner::new());
        Self {
            topics: FxHashMap::default(),
            agent_config: Some(config),
            verifier: Box::new(LocalVerifier::new(file_reader, test_runner)),
            inline_feasibility_state: InlineFeasibilityState::default(),
        }
    }

    pub fn with_verifier(verifier: Box<dyn Verifier>) -> Self {
        Self {
            topics: FxHashMap::default(),
            agent_config: None,
            verifier,
            inline_feasibility_state: InlineFeasibilityState::default(),
        }
    }

    /// Returns the effective require_scope_enumeration setting from agent config.
    fn require_scope_enumeration(&self) -> bool {
        self.agent_config
            .as_ref()
            .is_some_and(|c| c.require_scope_enumeration)
    }

    /// Compute effective allowed transitions from a state, considering agent config.
    /// When agent config is set:
    ///   1. Merge default transitions with transition_overrides (additive union)
    ///   2. Filter to step whitelist: only states in required_steps + [Status, Converge]
    ///
    /// When no agent config: use default allowed_next unchanged.
    fn effective_transitions(&self, state: GrcState) -> Vec<GrcState> {
        match &self.agent_config {
            None => state.allowed_next().to_vec(),
            Some(config) => {
                let merged = effective_allowed_next(state, &config.transition_overrides);
                // Step whitelist: restrict to states that appear in required_steps
                // plus Status and Converge (always allowed).
                let mut whitelist: Vec<GrcState> = config.required_steps.clone();
                if !whitelist.contains(&GrcState::Status) {
                    whitelist.push(GrcState::Status);
                }
                if !whitelist.contains(&GrcState::Converge) {
                    whitelist.push(GrcState::Converge);
                }
                merged
                    .into_iter()
                    .filter(|s| whitelist.contains(s))
                    .collect()
            }
        }
    }


    /// Parse a claim string with "type:reference" format into (VerificationMethod, stripped_ref).
    /// The "type:" prefix is ALWAYS stripped before dispatching to the Verifier.
    ///
    /// "artifact:" syntax: "artifact:artifact_name:query_text"
    /// The first ':' after "artifact:" separates artifact_name from query.
    /// artifact_name must match ^[a-zA-Z0-9_-]+$; if malformed the call site
    /// should fall back to FileCheck (this fn is infallible for compatibility).
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
        } else if let Some(rest) = claim.strip_prefix("artifact:") {
            // Format: "artifact:artifact_name:query_text"
            // Split on first ':' to separate artifact_name from query.
            let (artifact_name, query) = if let Some(idx) = rest.find(':') {
                (&rest[..idx], &rest[idx + 1..])
            } else {
                // Bare artifact name with no query — use empty query string.
                (rest, "")
            };
            (
                VerificationMethod::ArtifactCheck {
                    artifact_name: artifact_name.to_string(),
                    query: query.to_string(),
                },
                rest,
            )
        } else if let Some(rest) = claim.strip_prefix("training:") {
            // training: claims have no direct verification method; attempt FileCheck
            (VerificationMethod::FileCheck, rest)
        } else {
            // No recognized prefix — treat as FileCheck with the full claim
            (VerificationMethod::FileCheck, claim)
        }
    }

    /// Parse grounding string into (source_type, reference) pairs.
    /// Valid types: "file", "grep", "test", "search", "tool", "training", "artifact".
    /// Format: "type:reference" comma-separated.
    /// "artifact" citations use format "artifact:artifact_name:query"; the full
    /// "artifact_name:query" string is stored as the reference field.
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
                // artifact: grounding level is High — the artifact store is a
                // local authoritative source (comparable to a file read).
                "artifact" => GroundingLevel::High,
                "training" => GroundingLevel::Uncertain,
                _ => {
                    return Err(GrcError::MalformedGrounding {
                        raw: citation.to_string(),
                        reason: format!(
                            "unknown source type '{}' — valid: file, grep, test, search, tool, artifact, training",
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
        let config_budget = self
            .agent_config
            .as_ref()
            .map(|c| GrcBudget::new(c.budget.max_calls, c.budget.max_chars));
        let config_min_grounding = self.agent_config.as_ref().map(|c| c.grounding_level);
        let config_allowed_methods: Option<Vec<VerificationMethod>> = self
            .agent_config
            .as_ref()
            .map(|c| c.allowed_methods.clone());
        let config_role = self.agent_config.as_ref().map(|c| c.role.clone());
        // Pre-compute effective transitions for the last state (needed before mutable borrow).
        // We'll read last_state from topics, compute transitions, then proceed.
        let last_state_for_topic = self.topics.get(&topic_key).and_then(|s| s.last_state);
        let effective_allowed = last_state_for_topic.map(|last| self.effective_transitions(last));

        // Ensure topic exists. Use agent config budget if set, else default.
        let default_budget = config_budget.unwrap_or_default();
        let state = self
            .topics
            .entry(topic_key.clone())
            .or_insert_with(|| TopicState::new(default_budget));
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
                            allowed: Box::new([
                                GrcState::Observe,
                                GrcState::Converge,
                                GrcState::Status,
                            ]),
                        });
                    }
                }
                Some(last) => {
                    let allowed = effective_allowed.unwrap_or_else(|| last.allowed_next().to_vec());
                    if !allowed.contains(&step.step_type) {
                        return Err(GrcError::InvalidTransition {
                            from: Some(last),
                            to: step.step_type,
                            allowed: allowed.into_boxed_slice(),
                        });
                    }
                }
            }
        }
        // Converge is allowed from any state (including as first step).

        // Anti-rubber-stamp: auditor must have >= 1 FINDING: before Converge.
        //
        // C-1: when this branch returns `GrcError::InsufficientFindings`,
        // production callers (PipelineGrc wrappers in `orchestrator::task_runner`
        // / `orchestrator::agent_pool`) MUST emit a corresponding
        // `SsbActionType::AuditorRubberStampRejected` SSB row via
        // `orchestrator::task_runner::emit_auditor_rubber_stamp_rejected`.
        // The emit site lives at the caller (not here) because `engine.rs`
        // has no Ssb writer field and adding one would couple the
        // reasoning crate to the orchestrator's SSB store (GAP-2 in
        // explorer-C1.md). See workflow/reasoning/C-1-decisions.md.
        if step.step_type == GrcState::Converge {
            if let Some(ref role) = config_role {
                if role == "auditor" {
                    let finding_count = state
                        .steps
                        .iter()
                        .filter(|s| s.step_type == GrcState::Verify)
                        .filter(|s| s.content.trim_start().starts_with(FINDING_PREFIX))
                        .count();
                    if finding_count < 1 {
                        return Err(GrcError::InsufficientFindings {
                            role: role.clone(),
                            expected_min: 1,
                        });
                    }
                }
            }
        }

        // --- Budget enforcement (check-then-increment) ---
        if state.budget.is_exhausted() {
            return Err(GrcError::BudgetExhausted {
                topic: topic_key,
                calls_used: state.budget.calls_used,
                max_calls: state.budget.max_calls,
            });
        }

        let content_len = u32::try_from(step.content.len()).expect("GRC step content fits u32");
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
                    // Allowed methods enforcement: reject citations using disallowed methods.
                    // Uses discriminant comparison so data-carrying variants like
                    // ArtifactCheck are matched by kind, not by field values.
                    if let Some(ref allowed_methods) = config_allowed_methods {
                        // Pre-build discriminant set for O(n) lookup.
                        let allowed_disc: std::collections::HashSet<_> = allowed_methods
                            .iter()
                            .map(std::mem::discriminant)
                            .collect();
                        for (source_type, reference, _) in &citations {
                            let method = match source_type.as_str() {
                                "file" | "grep" => VerificationMethod::FileCheck,
                                "tool" => VerificationMethod::SourceCheck,
                                "test" => VerificationMethod::TestCheck,
                                "search" => VerificationMethod::WebCheck,
                                "artifact" => VerificationMethod::ArtifactCheck {
                                    artifact_name: String::new(),
                                    query: String::new(),
                                },
                                // B-42: cosmos:<artifact_name>:<query>:<category> citation maps
                                // to CosmosCheck. Discriminant-only matching here; the actual
                                // field values are populated by PipelineGrc::verify_claim when
                                // the citation is dispatched.
                                "cosmos" => VerificationMethod::CosmosCheck {
                                    artifact_name: String::new(),
                                    query: String::new(),
                                    claim_category: String::new(),
                                },
                                "training" => continue,
                                _ => return Err(GrcError::MalformedGrounding {
                                    raw: format!("{}:{}", source_type, reference),
                                    reason: format!(
                                        "unknown citation prefix '{}' — valid: file, grep, test, search, tool, artifact, cosmos, training",
                                        source_type
                                    ),
                                }),
                            };
                            if !allowed_disc.contains(&std::mem::discriminant(&method)) {
                                let method_for_msg = method.clone();
                                return Err(GrcError::VerificationError {
                                    method: Box::new(method),
                                    claim: format!("{}:{}", source_type, reference),
                                    reason: format!(
                                        "method {:?} not in allowed_methods for role '{}'",
                                        method_for_msg,
                                        config_role.as_deref().unwrap_or("unknown")
                                    ),
                                });
                            }
                        }
                    }
                    let level = Self::overall_grounding_level(&citations);
                    // Grounding level enforcement: reject if below agent config minimum.
                    if let Some(min_level) = config_min_grounding {
                        if level < min_level {
                            return Err(GrcError::UngroundedStep {
                                step_type: step.step_type,
                                topic: topic_key,
                            });
                        }
                    }
                    if level == GroundingLevel::Uncertain {
                        // base-phase-2 (2B): DELIBERATE SCOPE — think() WARNS on
                        // UNCERTAIN grounding; it does NOT auto-resolve here.
                        // The spec's "UNCERTAIN auto-resolved via
                        // FILE_CHECK/SOURCE_CHECK" requirement
                        // (docs/specs/crates/reasoning.txt line 270, surfaced in
                        // PlanGroundingResult.verify_needed at line 213) is a
                        // PLAN-level operation and lives in ground_plan(), which
                        // proactively re-verifies every "training:" claim across a
                        // whole plan string. think() is the per-step gate where the
                        // AGENT supplies its own grounding citation in the step;
                        // its contract is to validate-or-warn what the agent
                        // asserts, not to silently re-run I/O verification on every
                        // step (that would change the warn-vs-reject semantics and
                        // add a filesystem/network round-trip to the hot reasoning
                        // path). The "should be resolved before acting" warning
                        // mirrors reasoning.txt line 60 ("UNCERTAIN ... MUST be
                        // resolved before acting") at step granularity; the actual
                        // auto-resolution happens in ground_plan() at the PLANNING
                        // boundary. Keeping the two separate is intentional, not an
                        // omission.
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

        let claim_re = &*CLAIM_RE;

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

    fn generate_csq(
        &self,
        function_id: &str,
        sensitive_ops: &[String],
    ) -> Result<CsqResult, GrcError> {
        if sensitive_ops.is_empty() {
            return Ok(CsqResult {
                function_id: function_id.to_string(),
                checks: Vec::new(),
            });
        }

        // Patterns mapping to the spec's CRITICAL severity categories
        // (reasoning.txt s6): data_loss (fs/file/write/delete/remove),
        // security_breach (net/tcpstream), financial_error (money/payment),
        // auth_bypass (auth/token/credential/password). Any op matching one
        // of these is classified Critical (guard MANDATORY); all others are
        // Warning (guard recommended, does not block).
        let critical_patterns: &[&str] = &[
            "fs::", "file", "write", "delete", "remove", "net::", "tcpstream",
            "money", "payment", "auth", "token", "credential", "password",
        ];

        let checks: Vec<CsqCheck> = sensitive_ops
            .iter()
            .map(|op| {
                let op_lower = op.to_lowercase();
                let is_critical = critical_patterns
                    .iter()
                    .any(|pat| op_lower.contains(pat));

                let severity = if is_critical {
                    CsqSeverity::Critical
                } else {
                    CsqSeverity::Warning
                };

                let consequence = if is_critical {
                    format!(
                        "data loss/security breach/financial error possible if {} fails",
                        op
                    )
                } else {
                    format!("incorrect output possible if {} fails", op)
                };

                CsqCheck {
                    assumption: format!(
                        "The call to {} in {} behaves as expected",
                        op, function_id
                    ),
                    consequence,
                    severity,
                    guard_added: false,
                }
            })
            .collect();

        Ok(CsqResult {
            function_id: function_id.to_string(),
            checks,
        })
    }

    fn check_inline_feasibility(
        &mut self,
        claim: &str,
        sources: &[String],
    ) -> Result<InlineFeasibilityResult, GrcError> {
        // Budget check: max 2 inline feasibility calls per EXECUTING phase.
        if self.inline_feasibility_state.is_exhausted() {
            return Err(GrcError::BudgetExhausted {
                topic: "inline_feasibility".to_string(),
                calls_used: self.inline_feasibility_state.checks_used,
                max_calls: self.inline_feasibility_state.max_checks(),
            });
        }

        // Blocked detection: explicit CONTRADICTS_PLAN: prefix (phase 2D convention).
        // Full semantic contradiction detection deferred to phase 4F.
        let outcome = if let Some(stripped) = claim.strip_prefix("CONTRADICTS_PLAN:") {
            let stripped = stripped.trim();
            // Only trigger Blocked if there are sources to contradict against,
            // or if the prefix is present (spec: prefix is the signal).
            if !stripped.is_empty() {
                FeasibilityOutcome::Blocked
            } else {
                FeasibilityOutcome::Fail
            }
        } else {
            // Source matching: check if the trimmed claim appears as a substring
            // in any provided source. This is a simple contains() check — more
            // sophisticated matching (word-count thresholds, fuzzy match) is
            // deferred to phase 4F per the spec.
            let claim_trimmed = claim.trim();
            let found = sources.iter().any(|source| source.contains(claim_trimmed));

            if found {
                FeasibilityOutcome::Pass
            } else {
                FeasibilityOutcome::Fail
            }
        };

        // Record the check against the budget.
        self.inline_feasibility_state.record_check();

        Ok(InlineFeasibilityResult {
            claim: claim.to_string(),
            method: FeasibilityMethod::SourceMatch,
            outcome,
        })
    }
}

// ---------------------------------------------------------------------------
// D-3: intrinsic-signal extraction helper
// ---------------------------------------------------------------------------

/// Lightweight inference-output bundle consumed by [`extract_intrinsic_signals`].
///
/// This struct deliberately mirrors the four scalars the multi-verifier
/// consensus aggregator needs (D-3 `IntrinsicSignals`) without taking a
/// dependency on `elai-orchestrator` (the orchestrator depends on
/// `elai-reasoning`, not the other way around — pulling the orchestrator
/// type in here would create a cycle).
///
/// Producer wiring (sidecar streaming / local generation) populates this
/// struct from the actual forward-pass telemetry. For sidecar-served
/// inferences `kl_to_base` stays `0.0` until steering is wired.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InferenceOutput {
    /// Mean per-token log-probability of the produced chain.
    pub logprob_mean: f32,
    /// Variance of per-token log-probability across the chain.
    pub logprob_var: f32,
    /// Mean predictive entropy across positions (nats).
    pub entropy: f32,
    /// Number of reasoning steps the generator emitted.
    pub step_count: u32,
    /// KL divergence vs. the policy-mode baseline. `0.0` when no steering /
    /// contrast measurement is configured.
    pub kl_to_base: f32,
}

/// Extracted intrinsic signals. Mirrors `orchestrator::verification_result::
/// IntrinsicSignals` but lives in the reasoning crate (zero cross-crate dep
/// — orchestrator consumes from here, not the reverse).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtractedIntrinsicSignals {
    pub logprob_mean: f32,
    pub logprob_var: f32,
    pub entropy: f32,
    pub step_count: u32,
    pub kl_to_base: f32,
}

/// Project an [`InferenceOutput`] into the scalar bundle the D-3
/// verification aggregator consumes. Pure function — no I/O, no clock, no
/// allocation. Production callers run this once per candidate before
/// passing the result into `verification_result::IntrinsicSignals` (the
/// orchestrator-side type conversion is a single struct-literal copy).
///
/// NaN policy: NaN scalars in `InferenceOutput` are coerced to neutral
/// values (`logprob_mean = -10.0` so the intrinsic verdict registers as
/// "fail" rather than silently passing; `entropy = f32::INFINITY` for the
/// same reason). This matches the V-STaR §3.1 recommendation that
/// uninterpretable telemetry should fail the gate, never pass it.
#[must_use]
pub fn extract_intrinsic_signals(output: &InferenceOutput) -> ExtractedIntrinsicSignals {
    let logprob_mean = if output.logprob_mean.is_nan() {
        -10.0
    } else {
        output.logprob_mean
    };
    let logprob_var = if output.logprob_var.is_nan() {
        0.0
    } else {
        output.logprob_var
    };
    let entropy = if output.entropy.is_nan() {
        f32::INFINITY
    } else {
        output.entropy
    };
    let kl_to_base = if output.kl_to_base.is_nan() {
        0.0
    } else {
        output.kl_to_base
    };
    ExtractedIntrinsicSignals {
        logprob_mean,
        logprob_var,
        entropy,
        step_count: output.step_count,
        kl_to_base,
    }
}

#[cfg(test)]
mod d3_intrinsic_tests {
    use super::*;

    #[test]
    fn extract_passthrough_on_clean_input() {
        let out = InferenceOutput {
            logprob_mean: -1.2,
            logprob_var: 0.4,
            entropy: 2.1,
            step_count: 4,
            kl_to_base: 0.0,
        };
        let s = extract_intrinsic_signals(&out);
        assert_eq!(s.logprob_mean, -1.2);
        assert_eq!(s.logprob_var, 0.4);
        assert_eq!(s.entropy, 2.1);
        assert_eq!(s.step_count, 4);
        assert_eq!(s.kl_to_base, 0.0);
    }

    #[test]
    fn extract_coerces_nan_logprob_to_failing_floor() {
        let out = InferenceOutput {
            logprob_mean: f32::NAN,
            logprob_var: 0.0,
            entropy: 1.0,
            step_count: 1,
            kl_to_base: 0.0,
        };
        let s = extract_intrinsic_signals(&out);
        assert_eq!(s.logprob_mean, -10.0);
    }

    #[test]
    fn extract_coerces_nan_entropy_to_infinity() {
        let out = InferenceOutput {
            logprob_mean: -1.0,
            logprob_var: 0.1,
            entropy: f32::NAN,
            step_count: 1,
            kl_to_base: 0.0,
        };
        let s = extract_intrinsic_signals(&out);
        assert!(s.entropy.is_infinite());
    }
}
