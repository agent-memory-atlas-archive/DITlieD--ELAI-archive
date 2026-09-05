# J-65: power-of-ten generated-code gate — the code ELAI WRITES is held to NASA JPL rules, deterministically

- Track J (NASA-gates wave) | lever CANT-SHIP-UNSAFE | grounding STRONG (power-of-ten-interface.md W1, all 10 rules verbatim) / EXTERNAL (the C origin) | nature BUILD | effort M | WAVE NASA-GATES-1 | depends none (sibling of maintainability_check.py)
- HEADLINE of the wave. The user's "NASA-grade generated code" in one deterministic checker.

- Premise: ELAI's auditor already runs maintainability_check.py (MR-1 file-size, MR-2 casts, MR-3 long-fn, MR-4 branching), production_caller_check.py (wiring), traced_test_assert_check.py (fail-open asserts). NONE of them cover the Holzmann Power-of-Ten rule set (acyclic call graph, statically-bounded loops, assertion density, every-return-checked, zero-warnings-as-error). GROUNDED GAP (power-of-ten-interface.md DETERMINISTIC-GATE RULES, W1): the P10 rules are exactly the safety-critical-code discipline the generated-code path lacks. The fix is a NEW frozen checker the AUDITOR runs over the ELAI-GENERATED Rust/Python/TS in a changeset — off-agent, sibling of maintainability_check.py, never run by the executor on its own diff (main-llm-role: main model never runs the gate). P10 was written for C, so the Rust/Python/TS analogs are APPROXIMATIONS (ND-2): the binary rules (P10-4/7/10) are BLOCK, the approximate ones (P10-2 loop-bound, P10-9 dyn-dispatch) stay WARN. Ships OBSERVE-ONLY first (GD-4) and promotes the hard subset only after the FP rate is bounded on real diffs.

## Steal/build
1. .claude/tools/power_of_ten_check.py (new, stdlib-only per house): `--files <changed> --base HEAD --json`; named constants for every threshold; frozen in .claude/gate-manifest.txt (R8). Rules (from power-of-ten-interface.md, all 10 there verbatim):
   P10-1 acyclic call graph — no direct/indirect recursion in agent/task loop modules (BLOCK; via the B-29/codegraph cone over the changed symbols, GD-1 reuse, no new traversal).
   P10-2 every loop statically bounded — flag `loop{}` / `while cond{}` / `while True:` with no literal/`.len()` bound and no break-after-N (WARN, approximation).
   P10-3 no heap alloc in the hot dispatch path (WARN; flagged modules only).
   P10-4 fn <= 60 lines on flagged safety-critical modules only — a TIGHTENING of MR-3's 80, NEVER applied globally (BLOCK on flagged modules).
   P10-5 assertion density >= 2 per fn + ban trivially-true asserts (`assert(true)`, `assert!(1==1)`) (WARN).
   P10-7 every non-void return checked — Rust `clippy -D unused_must_use`, Python `mypy strict`, TS noUnusedLocals (BLOCK).
   P10-8 cyclomatic complexity <= 2 on non-test (WARN).
   P10-9 no fn-pointer / `dyn Fn` dispatch where a concrete type fits (WARN, approximation).
   P10-10 zero-warnings-as-BLOCK — `cargo clippy -D warnings`, `ruff`/`tsc`/`eslint --max-warnings 0` (BLOCK).
