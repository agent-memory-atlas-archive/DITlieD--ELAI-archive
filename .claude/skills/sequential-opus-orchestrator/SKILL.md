---
name: sequential-opus-orchestrator
description: Sequential single-Opus backlog walk over the ELAI plan corpus. Triggers when the user says "resume the opus walk", "opus walk", "continue the backlog walk", or "start the opus backlog walk". One paid Opus orchestrates FREE workers (kimi-k2.7 reads, glm-5.2 implements) to drain .elai_cc plans; a deterministic dispatch budget stops the session at a unit boundary and a FRESH Opus resumes from walk-state + handoff. Never two Opus concurrent.
---

sequential-opus-orchestrator — the Opus chain controller

Plan of record: workflow/reasoning/TEIKOKU-OPUS-BACKLOG-WALK-PLAN.md (LOCKED D1-D6; do not re-open).
State spine: workflow/orchestrator-walk/walk_state.py + walk-state.json (this skill is its production caller).
Handoff: workflow/handoff/opus-backlog-walk.md per .claude/rules/handoff-protocol.md.
Binding rules (pointers, not restatements): .claude/rules/dispatch-core.md (D-1..D-10), .claude/rules/agent-dispatch-preamble.md (inject on EVERY worker), .claude/rules/orchestrate-protocol.md (H-1..H-11 bind this controller), .claude/rules/wiring-enforcement.md (R1-R10), .claude/rules/production-caller-gate.md (D-030), .claude/rules/plan-standard.md, .claude/rules/proof-spine.md (queue order), .claude/skills/anti-spiral (stagnation), .claude/rules/commit-cadence.md (overridden ONLY by block 7 below).

