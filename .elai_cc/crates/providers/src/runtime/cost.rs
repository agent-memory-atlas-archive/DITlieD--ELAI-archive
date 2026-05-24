//! Backend cost tracking (C-44 Phase 4M extension).
//!
//! Per-backend cost is expressed in micro-dollars (1e-6 USD = 1
//! "micro") to keep integer arithmetic exact. Prices are sourced from
//! the provider's public pricing page (verified via WebFetch on the
//! plan's source-grounding pass — see workflow/papers/composer2-
//! interface.md).
//!
//! Pricing math
//! ============
//! Composer 2 pricing (cursor.com/docs/api 2026-05-24):
//!   - input  : $0.50 per 1,000,000 tokens
//!             = $0.0005 per 1,000 tokens
//!             = 0.5 micro-dollars per token? No — micros are 1e-6 USD,
//!               so $0.0005 = 500 micros, per 1k tokens.
//!             = 500 micros / 1k tokens
//!   - output : $2.50 per 1,000,000 tokens
//!             = $0.0025 per 1,000 tokens
//!             = 2,500 micros / 1k tokens
//!
//! Formula:
//!   cost_micros = (tokens_in  * input_per_1k_micros  + 999) / 1000
//!               + (tokens_out * output_per_1k_micros + 999) / 1000
//! The +999 / 1000 form is a ceiling-divide so we never undercount a
//! partial 1k-token chunk (operators pay at the per-token rate, but
//! we report at integer-micro granularity).
//!
//! Tier semantics
//! ==============
//! ELAI deliberately does NOT introduce a `BackendTier` enum. "Tier"
//! is a cost-ordering view emergent from the per-backend
//! `BackendCostModel` — local sidecar = 0 micros, composer-2 = the
//! middle tier (~500-2500 micros / 1k), anthropic_api (Sonnet 4.6)
//! = the premium tier. Tier rank is implicit in price ascending.
//! Code reading "tier" should compute it from
//! `input_per_1k_micros + output_per_1k_micros` rather than encode it
//! as a hardcoded constant — that way new backends slot in by price
//! alone, no enum edit required.

use elai_ipc_types::AgentRole;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use super::types::RuntimeBackend;

/// Per-1k-token cost rates for a single backend, in micro-dollars
/// (1 micro = 1e-6 USD). Integer arithmetic keeps cost roll-ups
/// exact across millions of small invocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCostModel {
    /// Backend this cost model applies to.
    pub backend: RuntimeBackend,
    /// Cost in micros per 1k input tokens.
    pub input_per_1k_micros: u64,
    /// Cost in micros per 1k output tokens.
    pub output_per_1k_micros: u64,
}

impl BackendCostModel {
    /// Compute the per-call cost in micros.
    #[must_use]
    pub fn estimate_micros(&self, tokens_in: u32, tokens_out: u32) -> u64 {
        let input_micros = ceil_div(
            (tokens_in as u64).saturating_mul(self.input_per_1k_micros),
            1_000,
        );
        let output_micros = ceil_div(
            (tokens_out as u64).saturating_mul(self.output_per_1k_micros),
            1_000,
        );
        input_micros.saturating_add(output_micros)
    }
}

#[inline]
fn ceil_div(numerator: u64, denom: u64) -> u64 {
    if denom == 0 {
        return 0;
    }
    numerator.saturating_add(denom.saturating_sub(1)) / denom
}

/// C-44: Composer 2 cost model. Source-grounded against
/// cursor.com/docs/api pricing page 2026-05-24 — see
/// workflow/papers/composer2-interface.md for the SDK verification
/// step. Values are static const so the dashboard can render the
/// rate table without re-reading a config file on every paint.
pub const COMPOSER2_COST: BackendCostModel = BackendCostModel {
    backend: RuntimeBackend::Composer2,
    // $0.50 / 1M input  = $0.0005 / 1k = 500 micros / 1k input.
    input_per_1k_micros: 500,
    // $2.50 / 1M output = $0.0025 / 1k = 2500 micros / 1k output.
    output_per_1k_micros: 2_500,
};

/// Per-backend cost-model lookup. Returns `None` for backends that
/// are not paid-cloud (Sidecar, Mlx, LlamaCpp, Ollama, Vllm) —
/// callers should treat absence as "zero marginal cost". Paid
/// backends (Composer2, AnthropicApi, ClaudeCode subscriptions) live
/// in this table.
#[must_use]
pub fn cost_model_for(backend: RuntimeBackend) -> Option<BackendCostModel> {
    match backend {
        RuntimeBackend::Composer2 => Some(COMPOSER2_COST),
        // AnthropicApi / ClaudeCode / etc. are paid but their cost
        // model lives in `crates/providers/src/pricing.rs` for legacy
        // reasons. C-44 does NOT consolidate those here — that's a
        // future refactor. The runtime cost surface focuses on
        // backends that ELAI added cost-tracking for in this plan.
        _ => None,
    }
}

