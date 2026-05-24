// C-25 Design 1: Compiled pipeline cache, template crystallizer, and
// compiled executor.
//
// SYNOPSIS
// --------
// After a task has succeeded N=3 consecutive times for the same
// (concept_class, plan_skeleton_hash) tuple, the `TemplateCrystallizer`
// freezes the pipeline shape into a `CompiledPipeline` and writes it to
// disk. On run N+1, the `CompiledExecutor` short-circuits the LLM-based
// decomposition path: the cached steps are emitted directly, producing a
// `plan_skeleton` SSB row with `source=Template`. On failure (or low
// confidence), the executor signals `Decompile` and the caller falls
// through to the normal LLM path.
//
// GROUNDING
// ---------
// Mechanism per `docs/specs/guide/compiled-pipeline-design.txt` Designs 1
// + 3, with the C-25 plan's WIRING_CONTRACT and stub_ban contract as
// the authoritative spec. See `workflow/reasoning/C-25-decisions.md`
// for decisions (a) margin, (b) decompile threshold, (c) TTL, (d) hash
// crate choice, (e) attention-ceiling clamp.
//
// NO STUBS, no .unwrap() in library paths (only inside #[cfg(test)]).
//
// ---------------------------------------------------------------------------

// B-15 HashMap ban: BTreeMap chosen for observation_log, pipeline_cache,
// and consecutive_failures. (String, u64) keys derive Ord natively, all
// access patterns are entry/get/remove/len/read/write (no iteration order
// dependency). BTreeMap also gives deterministic debug output via the
// `Debug` impl which is the only consumer of `.len()` on these maps.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::ssb::{
    CompiledPipelineFiredPayload, DecompileTriggeredPayload, DecompileTriggerReason,
    Ssb, SsbPayload, TemplateCrystallizedPayload,
};

// ---------------------------------------------------------------------------
// Constants — named, no magic numbers (per codeRules).
// ---------------------------------------------------------------------------

/// C-25 Task 6 — number of consecutive successful runs before
/// `TemplateCrystallizer::check_crystallization` materializes a
/// `CompiledPipeline`. Spec default = 3.
pub const CRYSTALLIZATION_THRESHOLD_N: u32 = 3;

/// C-25 Task 7 — confidence floor for `CompiledExecutor::try_execute`.
/// Below this, the executor returns `Err(Decompile)`.
pub const COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD: f32 = 0.70;

/// C-25 Task 7 + Q4 — consecutive-failure threshold for
/// `CompiledExecutor`. Two failures (not one) before decompiling so a
/// single transient flake does not invalidate a compiled pipeline.
pub const COMPILED_EXECUTOR_FAILURE_THRESHOLD: u8 = 2;

/// C-25 Task 5 — `compilation_eligible` requires max variable-region
/// ratio strictly below this number. 0.3 = at most 30% of any step's
/// template body may be variable regions; above this, the template is
/// too unstable to compile (per spec).
pub const VARIABLE_REGION_RATIO_CEILING: f32 = 0.30;

/// C-25 Task 5 — `compilation_eligible` requires at least this many
/// observed successful runs. Matches the crystallization threshold so a
/// pipeline that just crystallized always passes the eligibility check.
pub const COMPILATION_ELIGIBLE_MIN_USES: u32 = 3;

/// C-25 Q3 — TTL for on-disk compiled pipeline entries, in seconds.
/// 7 days = 604_800. Pipelines unused for longer than this are evicted
/// at `upsert` time. Justification: 7 days matches Anthropic's
/// prompt-cache breakpoint TTL norms and prevents pipelines from
/// accumulating across unrelated work weeks.
pub const COMPILED_PIPELINE_TTL_SECS: u64 = 604_800;

/// C-25 Task 8 — relative cache directory under the project root.
pub const COMPILED_PIPELINE_CACHE_DIR: &str = ".elai/cache";

/// C-25 Task 8 — cache filename for the persisted store.
pub const COMPILED_PIPELINE_CACHE_FILE: &str = "compiled_pipelines.msgpack";

// ---------------------------------------------------------------------------
// Newtype IDs
// ---------------------------------------------------------------------------

/// C-25 Task 5 — typed wrapper around a template UUID. Distinguishes
/// template-level identity from pipeline identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(pub Uuid);

impl TemplateId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TemplateId {
    fn default() -> Self {
        Self::new()
    }
}

/// C-25 Task 5 — typed wrapper around a per-run pipeline-outcome UUID.
/// Used as the correlation handle between an `ExecutionOutcome` and the
/// underlying baby-step trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineOutcomeId(pub Uuid);

impl PipelineOutcomeId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PipelineOutcomeId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Core types — TemplateVariable, CompiledStep, CompiledPipeline
// ---------------------------------------------------------------------------

