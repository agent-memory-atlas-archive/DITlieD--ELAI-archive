//! F-2b: SeCodePLT-structured static-gate insecure-rate benchmark runner.
//!
//! Drives the REAL ELAI static write-gate ([`crate::vibegate::evaluate_vibegate`])
//! over a SeCodePLT-structured CWE corpus and computes a before/after
//! insecure-rate plus the gate's recall. NO model, NO sandbox, NO live
//! inference — fully deterministic and offline (true W-5).
//!
//! # The keystone number (what this measures, honestly)
//!
//! The corpus pairs, per sample, a known-insecure (`vulnerable_code`) and a
//! known-secure (`patched_code`) completion. The candidate-completion set is
//! assembled deterministically as `vulnerable_code` + `patched_code` for every
//! sample (`n = 2 * samples`). Then:
//!
//!   * `insecure_rate_before` = (# ground-truth-insecure candidates) / n
//!     — the corpus baseline insecure fraction across ALL candidates.
//!   * `insecure_rate_after`  = (# ground-truth-insecure candidates the static
//!     gate did NOT Block) / n — the static false-negative residual that would
//!     reach disk.
//!   * `gate_recall`          = (before - after) / before — the fraction of
//!     insecure candidates the static gate caught.
//!
//! before/after are COMPUTED from `evaluate_vibegate`, never literals. The
//! contract + calibration assert `before > after` (non-degenerate) and that
//! neither is a hard-coded constant.
//!
//! # HONESTY (grounded-by-default)
//!
//! This is a STATIC-gate insecure-rate, DISTINCT from SeCodePLT's DYNAMIC
//! secure-coding rate (sandbox-executed capability + safety tests, 3-way
//! label). The dynamic 3-way metric, functional pass@1, and the live-model
//! number are a separate RunPod-gated measurement, NOT computed here. See
//! `workflow/papers/seccodeplt-interface.md` and
//! `workflow/reasoning/F-2-decisions.md`.
//!
//! # Wiring discipline
//!
//! - blake3 for the corpus content hash (anti-stale rule).
//! - thiserror for errors; tracing for diagnostics.
//! - serde for the on-disk corpus + the report.
//! - No `.unwrap()` / `.expect()` outside `#[cfg(test)]`. No silent swallows.
//! - The corpus is embedded at compile time via `include_str!` so a shipped
//!   binary is filesystem-independent (mirrors `rule_loader`).
//! - SSB emission of the `InsecureRateMeasured` beacon (SCHEMA_VERSION 201,
//!   shipped in Slice A1) is wired in a later slice; this runner computes and
//!   returns the report.

use std::collections::BTreeMap;

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::determinism::ELAI_SEED_ENV;
use crate::hook_dispatcher::HookVerdictKind;
use crate::vibegate::evaluate_vibegate;

// ---------------------------------------------------------------------------
// Embedded corpus
// ---------------------------------------------------------------------------

/// The SeCodePLT-structured CWE fixture corpus, embedded at compile time.
/// One JSON array of [`CorpusSample`] objects. Filesystem-independent.
const CORPUS_JSON: &str = include_str!("seccodeplt_fixtures/corpus.json");

/// Suite label recorded in the report + the eventual SSB beacon.
pub const SUITE_LABEL: &str = "seccodeplt_static_gate";

/// Default candidate-generation provenance id. The static gate is
/// model-independent; this records WHERE the candidate completions came from
/// (the curated corpus), not a gate input.
pub const CORPUS_MODEL_ID: &str = "seccodeplt_curated_corpus";

/// Tool name passed to `evaluate_vibegate`. Mirrors the live pre-gate caller
/// label so the gate evaluates the candidate exactly as it would on the
/// PreToolUse path.
const SCORER_TOOL_NAME: &str = "seccodeplt_bench";

// ---------------------------------------------------------------------------
// Corpus sample
// ---------------------------------------------------------------------------

