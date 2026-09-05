# K-1: problem-quantification dossier + claims policy — bind the EXTERNAL problem numbers to ELAI's MEASURED numbers, and make "no claim without a number" enforceable

STATUS: PLANNED (WAVE KW-1; E-23-class, near-zero code; doc + a deterministic claims-policy lint — the dossier itself assembles as A-1/A-2/A-3 numbers land)

Objective: the 2026-06 research handed ELAI the problem-quantification it lacked (KS-2 §1.4) and the
operational corollary "the number is everything." This session ships TWO artifacts: (1) a DOSSIER that
binds the verified EXTERNAL problem numbers (KW-1..KW-5) to ELAI's own MEASURED numbers as they land
(the A-1 kill-gate verdict KC-1, A-2 SeCodePLT, A-3 SAFEDELTA delta); (2) a CLAIMS POLICY doc + a
deterministic LINT that fails any externally-visible claim lacking either a measured journal-recomputable
number or an explicit [EXTERNAL]/[DESIGN] tag. Hard boundary it must NOT cross: it does NOT run A-1/A-2/
A-3 itself as new code (A-1 is SHIPPED, INVOKE it — KC-1; A-2/A-3 are Section-A pointers), it does NOT
fabricate a measured number before the bench produces one (KD-1), and it does NOT re-implement the I-43
honest-wording table — it cites it as normative.

Estimated effort: ~1 session (~3-4 h). In-process at gate time (the lint is a deterministic text check
over README/positioning copy; no network, no model). The dossier's measured rows fill in as the
bench numbers land — the FILE is authored now, the numbers are pointers + placeholders gated by KD-1.
Pre-requisites: A-1 SHIPPED (`elai bench claim-scope`, args_bench.rs:70 / ssb.rs:9127 — KG-1); the KW
set web-verified (grounding map KW-1..KW-5). A-2/A-3 numbers are NOT prerequisites for authoring the
dossier shell — they are filled in as they land, each gated by KD-1.
Parallel-safe with: K-8 (audit+rule), K-9 (cadence) — disjoint files. NOT parallel with K-7 (shares the
claims-policy lint; K-7 consumes it).
Source: K-MASTER-BRIEF §3.2 K-1 [KS-1]; KS-2 §1.4; grounding map KW-1..KW-6, KD-1. Doctrine: K-0_INDEX.md.

## Context
Why this matters: the research converged on exactly SAFEDELTA's methodology (gate-ON vs gate-OFF, hidden
suites, dynamic exploitation — KS-2 §1.4) and gave ELAI citable external numbers proving the problem is
real and large. The dossier is the marketing/grounding artifact that pairs each external problem number
with ELAI's measured answer; the claims policy is the GUARD that the B-40/B-41/B-42 bad-cite incidents
proved ELAI needs at the marketing surface, not just in the journal. Mechanism: a one-row-per-claim
ledger where each row is either (measured ELAI number + recompute pointer) or ([EXTERNAL] + KW source),
and a lint that refuses any positioning string that asserts a number/superlative without one of those.
External numbers (each VERIFIED, grounding map KW set; all tagged [EXTERNAL], never claimed as ELAI's):
KW-1 USENIX 2025 packages 5.2%/21.7%, 58%/43% recurrence; KW-2 arXiv:2507.02976 11x-vulns (135 vs 12);
KW-3 reward-hacking 73.8%/46.8%; KW-4 CodeAct ~50% latency/-60% tokens; KW-5 SpecBench +28pp (exact
value PLAN-asserted, quote at dossier build). KW-6 (DORA/METR/vendor) is PENDING-VERIFY — excluded until
the dossier-build re-verification.

## Existing state (A/B..J coverage)
- A-1 I-43 SHIPPED: `ClaimScopeMeasured` SSB at ssb.rs:9127; CLI `elai bench claim-scope` (args_bench.rs:70,
  parser :1011-1063); emitter cli::bench_claim_scope::run_inner; fixture contract
  i43_claim_scope_fixture_contract.rs (KG-1/KC-1). Its honest-wording table is the normative reference.
- A-2 F-2b SeCodePLT substrate present (bench_seccodeplt.rs + the two smokes — KG-6); the LIVE local-model
  run is the missing leg (W-10 RunPod).
- A-3 G-6 SAFEDELTA substrate present (safedelta_runner.rs + bench_safedelta.rs + the two contracts — KG-4).
- GD-3 (no SWE-bench-Verified headline; ablation framing) and GD-5 (honest residue) are the doctrine the
  claims policy operationalizes onto the marketing surface.
- E-23 is the positioning-artifact precedent (prose, near-zero code).

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KW-1..KW-6, KC-1, KD-1).
2. K-MASTER-BRIEF §2.4 (the external-number list) + §3.2 K-1 + KS-2 §1.4.
3. I-43's plan file (the honest-wording table + the claim-scope CLI/output shape) — re-grep
   args_bench.rs:1011-1063 + the ClaimScopeMeasured payload live (W-7).
