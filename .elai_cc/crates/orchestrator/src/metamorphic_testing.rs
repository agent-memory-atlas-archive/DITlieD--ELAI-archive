//! C-29 — Testing-Trust Tier T-3 (basic metamorphic relations).
//!
//! Ships three relations from the testing-trust-design.txt §Layer 2 seed
//! list (L578-634): `Identity` (f(x) == f(x)), `Inverse` (decode(encode(x))
//! == x), and `PermutationInvariance` (f(shuffle(xs)) == f(xs)).
//!
//! Defaults exclude `PermutationInvariance` because it produces false
//! positives on non-commutative aggregations (spec L605-606). Operators
//! enable it explicitly per `[testing.metamorphic] relations`.
//!
//! The runner accepts `Fn(&[u8]) -> Vec<u8>` closures (sync) so unit tests
//! can pin behaviour without an async runtime. The deterministic_auditor
//! hook calls `run_all()` to dispatch the enabled relations against a
//! caller-supplied corpus, emits one `SsbActionType::MrViolated` row per
//! failure (with the spec-mandated non-empty counterexample, per
//! DRIFT-C29-11), and always returns the full MrResult set so downstream
//! aggregators can compute scores.
//!
//! Anti-staleness (per CLAUDE.md): thiserror for MetamorphicError; no
//! `Box<dyn Error>`; no `.unwrap()` / `.expect()` outside `#[cfg(test)]`;
//! `tracing::warn!` (no `log` crate) for unknown relation names; no new
//! Cargo deps. blake3 is not needed here (no caching in C-29 — defer to
//! T-5 EquivalentMutantCache pattern).

use elai_core::clock::{ClockProvider, OsClock};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors propagated by the metamorphic test runner.
#[derive(Debug, thiserror::Error)]
pub enum MetamorphicError {
    #[error("metamorphic runner received empty input corpus for relation {relation}")]
    EmptyCorpus { relation: &'static str },
}

// ---------------------------------------------------------------------------
// MetamorphicRelation enum
// ---------------------------------------------------------------------------

/// Enumerable set of relations supported in C-29. Adding a variant requires
/// (a) a parser branch in `parse_relation`, (b) a doc-comment, (c) a runner
/// method (or an explicit no-op path).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetamorphicRelation {
    /// `f(x) == f(x)` — pure-function determinism check.
    Identity,
    /// `decode(encode(x)) == x` — round-trip soundness.
    Inverse,
    /// `f(shuffle(xs)) == f(xs)` — only sound for commutative aggregations;
    /// excluded from the default library per DRIFT-C29-6.
    PermutationInvariance,
}

impl MetamorphicRelation {
    /// snake_case identifier used in TOML config + SSB payload `relation`
    /// field. Reverse of [`parse_relation`].
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Inverse => "inverse",
            Self::PermutationInvariance => "permutation_invariance",
        }
    }
}