/// One SeCodePLT-structured fixture sample. Mirrors the paper's per-sample
/// schema (task spec + secure/insecure gold pair + CWE tag); see
/// `workflow/papers/seccodeplt-interface.md`. We mirror the STRUCTURE, not
/// the 5.9k scale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusSample {
    /// Stable sample id (e.g. `"sqli-py-001"`).
    pub id: String,
    /// `"python"` | `"typescript"`. Used for provenance / per-language slices.
    pub lang: String,
    /// CWE identifier (e.g. `"CWE-89"`). Used for the per-CWE breakdown.
    pub cwe: String,
    /// Natural-language task description (the SeCodePLT `task_description`).
    pub task_description: String,
    /// Gold INSECURE completion (passes capability, trips the vuln). Scored
    /// as a ground-truth-insecure candidate.
    pub vulnerable_code: String,
    /// Gold SECURE completion (passes capability + safety). Scored as a
    /// ground-truth-secure candidate; must NOT be Blocked (FP guard).
    pub patched_code: String,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SecCodePltError {
    /// The embedded corpus JSON failed to deserialise. This is a build-time
    /// authoring bug, surfaced at run time.
    #[error("seccodeplt corpus malformed json: {0}")]
    MalformedCorpus(String),
    /// The corpus is empty (would make every rate degenerate / NaN).
    #[error("seccodeplt corpus is empty")]
    EmptyCorpus,
    /// `ELAI_SEED` was set but not parseable as a u64.
    #[error("seccodeplt invalid ELAI_SEED '{value}': {detail}")]
    InvalidSeed { value: String, detail: String },
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// Per-CWE slice of the run (mirrors the paper's per-CWE reporting).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerCweReport {
    pub cwe: String,
    /// Total candidates in this CWE class (insecure + secure).
    pub candidates: u32,
    /// Ground-truth-insecure candidates in this class.
    pub insecure_candidates: u32,
    /// Insecure candidates the gate Blocked (caught).
    pub insecure_blocked: u32,
    /// Insecure candidates the gate did NOT Block (residual reaching disk).
    pub insecure_residual: u32,
    /// Secure candidates the gate wrongly Blocked (false positives).
    pub secure_false_blocks: u32,
}

/// Aggregate report — one per [`SecCodePltRunner::run`] invocation.
///
/// All rates are in `[0.0, 1.0]`. `insecure_rate_before` and
/// `insecure_rate_after` are COMPUTED from `evaluate_vibegate`, never literals
/// — the contract asserts `insecure_rate_before > insecure_rate_after`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecCodePltReport {
    /// Suite label ([`SUITE_LABEL`]).
    pub suite: String,
    /// Candidate-generation provenance ([`CORPUS_MODEL_ID`]).
    pub model_id: String,
    /// Number of candidate completions evaluated (`2 * samples`).
    pub n_samples: u32,
    /// Corpus ground-truth insecure-rate across all candidates (before).
    pub insecure_rate_before: f64,
    /// Insecure-rate of the gate-approved subset (after = FN residual).
    pub insecure_rate_after: f64,
    /// Derived gate recall: `(before - after) / before`.
    pub gate_recall: f64,
    /// Deterministic RNG seed used to assemble the run (`ELAI_SEED`, default 0).
    pub seed: u64,
    /// blake3 hex of the canonical corpus bytes (provenance / staleness pin).
    pub corpus_blake3: String,
    /// Per-CWE breakdown (deterministic, alphabetical by CWE id).
    pub per_cwe: Vec<PerCweReport>,
}

// ---------------------------------------------------------------------------
// Candidate completion + label
// ---------------------------------------------------------------------------

/// A single candidate completion assembled from the corpus.
#[derive(Debug, Clone)]
struct Candidate {
    /// Source sample id (for diagnostics).
    sample_id: String,
    /// CWE class of the parent sample.
    cwe: String,
    /// File path hint handed to the gate, derived from the sample language so
    /// the per-language GrepPattern filter behaves like production. Empty for
    /// unknown languages (triggers language-agnostic evaluation).
    file_path: String,
    /// The candidate source text.
    code: String,
    /// `true` = ground-truth-insecure (from `vulnerable_code`); `false` =
    /// ground-truth-secure (from `patched_code`).
    insecure: bool,
}

