//! `AnthropicApiRuntime` — adapter that wraps the existing
//! `AnthropicClient` (`providers/src/providers/anthropic.rs`).
//!
//! The wrapper translates between the multi-runtime envelope
//! (`Message` / `CompletionConfig` / `CompletionResponse`) and the
//! Anthropic-specific `MessageRequest` / `MessageResponse`. All
//! HTTP / retry / OAuth logic remains inside `AnthropicClient` --
//! that's the point of the wrapper.
//!
//! Streaming is wired through `AnthropicClient::stream_message`, with
//! `MessageStream::next_event` translated to `StreamChunk::TextDelta`
//! one delta at a time. Non-text events are mapped to `Warning`
//! chunks so the consumer can still pull tokens without us silently
//! dropping data.

use std::sync::Arc;

use futures_util::stream::{self, BoxStream, StreamExt};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::providers::anthropic::{AuthSource as AnthropicAuthSource, MessageStream};
use crate::types::{
    ContentBlockDelta, InputContentBlock, InputMessage, MessageRequest, OutputContentBlock,
    StreamEvent, SystemBlock,
};
use crate::AnthropicClient;

use super::types::{
    AuthSource, CompletionConfig, CompletionResponse, Message, MessageRole, RuntimeAdapter,
    RuntimeAdapterError, RuntimeBackend, RuntimeCapabilities, RuntimeFactoryConfig, RuntimeFuture,
    RuntimeHealth, RuntimeHealthFuture, StreamChunk,
};

const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-6";
const DEFAULT_MAX_TOKENS: u32 = 512;
const HEALTH_CANARY_MAX_TOKENS: u32 = 8;

/// Adapter wrapping `AnthropicClient`.
#[derive(Clone)]
pub struct AnthropicApiRuntime {
    client: Arc<AnthropicClient>,
    default_model: String,
    default_max_tokens: u32,
}

impl std::fmt::Debug for AnthropicApiRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnthropicApiRuntime")
            .field("default_model", &self.default_model)
            .field("default_max_tokens", &self.default_max_tokens)
            .finish()
    }
}

impl AnthropicApiRuntime {
    /// Build from `RuntimeFactoryConfig`. Requires one of:
    /// `cfg.auth_source` (preferred), or `cfg.api_key`.
    pub fn from_factory_config(cfg: &RuntimeFactoryConfig) -> Result<Self, RuntimeAdapterError> {
        let auth = if let Some(source) = cfg.auth_source.clone() {
            AnthropicAuthSource::from(source)
        } else if let Some(key) = cfg.api_key.clone() {
            AnthropicAuthSource::ApiKey(key.into_inner())
        } else {
            return Err(RuntimeAdapterError::MissingField {
                backend: RuntimeBackend::AnthropicApi,
                field: "auth_source",
                reason: "AnthropicApi runtime requires either auth_source or api_key".to_string(),
            });
        };

        let mut client = AnthropicClient::from_auth(auth);
        if let Some(base_url) = cfg.base_url.clone() {
            client = client.with_base_url(base_url);
        }

        let default_model = cfg
            .model
            .clone()
            .unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.to_string());

        Ok(Self {
            client: Arc::new(client),
            default_model,
            default_max_tokens: DEFAULT_MAX_TOKENS,
        })
    }

    /// Direct constructor for tests / call sites that already hold an
    /// `AnthropicClient` instance.
    #[must_use]
    pub fn from_client(client: AnthropicClient, model: Option<String>) -> Self {
        Self {
            client: Arc::new(client),
            default_model: model.unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.to_string()),
            default_max_tokens: DEFAULT_MAX_TOKENS,
        }
    }

    fn build_request(&self, messages: Vec<Message>, config: &CompletionConfig) -> MessageRequest {
        let model = config
            .model
            .clone()
            .unwrap_or_else(|| self.default_model.clone());

        let max_tokens = if config.max_tokens == 0 {
            self.default_max_tokens
        } else {
            config.max_tokens
        };

        let mut system = None;
        if let Some(text) = &config.system {
            system = Some(vec![SystemBlock::text(text.clone())]);
        }

        let mut wire_messages = Vec::with_capacity(messages.len());
        for msg in messages {
            let role = match msg.role {
                MessageRole::Assistant => "assistant",
                MessageRole::User | MessageRole::Tool => "user",
                // Anthropic's chat surface forbids `system` inside the
                // messages array; route to the `system` slot instead
                // (last system wins, to mirror the chat semantics).
                MessageRole::System => {
                    let blocks =
                        system.get_or_insert_with(|| vec![SystemBlock::text(String::new())]);
                    blocks.push(SystemBlock::text(msg.content));
                    continue;
                }
            };
            wire_messages.push(InputMessage {
                role: role.to_string(),
                content: vec![InputContentBlock::Text {
                    text: msg.content,
                    cache_control: None,
                }],
            });
        }

        MessageRequest {
            model,
            max_tokens,
            messages: wire_messages,
            system,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: config.seed,
        }
    }

    /// Build a 1-token canary request used by `health_check`.
    fn canary_request(&self) -> MessageRequest {
        MessageRequest {
            model: self.default_model.clone(),
            max_tokens: HEALTH_CANARY_MAX_TOKENS,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "ok".to_string(),
                    cache_control: None,
                }],
            }],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
            seed: None,
        }
    }
}

