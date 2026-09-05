# J-81: Autonomous supervision hardening — process-wedge detection + durable wake-queue + mid-loop cost cap

STATUS: PLANNED (WAVE KAT-1; M; EXTENSION(auto-orchestrate / orchestrate supervision + E-39) — deterministic supervision legs, no new subsystem)

Objective: three deterministic supervision gaps the kunchen firstmate/gnhf cluster exposes, none of which is a new subsystem — each rides existing seams. (1) PROCESS-LEVEL WEDGE DETECTION: ELAI's anti-spiral is REASONING-level (phase-3C blake3 3-identical-hash stop) and orchestrate's T4 tripwire is sweep-fail-3x / fixer-3x — NEITHER catches a silently-hung worker producing NO output. firstmate fm-watch.sh:130-264 detects stale via a status-file size:mtime signature + pane-hash-identical-2-cycles + no-busy-signature. Add output-unchanged-N-cycles = STUCK detection to the auto-orchestrate / orchestrate supervision loop (deterministic, no LLM). (2) DURABLE WAKE-QUEUE: resume the supervision loop across a SUPERVISOR crash (firstmate's wake-queue survives watcher death + singleton lock + signal coalescing FM_SIGNAL_GRACE=30s); ELAI's sleep-poll (tools/dep-wait.ps1, mcp_session_list every 60s) is cruder and dies with the supervisor. Compose it with SESSION_HANDOFF / PreCompact. (3) MID-LOOP DETERMINISTIC COST/ITERATION/WALLCLOCK CAP: gnhf aborts the CURRENT step mid-iteration via onUsage→AbortController (RunLimits maxIterations/maxTokens/stopWhen, orchestrator.ts:74-79); ELAI's E-39 COST ECONOMICS HARD CEILING (DARK, "blocks-not-warns wallet-loop closure") is the analog but has NO task_runner caller, and auto-orchestrate's envelope counts DISPATCHES, not mid-run tokens. Add a deterministic mid-loop token/iteration/wallclock cap that aborts the current step (rides E-39 + the auto-orchestrate envelope). HARD BOUNDARY: NOT gnhf's LLM-self-judged stopWhen (should_fully_stop) — TD-3 bans an LLM judging its own stop; the stop predicate is deterministic/observable. DO-NOT-COPY: gnhf `git reset --hard` iteration rollback (use H-49 CAS / RV-1); firstmate `--dangerously-skip-permissions` / `--dangerously-bypass-approvals-and-sandbox` (the precise anti-pattern ELAI's Tier-C + VibeGate refuse); firstmate secondmates (hierarchical autonomous supervisors — OFF-WEDGE vs the central-FSM-gate trust spine, same class as goose-topology NO-STEAL). Source: workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C1.2 + C1.3 + SYNTHESIS TIER-1(3) + TIER-2(5).

Estimated effort: ~1 session (~M). Mostly HARNESS (.claude skills/rules, JD-1 evidence) + the E-39 runtime cost-cap leg.
Pre-requisites (re-grep live per W-7): auto-orchestrate + orchestrate SKILL.md supervision envelope (run-state-unchanged-3x / BLOCKED-twice / T4); orchestrate tools/dep-wait.ps1 (the sleep-poll to harden); anti-spiral phase-3C blake3 3-hash stop; E-39 COST ECONOMICS HARD CEILING (velocity/automix/cost_router, DARK, no task_runner caller); SESSION_HANDOFF / PreCompact hooks; bubble-up.md retry budgets (50-event cap + fixer-3x).
Parallel-safe with: J-80, J-82, K-11, K-12 (disjoint files).
Source: steal-kunchen C1.2 (gnhf RunLimits) + C1.3 (firstmate fm-watch waker). Doctrine: J-0_INDEX.md (W/GD/JD inherited; ADDS no delta family).