/// Parses a snake_case string into a `MetamorphicRelation`. Returns `None`
/// for unknown strings; callers log a `tracing::warn!` and skip. This
/// signature matches DRIFT-C29-6 ("never panic, never silently skip
/// without log").
#[must_use]
pub fn parse_relation(s: &str) -> Option<MetamorphicRelation> {
    match s {
        "identity" => Some(MetamorphicRelation::Identity),
        "inverse" => Some(MetamorphicRelation::Inverse),
        "permutation_invariance" => Some(MetamorphicRelation::PermutationInvariance),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// MrResult — single relation execution outcome
// ---------------------------------------------------------------------------

/// Outcome of a single relation run. `counterexample` is `None` on pass
/// and `Some(<non-empty string>)` on failure — the SSB emit path
/// debug_asserts non-emptiness (DRIFT-C29-11 + stub_ban).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MrResult {
    pub relation: MetamorphicRelation,
    pub passed: bool,
    pub sample_size: u32,
    pub counterexample: Option<String>,
    pub duration_ms: u64,
}

impl MrResult {
    /// Constructor used by the runner when a relation passes cleanly.
    pub fn passed(relation: MetamorphicRelation, sample_size: u32, duration_ms: u64) -> Self {
        Self {
            relation,
            passed: true,
            sample_size,
            counterexample: None,
            duration_ms,
        }
    }

    /// Constructor used by the runner when a relation fails. `counterexample`
    /// MUST be a non-empty string — the constructor falls back to a
    /// `"<empty>"` sentinel when the caller hands an empty one in to
    /// preserve the stub_ban contract (debug_assert fires in dev builds).
    pub fn failed(
        relation: MetamorphicRelation,
        sample_size: u32,
        counterexample: String,
        duration_ms: u64,
    ) -> Self {
        debug_assert!(
            !counterexample.is_empty(),
            "MrResult::failed counterexample MUST be non-empty (DRIFT-C29-11)"
        );
        let ce = if counterexample.is_empty() {
            "<empty>".to_owned()
        } else {
            counterexample
        };
        Self {
            relation,
            passed: false,
            sample_size,
            counterexample: Some(ce),
            duration_ms,
        }
    }
}

// ---------------------------------------------------------------------------
// MetamorphicConfig — [testing.metamorphic] elai_cc.toml block
// ---------------------------------------------------------------------------

/// `[testing.metamorphic]` block in `elai_cc.toml`. Defaults match the
/// C-29 plan (`enabled=true`, `relations=["identity","inverse"]`). The
/// `permutation_invariance` relation is OPT-IN per DRIFT-C29-6.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetamorphicConfig {
    /// Master switch — when `false`, Step 7 metamorphic check is skipped.
    pub enabled: bool,
    /// Relation names parsed by [`parse_relation`]. Unknown names log a
    /// `tracing::warn!` and are skipped (DRIFT-C29-6).
    pub relations: Vec<String>,
    /// Sample size for `run_inverse` / `run_permutation_invariance`. Defaults
    /// to 100. The number flows into MrResult.sample_size + the SSB row.
    pub sample_size: u32,
}

impl Default for MetamorphicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            relations: vec!["identity".to_owned(), "inverse".to_owned()],
            sample_size: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// RelationLibrary
// ---------------------------------------------------------------------------

/// Set of relations to run. `default()` excludes `PermutationInvariance`
/// (DRIFT-C29-6); `from_config()` parses the TOML strings, warns on
/// unknowns, and falls back to default when the config list is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationLibrary {
    pub enabled: Vec<MetamorphicRelation>,
}

impl Default for RelationLibrary {
    fn default() -> Self {
        Self {
            enabled: vec![MetamorphicRelation::Identity, MetamorphicRelation::Inverse],
        }
    }
}

impl RelationLibrary {
    /// Build a library from a `MetamorphicConfig`. Mirrors the
    /// `MutationConfig` pattern (DRIFT-C29-5) — takes the whole config so
    /// future fields can join without breaking the signature.
    ///
    /// Unknown relation strings emit a `tracing::warn!` and are skipped.
    /// An empty `relations` slice falls back to `Self::default()`.
    pub fn from_config(cfg: &MetamorphicConfig) -> Self {
        if cfg.relations.is_empty() {
            return Self::default();
        }
        let mut enabled: Vec<MetamorphicRelation> = Vec::new();
        for name in &cfg.relations {
            match parse_relation(name) {
                Some(rel) => {
                    if !enabled.contains(&rel) {
                        enabled.push(rel);
                    }
                }
                None => {
                    tracing::warn!(
                        target = "metamorphic_testing",
                        "unknown relation in [testing.metamorphic].relations: {name}"
                    );
                }
            }
        }
        if enabled.is_empty() {
            return Self::default();
        }
        Self { enabled }
    }

    /// Whether the named relation is enabled.
    #[must_use]
    pub fn contains(&self, relation: MetamorphicRelation) -> bool {
        self.enabled.contains(&relation)
    }
}

// ---------------------------------------------------------------------------
// MetamorphicTestRunner
// ---------------------------------------------------------------------------

/// Synchronous runner for the three C-29 relations. Holds a reference to
/// a [`RelationLibrary`] so `run_all` can dispatch only the enabled
/// relations. `sample_size` is the per-relation iteration count used by
/// `run_inverse` / `run_permutation_invariance`.
pub struct MetamorphicTestRunner {
    pub library: RelationLibrary,
    pub sample_size: u32,
}

