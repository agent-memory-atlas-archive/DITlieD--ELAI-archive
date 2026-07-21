//! J-33 Tier-1: HTTP client for an external MinerU `serve` endpoint.
//!
//! Default-OFF behind [`DOC_PARSER_ENABLE_ENV`]. When disabled, the artifact
//! indexer's `.pdf`/`.docx` branch is never taken and today's behavior
//! (read_to_string + binary skip) is preserved. When enabled with a
//! configured `base_url`, [`DocParserClient::parse`] POSTs the file as
//! multipart to `/file_parse` and returns the markdown body the MinerU
//! `serve` shape emits. The result feeds [`crate::chunking::pdf_structured`]
//! via [`PdfStructuredChunker::finalize`] so the existing
//! `ParserBackend::MinerU` / `check_backend_eligibility` /
//! `compute_fallback_recommended` symbols gain a real production caller.
//!
//! This module is the Tier-1 HTTP leg only. It does NOT boot a GPU sidecar,
//! vendor MinerU, or substitute a local Qwen path. When the service is down
//! or the flag is unset, the caller surfaces an explicit
//! [`DocParserError::ServiceUnavailable`] / [`DocParserError::Disabled`]
//! so the indexer can record the failure instead of silently reading raw
//! PDF bytes as text.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::chunking::pdf_structured::{
    compute_fallback_recommended, ParseError, ParserBackend, PdfStructuredChunker,
    PdfStructuredOutput, DEFAULT_CLOUD_FALLBACK_THRESHOLD,
};

/// Env-gate for the J-33 doc-parser lane. Default-OFF; truthy words "1",
/// "true", "on" (case-insensitive) enable it. Mirrors the
/// `ELAI_AGENTIC_GREP_REPAIR_ENABLE_ENV` pattern in `agentic_grep.rs`.
pub const DOC_PARSER_ENABLE_ENV: &str = "ELAI_DOC_PARSER_ENABLE";

/// Env var for the MinerU `serve` base URL (e.g. `http://127.0.0.1:8760`).
/// Required when [`DOC_PARSER_ENABLE_ENV`] is truthy; absent => Disabled.
pub const DOC_PARSER_BASE_URL_ENV: &str = "ELAI_DOC_PARSER_BASE_URL";

/// Per-page confidence the MinerU `serve` shape reports when it does not
/// emit a per-page confidence array. The Tier-1 wire does not parse the
/// full content_list.json; it records a single placeholder page so
/// `compute_fallback_recommended` has a deterministic input. Tier-2 will
/// replace this with the real per-page confidences.
const PLACEHOLDER_PAGE_CONFIDENCE: f32 = 1.0;

/// HTTP timeout for the parse call. MinerU `serve` can take minutes on a
/// dense PDF, so the default is generous. Tunable in a later slice via
/// `[knowledge.doc_parser] request_timeout_secs`.
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 300;

/// Read [`DOC_PARSER_ENABLE_ENV`] and return `true` when the gate is on.
/// Default (unset / unrecognized) = `false`.
pub fn doc_parser_enabled_from_env() -> bool {
    match std::env::var(DOC_PARSER_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Read [`DOC_PARSER_BASE_URL_ENV`] and return the configured base URL when
/// present and non-empty. Trims trailing `/` so callers can join
/// `/file_parse` without producing `//file_parse`.
pub fn doc_parser_base_url_from_env() -> Option<String> {
    match std::env::var(DOC_PARSER_BASE_URL_ENV) {
        Ok(v) => {
            let t = v.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.trim_end_matches('/').to_string())
            }
        }
        Err(_) => None,
    }
}

/// Honest status snapshot for the doc-parser lane. Read from env only; no
/// HTTP probe, no side effect. The dashboard surfaces this via the
/// `get_doc_parser_status` Tauri command so the KnowledgeDrawer can render
/// an OFF / MISCONFIGURED / READY badge without inventing capability.
///
/// `lane`:
/// - `"off"`: flag unset (today's behavior; drawer must say OFF, no phantom)
/// - `"misconfigured"`: flag on but base_url missing
/// - `"ready"`: flag on and base_url configured (service reachability NOT
///   probed here; the badge says READY, not "live")
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocParserStatus {
    pub enabled: bool,
    pub base_url_configured: bool,
    pub backend: &'static str,
    pub lane: &'static str,
}

