//! B-39D — bridge module wiring the chat REFLECT pipeline
//! (`elai_reasoning::ChatReflectPipeline`) into the dashboard's chat-stream
//! command.
//!
//! Two trait impls live here so `commands.rs` can stay focused on routing:
//!
//!   1. `HttpInferenceClient` — implements `ChatInferenceClient`. Each
//!      `complete(prompt, max_tokens)` call does a non-streaming POST to
//!      the configured `/v1/messages` endpoint (local sidecar or
//!      api.anthropic.com) and returns the assistant text from
//!      `content[0].text`. Each call is independent — no conversation
//!      state is shared across stages (matches the REASONER N best-of-N
//!      design that wants fresh KV caches per call).
//!
//!   2. `TauriEventSink` — implements `ReflectEventSink`. Each pipeline
//!      stage event becomes a Tauri event `reflect-progress:{stream_id}`
//!      so the frontend can render a live timeline (TRIAGE running →
//!      CANDIDATE 1/N → … → AUDIT verdict → REFINED done). Mirrors
//!      `start_chat_stream`'s existing `chat-stream:{id}` pattern.
//!
//! Anti-stale grounding:
//!   - `tauri::Emitter::emit` (Tauri 2.x), not v1's `Window::emit`.
//!   - `tokio::sync` types only where used; no async-trait crate
//!     (native `Pin<Box<Future>>` per the trait definition).
//!   - `reqwest` reused from `AppState.http`.

use elai_reasoning::{
    ChatFuture, ChatInferenceClient, ReflectError, ReflectEvent, ReflectEventSink,
};
use serde_json::json;
use std::sync::Arc;
use tauri::Emitter;

/// Non-streaming HTTP client suitable for REFLECT's stage prompts. Each
/// `complete()` POST is independent. Caller pre-builds the URL + headers
/// so this struct stays free of routing logic.
pub struct HttpInferenceClient {
    pub client: reqwest::Client,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub model_id: String,
}

impl HttpInferenceClient {
    pub fn new(
        client: reqwest::Client,
        url: String,
        headers: Vec<(String, String)>,
        model_id: String,
    ) -> Self {
        Self {
            client,
            url,
            headers,
            model_id,
        }
    }
}

impl ChatInferenceClient for HttpInferenceClient {
    fn complete<'a>(&'a self, prompt: &'a str, max_tokens: u32) -> ChatFuture<'a, String> {
        Box::pin(async move {
            let body = json!({
                "model": self.model_id,
                "max_tokens": max_tokens,
                "stream": false,
                "messages": [{ "role": "user", "content": prompt }],
            });
            let mut req = self.client.post(&self.url).json(&body);
            for (k, v) in &self.headers {
                req = req.header(k.as_str(), v.as_str());
            }
            let resp = req
                .send()
                .await
                .map_err(|e| ReflectError::Inference(format!("http send: {e}")))?;
            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(ReflectError::Inference(format!(
                    "http {} {}",
                    status.as_u16(),
                    body
                )));
            }
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| ReflectError::Parse(format!("json: {e}")))?;
            // Anthropic /v1/messages shape: { content: [{ type:"text", text:"…" }] }
            json.pointer("/content/0/text")
                .and_then(|v| v.as_str())
                .map(String::from)
                .ok_or_else(|| ReflectError::Parse("missing /content/0/text".to_owned()))
        })
    }
}

/// Forwards every REFLECT stage event as a Tauri event tagged with the
/// stream_id, so the frontend ReasoningDrawer can render live progress
/// without polling SSB.
///
/// Event name: `reflect-progress:{stream_id}` (one channel per chat turn).
/// Payload shape is a flat object keyed by `stage` so the frontend can
/// switch on a single field.
pub struct TauriEventSink {
    app: tauri::AppHandle,
    stream_id: String,
}

impl TauriEventSink {
    pub fn new(app: tauri::AppHandle, stream_id: String) -> Self {
        Self { app, stream_id }
    }

    pub fn into_arc(self) -> Arc<dyn ReflectEventSink> {
        Arc::new(self)
    }
}

impl ReflectEventSink for TauriEventSink {
    fn emit(&self, event: ReflectEvent) {
        let mut payload = match &event {
            ReflectEvent::Triaged { turn_id, class } => json!({
                "stage": "triage",
                "turn_id": turn_id,
                "class": class.as_wire_str(),
            }),
            ReflectEvent::Candidate {
                turn_id,
                candidate_index,
                content,
            } => json!({
                "stage": "candidate",
                "turn_id": turn_id,
                "candidate_index": candidate_index,
                "content_preview": content.chars().take(160).collect::<String>(),
            }),
            ReflectEvent::AuditVerdict {
                turn_id,
                best_index,
                rationale,
            } => json!({
                "stage": "audit_verdict",
                "turn_id": turn_id,
                "best_index": best_index,
                "rationale_preview": rationale.chars().take(160).collect::<String>(),
            }),
            ReflectEvent::Refined {
                turn_id,
                total_tokens_used,
                synthesis,
            } => json!({
                "stage": "refined",
                "turn_id": turn_id,
                "total_tokens_used": total_tokens_used,
                "synthesis_preview": synthesis.chars().take(160).collect::<String>(),
            }),
            ReflectEvent::BudgetSoftWarn {
                turn_id,
                used,
                soft_cap,
                hard_cap,
                pct,
            } => json!({
                "stage": "budget_soft_warn",
                "turn_id": turn_id,
                "used": used,
                "soft_cap": soft_cap,
                "hard_cap": hard_cap,
                "pct": pct,
            }),
            // J-42 T2: distill-reset checkpoint stage beacon.
            ReflectEvent::ReasoningCheckpoint {
                turn_id,
                trigger,
                distillate_tokens,
                uncertainty_preserved,
                pass_index,
                outcome,
            } => json!({
                "stage": "reasoning_checkpoint",
                "turn_id": turn_id,
                "trigger": trigger.as_wire_str(),
                "distillate_tokens": distillate_tokens,
                "uncertainty_preserved": uncertainty_preserved,
                "pass_index": pass_index,
                "outcome": outcome.as_wire_str(),
            }),
        };
        // Tag the stream_id on the payload so the singleton-channel
        // ReasoningDrawer can filter / clear on turn boundaries.
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "stream_id".to_string(),
                serde_json::Value::String(self.stream_id.clone()),
            );
        }
        // Per-stream channel (CommandWindow can subscribe if it cares about
        // its own turn's transitions).
        let _ = self
            .app
            .emit(&format!("reflect-progress:{}", self.stream_id), &payload);
        // Singleton channel (ReasoningDrawer subscribes once for the
        // currently-running turn's timeline).
        let _ = self.app.emit("reflect-progress", &payload);
    }
}

/// Map a frontend effort level string ("low"|"medium"|"high"|"max") to
/// REASONER best-of-N width. Floor at N=2 (skipping N=1 since that would
/// erase the best-of-N purpose); ceiling at N=8 (latency safety on local
/// 3B/7B inference per `project_qwen3b_baseline.md`). Default N=4 mirrors
/// `N_CANDIDATES` and matches the leaked-source guided-refinement Pareto
/// elbow Liu et al. arXiv:2502.06703 Table 5.
pub fn effort_to_n_candidates(effort: Option<&str>) -> u32 {
    match effort.map(|s| s.to_ascii_lowercase()) {
        Some(ref s) if s == "low" => 2,
        Some(ref s) if s == "medium" => 4,
        Some(ref s) if s == "high" => 6,
        Some(ref s) if s == "max" => 8,
        _ => elai_reasoning::N_CANDIDATES,
    }
}
