// ---------------------------------------------------------------------------
// Pipeline Tree Navigator — 4E.7-4E.10
//
// Traverses a ConceptDecomposition tree depth-first, tracks execution state,
// handles failures via bubble-up policy, and records integration checkpoints.
// ---------------------------------------------------------------------------

use std::collections::{BTreeMap, BTreeSet};

use elai_web_validator::WebValidationTier;
use serde::{Deserialize, Serialize};

use crate::research_tree::{
    BabyStepId, BabyStepSpec, ConceptDecomposition, ConceptId, ConceptNode, ExitCriterion,
    IntegrationSpec, PhaseId, PhaseNode, StepId, StepNode,
};

// ---------------------------------------------------------------------------
// Named constants — bubble-up policy defaults (no magic numbers)
// ---------------------------------------------------------------------------

/// Max retries at baby-step level before escalating to step level.
pub const DEFAULT_MAX_RETRIES_BABY_STEP: u32 = 3;
/// Max retries at step level before escalating to phase level.
pub const DEFAULT_MAX_RETRIES_STEP: u32 = 2;
/// Max retries at phase level before escalating to concept level.
pub const DEFAULT_MAX_RETRIES_PHASE: u32 = 1;
/// Max retries at concept level before full escalation.
pub const DEFAULT_MAX_RETRIES_CONCEPT: u32 = 1;
/// Max lines modified per baby step before triggering RedecomposeStep.
pub const MAX_BABY_STEP_LINES_MODIFIED: usize = 400;

/// Per-criterion command timeout for IntegrationRunner (seconds).
pub const INTEGRATION_CRITERION_TIMEOUT_SECS: u64 = 30;
/// Sentinel BabyStepId used in IntegrationCheckpoint records at non-BabyStep levels.
pub const NO_BABY_STEP: BabyStepId = BabyStepId(0);

// ---------------------------------------------------------------------------
// WEB.3 — Web validation tier policy by decomposition level.
//
// Returns the deterministic set of [`WebValidationTier`]s the orchestrator
// must evaluate at each level when the active concept has a web template
// scaffolded. Order matters: cheaper checks first.
// ---------------------------------------------------------------------------

/// Returns the ordered tiers to run at `level` for a web-template node.
///
/// - BabyStep: WebBuild only (cheapest gate, catches compile/transpile errors).
/// - Step:     WebBuild + WebLint (advisory lint after build success).
/// - Phase:    WebBuild + WebDevServer (runtime probe after build).
/// - Concept:  WebBuild + WebDevServer + WebLint (full preview-promotion check).
pub fn web_validation_tiers_for_level(level: DecompositionLevel) -> Vec<WebValidationTier> {
    match level {
        DecompositionLevel::BabyStep => vec![WebValidationTier::WebBuild],
        DecompositionLevel::Step => vec![WebValidationTier::WebBuild, WebValidationTier::WebLint],
        DecompositionLevel::Phase => {
            vec![WebValidationTier::WebBuild, WebValidationTier::WebDevServer]
        }
        DecompositionLevel::Concept => vec![
            WebValidationTier::WebBuild,
            WebValidationTier::WebDevServer,
            WebValidationTier::WebLint,
        ],
    }
}

// ---------------------------------------------------------------------------
// 4E.7 — Skeleton types
// ---------------------------------------------------------------------------

/// Decomposition level for bubble-up tracking. `Ord` derive is load-bearing:
/// `NavigatorTree::integration_specs` keys on `(DecompositionLevel, u64)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DecompositionLevel {
    BabyStep,
    Step,
    Phase,
    Concept,
}

/// Actions the bubble-up handler can take on failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BubbleUpAction {
    /// Re-research with alternative approach at current level.
    ResearchAlternative { baby_step_id: BabyStepId },
    /// Step is too large — re-decompose into smaller baby steps.
    RedecomposeStep { step_id: StepId },
    /// Replan the entire phase with different approach.
    ReplanPhase { phase_id: PhaseId },
    /// Replan at concept level.
    ReplanConcept { concept_id: ConceptId },
    /// Diagnose the failure and retry with adjusted parameters.
    DiagnoseAndRetry { baby_step_id: BabyStepId, diagnosis: String },
    /// All retries exhausted — escalate to user/supervisor.
    Escalate { context: String },
}

/// Policy controlling when failures bubble up at each decomposition level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BubbleUpPolicy {
    pub max_retries_baby_step: u32,
    pub max_retries_step: u32,
    pub max_retries_phase: u32,
    pub max_retries_concept: u32,
}

impl Default for BubbleUpPolicy {
    fn default() -> Self {
        Self {
            max_retries_baby_step: DEFAULT_MAX_RETRIES_BABY_STEP,
            max_retries_step: DEFAULT_MAX_RETRIES_STEP,
            max_retries_phase: DEFAULT_MAX_RETRIES_PHASE,
            max_retries_concept: DEFAULT_MAX_RETRIES_CONCEPT,
        }
    }
}

/// Outcome of executing a single baby step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BabyStepOutcome {
    /// All acceptance criteria met.
    Passed,
    /// Tests failed or acceptance criteria not met.
    Failed { reason: String },
    /// Skipped due to pre-gate failure or dependency issue.
    Skipped { reason: String },
    /// Blocked by unmet dependency.
    Blocked { blocked_by: BabyStepId },
}

/// Result of executing a baby step (outcome + metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabyStepResult {
    pub baby_step_id: BabyStepId,
    pub outcome: BabyStepOutcome,
    pub lines_modified: usize,
    pub retry_count: u32,
}

/// A recorded failure for bubble-up tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    pub baby_step_id: BabyStepId,
    pub step_id: StepId,
    pub phase_id: PhaseId,
    pub concept_id: ConceptId,
    pub reason: String,
    pub attempts: u32,
}

/// Bubble-up event emitted when failure threshold is reached.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleUpEvent {
    pub level: DecompositionLevel,
    pub action: BubbleUpAction,
    pub failure_count: u32,
    pub threshold: u32,
}

/// Integration checkpoint recorded when a node with IntegrationSpec completes.
///
/// At `BabyStep` level, `baby_step_id` is the step's ID and `node_id == baby_step_id.0`.
/// At `Step`/`Phase`/`Concept` levels, `baby_step_id == NO_BABY_STEP` and `node_id`
/// carries the step/phase/concept id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCheckpoint {
    pub baby_step_id: BabyStepId,
    pub level: DecompositionLevel,
    pub wiring_target: String,
    pub callers: Vec<String>,
    pub timestamp_epoch_ms: u64,
    pub node_id: u64,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub duration_ms: u64,
}

/// Composite key for looking up parent IDs of a baby step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BabyStepLocation {
    pub concept_id: ConceptId,
    pub phase_id: PhaseId,
    pub step_id: StepId,
    pub baby_step_id: BabyStepId,
}

/// Navigator state — all mutable execution tracking.
///
/// Failure counters are split by source (baby-step cascade vs. integration
/// checkpoint) so that integration-triggered bubble-up at one level cannot
/// silently overcount baby-step cascades at the same level. Existing baby-step
/// cascade semantics in [`DepthFirstNavigator::handle_failure`] are preserved
/// against `*_failure_counts`; 4E.11 integration execution adds separate
/// `*_integration_failure_counts` and never mutates the cascade counters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNavigatorState {
    pub completed: BTreeSet<BabyStepId>,
    pub failed: BTreeSet<BabyStepId>,
    pub blocked: BTreeSet<BabyStepId>,
    pub skipped: BTreeSet<BabyStepId>,
    pub execution_log: Vec<BabyStepResult>,
    pub bubble_up_log: Vec<BubbleUpEvent>,
    pub integration_checkpoints: Vec<IntegrationCheckpoint>,
    pub failure_records: BTreeMap<BabyStepId, FailureRecord>,
    /// Baby-step cascade failure counts (4E.8 semantics — driven by
    /// [`DepthFirstNavigator::handle_failure`]).
    pub step_failure_counts: BTreeMap<StepId, u32>,
    pub phase_failure_counts: BTreeMap<PhaseId, u32>,
    pub concept_failure_counts: BTreeMap<ConceptId, u32>,
    /// Integration-checkpoint failure counts (4E.11 — incremented only by the
    /// `bubble_up_on_*_integration_fail` helpers). Separate from cascade
    /// counters so policy thresholds are never overcounted by mixed sources.
    #[serde(default)]
    pub step_integration_failure_counts: BTreeMap<StepId, u32>,
    #[serde(default)]
    pub phase_integration_failure_counts: BTreeMap<PhaseId, u32>,
    #[serde(default)]
    pub concept_integration_failure_counts: BTreeMap<ConceptId, u32>,
}

impl TreeNavigatorState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for TreeNavigatorState {
    fn default() -> Self {
        Self {
            completed: BTreeSet::new(),
            failed: BTreeSet::new(),
            blocked: BTreeSet::new(),
            skipped: BTreeSet::new(),
            execution_log: Vec::new(),
            bubble_up_log: Vec::new(),
            integration_checkpoints: Vec::new(),
            failure_records: BTreeMap::new(),
            step_failure_counts: BTreeMap::new(),
            phase_failure_counts: BTreeMap::new(),
            concept_failure_counts: BTreeMap::new(),
            step_integration_failure_counts: BTreeMap::new(),
            phase_integration_failure_counts: BTreeMap::new(),
            concept_integration_failure_counts: BTreeMap::new(),
        }
    }
}

