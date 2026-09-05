# J-70: gate-manifest traceability schema + IV&V independence gap-closure — every frozen gate is traced to an objective, and the auditor is structurally independent

- Track J (NASA-gates wave) | lever AUDITABLE | grounding STRONG (do333-case-studies W7 + nasa-std-8739-8b W11 interface maps) / CONFIRMATION (auditor!=executor + frozen-gate already satisfy most of W11) | nature BUILD (doctrine + manifest + small structural) | effort S-M | WAVE NASA-GATES-1 | depends none (enriches .claude/gate-manifest.txt)
- Mostly CONFIRMATION of the existing auditor!=executor + frozen-gate posture, with a DO-333 traceability schema + two measured IV&V independence gaps to close.

- Premise: ELAI freezes its gates (.claude/gate-manifest.txt + gate_freeze_check.py, R8) and separates auditor from executor — which already satisfies most of NASA-STD-8739.8B's IV&V technical independence + records-retention (W11). But two things are missing. (1) DO-333 (W7): each frozen gate does NOT declare WHICH verification objective it covers, which TECHNIQUE it uses, what EVIDENCE it produces, or whether it REPLACEs/AUGMENTs/PARTIALly-satisfies the objective — so there is no requirements->gate->evidence trace matrix, and an indeterminate gate result has no defined disposition. (2) NASA-STD W11 measured GAPS: FINANCIAL independence is absent (auditor and executor share one token budget — the orchestrator can raid the auditor's budget to "save cost", the exact rationalization orchestrate-protocol.md bans in PROSE but not STRUCTURE); MANAGERIAL independence is partial (orchestrate H-10 skip exceptions can short-circuit the auditor on a changeset). J-70 closes both: a manifest traceability schema (ND-3: indeterminate is never a pass) + two small structural changes that make the prose bans structural.

## Steal/build
1. E1 GATE-MANIFEST TRACEABILITY SCHEMA (W7/DO-333): extend .claude/gate-manifest.txt so every frozen-gate row declares {objective_covered, technique (proof | model-check | abstract-interp | RV | test | static), evidence_artifact (SSB-row | coverage | proof | hash), satisfaction (REPLACE | AUGMENT | PARTIAL)}. A row missing any field is a FAIL of the freeze check (gate_freeze_check.py extended). An indeterminate/UNKNOWN gate RESULT is never a pass (ND-3, mirrors maintainability unknown=FAIL). Output: a requirements->gate->evidence trace matrix (a `--trace` report over the manifest).
2. E2a FINANCIAL INDEPENDENCE: give the auditor a SEPARATE token/cycle budget the orchestrator cannot raid. Today the GRC/budget machinery is per-topic; J-70 reserves an auditor budget pool that the orchestrate run cannot draw down to fund executor work — making orchestrate-protocol.md's "saving token budget is a FALSE rationalization" STRUCTURAL, not just doctrinal.
3. E2b MANAGERIAL INDEPENDENCE: the auditor authority path must not be short-circuitable by the orchestrate H-10 skip exceptions for SAFETY-CRITICAL changesets (a changeset touching a frozen gate, a privileged Supervisor-plane fn, or a P10-SAFETY-CRITICAL module). H-10's E-1/E-2/E-3 skips are narrowed: a safety-critical changeset cannot use the audit-skip exception even under user override without an explicit separate confirmation logged.
EXCLUDED: a wholesale IV&V framework (W11 is mostly already satisfied — this is two riders + a schema, not a rebuild); an LLM judging the traceability (the schema is declarative + deterministically checked, TD-3); weakening any existing freeze (J-70 ADDS fields, never removes the R8 freeze); making the auditor a separate PROCESS (financial independence = a separate budget pool, not a second runtime).

## Existing ground truth (grep-confirmed 2026-06-22; re-grep at edit, W-7/W-9)
- .claude/gate-manifest.txt present (the freeze list R8) + .claude/tools/gate_freeze_check.py present — the schema J-70 enriches + the checker that enforces the new required fields.
- the frozen-checker family (maintainability_check.py, production_caller_check.py, traced_test_assert_check.py, python_wire_check.py, wiring_contract_check.py, spec_link_check.py, + J-65 power_of_ten_check.py, + J-68 cargo-kani/TLC) — each gets a traceability row.
- orchestrate-protocol.md H-1..H-11 (the audit-skip / token-budget rationalization bans, PROSE) — J-70 makes H-3 (no audit-skip) + the budget ban STRUCTURAL for safety-critical changesets.
- main-llm-role.md (main model never runs the gate; auditor!=executor) + the GRC per-topic budget machinery — the financial-independence change reserves an auditor pool within it.

## Integration
- MODIFY .claude/gate-manifest.txt (add the 4 traceability columns per row) + .claude/tools/gate_freeze_check.py (require the 4 fields; emit the `--trace` matrix; UNKNOWN result = FAIL). MODIFY orchestrate-protocol.md (H-10 safety-critical carve-out narrowing) + the budget machinery (the reserved auditor pool). NO .elai_cc runtime change beyond the budget-pool reservation; NO SSB action_type (harness/doctrine leg, JD-1 evidence — the trace matrix + the freeze-check verdict are the off-agent artifacts). 
- frontend_pair: N/A — harness manifest + doctrine. The trace matrix is a generated report. Justified.

