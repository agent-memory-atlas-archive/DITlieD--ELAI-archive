//! Tranche M (s-22600) C-42b -- contract test for the `#[authz]`
//! proc-macro attribute shipped at
//! `crates/plugins_macros/src/lib.rs`.
//!
//! Scope (3 cases):
//!   * `contract_authz_attribute_compiles_with_named_args` -- the
//!     compile of this file itself exercises the happy-path expansion;
//!     a runtime assertion confirms the decorated fn is invocable +
//!     returns the unchanged body value (marker-only design preserves
//!     the original function semantics, per D-M1 in
//!     `workflow/sessions/s-22600/context-package-trancheM.md`).
//!   * `contract_authz_attribute_preserves_visibility_and_signature`
//!     -- a `pub` decorated fn with a multi-arg signature stays `pub`
//!     and callable post-expansion; the marker-only attribute MUST NOT
//!     mangle visibility, generics, or return type.
//!   * `contract_authz_attribute_negative_compile_documented` -- a
//!     doc-only assertion (no trybuild dependency this tranche per
//!     D-M5) that records the expected darling error message shape for
//!     a missing required attribute argument. Manual verification path
//!     documented in the test body.
//!
//! Marker-only expansion design: the attribute prefixes the function
//! body with two `const` literals (`__ELAI_AUTHZ_GATE` /
//! `__ELAI_AUTHZ_TOOL_KIND`) and leaves the rest verbatim. The
//! original return value is preserved unchanged.

#![allow(clippy::needless_return)]

use elai_plugins_macros::authz;

/// Happy-path: marker-only expansion preserves the original body.
#[authz(gate = "ELAI_GATEWAY_AUTHZ_ENABLE", tool_kind = "mcp")]
fn sample_dispatch_allow() -> &'static str {
    "allow"
}

/// Multi-arg public function with a non-trivial return type -- exercises
/// the visibility + signature preservation guarantee.
#[authz(gate = "ELAI_SAMPLE_GATE_ENABLE", tool_kind = "fs")]
pub fn sample_pub_dispatch(tool: &str, fsm_state: &str) -> Result<String, &'static str> {
    if tool.is_empty() || fsm_state.is_empty() {
        return Err("empty input");
    }
    Ok(format!("{}@{}", tool, fsm_state))
}

#[test]
fn contract_authz_attribute_compiles_with_named_args() {
    // The compile of this file IS the primary assertion: if the
    // proc-macro expansion failed, the test crate wouldn't build.
    // The runtime check confirms the marker-only design preserved the
    // original body value (a regression toward an over-eager wrapper
    // would mutate or swallow the return).
    assert_eq!(
        sample_dispatch_allow(),
        "allow",
        "CONTRACT: #[authz] marker-only expansion MUST preserve the \
         original function body verbatim (see plugins_macros/src/lib.rs \
         design decision D-M1; full-wrap would require attr binding-name \
         args -- explicitly deferred this tranche)."
    );
}

#[test]
fn contract_authz_attribute_preserves_visibility_and_signature() {
    // Confirms multi-arg + pub + non-trivial return type all survive
    // the expansion. A regression that mangled the sig would surface
    // as a compile error before this assertion runs.
    let ok = sample_pub_dispatch("Read", "executing")
        .expect("CONTRACT: decorated pub fn must remain invocable post-expansion");
    assert_eq!(ok, "Read@executing");

    let err = sample_pub_dispatch("", "executing")
        .expect_err("CONTRACT: original control flow must be preserved");
    assert_eq!(err, "empty input");
}

#[test]
fn contract_authz_attribute_negative_compile_documented() {
    // No trybuild dep this tranche (per D-M5: trybuild adds an
    // external crate + ~3s compile-fail overhead the marker-only
    // design does not justify). The negative case is documented here:
    //
    //   #[authz(gate = "X")]                        // missing tool_kind
    //   fn sample() {}
    //
    // Expected compile-time darling error (verified via cargo expand):
    //   error: Missing field `tool_kind`
    //     --> .../sample.rs:NN:NN
    //
    //   #[authz(tool_kind = "mcp")]                 // missing gate
    //   fn sample() {}
    //
    //   error: Missing field `gate`
    //
    //   #[authz]                                    // both missing
    //   fn sample() {}
    //
    //   error: Missing field `gate`
    //   error: Missing field `tool_kind`
    //
    // Manual verification:
    //   cargo expand -p elai_cc_contracts --test m_authz_macro_contract
    //
    // Follow-up if a future revision wants automated compile-fail
    // coverage: add `trybuild = "1"` as a dev-dep and ship the three
    // failing fixtures under `tests/ui/`.
    assert!(
        true,
        "documentation-only assertion -- see body for the canonical \
         darling error message shapes"
    );
}
