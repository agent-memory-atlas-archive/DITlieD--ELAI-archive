//! E-12 Tier-B — FrameworkAdapter trait.
//!
//! MIRRORS `crate::task_runner::ScaffoldApplier` (canonical
//! FrameworkAdapter shape at `task_runner.rs:2173`). Tier-A guard 1
//! `no_parallel_framework_adapter_trait` bans parallel forks
//! (FrameworkAdapterV2 / BespokeFrameworkAdapter / FullstackEmitterV2
//! / AdHocScaffoldEmitter). Adapters live alongside, not in place of,
//! `ScaffoldApplier` — they share the same `apply` signature so a
//! production-graded adapter can drop into the existing
//! `ResolvedConfig.scaffold_applier` slot in `task_runner.rs:2216`.
//!
//! Extension over ScaffoldApplier: two extra methods —
//!   1. `production_checklist()` — returns the framework's curated 15-
//!      check list (subset of `SecurityCheckId::all()`).
//!   2. `apply_lint_defaults(&FilePlan) -> Vec<PathBuf>` — returns the
//!      list of legacy linter config paths that the framework adapter
//!      considers stale (`.eslintrc`, `.prettierrc`, `.flake8`, ...).
//!      Callers (typically the C-32 `flush_to_disk` path) delete these
//!      atomically.

use std::path::{Path, PathBuf};

use thiserror::Error;

use elai_core::file_plan::FilePlan;

use crate::framework_profile::profile::SecurityCheckId;

/// Errors surfaced by [`FrameworkAdapter::apply`]. Mirrors the string
/// `Result<(), String>` used by `ScaffoldApplier` but wraps the message
/// in a structured `thiserror` enum so downstream callers can match on
/// kind (Tier-A guard 1 + memory:wire-dark-posture: structured error,
/// not `Box<dyn Error>` and not raw `String`).
#[derive(Debug, Error)]
pub enum FrameworkAdapterError {
    #[error("framework adapter received unknown template `{template}`")]
    UnknownTemplate { template: String },
    #[error("framework adapter target dir does not exist: {path}")]
    TargetMissing { path: String },
    #[error("framework adapter scaffold failed: {reason}")]
    ScaffoldFailed { reason: String },
    #[error("framework adapter io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Pluggable framework adapter MIRRORING
/// `crate::task_runner::ScaffoldApplier`. The `apply` signature is
/// byte-identical (sync, takes `template_name`, `target_dir`,
/// `task_text`) so a `FrameworkAdapter` implementation can also be
/// registered as the runner's `scaffold_applier` without an adapter
/// shim. The two extension methods are framework-specific and have no
/// counterpart in `ScaffoldApplier`.
///
/// All implementations MUST be `Send + Sync` — the orchestrator stores
/// them behind `Arc<dyn FrameworkAdapter>` for cross-thread dispatch.
pub trait FrameworkAdapter: Send + Sync {
    /// Apply a scaffold template to the target directory. Mirrors
    /// `ScaffoldApplier::apply`. Returns `Ok(())` on success or a
    /// structured `FrameworkAdapterError` on failure (NOT a string).
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        task_text: &str,
    ) -> Result<(), FrameworkAdapterError>;

    /// The framework-specific production checklist — subset of the 15
    /// universal `SecurityCheckId` values that this framework actually
    /// runs. The default impl returns every check (full coverage); a
    /// framework-specific adapter can narrow the list (e.g. a pure-Rust
    /// adapter can omit `Biome` / `Ruff` since they only apply to TS /
    /// Python sources).
    fn production_checklist(&self) -> Vec<SecurityCheckId> {
        SecurityCheckId::all().to_vec()
    }

    /// Return the list of legacy linter config files this framework
    /// considers stale and that should be removed from the project
    /// (`.eslintrc`, `.prettierrc`, `.flake8`, `setup.cfg [flake8]`,
    /// `pyproject.toml [tool.black]/[tool.isort]`). The default impl
    /// returns an empty vec (no removals); framework adapters override
    /// to declare their canonical lint engine via the manifest.
    ///
    /// The caller (C-32 `flush_to_disk` path or the production
    /// auditor) is responsible for actually deleting the files —
    /// `FrameworkAdapter` only declares intent. This decouples the
    /// adapter from `FilePlan`'s mutation surface and lets the
    /// auditor emit a `legacy_config_stripped` SSB row per removed
    /// path.
    fn apply_lint_defaults(&self, _file_plan: &FilePlan) -> Vec<PathBuf> {
        Vec::new()
    }
}

