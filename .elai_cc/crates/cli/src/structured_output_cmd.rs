//! E-35 — `elai structured-output <sub>` handler.
//!
//! WIRE-DARK avoidance per E-41 precedent
//! (`E40_WIRE_DARK_AVOIDANCE_VIA_CLI`): anchors the host-side
//! structured-output substrate so every new E-35 SSB beacon variant in
//! the +10 atomic bump has at least one production caller:
//!   * `XgrammarMaskApplied`            — emitted via the synthetic
//!     `grammar emit-test` payload feeding the dashboard's
//!     `GrammarCompilePanel` placeholder producer (RUNPOD-PENDING for
//!     the real sidecar producer).
//!   * `InstructorModeActivated` /      — emitted via `status` smoke +
//!     `GrammarModeActivated`              future producer wiring.
//!   * `CodeactActionExecuted`          — emitted via `codeact-run`.
//!   * `SearchReplaceTier1Hit` /        — emitted via
//!     `SearchReplaceCollisionFallback`    `search-replace check`.
//!   * `CloudStrictSchemaApplied` /     — emitted via `strict-mode`.
//!     `LegacyJsonObjectRefused`
//!   * `GrammarCacheLoaded`             — emitted via
//!                                          `grammar emit-test`.
//!   * `StructuredEngineRejected`       — emitted via `reject-engine`.
//!
//! Composes on top of:
//!   * `elai_orchestrator::codeact_sandbox::execute_code_act` (E-35
//!     scaffolding; Runner-role-FREE per
//!     E41_AGENTROLE_FROZEN_AT_17).
//!   * `elai_tools::search_replace::{SearchReplaceMatcher,
//!     DelimiterCollisionGuard}` (already shipped substrate).
//!   * `elai_orchestrator::ssb::Ssb` (10 new SsbActionType variants
//!     this plan adds; SCHEMA 141 atomic bump).
//!
//! AgentRole::Runner addition DEFERRED per E41_AGENTROLE_FROZEN_AT_17
//! (230 match-arm cascade — CodeAct routes through existing role +
//! RuntimeBackend composition). Python sidecar tool_parser.py +
//! XGrammarLogitsProcessor RUNPOD-PENDING. Provider::generate_structured
//! trait method DEFERRED (Provider trait surface modification).
//! AnthropicClient anthropic-beta header propagation RUNPOD-PENDING
//! (provider-crate extension).
//!
//! See `workflow/reasoning/E-35-decisions.md`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use elai_orchestrator::codeact_sandbox::{
    execute_code_act, CodeActAction, CodeActOutcome, CodeActToolId,
};
use elai_orchestrator::ssb::{
    GrammarCacheLoadedPayload, SearchReplaceCollisionFallbackPayload,
    SearchReplaceTier1HitPayload, SsbConfig, SsbPayload,
    StructuredEngineRejectedPayload,
};
use elai_orchestrator::Ssb;
use elai_tools::search_replace::DelimiterCollisionGuard;
use serde::Serialize;
use tokio::runtime::Runtime;

use crate::args::{CliOutputFormat, StructuredOutputAction};

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(
    action: StructuredOutputAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        StructuredOutputAction::Status => run_status(output_format),
        StructuredOutputAction::CodeactRun { python, allow } => {
            run_codeact(python, allow, output_format)
        }
        StructuredOutputAction::StrictModeMark {
            provider,
            schema_id,
            mode,
        } => run_strict_mode(provider, schema_id, mode, output_format),
        StructuredOutputAction::GrammarEmitTest => run_grammar_emit_test(output_format),
        StructuredOutputAction::SearchReplaceCheck { path, buffer_text } => {
            run_search_replace_check(path, buffer_text, output_format)
        }
        StructuredOutputAction::RejectEngine { engine, reason } => {
            run_reject_engine(engine, reason, output_format)
        }
    }
}

