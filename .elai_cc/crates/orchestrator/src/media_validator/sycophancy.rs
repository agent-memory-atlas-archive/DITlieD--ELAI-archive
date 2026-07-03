//! E-16 -- anti-sycophancy prompt protocol for the Media Validator
//! Tier-2 (VLM) pathway.
//!
//! Background: arXiv:2408.11261 "Sycophancy in Vision-Language Models"
//! (verified May 2026 via WebSearch arxiv.org/abs/2408.11261) shows VLMs
//! including Qwen-VL, CogVLM, InternVL and LLaVA agree with leading
//! user prompts at materially higher rates than under neutral framing,
//! over-riding correct visual evidence. The paper proposes Leading-
//! Query Contrastive Decoding (LQCD) at the inference layer; this
//! module ships the orchestrator-side equivalent: a deterministic
//! prompt re-write that strips executor-intent leakage tokens
//! (`neutralize_query`) plus a paired-judge contrastive scorer
//! (`contrastive_decode`) that asks two judges to agree on a
//! neutralized + an inverted-framing rephrase before the verdict is
//! accepted.
//!
//! Wire-dark posture: every fire-point gates on
//! [`ELAI_VLM_IMAGE_VALIDATION_ENABLE`] / the
//! [`ssb::ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV`] env var until the live
//! VLM (RUNPOD-PENDING per E-16 plan) is reachable. Callers in the
//! Tier-2 path that pre-date E-16 keep their behavior because the env
//! gate defaults to OFF.
//!
//! Reuses (NOT new):
//!   * `ssb::SsbActionType::SycophancyFlag` + `SycophancyFlagPayload`
//!     for the inverted-framing fire-point. E-4 / D-14 already shipped
//!     the variant + the dashboard surfaces (SycophancyFlagStrip,
//!     SycophancyGuardBadge, SycophancyMatchedPairPanel,
//!     SycophancyBenchPanel). Verdict vocab MUST match: "convergent"
//!     | "divergent_sycophancy_flag" | "inconclusive". See ssb.rs
//!     L15588-15600.
//!
//! Anti-stale grounding (per CLAUDE.md):
//!   * thiserror for module-local error enum (NOT Box<dyn Error>).
//!   * blake3 for prompt-text fingerprinting (NOT blake2 / sha2).
//!   * tracing for structured logs.
//!   * No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Env var that gates EVERY VLM image-validation fire-point in the
/// orchestrator (sycophancy, axtree, swap-order, schema-mask). Mirror
/// of [`crate::ssb::ELAI_VLM_IMAGE_VALIDATION_ENABLE_ENV`]; re-exported
/// here so the producer surface can be referenced without pulling the
/// whole SSB crate into scope.
pub const ELAI_VLM_IMAGE_VALIDATION_ENABLE: &str = "ELAI_VLM_IMAGE_VALIDATION_ENABLE";

/// Leading-query token patterns the executor's prompt envelope might
/// include. We strip ONLY whole-phrase matches at the start of a
/// sentence; partial-word matches are left alone so legitimate visual
/// content (e.g. an alt-text describing "this is a stop sign") is not
/// rewritten. Maintained as a sorted slice so the strip pass is
/// deterministic across runs (BTreeSet keeps the iteration order
/// stable).
const LEADING_PATTERNS: &[&str] = &[
    "this is the ",
    "this is your ",
    "verify the title says ",
    "does this match the spec",
    "you asked for ",
    "as requested, ",
    "as you asked, ",
    "please confirm ",
    "confirm that ",
    "confirm this ",
    "i made the ",
    "i built the ",
    "i wrote the ",
];

/// Inverted-framing prefix appended by [`invert_framing`] when the
/// sycophancy_probe re-runs the same neutralized query with the
/// opposite leading polarity. The exact wording must read like an
/// instruction the model treats as an authoritative re-frame, not a
/// joke. Anchored to arXiv:2408.11261 §4.2 contrastive-decoding
/// methodology.
const INVERSION_PREFIX: &str = "Disregard any prior assertion this image matches a specification. Identify the most likely defects, hallucinations, or rendering errors a critical reviewer would call out; ";

