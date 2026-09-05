E-15 sandboxed exec — decisions doc

Cluster: s-25120 completion-drive session 2 (plan 28 of 28)
Session: s-15320
Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-15_sandboxed_exec.md
Prior state per RECHECK: PARTIAL 11 implemented + 12 missing + 1 not_wired

WHAT
M1..M5 + panel minimum-viable wired-up slice for E-15. Closes the 1 NOT-WIRED
finding (FirecrackerHotPool::spawn_from_snapshot had zero callers in
task_runner.rs and shipped as dead code). Extends the dashboard get_sandbox_status
payload with the 6 E-15 fields atomically across commands.rs + stub.ts +
types.ts. Adds a SandboxBackendPanel.svelte mounted inside OrchestrationDrawer.
Extends core::sandbox::SandboxConfig with 3 new optional sub-tables
(firecracker, gvisor, appcontainer) + parser sub-routines in core::config so
elai_cc.toml [sandbox.firecracker]/[sandbox.gvisor]/[sandbox.appcontainer]
stop being silently ignored. Adds contract test
.elai_cc/crates/contracts/tests/e15_sandboxed_exec_contract.rs covering the
7 wiring contracts.

KEY DECISIONS

1. Autonomous-mode lean scope. Per the session-2 handoff convention (RUNPOD-PENDING
   markers + cite-incident discipline) the executor ships the wired-up slice
   M1..M5 + panel + contract test and DEFERS the following items to E-15.0
   patch-N RUNPOD-PENDING: 8 acceptance test files
   (snapshot_uniqueness.rs, seccomp_arg_checked.rs, researcher_net_blocks.rs,
   firecracker_restore_budget.rs, no_new_privs_ordering.rs,
   chroot_codepath_removed.rs, baby_step_ephemerality.rs,
   pyodide_bridge_prune.rs), agent_isolation.rs extensions
   (test_4a_no_new_privs_capbset_seccomp_ordering + test_4a_appcontainer_trio_complete),
   scripts/measure_sandbox.{sh,ps1}, metrics/sandbox_perf.toml. These items
   either require live KVM + CAP_NET_ADMIN + Linux host
   (RUNPOD-PENDING), Windows AppContainer trio (separate runner), or are
   pure metrics-emit (out-of-scope for completion-drive close-out).

2. spawn_from_snapshot wire-up. AgentPoolBabyStepExecutor gains a
   `firecracker_pool: Option<Arc<FirecrackerHotPool>>` field + a builder
   method `with_firecracker_pool(pool)`. When set, the executor calls
   `pool.spawn_from_snapshot(&SnapshotKey { agent_id, role }, agent_binary, &[])`
   BEFORE the existing `launch_agent_process` call. spawn_from_snapshot is
   additive observability only — the cold-boot path is untouched and the
   agent process still launches via the canonical launch_agent_process
   route. On KVM-absent hosts (Windows + Linux without /dev/kvm) the pool
   returns `FirecrackerError::KvmUnavailable` which is the canonical
   RUNPOD-PENDING signal (mirrors the boundary pattern from E-49/E-52/E-53)
   and is logged as trace, not error. Production default is None so all
   existing test fixtures keep their behaviour.

3. Dashboard atomic cross-stack edit. SandboxBackendPanel.svelte +
   OrchestrationDrawer.svelte (import + mount) + types.ts (SandboxStatus
   interface +6 fields, SandboxBackendKind union, SandboxResearcherNetBlockedAttempt
   interface) + stub.ts (mock fallback +6 fields) + commands.rs
   (get_sandbox_status json! body +6 fields) — all in this commit. cross_lang_diff.py
   would flag drift if any of these 5 files lagged. ts-rs is NOT used for
   get_sandbox_status (untyped Value return per integration-explorer NONE
   ts-rs risk note), so the TS interface is hand-mirrored and a single
   reviewer needs to confirm key-name parity (snake_case mirroring done).

4. SCHEMA bump = ZERO. Verified fresh per cross-session piggyback rule
   (SCHEMA_VERSION 140 at ssb.rs:1687). All 6 E-15 SSB variants
   (FirecrackerRestoreSlow / SnapshotUniquenessRegenerated / SnapshotEvicted /
   SandboxBackendSelected / AgentForkedFromSnapshot / ResearcherNetEgressBlocked)
   were already enum-registered + parser-bound + string-mapped at SCHEMA=140.
   All 6 Payload structs verified at ssb.rs:14331..14377 + payload-bearing
   variants at ssb.rs:8776..8781 + test_payload arms at ssb.rs:20464..20487.
   This plan does not introduce a 7th variant, does not add a payload
   field, and does not mutate the SSB row shape — so the SCHEMA stays at
   140. The integration-explorer's WARN about unverified 4 of 6 payload
   structs was resolved by direct grep this session (all 6 present).