4. The G-6/F-2 plan files for the measured-number shapes the dossier will cite (SAFEDELTA delta,
   SeCodePLT pass/fail).
5. CLAUDE.md feedback_paper_citation_verify + KD-1 (the binding rule this session enforces).

## Tasks (WAVE KW-1 — dossier + claims policy)
1. Author the dossier (docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md or workflow/reasoning/):
   one row per problem -> (external number [EXTERNAL]+KW source) paired with (ELAI's measured answer +
   recompute pointer OR an explicit "PENDING — fills from A-N"). ~120-180 lines. Every external number
   re-verified at this authoring per KD-1 (the KW set is already verified; re-quote SpecBench's exact
   value + page anchor, KW-5).
2. Author the CLAIMS POLICY doc: no externally-visible claim ships without a measured journal-recomputable
   number or an explicit [EXTERNAL]/[DESIGN] tag; the honest-wording table (trace-authentication on
   cloud vs deterministic-replay where J-13 caps pass) is normative. ~40-60 lines.
3. Build the claims-policy LINT (.claude/tools/claims_policy_lint.py, stdlib-only, OR an `elai`-side
   check): scan README/positioning copy for number-or-superlative patterns; each must be adjacent to a
   measured-pointer or an [EXTERNAL]/[DESIGN] tag; exit nonzero on a violation (fail-closed). ~80-120 LOC.
4. Wire the lint into the audit path (the auditor runs it over positioning copy) and a conditional-routing
   trigger for "editing README/positioning". Do NOT make the main LLM run it (auditor-applied).
5. NEGATIVE arm: a fixtures file with a seeded unmeasured superlative ("the safest coding agent") and a
   bare untagged external number — the lint must FAIL both.
6. workflow/reasoning/K-1-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (the claim->number ledger)
  - docs/strategy/ELAI-CLAIMS-POLICY.md (the normative no-claim-without-a-number doc)
  - .claude/tools/claims_policy_lint.py (the deterministic fail-closed lint)
consumes_symbols:
  - cli::bench_claim_scope (A-1 verdict — INVOKE, do NOT rebuild; KC-1)
  - benchmarks::safedelta_runner + bench_seccodeplt (A-3/A-2 measured-number sources)
  - the I-43 honest-wording table (normative; not re-implemented)
reachable_from:
  - the auditor's positioning-copy review step runs claims_policy_lint.py [PRODUCTION audit gate]
  - conditional-routing loads the claims policy when README/positioning copy is edited
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO fabricated ELAI measured number before its bench produced it (KD-1)
  - NO external number without an [EXTERNAL]/[DESIGN] tag + its KW source
  - NO claims-policy lint that emits PASS on missing evidence (fail-closed; R1)
  - NO main-LLM self-run of the lint (auditor-applied)
frontend_pair:
  N/A — positioning docs + a CI/audit lint, no dashboard surface. Justified (W-6).
observable_effect:
  trigger: "python .claude/tools/claims_policy_lint.py --files docs/strategy/positioning/*.md"
  expect_ssb: "none (E-23-class; the lint is a deterministic check, not a journaled event)"
  expect_artifact: "the dossier + claims-policy docs exist; the lint exits 0 on compliant copy and
    nonzero on the seeded unmeasured-superlative fixture (the NEGATIVE arm that MUST fail)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-1_problem_quantification_dossier_claims_policy.md
