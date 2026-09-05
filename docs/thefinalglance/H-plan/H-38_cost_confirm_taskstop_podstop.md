# H-38 -- cost-confirmation gate on TaskStop / pod_stop

- Track 4 (UX + USER-MEMORY) | lever SAFETY/STEERING | grounding MODERATE | nature BUILD | effort S | WAVE 4 | depends H-9 (SpendAccumulator/CloudSpend ledger), composes H-37 (InteractiveGate), composes orchestrate T1 tripwire
- Premise: when the user (or autonomy loop) stops a run, today nothing tells them what that stop costs or destroys. If a billable RunPod is still up, or an in-flight cloud job has spent real money and holds unsaved artifacts, the stop happens silently and the spend + work are gone. This is the Claude Code #28875 incident class: a stop ate ~$30 of pod time and ~1h of unsaved progress with zero confirmation. H-38 surfaces cost-so-far + unsaved-progress at the stop boundary and requires an explicit confirm before a billable/unsaved stop proceeds; a non-billable, fully-saved stop passes with zero friction (false-positive guard).

## Steal/build
BUILD (nature=BUILD). The cost-confirmation decision is new logic. But it COMPOSES four already-shipped pieces rather than restating them (GD-1): (a) the spend ledger from H-9 — SpendAccumulator/CloudSpend already compute day_usd/month_usd and the SsbViewer already aggregates it; (b) the live in-runtime stop hook (hook_dispatcher dispatch_stop); (c) the H-37 InteractiveGate present_gate human-I/O seam (reuse the same CliInteractiveGate already on RuntimeWiring — do NOT add a second prompt path); (d) the orchestrate-skill T1 irreversible-op tripwire for the HARNESS layer (pod-lifecycle pod_stop is a harness MCP tool, NOT ELAI Rust — see Integration). odysseus has no direct prior art for a cost gate; the nearest reimplement-reference is its untrusted/validate-before-commit posture (teacher_escalation.py:621 NO_SKILL distill gate — "only act when the result is parseable+valid"), which maps to "only stop without friction when the stop is provably cheap+saved." Reimplement the gate predicate in Rust against the SSB spend ledger; do not port Python.

## Existing ground truth (code-read)
Spend ledger (H-9, PROD-WIRED):
- crates/orchestrator/src/cloud_mode.rs:646 `struct SpendAccumulator` (:659 impl, :663 for_claude_sonnet_4_6, :683 for_model, :697 accumulate, :708 accumulate_cache; finalize -> CloudSpendPayload per doc :641-644).
- crates/orchestrator/src/cloud_mode.rs:521-531 `CloudSpendSnapshot` day_usd/month_usd (:531 from_day_usd).
- crates/orchestrator/src/cloud_mode.rs:309 `route_with_spend`, :326 `route_cloud`, daily-cap block at :357-365 (`snap.day_usd >= cap` -> CloudModeError::DailySpendCapExceeded), monthly at :366-373.
- crates/orchestrator/src/ssb_viewer.rs:5062 `cloud_spend_summary(now_ms)` aggregates CloudSpend rows into `day_usd` (:356 field, :5106 accumulation). This is the off-agent recompute source for the confirm figure.
- Live daily-cap enforcement on the send path (PROD-WIRED): crates/orchestrator/src/grounding_client.rs:841 `send_with_ledger`, cap block :848-905 (opens SsbViewer at :852, reads cloud_spend_summary :853, blocks at :855 when `summary.day_usd >= cap`, emits a reused EscalateToCloud row trigger_reason="cap_reached" :859-866, degrades-to-skip on viewer-open failure :897-904, non-fatal). NOTE this is a POST-HOC daily-cap block, NOT a pre-spend or pre-stop confirm — the H-38 gap.

Stop path (PROD-WIRED, but no cost awareness):
- crates/orchestrator/src/hook_dispatcher.rs:404 `dispatch_stop` (trait), :1146 InProcessHookDispatcher impl. Current behavior: B-19 non-IDLE block only (:1153-1173 returns HookVerdictKind::Block when fsm_state is set and not IDLE/CONFIGURING). It has ZERO cost/pod awareness — it never reads the spend ledger or pod state. This is the in-runtime stop boundary to wire onto.
- task_runner stop path: the cost-gated confirm-before-stop is NOT wired anywhere today. The only spend gate is the post-hoc daily-cap block in grounding_client (above).

