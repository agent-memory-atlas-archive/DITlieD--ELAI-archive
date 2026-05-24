//! C-52 — Per-tier Dafny prompt builder per arXiv:2604.22601.
//!
//! Three tiers of prompt scaffolding mirror paper RQ1 / RQ2 / RQ3.
//! Each tier emits a deterministic prompt body the caller feeds to
//! the LLM (`anthropic` cloud / remote sidecar / local model).
//!
//! Outputs are PURE text — the function does NOT shell out, does not
//! contact any model, and does not invoke Dafny. The caller is
//! responsible for the inference call.

use crate::dafny_verify::tier::DafnyTier;

/// Build the tier-appropriate prompt body for a Dafny-verified code
/// generation attempt.
///
/// Arguments:
///   - `nl_problem`: natural-language algorithmic spec (paper inputs
///     are UVa Online Judge problems averaging ~179 words).
///   - `tier`: which prompting tier (Contextless / Signature /
///     SelfHealing) to use.
///   - `prior_attempts`: prior `(dafny_source, verifier_feedback)`
///     tuples concatenated as raw text. Required for `SelfHealing`,
///     ignored for the other tiers.
///
/// Returns a deterministic prompt string. Determinism matters because
/// the contract test G1 + G2 assert string-shape stability across
/// runs.
pub fn build_dafny_prompt(
    nl_problem: &str,
    tier: DafnyTier,
    prior_attempts: &[String],
) -> String {
    match tier {
        DafnyTier::Contextless => build_contextless(nl_problem),
        DafnyTier::Signature => build_signature(nl_problem),
        DafnyTier::SelfHealing => build_self_healing(nl_problem, prior_attempts),
    }
}

/// RQ1 — natural-language problem only. The paper reports
/// near-universal failure across 6 of 7 models on this tier.
fn build_contextless(nl_problem: &str) -> String {
    let mut buf = String::with_capacity(nl_problem.len() + 512);
    buf.push_str("# Dafny-verified code generation (tier=contextless, paper RQ1)\n");
    buf.push_str("# arXiv:2604.22601 baseline tier. Near-universal failure expected;\n");
    buf.push_str("# Codestral-22B 27.27% and Gemma 4-31B 54.55% are paper-headline outliers.\n\n");
    buf.push_str("Problem statement:\n");
    buf.push_str(nl_problem);
    buf.push_str("\n\nWrite a Dafny program that solves the problem. Include all\n");
    buf.push_str("preconditions and postconditions necessary for the Dafny verifier\n");
    buf.push_str("(Dafny 4.11.0 + Z3) to discharge the proof obligation.\n");
    buf
}

/// RQ2 — NL problem + method signature with declared pre/post. The
/// paper's minimum-viable tier (Section 3 reports near-universal
/// failure on contextless; signature is the floor for non-trivial
/// results).
fn build_signature(nl_problem: &str) -> String {
    let mut buf = String::with_capacity(nl_problem.len() + 768);
    buf.push_str("# Dafny-verified code generation (tier=signature, paper RQ2)\n");
    buf.push_str("# arXiv:2604.22601 RQ2. Includes the method signature with\n");
    buf.push_str("# preconditions / postconditions as scaffolding. Mandatory tier\n");
    buf.push_str("# for any non-trivial verification result per paper Section 3.\n\n");
    buf.push_str("Problem statement:\n");
    buf.push_str(nl_problem);
    buf.push_str("\n\nProvide the Dafny method signature first (with preconditions\n");
    buf.push_str("via `requires` and postconditions via `ensures`), then implement\n");
    buf.push_str("the body. The signature MUST be discoverable as a paper-style\n");
    buf.push_str("RQ2 anchor for downstream tooling. Include `decreases` clauses\n");
    buf.push_str("for any recursive call to satisfy the termination checker.\n");
    buf
}

/// RQ3 — NL problem + signature + prior verifier feedback. Drives
/// the self-healing loop. The contract test G1 asserts the
/// `# tier=self_healing` header is present so downstream tooling
/// can introspect tier from a raw prompt dump.
fn build_self_healing(nl_problem: &str, prior_attempts: &[String]) -> String {
    let mut buf = String::with_capacity(nl_problem.len() + 1024);
    buf.push_str("# Dafny-verified code generation (tier=self_healing, paper RQ3)\n");
    buf.push_str("# arXiv:2604.22601 RQ3. Iterative-repair tier; reads prior verifier\n");
    buf.push_str("# feedback and emits a corrected Dafny program. Capped at 10\n");
    buf.push_str("# iterations per paper Section 3.\n\n");
    buf.push_str("Problem statement:\n");
    buf.push_str(nl_problem);
    buf.push_str("\n\nPrior attempts (most recent last):\n");
    if prior_attempts.is_empty() {
        buf.push_str("  (none — first self-healing iteration)\n");
    } else {
        for (i, attempt) in prior_attempts.iter().enumerate() {
            buf.push_str(&format!("--- attempt {} ---\n", i + 1));
            buf.push_str(attempt);
            if !attempt.ends_with('\n') {
                buf.push('\n');
            }
        }
    }
    buf.push_str("\nUsing the verifier feedback above, emit a corrected Dafny\n");
    buf.push_str("program. Address every error from the most recent attempt; do\n");
    buf.push_str("NOT regress prior fixes. Preserve the method signature unless\n");
    buf.push_str("the verifier explicitly indicates a precondition mismatch.\n");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextless_prompt_lacks_signature_scaffolding() {
        let p = build_dafny_prompt("compute the GCD of two integers", DafnyTier::Contextless, &[]);
        assert!(p.contains("tier=contextless"));
        // Contextless does NOT include the signature scaffolding.
        assert!(!p.contains("method signature first"));
    }

    #[test]
    fn signature_prompt_includes_signature_scaffolding() {
        let p = build_dafny_prompt("compute the GCD", DafnyTier::Signature, &[]);
        assert!(p.contains("tier=signature"));
        assert!(p.contains("Dafny method signature"));
        // The paper's mandatory RQ2 scaffolding.
        assert!(p.contains("preconditions"));
        assert!(p.contains("postconditions"));
    }

    #[test]
    fn self_healing_prompt_includes_prior_attempts_when_present() {
        let prior = vec!["method Gcd(a: int, b: int) { /* attempt 1 */ }".to_string()];
        let p = build_dafny_prompt("compute the GCD", DafnyTier::SelfHealing, &prior);
        assert!(p.contains("tier=self_healing"));
        assert!(p.contains("attempt 1"));
        assert!(p.contains("Prior attempts"));
    }

    #[test]
    fn self_healing_prompt_handles_empty_prior_attempts() {
        let p = build_dafny_prompt("compute the GCD", DafnyTier::SelfHealing, &[]);
        assert!(p.contains("(none — first self-healing iteration)"));
    }

    #[test]
    fn prompt_is_deterministic_across_calls() {
        // Same inputs must produce byte-identical output. Required so
        // the contract test can assert shape stability without a
        // golden file.
        let a = build_dafny_prompt("problem A", DafnyTier::Signature, &[]);
        let b = build_dafny_prompt("problem A", DafnyTier::Signature, &[]);
        assert_eq!(a, b);
    }
}