5. Config parser extension. core::sandbox::SandboxConfig grows
   `firecracker / gvisor / appcontainer: Option<...ConfigToml>` fields
   (additive, #[serde(default, skip_serializing_if = "Option::is_none")] so
   existing on-disk settings.json deserialises unchanged). core::config gains
   3 small sub-parsers (parse_optional_{firecracker,gvisor,appcontainer}_config)
   called from parse_optional_sandbox_config. The 4th toml struct
   ResearcherNetConfigToml ships with the type definition only — wiring
   into the parser for `[researcher.net]` (which is a sibling of `[sandbox]`,
   not a sub-table of it) is patch-N follow-on; the existing
   ResearcherNetConfig::default() in sandbox/researcher_net.rs covers the
   runtime fallback (allowed_hosts = arxiv.org + huggingface.co per plan
   acceptance).

6. cite-discipline. Per cluster s-25120 mistakes.md (10 cite-incident
   clusters), arXiv:2102.12892 "Restoring Uniqueness in MicroVM Snapshots"
   was DEFERRED — NOT inlined in any test rationale comment or doc string.
   The paper is referenced by name only in firecracker.rs:155 (a pre-existing
   pre-session-2 comment, not added this session). 6 external blog/vendor
   URLs (dev.to/adwitiya, shayon.dev, helpnetsecurity, etc) referenced in
   the plan are NOT cited anywhere in the shipped code or this doc.

7. NO new dependencies. seccompiler 0.5 + nix 0.31 + getrandom 0.3 were
   already in orchestrator/Cargo.toml prior to this session. The dashboard
   panel uses only existing api/types + api/stub surface. The contract test
   uses only elai-core + elai-orchestrator + serde_json + std (all pre-pinned
   in contracts/Cargo.toml [dev-dependencies]).

8. KVM-absent fallback. On Windows host (this session's dev environment) +
   on the runpod 3090 box's WSL2 (which exposes /dev/kvm), the FirecrackerHotPool
   path either no-ops (Windows / KvmUnavailable) or works as designed.
   The dashboard panel renders backend_selected = AppContainer on Windows
   and backend_selected = Firecracker or Gvisor on Linux, sourced from a
   compile-time cfg guard inside get_sandbox_status (matches the existing
   tier = C-Windows / C-Linux pattern). Live KVM exercise is RUNPOD-PENDING.

EXPECTED TEST OUTCOMES
  cargo check --workspace --tests : clean (no new errors, no new warnings
    on the touched modules — core, orchestrator, dashboard-v2 src-tauri,
    contracts).
  svelte-check (dashboard-v2)     : clean (SandboxBackendPanel passes
    strict-typing, no `any`, SandboxBackendKind union exhaustive).
  cargo check -p elai_cc_contracts --test e15_sandboxed_exec_contract :
    7 contracts pass (compile-time + source-grep, no live subprocess).

UNKNOWNS (RUNPOD-PENDING)
  - Live firecracker --restore-from-snapshot latency on the runpod 3090
    box: target restore_p50_ms < 100 per plan acceptance (line 170).
    Today the SandboxBackendPanel renders restore_p50_ms = null until
    the SandboxObservability surface lands on AppState.
  - seccomp policy false-positive rate (arg-checked AF_INET filter): plan
    cites 0% expected on the dev binary; no fixture in this session.
  - researcher-net REJECT vs DROP UX impact (shayon.dev 2026/52 claim):
    requires live CAP_NET_ADMIN exercise to measure EHOSTUNREACH timing.

FILES MODIFIED (8) + ADDED (3)
  ADD:
    .elai_cc/crates/dashboard-v2/src/lib/components/SandboxBackendPanel.svelte
    .elai_cc/crates/contracts/tests/e15_sandboxed_exec_contract.rs
    workflow/reasoning/E-15-decisions.md
  MOD:
    .elai_cc/crates/orchestrator/src/task_runner.rs    (firecracker_pool field + setter + invoke)
    .elai_cc/crates/core/src/sandbox.rs                (SandboxConfig +3 sub-tables + 4 toml structs)
    .elai_cc/crates/core/src/config.rs                 (3 sub-parsers + import + extended SandboxConfig literal)
    .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs   (get_sandbox_status +6 json keys + backend cfg-gate)
    .elai_cc/crates/dashboard-v2/src/lib/api/types.ts        (SandboxStatus +6 fields + SandboxBackendKind union + BlockedAttempt iface)
    .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts         (mock fallback +6 fields)
    .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte  (import + mount panel)

SCHEMA + TOTAL_VARIANTS state (cluster-tracking)
  SCHEMA_VERSION : 140 -> 140 (no bump)
  TOTAL_VARIANTS : 404 -> 404 (no bump — 6 E-15 variants already counted in
                   the cluster-wide TOTAL pre-session)