2. CONSUMER: the auditor invokes it in AUDITING over the changeset, exactly like maintainability_check.py; a P10-BLOCK class fails the audit verdict. The executor MAY self-run it (necessary-not-sufficient self-verify) but the auditor's off-agent re-run is the authority.
3. SCOPE FLAGGING: a per-module `// P10-SAFETY-CRITICAL` marker (or a frozen module list in the manifest) selects where the BLOCK subset (P10-4 fn<=60, full assertion density) applies; everywhere else the gate runs the language-universal subset (P10-7/P10-10) only. No global MR-3 tightening (the plan's explicit RISK).
EXCLUDED: any LLM judgment of "is this code NASA-grade" (irreducibly advisory, routes to the maintainability/critic lens, not this gate); auto-fixing a violation (surface only); applying the C-specific rules (P10-3 hot-path alloc, P10-9 concrete-dispatch) as BLOCK on Rust/Python where they are approximations (ND-2: a layer that cannot soundly decide a property does not BLOCK on it).

## Existing ground truth (grep-confirmed 2026-06-22; re-grep at edit, W-7/W-9)
- .claude/tools/maintainability_check.py (MR-1..MR-4), production_caller_check.py, traced_test_assert_check.py, python_wire_check.py, wiring_contract_check.py, spec_link_check.py, gate_freeze_check.py — all present; the frozen-checker family J-65 joins.
- .claude/gate-manifest.txt present — the freeze list R8 reads; J-65 + its thresholds get a row (and the traceability fields J-70 adds).
- F-4 AntiFabricationGate at task_runner.rs (feature-map 1.6 [WIRED]) — the auditor-owned deterministic-check precedent on the post-EXECUTING boundary.
- B-29 symbol-level impact API + codegraph-elai MCP — the SHIPPED call graph J-65 reuses for P10-1 recursion detection (GD-1; re-implementing a call graph = P0).

## Integration
- NEW .claude/tools/power_of_ten_check.py (~300 LOC, stdlib-only + shelling clippy/ruff/tsc which the auditor already has); a row in .claude/gate-manifest.txt (frozen); a finding-tag set the auditor emits. NO .elai_cc change, NO SSB (harness leg, JD-1 evidence — the off-agent artifact is the checker's JSON verdict + exit code the executor did not author).
- frontend_pair: N/A — harness tooling (the audit verdict surfaces through the existing AuditTimeline). Justified.
- TAGS: C-P10-RECURSION / C-P10-LOOP / C-P10-ASSERT / C-P10-RETURN / C-P10-WARN / C-P10-FNLEN / C-P10-CFG / C-P10-DISPATCH. BLOCK on RECURSION/RETURN/WARN(P10-10)/FNLEN(flagged); WARN otherwise.

## WIRING_CONTRACT
produces_symbols:
- .claude/tools/power_of_ten_check.py (the checker; `--files`/`--base`/`--json`, named-constant thresholds)
- a .claude/gate-manifest.txt row for power_of_ten_check.py (frozen; R8)
consumes_symbols:
- B-29 impact API / codegraph-elai (the call graph for P10-1; NOT reauthored, GD-1)
- the auditor's existing clippy/ruff/tsc invocation surface (P10-7 / P10-10 shell out)
- the orchestrator-captured changeset diff (the off-agent input the executor did not author, R4)
reachable_from:
- the AUDITING FSM transition on a real `elai` task/fix run (auditor-driven, off-agent)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7); the executor self-run is necessary-not-sufficient
stub_ban:
- NO LLM call inside the gate (deterministic AST/regex/clippy only; TD-3)
- NO auto-fix of any flagged construct (surface only)
- NO global tightening of MR-3 80->60 (only `// P10-SAFETY-CRITICAL`-flagged modules; the plan's RISK)
- NO BLOCK on the approximate rules (P10-2/P10-3/P10-9) on Rust/Python — WARN only (ND-2 soundness)
- NO promotion of any rule from WARN to BLOCK before the FP rate is bounded on >=2 real plan cycles (GD-4) + a manifest note
stub_ban (evidence): an UNKNOWN/parse-failure verdict is FAIL, never SKIP (ND-3; the dead path screams, R2)
frontend_pair: N/A — harness checker; the verdict rides the existing audit surface. Justified.
observable_effect:
- trigger: "the auditor runs `python .claude/tools/power_of_ten_check.py --files <a hand-seeded violating fixture> --json` over a changeset that contains an unbounded `loop{}`, an 80-line fn in a P10-SAFETY-CRITICAL module, an `assert!(true)`, and a `let _ = result;` on a #[must_use]"
- expect_artifact: "exit 1 with C-P10-LOOP (WARN), C-P10-FNLEN (BLOCK, flagged module), C-P10-ASSERT (WARN), C-P10-RETURN (BLOCK) tags in the JSON; the BLOCK classes fail the audit verdict"
- expect_artifact (NEGATIVE, R2): "a clean fixture exits 0; a fixture the checker CANNOT parse exits FAIL (never 0); a non-flagged module with a 70-line fn does NOT BLOCK (no global tightening); the executor running it on its own diff and reporting PASS is NOT the authority — the auditor's off-agent re-run is (gate_freeze_check.py proves the checker was not edited by the run)"

## Grounding
- W1 power-of-ten-interface.md (all 10 rules verbatim, the DETERMINISTIC-GATE RULES section); origin Holzmann 2006 "The Power of Ten" (NASA/JPL), C-targeted — the Rust/Python/TS analogs are this plan's approximations, graded per ND-2.
- precedent: maintainability-review.md MR-1..MR-4 (the off-agent frozen-checker pattern this mirrors exactly) + the F-4 cant-lie gate (auditor-owned deterministic check).
- kani/clippy/mypy toolchain present in the auditor environment (re-verify at edit).
- LIVE SSB baseline 268/577 (ssb.rs:1503/1508/62) — J-65 adds none.

## COMPOSES-WITH
maintainability_check.py (sibling frozen checker; J-65 is the NASA-rule leg the MR-set lacks), production_caller_check.py + python_wire_check.py (wiring axis — orthogonal: does it RUN), traced_test_assert_check.py (fail-open-assert axis), B-29 impact API (P10-1 call graph), gate_freeze_check.py + .claude/gate-manifest.txt (R8 freeze), J-70 (adds the DO-333 traceability row this checker declares). Restates none.

## Open questions / risks
- FP RATE on real diffs is unknown — P10-5 (assertion density >= 2) and P10-8 (cfg <= 2) are noisy on idiomatic Rust; both ship WARN and are measured before any BLOCK promotion (GD-4), and the OBSERVE-ONLY window LOGS the would-be-BLOCK count (no silent calibration).
- SCOPE of "safety-critical module" — the `// P10-SAFETY-CRITICAL` marker set must be small and frozen, or P10-4's fn<=60 churns the whole tree; decide the initial set at edit (candidates: ssb/payloads.rs, the FSM transition fn, gate predicates — the same small core J-68 verifies).
- P10-1 recursion via the B-29 cone has the DKG-completeness false-negative (build-time-generated symbols have no edges, the J-16 caveat) — DECLARE the coverage and FAIL-LOUD on an unmappable changeset, never silently pass (R2).

## Next
J-65 is the headline deterministic gate; J-70 gives its manifest row the DO-333 traceability fields (objective/technique/evidence/satisfaction) so the requirements->gate->evidence matrix is complete; J-68 formally verifies the SAME small core J-65's P10-4/P10-5 most aggressively flag. The OBSERVE-ONLY -> measured-promotion path is the template the rest of the wave's default-OFF gates follow.

## WARGAME BLOCKS

EXPECTED_OBSERVATION: after writing .claude/tools/power_of_ten_check.py and adding its row to .claude/gate-manifest.txt, the executor runs python .claude/tools/power_of_ten_check.py --files <hand-seeded violating fixture> --base HEAD --json and observes exit 1 with JSON findings carrying tags C-P10-LOOP (WARN), C-P10-FNLEN (BLOCK, flagged module), C-P10-ASSERT (WARN), C-P10-RETURN (BLOCK). A second run on a clean fixture exits 0. The auditor off-agent re-run on the SAME fixtures produces the identical verdict (gate_freeze_check.py proves the checker was not edited between runs, R8).

ON_FAIL: if the checker exits 0 on the violating fixture, cause = a rule-implementation bug (regex too narrow, AST walk missed the construct), counter-move = fix the rule implementation and re-run on the fixture. If the checker exits 2 (scan error), cause = a parse failure on the fixture (malformed Python/TS, unhandled syntax), counter-move = handle the parse error in the checker (ND-3: UNKNOWN/parse-failure is FAIL, never SKIP). If production_caller_check.py reports the checker itself as DARK, cause = the checker has no production caller (the auditor invokes it but the checker is a script not a pub fn), counter-move = this is expected for a harness tool (the WIRING_CONTRACT reachable_from names the AUDITING transition, not a code symbol; the auditor confirms the manifest row exists).

FORKS: none. The plan is a single deterministic build: one checker file, one manifest row, one finding-tag set. No decision points for the executor.

ABORT: (1) if B-29 impact API / codegraph-elai is unreachable (P10-1 recursion cone depends on it, GD-1 reuse), STOP and handoff. Do not re-implement a call graph. (2) if clippy / ruff / tsc are not installed in the auditor environment (P10-7 / P10-10 shell out to them), STOP and handoff. Do not substitute a different linter. (3) if the hand-seeded violating fixture cannot be constructed (the plan observable_effect spec is internally inconsistent), STOP and handoff. Do not weaken the assertion.

VERIFY: (1) the auditor runs python .claude/tools/power_of_ten_check.py --files <violating fixture> --json off-agent and observes exit 1 with the expected finding tags (C-P10-LOOP, C-P10-FNLEN, C-P10-ASSERT, C-P10-RETURN). The off-agent artifact is the checker JSON verdict the executor did not author (R4). (2) the auditor runs the checker on a clean fixture and observes exit 0. (3) the auditor runs python .claude/tools/gate_freeze_check.py and confirms the frozen set now includes power_of_ten_check.py (R8: the gate is frozen and read-only to the executing run; the re-lock is out-of-band). (4) the auditor confirms the gate-manifest.txt row exists and names the checker.
RUN: `python .claude/tools/power_of_ten_check.py --files <violating fixture> --json` exits 1; `python .claude/tools/power_of_ten_check.py --files <clean fixture> --json` exits 0; `python .claude/tools/gate_freeze_check.py` exits 0 with power_of_ten_check.py in the frozen set.

REDTEAM: attack 1, the executor ships a checker that always exits 0 (rubber-stamp). FAILED: the auditor off-agent re-run on the violating fixture would exit 0 instead of 1, caught by VERIFY step 1. Attack 2, the checker edits gate-manifest.txt to remove itself from the frozen set during the run. FAILED: gate_freeze_check.py runs after the commit and detects the frozen-set mutation (R8); the executor cannot edit the frozen set because R-3 (agent-dispatch-preamble) bans editing gate files. Attack 3, the checker passes on the clean fixture but silently skips unparseable files (fail-open). FAILED: the plan stub_ban declares an UNKNOWN/parse-failure verdict is FAIL, never SKIP (ND-3; R2); the auditor fixture includes an unparseable file and asserts exit 2 (FAIL), not 0. Attack 4, the checker applies P10-4 fn<=60 globally instead of only on P10-SAFETY-CRITICAL-flagged modules. FAILED: the auditor fixture includes a 70-line fn in a non-flagged module and asserts NO C-P10-FNLEN finding for it; a global tightening would produce a false BLOCK. PATCH: none of the attacks succeeded, no patches needed.

INTEGRATION_SMOKE: n/a. Single-unit harness plan (one checker file + one manifest row, no cross-seam integration; the auditor off-agent re-run IS the integration proof).

End of J-65.