test -f docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md
test -f docs/strategy/ELAI-CLAIMS-POLICY.md
python .claude/tools/claims_policy_lint.py --files docs/thefinalglance/K-plan/_fixtures/claims_negative.md ; test $? -ne 0   # the seeded unmeasured superlative MUST fail
rg -n "\[EXTERNAL\]|\[DESIGN\]" docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md   # every external number tagged
test -f workflow/reasoning/K-1-decisions.md
```

## ACCEPTANCE
- [ ] KD-1 holds: every claim row is measured+recompute-pointer OR [EXTERNAL]/[DESIGN]+KW source.
- [ ] every external number re-verified at authoring (KW set; SpecBench exact value quoted with anchor).
- [ ] the claims-policy lint is fail-closed (missing evidence = nonzero) and auditor-applied (not main-LLM).
- [ ] No new dependency (stdlib-only lint).
- [ ] No new SSB action_type (E-23-class).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the lint runs in the audit path + the negative fixture FAILS, not "the doc exists".
- W-2: N/A (no feature flag). W-3: N/A (no port). W-4/W-6: N/A dashboard; justified.
- W-5: trigger = the real lint command over real positioning copy. W-7: A-1 anchors re-grepped at edit.
- GD-2: each measured row carries a recompute pointer. GD-3: no SWE-bench headline in the dossier.
- KD-1: the lint IS the enforcement of this rule.

## Tests to add
- crates/contracts/tests/k1_claims_policy_contract.rs (~60 LOC) OR a Python test for the lint:
  (a) compliant copy passes; (b) an unmeasured superlative fails; (c) a bare untagged external number
  fails; (d) a correctly [EXTERNAL]-tagged number passes; (e) an empty/missing input fails-loud (R1).

## Files touched
- CREATE: docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (~160 LOC)
- CREATE: docs/strategy/ELAI-CLAIMS-POLICY.md (~50 LOC)
- CREATE: .claude/tools/claims_policy_lint.py (~110 LOC)
- CREATE: docs/thefinalglance/K-plan/_fixtures/claims_negative.md (~10 LOC, the negative arm)
- MODIFY: .claude/rules/conditional-routing.md (~2 LOC: load the claims policy when editing positioning)
- CREATE: workflow/reasoning/K-1-decisions.md (~30 LOC)

## Citations / Grounding
- KW-1 USENIX Security 2025 Spracklen "We Have a Package for You!" (usenix.org/.../spracklen):
  5.2%/21.7%, 58% recur across 10 / 43% recur in all 10. [EXTERNAL]
- KW-2 arXiv:2507.02976 "How Safe Are AI-Generated Patches?": 135 vs 12 = 11x; CWE-95/502 novel. [EXTERNAL]
- KW-3 OpenReview ikrQWGgxYg reward hacking 73.8%/46.8%, gap 26.4%->57.8%. [EXTERNAL]
- KW-4 Microsoft Agent Framework CodeAct (BUILD 2026): ~50% latency / >60% tokens. [EXTERNAL]
- KW-5 SpecBench arXiv:2605.21384 +28pp (exact value PLAN-asserted; quote at build). [EXTERNAL]
- ELAI ground truth (KG-1/KC-1): A-1 `elai bench claim-scope` SHIPPED; INVOKE for the measured rows.
- CLAUDE.md: feedback_paper_citation_verify, KD-1, GD-3.

## Open questions / risks
- Dossier home: docs/strategy/ vs workflow/reasoning/ — recommend docs/strategy/ (it is a positioning
  artifact; workflow/ is gitignored and per-build). Decide at authoring.
- The lint's superlative pattern set risks false positives on legitimate prose — keep it NARROW (numbers
  + a short banned-superlative list), observe-only first if it over-fires (GD-4 posture for a text gate).
- KW-6 numbers must NOT leak into the dossier before re-verification — the PENDING rows are explicit.

## Next
This makes KD-1 enforceable and gives K-7 its number source. It pairs with A-1 (the measured verdict),
A-2/A-3 (the delta), and gates every external claim ELAI ships. After it, the dossier fills in as the
bench numbers land and the brief (K-7) can be written against verified rows.

End of K-1.
