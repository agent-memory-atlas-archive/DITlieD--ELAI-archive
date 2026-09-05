# E-15: Sandboxed agent code execution — Firecracker snapshot-restore + arg-checked seccomp + per-baby-step ephemerality
Objective: harden ELAI's Tier C sandbox to 2026 production standards. Add a hot Firecracker snapshot-restore pool (~28ms restore vs ~1s cold), upgrade BLOCK-1 seccomp BPF from blanket socket() allow to argument-checked AF_INET/AF_INET6/SOCK_STREAM/SOCK_DGRAM, drop runc/chroot-only isolation (defeated by Nov-2025 CVE-2025-31133/52565/52881 bind-mount races), enforce NO_NEW_PRIVS+CAPBSET_DROP ordering before seccomp, switch researcher network egress to veth+iptables OUTPUT allowlist with CAP_NET_RAW dropped, and treat every baby-step as ephemeral (fork-from-snapshot, never reuse). Windows fallback ships the AppContainer + Job-object + LowIL trio exactly per Microsoft SandboxSecurityTools + OpenAI Codex 2026.
Estimated effort: 3 sessions (~12 h).
Pre-requisites: B-18 PLANNED (Tier C Sandbox Activation — clone3 + seccomp baseline + AppContainer Windows fallback), C-11 PLANNED (Sandbox Completeness — PR_CAPBSET_DROP loop + setrlimit + hidepid=2 + NSpid test).
Parallel-safe with: E-11, E-12, E-13, E-14. AUGMENTS B-18 + C-11; REPLACES the chroot-Phase-0-1 fallback path.
Source report: workflow/improvement-sweep-2026-05-19/reports/15-sandboxed-exec.md

## Context

The November 2025 runc CVE bundle (CVE-2025-31133, 52565, 52881) demonstrates that bind-mount race conditions defeat maskedPaths inside Linux containers even with capabilities dropped and seccomp on; the patched runc versions are 1.2.8 / 1.3.3 / 1.4.0-rc.3. CVE-2025-68668 (n8scape, Pyodide in n8n) shows that Pyodide is not a sandbox — the host bridge surface is the actual trust boundary, and reaching Node `require('child_process')` from inside WASM Python is post-auth RCE. EchoLeak / CVE-2025-32711 (Microsoft 365 Copilot, CVSS 9.3) and Pillar Security's Antigravity escape (late 2025) confirm prompt-injection + a sandbox compatibility hole = exfiltration. TeamPCP / LiteLLM PyPI compromise (Mar 2026) shows that a malicious pip install during a baby step can plant a credential stealer that survives across steps — fork-from-snapshot evicts it. Firecracker snapshot-restore is measured at ~28ms (dev.to/adwitiya, AgentSphere production) vs ~1s cold; 5MiB overhead per microVM. gVisor (runsc, Systrap) is 10-20% syscall overhead — acceptable as KVM-less fallback. Apple/container reaches sub-second VM-per-container on Apple silicon. Windows: AppContainer + Job-object + LowIL is 6 years Chromium-hardened, OpenAI Codex May 2026 ships the same trio per WinBuzzer. ELAI today (per .elai_cc/crates/orchestrator/src/sandbox.rs:611 launch_agent + sandbox.rs:1021-1048 seccomp filter creation + the existing test_4a_isolation_pid_namespace_real in tests/agent_isolation.rs) has a clone3 + seccomp baseline but: (a) BLOCK-1 seccomp uses blanket socket() allow (must be argument-checked), (b) NO_NEW_PRIVS + CAPBSET_DROP + seccomp ordering not explicitly asserted, (c) no Firecracker / gVisor microVM boundary — namespaces + capability drop alone is defeated by Nov-2025 runc CVEs, (d) per-baby-step ephemerality not enforced (process can survive across steps and persist malicious pip), (e) Pyodide host-bridge surface unaudited if added, (f) Windows AppContainer present at launch_agent but Job-object kill-on-close + active-process-limit + LowIL working-dir descriptor not asserted as MUST.

## Existing state (B/C/D coverage)

