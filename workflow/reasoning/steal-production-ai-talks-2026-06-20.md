steal/validation source-of-record (W-3): the two production-AI talk summaries the user dropped
2026-06-20(c). source files (external, not vendored): C:\Users\user\Downloads\Five-Pillars-of-
Production-AI-SUMMARY.md (Databricks "five pillars" framework talk — VENDOR-framework lens, tier-2/3,
treat as a checklist not ground truth) and C:\Users\user\Downloads\Long-Running-Agents-Anthropic-Talk-
Summary.md (Anthropic Applied-AI long-running-agents talk — FIRST-PARTY on their own harness, tier-2 but
authoritative on what they actually shipped). lens: spec-eval AS-SPECIFIED (judge ELAI's design on its
merits; planned/dark is a sequencing fact, never a mark-down). do not re-litigate.

VERDICT: both talks are MOSTLY CONFIRMATION; on several axes ELAI's specified design is AHEAD of what the
talks describe. NO large net-new plan is justified. four small, grounded EXTENSION/ACTIVATION deltas ->
J-56..J-59. this matches the pattern of every recent external-source run (CONFIRMATION + a few riders).

WHAT IS ALREADY COVERED (compose-not-restate, GD-1 — do NOT re-author these):
- generator/evaluator zero-shared-context adversarial split: SPRINT CONTRACTS + RALPH LOOP (I-21, WIRED) +
  ADVERSARY CROSS-FAMILY (D-13) + PER-LEVEL AUDITOR (B-41). the talk's "self-eval is a trap" = ELAI's
  whole off-agent doctrine (auditor sole test-runner, freeze-the-gate R8, AntiFabricationGate F-4).
- "subjective quality IS gradable, write the rubric": .claude/rules/visual-ui-loop.md (schema-constrained
  pass/fail/unknown, deterministic diff first, oscillation detector) + design-taste + cook. ELAI AHEAD.
- file-system-for-state / JSON breadcrumbs: SSB (append-only SQLite single-writer) + Session DAG + RRA.
  ELAI AHEAD (append-only beats overwritable JSON; the talk uses JSON precisely because md gets clobbered).
- compaction != coherence: REGIME-AWARE CONTEXT POLICY (H-50) + EMPIRICAL CONTEXT CALIBRATION (J-34) +
  context-rot probe. read-traces-to-tune: SSB journal + RRA replay + reviewer cockpit (G-5).
- git worktrees for parallel features: COW WORKSPACE ISOLATION (I-2).
- model-change-management ("re-run YOUR eval on upgrade, don't trust leaderboards"): MODEL-DRIFT CANARY
  (G-10, pin+fingerprint, surface-never-auto-swap) + J-34 "kill folklore thresholds, measure per-model".
  ELAI AHEAD. observability pillar: SSB + RRA + OTEL (E-29) is ELAI's STRONGEST axis.
- orchestration patterns: orchestrate (orchestrator-worker) + BSP. the talk's CHOREOGRAPHY (autonomous
  agents on a message bus, no orchestrator) is DELIBERATELY NOT adopted — ELAI's central FSM gate is the
  trust spine; choreography would dissolve it. defensible divergence, NOT a gap. NO STEAL.

