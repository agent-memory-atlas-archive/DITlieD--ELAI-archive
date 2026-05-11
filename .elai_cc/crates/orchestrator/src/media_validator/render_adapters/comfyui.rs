//! B-24 Phase 4I Group C6 (4I.9 IMPLEMENT): `ComfyUIRenderAdapter` — real impl.
//!
//! Spec phase-4I.txt L401/468-469 + workflow/papers/comfyui-api-interface.md.
//! HTTP client using `reqwest` (already a non-optional workspace dep
//! per Cargo.toml L23). Polling pattern (NOT WebSocket) is the
//! documented ComfyUI client surface.
//!
//! Endpoints:
//!   * POST /prompt          -> submit workflow.json; returns `{prompt_id}`
//!   * GET  /history/{id}    -> poll status; status.completed -> outputs
//!   * GET  /view?...        -> download a single image
//!
//! Anti-stale:
//!   * reqwest::Client with rustls-tls (no openssl) — matches existing
//!     workspace config.
//!   * tokio::time::sleep + tokio::time::timeout for polling.
//!   * serde_json::Value for the workflow payload (no schema lock-in).
//!   * u64::try_from for ms narrowing.

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tracing::{debug, warn};

use super::types::{CapturedFrame, FrameMetadata, OutputType, RenderConfig, Viewport};
use super::{MediaRenderAdapter, RenderAdapterError};

/// Default ComfyUI server URL — overridable per-instance.
pub const DEFAULT_COMFYUI_URL: &str = "http://127.0.0.1:8188";

/// Default per-call timeout (600 s — spec phase-4I.txt L555-557 mandates
/// 600s for AI-image renders; ComfyUI workflows commonly take 5-10 min
/// for high-step diffusion). The previous 60s was a stale carry-over
/// from the SKELETON; audit BLOCK-1 (decision 70) restores spec value.
pub const DEFAULT_COMFYUI_TIMEOUT: Duration = Duration::from_secs(600);

/// HTTP endpoints (paper-grounded; see comfyui-api-interface.md).
pub const COMFYUI_ENDPOINT_PROMPT: &str = "/prompt";
pub const COMFYUI_ENDPOINT_HISTORY: &str = "/history";
pub const COMFYUI_ENDPOINT_VIEW: &str = "/view";

const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub struct ComfyUIRenderAdapter {
    pub api_url: String,
    pub timeout: Duration,
}

impl Default for ComfyUIRenderAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ComfyUIRenderAdapter {
    pub fn new() -> Self {
        Self {
            api_url: DEFAULT_COMFYUI_URL.to_string(),
            timeout: DEFAULT_COMFYUI_TIMEOUT,
        }
    }

    pub fn with_url(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            timeout: DEFAULT_COMFYUI_TIMEOUT,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Deserialize)]
struct PromptResponse {
    prompt_id: String,
}

