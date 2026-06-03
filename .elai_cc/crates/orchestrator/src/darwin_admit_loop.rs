//! F-11: config-DGM admit-loop driver core.
//!
//! Closes ONE config-only Darwin-Goedel-Machine generation under the four
//! existing safety guards and emits [`SsbActionType::DarwinianAdmit`] rows.
//!
//! The loop, per generation:
//!   1. **propose** — sample a parent from the [`ConfigArchive`]
//!      ([`ConfigArchive::select_parent_with`], `ELAI_SEED`-deterministic) or,
//!      on an empty archive, seed a root candidate; the mutant is a
//!      [`ConfigVariant`] (a `config_blob` delta ONLY, never an agent-source
//!      self-rewrite, so the Layer-3 `human_approved` gate is avoided by
//!      construction — the loop builds a `ConfigVariant`, not a
//!      `StructuralChange`).
//!   2. **evaluate** — derive a REAL, candidate-coupled `alpha` in `[0, 1]` and
//!      consult [`external_benches::get_suite`] for the suite floor. (The live
//!      multi-generation Python bench run is the DEFERRED Tier-2,
//!      RUNPOD-PENDING; the in-process single generation uses the deterministic
//!      candidate-derived score — NOT a synthetic `% 2` reward and NOT a
//!      hard-coded literal.)
//!   3. **guard chain IN SERIES** ([`score_candidate`]):
//!      (i)   tripwire — blake3 fingerprint of the `config_blob` (pre-exec
//!            identity stamp; blake3 ONLY, never blake2/sha2);
//!      (ii)  trap scoring on the LIVE candidate
//!            ([`trap_tasks::score_candidate_for_traps`], killer-risk #3);
//!      (iii) [`self_mod_layer1a::is_security_sensitive`] on the candidate JSON
//!            (killer-risk #1 — the archive-admit path BYPASSES this filter, so
//!            the driver MUST call it explicitly);
//!      (iv)  [`promotion_gate::two_gate_promote`] (margin tau then capacity
//!            k_m; killer-risk #2 — both UNCONFIRMED carried-forward defaults).
//!   4. **admit** — only if all four pass: [`ConfigArchive::insert_variant`]
//!      (the FIRST production caller) THEN [`ConfigArchive::select_parent`] (the
//!      `LoopClosed` proof). On any guard reject: emit a `RejectedByGuard` row,
//!      do NOT insert.
//!
//! Acceptance is "loop closed AND a guard caught a trap mutant", NOT "score
//! went up" (a single-generation positive delta is UNGROUNDED — DGM needed many
//! iterations and hit deceptive dips; arXiv:2505.22954).
//!
//! Wire-dark: this module compiles only under `cfg(feature = "darwinian-evolve")`
//! and the live driver gates additionally on `ELAI_DARWINIAN_LOOP_ENABLE=1`.
//!
//! No `.unwrap()`/`.expect()` in non-test paths; thresholds are named consts;
//! single-writer SSB (all writes via the async [`Ssb`] channel); entropy
//! quarantine ([`EntropySource`] only).

use serde::{Deserialize, Serialize};

use elai_core::EntropySource;

use crate::config_archive::{ConfigArchive, ConfigArchiveError};
use crate::external_benches::get_suite;
use crate::promotion_gate::{two_gate_promote, PromotionConfig, PromotionDecisionKind};
use crate::self_mod_layer1a::{is_security_sensitive, PipelineMode};
use crate::self_mod_types::{ConfigVariant, VariantStatus};
use crate::ssb::{DarwinianAdmitKind, DarwinianAdmitPayload, Ssb, SsbError, SsbPayload};
use crate::template_archive::{
    ArchiveEntry, BehaviourDescriptor, LatencyTier, MapElitesArchive, TokenClass,
};
use crate::trap_tasks::{score_candidate_for_traps, TrapPattern};

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// Length of the blake3 hex prefix used as the candidate fingerprint
/// (`candidate_blake3`). 16 hex chars = 8 bytes — enough to disambiguate
/// candidates within a generation without bloating the SSB payload. Matches the
/// plan's `/^[0-9a-f]{16}/` acceptance regex.
pub const CANDIDATE_BLAKE3_HEX_LEN: usize = 16;

