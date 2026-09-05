Phase C2 decisions — InterventionType::RequestCredential

WHAT: Add RequestCredential variant to InterventionType; wire credential form in dashboard; store credentials in per-session map; inject into next subprocess via Command::env.

KEY DECISIONS

D1 — Credential round-trip via existing SSB channel.
  Reuse SsbActionType::InterventionRequest / InterventionResponse wire.
  Add InterventionOption::Credentials(BTreeMap<String,String>) as a new untagged variant.
  No new SsbActionType needed; no SCHEMA_VERSION bump.
  Reason: minimal blast radius; existing run_confirmation_gate loop already polls InterventionResponse rows.

D2 — InterventionType loses Copy.
  Adding Vec<CredentialField> to RequestCredential variant requires removing #[derive(Copy)].
  Cargo check surfaces all copy-consumer sites; all existing uses are field assignments (by value), Clone is sufficient.

D3 — run_confirmation_gate signature extended.
  Old: async fn run_confirmation_gate(...) -> bool
  New: async fn run_confirmation_gate(..., credential_type: bool) -> (bool, BTreeMap<String,String>)
  Actually cleaner: accept InterventionType as parameter; return (rejected: bool, credentials: BTreeMap<String,String>).
  Callers: only one (task_runner.rs:4742). Credential map is empty for non-RequestCredential calls.

D4 — Per-session credential map on ResolvedConfig.
  Add session_credentials: Mutex<BTreeMap<String,String>> to ResolvedConfig.
  Updated by run_confirmation_gate result at the call site in drive_task.
  Persists across baby steps within a task run.

D5 — Env injection via BashCommandInput::env_overrides.
  Add env_overrides: Option<BTreeMap<String,String>> to BashCommandInput.
  Plumb into prepare_command and prepare_tokio_command via .envs(map).
  The credential store is copied into the BashCommandInput when the agent dispatcher or tools crate dispatches a bash command.
  PROBLEM: BashCommandInput is constructed by the agent (tools crate) and the credentials are in the orchestrator's ResolvedConfig.
  RESOLUTION: The credential map must be accessible at the call site. Since tools/src/lib.rs calls execute_bash directly from the agent process, and the session_credentials live in the orchestrator process, direct injection at the tools level is not viable without IPC changes.
  REVISED D5: Store credentials in ResolvedConfig; apply them ONLY at the orchestrator's own bash dispatch sites (not the agent's). This matches the brief's intent: "next ToolDispatch's subprocess Command env injection." The orchestrator does not itself call execute_bash for baby steps; it spawns agents. So credentials go into AgentEnvironment::new(scratch_dir) or via .envs() after .envs(env.as_env_map()) in agent_pool.rs.
  FINAL D5: Apply session credentials as additional envs in the agent spawn path (agent_pool.rs:369) via .envs(&credential_map) after .envs(env.as_env_map()). session_credentials must be passed to spawn_agent or made accessible on ResolvedConfig shared via Arc.

D6 — Field id sanitization.
  Sanitize to env var name: id.to_ascii_uppercase(), replace non-[A-Z0-9_] with _.
  Applied before both storage and Command::env injection.

D7 — Credential value masking in logs.
  tracing::info!(field_id = %id, value_len = value.len(), "credential captured") — value never logged.

EXPECTED TEST OUTCOME
  intervention_credential_contract: 4 tests, all PASS < 5s.
  e2e regressions: existing tests unaffected (Copy removal surfaces at compile time only).

UNKNOWNS
  Whether ssb_4k3_4k4_4k5_contract serialization test will break on InterventionOption::Credentials serde shape — check at cargo check time.

POST-AUDITOR FIXER UPDATE (2026-05-16)

D5 SUPERSEDED — actual wiring:
  BabyStepExecutionCtx gains session_credentials: BTreeMap<String,String> field.
  drive_task constructs the ctx with session_credentials = resolved.session_credentials.lock().unwrap().clone() at exec_ctx assembly time.
  launch_agent_process gains extra_env: BTreeMap<String,String> parameter.
  In the no-sandbox spawn path, Command::env(k,v) is called for each (k,v) in extra_env after the existing env.as_env_map() loop.
  Sandbox path (AgentSandbox::launch_agent trait) NOT wired — pre-existing trait limitation (the trait does not accept env). Document as known follow-up; not C2-introduced.
  fix_and_rerun_until_green path: passes session_credentials: BTreeMap::new() (no ResolvedConfig access on that code path; documented inline).

BLOCK-2 RESOLUTION: contract test rewritten to call run_confirmation_gate end-to-end.
  Spawns gate task + responder task. Responder polls SSB for InterventionRequest, writes Credentials response carrying API_KEY=sk-test-12345.
  Asserts run_confirmation_gate returns (rejected=false, BTreeMap{"API_KEY" → "sk-test-12345"}).
  Then constructs a std::process::Command, calls .env() with the returned map, and asserts Command::get_envs() contains the credential.

WARN-2 RESOLUTION: CredentialResponse struct DELETED entirely from ssb.rs + lib.rs + types.ts. Only InterventionCredentials remains on the wire.

KNOWN POST-C2 GAPS (deferred, NOT blockers):
  - RequestCredential variant has no production emitter today (only emitted by the contract test). A future phase wires an agent / scaffold / task_runner step to actually emit RequestCredential when an API key / OAuth token is needed.
  - Sandboxed agent launch path doesn't receive credentials via extra_env (AgentSandbox trait limitation). Non-sandbox path works.
  - Pre-existing: getrandom .expect() at task_runner.rs:7649; commands.rs:215 action_id parse fallback to 0.

AUDITOR (post-fixer): not re-run per protocol (one cycle only). The 2 BLOCKs are resolved by code review + test results (4/4 contract incl. rewritten run_confirmation_gate test).