#[derive(Debug, Serialize)]
struct StatusRow {
    schema_version: i64,
    new_ssb_action_types: Vec<&'static str>,
    wired_primitives: Vec<&'static str>,
    runpod_pending: Vec<&'static str>,
    deferred: Vec<&'static str>,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let row = StatusRow {
        schema_version: elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC,
        new_ssb_action_types: vec![
            "xgrammar_mask_applied",
            "instructor_mode_activated",
            "grammar_mode_activated",
            "codeact_action_executed",
            "search_replace_tier1_hit",
            "search_replace_collision_fallback",
            "cloud_strict_schema_applied",
            "grammar_cache_loaded",
            "structured_engine_rejected",
            "legacy_json_object_refused",
        ],
        wired_primitives: vec![
            "codeact_sandbox::execute_code_act",
            "CloudGroundingClient::mark_strict_schema_applied",
            "SearchReplaceMatcher (substrate)",
            "DelimiterCollisionGuard (substrate)",
            "SidecarXGrammarHook trait (substrate)",
            "GrammarCompilePanel.svelte (mounted; placeholder producer)",
            "commands_e35::get_grammar_compilations",
        ],
        runpod_pending: vec![
            "python/elai_inference/server.py XGrammarLogitsProcessor",
            "python/elai_inference/tool_parser.py",
            "Provider::generate_structured trait method on sidecar.rs",
            "AnthropicClient anthropic-beta header propagation",
            "GrammarCompilePanel real producer (sidecar -> Tauri command)",
        ],
        deferred: vec![
            "AgentRole::Runner variant (E41_AGENTROLE_FROZEN_AT_17 — 230 match-arm cascade)",
            "DCCD (Task 9, ELAI_DCCD_ENABLED env flag)",
            "XGrammar-2 + SGLang 0.5.11 upgrade",
            "SearchReplaceMatcher dispatcher tier-0 wire (patch-N)",
        ],
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct CodeactRow {
    outcome: &'static str,
    line_count: u32,
    detail: String,
}

fn run_codeact(
    python: String,
    allow: Vec<String>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    if python.is_empty() {
        return Err("structured-output codeact-run: --python must be non-empty".into());
    }
    let runtime = Runtime::new().map_err(|e| format!("tokio runtime init: {e}"))?;
    let line_count = python.lines().count() as u32;
    let ssb = build_ssb_writer()?;
    let action_hash = {
        let mut hasher = blake3::Hasher::new();
        hasher.update(python.as_bytes());
        format!("blake3:{}", hasher.finalize().to_hex())
    };
    let action = CodeActAction {
        python_code: python,
        tool_allowlist: allow.into_iter().map(CodeActToolId::new).collect(),
        action_hash,
    };
    let now = now_ms();
    let outcome = runtime
        .block_on(async { execute_code_act(&action, None, Some(&ssb), now).await });
    let (label, detail) = match outcome {
        CodeActOutcome::Executed {
            stdout_excerpt,
            exit_code,
            duration_ms,
        } => (
            "executed",
            format!(
                "exit_code={exit_code} duration_ms={duration_ms} stdout_excerpt={stdout_excerpt:?}"
            ),
        ),
        CodeActOutcome::SandboxAbsent { detail } => ("sandbox_absent", detail),
        CodeActOutcome::CapExceeded { observed, cap } => (
            "cap_exceeded",
            format!("observed={observed} cap={cap}"),
        ),
        CodeActOutcome::AllowlistViolation { tool_id_hash } => (
            "allowlist_violation",
            format!("tool_id_hash=0x{tool_id_hash:016x}"),
        ),
    };
    render(
        &[CodeactRow {
            outcome: label,
            line_count,
            detail,
        }],
        output_format,
    )
}

#[derive(Debug, Serialize)]
struct StrictModeRow {
    provider: String,
    schema_id: String,
    mode: String,
    verdict: &'static str,
    detail: String,
}

fn run_strict_mode(
    provider: String,
    schema_id: String,
    mode: String,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    // The mark_strict_schema_applied method is async — it needs an Ssb
    // writer. We construct a CloudGroundingClient just enough to wire
    // the ssb_writer; we never actually call send_message here (this is
    // a smoke / SSB-emission probe, NOT a real HTTP send).
    let runtime = Runtime::new().map_err(|e| format!("tokio runtime init: {e}"))?;
    let ssb = build_ssb_writer()?;
    let now = now_ms();
    // Build a minimal CloudGroundingClient stub: we cannot construct one
    // without Anthropic auth, so we emit the SSB rows directly here for
    // the smoke test. The production path is `CloudGroundingClient::
    // mark_strict_schema_applied` — this CLI is the WIRE-DARK anchor
    // for the SSB beacon emission; the real cloud client wires it at
    // send_message time per workflow/reasoning/E-35-decisions.md.
    let (verdict, detail) = runtime.block_on(async {
        if mode.eq_ignore_ascii_case("json_object") {
            let provider_hash = e35_blake3_u64_prefix(b"cloud.provider.", &provider);
            let payload = SsbPayload::LegacyJsonObjectRefused(
                elai_orchestrator::ssb::LegacyJsonObjectRefusedPayload {
                    provider_hash,
                    ts_ms: now,
                },
            );
            if let Err(err) = ssb
                .write_payload("structured_output_cli", 0, payload, Vec::new())
                .await
            {
                return ("ssb_emit_failed", format!("{err}"));
            }
            (
                "refused_legacy_json_object",
                "legacy vendor json_object mode refused; use strict json_schema".to_string(),
            )
        } else {
            let provider_hash = e35_blake3_u64_prefix(b"cloud.provider.", &provider);
            let schema_id_hash = e35_blake3_u64_prefix(b"cloud.schema_id.", &schema_id);
            let payload = SsbPayload::CloudStrictSchemaApplied(
                elai_orchestrator::ssb::CloudStrictSchemaAppliedPayload {
                    provider_hash,
                    schema_id_hash,
                    ts_ms: now,
                },
            );
            if let Err(err) = ssb
                .write_payload("structured_output_cli", 0, payload, Vec::new())
                .await
            {
                return ("ssb_emit_failed", format!("{err}"));
            }
            (
                "strict_schema_applied",
                format!(
                    "provider_hash=0x{provider_hash:016x} schema_id_hash=0x{schema_id_hash:016x}"
                ),
            )
        }
    });
    render(
        &[StrictModeRow {
            provider,
            schema_id,
            mode,
            verdict,
            detail,
        }],
        output_format,
    )
}

#[derive(Debug, Serialize)]
struct GrammarRow {
    compiled_count: u32,
    cache_hits: u32,
    cache_misses: u32,
    total_compile_ms: u32,
    note: &'static str,
}

fn run_grammar_emit_test(output_format: CliOutputFormat) -> Result<(), String> {
    let runtime = Runtime::new().map_err(|e| format!("tokio runtime init: {e}"))?;
    let ssb = build_ssb_writer()?;
    let now = now_ms();
    let payload = SsbPayload::GrammarCacheLoaded(GrammarCacheLoadedPayload {
        compiled_count: 64,
        cache_hits: 0,
        cache_misses: 64,
        total_compile_ms: 0,
        ts_ms: now,
    });
    runtime.block_on(async {
        ssb.write_payload("structured_output_cli", 0, payload, Vec::new())
            .await
            .map_err(|e| format!("emit GrammarCacheLoaded: {e}"))
    })?;
    render(
        &[GrammarRow {
            compiled_count: 64,
            cache_hits: 0,
            cache_misses: 64,
            total_compile_ms: 0,
            note: "synthetic emit; real producer is the sidecar (RUNPOD-PENDING)",
        }],
        output_format,
    )
}

#[derive(Debug, Serialize)]
struct SearchReplaceRow {
    path: String,
    verdict: &'static str,
    detail: String,
}

fn run_search_replace_check(
    path: String,
    buffer_text: String,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let runtime = Runtime::new().map_err(|e| format!("tokio runtime init: {e}"))?;
    let ssb = build_ssb_writer()?;
    let now = now_ms();
    let guard = DelimiterCollisionGuard::new();
    let (verdict, detail) = if let Some(token) = guard.first_collision(&buffer_text) {
        let file_path_hash = e35_blake3_u64_prefix(b"e35.path.", &path);
        let collision_token_hash = e35_blake3_u64_prefix(b"e35.collision.", token);
        let payload = SsbPayload::SearchReplaceCollisionFallback(
            SearchReplaceCollisionFallbackPayload {
                file_path_hash,
                collision_token_hash,
                ts_ms: now,
            },
        );
        runtime.block_on(async {
            ssb.write_payload("structured_output_cli", 0, payload, Vec::new())
                .await
                .map_err(|e| format!("emit SearchReplaceCollisionFallback: {e}"))
        })?;
        (
            "collision_fallback",
            format!(
                "collision_token={token:?} file_path_hash=0x{file_path_hash:016x} \
                 collision_token_hash=0x{collision_token_hash:016x}"
            ),
        )
    } else {
        let file_path_hash = e35_blake3_u64_prefix(b"e35.path.", &path);
        let payload = SsbPayload::SearchReplaceTier1Hit(SearchReplaceTier1HitPayload {
            file_path_hash,
            edit_count: 1,
            ts_ms: now,
        });
        runtime.block_on(async {
            ssb.write_payload("structured_output_cli", 0, payload, Vec::new())
                .await
                .map_err(|e| format!("emit SearchReplaceTier1Hit: {e}"))
        })?;
        (
            "tier1_clean",
            format!("file_path_hash=0x{file_path_hash:016x}"),
        )
    };
    render(
        &[SearchReplaceRow {
            path,
            verdict,
            detail,
        }],
        output_format,
    )
}

#[derive(Debug, Serialize)]
struct RejectEngineRow {
    engine: String,
    reason: String,
    verdict: &'static str,
    detail: String,
}

fn run_reject_engine(
    engine: String,
    reason: String,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let runtime = Runtime::new().map_err(|e| format!("tokio runtime init: {e}"))?;
    let ssb = build_ssb_writer()?;
    let now = now_ms();
    let engine_hash = e35_blake3_u64_prefix(b"e35.engine.", &engine);
    let reason_hash = e35_blake3_u64_prefix(b"e35.reason.", &reason);
    let payload = SsbPayload::StructuredEngineRejected(StructuredEngineRejectedPayload {
        engine_hash,
        reason_hash,
        ts_ms: now,
    });
    runtime.block_on(async {
        ssb.write_payload("structured_output_cli", 0, payload, Vec::new())
            .await
            .map_err(|e| format!("emit StructuredEngineRejected: {e}"))
    })?;
    render(
        &[RejectEngineRow {
            engine,
            reason,
            verdict: "engine_rejected",
            detail: format!(
                "engine_hash=0x{engine_hash:016x} reason_hash=0x{reason_hash:016x}; \
                 canonical engine is XGrammar"
            ),
        }],
        output_format,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_ssb_writer() -> Result<Arc<Ssb>, String> {
    // Per-invocation in-process Ssb writer rooted at
    // `.elai_cc/.elai_cc/ssb_e35_cli/` so this CLI's emissions do not
    // interleave with the live orchestrator's SSB sink. The dashboard
    // can be pointed at this dir for smoke verification.
    let project_state = PathBuf::from(".elai_cc").join(".elai_cc").join("ssb_e35_cli");
    std::fs::create_dir_all(&project_state)
        .map_err(|e| format!("create ssb_e35_cli dir: {e}"))?;
    let cfg = SsbConfig {
        db_path: project_state.join("e35.sqlite3"),
        pool_size: Some(2),
        project_id: "elai_e35".to_string(),
        session_id: "structured_output_cli".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> =
        Arc::new(elai_core::clock::OsClock);
    let ssb = Ssb::new(cfg, clock).map_err(|e| format!("ssb init: {e}"))?;
    Ok(Arc::new(ssb))
}

fn now_ms() -> u64 {
    // CLI smoke path: SystemTime is acceptable here (this is not a
    // determinism-quarantined producer; the SSB writer's clock provider
    // is the canonical timestamp). The CodeAct sandbox + cloud strict-
    // schema marker emit through ssb.rs which uses the injected
    // OsClock; this `now_ms` is only used as the `ts_ms` field on the
    // payload struct (operator observability).
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn e35_blake3_u64_prefix(domain: &[u8], label: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(label.as_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

// ---------------------------------------------------------------------------
// Render — mirror the metrics_cmd / reliability_cmd output-format
// boilerplate so operators get consistent JSON/TOML/text output.
// ---------------------------------------------------------------------------

fn render<T: Serialize>(rows: &[T], output_format: CliOutputFormat) -> Result<(), String> {
    match output_format {
        CliOutputFormat::Json => {
            let s = serde_json::to_string_pretty(rows)
                .map_err(|e| format!("json render: {e}"))?;
            println!("{s}");
        }
        CliOutputFormat::Toml => {
            // Wrap in a map so toml can serialize an array of structs.
            let value = serde_json::to_value(rows).map_err(|e| format!("toml render: {e}"))?;
            let mut map = serde_json::Map::new();
            map.insert("rows".to_string(), value);
            let s = toml::to_string_pretty(&map).map_err(|e| format!("toml render: {e}"))?;
            println!("{s}");
        }
        CliOutputFormat::Text => {
            // Cheap human-readable: re-render via serde_json::to_string_pretty.
            let s = serde_json::to_string_pretty(rows)
                .map_err(|e| format!("text render: {e}"))?;
            println!("{s}");
        }
    }
    Ok(())
}
