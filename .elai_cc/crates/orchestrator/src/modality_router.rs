//! J-55 W1.1 — deterministic action-modality router (SSB SCHEMA_VERSION 311).
//!
//! TD-3 keystone of J-55: the LLM never picks its own action modality.
//! [`resolve`] is a total, pure function over the model record
//! ([`ModelCapabilityProfile`]), the recorded agent role, and the E-31
//! cloud-vs-local [`Tier`] signal. No I/O, no environment reads (the
//! kill-switch consult lives beside the call sites), no heuristics beyond
//! named directional constants and the measured crossover record.
//!
//! Resolution order is contractual (plan: J-55_model_strength_modality_router.md
//! UNIT W1.1):
//!   1. `tier == Cloud` → CodeAct (the E-31 hosted surface serves tool calls).
//!   2. local && parameters ≥ viability floor (measured crossover record when
//!      present, else the directional constant) → CodeAct.
//!   3. local && no native tool-call capability on the record → TextFallback.
//!   4. otherwise → ConstrainedToolCall (single constrained call under
//!      grammar + compiled minimal schema; the I-36 leg supplies schemas).
//!
//! Production wiring (RT-2 census): exactly TWO call sites exist, both wired
//! in this changeset — `task_runner.rs` `drive_task` turn-prep beside
//! `build_dispatch_context` (:6742) and `cli/runtime_client.rs`
//! `RuntimeClient::stream` per-turn prep (:296–:308).

use elai_ipc_types::AgentRole;

use crate::context_budget::ModelCapabilityProfile;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// J-55 W1.1: the action surface a turn presents. Deterministic verdict of
/// [`resolve`]; consumed by the executor/emission-format dispatch path at the
/// two wired call sites (branching only through existing surfaces).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionModality {
    /// CodeAct emission (E-35): tools imported as functions inside generated
    /// code; collapses the tool-schema and action-envelope token sinks.
    CodeAct,
    /// One constrained tool-call under grammar plus a compiled minimal schema
    /// (I-36). Keystone arm: the canonical Qwen2.5-Coder-7B local lane
    /// resolves here, never to [`ActionModality::CodeAct`].
    ConstrainedToolCall,
    /// Text-only turn for lanes whose record carries no native tool-call
    /// capability (nothing to constrain — no schema is posed).
    TextFallback,
}

/// J-55 W1.1: cloud-vs-local routing signal (E-31 axis) reduced for TD-3 use.
///
/// Converted from `BackendConfig.mode` AT THE CALL SITES (`task_runner.rs`
/// drive_task turn-prep and `cli/runtime_client.rs` stream per-turn prep);
/// this module stays pure so either host shares it unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// A hosted provider front door serves the turn (E-31 Api mode).
    Cloud,
    /// The local sidecar serves the turn (E-31 Local mode, or the first
    /// attempt of a Hybrid lane — hybrid promotes later attempts to cloud
    /// only on escalation, which re-runs resolution at that attempt's own
    /// turn-prep).
    Local,
}

// ---------------------------------------------------------------------------
// Directional thresholds (D1 bench re-bases them)
// ---------------------------------------------------------------------------

/// Directional CodeAct viability floor in billions of parameters.
///
/// Provenance: plan-cited arXiv:2509.04508 (ProST) — multi-tool code-mode
/// viability appears near the ~14B class; zero-shot code-mode at 3-7B is
/// unproven (J-55 plan Context section, WebSearch-verified 2026-06-20).
/// DIRECTIONAL until UNIT D1's `elai bench modality` harness writes a
/// measured `ModalityCrossover` onto the profile record, which takes
/// precedence here whenever present.
const CODEACT_VIABILITY_FLOOR_B: f32 = 14.0;

// ---------------------------------------------------------------------------
// resolve — the load-bearing decision
// ---------------------------------------------------------------------------

/// Resolve the action modality for one turn (TD-3: deterministic over
/// record fields only).
///
/// Total function: every branch terminates in a variant, no unwrap/expect,
/// no I/O. `role` is a recorded signature input per the plan contract
/// (deterministic over record fields; no role-keyed folkloric adjustments).
#[must_use]
pub fn resolve(profile: &ModelCapabilityProfile, role: AgentRole, tier: Tier) -> ActionModality {
    // TD-3: routing keys on the record + tier exclusively; `role` rides along
    // as the recorded dispatch role until D1 needs it as an additive signal.
    let _ = role;

    // Measured crossover wins when seeded (D1); else the directional const.
    let floor_b = profile
        .modality_crossover
        .as_ref()
        .map(|mc| mc.codeact_viability_floor_b)
        .unwrap_or(CODEACT_VIABILITY_FLOOR_B);

    match tier {
        // Contractual ordering: cloud tier resolves CodeAct BEFORE any
        // parameter check (hosted surface always serves tool calls today).
        Tier::Cloud => ActionModality::CodeAct,
        Tier::Local if profile.model_size_b >= floor_b => ActionModality::CodeAct,
        Tier::Local if !profile.features.tool_calls_native => ActionModality::TextFallback,
        Tier::Local => ActionModality::ConstrainedToolCall,
    }
}

// ---------------------------------------------------------------------------
// Kill-switch (W-2 default-ON law)
// ---------------------------------------------------------------------------

/// J-55 W1.2 kill-switch env var. The router ships DEFAULT-ON (W-2 law: a
/// feature behind an opt-in flag is wire-dark by design); setting this var
/// to `1` / `true` (case-insensitive) restores legacy presentation semantics.
/// There is intentionally NO enable flag.
pub const ELAI_MODALITY_ROUTER_DISABLE_ENV: &str = "ELAI_MODALITY_ROUTER_DISABLE";

/// Read the kill-switch. Returns true (router DISABLED) only when the env
/// var is `1` / `true` (case-insensitive). Unset / any other value leaves
/// the router ENABLED — mirrors `plugins/src/mcp_bridge.rs` G-4
/// `mcp_repin_disabled` verbatim-style.
#[must_use]
pub fn modality_router_disabled() -> bool {
    match std::env::var(ELAI_MODALITY_ROUTER_DISABLE_ENV) {
        Ok(v) => {
            let lo = v.trim().to_ascii_lowercase();
            lo == "1" || lo == "true"
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Row-label helper shared by both emit seams
// ---------------------------------------------------------------------------

/// Stable snake_case wire label for an agent role, matching the SSB payload
/// string convention (payload `role` fields carry snake_case tags such as
/// `"executor"` — see `FindingPayload.agent_role`). Derived through the IPC
/// serde representation rather than hand-rolled per-variant strings.
#[must_use]
pub fn agent_role_tag(role: AgentRole) -> String {
    match serde_json::to_value(role) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(other) => other.to_string(),
        // Unit variants serialize infallibly in practice; kept total anyway
        // with the debug-derived lowercase form (no panic on prod paths).
        Err(_) => format!("{role:?}").to_ascii_lowercase(),
    }
}
