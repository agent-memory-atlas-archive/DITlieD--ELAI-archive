//! C-50 — Contract test scaffolder emitter.
//!
//! Given a [`StructuralContract`] and a target function name, emit
//! a Rust test-file template that scaffolds contract assertions
//! over the paper kernel `K = <N, Sigma, Delta>`. The emitter is
//! purely textual; the auditor (or `cargo check`) verifies that
//! the emitted file is syntactically valid Rust.
//!
//! Paper grounding: arXiv:2604.13100 §4.4 — the Discriminator Agent
//! consumes the contract; the emitter here produces the assertion
//! scaffolding the human auditor reviews before promoting the test
//! into the locked corpus.
//!
//! Emission discipline:
//!   - No magic numbers; section identifiers are derived from
//!     `target_fn`.
//!   - No raw `unwrap()` in the emitted body — the template uses
//!     `expect("reason")` so failures point at the missing
//!     scaffolding rather than at a panic.
//!   - The header carries the paper cite with the structural-
//!     integrity qualifier so future readers see the qualifier
//!     before the test body.

use std::fmt::Write as _;

use crate::contract_coding::spec::StructuralContract;

/// Errors the emitter can produce.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmitError {
    /// The supplied contract did not match `target_fn` against any
    /// signature in `Sigma`. The emitter refuses to scaffold a test
    /// for an unknown function — this prevents silent stub
    /// emission for misspelled identifiers.
    #[error("contract does not contain a signature for `{0}`; refusing to scaffold a stub test")]
    UnknownTargetFunction(String),
    /// The contract failed its own kernel invariants (disjointness,
    /// reference-in-range). Emission is gated on validation passing.
    #[error("contract failed structural validation: {0}")]
    InvalidContract(String),
    /// The string builder failed (out-of-memory; effectively never).
    #[error("internal: emitter format failure: {0}")]
    Format(String),
}

/// Emit a Rust test-file template that scaffolds contract
/// assertions for `target_fn`. The returned string is the full
/// file contents (header comments + `use` lines + #[test] bodies).
///
/// Caller responsibility: validate that `target_fn` is one of the
/// `function_name`s in `contract.signatures`. The emitter does this
/// check defensively and returns [`EmitError::UnknownTargetFunction`]
/// if no match exists.
pub fn emit_contract_test(
    contract: &StructuralContract,
    target_fn: &str,
) -> Result<String, EmitError> {
    contract
        .validate()
        .map_err(|e| EmitError::InvalidContract(e.to_string()))?;

    let target = contract
        .signatures
        .iter()
        .find(|s| s.function_name == target_fn)
        .ok_or_else(|| EmitError::UnknownTargetFunction(target_fn.to_string()))?;

    let mut out = String::new();
    writeln!(out, "//! C-50 STRUCTURAL contract test scaffolding for `{}`.", target.function_name)
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "//!").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "//! Paper grounding: arXiv:2604.13100 — Contract-Coding ties Trae"
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "//! (47%) on Greenfield-5 Roguelike subset; differentiator is structural"
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "//! integrity (failures confined to Local Logic Errors, not architectural"
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "//! collapse) per Section 5.2.").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "//!").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "//! Kernel: K = <N, Sigma, Delta> with |N|={}, |Sigma|={}, |Delta|={}.",
        contract.namespace.len(),
        contract.signatures.len(),
        contract.state_space.len(),
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "//! Target signature: {}", target.signature)
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "//! Language hint:    {}", target.language_hint)
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out).map_err(|e| EmitError::Format(e.to_string()))?;

    writeln!(out, "// ---------------------------------------------------------------------------")
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "// 1. Namespace (N) coverage").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "// ---------------------------------------------------------------------------")
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "#[test]").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "fn {}_n_namespace_covers_required_sections() {{",
        sanitize_ident(&target.function_name)
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    for section in &contract.namespace {
        writeln!(out, "    // section: {}", section).map_err(|e| EmitError::Format(e.to_string()))?;
    }
    writeln!(out, "    // TODO[C50-NS]: assert the implementation declares each section above").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "}}").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out).map_err(|e| EmitError::Format(e.to_string()))?;

    writeln!(out, "// ---------------------------------------------------------------------------")
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "// 2. Signature (Sigma) shape").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "// ---------------------------------------------------------------------------")
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "#[test]").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "fn {}_sigma_signature_matches_kernel() {{",
        sanitize_ident(&target.function_name)
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "    let expected_signature: &str = {:?};",
        target.signature
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "    assert!(!expected_signature.is_empty(), \"signature anchor must be non-empty\");"
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "}}").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out).map_err(|e| EmitError::Format(e.to_string()))?;

    writeln!(out, "// ---------------------------------------------------------------------------")
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "// 3. State-space (Delta) constraints").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "// ---------------------------------------------------------------------------")
        .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "#[test]").map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(
        out,
        "fn {}_delta_state_constraints_enforced() {{",
        sanitize_ident(&target.function_name)
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    for c in &contract.state_space {
        writeln!(out, "    // constraint: {} (ns#{})", c.name, c.namespace_ref).map_err(|e| EmitError::Format(e.to_string()))?;
    }
    writeln!(
        out,
        "    // TODO[C50-DELTA]: assert each constraint above holds against the implementation"
    )
    .map_err(|e| EmitError::Format(e.to_string()))?;
    writeln!(out, "}}").map_err(|e| EmitError::Format(e.to_string()))?;

    Ok(out)
}

