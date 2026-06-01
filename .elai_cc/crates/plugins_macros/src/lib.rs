//! C-42b (Tranche M, s-22600) -- declarative `#[authz]` proc-macro
//! attribute.
//!
//! FIRST proc-macro crate in the ELAI workspace. Ships a marker-only
//! attribute that validates a named-argument `(gate, tool_kind)` pair via
//! `darling::FromMeta` over `syn 2`'s `Punctuated<Meta, Comma>` parser
//! (the canonical replacement for the deprecated
//! `syn::AttributeArgs`; see darling 0.20 `ast::NestedMeta::parse_meta_list`).
//!
//! Design decision -- MARKER-ONLY (per workflow/sessions/s-22600/
//! context-package-trancheM.md D-M1):
//!
//! The expansion emits the original function body UNCHANGED. The
//! attribute parses + validates the `(gate, tool_kind)` pair and embeds
//! two `const` literals (`__ELAI_AUTHZ_GATE` / `__ELAI_AUTHZ_TOOL_KIND`)
//! into the function body as a static-analysis anchor. NO runtime
//! behaviour change is introduced -- a future lint can enforce the
//! presence of this attribute on every dispatcher that calls
//! `::elai_plugins::authz::compose_with_tool_gate_emit`, but the
//! present revision ships scaffolding only.
//!
//! Rationale: per `workflow/reasoning/D2-phase-c-deferral.md` L10, the
//! attribute is a developer-UX shorthand over the runtime guard at
//! `crates/plugins/src/authz/compose.rs` and NOT a correctness or
//! security requirement. The full-wrap variant (auto-call the gate
//! emitter from the macro body) would require attribute arguments
//! naming the policy / gate / emitter bindings at every call site --
//! out of scope for this tranche.
//!
//! Anti-slopsquatting (CLAUDE.md HARD RULE): all four direct deps
//! (`syn` 2.0.117, `quote` 1.0.45, `proc-macro2` 1.0.106,
//! `darling` 0.20.11) are pre-resolved in the workspace `Cargo.lock`.
//! Adding this crate produces zero lock churn. See `Cargo.toml` header
//! and `workflow/reasoning/M-trancheM-decisions.md` for the per-dep
//! `cargo info` trace.

use darling::FromMeta;
use darling::ast::NestedMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Named-argument shape parsed from `#[authz(gate = "...", tool_kind = "...")]`.
///
/// `gate` is the name of an environment-variable wire-dark switch that
/// gates the dispatcher (e.g. `"ELAI_GATEWAY_AUTHZ_ENABLE"`). `tool_kind`
/// is the MCP tool family the dispatcher serves (e.g. `"mcp"`, `"http"`,
/// `"fs"`). Both arguments are MANDATORY -- omission produces a
/// compile-time darling error at the attribute site.
#[derive(FromMeta, Debug)]
struct AuthzArgs {
    gate: String,
    tool_kind: String,
}

/// `#[authz(gate = "...", tool_kind = "...")]` proc-macro attribute.
///
/// Marker-only expansion -- see module-level docs. Validates the
/// attribute argument shape via `darling::FromMeta` and embeds the two
/// parsed literals as `const` items inside the original function body.
/// The decorated function body is otherwise preserved verbatim.
#[proc_macro_attribute]
pub fn authz(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute argument list via the syn 2 + darling 0.20
    // canonical path: NestedMeta::parse_meta_list returns a
    // Vec<NestedMeta>; FromMeta::from_list consumes the slice.
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };
    let parsed = match AuthzArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let item_fn = parse_macro_input!(input as ItemFn);

    // Decompose so we can prepend the marker `const` items to the
    // function body without disturbing attrs / vis / sig.
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = item_fn;

    let gate_lit = parsed.gate;
    let tool_kind_lit = parsed.tool_kind;
    let stmts = &block.stmts;

    // Marker-only expansion: original fn body verbatim, prefixed with
    // two `const` items the future lint can scrape via cargo-expand.
    // The `#[allow(dead_code)]` lets the consts coexist with downstream
    // clippy lint level = warn workspace defaults without firing
    // unused-const warnings inside the user's function.
    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            #[allow(dead_code)]
            const __ELAI_AUTHZ_GATE: &str = #gate_lit;
            #[allow(dead_code)]
            const __ELAI_AUTHZ_TOOL_KIND: &str = #tool_kind_lit;
            #(#stmts)*
        }
    };

    TokenStream::from(expanded)
}