## Context
Why this matters: a silently-hung worker is invisible to ELAI's current detectors (reasoning-level hash-stop, dispatch-count envelope, sweep/fixer counters) — it can burn the autonomy window producing nothing. firstmate's output-signature waker catches exactly that, deterministically. The durable wake-queue makes the supervision loop crash-survivable (composes the existing SESSION_HANDOFF). The mid-loop cost cap closes the gnhf gap where a runaway step blows the budget before any per-dispatch check fires — but as a DETERMINISTIC observable cap, never gnhf's LLM-self-judged stop (TD-3). All three are supervision-hardening RIDERS, not a new subsystem; the firstmate hierarchical-supervisor (secondmates) and the `--dangerously-*` posture are explicitly refused as the trust-light foil ELAI's wedge is built against.

## Existing state (B/C/D/E/F/G/H/I coverage — W-7 anchors, executor re-greps)
- auto-orchestrate SKILL.md autonomy envelope [WIRED doctrine] — 30-dispatch check-in, BLOCKED-twice, run-state-unchanged-3x; counts DISPATCHES not mid-run output/tokens.
- orchestrate T4 tripwire [WIRED doctrine] — sweep-fail-3x / fixer-3x; no process-wedge or output-signature detector.
- anti-spiral phase-3C [WIRED] — blake3 3-identical-hash stop (REASONING-level, not process-level).
- E-39 COST ECONOMICS HARD CEILING [DARK(ELAI_COST_TOKEN_ECONOMICS_ENABLE)] — blocks-not-warns wallet-loop closure; velocity/automix/cost_router has NO task_runner caller (the mid-loop cost cap wires it).
- SESSION_HANDOFF / PreCompact hooks [WIRED] — the durable-wake-queue composition target.
- bubble-up.md [WIRED doctrine] — 50-event cap + fixer-3x stall (the gnhf commit-repair/backoff RIDER lands here).
- HOOK SYSTEM Stop/PreCompact [WIRED] — the turn-end signal seam (the B-37/J-64 turn-end RIDER for external vendor CLIs).

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md; workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C1.2 + C1.3; the firstmate/gnhf interface maps workflow/papers/{firstmate,gnhf}-interface.md.
2. .claude/skills/auto-orchestrate/SKILL.md + .claude/skills/orchestrate/SKILL.md + protocol.md (the supervision loop); .claude/rules/bubble-up.md; E-39 cost-ceiling module; SESSION_HANDOFF / PreCompact hook files.
3. J-0 JD-1..JD-4 (harness legs have no SSB — evidence = hook-log/state.json); F-0 W-1..W-10; CLAUDE.md (TD-3 LLM-advisory, worker-git-read-only, no .unwrap on prod).

