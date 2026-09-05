# J-DOCTRINE-PROPAGATION-GROUNDING-MAP — grounding for the DP wave (J-43..J-47)
Grounded 2026-06-16, session s-dp-author (main LLM direct, no agents — doc-authoring wave like the prior J runs).
Method: read the DP design doc + handoff in full (the authoritative grounded baseline from the prior design session),
the ELAI-DETERMINISM-DOCTRINE.md lens, the four precedent gate rules, the J-39 exemplar + both J indexes; a live
glob/grep sweep confirming every named seam file EXISTS. Every DP detail file cites these DG-* IDs instead of
re-asserting (R-a). PLAN-asserted anchors are labelled; executors re-grep at edit (W-7). No line numbers are
fabricated — files are cited by path + section/symbol, like the J-39 exemplar's UNCONFIRMED-flagged anchors.

SOURCE OF TRUTH (the prior session's grounded baseline — read first, do not re-derive):
- workflow/reasoning/determinism-doctrine-propagation-design-2026-06-16.md (sections 1-12 + addenda b/c/d) — THE design.
- workflow/handoff/determinism-doctrine-propagation.md — the pickup spec (DP-1..DP-5 decomposition, the V0/C1/C2/C3/V4/V5 corrections).
- workflow/reasoning/context-controller-carry-state-design-2026-06-16.md — the sibling flagship (its own later wave).
- workflow/reasoning/distill-reset-propagation-reasoning-2026-06-16.md — the upstream reasoning the carry-state design grew from.

================================================================================
DG — SEAM GROUND TRUTH (glob/grep-confirmed 2026-06-16, this session)
================================================================================
DG-1  .claude/rules/maintainability-review.md EXISTS (glob) — the PRECEDENT TEMPLATE: deterministic-subset
      (MR-1..MR-4, off-agent, frozen, fail-closed) + qualitative catalog + "unknown = FAIL" + "auditor-applied,
      never executor-self-applied" + C-MAINTAINABILITY-* finding tags + SELF-CHECK. determinism-doctrine.md (J-43)
      is this shape ONE ALTITUDE UP. Loaded in full this session (it is in the CLAUDE.md rule set).
DG-2  .claude/tools/maintainability_check.py EXISTS (glob); .claude/tools/production_caller_check.py,
      python_wire_check.py, gate_freeze_check.py, bn_status_report.py EXIST (glob). The floor-checker + derived-metric
      tool shape to mirror (--files/--base/--json, exit 1 BLOCK / exit 2 scan-config-error, named-constant thresholds,
      manifest-frozen, stdlib). doctrine_floor_check.py (J-43) / reasoning_artifact_shape_check.py (J-44) /
      doctrine_debt_rate.py (J-46) are siblings.
DG-3  .claude/tools/gate_freeze_check.py + .claude/gate-manifest.txt EXIST (glob). The freeze mechanism (git diff over
      the frozen fileset; a run that modified its own gate is BLOCK). J-43 injection 5 adds determinism-doctrine.md +
      doctrine_floor_check.py to the manifest.
DG-4  .elai_cc/wire-dark-allowlist.toml EXISTS (glob). The debt-ledger posture precedent (type/owner/expiry/
      promotion_plan_id rows + the in-code WIRE-DARK[<plan-id>] marker that downgrades BLOCK to WARN). doctrine-debt-
      ledger.toml (J-43) mirrors it; the DOCTRINE-DEBT[<plan-id>] marker is the analog.
DG-5  .claude/agents/auditor.md EXISTS (glob). The off-agent checker host — already runs production_caller_check.py /
      maintainability_check.py / traced_test_assert_check.py over the changeset (per .claude/rules/*). The doctrine
      floor (J-43) + the reasoning-shape check + substance-audit panel (J-44) join that set; executor never self-runs.
DG-6  .claude/rules/conditional-routing.md EXISTS (in the CLAUDE.md rule set this session) — the auto-load registry
      with the existing wiring/maintainability/production-caller entries. determinism-doctrine.md + reasoning-method.md
      get parallel "ships/audits a feature or touches a gate / authors a reasoning artifact" entries.
DG-7  docs/strategy/ELAI-DETERMINISM-DOCTRINE.md EXISTS + read in full this session — D1-D5, the model/harness boundary,
      the 8-question reviewer checklist, the named instances (TD-3, D-13, VD-4, D-10, HD-3, F-4, R8/R9). The LAW the DP
      wave enforces; imported by reference, never restated.
DG-8  docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md — the METHOD (M1-M12 + the stance question) + the per-axis stance
      sketches. PLAN-ASSERTED from the DP design doc's verbatim citation (the design's addendum b/d read it); the file
      itself was NOT opened this session. CONFIRM existence + M-numbering at edit before J-44/J-47 cite it load-bearing (W-7).
DG-9  .claude/skills/planning-workflow/SKILL.md EXISTS (glob) + its plan-improvement protocol emitted in full this
      session (9 steps; step 4 WALKING-SKELETON PRE_GATE; step 7 ACCEPTANCE/OFF-AGENT PROOF; GROUNDING + ADVERSARIAL
      injections). The PARTIAL Track-2 precedent J-44 generalizes (adding the M2/stance/M7/M10 presence checks it lacks).
DG-10 .claude/skills/spec-eval/SKILL.md EXISTS (glob). The design-time host — already runs an adversarial attack-the-
      design habit; J-44 adds the DECISION LEDGER as a required output dimension (injection 1).
DG-11 .claude/skills/idea-intake/SKILL.md EXISTS (glob). The per-level plan-auditor whose rubric gains the doctrine
      check (it already checks hard D-rule violations); J-44 injection 2.
DG-12 .claude/rules/agent-dispatch-preamble.md EXISTS (glob) + loaded in full this session (R-1..R-8, the R10 integration
      gate reference). J-44 adds the doctrine pointer + the R10 seam-disposal check (injection 4).
DG-13 the gpt-bridge MCP is CONNECTED this session — gpt_chat / grok_chat (second + third pretraining families) /
      perplexity_chat / gemini_chat / claude_chat. SHARED apps_workflow pool, NO repo access (serialize the snippet in),
      rate-limit-fragile, one vote that never grounds a critical path. J-45's cross-family advisory carrier.
DG-14 .claude/rules/handoff-protocol.md EXISTS (in the CLAUDE.md rule set) + the orchestrate/auto-orchestrate
      handoff-mediation loop (skills exist, glob). J-45's escalation route (a flagged finding -> orchestrator/user
      decision entry, never a dashboard).
DG-15 .claude/skills/oracle/SKILL.md EXISTS (glob). A legitimate host for a doctrine diversity-vote prompt at design
      time (DP design injection-1 cross-family vote).
DG-16 docs/thefinalglance/ELAI-FEATURE-MAP.md — the living source of truth (status flags + composition edges + the
      Section 3 doctrine blocks + the REJECTED ledger) with a documented FEATURE-MAP UPSERT seam (the planning-workflow
      MEMORIZING step emitted this session names it; memory project_feature_map_canonical). J-46's conformance stamp
      upserts here; WIRED status drives the sweep. PLAN-ASSERTED file existence (not globbed this session) — confirm at edit.
DG-17 SAFEDELTA G-6 / E-9 (gate-ON-vs-OFF ablation, NOT a leaderboard rank) — the D4 "measure the delta not a rank"
      precedent J-46's debt-rate honors. In-repo per the doctrine D4 example + memory; cite by name.
DG-18 docs/thefinalglance/J-plan/J-39..J-42 EXIST (glob) — the four ALREADY-BUILT axis-doctrines (decomposition /
      research / grounding / reasoning), each ending "ELAI-DETERMINISM-DOCTRINE: the canonical <X> application." The
      precedent that the doctrine propagates cleanly as J-format files; J-47's per-axis discipline must NOT re-author them.
DG-19 the per-file falsifiers EXP-DP1..DP4 (this wave) — the per-gate altitude; J-47's EXP-DP5 is the program altitude.
DG-20 the orchestrate / auto-orchestrate parallel-run machinery (cap 3) + GD-4 observe-only-then-promote — J-47's A/B/C
      harness substrate (reuse, not a new orchestrator).

================================================================================
SSB BASELINE + DP-WAVE BUMP POLICY
================================================================================
LIVE baseline at authoring: SCHEMA_VERSION = -309 + SSB_ACTION_TYPES.len() (ssb.rs:1500, COMPUTED not a literal) ->
SCHEMA 268 / TOTAL_VARIANTS 577 (action_type_lock.rs ALL_VARIANTS.len()), per the J-35..J-42 re-greps. Re-grep BOTH at
edit (W-9, RT-1 — the const is computed, never hard-coded; stamps go stale within days).
DP wave is a HARNESS wave (.claude/ rules + tools + the auditor + skills + docs). Done-gate is JD-1 (a real harness
session produced the off-agent artifact: checker findings, the debt ledger, the rate file, the FEATURE-MAP stamp, the
A/B/C comparison), NOT "the file exists in .claude/". Net 0 new SSB action_type across J-43..J-47 (JC-3 / J-22 / J-27
zero-SSB harness-leg precedent). OPTIONAL +1 DoctrineDebtEvent (Pattern-2; Kind: FloorFinding / DebtRegistered /
DebtPromoted) ONLY if doctrine-violation telemetry is wanted as a first-class runtime-queryable stream (J-43/J-46 G3);
baseline+1 COMPUTED at impl, NEVER hard-coded. Default across the wave: net 0.

================================================================================
INHERITED DOCTRINE (verbatim, canon)
================================================================================
INHERITS the J-track JD-1..JD-4 (J-0_INDEX.md) — JD-1 harness done-gate (real session event -> off-agent artifact, missing
= FAIL); JD-2 derived-never-recalled (the debt-rate + ledger come from recorded state, not recollection); JD-3 observe-only
first (every DP gate ships non-blocking until its FP/friction rate is observed >=3 sessions); JD-4 frozen-gate respect
(DP NEVER edits a registered hook; new tools are new files; the gate-manifest freeze is added by a user-reviewed change).
INHERITS W-1..W-10 (F-0), GD-1/GD-2/GD-4/GD-5 (G-0), HD-1/HD-2 (H-0) where they bite — done = run-trigger/JD-1 not
build-green; compose-not-restate (the DP wave re-implements NO existing gate — it names them as instances); every emitted
number recomputable from its source; no gate blocks before its FP/friction rate is bounded on >=3 runs; honest residue
named in-file. NEW DELTA FAMILY: none — the DP wave is governed by the existing D1-D5 doctrine + JD/W/GD/HD/TD/VD families
(per R-j: the doctrine it propagates IS the rule-set; the DP wave adds enforcement, not a new doctrine).

================================================================================
THE FIVE DP FILES (one line each)
================================================================================
J-43 DOCTRINE-PROPAGATION KEYSTONE (Track 1, the LENS): determinism-doctrine.md meta-rule + doctrine_floor_check.py
     (DD-1..DD-5, observe-only) + doctrine-debt-ledger.toml + auditor off-agent integration + gate-manifest freeze. net 0 SSB.
J-44 REASONING-METHOD ARTIFACT-SHAPE (Track 2, the METHOD): reasoning-method.md + reasoning_artifact_shape_check.py +
     the random-sample SUBSTANCE-AUDIT panel (C2, load-bearing) + 4 lifecycle injections (spec-eval/planning-workflow/
     idea-intake/agent-dispatch-preamble). net 0 SSB.
J-45 CROSS-FAMILY ADVISORY CALIBRATION (the D5 leg): cross-family advisory via gpt-bridge + the 100-300-trace calibration
     corpus + the kappa>0.6 gate + escalation routing + single-family-degraded debt honesty (C3). net 0 SSB.
J-46 DOCTRINE-DEBT-RATE MEASUREMENT (D4 on the propagation): doctrine_debt_rate.py + the only-WIRED-first baseline sweep +
     the ELAI-FEATURE-MAP conformance stamp + the implausible-zero self-test (Q4). net 0 SSB.
J-47 PROGRAM FALSIFIER + PER-AXIS DISCIPLINE (M9 at the program altitude): the A/B/C experiment harness (V5, step 1 of
     the whole program) + the per-axis doctrine-family discipline (V4, measured-pain-only + a count cap). net 0 SSB.

End of J-DOCTRINE-PROPAGATION-GROUNDING-MAP.