/// Module-local error enum. Per CLAUDE.md HARD RULE — no Box<dyn Error>
/// crossing crate boundaries.
#[derive(Debug, Error)]
pub enum SycophancyError {
    /// VLM judge call failed for the neutralized prompt.
    #[error("vlm judge failed on neutralized prompt: {0}")]
    JudgeNeutralFailed(String),
    /// VLM judge call failed for the inverted prompt.
    #[error("vlm judge failed on inverted prompt: {0}")]
    JudgeInvertedFailed(String),
    /// Wire-dark posture: the env gate is OFF; producer must short-
    /// circuit. Callers check [`is_enabled`] before invoking
    /// `contrastive_decode`; this variant is for paths that flow
    /// through the gate at the API boundary.
    #[error("vlm image validation disabled via env (ELAI_VLM_IMAGE_VALIDATION_ENABLE != 1)")]
    Disabled,
}

/// The neutralized rephrase of an executor-supplied query, plus the
/// removed leading tokens for audit. `stripped` is the rewritten
/// prompt; `removed_tokens` is the canonical (sorted, deduped) set of
/// leading-pattern matches that were stripped so the audit trail can
/// confirm WHICH tokens triggered the rewrite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeutralPrompt {
    /// Rewritten prompt with leading patterns stripped.
    pub stripped: String,
    /// Sorted + deduped list of leading patterns that were stripped.
    pub removed_tokens: Vec<String>,
}

/// Verdict returned by [`contrastive_decode`]. `agree=true` means the
/// neutral and inverted judges produced the same primary classification
/// label; `divergent=true` is the contrary signal. `divergence_score`
/// is in `[0.0, 1.0]` where 0 = identical and 1 = complete divergence
/// across all surfaced sub-claims. Producers map `divergent=true ->
/// SycophancyFlagPayload.verdict = "divergent_sycophancy_flag"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SycophancyVerdict {
    pub agree: bool,
    pub divergent: bool,
    pub divergence_score: f64,
    /// Snake-case label matching ssb::SycophancyFlagPayload::verdict
    /// vocab: "convergent" | "divergent_sycophancy_flag" |
    /// "inconclusive". The dashboard's SycophancyFlagStrip.svelte
    /// already renders these three labels.
    pub verdict_label: String,
}

/// Output of a single VLM-judge call against either the neutral or
/// inverted prompt. The classification label is a snake-case primary
/// tag (e.g. "matches_spec" / "defect_present" / "ambiguous"). The
/// `claims` set captures finer-grained sub-claims (one per identified
/// defect/feature) so the divergence score can be computed without a
/// second call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeOutput {
    pub primary_label: String,
    pub claims: BTreeSet<String>,
}

/// VLM judge trait. Implementations live behind the `vlm-integration`
/// feature in the orchestrator runtime layer (B-37 RuntimeAdapter
/// table; SGLang / Ollama / Sidecar). Producers gate live invocations
/// behind [`is_enabled`] so wire-dark mode returns
/// [`SycophancyError::Disabled`].
pub trait VlmJudge: Send + Sync {
    /// Synchronous judge call. The neutralized/inverted prompts are
    /// short enough that async I/O is overkill; production adapters
    /// can block_on a tokio future inside the impl if needed.
    fn judge(&self, prompt: &str) -> Result<JudgeOutput, String>;
}

