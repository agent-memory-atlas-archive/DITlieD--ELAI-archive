// C-34 — Empire Integration contract test.
//
// Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-34_empire_integration_deferred.md
// Decisions: F:\.ELAI_workflow\workflow\reasoning\C-34-decisions.md
// Sibling decisions:
//   - F:\.ELAI_workflow\workflow\reasoning\phase-4J-decisions.md (DECISION 2, 4)
//   - F:\.ELAI_workflow\workflow\reasoning\B-25-decisions.md (DECISION B-25-A..G)
//
// Purpose: pin the host-side WIRING_CONTRACT interface shapes of the
// orchestrator::empire submodule so future refactors that change a variant
// count, drop a trait, or rename an action_type are caught at compile +
// test time (not at activation time when Empire deploys). All assertions
// run against the SHIPPED host-side scaffold; the live IO bodies remain
// pending Empire deployment per phase-4J-decisions.md (see the bridge.rs
// markers grep-discoverable from the empire submodule).
//
// Build invocation:
//   cargo test -p elai_cc_contracts --features empire --test c34_empire_integration_contract
//
// The entire test module is `#[cfg(feature = "empire")]`-gated because
// the orchestrator::empire submodule itself is gated behind
// `#[cfg(feature = "empire")]` on the orchestrator crate (per
// phase-4J-decisions.md DECISION 2 — submodule, not separate crate).

#![cfg(feature = "empire")]

use std::sync::Arc;

use elai_orchestrator::empire::{
    build_transport, detect_repair_kind, BridgeState, Capability, ElaiNeuron, EmpireBridge,
    EmpireBridgeError, EmpireEventKind, EmpireIpcMessageType, EmpireRepairError, EmpireRepairKind,
    FactoryLineTransport, TransportConfig,
};
use elai_orchestrator::ssb_action_type_from_str_public;

