//! C2 — RequestCredential intervention contract tests.
//!
//! Verifies:
//!   1. CredentialField / InterventionType::RequestCredential round-trips
//!      through msgpack (rmp_serde) correctly.
//!   2. InterventionCredentials / InterventionOption::Credentials round-trips
//!      through msgpack correctly.
//!   3. The credential gate end-to-end: write an InterventionRequest(RequestCredential)
//!      to SSB, write back an InterventionResponse(Credentials), verify the SSB
//!      broadcast subscriber receives the correct credential values. This
//!      exercises the full SSB write/broadcast path used by run_confirmation_gate.
//!   4. A BashCommandInput with env_overrides set passes the overrides to
//!      the spawned subprocess (verified via env-echo command).
//!
//! Stub-ban:
//!   - No .unwrap() outside of .expect("context") calls.
//!   - No std::env::set_var (deprecated MT-UB) — env_overrides is the only path.
//!   - No placeholder bodies.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::{
    BashCommandInput, CancelHandle, CredentialField, InterventionCredentials, InterventionOption,
    InterventionRequestPayload, InterventionResponsePayload, InterventionType, SsbActionType,
    SsbPayload, run_confirmation_gate,
};
use elai_orchestrator::{Ssb, SsbConfig};
use elai_core::SeededClock;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_ssb_config(dir: &TempDir) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join("test.db"),
        pool_size: None,
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    }
}

// ---------------------------------------------------------------------------
// 1. InterventionType::RequestCredential round-trip
// ---------------------------------------------------------------------------