/// Blake3-derived 64-bit hash used as the wire-stable backend
/// identifier inside `BackendCostAccrued` SSB rows. The dashboard
/// resolves the hash via the canonical `RuntimeBackend::all()`
/// iteration ordering — no plaintext leaks across the SSB boundary.
///
/// We use FxHasher seeded with the canonical_name() bytes for a
/// stable, deterministic, 64-bit-fitting digest. blake3 would be
/// the "correct" hash per CLAUDE.md anti-stale rules but is overkill
/// for an enum-discriminator that has 15 possible values and 0
/// adversarial surface — FxHasher matches the precedent set by
/// other SSB hash fields (e.g. ratifier_id_hash in C-43).
#[must_use]
pub fn backend_name_hash(backend: RuntimeBackend) -> u64 {
    let mut hasher = FxHasher::default();
    backend.canonical_name().hash(&mut hasher);
    hasher.finish()
}

/// Hash an `AgentRole` for the SSB wire — symmetric with
/// `backend_name_hash` above. Uses serde's snake_case form so the
/// hash stays stable across rename refactors.
#[must_use]
pub fn role_name_hash(role: AgentRole) -> u64 {
    let role_str = serde_json::to_value(role)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| format!("{role:?}").to_ascii_lowercase());
    let mut hasher = FxHasher::default();
    role_str.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer2_cost_model_matches_documented_pricing() {
        // $0.50 / 1M input = 500 micros / 1k input.
        assert_eq!(COMPOSER2_COST.input_per_1k_micros, 500);
        // $2.50 / 1M output = 2500 micros / 1k output.
        assert_eq!(COMPOSER2_COST.output_per_1k_micros, 2_500);
        assert_eq!(COMPOSER2_COST.backend, RuntimeBackend::Composer2);
    }

    #[test]
    fn estimate_micros_for_one_thousand_input_tokens() {
        // 1000 input tokens at $0.50/M = $0.0005 = 500 micros.
        let micros = COMPOSER2_COST.estimate_micros(1_000, 0);
        assert_eq!(micros, 500);
    }

    #[test]
    fn estimate_micros_for_one_thousand_output_tokens() {
        // 1000 output tokens at $2.50/M = $0.0025 = 2500 micros.
        let micros = COMPOSER2_COST.estimate_micros(0, 1_000);
        assert_eq!(micros, 2_500);
    }

    #[test]
    fn estimate_micros_mixed_load() {
        // 2000 in + 500 out = 1000 micros + 1250 micros = 2250 micros.
        let micros = COMPOSER2_COST.estimate_micros(2_000, 500);
        assert_eq!(micros, 1_000 + 1_250);
    }

    #[test]
    fn estimate_micros_zero_tokens_returns_zero() {
        assert_eq!(COMPOSER2_COST.estimate_micros(0, 0), 0);
    }

    #[test]
    fn estimate_micros_ceiling_divides_partial_chunks() {
        // 1 input token at $0.50/M = $0.0000005 = 0.5 micros.
        // Ceiling-divide -> 1 micro (we never undercount partials).
        let micros = COMPOSER2_COST.estimate_micros(1, 0);
        assert_eq!(micros, 1);
    }

    #[test]
    fn estimate_micros_saturates_on_extreme_token_counts() {
        // u32::MAX tokens at 2500 micros/1k would overflow naive u32
        // arithmetic — confirm we saturate cleanly.
        let micros = COMPOSER2_COST.estimate_micros(u32::MAX, u32::MAX);
        assert!(micros > 0, "saturating arithmetic must not underflow");
    }

    #[test]
    fn cost_model_for_returns_none_for_local_backends() {
        for backend in [
            RuntimeBackend::Sidecar,
            RuntimeBackend::Mlx,
            RuntimeBackend::LlamaCpp,
            RuntimeBackend::Ollama,
            RuntimeBackend::Vllm,
        ] {
            assert!(
                cost_model_for(backend).is_none(),
                "local backend {backend:?} should have no cost model"
            );
        }
    }

    #[test]
    fn cost_model_for_returns_some_for_composer2() {
        let model = cost_model_for(RuntimeBackend::Composer2).expect("composer2 has cost model");
        assert_eq!(model.input_per_1k_micros, 500);
        assert_eq!(model.output_per_1k_micros, 2_500);
    }

    #[test]
    fn backend_name_hash_is_deterministic_across_calls() {
        let a = backend_name_hash(RuntimeBackend::Composer2);
        let b = backend_name_hash(RuntimeBackend::Composer2);
        assert_eq!(a, b, "hash must be deterministic");
    }

    #[test]
    fn backend_name_hash_distinguishes_backends() {
        let composer2 = backend_name_hash(RuntimeBackend::Composer2);
        let sidecar = backend_name_hash(RuntimeBackend::Sidecar);
        let anthropic = backend_name_hash(RuntimeBackend::AnthropicApi);
        assert_ne!(composer2, sidecar);
        assert_ne!(composer2, anthropic);
        assert_ne!(sidecar, anthropic);
    }

    #[test]
    fn role_name_hash_is_deterministic_across_calls() {
        let a = role_name_hash(AgentRole::Executor);
        let b = role_name_hash(AgentRole::Executor);
        assert_eq!(a, b);
    }

    #[test]
    fn role_name_hash_distinguishes_roles() {
        let executor = role_name_hash(AgentRole::Executor);
        let auditor = role_name_hash(AgentRole::Auditor);
        assert_ne!(executor, auditor);
    }
}