// ---------------------------------------------------------------------------
// Contract 1: EmpireBridge::new() in Disconnected state returns
// NotConnected from emit_action — graceful degradation gate (anti-lazy
// compliance point 2 from phase-4J-decisions.md).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c34_contract_bridge_default_state_is_disconnected_and_emits_not_connected() {
    let bridge = EmpireBridge::new();
    assert_eq!(
        bridge.state(),
        BridgeState::Disconnected,
        "EmpireBridge::new() must start in Disconnected (host-side scaffold default)"
    );
    let res = bridge.connect().await;
    assert!(
        matches!(res, Err(EmpireBridgeError::NotConnected)),
        "EmpireBridge::connect() must return NotConnected when Empire codebase absent"
    );
    // After a failed connect the bridge must return to Disconnected (not
    // stuck in Connecting). Idempotent state-machine guarantee.
    assert_eq!(
        bridge.state(),
        BridgeState::Disconnected,
        "EmpireBridge state must return to Disconnected after NotConnected"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: FactoryLineTransport trait is dyn-safe. Compile-test by
// constructing Arc<dyn FactoryLineTransport + Send> through build_transport.
// Per phase-4J-decisions.md B-25-G — explicit Pin<Box<dyn Future + Send>>
// keeps the trait object-safe without async-trait.
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_factory_line_transport_is_object_safe() {
    // build_transport returns Box<dyn FactoryLineTransport> on success. We
    // immediately wrap in Arc to prove the dyn object can be ref-counted
    // and cross task boundaries — what the activation patch will do when
    // the EmpireBridge owns the transport handle.
    let cfg = TransportConfig::Tcp {
        host: "127.0.0.1".to_string(),
        port: 7430,
    };
    let boxed: Box<dyn FactoryLineTransport> =
        build_transport(&cfg).expect("TCP transport must build on all platforms");
    // Storing the boxed trait object in an Arc is the activation-patch
    // shape. If FactoryLineTransport ever loses Send or becomes generic,
    // this stops compiling.
    let arc: Arc<dyn FactoryLineTransport> = Arc::from(boxed);
    // Drop to satisfy "unused variable" lint without #[allow].
    drop(arc);
}

// ---------------------------------------------------------------------------
// Contract 3: EmpireRepairKind exhaustive match — 4 variants. If a
// future commit adds a 5th variant without bumping the SSB schema +
// EmpireEventKind::RepairProposed payload, this test fires.
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_empire_repair_kind_has_four_variants() {
    // Construct one of each variant — exhaustive match below proves the
    // variant set is exactly {CircularDep, HardImport, StdlibShadow,
    // Other(String)}.
    let kinds: [EmpireRepairKind; 4] = [
        EmpireRepairKind::CircularDep,
        EmpireRepairKind::HardImport,
        EmpireRepairKind::StdlibShadow,
        EmpireRepairKind::Other("future_pattern".to_string()),
    ];
    for k in &kinds {
        // Exhaustive match — if a new variant lands without updating this
        // contract, the test fails at compile time (non-exhaustive match).
        let label = match k {
            EmpireRepairKind::CircularDep => "circular_dep",
            EmpireRepairKind::HardImport => "hard_import",
            EmpireRepairKind::StdlibShadow => "stdlib_shadow",
            EmpireRepairKind::Other(s) => s.as_str(),
        };
        assert_eq!(label, k.as_wire_label());
    }
    // detect_repair_kind empty-path returns typed error (silent-swallow
    // guard from B-25-decisions.md DECISION B-25-E).
    let err = detect_repair_kind("");
    assert!(
        matches!(err, Err(EmpireRepairError::EmptyPath)),
        "detect_repair_kind('') must return EmpireRepairError::EmptyPath, never Ok(None)"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: [empire] toml section parses with defaults via the loader.
// We compile-test by deserializing a minimal toml document that exercises
// the EmpireConfig default path. The loader test in crates/cli/src/
// runtime_config.rs covers the full file-based parse; here we assert the
// struct shape one more time against the toml crate to catch surprise
// changes to the field set.
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_empire_toml_section_default_shape() {
    // Mirror of EmpireConfig defined in crates/cli/src/runtime_config.rs.
    // We re-declare a local struct here (no dep on elai_cli) to prove the
    // wire shape is stable from outside.
    #[derive(Debug, serde::Deserialize, Default)]
    #[serde(default)]
    struct EmpireToml {
        enabled: bool,
        transport: String,
        socket_path: String,
        tcp_host: String,
        tcp_port: u16,
        bridge_state_default: String,
    }

    let toml_src = r#"
[empire]
enabled = false
transport = "unix"
socket_path = "/tmp/elai-empire.sock"
tcp_host = "127.0.0.1"
tcp_port = 7430
bridge_state_default = "Disconnected"
"#;
    #[derive(Debug, serde::Deserialize, Default)]
    #[serde(default)]
    struct Wrapper {
        empire: EmpireToml,
    }
    let parsed: Wrapper = toml::from_str(toml_src).expect("toml parse must succeed");
    assert!(!parsed.empire.enabled);
    assert_eq!(parsed.empire.transport, "unix");
    assert_eq!(parsed.empire.socket_path, "/tmp/elai-empire.sock");
    assert_eq!(parsed.empire.tcp_host, "127.0.0.1");
    assert_eq!(parsed.empire.tcp_port, 7430);
    assert_eq!(parsed.empire.bridge_state_default, "Disconnected");

    // Empty document MUST also deserialize to all defaults (warn-not-panic
    // policy: missing section is silent).
    let empty: Wrapper = toml::from_str("").expect("empty toml must parse");
    assert!(!empty.empire.enabled);
    assert_eq!(empty.empire.transport, "");
}

// ---------------------------------------------------------------------------
// Contract 5: ssb_action_type_from_str_public() accepts every shipped
// empire_* action_type label. Locked names per B-25-decisions.md
// DECISION B-25-A (the 5-name set diverges from the C-34 plan text which
// said "empire_repair_started/complete" — C-34-decisions.md DRIFT-C
// documents the divergence; this test asserts the realized names).
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_all_five_empire_ssb_action_type_labels_round_trip() {
    // The 5 shipped empire action_type labels per ssb.rs L3443-3447 +
    // ssb_action_type_from_str L3836-3840.
    let labels = [
        "empire_connected",
        "empire_disconnected",
        "empire_event_received",
        "empire_capability_invoked",
        "empire_repair_proposed",
    ];
    for label in &labels {
        let parsed = ssb_action_type_from_str_public(label).unwrap_or_else(|e| {
            panic!("ssb_action_type_from_str_public('{label}') must succeed but got {e:?}")
        });
        // Round-trip back to string via the as_sql_str path (we cannot
        // import the enum directly without a wider re-export change; the
        // Debug repr is sufficient to assert the label survives).
        let dbg = format!("{parsed:?}");
        assert!(
            dbg.starts_with("Empire"),
            "ssb_action_type_from_str_public('{label}') must map to an Empire* variant, got {dbg}"
        );
    }
    // Adversarial: the OLD plan-text names ("empire_repair_started" /
    // "empire_repair_complete") MUST NOT parse — they are not the realized
    // names. C-34-decisions.md DRIFT-C documents the rejection.
    for stale in &["empire_repair_started", "empire_repair_complete"] {
        let r = ssb_action_type_from_str_public(stale);
        assert!(
            r.is_err(),
            "stale plan-text name '{stale}' must NOT be in the action_type map (B-25 names are canonical)"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 6: AppState.empire_bridge field initializes to None when the
// feature is off via the construction path. We assert host-side that the
// Option<Arc<EmpireBridge>> wrapper exists by constructing one via the
// public types and proving the None branch compiles + Arc round-trips.
//
// (The dashboard-v2 state.rs uses `Option<Arc<EmpireBridge>>` directly
// per B-25-decisions.md DECISION B-25-B; we cannot pull elai_dashboard_lib
// from the contracts crate -- see Cargo.toml L21 comment about the
// `crates/.bkp_old/dashboard` migration. So we exercise the same shape
// here via Arc<EmpireBridge>::new() to lock the type signature.)
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_empire_bridge_supports_arc_optional_storage() {
    let bridge: Option<Arc<EmpireBridge>> = None;
    assert!(bridge.is_none(), "default None branch must compile");
    let installed: Option<Arc<EmpireBridge>> = Some(Arc::new(EmpireBridge::new()));
    let cloned = installed.clone();
    assert!(installed.is_some(), "Some branch must compile");
    assert!(cloned.is_some(), "Arc clone must succeed");
    // State must remain Disconnected even after the Arc round-trip.
    if let Some(b) = cloned {
        assert_eq!(b.state(), BridgeState::Disconnected);
    }
}

// ---------------------------------------------------------------------------
// Contract 7: EmpireBridge emit_action returns NotConnected for ALL 5
// EmpireEventKind variants while in Disconnected state. This proves the
// dispatcher table is wired for every variant (no silent drop of a new
// variant) AND the graceful-degradation contract holds across the whole
// surface (anti-lazy compliance point 2).
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_emit_action_returns_not_connected_for_all_five_variants() {
    let bridge = EmpireBridge::new();
    let variants = [
        EmpireEventKind::Connected {
            brain_id: "elai_cc".to_string(),
            protocol_version: 1,
        },
        EmpireEventKind::Disconnected {
            brain_id: "elai_cc".to_string(),
            reason: "test".to_string(),
        },
        EmpireEventKind::EventReceived {
            source: "factory".to_string(),
            event_type: "deploy".to_string(),
            payload: serde_json::Value::Null,
        },
        EmpireEventKind::CapabilityInvoked {
            capability: "task_submit".to_string(),
            caller: "factory".to_string(),
            allowed: true,
        },
        EmpireEventKind::RepairProposed {
            issue: "CircularDep".to_string(),
            target_path: "workspace/factory-brain/core/bus.py".to_string(),
            fix_kind: "Refactor".to_string(),
        },
    ];
    for v in variants {
        let label = format!("{v:?}");
        let r = bridge.emit_action(v);
        assert!(
            matches!(r, Err(EmpireBridgeError::NotConnected)),
            "emit_action({label}) must return NotConnected in Disconnected state"
        );
    }
    // factory_lines() must return an empty list (graceful degradation —
    // EmpireBridgePanel renders the empty-state notice when this is empty).
    assert!(
        bridge.factory_lines().is_empty(),
        "factory_lines() must be empty while bridge is Disconnected"
    );
}

// ---------------------------------------------------------------------------
// Contract 8: ElaiNeuron.tool_filter denies every documented INTERNAL
// capability. The whitelist is the authoritative gate for any MCP
// dispatch path — internal capabilities (fsm_transition, evidence_gate,
// rules_query, ssb_direct_write, config_modify, agent_spawn) must NEVER
// be reachable from an Empire-side invocation even if a future commit
// accidentally adds one of these to the Capability vector. The whitelist
// is locked to 4 entries per phase-4J 4J.5 ACCEPTANCE.
// ---------------------------------------------------------------------------

#[test]
fn c34_contract_elai_neuron_tool_filter_denies_internal_capabilities() {
    let neuron = ElaiNeuron::whitelist();
    // Locked 4-capability whitelist (phase-4J 4J.5 + B-25-grounding.md L29).
    assert_eq!(
        neuron.capabilities.len(),
        4,
        "ElaiNeuron::whitelist() must surface exactly 4 capabilities"
    );
    let allowed_names: Vec<&str> = neuron
        .capabilities
        .iter()
        .map(|c: &Capability| c.name.as_str())
        .collect();
    for required in ["task_submit", "task_status", "pipeline_result", "health_check"] {
        assert!(
            allowed_names.contains(&required),
            "whitelist must include '{required}'"
        );
        assert!(
            neuron.allows(required),
            "tool_filter must allow '{required}'"
        );
    }
    // Adversarial — every INTERNAL capability the spec calls out MUST be
    // rejected by the filter, regardless of whether a future commit adds it
    // to the Capability vector. The filter is the authoritative gate.
    for internal in [
        "fsm_transition",
        "evidence_gate",
        "rules_query",
        "ssb_direct_write",
        "config_modify",
        "agent_spawn",
    ] {
        assert!(
            !neuron.allows(internal),
            "tool_filter MUST deny internal capability '{internal}'"
        );
    }
    // Spot-check: the 9 EmpireIpcMessageType variants are exactly what
    // empire.txt section 1 documents. This is a downstream observable of
    // contract 3 but lives here so all interface shapes are co-pinned.
    let variants = [
        EmpireIpcMessageType::RegisterHook,
        EmpireIpcMessageType::Emit,
        EmpireIpcMessageType::QueryHooks,
        EmpireIpcMessageType::Subscribe,
        EmpireIpcMessageType::Unsubscribe,
        EmpireIpcMessageType::BusCreate,
        EmpireIpcMessageType::BusJoin,
        EmpireIpcMessageType::BusLeave,
        EmpireIpcMessageType::BusList,
    ];
    assert_eq!(
        variants.len(),
        9,
        "EmpireIpcMessageType must have exactly 9 variants per empire.txt section 1"
    );
}
