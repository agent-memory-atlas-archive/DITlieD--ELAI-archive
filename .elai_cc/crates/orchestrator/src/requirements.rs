//! I-37 — EARS requirement objects.
//!
//! Typed, hash-keyed, mutation-covered EARS requirement records with a
//! deterministic VERIFIED / UNVERIFIED / REGRESSED status. The field's named
//! open problem in specification-driven testing is the *behaviour harness*:
//! linking a natural-language requirement to a deterministic verified status
//! without a human reviewer in the loop. [`IntentPair`](crate::intent_pair::IntentPair)
//! (IS-15, C-29) is the nearest substrate but captures a single intent-evidence
//! pair, not a structured typed requirement with bound tests and coverage. I-37
//! builds the missing layer.
//!
//! Each [`EarsRequirement`] carries:
//!   * `content_hash` — blake3 of the canonical requirement text (drift detector).
//!   * `pattern`      — the EARS stereotype ([`EarsPattern`], an enum, never a
//!     freeform string).
//!   * `bound_tests`  — the test symbols the record claims cover this requirement.
//!   * `mutation_coverage` — the I-20 sampled mutation score on the bound tests at
//!     the time of the last audit (composed from
//!     [`crate::mutation_budget`], NOT recomputed here).
//!   * `metamorphic_relations` — the IS-8 metamorphic-relation ids that bind to
//!     this requirement (composed from [`crate::metamorphic_testing`]).
//!   * `verified_status` — [`VerifiedStatus`]: Satisfied iff
//!     `mutation_coverage >= floor` AND every bound metamorphic relation holds AND
//!     at least one bound test exists; Regressed when a previously-bound test was
//!     deleted; Unverified when the record has no bound tests (SHOWN, never
//!     dropped — R1).
//!
//! ## EARS parsing
//!
//! EARS (Easy Approach to Requirements Syntax; Mavin et al., Requirements
//! Engineering Conference) defines five stereotyped line shapes. [`parse_speckit_file`]
//! / [`parse_ears_line`] recognise them with a hand-rolled line state-machine
//! (NO new dependency — the five templates are line-prefix patterns, not a
//! structured-document grammar):
//!   * Ubiquitous     — `the <system> shall <response>`
//!   * Event-driven   — `when <trigger> the <system> shall <response>`
//!   * State-driven    — `while <state> the <system> shall <response>`
//!   * Unwanted        — `if <condition> then the <system> shall <response>`
//!   * Optional        — `where <feature> the <system> shall <response>`
//!
//! ## Wiring
//!
//! The blocking [`RequirementsGate`] (PLANNING → TESTING_FIRST) is **default-OFF**
//! behind [`ELAI_REQUIREMENTS_GATE_ENABLE_ENV`]; with the flag absent it
//! degrades to pass (changes no production gate behaviour). The observability
//! emit — one [`RequirementVerified`](crate::ssb::SsbActionType::RequirementVerified)
//! SSB row per requirement — is driven from the PLANNING → TESTING_FIRST seam in
//! `task_runner` via [`emit_requirement_verified_to_ssb`] and is independent of
//! the blocking flag (the row always lands when an `EarsRequirementSet` is present,
//! so the cockpit shows Unverified records rather than silently dropping them).
//!
//! ## Grounding
//!
//! * Plan: `docs/thefinalglance/I-plan/I-37_ears_requirement_objects.md`
//! * IS-15 substrate: `crate::intent_pair::IntentPair` (the Ubiquitous base case).
//! * IS-8 substrate: `crate::metamorphic_testing::RelationLibrary` (MR binding).
//! * I-20 substrate: `crate::mutation_budget` (sampled coverage source).
//! * blake3 only for `content_hash` (CLAUDE.md anti-stale; never blake2/sha2).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::evidence::{EvidenceError, EvidenceGate, TransitionContext};
use crate::intent_pair::IntentPair;
use crate::ssb::{
    env_gate_truthy, RequirementVerifiedKind, RequirementVerifiedPayload, Ssb, SsbPayload,
    ELAI_REQUIREMENTS_GATE_ENABLE_ENV,
};
use crate::FsmState;

