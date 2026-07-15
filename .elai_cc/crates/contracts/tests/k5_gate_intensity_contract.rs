// K-5 — deterministic GateIntensity selector contract test.
//
// Proves the production `DefaultDeterministicAuditGate` emits a
// `GateIntensitySelected` SSB row when evaluated on a real `_ -> Auditing`
// transition, and that the payload carries the observe-only tier selection
// plus the expected source/target FSM state names.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use elai_core::{
    ArchitecturePattern, BuildCommandRegistry, BuildSystem, BuildSystemProfile,
    Convention, EntryPoint, EntryPointKind,
    ErrorHandlingProfile, ErrorPattern, Language, LanguageProfile, ModuleOrganization,
    NamingConventions, OrgPattern, ProjectProfile, PropagationStyle,
    PROFILE_SCHEMA_VERSION, SeededClock,
};
use elai_core::project_profile::{DomainClassification, ProjectDomain};
use elai_orchestrator::{
    ssb::{Ssb, SsbConfig, SsbPayload},
    DefaultDeterministicAuditGate, Fsm, FsmState,
};
use tempfile::TempDir;

async fn drive_to_auditing(fsm: &Fsm) {
    fsm.transition(FsmState::Planning, "p", vec![]).await.unwrap();
    fsm.transition(FsmState::TestingFirst, "p", vec![])
        .await
        .unwrap();
    fsm.transition(FsmState::Executing, "p", vec![]).await.unwrap();
    fsm.transition(FsmState::Auditing, "p", vec![]).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn production_gate_emits_gate_intensity_selected_row() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("k5_smoke.db"),
                project_id: "k5-smoke".into(),
                session_id: "s-k5-smoke".into(),
                pool_size: None,
            },
            Arc::new(SeededClock::new()),
        )
        .expect("SSB init"),
    );

    let fixture_profile = ProjectProfile {
        schema_version: PROFILE_SCHEMA_VERSION,
        generated_at: "2026-06-15T00:00:00Z".to_owned(),
        project_root: PathBuf::from("/smoke-test"),
        build_system: BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        },
        languages: LanguageProfile {
            primary: Language::Rust,
            secondary: Vec::new(),
            distribution: BTreeMap::new(),
            tree_sitter_coverage: 0.0,
        },
        frameworks: Vec::new(),
        test_profile: Default::default(),
        lint_tools: Vec::new(),
        entry_points: vec![EntryPoint {
            file: PathBuf::from("src/main.rs"),
            kind: EntryPointKind::Server,
        }],
        ci: None,
        naming: NamingConventions {
            functions: Convention::Strong("snake_case".to_owned()),
            types: Convention::Strong("PascalCase".to_owned()),
            constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
            modules: Convention::Unknown,
        },
        error_handling: ErrorHandlingProfile {
            pattern: ErrorPattern::ResultBased,
            propagation: PropagationStyle::QuestionMark,
            error_crate: Some("thiserror".to_owned()),
        },
        module_organization: ModuleOrganization { pattern: OrgPattern::Layered },
        dep_profile: Default::default(),
        project_vision: None,
        architecture: Some(ArchitecturePattern::Layered),
        domains: vec![DomainClassification {
            domain: ProjectDomain::WebBackend,
            confidence: 0.9,
            signals: vec!["smoke-test fixture".to_owned()],
        }],
        commands: BuildCommandRegistry {
            build: Vec::new(),
            test: Vec::new(),
            lint: Vec::new(),
            format: Vec::new(),
            run: Vec::new(),
            clean: Vec::new(),
        },
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        overrides_applied: false,
    };
    let fixture_arc = Arc::new(fixture_profile);
    let profile_supplier: elai_orchestrator::deterministic_audit_gate::ProjectProfileSupplier =
        Arc::new(move || (*fixture_arc).clone());
    let changed_files_supplier: elai_orchestrator::deterministic_audit_gate::ChangedFilesSupplier =
        Arc::new(|| Vec::new());

    let gate = Arc::new(DefaultDeterministicAuditGate::new(
        Arc::clone(&ssb),
        profile_supplier,
        changed_files_supplier,
    ));

    let fsm = Arc::new(Fsm::new(None, 8192));
    fsm.set_deterministic_audit_gate(Arc::clone(&gate) as Arc<dyn elai_orchestrator::deterministic_audit_gate::DeterministicAuditGate>);

    drive_to_auditing(&fsm).await;

    // The gate emits on a detached spawn, so poll the SSB stream until the row lands.
    let ssb_for_loop = Arc::clone(&ssb);
    let rows = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        async move {
            loop {
                let rows = ssb_for_loop
                    .query_session_stream_asc("s-k5-smoke")
                    .expect("query session stream");
                let found = rows.iter().any(|e| {
                    matches!(e.action_type, elai_orchestrator::ssb::SsbActionType::GateIntensitySelected)
                });
                if found {
                    return rows;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        },
    )
    .await
    .expect("GateIntensitySelected row did not land within 2s");

    let intensity_rows: Vec<_> = rows
        .iter()
        .filter(|e| matches!(e.action_type, elai_orchestrator::ssb::SsbActionType::GateIntensitySelected))
        .collect();

    assert!(
        !intensity_rows.is_empty(),
        "at least one GateIntensitySelected row must be emitted after _ -> Auditing"
    );

    let payload = &intensity_rows[0].payload;
    let decoded: SsbPayload = rmp_serde::from_slice(payload)
        .expect("payload must deserialize as SsbPayload");

    match decoded {
        SsbPayload::GateIntensitySelected(p) => {
            assert_eq!(p.source_state, "EXECUTING", "source state is the pre-Auditing state");
            assert_eq!(p.target_state, "AUDITING", "target state is Auditing");
            assert_eq!(p.policy_version, "K-5.0", "policy version matches selector const");
            // WebBackend + Layered architecture drives risk_level High,
            // and all other degrade-safe signals default to Core/Unknown.
            assert_eq!(p.tier, "core", "high-risk fixture degrades to Core");
            assert!(
                p.selected.contains(&"Mutation".to_string()),
                "Core tier must include Mutation"
            );
            assert!(
                p.selected.contains(&"Metamorphic".to_string()),
                "Core tier must include Metamorphic"
            );
        }
        other => panic!("expected GateIntensitySelected payload, got {other:?}"),
    }
}