/// C-25 Task 5 — classification of a variable region inside a compiled
/// step's template prompt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariableType {
    ConceptClass,
    TaskDescription,
    CodeTarget,
    Other(String),
}

/// C-25 Task 5 — one variable region inside a compiled step's template
/// prompt. `region_start` and `region_end` are byte indices into the
/// `template_prompt` string (half-open: end is exclusive).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub region_start: u32,
    pub region_end: u32,
    pub variable_type: VariableType,
}

impl TemplateVariable {
    /// Region length in bytes. Saturates at 0 if `region_end < region_start`
    /// (defensive — should never happen in well-formed templates).
    #[must_use]
    pub fn region_len(&self) -> u32 {
        self.region_end.saturating_sub(self.region_start)
    }
}

/// C-25 Task 5 — one step inside a compiled pipeline. `step_index` is
/// the position in the parent pipeline's step list (0-based). `l1_tool_only`
/// is true when this step's tool repertoire is restricted to L1 (search +
/// file_read + reasoning); compilation requires this on EVERY step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledStep {
    pub step_index: u8,
    pub template_prompt: String,
    pub variables: Vec<TemplateVariable>,
    pub variable_region_ratio: f32,
    pub l1_tool_only: bool,
}

impl CompiledStep {
    /// Compute `variable_region_ratio` directly from `template_prompt` and
    /// `variables`. Returns 0.0 for empty prompts.
    #[must_use]
    pub fn compute_ratio(template_prompt: &str, variables: &[TemplateVariable]) -> f32 {
        let total = template_prompt.len();
        if total == 0 {
            return 0.0;
        }
        let var_bytes: u64 = variables.iter().map(|v| u64::from(v.region_len())).sum();
        let total_u64 = total as u64;
        if var_bytes >= total_u64 {
            return 1.0;
        }
        (var_bytes as f64 / total_u64 as f64) as f32
    }
}

/// C-25 Task 5 — a fully crystallized pipeline. `pipeline_id` is a fresh
/// UUID per crystallization; `plan_skeleton_hash` is a blake3-truncated-u64
/// over the concept_class + step prompts (stable across rustc versions
/// per Q5 resolution). `created_at` and `last_used_at` are unix-epoch
/// seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledPipeline {
    pub pipeline_id: Uuid,
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub steps: Vec<CompiledStep>,
    pub created_at: u64,
    pub last_used_at: u64,
    pub use_count: u32,
    pub first_try_success_rate: f32,
    pub compilation_eligible: bool,
}

impl CompiledPipeline {
    /// Compute `compilation_eligible` from the current step list +
    /// observed use_count. Eligibility: every step is `l1_tool_only`,
    /// max variable_region_ratio < `VARIABLE_REGION_RATIO_CEILING`, AND
    /// use_count >= `COMPILATION_ELIGIBLE_MIN_USES`.
    #[must_use]
    pub fn compute_eligibility(steps: &[CompiledStep], use_count: u32) -> bool {
        if steps.is_empty() {
            return false;
        }
        let all_l1 = steps.iter().all(|s| s.l1_tool_only);
        let max_ratio = steps
            .iter()
            .map(|s| s.variable_region_ratio)
            .fold(0.0_f32, f32::max);
        all_l1
            && max_ratio < VARIABLE_REGION_RATIO_CEILING
            && use_count >= COMPILATION_ELIGIBLE_MIN_USES
    }

    /// Cache key tuple: `(concept_class, plan_skeleton_hash)`.
    #[must_use]
    pub fn cache_key(&self) -> (String, u64) {
        (self.concept_class.clone(), self.plan_skeleton_hash)
    }
}

// ---------------------------------------------------------------------------
// blake3 hash helper — stable across rustc versions (per Q5 resolution).
// ---------------------------------------------------------------------------

/// C-25 Q5 — stable blake3-truncated-u64 hash over the concept_class +
/// concatenated step prompts. Used by `classify_task_for_template` to
/// generate the `plan_skeleton_hash` cache key. Stable across rustc
/// versions, unlike `std::collections::hash_map::DefaultHasher`.
///
/// The function is `pub` so `pipeline_builder.rs`'s
/// `classify_task_for_template` can reuse it without re-deriving the
/// hashing scheme.
#[must_use]
pub fn stable_plan_skeleton_hash(concept_class: &str, step_prompts: &[&str]) -> u64 {
    let mut h = Hasher::new();
    h.update(concept_class.as_bytes());
    // Separator between concept_class and step list so "ab" + ["cd"]
    // and "a" + ["bcd"] do not collide.
    h.update(b"\x00steps\x00");
    for (i, prompt) in step_prompts.iter().enumerate() {
        h.update(&u32::try_from(i).unwrap_or(u32::MAX).to_le_bytes());
        h.update(prompt.as_bytes());
        h.update(b"\x00");
    }
    let bytes = h.finalize();
    let id_bytes: [u8; 8] = match bytes.as_bytes()[..8].try_into() {
        Ok(b) => b,
        Err(_) => [0; 8],
    };
    u64::from_le_bytes(id_bytes)
}