// ---------------------------------------------------------------------------
// Constants (ID-5: every threshold is a NAMED constant, never a magic literal)
// ---------------------------------------------------------------------------

/// Default mutation-coverage floor a requirement must clear (on its bound
/// tests) to be considered Satisfied. ID-5: this is the wired DEFAULT used when
/// a caller does not supply a measured floor; production callers pass the
/// I-20-measured floor (`metrics/mutation_baseline.toml`) via
/// [`verify_requirement`]'s `coverage_floor` argument. It is NOT a paper
/// constant baked into the gate logic — it is the conservative default for the
/// fixture / unconfigured path.
pub const DEFAULT_MUTATION_COVERAGE_FLOOR: f64 = 0.80;

// ---------------------------------------------------------------------------
// EARS pattern + verified status enums
// ---------------------------------------------------------------------------

/// The five EARS stereotypes (Mavin et al.). An enum, never a freeform string
/// (stub_ban: a freeform pattern field is a type violation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarsPattern {
    /// `the <system> shall <response>` — always-active requirement.
    Ubiquitous,
    /// `when <trigger> the <system> shall <response>` — event-driven.
    EventDriven,
    /// `while <state> the <system> shall <response>` — state-driven.
    StateDriven,
    /// `where <feature> the <system> shall <response>` — optional / feature-gated.
    Optional,
    /// `if <condition> then the <system> shall <response>` — unwanted-behaviour.
    UnwantedBehaviour,
}

impl EarsPattern {
    /// Snake-case wire label (used in SSB details + round-trip tests).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ubiquitous => "ubiquitous",
            Self::EventDriven => "event_driven",
            Self::StateDriven => "state_driven",
            Self::Optional => "optional",
            Self::UnwantedBehaviour => "unwanted_behaviour",
        }
    }
}

/// Deterministic verification status of a single requirement.
///
/// Maps 1:1 onto [`RequirementVerifiedKind`] for the SSB row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifiedStatus {
    /// Coverage cleared the floor AND every bound MR held AND >=1 bound test.
    Satisfied,
    /// No bound tests at all — the record is shown as unverifiable, NEVER
    /// silently dropped (R1: missing evidence is a hard, VISIBLE state).
    Unverified,
    /// A previously-bound test was deleted (the re-audit status flip) OR a
    /// bound MR no longer holds OR coverage fell below the floor.
    Regressed,
}

impl VerifiedStatus {
    /// Map to the SSB discriminator.
    #[must_use]
    pub fn to_ssb_kind(self) -> RequirementVerifiedKind {
        match self {
            Self::Satisfied => RequirementVerifiedKind::Satisfied,
            Self::Unverified => RequirementVerifiedKind::Unverified,
            Self::Regressed => RequirementVerifiedKind::Regressed,
        }
    }
}

// ---------------------------------------------------------------------------
// EarsRequirement record
// ---------------------------------------------------------------------------

/// A single typed EARS requirement record.
///
/// `content_hash` is the blake3 of the canonical requirement text and is the
/// drift detector / idempotent round-trip key. `bound_tests` /
/// `metamorphic_relations` are foreign-key references into the I-20 / IS-8
/// substrates — this record stores the bindings, it does NOT re-implement test
/// running or metamorphic testing (GD-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarsRequirement {
    /// blake3 hex of the canonical requirement text (drift detector).
    pub content_hash: String,
    /// The canonical (whitespace-normalised) requirement text.
    pub canonical_text: String,
    /// The EARS stereotype (enum, never a freeform string).
    pub pattern: EarsPattern,
    /// Test symbols the record claims cover this requirement.
    pub bound_tests: Vec<String>,
    /// I-20 sampled mutation score on `bound_tests` at the last audit. `None`
    /// when no audit has scored this requirement yet. Composed from
    /// `crate::mutation_budget`; NEVER fabricated (GD-2: recomputable under seed).
    pub mutation_coverage: Option<f64>,
    /// IS-8 metamorphic-relation ids bound to this requirement.
    pub metamorphic_relations: Vec<String>,
    /// Last-computed verification status (defaults to Unverified until an audit
    /// runs [`verify_requirement`]).
    pub verified_status: VerifiedStatus,
}