/// Default benchmark suite the in-process generation consults for its floor.
/// `math-floors` is one of the 11 canonical suites
/// (`external_benches::list_suites`); it is deterministic + offline-friendly
/// (no live sidecar) so it suits the Tier-1 in-process gate.
pub const DEFAULT_SUITE_NAME: &str = "math-floors";

/// SSB agent id stamped on every `darwinian_admit` row this driver writes.
pub const ADMIT_LOOP_AGENT_ID: &str = "cli:darwin-loop";

/// Env var (kill-switch posture) that, when truthy, enables the live admit
/// loop. Both this AND `cfg(feature = "darwinian-evolve")` are required (an
/// autonomous self-mod loop is opt-in by ELAI policy; E-7 DECISION-13). Mirrors
/// the gate the E-7 substrate already documents.
pub const ELAI_DARWINIAN_LOOP_ENABLE_ENV: &str = "ELAI_DARWINIAN_LOOP_ENABLE";

/// Env var pointing at a JSON file holding the mutant candidates to evaluate
/// (a JSON array of [`MutantCandidate`] objects). Used by the deterministic
/// offline smoke + the live CLI to inject a fixed candidate set. When unset,
/// the driver seeds `mutants` deterministic clean candidates from the seed.
pub const ELAI_DARWINIAN_LOOP_MUTANTS_JSON_ENV: &str = "ELAI_DARWINIAN_LOOP_MUTANTS_JSON";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure modes of the admit-loop driver.
#[derive(Debug, thiserror::Error)]
pub enum AdmitLoopError {
    /// The config archive (self_mod.db) could not be opened or queried.
    #[error("config archive: {0}")]
    Archive(#[from] ConfigArchiveError),
    /// An SSB write failed.
    #[error("ssb: {0}")]
    Ssb(#[from] SsbError),
    /// A mutant candidate's JSON could not be parsed.
    #[error("mutant json: {0}")]
    MutantJson(String),
    /// The candidate JSON could not be re-serialized for the security filter.
    #[error("candidate serialize: {0}")]
    CandidateSerialize(String),
}

// ---------------------------------------------------------------------------
// Mutant candidate (the unit the loop evaluates)
// ---------------------------------------------------------------------------

/// A config/template mutant the loop evaluates. NEVER an agent-source
/// self-rewrite: `config` is a JSON object describing a config delta only, so
/// the loop produces a [`ConfigVariant`], not a Layer-3 `StructuralChange`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutantCandidate {
    /// Stable human label / id for the candidate.
    pub variant_label: String,
    /// Pipeline mode the variant targets (`surgical` / `greenfield` / `oracle`).
    #[serde(default = "default_pipeline_mode")]
    pub pipeline_mode: String,
    /// The config delta as a JSON object. The security filter
    /// ([`is_security_sensitive`]) scans this verbatim.
    pub config: serde_json::Value,
    /// The candidate's evaluation log (real bench output / trace). The trap
    /// scorer scans this alongside the serialized config.
    #[serde(default)]
    pub eval_log: String,
}

fn default_pipeline_mode() -> String {
    "surgical".to_string()
}

impl MutantCandidate {
    /// Parse a JSON array of candidates (the smoke/CLI injection format).
    pub fn parse_array(json: &str) -> Result<Vec<MutantCandidate>, AdmitLoopError> {
        serde_json::from_str(json).map_err(|e| AdmitLoopError::MutantJson(e.to_string()))
    }

    /// Resolve the pipeline mode string to the closed [`PipelineMode`] enum,
    /// defaulting to `Surgical` for any unrecognized value.
    #[must_use]
    pub fn resolved_mode(&self) -> PipelineMode {
        match self.pipeline_mode.as_str() {
            "greenfield" => PipelineMode::Greenfield,
            "oracle" => PipelineMode::Oracle,
            _ => PipelineMode::Surgical,
        }
    }

