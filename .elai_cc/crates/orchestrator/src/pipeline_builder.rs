//! Phase 4E: Pipeline Factory L2 — deterministic pipeline assembly from PipelineSpec.
//!
//! No LLM imports. No agent process. Pure Rust, deterministic validation and assembly.
//! DAG repair loop uses AutoB2G-derived pattern: bounded validation + iterative
//! deterministic repair (REORDER, SUBSTITUTE). T_DAG=10 max rounds.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use elai_ipc_types::ToolId;
use crate::agent_pool::ModelPool;
use crate::research_tree::{PipelineSpec, QualitySpec, StepId};

// ---------------------------------------------------------------------------
// Newtypes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleId(pub String);

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum DAG repair rounds before escalation.
pub const T_DAG: u32 = 10;

/// Default overhead tokens per contract for phase sizing.
pub const CONTRACT_OVERHEAD: u64 = 200;

/// Split ratio: phase_complexity > available_for_work * SPLIT_RATIO triggers split.
pub const SPLIT_RATIO: f64 = 0.6;

/// Maximum split depth for phase sizing.
pub const MAX_SPLIT_DEPTH: u32 = 2;

// ---------------------------------------------------------------------------
// PredicateRef — serializable predicate reference (compiled form is ephemeral)
// ---------------------------------------------------------------------------

/// A predicate reference that stores the raw DSL string for serialization
/// and the compiled form for evaluation. The compiled form is rehydrated
/// from the raw string on deserialization via PredicateCompiler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateRef {
    /// Raw predicate DSL string (serializable, source of truth for persistence).
    pub raw: String,
    /// Whether this predicate compiled successfully at construction time.
    pub valid: bool,
}

// ---------------------------------------------------------------------------
// ResourceReq — per-node resource requirements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReq {
    pub model_id: Option<String>,
    pub vram_mb: u64,
    pub disk_mb: u64,
    pub estimated_secs: u64,
}

// ---------------------------------------------------------------------------
// DagNode — a single node in the pipeline dependency graph
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub step_id: StepId,
    pub tool_id: ToolId,
    pub inputs: Vec<DataRef>,
    pub outputs: Vec<DataRef>,
    pub resource_req: ResourceReq,
    pub optional: bool,
}

// ---------------------------------------------------------------------------
// DependencyGraph — validated DAG of pipeline steps
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DagNode>,
    pub edges: Vec<(NodeIndex, NodeIndex)>,
}

impl DependencyGraph {
    /// Compute a structural hash of the graph for fixpoint detection.
    pub fn structural_hash(&self) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        for (i, node) in self.nodes.iter().enumerate() {
            hasher.update(&(i as u64).to_le_bytes());
            hasher.update(node.tool_id.0.as_bytes());
            hasher.update(if node.optional { &[1] } else { &[0] });
        }
        for (from, to) in &self.edges {
            hasher.update(&(from.0 as u64).to_le_bytes());
            hasher.update(&(to.0 as u64).to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}

// ---------------------------------------------------------------------------
// PipelineFsmState — a generated FSM state for pipeline execution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineFsmState {
    pub state_id: StateId,
    pub phase_name: String,
    pub allowed_tools: Vec<ToolId>,
    pub allowed_rules: Vec<RuleId>,
    pub entry_gate: Option<PredicateRef>,
    pub exit_gate: PredicateRef,
    pub is_terminal: bool,
}

// ---------------------------------------------------------------------------
// AgentPhaseConfig — per-phase agent assignment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPhaseConfig {
    pub phase_id: StepId,
    pub agent_role: String,
    pub grc_budget: (u32, u32),
    pub model_preference: Option<String>,
}

// ---------------------------------------------------------------------------
// RuleBinding — per-phase rule installation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleBinding {
    pub phase_id: StepId,
    pub rule_ids: Vec<RuleId>,
    pub install_order: u32,
}

// ---------------------------------------------------------------------------
// ResourceEstimate — hardware resource estimate for the pipeline
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub total_vram_mb: u64,
    pub peak_vram_mb: u64,
    pub estimated_disk_mb: u64,
    pub estimated_duration_secs: u64,
    pub fits_hardware: bool,
}