- AUGMENTS B-18 (PLANNED) Tier C Sandbox Activation — extends clone3 + seccomp baseline with Firecracker snapshot-restore pool and arg-checked socket().
- AUGMENTS C-11 (PLANNED) Sandbox Completeness — extends C-11's PR_CAPBSET_DROP + setrlimit + hidepid=2 sequence with explicit NO_NEW_PRIVS → CAPBSET_DROP → seccomp ordering assertion and tests.
- REPLACES the "chroot in Phase 0-1, Firecracker/gVisor in Phase 3+" plan with "Firecracker/gVisor for all bash execution from Phase 0". chroot adds no boundary namespaces don't already provide (report best-technique #4).
- REUSES B-37 (SHIPPED) TierCSpawn adapter pattern at .elai_cc/crates/providers/src/runtime/cli_common.rs:337 — Firecracker hot pool registers as a TierCSpawn implementation alongside existing backends.
- REUSES the existing sandbox_policy.rs Passthrough vs Sandbox dispatch (sandbox_policy.rs:84 + sandbox_policy.rs:340 program_allowlist surface).
- NET-NEW: Firecracker snapshot-restore hot pool, per-agent-id+role snapshots, snapshot-uniqueness mitigation (arXiv:2102.12892 TCP ISN / entropy collision), veth+iptables OUTPUT allowlist with researcher zero-caps, fork-from-snapshot per baby step.
- NET-NEW: Windows AppContainer hardening — Job-object kill-on-close + active-process-limit + LowIL descriptor on working dir as MUST, not optional.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/15-sandboxed-exec.md
- .elai_cc/crates/orchestrator/src/sandbox.rs lines 611-1239 (existing launch_agent + clone_cb + seccomp filter creation)
- .elai_cc/crates/orchestrator/src/sandbox_policy.rs lines 84-599 (existing policy + allowlist + Passthrough)
- .elai_cc/crates/orchestrator/tests/sandbox_wireup.rs lines 71-251 (existing test harness pattern)
- .elai_cc/crates/orchestrator/tests/agent_isolation.rs lines 203-532 (existing NSpid + AppContainer tests)
- .elai_cc/crates/contracts/tests/sandbox_policy_contract.rs lines 326-340 (existing dispatch tests)
- .elai_cc/crates/providers/src/runtime/cli_common.rs lines 336-343 (TierCSpawn adapter shape)
- .elai_cc/crates/core/src/config.rs lines 762-771 (parse_optional_sandbox_config)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs line 4962 (get_sandbox_status)
- workflow/papers/firecracker-interface.md (after paper-ingestor mode=source on firecracker-microvm.github.io + dev.to/adwitiya snapshot deep-dive)
- workflow/papers/gvisor-interface.md (after paper-ingestor mode=source on gvisor.dev/docs/architecture_guide/performance/)
- arXiv:2102.12892 Restoring Uniqueness in MicroVM Snapshots (via paper-ingestor)
- workflow/papers/sysdig-runc-cve-2025-31133.md (after paper-ingestor mode=source)
- workflow/papers/echoleak-cve-2025-32711.md (after paper-ingestor mode=source)

## Tasks

1. Add Firecracker hot snapshot-restore pool. New file `.elai_cc/crates/orchestrator/src/sandbox/firecracker.rs`. Struct `FirecrackerHotPool { snapshots: HashMap<(AgentId, AgentRole), SnapshotHandle>, restore_budget_ms: 200 }`. First spawn for each (agent_id, role) does cold clone3+namespaces boot (~125-1000ms per Firecracker official) then writes a CoW snapshot via `firecracker --create-snapshot`. Subsequent spawns for the same (agent_id, role) `--restore-from-snapshot` in ~28ms (dev.to/adwitiya measured). Snapshot lives at .elai_cc/sessions/{session_id}/snapshots/{agent_id}-{role}.snap. Restore-budget guard: if restore > 200ms, fall back to cold path and emit `firecracker_restore_slow` SSB. Snapshot uniqueness mitigation per arXiv:2102.12892: regenerate per-VM TCP ISN seed + /dev/random pool + hostname salt before exec; emit `snapshot_uniqueness_regenerated` SSB per restore.

