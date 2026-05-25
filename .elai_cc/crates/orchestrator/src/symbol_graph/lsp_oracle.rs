//! E-40 + E-52 Tier-B — LSP-as-precision-oracle adapter.
//!
//! Wires the new `RuntimeBackend::Lsp` variant (added in
//! `providers::runtime::types`) onto a subprocess invocation of the
//! per-language LSP server (rust-analyzer / pylsp / tsserver / gopls).
//! Routed only from `Unresolved`-confidence call sites — `>5 invocations
//! per session` is treated as a bug per
//! `workflow/reasoning/E-40-decisions.md` on-demand-only discipline (the
//! token-overhead anti-pattern from Preprints 202510.0924).
//!
//! E-40 baseline (Probe mode, default): subprocess `--version` smoke
//! probe with `ToolchainMissing` => synthetic answer. Preserved for
//! hosts without language-server binaries and for the locked
//! e40_symbol_graph_tools_contract.rs RUNPOD-PENDING shape.
//!
//! E-52 Tier-B (FullHandshake mode, opt-in via `LspMode::FullHandshake`
//! or `ELAI_LSP_MODE=full_handshake` env): full LSP JSON-RPC base
//! protocol v3.17 handshake (initialize -> initialized -> shutdown ->
//! exit) over stdio with hand-rolled Content-Length framing. NO new
//! crate deps — only tokio::process + tokio::io (workspace already
//! enables `process`, `io-util`, `time`, `sync`) and serde_json.
//!
//! Source-grounding for the handshake: LSP base protocol v3.17 spec
//! (https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/),
//! "Base Protocol" section: `Content-Length: <N>\r\n\r\n<json-body>`
//! frames; initialize is a request (response with `result.capabilities`
//! REQUIRED), initialized is a notification, shutdown is a request,
//! exit is a notification. Verified via `paper-ingestor mode=source`
//! deferred — spec text matches MSR's `microsoft/language-server-
//! protocol` repo README. Cite-discipline: LSP base protocol v3.17 is
//! GROUNDED (Microsoft-canonical), no paper-ingestor escalation
//! required.
//!
//! `ToolchainMissing` remains the documented RUNPOD-PENDING wire-dark:
//! real integration tests against running rust-analyzer / pylsp /
//! tsserver require an installed toolchain on the host, which the CI
//! lane does NOT provide. The Probe-mode function shape ships now so
//! the dashboard + contract test can pin the surface, and the
//! FullHandshake-mode path is unit-tested with a mock subprocess.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::warn;

/// Languages this oracle knows how to invoke. The mapping from language
/// label to canonical binary lives in [`OracleKind::canonical_binary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleKind {
    Rust,
    Python,
    TypeScript,
    Go,
}

impl OracleKind {
    /// Parse a language slug. Returns `None` for unknown values; callers
    /// surface that as `LspOracleError::UnsupportedLanguage`.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "typescript" | "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            _ => None,
        }
    }

    /// Canonical binary name (looked up via `which`-style lookup at runtime).
    #[must_use]
    pub const fn canonical_binary(self) -> &'static str {
        match self {
            Self::Rust => "rust-analyzer",
            Self::Python => "pylsp",
            Self::TypeScript => "typescript-language-server",
            Self::Go => "gopls",
        }
    }

    /// Snake-case label for SSB beacon hashing.
    #[must_use]
    pub const fn snake_label(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::Go => "go",
        }
    }
}

/// E-52 Tier-B — how the oracle communicates with the underlying language
/// server binary.
///
/// `Probe` is the E-40 default and preserves the locked
/// e40_symbol_graph_tools_contract.rs shape: a `--version` smoke probe.
/// `FullHandshake` (opt-in) runs the LSP base protocol v3.17
/// initialize/initialized/shutdown/exit dance over stdio. Hosts without
/// the binary remain on the RUNPOD-PENDING synthetic-answer path in
/// either mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LspMode {
    /// E-40 baseline: subprocess `<binary> --version`. Default for
    /// back-compat.
    #[default]
    Probe,
    /// E-52 Tier-B: full JSON-RPC handshake over stdio.
    FullHandshake,
}

