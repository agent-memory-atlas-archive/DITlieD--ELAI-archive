//! Batch 4C (A-23) — Micro-reflection emitter.
//!
//! After a baby-step failure the task_runner writes a `MistakeEntry` row
//! (see `task_runner::record_mistake_on_failure`). This module adds the
//! second A-23 half: calling a local LLM for a 500-token constrained JSON
//! reflection (`root_cause`, `faulty_line`, `action`) and recording a
//! `MicroReflection` SSB row + a mirrored `MistakeEntry` with
//! `action_taken = "micro_reflection"`.
//!
//! The heavy lifting (prompt assembly, JSON schema, response parsing) lives
//! in [`elai_core::micro_reflection`]. This file provides only the glue:
//! call the client, parse the response, persist to SSB. The emitter is
//! infallible from the caller's point of view — client errors produce a
//! log row and `None`, matching the "runner stays alive on IO failure"
//! rule.
//!
//! The emitter is decoupled from [`crate::grounding_client::GroundingClient`]
//! because that trait returns `ResearchReport` (tree-shaped, multi-alternative);
//! micro-reflections are a single JSON response. We take a narrower
//! `ReflectionClient` trait so tests can inject canned JSON without
//! constructing a synthetic research report.

use std::sync::Arc;

use elai_core::micro_reflection::{
    parse_response as parse_reflection_response, MicroReflectionRequest, MicroReflectionResponse,
    ReflectionError,
};
use elai_core::mistake_book::FailureCategory;
use tracing::{debug, warn};

use crate::mistake_book::write_mistake as mistake_book_write_mistake;
use crate::ssb::{MicroReflectionPayload, Ssb, SsbPayload};

/// Errors returned by a [`ReflectionClient`].
#[derive(Debug, thiserror::Error)]
pub enum ReflectionClientError {
    #[error("client unavailable: {0}")]
    Unavailable(String),
    #[error("response parse failed: {0}")]
    Parse(#[from] ReflectionError),
    #[error("transport error: {0}")]
    Transport(String),
}

/// Narrow client trait for the A-23 micro-reflection call.
///
/// Implementations: production wraps `SidecarGroundingClient` (composition,
/// not trait-sharing); tests provide a canned-JSON stub.
///
/// The `impl Future` return type is not dyn-compatible; use
/// [`DynReflectionClient`] when an `Arc<dyn ...>` storage is required (e.g.
/// on `TaskRunnerConfig`). [`DynReflectionClient`] has a blanket impl over
/// every `ReflectionClient` so downstream types that only implement this
/// trait pick the object-safe adapter for free.
pub trait ReflectionClient: Send + Sync {
    fn reflect(
        &self,
        prompt: &str,
    ) -> impl std::future::Future<Output = Result<MicroReflectionResponse, ReflectionClientError>>
           + Send;
}

/// Object-safe mirror of [`ReflectionClient`]. Returns a boxed future so
/// the trait can be stored behind `Arc<dyn DynReflectionClient>` on the
/// runner config. The blanket impl below means every `ReflectionClient`
/// automatically satisfies `DynReflectionClient` — call sites pass either
/// interchangeably and [`emit_micro_reflection`] accepts both via the
/// `?Sized` bound.
pub trait DynReflectionClient: Send + Sync {
    fn reflect_boxed<'a>(
        &'a self,
        prompt: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<MicroReflectionResponse, ReflectionClientError>>
                + Send
                + 'a,
        >,
    >;
}

impl<T: ReflectionClient + ?Sized> DynReflectionClient for T {
    fn reflect_boxed<'a>(
        &'a self,
        prompt: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<MicroReflectionResponse, ReflectionClientError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.reflect(prompt))
    }
}

/// Wrapper that makes any [`DynReflectionClient`] usable where a concrete
/// [`ReflectionClient`] is expected. Call sites receiving an
/// `Arc<dyn DynReflectionClient>` wrap it via `DynReflectionAdapter::new`
/// and pass the adapter reference to [`emit_micro_reflection`].
pub struct DynReflectionAdapter<'a> {
    inner: &'a (dyn DynReflectionClient + 'a),
}