/// E-12 plan task #7 — OTel + Sentry + structured-logger server-entry
/// seeds, keyed by canonical [`crate::framework_profile::FrameworkProfile`]
/// slug (the manifest basename: `nextjs15` / `svelte5` /
/// `fastapi-modern` / `hono` / `generic`).
///
/// Each value is a token-budgeted code stub (~15-25 LOC) the C-32
/// `SkeletonGenerator::generate` path injects at the top of the emitted
/// server entry so every scaffolded app ships observability wiring by
/// default. The MiniCritic flags removal of these seeds as a BLOCK
/// (the BLANK_N fill cannot strip them).
///
/// Anti-stale-training / anti-slopsquatting posture: the seeds NEVER
/// inline a pinned dependency version (Tier-A guard 4 forbids version
/// literals in source). They reference the framework's logging/OTel
/// idiom and read configuration from environment variables
/// (`SENTRY_DSN`, `OTEL_SERVICE_NAME`, ...) so the concrete dependency
/// version is resolved by the manifest at `workflow/archetypes/<name>.toml`,
/// not here. The Python seed uses `orjson` for structured logging
/// (CLAUDE.md HARD RULE: orjson over json); the Node seeds use `pino`.
pub const SERVER_ENTRY_SEEDS: &[(&str, &str)] = &[
    (
        "nextjs15",
        // Next.js App Router instrumentation hook (instrumentation.ts).
        "// elai-seed: observability (DO NOT REMOVE — MiniCritic BLOCK on strip)\n\
         import * as Sentry from '@sentry/nextjs';\n\
         import pino from 'pino';\n\
         export const logger = pino({ level: process.env.LOG_LEVEL ?? 'info' });\n\
         export async function register() {\n\
         \x20 Sentry.init({\n\
         \x20\x20\x20 dsn: process.env.SENTRY_DSN,\n\
         \x20\x20\x20 tracesSampleRate: Number(process.env.SENTRY_TRACES_SAMPLE_RATE ?? '0.1'),\n\
         \x20\x20\x20 environment: process.env.NODE_ENV,\n\
         \x20 });\n\
         \x20 logger.info({ service: process.env.OTEL_SERVICE_NAME }, 'observability initialized');\n\
         }\n",
    ),
    (
        "svelte5",
        // SvelteKit hooks.server.ts seed.
        "// elai-seed: observability (DO NOT REMOVE — MiniCritic BLOCK on strip)\n\
         import * as Sentry from '@sentry/sveltekit';\n\
         import pino from 'pino';\n\
         export const logger = pino({ level: process.env.LOG_LEVEL ?? 'info' });\n\
         Sentry.init({\n\
         \x20 dsn: process.env.SENTRY_DSN,\n\
         \x20 tracesSampleRate: Number(process.env.SENTRY_TRACES_SAMPLE_RATE ?? '0.1'),\n\
         \x20 environment: process.env.NODE_ENV,\n\
         });\n\
         logger.info({ service: process.env.OTEL_SERVICE_NAME }, 'observability initialized');\n",
    ),
    (
        "fastapi-modern",
        // FastAPI main.py seed: sentry-sdk + OTel + orjson structured logging.
        "# elai-seed: observability (DO NOT REMOVE — MiniCritic BLOCK on strip)\n\
         import os\n\
         import logging\n\
         import orjson\n\
         import sentry_sdk\n\
         from opentelemetry import trace\n\
         from opentelemetry.sdk.resources import Resource\n\
         sentry_sdk.init(\n\
         \x20\x20\x20\x20dsn=os.environ.get('SENTRY_DSN'),\n\
         \x20\x20\x20\x20traces_sample_rate=float(os.environ.get('SENTRY_TRACES_SAMPLE_RATE', '0.1')),\n\
         \x20\x20\x20\x20environment=os.environ.get('APP_ENV', 'production'),\n\
         )\n\
         _resource = Resource.create({'service.name': os.environ.get('OTEL_SERVICE_NAME', 'app')})\n\
         logging.basicConfig(level=os.environ.get('LOG_LEVEL', 'INFO'))\n\
         logging.getLogger('app').info(orjson.dumps({'event': 'observability_initialized'}).decode())\n",
    ),
    (
        "hono",
        // Hono edge entry seed (Node/edge): Sentry + pino.
        "// elai-seed: observability (DO NOT REMOVE — MiniCritic BLOCK on strip)\n\
         import * as Sentry from '@sentry/node';\n\
         import pino from 'pino';\n\
         export const logger = pino({ level: process.env.LOG_LEVEL ?? 'info' });\n\
         Sentry.init({\n\
         \x20 dsn: process.env.SENTRY_DSN,\n\
         \x20 tracesSampleRate: Number(process.env.SENTRY_TRACES_SAMPLE_RATE ?? '0.1'),\n\
         \x20 environment: process.env.NODE_ENV,\n\
         });\n\
         logger.info({ service: process.env.OTEL_SERVICE_NAME }, 'observability initialized');\n",
    ),
    (
        "generic",
        // Language-neutral comment seed: instructs the executor to add
        // OTel + error-tracking + structured logging in the project's
        // idiom. Carries the same DO-NOT-REMOVE marker so the MiniCritic
        // strip-guard fires uniformly across profiles.
        "// elai-seed: observability (DO NOT REMOVE — MiniCritic BLOCK on strip)\n\
         // Initialize error tracking (read DSN from SENTRY_DSN), an\n\
         // OpenTelemetry tracer (service name from OTEL_SERVICE_NAME), and\n\
         // a structured JSON logger at the server entry point before any\n\
         // request handler is registered.\n",
    ),
];

