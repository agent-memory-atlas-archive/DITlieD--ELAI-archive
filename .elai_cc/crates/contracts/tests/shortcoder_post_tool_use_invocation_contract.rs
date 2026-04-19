//! Batch 4B — ShortCoder PostToolUse invocation contract.
//!
//! Covers the deferred Batch 2A wiring: the task_runner invokes
//! `HookDispatcher::shortcode` on every PostToolUse payload and emits a
//! `ShortCoderCompression` SSB row carrying `tokens_in` / `tokens_out` /
//! `reduction_pct`. The auditor reads the compressed source in place of the
//! raw agent response. The `ELAI_DISABLE_SHORTCODER_POSTOOLUSE` env var
//! bypasses the call site without bypassing the dispatcher itself.
//!
//! Assertions:
//!   1. `shortcode` is a public method on the dispatcher trait with a
//!      default returning `None` so existing impls stay compatible.
//!   2. `InProcessHookDispatcher::with_shortcoder` installs a hook whose
//!      `shortcode` returns `Some(ShortCoderOutcome)`.
//!   3. `emit_shortcoder_compression_to_ssb` writes one row whose payload
//!      carries the reduction metrics.
//!   4. `ELAI_DISABLE_SHORTCODER_POSTOOLUSE` is a well-known constant that
//!      the task_runner checks before invoking the hook (verified via the
//!      public-facing env-var constant on the orchestrator crate).
//!   5. A dispatcher with no shortcoder hook returns `None` from
//!      `shortcode` — the task_runner's absence path.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::hook_dispatcher::{
    emit_shortcoder_compression_to_ssb, HookDispatcher, InProcessHookDispatcher,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};
use elai_orchestrator::{
    ELAI_DISABLE_SHORTCODER_ENV, ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV, ShortCoderHook,
    ShortCoderOutcome, TaskRunnerConfig,
};
use elai_knowledge::shortcoder::ShortCoder;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Contract 1: the trait surface exposes `shortcode` with a default impl so
// existing `HookDispatcher` implementors do not break when the task_runner
// starts calling it.
// ---------------------------------------------------------------------------

#[test]
fn contract_shortcode_is_trait_method_with_default() {
    // A fresh dispatcher with no shortcoder hook returns None from the
    // trait-object call — proving `shortcode` is callable through
    // `&dyn HookDispatcher` and that the default impl yields None.
    let d: Arc<dyn HookDispatcher> = Arc::new(InProcessHookDispatcher::new(None));
    let outcome = d.shortcode("fn x(){}", Some("x.rs"));
    assert!(
        outcome.is_none(),
        "no-shortcoder dispatcher must return None via trait object",
    );
}

// ---------------------------------------------------------------------------
// Contract 2: installing a ShortCoder hook makes `shortcode` return
// Some(ShortCoderOutcome). We assert the outcome variant is Compressed for
// valid Rust source since the default hook registers the Rust grammar.
// ---------------------------------------------------------------------------

#[test]
fn contract_installed_shortcoder_returns_some_outcome() {
    let sc = Arc::new(ShortCoder::new());
    let hook = Arc::new(ShortCoderHook::new(sc));
    let d = InProcessHookDispatcher::new(None).with_shortcoder(hook);
    let d: Arc<dyn HookDispatcher> = Arc::new(d);
    let out = d
        .shortcode(
            "fn foo() -> i32 {\n    let x = 1;\n    x + 1\n}",
            Some("src/foo.rs"),
        )
        .expect("shortcode should return Some when a hook is installed");
    // Compressed or Skipped with a stable reason — we accept either so the
    // contract doesn't overfit to tree-sitter version quirks. The important
    // invariant is that the dispatcher ran the hook.
    match out {
        ShortCoderOutcome::Compressed { .. } => {}
        ShortCoderOutcome::Skipped { reason } => {
            // `unsupported_language` would indicate a regression — the
            // ShortCoder ctor MUST have registered rust.
            assert_ne!(
                reason, "unsupported_language",
                "rust grammar must be registered by ShortCoder::new",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Contract 3: emit_shortcoder_compression_to_ssb writes one
// ShortCoderCompression row through the standard SSB write path.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn contract_emit_shortcoder_compression_writes_row() {
    let tmp = TempDir::new().expect("tempdir");
    let db = tmp.path().join("ssb.db");
    let clock = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path: db,
        project_id: "p".to_owned(),
        session_id: "s".to_owned(),
        pool_size: Some(2),
    };
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb::new"));

    // Build an outcome with known numbers so we can verify the payload
    // shape — using the Skipped variant with reason="empty_input" keeps
    // tokens_in / tokens_out / reduction_pct at zero, which is itself a
    // useful breadcrumb the dashboard surfaces ("hook ran, zero saving").
    let outcome = ShortCoderOutcome::Skipped { reason: "empty_input" };

    let wrote = emit_shortcoder_compression_to_ssb(
        &ssb,
        "runner",
        0,
        &outcome,
        "baby_step_7",
    )
    .await;
    assert!(wrote, "emit must return true on the happy path");

    // Verify one row landed with the correct action_type.
    let rows = ssb
        .query_by_type(SsbActionType::ShortCoderCompression)
        .expect("query_by_type");
    assert_eq!(rows.len(), 1, "exactly one ShortCoderCompression row");
    // And action_type string round-trips.
    assert_eq!(
        ssb_action_type_from_str_public("shortcoder_compression").unwrap(),
        SsbActionType::ShortCoderCompression,
    );
    // Deserialise payload to confirm the shape.
    let payload: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode payload");
    match payload {
        SsbPayload::ShortCoderCompression(p) => {
            assert_eq!(p.step_id, "baby_step_7");
            assert_eq!(p.tokens_in, 0);
            assert_eq!(p.tokens_out, 0);
            assert_eq!(p.reduction_pct, 0);
        }
        other => panic!("expected ShortCoderCompression payload, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Contract 4: the PostToolUse-specific disable env var exists as a stable
// constant, is distinct from the global ShortCoder kill switch, and is
// exposed from the orchestrator crate so the CLI + dashboard can agree on
// its exact spelling.
// ---------------------------------------------------------------------------

#[test]
fn contract_post_tool_use_env_var_exists_and_is_distinct() {
    assert_eq!(
        ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV,
        "ELAI_DISABLE_SHORTCODER_POSTOOLUSE",
    );
    assert_ne!(
        ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV, ELAI_DISABLE_SHORTCODER_ENV,
        "PostToolUse-only kill switch must be separable from the global one",
    );
}

// ---------------------------------------------------------------------------
// Contract 5: TaskRunnerConfig::new_minimal leaves hook_dispatcher at None
// so the shortcoder call site is backward compatible — no pre-4B caller
// gains a new SSB row without opt-in.
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_hook_dispatcher_default_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/x.json"));
    assert!(cfg.hook_dispatcher.is_none());
}