2. Replace BLOCK-1 blanket socket() allow with argument-checked filter. Edit `.elai_cc/crates/orchestrator/src/sandbox.rs` seccomp filter creation block (around lines 1021-1048). In IPC_BASELINE profile: DENY socket() entirely. In NETWORK_EXTRA (researcher) profile: ALLOW socket() ONLY when `args[0] == AF_INET (2) || args[0] == AF_INET6 (10)` AND `args[1] == SOCK_STREAM (1) || args[1] == SOCK_DGRAM (2)`. DENY explicitly: AF_UNIX (1), AF_NETLINK (16), AF_PACKET (17), AF_BLUETOOTH (31). seccompiler BPF supports scalar arg checks; the kernel cannot deref pointers (no struct sockaddr* inspection — that goes through veth+iptables). Add comment block citing Docker default profile + kernel.org seccomp_filter docs.

3. Enforce NO_NEW_PRIVS → CAPBSET_DROP → seccomp ordering. Edit `.elai_cc/crates/orchestrator/src/sandbox.rs::clone_cb` (around line 1112). Explicit ordering: `prctl(PR_SET_NO_NEW_PRIVS, 1)` MUST execute before `for cap in 0..CAP_LAST_CAP { prctl(PR_CAPBSET_DROP, cap) }` MUST execute before `seccomp(SECCOMP_SET_MODE_FILTER, ...)`. Add `assert_ordering()` test that fails on inversion. Inverted ordering is a frequent bug per mpdesouza no_new_privs blog + man7.org/prctl grounding. Test path: `tests/agent_isolation.rs::test_4a_no_new_privs_capbset_seccomp_ordering` (new test extending the existing NSpid test pattern).

4. Drop chroot from Phase 0-1 fallback path. Edit any code path that adds chroot to the sandbox profile (sandbox.rs / sandbox_policy.rs). chroot is escapable with CAP_SYS_CHROOT or fd into parent root (per containers/bubblewrap docs); namespaces already give chroot-equivalent. Remove the chroot codepath entirely; emit `chroot_removed` migration note in the next memory-bank entry. Sandbox profiles now: Firecracker (if KVM available), gVisor/runsc (KVM-less fallback), AppContainer (Windows). No chroot lane.

5. Add gVisor (runsc) KVM-less fallback. New file `.elai_cc/crates/orchestrator/src/sandbox/gvisor.rs`. When `kvm_available()` returns false, spawn agent via `runsc` with platform=systrap (gVisor default since 2023). Document the 10-20% syscall overhead vs Firecracker 1-2% (gvisor.dev/performance + usenix.org hotcloud19 grounding). Emit `sandbox_backend_selected` SSB with backend=Firecracker|Gvisor|AppContainer per launch.

6. Researcher network egress: veth + iptables OUTPUT allowlist + DNS-via-trusted-resolver. New file `.elai_cc/crates/orchestrator/src/sandbox/researcher_net.rs`. Setup steps per researcher spawn (zero-caps): create veth pair, move one end into the researcher netns, configure iptables OUTPUT chain to ALLOW only curated arXiv mirror IP set + HF mirror IP set + trusted resolver (e.g. 1.1.1.1:53). DROP everything else with REJECT (not DROP) so the agent gets fast EHOSTUNREACH not a 30s hang. DENY CAP_NET_RAW + CAP_NET_ADMIN (researcher has zero caps already; verify). DNS bypass via raw sockets is closed by zero-cap. Allowlist sourced from elai_cc.toml [researcher.net].allowed_hosts (FQDN list resolved at boot, IPs pinned into the OUTPUT chain). Per shayon.dev 2026 sandbox-isolation analysis grounding.

7. Enforce per-baby-step ephemerality: fork-from-snapshot, never reuse process. Edit task_runner baby-step dispatch to call FirecrackerHotPool.spawn_from_snapshot per baby step rather than reusing a long-lived agent process. The "1" of the "1+4+postcard IPC" stays long-lived (control plane); the "4" workers are snapshot-restored per baby step (data plane). Grounding: TeamPCP / LiteLLM Mar 2026 — malicious pip install during baby step plants stealer; survives only if process survives. Emit `agent_forked_from_snapshot` SSB per fork.