// ---------------------------------------------------------------------------
// TemplateCrystallizer — observation log + crystallization gate
// ---------------------------------------------------------------------------

/// C-25 Task 6 — one execution outcome row, appended to the
/// crystallizer's observation log per baby step completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    pub success: bool,
    pub concept_class: String,
    pub plan_skeleton_hash: u64,
    pub pipeline_id: Option<Uuid>,
    pub token_count: u64,
    pub timestamp: u64,
}

/// C-25 Task 6 — observation log + crystallization gate. Drives the
/// "N consecutive successes" rule that decides when to materialize a
/// CompiledPipeline.
pub struct TemplateCrystallizer {
    /// Per (concept_class, plan_skeleton_hash) bucket of historical outcomes.
    observation_log: BTreeMap<(String, u64), Vec<ExecutionOutcome>>,
    /// Consecutive-success threshold (default `CRYSTALLIZATION_THRESHOLD_N`).
    threshold: u32,
    /// Optional SSB writer for `template_crystallized` rows.
    ssb_tx: Option<Arc<Ssb>>,
}

impl TemplateCrystallizer {
    #[must_use]
    pub fn new(ssb_tx: Option<Arc<Ssb>>) -> Self {
        Self::with_threshold(ssb_tx, CRYSTALLIZATION_THRESHOLD_N)
    }

    #[must_use]
    pub fn with_threshold(ssb_tx: Option<Arc<Ssb>>, threshold: u32) -> Self {
        Self {
            observation_log: BTreeMap::new(),
            threshold,
            ssb_tx,
        }
    }

    /// Append an outcome to the observation log. Returns the new bucket
    /// length so callers can drive logging/metrics without re-locking.
    pub fn record_execution(&mut self, outcome: ExecutionOutcome) -> usize {
        let key = (outcome.concept_class.clone(), outcome.plan_skeleton_hash);
        let bucket = self.observation_log.entry(key).or_default();
        bucket.push(outcome);
        bucket.len()
    }

    /// Inspect the bucket at `(concept_class, plan_skeleton_hash)` and
    /// return Some(CompiledPipeline) iff the last `threshold` outcomes
    /// are ALL `success=true` AND the candidate pipeline passes
    /// `compute_eligibility`. Emits a `template_crystallized` SSB row
    /// on success.
    ///
    /// `step_template_provider` is invoked lazily (only when the
    /// threshold gate fires) so callers don't pay the cost of building
    /// `CompiledStep`s on every observation. Returns `None` if the
    /// gate failed OR if the provider returned an empty step list.
    pub async fn check_crystallization<F>(
        &self,
        concept_class: &str,
        plan_skeleton_hash: u64,
        step_template_provider: F,
        now_secs: u64,
    ) -> Option<CompiledPipeline>
    where
        F: FnOnce() -> Vec<CompiledStep>,
    {
        let key = (concept_class.to_string(), plan_skeleton_hash);
        let bucket = self.observation_log.get(&key)?;
        let threshold = self.threshold as usize;
        if bucket.len() < threshold {
            return None;
        }
        let tail = &bucket[bucket.len() - threshold..];
        if !tail.iter().all(|o| o.success) {
            return None;
        }
        // Threshold met — materialize the pipeline. If the provider yields
        // no steps, treat as ineligible (no crystallization).
        let steps = step_template_provider();
        if steps.is_empty() {
            return None;
        }
        let use_count = u32::try_from(bucket.len()).unwrap_or(u32::MAX);
        // first_try_success_rate over the OBSERVED bucket — strictly
        // observational; survives across threshold boundary because we
        // store the full bucket.
        let successes = bucket.iter().filter(|o| o.success).count();
        let first_try_success_rate = if bucket.is_empty() {
            0.0
        } else {
            (successes as f32) / (bucket.len() as f32)
        };
        let compilation_eligible = CompiledPipeline::compute_eligibility(&steps, use_count);
        if !compilation_eligible {
            return None;
        }
        let pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: concept_class.to_string(),
            plan_skeleton_hash,
            steps,
            created_at: now_secs,
            last_used_at: now_secs,
            use_count,
            first_try_success_rate,
            compilation_eligible,
        };
        // Emit the SSB row (best-effort — log + continue on failure).
        if let Some(ssb) = self.ssb_tx.as_ref() {
            let payload = SsbPayload::TemplateCrystallized(TemplateCrystallizedPayload {
                concept_class: concept_class.to_string(),
                plan_skeleton_hash,
                pipeline_id: pipeline.pipeline_id.to_string(),
                threshold_n: self.threshold,
            });
            if let Err(e) = ssb
                .write_payload("template_crystallizer", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "template_crystallizer: SSB write failed (non-fatal)"
                );
            }
        }
        Some(pipeline)
    }

    /// Test/diagnostics accessor — bucket length.
    #[must_use]
    pub fn bucket_len(&self, concept_class: &str, plan_skeleton_hash: u64) -> usize {
        self.observation_log
            .get(&(concept_class.to_string(), plan_skeleton_hash))
            .map_or(0, Vec::len)
    }
}