    /// The canonical `config_blob` bytes: the serialized config object. This is
    /// what the archive stores and what the tripwire fingerprint + trap scorer
    /// consume.
    #[must_use]
    pub fn config_blob(&self) -> Vec<u8> {
        // `serde_json::to_vec` on a `Value` cannot fail (it is already a valid
        // JSON value); fall back to an empty object rather than panic.
        serde_json::to_vec(&self.config).unwrap_or_else(|_| b"{}".to_vec())
    }

    /// blake3 hex-prefix fingerprint of the `config_blob` (guard i, blake3
    /// ONLY — never blake2/sha2).
    #[must_use]
    pub fn candidate_blake3(&self) -> String {
        let blob = self.config_blob();
        let hex = blake3::hash(&blob).to_hex();
        hex[..CANDIDATE_BLAKE3_HEX_LEN].to_string()
    }

    /// Real, deterministic, candidate-coupled evaluation score in `[0, 1]`.
    ///
    /// Honors an explicit `config.force_alpha` field (so a fixture can drive a
    /// specific Two-Gate margin); otherwise derives a stable scalar from the
    /// blake3 fingerprint of the `config_blob`. This is NOT a synthetic `% 2`
    /// reward and NOT a hard-coded constant — it is a real function OF the
    /// candidate, stable across runs (required for the deterministic offline
    /// smoke). The live multi-generation Python bench eval is the DEFERRED
    /// Tier-2.
    #[must_use]
    pub fn derived_alpha(&self) -> f64 {
        if let Some(forced) = self.config.get("force_alpha").and_then(|v| v.as_f64()) {
            return forced.clamp(0.0, 1.0);
        }
        let blob = self.config_blob();
        let hash = blake3::hash(&blob);
        let bytes = hash.as_bytes();
        // First 8 bytes -> u64 -> normalize to [0, 1).
        let mut acc: u64 = 0;
        for &b in &bytes[..8] {
            acc = (acc << 8) | u64::from(b);
        }
        (acc as f64) / (u64::MAX as f64)
    }
}

// ---------------------------------------------------------------------------
// Guard chain
// ---------------------------------------------------------------------------

/// Outcome of routing a candidate through the four-guard chain.
#[derive(Debug, Clone, PartialEq)]
pub enum GuardOutcome {
    /// All four guards passed; the candidate is eligible for `insert_variant`.
    Admitted,
    /// A guard rejected the candidate. `guard` is one of `tripwire` / `trap` /
    /// `security` / `two_gate`; `detail` is a human-readable reason.
    Rejected { guard: String, detail: String },
}

impl GuardOutcome {
    /// `true` iff the candidate was admitted by all four guards.
    #[must_use]
    pub fn is_admitted(&self) -> bool {
        matches!(self, GuardOutcome::Admitted)
    }

