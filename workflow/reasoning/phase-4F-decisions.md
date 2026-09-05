B-21 Phase 4F — Decision Doc
Session: s-32820 | FSM phase: B-21 4F Feasibility Probe completion
Constraints: 1 audit cycle hard cap, no commits, project boundary = .elai_cc

What
Complete the Feasibility Probe subsystem (Rust backend mostly shipped in batch-3a c5abfeb) by closing the wiring gaps and replacing the dashboard's UnwiredNote with live SSB-backed data.

Why now
Plan B-21 phase 4F is the gate before EXECUTING for tasks with novel steps. The Rust backend is in place but never fires in production (prober=None) and the Tauri command returns a static stub. Without these final wires the dashboard cannot show whether a probe ran, and no probe ever runs in production.

Decisions

D1 — SSB action_type reconciliation: use existing v26 variants (novel_step_detected, stub_run, feasibility_report, system_probe). Plan's spec referenced 4 additional variants (feasibility_probe_started, feasibility_probe_complete, alternative_explored, feasibility_escalated) that batch-3a deliberately did not ship — semantic content is fully covered by the 3 shipped variants. Adding 4 new variants would force an SSB schema bump v26 -> v27 with migration; that is out of scope and explicitly forbidden this phase. The plan's DETERMINISTIC_CHECK grep for FeasibilityProbeStarted is reconciled by treating NovelStepDetected as the start-marker and FeasibilityReport as the end-marker.

D2 — Frontend wiring location: render the live feasibility section inline inside ReasoningDrawer.svelte by replacing the existing UnwiredNote block (section 4 "INLINE FEASIBILITY"). No new src/lib/components/ directory. Existing pattern (drawer-level panels inline) is preserved. If the inline section grows past 150 lines we extract to src/lib/drawers/FeasibilityPanel.svelte. Rationale: lower risk, matches existing drawers pattern, no import-graph churn.

D3 — DefaultFeasibilityProber production wiring scope: dashboard-side only. Add a `feasibility_prober: Arc<dyn FeasibilityProber>` field to AppState constructed at AppState::new() rooted at `<project_root>/.elai_cc/feasibility/`. Task_runner production construction paths (cli/main.rs, agent_pool, etc.) keep `feasibility_prober: None` for this phase. Rationale: narrowest scope; W1-SEMANTIC deferred fix at task_runner.rs:4657-4672 makes broader task_runner edits risky.

D4 — Per-concept probe_baby_step: env-gated, fire-and-forget SSB emission only — no control-flow change. Insert at the top of the baby-step loop iteration (right after `steps_executed += 1;` at task_runner.rs:3722) so it executes after `nav.next_baby_step` returns. Gates: (a) `resolved.feasibility_prober.is_some()`, (b) `!feasibility_probe_disabled_via_env()`, (c) `tree.find_baby_step(bs_id).is_some()`. Probe runs, the resulting BabyStepFeasibilityReport is emitted as a `StubRun` SSB row (existing variant); when feasible=false a NovelStepDetected row is also emitted. The runner does NOT short-circuit on infeasible — the existing baby-step machinery (CSQ, hooks, executor, mistake-book) continues unchanged. Rationale: any flow change would interact with the W1-SEMANTIC deferred site at L4657-4672 and the heavily-conditional outcome propagation; keeping the probe purely informational eliminates that risk while still firing real SSB rows the dashboard can read.

Risks (top 3)
R1 (probability MEDIUM, impact MEDIUM): SsbViewer needs a new query method to surface feasibility rows with full payload bytes (recent_actions only returns truncated summaries). Mitigation: add a focused `feasibility_summary(limit)` viewer method that decodes FeasibilityReport / NovelStepDetected / StubRun rows. Method scope is narrow and matches existing patterns (`pending_interventions`, `rule_violations`).
R2 (probability LOW, impact HIGH): wiring DefaultFeasibilityProber in dashboard AppState::new() runs the system prober at construction time which shells out to nvidia-smi. Mitigation: prober itself does NOT probe at construction; it only constructs when probe() is called. AppState::new() stays fast.
R3 (probability LOW, impact MEDIUM): per-concept probe in task_runner adds two SSB writes per baby step. Mitigation: gated by env (production CI sets ELAI_DISABLE_FEASIBILITY_PROBE=1) and feasibility_prober=None default, so the cost is opt-in.

Top 3 failure modes
F1: get_feasibility_report panics when SSB DB is empty -> dashboard crash. Mitigation: empty rows path returns `{ overall: "PASS", results: [], probed_at: now }` without panic. Tested by zero-rows assertion.
F2: msgpack decode error on feasibility_report row (stale schema). Mitigation: errors -> tracing::warn + skip the row; partial result is still returned.
F3: probe_baby_step adds latency to every baby-step iteration. Mitigation: synthetic stub takes <1 ms; gated behind ELAI_DISABLE_FEASIBILITY_PROBE for CI.

Expected test outcomes
- 5 new contract tests in feasibility_probe_contract.rs reach 16 contracts total (matches plan's "16 contract tests" target). System_probe_contract.rs untouched.
- New SsbViewer method `feasibility_summary` exercised by tests in ssb_viewer.rs unit test module (zero-row, populated-row).
- cargo test -p elai-contracts -> all green.
- cargo test -p elai-orchestrator feasibility -> all green.
- cargo check --workspace -> clean.

Acceptance
- 16 feasibility contracts + 11 system_probe contracts present.
- get_feasibility_report decodes real SSB rows; static stub removed.
- AppState carries `feasibility_prober`, constructed at startup.
- Per-concept probe_baby_step fires when prober plumbed AND env not set.
- ReasoningDrawer renders live data; UnwiredNote removed.
- drawers.svelte.ts: `unwired.feasibility = false`.
- No new SsbActionType variants. SSB schema stays v26.
- No new dependencies. No .unwrap() in library code. Native async fn / Pin<Box<dyn Future>> traits preserved.
- task_runner.rs lines 4640-4720 untouched (W1-SEMANTIC NTZ).