/// The 4E version of the decomposition tree — wraps ConceptDecomposition
/// with traversal order and execution state. NOT the same as
/// `crate::research_tree::DecompositionTree` (the 4D version).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatorTree {
    pub decomposition: ConceptDecomposition,
    pub traversal_order: Vec<BabyStepId>,
    pub location_index: BTreeMap<BabyStepId, BabyStepLocation>,
    pub policy: BubbleUpPolicy,
    /// Integration specs keyed by `(level, node_id as u64)`. Populated by callers
    /// (L2 builder or tests) via `register_integration_spec`. Tree nodes do not
    /// carry IntegrationSpec inline — the navigator stores them in this side table
    /// so the same tree structure supports zero-to-many checkpoints at any level.
    #[serde(default)]
    pub integration_specs: BTreeMap<(DecompositionLevel, u64), IntegrationSpec>,
}

/// Object-safe trait for tree navigation.
///
/// All methods are synchronous and use concrete types only (no generics,
/// no `Self` in return position) to ensure `dyn TreeNavigator` works.
pub trait TreeNavigator {
    /// Returns the next baby step to execute, or None if all complete/blocked.
    fn next_baby_step(&self, state: &TreeNavigatorState) -> Option<BabyStepId>;

    /// Record the result of executing a baby step. Returns an optional
    /// bubble-up action if failure thresholds are exceeded.
    fn record_result(
        &self,
        result: BabyStepResult,
        state: &mut TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> Option<BubbleUpAction>;

    /// Returns true when all baby steps in traversal_order are completed.
    fn is_complete(&self, state: &TreeNavigatorState, tree: &NavigatorTree) -> bool;

    /// Check if all integration dependencies for a baby step are satisfied.
    fn integration_ready(
        &self,
        baby_step_id: BabyStepId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool;
}

/// Depth-first navigator implementation.
/// Stores a copy of traversal_order so it can implement TreeNavigator
/// without needing a reference to NavigatorTree in next_baby_step.
#[derive(Debug, Clone)]
pub struct DepthFirstNavigator {
    traversal_order: Vec<BabyStepId>,
}

// ---------------------------------------------------------------------------
// NavigatorTree construction + traversal order
// ---------------------------------------------------------------------------

impl NavigatorTree {
    /// Build a NavigatorTree from a ConceptDecomposition.
    /// Computes traversal order once at construction (not lazy).
    pub fn new(decomposition: ConceptDecomposition, policy: BubbleUpPolicy) -> Self {
        let (traversal_order, location_index) = compute_traversal_order(&decomposition);
        Self {
            decomposition,
            traversal_order,
            location_index,
            policy,
            integration_specs: BTreeMap::new(),
        }
    }

    /// Register an IntegrationSpec at the given decomposition level / node id.
    /// Replaces any prior registration for the same `(level, node_id)` key.
    /// `node_id` must be the raw u64 of the associated ConceptId / PhaseId /
    /// StepId / BabyStepId.
    pub fn register_integration_spec(
        &mut self,
        level: DecompositionLevel,
        node_id: u64,
        spec: IntegrationSpec,
    ) {
        self.integration_specs.insert((level, node_id), spec);
    }

    /// Look up a previously registered IntegrationSpec.
    pub fn integration_spec(
        &self,
        level: DecompositionLevel,
        node_id: u64,
    ) -> Option<&IntegrationSpec> {
        self.integration_specs.get(&(level, node_id))
    }

    /// Look up the baby step spec by ID.
    pub fn find_baby_step(&self, id: BabyStepId) -> Option<&BabyStepSpec> {
        let loc = self.location_index.get(&id)?;
        let concept = self.decomposition.concepts.iter().find(|c| c.id == loc.concept_id)?;
        let phase = concept.phases.iter().find(|p| p.id == loc.phase_id)?;
        let step = phase.steps.iter().find(|s| s.id == loc.step_id)?;
        step.baby_steps.iter().find(|bs| bs.id == id)
    }

    /// Look up which step a baby step belongs to.
    pub fn find_step_for_baby_step(&self, id: BabyStepId) -> Option<&StepNode> {
        let loc = self.location_index.get(&id)?;
        let concept = self.decomposition.concepts.iter().find(|c| c.id == loc.concept_id)?;
        let phase = concept.phases.iter().find(|p| p.id == loc.phase_id)?;
        phase.steps.iter().find(|s| s.id == loc.step_id)
    }
}

/// Compute depth-first traversal order.
///
/// Order: concepts sorted by topological order (using depends_on),
/// then phases sequential within concept, steps sequential within phase,
/// baby steps sequential within step.
fn compute_traversal_order(
    decomposition: &ConceptDecomposition,
) -> (Vec<BabyStepId>, BTreeMap<BabyStepId, BabyStepLocation>) {
    let mut order = Vec::new();
    let mut index = BTreeMap::new();

    // Topological sort of concepts by depends_on (PhaseNode.depends_on: Vec<ConceptId>).
    // Concepts with no dependencies come first.
    let sorted_concepts = topological_sort_concepts(&decomposition.concepts);

    for concept_id in &sorted_concepts {
        let concept = match decomposition.concepts.iter().find(|c| c.id == *concept_id) {
            Some(c) => c,
            None => continue,
        };
        for phase in &concept.phases {
            for step in &phase.steps {
                for bs in &step.baby_steps {
                    order.push(bs.id);
                    index.insert(
                        bs.id,
                        BabyStepLocation {
                            concept_id: concept.id,
                            phase_id: phase.id,
                            step_id: step.id,
                            baby_step_id: bs.id,
                        },
                    );
                }
            }
        }
    }

    (order, index)
}

/// Topological sort of concepts. Phases have depends_on: Vec<ConceptId>,
/// meaning a phase depends on another concept being complete first.
/// We aggregate all phase dependencies per concept and sort concepts accordingly.
fn topological_sort_concepts(concepts: &[ConceptNode]) -> Vec<ConceptId> {
    // Build adjacency: concept A depends on concept B if any phase in A
    // has B in its depends_on.
    let concept_ids: BTreeSet<ConceptId> = concepts.iter().map(|c| c.id).collect();
    let mut in_degree: BTreeMap<ConceptId, usize> = BTreeMap::new();
    let mut dependents: BTreeMap<ConceptId, Vec<ConceptId>> = BTreeMap::new();

    for c in concepts {
        in_degree.entry(c.id).or_insert(0);
        dependents.entry(c.id).or_insert_with(Vec::new);
    }

    for concept in concepts {
        for phase in &concept.phases {
            for dep_concept_id in &phase.depends_on {
                if concept_ids.contains(dep_concept_id) && *dep_concept_id != concept.id {
                    *in_degree.entry(concept.id).or_insert(0) += 1;
                    dependents
                        .entry(*dep_concept_id)
                        .or_insert_with(Vec::new)
                        .push(concept.id);
                }
            }
        }
    }

    // Kahn's algorithm with BTreeSet for deterministic ordering.
    let mut queue: BTreeSet<ConceptId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut sorted = Vec::with_capacity(concepts.len());

    while let Some(&id) = queue.iter().next() {
        queue.remove(&id);
        sorted.push(id);
        if let Some(deps) = dependents.get(&id) {
            for dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.insert(*dep);
                    }
                }
            }
        }
    }

    // If cycle detected, append remaining in BTree order (deterministic fallback).
    if sorted.len() < concepts.len() {
        for c in concepts {
            if !sorted.contains(&c.id) {
                sorted.push(c.id);
            }
        }
    }

    sorted
}

// ---------------------------------------------------------------------------
// 4E.10 — DepthFirstNavigator implementation
// ---------------------------------------------------------------------------

impl DepthFirstNavigator {
    pub fn new(traversal_order: Vec<BabyStepId>) -> Self {
        Self { traversal_order }
    }

    /// Convenience: build from a NavigatorTree.
    pub fn from_tree(tree: &NavigatorTree) -> Self {
        Self::new(tree.traversal_order.clone())
    }
}

impl TreeNavigator for DepthFirstNavigator {
    fn next_baby_step(&self, state: &TreeNavigatorState) -> Option<BabyStepId> {
        self.traversal_order
            .iter()
            .copied()
            .find(|id| {
                !state.completed.contains(id)
                    && !state.failed.contains(id)
                    && !state.blocked.contains(id)
                    && !state.skipped.contains(id)
            })
    }

