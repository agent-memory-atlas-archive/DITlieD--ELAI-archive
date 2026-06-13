// cross-lang-skip: tauri-svelte
//! I-41 — `elai critique <spec-path> [--lenses L1,L4,L8] [--risk simple|heavy] [--mock]`
//!
//! Runs the I-41 spec-time critique lenses over a plan/spec artifact and writes
//! findings to `workflow/reasoning/<stem>-critique.md`.
//!
//! ## Kill-switch
//! Set `ELAI_CRITIQUE_LENSES_ENABLE=1` to activate LLM lens runs.
//! Without the env var (the default) the lens runner returns empty findings
//! and no SSB advisory rows are emitted. The `--mock` flag additionally
//! bypasses the real runtime with an inline stub for test/demo purposes.
//!
//! ## Run points (both must be wired per WIRING_CONTRACT reachable_from)
//! 1. `elai critique <spec-path>` — this file (CLI entrypoint).
//! 2. `idea_framing::run_critic_loop` post-convergence advisory call (I-41).
//!
//! No `.unwrap()` / `.expect()` on production paths.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::critique::lenses::{LensId, LensSet, LensSeverity, RiskLevel};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_providers::runtime::types::{
    CompletionConfig, CompletionResponse, Message, RuntimeAdapter,
    RuntimeBackend, RuntimeCapabilities, RuntimeFuture, RuntimeHealth, RuntimeHealthFuture,
    RuntimeStream, RuntimeFactoryConfig,
};
use elai_providers::runtime::SecretString;
use elai_providers::runtime::anthropic_api::AnthropicApiRuntime;
use tracing::{info, warn};

use crate::args::{CritiqueAction, CliOutputFormat};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CRITIQUE_AGENT_ID: &str = "cli:critique";
const CRITIQUE_SUPERSTEP: i64 = 0;

// ---------------------------------------------------------------------------
// Inline mock runtime (fixture/demo mode, local to this module)
// ---------------------------------------------------------------------------

/// Stub [`RuntimeAdapter`] that returns a minimal valid JSON findings array.
/// Used when `--mock` is passed or for contract-test fixture mode.
/// Never used on the gate/verdict path (advisory only).
pub(crate) struct StubMockRuntime;

impl RuntimeAdapter for StubMockRuntime {
    fn backend(&self) -> RuntimeBackend {
        RuntimeBackend::AnthropicApi
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::none()
    }

