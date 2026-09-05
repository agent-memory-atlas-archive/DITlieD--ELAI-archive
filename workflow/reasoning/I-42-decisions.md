I-42 critique-only operating profile — decisions

What: a RunProfile { Orchestrate (default), CritiqueOnly } selected at dispatch
time. In CritiqueOnly the executor/fixer agent spawn is structurally refused so
no spawned agent touches disk; the operator writes the code, the I-41 lenses run
at design time, the shipped auditor + off-agent spine run at ship time. One
engine, one spine; the only difference is who writes and whether executor agents
spawn.

Key decisions (both forks resolved by the controller, authoritative):

FORK-1 = A (observable). The plan's stated invariant "zero executor-attributed
file_write SSB rows" has NO live producer (the explorer confirmed: no production
SsbPayload::FileWrite write site; absence is trivially true today). So the
contract does NOT assert absent FileWrite rows and does NOT invent a producer
(scope creep). The real observable is the SPAWN GATE FIRING structurally: with
--profile orchestrate the resolved baby_step_executor is installed (Some); with
--profile critique-only it is forced None at the install chokepoint. This is a
deterministic, off-agent (config-state) assertion, R7-clean (no test-file caller
satisfies it), and proves R2 (the gate is the differentiator).

FORK-2 = a (gate location). The gate sits at INSTALL TIME in
crates/orchestrator/src/runtime_setup.rs::apply_task_runner_wiring, immediately
before `cfg.baby_step_executor = resolved_executor;`. When the resolved
RunProfile refuses the executor spawn, resolved_executor is forced None. This
ONE gate covers BOTH executor AND fixer, because the fixer loop
(fix_and_rerun_until_green) derives from the SAME resolved baby_step_executor.
Fail-closed by construction: there is nothing to spawn. Rejected option b
(routing the refusal through the I-1 supervisor Discard path): it is conditional
on a hook_dispatcher+supervisor being installed and stamps RUNNER_AGENT_ID, so
it has a bypass path on the else-branch — not fail-closed.

No SSB / SCHEMA change. I-42 adds no action_type, no payload, no SQL CHECK, no
golden bump. The profile is a run fact; SCHEMA_VERSION_PUBLIC and the
action_type golden lock are UNTOUCHED. e45 (the sole exact lock) is unaffected.

New module, not an overload. crates/orchestrator/src/run_profile.rs holds a
Copy enum + Display + from_str + from_cli_args(Option<&str>) mirroring the
shipped GateMode idiom (autonomous_gate.rs). RunProfile is ORTHOGONAL to GateMode
(autonomy of the gate vs whether executors spawn at all); GateMode is NOT
overloaded (stub_ban).

CLI flag, no env / no allowlist row. --profile orchestrate|critique-only is a
CLI flag with default Orchestrate, threaded args.rs -> CliAction::Run ->
main.rs RunArgs -> run_subcommand.rs (wiring.run_profile = Some(
RunProfile::from_cli_args(...))). It is NOT a default-OFF feature flag
(ELAI_*_ENABLE / cargo feature), so wiring-enforcement R9 / wire-dark-allowlist
does not apply.

Critique path. Under CritiqueOnly the spawn suppression is the Tier-1
deliverable. The I-41 LensSet design-time critique (LensSet::from_risk + async
run, gated ELAI_CRITIQUE_LENSES_ENABLE) and the shipped ship-time spine are
reused unchanged — the plan scopes the spawn-gate as the structural change and
leaves the lens-run on I-41's existing dispatch path; the minimal-correct change
ships the gate that prevents disk-mutating spawns. The lens-run wiring is not
re-implemented here (it composes I-41 as-shipped).

Frontend badge. The WindowFooterMode / StatusBar RunProfile badge is OUT of
scope for this changeset (Stage-5 / Tier-2 deferred per the plan's REVISED
scope); Tier-1 ships the CLI flag + structural gate only.

Files: crates/orchestrator/src/run_profile.rs (new), lib.rs (mod + re-export),
runtime_setup.rs (RuntimeWiring.run_profile field + build default + install
gate), crates/cli/src/args.rs (--profile parse + Run variant field + construct),
crates/cli/src/main.rs (destructure + RunArgs), crates/cli/src/run_subcommand.rs
(RunArgs field + wiring.run_profile assign), crates/contracts/tests/
i42_run_profile_contract.rs (new), plus run_profile field added to two existing
RuntimeWiring literals in agent_pool_executor_wiring_contract.rs and
c9_autonomous_infra_contract.rs (mechanical field addition, None = legacy posture).