8. Audit + close Pyodide host-bridge surface IF Pyodide is added to ELAI. Add a guard: search ELAI codebase for any Pyodide import. If found, the host-bridge must aggressively prune `globalThis.require`, `globalThis.process`, any handle to Node `child_process`, `fs`, `net` modules before exec. New module `.elai_cc/crates/orchestrator/src/sandbox/pyodide_bridge.rs` (file-stub if Pyodide absent; full implementation if added). Per cyera CVE-2025-68668 grounding.

9. Windows AppContainer hardening — ship the trio exactly. Edit Windows fallback code path. MUST: AppContainer profile with zero capabilities (per agent_id, unique SID); Job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` + `JOB_OBJECT_LIMIT_ACTIVE_PROCESS=1`; LowIL (Low Mandatory Label) descriptor on the working directory. Asserted in `tests/agent_isolation.rs::test_4a_appcontainer_trio_complete` (extends existing test_4a_appcontainer_spawn_real at line 293). Per Microsoft SandboxSecurityTools LaunchAppContainer reference + Chromium sandbox.md + WinBuzzer OpenAI Codex May 2026 grounding.

10. Snapshot per (agent_id, role) garbage collection. Snapshots accumulate at .elai_cc/sessions/{session_id}/snapshots/; LRU eviction at max_snapshots (default 64) AND max_age (default 24h since last restore). Emit `snapshot_evicted` SSB per eviction. Auditor checks snapshot dir size and reports in metrics.

11. Add `[sandbox.firecracker]` table to elai_cc.toml: `enabled = true` (default if KVM detected), `snapshot_dir = ".elai_cc/sessions/{session_id}/snapshots"`, `restore_budget_ms = 200`, `max_snapshots = 64`, `max_age_secs = 86400`, `uniqueness_mitigation = true`. Add `[sandbox.gvisor]`: `enabled_if_no_kvm = true`, `platform = "systrap"`. Add `[sandbox.appcontainer]`: `job_kill_on_close = true`, `active_process_limit = 1`, `low_il_descriptor = true`. Add `[researcher.net]`: `allowed_hosts = ["arxiv.org","huggingface.co"]`, `trusted_resolvers = ["1.1.1.1","8.8.8.8"]`, `reject_not_drop = true`.

12. Dashboard: extend get_sandbox_status (existing dashboard-v2/src-tauri/src/commands.rs:4962) with: backend_selected (Firecracker|Gvisor|AppContainer), restore_p50_ms, snapshot_count, uniqueness_regenerations_count, researcher_net_allowed_hosts, researcher_net_blocked_attempts (sample of last 10). New `SandboxBackendPanel.svelte` under crates/dashboard-v2/src/lib/components/ rendering these fields. Mount in OrchestrationDrawer. New SSB action_types: `firecracker_restore_slow`, `snapshot_uniqueness_regenerated`, `snapshot_evicted`, `sandbox_backend_selected`, `agent_forked_from_snapshot`, `researcher_net_egress_blocked`.

13. Measurements + acceptance baselines. New script `.elai_cc/scripts/measure_sandbox.sh` (POSIX + PowerShell variant) that boots 20 sandboxes, captures cold-boot p50/p95 + snapshot-restore p50/p95, writes to metrics/sandbox_perf.toml. End-to-end clone+restore+seccomp+ns-net wiring target: p50 < 100ms (report open Q1 — must be measured, not assumed). Snapshot-uniqueness orthogonality test (report open Q3): restore N=10 VMs and assert distinct TCP ISN + /dev/random pool + hostname; assertion in tests/snapshot_uniqueness.rs.

## WIRING_CONTRACT
produces_symbols:
  - orchestrator::sandbox::firecracker::FirecrackerHotPool
  - orchestrator::sandbox::firecracker::SnapshotHandle
  - orchestrator::sandbox::gvisor::GvisorBackend
  - orchestrator::sandbox::researcher_net::ResearcherNetSetup
  - orchestrator::sandbox::pyodide_bridge::PyodideBridge (file-stub if Pyodide absent)
  - orchestrator::sandbox::SandboxBackend (enum: Firecracker | Gvisor | AppContainer)
consumes_symbols:
  - orchestrator::sandbox::launch_agent (existing; refactored to dispatch via SandboxBackend)
  - orchestrator::sandbox_policy::Passthrough / SandboxConfig (existing)
  - providers::runtime::cli_common::TierCSpawn (B-37; Firecracker registers as a TierCSpawn impl)
  - core::config::parse_optional_sandbox_config (existing; extended with new tables)
  - dashboard-v2::commands::get_sandbox_status (existing; payload extended)
reachable_from:
  - crates/cli/src/main.rs::main → task_runner::drive_task → sandbox::launch_agent
  - crates/dashboard-v2/src-tauri/src/lib.rs::run → submit_task → sandbox spawn
  - tests/agent_isolation.rs::test_4a_* (new tests added)
stub_ban:
  - todo!() / unimplemented!() in any new sandbox sub-module
  - chroot codepath retention (must be REMOVED, not deprecated)
  - blanket socket() allow in seccomp filter
  - inverted NO_NEW_PRIVS / CAPBSET_DROP / seccomp ordering
  - Pyodide host bridge exposing require/process/child_process/fs/net without prune
  - Windows AppContainer without Job-object kill-on-close
  - Box<dyn Error> in any new module — use thiserror SandboxBackendError enum
  - .unwrap() outside #[cfg(test)] in any new file
frontend_pair: crates/dashboard-v2/src/lib/components/SandboxBackendPanel.svelte
observable_effect:
  trigger: "elai /task --mode cloud 'run 5 baby steps that each invoke a Python subprocess that does pip install of a known-malicious-style local fixture'"
  expect_ssb: "sandbox_backend_selected with backend=Firecracker (or Gvisor on no-KVM) per spawn; agent_forked_from_snapshot 5x; snapshot_uniqueness_regenerated 5x; researcher_net_egress_blocked if subprocess tries unknown host"
  expect_artifact: "20-spawn sandbox_perf.toml shows snapshot-restore p50 < 50ms target band; per-baby-step process IDs are distinct (no reuse); .elai_cc/sessions/{id}/snapshots/ contains LRU-bounded snapshot files"

## DETERMINISTIC_CHECK
```bash
# 1. New sandbox sub-modules exist
ls .elai_cc/crates/orchestrator/src/sandbox/firecracker.rs \
   .elai_cc/crates/orchestrator/src/sandbox/gvisor.rs \
   .elai_cc/crates/orchestrator/src/sandbox/researcher_net.rs
