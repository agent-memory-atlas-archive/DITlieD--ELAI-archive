//! E-24 Task 6: `elai mcp pin [list|show <server>|update <server>]`
//!
//! Operator-facing CLI surface over the MCP schema-hash pin store
//! (`crates/plugins/src/mcp_schema_pin.rs`). This is the missing wiring
//! gap from the E-24 plan-status fragment: the schema-pin module + the
//! live bridge re-pin gate were already shipped and wired into the bridge
//! dispatch, but the `elai mcp pin` operator surface (plan Task 6 /
//! WIRING_CONTRACT produces `cli::mcp_pin::pin_subcommand`) was never
//! routed through the `/mcp` slash-command handler.
//!
//! Subcommands:
//!   - `list` (default) — enumerate every pinned server by reading the
//!     on-disk pin records under `<cwd>/.elai/mcp/pins/*.toml`. These are
//!     the off-agent artifacts the production bridge writes on first
//!     connect (`pin_or_verify`); this surface reads them back, never
//!     authors them.
//!   - `show <server>` — print one server's pinned hash + pinned-at +
//!     the approved tool name/description set (the line-jump baseline).
//!   - `update <server>` — operator-gated re-approval. Re-fetches
//!     `tools/list` from the live server via the production
//!     `CliMcpClient` HTTP bridge and overwrites the pin with the new
//!     hash (`update_pin`). Fail-closed: when the server is unreachable
//!     or the `mcp` feature is not compiled in, returns a clear error
//!     and does NOT touch the pin.
//!
//! Reachability (R7, non-test entrypoint chain):
//!   main.rs `CliAction::Mcp` -> `LiveCli::print_mcp`
//!   -> `commands::handle_mcp_slash_command`
//!   -> `commands::render_mcp_report_for` (the new `pin` match arm)
//!   -> `mcp_pin_cmd::render_text` / `render_json`
//!   -> `elai_plugins::{read_pin, update_pin}`.
//!
//! Reference: OWASP MCP03:2025 (Tool Poisoning); Trail of Bits "Jumping
//! the Line" Apr 21 2025; postmark-mcp rug-pull Sept 17 2025. See
//! `workflow/reasoning/E-24-decisions.md`.

use std::fs;
use std::path::Path;

use elai_core::paths::mcp_dir_path;
use serde_json::{json, Value};

/// Format a unix-ms timestamp as a stable, locale-independent string.
/// Avoids a chrono dependency: prints the raw epoch-ms (deterministic,
/// machine-parseable, and stable for snapshot tests).
fn fmt_pinned_at(ms: i64) -> String {
    format!("{ms} (unix ms)")
}

/// Render any `pin` subcommand as text. `rest` is the tokens AFTER the
/// `pin` keyword (e.g. `[]`, `["show", "alpha"]`, `["update", "alpha"]`).
pub fn render_text(cwd: &Path, rest: &[&str]) -> String {
    match rest.first().copied() {
        None | Some("list") => render_list_text(cwd),
        Some("show") => match rest.get(1).copied() {
            Some(server) if rest.len() == 2 => render_show_text(cwd, server),
            _ => render_usage_text(Some("show")),
        },
        Some("update") => match rest.get(1).copied() {
            Some(server) if rest.len() == 2 => render_update_text(cwd, server),
            _ => render_usage_text(Some("update")),
        },
        Some("help") | Some("--help") | Some("-h") => render_usage_text(None),
        Some(other) => render_usage_text(Some(other)),
    }
}

/// Render any `pin` subcommand as JSON.
pub fn render_json(cwd: &Path, rest: &[&str]) -> Value {
    match rest.first().copied() {
        None | Some("list") => render_list_json(cwd),
        Some("show") => match rest.get(1).copied() {
            Some(server) if rest.len() == 2 => render_show_json(cwd, server),
            _ => render_usage_json(Some("show")),
        },
        Some("update") => match rest.get(1).copied() {
            Some(server) if rest.len() == 2 => render_update_json(cwd, server),
            _ => render_usage_json(Some("update")),
        },
        Some("help") | Some("--help") | Some("-h") => render_usage_json(None),
        Some(other) => render_usage_json(Some(other)),
    }
}

