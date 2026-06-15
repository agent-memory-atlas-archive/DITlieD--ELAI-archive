// I-50 (SCHEMA_VERSION 268): subscription auth broker (credential-resolution layer).
//
// This module lives in elai-providers (no dependency on elai-orchestrator, which
// would create a circular dep: elai-orchestrator -> elai-providers -> elai-orchestrator).
//
// SSB emit is intentionally absent here. A thin SSB-emit wrapper lives in
// `elai-orchestrator::auth_broker_emit` (which can depend on both crates), and is the
// PRODUCTION_HOOK seam called from `cli/src/runtime_cmd.rs:387`.
//
// This module defines:
//   - `AuthProvider` trait: generalized credential-lifecycle contract.
//   - `AnthropicAuthProvider`: delegates to `resolve_startup_auth_source`.
//   - `AuthBroker`: owns an `AuthProvider` and resolves credentials.
//   - `broker_resolve_anthropic_auth()`: convenience entry point for the Anthropic provider.
//
// WIRE-DARK[I-50]: the PRODUCTION caller is `runtime_cmd.rs:387` via
// `orchestrator::auth_broker_emit::broker_resolve_anthropic_auth_with_ssb(None)`.
// The `None` is the DARK part: `runtime_cmd.rs` has no `Arc<Ssb>` today.
// Threading Ssb is deferred (I-50 fork resolution A183).
//
// SECURITY: this module never emits token material to logs or SSB rows.

use crate::error::ApiError;
use crate::providers::anthropic::{oauth_token_is_expired, resolve_startup_auth_source, OAuthTokenSet as WireTokenSet};
use crate::{default_claude_code_oauth_config, load_oauth_credentials, AuthSource as PubAuthSource};

// ---------------------------------------------------------------------------
// AuthBrokerKind: outcome classifier (no SSB dependency)
// ---------------------------------------------------------------------------

/// Outcome of a credential-resolve call, used by the SSB-emit wrapper in
/// `elai-orchestrator::auth_broker_emit` to select the correct
/// `AuthBrokerEventKind` without re-running the waterfall.
///
/// This is a providers-layer enum — no SSB dependency. The orchestrator wrapper
/// maps it to `AuthBrokerEventKind` (the SSB wire type) after the fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveOutcome {
    /// Credential resolved from env-key, env-bearer, or non-expired saved token.
    TokenAcquired,
    /// Expired saved token was refreshed via HTTP exchange and re-persisted.
    Refreshed,
    /// Resolution failed; error surfaced as `ApiError`.
    ///
    /// Reserved: emitted by the I-50-ssb-threading promotion path (writer not yet threaded). Currently unconstructed.
    Failed,
}

// ---------------------------------------------------------------------------
// AuthProvider trait
// ---------------------------------------------------------------------------

/// Generalized credential-lifecycle contract.
///
/// Currently implemented by [`AnthropicAuthProvider`].  Future impls:
/// `AwsBedrockAuthProvider`, `AzureAuthProvider`, etc.
pub trait AuthProvider {
    /// Provider identifier (e.g. `"anthropic"`).  Used in SSB rows via the
    /// orchestrator wrapper; never contains token material.
    fn provider_id(&self) -> &str;

    /// Resolve the current credential for this provider.
    ///
    /// Returns `(AuthSource, ResolveOutcome)` on success so callers can emit a
    /// correctly-labelled SSB row without re-running the waterfall.
    fn resolve(&self) -> Result<(PubAuthSource, ResolveOutcome), ApiError>;
}

// ---------------------------------------------------------------------------
// AnthropicAuthProvider
// ---------------------------------------------------------------------------

/// Anthropic impl of [`AuthProvider`].
///
/// Delegates entirely to [`resolve_startup_auth_source`] via the canonical
/// `default_claude_code_oauth_config` closure, preserving byte-compat with the
/// existing `resolve_auth_with_auto_refresh()` path.  Does NOT re-implement the
/// credential waterfall; it only classifies the outcome by mirroring the two
/// cheapest guards.
pub struct AnthropicAuthProvider;