# expect: all 3 files exist

# 2. Arg-checked socket() in seccomp filter
rg "AF_INET|AF_INET6|AF_UNIX|AF_NETLINK|AF_PACKET" .elai_cc/crates/orchestrator/src/sandbox.rs
# expect: AF_INET / AF_INET6 in ALLOW path, AF_UNIX / AF_NETLINK / AF_PACKET in DENY path; >=5 hits

# 3. NO_NEW_PRIVS + CAPBSET_DROP + seccomp ordering asserted
rg "PR_SET_NO_NEW_PRIVS|PR_CAPBSET_DROP" .elai_cc/crates/orchestrator/src/sandbox.rs
# expect: both present and in correct order; >=2 hits

# 4. chroot codepath removed
rg "chroot" .elai_cc/crates/orchestrator/src/sandbox.rs --type rust
# expect: 0 hits (or only inside #[cfg(test)])

# 5. Snapshot-restore + uniqueness mitigation present
rg "FirecrackerHotPool|snapshot_uniqueness|spawn_from_snapshot" .elai_cc/crates/orchestrator/src/sandbox/
# expect: >=3 hits

# 6. SSB action_types registered
rg "firecracker_restore_slow|snapshot_uniqueness_regenerated|snapshot_evicted|sandbox_backend_selected|agent_forked_from_snapshot|researcher_net_egress_blocked" .elai_cc/crates/orchestrator/src/ssb.rs
# expect: >=6 hits