InteractiveGate seam (H-37, present_gate PROD-WIRED, decision-loop WIRE-DARK):
- crates/orchestrator/src/gate.rs:126 `trait InteractiveGate` (:129 present_gate), :25 `enum GateDecision` (Approve/Reject/Modify/Redirect), :206 InteractiveGateOutcome.
- crates/cli/src/runtime_builder.rs:429 `CliInteractiveGate` (:431 impl InteractiveGate, :432 present_gate = real human I/O). Installed in prod: crates/cli/src/run_subcommand.rs:120 `Arc::new(CliInteractiveGate)` onto RuntimeWiring, threaded via runtime_setup.rs:100 interactive_gate -> :514 TaskRunnerConfig. REUSE this; do NOT author a second prompt path.

pod_stop / pod_cost_status (HARNESS-SIDE, NOT ELAI Rust — load-bearing correction):
- The only appearance of "mcp__pod-lifecycle__pod_stop" in ELAI source is a string literal in a corpus example: crates/plugins_macros/examples/corpusgen.rs:71 (NOT a caller). ELAI runtime has NO pod-lifecycle client. pod_stop / pod_cost_status are Claude Code HARNESS MCP tools. So the pod-cost half of H-38 is a workflow-harness concern (orchestrate skill / harness MCP), NOT an ELAI Rust wire-in. The ELAI Rust half is the cloud-spend confirm on the in-runtime stop hook.

orchestrate T1 tripwire (HARNESS-SIDE, COMPOSE not restate):
- F:\.ELAI_workflow\.claude\skills\orchestrate\SKILL.md:57 "T1 IRREVERSIBLE OP detected in proposed Bash command (git push --force, rm -rf, db drop, prod deploy, npm publish, cargo publish): surface to user with the exact command and a Y/N. Default no." H-38 ADDS pod_stop-when-billable + cost figure to the T1 surface list; it does not re-author the tripwire (GD-1).

SSB schema (live, confirmed at authoring 2026-06-04):
- crates/orchestrator/src/ssb.rs:2990 `const SCHEMA_VERSION: i64 = 220;` (public alias :2995). crates/orchestrator/tests/action_type_lock.rs:624 `const TOTAL_VARIANTS: usize = 497;` (golden lock, lockstep). There is NO existing action_type for a stop-confirmation event.

The precise production gap: stops happen at two distinct boundaries (ELAI in-runtime hook_dispatcher dispatch_stop; HARNESS pod_stop MCP) and NEITHER reads the spend ledger nor surfaces unsaved-progress before proceeding. The ledger to read it from EXISTS and is PROD-WIRED. The gap is a gate predicate + a confirm surface + an off-agent SSB beacon, not a new ledger.

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1, the vertical slice):
- New ELAI Rust gate predicate `StopCostGate` (crates/orchestrator/src/stop_cost_gate.rs) with a pure fn `evaluate_stop(snapshot: StopCostSnapshot) -> StopGateVerdict` where StopCostSnapshot carries { day_usd: f64, unsaved_artifacts: usize, pod_billable: bool } and StopGateVerdict is Pass | ConfirmRequired { cost_usd, unsaved_count, reason }. Predicate: ConfirmRequired iff (day_usd > CONFIRM_COST_FLOOR_USD) OR (unsaved_artifacts > 0) OR pod_billable; else Pass (the FP guard — a $0 fully-saved non-pod stop passes silently). CONFIRM_COST_FLOOR_USD is a named const (no magic number).
- Wire `StopCostGate::evaluate_stop` into the in-runtime stop boundary: read day_usd from SsbViewer::cloud_spend_summary (ssb_viewer.rs:5062), read unsaved-artifact count from the run's pending-write set, and on ConfirmRequired present via the EXISTING CliInteractiveGate present_gate (gate.rs:129). A GateDecision::Reject aborts the stop; Approve proceeds. Default-NO on a non-interactive/headless context (fail-closed: when no human can answer, do not silently destroy spend/work — surface and hold).
- Emit ONE off-agent SSB beacon `StopConfirmationEvent` carrying the cost_usd (recomputable from the spend ledger), unsaved_count, and an outcome arm (ConfirmedProceed / RejectedHeld / PassedNoFriction). New Pattern-2 single-variant.
- HARNESS half: extend the orchestrate SKILL.md T1 list so a proposed `mcp__pod-lifecycle__pod_stop` (or `pod_terminate`) on a pod whose `pod_cost_status` shows nonzero cost-so-far surfaces the cost figure in the Y/N (composes the existing T1, does not restate it). This is a skill-doc edit, not Rust.