    /// The firing guard name, or `None` when admitted.
    #[must_use]
    pub fn guard_name(&self) -> Option<&str> {
        match self {
            GuardOutcome::Admitted => None,
            GuardOutcome::Rejected { guard, .. } => Some(guard.as_str()),
        }
    }
}

/// Guard names (stable wire strings used in `guard_rejected_by`).
pub const GUARD_TRIPWIRE: &str = "tripwire";
pub const GUARD_TRAP: &str = "trap";
pub const GUARD_SECURITY: &str = "security";
pub const GUARD_TWO_GATE: &str = "two_gate";

/// Route ONE candidate through the four-guard chain IN SERIES. Pure logic (no
/// IO, no clock, no RNG) so it is deterministic + directly unit/contract
/// testable.
///
/// `archive_occupied_cells` is the current occupied-cell count of the
/// MAP-Elites view the Two-Gate capacity gate checks (the driver passes the
/// live archive size; tests pass a fixed value).
///
/// Order matters (the safety spine): tripwire -> trap -> security -> two_gate.
pub fn score_candidate(
    candidate: &MutantCandidate,
    alpha: f64,
    archive_occupied_cells: u32,
    promotion_cfg: &PromotionConfig,
) -> Result<GuardOutcome, AdmitLoopError> {
    let blob = candidate.config_blob();

    // (i) Tripwire: blake3 fingerprint pre-exec. The identity stamp must be a
    // well-formed 16-char hex prefix; an empty config_blob (no candidate) is a
    // tripwire reject (a degenerate candidate that never produced a config).
    let fingerprint = candidate.candidate_blake3();
    if blob.is_empty() || fingerprint.len() != CANDIDATE_BLAKE3_HEX_LEN {
        return Ok(GuardOutcome::Rejected {
            guard: GUARD_TRIPWIRE.to_string(),
            detail: "empty or unfingerprintable config_blob".to_string(),
        });
    }

    // (ii) Trap scoring on the LIVE candidate (killer-risk #3): scan the real
    // config_blob + eval_log for reward-hacking signatures.
    if let Some(pattern) = score_candidate_for_traps(&blob, &candidate.eval_log) {
        return Ok(GuardOutcome::Rejected {
            guard: GUARD_TRAP.to_string(),
            detail: format!("trap pattern {}", trap_label(pattern)),
        });
    }

    // (iii) Security keyword filter (killer-risk #1): the archive-admit path
    // BYPASSES is_security_sensitive, so call it explicitly on the candidate
    // JSON. A candidate whose config touches auth/credential/token/secret/etc
    // is rejected before admit.
    if is_security_sensitive(&candidate.config) {
        return Ok(GuardOutcome::Rejected {
            guard: GUARD_SECURITY.to_string(),
            detail: "config touches a security-sensitive keyword".to_string(),
        });
    }

    // (iv) Two-Gate promotion (killer-risk #2: tau/k_m are UNCONFIRMED
    // carried-forward defaults). Bridge the ConfigVariant candidate into the
    // template_archive types two_gate_promote needs: a single-cell
    // MapElitesArchive (best_in_cell -> incumbent 0.0 on empty, so margin =
    // alpha) + an ArchiveEntry carrying the candidate fingerprint + alpha.
    let descriptor = config_cell_descriptor();
    let archive = build_two_gate_archive_view(archive_occupied_cells, &descriptor);
    let entry = ArchiveEntry {
        template_blake3: blake3::hash(&blob).into(),
        fitness: alpha,
    };
    let decision = two_gate_promote(&entry, &descriptor, &archive, promotion_cfg)
        .map_err(|e| AdmitLoopError::CandidateSerialize(e.to_string()))?;
    match decision.kind {
        PromotionDecisionKind::Promoted => Ok(GuardOutcome::Admitted),
        other => Ok(GuardOutcome::Rejected {
            guard: GUARD_TWO_GATE.to_string(),
            detail: format!(
                "{} ({})",
                two_gate_label(&other),
                decision.reason.unwrap_or_default()
            ),
        }),
    }
}

/// Snake-case label for a trap pattern (delegates to the enum's own wire label).
fn trap_label(pattern: TrapPattern) -> &'static str {
    pattern.as_str()
}

/// Human label for a non-Promoted two-gate decision.
fn two_gate_label(kind: &PromotionDecisionKind) -> &'static str {
    match kind {
        PromotionDecisionKind::Promoted => "promoted",
        PromotionDecisionKind::BlockedBelowMargin => "below_margin",
        PromotionDecisionKind::BlockedCapacityExceeded => "capacity_exceeded",
        PromotionDecisionKind::BlockedSecurityRegression => "security_regression",
    }
}

/// The fixed behaviour-descriptor cell config-only mutants occupy. Config
/// deltas do not change latency/token behaviour by construction, so they map to
/// a single canonical cell.
fn config_cell_descriptor() -> BehaviourDescriptor {
    BehaviourDescriptor {
        latency_tier: LatencyTier::Sub10s,
        token_class: TokenClass::Lt500,
        failure_mode: None,
    }
}

