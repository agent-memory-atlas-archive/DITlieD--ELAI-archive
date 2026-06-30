// I-50 (SCHEMA_VERSION 268): auth broker SSB-emit wrapper.
//
// This module lives in elai-orchestrator (depends on both elai-providers and
// elai-orchestrator::ssb) to avoid the circular dependency that would arise if
// auth_broker.rs in elai-providers imported SSB types directly.
//
// Separation:
//   - elai-providers::auth_broker — credential resolution only (`AuthProvider`,
//     `AnthropicAuthProvider`, `AuthBroker`, `ResolveOutcome`; no SSB types).
//   - THIS module — maps `ResolveOutcome` → `AuthBrokerEventKind`, emits SSB row,
//     calls the providers layer, and exposes the production entry point:
//     `broker_resolve_anthropic_auth_with_ssb`.
//
// PRODUCTION_HOOK: `cli/src/runtime_cmd.rs:387`
//   `resolve_anthropic_auth_for_factory()` calls
//   `broker_resolve_anthropic_auth_with_ssb(None)`.
//
// WIRE-DARK[I-50]: the `writer` arg is `None` at `runtime_cmd.rs:387` until
// `Arc<Ssb>` is threaded through the CLI dispatch path (I-50 fork resolution A183).
// The emit path is proven reachable by `i50_auth_broker_contract` (real TempDir SQLite).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_providers::auth_broker::{
    broker_resolve_anthropic_auth, AnthropicAuthProvider, AuthBroker, ResolveOutcome,
};
use elai_providers::{ApiError, AuthSource};

use crate::ssb::{AuthBrokerEventKind, AuthBrokerEventPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// ResolveOutcome → AuthBrokerEventKind mapping
// ---------------------------------------------------------------------------

fn outcome_to_kind(outcome: ResolveOutcome) -> AuthBrokerEventKind {
    match outcome {
        ResolveOutcome::TokenAcquired => AuthBrokerEventKind::TokenAcquired,
        ResolveOutcome::Refreshed => AuthBrokerEventKind::Refreshed,
        ResolveOutcome::Failed => AuthBrokerEventKind::RefreshFailed,
    }
}

// ---------------------------------------------------------------------------
// SSB emit helper
// ---------------------------------------------------------------------------

/// Emit a best-effort `AuthBrokerEvent` SSB row from a sync context.
///
/// Uses `tokio::runtime::Handle::try_current()` to spawn an async write task.
/// If no tokio runtime is active, the emit is skipped (best-effort; a missed
/// SSB row is never a correctness failure).
fn emit_auth_broker_event(ssb: &Arc<Ssb>, kind: AuthBrokerEventKind, error_kind: &str) {
    let ts_ms = SystemTime::now() // entropy-allow: SSB journal ts_ms, no ClockProvider in scope (sync fire-and-forget emitter)
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let payload = SsbPayload::AuthBrokerEvent(AuthBrokerEventPayload {
        kind,
        provider_id: "anthropic".to_owned(),
        error_kind: error_kind.to_owned(),
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(async move {
                if let Err(e) = ssb.write_payload("auth_broker", 0, payload, vec![]).await {
                    tracing::warn!(
                        auth_broker.ssb_err = %e,
                        "AuthBrokerEvent SSB emit failed (auth result unaffected)"
                    );
                }
            });
        }
        Err(_) => {
            tracing::debug!("auth_broker_emit: no tokio runtime, skipping SSB row");
        }
    }
}

/// Short machine-readable classifier for `AuthBrokerEventPayload::error_kind`.
/// Extracts the first word of the `ApiError` Debug repr (max 32 chars).
/// No token material or PII.
fn classify_error(err: &ApiError) -> String {
    let s = format!("{err:?}");
    let short = s.split_once('(').map(|(tag, _)| tag).unwrap_or(&s);
    short.chars().take(32).collect()
}

// ---------------------------------------------------------------------------
// Production entry point
// ---------------------------------------------------------------------------

/// Resolve Anthropic credentials via the auth broker and emit an SSB row.
///
/// This is the production-seam function, called from
/// `cli/src/runtime_cmd.rs:387` (`resolve_anthropic_auth_for_factory`).
///
/// `writer` is `None` in the current production path (WIRE-DARK[I-50]).
/// The contract test passes a real `Arc<Ssb>` to prove the emit path is reachable.
pub fn broker_resolve_anthropic_auth_with_ssb(
    writer: Option<Arc<Ssb>>,
) -> Result<AuthSource, ApiError> {
    match broker_resolve_anthropic_auth() {
        Ok((auth_source, outcome)) => {
            if let Some(ref ssb) = writer {
                emit_auth_broker_event(ssb, outcome_to_kind(outcome), "");
            }
            Ok(auth_source)
        }
        Err(err) => {
            let error_kind = classify_error(&err);
            if let Some(ref ssb) = writer {
                emit_auth_broker_event(ssb, AuthBrokerEventKind::RefreshFailed, &error_kind);
            }
            Err(err)
        }
    }
}