# 7. elai_cc.toml tables present
rg "\[sandbox.firecracker\]|\[sandbox.gvisor\]|\[sandbox.appcontainer\]|\[researcher.net\]" .elai_cc/elai_cc.toml
# expect: >=4 hits

# 8. Smoke: ordering test fails on inversion (Linux only)
cargo test -p elai-orchestrator --test agent_isolation test_4a_no_new_privs_capbset_seccomp_ordering -- --ignored
# expect: pass (when run under CAP_SYS_ADMIN per existing pattern)

# 9. Smoke: AppContainer trio assertion (Windows only)
cargo test -p elai-orchestrator --test agent_isolation test_4a_appcontainer_trio_complete -- --ignored
# expect: pass

# 10. Smoke: snapshot uniqueness — 10 restores produce distinct TCP ISN seeds
cargo test -p elai-orchestrator --test snapshot_uniqueness ten_restores_distinct
# expect: pass

# 11. Smoke: researcher net DENY for non-allowlisted host (REJECT path)
cargo test -p elai-orchestrator --test researcher_net_blocks blocks_unknown_host_with_reject
# expect: pass

# 12. Snapshot-restore p50 measured
cat .elai_cc/metrics/sandbox_perf.toml | grep "snapshot_restore_p50_ms"
# expect: value < 100
```

## ACCEPTANCE
- [ ] FirecrackerHotPool implemented; cold-boot first, snapshot-restore subsequent; per (agent_id, role) snapshot
- [ ] Snapshot-restore p50 < 100ms measured on user's 3090 box (per report open Q1)
- [ ] Snapshot-uniqueness mitigation regenerates TCP ISN + /dev/random pool + hostname per restore; 10-restore test asserts orthogonality
- [ ] BLOCK-1 socket() argument-checked in seccomp; AF_INET/AF_INET6 + SOCK_STREAM/SOCK_DGRAM allowed in NETWORK_EXTRA; AF_UNIX/AF_NETLINK/AF_PACKET DENIED
- [ ] NO_NEW_PRIVS → CAPBSET_DROP → seccomp ordering explicit + asserted by test
- [ ] chroot codepath removed entirely (not deprecated)
- [ ] GvisorBackend present and selected when kvm_available()==false
- [ ] ResearcherNetSetup uses veth + iptables OUTPUT allowlist + trusted-resolver DNS; CAP_NET_RAW + CAP_NET_ADMIN dropped; REJECT not DROP
- [ ] Per-baby-step fork-from-snapshot in task_runner; agent_forked_from_snapshot SSB per fork
- [ ] PyodideBridge file-stub or full impl with host-bridge prune (require/process/child_process/fs/net)
- [ ] AppContainer trio MUST: Job-object kill-on-close + active-process-limit=1 + LowIL descriptor; test_4a_appcontainer_trio_complete asserts
- [ ] Snapshot GC: LRU eviction at max_snapshots, max_age; snapshot_evicted SSB
- [ ] elai_cc.toml: [sandbox.firecracker] + [sandbox.gvisor] + [sandbox.appcontainer] + [researcher.net] tables
- [ ] 6+ new SSB action_types registered
- [ ] SandboxBackendPanel.svelte mounted; get_sandbox_status extended with new fields
- [ ] No NoopSsbNotifier; no Box<dyn Error>; no .unwrap() outside #[cfg(test)]
- [ ] scripts/measure_sandbox.sh produces metrics/sandbox_perf.toml with non-placeholder values

## Tests to add
- `.elai_cc/crates/orchestrator/tests/snapshot_uniqueness.rs`: 3 cases (ten_restores_distinct / hostname_salt_varies / tcp_isn_seed_varies)
- `.elai_cc/crates/orchestrator/tests/seccomp_arg_checked.rs`: 8 cases (AF_INET allow / AF_INET6 allow / AF_UNIX deny / AF_NETLINK deny / AF_PACKET deny / SOCK_RAW deny / SOCK_STREAM allow / SOCK_DGRAM allow)
- `.elai_cc/crates/orchestrator/tests/researcher_net_blocks.rs`: 5 cases (allowed_host_passes / unknown_host_rejects / dns_via_trusted_resolver_only / cap_net_raw_dropped / cap_net_admin_dropped)
- `.elai_cc/crates/orchestrator/tests/firecracker_restore_budget.rs`: 4 cases (restore_under_budget / restore_over_budget_falls_back / cold_boot_first_spawn / snapshot_evicted_on_lru)
- `.elai_cc/crates/orchestrator/tests/no_new_privs_ordering.rs`: 3 cases (correct_order / capbset_before_no_new_privs_fails / seccomp_before_capbset_fails)
- `.elai_cc/crates/orchestrator/tests/chroot_codepath_removed.rs`: regression test that fails if chroot is reintroduced
- `.elai_cc/crates/orchestrator/tests/baby_step_ephemerality.rs`: 3 cases (process_id_unique_per_step / malicious_pip_does_not_survive_step_boundary / snapshot_restored_clean)
- Extension to `tests/agent_isolation.rs`: test_4a_no_new_privs_capbset_seccomp_ordering (Linux), test_4a_appcontainer_trio_complete (Windows)
- `.elai_cc/crates/orchestrator/tests/pyodide_bridge_prune.rs`: 4 cases (require_blocked / process_blocked / child_process_blocked / fs_net_blocked)

## Files touched
- CREATE: `.elai_cc/crates/orchestrator/src/sandbox/firecracker.rs` (~480 LOC; HotPool + SnapshotHandle + uniqueness)
- CREATE: `.elai_cc/crates/orchestrator/src/sandbox/gvisor.rs` (~260 LOC; runsc subprocess + platform=systrap)
- CREATE: `.elai_cc/crates/orchestrator/src/sandbox/researcher_net.rs` (~340 LOC; veth + iptables + resolver)
- CREATE: `.elai_cc/crates/orchestrator/src/sandbox/pyodide_bridge.rs` (~180 LOC if Pyodide absent: file-stub; ~480 LOC if present)
- CREATE: `.elai_cc/crates/dashboard-v2/src/lib/components/SandboxBackendPanel.svelte` (~220 LOC)
- CREATE: `.elai_cc/scripts/measure_sandbox.sh` + `measure_sandbox.ps1` (~120 LOC each)
- MODIFY: `.elai_cc/crates/orchestrator/src/sandbox.rs` (+~220 LOC; arg-checked seccomp + ordering + dispatch to backend; -chroot codepath ~40 LOC)
- MODIFY: `.elai_cc/crates/orchestrator/src/sandbox_policy.rs` (+~40 LOC; SandboxBackend enum + selector)
- MODIFY: `.elai_cc/crates/orchestrator/src/task_runner.rs` (+~50 LOC; fork-from-snapshot per baby step)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~50 LOC; 6 new SsbActionType variants)
- MODIFY: `.elai_cc/crates/core/src/config.rs` (+~60 LOC; parse new sandbox tables)
- MODIFY: `.elai_cc/elai_cc.toml` (+~50 LOC; 4 new tables)
- MODIFY: `.elai_cc/crates/orchestrator/tests/agent_isolation.rs` (+~120 LOC; new tests)
- MODIFY: `.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs` (+~50 LOC; get_sandbox_status payload extension)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte` (+~25 LOC; mount SandboxBackendPanel)