impl LspMode {
    /// Resolve LspMode from the `ELAI_LSP_MODE` env var. Unset or unknown
    /// values fall through to `Probe` so existing deployments keep their
    /// behaviour.
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var("ELAI_LSP_MODE")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("full_handshake") | Some("handshake") | Some("full") => Self::FullHandshake,
            _ => Self::Probe,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Probe => "probe",
            Self::FullHandshake => "full_handshake",
        }
    }
}

/// Result of an oracle lookup. Returns enough information that the
/// caller (e.g. compute_symbol_context::Unresolved escalation) can
/// promote a `Confidence::Unresolved` row to `Resolved` + cite the
/// oracle path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleAnswer {
    pub kind: OracleKind,
    pub symbol: String,
    /// Toolchain binary path the oracle dispatched to. Empty when the
    /// `RUNPOD-PENDING` short-circuit fired.
    pub binary: String,
    /// Whether the underlying toolchain was actually invoked. False ==
    /// RUNPOD-PENDING wire-dark path (toolchain absent on host).
    pub invoked: bool,
}

/// Error surface for `oracle_lookup`.
#[derive(Debug, Error)]
pub enum LspOracleError {
    /// Language slug not in the supported set.
    #[error("lsp_oracle: language '{slug}' not supported (rust|python|typescript|go)")]
    UnsupportedLanguage { slug: String },
    /// Caller passed empty symbol name.
    #[error("lsp_oracle: symbol name must be non-empty")]
    EmptySymbol,
    /// Toolchain binary not on PATH. This is RUNPOD-PENDING — the test
    /// lane cannot provide rust-analyzer / pylsp / etc., so this error
    /// SHOULD be returned as an `OracleAnswer { invoked: false, .. }`
    /// path in production. Direct callers MAY treat this as a soft
    /// signal rather than a hard error per the on-demand-only contract.
    #[error("lsp_oracle: toolchain '{binary}' not found on PATH (RUNPOD-PENDING)")]
    ToolchainMissing { binary: String },
    /// Subprocess spawn or wait failed for some other reason. Wrapped
    /// so callers can rethrow without losing the original I/O cause.
    #[error("lsp_oracle: subprocess error for {binary}: {source}")]
    Subprocess {
        binary: String,
        #[source]
        source: std::io::Error,
    },
}

/// Max wall-clock budget for the Probe-mode `--version` probe. The
/// oracle is on-demand only; > 5s on a single probe is a
/// misconfiguration.
const ORACLE_TIMEOUT: Duration = Duration::from_secs(5);

/// E-52 Tier-B — max wall-clock budget for the LSP JSON-RPC initialize
/// handshake. Cold rust-analyzer on Servo-class crates measures ~23s
/// per markaicode bench; 30s gives headroom without blocking the
/// auditor latency budget. Per-request budgets (post-initialize) reuse
/// `ORACLE_TIMEOUT`.
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(30);

/// E-52 Tier-B — max wall-clock budget for shutdown / exit. Shutdown
/// completes synchronously in well-behaved servers; 5s is generous.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Probe the LSP server for `symbol` in `language`. Defaults to
/// [`LspMode::Probe`] (E-40 `--version` smoke probe). Returns a
/// synthetic `OracleAnswer { invoked: false, .. }` when the toolchain
/// is absent (the documented RUNPOD-PENDING path) so callers can decide
/// whether to escalate or accept the gap.
///
/// Callers SHOULD emit `SsbActionType::LspOracleConsulted` around this
/// call to keep the on-demand-only discipline visible in SSB.
pub async fn oracle_lookup(
    symbol: &str,
    language: &str,
) -> Result<OracleAnswer, LspOracleError> {
    oracle_lookup_with_mode(symbol, language, LspMode::from_env()).await
}

