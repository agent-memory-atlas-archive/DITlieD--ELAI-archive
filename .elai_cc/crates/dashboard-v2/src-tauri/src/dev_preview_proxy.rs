//! H-14: Dev-preview proxy scaffold (Tier-1 shape + Tier-2 done-gate deferred).
//!
//! The dev-preview proxy serves the design-target app through an ELAI-controlled
//! port. It strips / relaxes X-Frame-Options and rewrites frame-ancestors in
//! HTTP responses so the Tauri webview can embed the proxied origin.
//!
//! Design decisions (H-14-decisions.md, CANVAS-WORKBENCH-DESIGN.md §8):
//!
//!   Context: a Tauri <iframe> pointed at a raw dev server (e.g. localhost:3000)
//!   will typically be blocked because:
//!     1. The dev server's X-Frame-Options: DENY / SAMEORIGIN rejects embedding.
//!     2. The dev server's Content-Security-Policy may include
//!        `frame-ancestors 'none'` or a specific origin allow-list.
//!     3. A Service Worker registered by the embedded app can intercept requests.
//!
//!   ELAI's proxy intercepts responses and:
//!     - Removes X-Frame-Options response headers (obsolete; MDN confirmed).
//!     - Rewrites or removes frame-ancestors CSP directives to permit the Tauri
//!       webview origin (ALLOW-FROM is obsolete; frame-ancestors is modern — MDN).
//!     - Scopes Service Worker registration so the embedded SW cannot hijack the
//!       preview frame.
//!
//!   Honest limit (GD-5): this works ONLY for web dev servers ELAI can front
//!   (i.e. servers ELAI starts or controls on a local port). An arbitrary
//!   deployed third-party site with a hard CSP ELAI cannot rewrite is OUT OF
//!   SCOPE. This constraint is stated explicitly in the WIRING_CONTRACT stub_ban
//!   and must NOT be advertised as a capability.
//!
//! Tier-1 shipped: the proxy start/stop commands + header-rewrite logic.
//! Tier-2 deferred: the live painted-preview done-gate (requires a real running
//!   dev server + Tauri webview render + off-agent screenshot of the PAINTED
//!   preview, NOT a 200 status code). Registered as stub-10.
//!
//! Cross-lang pairs:
//!   - `start_dev_proxy` Tauri command <-> stub.ts::start_dev_proxy
//!   - `stop_dev_proxy` Tauri command <-> stub.ts::stop_dev_proxy
//!
//! Anti-slopsquatting: reqwest is already a workspace dependency in
//! dashboard-v2 Cargo.toml (used by AppState.http). No new external crate added.
//!
//! HARD RULES:
//!   - No .unwrap() / .expect() in production paths.
//!   - No raw dev-server iframe (ELAI-controlled proxied origin only — stub_ban).
//!   - X-Frame-Options relaxation + frame-ancestors rewrite verified via MDN
//!     (H-14-grounding.md, 2026-06-05).
//!   - blake3 for any hash (not sha2/blake2).

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Named constants
// ---------------------------------------------------------------------------

/// HTTP header name for the framing control header (obsolete; still stripped).
/// MDN: "X-Frame-Options is obsolete. Modern replacement = frame-ancestors
/// in Content-Security-Policy."
pub const HEADER_X_FRAME_OPTIONS: &str = "X-Frame-Options";

/// HTTP response header that contains the Content-Security-Policy.
pub const HEADER_CSP: &str = "Content-Security-Policy";

/// The frame-ancestors CSP directive prefix (case-insensitive in actual CSP,
/// but we match lowercase for simplicity — the rewrite is applied regardless).
pub const CSP_FRAME_ANCESTORS_DIRECTIVE: &str = "frame-ancestors";

/// Default listen port for the design-preview proxy.
/// Chosen to avoid collision with common dev server ports (3000, 5173, 8080).
pub const DEFAULT_PROXY_PORT: u16 = 19_876;

/// Maximum number of response headers rewritten per request (safety bound).
pub const MAX_HEADERS_REWRITTEN: usize = 8;

// ---------------------------------------------------------------------------
// Proxy state
// ---------------------------------------------------------------------------