## Citations / Grounding
- firecracker-microvm.github.io official — 125ms boot, <5 MiB per VM, 150 VMs/sec/host
- dev.to/adwitiya 2026 — cold ~1s, snapshot-restore ~28ms via CoW snapshot files (measured)
- gvisor.dev/docs/architecture_guide/performance — ~10-20% syscall overhead, Systrap default since 2023
- usenix.org HotCloud19 "True cost of containing" — per-syscall overhead in gVisor is workload-defining
- sysdig.com runc-container-escape-vulnerabilities — CVE-2025-31133/52565/52881 bind-mount race; maskedPaths defeated
- cyera.com / cvereports.com CVE-2025-68668 (n8scape Pyodide) — host bridge is the trust boundary, not the WASM VM
- pillar.security Antigravity escape — prompt injection + sandbox compat hole = RCE
- softwareseni AI Agents in Production Sandboxing — EchoLeak CVE-2025-32711 (CVSS 9.3) three chained flaws
- helpnetsecurity TeamPCP / bastion.tech LiteLLM Mar 2026 — malicious release bypassed GitHub release pipeline; survives across baby steps if process reused
- docs.docker.com seccomp + kernel.org seccomp_filter.html — argument filtering pattern for socket()
- man7.org/linux/man-pages/man2/prctl.2.html + kernel.org no_new_privs.txt — NO_NEW_PRIVS prereq for unprivileged seccomp mode 2
- arXiv:2102.12892 Restoring Uniqueness in MicroVM Snapshots — TCP ISN / entropy collision mitigation
- shayon.dev 2026/52 sandbox-isolation — drop CAP_NET_RAW; force traffic through proxy with domain allowlist
- microsoft/SandboxSecurityTools LaunchAppContainer — canonical Windows pattern
- chromium sandbox.md — 6+ years AppContainer hardening
- winbuzzer 2026/05/14 Codex Windows — OpenAI Codex ships AppContainer + Job + DPAPI + firewall trio
- northflank.com Kata vs Firecracker vs gVisor — Firecracker 1-2% overhead, gVisor 10-20%, Kata 5-10%
- daytona.io / e2b.dev / modal.com — managed-sandbox catalog (not chosen for ELAI Tier C; local-first per CLAUDE.md)
- github.com/apple/container — sub-second VM-per-container Swift; macOS-only, deferred

