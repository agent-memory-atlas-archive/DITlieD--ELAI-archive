//! Batch 3B (Phase 4G) — /mode + /model switch contract.
//!
//! Pins the parser + SSB emission semantics for the two slash commands
//! routed through the COMMAND window. The Tauri command plumbing itself is
//! exercised end-to-end at runtime (`dashboard/src-frontend/src/Workspace.svelte`
//! -> `commands::set_gate_mode`); here we prove the value types are stable
//! and the parser rejects garbage.

use elai_orchestrator::{GateMode, ModeChangedPayload, SsbActionType, SsbPayload};

// ---------------------------------------------------------------------------
// Contract 1 — /mode arg parser accepts both case variants and both modes.
// ---------------------------------------------------------------------------

#[test]
fn contract_mode_parser_accepts_interactive_and_autonomous() {
    assert_eq!(GateMode::from_str("interactive"), Some(GateMode::Interactive));
    assert_eq!(GateMode::from_str("Interactive"), Some(GateMode::Interactive));
    assert_eq!(GateMode::from_str("  AUTONOMOUS  "), Some(GateMode::Autonomous));
    assert_eq!(GateMode::from_str("autonomous"), Some(GateMode::Autonomous));
}

#[test]
fn contract_mode_parser_rejects_unknown() {
    assert!(GateMode::from_str("yolo").is_none());
    assert!(GateMode::from_str("").is_none());
    assert!(GateMode::from_str("auto").is_none());
}

// ---------------------------------------------------------------------------
// Contract 2 — ModeChanged payload round-trips through the SsbPayload enum
// and maps to SsbActionType::ModeChanged.
// ---------------------------------------------------------------------------

#[test]
fn contract_mode_changed_payload_action_type() {
    let p = SsbPayload::ModeChanged(ModeChangedPayload {
        from: "interactive".into(),
        to: "autonomous".into(),
    });
    assert_eq!(p.action_type(), SsbActionType::ModeChanged);
}

#[test]
fn contract_mode_changed_payload_msgpack_roundtrip() {
    let payload = ModeChangedPayload {
        from: "interactive".into(),
        to: "autonomous".into(),
    };
    let bytes = rmp_serde::to_vec_named(&SsbPayload::ModeChanged(payload.clone()))
        .expect("serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    match decoded {
        SsbPayload::ModeChanged(p) => {
            assert_eq!(p.from, payload.from);
            assert_eq!(p.to, payload.to);
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3 — set_gate_mode / set_model_routing accept the documented
// argument set. We exercise the parsing surface used by the Tauri handler
// (`mode.trim().to_ascii_lowercase()` + matches!) directly here so the
// parser rules are pinned by a test.
// ---------------------------------------------------------------------------

fn accepts_routing(raw: &str) -> bool {
    let lower = raw.trim().to_ascii_lowercase();
    matches!(lower.as_str(), "local" | "api" | "hybrid")
}

#[test]
fn contract_model_routing_accepts_local_api_hybrid() {
    assert!(accepts_routing("local"));
    assert!(accepts_routing("API"));
    assert!(accepts_routing("  Hybrid  "));
}

#[test]
fn contract_model_routing_rejects_unknown() {
    assert!(!accepts_routing("gpt"));
    assert!(!accepts_routing(""));
    assert!(!accepts_routing("self_hosted"));
}

// ---------------------------------------------------------------------------
// Contract 4 — GateMode Display matches the slash-command spelling.
// ---------------------------------------------------------------------------

#[test]
fn contract_gate_mode_display_matches_slash_syntax() {
    assert_eq!(GateMode::Interactive.to_string(), "interactive");
    assert_eq!(GateMode::Autonomous.to_string(), "autonomous");
}
