//! E-40 — `elai scip <query|defs|refs|callers|callees|pagerank|status|import|lsp-probe|refresh>`
//! handler.
//!
//! Production caller for the orchestrator-rooted symbol-graph module
//! (`elai_orchestrator::symbol_graph`). Defends against the
//! E37_WIRE_DARK_GAP_PATTERN by being the canonical entry point that
//! every new function in `symbol_graph::{cache,query,pagerank,scip_import,lsp_oracle}`
//! is reachable from.
//!
//! Read-only — no destructive sub-verbs. Loads
//! `.claude/tools/elai_graph.json` (path-resolved from the current cwd
//! at dispatch time) and dispatches to the requested helper.
//!
//! Paper grounding deferred per workflow/sessions/s-25120/mistakes.md
//! cluster #6 (Stack Graphs / GraphCoder / CodexGraph / RANGER / Glean
//! Angle). See workflow/reasoning/E-40-decisions.md.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::symbol_graph::{
    self, oracle_lookup, oracle_lookup_with_mode, personalized_top_k, LspMode, OracleKind,
    SymbolGraphCache,
};
use elai_orchestrator::{default_ssb_path_for_base, Ssb, SsbConfig};
use elai_orchestrator::ssb::{ScipRefreshCompletedPayload, SsbPayload};
use serde::Serialize;

use crate::args::{CliOutputFormat, SymbolGraphAction};

/// Environment variable that gates SSB emission for `scip refresh`.
/// Default-OFF (wire-dark): truthy check only (`"1"`, `"true"`, `"yes"`).
const ELAI_SCIP_REFRESH_EMIT_ENABLE: &str = "ELAI_SCIP_REFRESH_EMIT_ENABLE";

/// Python script name for SCIP regeneration relative to project root.
const CODEBASE_MAPPER_PY: &str = ".claude/tools/codebase_mapper.py";
/// Env var supplying the workflow root when project-root is not passed explicitly.
const ELAI_WORKFLOW_ROOT: &str = "ELAI_WORKFLOW_ROOT";
/// Default SCIP indices directory (relative to project root / cwd).
const DEFAULT_SCIP_INDICES_DIR: &str = ".claude/tools/scip_indices";