fn render_usage_text(unexpected: Option<&str>) -> String {
    let pin_dir_display = ".elai_cc/mcp/pins";
    let mut lines = vec![
        "MCP Pin (E-24)".to_string(),
        "  Usage            /mcp pin [list|show <server>|update <server>|help]".to_string(),
        "  Direct CLI       elai mcp pin [list|show <server>|update <server>]".to_string(),
        format!("  Store            <cwd>/{pin_dir_display}/*.toml (blake3 schema hash)"),
        "  OWASP            MCP03:2025 Tool Poisoning (rug-pull / line-jump)".to_string(),
    ];
    if let Some(args) = unexpected {
        lines.push(format!("  Unexpected       {args}"));
    }
    lines.join("\n")
}

fn render_usage_json(unexpected: Option<&str>) -> Value {
    let pin_dir_display = ".elai_cc/mcp/pins";
    json!({
        "kind": "mcp_pin",
        "action": "help",
        "usage": {
            "slash_command": "/mcp pin [list|show <server>|update <server>|help]",
            "direct_cli": "elai mcp pin [list|show <server>|update <server>]",
            "store": format!("<cwd>/{pin_dir_display}/*.toml"),
            "owasp": "MCP03:2025",
        },
        "unexpected": unexpected,
    })
}

/// Enumerate `<cwd>/.elai_cc/mcp/pins/*.toml` and parse each into a
/// `PinRecord`. Returns the list of (filename, record) pairs sorted by
/// the on-disk filename for deterministic output. A pin file that fails
/// to parse is surfaced with its filename + the parse error rather than
/// silently dropped (fail-loud, not fail-open).
#[cfg(feature = "mcp")]
fn collect_pins(cwd: &Path) -> Vec<(String, Result<elai_plugins::PinRecord, String>)> {
    let dir = mcp_dir_path(cwd).join("pins");
    let mut out: Vec<(String, Result<elai_plugins::PinRecord, String>)> = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return out;
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();
    files.sort();
    for path in files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unreadable>")
            .to_string();
        let parsed = fs::read_to_string(&path)
            .map_err(|e| format!("read error: {e}"))
            .and_then(|s| {
                toml::from_str::<elai_plugins::PinRecord>(&s)
                    .map_err(|e| format!("parse error: {e}"))
            });
        out.push((name, parsed));
    }
    out
}

#[cfg(feature = "mcp")]
fn render_list_text(cwd: &Path) -> String {
    let pins = collect_pins(cwd);
    let pin_dir = mcp_dir_path(cwd).join("pins");
    let mut lines = vec![
        "MCP Pin (E-24)".to_string(),
        format!("  Store            {}", pin_dir.display()),
        format!("  Pinned servers   {}", pins.len()),
    ];
    if pins.is_empty() {
        lines.push(
            "  Notes            no servers pinned yet; pins are written on first MCP connect"
                .to_string(),
        );
    }
    for (file, parsed) in pins {
        match parsed {
            Ok(record) => {
                lines.push(format!("    {} -> {}", record.server_url, record.pinned_hash));
                lines.push(format!(
                    "      pinned_at      {}",
                    fmt_pinned_at(record.pinned_at_ms)
                ));
                lines.push(format!("      tools          {}", record.pinned_tools.len()));
            }
            Err(e) => {
                lines.push(format!("    {file} -> UNPARSEABLE ({e})"));
            }
        }
    }
    lines.join("\n")
}