impl std::fmt::Debug for TemplateCrystallizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateCrystallizer")
            .field("buckets", &self.observation_log.len())
            .field("threshold", &self.threshold)
            .field("ssb_tx_wired", &self.ssb_tx.is_some())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// CompiledExecutor — pipeline cache lookup + decompile gate
// ---------------------------------------------------------------------------

/// C-25 Task 7 — typed error from `CompiledExecutor::try_execute`.
#[derive(Debug, Error)]
pub enum CompiledExecutorError {
    #[error("compiled-pipeline cache miss for key=({0}, {1:#x})")]
    CacheMiss(String, u64),
    /// Returned after `COMPILED_EXECUTOR_FAILURE_THRESHOLD` consecutive
    /// failures OR a confidence reading below
    /// `COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD`. The caller (in
    /// `pipeline_builder.rs`) is expected to fall through to the LLM
    /// decomposition path.
    #[error("compiled-pipeline decompile triggered for key=({0}, {1:#x})")]
    Decompile(String, u64),
}

/// C-25 Task 7 — compiled-pipeline lookup + decompile gate.
///
/// On cache hit, `try_execute` returns the matching `CompiledPipeline` and
/// the caller fast-paths the build. On cache miss, `CacheMiss` is
/// returned. After two consecutive failures (or low confidence), the
/// executor emits a `decompile_triggered` (signalled via the existing
/// `AntiSpiralHit` payload OR by returning `Decompile` so the caller
/// re-enters the LLM path).
pub struct CompiledExecutor {
    pub pipeline_cache: Arc<RwLock<BTreeMap<(String, u64), CompiledPipeline>>>,
    consecutive_failures: BTreeMap<(String, u64), u8>,
    confidence_threshold: f32,
    ssb_tx: Option<Arc<Ssb>>,
}

impl CompiledExecutor {
    #[must_use]
    pub fn new(ssb_tx: Option<Arc<Ssb>>) -> Self {
        Self {
            pipeline_cache: Arc::new(RwLock::new(BTreeMap::new())),
            consecutive_failures: BTreeMap::new(),
            confidence_threshold: COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD,
            ssb_tx,
        }
    }

    #[must_use]
    pub fn with_cache(
        ssb_tx: Option<Arc<Ssb>>,
        pipeline_cache: Arc<RwLock<BTreeMap<(String, u64), CompiledPipeline>>>,
    ) -> Self {
        Self {
            pipeline_cache,
            consecutive_failures: BTreeMap::new(),
            confidence_threshold: COMPILED_EXECUTOR_CONFIDENCE_THRESHOLD,
            ssb_tx,
        }
    }

    /// Try to look up a compiled pipeline for the given key. On a HIT,
    /// also emits a `compiled_pipeline_fired` SSB row. Confidence
    /// signal is the `first_try_success_rate` of the stored pipeline.
    pub async fn try_execute(
        &self,
        concept_class: &str,
        plan_skeleton_hash: u64,
    ) -> Result<CompiledPipeline, CompiledExecutorError> {
        let key = (concept_class.to_string(), plan_skeleton_hash);
        let cache = self.pipeline_cache.read().await;
        let Some(pipeline) = cache.get(&key) else {
            return Err(CompiledExecutorError::CacheMiss(
                key.0,
                key.1,
            ));
        };
        if pipeline.first_try_success_rate < self.confidence_threshold {
            // Low confidence — decompile (caller falls through to LLM).
            drop(cache);
            self.emit_decompile_triggered(
                &key,
                DecompileTriggerReason::Confidence,
                0,
            )
            .await;
            return Err(CompiledExecutorError::Decompile(key.0, key.1));
        }
        let failure_count = self
            .consecutive_failures
            .get(&key)
            .copied()
            .unwrap_or(0);
        if failure_count >= COMPILED_EXECUTOR_FAILURE_THRESHOLD {
            drop(cache);
            self.emit_decompile_triggered(
                &key,
                DecompileTriggerReason::ConsecutiveFailures,
                u32::from(failure_count),
            )
            .await;
            return Err(CompiledExecutorError::Decompile(key.0, key.1));
        }
        let mut pipeline_clone = pipeline.clone();
        // Bump use_count + last_used_at on every successful HIT.
        pipeline_clone.use_count = pipeline_clone.use_count.saturating_add(1);
        // Emit compiled_pipeline_fired SSB row. Best-effort.
        if let Some(ssb) = self.ssb_tx.as_ref() {
            let payload = SsbPayload::CompiledPipelineFired(CompiledPipelineFiredPayload {
                pipeline_id: pipeline_clone.pipeline_id.to_string(),
                concept_class: concept_class.to_string(),
                plan_skeleton_hash,
                use_count: pipeline_clone.use_count,
            });
            if let Err(e) = ssb
                .write_payload("compiled_executor", 0, payload, Vec::new())
                .await
            {
                tracing::warn!(
                    error = %e,
                    "compiled_executor: SSB write failed (non-fatal)"
                );
            }
        }
        Ok(pipeline_clone)
    }

