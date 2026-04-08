//! Hierarchical plan management: skeleton types and TOML serialization.
//!
//! Phase 3G.1 types + Phase 3G.2 serialization with token budget enforcement,
//! SSB recording, and version tracking.

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

        let total_full = skeleton_tokens
            + contracts_tokens
            + adjacent_tokens
            + detail_tokens;

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
            });
        }

        // Adaptive reduction: skeleton + contracts only, drop detail.
        let total_reduced = skeleton_tokens
            + contracts_tokens
            + adjacent_tokens;

        Ok(PhaseContext {
            skeleton_tokens,
            phase_detail: None,
            contracts,
            adjacent_contracts,
            total_tokens: total_reduced,
            reduced: true,
            staleness_warning,
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
                plan_hash: self.pipeline_id.clone(),
                phase: self.version.to_string(),
                step: self.phases.len().to_string(),
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
                            original_seq: i64::from(
                                self.version,
                            ),
                            stub_reason: phase_id.clone(),
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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

    #[tokio::test]
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
            )
            .unwrap();

        assert!(ctx.adjacent_contracts.is_empty());
        assert!(ctx.contracts.input_contracts.is_empty());
        assert!(ctx.contracts.output_contracts.is_empty());
        assert!(!ctx.reduced);
    }
}
