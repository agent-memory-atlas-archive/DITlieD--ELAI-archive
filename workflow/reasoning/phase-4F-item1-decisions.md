Phase 4F item 1 — Real Tier C stub spawn. Decision doc.

Session: s-11136 (resumes per HANDOFF s-32820 NEXT-SESSION ENTRY POINT)
FSM entry: PLANNING -> TESTING_FIRST -> EXECUTING -> AUDITING (1 cycle cap) -> MEMORIZING -> COMPLETE -> IDLE
Constraints: 1 audit cycle, no commits. Remaining BLOCKs/WARNs after the cycle go to HANDOFF.

WHAT
Extract a dyn-safe StubExecutor trait from feasibility_probe.rs. Implement Tier3StubExecutor (spawns elai_cc_agent via tokio::process::Command, dual timeout 25s graceful AgentKill IPC + 30s hard child.kill, child.wait reap). Implement SyntheticStubExecutor wrapping the existing run_synthetic_stub for the test default. Wire DefaultFeasibilityProber.stub_executor: Option<Arc<dyn StubExecutor>> with a with_stub_executor builder. Existing 23 contract tests stay untouched.

KEY DECISIONS
1. lib.rs not edited.
   Why: session s-38324 owns crates/orchestrator/src/lib.rs (Resume B15 finish). Multi-session lock conflict on session_claim.
   How: contract tests reference new symbols via the existing pub mod path: elai_orchestrator::feasibility_probe::{StubExecutor, Tier3StubExecutor, SyntheticStubExecutor, MockStubExecutor (test-only)}. Existing 23 tests continue to import via the flat re-export block lib.rs:L430-438 unchanged.
   Tradeoff: new public surface is reachable via crate::feasibility_probe::X rather than crate::X. Acceptable because all consumers within this session are in tests. If/when item 2 (adversarial tests A1-A8) wants the flat path, lib.rs will be patched in a separate session after s-38324 releases.

2. StubResult shape frozen at 4 fields.
   Why: SSB v25->v26 already shipped in commit 04f68b8. Adding fields (alternative_index, stdout_hash, stderr_summary, ResourceDelta) requires SSB v26->v27 bump, which is item 4 — out of scope for item 1.
   How: blake3 hashing of stdout, if needed for verification, stays internal to Tier3StubExecutor and is not propagated to StubResult.

3. AgentRole::Executor used (not Feasibility).
   Why: ipc-types AgentRole enum has no Feasibility variant (verified ipc-types/src/lib.rs:L19-28). Spec language "role=feasibility" describes the operational mode, not a literal enum variant.
   How: Tier3StubExecutor builds AgentDispatchConfig{role: AgentRole::Executor, tool_filter: ToolFilter{allowed_tools: vec![ToolId("stub_write"), ToolId("bash")]}, grc_budget_max_calls: 3, grc_budget_max_chars: 600, ...}. The FEASIBILITY tool restriction is enforced via the tool_filter, not the role tag.

4. Contract tests use a MockStubExecutor.
   Why: spawning a real elai_cc_agent binary in contract tests is non-deterministic, slow, and OS-dependent (Unix socket vs Windows named pipe). The trait abstraction is the unit under test; the real-binary integration test belongs in item 2 / adversarial tests.
   How: MockStubExecutor: pub(crate) struct with a configurable closure or fixed StubResult. Tests assert: (a) trait dyn-safety holds for Box<dyn StubExecutor>, (b) DefaultFeasibilityProber falls back to SyntheticStubExecutor when stub_executor=None, (c) DefaultFeasibilityProber delegates to the configured executor when Some, (d) line-limit rejection returns StubResult{stub_passed:false, error:Some("stub_line_limit_exceeded:N")} BEFORE invoking the executor.

5. Trait signature uses Pin<Box<dyn Future>>.
   Why: dyn-safety. async-trait crate is forbidden by anti-stale-training rules; native async fn in traits is fine when the call site does not need dyn dispatch, but Arc<dyn StubExecutor> requires explicit Pin<Box> until 1.86's RTITIT for Send+Sync is universally available in our toolchain.
   How: pub trait StubExecutor: Send + Sync { fn run_stub<'a>(&'a self, step_id: &'a str, stub_source: Option<&'a str>) -> Pin<Box<dyn Future<Output = StubResult> + Send + 'a>>; }

6. Existing run_synthetic_stub preserved.
   Why: spec design intent — synthetic stub is the test-default and the fallback when no real executor is configured.
   How: SyntheticStubExecutor::run_stub delegates to run_synthetic_stub. No behavior change for the 23 existing tests that exercise the synthetic path.

7. Constants reused.
   Why: codeRules rule 17 (no magic numbers) and the constants STUB_TIMEOUT_GRACEFUL=25s, STUB_TIMEOUT_HARD=30s already exist at feasibility_probe.rs:L39-42.
   How: Tier3StubExecutor uses tokio::time::timeout(STUB_TIMEOUT_GRACEFUL, ...) and STUB_TIMEOUT_HARD nowhere as a literal.

8. Zombie reap mandatory in both kill paths.
   Why: tokio::process::Child kill_on_drop = false. If Tier3 hits the 30s hard kill, child.kill().await leaves the child in a zombie state until reaped. Same for the graceful path if the child exits but the orchestrator drops the handle.
   How: explicit child.wait().await after every terminal branch (graceful exit, graceful timeout->kill, hard kill, response received).

9. No new dependencies.
   Why: anti-slopsquatting + codeRules rule 18.
   How: tokio (process feature), blake3, postcard, uuid, thiserror, tracing — all already in crates/orchestrator/Cargo.toml. nvml-wrapper / sysinfo deferred to item 3 with explicit user approval.

10. No .unwrap()/.expect() in new production code.
    Why: codeRules rule 1.
    How: spawn errors mapped through StubSpawnError (thiserror). All Results propagated via ? or matched. tracing::warn!() for non-fatal errors (kill failures on already-dead children).

EXPECTED TEST OUTCOMES
- Pre-implementation contract test run (TESTING_FIRST): new tests added compile-fail (StubExecutor / Tier3StubExecutor / SyntheticStubExecutor / MockStubExecutor / with_stub_executor not yet present). Existing 23 still pass.
- Post-implementation (EXECUTING done): cargo check --workspace clean. cargo test -p elai_cc_contracts --test feasibility_probe_contract: 23 existing + 4-7 new = 27-30 passing. cargo test -p elai-orchestrator --lib feasibility: >= 8 passing.
- Auditor 1-cycle: verify done-criteria 1-13 from transformed prompt. Remaining BLOCKs -> HANDOFF.

UNKNOWNS / RISKS
- Tier3 dual-timeout sequence has multiple await points; getting the cancel semantics wrong could leak processes. Mitigated by explicit child.wait() after every terminal branch + tracing::warn for non-fatal kill errors.
- Windows named-pipe vs Unix socket: code path branches via cfg!(target_os). Contract tests use mock executor so no OS dependency in the suite.
- s-38324 may release lib.rs lock mid-session. If so, no action — keep using the pub mod path. Do not retroactively edit lib.rs.

OUT OF SCOPE (deferred items, do not touch)
- SSB branch per stub alternative (item 6).
- StubResult field enrichment (item 4, needs SSB v27).
- VRAM re-check before stub launch (item 7).
- task_runner.rs NTZ L4640-4720 wiring (item 5).
- nvml-wrapper / sysinfo (item 3, dep approval gate).
- Adversarial tests A1-A8 (item 2, gates on item 1).
