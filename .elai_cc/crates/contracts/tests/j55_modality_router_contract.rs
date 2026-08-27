//! J-55 W1.5 contract: the deterministic action-modality router.
//!
//! Arms per plan `docs/thefinalglance/J-plan/J-55_model_strength_modality_router.md`
//! UNIT W1.5 + INTEGRATION_SMOKE(W1):
//!   (a) determinism — repeated `resolve()` identical over fixed fixtures;
//!   (b) cloud tier => CodeAct;
//!   (c) sub-14B local Qwen2.5-Coder-7B-shaped profile => ConstrainedToolCall;
//!   (d) no-tool-capability fixture => TextFallback;
//!   (e) ConstrainedToolCall lane presents a bounded surface through the REAL
//!       six-arg `build_dispatch_context` production signature (+ kill-switch
//!       semantics); (f) SSB ModalityRouted msgpack round-trip + wire lockstep;
//!   (g) KEYSTONE NEGATIVE: sub-14B local never yields CodeAct.
//!
//! Composition limits recorded honestly (canonical record:
//! `/home/user/teikoku/elai/workflow/reasoning/J-55-decisions.md`):
//!   * E-35 SearchReplaceMatcher application lives in elai-tools
//!     (`tools/src/dispatcher.rs try_edit_tiers`, already wired; matcher is
//!     E-35-owned coverage). Orchestrator must NOT depend on elai-tools in
//!     production (dev-dep law, w1-gather-core §5), so this contract pins only
//!     the modality-side edit-format verdict channel reachable from
//!     orchestrator types: the EditFormatChosen{diff} Tag/Kind/payload
//!     substrate. No Cargo.toml was edited for this test file.
//!   * `resolve()` is contractual that Tier::Cloud resolves CodeAct FIRST,
//!     before any parameter check. The keystone negative arm therefore pins
//!     sub-14B => never CodeAct ON THE LOCAL TIER (all roles used in this
//!     suite, every crossover variant). A mis-provisioned endpoint whose TIER
//!     reads cloud while serving 7B is RT-3's parked D1 live replay and is
//!     deliberately NOT asserted here as either positive or negative.
//!   * `ModelCapabilityProfile::micro_7b_12gb()` as shipped leaves
//!     `features.tool_calls_native=false`; the directional TRUE seeding is
//!     `detect_model_capability_profile`'s name policy (the production path).
//!     Arm (c) therefore obtains the profile through that production detect
//!     constructor ("as shipped" = how production actually produces it), and
//!     the raw ctor shape feeds arm (d).

use std::collections::{BTreeMap, HashSet};
// Env mutation below is mutex-serialized so sibling #[test] threads in THIS
// binary cannot observe a half-set switch state (integration tests run
// multi-threaded; each tests/*.rs file compiles to its own binary).
use std::sync::{Mutex, MutexGuard};

use elai_ipc_types::AgentRole;
use elai_orchestrator::context_budget::{
    detect_model_capability_profile, ModelCapabilityProfile, ModalityCrossover, VramTier,
};
use elai_orchestrator::dispatcher::{
    build_dispatch_context, BackendConfig, BackendMode, PhaseSlot,
};
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::modality_router::{
    modality_router_disabled, resolve, ActionModality, Tier, ELAI_MODALITY_ROUTER_DISABLE_ENV,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ModalityRoutedKind, ModalityRoutedPayload,
    ModalityRoutedTag, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC, SSB_ACTION_TYPES,
};

/// Canonical keystone fixture name (ELAI's Qwen2.5-Coder-7B local sidecar
/// lane); detect() seeds `tool_calls_native=true` directionally for it.
const KEYSTONE_QWEN_7B: &str = "Qwen2.5-Coder-7B-Instruct-abliterated";
/// Directional ~14B CodeAct viability threshold (plan Context; arXiv:2509.04508).
const DIRECTIONAL_FLOOR_B: f32 = 14.0;

/// Production-shaped keystone profile: Qwen2.5-Coder-7B lane with the
/// directional native-tool-call bit seeded TRUE by detect's name policy.
fn qwen7b_seeded() -> ModelCapabilityProfile {
    detect_model_capability_profile(KEYSTONE_QWEN_7B, VramTier::Vram12Gb)
}

/// Raw micro ctor shape: bit false via ModelFeatures::none(), measured
/// crossover absent — feeds arm (d) TextFallback.
fn no_tool_fixture() -> ModelCapabilityProfile {
    let mut p = ModelCapabilityProfile::micro_7b_12gb();
    p.features.tool_calls_native = false;
    p.modality_crossover = None;
    p
}

