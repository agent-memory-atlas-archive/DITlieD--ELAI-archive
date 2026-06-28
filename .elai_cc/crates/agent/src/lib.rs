//! elai_cc_agent — library target exposing agent-side primitives to
//! integration tests.
//!
//! The agent crate is primarily a binary (`src/main.rs`) that runs as a
//! sandboxed child process of the orchestrator. This library target
//! re-exports the modules that integration tests under `tests/` need to
//! exercise without spawning the full IPC loop — currently just the
//! C-45 [`dropper`] module (AgentRoleDropper + RoleFallback).
//!
//! INVARIANT: this crate MUST NOT depend on reqwest, providers, tools,
//! rules, knowledge, or orchestrator (same invariant as the binary —
//! all LLM calls are proxied via IPC). The lib target adds NO new
//! dependencies; it only re-exports existing modules so `cargo test
//! -p elai_cc_agent --test <name>` can reach them.

pub mod dropper;

pub use dropper::{AgentRoleDropper, RoleFallback, ELAI_C45_DROPPED_ROLE_ENV};