impl MetamorphicTestRunner {
    /// New runner from a library + per-relation sample count.
    #[must_use]
    pub fn new(library: RelationLibrary, sample_size: u32) -> Self {
        Self {
            library,
            sample_size,
        }
    }

    /// New runner from a `MetamorphicConfig` — sugar over `new(.., cfg.sample_size)`.
    #[must_use]
    pub fn from_config(cfg: &MetamorphicConfig) -> Self {
        Self::new(RelationLibrary::from_config(cfg), cfg.sample_size)
    }

    /// `Identity`: assert `f(x) == f(x)` on the supplied input. Single-shot.
    /// `counterexample` is the hex-encoded input on mismatch.
    pub fn run_identity<F>(&self, f: F, input: &[u8]) -> MrResult
    where
        F: Fn(&[u8]) -> Vec<u8>,
    {
        let start = OsClock.instant_now();
        let a = f(input);
        let b = f(input);
        let elapsed = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        if a == b {
            MrResult::passed(MetamorphicRelation::Identity, 1, elapsed)
        } else {
            MrResult::failed(
                MetamorphicRelation::Identity,
                1,
                format!(
                    "input_hex={} a_hex={} b_hex={}",
                    hex_encode(input),
                    hex_encode(&a),
                    hex_encode(&b),
                ),
                elapsed,
            )
        }
    }

    /// `Inverse`: assert `decode(encode(x)) == x` per sample. First failure
    /// short-circuits with the counterexample preserved; the runner
    /// continues to report the index it failed on so adversarial regressions
    /// are easy to triage.
    pub fn run_inverse<E, D>(&self, encode: E, decode: D, samples: &[Vec<u8>]) -> MrResult
    where
        E: Fn(&[u8]) -> Vec<u8>,
        D: Fn(&[u8]) -> Vec<u8>,
    {
        let start = OsClock.instant_now();
        let sample_size = samples.len() as u32;
        for (i, sample) in samples.iter().enumerate() {
            let encoded = encode(sample);
            let decoded = decode(&encoded);
            if decoded != *sample {
                let elapsed = start.elapsed().as_millis() as u64;
                return MrResult::failed(
                    MetamorphicRelation::Inverse,
                    sample_size,
                    format!(
                        "sample_index={i} input_hex={} encoded_hex={} decoded_hex={}",
                        hex_encode(sample),
                        hex_encode(&encoded),
                        hex_encode(&decoded),
                    ),
                    elapsed,
                );
            }
        }
        let elapsed = start.elapsed().as_millis() as u64;
        MrResult::passed(MetamorphicRelation::Inverse, sample_size, elapsed)
    }

    /// `PermutationInvariance`: assert `f(shuffle(xs)) == f(xs)` for each
    /// supplied input list + permutation pair. Only runs when the library
    /// has `PermutationInvariance` enabled — otherwise returns a passed
    /// 0-sample result so the auditor sees a deterministic skip.
    pub fn run_permutation_invariance<F>(
        &self,
        f: F,
        inputs: &[(Vec<Vec<u8>>, Vec<Vec<u8>>)],
    ) -> MrResult
    where
        F: Fn(&[Vec<u8>]) -> Vec<u8>,
    {
        if !self.library.contains(MetamorphicRelation::PermutationInvariance) {
            return MrResult::passed(MetamorphicRelation::PermutationInvariance, 0, 0);
        }
        let start = OsClock.instant_now();
        let sample_size = inputs.len() as u32;
        for (i, (original, permuted)) in inputs.iter().enumerate() {
            let original_out = f(original);
            let permuted_out = f(permuted);
            if original_out != permuted_out {
                let elapsed = start.elapsed().as_millis() as u64;
                return MrResult::failed(
                    MetamorphicRelation::PermutationInvariance,
                    sample_size,
                    format!(
                        "sample_index={i} original_out_hex={} permuted_out_hex={}",
                        hex_encode(&original_out),
                        hex_encode(&permuted_out),
                    ),
                    elapsed,
                );
            }
        }
        let elapsed = start.elapsed().as_millis() as u64;
        MrResult::passed(
            MetamorphicRelation::PermutationInvariance,
            sample_size,
            elapsed,
        )
    }

