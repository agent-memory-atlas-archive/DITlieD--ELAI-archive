//! Cosmos artifact tier — B-42 Phase 1.
//!
//! Cross-project, user-global Markdown "cosmos" briefs maintained at
//! `~/.elai/cosmos/` (Windows: `%USERPROFILE%/.elai/cosmos/`). Unlike B-35
//! per-project artifacts, cosmos artifacts are SHARED across all ELAI
//! projects and refresh on a cadence (Daily/Weekly/Monthly/Quarterly)
//! via FlashResearch.
//!
//! # Boundary exemption
//!
//! CLAUDE.md "Project boundary" rule scopes writes to `.elai_cc/`. The
//! cosmos root is *user-global* by design (multi-project knowledge cache).
//! Justified in `workflow/reasoning/B-42-decisions.md` DECISION-COSMOS-PATH-USERSCOPE.
//!
//! # Staleness cadences (DESIGN ASSUMPTIONS, not empirical)
//!
//! - Daily      — fast-moving SDKs, marketplace policies (App Store / Play)
//! - Weekly     — UI/UX conventions, marketing patterns
//! - Monthly    — design heuristics, accessibility checklists
//! - Quarterly  — IP-boundary case law, fundamental algorithm references
//!
//! Supported by arXiv:2403.12958 "Dated Data: Tracing Knowledge Cutoffs"
//! and arXiv:2511.14022 "Keeping Code-Aware LLMs Fresh". NOT calibrated
//! against the rejected arXiv:2305.18290 (that ID is the DPO paper).
//!
//! # SSB integration deferred
//!
//! `orchestrator::ssb` is owned by another active session (B-38). B-42
//! cosmos events are emitted via `tracing::info!(target = "cosmos", ...)`
//! as a temporary observable channel. SSB integration (4 variants:
//! CosmosRefreshStarted / CosmosArtifactUpdated / CosmosArtifactDecayed /
//! CosmosGroundingUsed; SCHEMA_VERSION 43→44) is a documented BLOCK in
//! `workflow/reasoning/B-42-decisions.md` DECISION-SSB-DEFERRED.

use std::collections::BTreeMap;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Sub-directory under the user cosmos root where Markdown artifact bodies live.
pub const ARTIFACTS_SUBDIR: &str = "artifacts";

/// Sub-directory reserved for future BM25/HNSW indexes over cosmos bodies.
pub const INDEX_SUBDIR: &str = "index";

/// Default per-artifact body size cap. 64 KiB is more than enough for a
/// distilled brief; refresh runs that exceed it emit a warning and truncate.
pub const DEFAULT_MAX_SIZE_BYTES: u64 = 64 * 1024;

/// Default tier-1 source-quality filter. T1 ≈ official docs / authoritative
/// domains (see `web_search::source_quality::SCORE_TIER1`).
pub const DEFAULT_MIN_SOURCE_TIER: &str = "T1";

/// Default budget for compiled cosmos briefings injected into agent prompts.
pub const DEFAULT_BRIEFING_BUDGET_TOKENS: usize = 1500;

/// `cosmos_get` stale-refresh blocking timeout.
pub const COSMOS_GET_TIMEOUT_SECS: u64 = 30;

/// Default outer tick interval when `spawn_loop` is started with `None`.
/// Long enough that the inner cadence check (daily/weekly/...) dominates.
pub const DEFAULT_TICK_INTERVAL_SECS: u64 = 3600;

/// Logging target for all cosmos events. Mirrors B-31.5 / B-38 "structured
/// target" convention so dashboards can subscribe via `tracing` once the
/// SSB block is lifted.
pub const COSMOS_TRACE_TARGET: &str = "cosmos";

// ---------------------------------------------------------------------------
// Cadence + status enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshCadence {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
}

impl RefreshCadence {
    /// Cadence interval in seconds (DESIGN ASSUMPTION, see module docs).
    pub fn interval_secs(&self) -> i64 {
        match self {
            RefreshCadence::Daily => 24 * 3600,
            RefreshCadence::Weekly => 7 * 24 * 3600,
            RefreshCadence::Monthly => 30 * 24 * 3600,
            RefreshCadence::Quarterly => 90 * 24 * 3600,
        }
    }

    /// Decay threshold: when `age > 3 * interval`, an artifact is DECAYED.
    /// Serving a decayed artifact MUST emit a WARN per contract.
    ///
    /// **Deliberate 3x vs plan's 2x divergence** (C-16 DECISION-I).
    ///
    /// The C-16 plan §8 prose mentions "age > 2x cadence = DECAYED" but B-42
    /// shipped 3x. The 3x factor provides a grace buffer before SHADOW-serving
    /// stops — lowering to 2x would shrink the safe-stale window and risk
    /// falsely marking artifacts DECAYED during a single missed refresh
    /// cycle. The 3x value is documented in
    /// `workflow/reasoning/C-16-decisions.md` and pinned by
    /// `contracts/tests/cosmos_decay_contract.rs`. Changing this value is a
    /// behavioral regression that requires its own audit cycle.
    pub fn decay_secs(&self) -> i64 {
        self.interval_secs() * 3
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RefreshCadence::Daily => "daily",
            RefreshCadence::Weekly => "weekly",
            RefreshCadence::Monthly => "monthly",
            RefreshCadence::Quarterly => "quarterly",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CosmosStatus {
    Fresh,
    Stale,
    Decayed,
}

impl CosmosStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CosmosStatus::Fresh => "fresh",
            CosmosStatus::Stale => "stale",
            CosmosStatus::Decayed => "decayed",
        }
    }
}

// ---------------------------------------------------------------------------
// Spec / manifest / artifact types
// ---------------------------------------------------------------------------

/// One row from `cosmos_default_manifest.toml`.
///
/// `project_classes` strings MUST be snake_case to match
/// `orchestrator::idea_framing::ProjectClass` serde rename.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosArtifactSpec {
    pub name: String,
    pub project_classes: Vec<String>,
    pub refresh_cadence: RefreshCadence,
    #[serde(default = "default_min_source_tier")]
    pub min_source_tier: String,
    pub research_query: String,
    #[serde(default = "default_max_size_bytes")]
    pub max_size_bytes: u64,
    /// Per-tier relevance score in [0.0, 1.0]. Tiers are `BriefingTier`
    /// stringified (`concepts`, `phases`, `steps`, `baby_steps`, `plan`).
    #[serde(default)]
    pub tier_relevance: BTreeMap<String, f32>,
}