/// Live state of one dev-preview proxy instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    /// Whether the proxy is currently running.
    pub running: bool,
    /// The port the proxy is listening on.
    pub listen_port: u16,
    /// The target dev server URL being proxied.
    pub target_url: String,
    /// Tier-2 deferred: whether the painted-preview done-gate has been
    /// verified by an off-agent screenshot. `None` until Tier-2 lands.
    pub paint_verified: Option<bool>,
}

/// Handle to a live dev-preview proxy instance.
/// In Tier-2 this will hold the actual server join handle + port.
/// In Tier-1, it is a stub that records intent and enables clean stop.
pub struct DevPreviewProxy {
    status: Arc<Mutex<ProxyStatus>>,
}

impl DevPreviewProxy {
    /// Create a new (non-running) proxy handle.
    pub fn new(target_url: String, listen_port: u16) -> Self {
        Self {
            status: Arc::new(Mutex::new(ProxyStatus {
                running: false,
                listen_port,
                target_url,
                paint_verified: None,
            })),
        }
    }

    /// Return the current proxy status snapshot.
    pub fn status(&self) -> Result<ProxyStatus, String> {
        self.status
            .lock()
            .map(|g| g.clone())
            .map_err(|_| "DevPreviewProxy: status mutex poisoned".to_string())
    }