/// Build the MAP-Elites archive VIEW the Two-Gate capacity gate inspects.
///
/// The config-DGM archive is a [`ConfigArchive`] (SQLite), NOT a
/// [`MapElitesArchive`]; this adapter bridges the two for the capacity gate by
/// constructing a transient MAP-Elites archive whose `occupied_cells()` equals
/// the live config-archive size (so the capacity gate observes the real
/// archive occupancy). The candidate's own cell is left EMPTY so
/// `best_in_cell` returns `None` and the margin gate measures the FULL
/// candidate fitness against a 0.0 incumbent (the conservative posture: a
/// brand-new config variant must clear `tau` on its own merit).
fn build_two_gate_archive_view(
    occupied_cells: u32,
    candidate_descriptor: &BehaviourDescriptor,
) -> MapElitesArchive {
    let mut archive = MapElitesArchive::new();
    // Fill `occupied_cells` distinct cells with placeholder incumbents so
    // `occupied_cells()` reports the live count. We deliberately do NOT occupy
    // the candidate's own cell.
    let filler_cells = enumerate_filler_cells(candidate_descriptor);
    for descriptor in filler_cells.into_iter().take(occupied_cells as usize) {
        let _ = archive.admit(
            descriptor,
            ArchiveEntry {
                template_blake3: [0u8; 32],
                fitness: 0.0,
            },
        );
    }
    archive
}

