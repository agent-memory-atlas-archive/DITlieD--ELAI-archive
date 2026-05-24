//! C-50 — Public entry point for the contract test scaffolder.
//!
//! Given a single function name + its [`TypeSignature`], build a
//! minimal one-section / one-constraint [`StructuralContract`] and
//! defer to [`crate::contract_coding::emitter::emit_contract_test`]
//! for the text rendering.
//!
//! This is the surface the CLI (`elai contract scaffold ...`) and
//! the dashboard (when a "scaffold test for this function" action
//! is wired) consume. It keeps the StructuralContract construction
//! private to one path so all scaffolder entries produce kernel-
//! consistent contracts.

use crate::contract_coding::emitter::{emit_contract_test, EmitError};
use crate::contract_coding::spec::{StateConstraint, StructuralContract, TypeSignature};

/// Errors the scaffolder can produce.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScaffolderError {
    /// The provided signature was empty (no `function_name`). The
    /// scaffolder refuses to produce a test for an anonymous
    /// function — operator must supply a real identifier.
    #[error("scaffolder requires a non-empty function_name")]
    EmptyFunctionName,
    /// The downstream emitter failed.
    #[error("emit failed: {0}")]
    Emit(EmitError),
}

impl From<EmitError> for ScaffolderError {
    fn from(e: EmitError) -> Self {
        Self::Emit(e)
    }
}

/// Build a one-function structural contract and emit its test
/// scaffolding. The contract uses a single namespace section
/// (`"{fn_name}_module"`) and a single state constraint
/// (`"{fn_name}_signature_pinned"`) so the emitter has a non-empty
/// kernel to walk.
///
/// Caller passes an explicit `fn_name` (rather than relying on
/// `sig.function_name`) so the CLI surface can validate it once
/// at the boundary and reject empty / whitespace inputs before
/// constructing any kernel.
pub fn scaffold_contract_test_from_signature(
    fn_name: &str,
    sig: &TypeSignature,
) -> Result<String, ScaffolderError> {
    let trimmed = fn_name.trim();
    if trimmed.is_empty() {
        return Err(ScaffolderError::EmptyFunctionName);
    }

    // Normalise the canonical signature to match `fn_name` for the
    // kernel — the caller may pass a `TypeSignature` whose name
    // does not match the provided `fn_name` (e.g. when scaffolding
    // from a stripped signature). We rebuild the signature with
    // the canonical `fn_name` so the kernel stays consistent.
    let canonical_sig = TypeSignature::new(
        trimmed,
        sig.signature.clone(),
        sig.language_hint.clone(),
    );

    let namespace_section = format!("{trimmed}_module");
    let constraint_name = format!("{trimmed}_signature_pinned");

    let contract = StructuralContract::new(
        vec![namespace_section],
        vec![canonical_sig],
        vec![StateConstraint::new(
            constraint_name,
            0,
            "signature must match the kernel-pinned shape",
        )],
    );

    Ok(emit_contract_test(&contract, trimmed)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sig() -> TypeSignature {
        TypeSignature::new("add", "fn add(a: i64, b: i64) -> i64", "rust")
    }

    #[test]
    fn empty_fn_name_rejected() {
        let err = scaffold_contract_test_from_signature("", &sample_sig()).unwrap_err();
        assert_eq!(err, ScaffolderError::EmptyFunctionName);
    }

    #[test]
    fn whitespace_fn_name_rejected() {
        let err =
            scaffold_contract_test_from_signature("   ", &sample_sig()).unwrap_err();
        assert_eq!(err, ScaffolderError::EmptyFunctionName);
    }

    #[test]
    fn emits_test_with_kernel_anchors() {
        let body =
            scaffold_contract_test_from_signature("add", &sample_sig()).expect("scaffold");
        assert!(body.contains("add_module"));
        assert!(body.contains("add_signature_pinned"));
        assert!(body.contains("fn add(a: i64, b: i64) -> i64"));
    }

    #[test]
    fn fn_name_overrides_signature_name() {
        let mismatched = TypeSignature::new("OTHER", "fn add(a: i64, b: i64) -> i64", "rust");
        let body =
            scaffold_contract_test_from_signature("add", &mismatched).expect("scaffold");
        // emitter should locate `add` (the canonical name) in Sigma
        assert!(body.contains("add_n_namespace_covers_required_sections"));
    }

    #[test]
    fn trim_whitespace_in_fn_name() {
        let body =
            scaffold_contract_test_from_signature("  add  ", &sample_sig()).expect("scaffold");
        assert!(body.contains("add_module"));
    }
}
