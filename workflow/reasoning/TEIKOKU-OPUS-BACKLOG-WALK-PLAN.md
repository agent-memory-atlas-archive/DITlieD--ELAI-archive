TEIKOKU-OPUS-BACKLOG-WALK-PLAN

wargame-grade build plan. authored 2026-07-06. author = main-LLM (Opus). red-team + 10-point verdict owed to an auditor-class agent that did NOT author this (plan-standard.md role boundary). deterministic leg: python .claude/tools/plan_gate_check.py --file workflow/reasoning/TEIKOKU-OPUS-BACKLOG-WALK-PLAN.md

PROOF_SPINE: feeds PS-6 (wire-dark rate, longitudinal pre/post-stack) — a supervised Opus walk that ships wired-or-escalated slices is the cheapest instrument to move the wire-dark number without new build; secondary feed PS-2 (composed gate-tax) via measured drain-under-frozen-gates.

================================================================================
0. WHAT THIS IS (one paragraph)
================================================================================
A single paid Opus, run sequentially, orchestrates FREE workers to drain the ELAI plan backlog. Opus never writes product code; it dispatches workers, reads their evidence, decides, adapts, and can decompose a plan into worker-sized slices. Workers: kimi-k2.7 (262k ctx) for exploration/reading, glm-5.2 (200k ctx) for implementation + fix. Exactly ONE Opus at a time. It runs until a deterministic dispatch budget trips, warns, stops at a unit boundary, writes a handoff + walk-state, and a FRESH Opus resumes from those two artifacts and continues — a chain, never two Opus concurrent. This is NOT a greenfield build: the deterministic ancestor workflow/glm-orchestrator/ already landed 70+ commits on this same corpus under the same frozen gates via precheck-routing; this plan swaps its Python control loop for an Opus loop and adds the resume chain, reusing its state files, S1-S11 worker prompts, checkers, and agents.

LOCKED DECISIONS (user, 2026-07-06, do NOT re-open):
D1 exactly one Opus orchestrator at a time; never two.
D2 Opus orchestrates only; it never authors product code and never dictates the exact diff to a worker (authorship-with-extra-steps is banned — it poisons execution/evaluation separation).
D3 workers: kimi-k2.7 = exploration/read; glm-5.2 = implementation/fix. both free via the devin/codeium bridge, confirmed live 2026-07-06 despite the finish-plan D4 "dead" note.
D4 concurrency: bridge free ceiling = 4; reserve 1 for the research machine (now 1 lane); Opus drives up to 3; on a rate-limit signal Opus drops to 2 (a single boot-prompt line, no code).
D5 sequential handoff loop: dispatch-budget trip -> warn -> stop at a unit boundary -> write handoff + walk-state -> fresh Opus resumes -> loop until backlog drained.
D6 all BACKLOG-DRAIN output lands in .elai_cc/, committed AND pushed after every drained plan. The orchestrator tooling itself (skill, walk-state, this plan) lives in the gitignored harness (.gitignore tracks .elai_cc/ ONLY) and is local, never pushed — the honest split, stated so no session re-litigates it.

================================================================================
1. GROUNDING LEDGER (what is settled vs what a boot session must settle)
================================================================================
GROUNDED (verified this session or in cited files):
- workflow/glm-orchestrator/ exists with orchestrator.py, plan_walk.py, plan_parser.py, run-state.json, plan-status/{id}.json, evidence/{id}.json, S1-S11 prompts, watchdog scripts. It landed 70+ commits via precheck-routing; 8 genuine real-code B-plan passes (glm-orchestrator.md CURRENT STATUS).
- Blanket free-lane greenfield through teikoku landed 0 INTEGRATION_PUSHED; 37/40 stuck = genuine gate_fail = the free-model capability wall (teikoku-glm-orchestrator-merge.md; TEIKOKU-ELAI-FINISH-PLAN.md L47-53, L123-125). Same gates, opposite outcome: the variable is job shape, not gate strictness.
- .claude/agents/executor.md is already authored for glm-5.2 and expects a curated context packet + self-runs the wire checkers + returns DONE-CANDIDATE (not DONE).
- Frozen deterministic checkers exist and are auditor-owned: production_caller_check.py (.rs), python_wire_check.py (.py), wiring_contract_check.py, plan_gate_check.py, maintainability_check.py, gate_freeze_check.py, tauri_ipc_symmetry_check.py, svelte_mount_reachability.py.
- Triage tools: elai_status_query.py --punch (all letters), bn_status_report.py (B family), proof_spine_status.py (PSR-1 order).
- .gitignore tracks .elai_cc/ only (read 2026-07-06): harness tooling is local, .elai_cc output is pushable. (D6.)

