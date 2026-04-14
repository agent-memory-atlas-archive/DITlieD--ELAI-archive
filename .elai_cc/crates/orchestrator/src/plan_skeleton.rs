//! Hierarchical plan management: skeleton types and TOML serialization.
//!
//! Phase 3G.1 types + Phase 3G.2 serialization with token budget enforcement,
//! SSB recording, and version tracking.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::Path;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use elai_providers::LlmProvider;

use crate::ssb::{
    PlanUpdatePayload, Ssb, SsbPayload, StubReferencePayload,
};

// ---- Type aliases ---------------------------------------------------------

pub type PipelineId = String;
pub type PhaseId = String;

// ---- Constants ------------------------------------------------------------

const GOAL_MAX_LEN: usize = 120;
const TOKEN_ESTIMATE_THRESHOLD: usize = 3600;
const TOKEN_HARD_CAP: usize = 4096;

// ---- Error type -----------------------------------------------------------

#[derive(Debug, Error)]
pub enum PlanSkeletonError {
    #[error("goal exceeds {GOAL_MAX_LEN} chars: got {len}")]
    GoalTooLong { len: usize },

    #[error(
        "skeleton exceeds token budget: {actual} > {limit}"
    )]
    SkeletonBudgetExceeded { actual: usize, limit: usize },

    #[error("version overflow on increment")]
    VersionOverflow,

    #[error(
        "contract changed in phase {phase_id} but {kind} version not incremented"
    )]
    VersionNotIncremented {
        phase_id: String,
        kind: &'static str,
    },

    #[error("phase not found: {0}")]
    PhaseNotFound(String),

    #[error(
        "contract conflict: '{name}' has differing signatures in phases {phase_a} and {phase_b}"
    )]
    ContractConflict {
        name: String,
        phase_a: String,
        phase_b: String,
    },

    #[error("TOML serialization failed: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialization failed: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("token count failed: {0}")]
    TokenCount(String),

    #[error("SSB write failed: {0}")]
    SsbWrite(String),

    #[error(
        "skeleton alone ({skeleton_tokens}) exceeds Slot 3 budget ({slot3_budget})"
    )]
    SlotBudgetExceeded {
        skeleton_tokens: usize,
        slot3_budget: usize,
    },

    #[error(
        "phase detail stale: skeleton_version={detail_version} < current={current_version}"
    )]
    StalePhaseDetail {
        detail_version: u32,
        current_version: u32,
    },

    #[error(
        "dependency cycle detected among phases: {phases:?}"
    )]
    CycleDetected { phases: Vec<PhaseId> },

    #[error("split depth exceeded: max 2 levels")]
    SplitDepthExceeded,
}

// ---- OutputType -----------------------------------------------------------

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Image,
    Video,
    Audio,
    Document,
}

// ---- ContractRef ----------------------------------------------------------

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct ContractRef {
    pub name: String,
    pub type_signature: String,
    pub invariants: Vec<String>,
}

// ---- ExtractedContracts ---------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedContracts {
    pub input_contracts: Vec<ContractRef>,
    pub output_contracts: Vec<ContractRef>,
    pub shared_constraints: Vec<String>,
}

// ---- PhaseDetail ----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseDetail {
    pub phase_id: PhaseId,
    pub skeleton_version: u32,
    pub contracts: ExtractedContracts,
    pub implemented_contracts: Vec<ContractRef>,
    pub declared_interfaces: Vec<ContractRef>,
    pub content: String,
    pub token_count: usize,
}

impl PhaseDetail {
    /// Read a phase detail from a TOML file.
    pub fn read_from_file(
        path: &Path,
    ) -> Result<Self, PlanSkeletonError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(PlanSkeletonError::TomlDeserialize)
    }

    /// Write a phase detail to a TOML file.
    pub fn write_to_file(
        &self,
        path: &Path,
    ) -> Result<(), PlanSkeletonError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(PlanSkeletonError::TomlSerialize)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> Result<String, PlanSkeletonError> {
        toml::to_string_pretty(self)
            .map_err(PlanSkeletonError::TomlSerialize)
    }
}

// ---- PhaseContext ----------------------------------------------------------

/// Result of load_phase_context: everything needed for Slot 3.
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseContext {
    pub skeleton_tokens: usize,
    pub phase_detail: Option<PhaseDetail>,
    pub contracts: ExtractedContracts,
    pub adjacent_contracts: Vec<AdjacentContracts>,
    pub total_tokens: usize,
    pub reduced: bool,
    pub staleness_warning: Option<StalenessWarning>,
    /// Constitutional CONSTRAINTS block (A-7) injected into PhaseSpec content.
    /// Built from deprecated_patterns WHERE severity = 'BLOCK'.
    pub constraints_block: Option<crate::context_budget::ConstraintsBlock>,
}

/// Adjacent phase contracts (one hop in dependency graph).
#[derive(Debug, Clone, PartialEq)]
pub struct AdjacentContracts {
    pub phase_id: PhaseId,
    pub contracts: Vec<ContractRef>,
}

/// Warning emitted when phase detail was elaborated against older skeleton.
#[derive(Debug, Clone, PartialEq)]
pub struct StalenessWarning {
    pub detail_version: u32,
    pub current_version: u32,
}

// ---- PhaseEntry -----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseEntry {
    pub phase_id: PhaseId,
    pub goal: String,
    #[serde(default)]
    pub inputs: Vec<ContractRef>,
    #[serde(default)]
    pub outputs: Vec<ContractRef>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<PhaseId>,
    #[serde(default)]
    pub depended_by: Vec<PhaseId>,
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_type: Option<OutputType>,
    #[serde(default)]
    pub visual_check: bool,
    #[serde(default)]
    pub split_depth: u8,
}

impl PhaseEntry {
    pub fn new(
        phase_id: impl Into<PhaseId>,
        goal: impl Into<String>,
    ) -> Result<Self, PlanSkeletonError> {
        let goal = goal.into();
        if goal.len() > GOAL_MAX_LEN {
            return Err(PlanSkeletonError::GoalTooLong {
                len: goal.len(),
            });
        }
        Ok(Self {
            phase_id: phase_id.into(),
            goal,
            inputs: Vec::new(),
            outputs: Vec::new(),
            constraints: Vec::new(),
            depends_on: Vec::new(),
            depended_by: Vec::new(),
            version: 0,
            output_type: None,
            visual_check: false,
            split_depth: 0,
        })
    }

    /// Increment version with overflow protection.
    pub fn increment_version(
        &mut self,
    ) -> Result<(), PlanSkeletonError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(PlanSkeletonError::VersionOverflow)?;
        Ok(())
    }
}

// ---- PlanSkeleton ---------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanSkeleton {
    pub pipeline_id: PipelineId,
    #[serde(with = "indexmap::map::serde_seq")]
    pub phases: IndexMap<PhaseId, PhaseEntry>,
    #[serde(default)]
    pub global_constraints: Vec<String>,
    pub version: u32,
}

impl PlanSkeleton {
    pub fn new(pipeline_id: impl Into<PipelineId>) -> Self {
        Self {
            pipeline_id: pipeline_id.into(),
            phases: IndexMap::new(),
            global_constraints: Vec::new(),
            version: 0,
        }
    }

    /// Add a phase entry. Returns error if goal exceeds limit.
    pub fn add_phase(
        &mut self,
        entry: PhaseEntry,
    ) -> Result<(), PlanSkeletonError> {
        self.phases.insert(entry.phase_id.clone(), entry);
        Ok(())
    }