    fn record_result(
        &self,
        result: BabyStepResult,
        state: &mut TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> Option<BubbleUpAction> {
        let bs_id = result.baby_step_id;

        // Check lines-modified threshold FIRST (even on pass).
        if result.lines_modified > MAX_BABY_STEP_LINES_MODIFIED {
            if let Some(loc) = tree.location_index.get(&bs_id) {
                // Still mark as completed if it passed.
                if result.outcome == BabyStepOutcome::Passed {
                    state.completed.insert(bs_id);
                }
                state.execution_log.push(result);
                return Some(BubbleUpAction::RedecomposeStep {
                    step_id: loc.step_id,
                });
            }
        }

        // Record to execution log.
        state.execution_log.push(result.clone());

        match &result.outcome {
            BabyStepOutcome::Passed => {
                state.completed.insert(bs_id);
                None
            }
            BabyStepOutcome::Skipped { .. } => {
                state.skipped.insert(bs_id);
                None
            }
            BabyStepOutcome::Blocked { .. } => {
                state.blocked.insert(bs_id);
                None
            }
            BabyStepOutcome::Failed { reason } => {
                self.handle_failure(bs_id, reason, state, tree)
            }
        }
    }

    fn is_complete(&self, state: &TreeNavigatorState, tree: &NavigatorTree) -> bool {
        tree.traversal_order.iter().all(|id| {
            state.completed.contains(id)
                || state.failed.contains(id)
                || state.skipped.contains(id)
                || state.blocked.contains(id)
        })
    }

    fn integration_ready(
        &self,
        baby_step_id: BabyStepId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        let loc = match tree.location_index.get(&baby_step_id) {
            Some(l) => l,
            None => return false,
        };

        // Check all baby steps in dependent concepts are completed.
        // A concept's phases have depends_on: Vec<ConceptId>.
        // Find which concepts this baby step's concept depends on.
        let concept = match tree.decomposition.concepts.iter().find(|c| c.id == loc.concept_id) {
            Some(c) => c,
            None => return false,
        };

        let mut dep_concept_ids = BTreeSet::new();
        for phase in &concept.phases {
            for dep in &phase.depends_on {
                dep_concept_ids.insert(*dep);
            }
        }

        // All baby steps in dependent concepts must be completed.
        for dep_concept_id in &dep_concept_ids {
            if let Some(dep_concept) = tree.decomposition.concepts.iter().find(|c| c.id == *dep_concept_id) {
                for phase in &dep_concept.phases {
                    for step in &phase.steps {
                        for bs in &step.baby_steps {
                            if !state.completed.contains(&bs.id) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }
}

impl DepthFirstNavigator {
    /// Handle a failure: track attempts, evaluate bubble-up thresholds.
    fn handle_failure(
        &self,
        bs_id: BabyStepId,
        reason: &str,
        state: &mut TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> Option<BubbleUpAction> {
        let loc = match tree.location_index.get(&bs_id) {
            Some(l) => l.clone(),
            None => return None,
        };

        // Upsert failure record.
        let record = state.failure_records.entry(bs_id).or_insert_with(|| {
            FailureRecord {
                baby_step_id: bs_id,
                step_id: loc.step_id,
                phase_id: loc.phase_id,
                concept_id: loc.concept_id,
                reason: String::new(),
                attempts: 0,
            }
        });
        record.attempts += 1;
        record.reason = reason.to_string();

        let attempts = record.attempts;

        // Check baby-step threshold.
        if attempts >= tree.policy.max_retries_baby_step {
            // Mark as failed at baby-step level.
            state.failed.insert(bs_id);

            // Emit baby-step level bubble-up event.
            state.bubble_up_log.push(BubbleUpEvent {
                level: DecompositionLevel::BabyStep,
                action: BubbleUpAction::ResearchAlternative { baby_step_id: bs_id },
                failure_count: tree.policy.max_retries_baby_step,
                threshold: tree.policy.max_retries_baby_step,
            });

            // Increment step-level failure count and check cascade.
            {
                let count = state.step_failure_counts.entry(loc.step_id).or_insert(0);
                *count += 1;
            }
            let step_fail_count = *state.step_failure_counts.get(&loc.step_id).unwrap_or(&0);
            let all_step_bs_resolved = self.all_baby_steps_in_step_resolved(loc.step_id, state, tree);

            if step_fail_count >= tree.policy.max_retries_step || (all_step_bs_resolved && step_fail_count > 0) {
                state.bubble_up_log.push(BubbleUpEvent {
                    level: DecompositionLevel::Step,
                    action: BubbleUpAction::RedecomposeStep { step_id: loc.step_id },
                    failure_count: step_fail_count,
                    threshold: tree.policy.max_retries_step,
                });

                // Increment phase-level failure count and check cascade.
                {
                    let count = state.phase_failure_counts.entry(loc.phase_id).or_insert(0);
                    *count += 1;
                }
                let phase_fail_count = *state.phase_failure_counts.get(&loc.phase_id).unwrap_or(&0);
                let all_phase_resolved = self.all_steps_in_phase_resolved(loc.phase_id, state, tree);

                if phase_fail_count >= tree.policy.max_retries_phase || (all_phase_resolved && phase_fail_count > 0) {
                    state.bubble_up_log.push(BubbleUpEvent {
                        level: DecompositionLevel::Phase,
                        action: BubbleUpAction::ReplanPhase { phase_id: loc.phase_id },
                        failure_count: phase_fail_count,
                        threshold: tree.policy.max_retries_phase,
                    });

                    // Increment concept-level failure count and check cascade.
                    {
                        let count = state.concept_failure_counts.entry(loc.concept_id).or_insert(0);
                        *count += 1;
                    }
                    let concept_fail_count = *state.concept_failure_counts.get(&loc.concept_id).unwrap_or(&0);
                    let all_concept_resolved = self.all_phases_in_concept_resolved(loc.concept_id, state, tree);

                    if concept_fail_count >= tree.policy.max_retries_concept || (all_concept_resolved && concept_fail_count > 0) {
                        let escalate_ctx = format!(
                            "All retries exhausted for concept {:?}",
                            loc.concept_id
                        );
                        state.bubble_up_log.push(BubbleUpEvent {
                            level: DecompositionLevel::Concept,
                            action: BubbleUpAction::Escalate {
                                context: escalate_ctx.clone(),
                            },
                            failure_count: concept_fail_count,
                            threshold: tree.policy.max_retries_concept,
                        });

                        return Some(BubbleUpAction::Escalate {
                            context: escalate_ctx,
                        });
                    }

                    return Some(BubbleUpAction::ReplanConcept {
                        concept_id: loc.concept_id,
                    });
                }

                return Some(BubbleUpAction::RedecomposeStep {
                    step_id: loc.step_id,
                });
            }

            return Some(BubbleUpAction::ResearchAlternative {
                baby_step_id: bs_id,
            });
        }

        // Below threshold — suggest diagnose and retry.
        Some(BubbleUpAction::DiagnoseAndRetry {
            baby_step_id: bs_id,
            diagnosis: format!("Failure #{}: {}", attempts, reason),
        })
    }

    /// Check if all baby steps in a step are resolved (completed, failed, or skipped).
    fn all_baby_steps_in_step_resolved(
        &self,
        step_id: StepId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        let bs_ids: Vec<BabyStepId> = tree
            .location_index
            .iter()
            .filter(|(_, loc)| loc.step_id == step_id)
            .map(|(id, _)| *id)
            .collect();
        if bs_ids.is_empty() {
            return false;
        }
        bs_ids.iter().all(|id| {
            state.completed.contains(id)
                || state.failed.contains(id)
                || state.skipped.contains(id)
        })
    }

    /// Check if all steps in a phase are resolved (all their baby steps resolved).
    fn all_steps_in_phase_resolved(
        &self,
        phase_id: PhaseId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        let step_ids: BTreeSet<StepId> = tree
            .location_index
            .iter()
            .filter(|(_, loc)| loc.phase_id == phase_id)
            .map(|(_, loc)| loc.step_id)
            .collect();
        if step_ids.is_empty() {
            return false;
        }
        step_ids
            .iter()
            .all(|sid| self.all_baby_steps_in_step_resolved(*sid, state, tree))
    }

    /// Check if all phases in a concept are resolved.
    fn all_phases_in_concept_resolved(
        &self,
        concept_id: ConceptId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        let phase_ids: BTreeSet<PhaseId> = tree
            .location_index
            .iter()
            .filter(|(_, loc)| loc.concept_id == concept_id)
            .map(|(_, loc)| loc.phase_id)
            .collect();
        if phase_ids.is_empty() {
            return false;
        }
        phase_ids
            .iter()
            .all(|pid| self.all_steps_in_phase_resolved(*pid, state, tree))
    }

    // -----------------------------------------------------------------------
    // 4E.11 — Integration checkpoint execution / bubble-up wiring
    // -----------------------------------------------------------------------

    /// True when every baby step in `step_id` has a terminal outcome.
    /// Thin public wrapper over the existing cascade helper so external
    /// orchestrators (the measure binary, pipeline driver) can ask whether
    /// a step is ready for its integration checkpoint.
    pub fn step_is_terminal(
        &self,
        step_id: StepId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        self.all_baby_steps_in_step_resolved(step_id, state, tree)
    }

    /// True when every step in `phase_id` is terminal.
    pub fn phase_is_terminal(
        &self,
        phase_id: PhaseId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        self.all_steps_in_phase_resolved(phase_id, state, tree)
    }

    /// True when every phase in `concept_id` is terminal.
    pub fn concept_is_terminal(
        &self,
        concept_id: ConceptId,
        state: &TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> bool {
        self.all_phases_in_concept_resolved(concept_id, state, tree)
    }

    /// Run the step-level integration checkpoint registered on `tree` for
    /// `step_id` (if any). On aggregate failure, increments
    /// `step_failure_counts`, emits a Step-level BubbleUpEvent, and returns the
    /// resulting BubbleUpAction. Returns `Ok(None, None)` if no spec is
    /// registered (silently skipped — matches 4E.8 "no integration, no
    /// checkpoint" semantics).
    pub fn run_step_integration(
        &self,
        step_id: StepId,
        runner: &dyn IntegrationRunner,
        state: &mut TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> (Option<IntegrationRunResult>, Option<BubbleUpAction>) {
        let spec = match tree.integration_spec(DecompositionLevel::Step, step_id.0) {
            Some(s) => s.clone(),
            None => return (None, None),
        };
        let result = run_integration_checkpoint(
            DecompositionLevel::Step,
            step_id.0,
            &spec,
            runner,
            state,
        );
        let action = if result.success {
            None
        } else {
            Some(bubble_up_on_step_integration_fail(
                step_id,
                result.failure_reason.clone(),
                state,
                tree,
            ))
        };
        (Some(result), action)
    }

    /// Run the phase-level integration checkpoint registered on `tree` for
    /// `phase_id` (if any). See [`Self::run_step_integration`] for semantics.
    pub fn run_phase_integration(
        &self,
        phase_id: PhaseId,
        runner: &dyn IntegrationRunner,
        state: &mut TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> (Option<IntegrationRunResult>, Option<BubbleUpAction>) {
        let spec = match tree.integration_spec(DecompositionLevel::Phase, phase_id.0) {
            Some(s) => s.clone(),
            None => return (None, None),
        };
        let result = run_integration_checkpoint(
            DecompositionLevel::Phase,
            phase_id.0,
            &spec,
            runner,
            state,
        );
        let action = if result.success {
            None
        } else {
            Some(bubble_up_on_phase_integration_fail(
                phase_id,
                result.failure_reason.clone(),
                state,
                tree,
            ))
        };
        (Some(result), action)
    }

    /// Run the concept-level integration checkpoint registered on `tree` for
    /// `concept_id` (if any). See [`Self::run_step_integration`] for semantics.
    pub fn run_concept_integration(
        &self,
        concept_id: ConceptId,
        runner: &dyn IntegrationRunner,
        state: &mut TreeNavigatorState,
        tree: &NavigatorTree,
    ) -> (Option<IntegrationRunResult>, Option<BubbleUpAction>) {
        let spec = match tree.integration_spec(DecompositionLevel::Concept, concept_id.0) {
            Some(s) => s.clone(),
            None => return (None, None),
        };
        let result = run_integration_checkpoint(
            DecompositionLevel::Concept,
            concept_id.0,
            &spec,
            runner,
            state,
        );
        let action = if result.success {
            None
        } else {
            Some(bubble_up_on_concept_integration_fail(
                concept_id,
                result.failure_reason.clone(),
                state,
                tree,
            ))
        };
        (Some(result), action)
    }
}

/// Increment the step failure counter and emit a step-level BubbleUpEvent for
/// a failed integration check. The returned action is `RedecomposeStep` when
/// the counter is still under the step threshold, or cascades to phase level
/// via [`bubble_up_on_phase_integration_fail`] once the step threshold is hit.
fn bubble_up_on_step_integration_fail(
    step_id: StepId,
    reason: Option<String>,
    state: &mut TreeNavigatorState,
    tree: &NavigatorTree,
) -> BubbleUpAction {
    let count = state.step_integration_failure_counts.entry(step_id).or_insert(0);
    *count += 1;
    let failure_count = *count;
    state.bubble_up_log.push(BubbleUpEvent {
        level: DecompositionLevel::Step,
        action: BubbleUpAction::RedecomposeStep { step_id },
        failure_count,
        threshold: tree.policy.max_retries_step,
    });

    if failure_count >= tree.policy.max_retries_step {
        if let Some(phase_id) = phase_for_step(step_id, tree) {
            return bubble_up_on_phase_integration_fail(phase_id, reason, state, tree);
        }
    }
    BubbleUpAction::RedecomposeStep { step_id }
}

/// Phase-level counterpart of [`bubble_up_on_step_integration_fail`].
fn bubble_up_on_phase_integration_fail(
    phase_id: PhaseId,
    reason: Option<String>,
    state: &mut TreeNavigatorState,
    tree: &NavigatorTree,
) -> BubbleUpAction {
    let count = state.phase_integration_failure_counts.entry(phase_id).or_insert(0);
    *count += 1;
    let failure_count = *count;
    state.bubble_up_log.push(BubbleUpEvent {
        level: DecompositionLevel::Phase,
        action: BubbleUpAction::ReplanPhase { phase_id },
        failure_count,
        threshold: tree.policy.max_retries_phase,
    });

    if failure_count >= tree.policy.max_retries_phase {
        if let Some(concept_id) = concept_for_phase(phase_id, tree) {
            return bubble_up_on_concept_integration_fail(concept_id, reason, state, tree);
        }
    }
    BubbleUpAction::ReplanPhase { phase_id }
}

/// Concept-level counterpart of [`bubble_up_on_step_integration_fail`].
/// Cascades to `Escalate` once the concept threshold is exceeded.
fn bubble_up_on_concept_integration_fail(
    concept_id: ConceptId,
    reason: Option<String>,
    state: &mut TreeNavigatorState,
    tree: &NavigatorTree,
) -> BubbleUpAction {
    let count = state.concept_integration_failure_counts.entry(concept_id).or_insert(0);
    *count += 1;
    let failure_count = *count;

    if failure_count >= tree.policy.max_retries_concept {
        let context = match reason {
            Some(r) => format!(
                "Concept {:?} integration exhausted after {} failures: {}",
                concept_id, failure_count, r
            ),
            None => format!(
                "Concept {:?} integration exhausted after {} failures",
                concept_id, failure_count
            ),
        };
        state.bubble_up_log.push(BubbleUpEvent {
            level: DecompositionLevel::Concept,
            action: BubbleUpAction::Escalate {
                context: context.clone(),
            },
            failure_count,
            threshold: tree.policy.max_retries_concept,
        });
        return BubbleUpAction::Escalate { context };
    }

    state.bubble_up_log.push(BubbleUpEvent {
        level: DecompositionLevel::Concept,
        action: BubbleUpAction::ReplanConcept { concept_id },
        failure_count,
        threshold: tree.policy.max_retries_concept,
    });
    BubbleUpAction::ReplanConcept { concept_id }
}

fn phase_for_step(step_id: StepId, tree: &NavigatorTree) -> Option<PhaseId> {
    tree.location_index
        .values()
        .find(|loc| loc.step_id == step_id)
        .map(|loc| loc.phase_id)
}

fn concept_for_phase(phase_id: PhaseId, tree: &NavigatorTree) -> Option<ConceptId> {
    tree.location_index
        .values()
        .find(|loc| loc.phase_id == phase_id)
        .map(|loc| loc.concept_id)
}

/// Record an integration checkpoint for a completed baby step.
///
/// Preserved from 4E.8 for backward compatibility with the baby-step checkpoint
/// contract. `success` is recorded `true` by default because this API records
/// the checkpoint metadata only — use [`run_integration_checkpoint`] (4E.11) to
/// actually execute an IntegrationRunner and capture pass/fail outcomes at any
/// decomposition level.
pub fn record_integration_checkpoint(
    baby_step_id: BabyStepId,
    spec: &IntegrationSpec,
    state: &mut TreeNavigatorState,
) {
    let timestamp = current_unix_epoch_ms();

    state.integration_checkpoints.push(IntegrationCheckpoint {
        baby_step_id,
        level: DecompositionLevel::BabyStep,
        wiring_target: spec.wiring_target.clone(),
        callers: spec.callers.clone(),
        timestamp_epoch_ms: timestamp,
        node_id: baby_step_id.0,
        success: true,
        failure_reason: None,
        duration_ms: 0,
    });
}

/// Wall-clock milliseconds since the Unix epoch. Returns 0 on pre-epoch clocks
/// and u64::MAX on clocks beyond year 584M AD — both edge cases are preserved
/// from the 4E.8 implementation and intentionally keep this helper panic-free.
fn current_unix_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// 4E.11 — Integration checkpoint EXECUTION (baby-step / step / phase / concept)
// ---------------------------------------------------------------------------

/// Outcome of a single exit-criterion run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriterionResult {
    pub description: String,
    pub success: bool,
    pub reason: Option<String>,
    pub duration_ms: u64,
}

/// Aggregate outcome of running all exit criteria for one IntegrationSpec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRunResult {
    pub level: DecompositionLevel,
    pub node_id: u64,
    pub success: bool,
    pub criterion_results: Vec<CriterionResult>,
    pub failure_reason: Option<String>,
    pub total_duration_ms: u64,
}

/// Object-safe runner that executes the `check_command` of each `ExitCriterion`.
/// Implementations decide whether to shell out, mock a result, or consult an
/// external system. The navigator never runs commands itself.
pub trait IntegrationRunner: Send + Sync {
    /// Run a single exit criterion. Return `success=true` if it met the bar.
    /// Must be panic-free — any runtime error should surface as `success=false`
    /// with a populated `reason`.
    fn run_criterion(
        &self,
        level: DecompositionLevel,
        node_id: u64,
        criterion: &ExitCriterion,
    ) -> CriterionResult;
}

/// No-op runner that reports every criterion as passing. Intended for pipeline
/// dry-runs and unit tests — production callers must inject a real runner.
#[derive(Debug, Clone, Default)]
pub struct NoOpIntegrationRunner;

impl IntegrationRunner for NoOpIntegrationRunner {
    fn run_criterion(
        &self,
        _level: DecompositionLevel,
        _node_id: u64,
        criterion: &ExitCriterion,
    ) -> CriterionResult {
        CriterionResult {
            description: criterion.description.clone(),
            success: true,
            reason: None,
            duration_ms: 0,
        }
    }
}

/// Runner that shells out via [`std::process::Command`] with
/// [`INTEGRATION_CRITERION_TIMEOUT_SECS`] per-criterion wall-clock budget.
/// Criteria without a `check_command` are treated as passing (informational).
///
/// Uses `cmd /C` on Windows and `sh -c` elsewhere. Executes synchronously; the
/// caller is responsible for running this off the async executor if blocking
/// matters.
#[derive(Debug, Clone, Default)]
pub struct CommandIntegrationRunner {
    working_dir: Option<std::path::PathBuf>,
}

impl CommandIntegrationRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_working_dir(dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            working_dir: Some(dir.into()),
        }
    }
}

/// Polling interval used when waiting for a spawned criterion command to exit.
/// Balances responsiveness (sub-100ms wake-ups on completion) against wasted
/// CPU when criteria take minutes (e.g. `cargo test`).
const INTEGRATION_CRITERION_POLL_MS: u64 = 50;

impl IntegrationRunner for CommandIntegrationRunner {
    fn run_criterion(
        &self,
        _level: DecompositionLevel,
        _node_id: u64,
        criterion: &ExitCriterion,
    ) -> CriterionResult {
        let command = match criterion.check_command.as_deref() {
            Some(c) if !c.trim().is_empty() => c,
            _ => {
                return CriterionResult {
                    description: criterion.description.clone(),
                    success: true,
                    reason: Some("no check_command — treated as informational".to_string()),
                    duration_ms: 0,
                };
            }
        };

        let start = std::time::Instant::now();
        let mut cmd = if cfg!(windows) {
            let mut c = std::process::Command::new("cmd");
            c.arg("/C").arg(command);
            c
        } else {
            let mut c = std::process::Command::new("sh");
            c.arg("-c").arg(command);
            c
        };
        if let Some(dir) = self.working_dir.as_ref() {
            cmd.current_dir(dir);
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return CriterionResult {
                    description: criterion.description.clone(),
                    success: false,
                    reason: Some(format!("spawn error: {e}")),
                    duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                };
            }
        };

        let deadline =
            start + std::time::Duration::from_secs(INTEGRATION_CRITERION_TIMEOUT_SECS);
        let poll = std::time::Duration::from_millis(INTEGRATION_CRITERION_POLL_MS);

        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break Ok(s),
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        // Kill the runaway and collect its exit status so no
                        // zombie process remains. `kill` is best-effort: if it
                        // fails (e.g. process already gone), we still wait().
                        let _ = child.kill();
                        let _ = child.wait();
                        return CriterionResult {
                            description: criterion.description.clone(),
                            success: false,
                            reason: Some(format!(
                                "timeout after {}s",
                                INTEGRATION_CRITERION_TIMEOUT_SECS
                            )),
                            duration_ms: u64::try_from(start.elapsed().as_millis())
                                .unwrap_or(u64::MAX),
                        };
                    }
                    std::thread::sleep(poll);
                }
                Err(e) => break Err(e),
            }
        };

        let output = match status {
            Ok(_) => match child.wait_with_output() {
                Ok(o) => o,
                Err(e) => {
                    return CriterionResult {
                        description: criterion.description.clone(),
                        success: false,
                        reason: Some(format!("wait_with_output: {e}")),
                        duration_ms: u64::try_from(start.elapsed().as_millis())
                            .unwrap_or(u64::MAX),
                    };
                }
            },
            Err(e) => {
                return CriterionResult {
                    description: criterion.description.clone(),
                    success: false,
                    reason: Some(format!("try_wait: {e}")),
                    duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                };
            }
        };

        let duration_ms =
            u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let success = output.status.success();
        let reason = if success {
            None
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let code = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".to_string());
            Some(format!("exit={code}: {}", stderr.trim()))
        };

        CriterionResult {
            description: criterion.description.clone(),
            success,
            reason,
            duration_ms,
        }
    }
}

/// Execute every criterion in `spec` and append an IntegrationCheckpoint to
/// `state.integration_checkpoints`. Short-circuits on the first failure so
/// downstream criteria are not billed when an earlier one already fails the
/// aggregate. All recorded state is deterministic for a given runner output.
pub fn run_integration_checkpoint(
    level: DecompositionLevel,
    node_id: u64,
    spec: &IntegrationSpec,
    runner: &dyn IntegrationRunner,
    state: &mut TreeNavigatorState,
) -> IntegrationRunResult {
    let start = std::time::Instant::now();
    let mut criterion_results: Vec<CriterionResult> = Vec::with_capacity(spec.exit_criteria.len());
    let mut aggregate_success = true;
    let mut first_failure: Option<String> = None;

    for criterion in &spec.exit_criteria {
        let result = runner.run_criterion(level, node_id, criterion);
        if !result.success && aggregate_success {
            aggregate_success = false;
            first_failure = Some(match &result.reason {
                Some(r) => format!("{}: {}", criterion.description, r),
                None => criterion.description.clone(),
            });
        }
        criterion_results.push(result);
        if !aggregate_success {
            break;
        }
    }

    let total_duration_ms =
        u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let timestamp_epoch_ms = current_unix_epoch_ms();
    let baby_step_id = if matches!(level, DecompositionLevel::BabyStep) {
        BabyStepId(node_id)
    } else {
        NO_BABY_STEP
    };

    state.integration_checkpoints.push(IntegrationCheckpoint {
        baby_step_id,
        level,
        wiring_target: spec.wiring_target.clone(),
        callers: spec.callers.clone(),
        timestamp_epoch_ms,
        node_id,
        success: aggregate_success,
        failure_reason: first_failure.clone(),
        duration_ms: total_duration_ms,
    });

    IntegrationRunResult {
        level,
        node_id,
        success: aggregate_success,
        criterion_results,
        failure_reason: first_failure,
        total_duration_ms,
    }
}

// ---------------------------------------------------------------------------
// Tests — 4E.8 navigation + bubble-up + checkpoint, 4E.9 execution cycle
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_tree::{
        BabyStepGrounding, ExecutionMode, ExitCriterion, SourceCitation,
    };

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn make_source_citation() -> SourceCitation {
        SourceCitation {
            url: "https://example.com".to_string(),
            title: "test".to_string(),
            quality: crate::research_tree::SourceQuality::Tier1,
            evidence_level: crate::research_tree::EvidenceLevel::Anecdotal,
            excerpt: "test excerpt".to_string(),
        }
    }