fn make_tempdir(prefix: &str) -> Result<PathBuf, RenderAdapterError> {
    let base = std::env::temp_dir();
    let id = uuid::Uuid::new_v4();
    let dir = base.join(format!("{prefix}-{id}"));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

impl MediaRenderAdapter for ComfyUIRenderAdapter {
    fn can_handle(&self, output_type: &OutputType) -> bool {
        matches!(output_type, OutputType::AiGenerated)
    }

    fn render<'a>(
        &'a self,
        artifact: &'a Path,
        config: &'a RenderConfig,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<CapturedFrame>, RenderAdapterError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let started = Instant::now();

            // 1. Read + parse the workflow JSON from the artifact path.
            let workflow_bytes = std::fs::read(artifact).map_err(|_| {
                RenderAdapterError::ArtifactPathInvalid {
                    path: artifact.to_path_buf(),
                }
            })?;
            let workflow_json: serde_json::Value = serde_json::from_slice(&workflow_bytes)
                .map_err(|e| RenderAdapterError::MalformedOutput(
                    format!("workflow JSON parse failed: {e}"),
                ))?;

            let work_dir = make_tempdir("elai-comfyui-render")?;

            // Per-call timeout: prefer `config.timeout` when nonzero else
            // `self.timeout`. Fallback narrows to DEFAULT_COMFYUI_TIMEOUT
            // when `try_from` overflows (decision 70: 600s default).
            let default_ms = u64::try_from(DEFAULT_COMFYUI_TIMEOUT.as_millis())
                .unwrap_or(600_000);
            let timeout_ms = if config.timeout.as_millis() == 0 {
                u64::try_from(self.timeout.as_millis()).unwrap_or(default_ms)
            } else {
                u64::try_from(config.timeout.as_millis()).unwrap_or(default_ms)
            };
            let timeout = Duration::from_millis(timeout_ms);

            // 2. Build the reqwest client.
            let client = reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|e| RenderAdapterError::MalformedOutput(format!(
                    "reqwest client build: {e}"
                )))?;

            // 3. POST /prompt with the workflow payload. ComfyUI accepts
            //    either {"prompt": <workflow>} OR raw workflow at top level
            //    depending on version; we wrap conservatively.
            let post_body = if workflow_json.get("prompt").is_some() {
                workflow_json.clone()
            } else {
                serde_json::json!({ "prompt": workflow_json })
            };
            let prompt_url = format!("{}{}", self.api_url, COMFYUI_ENDPOINT_PROMPT);
            debug!(comfyui_url = %prompt_url, "POST /prompt");
            let post_res = tokio::time::timeout(timeout, async {
                client.post(&prompt_url).json(&post_body).send().await
            })
            .await
            .map_err(|_| RenderAdapterError::Timeout { millis: timeout_ms })?
            .map_err(|e| {
                warn!(error = %e, "ComfyUI POST /prompt failed");
                RenderAdapterError::ToolUnavailable { tool: "comfyui" }
            })?;
            if !post_res.status().is_success() {
                return Err(RenderAdapterError::MalformedOutput(format!(
                    "ComfyUI /prompt status={}",
                    post_res.status()
                )));
            }
            let post_json: PromptResponse = post_res.json().await.map_err(|e| {
                RenderAdapterError::MalformedOutput(format!("/prompt body: {e}"))
            })?;
            let prompt_id = post_json.prompt_id;

            // 4. Poll GET /history/{id} until status.completed OR timeout.
            let history_url = format!(
                "{}{}/{}",
                self.api_url, COMFYUI_ENDPOINT_HISTORY, prompt_id
            );
            let deadline = Instant::now() + timeout;
            let outputs = loop {
                if Instant::now() >= deadline {
                    return Err(RenderAdapterError::Timeout { millis: timeout_ms });
                }
                let r = match client.get(&history_url).send().await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!(error = %e, "ComfyUI GET /history transient error");
                        tokio::time::sleep(POLL_INTERVAL).await;
                        continue;
                    }
                };
                if !r.status().is_success() {
                    tokio::time::sleep(POLL_INTERVAL).await;
                    continue;
                }
                let body: serde_json::Value = match r.json().await {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "ComfyUI /history body parse");
                        tokio::time::sleep(POLL_INTERVAL).await;
                        continue;
                    }
                };
                // The history endpoint returns either {} (not yet) or
                // { "<prompt_id>": { "outputs": { "<node_id>": { "images": [...] } } } }
                if let Some(entry) = body.get(&prompt_id) {
                    if let Some(outs) = entry.get("outputs") {
                        // Treat presence-of-outputs as completion.
                        break outs.clone();
                    }
                }
                tokio::time::sleep(POLL_INTERVAL).await;
            };

            // 5. Extract image entries: { node_id: { "images": [{filename, subfolder, type}] } }.
            let mut frames: Vec<CapturedFrame> = Vec::new();
            let viewport = config.viewports.first().cloned().unwrap_or_else(|| Viewport {
                width: 512,
                height: 512,
                device: Some("ai-generated".into()),
            });
            if let Some(obj) = outputs.as_object() {
                for (_node_id, node_out) in obj.iter() {
                    let imgs = match node_out.get("images").and_then(|v| v.as_array()) {
                        Some(a) => a,
                        None => continue,
                    };
                    for img in imgs {
                        let filename = img
                            .get("filename")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let subfolder = img
                            .get("subfolder")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let typ = img.get("type").and_then(|v| v.as_str()).unwrap_or("output");
                        if filename.is_empty() {
                            continue;
                        }
                        let view_url = format!(
                            "{}{}?filename={}&subfolder={}&type={}",
                            self.api_url,
                            COMFYUI_ENDPOINT_VIEW,
                            urlencode(filename),
                            urlencode(subfolder),
                            urlencode(typ),
                        );
                        let view_res = match client.get(&view_url).send().await {
                            Ok(r) => r,
                            Err(e) => {
                                warn!(error = %e, "ComfyUI GET /view failed");
                                continue;
                            }
                        };
                        if !view_res.status().is_success() {
                            continue;
                        }
                        let bytes = match view_res.bytes().await {
                            Ok(b) => b,
                            Err(_) => continue,
                        };
                        let out_path = work_dir.join(filename);
                        if let Err(e) = std::fs::write(&out_path, &bytes) {
                            warn!(error = %e, path = %out_path.display(), "write image bytes");
                            continue;
                        }
                        frames.push(CapturedFrame {
                            image_path: out_path,
                            viewport: viewport.clone(),
                            metadata: FrameMetadata {
                                capture_time: started.elapsed(),
                                adapter_name: "comfyui".to_string(),
                                state_description: format!("ai_generated_{filename}"),
                            },
                        });
                    }
                }
            }

            if frames.is_empty() {
                return Err(RenderAdapterError::MalformedOutput(
                    "ComfyUI returned no images".into(),
                ));
            }

            Ok(frames)
        })
    }

    fn name(&self) -> &'static str {
        "comfyui"
    }
}