impl<'a> DynReflectionAdapter<'a> {
    pub fn new(inner: &'a (dyn DynReflectionClient + 'a)) -> Self {
        Self { inner }
    }
}

impl<'a> ReflectionClient for DynReflectionAdapter<'a> {
    async fn reflect(
        &self,
        prompt: &str,
    ) -> Result<MicroReflectionResponse, ReflectionClientError> {
        self.inner.reflect_boxed(prompt).await
    }
}

/// Invoke the reflection client to diagnose a baby-step failure and record
/// the reflection. Returns the parsed response on success; `None` when the
/// client is unreachable or the response fails validation.
pub async fn emit_micro_reflection<C: ReflectionClient + ?Sized>(
    writer: &Arc<Ssb>,
    client: &C,
    agent_id: &str,
    superstep: i64,
    baby_step_id: &str,
    goal: &str,
    code_snippet: &str,
    error: &str,
) -> Option<MicroReflectionResponse> {
    let request = MicroReflectionRequest::assemble(goal, code_snippet, error, &[]);
    let prompt = request.to_prompt();
    debug!(
        baby_step_id,
        prompt_bytes = prompt.len(),
        "micro_reflection: calling client"
    );
    let response = match client.reflect(&prompt).await {
        Ok(r) => r,
        Err(e) => {
            warn!(
                baby_step_id,
                error = %e,
                "micro_reflection: client call failed"
            );
            return None;
        }
    };
    // Mirror the reflection into the MistakeBook via the existing entry
    // writer so consumers querying MistakeBook see the `micro_reflection`
    // action label.
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let diff_seed = format!("{baby_step_id}|micro_reflection");
    let entry =
        response.to_mistake_entry(0, &diff_seed, FailureCategory::LogicError, timestamp);
    if let Err(e) = mistake_book_write_mistake(
        writer.as_ref(),
        agent_id,
        superstep,
        baby_step_id,
        entry,
    )
    .await
    {
        warn!(
            baby_step_id,
            error = %e,
            "micro_reflection: MistakeBook mirror write failed"
        );
    }
    // Emit the dedicated SSB row so the dashboard can render a purple
    // reflection line without scanning MistakeEntry rows. Confidence is
    // synthesized from the validated response (1.0 since validate() passed).
    let payload = SsbPayload::MicroReflection(MicroReflectionPayload {
        step_id: baby_step_id.to_string(),
        root_cause: response.root_cause.clone(),
        proposed_fix: response.action.clone(),
        confidence: 1.0,
    });
    if let Err(e) = writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        warn!(
            baby_step_id,
            error = %e,
            "micro_reflection: SSB write failed"
        );
        return None;
    }
    Some(response)
}

/// Parse a raw JSON string into a [`MicroReflectionResponse`] via the
/// core validator. Re-exported so callers don't need a direct
/// `elai_core::micro_reflection` dependency.
pub fn parse_reflection_json(raw: &str) -> Result<MicroReflectionResponse, ReflectionError> {
    parse_reflection_response(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CannedClient {
        json: String,
    }

    impl ReflectionClient for CannedClient {
        async fn reflect(
            &self,
            _prompt: &str,
        ) -> Result<MicroReflectionResponse, ReflectionClientError> {
            Ok(parse_reflection_response(&self.json)?)
        }
    }

    #[tokio::test]
    async fn parse_reflection_json_validates_shape() {
        let raw = r#"{"root_cause":"missing return type","faulty_line":3,"action":"add return i32"}"#;
        let parsed = parse_reflection_json(raw).expect("parse");
        assert_eq!(parsed.faulty_line, 3);
    }

    #[tokio::test]
    async fn canned_client_round_trips() {
        let raw = r#"{"root_cause":"missing return","faulty_line":2,"action":"add return"}"#;
        let client = CannedClient {
            json: raw.to_string(),
        };
        let r = client.reflect("dummy").await.expect("reflect");
        assert_eq!(r.root_cause, "missing return");
    }
}