    /// Increment skeleton version with overflow protection.
    pub fn increment_version(
        &mut self,
    ) -> Result<(), PlanSkeletonError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(PlanSkeletonError::VersionOverflow)?;
        Ok(())
    }

    /// Kahn's algorithm cycle detection on the dependency DAG.
    ///
    /// Returns `Ok(())` if the graph is acyclic, or
    /// `Err(CycleDetected)` with a sorted list of phases
    /// involved in cycles.
    fn kahn_cycle_check(
        &self,
    ) -> Result<(), PlanSkeletonError> {
        // Build in_degree from depends_on (authoritative edge source).
        // A phase with depends_on=[X,Y] has in_degree 2.
        let mut in_degree: HashMap<&PhaseId, usize> =
            self.phases
                .keys()
                .map(|k| (k, 0usize))
                .collect();
        // Build adjacency list from depends_on edges (reverse direction).
        // If B.depends_on contains A, then A has an outgoing edge to B.
        let mut forward_edges: HashMap<
            &PhaseId,
            Vec<&PhaseId>,
        > = self
            .phases
            .keys()
            .map(|k| (k, Vec::new()))
            .collect();
        for (phase_id, phase) in &self.phases {
            for dep in &phase.depends_on {
                if let Some(edges) =
                    forward_edges.get_mut(dep)
                {
                    edges.push(phase_id);
                }
                if let Some(d) =
                    in_degree.get_mut(phase_id)
                {
                    *d += 1;
                }
            }
        }
        let mut queue: VecDeque<&PhaseId> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| *k)
            .collect();
        let mut visited_count: usize = 0;
        while let Some(node) = queue.pop_front() {
            visited_count += 1;
            if let Some(neighbors) =
                forward_edges.get(node)
            {
                for neighbor in neighbors {
                    if let Some(d) =
                        in_degree.get_mut(*neighbor)
                    {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
        if visited_count != self.phases.len() {
            let cycle_phases: Vec<PhaseId> = in_degree
                .into_iter()
                .filter(|(_, d)| *d > 0)
                .map(|(k, _)| k.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            return Err(PlanSkeletonError::CycleDetected {
                phases: cycle_phases,
            });
        }
        Ok(())
    }

    /// BFS propagation of contract changes through dependents.
    ///
    /// Starting from `changed_phase_id`, traverses `depended_by`
    /// edges via BFS. Each reached phase (except the source) is
    /// flagged and recorded in SSB as a `plan_update`.
    /// Returns the list of flagged phase IDs.
    ///
    /// Uses a visited set to prevent diamond-topology duplicates.
    pub async fn propagate_changes(
        &self,
        changed_phase_id: &PhaseId,
        ssb: &Ssb,
    ) -> Result<Vec<PhaseId>, PlanSkeletonError> {
        if !self.phases.contains_key(changed_phase_id) {
            return Err(PlanSkeletonError::PhaseNotFound(
                changed_phase_id.clone(),
            ));
        }
        let mut visited: HashSet<PhaseId> =
            HashSet::new();
        let mut queue: VecDeque<PhaseId> =
            VecDeque::new();
        let mut flagged: Vec<PhaseId> = Vec::new();

        queue.push_back(changed_phase_id.clone());
        visited.insert(changed_phase_id.clone());

        while let Some(current) = queue.pop_front() {
            let phase = match self.phases.get(&current) {
                Some(p) => p,
                // Phantom depended_by ref — skip silently.
                None => continue,
            };
            if current != *changed_phase_id {
                flagged.push(current.clone());
                let payload =
                    SsbPayload::PlanUpdate(
                        PlanUpdatePayload {
                            phase_id: current.clone(),
                            version: self.version.to_string(),
                            change_summary: format!("phase changed to {}", changed_phase_id),
                        },
                    );
                ssb.write_payload(
                    "plan_skeleton",
                    0,
                    payload,
                    vec![],
                )
                .await
                .map_err(|e| {
                    PlanSkeletonError::SsbWrite(
                        e.to_string(),
                    )
                })?;
            }
            for dep in &phase.depended_by {
                if !visited.contains(dep) {
                    visited.insert(dep.clone());
                    queue.push_back(dep.clone());
                }
            }
        }
        Ok(flagged)
    }

    /// Split a phase into sub-phases at the given split points.
    ///
    /// `split_points` determines boundaries: N split points produce
    /// N+1 sub-phases. Sub-phase IDs: `{phase_id}_1` .. `{phase_id}_{n}`.
    ///
    /// When `preserve_contracts` is true:
    /// - First sub-phase inherits original inputs.
    /// - Last sub-phase inherits original outputs.
    /// - Adjacent sub-phases get internal contracts.
    ///
    /// Max split depth is 2. Attempting to split a phase at depth >= 2
    /// returns `SplitDepthExceeded`.
    pub fn split_phase(
        &mut self,
        phase_id: &PhaseId,
        split_points: &[String],
        preserve_contracts: bool,
    ) -> Result<(), PlanSkeletonError> {
        let original = self
            .phases
            .get(phase_id)
            .ok_or_else(|| {
                PlanSkeletonError::PhaseNotFound(
                    phase_id.clone(),
                )
            })?
            .clone();

        if original.split_depth >= 2 {
            return Err(
                PlanSkeletonError::SplitDepthExceeded,
            );
        }

        let sub_count = split_points.len() + 1;
        let new_depth = original.split_depth + 1;
        let mut sub_phases: Vec<PhaseEntry> =
            Vec::with_capacity(sub_count);

        for i in 1..=sub_count {
            let sub_id =
                format!("{phase_id}_{i}");
            let goal = if i <= split_points.len() {
                split_points[i - 1].clone()
            } else {
                original.goal.clone()
            };
            let truncate_at = goal
                .char_indices()
                .nth(GOAL_MAX_LEN)
                .map_or(goal.len(), |(idx, _)| idx);
            let goal_str = &goal[..truncate_at];
            let mut entry =
                PhaseEntry::new(&sub_id, goal_str)?;
            entry.split_depth = new_depth;
            entry.version = original.version;

            if preserve_contracts {
                if i == 1 {
                    entry.inputs =
                        original.inputs.clone();
                }
                if i == sub_count {
                    entry.outputs =
                        original.outputs.clone();
                }
                // Internal contracts: output of Bi -> input of Bi+1
                if i < sub_count {
                    let internal_name = format!(
                        "{}::split_output",
                        sub_id
                    );
                    entry.outputs.push(ContractRef {
                        name: internal_name.clone(),
                        type_signature:
                            "fn() -> SplitData"
                                .to_owned(),
                        invariants: vec![],
                    });
                }
                if i > 1 {
                    let prev_id = format!(
                        "{phase_id}_{}",
                        i - 1
                    );
                    let internal_name = format!(
                        "{}::split_output",
                        prev_id
                    );
                    entry.inputs.push(ContractRef {
                        name: internal_name,
                        type_signature:
                            "fn() -> SplitData"
                                .to_owned(),
                        invariants: vec![],
                    });
                }
            }

            // Wire dependency chain: Bi depends on Bi-1
            if i > 1 {
                let prev_id = format!(
                    "{phase_id}_{}",
                    i - 1
                );
                entry.depends_on.push(prev_id);
            } else {
                // First sub-phase inherits original's depends_on
                entry.depends_on =
                    original.depends_on.clone();
            }

            // Last sub-phase inherits original's depended_by
            if i == sub_count {
                entry.depended_by =
                    original.depended_by.clone();
            }

            entry.constraints =
                original.constraints.clone();

            sub_phases.push(entry);
        }

        // Wire depended_by for adjacent sub-phases
        for i in 0..sub_phases.len() - 1 {
            let next_id =
                sub_phases[i + 1].phase_id.clone();
            sub_phases[i].depended_by.push(next_id);
        }

        // Update upstream phases: replace phase_id in their depended_by with first sub-phase
        let first_sub_id =
            sub_phases[0].phase_id.clone();
        for dep_id in &original.depends_on {
            if let Some(dep) =
                self.phases.get_mut(dep_id)
            {
                if let Some(pos) = dep
                    .depended_by
                    .iter()
                    .position(|d| d == phase_id)
                {
                    dep.depended_by[pos] =
                        first_sub_id.clone();
                }
            }
        }

        // Update downstream phases: replace phase_id in their depends_on with last sub-phase
        let last_sub_id =
            sub_phases.last()
                .map(|p| p.phase_id.clone())
                .unwrap_or_default();
        for dep_id in &original.depended_by {
            if let Some(dep) =
                self.phases.get_mut(dep_id)
            {
                if let Some(pos) = dep
                    .depends_on
                    .iter()
                    .position(|d| d == phase_id)
                {
                    dep.depends_on[pos] =
                        last_sub_id.clone();
                }
            }
        }

        // Replace original with sub-phases preserving insertion order.
        // Rebuild the map: entries before original, then sub-phases,
        // then entries after original.
        let mut new_phases: IndexMap<PhaseId, PhaseEntry> =
            IndexMap::with_capacity(
                self.phases.len() - 1 + sub_phases.len(),
            );
        for (id, entry) in self.phases.drain(..) {
            if id == *phase_id {
                for sub in sub_phases.drain(..) {
                    new_phases.insert(
                        sub.phase_id.clone(),
                        sub,
                    );
                }
            } else {
                new_phases.insert(id, entry);
            }
        }
        self.phases = new_phases;

        Ok(())
    }

    /// Extract contracts for a phase from its dependency graph.
    ///
    /// - `input_contracts`: union of `outputs` from all `depends_on` phases.
    /// - `output_contracts`: union of `inputs` from all `depended_by` phases.
    /// - `shared_constraints`: `global_constraints` followed by phase-local `constraints`.
    ///
    /// Returns `PhaseNotFound` if `phase_id` or any referenced dep is missing.
    /// Returns `ContractConflict` if two deps export the same contract name
    /// with differing `type_signature`.
    pub fn extract_contracts(
        &self,
        phase_id: &str,
    ) -> Result<ExtractedContracts, PlanSkeletonError> {
        let phase = self
            .phases
            .get(phase_id)
            .ok_or_else(|| {
                PlanSkeletonError::PhaseNotFound(
                    phase_id.to_owned(),
                )
            })?;

        // Step 1: input_contracts = union of dep.outputs for deps in depends_on.
        // Track (name -> (type_signature, source_phase)) for conflict detection.
        let mut seen_inputs: IndexMap<
            String,
            (String, String),
        > = IndexMap::new();
        let mut input_contracts = Vec::new();

        for dep_id in &phase.depends_on {
            let dep = self.phases.get(dep_id.as_str()).ok_or_else(
                || {
                    PlanSkeletonError::PhaseNotFound(
                        dep_id.clone(),
                    )
                },
            )?;
            for c in &dep.outputs {
                match seen_inputs.get(&c.name) {
                    Some((existing_sig, existing_phase)) => {
                        if existing_sig != &c.type_signature {
                            return Err(
                                PlanSkeletonError::ContractConflict {
                                    name: c.name.clone(),
                                    phase_a: existing_phase
                                        .clone(),
                                    phase_b: dep_id.clone(),
                                },
                            );
                        }
                        // Same name + same signature: deduplicate silently.
                    }
                    None => {
                        seen_inputs.insert(
                            c.name.clone(),
                            (
                                c.type_signature.clone(),
                                dep_id.clone(),
                            ),
                        );
                        input_contracts.push(c.clone());
                    }
                }
            }
        }

        // Step 2: output_contracts = union of dep.inputs for deps in depended_by.
        let mut seen_outputs: IndexMap<
            String,
            (String, String),
        > = IndexMap::new();
        let mut output_contracts = Vec::new();

        for dep_id in &phase.depended_by {
            let dep = self.phases.get(dep_id.as_str()).ok_or_else(
                || {
                    PlanSkeletonError::PhaseNotFound(
                        dep_id.clone(),
                    )
                },
            )?;
            for c in &dep.inputs {
                match seen_outputs.get(&c.name) {
                    Some((existing_sig, existing_phase)) => {
                        if existing_sig != &c.type_signature {
                            return Err(
                                PlanSkeletonError::ContractConflict {
                                    name: c.name.clone(),
                                    phase_a: existing_phase
                                        .clone(),
                                    phase_b: dep_id.clone(),
                                },
                            );
                        }
                    }
                    None => {
                        seen_outputs.insert(
                            c.name.clone(),
                            (
                                c.type_signature.clone(),
                                dep_id.clone(),
                            ),
                        );
                        output_contracts.push(c.clone());
                    }
                }
            }
        }

        // Step 3: shared_constraints = global first, then phase-local.
        let mut shared_constraints =
            self.global_constraints.clone();
        shared_constraints
            .extend(phase.constraints.iter().cloned());

        Ok(ExtractedContracts {
            input_contracts,
            output_contracts,
            shared_constraints,
        })
    }

    /// Collect adjacent contracts (one hop) for a phase.
    ///
    /// For each dep in `depends_on`: include dep's outputs.
    /// For each dep in `depended_by`: include dep's inputs.
    fn collect_adjacent_contracts(
        &self,
        phase_id: &str,
    ) -> Result<Vec<AdjacentContracts>, PlanSkeletonError> {
        let phase = self
            .phases
            .get(phase_id)
            .ok_or_else(|| {
                PlanSkeletonError::PhaseNotFound(
                    phase_id.to_owned(),
                )
            })?;

        let mut adjacent = Vec::new();

        for dep_id in &phase.depends_on {
            if let Some(dep) = self.phases.get(dep_id.as_str())
            {
                if !dep.outputs.is_empty() {
                    adjacent.push(AdjacentContracts {
                        phase_id: dep_id.clone(),
                        contracts: dep.outputs.clone(),
                    });
                }
            }
        }

        for dep_id in &phase.depended_by {
            if let Some(dep) = self.phases.get(dep_id.as_str())
            {
                if !dep.inputs.is_empty() {
                    adjacent.push(AdjacentContracts {
                        phase_id: dep_id.clone(),
                        contracts: dep.inputs.clone(),
                    });
                }
            }
        }

        Ok(adjacent)
    }

    /// Load phase context for Slot 3: skeleton + detail + contracts.
    ///
    /// Verifies combined token count fits within the Slot 3 allocation.
    /// If it exceeds, attempts adaptive reduction: loads skeleton +
    /// contracts only (defers full phase detail).
    ///
    /// `slot3_budget`: the allocated_tokens for ContextSlot::PhaseSpec.
    /// `phases_dir`: directory containing `{phase_id}.toml` files.
    pub fn load_phase_context(
        &self,
        phase_id: &str,
        slot3_budget: usize,
        provider: &dyn LlmProvider,
        phases_dir: &Path,
        constraints: Option<crate::context_budget::ConstraintsBlock>,
    ) -> Result<PhaseContext, PlanSkeletonError> {
        // Measure skeleton tokens.
        let skeleton_toml = self.to_toml()?;
        let skeleton_tokens = provider
            .token_count(&skeleton_toml)
            .map_err(|e| {
                PlanSkeletonError::TokenCount(
                    e.to_string(),
                )
            })?;

        // If skeleton alone exceeds Slot 3, system cannot operate.
        if skeleton_tokens > slot3_budget {
            return Err(
                PlanSkeletonError::SlotBudgetExceeded {
                    skeleton_tokens,
                    slot3_budget,
                },
            );
        }

        // Extract contracts for the phase.
        let contracts =
            self.extract_contracts(phase_id)?;

        // Measure contracts token cost.
        let contracts_toml =
            toml::to_string_pretty(&contracts)
                .map_err(PlanSkeletonError::TomlSerialize)?;
        let contracts_tokens = provider
            .token_count(&contracts_toml)
            .map_err(|e| {
                PlanSkeletonError::TokenCount(
                    e.to_string(),
                )
            })?;

        // Collect adjacent contracts (one hop).
        let adjacent_contracts =
            self.collect_adjacent_contracts(phase_id)?;
        let adjacent_toml: String = adjacent_contracts
            .iter()
            .map(|ac| {
                format!(
                    "# {}\n{}",
                    ac.phase_id,
                    ac.contracts
                        .iter()
                        .map(|c| {
                            format!(
                                "{}: {}",
                                c.name,
                                c.type_signature
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let adjacent_tokens = if adjacent_toml.is_empty()
        {
            0
        } else {
            provider
                .token_count(&adjacent_toml)
                .map_err(|e| {
                    PlanSkeletonError::TokenCount(
                        e.to_string(),
                    )
                })?
        };

        // Try loading phase detail file.
        let detail_path =
            phases_dir.join(format!("{phase_id}.toml"));
        let phase_detail = if detail_path.exists() {
            Some(PhaseDetail::read_from_file(
                &detail_path,
            )?)
        } else {
            None
        };

        // Staleness detection.
        let staleness_warning =
            phase_detail.as_ref().and_then(|d| {
                if d.skeleton_version < self.version {
                    Some(StalenessWarning {
                        detail_version: d.skeleton_version,
                        current_version: self.version,
                    })
                } else {
                    None
                }
            });

        // Measure phase detail tokens.
        let detail_tokens =
            if let Some(ref detail) = phase_detail {
                let detail_toml = detail.to_toml()?;
                provider
                    .token_count(&detail_toml)
                    .map_err(|e| {
                        PlanSkeletonError::TokenCount(
                            e.to_string(),
                        )
                    })?
            } else {
                0
            };

        // Constitutional constraints token cost (A-7).
        let constraints_tokens = constraints
            .as_ref()
            .map(|c| c.estimated_tokens())
            .unwrap_or(0);

        let total_full = skeleton_tokens
            + contracts_tokens
            + adjacent_tokens
            + detail_tokens
            + constraints_tokens;

        // If combined fits within Slot 3, return full context.
        if total_full <= slot3_budget {
            return Ok(PhaseContext {
                skeleton_tokens,
                phase_detail,
                contracts,
                adjacent_contracts,
                total_tokens: total_full,
                reduced: false,
                staleness_warning,
                constraints_block: constraints,
            });
        }

        // Adaptive reduction: skeleton + contracts only, drop detail.
        let total_reduced = skeleton_tokens
            + contracts_tokens
            + adjacent_tokens
            + constraints_tokens;

        Ok(PhaseContext {
            skeleton_tokens,
            phase_detail: None,
            contracts,
            adjacent_contracts,
            total_tokens: total_reduced,
            reduced: true,
            staleness_warning,
            constraints_block: constraints,
        })
    }

    /// Serialize skeleton to TOML string.
    pub fn to_toml(&self) -> Result<String, PlanSkeletonError> {
        toml::to_string_pretty(self)
            .map_err(PlanSkeletonError::TomlSerialize)
    }

    /// Deserialize skeleton from TOML string.
    pub fn from_toml(
        s: &str,
    ) -> Result<Self, PlanSkeletonError> {
        toml::from_str(s)
            .map_err(PlanSkeletonError::TomlDeserialize)
    }

    /// Read skeleton from a TOML file on disk.
    pub fn read_from_file(
        path: &Path,
    ) -> Result<Self, PlanSkeletonError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Write skeleton to file with token budget enforcement and SSB recording.
    ///
    /// Two-phase token check:
    /// 1. Fast estimate via `token_count_estimate`.
    /// 2. If estimate > 3600, exact check via `token_count`.
    /// 3. If exact > 4096, reject with `SkeletonBudgetExceeded`.
    ///
    /// Records `plan_update` in SSB. For phases without a detail file,
    /// emits `stub_reference`.
    pub async fn write_to_file(
        &self,
        path: &Path,
        provider: &dyn LlmProvider,
        ssb: &Ssb,
        previous: Option<&PlanSkeleton>,
    ) -> Result<(), PlanSkeletonError> {
        // Cycle detection: reject invalid DAGs before any write.
        self.kahn_cycle_check()?;

        // Version enforcement: if contracts changed, versions must
        // have been incremented.
        if let Some(prev) = previous {
            self.verify_version_increments(prev)?;
        }

        let toml_str = self.to_toml()?;

        // Phase 1: fast estimate
        let estimate =
            provider.token_count_estimate(&toml_str);
        if estimate > TOKEN_ESTIMATE_THRESHOLD {
            // Phase 2: exact count
            let exact = provider
                .token_count(&toml_str)
                .map_err(|e| {
                    PlanSkeletonError::TokenCount(
                        e.to_string(),
                    )
                })?;
            if exact > TOKEN_HARD_CAP {
                return Err(
                    PlanSkeletonError::SkeletonBudgetExceeded {
                        actual: exact,
                        limit: TOKEN_HARD_CAP,
                    },
                );
            }
        }

        // Record plan_update in SSB BEFORE writing file to disk.
        // If SSB write fails, no orphan file is left on disk.
        // If SSB succeeds but file write fails, a dangling SSB
        // record is detectable (safer than invisible orphan file).
        let payload = SsbPayload::PlanUpdate(
            PlanUpdatePayload {
                phase_id: self.pipeline_id.clone(),
                version: self.version.to_string(),
                change_summary: format!("plan saved with {} phases", self.phases.len()),
            },
        );
        ssb.write_payload(
            "plan_skeleton",
            0,
            payload,
            vec![path
                .to_string_lossy()
                .into_owned()],
        )
        .await
        .map_err(|e| {
            PlanSkeletonError::SsbWrite(e.to_string())
        })?;

        // Emit stub_reference for phases without detail files
        let phases_dir =
            path.parent().unwrap_or(path).join("phases");
        for phase_id in self.phases.keys() {
            let detail_path =
                phases_dir.join(format!("{phase_id}.toml"));
            if !detail_path.exists() {
                let stub_payload =
                    SsbPayload::StubReference(
                        StubReferencePayload {
                            phase_id: phase_id.clone(),
                            step_id: format!("step_{}", self.version),
                            stub_hash: self.pipeline_id.clone(),
                        },
                    );
                ssb.write_payload(
                    "plan_skeleton",
                    0,
                    stub_payload,
                    vec![],
                )
                .await
                .map_err(|e| {
                    PlanSkeletonError::SsbWrite(
                        e.to_string(),
                    )
                })?;
            }
        }

        // Write TOML to disk (after SSB writes succeed)
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &toml_str)?;

        Ok(())
    }

    /// Check that version increments were applied when contracts
    /// changed.
    fn verify_version_increments(
        &self,
        previous: &PlanSkeleton,
    ) -> Result<(), PlanSkeletonError> {
        for (phase_id, entry) in &self.phases {
            if let Some(prev_entry) =
                previous.phases.get(phase_id)
            {
                let contracts_changed =
                    entry.inputs != prev_entry.inputs
                        || entry.outputs != prev_entry.outputs
                        || entry.constraints
                            != prev_entry.constraints;
                if contracts_changed
                    && entry.version <= prev_entry.version
                {
                    return Err(
                        PlanSkeletonError::VersionNotIncremented {
                            phase_id: phase_id.clone(),
                            kind: "phase",
                        },
                    );
                }
                if contracts_changed
                    && self.version <= previous.version
                {
                    return Err(
                        PlanSkeletonError::VersionNotIncremented {
                            phase_id: phase_id.clone(),
                            kind: "skeleton",
                        },
                    );
                }
            }
        }
        Ok(())
    }
}

// ---- Tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use elai_core::clock::SeededClock;
    use elai_providers::{ApiError, LlmFuture};
    use elai_providers::OllamaConfig;
    use tempfile::TempDir;

    use crate::coherence_check::run_coherence_checks;
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader};

    // Mock provider: deterministic token counting based on byte length.
    // Approximately 4 chars per token (standard rough estimate).
    struct MockProvider {
        chars_per_token: usize,
    }

    impl MockProvider {
        fn new(chars_per_token: usize) -> Self {
            Self { chars_per_token }
        }
    }

    impl LlmProvider for MockProvider {
        fn complete<'a>(
            &'a self,
            _prompt: &'a str,
            _config: &'a OllamaConfig,
        ) -> LlmFuture<'a, Vec<String>> {
            Box::pin(async { Ok(vec![]) })
        }

        fn embed<'a>(
            &'a self,
            _text: &'a str,
        ) -> LlmFuture<'a, Vec<f32>> {
            Box::pin(async { Ok(vec![]) })
        }

        fn token_count(
            &self,
            text: &str,
        ) -> Result<usize, ApiError> {
            Ok(text.len() / self.chars_per_token)
        }

        fn token_count_estimate(
            &self,
            text: &str,
        ) -> usize {
            text.len() / self.chars_per_token
        }
    }

    // Deliberately underestimating provider for adversarial tests.
    struct UnderestimatingProvider {
        estimate_divisor: usize,
        exact_divisor: usize,
    }

    impl LlmProvider for UnderestimatingProvider {
        fn complete<'a>(
            &'a self,
            _prompt: &'a str,
            _config: &'a OllamaConfig,
        ) -> LlmFuture<'a, Vec<String>> {
            Box::pin(async { Ok(vec![]) })
        }

        fn embed<'a>(
            &'a self,
            _text: &'a str,
        ) -> LlmFuture<'a, Vec<f32>> {
            Box::pin(async { Ok(vec![]) })
        }

        fn token_count(
            &self,
            text: &str,
        ) -> Result<usize, ApiError> {
            Ok(text.len() / self.exact_divisor)
        }

        fn token_count_estimate(
            &self,
            text: &str,
        ) -> usize {
            text.len() / self.estimate_divisor
        }
    }

    fn make_skeleton(
        num_phases: usize,
        goal_len: usize,
    ) -> PlanSkeleton {
        let mut skel =
            PlanSkeleton::new("test-pipeline");
        for i in 0..num_phases {
            let phase_id = format!("P{i}");
            let goal: String = (b'A' + (i % 26) as u8)
                .to_string()
                .repeat(goal_len.min(GOAL_MAX_LEN));
            let entry =
                PhaseEntry::new(&phase_id, &goal[..goal_len.min(GOAL_MAX_LEN)])
                    .expect("goal within limit");
            skel.add_phase(entry)
                .expect("add_phase should succeed");
        }
        skel
    }

    fn make_ssb(dir: &Path) -> Ssb {
        let config = SsbConfig {
            db_path: dir.join("ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".to_string(),
            session_id: "test-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> =
            Arc::new(SeededClock::new());
        Ssb::new(config, clock).expect("ssb new")
    }

    // -- 3G.1 type tests --

    #[test]
    fn new_skeleton_version_zero() {
        let skel = PlanSkeleton::new("pipe-1");
        assert_eq!(skel.version, 0);
    }

    #[test]
    fn new_skeleton_phases_empty() {
        let skel = PlanSkeleton::new("pipe-1");
        assert!(skel.phases.is_empty());
    }

    #[test]
    fn phase_entry_goal_too_long() {
        let long_goal = "x".repeat(GOAL_MAX_LEN + 1);
        let result = PhaseEntry::new("P1", long_goal);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("goal exceeds"),
            "expected GoalTooLong, got: {err}"
        );
    }

    #[test]
    fn phase_entry_max_goal_ok() {
        let goal = "x".repeat(GOAL_MAX_LEN);
        let result = PhaseEntry::new("P1", goal);
        assert!(result.is_ok());
    }

    #[test]
    fn version_increment_overflow() {
        let mut entry =
            PhaseEntry::new("P1", "test").unwrap();
        entry.version = u32::MAX;
        let result = entry.increment_version();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("overflow"));
    }

    #[test]
    fn skeleton_version_increment_overflow() {
        let mut skel = PlanSkeleton::new("pipe-1");
        skel.version = u32::MAX;
        let result = skel.increment_version();
        assert!(result.is_err());
    }

    // -- 3G.2 serialization tests --

    #[test]
    fn toml_round_trip_identical() {
        let skel = make_skeleton(5, 80);
        let toml1 = skel.to_toml().unwrap();
        let parsed =
            PlanSkeleton::from_toml(&toml1).unwrap();
        let toml2 = parsed.to_toml().unwrap();
        assert_eq!(toml1, toml2, "round-trip must produce identical TOML");
        assert_eq!(skel, parsed, "struct equality after round-trip");
    }

    #[test]
    fn toml_round_trip_with_contracts() {
        let mut skel = PlanSkeleton::new("pipe-1");
        let mut entry =
            PhaseEntry::new("3B", "SSB single-writer")
                .unwrap();
        entry.inputs.push(ContractRef {
            name: "AgentPool::write(action)".into(),
            type_signature: "fn(SsbAction) -> Result<SsbSeq>"
                .into(),
            invariants: vec![
                "append-only within session".into(),
            ],
        });
        entry.outputs.push(ContractRef {
            name: "ssb::query(filter)".into(),
            type_signature: "fn(SsbFilter) -> Vec<SsbRow>"
                .into(),
            invariants: vec![],
        });
        entry.depends_on.push("3A".into());
        entry.depended_by.push("3C".into());
        entry.constraints.push(
            "conflict detection on file_path".into(),
        );
        skel.add_phase(entry).unwrap();
        skel.global_constraints
            .push("deterministic execution".into());

        let toml1 = skel.to_toml().unwrap();
        let parsed =
            PlanSkeleton::from_toml(&toml1).unwrap();
        let toml2 = parsed.to_toml().unwrap();
        assert_eq!(toml1, toml2);
        assert_eq!(skel, parsed);
    }

    #[test]
    fn toml_round_trip_preserves_insertion_order() {
        let mut skel = PlanSkeleton::new("pipe-1");
        for id in &["3C", "3A", "3B", "3D"] {
            let entry =
                PhaseEntry::new(*id, format!("goal for {id}"))
                    .unwrap();
            skel.add_phase(entry).unwrap();
        }
        let toml_str = skel.to_toml().unwrap();
        let parsed =
            PlanSkeleton::from_toml(&toml_str).unwrap();
        let keys: Vec<&String> =
            parsed.phases.keys().collect();
        assert_eq!(
            keys,
            vec!["3C", "3A", "3B", "3D"],
            "insertion order must be preserved"
        );
    }

    #[test]
    fn twenty_phases_under_2000_tokens() {
        let skel = make_skeleton(20, 80);
        let toml_str = skel.to_toml().unwrap();
        let provider = MockProvider::new(4);
        let count = provider.token_count_estimate(&toml_str);
        assert!(
            count < 2000,
            "20 phases with ~80 char goals should be < 2000 tokens, got {count}"
        );
    }

    #[test]
    fn fifty_phases_under_4096_tokens() {
        let skel = make_skeleton(50, 120);
        let toml_str = skel.to_toml().unwrap();
        let provider = MockProvider::new(4);
        let count =
            provider.token_count(&toml_str).unwrap();
        assert!(
            count < 4096,
            "50 phases with 120 char goals should be < 4096 tokens, got {count}"
        );
    }

    #[test]
    fn toml_whitespace_stability() {
        let skel = make_skeleton(10, 80);
        let toml1 = skel.to_toml().unwrap();
        let provider = MockProvider::new(4);
        let count1 =
            provider.token_count(&toml1).unwrap();
        let parsed =
            PlanSkeleton::from_toml(&toml1).unwrap();
        let toml2 = parsed.to_toml().unwrap();
        let count2 =
            provider.token_count(&toml2).unwrap();
        assert_eq!(
            count1, count2,
            "token count must be stable across re-serialization"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_read_round_trip() {
        let tmp = TempDir::new().unwrap();
        let skel = make_skeleton(5, 80);
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());

        let skel_path =
            tmp.path().join("skeleton.toml");
        skel.write_to_file(
            &skel_path, &provider, &ssb, None,
        )
        .await
        .unwrap();

        let loaded =
            PlanSkeleton::read_from_file(&skel_path)
                .unwrap();
        assert_eq!(skel, loaded);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_records_ssb_plan_update() {
        let tmp = TempDir::new().unwrap();
        let skel = make_skeleton(3, 40);
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());

        let skel_path =
            tmp.path().join("skeleton.toml");
        skel.write_to_file(
            &skel_path, &provider, &ssb, None,
        )
        .await
        .unwrap();

        // Verify SSB has plan_update entry
        let entries = ssb
            .query_by_type(
                SsbActionType::PlanUpdate,
            )
            .unwrap();
        assert!(
            !entries.is_empty(),
            "SSB must contain plan_update after write"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_emits_stub_references() {
        let tmp = TempDir::new().unwrap();
        let skel = make_skeleton(3, 40);
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());

        let skel_path =
            tmp.path().join("skeleton.toml");
        skel.write_to_file(
            &skel_path, &provider, &ssb, None,
        )
        .await
        .unwrap();

        let entries = ssb
            .query_by_type(
                SsbActionType::StubReference,
            )
            .unwrap();
        assert_eq!(
            entries.len(),
            3,
            "3 phases without detail files = 3 stub_references"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn budget_exceeded_returns_error() {
        let tmp = TempDir::new().unwrap();
        // Build a large skeleton with verbose constraints
        let mut skel = make_skeleton(50, 120);
        for entry in skel.phases.values_mut() {
            for i in 0..10 {
                entry.constraints.push(format!(
                    "verbose constraint number {} that adds significant token overhead to the skeleton TOML representation forcing it over budget",
                    i
                ));
            }
        }
        // Use a provider that counts 1 token per 2 chars (tight)
        let provider = MockProvider::new(2);
        let ssb = make_ssb(tmp.path());

        let skel_path =
            tmp.path().join("skeleton.toml");
        let result = skel
            .write_to_file(
                &skel_path, &provider, &ssb, None,
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("exceeds token budget"),
            "expected SkeletonBudgetExceeded, got: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn estimate_undercount_caught_by_safety_margin()
    {
        let tmp = TempDir::new().unwrap();
        // Build skeleton that's borderline
        let mut skel = make_skeleton(30, 100);
        for entry in skel.phases.values_mut() {
            for i in 0..5 {
                entry.constraints.push(format!(
                    "constraint {i} padding text for token budget testing"
                ));
            }
        }
        // Provider where estimate underestimates by ~20%:
        // estimate says ~3500 (under 3600 threshold),
        // but exact says ~4200 (over 4096 cap)
        // We need to tune divisors so that:
        //   text.len()/estimate_div ~ 3500 (passes estimate)
        //   text.len()/exact_div ~ 4200 (fails exact)
        // With this setup, estimate > 3600 triggers exact check.
        let toml_preview = skel.to_toml().unwrap();
        let text_len = toml_preview.len();

        // Compute divisors that produce the right estimates
        let estimate_divisor = text_len / 3700; // estimate ~3700 > 3600 -> triggers exact
        let exact_divisor = text_len / 4200; // exact ~4200 > 4096 -> rejected

        let provider = UnderestimatingProvider {
            estimate_divisor: estimate_divisor.max(1),
            exact_divisor: exact_divisor.max(1),
        };
        let ssb = make_ssb(tmp.path());

        let skel_path =
            tmp.path().join("skeleton.toml");
        let result = skel
            .write_to_file(
                &skel_path, &provider, &ssb, None,
            )
            .await;
        assert!(
            result.is_err(),
            "safety margin should trigger exact check which rejects"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn version_not_incremented_rejects_write() {
        let tmp = TempDir::new().unwrap();
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());

        // Previous skeleton
        let mut prev = PlanSkeleton::new("pipe-1");
        let entry =
            PhaseEntry::new("3B", "SSB single-writer")
                .unwrap();
        prev.add_phase(entry).unwrap();

        // Modified skeleton: changed contracts but
        // did NOT increment versions
        let mut current = prev.clone();
        current
            .phases
            .get_mut("3B")
            .unwrap()
            .inputs
            .push(ContractRef {
                name: "new_input".into(),
                type_signature: "fn() -> ()".into(),
                invariants: vec![],
            });
        // version still 0 for both phase and skeleton

        let skel_path =
            tmp.path().join("skeleton.toml");
        let result = current
            .write_to_file(
                &skel_path,
                &provider,
                &ssb,
                Some(&prev),
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("version not incremented"),
            "expected VersionNotIncremented, got: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn version_incremented_allows_write() {
        let tmp = TempDir::new().unwrap();
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());

        let mut prev = PlanSkeleton::new("pipe-1");
        let entry =
            PhaseEntry::new("3B", "SSB single-writer")
                .unwrap();
        prev.add_phase(entry).unwrap();

        let mut current = prev.clone();
        current
            .phases
            .get_mut("3B")
            .unwrap()
            .inputs
            .push(ContractRef {
                name: "new_input".into(),
                type_signature: "fn() -> ()".into(),
                invariants: vec![],
            });
        // Properly increment versions
        current
            .phases
            .get_mut("3B")
            .unwrap()
            .increment_version()
            .unwrap();
        current.increment_version().unwrap();

        let skel_path =
            tmp.path().join("skeleton.toml");
        let result = current
            .write_to_file(
                &skel_path,
                &provider,
                &ssb,
                Some(&prev),
            )
            .await;
        assert!(
            result.is_ok(),
            "write should succeed when versions incremented: {:?}",
            result.unwrap_err()
        );
    }

    #[test]
    fn read_from_nonexistent_file_returns_error() {
        let result = PlanSkeleton::read_from_file(
            Path::new("/nonexistent/skeleton.toml"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn read_from_invalid_toml_returns_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.toml");
        std::fs::write(&path, "not valid toml {{{}}}").unwrap();
        let result = PlanSkeleton::read_from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn empty_skeleton_round_trip() {
        let skel = PlanSkeleton::new("empty-pipe");
        let toml_str = skel.to_toml().unwrap();
        let parsed =
            PlanSkeleton::from_toml(&toml_str).unwrap();
        assert_eq!(skel, parsed);
        assert!(parsed.phases.is_empty());
    }

    // -- 3G.4 phase elaboration loading tests --

    fn make_chain_skeleton() -> PlanSkeleton {
        let mut skel = PlanSkeleton::new("chain-pipe");
        let mut a = PhaseEntry::new("A", "phase A goal").unwrap();
        a.outputs.push(ContractRef {
            name: "A::output".into(),
            type_signature: "fn() -> Vec<u8>".into(),
            invariants: vec!["deterministic".into()],
        });
        a.depended_by.push("B".into());
        skel.add_phase(a).unwrap();

        let mut b = PhaseEntry::new("B", "phase B goal").unwrap();
        b.inputs.push(ContractRef {
            name: "B::input".into(),
            type_signature: "fn(Vec<u8>) -> ()".into(),
            invariants: vec![],
        });
        b.outputs.push(ContractRef {
            name: "B::output".into(),
            type_signature: "fn() -> String".into(),
            invariants: vec![],
        });
        b.depends_on.push("A".into());
        b.depended_by.push("C".into());
        skel.add_phase(b).unwrap();

        let mut c = PhaseEntry::new("C", "phase C goal").unwrap();
        c.inputs.push(ContractRef {
            name: "C::input".into(),
            type_signature: "fn(String) -> ()".into(),
            invariants: vec![],
        });
        c.depends_on.push("B".into());
        skel.add_phase(c).unwrap();

        skel.global_constraints
            .push("entropy-quarantined".into());
        skel
    }

    fn make_phase_detail(
        phase_id: &str,
        skeleton_version: u32,
        content_size: usize,
    ) -> PhaseDetail {
        let contracts = ExtractedContracts {
            input_contracts: vec![],
            output_contracts: vec![],
            shared_constraints: vec![],
        };
        PhaseDetail {
            phase_id: phase_id.to_owned(),
            skeleton_version,
            contracts,
            implemented_contracts: vec![],
            declared_interfaces: vec![],
            content: "x".repeat(content_size),
            token_count: 0,
        }
    }

    #[test]
    fn phase_detail_toml_round_trip() {
        let detail = make_phase_detail("B", 1, 200);
        let toml_str = detail.to_toml().unwrap();
        let parsed: PhaseDetail =
            toml::from_str(&toml_str).unwrap();
        assert_eq!(detail, parsed);
    }

    #[test]
    fn phase_detail_write_read_file() {
        let tmp = TempDir::new().unwrap();
        let detail = make_phase_detail("B", 1, 200);
        let path = tmp.path().join("phases").join("B.toml");
        detail.write_to_file(&path).unwrap();
        let loaded =
            PhaseDetail::read_from_file(&path).unwrap();
        assert_eq!(detail, loaded);
    }

    // DETERMINISTIC_CHECK 1: fits within budget
    #[test]
    fn load_phase_context_fits_within_budget() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        let provider = MockProvider::new(4);

        // Write a small phase detail
        let phases_dir = tmp.path().join("phases");
        let detail = make_phase_detail("B", 0, 100);
        let detail_path = phases_dir.join("B.toml");
        detail.write_to_file(&detail_path).unwrap();

        let ctx = skel
            .load_phase_context(
                "B",
                2000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(
            ctx.total_tokens <= 2000,
            "total_tokens {} should fit within budget 2000",
            ctx.total_tokens
        );
        assert!(!ctx.reduced, "should not be reduced");
        assert!(
            ctx.phase_detail.is_some(),
            "detail should be loaded"
        );
        assert!(
            ctx.skeleton_tokens > 0,
            "skeleton tokens must be nonzero"
        );
    }

    // DETERMINISTIC_CHECK 2: exceeds budget -> reduction
    #[test]
    fn load_phase_context_exceeds_triggers_reduction() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        // Provider with tight token counting (1 token per 2 chars)
        let provider = MockProvider::new(2);

        // Write a large phase detail
        let phases_dir = tmp.path().join("phases");
        let detail = make_phase_detail("B", 0, 3000);
        let detail_path = phases_dir.join("B.toml");
        detail.write_to_file(&detail_path).unwrap();

        // Use a very tight budget
        let skeleton_toml = skel.to_toml().unwrap();
        let skel_tokens =
            provider.token_count(&skeleton_toml).unwrap();

        let ctx = skel
            .load_phase_context(
                "B",
                skel_tokens + 200, // just enough for skeleton + contracts, not detail
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(
            ctx.reduced,
            "should be reduced when detail pushes over budget"
        );
        assert!(
            ctx.phase_detail.is_none(),
            "detail should be dropped in reduction"
        );
    }

    // DETERMINISTIC_CHECK 3: staleness warning
    #[test]
    fn load_phase_context_staleness_warning() {
        let tmp = TempDir::new().unwrap();
        let mut skel = make_chain_skeleton();
        skel.version = 5; // skeleton advanced

        let phases_dir = tmp.path().join("phases");
        let detail = make_phase_detail("B", 2, 100); // elaborated at v2
        let detail_path = phases_dir.join("B.toml");
        detail.write_to_file(&detail_path).unwrap();

        let provider = MockProvider::new(4);
        let ctx = skel
            .load_phase_context(
                "B",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(
            ctx.staleness_warning.is_some(),
            "should warn about stale detail"
        );
        let warning = ctx.staleness_warning.unwrap();
        assert_eq!(warning.detail_version, 2);
        assert_eq!(warning.current_version, 5);
    }

    // DETERMINISTIC_CHECK 4: adjacent contracts included
    #[test]
    fn load_phase_context_includes_adjacent_contracts() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        let provider = MockProvider::new(4);
        let phases_dir = tmp.path().join("phases");

        let ctx = skel
            .load_phase_context(
                "B",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        // B depends on A (has outputs) and is depended by C (has inputs)
        assert!(
            !ctx.adjacent_contracts.is_empty(),
            "should include adjacent contracts"
        );
        let adj_ids: Vec<&str> = ctx
            .adjacent_contracts
            .iter()
            .map(|a| a.phase_id.as_str())
            .collect();
        assert!(
            adj_ids.contains(&"A"),
            "A's outputs should be in adjacent: {:?}",
            adj_ids
        );
        assert!(
            adj_ids.contains(&"C"),
            "C's inputs should be in adjacent: {:?}",
            adj_ids
        );
    }

    // DETERMINISTIC_CHECK: no detail file -> detail is None, no error
    #[test]
    fn load_phase_context_no_detail_file() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        let provider = MockProvider::new(4);
        let phases_dir = tmp.path().join("phases");
        std::fs::create_dir_all(&phases_dir).unwrap();

        let ctx = skel
            .load_phase_context(
                "B",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(
            ctx.phase_detail.is_none(),
            "no detail file means None"
        );
        assert!(
            !ctx.reduced,
            "not reduced — simply no detail to load"
        );
    }

    // ADVERSARIAL 1: skeleton alone exceeds Slot 3 budget
    #[test]
    fn load_phase_context_skeleton_exceeds_slot3() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        // Tight provider: 1 token per char
        let provider = MockProvider::new(1);
        let phases_dir = tmp.path().join("phases");

        let result = skel.load_phase_context(
            "B",
            10, // impossibly small budget
            &provider,
            &phases_dir,
            None,
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("exceeds Slot 3 budget"),
            "expected SlotBudgetExceeded, got: {err}"
        );
    }

    // ADVERSARIAL 2: skeleton_version mismatch forces re-extraction
    #[test]
    fn load_phase_context_stale_forces_reextraction() {
        let tmp = TempDir::new().unwrap();
        let mut skel = make_chain_skeleton();
        skel.version = 10;

        let phases_dir = tmp.path().join("phases");
        // Detail elaborated at skeleton v3 with stale contracts
        let mut detail = make_phase_detail("B", 3, 100);
        detail.contracts = ExtractedContracts {
            input_contracts: vec![ContractRef {
                name: "stale_contract".into(),
                type_signature: "fn() -> ()".into(),
                invariants: vec![],
            }],
            output_contracts: vec![],
            shared_constraints: vec![],
        };
        let detail_path = phases_dir.join("B.toml");
        detail.write_to_file(&detail_path).unwrap();

        let provider = MockProvider::new(4);
        let ctx = skel
            .load_phase_context(
                "B",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        // Contracts are freshly extracted from skeleton, not from detail
        assert_eq!(
            ctx.contracts,
            skel.extract_contracts("B").unwrap(),
            "contracts must be freshly extracted, not from stale detail"
        );
        assert!(ctx.staleness_warning.is_some());
    }

    // ADVERSARIAL: phase_id not found
    #[test]
    fn load_phase_context_phase_not_found() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        let provider = MockProvider::new(4);
        let phases_dir = tmp.path().join("phases");

        let result = skel.load_phase_context(
            "NONEXISTENT",
            5000,
            &provider,
            &phases_dir,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("phase not found"));
    }

    // No staleness when versions match
    #[test]
    fn load_phase_context_no_staleness_when_versions_match()
    {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton(); // version = 0

        let phases_dir = tmp.path().join("phases");
        let detail = make_phase_detail("B", 0, 100); // same version
        detail
            .write_to_file(&phases_dir.join("B.toml"))
            .unwrap();

        let provider = MockProvider::new(4);
        let ctx = skel
            .load_phase_context(
                "B",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(
            ctx.staleness_warning.is_none(),
            "no staleness when versions match"
        );
    }

    // Reduced context still fits: skeleton + contracts + adjacent < budget
    #[test]
    fn load_phase_context_reduced_fits_budget() {
        let tmp = TempDir::new().unwrap();
        let skel = make_chain_skeleton();
        let provider = MockProvider::new(2); // tight counting

        let phases_dir = tmp.path().join("phases");
        let detail = make_phase_detail("B", 0, 5000); // huge detail
        detail
            .write_to_file(&phases_dir.join("B.toml"))
            .unwrap();

        // Budget big enough for skeleton+contracts but not detail
        let skeleton_toml = skel.to_toml().unwrap();
        let skel_tokens =
            provider.token_count(&skeleton_toml).unwrap();

        let ctx = skel
            .load_phase_context(
                "B",
                skel_tokens + 300,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(ctx.reduced);
        assert!(
            ctx.total_tokens <= skel_tokens + 300,
            "reduced context must fit: {} <= {}",
            ctx.total_tokens,
            skel_tokens + 300
        );
    }

    // Leaf phase (no deps, no dependents) loads fine
    #[test]
    fn load_phase_context_leaf_phase() {
        let tmp = TempDir::new().unwrap();
        let mut skel = PlanSkeleton::new("leaf-pipe");
        let entry =
            PhaseEntry::new("X", "standalone phase")
                .unwrap();
        skel.add_phase(entry).unwrap();

        let provider = MockProvider::new(4);
        let phases_dir = tmp.path().join("phases");

        let ctx = skel
            .load_phase_context(
                "X",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .unwrap();

        assert!(ctx.adjacent_contracts.is_empty());
        assert!(ctx.contracts.input_contracts.is_empty());
        assert!(ctx.contracts.output_contracts.is_empty());
        assert!(!ctx.reduced);
    }

    // -- 3G.6 cycle detection tests --

    fn make_linear_dag() -> PlanSkeleton {
        let mut skel = PlanSkeleton::new("dag-pipe");
        let mut a =
            PhaseEntry::new("A", "phase A").unwrap();
        a.depended_by.push("B".into());
        skel.add_phase(a).unwrap();

        let mut b =
            PhaseEntry::new("B", "phase B").unwrap();
        b.depends_on.push("A".into());
        b.depended_by.push("C".into());
        skel.add_phase(b).unwrap();

        let mut c =
            PhaseEntry::new("C", "phase C").unwrap();
        c.depends_on.push("B".into());
        c.depended_by.push("D".into());
        skel.add_phase(c).unwrap();

        let mut d =
            PhaseEntry::new("D", "phase D").unwrap();
        d.depends_on.push("C".into());
        skel.add_phase(d).unwrap();

        skel
    }

    #[test]
    fn kahn_acyclic_linear_chain() {
        let skel = make_linear_dag();
        assert!(skel.kahn_cycle_check().is_ok());
    }

    #[test]
    fn kahn_cycle_detected_three_phases() {
        let mut skel = PlanSkeleton::new("cycle-pipe");
        let mut a =
            PhaseEntry::new("A", "phase A").unwrap();
        a.depends_on.push("C".into());
        a.depended_by.push("B".into());
        skel.add_phase(a).unwrap();

        let mut b =
            PhaseEntry::new("B", "phase B").unwrap();
        b.depends_on.push("A".into());
        b.depended_by.push("C".into());
        skel.add_phase(b).unwrap();

        let mut c =
            PhaseEntry::new("C", "phase C").unwrap();
        c.depends_on.push("B".into());
        c.depended_by.push("A".into());
        skel.add_phase(c).unwrap();

        let err = skel.kahn_cycle_check().unwrap_err();
        match err {
            PlanSkeletonError::CycleDetected {
                phases,
            } => {
                assert_eq!(
                    phases,
                    vec!["A", "B", "C"]
                );
            }
            other => panic!(
                "expected CycleDetected, got: {other}"
            ),
        }
    }

    #[test]
    fn kahn_empty_skeleton_ok() {
        let skel = PlanSkeleton::new("empty-pipe");
        assert!(skel.kahn_cycle_check().is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_rejects_cyclic_graph() {
        let tmp = TempDir::new().unwrap();
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());

        let mut skel = PlanSkeleton::new("cyc-pipe");
        let mut a =
            PhaseEntry::new("A", "phase A").unwrap();
        a.depends_on.push("B".into());
        a.depended_by.push("B".into());
        skel.add_phase(a).unwrap();

        let mut b =
            PhaseEntry::new("B", "phase B").unwrap();
        b.depends_on.push("A".into());
        b.depended_by.push("A".into());
        skel.add_phase(b).unwrap();

        let path = tmp.path().join("skeleton.toml");
        let result = skel
            .write_to_file(
                &path, &provider, &ssb, None,
            )
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cycle detected"));
    }

    // -- 3G.6 propagate_changes tests --

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn propagate_chain_flags_all_downstream() {
        let tmp = TempDir::new().unwrap();
        let skel = make_linear_dag();
        let ssb = make_ssb(tmp.path());

        let flagged = skel
            .propagate_changes(
                &"A".to_owned(),
                &ssb,
            )
            .await
            .unwrap();
        assert_eq!(flagged, vec!["B", "C", "D"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn propagate_diamond_no_duplicates() {
        let tmp = TempDir::new().unwrap();
        let mut skel = PlanSkeleton::new("diamond-pipe");

        let mut a =
            PhaseEntry::new("A", "phase A").unwrap();
        a.depended_by.push("B".into());
        a.depended_by.push("C".into());
        skel.add_phase(a).unwrap();

        let mut b =
            PhaseEntry::new("B", "phase B").unwrap();
        b.depends_on.push("A".into());
        b.depended_by.push("D".into());
        skel.add_phase(b).unwrap();

        let mut c =
            PhaseEntry::new("C", "phase C").unwrap();
        c.depends_on.push("A".into());
        c.depended_by.push("D".into());
        skel.add_phase(c).unwrap();

        let mut d =
            PhaseEntry::new("D", "phase D").unwrap();
        d.depends_on.push("B".into());
        d.depends_on.push("C".into());
        skel.add_phase(d).unwrap();

        let ssb = make_ssb(tmp.path());
        let flagged = skel
            .propagate_changes(
                &"A".to_owned(),
                &ssb,
            )
            .await
            .unwrap();

        // D must appear exactly once
        let d_count = flagged
            .iter()
            .filter(|p| p.as_str() == "D")
            .count();
        assert_eq!(
            d_count, 1,
            "D must be flagged exactly once, got {d_count}"
        );
        assert_eq!(flagged.len(), 3); // B, C, D
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn propagate_isolated_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let mut skel = PlanSkeleton::new("iso-pipe");
        let e =
            PhaseEntry::new("E", "isolated").unwrap();
        skel.add_phase(e).unwrap();

        let ssb = make_ssb(tmp.path());
        let flagged = skel
            .propagate_changes(
                &"E".to_owned(),
                &ssb,
            )
            .await
            .unwrap();
        assert!(
            flagged.is_empty(),
            "isolated phase should flag nobody"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn propagate_unknown_phase_returns_error() {
        let tmp = TempDir::new().unwrap();
        let skel = PlanSkeleton::new("empty-pipe");
        let ssb = make_ssb(tmp.path());

        let result = skel
            .propagate_changes(
                &"GHOST".to_owned(),
                &ssb,
            )
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("phase not found"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn propagate_records_ssb_entries() {
        let tmp = TempDir::new().unwrap();
        let skel = make_linear_dag();
        let ssb = make_ssb(tmp.path());

        let flagged = skel
            .propagate_changes(
                &"A".to_owned(),
                &ssb,
            )
            .await
            .unwrap();

        let entries = ssb
            .query_by_type(SsbActionType::PlanUpdate)
            .unwrap();
        assert_eq!(
            entries.len(),
            flagged.len(),
            "one SSB entry per flagged phase"
        );
    }

    // -- 3G.6 split_phase tests --

    #[test]
    fn split_phase_basic_two_way() {
        let mut skel = make_linear_dag();
        skel.split_phase(
            &"B".to_owned(),
            &["sub-goal 1".to_owned()],
            false,
        )
        .unwrap();

        assert!(
            !skel.phases.contains_key("B"),
            "original B must be removed"
        );
        assert!(skel.phases.contains_key("B_1"));
        assert!(skel.phases.contains_key("B_2"));

        // B_1 depends on A, B_2 depends on B_1
        assert_eq!(
            skel.phases["B_1"].depends_on,
            vec!["A"]
        );
        assert_eq!(
            skel.phases["B_2"].depends_on,
            vec!["B_1"]
        );

        // B_2 is depended_by C (inherited from B)
        assert!(
            skel.phases["B_2"]
                .depended_by
                .contains(&"C".to_owned()),
        );

        // Split depth = 1
        assert_eq!(skel.phases["B_1"].split_depth, 1);
        assert_eq!(skel.phases["B_2"].split_depth, 1);
    }

    #[test]
    fn split_phase_preserves_contracts() {
        let mut skel = PlanSkeleton::new("split-pipe");
        let mut b =
            PhaseEntry::new("B", "phase B").unwrap();
        b.inputs.push(ContractRef {
            name: "x_in".into(),
            type_signature: "fn() -> X".into(),
            invariants: vec![],
        });
        b.outputs.push(ContractRef {
            name: "y_out".into(),
            type_signature: "fn() -> Y".into(),
            invariants: vec![],
        });
        skel.add_phase(b).unwrap();

        skel.split_phase(
            &"B".to_owned(),
            &["split-goal".to_owned()],
            true,
        )
        .unwrap();

        // B_1 inherits B's inputs
        assert_eq!(
            skel.phases["B_1"].inputs[0].name,
            "x_in"
        );
        // B_2 inherits B's outputs
        assert_eq!(
            skel.phases["B_2"].outputs[0].name,
            "y_out"
        );
        // B_1 has internal output contract
        assert!(skel.phases["B_1"]
            .outputs
            .iter()
            .any(|c| c.name == "B_1::split_output"));
        // B_2 has internal input contract
        assert!(skel.phases["B_2"]
            .inputs
            .iter()
            .any(|c| c.name == "B_1::split_output"));
    }

    #[test]
    fn split_phase_depth_exceeded() {
        let mut skel = PlanSkeleton::new("depth-pipe");
        let mut b =
            PhaseEntry::new("B", "phase B").unwrap();
        b.split_depth = 2; // already at max
        skel.add_phase(b).unwrap();

        let result = skel.split_phase(
            &"B".to_owned(),
            &["g1".to_owned()],
            false,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("split depth exceeded"));
    }

    #[test]
    fn split_phase_not_found() {
        let mut skel = PlanSkeleton::new("empty-pipe");
        let result = skel.split_phase(
            &"GHOST".to_owned(),
            &["g".to_owned()],
            false,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("phase not found"));
    }

    #[test]
    fn split_phase_three_way() {
        let mut skel = PlanSkeleton::new("tri-pipe");
        let b =
            PhaseEntry::new("B", "phase B").unwrap();
        skel.add_phase(b).unwrap();

        skel.split_phase(
            &"B".to_owned(),
            &[
                "g1".to_owned(),
                "g2".to_owned(),
            ],
            false,
        )
        .unwrap();

        assert!(skel.phases.contains_key("B_1"));
        assert!(skel.phases.contains_key("B_2"));
        assert!(skel.phases.contains_key("B_3"));
        assert!(!skel.phases.contains_key("B"));
    }

    #[test]
    fn split_updates_upstream_depended_by() {
        let mut skel = make_linear_dag();
        skel.split_phase(
            &"B".to_owned(),
            &["g1".to_owned()],
            false,
        )
        .unwrap();

        // A's depended_by should now reference B_1, not B
        assert!(
            skel.phases["A"]
                .depended_by
                .contains(&"B_1".to_owned()),
            "A should depend_by B_1"
        );
        assert!(
            !skel.phases["A"]
                .depended_by
                .contains(&"B".to_owned()),
            "A should no longer depend_by B"
        );
    }

    #[test]
    fn split_updates_downstream_depends_on() {
        let mut skel = make_linear_dag();
        skel.split_phase(
            &"B".to_owned(),
            &["g1".to_owned()],
            false,
        )
        .unwrap();

        // C's depends_on should now reference B_2, not B
        assert!(
            skel.phases["C"]
                .depends_on
                .contains(&"B_2".to_owned()),
            "C should depend_on B_2"
        );
        assert!(
            !skel.phases["C"]
                .depends_on
                .contains(&"B".to_owned()),
            "C should no longer depend_on B"
        );
    }

    // -- 3G.7 integration test --

    /// Build 5-phase skeleton: A->B->C, A->D, D->E.
    /// A.outputs and C.inputs carry ContractRefs for extract_contracts testing.
    fn make_5phase_dag() -> PlanSkeleton {
        let mut skel = PlanSkeleton::new("integ-5phase");

        let mut a =
            PhaseEntry::new("A", "phase A goal").unwrap();
        a.outputs.push(ContractRef {
            name: "data_stream".into(),
            type_signature: "Vec<u8>".into(),
            invariants: vec!["non-empty".into()],
        });
        a.depended_by.push("B".into());
        a.depended_by.push("D".into());
        skel.add_phase(a).unwrap();

        let mut b =
            PhaseEntry::new("B", "phase B goal").unwrap();
        b.depends_on.push("A".into());
        b.depended_by.push("C".into());
        skel.add_phase(b).unwrap();

        let mut c =
            PhaseEntry::new("C", "phase C goal").unwrap();
        c.depends_on.push("B".into());
        c.inputs.push(ContractRef {
            name: "processed_data".into(),
            type_signature: "String".into(),
            invariants: vec!["utf8-valid".into()],
        });
        skel.add_phase(c).unwrap();

        let mut d =
            PhaseEntry::new("D", "phase D goal").unwrap();
        d.depends_on.push("A".into());
        d.depended_by.push("E".into());
        skel.add_phase(d).unwrap();

        let e =
            PhaseEntry::new("E", "phase E goal").unwrap();
        // E depends on D but PhaseEntry::new doesn't set it
        let mut e = e;
        e.depends_on.push("D".into());
        skel.add_phase(e).unwrap();

        skel
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn lifecycle_full_5phase_integration() {
        let tmp = TempDir::new().unwrap();
        let provider = MockProvider::new(4);
        let ssb = make_ssb(tmp.path());
        let mut skel = make_5phase_dag();

        // Step 1: write_to_file
        let skel_path =
            tmp.path().join("skeleton.toml");
        skel.write_to_file(
            &skel_path, &provider, &ssb, None,
        )
        .await
        .expect("write_to_file should succeed");

        // Step 2: verify SSB entries
        let plan_updates = ssb
            .query_by_type(SsbActionType::PlanUpdate)
            .expect("query PlanUpdate");
        assert!(
            !plan_updates.is_empty(),
            "SSB must have at least 1 PlanUpdate after write"
        );
        let stub_refs = ssb
            .query_by_type(SsbActionType::StubReference)
            .expect("query StubReference");
        assert_eq!(
            stub_refs.len(),
            5,
            "5 phases without detail files = 5 StubReference entries"
        );

        // Step 3: extract_contracts("B")
        let extracted = skel
            .extract_contracts("B")
            .expect("extract_contracts B");
        // B depends_on A => input_contracts = A.outputs
        assert_eq!(extracted.input_contracts.len(), 1);
        assert_eq!(
            extracted.input_contracts[0].name,
            "data_stream"
        );
        // B depended_by C => output_contracts = C.inputs
        assert_eq!(extracted.output_contracts.len(), 1);
        assert_eq!(
            extracted.output_contracts[0].name,
            "processed_data"
        );

        // Step 4: load_phase_context("B", budget=5000)
        let phases_dir = tmp.path().join("phases");
        std::fs::create_dir_all(&phases_dir)
            .expect("create phases_dir");
        let ctx = skel
            .load_phase_context(
                "B",
                5000,
                &provider,
                &phases_dir,
                None,
            )
            .expect("load_phase_context B");
        assert!(
            ctx.total_tokens <= 5000,
            "total_tokens {} must fit slot3 budget",
            ctx.total_tokens,
        );
        assert!(
            ctx.phase_detail.is_none(),
            "no detail file written yet"
        );
        assert!(
            ctx.staleness_warning.is_none(),
            "no staleness when detail is None"
        );

        // Step 5: valid coherence check
        let all_contracts: Vec<ContractRef> = extracted
            .input_contracts
            .iter()
            .chain(extracted.output_contracts.iter())
            .cloned()
            .collect();
        // declared_interfaces: union of all interface names in the skeleton
        // that B can reference (A.outputs + C.inputs + B.inputs + B.outputs)
        let declared_interfaces: Vec<ContractRef> =
            skel.phases["A"]
                .outputs
                .iter()
                .chain(skel.phases["C"].inputs.iter())
                .chain(skel.phases["B"].inputs.iter())
                .chain(skel.phases["B"].outputs.iter())
                .cloned()
                .collect();
        let valid_detail = PhaseDetail {
            phase_id: "B".into(),
            skeleton_version: skel.version,
            contracts: extracted.clone(),
            implemented_contracts: all_contracts.clone(),
            declared_interfaces,
            content: "phase B elaboration content".into(),
            token_count: 42,
        };
        let valid_report = run_coherence_checks(
            &valid_detail,
            &extracted,
            &skel,
            None,
        );
        assert_eq!(
            valid_report.block_count, 0,
            "valid detail should have 0 BLOCKs, got violations: {:?}",
            valid_report.violations,
        );

        // Step 6: invalid coherence check — missing one implemented contract
        let mut incomplete_contracts = all_contracts.clone();
        incomplete_contracts.pop(); // remove last
        let invalid_detail = PhaseDetail {
            phase_id: "B".into(),
            skeleton_version: skel.version,
            contracts: extracted.clone(),
            implemented_contracts: incomplete_contracts,
            declared_interfaces: valid_detail
                .declared_interfaces
                .clone(),
            content: "phase B partial".into(),
            token_count: 30,
        };
        let invalid_report = run_coherence_checks(
            &invalid_detail,
            &extracted,
            &skel,
            None,
        );
        assert!(
            invalid_report.block_count >= 1,
            "missing contract should produce at least 1 BLOCK"
        );

        // Step 7: mutate A.outputs, increment versions, propagate
        let prev_skel = skel.clone();
        skel.phases
            .get_mut("A")
            .expect("phase A exists")
            .outputs[0]
            .type_signature = "Vec<u16>".into();
        skel.phases
            .get_mut("A")
            .expect("phase A exists")
            .increment_version()
            .expect("increment A version");
        skel.increment_version()
            .expect("increment skeleton version");

        let flagged = skel
            .propagate_changes(
                &"A".to_owned(),
                &ssb,
            )
            .await
            .expect("propagate_changes A");

        assert_eq!(
            flagged.len(),
            4,
            "A change must flag B, C, D, E; got {:?}",
            flagged,
        );
        assert!(
            flagged.contains(&"B".to_owned()),
            "B must be flagged"
        );
        assert!(
            flagged.contains(&"C".to_owned()),
            "C must be flagged"
        );
        assert!(
            flagged.contains(&"D".to_owned()),
            "D must be flagged"
        );
        assert!(
            flagged.contains(&"E".to_owned()),
            "E must be flagged"
        );

        // Verify SSB PlanUpdate count: 1 from write + 4 from propagation
        let all_plan_updates = ssb
            .query_by_type(SsbActionType::PlanUpdate)
            .expect("query PlanUpdate after propagation");
        assert!(
            all_plan_updates.len() >= 5,
            "expected >= 5 PlanUpdate entries (1 write + 4 propagation), got {}",
            all_plan_updates.len(),
        );

        // Step 8: final version > 0
        assert!(
            skel.version > 0,
            "skeleton version must be > 0 after increment"
        );

        // Write with previous skeleton to verify version enforcement passes
        let skel_path2 =
            tmp.path().join("skeleton_v2.toml");
        let ssb2 = make_ssb(tmp.path());
        skel.write_to_file(
            &skel_path2,
            &provider,
            &ssb2,
            Some(&prev_skel),
        )
        .await
        .expect(
            "write_to_file with version-bumped skeleton should succeed",
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn lifecycle_determinism_100_runs() {
        let skel = make_5phase_dag();
        let iteration_count = 100;
        let mut all_flagged: Vec<Vec<PhaseId>> =
            Vec::with_capacity(iteration_count);

        for _ in 0..iteration_count {
            let tmp = TempDir::new().unwrap();
            let ssb = make_ssb(tmp.path());
            let flagged = skel
                .propagate_changes(
                    &"A".to_owned(),
                    &ssb,
                )
                .await
                .expect("propagate_changes must succeed");
            assert_eq!(
                flagged.len(),
                4,
                "each iteration must flag exactly 4 phases"
            );
            all_flagged.push(flagged);
        }

        // All iterations must produce identical output
        let reference = &all_flagged[0];
        for (i, flagged) in
            all_flagged.iter().enumerate().skip(1)
        {
            assert_eq!(
                flagged, reference,
                "iteration {} diverged from reference: {:?} vs {:?}",
                i, flagged, reference,
            );
        }
    }
}