## Open questions / risks
- Ungrounded: p50 baby-step latency budget (report open Q1) — must be measured on user's 3090 box + Windows host before E-15 lands. scripts/measure_sandbox.sh produces the baseline; if p50 > 100ms, the Firecracker pool degrades to "warm" pool (pre-booted cold VMs) as a follow-up.
- Firecracker on Windows is Hyper-V-isolation-dependent (report open Q2); Windows path likely strictly AppContainer + Job. The dispatch must detect Hyper-V availability via WMI and degrade accordingly. Document the Windows decision in workflow/reasoning/E-15-decisions.md.
- Snapshot-uniqueness (arXiv:2102.12892 grounding): the mitigation regenerates seeds but does NOT defend against an attacker who reads /proc/sys/kernel/random/boot_id pre-restore. Document the residual risk in operatingManual.md.
- Cost of full netns teardown per baby step (report open Q4) is currently unmeasured; if it dominates the 100ms budget, reuse netns within a session and only teardown across sessions. Measure before deciding.
- gVisor compatibility tax for ELAI's bash workloads (cargo, npm install, python -m pip) per report open Q5 — measure end-to-end on the same baby step vs Firecracker; if gVisor adds >300ms to a typical install step, document and consider Kata as alternate.
- Pyodide is NOT currently in ELAI per the codebase grep; the pyodide_bridge.rs file is a file-stub now. If Pyodide is added later, the bridge must be filled in BEFORE the first Pyodide use; auditor blocks Pyodide imports until bridge implementation is present.
- The "REJECT not DROP" iptables policy in researcher_net gives the agent fast EHOSTUNREACH but also reveals the policy boundary. Acceptable trade per shayon.dev grounding (UX > stealth in inner loop).
- LiteLLM PyPI compromise (TeamPCP) extends to ALL pip installs in baby steps; the fork-from-snapshot mitigation evicts state but does NOT prevent the malicious download in the first place. Pair with E-11's PackageVerifier (registry HEAD + recent-publish-date check) for pre-install defense.

## Next
- Measure on user's 3090 + Windows host; produce metrics/sandbox_perf.toml; populate phase_4A.toml real metrics (replaces in-process-postcard placeholders per B-18's note).
- E-14 Playwright + npx requires Tier-C node access; E-15's PlaywrightWebAdapter sandbox profile must list node + npx in the program_allowlist of sandbox_policy.rs.
- Coordinate with E-11's PackageVerifier — both modules defend the pip-install pathway from different angles (E-11 prevents downloading slop; E-15 evicts state if it gets in).
- Quarterly review of CVE landscape via paper-ingestor mode=source on sysdig.com / kernel.org seccomp_filter / cf-detect patterns; the moving target is the threat surface, not the architecture.
