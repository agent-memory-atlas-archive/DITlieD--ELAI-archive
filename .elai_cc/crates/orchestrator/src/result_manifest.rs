// cross-lang-skip: rust-py-sidecar
//! H-25 (SCHEMA_VERSION 230): result manifest re-derivation METRIC_GATE (T-9).
//!
//! F-4 binds a test-PASS to an orchestrator-captured stdout_hash (a fabricated
//! PASS is detectable: a passing row with a None hash is the fabrication
//! signature -- evidence.rs:845, gate.rs:441). But NOTHING binds an empirical
//! RESULT NUMBER (a reported Sharpe / P&L / plot-derived stat) to its provenance.
//! The Replit failure class (faked tests + fabricated 4000 users) generalises: an
//! autonomous system can REPORT a number that was never produced by the pinned
//! inputs. H-25 generalises the F-4 discipline to a result number: every reported
//! number carries a [`ResultEntry`] binding it to {value, captured_output_blake3,
//! parser_expr, data_hash, code_commit, env_lock_hash, n_trials (H-17),
//! dsr_pvalue/pbo (H-18)}, and [`verify_result`] RE-PARSES the number from the
//! orchestrator-captured output and RE-HASHES the pinned inputs, asserting
//! equality. A hand-edited Sharpe whose captured-output digest does not re-derive
//! to that value -> `FabricationDetected` Block, naming the first divergent
//! result.
//!
//! # Composition (GD-1: REUSE, do NOT re-implement)
//!   - blake3 hashing convention: the per-result digest hex follows the RRA
//!     16-hex prefix convention ([`crate::rra`]). blake3 ONLY (CLAUDE.md).
//!   - first-divergence reporting: the declared and re-parsed per-result
//!     `"<name>=<value>"` sequences are wrapped as
//!     [`crate::determinism::ActionTypeSequence`] and compared via
//!     [`crate::determinism::DriftComparator::compare`], whose first-divergence
//!     loop names the first diverging result for free. The gate does NOT
//!     re-implement that loop.
//!   - the F-4 None-hash anti-fabrication discipline (evidence.rs:845 /
//!     gate.rs:441): a reported number with a missing/absent provenance hash is the
//!     fabrication signature. This gate REUSES that posture (a trivial/empty
//!     manifest, or a missing captured output, is the fabrication signature) rather
//!     than re-implementing the stdout_hash leg.
//!   - DSR/PBO/N: BOUND from the H-17 trial_count + H-18 dsr_pvalue/pbo fields the
//!     manifest carries, NEVER recomputed here (GD-1).
//!
//! # Restricted parser grammar (parser_expr SAFETY -- the load-bearing safety bit)
//! A `parser_expr` that was arbitrary code could "re-derive" any value (a malicious
//! parser hands back exactly the declared number, defeating the gate). So the
//! parser is a FIXED enum ([`ParseStrategy`]), NOT a string-eval: the only Tier-1
//! strategy is `labeled-float:<label>` -- find the first occurrence of `<label>` in
//! the captured output and parse the next float token after it. There is no
//! interpreter, no regex injection, no callback. An unrecognised `parser_expr` is a
//! hard `FabricationDetected` (a number whose re-parse cannot be performed is not
//! re-derived). The grammar is closed; adding a strategy requires a code change
//! (and is gate-frozen via the gate-manifest).
//!
//! # Tier boundary (cross-lang-skip rationale)
//! Tier-1 (THIS gate) consumes a declared manifest + a set of ORCHESTRATOR-captured
//! output byte-blobs as PLAIN DATA -- it runs no experiment. WHO produces the
//! captured output is the F-4 capture path (orchestrator-captured, not
//! agent-authored); the DEFERRED Tier-2 Python runner ENUMERATES the run's actual
//! output sink to prove COMPLETENESS-OF-ENUMERATION (no hidden / cherry-picked
//! output). That runner is RunPod-pending (cloud-only override); no stub is
//! shipped. The `// cross-lang-skip: rust-py-sidecar` header opts this file out of
//! the Rust<->Python pair drift check until the Tier-2 runner ships. The Tier-1
//! done-gate is exercisable WITHOUT a sidecar via the
//! `elai replay --verify --result-manifest` fixture path.
//!
//! # COMPLETENESS HONESTY (TD-5, honest residue -- do NOT overclaim)
//! Tier-1 is SOUND for: (a) declared-manifest RE-DERIVATION (each DECLARED entry
//! re-parses to its declared value AND re-hashes to its declared digest), and
//! (b) a NON-TRIVIAL guard (an empty / trivially-small manifest => Block;
//! `MIN_DECLARED_RESULTS`). Tier-1 does NOT prove COMPLETENESS-OF-ENUMERATION (no
//! hidden output, no cherry-picked nonempty subset) -- that produced-vs-declared
//! diff needs the deferred Tier-2 runner. in-toto/Sigstore/DVC are design-inspired
//! precedents ONLY; no non-repudiation is inherited (that is the separate shipped
//! G-8 attestation). See workflow/reasoning/H-25-decisions.md.
//!
//! # The fail-OPEN hazards this gate is built to close
//!   A. MISSING MANIFEST / TRIVIAL MANIFEST. A reported number with a 0/empty
//!      manifest is the F-4 None-hash analog (anti-sandbagging: "declare a trivial
//!      subset and pass"). This gate asserts the declared-result set is
//!      non-empty/non-trivial (`MIN_DECLARED_RESULTS`) BEFORE a `ReDerived` can be a
//!      PASS.
//!   B. MISSING CAPTURED OUTPUT. The output must be ORCHESTRATOR-captured, not
//!      agent-supplied; if it never landed (`rederived.is_none()`), the gate FAILS
//!      CLOSED (missing evidence = FAIL, R2), never a silent pass.
//!   C. HAND-EDITED NUMBER. A declared value that does not re-parse from the
//!      captured output, OR a pinned input that re-hashes to a different digest, is
//!      a `FabricationDetected` Block.
//!
//! # FAIL-CLOSED (R1/R2/W-1 -- "cannot prove the number is real" => Block)
//! Conditions that short-circuit to a hard `FabricationDetected` Block with an
//! emitted SSB row, NEVER a silent pass:
//!   1. the manifest is trivial / `< MIN_DECLARED_RESULTS` (hazard A),
//!   2. the captured-output bundle is missing (`None`) (hazard B),
//!   3. a declared value does not re-parse from the captured output (hazard C),
//!   4. a pinned input (captured_output / data / env / commit) re-hashes to a
//!      different digest (hazard C).
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::ResultManifestVerified`] row on EVERY
//! decision (ReDerived / FabricationDetected). Even a ReDerived row is evidence.
//! The captured output + the re-parsed value are the agent-un-authorable off-agent
//! artifacts (the re-parse is gate-driven over orchestrator-captured bytes). The
//! `MonkeypatchBlocked` arm is emitted by the SEEDED rule-engine path (the
//! FIN-RESMANIFEST-002 AstPattern), NOT by this pure gate -- same producer split as
//! H-24.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate; named constants (no magic numbers).
//!
//! References:
//!   - crates/orchestrator/src/rerun_equality.rs (H-23 pure-gate shape; mirrored).
//!   - crates/orchestrator/src/sim_real_parity.rs (H-24 3-arm-Kind / 2-arm-pure-gate
//!     producer split; mirrored).
//!   - crates/orchestrator/src/rra.rs (blake3 16-hex digest convention; REUSE).
//!   - crates/orchestrator/src/determinism.rs:364 (DriftComparator first-
//!     divergence -- REUSED over the declared/re-parsed result sequences).
//!   - crates/orchestrator/src/evidence.rs:845 + gate.rs:441 (the F-4 None-hash
//!     anti-fab discipline this gate generalises; do NOT re-implement).
//!   - workflow/reasoning/H-25-decisions.md.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::determinism::{ActionTypeSequence, DriftComparator, DriftSink};
use crate::ssb::{ResultManifestVerifiedKind, ResultManifestVerifiedPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// LOCKED named constants
// ---------------------------------------------------------------------------

/// Canonical rule id for the result-manifest re-derivation gate (documentary
/// parity with the sibling gates' `*_RULE_ID`). The re-parse/re-hash DECISION is
/// pure (no rules.db threshold); this id pins the two seeded sub-gates (the
/// FIN-RESMANIFEST-001 SchemaRequired manifest-shape + the FIN-RESMANIFEST-002
/// monkeypatch AstPattern) authored under
/// `crates/rules/seed/repro/result_manifest.toml`.
pub const RESULT_MANIFEST_RULE_ID: &str = "FIN-RESMANIFEST-001";

/// Companion rule id for the monkeypatch-ban AstPattern sub-gate (the seeded
/// path's `MonkeypatchBlocked` producer). Documentary parity with the
/// FIN-RESMANIFEST-001 SchemaRequired shape gate.
pub const RESULT_MANIFEST_MONKEYPATCH_RULE_ID: &str = "FIN-RESMANIFEST-002";

/// Minimum number of declared results required before a `ReDerived` verdict can be
/// a PASS (hazard A: a reported number with a 0/empty manifest is the F-4 None-hash
/// analog -- the anti-sandbagging "declare a trivial subset and pass" attack). A
/// re-derivation claim over ZERO declared results is vacuous; at least one declared
/// result must re-derive.
pub const MIN_DECLARED_RESULTS: usize = 1;

/// The RESTRICTED parser-grammar prefix for the only Tier-1 parse strategy:
/// `labeled-float:<label>` -- find the first occurrence of `<label>` in the
/// captured output and parse the next float token after it. There is NO string-eval
/// (a `parser_expr` that was arbitrary code could "re-derive" any value).
const PARSER_LABELED_FLOAT_PREFIX: &str = "labeled-float:";

/// SSB `action_type` wire tag for the off-agent artifact (matches
/// [`crate::ssb::SsbActionType::ResultManifestVerified::as_sql_str`]).
const RESULT_MANIFEST_ACTION_TYPE: &str = "result_manifest_verified";

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// One declared empirical result of a run: the reported NUMBER (kept as its exact
/// string repr so re-parse equality is byte-exact, not float-eq) bound to its
/// provenance.
///
/// The DSR/PBO/N fields are BOUND here from H-17 (`n_trials` = trial_count) and
/// H-18 (`dsr_pvalue`/`pbo`); they are NEVER recomputed by this gate (GD-1). They
/// are carried on the entry so the manifest is the single place those numbers are
/// bound to the reported value.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultEntry {
    /// Declared result name (e.g. "sharpe", "max_drawdown", "p_and_l").
    pub metric_name: String,
    /// The reported number, as its EXACT string repr (byte-exact re-parse equality,
    /// not float-eq).
    pub value: String,
    /// blake3 (16-hex prefix) the manifest DECLARES for the captured output bytes.
    pub captured_output_blake3: String,
    /// The RESTRICTED parser strategy (e.g. `labeled-float:sharpe=`); NOT
    /// arbitrary code (see [`ParseStrategy`]).
    pub parser_expr: String,
    /// blake3 (16-hex) the manifest DECLARES for the pinned input data.
    pub data_hash: String,
    /// The code commit the manifest DECLARES (a string id, compared for equality).
    pub code_commit: String,
    /// blake3 (16-hex) the manifest DECLARES for the pinned environment lock.
    pub env_lock_hash: String,
    /// Number of trials, BOUND from H-17 (`trial_count`); never recomputed (GD-1).
    pub n_trials: u32,
    /// Deflated-Sharpe p-value, BOUND from H-18; never recomputed (GD-1).
    pub dsr_pvalue: Option<f64>,
    /// Probability of backtest overfitting, BOUND from H-18; never recomputed.
    pub pbo: Option<f64>,
}

impl ResultEntry {
    /// True when this entry is trivially-empty (no name, no value, or no declared
    /// captured-output digest) -- the per-entry leg of the anti-sandbagging guard.
    fn is_trivial(&self) -> bool {
        self.metric_name.trim().is_empty()
            || self.value.trim().is_empty()
            || self.captured_output_blake3.trim().is_empty()
    }
}

/// A run's full declared result manifest. A SEPARATE typed struct (NOT an RRA
/// `RunArtifact` extension -- RRA's `.manifest` is gate-verdict hashes, the wrong
/// axis; extending it would break the frozen bundle tuple + bindings, GD-1).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResultManifest {
    /// The experiment/strategy under test (repo-relative path or id).
    pub strategy_path: String,
    /// The declared run id this manifest binds to.
    pub run_id: String,
    /// The DECLARED `ELAI_SEED` (GD-2). `None` => carried as 0 on the SSB row, never
    /// silently inferred.
    pub elai_seed: Option<u64>,
    /// The declared result entries, in declaration order.
    pub entries: Vec<ResultEntry>,
}

impl ResultManifest {
    /// Number of declared result entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when there are no declared result entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// True when there are no entries, OR every entry is trivially-empty (the
    /// structural anti-sandbagging check, hazard A).
    fn is_trivial(&self) -> bool {
        self.entries.is_empty() || self.entries.iter().all(ResultEntry::is_trivial)
    }

    /// The ordered declared `"<name>=<value>"` sequence (the declared-side input to
    /// the reused [`DriftComparator`] first-divergence over re-parsed values).
    fn declared_sequence(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| format!("{}={}", e.metric_name, e.value))
            .collect()
    }
}

/// The off-agent re-parse + re-hash bundle the verifier compares the declared
/// manifest against. `None` at the call site => the captured output never landed
/// (fail-closed, hazard B).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReDerivedInputs {
    /// The ORCHESTRATOR-captured output bytes, keyed by result name. The gate
    /// re-hashes these (must match `captured_output_blake3`) and re-parses the
    /// number out of them via the entry's restricted `parser_expr`.
    pub captured_outputs: Vec<(String, Vec<u8>)>,
    /// blake3 (16-hex) OBSERVED over the pinned input data (must match every
    /// entry's `data_hash`).
    pub data_bytes_hash: String,
    /// blake3 (16-hex) OBSERVED over the pinned environment lock (must match every
    /// entry's `env_lock_hash`).
    pub env_bytes_hash: String,
    /// The code commit OBSERVED (must match every entry's `code_commit`).
    pub code_commit_observed: String,
}

impl ReDerivedInputs {
    /// Look up the captured output bytes for a declared result name.
    fn captured_for(&self, name: &str) -> Option<&[u8]> {
        self.captured_outputs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, bytes)| bytes.as_slice())
    }
}

/// Verdict kind returned by [`verify_result`] (mirrors the inner
/// [`ResultManifestVerifiedKind`] but with only the two pure-gate-reachable arms --
/// `MonkeypatchBlocked` is emitted by the SEEDED rule-engine path, not by this pure
/// gate, same split as H-24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultVerdictKind {
    /// Every declared result re-parsed to its declared value AND every pinned input
    /// re-hashed to its declared digest, over a non-empty/non-trivial manifest: the
    /// reported results are real. PASS.
    ReDerived,
    /// A reported number did not re-derive, OR a fail-closed precondition failed
    /// (trivial manifest, missing captured output, value re-parse mismatch, input
    /// re-hash mismatch). BLOCK. Carries the first divergent result (when one
    /// exists).
    FabricationDetected,
}

/// Full verdict from [`verify_result`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultVerdict {
    /// ReDerived / FabricationDetected.
    pub kind: ResultVerdictKind,
    /// Machine-readable reason code ("re_derived" | "value_mismatch" |
    /// "hash_mismatch" | "trivial_manifest" | "missing_captured_output" |
    /// "unparseable_result").
    pub reason: String,
    /// Name of the first result that did not re-derive (empty on ReDerived /
    /// pre-comparison fail-closed).
    pub offending_metric: String,
    /// Index of the first divergent declared result (0 on ReDerived/fail-closed).
    pub first_divergence_index: u32,
    /// Number of declared results verified (0 on the empty/trivial path).
    pub declared_result_count: u32,
    /// blake3 (16-hex) over the DECLARED `"<name>=<value>"` sequence.
    pub manifest_digest: String,
    /// blake3 (16-hex) over the RE-PARSED `"<name>=<reparsed>"` sequence (empty on
    /// the missing-captured-output path).
    pub rederived_digest: String,
    /// Free-text evidence string.
    pub evidence: String,
}

/// Errors from [`verify_result`]. Note: a trivial manifest, a missing captured
/// output, a value re-parse mismatch, and an input re-hash mismatch are NOT errors
/// -- they are fail-closed `FabricationDetected` VERDICTS (each emits an SSB row).
/// Only an SSB write failure is an error here.
#[derive(Debug, Error)]
pub enum ResultManifestError {
    /// Writing the off-agent ResultManifestVerified SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// The restricted parser grammar (NO string-eval)
// ---------------------------------------------------------------------------

/// The closed set of parse strategies. `parser_expr` is matched against this FIXED
/// grammar -- there is no interpreter, no callback, no regex injection. An
/// unrecognised `parser_expr` is rejected (a number whose re-parse cannot be
/// performed is treated as not re-derived).
enum ParseStrategy<'a> {
    /// `labeled-float:<label>` -- find the FIRST occurrence of `<label>` in the
    /// captured output and parse the next float token after it.
    LabeledFloat(&'a str),
}

impl<'a> ParseStrategy<'a> {
    /// Compile a `parser_expr` into a restricted strategy, or `None` if it is not in
    /// the closed grammar (=> the result is unparseable => FabricationDetected).
    fn compile(parser_expr: &'a str) -> Option<Self> {
        parser_expr
            .strip_prefix(PARSER_LABELED_FLOAT_PREFIX)
            .map(|label| ParseStrategy::LabeledFloat(label))
    }

    /// Apply the strategy to captured output bytes, returning the re-parsed number's
    /// EXACT string token (so equality is byte-exact, not float-eq), or `None` if
    /// the label/float is absent.
    fn parse(&self, captured: &[u8]) -> Option<String> {
        let text = match std::str::from_utf8(captured) {
            Ok(t) => t,
            Err(_) => return None,
        };
        match self {
            ParseStrategy::LabeledFloat(label) => {
                if label.is_empty() {
                    return None;
                }
                let after = &text[text.find(label)? + label.len()..];
                // Take the first float-shaped token after the label (skip leading
                // whitespace / separators, then read a [-+0-9.eE] run).
                let trimmed = after.trim_start_matches(|c: char| {
                    c.is_whitespace() || c == ':' || c == '=' || c == ','
                });
                let token: String = trimmed
                    .chars()
                    .take_while(|c| {
                        c.is_ascii_digit()
                            || *c == '.'
                            || *c == '-'
                            || *c == '+'
                            || *c == 'e'
                            || *c == 'E'
                    })
                    .collect();
                if token.is_empty() {
                    None
                } else {
                    Some(token)
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main gate: verify_result
// ---------------------------------------------------------------------------

/// Verify that every DECLARED empirical result re-parses from its
/// orchestrator-captured output to its declared value AND that every pinned input
/// re-hashes to its declared digest (Tier-1 anti-fabrication METRIC_GATE). Emits
/// exactly one [`SsbPayload::ResultManifestVerified`] row on every decision (the
/// off-agent artifact, R4).
///
/// # Order of checks (FAIL-CLOSED first)
///   1. `manifest.is_trivial()` || `< MIN_DECLARED_RESULTS` => `FabricationDetected`
///      Block (reason `trivial_manifest`, hazard A): a reported number with a
///      0/empty manifest is the F-4 None-hash analog (anti-sandbagging).
///   2. `rederived.is_none()` => `FabricationDetected` Block (reason
///      `missing_captured_output`, hazard B): the captured output never landed;
///      missing evidence = FAIL (R2), never a silent pass.
///   3. per-entry RE-PARSE: for each entry, apply the restricted `parser_expr` to
///      the matching captured output and assert the re-parsed token == the declared
///      value. First mismatch => `FabricationDetected` (reason `value_mismatch` /
///      `unparseable_result`), naming the first via the REUSED
///      [`DriftComparator`].
///   4. per-entry RE-HASH: assert `captured_output_blake3` == blake3-16hex(captured
///      bytes), `data_hash` == observed, `env_lock_hash` == observed, `code_commit`
///      == observed. Any mismatch => `FabricationDetected` (reason `hash_mismatch`).
///   5. All entries re-derive over a non-trivial set => `ReDerived` PASS.
pub async fn verify_result(
    ssb: &Arc<Ssb>,
    manifest: ResultManifest,
    rederived: Option<ReDerivedInputs>,
) -> Result<ResultVerdict, ResultManifestError> {
    let seed = manifest.elai_seed.unwrap_or(0);

    // -- Step 1: FAIL-CLOSED on a trivial/empty manifest (hazard A). --------
    // A reported number with a 0/empty manifest is the F-4 None-hash analog. A
    // re-derivation claim over zero declared results is vacuous (anti-sandbagging).
    if manifest.is_trivial() || manifest.len() < MIN_DECLARED_RESULTS {
        warn!(
            strategy = %manifest.strategy_path,
            entries = manifest.len(),
            "result_manifest: trivial/empty manifest -> FabricationDetected Block \
             (fail-closed, hazard A; a number reported with a 0/empty manifest is \
             the F-4 None-hash analog, anti-sandbagging)"
        );
        let verdict = fabrication_verdict(
            "trivial_manifest",
            "The result manifest declares no (or only empty) results. A reported \
             empirical number with a 0/empty manifest is the anti-fabrication \
             signature (the F-4 None-hash analog): a re-derivation claim over zero \
             declared results is vacuous, and an empty manifest is the \
             'declare a trivial subset and pass' sandbagging attack. Fail-closed \
             Block (hazard A).",
            String::new(),
            String::new(),
            0,
        );
        emit_ssb(ssb, build_payload(&manifest, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 2: FAIL-CLOSED on a missing captured output (hazard B, R2). ---
    let rederived = match rederived {
        Some(r) => r,
        None => {
            warn!(
                strategy = %manifest.strategy_path,
                "result_manifest: captured output absent -> FabricationDetected \
                 Block (fail-closed, R2; the orchestrator-captured output never \
                 landed -- missing evidence = FAIL, not a silent pass)"
            );
            let verdict = fabrication_verdict(
                "missing_captured_output",
                "The orchestrator-captured output never landed: a reported number \
                 cannot be re-parsed from an absent capture. The output must be \
                 orchestrator-captured (not agent-supplied), so 'not found' means \
                 the run never produced it. Fail-closed Block (R2 \
                 missing-evidence=FAIL, hazard B).",
                combined_declared_digest(&manifest),
                String::new(),
                0,
            );
            emit_ssb(ssb, build_payload(&manifest, &verdict, seed)).await?;
            return Ok(verdict);
        }
    };

    // -- Step 3: per-entry RE-PARSE (REUSE DriftComparator first-divergence). -
    // Re-parse each declared number from its captured output via the RESTRICTED
    // grammar; the re-parsed `"<name>=<reparsed>"` sequence is compared against the
    // declared `"<name>=<value>"` sequence so DriftComparator names the FIRST
    // diverging result for free (GD-1). An unparseable result yields a sentinel that
    // can never equal the declared value, so it is caught as the first divergence.
    // `reparsed_strings` is the SINGLE source of truth for the re-parse (used by
    // both the comparator input and the re-derived combined digest).
    let reparsed_sequence = reparsed_strings(&manifest, &rederived);

    let seq_declared = ActionTypeSequence::new(manifest.declared_sequence());
    let seq_reparsed = ActionTypeSequence::new(reparsed_sequence.clone());
    let comparator = DriftComparator::new(seed, drift_sink_drop());
    let manifest_digest = combined_declared_digest(&manifest);
    let rederived_digest = combined_sequence_digest(&reparsed_sequence);

    if let Some(drift) = comparator.compare(&seq_declared, &seq_reparsed) {
        let idx = usize::try_from(drift.first_divergence_index).unwrap_or(usize::MAX);
        let offending_metric = manifest
            .entries
            .get(idx)
            .map(|e| e.metric_name.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        // The reason is index-specific: if THIS divergent entry's re-parse yielded
        // the `<unparseable>` sentinel (an unrecognised parser_expr or an absent
        // label), it is `unparseable_result`; otherwise the value re-parsed but did
        // not equal the declared value (a hand-edited number) => `value_mismatch`.
        let reason = match reparsed_sequence.get(idx) {
            Some(s) if s.ends_with("=<unparseable>") => "unparseable_result",
            _ => "value_mismatch",
        };
        warn!(
            strategy = %manifest.strategy_path,
            metric = %offending_metric,
            index = drift.first_divergence_index,
            reason,
            "result_manifest: a declared result did not re-parse to its declared \
             value from the captured output -> FabricationDetected Block (a \
             hand-edited number)"
        );
        let verdict = ResultVerdict {
            offending_metric,
            ..fabrication_verdict(
                reason,
                &format!(
                    "Declared result '{}' (index {}) did not re-derive: the value \
                     re-parsed from the orchestrator-captured output (via the \
                     restricted parser grammar) is not byte-equal to the declared \
                     value (a hand-edited / fabricated number). Fail-closed Block \
                     (hazard C).",
                    manifest
                        .entries
                        .get(idx)
                        .map(|e| e.metric_name.as_str())
                        .unwrap_or("<unknown>"),
                    drift.first_divergence_index
                ),
                manifest_digest,
                rederived_digest,
                drift.first_divergence_index,
            )
        };
        emit_ssb(ssb, build_payload(&manifest, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 4: per-entry RE-HASH of the pinned inputs (hazard C). ---------
    // The values re-parsed; now assert the pinned-input provenance digests match
    // what the manifest declared. Any mismatch => the inputs were not what the
    // number was produced from (a fabricated provenance binding).
    for (idx, entry) in manifest.entries.iter().enumerate() {
        let observed_capture = match rederived.captured_for(&entry.metric_name) {
            Some(bytes) => blake3_16hex(bytes),
            None => String::new(),
        };
        let mismatch = if observed_capture != entry.captured_output_blake3 {
            Some("captured_output")
        } else if rederived.data_bytes_hash != entry.data_hash {
            Some("data")
        } else if rederived.env_bytes_hash != entry.env_lock_hash {
            Some("env_lock")
        } else if rederived.code_commit_observed != entry.code_commit {
            Some("code_commit")
        } else {
            None
        };
        if let Some(field) = mismatch {
            warn!(
                strategy = %manifest.strategy_path,
                metric = %entry.metric_name,
                field,
                "result_manifest: a pinned input re-hashed to a different digest \
                 than the manifest declared -> FabricationDetected Block (a \
                 fabricated provenance binding)"
            );
            let verdict = ResultVerdict {
                offending_metric: entry.metric_name.clone(),
                ..fabrication_verdict(
                    "hash_mismatch",
                    &format!(
                        "Declared result '{}' (index {}) binds a '{}' provenance \
                         hash that does NOT match the observed re-hash: the reported \
                         number was not produced from the pinned inputs it claims. \
                         Fail-closed Block (hazard C).",
                        entry.metric_name, idx, field
                    ),
                    manifest_digest.clone(),
                    rederived_digest.clone(),
                    u32::try_from(idx).unwrap_or(u32::MAX),
                )
            };
            emit_ssb(ssb, build_payload(&manifest, &verdict, seed)).await?;
            return Ok(verdict);
        }
    }

    // -- Step 5: ReDerived PASS (all results re-parse + re-hash over a set). -
    debug!(
        strategy = %manifest.strategy_path,
        results = manifest.len(),
        "result_manifest: all declared results re-parsed to their declared values \
         and re-hashed to their declared provenance digests -> ReDerived (the \
         reported numbers are real)"
    );
    let verdict = ResultVerdict {
        kind: ResultVerdictKind::ReDerived,
        reason: "re_derived".to_string(),
        offending_metric: String::new(),
        first_divergence_index: 0,
        declared_result_count: u32::try_from(manifest.len()).unwrap_or(u32::MAX),
        manifest_digest,
        rederived_digest,
        evidence: format!(
            "ReDerived: all {} declared empirical results re-parsed (restricted \
             grammar, no string-eval) to their declared values AND re-hashed to \
             their declared provenance digests (seed={}). Tier-1 proves the \
             declared results are real + non-trivial; it does NOT prove the \
             declaration is COMPLETE (no hidden output) -- that produced-vs-declared \
             diff is the DEFERRED Tier-2 (TD-5 honest residue).",
            manifest.len(),
            seed
        ),
    };
    emit_ssb(ssb, build_payload(&manifest, &verdict, seed)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build the re-parsed `"<name>=<reparsed>"` strings (shared by the comparator
/// input and the re-derived combined digest), using the restricted grammar as the
/// SINGLE source of truth for the re-parse. An unrecognised `parser_expr` or an
/// absent label yields the `<unparseable>` sentinel, which can never equal a
/// declared float token (so the comparator flags it as a divergence).
fn reparsed_strings(manifest: &ResultManifest, rederived: &ReDerivedInputs) -> Vec<String> {
    manifest
        .entries
        .iter()
        .map(|entry| {
            let reparsed = ParseStrategy::compile(&entry.parser_expr)
                .and_then(|strategy| {
                    rederived
                        .captured_for(&entry.metric_name)
                        .and_then(|bytes| strategy.parse(bytes))
                })
                .unwrap_or_else(|| "<unparseable>".to_string());
            format!("{}={}", entry.metric_name, reparsed)
        })
        .collect()
}

/// Build a fail-closed / divergent `FabricationDetected` verdict.
fn fabrication_verdict(
    reason: &str,
    evidence: &str,
    manifest_digest: String,
    rederived_digest: String,
    first_divergence_index: u32,
) -> ResultVerdict {
    ResultVerdict {
        kind: ResultVerdictKind::FabricationDetected,
        reason: reason.to_string(),
        offending_metric: String::new(),
        first_divergence_index,
        declared_result_count: 0,
        manifest_digest,
        rederived_digest,
        evidence: evidence.to_string(),
    }
}

/// blake3 (16-hex prefix) over raw bytes, following the RRA convention. blake3
/// ONLY (CLAUDE.md HARD RULE).
fn blake3_16hex(bytes: &[u8]) -> String {
    let full = blake3::hash(bytes).to_hex();
    full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect()
}

/// The combined DECLARED-manifest digest: blake3 (16-hex) over the joined declared
/// `"<name>=<value>"` sequence (the manifest-level fingerprint surfaced on the SSB
/// row; mirrors the RRA root-over-bundle pattern). blake3 ONLY.
fn combined_declared_digest(manifest: &ResultManifest) -> String {
    combined_sequence_digest(&manifest.declared_sequence())
}

/// blake3 (16-hex) over a joined `"<name>=<x>"` sequence. blake3 ONLY.
fn combined_sequence_digest(sequence: &[String]) -> String {
    let joined = sequence.join("\n");
    blake3_16hex(joined.as_bytes())
}

/// A drop sink for the reused [`DriftComparator`]: the comparator emits its own
/// DeterminismDrift row on divergence, but the canonical off-agent artifact for
/// THIS gate is the ResultManifestVerified row. We route the comparator's emit
/// through a channel whose receiver is dropped (the send is a no-op warn, not a
/// failure), so the DriftComparator first-divergence logic is reused WITHOUT
/// emitting a duplicate DeterminismDrift row.
fn drift_sink_drop() -> DriftSink {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    drop(rx);
    DriftSink::Channel(tx)
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

async fn emit_ssb(
    ssb: &Arc<Ssb>,
    payload: ResultManifestVerifiedPayload,
) -> Result<(), ResultManifestError> {
    ssb.write_payload(
        RESULT_MANIFEST_ACTION_TYPE,
        0,
        SsbPayload::ResultManifestVerified(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| ResultManifestError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the off-agent [`ResultManifestVerifiedPayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact inputs so the verdict is recomputable (GD-2).
/// blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(
    manifest: &ResultManifest,
    verdict: &ResultVerdict,
    seed: u64,
) -> ResultManifestVerifiedPayload {
    let kind = match verdict.kind {
        ResultVerdictKind::ReDerived => ResultManifestVerifiedKind::ReDerived,
        ResultVerdictKind::FabricationDetected => ResultManifestVerifiedKind::FabricationDetected,
    };

    let declared_result_count = if verdict.declared_result_count > 0 {
        verdict.declared_result_count
    } else {
        // On ReDerived the verdict carries the count; on fail-closed paths it is 0.
        u32::try_from(manifest.len()).unwrap_or(u32::MAX)
    };

    let corpus_key = format!(
        "{}:{}:{}:{}",
        manifest.strategy_path, verdict.reason, verdict.first_divergence_index, seed
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    ResultManifestVerifiedPayload {
        kind,
        strategy_path: manifest.strategy_path.clone(),
        run_id: manifest.run_id.clone(),
        offending_metric: verdict.offending_metric.clone(),
        declared_result_count,
        manifest_digest: verdict.manifest_digest.clone(),
        rederived_digest: verdict.rederived_digest.clone(),
        verdict: verdict.reason.clone(),
        seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (re-hash + first-divergence wrap + the guards + the parser grammar)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;

    fn test_ssb() -> Arc<Ssb> {
        let dir = std::env::temp_dir().join(format!("h25-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = SsbConfig {
            db_path: dir.join(format!("ssb-{}.db", rand_suffix())),
            pool_size: Some(2),
            project_id: "h25-unit".to_string(),
            session_id: "h25-unit-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
    }

    fn rand_suffix() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now() // entropy-allow: test-tmpdir-collision-avoidance-nonce-not-replay-compared
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        nanos ^ (COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E3779B97F4A7C15))
    }

    /// A captured output blob carrying `sharpe=1.85` (and a second labeled number).
    fn captured(label_value: &[(&str, &str)]) -> Vec<u8> {
        label_value
            .iter()
            .map(|(l, v)| format!("{l}{v}"))
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes()
    }

    /// Build a clean entry whose declared value + provenance digests MATCH the
    /// supplied captured bytes / pinned hashes (a re-derivable result).
    fn clean_entry(
        name: &str,
        value: &str,
        captured_bytes: &[u8],
        data_hash: &str,
        env_hash: &str,
        commit: &str,
    ) -> ResultEntry {
        ResultEntry {
            metric_name: name.to_string(),
            value: value.to_string(),
            captured_output_blake3: blake3_16hex(captured_bytes),
            parser_expr: format!("labeled-float:{name}="),
            data_hash: data_hash.to_string(),
            code_commit: commit.to_string(),
            env_lock_hash: env_hash.to_string(),
            n_trials: 100,
            dsr_pvalue: Some(0.01),
            pbo: Some(0.12),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clean_manifest_rederives_pass() {
        let ssb = test_ssb();
        let cap = captured(&[("sharpe=", "1.85"), ("maxdd=", "0.07")]);
        let manifest = ResultManifest {
            strategy_path: "exp.py".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            entries: vec![
                clean_entry("sharpe", "1.85", &cap, "dh", "eh", "c0"),
                clean_entry("maxdd", "0.07", &cap, "dh", "eh", "c0"),
            ],
        };
        let rederived = ReDerivedInputs {
            captured_outputs: vec![
                ("sharpe".into(), cap.clone()),
                ("maxdd".into(), cap.clone()),
            ],
            data_bytes_hash: "dh".into(),
            env_bytes_hash: "eh".into(),
            code_commit_observed: "c0".into(),
        };
        let v = verify_result(&ssb, manifest, Some(rederived))
            .await
            .expect("verify");
        assert_eq!(v.kind, ResultVerdictKind::ReDerived);
        assert_eq!(v.reason, "re_derived");
        assert_eq!(v.declared_result_count, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn hand_edited_value_blocks_naming_first_result() {
        let ssb = test_ssb();
        let cap = captured(&[("sharpe=", "1.85"), ("maxdd=", "0.07")]);
        // The declared sharpe was HAND-EDITED to 2.99 (the captured output says
        // 1.85): the number does not re-derive.
        let mut sharpe = clean_entry("sharpe", "1.85", &cap, "dh", "eh", "c0");
        sharpe.value = "2.99".into();
        let manifest = ResultManifest {
            strategy_path: "exp.py".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            entries: vec![sharpe, clean_entry("maxdd", "0.07", &cap, "dh", "eh", "c0")],
        };
        let rederived = ReDerivedInputs {
            captured_outputs: vec![
                ("sharpe".into(), cap.clone()),
                ("maxdd".into(), cap.clone()),
            ],
            data_bytes_hash: "dh".into(),
            env_bytes_hash: "eh".into(),
            code_commit_observed: "c0".into(),
        };
        let v = verify_result(&ssb, manifest, Some(rederived))
            .await
            .expect("verify");
        assert_eq!(v.kind, ResultVerdictKind::FabricationDetected);
        assert_eq!(v.reason, "value_mismatch");
        assert_eq!(v.offending_metric, "sharpe");
        assert_eq!(v.first_divergence_index, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_manifest_is_fail_closed_block_anti_sandbagging() {
        let ssb = test_ssb();
        let manifest = ResultManifest {
            strategy_path: "exp.py".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            entries: vec![],
        };
        let v = verify_result(&ssb, manifest, Some(ReDerivedInputs::default()))
            .await
            .expect("verify");
        assert_eq!(v.kind, ResultVerdictKind::FabricationDetected);
        assert_eq!(v.reason, "trivial_manifest");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_captured_output_is_fail_closed_block() {
        let ssb = test_ssb();
        let cap = captured(&[("sharpe=", "1.85")]);
        let manifest = ResultManifest {
            strategy_path: "exp.py".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            entries: vec![clean_entry("sharpe", "1.85", &cap, "dh", "eh", "c0")],
        };
        // rederived = None: the captured output never landed.
        let v = verify_result(&ssb, manifest, None).await.expect("verify");
        assert_eq!(v.kind, ResultVerdictKind::FabricationDetected);
        assert_eq!(v.reason, "missing_captured_output");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn rehash_mismatch_on_data_blocks() {
        let ssb = test_ssb();
        let cap = captured(&[("sharpe=", "1.85")]);
        // The declared data_hash is "dh" but the observed data re-hash is "WRONG".
        let manifest = ResultManifest {
            strategy_path: "exp.py".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            entries: vec![clean_entry("sharpe", "1.85", &cap, "dh", "eh", "c0")],
        };
        let rederived = ReDerivedInputs {
            captured_outputs: vec![("sharpe".into(), cap.clone())],
            data_bytes_hash: "WRONG".into(),
            env_bytes_hash: "eh".into(),
            code_commit_observed: "c0".into(),
        };
        let v = verify_result(&ssb, manifest, Some(rederived))
            .await
            .expect("verify");
        assert_eq!(v.kind, ResultVerdictKind::FabricationDetected);
        assert_eq!(v.reason, "hash_mismatch");
        assert_eq!(v.offending_metric, "sharpe");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unparseable_parser_expr_is_fail_closed_block() {
        let ssb = test_ssb();
        let cap = captured(&[("sharpe=", "1.85")]);
        // A parser_expr OUTSIDE the restricted grammar (no string-eval): the number
        // cannot be re-parsed => FabricationDetected (not a silent pass).
        let mut entry = clean_entry("sharpe", "1.85", &cap, "dh", "eh", "c0");
        entry.parser_expr = "eval(open('x').read())".into();
        let manifest = ResultManifest {
            strategy_path: "exp.py".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            entries: vec![entry],
        };
        let rederived = ReDerivedInputs {
            captured_outputs: vec![("sharpe".into(), cap.clone())],
            data_bytes_hash: "dh".into(),
            env_bytes_hash: "eh".into(),
            code_commit_observed: "c0".into(),
        };
        let v = verify_result(&ssb, manifest, Some(rederived))
            .await
            .expect("verify");
        assert_eq!(v.kind, ResultVerdictKind::FabricationDetected);
        assert_eq!(v.reason, "unparseable_result");
    }

    #[test]
    fn restricted_parser_grammar_parses_labeled_float_only() {
        // The only valid strategy is labeled-float:<label>.
        let strat = ParseStrategy::compile("labeled-float:sharpe=").expect("compiles");
        let parsed = strat.parse(b"noise\nsharpe=1.85\nmore").expect("parses");
        assert_eq!(parsed, "1.85");
        // An arbitrary-code parser_expr does NOT compile (no string-eval).
        assert!(ParseStrategy::compile("os.system('rm -rf /')").is_none());
        // A label that is absent in the output yields None.
        let strat2 = ParseStrategy::compile("labeled-float:absent=").expect("compiles");
        assert!(strat2.parse(b"sharpe=1.0").is_none());
    }

    #[test]
    fn blake3_16hex_is_stable_and_prefix_len() {
        let h = blake3_16hex(b"some captured output");
        assert_eq!(h.len(), crate::rra::RRA_HEX_PREFIX_LEN);
        let h2 = blake3_16hex(b"some captured output");
        assert_eq!(h, h2);
        let h3 = blake3_16hex(b"different");
        assert_ne!(h, h3);
    }
}
