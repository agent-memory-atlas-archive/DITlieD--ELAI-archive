# J-NASA-GATES grounding map — the deterministic-gates wave (J-65..J-71)
Authored 2026-06-22 (main-LLM in-band per paper-to-plan phase 5). Source run: the deep-research
"Deterministic LLM Harness Rule Enforcement Research Report" (2026-06-21, 31-work catalogue), 13 works
faithfully ingested to workflow/papers/*-interface.md by 13 sonnet paper-ingestors. Steal analysis +
group proposals live at docs/thefinalglance/NASA-GRADE-DETERMINISTIC-GATES-PLAN.md (the phase-4 artifact).
This map is the W-7 anchor: every file:line a J-65..J-71 detail file uses is listed here or marked
PLAN-asserted (executor re-greps at edit).

CANONICAL CONSTANTS (LIVE, grepped 2026-06-22; NEVER hard-code, re-read at edit per W-9)
- SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH + SSB_ACTION_TYPES.len(); epoch = -309 (ssb.rs:1503),
  derived at ssb.rs:1508, public alias ssb.rs:1513.
- SSB_ACTION_TYPES array head at ssb.rs:62; len = 577 (the two trailing comments ssb.rs:1486/:1494
  read "575 -> 576" and "576 -> 577"), so LIVE SCHEMA_VERSION = -309 + 577 = 268.
- BASELINE FOR THIS WAVE: SCHEMA 268 / SSB_ACTION_TYPES 577. (The feature-map "241/550" stamp is the
  2026-06-12 number and is STALE; the 5b0a66d drain stamp "~268/~577" matches this grep.)

INGESTED-WORK INTERFACE MAPS (the ground truth; RAG-unindexed, on disk — N1 in the plan)
- workflow/papers/power-of-ten-interface.md          W1  -> J-65
- workflow/papers/enforceable-security-policies-interface.md   W2  -> ND-1 (feasibility), J-66/J-67
- workflow/papers/edit-automata-interface.md         W3  -> J-67
- workflow/papers/efficient-monitoring-safety-interface.md     W4  -> J-66 (past-time leg)
- workflow/papers/rv-survey-interface.md             W5  -> ND-2 (layer routing), J-66/J-68
- workflow/papers/cbmc-interface.md                  W6  -> J-68 (Kani analog)
- workflow/papers/do333-case-studies-interface.md    W7  -> ND-3, J-70
- workflow/papers/tla-specifying-systems-interface.md  W8  -> J-68 (D3 TLA+)
- workflow/papers/cedar-interface.md                 W9  -> J-71 (spike)
- workflow/papers/spark-gnatprove-interface.md       W10 -> J-68 (D2 contracts)
- workflow/papers/nasa-std-8739-8b-interface.md      W11 -> ND-4, J-70
- workflow/papers/raft-interface.md                  W12 -> J-69 (hash-chain / watermark)
- workflow/papers/monitoring-rewriting-interface.md  W13 -> J-66 (future-time leg)

ELAI SUBSTRATE ANCHORS (the seams the wave rides; PLAN-asserted unless marked grep-confirmed)
- SG-1 SSB append-only stream + action_type discipline: ssb.rs:62/1503/1508 (grep-confirmed 2026-06-22);
  the broadcast-after-commit path + MEMORIZING->COMPLETE transition are the J-66 consumer seams (PLAN-asserted).
- SG-2 F-4 cant-lie gate = AntiFabricationGate.check() at task_runner.rs, the auditor-owned deterministic
  check on the post-EXECUTING boundary (feature-map 1.6 [WIRED]); J-65/J-69 are its siblings. The "test_passed
  preceded by stdout_hash" temporal gate IS F-4 — do NOT rebuild as a J-66 monitor (plan PART 3).
- SG-3 frozen-gate harness: .claude/gate-manifest.txt + .claude/tools/gate_freeze_check.py (grep-confirmed
  present 2026-06-22) + the existing checkers maintainability_check.py / production_caller_check.py /
  traced_test_assert_check.py / python_wire_check.py / wiring_contract_check.py / spec_link_check.py
  (all grep-confirmed in .claude/tools/ 2026-06-22). J-65 is a NEW sibling; J-70 enriches the manifest.
- SG-4 J-12 RuntimeInterventionHarness (PLANNED): the H2 pre-tool gate riding rules lifecycle.rs:1 in the
  PreToolUse chain at runtime_setup.rs:120; +HarnessInterventionEvent. J-67 EXTENDS its Block-only action
  to the typed {Accept,Suppress,Insert,Block} taxonomy. (PLAN-asserted; J-12 is the substrate.)
- SG-5 I-18 per-row chain_prev_hash + 5-layer-stack hash (PLANNED) + J-22 writer-isolation contract /
  sole-commit-point (PLANNED): J-69 is their watermark-monotonicity + O(n) chain-SCAN-gate consumer, NOT a
  competing chain producer (plan GROUP F: "do NOT author a competing plan"). PLAN-asserted.
- SG-6 file_ops.rs:272/300 captures-then-discards the pre-image (cited by J-14/H-49); J-69's snapshot leg and
  J-68's serializer round-trip both touch the SSB row bytes. PLAN-asserted; re-grep at edit.
- SG-7 phase-3I DST (madsim proptest 10000) — the runtime FSM simulator; J-68 D3 (TLA+/TLC) is its
  design-time exhaustive-BFS complement, NOT a replacement. PLAN-asserted.
- SG-8 G-1 capability/taint sets + rules-engine COMPOSITE (feature-map 1.x); J-71's Cedar spike re-expresses
  ONE existing capability gate as a policy over (principal,action,resource,context). PLAN-asserted.

NEW DOCTRINE FAMILY (ND, "NASA-Doctrine") — earned per R-j (the 13-work corpus introduces blocking
authoring rules no existing W/GD/HD/JD family covers). Full statements live in J-0_INDEX.md doctrine section:
  ND-1 safety-property feasibility (W2): a runtime monitor gate MUST be a safety property + predicate on
       STEP/EVENT counts, never wall-clock; a liveness/wall-clock predicate is rejected at authoring.
  ND-2 RV layer routing (W5): assign each property to its cheapest SOUND layer (proof/model-check/RV/test);
       a property a layer cannot soundly decide is a finding; respect the 3-valued monitorability caveat.
  ND-3 indeterminate-never-pass + DO-333 traceability (W7): every frozen gate declares
       {objective, technique, evidence, satisfaction}; UNKNOWN verdict = FAIL (mirrors maintainability).
  ND-4 IV&V independence (W11): the auditor keeps FINANCIAL independence (own token/cycle budget the
       orchestrator cannot raid) + MANAGERIAL independence (not short-circuitable by orchestrate H-10 skips
       on safety-critical changesets).

ANTI-SLOPSQUAT VERIFICATIONS (carried from the plan, N4; re-verify at edit before any dep add — W-house)
- kani-verifier 0.67.0 — LIVE (the Rust bounded-model-checker, CBMC analog); PRIMARY for J-68 D1.
- creusot 0.12.0 — LIVE (optional gold contracts, J-68 D2); Prusti REJECTED (unmaintained, last 2024-03).
- cedar-policy 4.11.1 — LIVE, Apache-2.0 (J-71 spike only; SMT policy-equivalence per cedar-interface.md Thm 4.2).
- blake3 not sha256 for the J-69 chain (CLAUDE.md anti-stale); TLA+/TLC is a toolchain dep (J-68 D3), CI-cost-scoped.

CARRIED CORRECTIONS (must not regress)
- NG-1 the "test_passed -> stdout_hash" gate is ALREADY F-4 (SG-2); J-66 builds ONLY G1/G2 + the obligations.
- NG-2 GROUP F is an EXTENSION of I-18/J-22, never a competing chain (SG-5); J-69 adds the scan-gate + watermark.
- NG-3 N3 RETRACTED: ELAI-the-product already has the source-grounded copy-not-fabricate substrate
  (SOURCE_CHECK / source_ingest skill / A-RAG extractive / H-53 citation pre-binding / source-tier list) —
  NOT a steal candidate, do NOT re-propose (plan PART 3 N3).
- NG-4 every external figure (88.5% / -61.7% / 71% SIF / DO-333 objective counts) is DESIGN-rationale only;
  the done-gate produces ELAI-measured evidence (HD-1 / ID-5). Never claim a borrowed number.

End of J-NASA-GATES grounding map.