DEFER (Tier-2, out of this slice): per-pod cost reconciliation against RunPod billing API; multi-pod batch-stop confirm; a dashboard cost-confirm modal panel (the figure already surfaces via CloudPanel — no new Svelte panel in Tier-1). Auto-checkpoint-before-stop (save the unsaved artifacts instead of just warning) is a separate plan.

Local-token discipline: the gate reads day_usd (one f64) + an artifact count (one usize) on the stop path only — it is NOT an always-injected context block and never enters a model prompt. Zero per-turn token cost; the confirm string is human-facing CLI I/O, not context.

Staleness: not a memory plan; no bi-temporal store. The cost figure is read fresh from the SSB ledger at stop time (cloud_spend_summary now_ms), so it cannot be stale — it is recomputed, never cached.

## Integration
Files/sites to wire:
- NEW crates/orchestrator/src/stop_cost_gate.rs — StopCostGate, StopCostSnapshot, StopGateVerdict, evaluate_stop, CONFIRM_COST_FLOOR_USD const. Pure + sync (testable without a writer), mirroring route_cloud's pure-predicate shape (cloud_mode.rs:326).
- crates/orchestrator/src/lib.rs — `pub mod stop_cost_gate;` + re-export.
- crates/orchestrator/src/hook_dispatcher.rs:1146 dispatch_stop — after the B-19 non-IDLE check (:1153-1173) and only when state IS idle/configuring (i.e. a real task-end stop), build a StopCostSnapshot (day_usd via SsbViewer::cloud_spend_summary, unsaved count via the run's pending-write set, pod_billable threaded from config), call StopCostGate::evaluate_stop; on ConfirmRequired invoke the threaded InteractiveGate present_gate and emit the StopConfirmationEvent beacon; on GateDecision::Reject return a HookFinding Block (rule_id="stop_cost_confirm_required") so task_runner holds the stop. Thread the InteractiveGate Arc + SsbViewer/db_path into InProcessHookDispatcher (it already holds an SSB writer handle for HookEvent rows at hook_dispatcher.rs:1305).
- frontend_pair: crates/dashboard-v2/src/lib/screens/CloudPanel.svelte (already surfaces CloudSpend via commands.rs) — Tier-1 reads the same day_usd; no new command required. (Both files are in the git-modified set.)

SSB action_type + SCHEMA bump:
- New `StopConfirmationEvent` (Pattern-2 single-variant, mirroring CanvasActivityEvent: ssb.rs:8753 SsbActionType arm, :9557 as_str "stop_confirmation_event", :10265 from_str, :13743 SsbPayload arm, :14451 -> action_type map; payload struct StopConfirmationPayload near ssb.rs:27005 with a 3-arm StopOutcomeKind discriminator: ConfirmedProceed / RejectedHeld / PassedNoFriction; fields cost_usd: f64, unsaved_count: u32, ts_ms: i64).
- SCHEMA bump note: SCHEMA baseline+1 = 220 -> 221 at ssb.rs:2990 (confirm live const in crates/orchestrator/src/ssb.rs at authoring), and TOTAL_VARIANTS 497 -> 498 at action_type_lock.rs:624 in lockstep. Add the round-trip + fuzz arms (ssb.rs:31220 and :35291 patterns) and the action_type_lock coverage string.
- Default-OFF gate: the StopConfirmationEvent beacon is wire-dark unless `ELAI_STOP_COST_CONFIRM_ENABLE=1` while the gate predicate is on probation. PER W-2/R9 this needs a wire-dark-allowlist.toml row: type=experiment, owner=H-38, expiry_date=2026-09-04, promotion_plan_id=H-38-promote, plus a smoke test exercising the ON path (a billable stop that emits one ConfirmedProceed row). The Y/N confirm itself (the safety behavior) is ON unconditionally — only the SSB telemetry beacon is gated, so the gate cannot silently fail-open.

## WIRING_CONTRACT
- produces: SsbPayload::StopConfirmationEvent rows (cost_usd recomputable from the SSB CloudSpend ledger via SsbViewer::cloud_spend_summary); a HookFinding Block(rule_id="stop_cost_confirm_required") on Reject.
- consumes: SsbViewer::cloud_spend_summary (ssb_viewer.rs:5062) day_usd; the run pending-write set (unsaved artifact count); InteractiveGate::present_gate (gate.rs:129); CONFIRM_COST_FLOOR_USD const.
- reachable_from: the in-runtime stop boundary InProcessHookDispatcher::dispatch_stop (hook_dispatcher.rs:1146), driven by a REAL task-end stop (the production stop hook fired by task_runner at run end), NOT a unit test calling evaluate_stop directly. HARNESS half reachable from the orchestrate skill T1 surface on a proposed pod_stop.
- stub_ban: NO empty StopConfirmationPayload arm; NO `let _ = verdict` swallow of ConfirmRequired; NO unconditional Pass; NO magic cost floor (named const); on a headless/non-interactive context the verdict is HOLD (fail-closed), never silent-proceed.
- observable_effect (W-1 done-gate): a stop on a BILLABLE/UNSAVED job surfaces cost-so-far + unsaved-progress and requires confirmation, AND a `stop_confirmation_event` SSB row lands with the cost figure (recomputable from the spend ledger); a NON-billable, fully-saved job stops without friction and lands a PassedNoFriction row (the FP guard). Gate = wiring_contract_check --run-trigger PASS: a real dispatch_stop ran and the off-agent SSB row appeared in the window; missing row = FAIL (R1). NOT cargo-green.

## Grounding
- MEASURED incident (the premise): Claude Code GitHub issue #28875 — a stop killed a billable pod + lost ~1h unsaved work, ~$30 class. github.com/anthropics/claude-code/issues/28875 (dated snapshot 2026-06-04; verify the exact $ + time figures against the issue thread at authoring — the ~$30/~1h numbers are from the issue report, MEASURED-by-reporter, not by ELAI). This is the motivating MEASURED data point; ELAI's own per-run cost figure in the gate is MEASURED from the SSB CloudSpend ledger (recomputable under ELAI_SEED), never a literature number (HD-1/GD-2).
- ELAI ground truth (code-read 2026-06-04): spend ledger cloud_mode.rs:646/521/309/357; ssb_viewer.rs:5062 cloud_spend_summary day_usd; grounding_client.rs:848-905 post-hoc cap precedent; hook_dispatcher.rs:1146 dispatch_stop; gate.rs:126/129 InteractiveGate; runtime_builder.rs:429 CliInteractiveGate; run_subcommand.rs:120 install; ssb.rs:2990 SCHEMA 220; action_type_lock.rs:624 TOTAL_VARIANTS 497; corpusgen.rs:71 pod_stop literal (not a caller); SKILL.md:57 T1 tripwire.
- odysseus reimplement-reference (MIT prior-art, REIMPLEMENT in Rust, never port): docs/.source/odysseus-main/src/teacher_escalation.py:621 NO_SKILL distill gate ("only act when the result is valid/parseable") — analog of "only stop without friction when the stop is provably cheap+saved; otherwise hold for confirm." No odysseus cost-gate exists; this is the closest validate-before-act posture. (claimed analogy, not a measured number.)
- Pattern-2 single-variant template followed verbatim from CanvasActivityEvent (ssb.rs:8753/9557/10265/13743/14451/26981/27005/31220/35291; action_type_lock.rs:620). (MEASURED structural template, live.)

## COMPOSES-WITH
- H-9 (SpendAccumulator/CloudSpend ledger) — the cost source; H-38 reads it, never re-implements spend math (GD-1).
- H-37 (InteractiveGate / compliance pre-gate) — reuses CliInteractiveGate present_gate as the confirm surface; H-38's confirm is a second consumer of the SAME installed gate, not a parallel prompt path.
- orchestrate skill T1 irreversible-op tripwire (SKILL.md:57) — H-38 extends the T1 surface list for pod_stop-when-billable; does not restate the tripwire.
- F-9 / cloud daily-cap (grounding_client.rs:848) — H-38 is the PRE-stop confirm complement to that POST-hoc cap; both read the same ledger.