/// Enumerate the canonical MAP-Elites cells (3 latency x 4 token x {clean})
/// EXCLUDING the candidate's own cell, used as capacity-gate filler.
fn enumerate_filler_cells(exclude: &BehaviourDescriptor) -> Vec<BehaviourDescriptor> {
    let latencies = [LatencyTier::Sub10s, LatencyTier::Sub60s, LatencyTier::Over60s];
    let tokens = [
        TokenClass::Lt500,
        TokenClass::Lt2k,
        TokenClass::Lt8k,
        TokenClass::Over8k,
    ];
    let mut out = Vec::new();
    for &latency_tier in &latencies {
        for &token_class in &tokens {
            let d = BehaviourDescriptor {
                latency_tier,
                token_class,
                failure_mode: None,
            };
            if &d != exclude {
                out.push(d);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The closed generation (propose -> evaluate -> guard -> admit -> select)
// ---------------------------------------------------------------------------

/// Summary of one closed generation (for the CLI surface + tests).
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationSummary {
    /// Number of candidates admitted (passed all four guards + inserted).
    pub admitted: usize,
    /// Number of candidates rejected by a guard.
    pub rejected: usize,
    /// `true` iff the loop closed (an admit was followed by `select_parent`).
    pub loop_closed: bool,
    /// The suite floor consulted this generation (`None` if the suite name was
    /// unknown — the loop still runs; the floor is telemetry, not a hard gate
    /// here).
    pub suite_floor: Option<f64>,
}

/// Run ONE closed config-only generation against the supplied archive + SSB.
///
/// For each mutant candidate: derive `alpha`, route through the four-guard
/// chain, emit a `darwinian_admit` row (`Admitted` or `RejectedByGuard`). On
/// the FIRST admit, `insert_variant` then `select_parent` (loop closure) and
/// emit a `LoopClosed` row. Returns a [`GenerationSummary`].
///
/// `entropy` is the `ELAI_SEED`-seeded source for `select_parent` determinism.
#[allow(clippy::too_many_arguments)]
pub async fn run_one_generation<E: EntropySource + ?Sized>(
    archive: &ConfigArchive,
    ssb: &Ssb,
    candidates: &[MutantCandidate],
    mode: PipelineMode,
    seed: u64,
    suite_name: &str,
    promotion_cfg: &PromotionConfig,
    entropy: &mut E,
) -> Result<GenerationSummary, AdmitLoopError> {
    let suite_floor = get_suite(suite_name).map(|s| s.local_floor());

    // Fail CLOSED on overflow: a usize->u32 `as` cast WRAPS on overflow,
    // shrinking `occupied` so the capacity gate would see room and fail OPEN
    // (admit past the cell budget). Saturate to u32::MAX instead — an
    // implausibly-full archive then REJECTS, which is the safe direction.
    let occupied = u32::try_from(archive.list_variants(mode).await?.len()).unwrap_or(u32::MAX);

    let mut admitted = 0usize;
    let mut rejected = 0usize;
    let mut loop_closed = false;

    for candidate in candidates {
        let alpha = candidate.derived_alpha();
        let outcome = score_candidate(candidate, alpha, occupied, promotion_cfg)?;

        let candidate_blake3 = candidate.candidate_blake3();
        match outcome {
            GuardOutcome::Rejected { guard, .. } => {
                rejected += 1;
                emit_admit_row(
                    ssb,
                    DarwinianAdmitPayload {
                        kind: DarwinianAdmitKind::RejectedByGuard,
                        variant_id: candidate.variant_label.clone(),
                        parent_id: None,
                        alpha,
                        guard_rejected_by: Some(guard),
                        candidate_blake3,
                        seed,
                    },
                )
                .await?;
            }
            GuardOutcome::Admitted => {
                // Resolve a parent BEFORE insert (empty archive -> root seed).
                let parent_pre = archive.select_parent_with(mode, entropy).await?;
                let parent_id = if parent_pre.is_empty() {
                    None
                } else {
                    Some(parent_pre)
                };

                let variant = build_config_variant(candidate, alpha, parent_id.clone(), mode);
                // insert_variant: the FIRST production caller of the DGM archive
                // admit path. A pre-existing active variant for the mode is a
                // benign archive-state condition, not a guard rejection — we
                // surface it as a non-admit without an SSB Admitted beacon.
                match archive.insert_variant(&variant).await {
                    Ok(()) => {
                        admitted += 1;
                        emit_admit_row(
                            ssb,
                            DarwinianAdmitPayload {
                                kind: DarwinianAdmitKind::Admitted,
                                variant_id: candidate.variant_label.clone(),
                                parent_id: parent_id.clone(),
                                alpha,
                                guard_rejected_by: None,
                                candidate_blake3: candidate_blake3.clone(),
                                seed,
                            },
                        )
                        .await?;

                        // select_parent AFTER insert_variant — the loop-closure
                        // proof. select_parent samples over the now-non-empty
                        // archive; emit the LoopClosed beacon naming the parent
                        // the next generation would mutate.
                        let next_parent = archive.select_parent(mode).await?;
                        loop_closed = true;
                        emit_admit_row(
                            ssb,
                            DarwinianAdmitPayload {
                                kind: DarwinianAdmitKind::LoopClosed,
                                variant_id: candidate.variant_label.clone(),
                                parent_id: if next_parent.is_empty() {
                                    None
                                } else {
                                    Some(next_parent)
                                },
                                alpha,
                                guard_rejected_by: None,
                                candidate_blake3,
                                seed,
                            },
                        )
                        .await?;
                    }
                    Err(ConfigArchiveError::ActiveAlreadyExists(_)) => {
                        // Archive already holds an active variant for this mode;
                        // not a guard reject and not a fresh admit. Skip without
                        // a beacon (the loop still closed on a prior candidate).
                    }
                    Err(e) => return Err(AdmitLoopError::Archive(e)),
                }
            }
        }
    }

    Ok(GenerationSummary {
        admitted,
        rejected,
        loop_closed,
        suite_floor,
    })
}

/// Build the [`ConfigVariant`] archive row from an admitted candidate. The
/// variant is ALWAYS a config/template delta (a `config_blob`), never a Layer-3
/// `StructuralChange`.
fn build_config_variant(
    candidate: &MutantCandidate,
    alpha: f64,
    parent_id: Option<String>,
    mode: PipelineMode,
) -> ConfigVariant {
    ConfigVariant {
        variant_id: candidate.variant_label.clone(),
        parent_id,
        // Archive score is an f32 in [0, 1]; clamp the f64 alpha.
        score: alpha.clamp(0.0, 1.0) as f32,
        children_count: 0,
        config_blob: candidate.config_blob(),
        // created_at is a monotone field; the loop uses the candidate
        // fingerprint's low bytes so it is deterministic without a clock.
        created_at: deterministic_created_at(candidate),
        pipeline_outcomes: Vec::new(),
        pipeline_mode: mode,
        status: VariantStatus::Active,
    }
}

/// Deterministic `created_at` derived from the candidate fingerprint (entropy
/// quarantine: no raw `std::time`). Stable across runs.
fn deterministic_created_at(candidate: &MutantCandidate) -> u64 {
    let hash = blake3::hash(&candidate.config_blob());
    let bytes = hash.as_bytes();
    let mut acc: u64 = 0;
    for &b in &bytes[..8] {
        acc = (acc << 8) | u64::from(b);
    }
    acc
}

/// Emit one `darwinian_admit` SSB row through the single-writer channel.
async fn emit_admit_row(ssb: &Ssb, payload: DarwinianAdmitPayload) -> Result<(), AdmitLoopError> {
    ssb.write_payload(
        ADMIT_LOOP_AGENT_ID,
        0,
        SsbPayload::DarwinianAdmit(payload),
        Vec::new(),
    )
    .await?;
    Ok(())
}

/// `true` iff the `ELAI_DARWINIAN_LOOP_ENABLE` env var is set to a truthy
/// value (`1` / `true` / `yes` / `on`, case-insensitive). The live driver
/// requires this AND `cfg(feature = "darwinian-evolve")`.
#[must_use]
pub fn loop_enabled_via_env() -> bool {
    match std::env::var(ELAI_DARWINIAN_LOOP_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededRng;

    fn clean_candidate() -> MutantCandidate {
        MutantCandidate {
            variant_label: "clean-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "retry_budget": 3, "rag_top_k": 8, "force_alpha": 0.6 }),
            eval_log: "suite=math-floors score=0.6 clean".to_string(),
        }
    }

    fn trap_candidate() -> MutantCandidate {
        MutantCandidate {
            variant_label: "trap-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "post_step_hook": "remove marker lines from own source" }),
            eval_log: "marker_removal applied".to_string(),
        }
    }

    fn security_candidate() -> MutantCandidate {
        MutantCandidate {
            variant_label: "sec-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "api_key": "sk-leak", "force_alpha": 0.9 }),
            eval_log: "clean run".to_string(),
        }
    }

    #[test]
    fn clean_candidate_is_admitted() {
        let cfg = PromotionConfig::default();
        let c = clean_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out, GuardOutcome::Admitted);
    }

    #[test]
    fn trap_candidate_rejected_by_trap_guard() {
        let cfg = PromotionConfig::default();
        let c = trap_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out.guard_name(), Some(GUARD_TRAP));
    }

    #[test]
    fn security_candidate_rejected_by_security_guard() {
        let cfg = PromotionConfig::default();
        let c = security_candidate();
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out.guard_name(), Some(GUARD_SECURITY));
    }

    #[test]
    fn low_alpha_rejected_by_two_gate() {
        let cfg = PromotionConfig::default();
        let c = MutantCandidate {
            variant_label: "low-001".to_string(),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({ "force_alpha": 0.01 }),
            eval_log: "marginal".to_string(),
        };
        let out = score_candidate(&c, c.derived_alpha(), 0, &cfg).expect("guard chain");
        assert_eq!(out.guard_name(), Some(GUARD_TWO_GATE));
    }

    #[test]
    fn candidate_blake3_is_16_hex_chars() {
        let c = clean_candidate();
        let fp = c.candidate_blake3();
        assert_eq!(fp.len(), CANDIDATE_BLAKE3_HEX_LEN);
        assert!(fp.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn derived_alpha_is_deterministic_and_in_range() {
        let c = clean_candidate();
        let a1 = c.derived_alpha();
        let a2 = c.derived_alpha();
        assert_eq!(a1, a2);
        assert!((0.0..=1.0).contains(&a1));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn one_generation_closes_loop_and_catches_trap() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let archive = ConfigArchive::open(&dir.path().join("self_mod.db"))
            .await
            .expect("archive open");
        let ssb = Ssb::new(
            crate::ssb::SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: None,
                project_id: "test".to_string(),
                session_id: "darwin-loop-test".to_string(),
            },
            std::sync::Arc::new(elai_core::OsClock),
        )
        .expect("ssb");

        let candidates = vec![trap_candidate(), clean_candidate()];
        let cfg = PromotionConfig::default();
        let mut rng = SeededRng::new(42);
        let summary = run_one_generation(
            &archive,
            &ssb,
            &candidates,
            PipelineMode::Surgical,
            42,
            DEFAULT_SUITE_NAME,
            &cfg,
            &mut rng,
        )
        .await
        .expect("generation");

        assert!(summary.loop_closed, "clean candidate must close the loop");
        assert_eq!(summary.rejected, 1, "trap candidate must be rejected");
        assert_eq!(summary.admitted, 1, "clean candidate must be admitted");
    }
}