impl EarsRequirement {
    /// Construct a requirement from its canonical text + pattern, computing the
    /// blake3 `content_hash`. Starts Unverified (no bound tests yet).
    #[must_use]
    pub fn new(canonical_text: impl Into<String>, pattern: EarsPattern) -> Self {
        let canonical_text = normalize_requirement_text(&canonical_text.into());
        let content_hash = blake3::hash(canonical_text.as_bytes()).to_hex().to_string();
        Self {
            content_hash,
            canonical_text,
            pattern,
            bound_tests: Vec::new(),
            mutation_coverage: None,
            metamorphic_relations: Vec::new(),
            verified_status: VerifiedStatus::Unverified,
        }
    }

    /// Build a Ubiquitous-pattern requirement from an IS-15 [`IntentPair`]
    /// (the simplest EARS record: one intent, one bound test). Composes IS-15
    /// rather than re-implementing intent capture (GD-1).
    // WIRE-DARK[I-37]: IS-15 IntentPair->EARS base-case bridge; the production consumer (threading the active BabyStep's IntentPairSet into the requirement set at the PLANNING seam) lands in I-37-promotion; backing row in wire-dark-allowlist.toml ELAI_REQUIREMENTS_GATE_ENABLE.
    #[must_use]
    pub fn from_intent_pair(pair: &IntentPair) -> Self {
        let mut req = Self::new(&pair.natural_language_intent, EarsPattern::Ubiquitous);
        if !pair.concrete_test_assertion.is_empty() {
            req.bound_tests.push(pair.concrete_test_assertion.clone());
        }
        req
    }
}

/// Normalise requirement text for hashing: trim, collapse internal whitespace,
/// lowercase. Keeps the `content_hash` stable across cosmetic re-imports so the
/// spec-kit round-trip is idempotent.
#[must_use]
pub fn normalize_requirement_text(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

// ---------------------------------------------------------------------------
// EarsRequirementSet — per-task registry
// ---------------------------------------------------------------------------

/// Per-task collection of typed EARS requirements. The parallel of
/// `IntentPairSet` for the structured-requirement layer.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EarsRequirementSet {
    /// The requirements, insertion order, de-duplicated on `content_hash`.
    pub requirements: Vec<EarsRequirement>,
}

impl EarsRequirementSet {
    /// Empty set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a requirement, skipping it if an identical `content_hash` is
    /// already present (idempotent round-trip on the hash).
    pub fn insert(&mut self, req: EarsRequirement) {
        if self
            .requirements
            .iter()
            .any(|r| r.content_hash == req.content_hash)
        {
            return;
        }
        self.requirements.push(req);
    }

    /// Number of requirements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.requirements.len()
    }

    /// True when the set has no requirements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }

    /// Count of requirements NOT in the Satisfied state (the gate's block
    /// signal when the gate is enabled).
    #[must_use]
    pub fn unsatisfied_count(&self) -> usize {
        self.requirements
            .iter()
            .filter(|r| r.verified_status != VerifiedStatus::Satisfied)
            .count()
    }
}

// ---------------------------------------------------------------------------
// EARS line parsing (hand-rolled, no new dependency)
// ---------------------------------------------------------------------------