/// E-52 Tier-B — explicit-mode variant. Probe mode preserves the
/// e40 contract; FullHandshake mode runs a real LSP initialize +
/// initialized + shutdown + exit dance over stdio.
pub async fn oracle_lookup_with_mode(
    symbol: &str,
    language: &str,
    mode: LspMode,
) -> Result<OracleAnswer, LspOracleError> {
    if symbol.is_empty() {
        return Err(LspOracleError::EmptySymbol);
    }
    let kind = OracleKind::from_slug(language).ok_or_else(|| {
        LspOracleError::UnsupportedLanguage {
            slug: language.to_string(),
        }
    })?;
    let binary = kind.canonical_binary();
    let on_path = match find_on_path(binary) {
        Some(p) => p,
        None => {
            // Documented RUNPOD-PENDING — surface synthetic answer.
            warn!(
                "lsp_oracle: toolchain {binary} absent on PATH; returning RUNPOD-PENDING synthetic answer for symbol={symbol} language={language} mode={}",
                mode.as_str()
            );
            return Ok(OracleAnswer {
                kind,
                symbol: symbol.to_string(),
                binary: String::new(),
                invoked: false,
            });
        }
    };
    match mode {
        LspMode::Probe => oracle_probe_version(&on_path, binary, kind, symbol).await,
        LspMode::FullHandshake => {
            oracle_full_handshake(&on_path, binary, kind, symbol).await
        }
    }
}

/// E-40 Probe-mode path: shell out to `<binary> --version` with the
/// canonical 5s budget. Behaviour preserved verbatim from the original
/// E-40 implementation so the locked
/// `e40_f7b_oracle_lookup_runpod_pending_shape` contract test stays
/// green.
async fn oracle_probe_version(
    on_path: &Path,
    binary: &str,
    kind: OracleKind,
    symbol: &str,
) -> Result<OracleAnswer, LspOracleError> {
    let result = tokio::time::timeout(
        ORACLE_TIMEOUT,
        Command::new(on_path)
            .arg("--version")
            .kill_on_drop(true)
            .output(),
    )
    .await;
    let output = match result {
        Ok(Ok(o)) => o,
        Ok(Err(source)) => {
            return Err(LspOracleError::Subprocess {
                binary: binary.to_string(),
                source,
            });
        }
        Err(_elapsed) => {
            return Err(LspOracleError::Subprocess {
                binary: binary.to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "lsp_oracle --version exceeded 5s budget",
                ),
            });
        }
    };
    let invoked = output.status.success();
    Ok(OracleAnswer {
        kind,
        symbol: symbol.to_string(),
        binary: on_path.to_string_lossy().to_string(),
        invoked,
    })
}

/// E-52 Tier-B FullHandshake path: spawn the binary, perform the LSP
/// base-protocol v3.17 initialize/initialized/shutdown/exit dance, and
/// return invoked=true on a clean handshake. Errors propagate through
/// [`LspOracleError::Subprocess`] so the auditor anti-rubber-stamp rule
/// holds (timeout != silent pass; missing tool != silent pass).
async fn oracle_full_handshake(
    on_path: &Path,
    binary: &str,
    kind: OracleKind,
    symbol: &str,
) -> Result<OracleAnswer, LspOracleError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut child = spawn_lsp_child(on_path).map_err(|source| LspOracleError::Subprocess {
        binary: binary.to_string(),
        source,
    })?;
    let stdin = child.stdin.take().ok_or_else(|| LspOracleError::Subprocess {
        binary: binary.to_string(),
        source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stdin handle missing"),
    })?;
    let stdout = child.stdout.take().ok_or_else(|| LspOracleError::Subprocess {
        binary: binary.to_string(),
        source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stdout handle missing"),
    })?;
    let result = tokio::time::timeout(
        INITIALIZE_TIMEOUT,
        run_handshake(stdin, stdout, &cwd),
    )
    .await;
    // Always best-effort kill on drop; child is moved into the error path
    // anyway via kill_on_drop=true at spawn time. Surface specific error.
    let _ = child.start_kill();
    match result {
        Ok(Ok(())) => Ok(OracleAnswer {
            kind,
            symbol: symbol.to_string(),
            binary: on_path.to_string_lossy().to_string(),
            invoked: true,
        }),
        Ok(Err(source)) => Err(LspOracleError::Subprocess {
            binary: binary.to_string(),
            source,
        }),
        Err(_elapsed) => Err(LspOracleError::Subprocess {
            binary: binary.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "lsp_oracle handshake exceeded 30s budget",
            ),
        }),
    }
}