HARD INVARIANTS (from the plan's locked decisions)
- D1 exactly ONE Opus: step 1 of the resume protocol is walk_state.claim_lock(session_id); LockRefused = ABORT immediately with "another Opus holds the walk" — never proceed, never delete the lock by hand.
- D2 Opus never authors product code and never dictates exact diffs. Blocked executor = escalation ladder (block 4), never "I'll write it myself". Harness tooling (this skill, walk_state.py fixes) is exempt; .elai_cc code is not.
- D6 all product output lands in .elai_cc/, committed AND pushed per drained plan (block 7). Harness tooling stays local (gitignored), never pushed.

1. RESUME PROTOCOL (every fresh Opus boot, in order; zero plan-file reads before step 5)
  1 claim_lock: python -c "import sys; sys.path.insert(0,'workflow/orchestrator-walk'); import walk_state; walk_state.claim_lock('<session-id>')" — LockRefused = ABORT (D1).
  2 read state: walk-state.json (whole file, it is small) + handoff HEAD only (first ~40 lines: CURRENT STATUS + INDEX). NEVER the handoff body, NEVER plan files, NEVER memory-bank.
  3 preflight (all four must pass before any dispatch):
    a python .claude/tools/gate_freeze_check.py           (a run that can edit its gates writes itself to PASS; also catches the blake3-vs-sha3 env-drift class)
    b python .claude/tools/elai_status_query.py --max-age-min 180   (exit 3 = stale: refresh via the feeder before triage)
    c git -C /workspace/ELAI status --porcelain          (dirty tree = reconcile first: a half-applied worker edit from a crashed session is DIAGNOSED via reader agents, never reset)
    d bridge health: curl the lane proxy /v1/models (lane ports in block 3; glm-5-2 AND kimi-k2-7 present, else plan Phase-0 FORK-A/ABORT applies)
  4 reconcile cursor: git log + the frozen checkers BEAT walk-state. A DONE row whose commit_sha is not in git log is a LIE: strip it, re-precheck that plan. cursor.pickup names the exact next action; trust it only after this reconcile.
  5 continue the per-plan loop (block 2) from cursor; walk_state.record_dispatch() before EVERY worker dispatch; walk_state.heartbeat() at least every stage boundary.

2. PER-PLAN LOOP (reuses the S-stage prompt layer at .claude/skills/glm-orchestrate/prompts/ — see its README.md for the evidence chain; substitute evidence_ledger_path=workflow/orchestrator-walk/evidence/{plan_id}.json and plan-status dir=workflow/orchestrator-walk/plan-status/)
  - queue order: PSR-1 precedence (proof-spine.md); take cursor plan first, else queue head.
  - S1 context + S2 integration (paired, kimi) -> S3 precheck-route (NON-SKIPPABLE, prompts/precheck-route.md, wiring agents in mode=precheck):
      wired=yes    -> POLISH route: S9 -> S7 done-gate -> S10 commit. No rebuild.
      wired=partial|no -> class the work: recognize/activation -> EXECUTE route S4 -> S5 -> S6(+S6b) -> S7 -> S8 -> S9 -> S10.
                          greenfield -> escalation_queue, SET_ASIDE, move on (fail-closed: the 0-commit experiment was blanket-greenfield on the free lane; R2 in the plan).
  - the frozen checkers (production_caller_check.py / python_wire_check.py / wiring_contract_check.py / maintainability_check.py / plan_gate_check.py) run OFF-AGENT, shelled by the controller or the S6 auditor — never self-run-as-authority by the executor (its self-run is a courtesy).
  - S11 memory once per drained PLAN, after S10.

3. MODEL MAP + DISPATCH PRIMITIVE (explicit D-2 override, cross-family judgment per the D-13 adversary principle)
  - kimi-k2.7 (262k ctx): S1 context-compiler, S2 integration-explorer-elai, spec-grounder reads, S8 hostile review (cross-family: kimi judges glm's work, never glm-audits-glm).
  - glm-5.2 (200k ctx): S5 executor, S6b fixer.
  - deterministic checkers: any lane, off-agent (block 2).
  - the controller: the ONE Opus (this session). Never spawn a second Opus/Fable worker.
  - dispatch primitive: python workflow/glm-orchestrator/glm_dispatch.py (prompt over STDIN, STATUS-line outcome classes incl. INFRA/RATE_LIMITED/CONTENT_POLICY; INFRA never burns an attempt). Lanes a..d -> Windows-side proxy ports 8814/8811/8812/8813 (8810 is permanently untrustworthy Windows-side — a WSL TEIKOKU proxy shadows it; glm-worker.ps1 header has the incident). kimi dispatch = same delegate chain, claudedevin.ps1 -Model kimi-k2-7 on its OWN CLAUDEDEVIN_PORT (never share a live glm lane port). The harness Agent tool CANNOT route glm/kimi (verified 2026-07-06: model unresolvable) — always dispatch via the bridge scripts.
  - every worker prompt starts with the agent-dispatch-preamble pointer line (D-1) + its S-stage template; workers run CLAUDE_CODE_DISABLE_CLAUDE_MDS=1, so the template carries every rule they get.
  - codex gpt-5.5 (bmapi proxy; THIRD family, cross-family from BOTH glm and kimi): the E2E TESTER lane — see block 8. Dispatch: wsl.exe -d codex-vm -u root (mount guard: mountpoint -q /mnt/f || mount -t drvfs F: /mnt/f) then `codex exec --profile account-a --json -s workspace-write -` with the task over stdin (proven shape, conductor worker.py). Credits EXPIRE 2026-07-15; after expiry or POOL exhaustion the tester lane falls back to kimi-k2.7 (still cross-family vs the glm executor). The same proxy is a first-class ELAI provider in .elai_cc (Gpt55Proxy, commit d91ec95) for in-product runs.

4. ESCALATION LADDER (D-9: a third identical dispatch is BANNED)
  attempt 1: S5 executor -> gates fail -> ONE guided retry: S6b fixer fed the checker's OWN json output (the proven M8 pattern), never a blind re-run.
  second fail -> Opus forks, exactly two legal moves:
    (a) re-decompose into a THINNER vertical slice (producer+consumer+smoke still in one unit, wiring-enforcement PRE_GATE) and run ONE cycle on the new decomposition;
    (b) walk_state append to escalation_queue {plan_id, reason, attempts, gate_json_ref}, SET_ASIDE, move to the next plan (precedent: the ancestor set aside B-4/B-5 after 2 real failures).
  3 consecutive plans ending BLOCKED/ESCALATED -> HALT the session + surface to user (plan Phase-4 ABORT; mirrors the ancestor's 3x-halt).
  anti-spiral triggers (same file >3 edits same issue, BLOCK twice in a row) -> .claude/skills/anti-spiral.

5. CONCURRENCY POLICY (D4, measured 2026-07-06: free ceiling 4, 5th slot queues)
  drive up to 3 workers; ONE observed 429/RATE_LIMITED -> drop to 2 for the rest of the session; 1 bridge slot stays reserved for the research machine. Batch respects dispatch-core D-3 (max 3 per message). S5 executors are SERIAL per plan (a second executor starts only from the first's handoff entry).

6. STOP PROTOCOL (deterministic budget, never Opus token self-estimation)
  trip = budget.dispatch_cap OR budget.plans_per_session_cap, whichever first (walk_state.record_dispatch/mark_plan_done return tripped=True).
  On trip: WARN in-channel; finish ONLY the current unit (never start a new S-stage); then in order:
    1 walk_state.update_cursor(plan_id, stage, pickup=exact next action, one line)
    2 walk_state.session_rollover(session_id) — increments sessions_completed AND zeroes dispatches_this_session/plans_this_session so the NEXT session starts a fresh budget window (never hand-edit the json; without this reset the first tripped cap wedges every later session)
    3 append handoff entry + overwrite CURRENT STATUS (handoff-protocol.md entry shape)
    4 commit+push IF a plan closed since the last commit (block 7)
    5 walk_state.release_lock(session_id)
    6 final message = the warning + the ONE resume command: "resume the opus walk"
  PreCompact hook = crash backstop only, never the primary stop.

7. COMMIT CADENCE (D6 override of commit-cadence.md push-at-end; user-authorized in the plan of record)
  one commit per DRAINED PLAN via the S10 committer (prompts/S10-commit.md), PUSHED immediately — a dead Opus loses at most one plan's coordination.
  MANDATORY committer hardening (the recorded B-3/B-15 doc-only false-pass bug): before commit, `git diff --cached --name-only` — ANY staged path outside .elai_cc/ = unstage and REFUSE; the committer returns BLOCKED, never "committed anyway".
  maintainer-voice message rules per commit-cadence.md; mark_plan_done(plan_id, commit_sha, gate_evidence_ref) ONLY after the push succeeds (DoneRowRejected guards the evidence fields).

8. INTEGRATION SMOKE CADENCE (wiring-enforcement R10 / H-11) — THE CODEX TESTER LANE
  after every walk-state.integration_smoke.cadence_plans (=3) drained plans AND before any chain checkpoint report: an INDEPENDENT cross-family agent drives the REAL .elai_cc entrypoint touched by those plans on a HARD input (strict peer, multi-turn, long enough to fire secondary paths). Toy happy-path smokes do not count. Record in walk-state.integration_smoke.last_after_plan.
  DIVISION OF LABOR (R8: the executor never authors its own gate; the tester is codex gpt-5.5 = a third family, so neither the glm author nor the kimi reviewer judges itself):
    - codex (the tester BRAIN, on codex-vm): reads the drained plans' evidence ledgers + diffs, AUTHORS the e2e scenario (which frontend entrypoint, which hard input, which off-agent artifact must land — an SSB row / checker exit-0 / CDP-observed state), and JUDGES the collected evidence pass|fail per scenario step, returning a structured verdict.
    - the controller (the hands, on Windows): executes the scenario's deterministic legs against the FULL WIRED PATH — frontend entrypoints first-class: .claude/tools/visual/ui-wire-smoke.mjs (bun+CDP real-click -> NEW Rust-owned SSB row, baseline-delta, exit 2 fail-closed), the CDP capture harness, svelte_mount_reachability/tauri_ipc_symmetry checkers, and real CLI entrypoints (elai run / elai.bat) — then feeds raw outputs back to codex verbatim.
    - a codex verdict of fail on any step = the smoke FAILED: the offending plan goes to escalation_queue and the chain does NOT advance past the checkpoint (missing evidence = FAIL, R1).
  Per-plan option (cheap, in addition to the cadence smoke): on any plan that touched dashboard-v2 or a CLI surface, a single codex tester episode may run as the S7 done-gate companion (tester-dispatch.md pattern) before S10 commit. Same division of labor; budget one episode (~10K-token overhead measured 2026-07-06 on the conductor smoke).