impl AuthProvider for AnthropicAuthProvider {
    fn provider_id(&self) -> &str {
        "anthropic"
    }

    fn resolve(&self) -> Result<(PubAuthSource, ResolveOutcome), ApiError> {
        // Classify the anticipated outcome before calling the waterfall (cheapest
        // guards only; avoids instrumenting the private `resolve_startup_auth_source`).
        let anticipated_outcome = anticipate_outcome();
        let auth_source =
            resolve_startup_auth_source(|| Ok(Some(default_claude_code_oauth_config())))?;
        Ok((auth_source, anticipated_outcome))
    }
}

/// Classify the anticipated [`ResolveOutcome`] by mirroring the cheapest guards
/// from `resolve_startup_auth_source` (env-var check + token expiry).
///
/// Fails-safe to `TokenAcquired` when uncertain.
fn anticipate_outcome() -> ResolveOutcome {
    // Guard 1: env-key branches → TokenAcquired.
    if std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
        || std::env::var("ANTHROPIC_AUTH_TOKEN")
            .ok()
            .filter(|v| !v.is_empty())
            .is_some()
    {
        return ResolveOutcome::TokenAcquired;
    }

    // Guard 2: saved-token state.
    match load_oauth_credentials() {
        Ok(Some(stored)) => {
            // Convert storage OAuthTokenSet to wire OAuthTokenSet for the shared expiry check.
            let wire = WireTokenSet {
                access_token: stored.access_token.clone(),
                refresh_token: stored.refresh_token.clone(),
                expires_at: stored.expires_at,
                scopes: stored.scopes.clone(),
            };
            if oauth_token_is_expired(&wire) && stored.refresh_token.is_some() {
                // Waterfall will attempt HTTP refresh.
                ResolveOutcome::Refreshed
            } else {
                ResolveOutcome::TokenAcquired
            }
        }
        // No saved token or I/O error → waterfall will fail.
        _ => ResolveOutcome::TokenAcquired,
    }
}

// ---------------------------------------------------------------------------
// AuthBroker
// ---------------------------------------------------------------------------

/// Central broker owning the credential lifecycle for one or more providers.
///
/// This is the providers-layer broker (no SSB).  For the SSB-emitting wrapper,
/// see `elai-orchestrator::auth_broker_emit::broker_resolve_anthropic_auth_with_ssb`.
pub struct AuthBroker;

impl AuthBroker {
    /// Resolve credentials via `provider`.
    ///
    /// Returns `(AuthSource, ResolveOutcome)` on success.  The outcome is used by
    /// the orchestrator wrapper to emit the correctly-labelled SSB row.
    pub fn resolve<P: AuthProvider>(
        provider: &P,
    ) -> Result<(PubAuthSource, ResolveOutcome), ApiError> {
        match provider.resolve() {
            Ok((auth_source, outcome)) => Ok((auth_source, outcome)),
            Err(err) => {
                // Surface Failed outcome so the orchestrator wrapper can emit a
                // RefreshFailed SSB row before re-raising the error.
                Err(err)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience: pure credential-resolve entry point (no SSB)
// ---------------------------------------------------------------------------

/// Resolve Anthropic credentials via the broker (no SSB emit).
///
/// The SSB-emitting wrapper (`broker_resolve_anthropic_auth_with_ssb`) in
/// `elai-orchestrator::auth_broker_emit` calls this fn and adds SSB emit on top.
///
/// WIRE-DARK[I-50]: production callers go through the orchestrator wrapper, not
/// this fn directly, so SSB emit is not bypassed.
pub fn broker_resolve_anthropic_auth() -> Result<(PubAuthSource, ResolveOutcome), ApiError> {
    AuthBroker::resolve(&AnthropicAuthProvider)
}