THE FOUR GENUINE DELTAS (each a vertical slice, composes shipped substrate):
- J-56 LIVING EVAL CORPUS + FAILURE-INGEST + CI GOVERNANCE [ACTIVATION]. Five-Pillars Pillar-1 ("eval is
  the spec; the test-case library is a LIVING governed system — owner, categorized by problem-type, CI runs
  a SUBSET on change + FULL on merge") + Anthropic "grow the corpus from caught failures". ELAI has the
  substrate but DARK: EVAL CORPUS DESIGN (E-54, 13-gram filter, ingest_task has NO CLI entrypoint,
  feature-map ~1331) + SAFEDELTA (G-6, CLI+dashboard wired, bench never invoked, 0 rows, ~1324) +
  held-out store (D-6). net-new = the `elai eval ingest` entrypoint + the category taxonomy + the
  CI subset-vs-full-on-merge governance. ACTIVATION, not invention.
- J-57 EVAL-GATED PROMPT-TEMPLATE PROMOTION + ROLLBACK [EXTENSION]. Five-Pillars Pillar-5 ("treat prompts
  as code; every change goes test->staging->prod with eval gating + rollback; governed commit messages").
  ELAI has the STORAGE (REASONING TEMPLATES D-1 blake3 hash-chain + template-migrate CLI, but select_template
  dark in prod, ~236; CompiledPromptStore I-36, ~1081) + the drift signal (G-10) but NO eval-gated
  promotion/rollback WORKFLOW. depends J-56 (the eval gate it runs). this is what makes the versioning
  load-bearing.
- J-58 PRE-EXECUTION SPRINT-CONTRACT NEGOTIATION LOOP [EXTENSION of I-21]. the Anthropic talk's single
  distinct mechanism not already in ELAI: the generator and evaluator NEGOTIATE what "done" means BEFORE
  building, multi-round, until both agree — and the negotiation PRESSURE is what produces a granular
  contract (their retro-game run settled on 27 criteria; "granular criteria -> actionable critique, vague
  criteria -> the generator shrugs"). I-21 (read in full) ships the contract struct + the ISOLATED
  evaluator + a SINGLE pre-exec Approve/Reject, and its only iteration (the Ralph loop) is POST-execution
  on OUTPUT failure. it has NO pre-exec re-propose-until-agree. J-58 adds exactly that bounded loop,
  composing I-21's struct + isolated evaluator + SprintContractEvent. freeze-the-gate R8 is preserved —
  the evaluator stays isolated (zero generator scratchpad); negotiation is via the sealed contract payload
  + the structured objection field, never a shared context.
- J-59 PER-SEAT MEMORY + TRUST ISOLATION [EXTENSION, multi-seat]. Five-Pillars Pillars 3+5 under
  MULTI-TENANT expose that J-49's multi-seat scope (admission + quota + spend, ECONOMICS only; defers
  identity to J-2; NO memory/trace isolation) is the thinnest slice. ELAI's memory substrate (E-5,
  H-31/H-32) is single-user. net-new = per-seat memory namespacing (key by the J-49 seat-id / dormant
  UsageTurnEvent.identity), cross-seat read/write isolation, cross-seat poisoning block, and the
  AUDITOR-OBJECTIVITY FIREWALL (H-31, arXiv:2510.09905 — already grounded there) extended per-seat. this
  is a TRUST/SAFETY gap, which is on-brand; the per-seat audit-VIEW UI is explicitly DEFERRED (lower value,
  and it pushes ELAI toward a multi-tenant platform identity it has not chosen).

REJECTED / ALREADY-COVERED (do NOT re-propose — P0 if re-litigated):
- choreography / autonomous message-bus agents: REJECTED (central FSM gate is the trust spine; see above).
- LLM-as-a-Judge on the critical path (Five-Pillars Pillar-1 Layer-2): STAYS REJECTED — ELAI's TD-3
  ("deterministic, LLM never adjudicates a gate") is a deliberate, stronger answer. the eval corpus (J-56)
  scores against deterministic gate verdicts, never an LLM judge. the talk's Layer-3 BEHAVIOURAL eval
  (right tool, right order, no duplicate calls, no loops) is ALREADY ELAI: anti-spiral, temporal
  tool-sequence (I-35), MAST (E-6).
- "central multi-cloud governed trace store serving every team" (Five-Pillars Pillar-2/3): NOT ADOPTED —
  ELAI is local-first single-record (GD-2 never an external store as the record). per-seat views ride the
  local SSB, not a cloud lake.
- abstract credit/billing display unit for multi-seat: STAYS REJECTED (goose ledger; raw tokens + USD).

SOURCE QUALITY CAVEAT (GD-5 honest residue): both inputs are conference-talk SUMMARIES, not primary docs
or papers. the Anthropic talk is weighted higher (first-party on their harness) than the Databricks talk
(vendor framework). neither was re-verified against the original video. no arXiv IDs are introduced by this
run; the one paper cited (arXiv:2510.09905, auditor-objectivity firewall) is carried from H-31 where it is
already grounded. every ELAI file:line anchor in J-56..J-59 is PLAN-asserted and re-grepped by the executor
at EXECUTING (W-7).

OUTCOME: J-56, J-57 (depends J-56), J-58 (extends I-21), J-59 (extends J-49) created 2026-06-20(c).