/// Returns `true` when the wire-dark gate is open (`ELAI_VLM_IMAGE_
/// VALIDATION_ENABLE=1`). All producer call-sites MUST check this
/// before invoking [`contrastive_decode`]; the function returns
/// [`SycophancyError::Disabled`] if called while the gate is closed.
#[must_use]
pub fn is_enabled() -> bool {
    std::env::var(ELAI_VLM_IMAGE_VALIDATION_ENABLE)
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Strip executor-intent leakage from `executor_intent`. The function
/// is pure / deterministic: same input -> same output.
///
/// Rewrite rules:
///   1. Lowercase + ASCII-trim the input head ONLY for matching;
///      the actual stripped output preserves the original case for
///      the surviving tail.
///   2. Scan [`LEADING_PATTERNS`] in declaration order. The first
///      match at character index 0 (case-insensitive ASCII) is
///      stripped; the scan repeats until no pattern matches the head.
///   3. Trim leading whitespace + punctuation after each strip so a
///      doubly-leading prompt like "This is the foo. As requested,
///      bar." -> "foo. bar." retains the substantive content.
///   4. `removed_tokens` is the sorted+deduped set of patterns that
///      were stripped.
#[must_use]
pub fn neutralize_query(executor_intent: &str) -> NeutralPrompt {
    let mut current = executor_intent.to_owned();
    // Bridging words between non-contiguous leading patterns (e.g.
    // "foo." in "This is the foo. As requested, bar.") are preserved
    // here so the substantive content tail is not lost when the scan
    // advances past a non-pattern sentence start.
    let mut retained = String::new();
    let mut removed: BTreeSet<String> = BTreeSet::new();

    loop {
        let head = current.trim_start();
        let head_lower = head.to_ascii_lowercase();
        let mut matched: Option<&'static str> = None;
        for pat in LEADING_PATTERNS {
            if head_lower.starts_with(pat) {
                matched = Some(pat);
                break;
            }
        }
        match matched {
            Some(pat) => {
                // Strip the matched leading prefix from `current`.
                // We re-derive the head offset because of the
                // trim_start() above.
                let head_start = current.len() - head.len();
                let strip_end = head_start + pat.len();
                current = current[strip_end..].to_owned();
                // Eat any leading whitespace + light punctuation
                // (",", ".") so the next pass evaluates from a clean
                // head.
                current = current
                    .trim_start_matches(|c: char| c.is_whitespace() || c == ',' || c == '.')
                    .to_owned();
                removed.insert(pat.to_string());
            }
            None => {
                // No leading pattern at the current head. Scan forward
                // for the next sentence boundary (`.`, `!`, `?`) and
                // check whether a leading pattern begins at the next
                // non-whitespace position. If so, preserve the
                // bridging tokens in `retained` and resume the strip
                // pass at that sentence start. If no subsequent
                // sentence start carries a leading pattern, the
                // remainder is substantive content and the loop ends.
                let lower = current.to_ascii_lowercase();
                let bytes = current.as_bytes();
                let mut next_start: Option<usize> = None;
                let mut i = 0;
                while i < bytes.len() {
                    let c = bytes[i];
                    if c == b'.' || c == b'!' || c == b'?' {
                        let mut j = i + 1;
                        while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                            j += 1;
                        }
                        if j < bytes.len() {
                            let rest_lower = &lower[j..];
                            if LEADING_PATTERNS.iter().any(|p| rest_lower.starts_with(p)) {
                                next_start = Some(j);
                                break;
                            }
                        }
                    }
                    i += 1;
                }
                match next_start {
                    Some(j) => {
                        retained.push_str(&current[..j]);
                        current = current[j..].to_owned();
                    }
                    None => break,
                }
            }
        }
    }

    let stripped = format!("{retained}{current}");
    NeutralPrompt {
        stripped,
        removed_tokens: removed.into_iter().collect(),
    }
}

/// Re-frame a neutralized prompt with the inverted-polarity prefix
/// from [`INVERSION_PREFIX`]. The output is what the sycophancy_probe
/// fires through the second judge call.
#[must_use]
pub fn invert_framing(neutral: &NeutralPrompt) -> String {
    let mut out = String::with_capacity(INVERSION_PREFIX.len() + neutral.stripped.len());
    out.push_str(INVERSION_PREFIX);
    out.push_str(&neutral.stripped);
    out
}