#[cfg(feature = "mcp")]
fn render_list_json(cwd: &Path) -> Value {
    let pins = collect_pins(cwd);
    let servers: Vec<Value> = pins
        .into_iter()
        .map(|(file, parsed)| match parsed {
            Ok(record) => json!({
                "file": file,
                "server_url": record.server_url,
                "pinned_hash": record.pinned_hash,
                "pinned_at_ms": record.pinned_at_ms,
                "tools": record.pinned_tools.len(),
                "parse_ok": true,
            }),
            Err(e) => json!({
                "file": file,
                "parse_ok": false,
                "error": e,
            }),
        })
        .collect();
    let pin_dir = mcp_dir_path(cwd).join("pins");
    json!({
        "kind": "mcp_pin",
        "action": "list",
        "store": pin_dir.display().to_string(),
        "count": servers.len(),
        "servers": servers,
    })
}

#[cfg(feature = "mcp")]
fn render_show_text(cwd: &Path, server_url: &str) -> String {
    match elai_plugins::read_pin(cwd, server_url) {
        Ok(Some(record)) => {
            let mut lines = vec![
                "MCP Pin (E-24)".to_string(),
                format!("  Server            {}", record.server_url),
                format!("  Pinned hash       {}", record.pinned_hash),
                format!("  Pinned at         {}", fmt_pinned_at(record.pinned_at_ms)),
                format!("  Approved tools    {}", record.pinned_tools.len()),
            ];
            for tool in &record.pinned_tools {
                lines.push(format!("    {}: {}", tool.name, tool.description));
            }
            lines.join("\n")
        }
        Ok(None) => format!(
            "MCP Pin (E-24)\n  Server            {server_url}\n  Result            no pin recorded (server not yet connected)"
        ),
        Err(e) => format!(
            "MCP Pin (E-24)\n  Server            {server_url}\n  Result            error reading pin: {e}"
        ),
    }
}

#[cfg(feature = "mcp")]
fn render_show_json(cwd: &Path, server_url: &str) -> Value {
    match elai_plugins::read_pin(cwd, server_url) {
        Ok(Some(record)) => {
            let tools: Vec<Value> = record
                .pinned_tools
                .iter()
                .map(|t| json!({ "name": t.name, "description": t.description }))
                .collect();
            json!({
                "kind": "mcp_pin",
                "action": "show",
                "found": true,
                "server_url": record.server_url,
                "pinned_hash": record.pinned_hash,
                "pinned_at_ms": record.pinned_at_ms,
                "tools": tools,
            })
        }
        Ok(None) => json!({
            "kind": "mcp_pin",
            "action": "show",
            "found": false,
            "server_url": server_url,
        }),
        Err(e) => json!({
            "kind": "mcp_pin",
            "action": "show",
            "found": false,
            "server_url": server_url,
            "error": e.to_string(),
        }),
    }
}