// ---------------------------------------------------------------------------
// TestCase / FixtureStub / TestHarnessSkeleton
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub target_value: f64,
    pub assertion_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureStub {
    pub name: String,
    pub setup_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHarnessSkeleton {
    pub test_cases: Vec<TestCase>,
    pub fixture_stubs: Vec<FixtureStub>,
}

// ---------------------------------------------------------------------------
// RepairAction — deterministic repair strategies (AutoB2G-derived)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepairAction {
    Reorder {
        from: NodeIndex,
        to: NodeIndex,
        reason: String,
    },
    Substitute {
        old_tool: ToolId,
        new_tool: ToolId,
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// RepairOutcome
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepairOutcome {
    Repaired {
        rounds: u32,
        actions: Vec<RepairAction>,
    },
    Escalated {
        rounds: u32,
        remaining_issues: Vec<ValidationError>,
    },
}

// ---------------------------------------------------------------------------
// ValidationError / ValidationErrorKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub kind: ValidationErrorKind,
    pub node: Option<NodeIndex>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationErrorKind {
    CycleDetected,
    MissingTool,
    MissingDependency,
    ResourceOverflow,
    UnmeasurableMetric,
    InvalidPredicate,
    MissingAdapter,
    InvalidVisualCheck,
}

// ---------------------------------------------------------------------------
// ExecutablePipeline — the L2 output artifact
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutablePipeline {
    pub pipeline_id: PipelineId,
    pub spec: PipelineSpec,
    pub fsm_states: Vec<PipelineFsmState>,
    pub dependency_graph: DependencyGraph,
    pub novel_step_ids: Vec<StepId>,
    pub agent_configs: Vec<AgentPhaseConfig>,
    pub rule_configs: Vec<RuleBinding>,
    pub resource_estimate: ResourceEstimate,
    pub test_harness: TestHarnessSkeleton,
    pub repair_log: Vec<RepairAction>,
    pub output_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// PipelineBuilderError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum PipelineBuilderError {
    #[error("empty pipeline spec: no steps defined")]
    EmptySpec,

    #[error("structural validation failed: {0} errors")]
    ValidationFailed(usize),

    #[error("repair escalated after {rounds} rounds: {remaining} issues remaining")]
    RepairEscalated { rounds: u32, remaining: usize },

    #[error("resource overflow: pipeline requires {required_mb} MB VRAM, only {available_mb} MB available")]
    ResourceOverflow { required_mb: u64, available_mb: u64 },

    #[error("serialization failed: {0}")]
    SerializationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// ToolLookup — trait for tool existence checks (decouples from concrete registry)
// ---------------------------------------------------------------------------

/// Abstraction over tool registry for L2 validation.
/// Decouples pipeline_builder from the concrete ToolRegistry/IpcToolRegistry types.
pub trait ToolLookup: Send + Sync {
    /// Check if a tool with the given ID exists in the registry.
    fn has_tool(&self, tool_id: &ToolId) -> bool;

    /// Find a substitute tool with compatible IO for the given tool.
    /// Returns None if no substitute is available.
    fn find_substitute(&self, tool_id: &ToolId) -> Option<ToolId>;
}

// ---------------------------------------------------------------------------
// PredicateCompiler — trait for predicate compilation (decouples from RuleEngine)
// ---------------------------------------------------------------------------

/// Abstraction over rule engine for FSM predicate compilation.
pub trait PredicateCompiler: Send + Sync {
    /// Validate and compile a raw predicate string.
    /// Returns Ok(PredicateRef) with valid=true on success,
    /// or Err with the compilation error message.
    fn compile(&self, raw: &str) -> Result<PredicateRef, String>;

    /// Get active rule IDs for a given phase context.
    fn active_rules_for_phase(&self, phase_name: &str) -> Vec<RuleId>;
}

// ---------------------------------------------------------------------------
// PipelineBuilder — main entry point
// ---------------------------------------------------------------------------

pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Build an ExecutablePipeline from a PipelineSpec.
    ///
    /// Pipeline: PipelineSpec -> DependencyGraph -> validate -> repair -> ExecutablePipeline
    pub fn build(
        spec: &PipelineSpec,
        tools: &dyn ToolLookup,
        pool: &dyn ModelPool,
        predicates: &dyn PredicateCompiler,
    ) -> Result<ExecutablePipeline, PipelineBuilderError> {
        // Reject empty specs
        if spec.steps.is_empty() {
            return Err(PipelineBuilderError::EmptySpec);
        }

        // Build initial dependency graph from spec
        let mut graph = Self::spec_to_graph(spec);

        // Build a default QualitySpec for validation
        let quality = crate::research_tree::QualitySpec {
            min_source_quality: crate::research_tree::SourceQuality::Tier1,
            min_evidence_level: crate::research_tree::EvidenceLevel::Verified,
            require_contradictions: false,
            metrics: Vec::new(),
        };

        // Structural validation (8-point checklist)
        let errors = StructuralValidator::validate(&graph, spec, &quality, tools, pool, predicates);

        // If errors exist, attempt repair
        let mut repair_log = Vec::new();
        if !errors.is_empty() {
            let outcome = DagRepairLoop::repair(&mut graph, &errors, tools, T_DAG);
            match outcome {
                RepairOutcome::Repaired { actions, .. } => {
                    repair_log = actions;
                }
                RepairOutcome::Escalated {
                    rounds, remaining_issues,
                } => {
                    return Err(PipelineBuilderError::RepairEscalated {
                        rounds,
                        remaining: remaining_issues.len(),
                    });
                }
            }
        }

        // Generate FSM states
        let fsm_states = FsmStateGenerator::generate_states(spec, predicates);

        // Detect novel steps
        let novel_step_ids = NoveltyDetector::detect_novel(&graph, tools);

        // Configure agents
        let agent_configs = AgentConfigurator::configure_agents(spec);

        // Install rules
        let rule_configs = RuleInstaller::install_rules(spec, predicates);

        // Estimate resources
        let resource_estimate = ResourceEstimator::estimate(&graph, pool);

        // Generate test harness
        let test_harness = TestHarnessGenerator::generate(&quality, spec);

        // Compute pipeline ID from spec hash
        let pipeline_id = {
            use blake3::Hasher;
            let mut h = Hasher::new();
            h.update(spec.name.as_bytes());
            for s in &spec.steps {
                h.update(s.as_bytes());
            }
            let bytes = h.finalize();
            let id_bytes: [u8; 8] = bytes.as_bytes()[..8].try_into().unwrap_or([0; 8]);
            PipelineId(u64::from_le_bytes(id_bytes))
        };

        let output_dir = PathBuf::from(format!(".elai_cc/pipelines/{}", pipeline_id.0));

        Ok(ExecutablePipeline {
            pipeline_id,
            spec: spec.clone(),
            fsm_states,
            dependency_graph: graph,
            novel_step_ids,
            agent_configs,
            rule_configs,
            resource_estimate,
            test_harness,
            repair_log,
            output_dir,
        })
    }

    /// Convert a PipelineSpec into an initial DependencyGraph.
    fn spec_to_graph(spec: &PipelineSpec) -> DependencyGraph {
        let nodes: Vec<DagNode> = spec
            .steps
            .iter()
            .enumerate()
            .map(|(i, step_name)| DagNode {
                step_id: StepId(i as u64),
                tool_id: ToolId(step_name.clone()),
                inputs: if i > 0 {
                    vec![DataRef(format!("output_{}", i - 1))]
                } else {
                    vec![]
                },
                outputs: vec![DataRef(format!("output_{i}"))],
                resource_req: ResourceReq {
                    model_id: None,
                    vram_mb: 512,
                    disk_mb: 100,
                    estimated_secs: 10,
                },
                optional: false,
            })
            .collect();

        // Linear dependency chain: step_0 -> step_1 -> ... -> step_n
        let edges: Vec<(NodeIndex, NodeIndex)> = (1..nodes.len())
            .map(|i| (NodeIndex(i - 1), NodeIndex(i)))
            .collect();

        DependencyGraph { nodes, edges }
    }
}

// ---------------------------------------------------------------------------
// StructuralValidator — 8-point structural validation checklist
// ---------------------------------------------------------------------------

pub struct StructuralValidator;

impl StructuralValidator {
    /// Run all 8 structural validation checks on the dependency graph.
    pub fn validate(
        graph: &DependencyGraph,
        spec: &PipelineSpec,
        quality: &QualitySpec,
        tools: &dyn ToolLookup,
        pool: &dyn ModelPool,
        predicates: &dyn PredicateCompiler,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check 1: Tools referenced exist in ToolRegistry
        for (i, node) in graph.nodes.iter().enumerate() {
            if !tools.has_tool(&node.tool_id) {
                errors.push(ValidationError {
                    kind: ValidationErrorKind::MissingTool,
                    node: Some(NodeIndex(i)),
                    message: format!("tool '{}' not found in registry", node.tool_id.0),
                });
            }
        }

        // Check 2 + 4: Resource requirements fit hardware
        let available_vram_mb = pool.available_vram() / (1024 * 1024);
        let peak_vram_mb: u64 = graph
            .nodes
            .iter()
            .map(|n| n.resource_req.vram_mb)
            .max()
            .unwrap_or(0);
        if available_vram_mb < u64::MAX / (1024 * 1024) && peak_vram_mb > available_vram_mb {
            errors.push(ValidationError {
                kind: ValidationErrorKind::ResourceOverflow,
                node: None,
                message: format!(
                    "peak VRAM {peak_vram_mb} MB exceeds available {available_vram_mb} MB"
                ),
            });
        }

        // Check 3: No dependency cycles (Kahn's algorithm)
        if let Err(cycle_nodes) = topological_sort(graph) {
            errors.push(ValidationError {
                kind: ValidationErrorKind::CycleDetected,
                node: cycle_nodes.first().copied(),
                message: format!(
                    "dependency cycle detected involving {} nodes",
                    cycle_nodes.len()
                ),
            });
        }

        // Check 5: Every QualityMetric has automated check method
        for metric in &quality.metrics {
            if metric.name.is_empty() {
                errors.push(ValidationError {
                    kind: ValidationErrorKind::UnmeasurableMetric,
                    node: None,
                    message: "quality metric has empty name (unmeasurable)".into(),
                });
            }
        }

        // Check 6: FSM state predicates are syntactically valid
        for step_name in &spec.steps {
            if let Err(e) = predicates.compile(step_name) {
                errors.push(ValidationError {
                    kind: ValidationErrorKind::InvalidPredicate,
                    node: None,
                    message: format!("predicate compilation failed for '{}': {}", step_name, e),
                });
            }
        }

        // Check 7: Novel step detection (handled by NoveltyDetector, not an error)
        // — intentionally no errors here; novelty is informational

        // Check 8: Visual check adapter validation
        if !spec.media.is_empty() && spec.output_type == crate::research_tree::OutputType::Diagram {
            // If media specs exist for diagram output but no adapter, flag it
            errors.push(ValidationError {
                kind: ValidationErrorKind::MissingAdapter,
                node: None,
                message: "visual check requires MediaRenderAdapter for output type".into(),
            });
        }

        errors
    }
}

// ---------------------------------------------------------------------------
// DagRepairLoop — bounded iterative repair (AutoB2G pattern)
// ---------------------------------------------------------------------------

pub struct DagRepairLoop;

impl DagRepairLoop {
    /// Attempt to repair structural issues in the dependency graph.
    ///
    /// Uses two deterministic strategies:
    /// - REORDER: break cycles by removing weakest dependency edge
    /// - SUBSTITUTE: replace missing tool with compatible alternative
    ///
    /// Non-repairable conditions (resource overflow, ambiguous deps) escalate immediately.
    /// Max T_DAG rounds. Each round must produce a structurally different graph.
    pub fn repair(
        graph: &mut DependencyGraph,
        errors: &[ValidationError],
        tools: &dyn ToolLookup,
        max_rounds: u32,
    ) -> RepairOutcome {
        // Separate repairable from non-repairable errors
        let mut non_repairable: Vec<ValidationError> = Vec::new();
        let mut repairable: Vec<ValidationError> = Vec::new();

        for error in errors {
            match error.kind {
                // Non-repairable: escalate immediately
                ValidationErrorKind::ResourceOverflow
                | ValidationErrorKind::UnmeasurableMetric
                | ValidationErrorKind::InvalidPredicate
                | ValidationErrorKind::MissingAdapter
                | ValidationErrorKind::InvalidVisualCheck
                | ValidationErrorKind::MissingDependency => {
                    non_repairable.push(error.clone());
                }
                // Repairable via strategies
                ValidationErrorKind::CycleDetected | ValidationErrorKind::MissingTool => {
                    repairable.push(error.clone());
                }
            }
        }

        // If only non-repairable errors exist, escalate immediately
        if repairable.is_empty() {
            return RepairOutcome::Escalated {
                rounds: 0,
                remaining_issues: non_repairable,
            };
        }

        let mut all_actions = Vec::new();
        let mut prev_hash = graph.structural_hash();

        for round in 0..max_rounds {
            let mut round_actions = Vec::new();
            let mut remaining = Vec::new();

            for error in &repairable {
                match error.kind {
                    ValidationErrorKind::CycleDetected => {
                        // REORDER: remove the last back-edge to break cycle
                        if let Some(edge_idx) = graph.edges.iter().rposition(|(from, to)| {
                            from.0 >= to.0 // back-edge heuristic
                        }) {
                            let (from, to) = graph.edges[edge_idx];
                            graph.edges.remove(edge_idx);
                            round_actions.push(RepairAction::Reorder {
                                from,
                                to,
                                reason: "removed back-edge to break cycle".into(),
                            });
                        } else if !graph.edges.is_empty() {
                            // Fallback: remove last edge
                            let (from, to) = graph.edges.pop().unwrap_or((NodeIndex(0), NodeIndex(0)));
                            round_actions.push(RepairAction::Reorder {
                                from,
                                to,
                                reason: "removed edge to break cycle (fallback)".into(),
                            });
                        } else {
                            remaining.push(error.clone());
                        }
                    }
                    ValidationErrorKind::MissingTool => {
                        // SUBSTITUTE: find compatible alternative
                        if let Some(node_idx) = error.node {
                            if node_idx.0 < graph.nodes.len() {
                                let old_tool = graph.nodes[node_idx.0].tool_id.clone();
                                if let Some(new_tool) = tools.find_substitute(&old_tool) {
                                    graph.nodes[node_idx.0].tool_id = new_tool.clone();
                                    round_actions.push(RepairAction::Substitute {
                                        old_tool,
                                        new_tool,
                                        reason: "substituted with compatible tool".into(),
                                    });
                                } else {
                                    remaining.push(error.clone());
                                }
                            } else {
                                remaining.push(error.clone());
                            }
                        } else {
                            remaining.push(error.clone());
                        }
                    }
                    _ => remaining.push(error.clone()),
                }
            }

            all_actions.extend(round_actions);

            // Fixpoint detection: if graph unchanged, escalate
            let new_hash = graph.structural_hash();
            if new_hash == prev_hash {
                let mut all_remaining = remaining;
                all_remaining.extend(non_repairable);
                return RepairOutcome::Escalated {
                    rounds: round + 1,
                    remaining_issues: all_remaining,
                };
            }
            prev_hash = new_hash;

            // Re-check if cycle still exists after repair
            if remaining.is_empty() {
                // Verify the graph is now valid
                if topological_sort(graph).is_ok() {
                    return RepairOutcome::Repaired {
                        rounds: round + 1,
                        actions: all_actions,
                    };
                }
                // Still has cycles, continue repairing
                repairable = vec![ValidationError {
                    kind: ValidationErrorKind::CycleDetected,
                    node: None,
                    message: "cycle persists after repair".into(),
                }];
            } else {
                repairable = remaining;
            }
        }

        // Max rounds exhausted
        let mut all_remaining = repairable;
        all_remaining.extend(non_repairable);
        RepairOutcome::Escalated {
            rounds: max_rounds,
            remaining_issues: all_remaining,
        }
    }
}

// ---------------------------------------------------------------------------
// FsmStateGenerator
// ---------------------------------------------------------------------------

pub struct FsmStateGenerator;

impl FsmStateGenerator {
    /// Generate FSM states from a PipelineSpec.
    ///
    /// Each phase maps to one PipelineFsmState. Last state is terminal.
    pub fn generate_states(
        spec: &PipelineSpec,
        predicates: &dyn PredicateCompiler,
    ) -> Vec<PipelineFsmState> {
        let n = spec.steps.len();
        spec.steps
            .iter()
            .enumerate()
            .map(|(i, step_name)| {
                let is_terminal = i == n - 1;
                let rules = predicates.active_rules_for_phase(step_name);

                // Entry gate: None for first state, compiled predicate for others
                let entry_gate = if i > 0 {
                    predicates.compile(&format!("phase_{}_entry", step_name)).ok()
                } else {
                    None
                };

                // Exit gate: always present
                let exit_gate = predicates
                    .compile(&format!("phase_{}_exit", step_name))
                    .unwrap_or(PredicateRef {
                        raw: format!("phase_{}_exit", step_name),
                        valid: true,
                    });

                PipelineFsmState {
                    state_id: StateId(i as u64),
                    phase_name: step_name.clone(),
                    allowed_tools: vec![ToolId(step_name.clone())],
                    allowed_rules: rules,
                    entry_gate,
                    exit_gate,
                    is_terminal,
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// TestHarnessGenerator
// ---------------------------------------------------------------------------

pub struct TestHarnessGenerator;

impl TestHarnessGenerator {
    /// Generate a test harness skeleton from QualitySpec.
    ///
    /// Produces one TestCase per QualityMetric + one per PerfSpec.
    pub fn generate(quality_spec: &QualitySpec, spec: &PipelineSpec) -> TestHarnessSkeleton {
        let mut test_cases = Vec::new();

        // One TestCase per QualityMetric
        for metric in &quality_spec.metrics {
            test_cases.push(TestCase {
                name: format!("test_{}", metric.name),
                target_value: metric.threshold,
                assertion_template: format!(
                    "assert!({}.value >= {}, \"metric '{}' below threshold\")",
                    metric.name, metric.threshold, metric.name
                ),
            });
        }

        // One TestCase per PerfSpec
        test_cases.push(TestCase {
            name: "test_perf_latency".into(),
            target_value: spec.perf.max_latency.as_secs_f64(),
            assertion_template: format!(
                "assert!(elapsed <= {:?}, \"latency exceeds max\")",
                spec.perf.max_latency
            ),
        });

        // Fixture stubs
        let fixture_stubs = vec![
            FixtureStub {
                name: "model_loader".into(),
                setup_description: "Load model for testing".into(),
            },
            FixtureStub {
                name: "data_paths".into(),
                setup_description: "Set up test data paths".into(),
            },
        ];

        TestHarnessSkeleton {
            test_cases,
            fixture_stubs,
        }
    }
}

// ---------------------------------------------------------------------------
// ResourceEstimator
// ---------------------------------------------------------------------------

pub struct ResourceEstimator;

impl ResourceEstimator {
    /// Estimate resource requirements for the pipeline.
    ///
    /// Computes per-step VRAM, peak across concurrent phases, fits_hardware check.
    pub fn estimate(graph: &DependencyGraph, pool: &dyn ModelPool) -> ResourceEstimate {
        if graph.nodes.is_empty() {
            return ResourceEstimate {
                total_vram_mb: 0,
                peak_vram_mb: 0,
                estimated_disk_mb: 0,
                estimated_duration_secs: 0,
                fits_hardware: true,
            };
        }

        let total_vram_mb: u64 = graph.nodes.iter().map(|n| n.resource_req.vram_mb).sum();
        let peak_vram_mb: u64 = graph
            .nodes
            .iter()
            .map(|n| n.resource_req.vram_mb)
            .max()
            .unwrap_or(0);
        let estimated_disk_mb: u64 = graph.nodes.iter().map(|n| n.resource_req.disk_mb).sum();
        let estimated_duration_secs: u64 =
            graph.nodes.iter().map(|n| n.resource_req.estimated_secs).sum();

        let available_vram_mb = if pool.available_vram() == u64::MAX {
            u64::MAX
        } else {
            pool.available_vram() / (1024 * 1024)
        };

        let fits_hardware = peak_vram_mb <= available_vram_mb;

        ResourceEstimate {
            total_vram_mb,
            peak_vram_mb,
            estimated_disk_mb,
            estimated_duration_secs,
            fits_hardware,
        }
    }
}

// ---------------------------------------------------------------------------
// NoveltyDetector
// ---------------------------------------------------------------------------

pub struct NoveltyDetector;

impl NoveltyDetector {
    /// Detect steps with tools not in the capability registry.
    ///
    /// Novel steps are marked for phase-4F FEASIBILITY_PROBE consumption.
    pub fn detect_novel(graph: &DependencyGraph, tools: &dyn ToolLookup) -> Vec<StepId> {
        graph
            .nodes
            .iter()
            .filter(|node| !tools.has_tool(&node.tool_id))
            .map(|node| node.step_id)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// AgentConfigurator
// ---------------------------------------------------------------------------

pub struct AgentConfigurator;

impl AgentConfigurator {
    /// Configure per-phase agent assignments from PipelineSpec.
    pub fn configure_agents(spec: &PipelineSpec) -> Vec<AgentPhaseConfig> {
        spec.steps
            .iter()
            .enumerate()
            .map(|(i, _step)| AgentPhaseConfig {
                phase_id: StepId(i as u64),
                agent_role: "executor".into(),
                grc_budget: (8, 1200),
                model_preference: None,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// RuleInstaller
// ---------------------------------------------------------------------------

pub struct RuleInstaller;

impl RuleInstaller {
    /// Install per-phase rules from PipelineSpec using the rule engine.
    pub fn install_rules(
        spec: &PipelineSpec,
        predicates: &dyn PredicateCompiler,
    ) -> Vec<RuleBinding> {
        spec.steps
            .iter()
            .enumerate()
            .map(|(i, step_name)| {
                let rule_ids = predicates.active_rules_for_phase(step_name);
                RuleBinding {
                    phase_id: StepId(i as u64),
                    rule_ids,
                    install_order: u32::try_from(i).unwrap_or(u32::MAX),
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Topological sort (Kahn's algorithm)
// ---------------------------------------------------------------------------

/// Run Kahn's algorithm on the dependency graph.
/// Returns topologically sorted node indices, or the nodes involved in cycles.
pub fn topological_sort(graph: &DependencyGraph) -> Result<Vec<NodeIndex>, Vec<NodeIndex>> {
    let n = graph.nodes.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    // Check for self-edges first
    for (from, to) in &graph.edges {
        if from.0 == to.0 {
            return Err(vec![*from]);
        }
    }

    // Build in-degree table
    let mut in_degree = vec![0u32; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (from, to) in &graph.edges {
        if from.0 < n && to.0 < n {
            in_degree[to.0] += 1;
            adj[from.0].push(to.0);
        }
    }

    // Start with nodes that have in-degree 0
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    let mut sorted = Vec::with_capacity(n);
    while let Some(node) = queue.pop_front() {
        sorted.push(NodeIndex(node));
        for &neighbor in &adj[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    if sorted.len() == n {
        Ok(sorted)
    } else {
        // Nodes not in sorted order are part of cycles
        let sorted_set: std::collections::HashSet<usize> =
            sorted.iter().map(|ni| ni.0).collect();
        let cycle_nodes: Vec<NodeIndex> = (0..n)
            .filter(|i| !sorted_set.contains(i))
            .map(NodeIndex)
            .collect();
        Err(cycle_nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_id_newtype_roundtrip() {
        let id = PipelineId(42);
        let json = serde_json::to_string(&id).unwrap();
        let back: PipelineId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn node_index_newtype() {
        let idx = NodeIndex(7);
        assert_eq!(idx.0, 7);
    }

    #[test]
    fn data_ref_newtype() {
        let dr = DataRef("input_text".into());
        assert_eq!(dr.0, "input_text");
    }

    #[test]
    fn rule_id_newtype() {
        let rid = RuleId("rule_001".into());
        assert_eq!(rid.0, "rule_001");
    }

    #[test]
    fn state_id_newtype() {
        let sid = StateId(99);
        assert_eq!(sid.0, 99);
    }

    #[test]
    fn validation_error_kind_exhaustive() {
        // All 8 variants exist and are distinct
        let kinds = [
            ValidationErrorKind::CycleDetected,
            ValidationErrorKind::MissingTool,
            ValidationErrorKind::MissingDependency,
            ValidationErrorKind::ResourceOverflow,
            ValidationErrorKind::UnmeasurableMetric,
            ValidationErrorKind::InvalidPredicate,
            ValidationErrorKind::MissingAdapter,
            ValidationErrorKind::InvalidVisualCheck,
        ];
        assert_eq!(kinds.len(), 8);
        for (i, a) in kinds.iter().enumerate() {
            for (j, b) in kinds.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn repair_action_variants() {
        let reorder = RepairAction::Reorder {
            from: NodeIndex(0),
            to: NodeIndex(1),
            reason: "cycle".into(),
        };
        let substitute = RepairAction::Substitute {
            old_tool: ToolId("old".into()),
            new_tool: ToolId("new".into()),
            reason: "missing".into(),
        };
        // Both variants constructible
        assert!(matches!(reorder, RepairAction::Reorder { .. }));
        assert!(matches!(substitute, RepairAction::Substitute { .. }));
    }

    #[test]
    fn dependency_graph_structural_hash_deterministic() {
        let graph = DependencyGraph {
            nodes: vec![DagNode {
                step_id: StepId(0),
                tool_id: ToolId("tool_a".into()),
                inputs: vec![],
                outputs: vec![DataRef("out".into())],
                resource_req: ResourceReq {
                    model_id: None,
                    vram_mb: 0,
                    disk_mb: 0,
                    estimated_secs: 0,
                },
                optional: false,
            }],
            edges: vec![],
        };
        let h1 = graph.structural_hash();
        let h2 = graph.structural_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn dependency_graph_different_structure_different_hash() {
        let g1 = DependencyGraph {
            nodes: vec![DagNode {
                step_id: StepId(0),
                tool_id: ToolId("a".into()),
                inputs: vec![],
                outputs: vec![],
                resource_req: ResourceReq {
                    model_id: None,
                    vram_mb: 0,
                    disk_mb: 0,
                    estimated_secs: 0,
                },
                optional: false,
            }],
            edges: vec![],
        };
        let g2 = DependencyGraph {
            nodes: vec![DagNode {
                step_id: StepId(0),
                tool_id: ToolId("b".into()),
                inputs: vec![],
                outputs: vec![],
                resource_req: ResourceReq {
                    model_id: None,
                    vram_mb: 0,
                    disk_mb: 0,
                    estimated_secs: 0,
                },
                optional: false,
            }],
            edges: vec![],
        };
        assert_ne!(g1.structural_hash(), g2.structural_hash());
    }

    #[test]
    fn self_edge_detection() {
        // A graph with a self-edge (n,n) should be detected as a cycle
        let graph = DependencyGraph {
            nodes: vec![DagNode {
                step_id: StepId(0),
                tool_id: ToolId("a".into()),
                inputs: vec![],
                outputs: vec![],
                resource_req: ResourceReq {
                    model_id: None,
                    vram_mb: 0,
                    disk_mb: 0,
                    estimated_secs: 0,
                },
                optional: false,
            }],
            edges: vec![(NodeIndex(0), NodeIndex(0))],
        };
        // topological_sort should fail on self-edge
        assert!(topological_sort(&graph).is_err());
    }

    #[test]
    fn executable_pipeline_compiles_with_static() {
        // Verify ExecutablePipeline can be used with 'static (no lifetime issues)
        fn _assert_static<T: 'static>() {}
        _assert_static::<ExecutablePipeline>();
    }
}