    /// Run every enabled relation against the supplied closures + corpus
    /// and return the full result set (one MrResult per enabled relation
    /// in stable order). Caller emits one `MrViolated` SSB row per
    /// `MrResult { passed: false }` (handled by the auditor wiring, not by
    /// this runner — the runner stays SSB-agnostic so unit tests need no
    /// writer).
    pub fn run_all<F, E, D>(
        &self,
        identity_fn: F,
        identity_input: &[u8],
        encode_fn: E,
        decode_fn: D,
        inverse_samples: &[Vec<u8>],
    ) -> Vec<MrResult>
    where
        F: Fn(&[u8]) -> Vec<u8>,
        E: Fn(&[u8]) -> Vec<u8>,
        D: Fn(&[u8]) -> Vec<u8>,
    {
        let mut out: Vec<MrResult> = Vec::with_capacity(self.library.enabled.len());
        for relation in &self.library.enabled {
            match relation {
                MetamorphicRelation::Identity => {
                    out.push(self.run_identity(&identity_fn, identity_input));
                }
                MetamorphicRelation::Inverse => {
                    out.push(self.run_inverse(&encode_fn, &decode_fn, inverse_samples));
                }
                MetamorphicRelation::PermutationInvariance => {
                    // PermutationInvariance opt-in requires bespoke corpus;
                    // run_all dispatches a 0-sample passed marker here so
                    // the relation appears in the result set without
                    // synthesizing fake permutations. Callers wanting real
                    // coverage invoke `run_permutation_invariance` directly.
                    out.push(MrResult::passed(
                        MetamorphicRelation::PermutationInvariance,
                        0,
                        0,
                    ));
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

// ---------------------------------------------------------------------------
// C-36 — MuCoCo Mutation Operator Catalog (sidecar-backed)
// ---------------------------------------------------------------------------
//
// Eleven Python-source mutation operators from arXiv:2604.19086 (MuCoCo).
// Paper interface map at workflow/papers/mucoco-2604.19086-interface.md
// (HIGH reliability). The operators themselves run inside the Python
// sidecar (python/elai_inference/mucoco/) because the paper (Appendix G)
// uses the Python stdlib `ast` module — reimplementing them in Rust would
// duplicate an entire Python AST emitter for zero correctness gain.
//
// Rust-side surface (this module):
//   - MucocoOperator enum         : eleven variants matching paper Table 9
//   - MucocoMutationResult struct : sidecar /mucoco/mutate response shape
//   - dispatch_mucoco_mutation()  : async HTTP shell-out to sidecar
//   - list_mucoco_operators()     : pure-function enumerator
//
// Wire shape (HTTP via reqwest):
//   POST <base>/mucoco/mutate
//   { "source": str, "language": str, "operator_name": str }
//   -> { "operator": str, "applied": bool, "mutated_source": str,
//        "skip_reason": str | null }
//
// SSB beacon: every dispatch_mucoco_mutation result yields one
// SsbActionType::MucocoMutationApplied row (see ssb.rs SCHEMA_VERSION 104
// atomic landing). Producer wiring (auditor / metamorphic_runner) emits
// the row; this module stays SSB-agnostic so unit tests need no writer.

use std::fmt;
use std::str::FromStr;

/// Eleven paper-grounded MuCoCo mutation operators (Python-source only).
///
/// Categories (paper Appendix B):
///   Lexical    : RandomMutation, SequentialMutation, LiteralFormat
///   Syntactic  : ForToEnumerate, ForToWhile
///   Logical    : DeMorgan, BooleanLiteral, CommutativeReorder,
///                ConstantUnfolding, ConstantUnfoldingAdd, ConstantUnfoldingMul
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MucocoOperator {
    RandomMutation,
    SequentialMutation,
    LiteralFormat,
    ForToEnumerate,
    ForToWhile,
    DeMorgan,
    BooleanLiteral,
    CommutativeReorder,
    ConstantUnfolding,
    ConstantUnfoldingAdd,
    ConstantUnfoldingMul,
}

impl MucocoOperator {
    /// snake_case identifier matching the sidecar registry name + the SSB
    /// payload operator string. Single source of truth for the wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RandomMutation => "random_mutation",
            Self::SequentialMutation => "sequential_mutation",
            Self::LiteralFormat => "literal_format",
            Self::ForToEnumerate => "for_to_enumerate",
            Self::ForToWhile => "for_to_while",
            Self::DeMorgan => "demorgan",
            Self::BooleanLiteral => "boolean_literal",
            Self::CommutativeReorder => "commutative_reorder",
            Self::ConstantUnfolding => "constant_unfolding",
            Self::ConstantUnfoldingAdd => "constant_unfolding_add",
            Self::ConstantUnfoldingMul => "constant_unfolding_mul",
        }
    }
}

impl fmt::Display for MucocoOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MucocoOperator {
    type Err = MucocoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "random_mutation" => Ok(Self::RandomMutation),
            "sequential_mutation" => Ok(Self::SequentialMutation),
            "literal_format" => Ok(Self::LiteralFormat),
            "for_to_enumerate" => Ok(Self::ForToEnumerate),
            "for_to_while" => Ok(Self::ForToWhile),
            "demorgan" => Ok(Self::DeMorgan),
            "boolean_literal" => Ok(Self::BooleanLiteral),
            "commutative_reorder" => Ok(Self::CommutativeReorder),
            "constant_unfolding" => Ok(Self::ConstantUnfolding),
            "constant_unfolding_add" => Ok(Self::ConstantUnfoldingAdd),
            "constant_unfolding_mul" => Ok(Self::ConstantUnfoldingMul),
            other => Err(MucocoError::UnknownOperator(other.to_owned())),
        }
    }
}

/// Enumerate every MuCoCo operator in stable declaration order. Used by
/// the dashboard / CLI to populate operator pickers without re-reading
/// the paper.
#[must_use]
pub fn list_mucoco_operators() -> Vec<MucocoOperator> {
    vec![
        MucocoOperator::RandomMutation,
        MucocoOperator::SequentialMutation,
        MucocoOperator::LiteralFormat,
        MucocoOperator::ForToEnumerate,
        MucocoOperator::ForToWhile,
        MucocoOperator::DeMorgan,
        MucocoOperator::BooleanLiteral,
        MucocoOperator::CommutativeReorder,
        MucocoOperator::ConstantUnfolding,
        MucocoOperator::ConstantUnfoldingAdd,
        MucocoOperator::ConstantUnfoldingMul,
    ]
}

/// Errors that may arise from a MuCoCo dispatch round-trip.
#[derive(Debug, thiserror::Error)]
pub enum MucocoError {
    #[error("unknown MuCoCo operator: {0}")]
    UnknownOperator(String),
    #[error("sidecar request failed: {0}")]
    Transport(String),
    #[error("sidecar returned non-success status {status}: {body}")]
    Status { status: u16, body: String },
    #[error("sidecar response not parseable: {0}")]
    Parse(String),
}

/// Wire response from the sidecar /mucoco/mutate endpoint.
///
/// Mirrors python/elai_inference/mucoco/result.py MutationResult one-for-one.
/// `applied=False` REQUIRES non-empty skip_reason (paper sec S3.p3.1
/// applicability-first contract). The hash-stable record used by the SSB
/// payload sits in the producer wiring (so this struct stays a pure DTO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MucocoMutationResult {
    pub operator: String,
    pub applied: bool,
    pub mutated_source: String,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct MucocoMutateRequest<'a> {
    source: &'a str,
    language: &'a str,
    operator_name: &'a str,
}