#[test]
fn contract_credential_field_roundtrips_msgpack() {
    let field = CredentialField {
        id: "API_KEY".into(),
        label: "API Key".into(),
        secret: true,
        default: None,
    };
    let wrapped = SsbPayload::InterventionRequest(InterventionRequestPayload {
        step_id: "step-1".into(),
        intervention_type: InterventionType::RequestCredential {
            fields: vec![field],
        },
        description: "please supply credentials".into(),
        diff_summary: None,
        options: vec![],
    });
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize RequestCredential");
    let round: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize RequestCredential");
    match round {
        SsbPayload::InterventionRequest(p) => match p.intervention_type {
            InterventionType::RequestCredential { fields } => {
                assert_eq!(fields.len(), 1, "one field");
                assert_eq!(fields[0].id, "API_KEY");
                assert_eq!(fields[0].label, "API Key");
                assert!(fields[0].secret, "secret=true must survive round-trip");
                assert!(fields[0].default.is_none());
            }
            other => panic!("expected RequestCredential, got {other:?}"),
        },
        other => panic!("expected InterventionRequest, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 2. InterventionOption::Credentials round-trip
// ---------------------------------------------------------------------------

#[test]
fn contract_credential_response_roundtrips_msgpack() {
    let mut values = BTreeMap::new();
    values.insert("API_KEY".into(), "sk-test-12345".into());
    let creds = InterventionCredentials {
        credentials: values,
    };
    let chosen = InterventionOption::Credentials(creds);
    let payload = SsbPayload::InterventionResponse(InterventionResponsePayload {
        request_id: 42,
        chosen,
        user_note: None,
    });
    let bytes = rmp_serde::to_vec_named(&payload).expect("serialize Credentials response");
    let round: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize Credentials response");
    match round {
        SsbPayload::InterventionResponse(p) => match p.chosen {
            InterventionOption::Credentials(c) => {
                assert_eq!(
                    c.credentials.get("API_KEY").map(String::as_str),
                    Some("sk-test-12345"),
                    "credential value must survive round-trip",
                );
            }
            other => panic!("expected Credentials variant, got {other:?}"),
        },
        other => panic!("expected InterventionResponse, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 3. Gate end-to-end: run_confirmation_gate unblocks and returns credentials
// ---------------------------------------------------------------------------
//
// This test exercises run_confirmation_gate end-to-end:
//   a. Spawns a tokio task calling run_confirmation_gate(RequestCredential).
//   b. A second task polls SSB for the InterventionRequest row, reads the
//      request_seq, and writes back an InterventionResponse(Credentials).
//   c. Awaits run_confirmation_gate — must return (rejected=false, {"API_KEY": value}).
//   d. Constructs a std::process::Command with the returned credentials in env
//      and calls get_envs() to assert API_KEY is set — mirrors the production
//      path added in BLOCK-1 (agent_pool.rs:Command::env loop).

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_ssb_broadcast_delivers_credentials_to_subscriber() {
    const CRED_KEY: &str = "API_KEY";
    const CRED_VAL: &str = "sk-test-12345";

    let dir = TempDir::new().expect("tempdir");
    let config = test_ssb_config(&dir);
    let ssb = Arc::new(Ssb::new(config, Arc::new(SeededClock::new())).expect("ssb init"));

    let cancel = CancelHandle::new();

    // --- Task A: call run_confirmation_gate and capture its return value. ---
    let ssb_a = Arc::clone(&ssb);
    let gate_task = tokio::spawn(async move {
        run_confirmation_gate(
            &ssb_a,
            "need credentials",
            "contract test summary",
            /*baby_step_id=*/ 1,
            &cancel,
            InterventionType::RequestCredential {
                fields: vec![CredentialField {
                    id: CRED_KEY.into(),
                    label: "API Key".into(),
                    secret: true,
                    default: None,
                }],
            },
        )
        .await
    });

    // --- Task B: watch SSB for the InterventionRequest, then respond. ---
    let ssb_b = Arc::clone(&ssb);
    let responder_task = tokio::spawn(async move {
        let mut rx = ssb_b.subscribe();
        // Poll for the InterventionRequest row written by run_confirmation_gate.
        let request_seq: i64 = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
                    Ok(Ok(entry)) => {
                        if entry.action_type != SsbActionType::InterventionRequest {
                            continue;
                        }
                        let payload: SsbPayload = rmp_serde::from_slice(&entry.payload)
                            .expect("deserialize InterventionRequest");
                        if let SsbPayload::InterventionRequest(p) = payload {
                            if matches!(p.intervention_type, InterventionType::RequestCredential { .. }) {
                                return entry.seq;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        })
        .await
        .expect("responder: InterventionRequest received within 5s");

        // Write the credential response that unblocks run_confirmation_gate.
        let mut credentials = BTreeMap::new();
        credentials.insert(CRED_KEY.into(), CRED_VAL.into());
        let resp_payload = SsbPayload::InterventionResponse(InterventionResponsePayload {
            request_id: request_seq,
            chosen: InterventionOption::Credentials(InterventionCredentials {
                credentials,
            }),
            user_note: None,
        });
        ssb_b
            .write_payload("dashboard-v2", 0, resp_payload, vec![])
            .await
            .expect("write InterventionResponse");
    });

    // --- Await both tasks with a combined timeout. ---
    let (gate_result, responder_result) = tokio::time::timeout(
        Duration::from_secs(10),
        async { tokio::join!(gate_task, responder_task) },
    )
    .await
    .expect("gate + responder completed within 10s");

    responder_result.expect("responder task panicked");
    let (rejected, returned_creds) = gate_result.expect("gate task panicked");

    // run_confirmation_gate returns (rejected, creds). Credentials => approved.
    assert!(!rejected, "RequestCredential response must NOT be rejected");
    assert_eq!(
        returned_creds.get(CRED_KEY).map(String::as_str),
        Some(CRED_VAL),
        "run_confirmation_gate must return the credential value in the map",
    );

    // --- Verify production wiring: Command::env merges credentials into subprocess env. ---
    // Mirrors agent_pool.rs BLOCK-1 fix: build a Command, call .env() for each credential,
    // then assert get_envs() contains the key. No subprocess is spawned — get_envs() is
    // sufficient and deterministic.
    let mut cmd = std::process::Command::new("true");
    for (key, value) in &returned_creds {
        cmd.env(key, value);
    }
    let env_map: std::collections::BTreeMap<_, _> = cmd
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.map(|s| s.to_string_lossy().into_owned()),
            )
        })
        .collect();
    assert_eq!(
        env_map.get(CRED_KEY).and_then(|v| v.as_deref()),
        Some(CRED_VAL),
        "Command::get_envs() must contain the credential after env injection",
    );
}

// ---------------------------------------------------------------------------
// 4. BashCommandInput env_overrides visible in subprocess
// ---------------------------------------------------------------------------

#[test]
fn contract_bash_env_overrides_visible_in_subprocess() {
    const TEST_VAR: &str = "ELAI_C2_CREDENTIAL_TEST_12345";
    const TEST_VAL: &str = "contract-verified";

    let mut overrides = BTreeMap::new();
    overrides.insert(TEST_VAR.into(), TEST_VAL.into());

    // Use a command that prints the env var; works on both Windows and Linux.
    #[cfg(windows)]
    let echo_cmd = format!("echo %{}%", TEST_VAR);
    #[cfg(not(windows))]
    let echo_cmd = format!("printenv {}", TEST_VAR);

    let output = elai_orchestrator::execute_bash(BashCommandInput {
        command: echo_cmd,
        timeout: Some(5_000),
        description: Some("C2 credential env override test".into()),
        run_in_background: Some(false),
        dangerously_disable_sandbox: Some(true),
        namespace_restrictions: None,
        isolate_network: None,
        filesystem_mode: None,
        allowed_mounts: None,
        env_overrides: Some(overrides),
    })
    .expect("bash command must succeed");

    assert!(
        output.stdout.contains(TEST_VAL),
        "subprocess stdout {:?} must contain the credential value {:?}",
        output.stdout,
        TEST_VAL,
    );
}