## Tasks (WAVE KAT-1 — harden the supervision loop)
1. Process-wedge detector (~50 LOC, harness): an output-signature (size:mtime + content-hash of the worker's last-output region) compared across N supervision cycles; identical for N cycles with no busy-signature = WEDGE → escalate via the existing T4 surface. Deterministic; no LLM. JD-1 evidence = the hook-log row.
2. Durable wake-queue (~60 LOC, harness): persist the supervision wake set so a supervisor crash resumes the loop (compose SESSION_HANDOFF + a singleton lock + signal coalescing). Replaces the naive dep-wait sleep-poll's volatility; the poll stays the fallback transport.
3. Mid-loop deterministic cost cap (~50 LOC, runtime): wire E-39's ceiling into the task_runner drive loop as a mid-step token/iteration/wallclock check that aborts the CURRENT step (AbortController-equivalent), distinct from the dispatch-count envelope. Deterministic predicate ONLY — never an LLM stop judgment (TD-3).
4. Record the riders in-place (no new files): gnhf commit-repair + backoff failure-taxonomy → .claude/rules/bubble-up.md; turn-end completion signal (claude Stop-hook touches a file) → B-37 / J-64 when driving external vendor CLIs.
5. workflow/reasoning/J-81-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - .claude/tools/ (or rag/hooks) — the process-wedge output-signature detector + the durable wake-queue (harness, JD-1 evidence)
  - crates/orchestrator/src/task_runner.rs — the mid-loop deterministic cost/iteration/wallclock cap wiring E-39's ceiling (the missing task_runner caller)
consumes_symbols:
  - auto-orchestrate/orchestrate supervision envelope + T4 surface (escalation target); SESSION_HANDOFF/PreCompact (wake-queue durability)
  - E-39 COST ECONOMICS HARD CEILING (velocity/automix/cost_router — WIRE its existing ceiling, never re-implement); bubble-up.md retry budgets
reachable_from:
  - the orchestrate / auto-orchestrate supervision loop → wedge detector + wake-queue (PRODUCTION supervision; JD-1 hook-log evidence)
  - task_runner drive loop → mid-loop cost cap (PRODUCTION execution; the E-39 caller that did not exist)
  - a #[cfg(test)] / contracts/ caller NEVER satisfies the runtime leg (R7)
stub_ban:
  - NO LLM-self-judged stop (gnhf stopWhen / should_fully_stop) — the stop predicate is deterministic/observable (TD-3)
  - NO `git reset --hard` rollback between iterations (H-49 CAS / RV-1 instead); NO `--dangerously-*` permission bypass
  - NO firstmate-style hierarchical autonomous supervisor (secondmates) — central FSM gate stays the trust spine
  - NO re-implementation of E-39 (wire its existing ceiling); NO .unwrap()/.expect() on prod paths
frontend_pair:
  N/A with justification — supervision legs are harness/runtime; the wedge/abort events surface through the existing T4 escalation + the cost panels (H-9/CloudPanel) the cost cap already feeds. Justified.
observable_effect:
  trigger: "run a worker that emits no output for N cycles (integration smoke, R10); crash the supervisor mid-loop; run a step that exceeds the token/wallclock cap"
  expect_ssb: "harness legs = JD-1 hook-log/state.json rows (wedge flagged, wake-queue resumed); the cost-cap leg = E-39 ceiling rows (reuse; no new action_type unless E-39 needs one — decide at edit from LIVE ~267/577, W-9)"
  expect_artifact: "the no-output worker is flagged WEDGE and escalated (not left to burn the window); a supervisor restart resumes the wake set; NEGATIVE arm: a runaway step exceeding the cap MUST abort the current step (not only fail the next dispatch-count check), and an LLM-self-judged stop MUST NOT be the gate"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-81_autonomous_supervision_hardening.md --run-trigger
rg "wedge|output.*signature|wake.queue|unchanged.*cycles" .claude/tools/ -l            # wedge detector + wake-queue present (harness)
rg "mid.loop|abort.*step|cost.*ceiling|E-39|wallclock" .elai_cc/crates/orchestrator/src/task_runner.rs  # the E-39 task_runner caller
rg -ci "should_fully_stop|llm.*decides.*stop" .claude/skills/auto-orchestrate/ .elai_cc/crates/orchestrator/src/task_runner.rs  # MUST be 0 (no LLM-judged stop, TD-3)
test -f workflow/reasoning/J-81-decisions.md
```

## ACCEPTANCE
- [ ] process-wedge: an output-unchanged-N-cycles worker is flagged and escalated via T4 (deterministic).
- [ ] durable wake-queue: the supervision loop resumes after a supervisor crash (composes SESSION_HANDOFF).
- [ ] mid-loop cap: E-39's ceiling now has a task_runner caller that aborts the current step on token/iteration/wallclock breach.
- [ ] TD-3: the stop predicate is deterministic; no LLM-self-judged stop anywhere.
- [ ] DO-NOT-COPY honored: no reset-hard, no `--dangerously-*`, no secondmates.
- [ ] JD-1: harness legs leave hook-log/state.json evidence; cargo check --workspace GREEN; auditor PASS on -p orchestrator.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the wedge/crash/cap integration smoke run-trigger (not cargo-green / file-exists).
- W-2: the cost cap rides E-39's existing flag; new harness legs default-ON behind a *_DISABLE kill-switch.
- W-3: extends the live supervision loop + wires DARK E-39, not a summary-only port of firstmate.
- W-4: frontend_pair N/A (surfaces via T4 + cost panels); justified.
- W-5/W-10: in-process + harness; no network leg (dep-wait poll stays the fallback transport).
- W-6: N/A (no new dashboard surface); justified.
- W-7: every anchor re-grepped at edit.
- W-9: cost-cap SSB prefer-reuse E-39 rows; harness legs have no SSB (JD-1).
- GD-5: the wedge/abort decisions are deterministic and journaled/logged; no silent kill.

## Tests to add
- crates/contracts/tests/j81_supervision_hardening_contract.rs (~LOC, runtime cost-cap leg) + a harness smoke (JD-1): (a) the mid-loop cap aborts a step exceeding the token budget; (b) the wallclock cap fires deterministically; (c) the stop predicate rejects an LLM should_fully_stop signal (it is never the gate); (d) [harness] the wedge detector flags output-unchanged-N-cycles; (e) [harness] the wake-queue resumes after a simulated supervisor restart.

## Files touched
- CREATE: .claude/tools/supervision_wedge.py + wake_queue.py (~110 LOC, stdlib, JD-1)
- MODIFY: .claude/skills/auto-orchestrate/SKILL.md + orchestrate/protocol.md (wire the detector + wake-queue into the loop) (~30 LOC prose)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (the E-39 mid-loop cost-cap caller) (~50 LOC)
- MODIFY: .claude/rules/bubble-up.md (gnhf commit-repair + backoff-taxonomy rider) (~10 LOC)
- CREATE: crates/contracts/tests/j81_supervision_hardening_contract.rs (~60 LOC)
- CREATE: workflow/reasoning/J-81-decisions.md

## Citations / Grounding
- steal-kunchen-agentic-tools-2026-06-24.md C1.2 (gnhf RunLimits orchestrator.ts:74-79; mid-iteration onUsage→AbortController; stopWhen=DO-NOT-COPY-AS-GATE TD-3; reset-hard DO-NOT-COPY) + C1.3 (firstmate fm-watch.sh:130-264 stale/wake-queue/coalescing; secondmates OFF-WEDGE; `--dangerously-*` DO-NOT-COPY) + SYNTHESIS TIER-1(3)/TIER-2(5). firstmate/gnhf are macOS/Node — Rust+harness reimpl, no code copied; pnpm/Node-20 noted (bun-not-node for any JS).
- ELAI ground truth: auto-orchestrate/orchestrate supervision envelope; anti-spiral phase-3C; E-39 (DARK, no task_runner caller); SESSION_HANDOFF/PreCompact; bubble-up.md (re-grepped at edit, W-7).
- CLAUDE.md: TD-3 (LLM advisory not the gate), worker-git-read-only, no .unwrap on prod, the 2026-06-11 reset-hard incident.

## Open questions / risks
- "output-unchanged" definition for ELAI's Agent-tool dispatch (which returns on completion, harness-notified) vs an EXTERNAL vendor-CLI fleet (B-37/J-64) where the wedge risk is real: scope the detector to the external-CLI / long-running-loop case where output is the only liveness signal; ELAI's own Agent dispatch already has completion notification. Decide at authoring.
- E-39 wiring is the load-bearing runtime change; verify the ceiling's existing types are the right seam before adding the caller (it is currently DARK with none). If E-39's shape is wrong, that is a material fork (STOP + handoff), not a silent re-implement.
- The wake-queue must not become a second source of truth vs SESSION_HANDOFF; make it a projection/extension of the handoff, not a parallel store.

## Next
J-81 hardens autonomous supervision against the silent-hang + supervisor-crash + runaway-cost failure modes, deterministically, and finally gives E-39 a production caller. Pairs with J-80 (isolated workers the supervisor watches) and orchestrate/auto-orchestrate (the loops it hardens). The firstmate trust-light foil (secondmates, `--dangerously-*`) stays explicitly refused.

End of J-81.