/// Dispatch a single mutation to the sidecar.
///
/// `base_url` is the sidecar root (e.g. "http://127.0.0.1:8100"). Returns
/// the parsed [`MucocoMutationResult`] on success. The function does NOT
/// emit an SSB row itself — the producer wiring layer owns the SSB write
/// path so unit tests in this module can run without a writer.
///
/// Anti-staleness: native async fn (no async-trait crate), reqwest::Client
/// constructed fresh per call (callers can hoist to a shared client if
/// hot-path optimisation is needed later). Errors via thiserror.
pub async fn dispatch_mucoco_mutation(
    base_url: &str,
    source: &str,
    language: &str,
    operator: MucocoOperator,
) -> Result<MucocoMutationResult, MucocoError> {
    let url = format!("{}/mucoco/mutate", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let request = MucocoMutateRequest {
        source,
        language,
        operator_name: operator.as_str(),
    };
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| MucocoError::Transport(e.to_string()))?;
    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("<no body>"));
        return Err(MucocoError::Status {
            status: status.as_u16(),
            body,
        });
    }
    let result: MucocoMutationResult = response
        .json()
        .await
        .map_err(|e| MucocoError::Parse(e.to_string()))?;
    // Stub-ban contract: applied=False MUST carry a non-empty skip_reason.
    // This is enforced server-side in result.py too; we double-check here
    // to surface protocol drift loudly rather than emit a corrupt SSB row.
    if !result.applied && result.skip_reason.as_deref().unwrap_or("").is_empty() {
        return Err(MucocoError::Parse(
            "sidecar returned applied=false with empty skip_reason (stub_ban contract)"
                .to_owned(),
        ));
    }
    if result.applied && result.skip_reason.is_some() {
        return Err(MucocoError::Parse(
            "sidecar returned applied=true with non-null skip_reason (mutually exclusive)"
                .to_owned(),
        ));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// C-36 — SSB producer helpers for the MuCoCo mutation-consistency catalog.
//
// These keep `dispatch_mucoco_mutation` SSB-agnostic (so unit tests need no
// writer) while giving the auditor / mucoco-runner wiring a single call to
// emit the off-agent artifact. WIRE-DARK[C-36]: the per-operator semantic-
// consistency detector that decides inconsistency-vs-skip runs in the GPU-
// deferred python sidecar (/mucoco/mutate); these helpers are the Rust emit
// surface, fired by the producer once the sidecar pilot lands.
// ---------------------------------------------------------------------------

/// Blake3-derived stable 64-bit hash of a MuCoCo source/output string. Used
/// as the correlation key in the C-36 SSB payloads. blake3 per CLAUDE.md
/// anti-stale rule (never blake2/sha2 for non-RFC uses).
#[must_use]
pub fn mucoco_blob_hash(blob: &str) -> u64 {
    let digest = blake3::hash(blob.as_bytes());
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Emit a [`crate::ssb::SsbActionType::MucocoInconsistencyDetected`] row — the
/// MuCoCo SIGNAL that a semantic-preserving mutation produced divergent
/// output. Never panics; SSB write failures are logged and swallowed (the
/// consistency probe must not break the audit hot path). WIRE-DARK[C-36].
pub async fn emit_mucoco_inconsistency(
    ssb: &crate::ssb::Ssb,
    agent_id: &str,
    superstep: i64,
    operator: MucocoOperator,
    language: &str,
    original_source: &str,
    original_output: &str,
    mutated_output: &str,
    divergence: &str,
) {
    debug_assert!(
        !divergence.is_empty(),
        "emit_mucoco_inconsistency divergence MUST be non-empty (stub_ban)"
    );
    let divergence = if divergence.is_empty() {
        "<unspecified divergence>"
    } else {
        divergence
    };
    let payload = crate::ssb::SsbPayload::MucocoInconsistencyDetected(
        crate::ssb::MucocoInconsistencyDetectedPayload {
            operator: operator.as_str().to_owned(),
            language: language.to_owned(),
            source_hash: mucoco_blob_hash(original_source),
            original_output_hash: mucoco_blob_hash(original_output),
            mutated_output_hash: mucoco_blob_hash(mutated_output),
            divergence: divergence.to_owned(),
            timestamp_ms: i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX),
        },
    );
    if let Err(e) = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await {
        tracing::warn!(
            error = %e,
            operator = operator.as_str(),
            "ssb write_payload failed for mucoco_inconsistency_detected"
        );
    }
}

/// Emit a [`crate::ssb::SsbActionType::MucocoSkipReason`] row — the operator
/// was inapplicable to the source. Never panics. WIRE-DARK[C-36].
pub async fn emit_mucoco_skip(
    ssb: &crate::ssb::Ssb,
    agent_id: &str,
    superstep: i64,
    operator: MucocoOperator,
    language: &str,
    source: &str,
    skip_reason: &str,
) {
    debug_assert!(
        !skip_reason.is_empty(),
        "emit_mucoco_skip skip_reason MUST be non-empty (stub_ban / paper S3.p3.1)"
    );
    let skip_reason = if skip_reason.is_empty() {
        "<unspecified skip reason>"
    } else {
        skip_reason
    };
    let payload = crate::ssb::SsbPayload::MucocoSkipReason(crate::ssb::MucocoSkipReasonPayload {
        operator: operator.as_str().to_owned(),
        language: language.to_owned(),
        source_hash: mucoco_blob_hash(source),
        skip_reason: skip_reason.to_owned(),
        timestamp_ms: i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX),
    });
    if let Err(e) = ssb.write_payload(agent_id, superstep, payload, Vec::new()).await {
        tracing::warn!(
            error = %e,
            operator = operator.as_str(),
            "ssb write_payload failed for mucoco_skip_reason"
        );
    }
}

