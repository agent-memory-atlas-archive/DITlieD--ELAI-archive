//! B-37D Phase 4M: `/runtime` slash-command handler.
//!
//! Three actions:
//! - `/runtime list` — render an ASCII table of the 11 canonical
//!   `RuntimeBackend`s with their current per-role assignments (from
//!   `elai_cc.toml [agents.<role>.runtime] backend = "<name>"`) and live
//!   `health_check` results (wrapped in a 5-second timeout).
//! - `/runtime test <backend>` — invoke `health_check` on a single
//!   adapter and report pass/fail + elapsed milliseconds.
//! - `/runtime set --role <name> --backend <name>` — persist a new
//!   per-role runtime mapping to `elai_cc.toml`. The write is atomic
//!   (write to `.tmp` sibling then rename) per gotchas.md best practice.
//!
//! Until Wave-1B lands `RuntimeOrchestrator::for_role` /
//! `health_snapshot`, the live-orchestrator query path is gated behind
//! a TODO sentinel and the handler talks to `providers::runtime`
//! directly. This is intentional — the CLI surface ships fully wired
//! and the orchestrator path becomes a one-import swap once Wave-1B
//! commits.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::{
    default_ssb_path_for_base, AgentRole, RuntimeBackendConfig, RuntimeOrchestrator, Ssb, SsbConfig,
};
use elai_providers::runtime::{
    create_runtime, resolve_backend, AuthSource as RuntimeAuthSource, RuntimeBackend,
    RuntimeFactoryConfig, RuntimeHealth, SecretString,
};
use elai_orchestrator::auth_broker_emit::broker_resolve_anthropic_auth_with_ssb;
use elai_providers::AuthSource as ProviderAuthSource;
use thiserror::Error;
use tokio::time::timeout;
use tracing::warn;

/// Hard cap on the rendered HEALTH cell width in `/runtime list`. When a
/// backend reports an `Unavailable` reason that inlines the full system
/// PATH (e.g. `BinaryNotFound`), the column would otherwise balloon to
/// ~5KB and shred the table. We truncate to this width and tell the
/// operator how to read the full message.
const HEALTH_CELL_MAX_WIDTH: usize = 60;

