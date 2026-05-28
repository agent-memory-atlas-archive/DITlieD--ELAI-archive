//! E-48 — Research substrate root.
//!
//! Holds the E-48 web-research-orchestrator hardening modules in a single
//! subtree so they sit alongside the future E-22 + E-23 research add-ons
//! the session prompt anticipates ("next to E-22, E-23, E-38 research/").
//! Today this contains only the [`web_orchestrator`] subtree; future
//! E-22 (research index telemetry) and E-23 (research discourse) will
//! land as siblings.
//!
//! All producer telemetry rides the single
//! [`crate::ssb::SsbActionType::WebResearchEvent`] variant per
//! the Pattern-2 single-variant precedent — no per-task SsbActionType
//! cascade. See `workflow/reasoning/E-48-decisions.md`.

pub mod web_orchestrator;