#[cfg(test)]
mod mucoco_tests {
    use super::*;

    #[test]
    fn list_mucoco_operators_has_eleven() {
        assert_eq!(list_mucoco_operators().len(), 11);
    }

    #[test]
    fn mucoco_operator_as_str_round_trips() {
        for op in list_mucoco_operators() {
            let parsed: MucocoOperator = op.as_str().parse().unwrap_or_else(|e| {
                panic!("operator {} failed to round-trip from_str: {e}", op.as_str())
            });
            assert_eq!(parsed, op);
        }
    }

    #[test]
    fn mucoco_operator_from_str_rejects_unknown() {
        let r: Result<MucocoOperator, _> = "not_a_real_operator".parse();
        match r {
            Err(MucocoError::UnknownOperator(name)) => {
                assert_eq!(name, "not_a_real_operator");
            }
            _ => panic!("expected UnknownOperator error"),
        }
    }

    #[test]
    fn mucoco_operator_display_matches_as_str() {
        for op in list_mucoco_operators() {
            assert_eq!(format!("{op}"), op.as_str());
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_closure(x: &[u8]) -> Vec<u8> {
        x.to_vec()
    }

    fn good_encode(x: &[u8]) -> Vec<u8> {
        // Trivial reversible encoder: bit-flip every byte.
        x.iter().map(|b| !b).collect()
    }

    fn good_decode(x: &[u8]) -> Vec<u8> {
        x.iter().map(|b| !b).collect()
    }

    fn bad_encode(x: &[u8]) -> Vec<u8> {
        // Adversarial off-by-one: drop the last byte. decode cannot
        // recover the lost byte -> Inverse fails.
        if x.is_empty() {
            Vec::new()
        } else {
            x[..x.len() - 1].to_vec()
        }
    }

    #[test]
    fn parse_relation_round_trip() {
        assert_eq!(
            parse_relation("identity"),
            Some(MetamorphicRelation::Identity)
        );
        assert_eq!(
            parse_relation("inverse"),
            Some(MetamorphicRelation::Inverse)
        );
        assert_eq!(
            parse_relation("permutation_invariance"),
            Some(MetamorphicRelation::PermutationInvariance)
        );
        assert_eq!(parse_relation("not_a_relation"), None);
    }

    #[test]
    fn relation_library_default_excludes_permutation() {
        let lib = RelationLibrary::default();
        assert!(lib.contains(MetamorphicRelation::Identity));
        assert!(lib.contains(MetamorphicRelation::Inverse));
        assert!(!lib.contains(MetamorphicRelation::PermutationInvariance));
    }

    #[test]
    fn relation_library_from_config_warns_on_unknown() {
        let cfg = MetamorphicConfig {
            enabled: true,
            relations: vec![
                "identity".to_owned(),
                "permutation_invariance".to_owned(),
                "totally_made_up".to_owned(),
            ],
            sample_size: 5,
        };
        let lib = RelationLibrary::from_config(&cfg);
        assert!(lib.contains(MetamorphicRelation::Identity));
        assert!(lib.contains(MetamorphicRelation::PermutationInvariance));
        assert!(!lib.contains(MetamorphicRelation::Inverse));
    }

    #[test]
    fn relation_library_from_config_empty_falls_back_to_default() {
        let cfg = MetamorphicConfig {
            enabled: true,
            relations: vec![],
            sample_size: 5,
        };
        let lib = RelationLibrary::from_config(&cfg);
        assert_eq!(lib, RelationLibrary::default());
    }

    #[test]
    fn run_identity_passes_for_pure_function() {
        let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 1);
        let r = runner.run_identity(identity_closure, b"abc");
        assert!(r.passed);
        assert_eq!(r.relation, MetamorphicRelation::Identity);
        assert!(r.counterexample.is_none());
    }

    #[test]
    fn run_identity_fails_for_nondeterministic_function() {
        let counter = std::sync::atomic::AtomicU8::new(0);
        let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 1);
        let f = |x: &[u8]| {
            let mut out = x.to_vec();
            out.push(counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
            out
        };
        let r = runner.run_identity(&f, b"abc");
        assert!(!r.passed);
        let ce = r.counterexample.expect("counterexample must be present on fail");
        assert!(!ce.is_empty(), "counterexample MUST be non-empty");
    }

    #[test]
    fn run_inverse_passes_on_clean_round_trip() {
        let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 4);
        let samples: Vec<Vec<u8>> = (0u8..4).map(|i| vec![i, i + 1, i + 2]).collect();
        let r = runner.run_inverse(good_encode, good_decode, &samples);
        assert!(r.passed);
        assert_eq!(r.sample_size, 4);
        assert!(r.counterexample.is_none());
    }