    fn health_check(&self) -> RuntimeHealthFuture<'_> {
        Box::pin(async { RuntimeHealth::Ready })
    }

    fn complete(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> RuntimeFuture<'_, CompletionResponse> {
        Box::pin(async move {
            // Return a minimal JSON array with one advisory finding.
            let json = r#"[
  {
    "lens": "L1",
    "severity": "CONCERN",
    "location": "stub mock finding",
    "claim": "mock: critique lenses stub response",
    "why_it_matters": "this is a fixture response; live lens runs require ELAI_CRITIQUE_LENSES_ENABLE=1 and a real runtime",
    "suggested_resolution": "set ELAI_CRITIQUE_LENSES_ENABLE=1 and provide a real runtime",
    "confidence": 0.5
  }
]"#;
            Ok(CompletionResponse {
                text: json.to_owned(),
                stop_reason: Some("end_turn".to_owned()),
                usage: (10, 50),
                model: "stub-mock".to_owned(),
            })
        })
    }

    fn stream(
        &self,
        _messages: Vec<Message>,
        _config: CompletionConfig,
    ) -> RuntimeStream<'static> {
        Box::pin(futures_util::stream::empty())
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run `elai critique <spec-path>`.
///
/// Returns `Err(String)` on unrecoverable failure. Advisory errors (lens
/// parse failures, SSB write errors) are downgraded to warnings — the CLI
/// still exits 0 and writes whatever findings are available.
pub(crate) fn run_critique(action: CritiqueAction, _output_format: CliOutputFormat) -> Result<(), String> {
    let spec_path = &action.spec_path;
    let spec_text = std::fs::read_to_string(spec_path)
        .map_err(|e| format!("critique: cannot read spec file {}: {e}", spec_path.display()))?;

    let session_id = std::env::var("ELAI_SESSION_ID")
        .unwrap_or_else(|_| "cli-critique".to_string());

    // Resolve optional SSB path.
    let db_path = locate_ssb_db();
    let ssb_opt: Option<Arc<Ssb>> = match db_path {
        Some(p) => {
            let cfg = SsbConfig {
                db_path: p,
                pool_size: None,
                project_id: "elai-critique".to_string(),
                session_id: session_id.clone(),
            };
            let clock = Arc::new(OsClock) as Arc<dyn elai_core::clock::ClockProvider>;
            match Ssb::new(cfg, clock) {
                Ok(s) => Some(Arc::new(s)),
                Err(e) => {
                    warn!(err = %e, "critique: SSB open failed; findings will not be persisted to SSB");
                    None
                }
            }
        }
        None => {
            warn!("critique: no ssb.db found; findings will not be persisted to SSB");
            None
        }
    };

    // Build the LensSet from the action's lenses or risk level.
    let lens_set = if action.lenses.is_empty() {
        LensSet::from_risk(action.risk)
    } else {
        LensSet::new(action.lenses.clone(), action.risk)
    };

    // Build tokio runtime for the async lens run.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("critique: build tokio runtime: {e}"))?;

    let findings = rt.block_on(async move {
        let ssb_ref = ssb_opt.as_ref();

        if action.mock {
            // Mock mode: use the inline stub runtime.
            let stub = StubMockRuntime;
            lens_set
                .run(&spec_text, Some(&stub), ssb_ref, &session_id)
                .await
        } else {
            // Live mode: requires ELAI_CRITIQUE_LENSES_ENABLE=1 and a real runtime.
            // Without ELAI_CRITIQUE_LENSES_ENABLE=1 LensSet::run returns empty findings.
            // Build a minimal AnthropicApi runtime from env (ANTHROPIC_API_KEY).
            match build_anthropic_runtime() {
                Ok(runtime) => {
                    lens_set
                        .run(&spec_text, Some(runtime.as_ref()), ssb_ref, &session_id)
                        .await
                }
                Err(e) => {
                    warn!(err = %e, "critique: cannot build runtime; running with no runtime (findings will be empty unless ELAI_CRITIQUE_LENSES_ENABLE is unset)");
                    lens_set
                        .run(&spec_text, None, ssb_ref, &session_id)
                        .await
                }
            }
        }
    });

    let findings = match findings {
        Ok(f) => f,
        Err(e) => {
            warn!(err = %e, "critique: lens run returned error; writing empty findings");
            Vec::new()
        }
    };

    // Write findings markdown.
    let stem = spec_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("spec");
    let out_path = resolve_output_path(stem);

    if let Err(e) = write_findings_markdown(&out_path, spec_path, &findings) {
        warn!(err = %e, path = %out_path.display(), "critique: failed to write findings markdown");
    } else {
        info!(path = %out_path.display(), finding_count = findings.len(), "critique: findings written");
    }

    // Print summary.
    if findings.is_empty() {
        println!("critique: 0 findings (kill-switch off or no runtime — set ELAI_CRITIQUE_LENSES_ENABLE=1)");
    } else {
        println!("critique: {} finding(s) written to {}", findings.len(), out_path.display());
        for f in &findings {
            let sev = match f.severity {
                LensSeverity::Flag => "FLAG",
                LensSeverity::Concern => "CONCERN",
                LensSeverity::Note => "NOTE",
            };
            println!("  [{sev}] {} @ {}", f.lens.label(), f.location);
            println!("         {}", f.claim);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn locate_ssb_db() -> Option<PathBuf> {
    // Try ELAI_SSB_PATH env, then workflow/sessions/ssb.db, then ../ssb.db
    if let Ok(p) = std::env::var("ELAI_SSB_PATH") {
        return Some(PathBuf::from(p));
    }
    let candidates = [
        "workflow/sessions/ssb.db",
        "ssb.db",
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn resolve_output_path(stem: &str) -> PathBuf {
    let dir = PathBuf::from("workflow/reasoning");
    if dir.exists() {
        dir.join(format!("{stem}-critique.md"))
    } else {
        PathBuf::from(format!("{stem}-critique.md"))
    }
}

fn write_findings_markdown(
    out_path: &Path,
    spec_path: &Path,
    findings: &[elai_orchestrator::critique::lenses::LensFinding],
) -> std::io::Result<()> {
    use std::io::Write as _;

    let mut buf = Vec::new();
    writeln!(buf, "# Critique findings: {}", spec_path.display())?;
    writeln!(buf)?;
    writeln!(buf, "Generated by `elai critique`. Advisory only — no accept/block verdict.")?;
    writeln!(buf)?;

    if findings.is_empty() {
        writeln!(buf, "_No findings (kill-switch off or runtime unavailable)._")?;
    } else {
        for (i, f) in findings.iter().enumerate() {
            let sev = f.severity.as_str();
            writeln!(buf, "## Finding {} — {} [{sev}]", i + 1, f.lens.label())?;
            writeln!(buf)?;
            writeln!(buf, "**Location:** {}", f.location)?;
            writeln!(buf, "**Claim:** {}", f.claim)?;
            writeln!(buf, "**Why it matters:** {}", f.why_it_matters)?;
            writeln!(buf, "**Suggested resolution:** {}", f.suggested_resolution)?;
            writeln!(buf, "**Confidence:** {:.2}", f.confidence)?;
            writeln!(buf)?;
        }
    }

    std::fs::write(out_path, buf)
}

fn build_anthropic_runtime() -> Result<Box<dyn RuntimeAdapter + Send + Sync>, String> {
    let mut cfg = RuntimeFactoryConfig::for_backend(RuntimeBackend::AnthropicApi);

    // Use ANTHROPIC_API_KEY or ANTHROPIC_BASE_URL (sidecar) from env.
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        cfg.api_key = Some(SecretString::new(key));
    }
    if let Ok(url) = std::env::var("ANTHROPIC_BASE_URL") {
        cfg.base_url = Some(url);
    }

    AnthropicApiRuntime::from_factory_config(&cfg)
        .map(|r| Box::new(r) as Box<dyn RuntimeAdapter + Send + Sync>)
        .map_err(|e| format!("build_anthropic_runtime: {e}"))
}
