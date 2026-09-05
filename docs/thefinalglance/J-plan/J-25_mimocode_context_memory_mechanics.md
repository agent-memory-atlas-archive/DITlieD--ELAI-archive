# J-25: MiMoCode context+memory mechanics — checkpoint template, budgeted read, boundary guard, pressure ladder, writer coalescing, completion gate, retrieval mechanics

STATUS: PLANNED (WAVE RSW-1/RSW-2; .elai_cc runtime; EXTENSION — seven source-verified mechanics from the MiMoCode source read landing into the SHIPPED/PARTIAL H-50/H-51/H-6/E-25/SessionDag/B-22/E-5/H-33 surfaces; nothing here is a new subsystem)

- Track J (repo-steal extension) | lever CANT-FORGET + EFFICIENT | grounding STRONG (direct source read of packages/opencode/src, MEDIUM reliability; ELAI anchors PLAN-asserted from the 2026-06-12 feature map — executor re-greps per W-7) | nature EXTENSION | effort M-L | WAVE RSW-1 (T3/T4/T6) then RSW-2 (T1/T2/T5/T7)

- Premise: MiMoCode (Xiaomi's opencode fork, docs/.source/MiMo-Code) is the closest shipping peer
  to ELAI's context-lifecycle plans: its headline feature (cross-session memory + checkpoint
  rebuild) is H-50+H-51 built and production-hardened. The steal is NOT the architecture (ELAI's
  is planned and stricter) — it is seven precise mechanics ELAI's plans describe more loosely or
  lack, each with an exact source citation in docs/.source/MiMo-Code-INDEX.md section 4 and
  workflow/papers/mimo-code-part-2/7.md. Steal file:
  workflow/reasoning/steal-mimocode-sia-agentskills-2026-06-12.md (A1..A7; riders A5/A10/A11/A12).

## Steal/build (the seven mechanics)
T1 TYPED CHECKPOINT TEMPLATE (A1; session/checkpoint-templates.ts). 11 sections with per-section
  token budgets (intent 500 / next-action 1000 / directives 800 / task-tree 1000 / current-work
  2000 / files 1500 / discoveries 2000 / errors 1500 / live-resources 1000 / decisions 3000 /
  notes 800, ~11K total) + a discoveries-dedup validator (prior-titles). Lands as the OUTPUT
  CONTRACT of H-51's handoff digest (crates/orchestrator/src/handoff_digest.rs, PLAN-asserted):
  typed DigestSection enum + per-section byte budget + dedup check, replacing free-form prose.
  NOT a second digest format — H-51's tiered load consumes the same struct.
T2 SECTION-AWARE BUDGETED READ (A2; session/budgeted-read.ts:65-118). Read primitive: ALL section
  headers + index lines preserved unconditionally (the skeleton always survives), body filled to
  budget in section order, file:line resumption hint on overflow, 0.95 safety factor. Lands as
  the loader H-51 tiered load + H-6 selective-reload use. Pure function, contract-testable.
T3 TOOL-PAIR BOUNDARY GUARD (A3; session/boundary.ts:34-77). Pure backward walk run before ANY
  trim/mask/compaction boundary: un-orphan tool_use/tool_result pairs, then include same-message
  thinking-block siblings. Zero I/O. Lands at the SessionDag 3-pass trim seam
  (session_dag.rs:1551 region, PLAN-asserted) + H-6 masking boundary + E-25 ladder tiers. This is
  a correctness invariant ELAI's trim descriptions do not state; a violated pair = malformed
  Anthropic request after rebuild.
T4 GRADED PRESSURE LADDER (A4; session/compaction.ts). pressureLevel 0/<50% 1/50-70% 2/70-85%
  3/>=85% of usable (= input_limit - min(20K, max_output)) as the TRIGGER INPUT to H-50's
  ContextPolicy (context_policy.rs, PLAN-asserted), replacing single-threshold gates
  (conversation.rs:22 flat auto-compact, cli_consts.rs:20 85% gate — both H-50 fix targets);
  protected classes (skill/anchor slots never erased; recent-turn floor 25% of usable) map to
  ELAI's ContextSlot floors (context_budget.rs phase-3E, SHIPPED — compose, don't re-implement).