    fn make_grounding() -> BabyStepGrounding {
        BabyStepGrounding {
            concept_name: "test_concept".to_string(),
            phase_name: "test_phase".to_string(),
            step_name: "test_step".to_string(),
            top_citations: vec![make_source_citation()],
            compressed_tokens: 100,
        }
    }

    fn make_baby_step(id: u64) -> BabyStepSpec {
        BabyStepSpec {
            id: BabyStepId(id),
            task: format!("baby step {}", id),
            estimated_tokens: 500,
            grounding: make_grounding(),
            tdd: None,
            acceptance: None,
        }
    }

    fn make_step(step_id: u64, baby_step_ids: &[u64]) -> StepNode {
        StepNode {
            id: StepId(step_id),
            description: format!("step {}", step_id),
            baby_steps: baby_step_ids.iter().map(|&id| make_baby_step(id)).collect(),
            evidence_level: crate::research_tree::EvidenceLevel::Anecdotal,
        }
    }

    fn make_phase(phase_id: u64, steps: Vec<StepNode>, depends_on: Vec<ConceptId>) -> PhaseNode {
        PhaseNode {
            id: PhaseId(phase_id),
            name: format!("phase {}", phase_id),
            steps,
            depends_on,
            research: None,
            is_parallel: false,
        }
    }