impl RuntimeAdapter for AnthropicApiRuntime {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::AnthropicApi
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities {
            supports_tools: true,
            supports_streaming: true,
            supports_vision: true,
            max_context: 200_000,
            max_output: 8_192,
            json_schema_mode: false,
            supports_prefill: true,
            supports_caching: true,
            native_thinking_budget: true,
        }
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        let client = Arc::clone(&self.client);
        let request = self.canary_request();
        Box::pin(async move {
            match client.send_message(&request).await {
                Ok(_) => RuntimeHealth::Ready,
                Err(err) => RuntimeHealth::Unavailable {
                    reason: format!("anthropic /v1/messages canary failed: {err}"),
                },
            }
        })
    }

    fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        let request = self.build_request(messages, &config);
        let client = Arc::clone(&self.client);
        Box::pin(async move {
            let response = client.send_message(&request).await?;
            let mut text = String::new();
            for block in &response.content {
                if let OutputContentBlock::Text { text: chunk } = block {
                    text.push_str(chunk);
                }
            }
            Ok(CompletionResponse {
                text,
                stop_reason: response.stop_reason.clone(),
                usage: (response.usage.input_tokens, response.usage.output_tokens),
                model: response.model,
            })
        })
    }

    fn stream(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
    ) -> BoxStream<'static, Result<StreamChunk, RuntimeAdapterError>> {
        let request = self.build_request(messages, &config).with_streaming();
        let client = Arc::clone(&self.client);

        // We need to await `stream_message` before emitting any chunks
        // so the upstream HTTP error (auth failure, etc.) surfaces as
        // a single Err item rather than panicking. We model that with
        // an async state machine via `stream::unfold`.
        let state = StreamState::Connecting {
            client,
            request: Box::new(request),
        };
        Box::pin(stream::unfold(state, |state| async move {
            match state {
                StreamState::Connecting { client, request } => {
                    match client.stream_message(&request).await {
                        Ok(stream) => {
                            // First chunk: warning that we're live (so
                            // consumers can use it as a sentinel if
                            // they want). Optional — disabled here.
                            Some((
                                Ok(StreamChunk::Warning {
                                    message: "stream connected".to_string(),
                                }),
                                StreamState::Streaming {
                                    stream: Arc::new(Mutex::new(stream)),
                                    done: false,
                                },
                            ))
                        }
                        Err(err) => Some((
                            Err(RuntimeAdapterError::Provider(err)),
                            StreamState::Terminated,
                        )),
                    }
                }
                StreamState::Streaming { stream, done } => {
                    if done {
                        return None;
                    }
                    let mut guard = stream.lock().await;
                    match guard.next_event().await {
                        Ok(Some(event)) => {
                            let mapped = map_event(&event);
                            drop(guard);
                            let new_state = StreamState::Streaming {
                                stream,
                                done: matches!(mapped, Ok(StreamChunk::Stop { .. })),
                            };
                            Some((mapped, new_state))
                        }
                        Ok(None) => Some((
                            Ok(StreamChunk::Stop { reason: None }),
                            StreamState::Terminated,
                        )),
                        Err(err) => Some((
                            Err(RuntimeAdapterError::Provider(err)),
                            StreamState::Terminated,
                        )),
                    }
                }
                StreamState::Terminated => None,
            }
        })
        .filter(|item| {
            // Drop the synthetic "stream connected" warning so we don't
            // pollute callers that just want text deltas. Errors and
            // real warnings still flow through.
            let keep = !matches!(
                item,
                Ok(StreamChunk::Warning { message }) if message == "stream connected"
            );
            async move { keep }
        }))
    }
}

/// Internal state for the streaming adapter. Held by `stream::unfold`.
enum StreamState {
    Connecting {
        client: Arc<AnthropicClient>,
        request: Box<MessageRequest>,
    },
    Streaming {
        stream: Arc<Mutex<MessageStream>>,
        done: bool,
    },
    Terminated,
}

fn map_event(event: &StreamEvent) -> Result<StreamChunk, RuntimeAdapterError> {
    match event {
        StreamEvent::ContentBlockDelta(delta_event) => match &delta_event.delta {
            ContentBlockDelta::TextDelta { text } => Ok(StreamChunk::TextDelta {
                text: text.clone(),
            }),
            ContentBlockDelta::InputJsonDelta { partial_json } => Ok(StreamChunk::Warning {
                message: format!("tool input delta: {partial_json}"),
            }),
            ContentBlockDelta::ThinkingDelta { thinking } => Ok(StreamChunk::Warning {
                message: format!("thinking delta ({} chars)", thinking.len()),
            }),
            ContentBlockDelta::SignatureDelta { signature: _ } => Ok(StreamChunk::Warning {
                message: "signature delta".to_string(),
            }),
        },
        StreamEvent::MessageDelta(msg_delta) => Ok(StreamChunk::Stop {
            reason: msg_delta.delta.stop_reason.clone(),
        }),
        StreamEvent::MessageStop(_) => Ok(StreamChunk::Stop { reason: None }),
        StreamEvent::MessageStart(_) => Ok(StreamChunk::Warning {
            message: "message start".to_string(),
        }),
        StreamEvent::ContentBlockStart(_) | StreamEvent::ContentBlockStop(_) => {
            Ok(StreamChunk::Warning {
                message: "content block boundary".to_string(),
            })
        }
    }
}

// Currently unused but keeps the public surface stable for B-37C.
#[allow(dead_code)]
fn json_text(value: &Value) -> Option<&str> {
    value.as_str()
}

impl AuthSource {
    /// Convenience for tests / call sites that already have a plain
    /// API key string.
    #[must_use]
    pub fn api_key_from_str(value: impl Into<String>) -> Self {
        Self::ApiKey(super::secret::SecretString::new(value.into()))
    }
}