T5 WRITER COALESCING + FORK SNAPSHOT (A6; checkpoint.ts + actor/spawn.ts:56-92). Background
  digest-writer with a 1-slot newest-wins pending queue (newer request evicts older — latest is
  always the superset) + a frozen (system_parts, tool_defs, watermark_msg_id) snapshot captured
  at spawn so the writer never recomputes the prompt (prefix-cache aligned, HD-2-friendly).
  Lands on H-51's digest-authoring path. Writer-isolation matches I-1 doctrine.
T6 DETERMINISTIC COMPLETION GATE (A7; actor/spawn.ts:402-437 + task/gate.ts). At agent-return
  time, query the task registry for non-terminal tasks owned by the returning agent; if any,
  ONE re-entry or downgrade the reported status to partial with the open IDs listed. Off-agent
  (DB rows), TD-3-clean (no LLM adjudication). Lands at the B-22 AutoAdvanceEvaluator /
  task_runner agent-return seam as one more EvidenceCheck. The LLM-judge sibling (goal.ts) is
  REJECTED per TD-3 — only the deterministic leg ports.
T7 RETRIEVAL MECHANICS (A9; memory/service.ts:52-133 + reconcile.ts:46-87). (a) relative score
  floor = 0.15 * topScore (corpus-size-stable, replaces absolute floors), 3x over-fetch cap 50;
  (b) file fingerprint = "{size}-{mtimeMs}" upsert-on-change reconcile with prune-on-delete.
  Lands in crates/memory retrieval (E-5 dark retrieval leg / H-33 hub) and is cited as a
  mechanics input to J-21's retrieval-time leg. NOT a platform swap (ledger-clean).
RIDERS (recorded here, NOT tasks): A5 microcompaction = H-49 RV-2 confirmation; A10 dream/distill
  interval + project-age + spawn-gap guards = cadence rider on H-51/I-12 consolidation jobs;
  A11 midpoint cache-marker position = H-2 rider (evaluate inside H-2, HD-2 governs); A12
  subscriber seq-continuity check = J-22 rider (fold into its writer-isolation contract test).

## Existing ground truth (PLAN-asserted from ELAI-FEATURE-MAP 2026-06-12; re-grep at edit, W-7)
- H-50 REGIME-AWARE CONTEXT POLICY [PARTIAL, uncommitted] context_policy.rs + contract
  h50_context_policy_contract.rs; H-51 SLEEP-TIME HANDOFF [PARTIAL, uncommitted]
  handoff_digest.rs + HandoffLifecycleEvent (227).
- SESSION DAG 3-pass lossless trim [WIRED] + H-6 masking [PARTIAL] + E-25 ladder [DARK x9 flags].
- CONTEXT BUDGET ALLOCATOR [WIRED] 8 slots, floors+steal (phase-3E) — T4 composes, never replaces.
- B-22 AutoAdvanceEvaluator deterministic EvidenceCheck [DARK flags]; anti-spiral variants fire
  unconditionally in drive_task (task_runner.rs:7143-7330).
- USER MEMORY SUBSTRATE E-5 [PARTIAL] crates/memory; MEMORY HUB H-33 [PARTIAL]; J-21 PLANNED.
- Source ground truth (W-3): docs/.source/MiMo-Code/ clone + MiMo-Code-INDEX.md +
  workflow/papers/mimo-code-part-{2,7,8}.md + mimo-code-interface.md.

## REVISED scope (W-10 tiers)
SHIP NOW Tier-1 (in-process, deterministic, fixture-testable): T3 boundary guard, T4 pressure
  ladder, T6 completion gate, T2 budgeted read. DEFER-WITH H-50/H-51 LANDING Tier-2: T1 template
  contract + T5 writer coalescing (they modify H-51 surfaces that are PARTIAL/uncommitted — land
  with or immediately after H-50/H-51, never against a moving tree). T7 lands with the E-5
  retrieval un-dark or J-21, whichever executes first.

