phase: B3 — e2e_full_pipeline_local test
date: 2026-05-15
session: s-5240

WHAT:
A new test crates/orchestrator/tests/e2e_full_pipeline_local.rs that submits "Build me a single-page React TODO app with localStorage persistence. Allow add, mark-complete, and delete actions." against the production orchestrator wired with ProductionAgentRpcServer + real sidecar (when reachable at http://localhost:8100). Asserts (1) ≥1 ConceptDecompositionNode SSB row, (2) ≥1 FileWrite SSB row under tempdir/src/ OR App.{jsx,tsx} hash differs from baseline scaffold, (3) npm run build exits 0, (4) vite preview HTTP probe returns 200 + body_bytes>1024 + lowercased body contains "todo"/"task".

KEY DECISIONS:
1. Test gated with #[ignore] + ELAI_E2E_FULL_PIPELINE=1. Default cargo test skips it. Live-run procedure documented in commit message + handoff (requires GPU pod + SSH tunnel up + sidecar booted).
2. Sidecar health-check (4-token AnthropicClient.send_message) fires BEFORE the 15-minute timer. Test fails fast with "sidecar not reachable at <url>" if unreachable — no 15-min wasted budget.
3. Wiring matches CLI run_subcommand.rs:91-200 (B2c ProductionAgentRpcServer + inline NoOpIntegrationCommandRunner). The inline struct is duplicated in the test file because the CLI's NoOpIntegrationCommandRunner is not pub-exported; duplication is acceptable per executor brief.
4. SSB schema correction: real wire is SsbActionType::FileWrite { path, size_bytes, hash } (per ssb.rs:917). Phase plan's reference to a non-existent BabyStepResult was corrected before executor ran.
5. Fallback for direct-fs writes: even if the agent process writes files directly to disk (bypassing IPC ToolRequest), the disk_write_detected branch compares baseline vs. final App.{jsx,tsx} hashes. So assertion 2 succeeds whether the path is FileWrite-emitting or direct-fs.
6. Prompt verbatim: "Build me a single-page React TODO app with localStorage persistence. Allow add, mark-complete, and delete actions." Confirmed routes to real agent decomposition path (does NOT match the "make me a" scaffold short-circuit at task_runner.rs:~3482).
7. Wall-clock budget: 15 min for LLM phase + 5 min for npm install/build/preview = 20 min total #[tokio::time::timeout] enforced at outer scope.
8. On any failure: panic message includes (a) full SSB row tally by type, (b) tempdir/src directory listing with sizes, (c) first 200 chars of App.{jsx,tsx}, (d) preview stdout/stderr tail. Honest failure mode — no silent success on missing assertions.

EXPECTED TEST OUTCOMES:
- Compile: PASS (cargo build -p elai-orchestrator --tests).
- cargo check --workspace: PASS.
- Live run: deferred — pod capacity-constrained in EU-RO-1 at session time. Migration attempts: L4 (default), RTX 4090, RTX A5000, RTX 3090, RTX A4000, A40 — all SUPPLY_CONSTRAINT. Eventual migration succeeded to a Blackwell-class pod (n1ndc1d4zz52ay) but SSH did NOT come up after 5 min (matches prior session's 12+ min Blackwell-no-SSH report). Pod was stopped to avoid burn. Test remains live-runnable when capacity returns.
- Regression suite (4 contracts + 1 e2e mock): 13/13 PASS.

UNKNOWNS RESOLVED:
- FileWrite is the real SSB row type for file writes (not BabyStepResult).
- "Build me" prompt prefix does NOT match the "make me a" scaffold short-circuit; routes to real agent decomp.
- The 4 WARNs surfaced by the auditor (env_set_var threading, port race, poll-loop wall-clock with no file-write signal, missing decision doc) are acceptable: (a) --test-threads=1 mitigates env race, (b) port race pattern matches existing shipped e2e, (c) disk_write_detected fallback covers direct-fs path, (d) this file resolves the missing decision doc.

OPEN UNKNOWNS / DEFERRED:
- Whether the L4 Qwen agent uses IPC ToolRequest for file writes OR direct-fs access: TBD by live run.
- Whether 15-min budget is sufficient for cold-start 3B model + 4-level decomposition: prior session's measurements indicate yes, but cold-start measurements with B2b+B2c wiring not yet taken.
- D2 will extend B3's HTTP probe with a JS-bundle semantic grep (localStorage / addTodo / removeTodo / completed). D2 is a separate phase.

AUDITOR RESULT: WARN-only, score 0.91. No BLOCKs. Proceed to MEMORIZING.

LIVE RUN PROCEDURE (when pod capacity returns):
  1. mcp__pod-lifecycle__pod_start (or with migrate_target_gpu_type if L4 unavailable)
  2. SSH up: ssh -p <port> -i ~/.ssh/embervow root@<host> 'echo SSH_UP'
  3. Sidecar boot: ssh ... '/workspace/launch_sidecar.sh' (or inline command from CLAUDE.md)
  4. Tunnel: ssh -f -N -L 127.0.0.1:8100:127.0.0.1:8100 -p <port> -i ~/.ssh/embervow root@<host>
  5. Health-check: Invoke-RestMethod http://127.0.0.1:8100/v1/messages with 4-token payload
  6. Run test: $env:ELAI_E2E_FULL_PIPELINE="1"; cargo test -p elai-orchestrator --test e2e_full_pipeline_local -- --ignored --nocapture --test-threads=1
  7. pod_stop after run.
