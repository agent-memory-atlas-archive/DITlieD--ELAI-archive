//! SP-3B -- live e2e smoke for the bmapi/gpt-5.5 provider (ProviderKind::Gpt55Proxy,
//! shipped SP-3A). AUDITOR-authored (wiring-enforcement.md: "the smoke harness is
//! an AUDITOR-authored artifact, deliberately NOT written by the SP-3A executor
//! whose provider code it exercises").
//!
//! WHAT THIS DRIVES (the REAL production entrypoint, not a unit test calling the
//! provider fn directly): `elai_tools::execute_tool("Agent", ..)` is the single
//! `pub fn` a real tool_use dispatch routes through by name (see
//! `crates/tools/src/lib.rs:1488` `execute_tool` / `:1688` `"Agent" => .. run_agent`).
//! From there the SAME production chain the SP-3A context package named as wired
//! fires: `execute_agent -> build_agent_runtime -> ProviderRuntimeClient::new(model)
//! -> ProviderClient::from_model(model) -> detect_provider_kind("gpt-5.5") ->
//! ProviderKind::Gpt55Proxy -> a REAL HTTP round trip against bmapi` (construction
//! itself live-verifies the model against bmapi's `/v1/models`, per
//! `OpenAiCompatClient::verify_model_available`).
//!
//! OFF-AGENT ARTIFACT (R1/R2/R4): this smoke does not author its own evidence.
//! `execute_agent_with_spawn` already (pre-existing, I-47, untouched by this file)
//! emits a `PromptHardeningApplied(RulesInjected)` SSB row for any WorkerLike
//! dispatch role, carrying the real `agent_id`. Selecting `subagent_type:
//! "general-purpose"` (WorkerLike per `prompt_hardening::classify_dispatch_role`)
//! guarantees that row fires on this exact dispatch. The smoke installs a real,
//! disk-persisted `Ssb` (not a mock emitter) via the same `install_prompt_hardening_ssb`
//! hook the CLI bootstrap uses (`crates/cli/src/live_cli.rs:817`), then queries that
//! same db afterward for the row keyed by this run's `agent_id` -- a db-not-found or
//! zero-row result is FAIL, never SKIP.
//!
//! PARAMETERIZED PROVIDER (TEIKOKU-ELAI-FINISH-PLAN.md 3.2: "the gate outlives the
//! credits"): the model under test is `ELAI_PROVIDER_LIVE_SMOKE_MODEL` (default
//! "gpt-5.5"). After the bmapi credits expire (2026-07-15) the SAME file re-runs
//! unchanged against e.g. `claude-sonnet-4-6` via
//! `ELAI_PROVIDER_LIVE_SMOKE_MODEL=claude-sonnet-4-6` -- only the env var changes.
//!
//! GATING: `ELAI_PROVIDER_LIVE_SMOKE=1` required (mirrors the established
//! `ELAI_OAUTH_LIVE_SMOKE` convention in
//! `crates/providers/tests/anthropic_oauth_live_smoke.rs`). `#[ignore]`'d so a
//! default `cargo test` never spends credits or touches the network.
//!
//! RUN (PowerShell):
//!   $env:ELAI_PROVIDER_LIVE_SMOKE="1"
//!   $env:BMAPI_API_KEY="<key>"        # never commit; sourced per-run only
//!   cargo test -p elai-tools --test gpt55_proxy_live_smoke -- --ignored --nocapture --test-threads=1
//!
//! See smoke-manifest.toml `[[feature]] plan = "SP-3B"` for the gate-owned
//! entrypoint/expect_ssb/flag_default declaration this file's evidence satisfies.

use std::sync::Arc;
use std::time::{Duration, Instant};

use elai_core::clock::ClockProvider;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig};
use serde_json::json;