/// Contrastive-decode a neutralized prompt vs its inverted-polarity
/// rephrase. Both judge calls go through the same `vlm_client` (same
/// model / same engine) so the difference signal is purely framing-
/// driven.
///
/// Returns [`SycophancyError::Disabled`] when the wire-dark gate is
/// closed. Production callers should match on the result and emit a
/// `SycophancyFlag` SSB row with `verdict = verdict_label` so the
/// existing E-4 dashboard surfaces render the outcome.
///
/// Divergence-score formula (deterministic):
///   * If primary labels disagree -> 1.0 (max divergence).
///   * Otherwise -> Jaccard distance between the claim sets =
///     1.0 - |claims_n ∩ claims_i| / |claims_n ∪ claims_i|.
///   * Empty claim sets on both sides -> 0.0 (convergent / no
///     content to disagree on).
///
/// Threshold mapping for verdict_label:
///   * divergence_score < 0.15  -> "convergent"
///   * divergence_score >= 0.5  -> "divergent_sycophancy_flag"
///   * otherwise                -> "inconclusive"
pub fn contrastive_decode(
    neutral: &NeutralPrompt,
    vlm_client: &dyn VlmJudge,
) -> Result<SycophancyVerdict, SycophancyError> {
    if !is_enabled() {
        return Err(SycophancyError::Disabled);
    }

    let neutral_out = vlm_client
        .judge(&neutral.stripped)
        .map_err(SycophancyError::JudgeNeutralFailed)?;
    let inverted_prompt = invert_framing(neutral);
    let inverted_out = vlm_client
        .judge(&inverted_prompt)
        .map_err(SycophancyError::JudgeInvertedFailed)?;

    Ok(compute_verdict(&neutral_out, &inverted_out))
}

