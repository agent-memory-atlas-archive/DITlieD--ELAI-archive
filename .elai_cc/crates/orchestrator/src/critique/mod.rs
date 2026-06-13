// cross-lang-skip: tauri-svelte
//! I-41 — spec-time critique lenses.
//!
//! Advisory-only module.  Zero imports from `idea_framing::CriticFinding`,
//! `idea_framing::Severity`, or any gate-plane type.  Isolation is
//! compiler-enforced: a `use crate::critique` in gate code would produce
//! no usable verdict types (this module exports none that map to gate verdicts).
//!
//! # Wire-dark surface (WIRE-DARK\[i41-framing-surface\])
//!
//! The `LensSet::run` advisory call inside `run_critic_loop` is gated on
//! `runtime.is_some()`.  The `frame_idea` call site passes `runtime = None`
//! because `frame_idea` has no `RuntimeAdapter` parameter; the advisory lens
//! pass therefore never fires on the idea-framing path.  This surface is
//! registered in `wire-dark-allowlist.toml` (`WIRE-DARK[i41-framing-surface]`,
//! promotion target I-42) and is not a regression — the CLI `elai critique`
//! path is the active surface and exercises `LensSet::run` end-to-end.

pub mod lenses;