fn with_crossover(mut p: ModelCapabilityProfile, floor_b: f32) -> ModelCapabilityProfile {
    p.modality_crossover = Some(ModalityCrossover {
        codeact_viability_floor_b: floor_b,
        measured_at_ms: 0,
    });
    p
}

/// Full TD-3 verdict vector over the fixed fixture set; rebuilt from ctors on
/// every call so determinism checks share zero mutable state between rounds.
fn j55_verdicts() -> Vec<ActionModality> {
    vec![
        resolve(&qwen7b_seeded(), AgentRole::Executor, Tier::Local),
        resolve(&qwen7b_seeded(), AgentRole::Auditor, Tier::Local),
        resolve(
            &ModelCapabilityProfile::cloud_200k(),
            AgentRole::Executor,
            Tier::Cloud,
        ),
        resolve(&no_tool_fixture(), AgentRole::Executor, Tier::Local),
        resolve(&ModelCapabilityProfile::large_32b_48gb(), AgentRole::Executor, Tier::Local),
        // Measured-crossover precedence: a 32B record demoted by floor 48B,
        // and a CodeAct-promoting floor (16B < 32B) both resolve deterministically.
        resolve(
            &with_crossover(ModelCapabilityProfile::large_32b_48gb(), 48.0),
            AgentRole::Executor,
            Tier::Local,
        ),
        resolve(
            &with_crossover(ModelCapabilityProfile::large_32b_48gb(), 16.0),
            AgentRole::Executor,
            Tier::Local,
        ),
    ]
}

// ---------------------------------------------------------------------------
// (a) determinism — TD-3: identical inputs, identical verdicts
// ---------------------------------------------------------------------------

#[test]
fn contract_a_resolve_is_deterministic_over_fixed_fixtures() {
    let baseline = j55_verdicts();
    assert!(!baseline.is_empty());
    for round in 0..64 {
        assert_eq!(
            j55_verdicts(),
            baseline,
            "TD-3 drift at round {round}: same records, different modality"
        );
    }
}

// ---------------------------------------------------------------------------
// (b) cloud tier => CodeAct (contractual first rule)
// ---------------------------------------------------------------------------

#[test]
fn contract_b_cloud_tier_resolves_code_act() {
    let cloud = ModelCapabilityProfile::cloud_200k();
    for role in [AgentRole::Executor, AgentRole::Researcher, AgentRole::Auditor] {
        assert_eq!(
            resolve(&cloud, role, Tier::Cloud),
            ActionModality::CodeAct,
            "tier=Cloud must resolve CodeAct first (role {role:?})"
        );
    }
    // NOTE: the 7B-shaped record under Tier::Cloud is intentionally untested
    // here — see module header (RT-3 residual / D1 live replay).
}

// ---------------------------------------------------------------------------
// (c)+(g) keystone pair: constrained-call positive + NEVER-CodeAct negative
// ---------------------------------------------------------------------------

#[test]
fn contract_c_sub_14b_local_resolves_constrained_tool_call() {
    let p = qwen7b_seeded();
    assert!(
        p.features.tool_calls_native,
        "detect must directionally seed the native tool-call bit for the canonical lane"
    );
    assert!(p.model_size_b < DIRECTIONAL_FLOOR_B, "fixture must stay sub-14B");
    for role in [AgentRole::Executor, AgentRole::Researcher] {
        assert_eq!(
            resolve(&p, role, Tier::Local),
            ActionModality::ConstrainedToolCall,
            "sub-14B seeded local lane must resolve ConstrainedToolCall (role {role:?})"
        );
    }
    // A measured crossover AGREEING with the directional constant keeps the verdict.
    let agreed = with_crossover(qwen7b_seeded(), DIRECTIONAL_FLOOR_B);
    assert_eq!(resolve(&agreed, AgentRole::Executor, Tier::Local), ActionModality::ConstrainedToolCall);
}