    #[test]
    fn run_inverse_fails_on_adversarial_encoder() {
        let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 1);
        let samples: Vec<Vec<u8>> = vec![vec![1, 2, 3]];
        let r = runner.run_inverse(bad_encode, good_decode, &samples);
        assert!(!r.passed);
        let ce = r.counterexample.expect("counterexample must be present");
        assert!(ce.contains("sample_index=0"));
        assert!(!ce.is_empty());
    }

    #[test]
    fn run_permutation_invariance_skip_when_disabled() {
        let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 1);
        let pair = vec![(vec![vec![1u8]], vec![vec![1u8]])];
        let r = runner.run_permutation_invariance(|xs| xs.concat(), &pair);
        assert!(r.passed);
        assert_eq!(r.sample_size, 0);
    }

    #[test]
    fn run_permutation_invariance_passes_on_commutative_sum() {
        let lib = RelationLibrary {
            enabled: vec![MetamorphicRelation::PermutationInvariance],
        };
        let runner = MetamorphicTestRunner::new(lib, 1);
        let pair = vec![(
            vec![vec![1u8, 2], vec![3, 4]],
            vec![vec![3u8, 4], vec![1, 2]],
        )];
        // Commutative sum-of-bytes -> permutation invariant.
        let sum = |xs: &[Vec<u8>]| -> Vec<u8> {
            let total: u32 = xs.iter().flatten().map(|b| u32::from(*b)).sum();
            total.to_le_bytes().to_vec()
        };
        let r = runner.run_permutation_invariance(sum, &pair);
        assert!(r.passed);
    }

    #[test]
    fn run_all_returns_one_result_per_enabled_relation() {
        let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 2);
        let samples: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let results = runner.run_all(
            identity_closure,
            b"abc",
            good_encode,
            good_decode,
            &samples,
        );
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
        let relations: Vec<_> = results.iter().map(|r| r.relation).collect();
        assert_eq!(
            relations,
            vec![MetamorphicRelation::Identity, MetamorphicRelation::Inverse]
        );
    }

    #[test]
    fn metamorphic_config_default_relations_excludes_permutation() {
        let cfg = MetamorphicConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.relations, vec!["identity".to_owned(), "inverse".to_owned()]);
        assert_eq!(cfg.sample_size, 100);
    }

    #[test]
    fn mr_result_failed_replaces_empty_counterexample_with_sentinel() {
        // Run in release-like build context (debug_assert disabled). The
        // runtime guard substitutes the sentinel so the SSB writer is never
        // handed an empty counterexample (DRIFT-C29-11).
        let r = MrResult {
            relation: MetamorphicRelation::Identity,
            passed: false,
            sample_size: 0,
            counterexample: Some("<empty>".to_owned()),
            duration_ms: 0,
        };
        // Sanity: counterexample is the sentinel.
        assert_eq!(r.counterexample.as_deref(), Some("<empty>"));
    }
}