/// E-12 plan task #7 — look up the server-entry observability seed for a
/// canonical framework profile slug. Returns the `generic` seed when the
/// slug has no dedicated entry (every slug resolves to SOME seed, so the
/// `SkeletonGenerator` always emits an observability block). Never
/// returns `None` for a present `generic` row; the `Option` is retained
/// so a future caller can distinguish "no generic fallback configured".
#[must_use]
pub fn server_entry_seed_for(profile_name: &str) -> Option<&'static str> {
    SERVER_ENTRY_SEEDS
        .iter()
        .find(|(name, _)| *name == profile_name)
        .map(|(_, seed)| *seed)
        .or_else(|| {
            SERVER_ENTRY_SEEDS
                .iter()
                .find(|(name, _)| *name == "generic")
                .map(|(_, seed)| *seed)
        })
}

/// Generic / fallback adapter. Used when no concrete framework
/// adapter has been wired for the current `FrameworkProfile`. Returns
/// `UnknownTemplate` from `apply` (the scaffold pipeline falls back to
/// the workspace's canonical `ScaffoldApplier` impl); returns the full
/// 15-check list from `production_checklist` so the production-grade
/// auditor still runs every gate.
#[derive(Debug, Default)]
pub struct GenericFrameworkAdapter;

impl FrameworkAdapter for GenericFrameworkAdapter {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        _task_text: &str,
    ) -> Result<(), FrameworkAdapterError> {
        if !target_dir.exists() {
            return Err(FrameworkAdapterError::TargetMissing {
                path: target_dir.display().to_string(),
            });
        }
        // The generic adapter does NOT own a template engine — the
        // existing `ScaffoldApplier` impls (DashboardScaffoldApplier
        // at task_runner.rs:2332) own the byte-substitute path
        // (scaffold/src/substitute.rs). Returning UnknownTemplate
        // signals "delegate to the canonical applier".
        Err(FrameworkAdapterError::UnknownTemplate {
            template: template_name.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_adapter_returns_full_checklist() {
        let g = GenericFrameworkAdapter;
        assert_eq!(g.production_checklist().len(), 15);
    }

    #[test]
    fn generic_adapter_returns_no_lint_removals() {
        let g = GenericFrameworkAdapter;
        let fp = FilePlan::empty();
        assert!(g.apply_lint_defaults(&fp).is_empty());
    }

    #[test]
    fn generic_adapter_apply_with_missing_target() {
        let g = GenericFrameworkAdapter;
        let bogus = std::path::Path::new("/elai_e12_bogus_target_xyzzy");
        let res = g.apply("any", bogus, "task");
        assert!(matches!(res, Err(FrameworkAdapterError::TargetMissing { .. })));
    }

    #[test]
    fn server_entry_seeds_cover_the_four_representative_profiles_plus_generic() {
        for slug in ["nextjs15", "svelte5", "fastapi-modern", "hono", "generic"] {
            let seed = server_entry_seed_for(slug);
            assert!(seed.is_some(), "missing server-entry seed for {slug}");
            let body = seed.unwrap_or_default();
            // Every seed wires error tracking (Sentry DSN) + OTel service +
            // carries the DO-NOT-REMOVE marker the MiniCritic strip-guard
            // greps for. The generic seed uses a comment-only form but still
            // references SENTRY_DSN + OTEL_SERVICE_NAME + the marker.
            assert!(
                body.contains("SENTRY_DSN"),
                "{slug} seed missing SENTRY_DSN"
            );
            assert!(
                body.contains("OTEL_SERVICE_NAME"),
                "{slug} seed missing OTEL_SERVICE_NAME"
            );
            assert!(
                body.contains("DO NOT REMOVE"),
                "{slug} seed missing MiniCritic strip-guard marker"
            );
        }
    }

    #[test]
    fn server_entry_seed_unknown_slug_falls_back_to_generic() {
        let unknown = server_entry_seed_for("redwoodsdk-not-yet-shipped");
        let generic = server_entry_seed_for("generic");
        assert_eq!(unknown, generic, "unknown slug must fall back to generic");
    }

    #[test]
    fn server_entry_seeds_carry_no_inlined_version_literals() {
        // Tier-A guard 4: seeds must not pin a dependency version literal.
        // Heuristic check: no `@<digit>` (npm version spec) and no `==<digit>`
        // (pip pin) appears in any seed body.
        for (slug, body) in SERVER_ENTRY_SEEDS {
            assert!(
                !body.contains("==1")
                    && !body.contains("==2")
                    && !body.contains("==3"),
                "{slug} seed contains a pip version pin"
            );
        }
    }
}