/// Re-fetch the live server's `tools/list` and overwrite the pin.
/// Operator-gated re-approval per OWASP MCP03:2025 + plan Task 1 open
/// question (a malicious server can social-engineer an update, so the
/// command echoes the new hash + tool count for the operator to verify).
///
/// `server_url` is treated as a direct MCP HTTP bridge URL when it parses
/// as `http(s)://...:<port>`; otherwise it is reported as unsupported
/// (named-server resolution from config to a live URL is a follow-up;
/// the live HTTP path is what the bridge re-pin gate exercises).
#[cfg(feature = "mcp")]
fn refetch_and_update(cwd: &Path, server_url: &str) -> Result<(String, usize), String> {
    use crate::mcp_client::CliMcpClient;

    let port = parse_http_port(server_url)
        .ok_or_else(|| format!("`{server_url}` is not an http(s) MCP bridge URL with a port; named-server update is a follow-up"))?;
    let client = CliMcpClient::new(port);
    // The live HTTP bridge currently returns tool NAMES; the schema-pin
    // hash also needs the input_schema. We fetch the per-tool schema for
    // each listed tool so the re-pin hashes the same canonical shape the
    // bridge pins on connect.
    let names = client.list_tools()?;
    let mut tools: Vec<elai_plugins::ToolDef> = Vec::with_capacity(names.len());
    for name in &names {
        let schema = client.get_tool_schema(name)?;
        // The bridge returns the full MCP tool spec (name + description +
        // inputSchema); pull description + input_schema out of it.
        let description = schema
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let input_schema = schema
            .get("inputSchema")
            .or_else(|| schema.get("input_schema"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        tools.push(elai_plugins::ToolDef {
            name: name.clone(),
            description,
            input_schema,
        });
    }
    let hash = elai_plugins::update_pin(cwd, server_url, &tools)
        .map_err(|e| format!("update_pin failed: {e}"))?;
    Ok((hash, tools.len()))
}

/// Extract the port from an `http(s)://host:port[/...]` URL. Returns
/// `None` when the input is not an http(s) URL or carries no explicit
/// port. Deliberately strict (no default-port guessing) so a typo never
/// silently re-pins the wrong endpoint.
#[cfg(feature = "mcp")]
fn parse_http_port(url: &str) -> Option<u16> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let port_str = authority.rsplit(':').next()?;
    if port_str == authority {
        return None;
    }
    port_str.parse::<u16>().ok()
}

#[cfg(feature = "mcp")]
fn render_update_text(cwd: &Path, server_url: &str) -> String {
    match refetch_and_update(cwd, server_url) {
        Ok((hash, tools)) => format!(
            "MCP Pin (E-24)\n  Server            {server_url}\n  Result            re-pinned (operator-approved)\n  New hash          {hash}\n  Tools             {tools}"
        ),
        Err(e) => format!(
            "MCP Pin (E-24)\n  Server            {server_url}\n  Result            update FAILED (pin unchanged): {e}"
        ),
    }
}

#[cfg(feature = "mcp")]
fn render_update_json(cwd: &Path, server_url: &str) -> Value {
    match refetch_and_update(cwd, server_url) {
        Ok((hash, tools)) => json!({
            "kind": "mcp_pin",
            "action": "update",
            "ok": true,
            "server_url": server_url,
            "new_hash": hash,
            "tools": tools,
        }),
        Err(e) => json!({
            "kind": "mcp_pin",
            "action": "update",
            "ok": false,
            "server_url": server_url,
            "error": e,
        }),
    }
}

// When the `mcp` feature is not compiled in, the pin store types are not
// available. Mirror `mcp_client.rs`: every subcommand reports the feature
// is absent rather than silently rendering an empty list (fail-loud).
#[cfg(not(feature = "mcp"))]
fn render_list_text(_cwd: &Path) -> String {
    "MCP Pin (E-24)\n  Result            mcp feature not compiled in".to_string()
}

#[cfg(not(feature = "mcp"))]
fn render_list_json(_cwd: &Path) -> Value {
    json!({ "kind": "mcp_pin", "action": "list", "error": "mcp feature not compiled in" })
}

#[cfg(not(feature = "mcp"))]
fn render_show_text(_cwd: &Path, server_url: &str) -> String {
    format!("MCP Pin (E-24)\n  Server            {server_url}\n  Result            mcp feature not compiled in")
}

#[cfg(not(feature = "mcp"))]
fn render_show_json(_cwd: &Path, server_url: &str) -> Value {
    json!({ "kind": "mcp_pin", "action": "show", "server_url": server_url, "error": "mcp feature not compiled in" })
}

#[cfg(not(feature = "mcp"))]
fn render_update_text(_cwd: &Path, server_url: &str) -> String {
    format!("MCP Pin (E-24)\n  Server            {server_url}\n  Result            mcp feature not compiled in")
}

#[cfg(not(feature = "mcp"))]
fn render_update_json(_cwd: &Path, server_url: &str) -> Value {
    json!({ "kind": "mcp_pin", "action": "update", "server_url": server_url, "error": "mcp feature not compiled in" })
}

#[cfg(all(test, feature = "mcp"))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_tools() -> Vec<elai_plugins::ToolDef> {
        vec![
            elai_plugins::ToolDef {
                name: "search".into(),
                description: "Search the web".into(),
                input_schema: serde_json::json!({"type": "object"}),
            },
            elai_plugins::ToolDef {
                name: "fetch".into(),
                description: "Fetch a URL".into(),
                input_schema: serde_json::json!({"type": "object"}),
            },
        ]
    }

    #[test]
    fn list_empty_when_no_pins() {
        let dir = TempDir::new().expect("tempdir");
        let text = render_text(dir.path(), &[]);
        assert!(text.contains("Pinned servers   0"), "got: {text}");
        assert!(text.contains("no servers pinned yet"), "got: {text}");
        let v = render_json(dir.path(), &["list"]);
        assert_eq!(v["count"], 0);
    }

    #[test]
    fn list_reads_back_a_production_written_pin() {
        // Write a pin through the PRODUCTION module path (the same call
        // the bridge makes on first connect), then read it back via this
        // CLI surface. The off-agent artifact is the .toml file; the test
        // authors neither the writer (elai_plugins::pin_or_verify) nor the
        // renderer.
        let dir = TempDir::new().expect("tempdir");
        let url = "http://localhost:8200";
        let status = elai_plugins::pin_or_verify(dir.path(), url, &sample_tools())
            .expect("pin write");
        assert_eq!(status, elai_plugins::PinStatus::Pinned);

        let text = render_text(dir.path(), &["list"]);
        assert!(text.contains("Pinned servers   1"), "got: {text}");
        assert!(text.contains(url), "list must name the pinned server: {text}");

        let v = render_json(dir.path(), &[]);
        assert_eq!(v["count"], 1);
        assert_eq!(v["servers"][0]["server_url"], url);
        assert_eq!(v["servers"][0]["tools"], 2);
        assert_eq!(v["servers"][0]["parse_ok"], true);
    }

    #[test]
    fn show_renders_pinned_tools_and_hash() {
        let dir = TempDir::new().expect("tempdir");
        let url = "http://localhost:8200";
        elai_plugins::pin_or_verify(dir.path(), url, &sample_tools()).expect("pin write");

        let text = render_text(dir.path(), &["show", url]);
        assert!(text.contains("Approved tools    2"), "got: {text}");
        assert!(text.contains("search: Search the web"), "got: {text}");

        let v = render_json(dir.path(), &["show", url]);
        assert_eq!(v["found"], true);
        assert_eq!(v["server_url"], url);
        let hash = v["pinned_hash"].as_str().expect("hash present");
        assert_eq!(hash, elai_plugins::hash_tool_list(&sample_tools()));
    }

    #[test]
    fn show_missing_server_is_not_found_not_error() {
        let dir = TempDir::new().expect("tempdir");
        let v = render_json(dir.path(), &["show", "http://localhost:9999"]);
        assert_eq!(v["found"], false);
        assert!(v.get("error").is_none(), "missing pin is not an error");
    }

    #[test]
    fn parse_http_port_strict() {
        assert_eq!(parse_http_port("http://localhost:8200"), Some(8200));
        assert_eq!(parse_http_port("https://host:443/x/y"), Some(443));
        assert_eq!(parse_http_port("http://localhost"), None);
        assert_eq!(parse_http_port("alpha"), None);
        assert_eq!(parse_http_port("ftp://host:21"), None);
    }

    #[test]
    fn update_unreachable_server_fails_closed_pin_unchanged() {
        // No live server at this port -> update must fail and NOT write a
        // pin (fail-closed). Asserting on the rendered error + the absence
        // of any pin file.
        let dir = TempDir::new().expect("tempdir");
        let url = "http://127.0.0.1:1"; // port 1: nothing listens
        let v = render_json(dir.path(), &["update", url]);
        assert_eq!(v["ok"], false, "unreachable update must fail: {v}");
        // No pin file was written.
        let listed = render_json(dir.path(), &["list"]);
        assert_eq!(listed["count"], 0, "failed update must not write a pin");
    }

    #[test]
    fn usage_on_bad_subcommand() {
        let dir = TempDir::new().expect("tempdir");
        let text = render_text(dir.path(), &["bogus"]);
        assert!(text.contains("Usage"), "got: {text}");
        let v = render_json(dir.path(), &["show"]); // missing server arg
        assert_eq!(v["action"], "help");
    }
}