/// Build a [`DocParserStatus`] from env vars only. No HTTP, no panic.
#[must_use]
pub fn status_from_env() -> DocParserStatus {
    let enabled = doc_parser_enabled_from_env();
    let base_url_configured = doc_parser_base_url_from_env().is_some();
    let lane: &'static str = if !enabled {
        "off"
    } else if !base_url_configured {
        "misconfigured"
    } else {
        "ready"
    };
    DocParserStatus {
        enabled,
        base_url_configured,
        backend: "mineru",
        lane,
    }
}

/// Errors emitted by the doc-parser lane. Distinct from
/// [`crate::artifacts::manifest::ArtifactError`] so the indexer can map
/// them to `ArtifactError::Other` without losing the original cause.
#[derive(Debug, thiserror::Error)]
pub enum DocParserError {
    #[error("doc_parser: flag disabled (ELAI_DOC_PARSER_ENABLE not set)")]
    Disabled,
    #[error("doc_parser: base_url not configured (ELAI_DOC_PARSER_BASE_URL_ENV unset)")]
    BaseUrlMissing,
    #[error("doc_parser: service unavailable at {url}: {source}")]
    ServiceUnavailable {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("doc_parser: non-2xx response from {url}: status={status}")]
    Non2xx { url: String, status: u16 },
    #[error("doc_parser: malformed response body: {0}")]
    MalformedBody(String),
    #[error("doc_parser: io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("doc_parser: backend eligibility rejected: {0}")]
    Eligibility(#[from] ParseError),
}

/// MinerU 3.4 `serve` `/file_parse` response entry under `results[<key>]`.
/// The Tier-1 wire reads `md_content` (MinerU 3.4 shape) and falls back to
/// `markdown` / `content` for older builds that emit those fields per-entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinerUResultEntry {
    /// MinerU 3.4 parsed markdown body. Preferred.
    #[serde(default)]
    pub md_content: String,
    /// Older per-entry markdown field name.
    #[serde(default)]
    pub markdown: String,
    /// Legacy per-entry content field name.
    #[serde(default)]
    pub content: String,
}

/// MinerU `serve` `/file_parse` response shape. MinerU 3.4 emits
/// `results[<file_key>].md_content`; older builds emitted top-level
/// `markdown` / `content`. The adapter prefers the 3.4 shape and falls back
/// to the legacy top-level fields for back-compat.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinerUFileParseResponse {
    /// MinerU 3.4 shape: map of `<file_key>` -> entry with `md_content`.
    /// Empty map on older builds that emit top-level `markdown`/`content`.
    #[serde(default)]
    pub results: std::collections::BTreeMap<String, MinerUResultEntry>,
    /// Legacy top-level markdown field (older MinerU builds).
    #[serde(default)]
    pub markdown: String,
    /// Legacy top-level content field name some MinerU builds emit.
    #[serde(default)]
    pub content: String,
}

impl MinerUFileParseResponse {
    /// Return the parsed markdown body, preferring `results[*].md_content`
    /// (MinerU 3.4), then per-entry `markdown`/`content`, then legacy
    /// top-level `markdown`/`content`.
    #[must_use]
    pub fn body(&self) -> &str {
        for (_key, entry) in &self.results {
            if !entry.md_content.is_empty() {
                return &entry.md_content;
            }
            if !entry.markdown.is_empty() {
                return &entry.markdown;
            }
            if !entry.content.is_empty() {
                return &entry.content;
            }
        }
        if !self.markdown.is_empty() {
            &self.markdown
        } else {
            &self.content
        }
    }
}

/// HTTP client against a MinerU `serve` endpoint. Cheap to clone; the
/// inner `reqwest::blocking::Client` is connection-pooled.
#[derive(Debug, Clone)]
pub struct DocParserClient {
    base_url: String,
    backend: ParserBackend,
    mode_allows_cloud: bool,
    timeout: Duration,
    http: reqwest::blocking::Client,
}

impl DocParserClient {
    /// Construct from environment variables. Returns `None` when the flag
    /// is off (the caller's no-op path). Returns `Err(Disabled)` only via
    /// [`Self::from_env_or_disable`] when an explicit Disabled signal is
    /// needed; this constructor is the indexer's preferred entrypoint.
    pub fn from_env() -> Option<Result<Self, DocParserError>> {
        if !doc_parser_enabled_from_env() {
            return None;
        }
        Some(Self::from_env_or_disable())
    }