    /// Stop the proxy. Sets running=false.
    pub fn stop(&self) -> Result<(), String> {
        let mut st = self
            .status
            .lock()
            .map_err(|_| "DevPreviewProxy: stop mutex poisoned".to_string())?;
        st.running = false;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Header rewrite logic (the Tier-1 core, tested in contract tests)
// ---------------------------------------------------------------------------

/// Result of rewriting HTTP response headers for framing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRewriteResult {
    /// Number of headers removed (X-Frame-Options).
    pub headers_removed: usize,
    /// Number of CSP directives rewritten (frame-ancestors).
    pub directives_rewritten: usize,
    /// The rewritten CSP value, or None if no CSP header was present.
    pub rewritten_csp: Option<String>,
}

/// Rewrite a CSP header value to permit framing by the Tauri webview.
///
/// Removes or replaces the `frame-ancestors` directive.
/// `allowed_origin` should be the Tauri webview origin (e.g. "tauri://localhost"
/// or "https://tauri.localhost" depending on the Tauri version and platform).
///
/// MDN grounding (H-14-grounding.md, 2026-06-05):
///   - X-Frame-Options ALLOW-FROM is obsolete; frame-ancestors is the modern
///     replacement. Both are handled by the proxy.
///   - frame-ancestors in CSP overrides X-Frame-Options when both are present.
///
/// If the CSP has no frame-ancestors directive, the function returns the original
/// CSP unchanged (no blind injection of permissive directives — minimal footprint).
pub fn rewrite_csp_for_framing(csp: &str, allowed_origin: &str) -> String {
    // Parse directives. CSP directives are semicolon-separated.
    let directives: Vec<&str> = csp.split(';').collect();
    let mut rewritten: Vec<String> = Vec::with_capacity(directives.len());
    let mut found_frame_ancestors = false;

    for directive in &directives {
        let trimmed = directive.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Case-insensitive match on "frame-ancestors" prefix.
        if trimmed
            .to_ascii_lowercase()
            .starts_with(CSP_FRAME_ANCESTORS_DIRECTIVE)
        {
            // Replace with a permissive frame-ancestors that allows the Tauri origin.
            // We allow 'self' + the provided allowed_origin.
            rewritten.push(format!("frame-ancestors 'self' {allowed_origin}"));
            found_frame_ancestors = true;
        } else {
            rewritten.push(trimmed.to_owned());
        }
    }

    if !found_frame_ancestors {
        // No frame-ancestors directive present; return original unchanged.
        // Do NOT blindly inject `frame-ancestors *` (that would be over-permissive
        // for sites that use other CSP directives for security).
        return csp.to_owned();
    }

    rewritten.join("; ")
}

/// Process a set of HTTP response headers to enable framing in the Tauri webview.
///
/// Returns a `HeaderRewriteResult` describing what was changed.
/// The `headers` slice is (name, value) pairs. The function builds a new list
/// with X-Frame-Options removed and frame-ancestors rewritten.
///
/// The caller is responsible for applying the returned modifications to the
/// proxied response. This function is PURE (no I/O) so it is directly testable.
pub fn relax_headers_for_framing(
    headers: &[(String, String)],
    allowed_origin: &str,
) -> (Vec<(String, String)>, HeaderRewriteResult) {
    let mut result_headers: Vec<(String, String)> = Vec::with_capacity(headers.len());
    let mut removed = 0usize;
    let mut rewritten = 0usize;
    let mut rewritten_csp: Option<String> = None;

    for (name, value) in headers {
        // Remove X-Frame-Options entirely (obsolete; frame-ancestors replaces it).
        if name.eq_ignore_ascii_case(HEADER_X_FRAME_OPTIONS) {
            removed += 1;
            // Do not add to result_headers — the header is dropped.
            continue;
        }
        // Rewrite frame-ancestors in CSP.
        if name.eq_ignore_ascii_case(HEADER_CSP) {
            let new_csp = rewrite_csp_for_framing(value, allowed_origin);
            if new_csp != *value {
                rewritten += 1;
                rewritten_csp = Some(new_csp.clone());
                result_headers.push((name.clone(), new_csp));
            } else {
                result_headers.push((name.clone(), value.clone()));
            }
            continue;
        }
        result_headers.push((name.clone(), value.clone()));
    }

    let rewrite_result = HeaderRewriteResult {
        headers_removed: removed,
        directives_rewritten: rewritten,
        rewritten_csp,
    };
    (result_headers, rewrite_result)
}

// ---------------------------------------------------------------------------
// Tier-2 stub marker
// ---------------------------------------------------------------------------

/// Start a live dev-preview proxy instance.
///
/// Tier-2 DEFERRED (stub-10): the live proxy server that actually accepts TCP
/// connections, proxies HTTP requests, and strips framing headers is the
/// Tier-2 deliverable. It requires:
///   - A real running dev server to proxy.
///   - A Tauri webview to embed the proxied origin.
///   - An off-agent screenshot of the PAINTED preview (not just a 200 status).
///
/// In Tier-1, this function returns a DevPreviewProxy handle with running=false
/// and a non-silent explanation. The handle is valid for `stop_dev_proxy`.
///
/// TODO[H-14.proxy.live]: replace with the real reqwest-based reverse proxy
/// server + X-Frame-Options strip + frame-ancestors rewrite in HTTP responses.
/// The header-rewrite logic (relax_headers_for_framing) is already shipped and
/// tested; the missing piece is the TCP listener + response pipe. stub-10.
pub fn start_dev_proxy(target_url: String, listen_port: Option<u16>) -> Result<DevPreviewProxy, String> {
    let port = listen_port.unwrap_or(DEFAULT_PROXY_PORT);

    // Validate target URL is not empty.
    if target_url.trim().is_empty() {
        return Err(
            "start_dev_proxy: target_url must not be empty; \
             provide the dev server URL to proxy (e.g. http://localhost:3000)"
                .to_owned(),
        );
    }

    // Validate port is in a reasonable range (not 0, not well-known system ports).
    if port == 0 || port < 1024 {
        return Err(format!(
            "start_dev_proxy: listen_port {port} is invalid; \
             must be >= 1024. Recommended: {DEFAULT_PROXY_PORT}"
        ));
    }

    // TODO[H-14.proxy.live]: spawn the actual TCP listener here (stub-10).
    // For Tier-1, return a non-running handle with a documented reason.
    // This is NOT a silent stub: the caller receives a handle with running=false
    // and must check ProxyStatus.running before using the proxy.

    tracing::info!(
        port = port,
        target = %target_url,
        "start_dev_proxy: proxy handle created (Tier-1; live TCP listener is stub-10)"
    );

    Ok(DevPreviewProxy::new(target_url, port))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_csp_removes_frame_ancestors_and_permits_tauri() {
        let csp = "default-src 'self'; frame-ancestors 'none'; script-src 'self'";
        let result = rewrite_csp_for_framing(csp, "tauri://localhost");
        assert!(
            result.contains("frame-ancestors 'self' tauri://localhost"),
            "must permit tauri origin: {result}"
        );
        assert!(
            result.contains("default-src 'self'"),
            "must preserve other directives: {result}"
        );
        assert!(
            !result.contains("frame-ancestors 'none'"),
            "must remove restrictive frame-ancestors: {result}"
        );
    }

    #[test]
    fn rewrite_csp_leaves_csp_unchanged_when_no_frame_ancestors() {
        let csp = "default-src 'self'; script-src 'nonce-abc123'";
        let result = rewrite_csp_for_framing(csp, "tauri://localhost");
        // No frame-ancestors in original: do NOT inject blindly.
        assert_eq!(result, csp, "must leave CSP unchanged when no frame-ancestors");
    }

    #[test]
    fn relax_headers_removes_x_frame_options() {
        let headers = vec![
            ("X-Frame-Options".to_owned(), "DENY".to_owned()),
            ("Content-Type".to_owned(), "text/html".to_owned()),
        ];
        let (result, report) = relax_headers_for_framing(&headers, "tauri://localhost");
        assert_eq!(report.headers_removed, 1, "must remove one X-Frame-Options header");
        assert!(
            result.iter().all(|(n, _)| !n.eq_ignore_ascii_case("X-Frame-Options")),
            "X-Frame-Options must be absent from result"
        );
        assert!(
            result.iter().any(|(n, _)| n == "Content-Type"),
            "Content-Type must be preserved"
        );
    }

    #[test]
    fn relax_headers_rewrites_csp_frame_ancestors() {
        let headers = vec![
            ("X-Frame-Options".to_owned(), "SAMEORIGIN".to_owned()),
            (
                "Content-Security-Policy".to_owned(),
                "default-src 'self'; frame-ancestors 'none'".to_owned(),
            ),
        ];
        let (result, report) = relax_headers_for_framing(&headers, "tauri://localhost");
        assert_eq!(report.headers_removed, 1, "X-Frame-Options removed");
        assert_eq!(report.directives_rewritten, 1, "one CSP rewrite");
        assert!(report.rewritten_csp.is_some(), "rewritten_csp must be Some");

        let csp = result
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("Content-Security-Policy"))
            .map(|(_, v)| v.as_str())
            .expect("CSP must be in result");

        assert!(
            csp.contains("frame-ancestors 'self' tauri://localhost"),
            "CSP must permit tauri: {csp}"
        );
    }