    fn make_concept(concept_id: u64, phases: Vec<PhaseNode>) -> ConceptNode {
        ConceptNode {
            id: ConceptId(concept_id),
            name: format!("concept {}", concept_id),
            description: format!("description for concept {}", concept_id),
            phases,
            research: None,
        }
    }

    fn make_decomposition(concepts: Vec<ConceptNode>) -> ConceptDecomposition {
        ConceptDecomposition {
            idea: "test idea".to_string(),
            concepts,
            mode: ExecutionMode::Autonomous,
            total_research_calls: 0,
        }
    }

    /// Build a simple tree: 1 concept, 1 phase, 1 step, N baby steps.
    fn simple_tree(baby_step_count: u64) -> NavigatorTree {
        let ids: Vec<u64> = (1..=baby_step_count).collect();
        let step = make_step(1, &ids);
        let phase = make_phase(1, vec![step], vec![]);
        let concept = make_concept(1, vec![phase]);
        let decomp = make_decomposition(vec![concept]);
        NavigatorTree::new(decomp, BubbleUpPolicy::default())
    }

    /// Build a tree with 2 concepts where concept 2 depends on concept 1.
    fn two_concept_tree() -> NavigatorTree {
        let step1 = make_step(1, &[1, 2]);
        let phase1 = make_phase(1, vec![step1], vec![]);
        let concept1 = make_concept(1, vec![phase1]);

        let step2 = make_step(2, &[3, 4]);
        // Phase 2 depends on concept 1
        let phase2 = make_phase(2, vec![step2], vec![ConceptId(1)]);
        let concept2 = make_concept(2, vec![phase2]);

        let decomp = make_decomposition(vec![concept2, concept1]); // intentionally reversed
        NavigatorTree::new(decomp, BubbleUpPolicy::default())
    }