## Integration
- crates/orchestrator/src/handoff_digest.rs (T1/T2/T5; MODIFY, ~250 LOC across the three).
- crates/orchestrator/src/session_dag.rs trim passes + context_policy.rs (T3/T4; MODIFY ~120 LOC;
  boundary guard is a new pure module crates/orchestrator/src/context_boundary.rs ~80 LOC).
- crates/orchestrator/src/task_runner.rs agent-return seam (T6; MODIFY ~60 LOC + gate module).
- crates/memory retrieval path (T7; MODIFY ~80 LOC).
- frontend_pair: N/A — no new dashboard surface; pressure level + digest events surface through
  the EXISTING H-51/H-9 panels. Justified.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/context_boundary.rs (adjust_boundary_for_invariants pure fn)
- handoff_digest.rs::DigestSection typed template + section budgets + dedup validator
- handoff_digest.rs::read_budgeted_section_aware loader
- context_policy.rs::pressure_level grading fn + protected-class wiring
- task_runner agent-return completion gate (open-task downgrade)
- crates/memory relative-floor scoring + fingerprint reconcile
consumes_symbols:
- handoff_digest.rs (H-51), context_policy.rs (H-50), session_dag.rs trim (phase-3H/B-9),
  context_budget.rs slots (phase-3E), AutoAdvanceEvaluator (B-22), crates/memory (E-5/H-33) —
  do NOT re-implement any of these; this unit only sharpens their internals.
reachable_from:
- the live session loop: drive_task -> context policy tick -> trim/digest paths (PRODUCTION;
  PLAN-asserted anchors, re-grep at edit). A contracts/-only caller never satisfies this (R7).
stub_ban:
- NO second digest format beside H-51's; NO LLM call in any of T2/T3/T4/T6/T7 (deterministic);
  NO absolute-score floor reintroduction; NO .unwrap()/.expect() on prod paths; NO blake2/sha2;
  NO re-summarization touching the cache prefix (HD-2).
observable_effect:
- trigger: "a long fixture session crossing pressure level 2 with an orphaned tool_use at the
  candidate boundary, then `elai run --emit-rra` replay of the rebuilt window"
- expect_ssb: "HandoffLifecycleEvent (or the decided-at-edit reuse rows) showing digest write with
  section budgets respected; existing trim/compaction rows show the adjusted boundary"
- expect_artifact: "rebuilt request validates against the Anthropic message invariants (no
  orphaned tool_result); NEGATIVE: a fixture with a deliberately orphaned pair and the guard
  DISABLED must produce the malformed-request failure — if it passes, the test harness is wrong
  (R2). An agent return with open task rows and the gate ON must come back `partial`, never DONE."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md --run-trigger