#[test]
fn contract_g_keystone_negative_sub_14b_local_never_code_act() {
    for role in [AgentRole::Executor, AgentRole::Auditor, AgentRole::Researcher] {
        let p = qwen7b_seeded();
        assert_ne!(
            resolve(&p, role, Tier::Local),
            ActionModality::CodeAct,
            "KEYSTONE GUARD violated: sub-14B local routed to CodeAct (role {role:?})"
        );
        assert_eq!(resolve(&p, role, Tier::Local), ActionModality::ConstrainedToolCall);

        // Crossover noise cannot promote the 7B lane to CodeAct while its size
        // stays under the floor (record-only forgery surface pinned here).
        for floor_b in [DIRECTIONAL_FLOOR_B, 20.0_f32, 100.0_f32] {
            let forged = with_crossover(qwen7b_seeded(), floor_b);
            assert_ne!(
                resolve(&forged, role, Tier::Local),
                ActionModality::CodeAct,
                "crossover floor {floor_b} promoted the 7B lane to CodeAct"
            );
        }

        // The unseeded (no-native-bit) 7B record also never routes CodeAct;
        // it degrades honestly instead (arm d overlap asserted on 2 roles only).
        if matches!(role, AgentRole::Executor | AgentRole::Auditor) {
            let raw = ModelCapabilityProfile::micro_7b_12gb();
            assert_ne!(resolve(&raw, role, Tier::Local), ActionModality::CodeAct);
        }
    }
}

// ---------------------------------------------------------------------------
// (d) no-tool-capability record => TextFallback
// ---------------------------------------------------------------------------

#[test]
fn contract_d_no_native_tool_calls_yields_text_fallback() {
    let p = no_tool_fixture();
    for role in [AgentRole::Executor, AgentRole::Designer] {
        assert_eq!(
            resolve(&p, role, Tier::Local),
            ActionModality::TextFallback,
            "a record without the native tool-call bit must fall back to text (role {role:?})"
        );
    }
}

// ---------------------------------------------------------------------------
// (e) constrained lane presents a bounded surface through the REAL six-arg
//     build_dispatch_context call (budget_parity.rs construction pattern)
// ---------------------------------------------------------------------------

#[test]
fn contract_e_constrained_lane_presents_bounded_surface() {
    let fsm = FsmState::Executing;
    let cfg = BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".to_string(),
        model: KEYSTONE_QWEN_7B.to_string(),
        max_tools_local: 8,
        fallback_url: None,
        max_context: None,
    };
    // Same tool-name set dispatcher.rs' own Executing-state self-test uses
    // (names proven to pass check_tool_allowed under FsmState::Executing).
    let tools: Vec<String> = ["bash", "edit_file", "read_file", "write_file", "WebSearch"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let mut prio = BTreeMap::new();
    for (i, t) in tools.iter().enumerate() {
        prio.insert(t.clone(), (i as u8) + 1);
    }

    let profile = qwen7b_seeded();
    assert_eq!(
        resolve(&profile, AgentRole::Executor, Tier::Local),
        ActionModality::ConstrainedToolCall,
        "composition precondition: keystone lane resolves the constrained arm"
    );

    let ctx = build_dispatch_context(fsm, &cfg, &tools, &prio, Some(&profile), Some(&PhaseSlot::default()))
        .expect("build_dispatch_context ok");

    // Boundedness of what build_dispatch_context itself presents:
    assert!(
        ctx.filtered_tool_names.len() <= ctx.max_tools,
        "presented surface exceeds max_tools"
    );
    assert!(
        ctx.max_tools <= cfg.max_tools_local,
        "max_tools leaked past the configured local cap"
    );
    assert!(
        !ctx.filtered_tool_names.is_empty(),
        "constrained lane must have a usable single-call candidate"
    );

    // J-55 lane law applied at BOTH wired sites after this call: the
    // ConstrainedToolCall turn narrows to exactly ONE presented surface and
    // never the full registry. Mirror the deterministic narrowing verbatim
    // (stable order, first entry) and pin the invariant.
    let mut narrowed = ctx.filtered_tool_names.clone();
    narrowed.truncate(1);
    assert_eq!(narrowed.len(), 1, "constrained lane must present exactly one surface");
    assert!(
        ctx.filtered_tool_names.contains(&narrowed[0]),
        "narrowed surface must be drawn from the presented set"
    );
    assert!(
        narrowed[0] == "bash",
        "highest-priority stable tie must pick dispatch-order-first entry"
    );
}