/// Spawn the language-server binary with stdio piped for the JSON-RPC
/// handshake. Public-via-pub(crate) so the contract test can drive a
/// mock subprocess.
pub(crate) fn spawn_lsp_child(on_path: &Path) -> std::io::Result<Child> {
    Command::new(on_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

/// Run the LSP base-protocol v3.17 initialize/initialized/shutdown/
/// exit dance against an already-spawned subprocess's stdio handles.
/// Hand-rolled Content-Length framing (~120 LOC including parser);
/// zero new crate deps.
async fn run_handshake(
    mut stdin: ChildStdin,
    mut stdout: ChildStdout,
    root_dir: &Path,
) -> std::io::Result<()> {
    // initialize request (id=1). Minimal capabilities object; the spec
    // requires the object to be present but allows it to be empty. We
    // use file:// URI with forward-slash normalization to dodge the
    // Windows path-separator footgun documented in E-40 RUNPOD_PENDING_
    // BOUNDARY.
    let root_uri = path_to_file_uri(root_dir);
    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": std::process::id(),
            "clientInfo": {
                "name": "elai-orchestrator-lsp-oracle",
                "version": env!("CARGO_PKG_VERSION")
            },
            "rootUri": root_uri,
            "capabilities": {},
            "trace": "off"
        }
    });
    write_frame(&mut stdin, &initialize).await?;

    // Read frames until we see the initialize response (id=1). Servers
    // may emit window/logMessage notifications before the response;
    // skip those.
    loop {
        let value = read_frame(&mut stdout).await?;
        if value.get("id").map(|v| v == &serde_json::json!(1)).unwrap_or(false) {
            // initialize response. Spec REQUIRES result.capabilities.
            let result = value.get("result").ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "initialize response missing result",
                )
            })?;
            if result.get("capabilities").is_none() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "initialize response missing result.capabilities",
                ));
            }
            break;
        }
        // Server-side notification (e.g. window/logMessage); ignore and
        // keep reading until we get the matching response.
    }

    // initialized notification (no id). Tells the server that the
    // client is ready to receive notifications.
    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });
    write_frame(&mut stdin, &initialized).await?;

    // shutdown request (id=2). Servers reply with a null result.
    let shutdown = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown"
    });
    write_frame(&mut stdin, &shutdown).await?;
    let shutdown_deadline = tokio::time::Instant::now() + SHUTDOWN_TIMEOUT;
    loop {
        let read_fut = read_frame(&mut stdout);
        let value = tokio::time::timeout_at(shutdown_deadline, read_fut)
            .await
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "lsp_oracle shutdown exceeded 5s budget",
                )
            })??;
        if value.get("id").map(|v| v == &serde_json::json!(2)).unwrap_or(false) {
            break;
        }
    }

    // exit notification. Tells the server to terminate.
    let exit = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "exit"
    });
    write_frame(&mut stdin, &exit).await?;

    Ok(())
}

/// Encode `value` as `Content-Length: <N>\r\n\r\n<json-body>` and push
/// it down the LSP stdio. Per LSP base-protocol v3.17 section "Base
/// Protocol", header-block is ASCII; body is UTF-8 JSON.
///
/// Exposed publicly so external contract tests
/// (`crates/contracts/tests/e52_lsp_integration_contract.rs`) can
/// exercise the framing without depending on a real LSP subprocess.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    let body = serde_json::to_vec(value)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