rg "adjust_boundary_for_invariants" .elai_cc/crates/orchestrator/src/ -l   # guard exists + has a non-test caller
rg -c "unwrap\(\)" .elai_cc/crates/orchestrator/src/context_boundary.rs    # MUST be 0
test -f workflow/reasoning/J-25-decisions.md
```

## Grounding
- Source: MiMo-Code clone (MEDIUM reliability), file:line cites per task above from
  docs/.source/MiMo-Code-INDEX.md section 4 + workflow/papers/mimo-code-part-2.md/part-7.md.
- ELAI anchors: PLAN-asserted from ELAI-FEATURE-MAP.md 2026-06-12 (executor re-greps, W-7).
- Doctrine that bites: HD-2 (T1/T5 never bust the byte-stable prefix), TD-3 (T6 stays
  deterministic; goal.ts LLM judge rejected), RV-2 (T3 trims views, never the record), CL-1..CL-4
  (T4 policy is per-BackendMode), GD-1 (compose with the eight named surfaces).
- SSB: prefer-reuse — H-51 HandoffLifecycleEvent (227) + existing trim/compaction rows; if a new
  row is genuinely needed at edit time, +ContextMechanicsEvent (Pattern-2; Kind: BoundaryAdjusted /
  PressureGraded / CompletionDowngraded / RetrievalRefloored) baseline+1 from LIVE 231/542
  (ssb.rs:3084 / action_type_lock.rs:671; re-grep at edit, W-9). Decide at edit; net 0..+1.

## COMPOSES-WITH
H-50, H-51, H-6, E-25, SessionDag (phase-3H/B-9), context-budget (phase-3E), B-22, E-5, H-33,
H-49 (rider), H-2 (rider), I-12 (rider), J-21 (T7 mechanics input), J-22 (rider). Restates none.

## FORKS
- FORK-1 (thinking siblings at trim seam): session::ContentBlock has NO Thinking variant (S1 recon).
  DEFAULT = implement tool_use/tool_result un-orphan leg only for Tier-1 T3; defer thinking-sibling
  leg until ContentBlock gains Thinking OR document FORKS: none for that sub-leg with explicit
  deferral note. TRIGGER: if the executor observes a Thinking variant in session::ContentBlock at
  edit time, take ROUTE B (include same-message thinking-block siblings in the backward walk);
  else stay on ROUTE A (tool-pair only).
- FORK-2 (Tier-2 T1/T5): remain DEFER-WITH H-50/H-51 landing as already written. TRIGGER: if
  H-50/H-51 lands before this execute wave closes, re-scope T1/T5 into a follow-up Tier-1 wave;
  else stay DEFERRED. No thinking-sibling work touches this fork.

## ABORT conditions
- ABORT-1: inventing a second digest format beside H-51 -> STOP, surface to orchestrator. The
  typed DigestSection enum is the OUTPUT CONTRACT of H-51's existing digest, not a parallel path.
- ABORT-2: an LLM call in T2/T3/T4/T6/T7 -> STOP. These are deterministic mechanics by design
  (TD-3); an LLM adjudication leg is a scope violation, not a fix.
- ABORT-3: modifying context_policy.rs inside the T3-only slice -> STOP. T3 owns the boundary
  guard at the trim seam; T4 owns the pressure_level grading inside context_policy.rs. Crossing
  the boundary in one slice is a scope violation (R-3).
- ABORT-4: greenfield undesigned seam (trim_context gone or signature incompatible with the
  PLAN-asserted 917-1046 region) -> SET_ASIDE, surface as a RECON_NEEDED entry; do NOT improvise
  a new seam. The executor re-greps session_dag.rs at edit (W-7) to confirm the trim seam.

## Tasks as MOVES (Tier-1 only for this refine; Tier-2 marked DEFERRED)
MOVE T3 — context_boundary.rs + trim_context caller
  EXPECTED_OBSERVATION: adjust_boundary_for_invariants exists; non-test caller in
  session_dag::trim_context; contract test with orphaned tool_use fixture FAILS when guard
  disabled / PASSES when enabled; unwrap count 0 in context_boundary.rs.
  ON_FAIL: production_caller_check dark_fn -> wire caller; pair still orphaned -> fix backward
  walk; ContentBlock lacks tool_use variant -> ABORT-4 (re-grep seam, surface).

MOVE T4 — pressure_level in context_policy.rs
  EXPECTED_OBSERVATION: pressure_level 0..3 grades; composes ContextSlot floors (phase-3E);
  contract test covers each grade boundary; no flat trim_threshold as sole trigger.
  ON_FAIL: flat trim_threshold still sole trigger -> grade fn not wired into ContextPolicy tick;
  grade fn dark -> wire from context_policy::tick.

MOVE T6 — completion gate at agent-return
  EXPECTED_OBSERVATION: open non-terminal tasks -> status partial with open IDs listed; no LLM;
  off-agent DB rows; EvidenceCheck lands on B-22 AutoAdvanceEvaluator / task_runner return seam.
  ON_FAIL: DONE despite open tasks -> gate not on return path; LLM adjudication present -> ABORT-2.

MOVE T2 — read_budgeted_section_aware
  EXPECTED_OBSERVATION: skeleton headers always survive; body fills to budget in section order;
  file:line resumption hint on overflow; 0.95 safety factor; pure function, contract-testable.
  ON_FAIL: headers dropped -> loader wrong; overflow silent -> resumption hint missing.

Tier-2 T1/T5/T7: DEFERRED. EXPECTED_OBSERVATION: "not in this execute wave". ON_FAIL: n/a.

## VERIFY
Off-agent (the auditor runs these; the executor never self-certifies):
- plan_gate_check.py --file docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md
  -> PASS (exit 0).
- wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md
  --run-trigger -> PASS.
- production_caller_check.py --files <changed .rs> -> 0 DARK on adjust_boundary_for_invariants
  and every new pub fn; a DARK symbol without a WIRE-DARK row is a BLOCK.
- cargo check -p elai-orchestrator -> GREEN.
- cargo nextest -p elai-orchestrator <j25_context_boundary contract filter> (or equivalent
  contract test name) -> GREEN; the orphaned-tool_use NEGATIVE fixture must FAIL with the guard
  disabled and PASS with it enabled (R2).
- rg "adjust_boundary_for_invariants" .elai_cc/crates/orchestrator/src/ -l -> at least one
  non-test caller in session_dag.rs.
PASS = every line above is GREEN off-agent; the executor's own stdout is NEVER the pass artifact
(R4). Missing evidence is a hard FAIL (R1), never SKIP/UNVERIFIED.

## REDTEAM
ATTACK-1: disable the boundary guard, run the orphaned tool_result fixture. The Anthropic-
  invariant check must FAIL (R2). If it passes, the test harness is wrong -> PATCH the harness,
  not the guard. Record the surviving attack + the patch in J-25-decisions.md.
ATTACK-2: the executor adds a Thinking variant to session::ContentBlock without owner FORK
  resolution -> REJECT; stay on the tool-pair leg (FORK-1 ROUTE A). Thinking-sibling work is
  deferred until ContentBlock gains Thinking via a separate owner-approved change.
ATTACK-3: absolute score floor reintroduced in T7 -> REJECT. T7 uses 0.15 * topScore (relative),
  corpus-size-stable. An absolute floor is the bug T7 exists to remove.
ATTACK-4: a second digest format beside H-51 -> ABORT-1. The typed DigestSection enum is the
  output contract of H-51's existing digest, not a parallel path.
ATTACK-5: LLM adjudication sneaks into T6 completion gate -> ABORT-2. The deterministic leg is
  the only leg that ports (TD-3); goal.ts LLM-judge sibling is REJECTED.

## INTEGRATION_SMOKE
Per Tier-1 unit (R10 — per-unit green is necessary, not sufficient; the seams are where plans
die):
- T3: cargo nextest -p elai-orchestrator <j25_context_boundary filter> + rg
  adjust_boundary_for_invariants with a non-test caller in session_dag.rs. Drive the trim seam
  with a fixture session crossing pressure level 2 and an orphaned tool_use at the candidate
  boundary; assert the rebuilt request validates against Anthropic invariants.
- T4: cargo nextest -p elai-orchestrator <j25_pressure_level filter> + assert pressure_level
  fires the correct ContextPolicy branch per grade; compose with ContextSlot floors, do not
  re-implement.
- T6: cargo nextest -p elai-orchestrator <j25_completion_gate filter> + assert an agent return
  with open task rows comes back partial, never DONE; off-agent DB rows only.
- T2: cargo nextest -p elai-orchestrator <j25_budgeted_read filter> + assert skeleton headers
  survive and body fills to budget with a 0.95 factor.
Full `elai run --emit-rra` replay deferred until T4 pressure fixture lands (note
INTEGRATION_SMOKE cadence n/a for deferred Tier-2 T1/T5/T7). The cross-seam smoke (T3 boundary
guard -> T4 pressure trigger -> T6 completion gate) runs after every 2-3 Tier-1 units AND before
DONE, driving the REAL entrypoint (drive_task -> context policy tick -> trim/digest paths) on a
HARD input (a long fixture session with orphans, pressure crossings, and open tasks), not a toy
happy-path.

End of J-25.
