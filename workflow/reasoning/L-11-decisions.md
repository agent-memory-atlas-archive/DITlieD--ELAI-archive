L-11 Output Quality Enforcement System — Decision Log

Plan: docs/thefinalglance/L-plan/L-11_output_quality_enforcement_system.md
Walk: opus-backlog-walk, sessions ending s-fallback-zenov-opus-1786278991-1604086-5

Shipped: slice 1 (0e48769), slice 2 (e8c4858), slice 3 (5367355), slice 4 (3be1b49),
slice 5 (6e3d9f9), slice 6 (7d6380b). Task 5 out-of-band half recorded below.
Task 6 designed and ratified, unbuilt.

---

1. The SOLID checker lives in .elai_cc/tools/, not .claude/tools/ as the plan text said

Why: .claude/ is untracked in this repo (the .gitignore is a whitelist whose first line
is `*` and whose only source allowance is .elai_cc/). A checker authored under .claude/
could never be committed, so it could never be reviewed, diffed or reverted. The frozen
manifest still names it and freezes it by path, which is what R8 actually requires. The
manifest itself stays untracked, because it is out-of-band gate config by definition.

2. The call graph comes from elai.map sound references, never a leaf-name grep (R5)

Why: the plan bans a grep-derived call graph. The checker reads CALL_RESOLVED, SCIP_REF,
TYPE_REF, DEF_METHOD, DEF_TRAIT and IMPL_TRAIT rows. Identifier matching is tokenized, so
`Foo` no longer matches `FooBarBaz`. This is also why Task 6 cannot use a GrepPattern
rules-engine predicate: the same ban applies to the rule that carries the verdict.

3. OCP and LSP are named residue, not silent gaps (OQ-4)

Why: neither is mechanizable from a symbol graph. They appear in the json `uncovered`
list on every run, so a reader of the artifact sees what was not checked instead of
inferring coverage from an empty findings list.

4. Thresholds are provisional constants with a --calibrate path, not literature numbers (ID-5)

Why: SOLID papers supply no numbers for this repo. --calibrate prints the repo's own
fan-out, method-count and trait-method distributions so the constants get replaced by
measured ELAI values. The promotion bound is already in code and is not restated here:
FPR_ACTIVE_THRESHOLD = 0.05 and TPR_ACTIVE_THRESHOLD = 0.80 at crates/rules/src/lifecycle.rs.

5. Write time gets only the deterministic MR floor; SOLID stays at audit time (OQ-2)

Why: the write-time gate sees one candidate source and no pre-edit size, so it cannot
observe the MR-1 boundary CROSSING that a block requires, and blocking on size alone
would fire on every edit to an already-large file (the A-1 thrash the plan forbids).
MR-1 is therefore WARN only at write time and MR-2 blocks, because MR-2 is fully
observable in the candidate text. The audit-time python checker keeps the real verdict.

6. The gen-time god-object predicate returns WARN and never BLOCK (OQ-5)

Why: StructuralRetry accepts only PASS, so a WARN already forces a re-roll of the fill in
loop. BLOCK would add a hard failure without adding any correction the WARN does not
already produce. This is the any-model leg: a weak backend that writes a god-object gets
corrected during generation instead of at review.

7. The C-31 responsibility budget ships observe-only, and the doc comment says which rung
   production actually reaches

Why: FilePlan::validate has no production caller today, so the cap it enforces is
test-reachable only. The rung production reaches is generate_from_file_plan, which
records the over-budget file set on SkeletonSpec and never blocks. Writing that down was
the point: a comment claiming coverage the code does not have is the failure mode the
remediation directive names.

8. The QDELTA quality arm is an ARM on the existing runner, not a second runner

Why: the plan says compose, do not fork. The arm reuses the statistical floor, the
degenerate-corpus guard, the bootstrap CI, the report writer and the beacon writer. It
lives in a new focused submodule because bench_qdelta.rs sits at 990 lines against the
MR-1 1000-line floor. A missing quality cell is a hard error naming the row index,
because a defaulted cell fabricates a zero violation rate and then reads as evidence.

9. Slice 6: a zero-row scan is a scan-config error, not a PASS

Why (this is the load-bearing decision of the whole unit): the checker compared the map's
location column to the caller's file names as raw strings, and the two are written
against different roots. elai.map names `crates/...` while every caller of a frozen
checker names `.elai_cc/crates/...`. Nothing matched, so the same 14,661-line file
returned two findings under one spelling and zero under the other, and the zero came back
as verdict PASS with an empty residue. The base is now chosen by scoring candidate
directories against the map's own first locations, and two guards exit 2 instead of
passing: no candidate base resolves any probed location, or the changeset named files
that exist on disk but match zero map rows. A gate that reports PASS having read no rows
does not merely miss a violation, it manufactures evidence that there was none.