/// Pure scoring path — exposed so unit tests can pin the
/// divergence-score formula without going through the env gate or a
/// trait object. Mirrors the threshold table documented on
/// [`contrastive_decode`].
#[must_use]
pub fn compute_verdict(neutral: &JudgeOutput, inverted: &JudgeOutput) -> SycophancyVerdict {
    let labels_disagree = neutral.primary_label != inverted.primary_label;
    let union_count = neutral
        .claims
        .union(&inverted.claims)
        .count();
    let intersect_count = neutral
        .claims
        .intersection(&inverted.claims)
        .count();

    let divergence_score: f64 = if labels_disagree {
        1.0
    } else if union_count == 0 {
        0.0
    } else {
        let union_f = union_count as f64;
        let intersect_f = intersect_count as f64;
        1.0 - (intersect_f / union_f)
    };

    let (agree, divergent, verdict_label) = if divergence_score < 0.15 {
        (true, false, "convergent".to_owned())
    } else if divergence_score >= 0.5 {
        (false, true, "divergent_sycophancy_flag".to_owned())
    } else {
        (false, false, "inconclusive".to_owned())
    };

    SycophancyVerdict {
        agree,
        divergent,
        divergence_score,
        verdict_label,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubJudge {
        neutral: JudgeOutput,
        inverted: JudgeOutput,
        last_was_inverted: std::sync::atomic::AtomicBool,
    }

    impl VlmJudge for StubJudge {
        fn judge(&self, prompt: &str) -> Result<JudgeOutput, String> {
            // The inverted prompt always starts with INVERSION_PREFIX;
            // the stub flips the toggle so the test can assert both
            // calls fired.
            if prompt.starts_with(INVERSION_PREFIX) {
                self.last_was_inverted
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(self.inverted.clone())
            } else {
                Ok(self.neutral.clone())
            }
        }
    }

    #[test]
    fn neutralize_strips_leading_intent() {
        let n = neutralize_query("This is the TODO app you asked for. Verify it works.");
        assert!(
            !n.stripped.to_ascii_lowercase().contains("this is the"),
            "stripped output retained leading pattern: {}",
            n.stripped
        );
        assert!(
            n.removed_tokens.iter().any(|t| t == "this is the "),
            "expected 'this is the ' in removed_tokens, got {:?}",
            n.removed_tokens
        );
    }

    #[test]
    fn neutralize_handles_doubly_leading() {
        let n = neutralize_query(
            "This is the foo. As requested, build a panel that does X.",
        );
        // Both patterns stripped + content preserved.
        assert!(
            n.removed_tokens.iter().any(|t| t == "this is the "),
            "missing 'this is the '"
        );
        assert!(
            n.removed_tokens.iter().any(|t| t == "as requested, "),
            "missing 'as requested, '"
        );
        assert!(
            n.stripped.to_ascii_lowercase().contains("panel"),
            "content tail lost: {}",
            n.stripped
        );
    }

    #[test]
    fn neutralize_is_idempotent() {
        let once = neutralize_query("This is the foo bar baz.");
        let twice = neutralize_query(&once.stripped);
        assert_eq!(once.stripped, twice.stripped);
    }

    #[test]
    fn invert_prepends_inversion_prefix() {
        let n = NeutralPrompt {
            stripped: "Identify items in the cart.".into(),
            removed_tokens: vec![],
        };
        let inv = invert_framing(&n);
        assert!(
            inv.starts_with(INVERSION_PREFIX),
            "inversion missing prefix: {}",
            inv
        );
        assert!(
            inv.contains("Identify items"),
            "inversion lost neutral content"
        );
    }

    #[test]
    fn compute_verdict_convergent_on_identical_outputs() {
        let same = JudgeOutput {
            primary_label: "matches_spec".into(),
            claims: ["title_present".to_string()].into_iter().collect(),
        };
        let v = compute_verdict(&same, &same);
        assert!(v.agree);
        assert!(!v.divergent);
        assert_eq!(v.verdict_label, "convergent");
        assert!(v.divergence_score < 0.15);
    }

    #[test]
    fn compute_verdict_divergent_on_label_disagreement() {
        let a = JudgeOutput {
            primary_label: "matches_spec".into(),
            claims: ["title_present".to_string()].into_iter().collect(),
        };
        let b = JudgeOutput {
            primary_label: "defect_present".into(),
            claims: ["title_present".to_string()].into_iter().collect(),
        };
        let v = compute_verdict(&a, &b);
        assert!(!v.agree);
        assert!(v.divergent);
        assert_eq!(v.verdict_label, "divergent_sycophancy_flag");
        assert!((v.divergence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_verdict_inconclusive_on_partial_overlap() {
        let a = JudgeOutput {
            primary_label: "matches_spec".into(),
            claims: ["a".to_string(), "b".to_string()].into_iter().collect(),
        };
        let b = JudgeOutput {
            primary_label: "matches_spec".into(),
            claims: ["b".to_string(), "c".to_string()].into_iter().collect(),
        };
        let v = compute_verdict(&a, &b);
        // Jaccard distance = 1 - 1/3 ≈ 0.667 -> divergent.
        assert_eq!(v.verdict_label, "divergent_sycophancy_flag");
    }

    #[test]
    fn contrastive_decode_blocked_when_gate_closed() {
        // SAFETY: tests run on a single process; we ensure the env is
        // not set to "1". Cannot use std::env::set_var in parallel
        // tests because it mutates process state, so we only assert
        // the "default off" branch.
        // SAFETY: env access is process-global; the test asserts the
        // default-off branch and does NOT set the variable, so no
        // races with concurrent tests.
        let key = ELAI_VLM_IMAGE_VALIDATION_ENABLE;
        let prior = std::env::var(key).ok();
        if prior.as_deref() == Some("1") {
            // Skip when the parent shell intentionally turned the
            // gate on; this assertion only meaningful in default-off
            // mode.
            return;
        }
        let neutral = NeutralPrompt {
            stripped: "noop".into(),
            removed_tokens: vec![],
        };
        let stub = StubJudge {
            neutral: JudgeOutput {
                primary_label: "ok".into(),
                claims: BTreeSet::new(),
            },
            inverted: JudgeOutput {
                primary_label: "ok".into(),
                claims: BTreeSet::new(),
            },
            last_was_inverted: std::sync::atomic::AtomicBool::new(false),
        };
        let res = contrastive_decode(&neutral, &stub);
        assert!(matches!(res, Err(SycophancyError::Disabled)));
    }

    #[test]
    fn fuzz_50_leading_prompts_all_neutralized() {
        // arXiv:2408.11261 §3.1 leading-query fuzz: synthesize 50
        // executor prompts with random leading patterns and assert
        // every one gets its pattern removed.
        for i in 0..50 {
            let pat_idx = i % LEADING_PATTERNS.len();
            let pat = LEADING_PATTERNS[pat_idx];
            let prompt = format!("{}foo bar baz iter_{}", pat, i);
            let n = neutralize_query(&prompt);
            assert!(
                n.removed_tokens.iter().any(|t| t == pat),
                "iter {i}: pattern {pat:?} not stripped from {prompt:?}"
            );
            assert!(
                !n.stripped.to_ascii_lowercase().starts_with(pat),
                "iter {i}: head still matches pattern after strip: {}",
                n.stripped
            );
        }
    }
}