/// Classify a single EARS requirement line into its stereotype, returning the
/// pattern plus the canonical text. Returns `None` for lines that are not EARS
/// requirements (blank lines, comments, YAML/markdown frontmatter delimiters,
/// list bullets without a `shall`).
///
/// The classifier is prefix-driven over the five EARS templates and is
/// case-insensitive. A line with NO `shall` token is not a requirement and is
/// skipped (NOT silently coerced — only real `shall` lines become records).
#[must_use]
pub fn parse_ears_line(raw: &str) -> Option<(EarsPattern, String)> {
    // Strip a leading markdown/YAML list bullet or frontmatter key if present.
    let line = raw
        .trim()
        .trim_start_matches(|c| c == '-' || c == '*' || c == '+')
        .trim();
    if line.is_empty() {
        return None;
    }
    let lower = line.to_lowercase();
    // A requirement MUST contain the EARS modal "shall".
    if !lower.contains(" shall ") && !lower.ends_with(" shall") {
        return None;
    }

    // Prefix dispatch. Order matters: `if ... then` before the others; `when`
    // / `while` / `where` are mutually exclusive prefixes.
    let pattern = if lower.starts_with("when ") {
        EarsPattern::EventDriven
    } else if lower.starts_with("while ") {
        EarsPattern::StateDriven
    } else if lower.starts_with("where ") {
        EarsPattern::Optional
    } else if lower.starts_with("if ") && lower.contains(" then ") {
        EarsPattern::UnwantedBehaviour
    } else {
        // `the <system> shall <response>` (or any non-prefixed shall line).
        EarsPattern::Ubiquitous
    };
    Some((pattern, normalize_requirement_text(line)))
}

/// Parse a spec-kit / Kiro-style requirements file (one EARS requirement per
/// line; markdown bullets and frontmatter delimiters are tolerated) into typed
/// [`EarsRequirement`] records. The round-trip is idempotent on `content_hash`
/// (re-parsing the same file yields the same set).
///
/// Returns an empty `Vec` for a file with no recognisable EARS lines rather
/// than erroring — an empty spec is a valid (if vacuous) input. I/O errors are
/// surfaced as `Err` so a missing file is a hard failure at the call site, not
/// a silent empty set.
// WIRE-DARK[I-37]: spec-kit FILE import wrapper; the production consumer (the PLANNING-time spec-kit/Kiro file-import path) lands in I-37-promotion -- the in-memory parse_speckit_str IS production-wired via the task_runner seam today; backing row in wire-dark-allowlist.toml ELAI_REQUIREMENTS_GATE_ENABLE.
pub fn parse_speckit_file(path: &std::path::Path) -> std::io::Result<Vec<EarsRequirement>> {
    let contents = std::fs::read_to_string(path)?;
    Ok(parse_speckit_str(&contents))
}