Consequence accepted: exit 2 now fires on changesets that add files the map predates. A
refused run is the correct answer there, and the auditor step says so explicitly and
requires the unscanned files to be named as residue.

10. The auditor's SOLID authority is agent prose plus the frozen manifest, not a Rust seam

Why: this is precedent, not convenience. maintainability_check.py and
production_caller_check.py have no production Rust caller either. Their authority is
exactly (a) a numbered step in .claude/agents/auditor.md and (b) a frozen row in
.claude/gate-manifest.txt. SEAM 4 got the identical shape: step o in the auditor, plus
the manifest row, plus gate_freeze_check.py --lock (71 files, verify OK). The one Rust
list of auditor checkers, build_audit_checker_invocations in
crates/orchestrator/src/benchmarks/gate_tax_audit_leg.rs, measures checker COST for K-4
and records no verdict, so it is not the authority seam and was left alone.

11. Task 6 is designed and ratified, and deliberately unbuilt in this session

Verdict BUILD-WITH-SEAM. Proposal: conductor/opus_walk/evidence/L-11/seam-design-proposal.md.
The earlier recon concluded the seam did not exist, on the grounds that MetricGate reads an
eval-context map nothing fills and GrepPattern is banned by R5. Both statements are true and
both were the wrong frontier. The design turn found the real one: ProjectRuleSeeder::seed_rules
(crates/rules/src/project_rule_seeder.rs) is already called in production at
crates/cli/src/runtime_builder.rs, and RuleCurator::run_promotion_tick
(crates/orchestrator/src/rule_curator.rs) is already spawned in production at
crates/cli/src/knowledge_stack.rs. The controller re-verified every cited seam live before
ratifying, including that SHADOW is a real LifecycleStatus and that the OQ-3 bound already
exists as named constants.

Ratified slice order: (1) seed the three SOLID rows at SHADOW in project_rule_seeder;
(2) add the SHADOW sweep to run_promotion_tick; (3) after that, have the auditor gate read
each row's status so SHADOW downgrades the checker's BLOCK to report-only, and write the
rule_metrics window rows.

Honest residue carried from the design: fpr and tpr need someone to say which fires were
true. Fixture-labeled QDELTA quality-arm windows can supply that; unlabeled production runs
cannot. So the OQ-3 ">=3 real runs" bound reads, for now, as three QDELTA quality-arm runs
over real changeset fixtures. Promotion from live traffic needs an adjudication source that
does not exist yet.

12. Measured calibration residue (run 2026-08-09 with --calibrate over elai.map)

    fan_out           count 4797   median 0   p90 1   p99 14
    method_count      count 4797   median 0   p90 0   p99 2
    trait_method_count count 179   median 0   p90 0   p99 0

Three things follow, and they are recorded rather than quietly carried:

- ISP is VACUOUS at the shipped threshold. K_TRAIT_METHODS is 10 and the repo's
  trait-method count does not reach 1 even at p99, so the ISP arm can never fire here. It
  is a rung that exists and observes nothing. It must be re-thresholded from this
  distribution before anyone reads an empty ISP result as an ISP verdict.
- The method-count half of the SRP rule is likewise unreachable (K_METHODS 10 against a p99
  of 2), so SRP reduces in practice to fan_out > 10 AND lines > 200.
- SRP is reachable but rare: fan-out passes 10 only near p99, which is the intended shape
  for a god-object rule and is the reason the rule ships SHADOW under OQ-3 rather than as a
  default BLOCK.

This is what ID-5 asked for. The constants stay provisional until they are replaced from
these measured numbers out-of-band, with a human review and a gate_freeze_check --lock
re-record, because the checker is frozen (decision 10).

13. Plan-level deterministic check status at close

wiring_contract_check.py --run-trigger reports the observable_effect trigger as FAIL with
`/bin/sh: Syntax error: "(" unexpected`, because the trigger line is prose describing an
audit, not an executable command, and expect_ssb reports FAIL because no ssb.db was present
in this run. Neither is a defect introduced by slices 1 to 6: both need a real audited run
of the product to discharge, which is Task 6 slice 3 territory. They are named here so a
later reader does not mistake an unrun check for a passed one.