    /// Record a per-key failure. Increments the consecutive-failures
    /// counter. On reaching `COMPILED_EXECUTOR_FAILURE_THRESHOLD`, the
    /// next `try_execute` will return `Decompile`.
    pub fn record_failure(&mut self, concept_class: &str, plan_skeleton_hash: u64) {
        let key = (concept_class.to_string(), plan_skeleton_hash);
        let entry = self.consecutive_failures.entry(key).or_insert(0);
        *entry = entry.saturating_add(1);
    }

    /// Reset the consecutive-failures counter on a key (called on a
    /// successful run).
    pub fn record_success(&mut self, concept_class: &str, plan_skeleton_hash: u64) {
        let key = (concept_class.to_string(), plan_skeleton_hash);
        self.consecutive_failures.remove(&key);
    }

    /// Test/diagnostics — read the current consecutive-failure count for a key.
    #[must_use]
    pub fn consecutive_failures_for(&self, concept_class: &str, plan_skeleton_hash: u64) -> u8 {
        self.consecutive_failures
            .get(&(concept_class.to_string(), plan_skeleton_hash))
            .copied()
            .unwrap_or(0)
    }

}

impl std::fmt::Debug for CompiledExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledExecutor")
            .field("consecutive_failures", &self.consecutive_failures.len())
            .field("confidence_threshold", &self.confidence_threshold)
            .field("ssb_tx_wired", &self.ssb_tx.is_some())
            .finish()
    }
}