/// Errors returned by the `/runtime` handler. All variants render to a
/// user-facing single-line string; nothing inside leaks SecretString
/// content.
#[derive(Debug, Error)]
pub enum RuntimeCmdError {
    #[error("unknown backend '{candidate}'. Supported: {supported}")]
    UnknownBackend {
        candidate: String,
        supported: String,
    },
    #[error("unknown agent role '{candidate}'. Supported: {supported}")]
    UnknownRole {
        candidate: String,
        supported: String,
    },
    #[error("missing required argument: {0}")]
    MissingArg(&'static str),
    #[error("unknown action '{0}'. Use list, test <backend>, set --role <name> --backend <name>, or help.")]
    UnknownAction(String),
    #[error("failed to read elai_cc.toml: {0}")]
    ConfigRead(String),
    #[error("failed to parse elai_cc.toml: {0}")]
    ConfigParse(String),
    #[error("failed to write elai_cc.toml: {0}")]
    ConfigWrite(String),
    #[error("failed to construct runtime adapter: {0}")]
    AdapterConstruct(String),
}

/// Snake-case identifiers for every `AgentRole` discriminant in
/// `ipc-types`. Kept as a `&'static [&'static str]` so we can render
/// the supported-list deterministically without pulling the ipc-types
/// crate into this file's interface; the validator only needs string
/// equality.
const SUPPORTED_ROLES: &[&str] = &[
    "executor",
    "auditor",
    "researcher",
    "designer",
    "prompt_engineer",
    "context_compiler",
    "memory_writer",
    "media_validator",
    "idea_framer",
    "idea_critic",
    "auditor_concepts",
    "auditor_phases",
    "auditor_steps",
    "auditor_baby_steps",
    "auditor_baby_step_detail",
];

fn validate_role(candidate: &str) -> Result<String, RuntimeCmdError> {
    let normalised = candidate.trim().to_ascii_lowercase();
    if SUPPORTED_ROLES.iter().any(|r| *r == normalised) {
        Ok(normalised)
    } else {
        Err(RuntimeCmdError::UnknownRole {
            candidate: normalised,
            supported: SUPPORTED_ROLES.join(", "),
        })
    }
}

/// Public entrypoint for the `/runtime` slash command, used by both the
/// REPL turn loop (live_cli.rs) and the `--resume SESSION /runtime …`
/// non-interactive path (resume.rs).
///
/// Returns a human-readable string to print. Errors are surfaced via
/// `RuntimeCmdError` and the caller renders them.
pub async fn handle_runtime_slash_command(
    action: Option<&str>,
    role: Option<&str>,
    target: Option<&str>,
    project_root: &Path,
) -> Result<String, RuntimeCmdError> {
    match action {
        None | Some("list") | Some("show") => render_runtime_list(project_root).await,
        Some("test") => {
            let backend_name = target.ok_or(RuntimeCmdError::MissingArg("<backend>"))?;
            handle_runtime_test(backend_name, project_root).await
        }
        Some("set") => {
            let role_name = role.ok_or(RuntimeCmdError::MissingArg("--role <name>"))?;
            let backend_name = target.ok_or(RuntimeCmdError::MissingArg("--backend <name>"))?;
            handle_runtime_set(role_name, backend_name, project_root)
        }
        // C-44 — Composer 2 paid HTTP backend subcommands.
        // /runtime composer2 status — show adapter health + cost rollup hint.
        // /runtime composer2 enable / disable — flip [runtime.composer2].enabled.
        Some("composer2") => {
            let sub = target.ok_or(RuntimeCmdError::MissingArg("composer2 <status|enable|disable>"))?;
            handle_composer2_subcommand(sub, project_root)
        }
        Some("help") => Ok(render_runtime_help()),
        Some(other) => Err(RuntimeCmdError::UnknownAction(other.to_string())),
    }
}

/// C-44 — `/runtime composer2 <status|enable|disable>` handler.
///
/// `status` reads `[runtime.composer2]` from elai_cc.toml and prints
/// the operator-visible state. `enable` / `disable` mutate the toml
/// (atomic write via the same helpers as `/runtime set`). Cost
/// rollup is not surfaced through the CLI — that lives on the
/// RuntimePanel dashboard. The CLI sticks to config + adapter
/// presence so it can run headless.
fn handle_composer2_subcommand(
    sub: &str,
    project_root: &Path,
) -> Result<String, RuntimeCmdError> {
    match sub {
        "status" => render_composer2_status(project_root),
        "enable" => set_composer2_enabled(true, project_root),
        "disable" => set_composer2_enabled(false, project_root),
        other => Err(RuntimeCmdError::UnknownAction(format!(
            "composer2 {other} (expected status|enable|disable)"
        ))),
    }
}

fn render_composer2_status(project_root: &Path) -> Result<String, RuntimeCmdError> {
    let config_path = project_root.join("elai_cc.toml");
    let original = std::fs::read_to_string(&config_path)
        .map_err(|e| RuntimeCmdError::ConfigRead(format!("{}: {e}", config_path.display())))?;
    let doc: toml::Value = toml::from_str(&original)
        .map_err(|e| RuntimeCmdError::ConfigParse(format!("{}: {e}", config_path.display())))?;
    let stanza = doc
        .get("runtime")
        .and_then(|r| r.get("composer2"))
        .cloned();
    let cursor_key_set = std::env::var("CURSOR_API_KEY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let enabled = stanza
        .as_ref()
        .and_then(|s| s.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let model = stanza
        .as_ref()
        .and_then(|s| s.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("composer-2")
        .to_string();
    let key_status = if cursor_key_set {
        "set"
    } else {
        "MISSING (export CURSOR_API_KEY)"
    };
    let enabled_label = if enabled { "true" } else { "false" };
    Ok(format!(
        "composer2 status:\n  enabled         = {enabled_label}\n  model           = {model}\n  CURSOR_API_KEY  = {key_status}\n  toml stanza     = {}\n  cost rollup     = surfaced via dashboard RuntimePanel (CLI hides it)",
        if stanza.is_some() {
            "present"
        } else {
            "absent (defaults applied)"
        }
    ))
}

fn set_composer2_enabled(enabled: bool, project_root: &Path) -> Result<String, RuntimeCmdError> {
    let config_path = project_root.join("elai_cc.toml");
    let original = std::fs::read_to_string(&config_path)
        .map_err(|e| RuntimeCmdError::ConfigRead(format!("{}: {e}", config_path.display())))?;
    let mut doc: toml::Value = toml::from_str(&original)
        .map_err(|e| RuntimeCmdError::ConfigParse(format!("{}: {e}", config_path.display())))?;
    set_composer2_enabled_in_toml(&mut doc, enabled);
    let rendered =
        toml::to_string_pretty(&doc).map_err(|e| RuntimeCmdError::ConfigWrite(e.to_string()))?;
    atomic_write(&config_path, rendered.as_bytes())?;
    Ok(format!(
        "[OK] [runtime.composer2].enabled = {enabled} (persisted to {})",
        config_path.display()
    ))
}

fn set_composer2_enabled_in_toml(doc: &mut toml::Value, enabled: bool) {
    let root_table = match doc.as_table_mut() {
        Some(t) => t,
        // Defensive: toml::from_str on a valid file always yields a
        // table. If we ever lose that invariant, bail by rewriting the
        // root rather than panicking — the atomic write downstream
        // catches a malformed file via the parser.
        None => {
            *doc = toml::Value::Table(toml::value::Table::new());
            doc.as_table_mut().expect("just rewrote to table")
        }
    };
    let runtime_entry = root_table
        .entry("runtime".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let runtime_table = match runtime_entry {
        toml::Value::Table(t) => t,
        _ => {
            *runtime_entry = toml::Value::Table(toml::value::Table::new());
            match runtime_entry {
                toml::Value::Table(t) => t,
                _ => unreachable!("just rewrote to table"),
            }
        }
    };
    let composer2_entry = runtime_table
        .entry("composer2".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let composer2_table = match composer2_entry {
        toml::Value::Table(t) => t,
        _ => {
            *composer2_entry = toml::Value::Table(toml::value::Table::new());
            match composer2_entry {
                toml::Value::Table(t) => t,
                _ => unreachable!("just rewrote to table"),
            }
        }
    };
    composer2_table.insert("enabled".to_string(), toml::Value::Boolean(enabled));
    // Seed defaults if absent so toggling enable on a fresh stanza
    // produces a complete block without a follow-up edit.
    composer2_table
        .entry("api_key_env".to_string())
        .or_insert_with(|| toml::Value::String("CURSOR_API_KEY".to_string()));
    composer2_table
        .entry("model".to_string())
        .or_insert_with(|| toml::Value::String("composer-2".to_string()));
    composer2_table
        .entry("fallback_to".to_string())
        .or_insert_with(|| toml::Value::String("sidecar".to_string()));
}

/// Render the `/runtime list` table. Uses
/// `RuntimeOrchestrator::health_snapshot` (Wave-1B) which probes every
/// backend through the shared adapter cache and short-circuits chain
/// walks via the orchestrator's own health table — replacing the
/// pre-Wave-1B 11-sequential-probe loop.
async fn render_runtime_list(project_root: &Path) -> Result<String, RuntimeCmdError> {
    let routing = load_runtime_routing(project_root)?;
    let orchestrator = RuntimeOrchestrator::new(RuntimeBackendConfig::default());
    // J-30 T7a: production caller for `for_role`/`release_turn_lock` with a
    // non-zero turn_id so the turn lock engages and releases on the real
    // entrypoint (`/runtime list`). This is a resolve/pin probe; ignore
    // factory failures so the list command still renders.
    let _ = orchestrator.for_role(AgentRole::Executor, 1).await;
    orchestrator.release_turn_lock(1).await;
    // `health_snapshot` already calls each adapter's `health_check`
    // through the orchestrator's lazy-create + cache path. Wrap the
    // whole snapshot in a generous outer timeout so a single hung
    // adapter cannot lock the CLI; the per-row timeouts are owned by
    // each adapter's own implementation.
    let snapshot = timeout(
        Duration::from_secs((RuntimeBackend::all().len() as u64).saturating_mul(5)),
        orchestrator.health_snapshot(),
    )
    .await
    .unwrap_or_default();

    // Build a backend -> health map so the row loop below can preserve
    // the canonical `RuntimeBackend::all()` ordering regardless of the
    // snapshot's BTreeMap iteration order.
    let mut health_by_backend: rustc_hash::FxHashMap<RuntimeBackend, RuntimeHealth> =
        rustc_hash::FxHashMap::with_capacity_and_hasher(snapshot.len(), Default::default());
    for (backend, health) in snapshot {
        health_by_backend.insert(backend, health);
    }

    let mut rows: Vec<RuntimeRow> = Vec::with_capacity(RuntimeBackend::all().len());
    for backend in RuntimeBackend::all() {
        let canonical = backend.canonical_name();
        let roles: Vec<&str> = routing
            .iter()
            .filter_map(|(role, b)| (b == canonical).then_some(role.as_str()))
            .collect();
        let roles_label = if roles.is_empty() {
            "-".to_string()
        } else {
            roles.join(",")
        };
        let health = health_by_backend.remove(backend).unwrap_or_else(|| {
            RuntimeHealth::Unavailable {
                reason: "no snapshot returned (likely timeout)".to_string(),
            }
        });
        let is_stub = match &health {
            RuntimeHealth::Unavailable { reason } => {
                reason.contains("not implemented") || reason.contains("stub")
            }
            _ => false,
        };
        rows.push(RuntimeRow {
            backend: canonical.to_string(),
            roles: roles_label,
            health: format_health(&health),
            is_stub,
        });
    }
    Ok(render_runtime_table(&rows))
}

/// `/runtime test <backend>` handler.
///
/// For cloud-style adapters (`AnthropicApi`) we thread the
/// CLI's OAuth/API-key credentials into the factory config via
/// `resolve_anthropic_auth_for_factory`, otherwise the
/// `AnthropicApiRuntime` constructor surfaces `MissingField` and the
/// probe always reports `[FAIL]` — even when the cloud path is fully
/// operational at the `AnthropicClient` level. See B-37 WARN-3 audit
/// finding.
async fn handle_runtime_test(
    candidate: &str,
    project_root: &Path,
) -> Result<String, RuntimeCmdError> {
    let backend = resolve_backend(candidate).map_err(|e| RuntimeCmdError::UnknownBackend {
        candidate: e.candidate,
        supported: e.supported.join(", "),
    })?;
    let test_clock = OsClock;
    let start = test_clock.now();
    let canonical = backend.canonical_name();
    let mut cfg = RuntimeFactoryConfig::for_backend(backend);
    if backend == RuntimeBackend::AnthropicApi {
        let ssb = open_runtime_ssb_writer(project_root);
        match resolve_anthropic_auth_for_factory(ssb) {
            Ok(auth) => cfg.auth_source = Some(auth),
            Err(reason) => {
                // Short-circuit: no creds available, no point hitting the
                // adapter constructor (which would return the same shape
                // of error but with less actionable wording).
                return Ok(format!("[FAIL] {canonical}: {reason}"));
            }
        }
    }
    let adapter = create_runtime(cfg)
        .map_err(|e| RuntimeCmdError::AdapterConstruct(e.to_string()))?;
    let health = timeout(Duration::from_secs(5), adapter.health_check())
        .await
        .unwrap_or(RuntimeHealth::Unavailable {
            reason: "health probe timeout (>5s)".to_string(),
        });
    let elapsed_ms = test_clock.elapsed_since(start).as_millis();
    match health {
        RuntimeHealth::Ready => Ok(format!("[OK] {canonical} ({elapsed_ms}ms)")),
        RuntimeHealth::Degraded { reason } => Ok(format!(
            "[OK] {canonical} ({elapsed_ms}ms) — degraded: {reason}"
        )),
        RuntimeHealth::Unavailable { reason } => Ok(format!("[FAIL] {canonical}: {reason}")),
    }
}

/// Resolve Anthropic credentials for the `RuntimeFactoryConfig` consumed
/// by `AnthropicApiRuntime::from_factory_config`. Order:
///   1. `ANTHROPIC_API_KEY` env var (handled inside `resolve_auth_with_auto_refresh`)
///   2. `ANTHROPIC_AUTH_TOKEN` env var (ditto)
///   3. Saved OAuth fixture at the providers crate's canonical location
///      (auto-refreshed if expired)
/// On total absence, returns a user-facing reason string so the caller
/// can surface a `[FAIL]` row pointing at `elai login anthropic`.
///
/// `writer` threads the SSB writer into
/// `broker_resolve_anthropic_auth_with_ssb` so the I-50
/// `AuthBrokerEvent` audit row (token-acquired / refresh-failed) is
/// emitted in production. `None` (SSB unavailable on this machine)
/// preserves the prior behavior: auth still resolves, the audit row
/// is simply skipped.
fn resolve_anthropic_auth_for_factory(
    writer: Option<Arc<Ssb>>,
) -> Result<RuntimeAuthSource, String> {
    // I-50 (SCHEMA_VERSION 268): credential resolution goes through the auth broker.
    // The broker calls resolve_startup_auth_source (byte-compat with the prior direct call)
    // and emits an AuthBrokerEvent SSB row when a writer is present.
    let provider_auth = broker_resolve_anthropic_auth_with_ssb(writer).map_err(|e| {
        format!(
            "no ANTHROPIC_API_KEY env var and no usable OAuth fixture ({e}). Run `elai login anthropic`."
        )
    })?;
    Ok(provider_auth_to_runtime_auth(provider_auth))
}

/// Open the SSB writer used by `/runtime test` to emit `AuthBrokerEvent`
/// rows. Mirrors the `critique_cmd.rs` / `scip_cmd.rs` pattern:
/// `default_ssb_path_for_base(project_root)`, create parents, open
/// `Ssb::new`, downgrade any open failure to `None` (best-effort; a
/// missing SSB row is never a correctness failure for auth resolution).
fn open_runtime_ssb_writer(project_root: &Path) -> Option<Arc<Ssb>> {
    let db_path = default_ssb_path_for_base(project_root);
    let Some(parent) = db_path.parent() else {
        warn!("runtime: ssb.db has no parent directory; AuthBrokerEvent row will be skipped");
        return None;
    };
    if let Err(e) = std::fs::create_dir_all(parent) {
        warn!(err = %e, "runtime: creating ssb sessions dir failed; AuthBrokerEvent row will be skipped");
        return None;
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-runtime-test".to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    match Ssb::new(cfg, clock) {
        Ok(s) => Some(Arc::new(s)),
        Err(e) => {
            warn!(err = %e, "runtime: SSB open failed; AuthBrokerEvent row will be skipped");
            None
        }
    }
}

/// Bridge between the providers crate's `AuthSource` (raw String inner)
/// and the runtime crate's `AuthSource` (SecretString inner). We have to
/// re-wrap in `SecretString` because the runtime tier hides the raw
/// String from Debug — but the providers tier has already validated the
/// shape (api_key / bearer_token are both non-empty).
fn provider_auth_to_runtime_auth(provider: ProviderAuthSource) -> RuntimeAuthSource {
    match provider {
        ProviderAuthSource::ApiKey(key) => RuntimeAuthSource::ApiKey(SecretString::new(key)),
        ProviderAuthSource::BearerToken(token) => {
            RuntimeAuthSource::BearerToken(SecretString::new(token))
        }
        ProviderAuthSource::ApiKeyAndBearer {
            api_key,
            bearer_token,
        } => RuntimeAuthSource::ApiKeyAndBearer {
            api_key: SecretString::new(api_key),
            bearer_token: SecretString::new(bearer_token),
        },
        // `None` is unreachable in practice — `resolve_auth_with_auto_refresh`
        // returns `Err(ApiError::MissingCredentials)` rather than
        // `AuthSource::None`. Treat it defensively as an empty API key so
        // the adapter surfaces a normal MissingField on construct.
        ProviderAuthSource::None => RuntimeAuthSource::ApiKey(SecretString::new(String::new())),
    }
}

/// `/runtime set` handler. Atomically updates `elai_cc.toml` so the
/// `[agents.<role>.runtime] backend = "<canonical>"` mapping reflects
/// the new choice. We use the toml_edit-free approach: read into a
/// `toml::Value`, mutate the relevant table, serialize back, then
/// atomic-rename. `toml` is already a workspace dep — no new crate.
fn handle_runtime_set(
    role: &str,
    candidate: &str,
    project_root: &Path,
) -> Result<String, RuntimeCmdError> {
    let role_canonical = validate_role(role)?;
    let backend = resolve_backend(candidate).map_err(|e| RuntimeCmdError::UnknownBackend {
        candidate: e.candidate,
        supported: e.supported.join(", "),
    })?;
    let canonical = backend.canonical_name();
    let config_path = project_root.join("elai_cc.toml");
    let original = std::fs::read_to_string(&config_path)
        .map_err(|e| RuntimeCmdError::ConfigRead(format!("{}: {e}", config_path.display())))?;
    let mut doc: toml::Value = toml::from_str(&original)
        .map_err(|e| RuntimeCmdError::ConfigParse(format!("{}: {e}", config_path.display())))?;
    set_role_runtime_in_toml(&mut doc, &role_canonical, canonical);
    let rendered =
        toml::to_string_pretty(&doc).map_err(|e| RuntimeCmdError::ConfigWrite(e.to_string()))?;
    atomic_write(&config_path, rendered.as_bytes())?;
    Ok(format!(
        "[OK] role {role_canonical} -> backend {canonical} (persisted to {})",
        config_path.display()
    ))
}

/// Mutate `[agents.<role>.runtime] backend = "<canonical>"` inside the
/// parsed TOML document, creating intermediate tables if missing.
fn set_role_runtime_in_toml(doc: &mut toml::Value, role: &str, backend_name: &str) {
    let root_table = doc.as_table_mut().expect("toml root must be table");
    let agents = root_table
        .entry("agents".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let agents_table = match agents {
        toml::Value::Table(t) => t,
        _ => {
            *agents = toml::Value::Table(toml::value::Table::new());
            match agents {
                toml::Value::Table(t) => t,
                _ => unreachable!("just rewrote to table"),
            }
        }
    };
    let role_entry = agents_table
        .entry(role.to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let role_table = match role_entry {
        toml::Value::Table(t) => t,
        _ => {
            *role_entry = toml::Value::Table(toml::value::Table::new());
            match role_entry {
                toml::Value::Table(t) => t,
                _ => unreachable!("just rewrote to table"),
            }
        }
    };
    let runtime_entry = role_table
        .entry("runtime".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    let runtime_table = match runtime_entry {
        toml::Value::Table(t) => t,
        _ => {
            *runtime_entry = toml::Value::Table(toml::value::Table::new());
            match runtime_entry {
                toml::Value::Table(t) => t,
                _ => unreachable!("just rewrote to table"),
            }
        }
    };
    runtime_table.insert(
        "backend".to_string(),
        toml::Value::String(backend_name.to_string()),
    );
}

/// Read the current `[agents.<role>.runtime] backend = "<name>"` map
/// from disk. Missing file or missing sections yield an empty map —
/// the table renderer treats absent assignments as `-`.
fn load_runtime_routing(project_root: &Path) -> Result<Vec<(String, String)>, RuntimeCmdError> {
    let config_path = project_root.join("elai_cc.toml");
    let Ok(content) = std::fs::read_to_string(&config_path) else {
        return Ok(Vec::new());
    };
    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Err(RuntimeCmdError::ConfigParse(format!(
                "{}: {e}",
                config_path.display()
            )));
        }
    };
    let mut out: Vec<(String, String)> = Vec::new();
    if let Some(agents) = parsed.get("agents").and_then(|v| v.as_table()) {
        for (role_name, role_value) in agents {
            let Some(backend_name) = role_value
                .get("runtime")
                .and_then(|r| r.get("backend"))
                .and_then(|b| b.as_str())
            else {
                continue;
            };
            out.push((role_name.clone(), backend_name.to_string()));
        }
    }
    Ok(out)
}

/// Atomic write: write to `<path>.tmp` then rename. Mirrors the
/// gotchas.md best practice for config persistence on Windows + Linux.
/// On rename failure, the temp file is left in place so the operator
/// can recover the new content manually.
fn atomic_write(path: &Path, content: &[u8]) -> Result<(), RuntimeCmdError> {
    let tmp_path: PathBuf = {
        let mut p = path.to_path_buf();
        let new_name = match p.file_name() {
            Some(name) => {
                let mut s = name.to_os_string();
                s.push(".tmp");
                s
            }
            None => {
                return Err(RuntimeCmdError::ConfigWrite(format!(
                    "config path has no filename: {}",
                    path.display()
                )));
            }
        };
        p.set_file_name(new_name);
        p
    };
    std::fs::write(&tmp_path, content).map_err(|e| {
        RuntimeCmdError::ConfigWrite(format!("temp write {}: {e}", tmp_path.display()))
    })?;
    std::fs::rename(&tmp_path, path).map_err(|e| {
        RuntimeCmdError::ConfigWrite(format!(
            "rename {} -> {}: {e}",
            tmp_path.display(),
            path.display()
        ))
    })?;
    Ok(())
}

/// Row in the `/runtime list` table.
struct RuntimeRow {
    backend: String,
    roles: String,
    health: String,
    is_stub: bool,
}

/// Cap `row.health` at `HEALTH_CELL_MAX_WIDTH`. When `BinaryNotFound`
/// inlines the full system PATH the column would otherwise balloon to
/// ~5KB and destroy the table layout. We truncate at a UTF-8-safe
/// boundary and direct the operator to `/runtime test <backend>` for
/// the full message. See WARN-2 audit finding.
fn truncate_health_cell(s: &str, backend: &str) -> String {
    if s.len() <= HEALTH_CELL_MAX_WIDTH {
        return s.to_string();
    }
    // UTF-8-safe: walk char_indices and slice at the last index that
    // fits in HEALTH_CELL_MAX_WIDTH - 4 (room for the " ..." marker).
    let budget = HEALTH_CELL_MAX_WIDTH.saturating_sub(4);
    let cut = s
        .char_indices()
        .take_while(|(idx, _)| *idx <= budget)
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let head = &s[..cut];
    format!("{head} ... (run /runtime test {backend} for full message)")
}

fn render_runtime_table(rows: &[RuntimeRow]) -> String {
    let header = ["BACKEND", "ROLES", "HEALTH", "STUB"];
    let truncated: Vec<(String, String, String, bool)> = rows
        .iter()
        .map(|r| {
            (
                r.backend.clone(),
                r.roles.clone(),
                truncate_health_cell(&r.health, &r.backend),
                r.is_stub,
            )
        })
        .collect();
    let width_backend = truncated
        .iter()
        .map(|(b, _, _, _)| b.len())
        .max()
        .unwrap_or(0)
        .max(header[0].len());
    let width_roles = truncated
        .iter()
        .map(|(_, r, _, _)| r.len())
        .max()
        .unwrap_or(0)
        .max(header[1].len());
    let width_health = truncated
        .iter()
        .map(|(_, _, h, _)| h.len())
        .max()
        .unwrap_or(0)
        .max(header[2].len());
    let width_stub = header[3].len();
    let mut out = String::new();
    out.push_str(&format!(
        "{:<wb$}  {:<wr$}  {:<wh$}  {:<ws$}\n",
        header[0],
        header[1],
        header[2],
        header[3],
        wb = width_backend,
        wr = width_roles,
        wh = width_health,
        ws = width_stub,
    ));
    out.push_str(&format!(
        "{}  {}  {}  {}\n",
        "-".repeat(width_backend),
        "-".repeat(width_roles),
        "-".repeat(width_health),
        "-".repeat(width_stub),
    ));
    for (backend, roles, health, is_stub) in &truncated {
        let stub_cell = if *is_stub { "yes" } else { "no" };
        out.push_str(&format!(
            "{:<wb$}  {:<wr$}  {:<wh$}  {:<ws$}\n",
            backend,
            roles,
            health,
            stub_cell,
            wb = width_backend,
            wr = width_roles,
            wh = width_health,
            ws = width_stub,
        ));
    }
    out
}

fn format_health(h: &RuntimeHealth) -> String {
    match h {
        RuntimeHealth::Ready => "ready".to_string(),
        RuntimeHealth::Degraded { reason } => format!("degraded: {reason}"),
        RuntimeHealth::Unavailable { reason } => format!("unavailable: {reason}"),
    }
}

fn render_runtime_help() -> String {
    "Usage: /runtime [list|test <backend>|set --role <name> --backend <name>|composer2 <sub>|help]\n\
     \n\
     list                                 List all runtime backends with role assignments and health.\n\
     test <backend>                       Probe a single backend's health_check (5s timeout).\n\
     set --role <name> --backend <name>   Persist a per-role runtime mapping to elai_cc.toml.\n\
     composer2 status                     Show Composer 2 paid-backend toml stanza + CURSOR_API_KEY state.\n\
     composer2 enable                     Flip [runtime.composer2].enabled = true (atomic toml write).\n\
     composer2 disable                    Flip [runtime.composer2].enabled = false.\n\
     help                                 Show this message.\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_role_accepts_canonical_names() {
        assert_eq!(validate_role("executor").unwrap(), "executor");
        assert_eq!(validate_role("AUDITOR").unwrap(), "auditor");
        assert_eq!(validate_role(" idea_critic ").unwrap(), "idea_critic");
    }

    #[test]
    fn validate_role_rejects_unknown() {
        let err = validate_role("nobody").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("nobody"), "{s}");
        assert!(s.contains("executor"), "{s}");
    }

    #[test]
    fn set_role_runtime_in_toml_creates_missing_tables() {
        let mut doc: toml::Value = toml::from_str("[output]\nstrip_emoji = true\n").unwrap();
        set_role_runtime_in_toml(&mut doc, "executor", "sidecar");
        let rendered = toml::to_string(&doc).unwrap();
        assert!(rendered.contains("[agents.executor.runtime]"), "{rendered}");
        assert!(
            rendered.contains("backend = \"sidecar\""),
            "{rendered}"
        );
    }

    #[test]
    fn set_role_runtime_in_toml_overwrites_existing() {
        let mut doc: toml::Value = toml::from_str(
            "[agents.executor.runtime]\nbackend = \"anthropic_api\"\n",
        )
        .unwrap();
        set_role_runtime_in_toml(&mut doc, "executor", "sidecar");
        let rendered = toml::to_string(&doc).unwrap();
        assert!(rendered.contains("backend = \"sidecar\""), "{rendered}");
        assert!(
            !rendered.contains("backend = \"anthropic_api\""),
            "{rendered}"
        );
    }

    #[tokio::test]
    async fn handle_runtime_test_returns_fail_for_unknown_backend() {
        let err = handle_runtime_slash_command(
            Some("test"),
            None,
            Some("not-a-backend"),
            Path::new("."),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, RuntimeCmdError::UnknownBackend { .. }));
    }

    #[test]
    fn render_runtime_help_lists_actions() {
        let help = render_runtime_help();
        assert!(help.contains("list"));
        assert!(help.contains("test"));
        assert!(help.contains("set"));
    }

    #[test]
    fn load_runtime_routing_returns_empty_when_missing() {
        let tmp = std::env::temp_dir().join(format!(
            "runtime_cmd_test_missing_{}",
            std::time::SystemTime::now() // TEST_KEEP: cfg(test) unique tmp dir name
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let out = load_runtime_routing(&tmp).unwrap();
        assert!(out.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn load_runtime_routing_reads_existing_section() {
        let tmp = std::env::temp_dir().join(format!(
            "runtime_cmd_test_load_{}",
            std::time::SystemTime::now() // TEST_KEEP: cfg(test) unique tmp dir name
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let toml_path = tmp.join("elai_cc.toml");
        std::fs::write(
            &toml_path,
            "[agents.executor.runtime]\nbackend = \"sidecar\"\n[agents.auditor.runtime]\nbackend = \"anthropic_api\"\n",
        )
        .unwrap();
        let mut out = load_runtime_routing(&tmp).unwrap();
        out.sort();
        assert_eq!(
            out,
            vec![
                ("auditor".to_string(), "anthropic_api".to_string()),
                ("executor".to_string(), "sidecar".to_string()),
            ]
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn atomic_write_replaces_file_contents() {
        let tmp = std::env::temp_dir().join(format!(
            "runtime_cmd_test_atomic_{}",
            std::time::SystemTime::now() // TEST_KEEP: cfg(test) unique tmp dir name
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let path = tmp.join("elai_cc.toml");
        std::fs::write(&path, b"old").unwrap();
        atomic_write(&path, b"new").unwrap();
        let read = std::fs::read_to_string(&path).unwrap();
        assert_eq!(read, "new");
        // Tmp sibling must be gone after rename.
        let tmp_sibling = tmp.join("elai_cc.toml.tmp");
        assert!(!tmp_sibling.exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
