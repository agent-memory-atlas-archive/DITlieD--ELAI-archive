# I-39: plan state from repo — `elai plan status` derived from journal + fragments, migration replay guard

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever ANTI-DRIFT | grounding STRONG | nature BUILD | effort S-M | WAVE 1

- Premise: plan status is currently maintained by hand across five drifting sources (CLAUDE.md, session
  handoffs, plan index files, grounding maps, memory-bank). Every SCHEMA_VERSION snapshot went stale within
  days of authoring. `elai plan status` is absent (IN-10). I-39 builds the command AND adds the journal-rot
  guard: a migration replay test that proves old-schema SSB rows (the trust product) can still be decoded
  by current code.

## Steal/build
Two deliverables in one session:

`elai plan status`: a CLI command that derives per-plan state from three deterministic sources:
  (a) the wiring_contract_check.py --static verdict for the plan file (PLANNED/IN-PROGRESS/DONE/BLOCKED)
  (b) the SSB journal: the presence of the plan's observable_effect SSB rows (verified vs absent)
  (c) plan-status fragments from the s-plan-audit-2026-06-04 FINISH files (the hand-audit evidence the
      command ingests and then supersedes for future runs)
  Each line carries a "verified against <commit-hash/map-date>" anchor. Hand-tallies demote to build
  artifacts on first run; the command is the single source of truth thereafter. Output is byte-identical
  from the same journal snapshot (GD-2).

JOURNAL-MIGRATION REPLAY GUARD: a single contract test that opens a REAL archived old-schema ssb.db
(from F-13's session archive) through the current code path and decodes N rows to completion. This is
the RRA-rot guard: old RRAs are the trust product; silent decode failure is silent loss of the evidence
chain. A corrupted or undecodable archive FAILS loudly (R2), never silently skips rows.

## Existing ground truth
- IN-10 (grounding map): NO `elai plan status` command — confirms BUILD
- PLAN-asserted: SSB viewer read paths (the journal query surface); executor re-greps per W-7
- PLAN-asserted: s-plan-audit-2026-06-04 FINISH files + wiring_contract_check.py (the evidence sources); executor re-greps per W-7
- PLAN-asserted: F-13 session archive (the old-db source for the migration replay test); executor re-greps per W-7
- PLAN-asserted: E-44 migration machinery (the decode path the replay test exercises); executor re-greps per W-7

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/cli/src/plan_status_cmd.rs (~180 LOC): derive_plan_state() (three-source query), format_status_line() (with evidence anchor), run_plan_status()
- `elai plan status [--letter X] [--id I-N]` subcommand
- PlanStateDerived SSB beacon (flat, carries letter + session_id + state + evidence_anchor)
- CONTRACT TEST: crates/contracts/tests/i39_plan_state_migration_contract.rs — opens a REAL F-13 archived ssb.db, decodes N rows, asserts the decode produces N well-formed rows and no silent skip; a corrupted archive FAILS loudly
- decision doc: workflow/reasoning/I-39-decisions.md

DEFER (Tier-2 / gated):
- Automatic plan-status sync on every git push (CI integration — infrastructure, not correctness)
- Cross-plan dependency graph visualization (post Wave 4 data)

## Integration
- new crates/cli/src/plan_status_cmd.rs (~180 LOC)
- seam to SSB viewer read paths: the journal query (GD-1)
- seam to wiring_contract_check.py --static: run as a subprocess; parse the exit code + stdout (GD-1)
- seam to F-13 archive: the migration replay test reads the archived db at a repo-relative path (GD-1)
- seam to E-44 migration machinery: the replay test uses E-44's decode path (GD-1)
- frontend_pair: N/A — CLI output + doc artifact; no dashboard surface (W-6 justified: plan status is a developer-facing read, not an agent-feedback signal). Justified.

## WIRING_CONTRACT
produces_symbols:
  - crates/cli/src/plan_status_cmd.rs::derive_plan_state() (the three-source query)
  - crates/cli/src/plan_status_cmd.rs::run_plan_status() (the CLI entry point)
  - ssb action_type::PlanStateDerived (flat beacon; migration-replay reuses existing rows) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - crates/contracts/tests/i39_plan_state_migration_contract.rs (the migration replay guard)
consumes_symbols:
  - SSB viewer read paths (journal query; GD-1)
  - wiring_contract_check.py --static (external subprocess; exit code + stdout parsed; GD-1)
  - F-13 session archive (the archived old-schema ssb.db; GD-1)
  - E-44 migration decode path (the replay test entry point; GD-1)
  - s-plan-audit-2026-06-04 FINISH fragments (the hand-audit evidence source, demoted to artifact; GD-1)
reachable_from:
  - `elai plan status` CLI invocation -> run_plan_status() -> derive_plan_state() -> PlanStateDerived SSB beacon  [PRODUCTION CLI surface]
  - crates/contracts/tests/i39_plan_state_migration_contract.rs test -> E-44 decode path -> N well-formed rows  [CONTRACT TEST — real archived db, not a mock]
  - test callers in crates/contracts/tests/ satisfy R7 for the migration guard ONLY (the guard's subject is the archived db, not a production entrypoint)
stub_ban:
  - NO mock ssb.db in the migration replay test (R7 analog: the contract test opens a REAL F-13 archive; a mock proves nothing about migration correctness)
  - NO silent row-skip on decode failure (R2: the guard exits nonzero if any row fails to decode)
  - NO hand-maintained plan status fields in the output (the command derives state mechanically; a hardcoded "DONE" field is a zero-lazy-code violation)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  N/A — CLI output + doc artifact; plan status is a developer-facing read, not an agent-feedback dashboard signal. W-6 justified. Justified.
observable_effect:
  trigger: "elai plan status --letter I (generates per-session states for I-plan) AND the migration replay contract test"
  expect_ssb: "PlanStateDerived beacon per queried session: state field, evidence_anchor field pointing to the wiring_contract fragment path + journal row — byte-identical output from the same journal snapshot (GD-2)"
  expect_artifact: "the migration replay contract test opens the REAL F-13 archived ssb.db and asserts N decoded rows with no silent skip (R2); a corrupted archive exits nonzero with a named error (R2 — not a skip); `elai plan status` output carries evidence anchors and regenerates identically (GD-2)"

## Grounding
- IN-10 (grounding map): `elai plan status` absent — confirms BUILD
- I-0 INDEX header: "every SCHEMA_VERSION snapshot went stale within days" — the motivation is grounded in the observed history of the I-0 INDEX itself
- QAfromfable §6 (the I-39 source): plan-state from repo as an anti-drift instrument
- F-13 session archive (PLAN-asserted; executor re-greps per W-7): the real old-schema db for the migration test
- E-44 migration machinery (PLAN-asserted; executor re-greps per W-7): the decode path under test
- R2 (wiring-enforcement.md): dead evidence path must scream, not shrug — the migration guard's design rationale
- SSB: +PlanStateDerived (flat beacon; migration-replay reuses existing rows) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)

## COMPOSES-WITH
- SSB viewer read paths — compose the journal query, do NOT re-implement the SSB read layer
- wiring_contract_check.py — run as subprocess (the tool is the authority); do NOT re-implement its logic
- F-13 session archive — reference the archived db path; do NOT copy or re-encode the archive
- E-44 migration decode path — the replay test calls into E-44's decode function; do NOT re-implement migration
- s-plan-audit-2026-06-04 FINISH fragments — ingest as the bootstrap evidence; the command supersedes them on subsequent runs

End of I-39.