RECON_NEEDED (a boot session settles these BEFORE any walk work; each names its exact check):
RECON_NEEDED bridge serves glm-5.2 + kimi-k2.7 today AND the free ceiling is 4 — check: curl the bridge /v1/models (devin_proxy on its port) and dispatch 4 concurrent trivial workers, observe the 5th 429/queue. Settles D3+D4.
RECON_NEEDED .claude/skills/glm-orchestrate/ currently holds ONLY guardrail.md + prompts/ (its SKILL.md is missing per Fable survey) — check: ls .claude/skills/glm-orchestrate/. Settles whether Phase 2 ABSORBS a half-skill vs authors fresh.
RECON_NEEDED the S1-S11 prompt contract in .claude/skills/glm-orchestrate/prompts/ still matches evidence/{id}.json shape — check: read precheck-route.md + one S-stage prompt + one evidence/*.json. Settles reuse-vs-rewrite of the worker prompt layer.
RECON_NEEDED PreCompact hook is registered + fires (C:/Users/user/.claude/rag/hooks/pre_compact_check.py in .claude/settings.local.json) — check: grep settings.local.json for pre_compact. Settles the secondary stop backstop.

================================================================================
2. PHASES (each a vertical slice; markers per plan-standard.md)
================================================================================

--------------------------------------------------------------------------------
PHASE 0 — BOOT PROBE + WORKFORCE/CONCURRENCY LOCK-IN
--------------------------------------------------------------------------------
Move 0.1 probe the bridge and lock the concurrency policy.
EXPECTED_OBSERVATION: /v1/models lists glm-5.2 and kimi-k2.7; 4 concurrent trivial dispatches all return; a 5th 429s or queues; the policy line "drive 3, drop to 2 on 429, 1 reserved for research" is recorded in walk-state.budget.
ON_FAIL: glm-5.2 or kimi absent from /v1/models -> CAUSE the finish-plan D4 window did close after all -> COUNTER-MOVE take FORK-A (swap free implementer to swe-1-6, run the measured probe below before any walk).
FORK: if glm-5.2 present -> Route-Live (D3 stands, proceed to Phase 1). If glm-5.2 ABSENT but swe-1-6 present -> FORK-A Route-SWE: substitute swe-1-6 as the free implementer, and because its activation-class capability is "assumed not measured" (finish plan L239-240), gate the walk behind a 5-task measured probe (5 activation-class plans; require >=3 gate-passes) before opening the drain. If BOTH absent -> ABORT.
RECON_NEEDED (settled here): the two D3/D4 recon lines in section 1.
ABORT: no free implementer model served by the bridge -> STOP, surface to user, do not fall back to paid workers silently (that would burn the Max plan on execution, defeating the design).
VERIFY: curl <bridge>/v1/models | grep -E 'glm-5-2|kimi-k2-7'; a 4-then-1 concurrent dispatch script observes the 5th queue/429; off-agent artifact = the recorded models list + the observed 5th-slot backpressure, pasted into walk-state.notes.

--------------------------------------------------------------------------------
PHASE 1 — WALK-STATE SPINE + SINGLE-OPUS LOCK
--------------------------------------------------------------------------------
WIRING_CONTRACT (Phase 1 code unit):
  produces: workflow/orchestrator-walk/walk-state.json (schema below) + a stdlib-only helper walk_state.py (read/claim-lock/update-cursor/append-decision/mark-done-with-evidence) OR an extension of the existing workflow/glm-orchestrator state helpers if RECON shows they fit.
  consumes: the helper is CALLED by the Phase 2 skill's resume protocol (step 2) and stop protocol (step N). That call site IS the production caller — a walk-state helper with no skill caller is wire-dark and fails this phase.
  reachable_from: the sequential-opus-orchestrator skill boot + stop paths (Phase 2).
  observable_effect: a fresh process reading walk-state.json recovers cursor/queue/budget without reading any plan file.
  stub_ban: no placeholder DONE rows; a DONE row missing commit_sha or gate_evidence_ref is rejected by the helper at write time.

walk-state.json schema (reuse the existing run-state.json + plan-status/ + evidence/ split; do NOT invent a third store):
  schema_version, walk_id
  lock {session_id, pid, heartbeat_iso}
  budget {dispatches_this_session, dispatch_cap(=30), plans_this_session, plans_per_session_cap(=3), sessions_completed, concurrency(=3), degrade_on_429_to(=2)}
  cursor {plan_id, stage, pickup (one exact next-action line)}
  queue [ordered plan ids, PSR-1 precedence]
  escalation_queue [{plan_id, reason, attempts, gate_json_ref}]
  decisions [{id, date, what, why, evidence}]
  integration_smoke {last_after_plan, cadence_plans(=3)}
Per-plan rows reuse plan-status/{id}.json verbatim ({plan_id,file,wired,route,gate,status,last_run_iso,executor_lane,notes}) + {class:recognize|activation|greenfield, attempts, commit_sha, evidence_ledger}.

Move 1.1 author the helper + schema; wire the lock with heartbeat.
EXPECTED_OBSERVATION: walk_state.py claim_lock() writes lock with a fresh heartbeat_iso; a second claim while heartbeat < 30min old and session_id differs returns REFUSED (the D1 single-Opus invariant, enforced in code not prose).
ON_FAIL: two sessions both acquire the lock -> CAUSE heartbeat staleness window wrong or no atomic write -> COUNTER-MOVE atomic write via tmp+rename, and treat any heartbeat < 30min from a different session as a hard REFUSE.
FORK: reuse the existing glm-orchestrator helper if RECON 1.x shows its run-state shape already carries a lock -> extend it; else -> new walk_state.py. Decide off the RECON read, not by guess.
RECON_NEEDED: does workflow/glm-orchestrator/ already have a lock/heartbeat field — check: grep -ri 'lock\|heartbeat' workflow/glm-orchestrator/*.py. If yes, extend; if no, add.
ABORT: if the existing glm-orchestrator state files are actively written by a running deterministic walk (a live process) -> STOP, do not co-write the same files; branch to a fresh workflow/orchestrator-walk/ store.
VERIFY: python -c "import walk_state; walk_state.claim_lock('s-a'); assert walk_state.claim_lock('s-b') is REFUSED"; off-agent artifact = the REFUSED return + the DONE-row-without-commit_sha rejection, both asserted in a tiny stdlib test.
INTEGRATION_SMOKE: n/a (single unit; the mechanism smoke is Phase 3).

--------------------------------------------------------------------------------
PHASE 2 — THE sequential-opus-orchestrator SKILL (boot instruction + reuse map)
--------------------------------------------------------------------------------
WIRING_CONTRACT (Phase 2 code unit):
  produces: .claude/skills/sequential-opus-orchestrator/SKILL.md (~150 lines, mostly pointers) + its command trigger.
  consumes: it is FIRED by the user trigger and READ at every fresh-Opus boot; it CALLS walk_state.py (Phase 1), the triage tools, the existing agents, and the frozen checkers. Its being fired IS its production caller.
  reachable_from: user types the trigger / a launcher invokes claude -p with the skill.
  observable_effect: a fresh Opus, given only the trigger, executes the resume protocol and continues the walk with zero re-reading of plan files.
  stub_ban: no "TODO: fill escalation ladder" — the ladder, stop protocol, and model map ship complete or the phase is not done.

The SKILL must contain (reuse-first; pointers not restatements — token-discipline):
- TRIGGER contract + the 5-step RESUME PROTOCOL: (1) claim_lock or ABORT on the single-Opus invariant; (2) read walk-state.json + handoff HEAD (~40 lines only, never the body, never plan files); (3) preflight = gate_freeze_check.py (the blake3-vs-sha3 env-drift class wedged run_gates for a full day — teikoku-elai-finish) + elai_status_query.py --max-age-min 180 + git status clean + bridge health; (4) reconcile the cursor plan: git + live checkers BEAT walk-state, a DONE row without a verified commit_sha is re-prechecked; (5) continue the loop from cursor.
- PER-PLAN LOOP: precheck-route (NON-SKIPPABLE) -> class routing -> S-stage dispatch by role. precheck reuses .claude/skills/glm-orchestrate/prompts/precheck-route.md.
- MODEL MAP as an explicit D-2-override (dispatch-core.md D-2 defaults canonical agents to sonnet; this walk overrides): kimi-k2.7 -> context-compiler / integration-explorer-elai / spec-grounder reads; glm-5.2 -> executor (S5) + fixer (S6b); deterministic checkers -> any lane, off-agent; the QUALITATIVE hostile-review leg -> kimi-k2.7 (cross-family per D-13 adversary principle, NOT glm-audits-glm); the controller -> the one Opus. This refines the raw "glm for audit" spec: glm still implements, the frozen checkers still gate off-agent, but the judgment leg goes cross-family — a strict improvement that keeps audit intact.
- ESCALATION LADDER: attempt-1 executor -> gate fail -> ONE guided retry off the checker's own JSON via the fixer (proven M8 pattern, not a blind re-run) -> second fail: D-9 bans a third identical dispatch -> Opus FORKS: (a) re-decompose into a thinner vertical slice and run one cycle on the new decomposition, OR (b) reclassify to escalation_queue with reason + gate refs, mark SET_ASIDE (precedent: glm-orchestrator set aside B-4/B-5 after 2 real failures), move on. Opus NEVER writes the code (D2).
- CONCURRENCY POLICY (D4): drive 3 workers; on a 429/rate-limit observation drop to 2 for the rest of the session; 1 slot reserved for research; the dispatch batch respects dispatch-core.md D-3 cap 3.
- STOP PROTOCOL: deterministic dispatch-budget trip (dispatch_cap OR plans_per_session_cap, whichever first) — NOT Opus self-estimating tokens (it cannot reliably); stop only at a unit boundary; PreCompact hook is the crash backstop only. On stop: update walk-state cursor/budget, release lock, append handoff entry + overwrite CURRENT STATUS, commit+push if a plan closed, final message = warning + the one resume command.
- COMMIT CADENCE (D6 override of commit-cadence.md push-at-end): one commit per drained plan via committer.md, PUSHED immediately (crash tolerance: a dead Opus loses at most one plan's coordination). MANDATORY committer hardening: refuse any staged path outside .elai_cc/ (the recorded B-3/B-15 doc-only false-pass bug).
- POINTERS (not restatements) to: agent-dispatch-preamble.md (D-1 rule injection on every worker), dispatch-core.md, orchestrate-protocol.md (H-1..H-11 bind the controller), handoff-protocol.md, plan-standard.md, wiring-enforcement.md, production-caller-gate.md, anti-spiral skill, proof-spine.md (queue order).

Move 2.1 author the skill; absorb the orphaned glm-orchestrate half-skill.
EXPECTED_OBSERVATION: the SKILL.md contains all eight blocks above with no placeholder; the model map, escalation ladder, and stop protocol are complete prose; a pointer (not a copy) resolves each cited rule.
ON_FAIL: the skill restates dispatch-core/handoff content inline -> CAUSE copy-instead-of-pointer -> COUNTER-MOVE replace the restated block with a one-line pointer (redundancy between files is a bug, token-discipline).
FORK: if RECON shows the glm-orchestrate SKILL.md still exists -> merge+deprecate it; if gone -> author fresh and delete the orphaned prompts/ only if unused by the new skill (they are reused, so KEEP prompts/).
RECON_NEEDED: settled by the section-1 glm-orchestrate recon lines.
ABORT: if authoring reveals the S1-S11 evidence contract has drifted from evidence/{id}.json -> STOP, do not ship a skill that dispatches against a stale contract; fix the contract mapping first (a named sub-task), then resume.
VERIFY: a dry-run boot — a throwaway Opus reads only the trigger + walk-state + handoff head and correctly names the next plan + stage + first worker dispatch WITHOUT opening a plan file; off-agent artifact = that dry-run transcript showing zero plan-file Reads before the first dispatch decision.
INTEGRATION_SMOKE: n/a here (the resume mechanism is smoked in Phase 3).

--------------------------------------------------------------------------------
PHASE 3 — RESUME-CHAIN PROOF (the mechanism integration smoke)
--------------------------------------------------------------------------------
Move 3.1 drive one real .elai_cc plan through the Opus loop, force a budget stop mid-walk, hand off, resume with a FRESH Opus, continue to a commit+push.
EXPECTED_OBSERVATION: session A drains part of a real activation-class plan, trips a deliberately-low dispatch_cap, warns, writes handoff + walk-state, releases lock; session B (fresh context) claims the lock, resumes from cursor, finishes the plan, and produces one pushed .elai_cc commit — all with session B reading only walk-state + handoff head (verified: no plan-file Read in B before its first dispatch).
ON_FAIL: session B re-derives from scratch (re-reads the plan, re-runs precheck it already had) -> CAUSE the handoff/cursor under-specifies the pickup -> COUNTER-MOVE enrich cursor.pickup to the exact next action + ensure the per-plan evidence_ledger tail carries the completed-stage record.
FORK: FORKS: none (single deterministic mechanism test).
RECON_NEEDED: RECON_NEEDED which single real plan is the safest activation-class candidate for the proof — check: elai_status_query.py --punch | pick an activation-class, single-crate, low-blast-radius plan (impact_radius.py to confirm blast < 5 files). GROUNDED that such plans exist (activation majority per section 1).
ABORT: if the forced stop leaves the tree dirty (a half-applied worker edit) and session B cannot cleanly reconcile -> STOP, surface; the resume protocol must handle a dirty-tree pickup or it is not crash-tolerant (fix before Phase 4).
VERIFY: git log shows exactly one new .elai_cc commit authored across the A->B handoff; the auditor re-runs production_caller_check.py + python_wire_check.py off-agent on that commit (DARK with no WIRE-DARK row = the proof FAILED); off-agent artifact = the pushed commit sha + the two checker exit-0s + the B-session transcript proving no plan-file read.
INTEGRATION_SMOKE: THIS phase IS the mechanism smoke — an independent fresh Opus (session B) drives the REAL skill entrypoint on a REAL plan through a REAL handoff, not a toy. Cadence for the drain proper: integration_smoke every 3 drained plans (walk-state.integration_smoke.cadence_plans) + before any chain checkpoint report.

--------------------------------------------------------------------------------
PHASE 4 — FIRST MEASURED DRAIN WAVE (acceptance)
--------------------------------------------------------------------------------
Move 4.1 compose the first wave by precheck-route triage and run it under the full loop.
EXPECTED_OBSERVATION: N plans triaged into {recognize, activation, greenfield}; the recognize+activation set is driven to pushed .elai_cc commits; the greenfield set lands in escalation_queue (NOT force-fed to the free lane); a measured drain rate (plans-pushed / dispatches) and a wire-dark count (PS-6) are recorded.
ON_FAIL: the free lane is fed greenfield and wedges into an escalation pileup burning Opus supervision -> CAUSE precheck-route was made skippable or the greenfield->escalate rule fail-open -> COUNTER-MOVE make precheck-route non-skippable and greenfield->escalate fail-closed in the skill (Risk 2), re-run.
FORK: if the activation lane's gate-pass rate is < 50% across the first 10 attempts -> FORK-B pause the drain and surface: the free implementer is weaker than assumed (esp. under Route-SWE), the walk is not yet economical, escalate the wave to a paid decision. Else continue.
RECON_NEEDED: RECON_NEEDED the actual pass rate is unknown until measured — check: run the first 10 activation attempts, compute gate-pass rate from the evidence ledgers. This is the "assumed not measured" line made into a tripline.
ABORT: 3 consecutive plans ending BLOCKED/ESCALATED -> halt + surface (mirrors the glm-orchestrator 3x-halt); do not keep dispatching into a wall.
VERIFY: git log --oneline shows the pushed commits for the drained set; escalation_queue holds the greenfield set with reasons; off-agent artifact = the drain-rate + wire-dark numbers in walk-state.decisions, recomputable from git + the evidence ledgers.
INTEGRATION_SMOKE: after every 3 drained plans, an independent agent drives the real .elai_cc entrypoint touched by those plans on a HARD input (strict peer, multi-turn) — seam defects between separately-drained plans pass every per-plan gate and only surface here (wiring-enforcement R10 / H-11).

================================================================================
3. REDTEAM (attacks run against THIS plan; the auditor adds more)
================================================================================
REDTEAM attack-1 (SURVIVED): "two Opus sessions race the lock and both walk." Defence: Phase-1 atomic tmp+rename lock + 30-min heartbeat REFUSE, enforced in walk_state.py not prose; a watchdog that relaunches must claim_lock first. Holds.
REDTEAM attack-2 (SURVIVED): "the Nth Opus inherits a DONE row that was never committed and skips real work." Defence: DONE rows require commit_sha + gate_evidence_ref at write time; resume step-4 reconciles against git + live checkers, which BEAT walk-state; a bare DONE is re-prechecked. Holds.
REDTEAM attack-3 (PATCHED): "Opus, blocked, writes the diff itself to make progress." This would pass gates while destroying the execution/evaluation separation and the whole free-workforce premise. PATCH: D2 in the skill bans Opus authorship AND dictating exact diffs; the escalation ladder's only Opus move is re-decompose or set-aside. Encoded as a hard rule, not a preference.
REDTEAM attack-4 (PATCHED): "the committer stages a non-.elai_cc path and a doc-only change reads as a plan pass" (the real B-3/B-15 bug). PATCH: mandatory committer hardening — refuse any staged path outside .elai_cc/ — is a Phase-2 deliverable, not a later step.
REDTEAM attack-5 (SURVIVED via FORK): "the free implementer is actually swe-1-6, weaker than glm on activation, and the walk looks economical then stalls." Defence: Phase-0 FORK-A + Phase-4 FORK-B both gate on a MEASURED pass rate before opening/continuing the drain; "assumed not measured" is a tripline, not an assumption.

================================================================================
4. TOP 3 RISKS (carry into execution)
================================================================================
R1 workforce reality: confirmed live 2026-07-06, but a boot probe (Phase 0) re-verifies every session — the free window could close between sessions; Route-SWE + measured probe is the fallback.
R2 skippable precheck = the 0-commit experiment: same gates, routing landed 70+, blanket-greenfield landed 0. precheck-route non-skippable + greenfield->escalate fail-closed are load-bearing, not optional.
R3 chain-state trust decay: over N sessions small walk-state lies compound; the lock, the git-beats-walk-state reconcile, and commit_sha-required DONE rows must ship in Phase 1, not later.

================================================================================
5. EXECUTABLE-BLIND SELF-CHECK
================================================================================
Every fork has a deterministic trigger (Phase-0 model presence, Phase-4 pass-rate). Every move states what you SEE when it works and what you DO when it fails. Every unsettled assumption is a RECON_NEEDED tripline with an exact check. Code units name their WIRING_CONTRACT seam. The mechanism and the drain both carry an INTEGRATION_SMOKE. A mid-tier executor could run Phases 0-1 blind; Phases 2-4 are the Opus controller's own job by construction (that is the product).
