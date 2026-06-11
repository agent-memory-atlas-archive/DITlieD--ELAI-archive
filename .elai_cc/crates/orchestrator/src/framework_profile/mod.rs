//! E-12 Tier-B — Framework profile module.
//!
//! Production-grade fullstack codegen needs a stable mapping from
//! `ProjectClass` (B-40 idea-framing output) to a concrete framework
//! profile (Wasp / Encore / RedwoodSDK / T3-Stack / Hono / FastAPI /
//! NextAppRouter / Generic). The module ships three layers:
//!
//!   1. [`profile::FrameworkProfile`] — value type. Carries the profile
//!      name, version pin source path, language, package manager, lint
//!      config path, test runner, and the list of production-checklist
//!      security check IDs to run. Versions are NOT inlined as string
//!      literals — they live in `workflow/archetypes/*.toml` manifests
//!      that the registry loads at startup (Tier-A guard 4
//!      `no_hardcoded_unpinned_framework_versions_in_source` enforces
//!      this).
//!
//!   2. [`adapter::FrameworkAdapter`] — trait MIRRORING the canonical
//!      `crate::task_runner::ScaffoldApplier` shape (Tier-A guard 1
//!      `no_parallel_framework_adapter_trait`). Three methods:
//!      `apply(template_name, target_dir, task_text)` (sync; matches
//!      ScaffoldApplier exactly), `production_checklist()` (returns the
//!      framework-specific 15-check list IDs), and
//!      `apply_lint_defaults(&mut FilePlan)` (strips legacy linter
//!      configs).
//!
//!   3. [`registry::FrameworkProfileRegistry`] — registry that loads
//!      manifests from `workflow/archetypes/` and resolves the
//!      `ProjectClass -> FrameworkProfile` mapping via the
//!      `PROJECT_CLASS_TO_FRAMEWORK` const table. The entry point is
//!      [`get_framework_profile`] which returns the current project's
//!      profile.
//!
//! Wire-dark posture (per `workflow/reasoning/E-12-tier-b-decisions.md`
//! + memory rule `wire-dark-posture` 5th application): registry loads
//! manifests if the archetypes dir is present; if absent, returns
//! `RegistryEmpty` and the caller falls back to the Generic profile.
//! The 7 framework manifests authored in this session live at
//! `workflow/archetypes/nextjs15.toml`, `svelte5.toml`, `fastapi-modern.toml`,
//! and `hono.toml` (4 representative — Wasp / Encore / RedwoodSDK / T3
//! manifests are RUNPOD-PENDING per the decisions doc).

pub mod adapter;
pub mod profile;
pub mod registry;

pub use adapter::{
    server_entry_seed_for, FrameworkAdapter, FrameworkAdapterError, GenericFrameworkAdapter,
    SERVER_ENTRY_SEEDS,
};
pub use profile::{
    framework_for_project_class, FrameworkProfile, FrameworkProfileError, SecurityCheckId,
    PROJECT_CLASS_TO_FRAMEWORK,
};
pub use registry::{
    get_framework_profile, FrameworkProfileRegistry, FrameworkProfileRegistryError,
};