    // -----------------------------------------------------------------------
    // 4E.8 — Navigation tests (6 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn test_traversal_order_correctness() {
        let tree = simple_tree(3);
        assert_eq!(
            tree.traversal_order,
            vec![BabyStepId(1), BabyStepId(2), BabyStepId(3)],
            "Traversal order should be sequential within a single concept/phase/step"
        );
    }

    #[test]
    fn test_next_baby_step_returns_none_when_complete() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.completed.insert(BabyStepId(1));
        state.completed.insert(BabyStepId(2));
        assert_eq!(
            nav.next_baby_step(&state),
            None,
            "Should return None when all baby steps completed"
        );
    }

    #[test]
    fn test_skipped_steps_excluded_from_traversal() {
        let tree = simple_tree(3);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.skipped.insert(BabyStepId(1));
        // Next should skip 1 and return 2
        let next = nav.next_baby_step(&state);
        assert_eq!(next, Some(BabyStepId(2)), "Should skip baby step 1 and return 2");
    }

    #[test]
    fn test_blocked_steps_excluded_from_traversal() {
        let tree = simple_tree(3);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.blocked.insert(BabyStepId(1));
        let next = nav.next_baby_step(&state);
        assert_eq!(next, Some(BabyStepId(2)), "Should skip blocked baby step 1");
    }

    #[test]
    fn test_concept_dependency_respected_in_traversal() {
        let tree = two_concept_tree();
        // Concept 1 (baby steps 1,2) should come before concept 2 (baby steps 3,4)
        // even though concept 2 was listed first in the Vec.
        assert_eq!(tree.traversal_order[0], BabyStepId(1));
        assert_eq!(tree.traversal_order[1], BabyStepId(2));
        assert_eq!(tree.traversal_order[2], BabyStepId(3));
        assert_eq!(tree.traversal_order[3], BabyStepId(4));
    }

    #[test]
    fn test_single_concept_path() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let state = TreeNavigatorState::new();
        let next = nav.next_baby_step(&state);
        assert_eq!(next, Some(BabyStepId(1)), "Single baby step should be returned");
    }

    // -----------------------------------------------------------------------
    // 4E.8 — Bubble-up tests (6 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn test_baby_step_failure_threshold_triggers_bubble_up() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Fail baby step 1 three times (DEFAULT_MAX_RETRIES_BABY_STEP = 3)
        for i in 0..DEFAULT_MAX_RETRIES_BABY_STEP {
            let result = BabyStepResult {
                baby_step_id: BabyStepId(1),
                outcome: BabyStepOutcome::Failed {
                    reason: format!("fail {}", i),
                },
                lines_modified: 10,
                retry_count: i,
            };
            nav.record_result(result, &mut state, &tree);
        }

        // After 3 failures, bubble_up_log should have an entry
        assert!(
            !state.bubble_up_log.is_empty(),
            "Bubble-up log should have entry after {} failures at baby-step level",
            DEFAULT_MAX_RETRIES_BABY_STEP
        );
        assert_eq!(state.bubble_up_log[0].level, DecompositionLevel::BabyStep);
    }

    #[test]
    fn test_step_failure_threshold_triggers_bubble_up() {
        let step = make_step(1, &[1, 2]);
        let phase = make_phase(1, vec![step], vec![]);
        let concept = make_concept(1, vec![phase]);
        let decomp = make_decomposition(vec![concept]);
        let tree = NavigatorTree::new(decomp, BubbleUpPolicy::default());
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Exhaust baby step 1 retries (triggers baby-step bubble-up -> step level)
        for _ in 0..DEFAULT_MAX_RETRIES_BABY_STEP {
            let result = BabyStepResult {
                baby_step_id: BabyStepId(1),
                outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
                lines_modified: 0,
                retry_count: 0,
            };
            nav.record_result(result, &mut state, &tree);
        }

        // Exhaust baby step 2 retries (second failure at step level)
        for _ in 0..DEFAULT_MAX_RETRIES_BABY_STEP {
            let result = BabyStepResult {
                baby_step_id: BabyStepId(2),
                outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
                lines_modified: 0,
                retry_count: 0,
            };
            nav.record_result(result, &mut state, &tree);
        }

        // Step level threshold is 2 — should have a step-level bubble-up
        let step_events: Vec<_> = state
            .bubble_up_log
            .iter()
            .filter(|e| e.level == DecompositionLevel::Step)
            .collect();
        assert!(
            !step_events.is_empty(),
            "Should have step-level bubble-up after {} baby-step exhaustions",
            DEFAULT_MAX_RETRIES_STEP
        );
    }

    #[test]
    fn test_concept_failure_triggers_escalate() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Exhaust all levels up to concept
        for _ in 0..DEFAULT_MAX_RETRIES_BABY_STEP {
            let result = BabyStepResult {
                baby_step_id: BabyStepId(1),
                outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
                lines_modified: 0,
                retry_count: 0,
            };
            nav.record_result(result, &mut state, &tree);
        }

        // After concept-level threshold exceeded, should have Escalate action
        let has_escalate = state.bubble_up_log.iter().any(|e| {
            matches!(e.action, BubbleUpAction::Escalate { .. })
        });
        // In a 1-step, 1-phase, 1-concept tree, exhausting baby step retries
        // cascades through step -> phase -> concept -> Escalate
        assert!(
            has_escalate,
            "Should have Escalate action when all levels exhausted"
        );
    }

    #[test]
    fn test_diagnose_and_retry_action_recorded() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // First failure should trigger DiagnoseAndRetry (not yet at threshold)
        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Failed {
                reason: "test failure".to_string(),
            },
            lines_modified: 10,
            retry_count: 0,
        };
        let action = nav.record_result(result, &mut state, &tree);

        // First failure should return DiagnoseAndRetry, not bubble up
        assert!(
            matches!(action, Some(BubbleUpAction::DiagnoseAndRetry { .. })),
            "First failure should suggest DiagnoseAndRetry, got {:?}",
            action
        );
    }

    #[test]
    fn test_bubble_up_log_entries_created() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Fail past threshold
        for _ in 0..DEFAULT_MAX_RETRIES_BABY_STEP {
            let result = BabyStepResult {
                baby_step_id: BabyStepId(1),
                outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
                lines_modified: 0,
                retry_count: 0,
            };
            nav.record_result(result, &mut state, &tree);
        }

        assert!(
            !state.bubble_up_log.is_empty(),
            "Bubble-up log should have entries after threshold exceeded"
        );
        let event = &state.bubble_up_log[0];
        assert_eq!(event.threshold, DEFAULT_MAX_RETRIES_BABY_STEP);
        assert_eq!(event.failure_count, DEFAULT_MAX_RETRIES_BABY_STEP);
    }

    #[test]
    fn test_bubble_up_policy_override_honored() {
        let step = make_step(1, &[1]);
        let phase = make_phase(1, vec![step], vec![]);
        let concept = make_concept(1, vec![phase]);
        let decomp = make_decomposition(vec![concept]);

        // Custom policy: only 1 retry at baby-step level
        let policy = BubbleUpPolicy {
            max_retries_baby_step: 1,
            max_retries_step: 1,
            max_retries_phase: 1,
            max_retries_concept: 1,
        };
        let tree = NavigatorTree::new(decomp, policy);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Single failure should trigger bubble-up with threshold=1
        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
            lines_modified: 0,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        let baby_step_events: Vec<_> = state
            .bubble_up_log
            .iter()
            .filter(|e| e.level == DecompositionLevel::BabyStep)
            .collect();
        assert!(
            !baby_step_events.is_empty(),
            "Custom policy with max_retries=1 should trigger bubble-up on first failure"
        );
        assert_eq!(baby_step_events[0].threshold, 1);
    }

    // -----------------------------------------------------------------------
    // 4E.8 — Integration checkpoint tests (6 tests)
    // -----------------------------------------------------------------------

    /// Build a tree where baby step 1 has IntegrationSpec (wiring_target="module_a").
    /// Since IntegrationSpec is on StepNode (exit_criteria), we need to check
    /// how integration checkpoints are triggered.
    fn integration_tree() -> (NavigatorTree, IntegrationSpec) {
        let bs = make_baby_step(1);
        let step = StepNode {
            id: StepId(1),
            description: "integration step".to_string(),
            baby_steps: vec![bs],
            evidence_level: crate::research_tree::EvidenceLevel::Anecdotal,
        };
        let phase = make_phase(1, vec![step], vec![]);
        let concept = make_concept(1, vec![phase]);
        let decomp = make_decomposition(vec![concept]);
        let tree = NavigatorTree::new(decomp, BubbleUpPolicy::default());
        let spec = IntegrationSpec {
            wiring_target: "module_a".to_string(),
            callers: vec!["caller_1".to_string(), "caller_2".to_string()],
            exit_criteria: vec![ExitCriterion {
                description: "wired".to_string(),
                check_command: Some("cargo test".to_string()),
            }],
        };
        (tree, spec)
    }

    #[test]
    fn test_checkpoint_recorded_on_completion() {
        let (tree, spec) = integration_tree();
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Complete baby step 1 — should record integration checkpoint
        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Passed,
            lines_modified: 50,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        // The navigator should record checkpoints for completed steps that have
        // integration specs. We'll use the step's IntegrationSpec.
        // For now, we record checkpoint via a separate call since IntegrationSpec
        // is on StepNode, not BabyStepSpec.
        record_integration_checkpoint(BabyStepId(1), &spec, &mut state);

        assert_eq!(state.integration_checkpoints.len(), 1);
        assert_eq!(state.integration_checkpoints[0].wiring_target, "module_a");
    }

    #[test]
    fn test_checkpoint_callers_propagated() {
        let (_, spec) = integration_tree();
        let mut state = TreeNavigatorState::new();

        record_integration_checkpoint(BabyStepId(1), &spec, &mut state);

        assert_eq!(
            state.integration_checkpoints[0].callers,
            vec!["caller_1".to_string(), "caller_2".to_string()]
        );
    }

    #[test]
    fn test_checkpoint_level_and_id_populated() {
        let (_, spec) = integration_tree();
        let mut state = TreeNavigatorState::new();

        record_integration_checkpoint(BabyStepId(1), &spec, &mut state);

        let cp = &state.integration_checkpoints[0];
        assert_eq!(cp.baby_step_id, BabyStepId(1));
        assert_eq!(cp.level, DecompositionLevel::BabyStep);
        assert!(cp.timestamp_epoch_ms > 0, "Timestamp should be populated");
    }

    #[test]
    fn test_checkpoint_timestamp_populated() {
        let (_, spec) = integration_tree();
        let mut state = TreeNavigatorState::new();

        record_integration_checkpoint(BabyStepId(42), &spec, &mut state);

        assert!(
            state.integration_checkpoints[0].timestamp_epoch_ms > 0,
            "Timestamp must be non-zero"
        );
    }

    #[test]
    fn test_checkpoint_wiring_target_captured() {
        let spec = IntegrationSpec {
            wiring_target: "custom_target".to_string(),
            callers: vec![],
            exit_criteria: vec![],
        };
        let mut state = TreeNavigatorState::new();

        record_integration_checkpoint(BabyStepId(1), &spec, &mut state);

        assert_eq!(state.integration_checkpoints[0].wiring_target, "custom_target");
    }

    #[test]
    fn test_no_checkpoint_for_step_without_integration() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Passed,
            lines_modified: 10,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        assert!(
            state.integration_checkpoints.is_empty(),
            "No checkpoint should be recorded when step has no IntegrationSpec"
        );
    }

    // -----------------------------------------------------------------------
    // 4E.9 — Baby-step execution cycle tests (15 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn test_cycle_pass_marks_completed() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Passed,
            lines_modified: 20,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        assert!(state.completed.contains(&BabyStepId(1)));
        assert!(!state.failed.contains(&BabyStepId(1)));
    }

    #[test]
    fn test_cycle_fail_marks_failed_after_threshold() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        for _ in 0..DEFAULT_MAX_RETRIES_BABY_STEP {
            let result = BabyStepResult {
                baby_step_id: BabyStepId(1),
                outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
                lines_modified: 0,
                retry_count: 0,
            };
            nav.record_result(result, &mut state, &tree);
        }

        assert!(
            state.failed.contains(&BabyStepId(1)),
            "Baby step should be marked failed after exhausting retries"
        );
    }

    #[test]
    fn test_cycle_skip_marks_skipped() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Skipped { reason: "pre-gate failed".to_string() },
            lines_modified: 0,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        assert!(state.skipped.contains(&BabyStepId(1)));
    }

    #[test]
    fn test_cycle_blocked_marks_blocked() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Blocked { blocked_by: BabyStepId(99) },
            lines_modified: 0,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        assert!(state.blocked.contains(&BabyStepId(1)));
    }

    #[test]
    fn test_cycle_execution_log_appended() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Passed,
            lines_modified: 30,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        assert_eq!(state.execution_log.len(), 1);
        assert_eq!(state.execution_log[0].baby_step_id, BabyStepId(1));
    }

    #[test]
    fn test_cycle_lines_over_limit_triggers_redecompose() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Passed,
            lines_modified: MAX_BABY_STEP_LINES_MODIFIED + 1,
            retry_count: 0,
        };
        let action = nav.record_result(result, &mut state, &tree);

        assert!(
            matches!(action, Some(BubbleUpAction::RedecomposeStep { .. })),
            "Lines > {} should trigger RedecomposeStep, got {:?}",
            MAX_BABY_STEP_LINES_MODIFIED,
            action
        );
    }

    #[test]
    fn test_cycle_is_complete_when_all_done() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.completed.insert(BabyStepId(1));
        state.completed.insert(BabyStepId(2));

        assert!(nav.is_complete(&state, &tree));
    }

    #[test]
    fn test_cycle_not_complete_with_remaining() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.completed.insert(BabyStepId(1));

        assert!(!nav.is_complete(&state, &tree));
    }

    #[test]
    fn test_cycle_complete_counts_skipped_and_failed() {
        let tree = simple_tree(3);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.completed.insert(BabyStepId(1));
        state.skipped.insert(BabyStepId(2));
        state.failed.insert(BabyStepId(3));

        // All accounted for (completed + skipped + failed = total)
        assert!(
            nav.is_complete(&state, &tree),
            "Should be complete when all baby steps are resolved (completed/skipped/failed)"
        );
    }

    #[test]
    fn test_cycle_advance_after_completion() {
        let tree = simple_tree(3);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Complete step 1, next should be step 2
        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Passed,
            lines_modified: 10,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        let next = nav.next_baby_step(&state);
        assert_eq!(next, Some(BabyStepId(2)));
    }

    #[test]
    fn test_cycle_retry_returns_same_step() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Fail once (below threshold) — should still return same step
        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
            lines_modified: 0,
            retry_count: 0,
        };
        nav.record_result(result, &mut state, &tree);

        let next = nav.next_baby_step(&state);
        assert_eq!(
            next,
            Some(BabyStepId(1)),
            "Should retry same baby step when below threshold"
        );
    }

    #[test]
    fn test_cycle_failure_record_tracks_attempts() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        let result = BabyStepResult {
            baby_step_id: BabyStepId(1),
            outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
            lines_modified: 0,
            retry_count: 0,
        };
        nav.record_result(result.clone(), &mut state, &tree);
        nav.record_result(result, &mut state, &tree);

        let record = state.failure_records.get(&BabyStepId(1));
        assert!(record.is_some(), "Failure record should exist");
        assert_eq!(record.map(|r| r.attempts).unwrap_or(0), 2, "Should track 2 attempts");
    }

    #[test]
    fn test_cycle_integration_ready_all_deps_complete() {
        // Build tree with 2 baby steps in same step
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        state.completed.insert(BabyStepId(1));

        // Baby step 2 integration readiness depends on step completion
        // With only 1 of 2 completed, step is not fully done
        let ready = nav.integration_ready(BabyStepId(2), &state, &tree);
        // integration_ready checks if all baby steps in the same step are complete
        // except the current one
        assert!(
            ready,
            "Baby step 2 should be integration-ready when baby step 1 is complete"
        );
    }

    #[test]
    fn test_cycle_integration_not_ready_deps_incomplete() {
        let tree = two_concept_tree();
        let nav = DepthFirstNavigator::from_tree(&tree);
        let state = TreeNavigatorState::new();

        // Baby step 3 (concept 2) depends on concept 1 being complete
        // Concept 1 baby steps (1, 2) not complete
        let ready = nav.integration_ready(BabyStepId(3), &state, &tree);
        assert!(
            !ready,
            "Baby step 3 should NOT be integration-ready when concept 1 is incomplete"
        );
    }

    #[test]
    fn test_cycle_multiple_results_interleaved() {
        let tree = simple_tree(3);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        // Pass 1, fail 2, pass 3
        nav.record_result(
            BabyStepResult {
                baby_step_id: BabyStepId(1),
                outcome: BabyStepOutcome::Passed,
                lines_modified: 10,
                retry_count: 0,
            },
            &mut state,
            &tree,
        );
        nav.record_result(
            BabyStepResult {
                baby_step_id: BabyStepId(2),
                outcome: BabyStepOutcome::Failed { reason: "fail".to_string() },
                lines_modified: 0,
                retry_count: 0,
            },
            &mut state,
            &tree,
        );

        assert!(state.completed.contains(&BabyStepId(1)));
        assert!(!state.completed.contains(&BabyStepId(2)));
        assert!(!state.failed.contains(&BabyStepId(2)), "Single failure should not mark as failed");
        assert_eq!(state.execution_log.len(), 2);
    }

    // -----------------------------------------------------------------------
    // 4E.11 — Integration checkpoint EXECUTION tests (12 new tests)
    // -----------------------------------------------------------------------

    /// Helper: fixed runner that reports every criterion as failing with the
    /// given reason. Used to drive bubble-up paths deterministically in tests.
    #[derive(Debug, Clone)]
    struct FailingRunner {
        reason: String,
    }

    impl IntegrationRunner for FailingRunner {
        fn run_criterion(
            &self,
            _level: DecompositionLevel,
            _node_id: u64,
            criterion: &ExitCriterion,
        ) -> CriterionResult {
            CriterionResult {
                description: criterion.description.clone(),
                success: false,
                reason: Some(self.reason.clone()),
                duration_ms: 1,
            }
        }
    }

    fn spec(wiring: &str) -> IntegrationSpec {
        IntegrationSpec {
            wiring_target: wiring.to_string(),
            callers: vec!["caller_x".to_string()],
            exit_criteria: vec![ExitCriterion {
                description: "criterion_1".to_string(),
                check_command: Some("echo ok".to_string()),
            }],
        }
    }

    #[test]
    fn test_register_integration_spec_step_level() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Step, 1, spec("target_step"));
        let got = tree.integration_spec(DecompositionLevel::Step, 1);
        assert!(got.is_some(), "Step-level spec should be retrievable");
        assert_eq!(got.unwrap().wiring_target, "target_step");
    }

    #[test]
    fn test_register_integration_spec_phase_level() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Phase, 1, spec("target_phase"));
        let got = tree.integration_spec(DecompositionLevel::Phase, 1);
        assert_eq!(got.map(|s| s.wiring_target.as_str()), Some("target_phase"));
    }

    #[test]
    fn test_register_integration_spec_concept_level() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Concept, 1, spec("target_concept"));
        let got = tree.integration_spec(DecompositionLevel::Concept, 1);
        assert_eq!(got.map(|s| s.wiring_target.as_str()), Some("target_concept"));
    }

    #[test]
    fn test_run_step_integration_pass_no_bubble_up() {
        let mut tree = simple_tree(2);
        tree.register_integration_spec(DecompositionLevel::Step, 1, spec("s_target"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        let (res, action) = nav.run_step_integration(StepId(1), &runner, &mut state, &tree);

        let res = res.expect("spec registered -> result should be Some");
        assert!(res.success, "NoOp runner must yield success");
        assert_eq!(res.level, DecompositionLevel::Step);
        assert_eq!(res.node_id, 1);
        assert!(action.is_none(), "Passing integration must not bubble up");
        assert_eq!(state.integration_checkpoints.len(), 1);
        assert_eq!(
            state.integration_checkpoints[0].level,
            DecompositionLevel::Step
        );
        assert!(state.integration_checkpoints[0].success);
        assert_eq!(state.integration_checkpoints[0].baby_step_id, NO_BABY_STEP);
    }

    #[test]
    fn test_run_step_integration_fail_triggers_redecompose() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Step, 1, spec("s_fail"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = FailingRunner { reason: "cargo test failed".to_string() };

        let (res, action) = nav.run_step_integration(StepId(1), &runner, &mut state, &tree);
        let res = res.expect("spec registered");
        assert!(!res.success);
        match action {
            Some(BubbleUpAction::RedecomposeStep { step_id }) => {
                assert_eq!(step_id, StepId(1));
            }
            Some(BubbleUpAction::ReplanPhase { .. }) => {
                // Acceptable if step threshold = 1 (cascades). Not expected at default policy.
                panic!("First step failure should emit RedecomposeStep, not ReplanPhase");
            }
            other => panic!("Expected RedecomposeStep, got {:?}", other),
        }
        let step_events: Vec<_> = state
            .bubble_up_log
            .iter()
            .filter(|e| e.level == DecompositionLevel::Step)
            .collect();
        assert!(
            !step_events.is_empty(),
            "Step-level bubble-up event must be recorded"
        );
        assert!(!state.integration_checkpoints[0].success);
        assert!(state.integration_checkpoints[0].failure_reason.is_some());
    }

    #[test]
    fn test_run_phase_integration_pass() {
        let mut tree = simple_tree(2);
        tree.register_integration_spec(DecompositionLevel::Phase, 1, spec("p_target"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        let (res, action) = nav.run_phase_integration(PhaseId(1), &runner, &mut state, &tree);
        let res = res.expect("spec registered");
        assert!(res.success);
        assert_eq!(res.level, DecompositionLevel::Phase);
        assert!(action.is_none());
    }

    #[test]
    fn test_run_phase_integration_fail_triggers_replan_phase() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Phase, 1, spec("p_fail"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = FailingRunner { reason: "phase check failed".to_string() };

        let (_, action) = nav.run_phase_integration(PhaseId(1), &runner, &mut state, &tree);
        // With default policy max_retries_phase=1, a single failure hits threshold
        // and cascades to concept. Either ReplanPhase (pre-cascade) or the concept
        // escalation (ReplanConcept/Escalate) is acceptable — but we must see at
        // least a Phase-level bubble-up event regardless.
        assert!(action.is_some(), "Phase-integration fail must return an action");
        let phase_events: Vec<_> = state
            .bubble_up_log
            .iter()
            .filter(|e| e.level == DecompositionLevel::Phase)
            .collect();
        assert!(
            !phase_events.is_empty(),
            "Phase-level bubble-up event must be recorded"
        );
    }

    #[test]
    fn test_run_concept_integration_pass() {
        let mut tree = simple_tree(2);
        tree.register_integration_spec(DecompositionLevel::Concept, 1, spec("c_target"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        let (res, action) = nav.run_concept_integration(ConceptId(1), &runner, &mut state, &tree);
        let res = res.expect("spec registered");
        assert!(res.success);
        assert_eq!(res.level, DecompositionLevel::Concept);
        assert!(action.is_none());
    }

    #[test]
    fn test_run_concept_integration_fail_escalates() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Concept, 1, spec("c_fail"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = FailingRunner { reason: "concept e2e failed".to_string() };

        let (_, action) = nav.run_concept_integration(ConceptId(1), &runner, &mut state, &tree);
        // Default policy max_retries_concept=1 -> first failure should escalate.
        match action {
            Some(BubbleUpAction::Escalate { context }) => {
                assert!(context.contains("concept e2e failed"));
            }
            other => panic!("Expected Escalate at concept level, got {:?}", other),
        }
        let concept_events: Vec<_> = state
            .bubble_up_log
            .iter()
            .filter(|e| e.level == DecompositionLevel::Concept)
            .collect();
        assert!(!concept_events.is_empty());
    }

    #[test]
    fn test_run_integration_no_spec_is_noop() {
        let tree = simple_tree(1);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        let (res, action) = nav.run_step_integration(StepId(1), &runner, &mut state, &tree);
        assert!(
            res.is_none(),
            "Unregistered step spec must return None result (no checkpoint recorded)"
        );
        assert!(action.is_none());
        assert!(state.integration_checkpoints.is_empty());
        assert!(state.bubble_up_log.is_empty());
    }

    #[test]
    fn test_integration_checkpoint_level_stored_correctly() {
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Step, 1, spec("s1"));
        tree.register_integration_spec(DecompositionLevel::Phase, 1, spec("p1"));
        tree.register_integration_spec(DecompositionLevel::Concept, 1, spec("c1"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        nav.run_step_integration(StepId(1), &runner, &mut state, &tree);
        nav.run_phase_integration(PhaseId(1), &runner, &mut state, &tree);
        nav.run_concept_integration(ConceptId(1), &runner, &mut state, &tree);

        assert_eq!(state.integration_checkpoints.len(), 3);
        let levels: Vec<DecompositionLevel> = state
            .integration_checkpoints
            .iter()
            .map(|c| c.level)
            .collect();
        assert_eq!(
            levels,
            vec![
                DecompositionLevel::Step,
                DecompositionLevel::Phase,
                DecompositionLevel::Concept,
            ]
        );
        assert_eq!(state.integration_checkpoints[0].wiring_target, "s1");
        assert_eq!(state.integration_checkpoints[1].wiring_target, "p1");
        assert_eq!(state.integration_checkpoints[2].wiring_target, "c1");
    }

    #[test]
    fn test_integration_empty_exit_criteria_passes() {
        let mut tree = simple_tree(1);
        let s = IntegrationSpec {
            wiring_target: "no_criteria".to_string(),
            callers: vec![],
            exit_criteria: vec![],
        };
        tree.register_integration_spec(DecompositionLevel::Step, 1, s);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = FailingRunner { reason: "should not be called".to_string() };

        let (res, action) = nav.run_step_integration(StepId(1), &runner, &mut state, &tree);
        let res = res.expect("spec registered");
        assert!(
            res.success,
            "Empty exit_criteria must trivially pass (no criteria to fail)"
        );
        assert!(action.is_none());
        assert_eq!(res.criterion_results.len(), 0);
    }

    #[test]
    fn test_integration_counters_separate_from_cascade_counters() {
        // Regression guard for 4E.11 audit BLOCK: integration checkpoint
        // failures must NOT increment `step_failure_counts` (cascade counter)
        // and vice versa — each source owns its own counter map.
        let mut tree = simple_tree(1);
        tree.register_integration_spec(DecompositionLevel::Step, 1, spec("iso_step"));
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let failing = FailingRunner { reason: "intentional".to_string() };

        nav.run_step_integration(StepId(1), &failing, &mut state, &tree);

        assert_eq!(
            state
                .step_integration_failure_counts
                .get(&StepId(1))
                .copied()
                .unwrap_or(0),
            1,
            "integration fail must increment step_integration_failure_counts"
        );
        assert_eq!(
            state.step_failure_counts.get(&StepId(1)).copied().unwrap_or(0),
            0,
            "integration fail must NOT touch the cascade step_failure_counts"
        );
    }

    #[test]
    fn test_terminal_detection_baby_step_step_phase_concept() {
        let tree = simple_tree(2);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();

        assert!(!nav.step_is_terminal(StepId(1), &state, &tree));
        assert!(!nav.phase_is_terminal(PhaseId(1), &state, &tree));
        assert!(!nav.concept_is_terminal(ConceptId(1), &state, &tree));

        state.completed.insert(BabyStepId(1));
        state.completed.insert(BabyStepId(2));

        assert!(nav.step_is_terminal(StepId(1), &state, &tree));
        assert!(nav.phase_is_terminal(PhaseId(1), &state, &tree));
        assert!(nav.concept_is_terminal(ConceptId(1), &state, &tree));
    }

    // -----------------------------------------------------------------------
    // WEB.3 — web validation tier policy
    // -----------------------------------------------------------------------

    #[test]
    fn test_web_validation_tiers_baby_step_only_build() {
        let tiers = web_validation_tiers_for_level(DecompositionLevel::BabyStep);
        assert_eq!(tiers, vec![WebValidationTier::WebBuild]);
    }

    #[test]
    fn test_web_validation_tiers_step_build_then_lint() {
        let tiers = web_validation_tiers_for_level(DecompositionLevel::Step);
        assert_eq!(
            tiers,
            vec![WebValidationTier::WebBuild, WebValidationTier::WebLint]
        );
    }

    #[test]
    fn test_web_validation_tiers_phase_build_then_dev_server() {
        let tiers = web_validation_tiers_for_level(DecompositionLevel::Phase);
        assert_eq!(
            tiers,
            vec![WebValidationTier::WebBuild, WebValidationTier::WebDevServer]
        );
    }

    #[test]
    fn test_web_validation_tiers_concept_full_pipeline() {
        let tiers = web_validation_tiers_for_level(DecompositionLevel::Concept);
        assert_eq!(
            tiers,
            vec![
                WebValidationTier::WebBuild,
                WebValidationTier::WebDevServer,
                WebValidationTier::WebLint,
            ]
        );
    }
}