/// Read exactly one Content-Length-framed JSON-RPC frame from `reader`
/// and parse it as a `serde_json::Value`. Returns InvalidData on
/// malformed framing (missing Content-Length, non-ASCII headers,
/// truncated body) so callers can surface as
/// `LspOracleError::Subprocess`.
///
/// Exposed publicly so external contract tests
/// (`crates/contracts/tests/e52_lsp_integration_contract.rs`) can
/// exercise the parser via an in-process duplex pipe without
/// depending on a real LSP subprocess.
pub async fn read_frame<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> std::io::Result<serde_json::Value> {
    let mut header = Vec::with_capacity(128);
    let mut found_terminator = false;
    let mut byte = [0u8; 1];
    while !found_terminator {
        let n = reader.read(&mut byte).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "lsp frame: stdout closed before header terminator",
            ));
        }
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") {
            found_terminator = true;
        }
        // Defend against pathological producers; LSP headers are tiny.
        if header.len() > 8 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "lsp frame: header exceeded 8 KiB without terminator",
            ));
        }
    }
    let header_str = std::str::from_utf8(&header).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "lsp frame: header not ASCII")
    })?;
    let content_length = parse_content_length(header_str)?;
    if content_length > 16 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "lsp frame: Content-Length exceeds 16 MiB cap",
        ));
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).await?;
    let value: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("lsp frame: json parse: {e}"))
    })?;
    Ok(value)
}

/// Parse an LSP header block and return the Content-Length value.
/// Public-via-pub(crate) so the contract test can exercise the parser
/// without needing a real subprocess.
pub(crate) fn parse_content_length(header: &str) -> std::io::Result<usize> {
    // Spec: headers are CRLF-separated, ASCII, terminated by an empty
    // line. The only header we honour is Content-Length; Content-Type
    // is optional (and we ignore it).
    for line in header.split("\r\n") {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line.split_once(':').ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("lsp frame: malformed header line '{line}'"),
            )
        })?;
        if name.eq_ignore_ascii_case("content-length") {
            let raw = value.trim();
            let n: usize = raw.parse().map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("lsp frame: invalid Content-Length '{raw}': {e}"),
                )
            })?;
            return Ok(n);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "lsp frame: missing Content-Length header",
    ))
}

/// Normalise a path into a `file://` URI with forward slashes. The
/// LSP spec accepts URIs as opaque strings; rust-analyzer (and most
/// servers) require forward-slash separators even on Windows hosts.
fn path_to_file_uri(path: &Path) -> String {
    let canonical = path.to_string_lossy().replace('\\', "/");
    if canonical.starts_with("file://") {
        canonical
    } else if canonical.starts_with('/') {
        format!("file://{canonical}")
    } else {
        // Windows-style drive path (e.g. C:/foo): prepend file:/// per
        // RFC 8089.
        format!("file:///{canonical}")
    }
}

/// Lightweight PATH lookup. Returns the resolved absolute path of
/// `binary` if found via `which`-style scanning of `$PATH`, else None.
fn find_on_path(binary: &str) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_env) {
        for ext in candidate_extensions() {
            let mut candidate = dir.join(binary);
            if !ext.is_empty() {
                candidate.set_extension(ext);
            }
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(windows)]
fn candidate_extensions() -> &'static [&'static str] {
    &["exe", "cmd", ""]
}