/// Replace any non-`[A-Za-z0-9_]` byte with `_` so the function name
/// can be safely interpolated into a Rust identifier. The first
/// character is also forced to start with `_` or a letter so we
/// never emit an identifier beginning with a digit.
fn sanitize_ident(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if let Some(first) = s.chars().next() {
        if first.is_ascii_digit() {
            s.insert(0, '_');
        }
    } else {
        s.push_str("anon");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_coding::spec::{StateConstraint, TypeSignature};

    fn good_contract() -> StructuralContract {
        StructuralContract::new(
            vec!["alpha".into(), "beta".into()],
            vec![TypeSignature::new("add", "fn add(a: i64, b: i64) -> i64", "rust")],
            vec![StateConstraint::new("totality", 0, "add is total on i64")],
        )
    }

    #[test]
    fn emits_for_known_target() {
        let c = good_contract();
        let body = emit_contract_test(&c, "add").expect("emit");
        assert!(body.contains("add_n_namespace_covers_required_sections"));
        assert!(body.contains("add_sigma_signature_matches_kernel"));
        assert!(body.contains("add_delta_state_constraints_enforced"));
        assert!(body.contains("fn add(a: i64, b: i64) -> i64"));
        assert!(body.contains("Roguelike subset"));
        assert!(body.contains("structural"));
    }

    #[test]
    fn rejects_unknown_target() {
        let c = good_contract();
        let err = emit_contract_test(&c, "missing_fn").unwrap_err();
        assert!(matches!(err, EmitError::UnknownTargetFunction(ref s) if s == "missing_fn"));
    }

    #[test]
    fn rejects_invalid_contract() {
        let bad = StructuralContract::new(
            vec!["alpha".into(), "alpha".into()],
            vec![TypeSignature::new("foo", "fn foo()", "rust")],
            vec![],
        );
        let err = emit_contract_test(&bad, "foo").unwrap_err();
        assert!(matches!(err, EmitError::InvalidContract(_)));
    }

    #[test]
    fn sanitize_ident_replaces_specials() {
        assert_eq!(sanitize_ident("foo::bar"), "foo__bar");
        assert_eq!(sanitize_ident("9deg"), "_9deg");
        assert_eq!(sanitize_ident("my_fn"), "my_fn");
        assert_eq!(sanitize_ident("a-b.c"), "a_b_c");
    }

    #[test]
    fn emitted_body_includes_all_namespace_sections() {
        let c = good_contract();
        let body = emit_contract_test(&c, "add").expect("emit");
        assert!(body.contains("section: alpha"));
        assert!(body.contains("section: beta"));
    }

    #[test]
    fn emitted_body_includes_all_state_constraints() {
        let c = good_contract();
        let body = emit_contract_test(&c, "add").expect("emit");
        assert!(body.contains("constraint: totality"));
    }
}