impl CompiledExecutor {
    /// C-25 audit follow-up — emit a typed `decompile_triggered` SSB row.
    /// Closes WARN-1 in `workflow/sessions/s-25672/audit-C25.md`: the v56
    /// implementation routed this event through the pre-existing
    /// `AntiSpiralHit` payload, so dashboard / contract queries filtered on
    /// `decompile_triggered` saw zero rows. The v57 atomic SCHEMA bump adds
    /// the dedicated [`SsbActionType::DecompileTriggered`] variant +
    /// [`DecompileTriggeredPayload`] (concept_class, plan_skeleton_hash,
    /// failure_count, trigger_reason).
    async fn emit_decompile_triggered(
        &self,
        key: &(String, u64),
        trigger_reason: DecompileTriggerReason,
        failure_count: u32,
    ) {
        if let Some(ssb) = self.ssb_tx.as_ref() {
            let payload = SsbPayload::DecompileTriggered(DecompileTriggeredPayload {
                concept_class: key.0.clone(),
                plan_skeleton_hash: key.1,
                failure_count,
                trigger_reason,
            });
            if let Err(e) = ssb
                .write_payload(
                    "compiled_executor",
                    0,
                    payload,
                    Vec::new(),
                )
                .await
            {
                tracing::warn!(
                    error = %e,
                    "compiled_executor: decompile SSB write failed (non-fatal)"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CompiledPipelineStore — on-disk persistence (msgpack + atomic write)
// ---------------------------------------------------------------------------

/// C-25 Task 8 — typed error from `CompiledPipelineStore` I/O paths.
#[derive(Debug, Error)]
pub enum CompiledPipelineStoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("encode error: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
}

/// C-25 Task 8 — on-disk store backing the compiled-pipeline cache.
///
/// `load_from_disk` is panic-free on corrupt input: it emits a
/// `tracing::warn` and returns an empty store. `persist_to_disk` writes
/// atomically (tempfile + rename) so a crash mid-write cannot corrupt
/// the cache file.
///
/// TTL: entries unused for longer than `ttl_seconds` are evicted at
/// every `upsert` call. The eviction sweep is O(N) over the in-memory
/// BTreeMap; acceptable because N << 1000 in practice.
///
/// `entries` uses `BTreeMap` (not `HashMap`) per B-15 HashMap ban —
/// msgpack on-disk byte order is then deterministic across runs and
/// platforms, which is load-bearing for the cache hit ratio measured
/// by C-25 task 6 contracts.
#[derive(Debug)]
pub struct CompiledPipelineStore {
    cache_path: PathBuf,
    entries: BTreeMap<(String, u64), CompiledPipeline>,
    ttl_seconds: u64,
}

impl CompiledPipelineStore {
    /// Construct a fresh store. Does NOT touch disk; call `load_from_disk`
    /// after construction to populate from the cache file.
    #[must_use]
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            cache_path,
            entries: BTreeMap::new(),
            ttl_seconds: COMPILED_PIPELINE_TTL_SECS,
        }
    }

    /// Construct with a custom TTL (test-only / diagnostics).
    #[must_use]
    pub fn with_ttl(cache_path: PathBuf, ttl_seconds: u64) -> Self {
        Self {
            cache_path,
            entries: BTreeMap::new(),
            ttl_seconds,
        }
    }

    /// Resolve the standard cache path under `project_root/.elai/cache/
    /// compiled_pipelines.msgpack`.
    #[must_use]
    pub fn standard_path(project_root: &Path) -> PathBuf {
        project_root
            .join(COMPILED_PIPELINE_CACHE_DIR)
            .join(COMPILED_PIPELINE_CACHE_FILE)
    }

    /// Read-only accessor for tests / `/pipeline list`.
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<(String, u64), CompiledPipeline> {
        &self.entries
    }

    /// Load entries from disk. On any decode error, emits a warning and
    /// returns an empty store (per stub_ban contract — NEVER panic).
    /// Missing file is treated as empty (no warn).
    pub fn load_from_disk(&mut self) {
        if !self.cache_path.exists() {
            return;
        }
        let bytes = match fs::read(&self.cache_path) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %self.cache_path.display(),
                    "compiled_pipeline_store: read failed; using empty store"
                );
                self.entries.clear();
                return;
            }
        };
        let decoded: Result<BTreeMap<(String, u64), CompiledPipeline>, _> =
            rmp_serde::from_slice(&bytes);
        match decoded {
            Ok(map) => {
                self.entries = map;
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %self.cache_path.display(),
                    "compiled_pipeline_store: corrupt msgpack; using empty store"
                );
                self.entries.clear();
            }
        }
    }

    /// Persist entries to disk atomically (tempfile + rename).
    /// Creates parent directories if absent.
    pub fn persist_to_disk(&self) -> Result<(), CompiledPipelineStoreError> {
        if let Some(parent) = self.cache_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = rmp_serde::to_vec_named(&self.entries)?;
        let tmp_path = self.cache_path.with_extension("msgpack.tmp");
        fs::write(&tmp_path, &bytes)?;
        // Best-effort fsync via File::sync_all on the tempfile is not
        // strictly required (rename is atomic on POSIX/NTFS); skipping
        // to avoid an extra syscall path on Windows.
        fs::rename(&tmp_path, &self.cache_path)?;
        Ok(())
    }

    /// Drop entries whose `last_used_at` is older than `now_secs -
    /// ttl_seconds`. Returns the number of evicted entries.
    pub fn evict_expired(&mut self, now_secs: u64) -> usize {
        let cutoff = now_secs.saturating_sub(self.ttl_seconds);
        let before = self.entries.len();
        self.entries.retain(|_, p| p.last_used_at >= cutoff);
        before - self.entries.len()
    }

    /// Insert or replace an entry. Always touches `last_used_at` to
    /// `now_secs` and evicts expired entries before returning.
    pub fn upsert(&mut self, mut pipeline: CompiledPipeline, now_secs: u64) {
        pipeline.last_used_at = now_secs;
        let key = pipeline.cache_key();
        self.entries.insert(key, pipeline);
        let _evicted = self.evict_expired(now_secs);
    }

    /// Clear the in-memory store AND delete the on-disk file (for
    /// `/pipeline reset`). Returns Ok(()) even if the file did not exist.
    pub fn clear_all(&mut self) -> Result<(), CompiledPipelineStoreError> {
        self.entries.clear();
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests — narrow, in-module, no I/O outside tempdir / clock injection.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn step(idx: u8, prompt: &str, vars: usize, l1: bool) -> CompiledStep {
        let variables: Vec<TemplateVariable> = (0..vars)
            .map(|i| TemplateVariable {
                name: format!("v{i}"),
                region_start: 0,
                region_end: 0,
                variable_type: VariableType::Other("test".into()),
            })
            .collect();
        let ratio = CompiledStep::compute_ratio(prompt, &variables);
        CompiledStep {
            step_index: idx,
            template_prompt: prompt.to_string(),
            variables,
            variable_region_ratio: ratio,
            l1_tool_only: l1,
        }
    }

    fn outcome(success: bool, concept: &str, hash: u64, ts: u64) -> ExecutionOutcome {
        ExecutionOutcome {
            success,
            concept_class: concept.to_string(),
            plan_skeleton_hash: hash,
            pipeline_id: None,
            token_count: 0,
            timestamp: ts,
        }
    }

    #[test]
    fn stable_hash_is_deterministic_and_collision_resistant() {
        let h1 = stable_plan_skeleton_hash("Counter", &["a", "b"]);
        let h2 = stable_plan_skeleton_hash("Counter", &["a", "b"]);
        let h3 = stable_plan_skeleton_hash("Counter", &["a", "c"]);
        let h4 = stable_plan_skeleton_hash("Adder", &["a", "b"]);
        assert_eq!(h1, h2, "same input must yield same hash");
        assert_ne!(h1, h3, "different step list must yield different hash");
        assert_ne!(h1, h4, "different concept_class must yield different hash");
    }

    #[test]
    fn compilation_eligibility_enforces_l1_and_ratio_and_use_count() {
        // Eligible base case.
        let s = step(0, "hello", 0, true);
        assert!(CompiledPipeline::compute_eligibility(&[s.clone()], 3));
        // Not L1 -> not eligible.
        let s_not_l1 = step(0, "hello", 0, false);
        assert!(!CompiledPipeline::compute_eligibility(&[s_not_l1], 3));
        // use_count too low -> not eligible.
        assert!(!CompiledPipeline::compute_eligibility(&[s.clone()], 2));
        // Too many variable bytes -> ratio violation.
        // 4 vars of width 1 inside a 10-char prompt = 40% > 30%.
        let mut s_high_ratio = step(0, "ABCDEFGHIJ", 0, true);
        s_high_ratio.variables = (0..4)
            .map(|i| TemplateVariable {
                name: format!("v{i}"),
                region_start: i,
                region_end: i + 1,
                variable_type: VariableType::Other("x".into()),
            })
            .collect();
        s_high_ratio.variable_region_ratio =
            CompiledStep::compute_ratio(&s_high_ratio.template_prompt, &s_high_ratio.variables);
        assert!(s_high_ratio.variable_region_ratio > VARIABLE_REGION_RATIO_CEILING);
        assert!(!CompiledPipeline::compute_eligibility(&[s_high_ratio], 3));
        // Empty steps -> not eligible.
        assert!(!CompiledPipeline::compute_eligibility(&[], 3));
    }

    #[tokio::test]
    async fn crystallizer_fires_only_after_n_consecutive_successes() {
        let mut c = TemplateCrystallizer::new(None);
        let key_concept = "Counter";
        let key_hash = 0xDEAD_BEEF;
        // Two successes — below threshold.
        c.record_execution(outcome(true, key_concept, key_hash, 1));
        c.record_execution(outcome(true, key_concept, key_hash, 2));
        let r = c
            .check_crystallization(
                key_concept,
                key_hash,
                || vec![step(0, "p", 0, true)],
                100,
            )
            .await;
        assert!(r.is_none(), "must not crystallize before threshold");
        // Third success — threshold reached.
        c.record_execution(outcome(true, key_concept, key_hash, 3));
        let r = c
            .check_crystallization(
                key_concept,
                key_hash,
                || vec![step(0, "p", 0, true)],
                100,
            )
            .await;
        let pipeline = r.expect("must crystallize on N=3 consecutive successes");
        assert_eq!(pipeline.concept_class, key_concept);
        assert_eq!(pipeline.plan_skeleton_hash, key_hash);
        assert!(pipeline.compilation_eligible);
    }

    #[tokio::test]
    async fn crystallizer_does_not_fire_on_mixed_outcomes() {
        let mut c = TemplateCrystallizer::new(None);
        c.record_execution(outcome(true, "K", 1, 1));
        c.record_execution(outcome(false, "K", 1, 2));
        c.record_execution(outcome(true, "K", 1, 3));
        let r = c
            .check_crystallization("K", 1, || vec![step(0, "p", 0, true)], 100)
            .await;
        assert!(r.is_none(), "any failure in tail must block crystallization");
    }

    #[tokio::test]
    async fn compiled_executor_cache_miss_returns_typed_err() {
        let exec = CompiledExecutor::new(None);
        let r = exec.try_execute("Counter", 42).await;
        assert!(matches!(r, Err(CompiledExecutorError::CacheMiss(..))));
    }

    #[tokio::test]
    async fn compiled_executor_hit_returns_pipeline_and_bumps_use_count() {
        let exec = CompiledExecutor::new(None);
        let pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: "Counter".into(),
            plan_skeleton_hash: 7,
            steps: vec![step(0, "p", 0, true)],
            created_at: 0,
            last_used_at: 0,
            use_count: 3,
            first_try_success_rate: 0.9,
            compilation_eligible: true,
        };
        {
            let mut cache = exec.pipeline_cache.write().await;
            cache.insert(("Counter".into(), 7), pipeline.clone());
        }
        let got = exec.try_execute("Counter", 7).await.expect("hit");
        assert_eq!(got.pipeline_id, pipeline.pipeline_id);
        assert_eq!(
            got.use_count, 4,
            "use_count must increment by 1 on hit"
        );
    }

    #[tokio::test]
    async fn compiled_executor_low_confidence_returns_decompile() {
        let exec = CompiledExecutor::new(None);
        let pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: "K".into(),
            plan_skeleton_hash: 1,
            steps: vec![step(0, "p", 0, true)],
            created_at: 0,
            last_used_at: 0,
            use_count: 3,
            first_try_success_rate: 0.5, // < 0.7 threshold
            compilation_eligible: true,
        };
        {
            let mut cache = exec.pipeline_cache.write().await;
            cache.insert(("K".into(), 1), pipeline);
        }
        let r = exec.try_execute("K", 1).await;
        assert!(matches!(r, Err(CompiledExecutorError::Decompile(..))));
    }

    #[tokio::test]
    async fn compiled_executor_two_failures_trigger_decompile() {
        let mut exec = CompiledExecutor::new(None);
        let pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: "K".into(),
            plan_skeleton_hash: 1,
            steps: vec![step(0, "p", 0, true)],
            created_at: 0,
            last_used_at: 0,
            use_count: 3,
            first_try_success_rate: 0.95, // high enough
            compilation_eligible: true,
        };
        {
            let mut cache = exec.pipeline_cache.write().await;
            cache.insert(("K".into(), 1), pipeline);
        }
        exec.record_failure("K", 1);
        exec.record_failure("K", 1);
        let r = exec.try_execute("K", 1).await;
        assert!(matches!(r, Err(CompiledExecutorError::Decompile(..))));
        // Reset on success.
        exec.record_success("K", 1);
        // After reset, hit should work again.
        let r = exec.try_execute("K", 1).await;
        assert!(r.is_ok());
    }

    #[test]
    fn store_load_corrupt_msgpack_returns_empty_no_panic() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("compiled_pipelines.msgpack");
        fs::write(&path, b"not valid msgpack at all").expect("seed corrupt");
        let mut store = CompiledPipelineStore::new(path);
        store.load_from_disk();
        // Must not panic + must return empty.
        assert!(store.entries().is_empty());
    }

    #[test]
    fn store_missing_file_returns_empty_silently() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("does_not_exist.msgpack");
        let mut store = CompiledPipelineStore::new(path);
        store.load_from_disk();
        assert!(store.entries().is_empty());
    }

    #[test]
    fn store_persist_then_reload_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("compiled.msgpack");
        let mut store = CompiledPipelineStore::new(path.clone());
        let pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: "Counter".into(),
            plan_skeleton_hash: 0x42,
            steps: vec![step(0, "p", 0, true)],
            created_at: 100,
            last_used_at: 100,
            use_count: 3,
            first_try_success_rate: 0.95,
            compilation_eligible: true,
        };
        store.upsert(pipeline.clone(), 100);
        store.persist_to_disk().expect("persist");
        // Reload.
        let mut store2 = CompiledPipelineStore::new(path);
        store2.load_from_disk();
        assert_eq!(store2.entries().len(), 1);
        let got = store2
            .entries()
            .get(&("Counter".to_string(), 0x42))
            .expect("entry");
        assert_eq!(got.pipeline_id, pipeline.pipeline_id);
    }

    #[test]
    fn store_evicts_expired_at_upsert() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("compiled.msgpack");
        let mut store = CompiledPipelineStore::with_ttl(path, 10);
        // Old entry (last_used_at = 0) and new entry (now = 100).
        let old_pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: "Old".into(),
            plan_skeleton_hash: 1,
            steps: vec![step(0, "p", 0, true)],
            created_at: 0,
            last_used_at: 0,
            use_count: 3,
            first_try_success_rate: 0.95,
            compilation_eligible: true,
        };
        store.upsert(old_pipeline, 0);
        let new_pipeline = CompiledPipeline {
            pipeline_id: Uuid::new_v4(),
            concept_class: "New".into(),
            plan_skeleton_hash: 2,
            steps: vec![step(0, "p", 0, true)],
            created_at: 100,
            last_used_at: 100,
            use_count: 3,
            first_try_success_rate: 0.95,
            compilation_eligible: true,
        };
        // 100 - 10 = 90 cutoff -> old (last_used_at=0) evicted.
        store.upsert(new_pipeline, 100);
        assert_eq!(store.entries().len(), 1);
        assert!(store.entries().contains_key(&("New".to_string(), 2)));
    }
}