## WIRING_CONTRACT
produces_symbols:
- .claude/gate-manifest.txt extended schema (objective_covered / technique / evidence_artifact / satisfaction per row)
- .claude/tools/gate_freeze_check.py `--trace` report + the 4-field-required validation (UNKNOWN = FAIL)
- the orchestrate-protocol.md H-10 safety-critical audit-skip narrowing (managerial independence)
- the reserved auditor budget pool (financial independence; the orchestrator cannot raid it)
consumes_symbols:
- the existing frozen-checker family (each declares its traceability row; NOT reauthored)
- gate_freeze_check.py R8 freeze (extended, never weakened)
- orchestrate-protocol.md H-3 / H-10 (the audit-skip rules made structural for safety-critical)
- the GRC per-topic budget machinery (the auditor pool is reserved within it)
reachable_from:
- the AUDITING path: gate_freeze_check.py runs over the manifest; a row missing a traceability field or an UNKNOWN result FAILS  [PRODUCTION]
- an orchestrate run on a safety-critical changeset: the audit-skip exception is refused without separate logged confirmation  [PRODUCTION]
stub_ban:
- NO manifest row missing any of the 4 traceability fields (gate_freeze_check.py FAILs it)
- NO UNKNOWN / indeterminate gate result treated as pass (ND-3)
- NO weakening of the existing R8 freeze (ADD fields only)
- NO LLM judging traceability (declarative + deterministic, TD-3)
- NO orchestrate audit-skip on a safety-critical changeset without a separate logged confirmation (managerial independence)
- NO orchestrator draw-down of the reserved auditor budget pool (financial independence)
frontend_pair: N/A — harness manifest + doctrine; the trace matrix is a generated report. Justified.
observable_effect:
- trigger: "run `python .claude/tools/gate_freeze_check.py --trace` over a manifest where one gate row is missing its technique field, and attempt an orchestrate audit-skip on a changeset that edits a frozen gate"
- expect_artifact: "the freeze check FAILS the incomplete row and names the missing field; the `--trace` matrix lists every gate -> objective -> technique -> evidence; the orchestrate audit-skip on the safety-critical changeset is REFUSED (managerial independence) and logged"
- expect_artifact (NEGATIVE, R1/R2): "a fully-populated manifest passes and emits the matrix; a gate that returns UNKNOWN is reported FAIL not pass; an orchestrate run that tries to fund executor work from the auditor pool is blocked (financial independence); a non-safety-critical changeset CAN still use the H-10 skip (the carve-out is scoped, not a blanket removal)"

## Grounding
- W7 do333-case-studies-interface.md (Cofer/Miller 2014: the verification-objective traceability + the REPLACE/AUGMENT/CANNOT-SATISFY classification; indeterminate never a pass) -> E1.
- W11 nasa-std-8739-8b-interface.md (2022: IV&V technical/managerial/financial independence + records-retention; auditor!=executor + frozen gate already satisfy technical + retention; the financial + managerial gaps are the measured net-new) -> E2.
- precedent: maintainability-review.md unknown=FAIL (the ND-3 disposition this mirrors); wiring-enforcement.md R8 freeze (extended); orchestrate-protocol.md H-3 (the prose ban made structural).
- LIVE SSB baseline 268/577 — J-70 adds none (harness/doctrine).

## COMPOSES-WITH
gate_freeze_check.py + .claude/gate-manifest.txt (the freeze J-70 enriches), every frozen checker (J-65/J-68 + the existing family — each declares a traceability row), orchestrate-protocol.md (H-3/H-10 made structural), main-llm-role.md (auditor!=executor, the financial/managerial independence formalized), the GRC budget machinery (the reserved auditor pool). Restates none.

## Open questions / risks
- the auditor budget pool must be real, not advisory — if it is just a number the orchestrator can still override, the financial-independence gap is not closed; decide the enforcement point at edit (a hard reservation the orchestrate run cannot allocate from), and treat a raid attempt as a logged BLOCK.
- "safety-critical changeset" scope: too broad and every orchestrate run loses the H-10 skip; pin it to {touches a frozen gate, a privileged Supervisor-plane fn, a P10-SAFETY-CRITICAL module} and freeze that definition (R8) so a run cannot quietly shrink it.
- traceability honesty: a row can declare technique=proof while the gate is actually a name-grep — the `--trace` matrix is only as honest as the rows; cross-check each technique claim against the actual checker at authoring (and J-68's real proofs vs J-65's static legs are the calibration).

## Next
J-70 makes the whole wave auditable: every gate J-65..J-71 ships declares its objective/technique/evidence so the trace matrix shows no property is silently unverified (ND-3), and the IV&V riders keep the auditor that runs J-65/J-66/J-68 structurally independent of the run that produced the code. It is the assurance discipline the headline gates hang on.

End of J-70.