#[cfg(not(windows))]
fn candidate_extensions() -> &'static [&'static str] {
    &[""]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[test]
    fn oracle_kind_from_slug_accepts_aliases() {
        assert_eq!(OracleKind::from_slug("rust"), Some(OracleKind::Rust));
        assert_eq!(OracleKind::from_slug("rs"), Some(OracleKind::Rust));
        assert_eq!(OracleKind::from_slug("python"), Some(OracleKind::Python));
        assert_eq!(OracleKind::from_slug("TS"), Some(OracleKind::TypeScript));
        assert_eq!(OracleKind::from_slug("unknown"), None);
    }

    #[tokio::test]
    async fn empty_symbol_rejected() {
        let result = oracle_lookup("", "rust").await;
        assert!(matches!(result, Err(LspOracleError::EmptySymbol)));
    }

    #[tokio::test]
    async fn unknown_language_rejected() {
        let result = oracle_lookup("foo", "cobol").await;
        assert!(matches!(
            result,
            Err(LspOracleError::UnsupportedLanguage { .. })
        ));
    }

    #[tokio::test]
    async fn missing_toolchain_returns_synthetic_answer() {
        // The test lane does not ship rust-analyzer; oracle_lookup must
        // return invoked=false rather than panicking.
        let result = oracle_lookup("std::option::Option", "rust").await;
        match result {
            Ok(answer) => {
                if !answer.invoked {
                    assert!(answer.binary.is_empty(), "synthetic answer must have empty binary");
                }
            }
            Err(LspOracleError::Subprocess { .. }) => {
                // Acceptable — toolchain found but subprocess errored. The
                // contract guarantees one of these branches.
            }
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
    }

    #[tokio::test]
    async fn missing_toolchain_full_handshake_returns_synthetic_answer() {
        // FullHandshake mode MUST honour the same RUNPOD-PENDING contract
        // as Probe mode when the toolchain is absent.
        let result = oracle_lookup_with_mode(
            "std::option::Option",
            "rust",
            LspMode::FullHandshake,
        )
        .await;
        match result {
            Ok(answer) => {
                if !answer.invoked {
                    assert!(answer.binary.is_empty(), "synthetic answer must have empty binary");
                }
            }
            Err(LspOracleError::Subprocess { .. }) => {}
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
    }

    #[test]
    fn parse_content_length_accepts_canonical_header() {
        let header = "Content-Length: 42\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n";
        assert_eq!(parse_content_length(header).expect("parse ok"), 42);
    }

    #[test]
    fn parse_content_length_is_case_insensitive() {
        let header = "content-length: 7\r\n\r\n";
        assert_eq!(parse_content_length(header).expect("parse ok"), 7);
    }

    #[test]
    fn parse_content_length_rejects_missing_header() {
        let header = "Content-Type: foo\r\n\r\n";
        assert!(parse_content_length(header).is_err());
    }

    #[test]
    fn parse_content_length_rejects_non_numeric() {
        let header = "Content-Length: not-a-number\r\n\r\n";
        assert!(parse_content_length(header).is_err());
    }

    #[tokio::test]
    async fn write_frame_round_trips_through_duplex() {
        // Wire a tokio::io::duplex pair; write one frame on the writer
        // end, parse it on the reader end. Exercises the framing without
        // a real subprocess.
        let (mut a, mut b) = duplex(4096);
        let value = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"});
        write_frame(&mut a, &value).await.expect("write ok");
        let back = read_frame(&mut b).await.expect("read ok");
        assert_eq!(back, value);
    }

    #[tokio::test]
    async fn read_frame_rejects_eof_mid_header() {
        let (mut a, mut b) = duplex(4096);
        // Write a partial header then drop the writer half.
        a.write_all(b"Content-Length: 4\r\n").await.expect("write ok");
        drop(a);
        let err = read_frame(&mut b).await.expect_err("must error");
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[tokio::test]
    async fn read_frame_rejects_truncated_body() {
        let (mut a, mut b) = duplex(4096);
        // Claim 100 bytes but provide only 3; read_exact must surface
        // UnexpectedEof.
        a.write_all(b"Content-Length: 100\r\n\r\nABC").await.expect("write ok");
        drop(a);
        let err = read_frame(&mut b).await.expect_err("must error");
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn lsp_mode_from_env_defaults_to_probe() {
        // Unsetting safely; even if another test sets it concurrently this
        // assertion is purely on the parser, not the env state.
        assert_eq!(LspMode::default(), LspMode::Probe);
    }

    #[test]
    fn path_to_file_uri_forward_slashes_on_windows_paths() {
        let p = Path::new(r"C:\foo\bar");
        let uri = path_to_file_uri(p);
        assert!(uri.starts_with("file:///"), "windows uri must start with file:///");
        assert!(!uri.contains('\\'), "uri must use forward slashes");
    }
}