/// Tiny URL-encoder for the 4 characters that appear in ComfyUI's
/// filename/subfolder/type query params. We don't pull in `percent-encoding`
/// because we have only 4 characters to escape.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            ' ' => out.push_str("%20"),
            '/' => out.push_str("%2F"),
            '\\' => out.push_str("%5C"),
            _ => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf).as_bytes();
                for b in bytes {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_only_ai_generated() {
        let a = ComfyUIRenderAdapter::new();
        assert!(a.can_handle(&OutputType::AiGenerated));
        assert!(!a.can_handle(&OutputType::Web));
    }

    #[test]
    fn default_url_is_local_loopback() {
        let a = ComfyUIRenderAdapter::new();
        assert_eq!(a.api_url, "http://127.0.0.1:8188");
        assert_eq!(a.timeout, Duration::from_secs(600));
    }

    #[test]
    fn endpoints_match_paper_grounded_routes() {
        assert_eq!(COMFYUI_ENDPOINT_PROMPT, "/prompt");
        assert_eq!(COMFYUI_ENDPOINT_HISTORY, "/history");
        assert_eq!(COMFYUI_ENDPOINT_VIEW, "/view");
    }

    #[test]
    fn urlencode_handles_safe_and_unsafe() {
        assert_eq!(urlencode("hello.png"), "hello.png");
        assert_eq!(urlencode("a b/c"), "a%20b%2Fc");
        assert_eq!(urlencode("AaZz09-_.~"), "AaZz09-_.~");
    }

    #[tokio::test]
    async fn render_errors_when_artifact_missing() {
        let a = ComfyUIRenderAdapter::with_url("http://127.0.0.1:1");
        let cfg = RenderConfig::default();
        let r = a.render(Path::new("/nonexistent-workflow.json"), &cfg).await;
        assert!(matches!(
            r,
            Err(RenderAdapterError::ArtifactPathInvalid { .. })
        ));
    }
}