    /// Construct assuming the flag is already on. Returns
    /// `Err(Disabled)` when the env var is unset, and
    /// `Err(BaseUrlMissing)` when the base URL is not configured.
    pub fn from_env_or_disable() -> Result<Self, DocParserError> {
        if !doc_parser_enabled_from_env() {
            return Err(DocParserError::Disabled);
        }
        let base_url = doc_parser_base_url_from_env()
            .ok_or(DocParserError::BaseUrlMissing)?;
        Ok(Self::new(base_url, ParserBackend::MinerU, false))
    }

    /// Build a client against `base_url` with an explicit backend + cloud
    /// mode. `mode_allows_cloud=false` is the Tier-1 default (MinerU is
    /// local); passing true is a Tier-2 hook.
    #[must_use]
    pub fn new(base_url: String, backend: ParserBackend, mode_allows_cloud: bool) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self {
            base_url,
            backend,
            mode_allows_cloud,
            timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            http,
        }
    }

    /// Override the request timeout. Builder-style; used by tests to keep
    /// the suite fast.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
        {
            self.http = client;
        }
        self.timeout = timeout;
        self
    }

    /// Return the configured base URL. Exposed so the indexer can include
    /// it in error telemetry without re-reading env vars.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Return the parser backend this client drives.
    #[must_use]
    pub fn backend(&self) -> ParserBackend {
        self.backend
    }

    /// POST `abs_path` to `/file_parse` as multipart, parse the JSON
    /// response, and build a [`PdfStructuredOutput`] via
    /// [`PdfStructuredChunker::finalize`]. The eligibility check runs
    /// first so a cloud backend in local mode fails fast with
    /// [`ParseError::CloudRequired`] instead of a network round-trip.
    ///
    /// The placeholder confidence array is a Tier-1 simplification: the
    /// MinerU `serve` JSON shape does not surface per-page confidences in
    /// the markdown-only response, so Tier-1 records a single high
    /// confidence page and lets `compute_fallback_recommended` return
    /// `false` deterministically. Tier-2 will parse `content_list.json`
    /// for real per-page confidences.
    pub fn parse(&self, abs_path: &Path) -> Result<PdfStructuredOutput, DocParserError> {
        PdfStructuredChunker::check_backend_eligibility(self.backend, self.mode_allows_cloud)?;

        let file_bytes = std::fs::read(abs_path).map_err(|source| DocParserError::Io {
            path: abs_path.to_path_buf(),
            source,
        })?;
        let file_name = abs_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document")
            .to_string();

        let form = reqwest::blocking::multipart::Form::new()
            .part(
                "files",
                reqwest::blocking::multipart::Part::bytes(file_bytes)
                    .file_name(file_name),
            )
            .text("backend", "pipeline");

        let url = format!("{}/file_parse", self.base_url);
        let resp = self
            .http
            .post(&url)
            .multipart(form)
            .send()
            .map_err(|source| DocParserError::ServiceUnavailable {
                url: url.clone(),
                source,
            })?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(DocParserError::Non2xx { url, status });
        }
        let body: MinerUFileParseResponse = resp
            .json()
            .map_err(|e| DocParserError::MalformedBody(e.to_string()))?;

        let markdown = body.body().to_string();
        let page_confidences = vec![PLACEHOLDER_PAGE_CONFIDENCE];
        let fallback_recommended = compute_fallback_recommended(
            &page_confidences,
            DEFAULT_CLOUD_FALLBACK_THRESHOLD,
            self.mode_allows_cloud,
        );

        Ok(PdfStructuredOutput {
            markdown,
            page_html_tables: Vec::new(),
            figure_crops: Vec::new(),
            page_confidences,
            fallback_recommended,
            parser: self.backend,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Env-mutating tests share global env vars; serialize them so `cargo
    // test` (without --test-threads=1) does not flake. Tests that only read
    // non-env state skip the lock.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn flag_defaults_off() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        assert!(!doc_parser_enabled_from_env());
    }

    #[test]
    fn flag_truthy_words() {
        let _g = ENV_LOCK.lock().unwrap();
        for v in ["1", "true", "TRUE", "on", "On"] {
            std::env::set_var(DOC_PARSER_ENABLE_ENV, v);
            assert!(doc_parser_enabled_from_env(), "flag must be ON for {v}");
        }
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
    }

    #[test]
    fn base_url_trims_trailing_slash() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DOC_PARSER_BASE_URL_ENV, "http://127.0.0.1:8760/");
        let url = doc_parser_base_url_from_env().expect("base url");
        assert_eq!(url, "http://127.0.0.1:8760");
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
    }

    #[test]
    fn from_env_returns_none_when_flag_off() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        assert!(DocParserClient::from_env().is_none());
    }

    #[test]
    fn from_env_or_disable_errors_when_flag_off() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        assert!(matches!(
            DocParserClient::from_env_or_disable(),
            Err(DocParserError::Disabled)
        ));
    }

    #[test]
    fn from_env_or_disable_errors_when_base_url_missing() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DOC_PARSER_ENABLE_ENV, "1");
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
        let r = DocParserClient::from_env_or_disable();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        assert!(matches!(r, Err(DocParserError::BaseUrlMissing)));
    }

    #[test]
    fn from_env_or_disable_constructs_when_configured() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DOC_PARSER_ENABLE_ENV, "1");
        std::env::set_var(DOC_PARSER_BASE_URL_ENV, "http://127.0.0.1:8760");
        let r = DocParserClient::from_env_or_disable();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
        assert!(r.is_ok());
        let c = r.expect("client");
        assert_eq!(c.backend(), ParserBackend::MinerU);
        assert_eq!(c.base_url(), "http://127.0.0.1:8760");
    }

    #[test]
    fn mineru_response_body_prefers_results_md_content() {
        let mut results = std::collections::BTreeMap::new();
        results.insert(
            "formula_stub".into(),
            MinerUResultEntry {
                md_content: "$$E=mc^2$$".into(),
                ..Default::default()
            },
        );
        let r = MinerUFileParseResponse {
            results,
            markdown: "# legacy top".into(),
            content: "legacy".into(),
        };
        assert_eq!(r.body(), "$$E=mc^2$$");
    }

    #[test]
    fn mineru_response_body_falls_back_to_results_markdown() {
        let mut results = std::collections::BTreeMap::new();
        results.insert(
            "k".into(),
            MinerUResultEntry {
                markdown: "# entry".into(),
                ..Default::default()
            },
        );
        let r = MinerUFileParseResponse {
            results,
            markdown: "# top".into(),
            content: String::new(),
        };
        assert_eq!(r.body(), "# entry");
    }

    #[test]
    fn mineru_response_body_falls_back_to_top_level_markdown() {
        let r = MinerUFileParseResponse {
            results: std::collections::BTreeMap::new(),
            markdown: "# Title".into(),
            content: "legacy".into(),
        };
        assert_eq!(r.body(), "# Title");
    }

    #[test]
    fn mineru_response_body_falls_back_to_top_level_content() {
        let r = MinerUFileParseResponse {
            results: std::collections::BTreeMap::new(),
            markdown: String::new(),
            content: "legacy body".into(),
        };
        assert_eq!(r.body(), "legacy body");
    }

    #[test]
    fn parse_eligibility_blocks_cloud_in_local_mode() {
        let client = DocParserClient::new(
            "http://127.0.0.1:8760".into(),
            ParserBackend::MistralCloud,
            false,
        );
        let tmp = tempfile::NamedTempFile::new().expect("tmp");
        let r = client.parse(tmp.path());
        assert!(matches!(r, Err(DocParserError::Eligibility(_))));
    }

    #[test]
    fn status_lane_off_when_flag_unset() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
        let s = status_from_env();
        assert!(!s.enabled);
        assert!(!s.base_url_configured);
        assert_eq!(s.backend, "mineru");
        assert_eq!(s.lane, "off");
    }

    #[test]
    fn status_lane_misconfigured_when_flag_on_base_url_missing() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DOC_PARSER_ENABLE_ENV, "1");
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
        let s = status_from_env();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        assert!(s.enabled);
        assert!(!s.base_url_configured);
        assert_eq!(s.lane, "misconfigured");
        assert_eq!(s.backend, "mineru");
    }

    #[test]
    fn status_lane_ready_when_flag_on_and_base_url_set() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(DOC_PARSER_ENABLE_ENV, "1");
        std::env::set_var(DOC_PARSER_BASE_URL_ENV, "http://127.0.0.1:8760");
        let s = status_from_env();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
        assert!(s.enabled);
        assert!(s.base_url_configured);
        assert_eq!(s.lane, "ready");
        assert_eq!(s.backend, "mineru");
    }

    #[test]
    fn status_lane_off_when_flag_off_but_base_url_set() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(DOC_PARSER_ENABLE_ENV);
        std::env::set_var(DOC_PARSER_BASE_URL_ENV, "http://127.0.0.1:8760");
        let s = status_from_env();
        std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
        assert!(!s.enabled);
        assert!(s.base_url_configured);
        assert_eq!(s.lane, "off");
    }
}