    #[test]
    fn start_dev_proxy_rejects_empty_target_url() {
        let result = start_dev_proxy("".into(), None);
        assert!(result.is_err(), "empty target_url must be rejected");
    }

    #[test]
    fn start_dev_proxy_rejects_system_port() {
        let result = start_dev_proxy("http://localhost:3000".into(), Some(80));
        assert!(result.is_err(), "system port 80 must be rejected");
    }

    #[test]
    fn start_dev_proxy_returns_non_running_handle_in_tier1() {
        // Tier-1: start_dev_proxy returns a non-running handle (stub-10),
        // not a live server. The handle is valid and stop() succeeds.
        let proxy = start_dev_proxy("http://localhost:3000".into(), None)
            .expect("start_dev_proxy must succeed for valid args");
        let status = proxy.status().expect("status");
        assert!(!status.running, "Tier-1 handle must be non-running (stub-10)");
        assert_eq!(status.listen_port, DEFAULT_PROXY_PORT);
        assert_eq!(status.target_url, "http://localhost:3000");

        // stop() must succeed on a non-running handle.
        proxy.stop().expect("stop must succeed");
    }

    #[test]
    fn header_rewrite_constants_are_correct_wire_values() {
        // Wire-ID stability guard (wiring-enforcement R5).
        assert_eq!(HEADER_X_FRAME_OPTIONS, "X-Frame-Options");
        assert_eq!(HEADER_CSP, "Content-Security-Policy");
        assert_eq!(CSP_FRAME_ANCESTORS_DIRECTIVE, "frame-ancestors");
    }

    #[test]
    fn default_proxy_port_is_above_system_range() {
        assert!(DEFAULT_PROXY_PORT >= 1024, "proxy port must be above system range");
    }
}