/// Build a fresh, real (disk-backed, not mocked) SSB instance for this smoke run.
/// Mirrors the temp-dir pattern in `crates/cli/src/role_ab_probe_cmd.rs::make_temp_ssb`
/// (this codebase's own precedent for a CLI-probe-style smoke that writes real SSB
/// rows as its off-agent evidence). MUST be called while a tokio runtime context is
/// entered (`Ssb::new` calls `tokio::spawn` for its single-writer task internally).
fn make_smoke_ssb(session_id: &str) -> (std::path::PathBuf, Ssb) {
    let ts = std::time::SystemTime::now() // entropy-allow: temp-dir-disambiguation-subsec-nanos
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("gpt55_proxy_live_smoke_{pid}_{ts}"));
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("could not create smoke ssb dir {}: {e}", dir.display()));
    let cfg = SsbConfig {
        db_path: dir.join("ssb.db"),
        pool_size: Some(2),
        project_id: "gpt55-proxy-live-smoke".into(),
        session_id: session_id.into(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
    let ssb = Ssb::new(cfg, clock)
        .unwrap_or_else(|e| panic!("Ssb::new for smoke db at {}: {e}", dir.display()));
    (dir, ssb)
}

#[test]
#[ignore = "live network call to the bmapi gpt-5.5 proxy (or the configured provider) -- consumes real credits/quota"]
fn gpt55_proxy_provider_live_e2e_smoke() {
    if std::env::var("ELAI_PROVIDER_LIVE_SMOKE").ok().as_deref() != Some("1") {
        eprintln!(
            "[skip] ELAI_PROVIDER_LIVE_SMOKE not set to '1'; this smoke is opt-in \
             to avoid spending provider credits on every cargo run"
        );
        return;
    }

    let model =
        std::env::var("ELAI_PROVIDER_LIVE_SMOKE_MODEL").unwrap_or_else(|_| "gpt-5.5".to_string());
    eprintln!("[setup] provider-under-test model={model}");

    let session_id = format!("s-gpt55-proxy-live-smoke-{}", std::process::id());

    // Ssb::new spawns its single-writer task via tokio::spawn, which requires an
    // entered runtime context. `Runtime::new()` builds a multi-thread runtime whose
    // worker threads keep driving spawned tasks (the writer loop, and later the
    // fire-and-forget PromptHardeningApplied write) even after the `.enter()` guard
    // is dropped, as long as `rt` itself stays alive for the rest of the test.
    let rt = tokio::runtime::Runtime::new().expect("build tokio runtime for smoke ssb + dispatch");
    let (_ssb_dir, ssb) = {
        let _guard = rt.enter();
        make_smoke_ssb(&session_id)
    };
    let ssb = Arc::new(ssb);

    // Real Agent-tool input. subagent_type "general-purpose" resolves to
    // DispatchRole::WorkerLike (prompt_hardening::classify_dispatch_role), which
    // unconditionally emits PromptHardeningApplied(RulesInjected) on this dispatch --
    // the off-agent artifact asserted below. The prompt forces a real bash tool
    // round-trip so the run is a genuine tool-loop, not a bare "say ok" turn.
    let input = json!({
        "description": "gpt55-proxy-live-smoke probe",
        "prompt": "Use your bash tool to run exactly this command: echo gpt55-proxy-smoke-ok \
                   -- then reply with one sentence summarizing the command's output.",
        "subagent_type": "general-purpose",
        "model": model,
    });

    eprintln!("[step 1] dispatching real Agent-tool call via elai_tools::execute_tool(\"Agent\", ..)...");
    let raw = {
        // `execute_agent_with_spawn`'s emit_prompt_hardening_row call is synchronous
        // and gates on `tokio::runtime::Handle::try_current()` succeeding -- keep the
        // runtime entered for this call so the row is not silently dropped (see
        // prompt_hardening.rs emit_prompt_hardening_row: no active runtime = silent
        // drop, by design, for CLI paths that never have one; this smoke always does).
        let _guard = rt.enter();
        // install_prompt_hardening_ssb must also be called while entered so any
        // internal state that assumes an active handle is consistent with the
        // dispatch call immediately below (idempotent; safe to call once per run).
        elai_tools::install_prompt_hardening_ssb(Arc::clone(&ssb));
        elai_tools::execute_tool("Agent", &input).unwrap_or_else(|e| {
            panic!(
                "execute_tool(\"Agent\", ..) failed at dispatch time for model={model} -- if \
                 this is a missing_credentials error, BMAPI_API_KEY (or the target \
                 provider's key) is not set in this process's env: {e}"
            )
        })
    };
    let manifest: serde_json::Value = serde_json::from_str(&raw).expect("manifest JSON parses");
    let agent_id = manifest["agentId"]
        .as_str()
        .expect("agentId present in dispatch response")
        .to_string();
    let manifest_file = manifest["manifestFile"]
        .as_str()
        .expect("manifestFile present in dispatch response")
        .to_string();
    let output_file = manifest["outputFile"]
        .as_str()
        .expect("outputFile present in dispatch response")
        .to_string();
    eprintln!("[step 1] OK agent_id={agent_id} manifest_file={manifest_file}");

    eprintln!("[step 2] polling manifest_file for a terminal status (timeout 180s)...");
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut last_status;
    loop {
        let text = std::fs::read_to_string(&manifest_file)
            .unwrap_or_else(|e| panic!("read manifest_file {manifest_file}: {e}"));
        let v: serde_json::Value = serde_json::from_str(&text).expect("manifest re-parses");
        last_status = v["status"].as_str().unwrap_or("running").to_string();
        if last_status != "running" {
            break;
        }
        if Instant::now() >= deadline {
            panic!(
                "sub-agent job for model={model} did not reach a terminal state within \
                 180s (still 'running') -- either the proxy/model is hung or slower than \
                 this smoke's timeout; inspect {output_file} manually"
            );
        }
        std::thread::sleep(Duration::from_millis(1500));
    }
    eprintln!("[step 2] terminal status={last_status}");
    assert_eq!(
        last_status, "completed",
        "sub-agent job against model={model} did not complete -- see {output_file} for the \
         recorded failure (missing_credentials / proxy 4xx-5xx / model unavailable, etc.)"
    );

    let transcript = std::fs::read_to_string(&output_file).unwrap_or_default();
    eprintln!(
        "[step 3] output_file transcript ({} bytes):\n{transcript}",
        transcript.len()
    );
    assert!(
        !transcript.trim().is_empty(),
        "terminal status was 'completed' but the transcript file is empty -- the real \
         HTTP round trip through ProviderKind::Gpt55Proxy produced no assistant content"
    );

    eprintln!("[step 4] querying the real ssb.db for the off-agent PromptHardeningApplied row...");
    let rows = ssb
        .query_session_stream_asc(&session_id)
        .expect("query_session_stream_asc against the real, disk-backed smoke ssb.db");
    assert!(
        !rows.is_empty(),
        "ssb.db returned zero rows for session {session_id} -- an empty store is FAIL, \
         never SKIP (R2); either install_prompt_hardening_ssb was not effective or the \
         dispatch never reached execute_agent_with_spawn"
    );
    let matched = rows.iter().any(|r| {
        r.agent_id == agent_id && matches!(r.action_type, SsbActionType::PromptHardeningApplied(_))
    });
    assert!(
        matched,
        "no PromptHardeningApplied row for agent_id={agent_id} in the real ssb.db \
         ({} rows present for this session) -- the production Agent-tool dispatch \
         chain did not fire the existing I-47 SSB beacon on this run",
        rows.len()
    );

    eprintln!(
        "[done] gpt55-proxy-live e2e smoke PROVEN: model={model} agent_id={agent_id} \
         terminal_status={last_status} ssb_row=prompt_hardening_applied"
    );
}