const DEFAULT_GRAPH_PATH: &str = ".claude/tools/elai_graph.json";
const DEFAULT_SCIP_DIR: &str = ".claude/tools/scip_indices";

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(
    action: SymbolGraphAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        SymbolGraphAction::Query { symbol, kind } => run_query(&symbol, &kind, output_format),
        SymbolGraphAction::Defs { symbol } => run_query(&symbol, "defs", output_format),
        SymbolGraphAction::Refs { symbol } => run_query(&symbol, "refs", output_format),
        SymbolGraphAction::PageRank { seed, k } => run_pagerank(&seed, k, output_format),
        SymbolGraphAction::Status => run_status(output_format),
        SymbolGraphAction::ImportScip { dir } => run_import(&dir, output_format),
        SymbolGraphAction::LspProbe { symbol, language, mode } => {
            run_lsp_probe(&symbol, &language, &mode, output_format)
        }
        SymbolGraphAction::Refresh { scip_dir, project_root } => {
            run_refresh(&scip_dir, &project_root, output_format)
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct QueryReport {
    pub kind: String,
    pub symbol: String,
    pub result_count: usize,
    pub results: Vec<symbol_graph::SymbolMatch>,
    pub graph_path: String,
    pub node_count: usize,
}

fn run_query(symbol: &str, kind: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let path = graph_path();
    let mut cache = SymbolGraphCache::load(&path).map_err(|e| format!("{e}"))?;
    let kind_norm = kind.to_ascii_lowercase();
    let results = match kind_norm.as_str() {
        "defs" => symbol_graph::find_defs(&mut cache, symbol).map_err(|e| format!("{e}"))?,
        "refs" => symbol_graph::find_refs(&mut cache, symbol).map_err(|e| format!("{e}"))?,
        "callers" => {
            symbol_graph::find_callers(&mut cache, symbol).map_err(|e| format!("{e}"))?
        }
        "callees" => {
            symbol_graph::find_callees(&mut cache, symbol).map_err(|e| format!("{e}"))?
        }
        other => {
            return Err(format!(
                "scip query: unknown kind '{other}' (expected defs|refs|callers|callees)"
            ));
        }
    };
    let report = QueryReport {
        kind: kind_norm,
        symbol: symbol.to_string(),
        result_count: results.len(),
        results,
        graph_path: path.to_string_lossy().to_string(),
        node_count: cache.node_count(),
    };
    emit(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct PageRankReport {
    pub seed: String,
    pub k: u32,
    pub ranked: Vec<symbol_graph::RankedSymbol>,
    pub graph_path: String,
    pub node_count: usize,
}

fn run_pagerank(seed: &str, k: u32, output_format: CliOutputFormat) -> Result<(), String> {
    if seed.is_empty() {
        return Err("scip pagerank: --seed must be non-empty".to_string());
    }
    let path = graph_path();
    let mut cache = SymbolGraphCache::load(&path).map_err(|e| format!("{e}"))?;
    let ranked = personalized_top_k(&mut cache, seed, k as usize);
    let report = PageRankReport {
        seed: seed.to_string(),
        k,
        ranked,
        graph_path: path.to_string_lossy().to_string(),
        node_count: cache.node_count(),
    };
    emit(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct StatusReport {
    pub graph_path: String,
    pub graph_present: bool,
    pub node_count: usize,
    pub lru_capacity: usize,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let path = graph_path();
    let present = path.exists();
    let (node_count, lru_capacity) = if present {
        match SymbolGraphCache::load(&path) {
            Ok(cache) => (cache.node_count(), symbol_graph::cache::DEFAULT_LRU_CAPACITY),
            Err(e) => return Err(format!("scip status: load error: {e}")),
        }
    } else {
        (0, symbol_graph::cache::DEFAULT_LRU_CAPACITY)
    };
    let report = StatusReport {
        graph_path: path.to_string_lossy().to_string(),
        graph_present: present,
        node_count,
        lru_capacity,
    };
    emit(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct ImportReport {
    pub scip_dir: String,
    pub shard_count: usize,
}

fn run_import(dir: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let dir_path = if dir.is_empty() {
        PathBuf::from(DEFAULT_SCIP_DIR)
    } else {
        PathBuf::from(dir)
    };
    let mut cache = SymbolGraphCache::empty_with_capacity(
        symbol_graph::cache::DEFAULT_LRU_CAPACITY,
    )
    .map_err(|e| format!("{e}"))?;
    let count = symbol_graph::import_scip_indices(&dir_path, &mut cache)
        .map_err(|e| format!("{e}"))?;
    let report = ImportReport {
        scip_dir: dir_path.to_string_lossy().to_string(),
        shard_count: count,
    };
    emit(&report, output_format)
}

/// Best-effort oracle probe exposed to the dispatcher. Wrapper around
/// the default-mode `oracle_lookup` for back-compat with the E-40
/// WIRE-DARK guard; the new `lsp-probe` sub-verb (E-52 Tier-B) drives
/// the explicit-mode variant. Returns the synthetic answer when the
/// toolchain is absent (the documented RUNPOD-PENDING path).
#[allow(dead_code)]
pub(crate) async fn oracle_probe(
    symbol: &str,
    language: &str,
) -> Result<symbol_graph::OracleAnswer, String> {
    oracle_lookup(symbol, language)
        .await
        .map_err(|e| format!("{e}"))
}

#[derive(Debug, Clone, Serialize)]
struct LspProbeReport {
    pub symbol: String,
    pub language: String,
    pub mode: String,
    pub binary: String,
    pub invoked: bool,
}

/// E-52 Tier-B — production caller for
/// `symbol_graph::lsp_oracle::oracle_lookup_with_mode`. Bridges the
/// async oracle into the synchronous CLI dispatcher via a per-call
/// `tokio::runtime::Runtime` (matches the safety_cmd async-bridging
/// pattern; no new top-level runtime required because main.rs is
/// already synchronous on this path).
fn run_lsp_probe(
    symbol: &str,
    language: &str,
    mode: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    if symbol.is_empty() {
        return Err("scip lsp-probe: --symbol must be non-empty".to_string());
    }
    let lsp_mode = match mode.to_ascii_lowercase().as_str() {
        "probe" | "" => LspMode::Probe,
        "full_handshake" | "handshake" | "full" => LspMode::FullHandshake,
        other => {
            return Err(format!(
                "scip lsp-probe: unknown --mode '{other}' (expected probe|full_handshake)"
            ));
        }
    };
    // Validate language slug up-front so the synchronous error path
    // surfaces cleanly without spinning up a runtime.
    if OracleKind::from_slug(language).is_none() {
        return Err(format!(
            "scip lsp-probe: unsupported language '{language}' (rust|python|typescript|go)"
        ));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("scip lsp-probe: runtime build failed: {e}"))?;
    let answer = runtime
        .block_on(oracle_lookup_with_mode(symbol, language, lsp_mode))
        .map_err(|e| format!("scip lsp-probe: {e}"))?;
    let report = LspProbeReport {
        symbol: answer.symbol,
        language: language.to_string(),
        mode: lsp_mode.as_str().to_string(),
        binary: answer.binary,
        invoked: answer.invoked,
    };
    emit(&report, output_format)
}

/// `elai scip refresh` — invoke `codebase_mapper.py` to regenerate SCIP
/// indices and emit one `ScipRefreshCompleted` SSB row.
///
/// `scip_dir`:       path to the SCIP indices directory (defaults to
///                   `DEFAULT_SCIP_INDICES_DIR` relative to cwd).
/// `project_root`:   project root passed as the first positional to
///                   `codebase_mapper.py`. Resolved in order:
///                     1. `--project-root` CLI flag (non-empty).
///                     2. `ELAI_WORKFLOW_ROOT` env var.
///                     3. Current working directory.
///
/// The mapper script is resolved as
/// `<project_root>/<CODEBASE_MAPPER_PY>`. Returns a clear error when
/// the script is absent so operators know to regenerate it.
///
/// The SSB row is emitted only when `ELAI_SCIP_REFRESH_EMIT_ENABLE` is
/// truthy (`"1"`, `"true"`, `"yes"` — case-insensitive). SSB failure is
/// best-effort: it is logged to stderr but does NOT change the exit code
/// so that the mapper invocation result is the authoritative signal.
#[derive(Debug, Clone, Serialize)]
struct RefreshReport {
    pub scip_dir: String,
    pub project_root: String,
    pub mapper_script: String,
    pub shard_count: u32,
    pub crate_count: u32,
    pub duration_ms: u32,
    pub ssb_emitted: bool,
}

fn run_refresh(
    scip_dir: &str,
    project_root_arg: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    // Resolve project root: explicit arg > env var > cwd.
    let project_root: PathBuf = if !project_root_arg.is_empty() {
        PathBuf::from(project_root_arg)
    } else if let Ok(env_root) = std::env::var(ELAI_WORKFLOW_ROOT) {
        if !env_root.is_empty() {
            PathBuf::from(env_root)
        } else {
            std::env::current_dir()
                .map_err(|e| format!("scip refresh: cannot determine cwd: {e}"))?
        }
    } else {
        std::env::current_dir()
            .map_err(|e| format!("scip refresh: cannot determine cwd: {e}"))?
    };

    // Resolve SCIP indices directory.
    let resolved_scip_dir: PathBuf = if scip_dir.is_empty() {
        project_root.join(DEFAULT_SCIP_INDICES_DIR)
    } else {
        PathBuf::from(scip_dir)
    };

    // Locate codebase_mapper.py.
    let mapper_script = project_root.join(CODEBASE_MAPPER_PY);
    if !mapper_script.exists() {
        return Err(format!(
            "scip refresh: codebase_mapper.py not found at {}; \
             ensure the project root is correct or set ELAI_WORKFLOW_ROOT",
            mapper_script.display()
        ));
    }

    // Record start time for duration_ms.
    let start = std::time::Instant::now(); // entropy-allow: observability duration_ms only — elapsed() never leaves fn as wall-clock ts

    // Invoke: python <mapper> <project_root> elai_graph.json
    //         --compact --map elai.map
    //         --scip-dir <resolved_scip_dir>
    let status = std::process::Command::new("python")
        .arg(&mapper_script)
        .arg(&project_root)
        .arg("elai_graph.json")
        .arg("--compact")
        .arg("--map")
        .arg("elai.map")
        .arg("--scip-dir")
        .arg(&resolved_scip_dir)
        .status()
        .map_err(|e| format!("scip refresh: failed to launch codebase_mapper.py: {e}"))?;

    if !status.success() {
        return Err(format!(
            "scip refresh: codebase_mapper.py exited with status {}",
            status.code().unwrap_or(-1)
        ));
    }

    let duration_ms = u32::try_from(start.elapsed().as_millis().min(u128::from(u32::MAX)))
        .unwrap_or(u32::MAX);

    // Count .scip files and distinct crate names from the SCIP directory.
    // Deterministic: no mapper stdout parsing required.
    let (shard_count, crate_count) =
        count_scip_shards_and_crates(&resolved_scip_dir).unwrap_or((0, 0));

    // SSB row — gated on ELAI_SCIP_REFRESH_EMIT_ENABLE (default-OFF).
    let ssb_emitted = if is_env_truthy(ELAI_SCIP_REFRESH_EMIT_ENABLE) {
        let ts_ms = u64::try_from(OsClock.now_ms().min(u128::from(u64::MAX)))
        .unwrap_or(u64::MAX);
        let payload = ScipRefreshCompletedPayload {
            shard_count,
            crate_count,
            duration_ms,
            ts_ms,
        };
        match emit_scip_refresh_ssb(&project_root, payload) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("warning: scip refresh: failed to emit ScipRefreshCompleted SSB row: {e}");
                false
            }
        }
    } else {
        false
    };

    let report = RefreshReport {
        scip_dir: resolved_scip_dir.to_string_lossy().into_owned(),
        project_root: project_root.to_string_lossy().into_owned(),
        mapper_script: mapper_script.to_string_lossy().into_owned(),
        shard_count,
        crate_count,
        duration_ms,
        ssb_emitted,
    };
    emit(&report, output_format)
}

/// Count `.scip` files and distinct crate names in `scip_dir`.
///
/// Crate name is derived from each shard's stem by stripping a trailing
/// `-<hash>` suffix if present (e.g. `elai_core-a3f0b1c2.scip` ->
/// `elai_core`). Files without a recognisable stem are counted but their
/// "crate name" falls back to the full stem to avoid undercounting.
fn count_scip_shards_and_crates(scip_dir: &PathBuf) -> Result<(u32, u32), String> {
    let entries = std::fs::read_dir(scip_dir)
        .map_err(|e| format!("count_scip_shards_and_crates: read_dir failed: {e}"))?;

    let mut shard_count: u32 = 0;
    let mut crate_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in entries {
        let entry = entry
            .map_err(|e| format!("count_scip_shards_and_crates: dir entry error: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("scip") {
            shard_count = shard_count.saturating_add(1);
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Strip trailing `-<8-hex-chars>` hash suffix if present.
                let crate_name = if let Some(idx) = stem.rfind('-') {
                    let suffix = &stem[idx + 1..];
                    if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                        stem[..idx].to_string()
                    } else {
                        stem.to_string()
                    }
                } else {
                    stem.to_string()
                };
                crate_names.insert(crate_name);
            }
        }
    }

    let crate_count = u32::try_from(crate_names.len()).unwrap_or(u32::MAX);
    Ok((shard_count, crate_count))
}

/// Emit one `ScipRefreshCompleted` SSB row using the per-call `Ssb::new`
/// pattern (matches `archive_verify.rs`). Uses a current-thread tokio
/// runtime so the synchronous CLI caller does not need to manage async.
fn emit_scip_refresh_ssb(
    project_root: &PathBuf,
    payload: ScipRefreshCompletedPayload,
) -> Result<(), String> {
    let db_path = default_ssb_path_for_base(project_root);
    let parent = db_path.parent().ok_or_else(|| {
        "scip refresh: ssb.db has no parent directory".to_string()
    })?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("scip refresh: creating ssb sessions dir: {e}"))?;

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-scip-refresh".to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("scip refresh: tokio runtime build failed: {e}"))?;
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| format!("scip refresh: opening SSB writer: {e}"))?;
        ssb.write_payload(
            "cli:scip-refresh",
            0,
            SsbPayload::ScipRefreshCompleted(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| format!("scip refresh: writing ScipRefreshCompleted row: {e}"))?;
        Ok(())
    })
}

/// Returns `true` if `env_var` is set and its value is `"1"`, `"true"`,
/// or `"yes"` (case-insensitive). Default-OFF for all unset/empty values.
fn is_env_truthy(env_var: &str) -> bool {
    match std::env::var(env_var) {
        Ok(val) => matches!(val.to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

/// Use [`OracleKind::canonical_binary`] downstream so the dispatcher can
/// resolve language slugs without re-exporting the type.
#[allow(dead_code)]
pub(crate) fn canonical_oracle_binary(slug: &str) -> Option<&'static str> {
    OracleKind::from_slug(slug).map(OracleKind::canonical_binary)
}

fn graph_path() -> PathBuf {
    PathBuf::from(DEFAULT_GRAPH_PATH)
}

fn emit<T: Serialize>(report: &T, output_format: CliOutputFormat) -> Result<(), String> {
    match output_format {
        CliOutputFormat::Json => {
            let s = serde_json::to_string_pretty(report)
                .map_err(|e| format!("scip: json serialize error: {e}"))?;
            println!("{s}");
            Ok(())
        }
        CliOutputFormat::Toml => {
            // toml::to_string surfaces an Err for shapes it cannot encode;
            // surface it as a String for the dispatcher.
            let s = toml::to_string(report)
                .map_err(|e| format!("scip: toml serialize error: {e}"))?;
            println!("{s}");
            Ok(())
        }
        CliOutputFormat::Text => {
            // Default: pretty-printed JSON. Mirrors the safety_cmd Text
            // arm so operators get a stable rendering.
            let s = serde_json::to_string_pretty(report)
                .map_err(|e| format!("scip: text-default serialize error: {e}"))?;
            println!("{s}");
            Ok(())
        }
    }
}