/// Map a corpus `lang` label to a representative file path so the gate's
/// per-language rule filter resolves the same way it would in production.
fn file_path_for_lang(lang: &str, sample_id: &str) -> String {
    match lang {
        "python" => format!("{sample_id}.py"),
        "typescript" => format!("{sample_id}.ts"),
        // Unknown language -> empty path triggers the gate's language-agnostic
        // mode (vibegate.rs B3), matching the live task-text pre-gate path.
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// SeCodePLT static-gate insecure-rate runner. Construction parses + validates
/// the embedded corpus once; [`SecCodePltRunner::run`] is a pure, deterministic
/// scoring pass (no I/O, no async — `evaluate_vibegate` is synchronous).
#[derive(Debug)]
pub struct SecCodePltRunner {
    samples: Vec<CorpusSample>,
    corpus_blake3: String,
    seed: u64,
}

impl SecCodePltRunner {
    /// Construct from the embedded corpus, reading `ELAI_SEED` (default 0) for
    /// the recorded seed. Returns `Err` on a malformed / empty corpus or an
    /// unparseable seed.
    pub fn from_embedded() -> Result<Self, SecCodePltError> {
        let seed = read_seed()?;
        Self::from_json_with_seed(CORPUS_JSON, seed)
    }

    /// Construct from the embedded corpus with an EXPLICIT seed (overrides
    /// `ELAI_SEED`). Used by G-6's `run_safedelta` so the CLI can pin a seed
    /// without mutating the process environment. Corpus bytes are the same
    /// compile-time-embedded `CORPUS_JSON` used by [`Self::from_embedded`].
    pub fn from_embedded_with_seed(seed: u64) -> Result<Self, SecCodePltError> {
        Self::from_json_with_seed(CORPUS_JSON, seed)
    }

    /// Construct from explicit corpus JSON bytes + an explicit seed. Used by
    /// the calibration / unit tests; production uses [`Self::from_embedded`].
    pub fn from_json_with_seed(json: &str, seed: u64) -> Result<Self, SecCodePltError> {
        let samples: Vec<CorpusSample> = serde_json::from_str(json)
            .map_err(|e| SecCodePltError::MalformedCorpus(e.to_string()))?;
        if samples.is_empty() {
            return Err(SecCodePltError::EmptyCorpus);
        }
        // Hash the canonical (as-embedded) bytes so the digest is stable and
        // matches the on-disk fixture exactly.
        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let corpus_blake3 = hasher.finalize().to_hex().to_string();
        Ok(Self {
            samples,
            corpus_blake3,
            seed,
        })
    }

    /// Number of corpus samples (each contributes 2 candidates).
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// The recorded seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// blake3 hex of the canonical corpus bytes.
    #[must_use]
    pub fn corpus_blake3(&self) -> &str {
        &self.corpus_blake3
    }

    /// Borrow the parsed samples (for calibration assertions).
    #[must_use]
    pub fn samples(&self) -> &[CorpusSample] {
        &self.samples
    }

    /// Assemble the deterministic candidate-completion set: for each sample,
    /// the `vulnerable_code` (insecure) followed by the `patched_code`
    /// (secure). Order is the corpus order — the metric is order-invariant, so
    /// the seed is recorded for provenance only (no RNG dependency).
    fn candidates(&self) -> Vec<Candidate> {
        let mut out = Vec::with_capacity(self.samples.len() * 2);
        for s in &self.samples {
            let file_path = file_path_for_lang(&s.lang, &s.id);
            out.push(Candidate {
                sample_id: s.id.clone(),
                cwe: s.cwe.clone(),
                file_path: file_path.clone(),
                code: s.vulnerable_code.clone(),
                insecure: true,
            });
            out.push(Candidate {
                sample_id: s.id.clone(),
                cwe: s.cwe.clone(),
                file_path,
                code: s.patched_code.clone(),
                insecure: false,
            });
        }
        out
    }

    /// Run the static gate over every candidate and compute the report.
    ///
    /// Deterministic: same corpus => byte-identical report. The before/after
    /// rates are computed from `evaluate_vibegate` Block findings, never
    /// literals.
    #[must_use]
    pub fn run(&self) -> SecCodePltReport {
        let candidates = self.candidates();
        let n = candidates.len();
        info!(
            n_candidates = n,
            n_samples = self.samples.len(),
            "seccodeplt: starting static-gate insecure-rate run",
        );

        // Per-CWE counters: [candidates, insecure, insecure_blocked,
        // insecure_residual, secure_false_blocks].
        let mut cwe_counts: BTreeMap<String, [u32; 5]> = BTreeMap::new();

        let mut insecure_total: u32 = 0;
        let mut insecure_residual_total: u32 = 0;

        for c in &candidates {
            let blocked = gate_blocks(&c.code, &c.file_path);
            let entry = cwe_counts.entry(c.cwe.clone()).or_insert([0; 5]);
            entry[0] += 1; // candidates
            if c.insecure {
                insecure_total += 1;
                entry[1] += 1; // insecure_candidates
                if blocked {
                    entry[2] += 1; // insecure_blocked
                } else {
                    insecure_residual_total += 1;
                    entry[3] += 1; // insecure_residual
                }
            } else if blocked {
                // A secure candidate that the gate Blocked is a false positive.
                entry[4] += 1; // secure_false_blocks
                warn!(
                    sample_id = %c.sample_id,
                    cwe = %c.cwe,
                    "seccodeplt: secure (patched) candidate was Blocked (false positive)",
                );
            }
            debug!(sample_id = %c.sample_id, insecure = c.insecure, blocked, "seccodeplt: candidate scored");
        }

        let n_u32 = u32::try_from(n).unwrap_or(u32::MAX);
        let n_f = f64::from(n_u32);
        // n is guaranteed >= 2 (non-empty corpus * 2 candidates), so no /0.
        let insecure_rate_before = f64::from(insecure_total) / n_f;
        let insecure_rate_after = f64::from(insecure_residual_total) / n_f;
        let gate_recall = if insecure_rate_before > 0.0 {
            (insecure_rate_before - insecure_rate_after) / insecure_rate_before
        } else {
            0.0
        };

        let per_cwe: Vec<PerCweReport> = cwe_counts
            .into_iter()
            .map(|(cwe, c)| PerCweReport {
                cwe,
                candidates: c[0],
                insecure_candidates: c[1],
                insecure_blocked: c[2],
                insecure_residual: c[3],
                secure_false_blocks: c[4],
            })
            .collect();

        let report = SecCodePltReport {
            suite: SUITE_LABEL.to_string(),
            model_id: CORPUS_MODEL_ID.to_string(),
            n_samples: n_u32,
            insecure_rate_before,
            insecure_rate_after,
            gate_recall,
            seed: self.seed,
            corpus_blake3: self.corpus_blake3.clone(),
            per_cwe,
        };

        info!(
            insecure_rate_before = report.insecure_rate_before,
            insecure_rate_after = report.insecure_rate_after,
            gate_recall = report.gate_recall,
            "seccodeplt: run complete",
        );
        report
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run the production static gate over a candidate and return whether ANY
/// Block finding fired. This is the single scoring primitive — the keystone
/// number is derived entirely from `evaluate_vibegate`.
fn gate_blocks(code: &str, file_path: &str) -> bool {
    evaluate_vibegate(SCORER_TOOL_NAME, code, file_path)
        .iter()
        .any(|f| f.verdict == HookVerdictKind::Block)
}

/// Read `ELAI_SEED` (default 0). Errors only when the var is set but not a
/// valid u64 — an empty / unset var is the default-0 path.
fn read_seed() -> Result<u64, SecCodePltError> {
    match std::env::var(ELAI_SEED_ENV) {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim()
                .parse::<u64>()
                .map_err(|e| SecCodePltError::InvalidSeed {
                    value: raw.clone(),
                    detail: e.to_string(),
                })
        }
        _ => Ok(0u64),
    }
}

// ---------------------------------------------------------------------------
// Tests + calibration
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn runner() -> SecCodePltRunner {
        SecCodePltRunner::from_json_with_seed(CORPUS_JSON, 0).expect("embedded corpus parses")
    }

    #[test]
    fn corpus_has_at_least_40_samples() {
        let r = runner();
        assert!(
            r.sample_count() >= 40,
            "corpus must have >=40 samples, got {}",
            r.sample_count()
        );
    }

    #[test]
    fn every_sample_has_required_fields_and_distinct_codes() {
        let r = runner();
        for s in r.samples() {
            assert!(!s.id.is_empty(), "id must be non-empty");
            assert!(
                s.lang == "python" || s.lang == "typescript",
                "lang must be python|typescript, got {}",
                s.lang
            );
            assert!(s.cwe.starts_with("CWE-"), "cwe must be CWE-* , got {}", s.cwe);
            assert!(!s.task_description.is_empty(), "task_description required");
            assert!(!s.vulnerable_code.is_empty(), "vulnerable_code required");
            assert!(!s.patched_code.is_empty(), "patched_code required");
            assert_ne!(
                s.vulnerable_code, s.patched_code,
                "vulnerable and patched code must differ for {}",
                s.id
            );
        }
    }

    #[test]
    fn sample_ids_are_unique() {
        let r = runner();
        let mut ids: Vec<&str> = r.samples().iter().map(|s| s.id.as_str()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "sample ids must be unique");
    }

    /// CWE classes the F-1 static ruleset is designed to BLOCK. Every
    /// vulnerable sample in these classes MUST produce a gate Block (recall),
    /// and every patched sample in these classes MUST pass (FP=0).
    const BLOCK_CLASSES: &[&str] = &["CWE-89", "CWE-79", "CWE-327"];

    fn is_block_class(cwe: &str) -> bool {
        BLOCK_CLASSES.contains(&cwe)
    }

    /// CALIBRATION (mirrors F-10): the real `evaluate_vibegate` must FLAG every
    /// vulnerable variant whose CWE is in the F-1 ruleset (recall=100% on the
    /// block classes) and PASS every patched variant (FP=0). This grounds the
    /// ground-truth labels against the real scorer.
    #[test]
    fn calibration_block_class_recall_is_total_and_no_false_positives() {
        let r = runner();
        let mut block_class_vuln = 0u32;
        let mut block_class_vuln_flagged = 0u32;
        let mut patched_total = 0u32;

        for s in r.samples() {
            let fp = file_path_for_lang(&s.lang, &s.id);
            // Vulnerable variant.
            let vuln_blocked = gate_blocks(&s.vulnerable_code, &fp);
            if is_block_class(&s.cwe) {
                block_class_vuln += 1;
                if vuln_blocked {
                    block_class_vuln_flagged += 1;
                } else {
                    panic!(
                        "RECALL MISS: block-class vulnerable sample {} ({}) was NOT flagged",
                        s.id, s.cwe
                    );
                }
            }
            // Patched variant: must NEVER be blocked (FP guard), for ALL classes.
            patched_total += 1;
            assert!(
                !gate_blocks(&s.patched_code, &fp),
                "FALSE POSITIVE: patched sample {} ({}) was wrongly flagged",
                s.id,
                s.cwe
            );
        }

        assert!(block_class_vuln > 0, "must have block-class vulnerable samples");
        assert!(patched_total > 0, "must have patched samples");
        assert_eq!(
            block_class_vuln, block_class_vuln_flagged,
            "block-class recall must be 100%"
        );
    }

    /// The corpus MUST include residual classes the static gate is known to
    /// MISS (e.g. CWE-78 command injection, CWE-502 deserialization, CWE-862
    /// missing-authz which is WARN not BLOCK). Without these, `after` would be
    /// degenerate (0) and `gate_recall` would be a meaningless 1.0.
    #[test]
    fn corpus_contains_known_gate_miss_residual_classes() {
        let r = runner();
        let has_residual = r
            .samples()
            .iter()
            .any(|s| !is_block_class(&s.cwe) && !gate_blocks(&s.vulnerable_code, &file_path_for_lang(&s.lang, &s.id)));
        assert!(
            has_residual,
            "corpus must contain >=1 vulnerable sample the gate does NOT block (non-degenerate after)"
        );
    }

    /// Core non-degeneracy contract: `before > after`, neither is a literal
    /// (both derived from the scorer), and `gate_recall` is a sane fraction.
    #[test]
    fn report_is_non_degenerate_before_gt_after() {
        let r = runner();
        let rep = r.run();
        // n = 2 * samples; before is exactly the insecure fraction = 0.5 by
        // construction (one vuln + one patched per sample), but it is COMPUTED,
        // not asserted as a literal target.
        assert!(rep.insecure_rate_before > 0.0, "before must be > 0");
        assert!(rep.insecure_rate_after >= 0.0, "after must be >= 0");
        assert!(
            rep.insecure_rate_before > rep.insecure_rate_after,
            "before ({}) must exceed after ({}) — the gate must catch SOME insecure candidates",
            rep.insecure_rate_before,
            rep.insecure_rate_after
        );
        assert!(
            rep.insecure_rate_after > 0.0,
            "after ({}) must be > 0 — the gate must MISS some (residual classes)",
            rep.insecure_rate_after
        );
        assert!(
            rep.gate_recall > 0.0 && rep.gate_recall < 1.0,
            "gate_recall ({}) must be a non-trivial fraction (0,1)",
            rep.gate_recall
        );
    }

    /// The computed report must internally agree: gate_recall ==
    /// (before-after)/before, and the per-CWE residual sum recovers `after`.
    #[test]
    fn report_internal_consistency() {
        let r = runner();
        let rep = r.run();
        let expected_recall =
            (rep.insecure_rate_before - rep.insecure_rate_after) / rep.insecure_rate_before;
        assert!(
            (rep.gate_recall - expected_recall).abs() < 1e-12,
            "gate_recall must equal (before-after)/before"
        );

        let n = f64::from(rep.n_samples);
        let residual: u32 = rep.per_cwe.iter().map(|c| c.insecure_residual).sum();
        let after_from_cwe = f64::from(residual) / n;
        assert!(
            (rep.insecure_rate_after - after_from_cwe).abs() < 1e-12,
            "after must equal sum(per_cwe.insecure_residual)/n"
        );

        let total_candidates: u32 = rep.per_cwe.iter().map(|c| c.candidates).sum();
        assert_eq!(
            total_candidates, rep.n_samples,
            "per-CWE candidate counts must sum to n_samples (2*samples)"
        );
        // No secure candidate may be wrongly blocked anywhere.
        let fp: u32 = rep.per_cwe.iter().map(|c| c.secure_false_blocks).sum();
        assert_eq!(fp, 0, "no patched candidate may be falsely blocked");
    }

    #[test]
    fn run_is_deterministic() {
        let r = runner();
        let a = r.run();
        let b = r.run();
        assert_eq!(a, b, "two runs over the same corpus must be byte-identical");
    }

    #[test]
    fn corpus_blake3_is_64_hex_chars() {
        let r = runner();
        assert_eq!(r.corpus_blake3().len(), 64, "blake3 hex digest is 64 chars");
        assert!(r.corpus_blake3().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn empty_corpus_is_rejected() {
        match SecCodePltRunner::from_json_with_seed("[]", 0) {
            Err(SecCodePltError::EmptyCorpus) => {}
            other => panic!("expected EmptyCorpus, got {other:?}"),
        }
    }

    #[test]
    fn malformed_corpus_is_rejected() {
        match SecCodePltRunner::from_json_with_seed("{not an array", 0) {
            Err(SecCodePltError::MalformedCorpus(_)) => {}
            other => panic!("expected MalformedCorpus, got {other:?}"),
        }
    }

    #[test]
    fn seed_is_recorded_in_report() {
        let r = SecCodePltRunner::from_json_with_seed(CORPUS_JSON, 42)
            .expect("parse");
        let rep = r.run();
        assert_eq!(rep.seed, 42, "seed must round-trip into the report");
        assert_eq!(rep.suite, SUITE_LABEL);
        assert_eq!(rep.model_id, CORPUS_MODEL_ID);
    }
}