/// Parse spec-kit requirement text (the in-memory form of [`parse_speckit_file`]).
#[must_use]
pub fn parse_speckit_str(contents: &str) -> Vec<EarsRequirement> {
    let mut out: Vec<EarsRequirement> = Vec::new();
    for raw in contents.lines() {
        // Skip YAML/markdown frontmatter fences outright.
        let t = raw.trim();
        if t == "---" || t.starts_with('#') {
            continue;
        }
        if let Some((pattern, canonical)) = parse_ears_line(raw) {
            let req = EarsRequirement::new(canonical, pattern);
            // De-dup on content_hash (idempotent round-trip).
            if !out.iter().any(|r| r.content_hash == req.content_hash) {
                out.push(req);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// verify_requirement — the deterministic status computation
// ---------------------------------------------------------------------------

/// Compute the [`VerifiedStatus`] for a requirement given its measured mutation
/// score, the set of test symbols that currently exist (the "live" test set),
/// and whether all of its bound metamorphic relations held.
///
/// Determinism rules (the open problem made concrete):
///   * No bound tests at all → `Unverified` (shown, never dropped — R1).
///   * A bound test is no longer present in `live_tests` → `Regressed` (the
///     re-audit status flip when a bound test is deleted).
///   * Any bound MR did not hold → `Regressed`.
///   * Coverage below `coverage_floor` → `Regressed`.
///   * Otherwise (coverage >= floor AND all MRs held AND every bound test
///     still present) → `Satisfied`.
///
/// `coverage_floor` is supplied by the caller (the I-20-measured floor in
/// production; [`DEFAULT_MUTATION_COVERAGE_FLOOR`] for the fixture) — ID-5: the
/// gate never bakes in a paper constant. `mutation_score` is `None` when I-20
/// never scored the bound tests, which (with bound tests present) is treated as
/// below-floor → `Regressed` (absent evidence is not a pass — R1).
#[must_use]
pub fn verify_requirement(
    bound_tests: &[String],
    mutation_score: Option<f64>,
    coverage_floor: f64,
    live_tests: &[String],
    all_mrs_held: bool,
) -> VerifiedStatus {
    if bound_tests.is_empty() {
        return VerifiedStatus::Unverified;
    }
    // A bound test that no longer exists in the live test set is a regression.
    let all_bound_present = bound_tests
        .iter()
        .all(|bt| live_tests.iter().any(|lt| lt == bt));
    if !all_bound_present {
        return VerifiedStatus::Regressed;
    }
    if !all_mrs_held {
        return VerifiedStatus::Regressed;
    }
    match mutation_score {
        Some(score) if score >= coverage_floor => VerifiedStatus::Satisfied,
        // Absent score OR below-floor score is a regression (R1: absent
        // evidence is not a pass).
        _ => VerifiedStatus::Regressed,
    }
}

/// Recompute and store the status on every requirement in the set, returning
/// the mutated set. `live_tests` is the set of test symbols that currently
/// exist; `coverage_floor` is the I-20-measured floor. Each requirement's
/// `mutation_coverage` field supplies the score; its `metamorphic_relations`
/// are checked against `held_mrs` (the relation ids that held this audit).
pub fn reverify_set(
    set: &mut EarsRequirementSet,
    coverage_floor: f64,
    live_tests: &[String],
    held_mrs: &[String],
) {
    for req in &mut set.requirements {
        let all_mrs_held = req
            .metamorphic_relations
            .iter()
            .all(|mr| held_mrs.iter().any(|h| h == mr));
        req.verified_status = verify_requirement(
            &req.bound_tests,
            req.mutation_coverage,
            coverage_floor,
            live_tests,
            all_mrs_held,
        );
    }
}

// ---------------------------------------------------------------------------
// SSB emit (production seam, fire-and-forget)
// ---------------------------------------------------------------------------

/// Emit one [`RequirementVerified`](crate::ssb::SsbActionType::RequirementVerified)
/// SSB row per requirement in the set. Fire-and-forget: a write failure is
/// traced but never propagated (the gate verdict / transition is unaffected by
/// SSB write failures). This is the observability leg driven from the
/// PLANNING → TESTING_FIRST seam in `task_runner`; it is INDEPENDENT of the
/// blocking [`ELAI_REQUIREMENTS_GATE_ENABLE_ENV`] flag so Unverified records are
/// always surfaced (never silently dropped — R1).
pub async fn emit_requirement_verified_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    set: &EarsRequirementSet,
) {
    let ts_ms = SystemTime::now() // entropy-allow: SSB journal ts_ms, no ClockProvider threaded into this gate
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0);

    for req in &set.requirements {
        let payload = RequirementVerifiedPayload {
            kind: req.verified_status.to_ssb_kind(),
            content_hash: req.content_hash.clone(),
            pattern: req.pattern.as_str().to_string(),
            bound_test_count: u32::try_from(req.bound_tests.len()).unwrap_or(u32::MAX),
            mutation_coverage: req.mutation_coverage,
            mr_count: u32::try_from(req.metamorphic_relations.len()).unwrap_or(u32::MAX),
            ts_ms,
        };
        let ssb_payload = SsbPayload::RequirementVerified(payload);
        match writer
            .write_payload(agent_id, superstep, ssb_payload, Vec::new())
            .await
        {
            Ok(_) => {
                debug!(
                    ears.content_hash = %req.content_hash,
                    ears.status = ?req.verified_status,
                    "emitted RequirementVerified row"
                );
            }
            Err(e) => {
                warn!(
                    ears.ssb_err = %e,
                    "RequirementVerified SSB emit failed (transition unaffected)"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// RequirementsGate — PLANNING -> TESTING_FIRST (default-OFF blocking)
// ---------------------------------------------------------------------------

/// Return `true` when the blocking requirements gate is activated via env var.
/// Default-OFF: with the flag absent the gate degrades to pass and changes no
/// production gate behaviour (only the additive SSB emit runs).
#[must_use]
pub fn requirements_gate_enabled() -> bool {
    env_gate_truthy(ELAI_REQUIREMENTS_GATE_ENABLE_ENV)
}

/// PLANNING → TESTING_FIRST: when enabled, block the transition if any
/// requirement in the active set is not Satisfied (Unverified / Regressed).
/// Co-fires with `GroundingSweepGate` + `IntentPairsGate`.
///
/// Default-degrades two ways so it never blocks a caller that has not threaded
/// the data:
///   * the env flag [`ELAI_REQUIREMENTS_GATE_ENABLE_ENV`] is absent → pass;
///   * `ctx.ears_requirements` is `None` → pass.
/// Only an explicitly-handed, non-empty set with >=1 unsatisfied requirement
/// blocks (and only when the flag is on). The status is computed by the caller
/// (via [`reverify_set`]) before the gate runs; the gate reads the stored
/// status, it does not recompute (separation of compute from gate dispatch).
pub struct RequirementsGate;

impl EvidenceGate for RequirementsGate {
    fn name(&self) -> &'static str {
        "ears_requirements"
    }

    fn transition(&self) -> (FsmState, FsmState) {
        (FsmState::Planning, FsmState::TestingFirst)
    }

    fn check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError> {
        // Default-OFF: the blocking behaviour is gated. With the flag absent the
        // gate passes (the additive SSB emit at the task_runner seam still runs).
        if !requirements_gate_enabled() {
            return Ok(());
        }
        // Back-compat: callers that have not threaded the requirement set pass
        // through (mirrors IntentPairsGate's None-degrade).
        let Some(set) = ctx.ears_requirements.as_ref() else {
            return Ok(());
        };
        let unsatisfied = set.unsatisfied_count();
        if unsatisfied > 0 {
            return Err(EvidenceError {
                gate_name: self.name(),
                missing_artifacts: vec!["ears_requirements_unsatisfied".into()],
                suggestion: format!(
                    "{unsatisfied} of {} EARS requirement(s) are not Satisfied \
                     (Unverified / Regressed). Bind tests + clear the mutation \
                     coverage floor, or mark the requirement out of scope, before \
                     PLANNING -> TESTING_FIRST.",
                    set.len()
                ),
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_five_ears_patterns() {
        let cases = [
            ("the system shall log every error", EarsPattern::Ubiquitous),
            (
                "when a request arrives the system shall respond",
                EarsPattern::EventDriven,
            ),
            (
                "while charging the system shall display the level",
                EarsPattern::StateDriven,
            ),
            (
                "where the premium feature is enabled the system shall sync",
                EarsPattern::Optional,
            ),
            (
                "if the disk is full then the system shall abort the write",
                EarsPattern::UnwantedBehaviour,
            ),
        ];
        for (line, expected) in cases {
            let (pattern, _canonical) =
                parse_ears_line(line).unwrap_or_else(|| panic!("line should parse: {line}"));
            assert_eq!(pattern, expected, "pattern mismatch for: {line}");
        }
    }

    #[test]
    fn non_requirement_lines_are_skipped() {
        assert!(parse_ears_line("").is_none());
        assert!(parse_ears_line("# a heading").is_none());
        assert!(parse_ears_line("just a description with no modal").is_none());
    }

    #[test]
    fn content_hash_is_idempotent_on_normalized_text() {
        let a = EarsRequirement::new("The  System   SHALL log", EarsPattern::Ubiquitous);
        let b = EarsRequirement::new("the system shall log", EarsPattern::Ubiquitous);
        assert_eq!(
            a.content_hash, b.content_hash,
            "normalised text must hash identically (idempotent round-trip)"
        );
    }

    #[test]
    fn speckit_round_trip_is_idempotent() {
        let spec = "\
---
# requirements
- the system shall log every error
- when a key is pressed the system shall echo it
the system shall log every error
";
        let first = parse_speckit_str(spec);
        let second = parse_speckit_str(spec);
        // Two distinct requirements (the duplicate ubiquitous line de-dups).
        assert_eq!(first.len(), 2, "expected 2 deduped requirements; got {first:?}");
        assert_eq!(first, second, "round-trip must be idempotent");
    }

    #[test]
    fn verify_no_bound_tests_is_unverified() {
        let status = verify_requirement(&[], Some(0.99), 0.8, &[], true);
        assert_eq!(status, VerifiedStatus::Unverified);
    }

    #[test]
    fn verify_deleted_bound_test_is_regressed() {
        let bound = vec!["test_alpha".to_string()];
        // live_tests does NOT contain test_alpha -> regressed.
        let status = verify_requirement(&bound, Some(0.99), 0.8, &[], true);
        assert_eq!(status, VerifiedStatus::Regressed);
    }

    #[test]
    fn verify_below_floor_is_regressed() {
        let bound = vec!["test_alpha".to_string()];
        let live = vec!["test_alpha".to_string()];
        let status = verify_requirement(&bound, Some(0.50), 0.8, &live, true);
        assert_eq!(status, VerifiedStatus::Regressed);
    }

    #[test]
    fn verify_satisfied_when_all_conditions_met() {
        let bound = vec!["test_alpha".to_string()];
        let live = vec!["test_alpha".to_string()];
        let status = verify_requirement(&bound, Some(0.90), 0.8, &live, true);
        assert_eq!(status, VerifiedStatus::Satisfied);
    }

    #[test]
    fn verify_failed_mr_is_regressed() {
        let bound = vec!["test_alpha".to_string()];
        let live = vec!["test_alpha".to_string()];
        let status = verify_requirement(&bound, Some(0.90), 0.8, &live, false);
        assert_eq!(status, VerifiedStatus::Regressed);
    }

    #[test]
    fn from_intent_pair_is_ubiquitous_with_bound_test() {
        let pair = IntentPair::new("the system shall persist state", "test_persist_state", "bs-1")
            .expect("valid intent pair");
        let req = EarsRequirement::from_intent_pair(&pair);
        assert_eq!(req.pattern, EarsPattern::Ubiquitous);
        assert_eq!(req.bound_tests, vec!["test_persist_state".to_string()]);
    }

    #[test]
    fn gate_passes_when_flag_absent() {
        // With ELAI_REQUIREMENTS_GATE_ENABLE absent the gate must pass even with
        // an unsatisfied set (default-OFF blocking).
        let mut set = EarsRequirementSet::new();
        set.insert(EarsRequirement::new("the system shall x", EarsPattern::Ubiquitous));
        let ctx = TransitionContext {
            ears_requirements: Some(set),
            ..TransitionContext::default()
        };
        // Only assert the flag-absent behaviour (the env may be set in CI; mirror
        // the I-19 wire-dark-guard test pattern).
        if !requirements_gate_enabled() {
            assert!(
                RequirementsGate.check(&ctx).is_ok(),
                "gate must pass when the enable flag is absent"
            );
        }
    }

    #[test]
    fn reverify_set_flips_statuses() {
        let mut set = EarsRequirementSet::new();
        let mut req = EarsRequirement::new("the system shall x", EarsPattern::Ubiquitous);
        req.bound_tests.push("test_x".to_string());
        req.mutation_coverage = Some(0.95);
        set.insert(req);
        // Live test set still has test_x, MR list empty (vacuously all held).
        reverify_set(&mut set, 0.8, &["test_x".to_string()], &[]);
        assert_eq!(set.requirements[0].verified_status, VerifiedStatus::Satisfied);
        // Now delete the bound test from the live set -> regressed.
        reverify_set(&mut set, 0.8, &[], &[]);
        assert_eq!(set.requirements[0].verified_status, VerifiedStatus::Regressed);
        assert_eq!(set.unsatisfied_count(), 1);
    }
}