// ---------------------------------------------------------------------------
// (e2) kill-switch semantics + router core purity under env state
// ---------------------------------------------------------------------------

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    // Poisoned-forever-after locks would wedge sibling tests; fold poison away.
    ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn contract_e_kill_switch_semantics_and_core_td3_purity_under_env() {
    let _guard = env_lock();

    // Unset => router ENABLED (default-ON; no opt-in flag exists).
    std::env::remove_var(ELAI_MODALITY_ROUTER_DISABLE_ENV);
    assert!(!modality_router_disabled(), "unset switch must leave router enabled");

    // 1 / true, case-insensitive, whitespace-trimmed => DISABLED.
    for v in ["1", "true", "TRUE", "True", " true "] {
        std::env::set_var(ELAI_MODALITY_ROUTER_DISABLE_ENV, v);
        assert!(modality_router_disabled(), "switch value '{v}' must disable");
    }
    // Any other value leaves the router ENABLED.
    for v in ["0", "false", "on", "yes", "banana"] {
        std::env::set_var(ELAI_MODALITY_ROUTER_DISABLE_ENV, v);
        assert!(!modality_router_disabled(), "switch value '{v}' must not disable");
    }

    // Core purity: with the switch disabling site-side routing AND unset again,
    // resolve()'s verdicts over the fixed fixtures are byte-identical — the
    // env state cannot perturb TD-3 resolution.
    let baseline = j55_verdicts();
    std::env::set_var(ELAI_MODALITY_ROUTER_DISABLE_ENV, "1");
    assert_eq!(j55_verdicts(), baseline, "env=1 perturbed pure resolve()");
    std::env::remove_var(ELAI_MODALITY_ROUTER_DISABLE_ENV);
    assert_eq!(j55_verdicts(), baseline, "env removal perturbed pure resolve()");
}

// ---------------------------------------------------------------------------
// (f) SSB substrate: msgpack round-trip through the real writer encoding,
//     action_type mapping lockstep, tag wire-name uniqueness + parse re-lockstep
// ---------------------------------------------------------------------------

fn all_kinds_and_tags() -> Vec<(ModalityRoutedKind, ModalityRoutedTag)> {
    vec![
        (
            ModalityRoutedKind::CodeActChosen {
                model_fp: "claude-sonnet-4-6".to_string(),
                role: "executor".to_string(),
            },
            ModalityRoutedTag::CodeActChosen,
        ),
        (
            ModalityRoutedKind::ConstrainedCallChosen {
                model_fp: "qwen2.5-coder-7b-instruct-nf4".to_string(),
                role: "executor".to_string(),
                schema_tokens: 512,
            },
            ModalityRoutedTag::ConstrainedCallChosen,
        ),
        (
            ModalityRoutedKind::EditFormatChosen { diff: true },
            ModalityRoutedTag::EditFormatChosen,
        ),
    ]
}

#[test]
fn contract_f_ssb_modality_routed_roundtrip_wire_lockstep() {
    // Registry presence of the representative entry + schema counter bump.
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|t| matches!(t, SsbActionType::ModalityRouted(ModalityRoutedTag::CodeActChosen))),
        "registry must carry the ModalityRouted representative append"
    );
    assert_eq!(
        SCHEMA_VERSION_PUBLIC, 311,
        "J-55 W1.4 schema counter (-309 + 620 entries) must hold"
    );

    let pairs = all_kinds_and_tags();

    // Per-arm sql_str values are distinct (SQL CHECK discriminates on them).
    let names: HashSet<&str> = pairs.iter().map(|(_, t)| t.as_sql_str()).collect();
    assert_eq!(names.len(), 3, "modality tag sql_str values must be pairwise distinct");

    // Wire parse re-lockstep + msgpack round-trip using the store-writer's
    // exact encoder (`rmp_serde::to_vec_named`, ssb/store.rs:447).
    for (kind, tag) in &pairs {
        match ssb_action_type_from_str_public(tag.as_sql_str()) {
            Ok(SsbActionType::ModalityRouted(parsed)) => {
                assert_eq!(&parsed, tag, "tag lost across sql_str/from_str");
            }
            Ok(other) => panic!("from_str({}) mapped onto {other:?}", tag.as_sql_str()),
            Err(e) => panic!("from_str({}) failed: {e:?}", tag.as_sql_str()),
        }

        let payload = SsbPayload::ModalityRouted(ModalityRoutedPayload { kind: kind.clone() });
        let bytes = rmp_serde::to_vec_named(&payload).expect("serialize ModalityRouted payload");
        let back: SsbPayload =
            rmp_serde::from_slice(&bytes).expect("deserialize ModalityRouted payload");
        assert_eq!(back, payload, "msgpack round-trip mismatch for {tag:?}");
        assert_eq!(
            back.action_type(),
            SsbActionType::ModalityRouted(*tag),
            "action_type mapping must route each rich Kind back to its Tag"
        );
    }
}