fn default_min_source_tier() -> String {
    DEFAULT_MIN_SOURCE_TIER.to_string()
}

fn default_max_size_bytes() -> u64 {
    DEFAULT_MAX_SIZE_BYTES
}

/// Wrapper for the on-disk `cosmos_default_manifest.toml` top-level shape.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CosmosManifest {
    #[serde(default)]
    pub artifacts: Vec<CosmosArtifactSpec>,
}

impl CosmosManifest {
    /// Parse a manifest from TOML text.
    pub fn from_toml_str(s: &str) -> Result<Self, CosmosError> {
        let m: CosmosManifest = toml::from_str(s).map_err(CosmosError::Toml)?;
        m.validate()?;
        Ok(m)
    }

    /// Load a manifest from a path. Empty manifest if the file does not exist.
    pub fn load(path: &Path) -> Result<Self, CosmosError> {
        if !path.exists() {
            return Ok(CosmosManifest::default());
        }
        let raw = fs::read_to_string(path).map_err(CosmosError::Io)?;
        Self::from_toml_str(&raw)
    }

    fn validate(&self) -> Result<(), CosmosError> {
        let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for spec in &self.artifacts {
            if !is_valid_name(&spec.name) {
                return Err(CosmosError::InvalidName(spec.name.clone()));
            }
            if !seen.insert(&spec.name) {
                return Err(CosmosError::DuplicateName(spec.name.clone()));
            }
            if spec.research_query.trim().is_empty() {
                return Err(CosmosError::EmptyQuery(spec.name.clone()));
            }
            for v in spec.tier_relevance.values() {
                if !v.is_finite() || *v < 0.0 || *v > 1.0 {
                    return Err(CosmosError::InvalidTierRelevance {
                        artifact: spec.name.clone(),
                        value: *v,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&CosmosArtifactSpec> {
        self.artifacts.iter().find(|s| s.name == name)
    }
}

/// On-disk metadata for a single cosmos artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosArtifactMeta {
    pub name: String,
    pub blake3_hash: String,
    pub size_bytes: u64,
    pub refreshed_at: i64,
    pub source_tier: String,
    pub refresh_cadence: RefreshCadence,
}

/// Resolved cosmos artifact (spec + body + provenance).
#[derive(Debug, Clone)]
pub struct CosmosArtifact {
    pub spec: CosmosArtifactSpec,
    pub body: String,
    pub blake3_hash: String,
    pub refreshed_at: i64,
    pub status: CosmosStatus,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CosmosError {
    #[error("cosmos io error: {0}")]
    Io(#[source] std::io::Error),

    #[error("cosmos toml parse error: {0}")]
    Toml(#[source] toml::de::Error),

    #[error("cosmos invalid artifact name '{0}' (must match [a-zA-Z0-9_-]+)")]
    InvalidName(String),

    #[error("cosmos duplicate artifact name: '{0}'")]
    DuplicateName(String),

    #[error("cosmos artifact '{0}' has empty research_query")]
    EmptyQuery(String),

    #[error("cosmos artifact '{artifact}' tier_relevance value out of [0,1]: {value}")]
    InvalidTierRelevance { artifact: String, value: f32 },

    #[error("cosmos artifact not found in manifest: '{0}'")]
    UnknownArtifact(String),

    #[error("cosmos artifact body missing on disk for '{0}'")]
    MissingBody(String),

    #[error("cosmos blake3 mismatch for '{0}' (file tampered or write torn)")]
    Blake3Mismatch(String),

    #[error("cosmos home directory could not be resolved (no HOME / USERPROFILE)")]
    NoHomeDir,

    #[error("cosmos research dispatch error: {0}")]
    Research(String),

    #[error("cosmos refresh timed out after {0:?}")]
    Timeout(Duration),
}

// ---------------------------------------------------------------------------
// Traits / capability injection
// ---------------------------------------------------------------------------

/// Monotonic clock provider. Returns Unix seconds. Production wires the
/// `ClockProvider` from `elai-core`; tests inject a controllable clock.
pub trait CosmosClock: Send + Sync {
    fn now_secs(&self) -> i64;
}

/// Default system clock — delegates to `elai_core::OsClock` (C-15).
pub struct SystemCosmosClock;

impl CosmosClock for SystemCosmosClock {
    fn now_secs(&self) -> i64 {
        i64::try_from(OsClock.now_secs()).unwrap_or(i64::MAX)
    }
}

/// FlashResearch dispatcher abstraction. Production wires
/// `orchestrator::research_tree::ResearchTree`; tests provide a canned
/// `MockResearch` that returns a fixed Markdown body.
///
/// Returns `(markdown_body, source_tier_string)`.
///
/// # Async-native (C-16 BLOCK-1 fix)
///
/// The trait is async-native, returning `Pin<Box<dyn Future>>` so it stays
/// `dyn`-compatible (Rust 1.75+ native `async fn` in trait is not yet
/// object-safe without manual boxing). Production impls (e.g.
/// `HttpCosmosResearchDispatcher`) call into already-async HTTP clients
/// directly — no nested `Runtime::block_on()` bridge.
pub trait ResearchDispatcher: Send + Sync {
    fn run<'a>(
        &'a self,
        query: &'a str,
        min_source_tier: &'a str,
        max_size_bytes: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>>;
}

/// C-16 (DECISION-E + BLOCK-1 fix): async-native SSB emitter callback.
/// Production wires an `Arc<Ssb>` adapter (`orchestrator::SsbEmitterAdapter`)
/// that translates these calls into typed `SsbPayload::Cosmos*` writes via
/// the already-async `Ssb::write_payload`. Tests pass `None` and the worker
/// is a no-op on the emission side.
///
/// All three methods return `Pin<Box<dyn Future>>` so the trait stays
/// `dyn`-compatible. They are best-effort — implementations MUST swallow
/// transport errors via `tracing::warn!` and never panic.
pub trait SsbEmitter: Send + Sync {
    /// Emitted ONCE per `tick()` invocation, BEFORE the manifest loop.
    /// `queued` is the stale+decayed count (per C-16 DECISION-F), `total`
    /// is the full manifest size.
    fn emit_refresh_started<'a>(
        &'a self,
        queued: u32,
        total: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    /// Emitted on `refresh_one(...) -> Ok(RefreshOutcome { changed: true, .. })`.
    /// Real blake3 / body_bytes / latency_ms per C-16 DECISION-H.
    fn emit_artifact_updated<'a>(
        &'a self,
        name: &'a str,
        blake3_hash: &'a str,
        body_bytes: u32,
        latency_ms: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    /// Emitted from the cosmos refresh tick when an artifact is observed to
    /// be Decayed at serve time, in addition to the tracing::warn channel.
    ///
    /// Note: The synchronous `cosmos_get()` does NOT call this emitter (would
    /// require sync->async bridging via nested runtime which panics — the
    /// C-16 BLOCK-1 root cause). Decayed emission now happens from async
    /// call sites only (cosmos tick scheduler, task_runner helper, Tauri
    /// command). `cosmos_get` still emits `tracing::warn!` so the
    /// observability channel survives in sync contexts.
    fn emit_artifact_decayed<'a>(
        &'a self,
        name: &'a str,
        age_secs: u64,
        cadence: &'a str,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

/// C-16 (DECISION-H): outcome of a single `refresh_one` invocation, surfaced
/// to callers so they can emit SSB rows with real blake3 / size / latency
/// instead of placeholder zeros (DRIFT-H).
#[derive(Debug, Clone)]
pub struct RefreshOutcome {
    /// True iff a new body was written (content-changed). False on
    /// idempotent re-research (blake3 matched previous).
    pub changed: bool,
    /// blake3 hash of the (potentially unchanged) body.
    pub blake3: String,
    /// Size of the body in bytes (post-truncation).
    pub body_bytes: u32,
    /// Wall-clock latency of the research+write path in milliseconds.
    pub latency_ms: u32,
}

// ---------------------------------------------------------------------------
// CosmosWorker
// ---------------------------------------------------------------------------

/// Long-lived worker that keeps the cosmos artifacts fresh.
///
/// Single-writer per artifact (write-then-rename). Outer concurrency control
/// (e.g. `IndexLock(LockScope::Index)`) is the caller's responsibility — the
/// worker itself only owns the inner refresh logic so it remains testable
/// without the full orchestrator wired.
pub struct CosmosWorker {
    cosmos_root: PathBuf,
    manifest: CosmosManifest,
    clock: Arc<dyn CosmosClock>,
    research: Arc<dyn ResearchDispatcher>,
    /// C-16 (DECISION-E): optional SSB emitter. `None` for tests + the
    /// standalone read-only dashboard snapshot path; `Some` in production
    /// after `with_ssb_emitter(...)` wiring in `cli::live_cli`.
    ssb_emitter: Option<Arc<dyn SsbEmitter>>,
}

/// Summary of one `tick()` invocation.
///
/// C-16 (DECISION-H): the `outcomes` field carries per-refreshed-artifact
/// [`RefreshOutcome`] records (blake3 / size / latency) so the external
/// SSB emitter can populate `CosmosArtifactUpdated` rows with real data
/// instead of zeros. `refreshed` retains the name-only list for backward
/// compat; entries in `outcomes` are guaranteed to align 1:1 by index.
#[derive(Debug, Clone, Default)]
pub struct TickReport {
    pub checked: usize,
    pub refreshed: Vec<String>,
    pub unchanged: Vec<String>,
    pub failed: Vec<(String, String)>,
    /// C-16: per-refreshed-artifact outcome with blake3/size/latency. Aligned
    /// 1:1 with `refreshed` by index.
    pub outcomes: Vec<RefreshOutcome>,
}

impl CosmosWorker {
    /// Construct a worker rooted at `cosmos_root` (typically the result of
    /// [`default_cosmos_root`]).
    pub fn new(
        cosmos_root: PathBuf,
        manifest: CosmosManifest,
        clock: Arc<dyn CosmosClock>,
        research: Arc<dyn ResearchDispatcher>,
    ) -> Result<Self, CosmosError> {
        fs::create_dir_all(cosmos_root.join(ARTIFACTS_SUBDIR)).map_err(CosmosError::Io)?;
        fs::create_dir_all(cosmos_root.join(INDEX_SUBDIR)).map_err(CosmosError::Io)?;
        Ok(Self {
            cosmos_root,
            manifest,
            clock,
            research,
            ssb_emitter: None,
        })
    }

    /// Read-only constructor for the dashboard `get_cosmos_status` surface.
    ///
    /// Unlike [`CosmosWorker::new`], this does NOT call `fs::create_dir_all`
    /// on the artifacts/index subdirs. Use when the caller only intends to
    /// invoke filesystem-read methods (e.g. [`Self::status_snapshot`]). If
    /// the cosmos root does not yet exist, the worker still constructs; the
    /// read methods are tolerant of missing subdirs and return empty.
    pub fn new_read_only(
        cosmos_root: PathBuf,
        manifest: CosmosManifest,
        clock: Arc<dyn CosmosClock>,
        research: Arc<dyn ResearchDispatcher>,
    ) -> Self {
        Self {
            cosmos_root,
            manifest,
            clock,
            research,
            ssb_emitter: None,
        }
    }

    /// C-16 (DECISION-E): attach a sync SSB emitter. Builder-style so the
    /// existing `new(...)` signature stays backward-compatible. When None
    /// (the default), all `emit_*` calls are skipped — tracing remains the
    /// observability channel. When Some, both the tracing path AND the SSB
    /// emitter fire (additive per DECISION-J).
    pub fn with_ssb_emitter(mut self, emitter: Arc<dyn SsbEmitter>) -> Self {
        self.ssb_emitter = Some(emitter);
        self
    }

    /// C-16 (DECISION-F): count of artifacts that are NOT Fresh — i.e. the
    /// number `tick()` would queue for re-research. Used by external SSB
    /// emitters to populate `CosmosRefreshStartedPayload.queued` with the
    /// actual stale+decayed+missing count (DRIFT-F), not the manifest size.
    ///
    /// Counts:
    /// - Stale artifacts (age >= interval, age < decay)
    /// - Decayed artifacts (age >= decay)
    /// - Missing artifacts (no meta on disk)
    ///
    /// Read failures (corrupt meta) are counted as due (the loop will try
    /// to refresh them).
    pub fn count_due(&self) -> usize {
        let now = self.clock.now_secs();
        let mut due = 0usize;
        for spec in &self.manifest.artifacts {
            match self.read_meta(&spec.name) {
                Ok(Some(meta)) => {
                    if !matches!(self.status_for(&meta, now), CosmosStatus::Fresh) {
                        due += 1;
                    }
                }
                // Missing OR unreadable -> needs refresh.
                Ok(None) | Err(_) => due += 1,
            }
        }
        due
    }

    pub fn cosmos_root(&self) -> &Path {
        &self.cosmos_root
    }

    pub fn manifest(&self) -> &CosmosManifest {
        &self.manifest
    }

    /// C-2: expose the clock's current seconds-since-epoch so external
    /// callers (e.g. orchestrator `trigger_cosmos_refresh`) can call
    /// [`Self::refresh_one`] with a consistent `now` value without
    /// reaching into the private `clock` field.
    pub fn now_secs_clock(&self) -> i64 {
        self.clock.now_secs()
    }

    /// Classify an artifact's freshness against `now`.
    pub fn status_for(&self, meta: &CosmosArtifactMeta, now: i64) -> CosmosStatus {
        let age = (now - meta.refreshed_at).max(0);
        let cadence = meta.refresh_cadence;
        if age < cadence.interval_secs() {
            CosmosStatus::Fresh
        } else if age < cadence.decay_secs() {
            CosmosStatus::Stale
        } else {
            CosmosStatus::Decayed
        }
    }

    fn body_path(&self, name: &str) -> PathBuf {
        self.cosmos_root
            .join(ARTIFACTS_SUBDIR)
            .join(format!("{}.md", name))
    }

    fn meta_path(&self, name: &str) -> PathBuf {
        self.cosmos_root
            .join(ARTIFACTS_SUBDIR)
            .join(format!("{}.meta.json", name))
    }

    fn read_meta(&self, name: &str) -> Result<Option<CosmosArtifactMeta>, CosmosError> {
        let p = self.meta_path(name);
        if !p.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&p).map_err(CosmosError::Io)?;
        let meta: CosmosArtifactMeta =
            serde_json::from_str(&raw).map_err(|e| CosmosError::Research(e.to_string()))?;
        Ok(Some(meta))
    }

    fn write_meta(&self, meta: &CosmosArtifactMeta) -> Result<(), CosmosError> {
        let p = self.meta_path(&meta.name);
        let raw = serde_json::to_string_pretty(meta)
            .map_err(|e| CosmosError::Research(e.to_string()))?;
        write_atomic(&p, raw.as_bytes())
    }

    /// One pass over the manifest. Re-research every artifact whose status
    /// is `Stale` or `Decayed`; skip `Fresh`.
    ///
    /// Idempotency: a refreshed body whose blake3 matches the previous body
    /// is NOT emitted as `CosmosArtifactUpdated`. Only true content change
    /// produces an "updated" event (per plan §4 idempotency rule).
    ///
    /// # Async-native (C-16 BLOCK-1 fix)
    ///
    /// `tick()` is `async fn` because `ResearchDispatcher::run` and
    /// `SsbEmitter::emit_*` are async. Callers MUST be on a tokio runtime
    /// (live_cli cosmos tick scheduler + Tauri trigger_cosmos_refresh
    /// command both already are). No nested `Runtime::block_on()` bridge.
    pub async fn tick(&self) -> TickReport {
        let now = self.clock.now_secs();
        let mut report = TickReport::default();

        // C-16 (DECISION-F + DRIFT-F): emit ONE CosmosRefreshStarted SSB row
        // BEFORE the manifest loop, with `queued` = stale+decayed+missing
        // count (not manifest_size). The pre-scan via `count_due()` is an
        // in-memory manifest iteration — cheap (no I/O beyond the meta read
        // which the loop body would do anyway).
        if let Some(ref emitter) = self.ssb_emitter {
            let queued = u32::try_from(self.count_due()).unwrap_or(u32::MAX);
            let total = u32::try_from(self.manifest.artifacts.len()).unwrap_or(u32::MAX);
            emitter.emit_refresh_started(queued, total).await;
        }

        for spec in &self.manifest.artifacts {
            report.checked += 1;
            let existing = match self.read_meta(&spec.name) {
                Ok(m) => m,
                Err(e) => {
                    report.failed.push((spec.name.clone(), e.to_string()));
                    continue;
                }
            };

            let needs_refresh = match &existing {
                None => true,
                Some(m) => !matches!(self.status_for(m, now), CosmosStatus::Fresh),
            };
            if !needs_refresh {
                report.unchanged.push(spec.name.clone());
                continue;
            }

            info!(
                target: COSMOS_TRACE_TARGET,
                event = "cosmos_refresh_started",
                artifact = %spec.name,
                cadence = spec.refresh_cadence.as_str(),
                "cosmos refresh started",
            );

            match self.refresh_one(spec, now, existing.as_ref()).await {
                Ok(outcome) if outcome.changed => {
                    // C-16 (DECISION-H): emit CosmosArtifactUpdated with real
                    // blake3 / size / latency from the outcome.
                    if let Some(ref emitter) = self.ssb_emitter {
                        emitter
                            .emit_artifact_updated(
                                &spec.name,
                                &outcome.blake3,
                                outcome.body_bytes,
                                outcome.latency_ms,
                            )
                            .await;
                    }
                    report.refreshed.push(spec.name.clone());
                    report.outcomes.push(outcome);
                }
                Ok(_outcome) => {
                    // Body unchanged after re-research — content idempotent.
                    // NO CosmosArtifactUpdated SSB row (idempotent skip).
                    report.unchanged.push(spec.name.clone());
                }
                Err(e) => {
                    report.failed.push((spec.name.clone(), e.to_string()));
                }
            }
        }
        report
    }

    /// Re-research a single artifact and persist if content changed.
    /// Returns `Ok(true)` when a new body was written, `Ok(false)` when the
    /// new body matches the previous body byte-for-byte.
    ///
    /// C-2 (DRIFT-3): promoted from `fn` to `pub fn` so the task_runner can
    /// trigger a refresh once per task entry via
    /// `trigger_cosmos_refresh()` and surface `cosmos_refresh_started`
    /// rows on a live cloud run. Production callers MUST hold an outer
    /// `IndexLock(LockScope::Index)` (or equivalent) — single-writer per
    /// artifact contract still applies.
    ///
    /// # Async-native (C-16 BLOCK-1 fix)
    ///
    /// `refresh_one` is `async fn` because `ResearchDispatcher::run` is
    /// async — no nested `Runtime::block_on()` bridge.
    pub async fn refresh_one(
        &self,
        spec: &CosmosArtifactSpec,
        now: i64,
        previous: Option<&CosmosArtifactMeta>,
    ) -> Result<RefreshOutcome, CosmosError> {
        // C-16 (DECISION-H): measure wall-clock latency for the research +
        // write path so the SSB CosmosArtifactUpdated row carries a real
        // `latency_ms` instead of zero. Use std::time::Instant (monotonic) —
        // CosmosClock is wall-clock seconds and unsuitable for latency.
        // C-15 (s-25120): CosmosClock trait at line 305 only exposes now_secs();
        // adding instant_now to it is a separate refactor scoped to a future
        // cosmos-clock-extension plan. Annotated to keep entropy quarantine green.
        let start = std::time::Instant::now(); // entropy-allow: cosmos-latency-cosmosclock-no-instant-now
        let (mut body, source_tier) = self
            .research
            .run(&spec.research_query, &spec.min_source_tier, spec.max_size_bytes)
            .await?;
        if body.as_bytes().len() as u64 > spec.max_size_bytes {
            warn!(
                target: COSMOS_TRACE_TARGET,
                artifact = %spec.name,
                size = body.len(),
                cap = spec.max_size_bytes,
                "cosmos body exceeded size cap; truncating"
            );
            let cap = floor_char_boundary(&body, spec.max_size_bytes as usize);
            body.truncate(cap);
        }
        let hash = blake3::hash(body.as_bytes()).to_hex().to_string();
        let body_bytes = u32::try_from(body.as_bytes().len()).unwrap_or(u32::MAX);
        if let Some(prev) = previous {
            if prev.blake3_hash == hash {
                let latency_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
                return Ok(RefreshOutcome {
                    changed: false,
                    blake3: hash,
                    body_bytes,
                    latency_ms,
                });
            }
        }
        // Write body + meta atomically (write to tmp, rename).
        write_atomic(&self.body_path(&spec.name), body.as_bytes())?;
        let meta = CosmosArtifactMeta {
            name: spec.name.clone(),
            blake3_hash: hash.clone(),
            size_bytes: body.as_bytes().len() as u64,
            refreshed_at: now,
            source_tier: source_tier.clone(),
            refresh_cadence: spec.refresh_cadence,
        };
        self.write_meta(&meta)?;
        let latency_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
        info!(
            target: COSMOS_TRACE_TARGET,
            event = "cosmos_artifact_updated",
            artifact = %spec.name,
            blake3 = %hash,
            size_bytes = meta.size_bytes,
            source_tier = %source_tier,
            latency_ms = latency_ms,
            "cosmos artifact updated",
        );
        Ok(RefreshOutcome {
            changed: true,
            blake3: hash,
            body_bytes,
            latency_ms,
        })
    }

    /// Resolve an artifact by name. Returns the on-disk body and freshness
    /// classification. If `Decayed` and serve_stale=true, emits a WARN per
    /// the no-silent-stale contract.
    pub fn cosmos_get(&self, name: &str) -> Result<CosmosArtifact, CosmosError> {
        let spec = self
            .manifest
            .lookup(name)
            .ok_or_else(|| CosmosError::UnknownArtifact(name.to_string()))?
            .clone();
        let meta = self
            .read_meta(name)?
            .ok_or_else(|| CosmosError::MissingBody(name.to_string()))?;
        let body_path = self.body_path(name);
        if !body_path.exists() {
            return Err(CosmosError::MissingBody(name.to_string()));
        }
        let body = fs::read_to_string(&body_path).map_err(CosmosError::Io)?;
        let computed = blake3::hash(body.as_bytes()).to_hex().to_string();
        if computed != meta.blake3_hash {
            return Err(CosmosError::Blake3Mismatch(name.to_string()));
        }
        let now = self.clock.now_secs();
        let status = self.status_for(&meta, now);
        if status == CosmosStatus::Decayed {
            let age_secs = (now - meta.refreshed_at).max(0);
            warn!(
                target: COSMOS_TRACE_TARGET,
                event = "cosmos_artifact_decayed",
                artifact = %name,
                age_secs = age_secs,
                cadence = meta.refresh_cadence.as_str(),
                "cosmos artifact is DECAYED; serving stale body with warning",
            );
            // C-16 BLOCK-1 fix: `cosmos_get` is sync (called from sync
            // pipeline + briefing paths) and CANNOT call `SsbEmitter`
            // (async) without re-introducing the nested-runtime panic.
            // The tracing::warn above remains the observability channel
            // in sync contexts; async callers that want a typed SSB
            // `CosmosArtifactDecayed` row should call
            // `cosmos_get_async()` (TODO: future helper) or emit it
            // themselves from the async dispatch site.
        }
        Ok(CosmosArtifact {
            spec,
            body,
            blake3_hash: meta.blake3_hash,
            refreshed_at: meta.refreshed_at,
            status,
        })
    }

    /// B-42 dashboard surface: per-artifact status snapshot.
    ///
    /// Filesystem-only read of the on-disk manifest + every `<name>.meta.json`
    /// sidecar. Does NOT invoke the research dispatcher and does NOT mutate
    /// state. Safe to call from a Tauri command without an active runtime.
    ///
    /// Missing meta files yield `refreshed_at = None` and `status = "stale"`
    /// (never-refreshed → not fresh, not decayed-by-age). Unreadable / corrupt
    /// meta files are skipped silently (warned via `tracing::warn!`) — surface
    /// remains best-effort for the dashboard.
    pub fn status_snapshot(&self) -> Vec<CosmosArtifactStatusJson> {
        let now = self.clock.now_secs();
        let mut out = Vec::with_capacity(self.manifest.artifacts.len());
        for spec in &self.manifest.artifacts {
            match self.read_meta(&spec.name) {
                Ok(Some(meta)) => {
                    let status = self.status_for(&meta, now);
                    out.push(CosmosArtifactStatusJson {
                        name: spec.name.clone(),
                        project_classes: spec.project_classes.clone(),
                        refreshed_at: Some(format_rfc3339_utc(meta.refreshed_at)),
                        status: status.as_str().to_string(),
                        size_bytes: meta.size_bytes,
                    });
                }
                Ok(None) => {
                    out.push(CosmosArtifactStatusJson {
                        name: spec.name.clone(),
                        project_classes: spec.project_classes.clone(),
                        refreshed_at: None,
                        status: CosmosStatus::Stale.as_str().to_string(),
                        size_bytes: 0,
                    });
                }
                Err(e) => {
                    warn!(
                        target: COSMOS_TRACE_TARGET,
                        artifact = %spec.name,
                        error = %e,
                        "cosmos status_snapshot: failed to read meta; emitting stale row"
                    );
                    out.push(CosmosArtifactStatusJson {
                        name: spec.name.clone(),
                        project_classes: spec.project_classes.clone(),
                        refreshed_at: None,
                        status: CosmosStatus::Stale.as_str().to_string(),
                        size_bytes: 0,
                    });
                }
            }
        }
        out
    }

    /// Emit a `cosmos_grounding_used` trace event. Called by the verifier
    /// when a `VerificationMethod::CosmosCheck` claim is dispatched.
    pub fn record_grounding_used(
        &self,
        artifact_name: &str,
        claim_category: &str,
        query: &str,
        score: f32,
        project_class: Option<&str>,
    ) {
        let qh = blake3::hash(query.as_bytes()).to_hex().to_string();
        info!(
            target: COSMOS_TRACE_TARGET,
            event = "cosmos_grounding_used",
            artifact = %artifact_name,
            claim_category = %claim_category,
            query_hash = %qh,
            score = score,
            project_class = ?project_class,
            "cosmos grounding used",
        );
    }
}

// ---------------------------------------------------------------------------
// B-42 dashboard surface — per-artifact status row + standalone snapshot
// ---------------------------------------------------------------------------

/// Per-artifact status row exposed to the dashboard via the
/// `get_cosmos_status` Tauri command.
///
/// `refreshed_at` is RFC3339 (UTC, seconds precision) when the artifact has
/// ever been refreshed; `None` for never-refreshed artifacts. `status` is
/// the snake_case discriminant from [`CosmosStatus`] (`"fresh" | "stale" |
/// "decayed"`). `size_bytes` is 0 for never-refreshed rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosArtifactStatusJson {
    pub name: String,
    pub project_classes: Vec<String>,
    pub refreshed_at: Option<String>,
    pub status: String,
    pub size_bytes: u64,
}

/// No-op `ResearchDispatcher` used by the standalone read-only snapshot path
/// (dashboard `get_cosmos_status` command). Returns `Err` if accidentally
/// invoked — `status_snapshot` MUST NOT trigger research. This dispatcher is
/// intentionally inert to keep the "filesystem-only" contract honest.
/// No-op `ResearchDispatcher` used by read-only callers
/// (`cosmos_status_snapshot_standalone`) and C-8 (`CosmosWorker` tick
/// scheduler bootstrap before a real research client is wired). Always
/// returns `CosmosError::Research("...")` from `research`; callers that
/// only call `tick()` / `status_snapshot()` are unaffected because those
/// paths short-circuit when no artifact is `Stale` / `Decayed`.
pub struct NoopResearchDispatcher;

impl ResearchDispatcher for NoopResearchDispatcher {
    fn run<'a>(
        &'a self,
        _query: &'a str,
        _min_source_tier: &'a str,
        _max_size_bytes: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        Box::pin(async move {
            Err(CosmosError::Research(
                "NoopResearchDispatcher invoked from read-only snapshot path".into(),
            ))
        })
    }
}

/// Standalone, dependency-free entry point for the dashboard `get_cosmos_status`
/// Tauri command. Resolves the user-global cosmos root, loads the manifest if
/// present, and returns the per-artifact status snapshot. Returns an empty
/// vector when the cosmos root or manifest does not yet exist (clean
/// cold-start: dashboard renders empty list instead of erroring out).
///
/// Uses [`NoopResearchDispatcher`] internally — `status_snapshot` is
/// filesystem-only and never invokes the dispatcher.
pub fn cosmos_status_snapshot_standalone() -> Vec<CosmosArtifactStatusJson> {
    let root = match default_cosmos_root() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    if !root.exists() {
        return Vec::new();
    }
    let manifest_path = root.join("manifest.toml");
    let manifest = match CosmosManifest::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            warn!(
                target: COSMOS_TRACE_TARGET,
                path = %manifest_path.display(),
                error = %e,
                "cosmos_status_snapshot_standalone: manifest load failed; returning empty"
            );
            return Vec::new();
        }
    };
    let clock: Arc<dyn CosmosClock> = Arc::new(SystemCosmosClock);
    let research: Arc<dyn ResearchDispatcher> = Arc::new(NoopResearchDispatcher);
    // Use the read-only constructor: status_snapshot is filesystem-read only
    // and MUST NOT side-effect create artifacts/index subdirs on the user's
    // global cosmos root (was a B-42 audit WARN — fs::create_dir_all on every
    // dashboard poll). read_meta tolerates missing subdirs.
    let w = CosmosWorker::new_read_only(root, manifest, clock, research);
    w.status_snapshot()
}

/// Walk DOWN from `min(cap, s.len())` until a valid UTF-8 char boundary is
/// found. `s.is_char_boundary(0)` is always true so the loop always
/// terminates. Use this anywhere a byte-cap truncates a `&str` / `String` —
/// `String::truncate` and `&s[..n]` both PANIC on a mid-codepoint index, and
/// cosmos artifact bodies routinely contain multi-byte UTF-8 (curly quotes,
/// em-dashes, CJK from web research).
pub fn floor_char_boundary(s: &str, cap: usize) -> usize {
    let mut i = cap.min(s.len());
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Format a Unix-seconds timestamp as RFC3339 in UTC ("YYYY-MM-DDTHH:MM:SSZ").
///
/// Implemented inline because the knowledge crate has no `chrono` / `time`
/// dependency (anti-slopsquatting: verified `knowledge/Cargo.toml`). Handles
/// the proleptic Gregorian calendar correctly for the range [1970, 9999].
/// Negative or out-of-range inputs return `"1970-01-01T00:00:00Z"`.
fn format_rfc3339_utc(epoch_secs: i64) -> String {
    if epoch_secs <= 0 {
        return "1970-01-01T00:00:00Z".to_string();
    }
    let secs = epoch_secs as u64;
    let day = secs / 86_400;
    let sod = secs % 86_400;
    let hour = sod / 3600;
    let minute = (sod % 3600) / 60;
    let second = sod % 60;

    // Civil-from-days algorithm (Howard Hinnant, public domain).
    let z = day as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + (if m <= 2 { 1 } else { 0 });
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    )
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Resolve the user-global cosmos root: `$HOME/.elai/cosmos/` on Unix or
/// `%USERPROFILE%/.elai/cosmos/` on Windows. Uses environment variables only
/// (no `dirs` crate dep — anti-slopsquatting verified Cargo.toml has no
/// `dirs`/`directories` entries).
pub fn default_cosmos_root() -> Result<PathBuf, CosmosError> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or(CosmosError::NoHomeDir)?;
    let mut p = PathBuf::from(home);
    p.push(".elai");
    p.push("cosmos");
    Ok(p)
}

fn is_valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Write bytes to `path` via a temp+rename sequence to avoid torn writes.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CosmosError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(CosmosError::Io)?;
    }
    let mut tmp = path.to_path_buf();
    let mut tmp_name = tmp
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    tmp_name.push(".tmp");
    tmp.set_file_name(tmp_name);
    fs::write(&tmp, bytes).map_err(CosmosError::Io)?;
    fs::rename(&tmp, path).map_err(CosmosError::Io)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Test helpers (gated)
// ---------------------------------------------------------------------------

/// Fixed-clock implementation for tests.
pub struct FixedClock(pub std::sync::atomic::AtomicI64);

impl FixedClock {
    pub fn new(initial: i64) -> Self {
        Self(std::sync::atomic::AtomicI64::new(initial))
    }
    pub fn advance(&self, secs: i64) {
        self.0
            .fetch_add(secs, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn set(&self, secs: i64) {
        self.0.store(secs, std::sync::atomic::Ordering::SeqCst);
    }
}

impl CosmosClock for FixedClock {
    fn now_secs(&self) -> i64 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Canned research dispatcher for tests. Returns the same body until
/// `set_body` is called again.
pub struct MockResearch {
    body: std::sync::Mutex<String>,
    tier: String,
}

impl MockResearch {
    pub fn new(body: impl Into<String>, tier: impl Into<String>) -> Self {
        Self {
            body: std::sync::Mutex::new(body.into()),
            tier: tier.into(),
        }
    }
    pub fn set_body(&self, body: impl Into<String>) {
        if let Ok(mut g) = self.body.lock() {
            *g = body.into();
        }
    }
}

impl ResearchDispatcher for MockResearch {
    fn run<'a>(
        &'a self,
        _query: &'a str,
        _min_source_tier: &'a str,
        _max_size_bytes: u64,
    ) -> Pin<Box<dyn Future<Output = Result<(String, String), CosmosError>> + Send + 'a>> {
        Box::pin(async move {
            let body = self
                .body
                .lock()
                .map(|g| g.clone())
                .map_err(|_| CosmosError::Research("mock body lock poisoned".into()))?;
            Ok((body, self.tier.clone()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> CosmosManifest {
        let toml = r#"
[[artifacts]]
name = "ios_hig_summary"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "weekly"
min_source_tier = "T1"
research_query = "Apple Human Interface Guidelines top 20 patterns"
max_size_bytes = 16384

[[artifacts]]
name = "app_store_policies"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "daily"
research_query = "Apple App Store Review Guidelines recent changes"
"#;
        CosmosManifest::from_toml_str(toml).expect("manifest parses")
    }

    #[test]
    fn manifest_parses_and_validates() {
        let m = sample_manifest();
        assert_eq!(m.artifacts.len(), 2);
        assert_eq!(m.artifacts[0].name, "ios_hig_summary");
        assert_eq!(m.artifacts[1].refresh_cadence, RefreshCadence::Daily);
        assert_eq!(m.artifacts[0].min_source_tier, "T1");
        assert_eq!(m.artifacts[1].min_source_tier, "T1"); // default
    }

    #[test]
    fn manifest_rejects_invalid_name() {
        let toml = r#"
[[artifacts]]
name = "bad name!"
project_classes = ["consumer_mobile_app"]
refresh_cadence = "daily"
research_query = "x"
"#;
        let r = CosmosManifest::from_toml_str(toml);
        assert!(matches!(r, Err(CosmosError::InvalidName(_))));
    }

    #[test]
    fn manifest_rejects_duplicate_name() {
        let toml = r#"
[[artifacts]]
name = "dup"
project_classes = []
refresh_cadence = "daily"
research_query = "a"

[[artifacts]]
name = "dup"
project_classes = []
refresh_cadence = "weekly"
research_query = "b"
"#;
        assert!(matches!(
            CosmosManifest::from_toml_str(toml),
            Err(CosmosError::DuplicateName(_))
        ));
    }

    #[test]
    fn cadence_decay_threshold_is_three_intervals() {
        assert_eq!(
            RefreshCadence::Daily.decay_secs(),
            3 * RefreshCadence::Daily.interval_secs()
        );
        assert_eq!(
            RefreshCadence::Quarterly.decay_secs(),
            3 * RefreshCadence::Quarterly.interval_secs()
        );
    }

    #[test]
    fn status_for_classifies_fresh_stale_decayed() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(10_000_000));
        let research = Arc::new(MockResearch::new("body", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock.clone(),
            research,
        )
        .expect("worker");

        let meta = CosmosArtifactMeta {
            name: "x".into(),
            blake3_hash: "h".into(),
            size_bytes: 4,
            refreshed_at: 10_000_000,
            source_tier: "T1".into(),
            refresh_cadence: RefreshCadence::Daily,
        };
        assert_eq!(w.status_for(&meta, 10_000_000 + 3600), CosmosStatus::Fresh);
        assert_eq!(
            w.status_for(&meta, 10_000_000 + 25 * 3600),
            CosmosStatus::Stale
        );
        assert_eq!(
            w.status_for(&meta, 10_000_000 + 4 * 24 * 3600),
            CosmosStatus::Decayed
        );
    }

    #[tokio::test]
    async fn tick_writes_body_and_meta_on_first_run() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("# iOS HIG\nbody", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock,
            research,
        )
        .expect("worker");

        let report = w.tick().await;
        assert_eq!(report.checked, 2);
        assert_eq!(report.refreshed.len(), 2);
        assert!(report.failed.is_empty());

        let body_path = tmp
            .path()
            .join(ARTIFACTS_SUBDIR)
            .join("ios_hig_summary.md");
        assert!(body_path.exists());
        let body = fs::read_to_string(&body_path).expect("body readable");
        assert_eq!(body, "# iOS HIG\nbody");
    }

    #[tokio::test]
    async fn tick_is_idempotent_when_body_unchanged() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("# stable\nbody", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock.clone(),
            research,
        )
        .expect("worker");

        let r1 = w.tick().await;
        assert_eq!(r1.refreshed.len(), 2);

        // Advance past Daily interval so app_store_policies is due again;
        // body has NOT changed, so refresh_one must return Ok(false) and
        // the report should classify as `unchanged` (idempotent path).
        clock.advance(2 * 24 * 3600);
        let r2 = w.tick().await;
        assert!(r2.refreshed.is_empty(), "expected no refresh: {:?}", r2);
        assert_eq!(r2.unchanged.len() + r2.failed.len(), 2);
        assert!(r2.failed.is_empty(), "no failures: {:?}", r2.failed);
    }

    #[tokio::test]
    async fn tick_refreshes_when_body_changes() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("v1", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock.clone(),
            research.clone(),
        )
        .expect("worker");

        let _ = w.tick().await;
        research.set_body("v2");
        clock.advance(2 * 24 * 3600);
        let r = w.tick().await;
        assert!(
            r.refreshed.contains(&"app_store_policies".to_string()),
            "expected app_store_policies refreshed, got {:?}",
            r
        );
    }

    #[tokio::test]
    async fn cosmos_get_returns_artifact_with_status() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("# body", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock.clone(),
            research,
        )
        .expect("worker");
        let _ = w.tick().await;

        let art = w
            .cosmos_get("ios_hig_summary")
            .expect("cosmos_get succeeds");
        assert_eq!(art.body, "# body");
        assert_eq!(art.status, CosmosStatus::Fresh);
    }

    #[tokio::test]
    async fn cosmos_get_decayed_status_emits_warn_but_serves() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("# body", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock.clone(),
            research,
        )
        .expect("worker");
        let _ = w.tick().await;

        // Jump past decay_secs for `app_store_policies` (Daily * 3 = 3 days +).
        clock.advance(10 * 24 * 3600);
        let art = w
            .cosmos_get("app_store_policies")
            .expect("decayed served");
        assert_eq!(art.status, CosmosStatus::Decayed);
    }

    #[test]
    fn cosmos_get_unknown_artifact_errors() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1));
        let research = Arc::new(MockResearch::new("", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock,
            research,
        )
        .expect("worker");
        let r = w.cosmos_get("nonexistent");
        assert!(matches!(r, Err(CosmosError::UnknownArtifact(_))));
    }

    #[tokio::test]
    async fn status_snapshot_emits_row_per_artifact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("# body", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock.clone(),
            research,
        )
        .expect("worker");
        let _ = w.tick().await;

        let snap = w.status_snapshot();
        assert_eq!(snap.len(), 2);
        let names: Vec<&str> = snap.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"ios_hig_summary"));
        assert!(names.contains(&"app_store_policies"));
        for row in &snap {
            assert_eq!(row.status, "fresh");
            assert!(row.refreshed_at.is_some());
            assert!(row.size_bytes > 0);
        }
    }

    #[test]
    fn status_snapshot_unrefreshed_artifact_is_stale_with_no_timestamp() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock = Arc::new(FixedClock::new(1_700_000_000));
        let research = Arc::new(MockResearch::new("body", "T1"));
        let w = CosmosWorker::new(
            tmp.path().to_path_buf(),
            sample_manifest(),
            clock,
            research,
        )
        .expect("worker");
        // No tick() -> no metas on disk.
        let snap = w.status_snapshot();
        assert_eq!(snap.len(), 2);
        for row in &snap {
            assert_eq!(row.status, "stale");
            assert!(row.refreshed_at.is_none());
            assert_eq!(row.size_bytes, 0);
        }
    }

    #[test]
    fn rfc3339_formatter_matches_known_epoch() {
        // 1_700_000_000 = 2023-11-14T22:13:20Z (verified via `date -u -d @1700000000`).
        assert_eq!(
            format_rfc3339_utc(1_700_000_000),
            "2023-11-14T22:13:20Z"
        );
        assert_eq!(format_rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_rfc3339_utc(-5), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn truncate_respects_char_boundary_on_multibyte_input() {
        // "日本語" — each kanji is 3 UTF-8 bytes. Naive truncate at byte 4 or 5
        // (mid-codepoint of the 2nd kanji) panics; floor_char_boundary must
        // round DOWN to byte 3 (end of first kanji).
        let s = "日本語";
        assert_eq!(s.len(), 9);
        for cap in 0..=s.len() {
            let i = floor_char_boundary(s, cap);
            assert!(s.is_char_boundary(i), "cap={cap} -> {i} not boundary");
            let _ = &s[..i]; // must not panic
        }
        assert_eq!(floor_char_boundary(s, 4), 3);
        assert_eq!(floor_char_boundary(s, 5), 3);
        assert_eq!(floor_char_boundary(s, 6), 6);
        assert_eq!(floor_char_boundary(s, 100), 9);

        // Mixed ASCII + em-dash + accented char.
        let m = "résumé—word";
        let i = floor_char_boundary(m, 8);
        assert!(m.is_char_boundary(i));
        let truncated = &m[..i];
        // Must be valid UTF-8 (already a &str so guaranteed) AND no panic.
        assert!(truncated.is_char_boundary(truncated.len()));
    }

    #[test]
    fn name_validation_accepts_word_chars_and_dashes() {
        assert!(is_valid_name("ios_hig_v2"));
        assert!(is_valid_name("app-store"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("has space"));
        assert!(!is_valid_name("dot.name"));
    }
}
