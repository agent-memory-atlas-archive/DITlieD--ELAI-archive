//! WAVE-A on-path smoke for the E-22 lazy-MCP-schema gate
//! (ELAI_LAZY_MCP_SCHEMA_ENABLE).
//!
//! Drives the REAL public gate predicate
//! `elai_plugins::mcp_bridge::lazy_mcp_schema_enabled()` (default-OFF pinned,
//! ON on "1"/"true"). The gate is read at bridge construction; the bridge
//! emits the LazySchemaSavings row in http_list_tools ONLY when this returns
//! true. The gate-read is the off-agent observable. Production live emission
//! is tracked by promotion_plan_id E-22.
//!
//! Gated on the `mcp` cargo feature (the mcp_bridge module is
//! `#[cfg(feature = "mcp")]`). A default `cargo test -p elai_cc_contracts`
//! compiles this file as empty; `--features mcp` exercises the asserts.
#![cfg(feature = "mcp")]

use std::sync::Mutex;

use elai_plugins::mcp_bridge::{lazy_mcp_schema_enabled, ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn lazy_mcp_schema_gate_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::remove_var(ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV);
    assert!(
        !lazy_mcp_schema_enabled(),
        "CONTRACT: {ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV} MUST default OFF (wire-dark per E-22)"
    );
}

#[test]
fn lazy_mcp_schema_gate_on_when_enabled() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV, "1");
    let on = lazy_mcp_schema_enabled();
    std::env::remove_var(ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV);
    assert!(
        on,
        "CONTRACT: gate=1 MUST enable lazy-MCP-schema telemetry emission"
    );
}

#[test]
fn lazy_mcp_schema_gate_accepts_true_word() {
    let _g = ENV_LOCK.lock().expect("env lock");
    std::env::set_var(ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV, "true");
    let on = lazy_mcp_schema_enabled();
    std::env::remove_var(ELAI_LAZY_MCP_SCHEMA_ENABLE_ENV);
    assert!(on, "CONTRACT: the word 'true' MUST be treated as enabled");
}
