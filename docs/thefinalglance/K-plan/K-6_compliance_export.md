# K-6: compliance export — one `elai export compliance` bundle (redacted RRA + attestation + audit trail + retention statement) mapped onto procurement language

STATUS: PLANNED (WAVE KW-3; BUILD-composition; KD-5; no new attestation machinery — composition + docs)

Objective: the buyer is org-side (AppSec/DevSecOps, GRC, platform, procurement) buying from EXISTING
budget lines, not the individual developer (KS-3 report 16); ELAI's artifacts must speak procurement
language WITHOUT an org-layer build. This session BUILDS `elai export compliance`: one bundle composing a
redacted RRA (G-9 precondition) + G-8 attestation + an audit-trail extract + a retention statement (I-44
mode noted) + a MAPPING DOC translating bundle fields onto procurement clauses (traceability
req->code->build->release, audit-log requirements, OMB M-25-22-style independent-evaluation/reproducibility
rights, EU AI Act Art-50 "designed to support" wording ONLY, PLD exposure, SOC2-adjacent control
narratives). Hard boundaries: NO new attestation machinery (composition only); the EU Art-50 framing is
"designed to support" wording ONLY — the Article-50-certified hook is KILLED per the GTM record and must
NOT be resurrected; NO org/SSO/RBAC build (non-goal).

Estimated effort: ~1 session (~4-5 h). In-process (composes shipped G-2/G-8/G-9/I-44 paths + authors a doc).
Pre-requisites: G-2 RRA (KG-7, rra_cmd.rs) + `elai verify-attestation` (KG-7), G-8 attestation, G-9
redaction, I-44 zero-retention, I-18 chain. Re-grep the export/verify symbols at edit (W-7).
Parallel-safe with: K-2, K-3, K-4, K-9 (disjoint). After K-1's numbers exist (the bundle's evidence
references them) — soft dependency.
Source: K-MASTER-BRIEF §3.2 K-6 [KS-1]; KS-3 report 16 (procurement demands); grounding map KG-7, KD-5. Doctrine: K-0_INDEX.md.

## Context
Why this matters: the wedge converts from developer-nice-to-have into a budget-owner purchase only if the
artifacts speak the procurement checklist (KS-2 §1.8). Mechanism: a single command assembles the
already-shipped evidence (redacted RRA, attestation, audit trail, retention statement) into one
offline-verifiable bundle + a mapping doc that cites each procurement clause to a bundle field; the
bundle verifies on a clean machine via `elai verify-attestation` + the conformance checker; a secret-grep
over the export is empty (G-9 composed). Procurement demands (KS-3 report 16, Tennessee RFIs / OMB
M-25-22 / EU AI Act / PLD / insurance): traceability req->code->build->release, audit logs, independent
evaluation rights, retention/localization, AI-flow disclosure, kill-switch. These are [EXTERNAL]
regulatory facts — cite the E-38 posture doc / primary sources, never training memory (CLAUDE.md E-38).

## Existing state (A/B..J coverage)
- KG-7 G-2 RRA CLI (rra_cmd.rs) + `elai verify-attestation` (referenced from run_subcommand/bench_cmd).
- G-8 attestation (verify-attestation-cmd.rs in cli — the offline verify path).
- G-9 redaction functions (the secret-scrub the export composes).
- I-44 zero-retention mode (the enterprise retention statement).
- I-18 chain-of-custody (the traceability spine the mapping doc cites).
- E-38 2026 posture doc (docs/strategy/ELAI-2026-POSTURE.md) — the canonical EU AI Act / PLD / timeline
  source (CLAUDE.md HARD RULE: never claim a 2026 regulatory fact from memory; cite the posture doc).

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KG-7, KD-5).
2. crates/cli/src/rra_cmd.rs + crates/cli/src/verify_attestation_cmd.rs (the export + offline-verify
   paths to compose — re-grep at edit, W-7).
3. G-9 redaction functions + I-44 zero-retention mode (the redaction + retention legs).
4. docs/strategy/ELAI-2026-POSTURE.md (E-38 — the EU AI Act Art-50 / PLD / OMB M-25-22 facts; cite, do
   not recall) + workflow/reasoning/E-38-decisions.md.
5. The GTM record (memory project_gtm_playbook_2026): Article-50-certified hook is KILLED — Art-50
   "designed to support" wording ONLY.

## Tasks (WAVE KW-3 — the export + mapping)
1. Build `elai export compliance` (a CLI arm composing rra_cmd's redacted-export + verify_attestation +
   audit-trail extract + I-44 retention statement) into one bundle (directory layout or single archive —
   decide at authoring per the open question). ~160-200 LOC, composition only.
2. Author the MAPPING DOC (docs/strategy/ELAI-COMPLIANCE-MAPPING.md): each procurement clause ->
   the bundle field that answers it. Traceability req->code->build->release (I-18), audit logs (the
   audit trail), independent-evaluation/reproducibility (G-2 RRA replay), retention (I-44), EU AI Act
   Art-50 "designed to support" ONLY, PLD exposure, SOC2-adjacent narratives. Every regulatory fact cited
   to the E-38 posture doc or a primary source (never memory). ~80-120 lines.
3. NEGATIVE arms: the bundle verifies offline on a clean machine; a grep for raw secrets over the export
   is EMPTY (G-9 composed); a bundle missing the attestation digest FAILS offline verify.
4. SSB: +ComplianceExported (Pattern-2; Kind: BundleAssembled / OfflineVerified / ClauseMapped) OR reuse
   G-8 attestation rows. Re-grep 236/545 at edit; sequential bump.
5. workflow/reasoning/K-6-decisions.md (records the bundle-format decision).

## WIRING_CONTRACT
produces_symbols:
  - the `elai export compliance` CLI arm (composing rra/attestation/redaction/retention)
  - docs/strategy/ELAI-COMPLIANCE-MAPPING.md (procurement-clause -> bundle-field mapping)
  - +ComplianceExported SSB (or reused G-8 rows — decide at edit)
consumes_symbols:
  - rra_cmd (G-2 redacted RRA export — compose, KG-7) + verify_attestation_cmd (G-8 offline verify)
  - G-9 redaction functions (the secret-scrub) + I-44 zero-retention mode (the retention statement)
  - I-18 chain-of-custody (the traceability the mapping doc cites)
  - docs/strategy/ELAI-2026-POSTURE.md (E-38 — the regulatory facts; cite, never recall)
reachable_from:
  - `elai export compliance [--out <dir|archive>]` [PRODUCTION CLI]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO new attestation/redaction machinery (composition only; GD-1)
  - NO raw secret in the export (G-9 composed; secret-grep must be empty)
  - NO Article-50-CERTIFIED claim (Art-50 "designed to support" wording ONLY — KILLED per GTM)
  - NO 2026 regulatory fact from training memory (cite E-38 posture doc; CLAUDE.md HARD RULE)
  - NO org/SSO build (non-goal)
frontend_pair:
  N/A — a CLI export + a docs mapping; no dashboard surface needed for v1 (W-6 justified) OR a
  compliance-export tile (decide at edit).
observable_effect:
  trigger: "elai export compliance --out /tmp/elai-compliance-bundle"
  expect_ssb: "ComplianceExported (BundleAssembled + OfflineVerified + ClauseMapped) — or reused G-8
    rows — recomputable from the journal (GD-2)"
  expect_artifact: "the bundle verifies offline on a clean machine (`elai verify-attestation` +
    conformance checker); a secret-grep over the export is EMPTY; the mapping doc cites each procurement
    clause to a bundle field; a bundle missing the attestation digest FAILS offline verify (NEGATIVE, R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-6_compliance_export.md --run-trigger
rg -n "export.compliance|compliance.export" crates/cli/src/   # the arm exists on a real entrypoint
test -f docs/strategy/ELAI-COMPLIANCE-MAPPING.md
rg -c "Article.50.certified|Art.50.certified|certified" docs/strategy/ELAI-COMPLIANCE-MAPPING.md  # MUST be 0 (designed-to-support only)
rg -n "ELAI-2026-POSTURE|E-38" docs/strategy/ELAI-COMPLIANCE-MAPPING.md   # regulatory facts cited to the posture doc
test -f workflow/reasoning/K-6-decisions.md
```

## ACCEPTANCE
- [ ] GD-1: composition only — no new attestation/redaction machinery.
- [ ] the bundle verifies offline on a clean machine; secret-grep empty (G-9).
- [ ] EU Art-50 = "designed to support" wording ONLY; no certified claim (GTM-killed).
- [ ] every 2026 regulatory fact cited to E-38 posture doc / a primary source (never memory).
- [ ] cargo check --workspace GREEN; auditor PASS on -p cli. SSB net 0..+1, sequential.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai export compliance` produces a bundle that verifies offline.
- W-2: N/A. W-3: N/A. W-4: N/A dashboard (W-6 justified) or a tile.
- W-5: real export + offline-verify trigger. W-7: rra/attestation/redaction symbols re-grepped at edit.
- GD-1/GD-2: as ACCEPTANCE. KD-5: time-boxed wedge artifact. E-38: regulatory facts cited, not recalled.

## Tests to add
- crates/contracts/tests/k6_compliance_export_contract.rs (~90 LOC): (a) the bundle assembles all five
  parts; (b) it verifies offline (attestation + conformance); (c) a secret-grep over the export is empty
  (G-9 composed); (d) a bundle missing the attestation digest FAILS offline verify (R2 negative); (e)
  the mapping doc has no "certified" claim and cites the posture doc.

## Files touched
- CREATE/MODIFY: crates/cli/src/compliance_export_cmd.rs (~180 LOC, composition) + the CLI arm registration
- CREATE: docs/strategy/ELAI-COMPLIANCE-MAPPING.md (~100 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (IF +ComplianceExported; sequential)
- CREATE: crates/contracts/tests/k6_compliance_export_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/K-6-decisions.md (~30 LOC, records the bundle-format decision)

## Citations / Grounding
- KS-3 report 16: procurement demands (Tennessee RFIs traceability/audit-logs/third-party-assessment/
  retention/kill-switch; OMB M-25-22 independent-evaluation + reproducible test results; EU AI Act GPAI
  enforcement 2026-08-02, Art-50 transparency; PLD covers software placed on market from 2026-12-09;
  insurer AI sublimits). [EXTERNAL — cite E-38 posture doc / primary sources]
- KS-2 §1.8: the buyer-side layer is composition + docs, not new architecture.
- ELAI ground truth: KG-7 (RRA + verify-attestation), G-8/G-9/I-44/I-18.
- CLAUDE.md: E-38 (never claim a 2026 regulatory fact from memory), GD-1, GTM record (Art-50 hook killed).

## Open questions / risks
- Bundle format: single archive vs directory layout — propose at authoring after reading the I-44/G-8
  output shapes (brief §7). Recommend a directory layout (inspectable) with an optional archive flag.
- Regulatory drift: the E-38 posture doc has a 90-day staleness check; if stale at authoring, refresh
  before citing (CLAUDE.md E-38). EU AI Act / PLD dates must be re-confirmed against the posture doc.
- Scope creep into an org-layer: K-6 is a CLI export + a doc, NOT an SSO/RBAC build (non-goal) — hold
  the line; the org layer is re-evaluated only after K-6 meets real buyers.

## Next
This converts the wedge into something an AppSec/GRC/procurement owner can buy with an existing budget
line. It pairs with K-7 (the brief that sells it) and K-1 (the numbers it references). After it, the org
layer is re-evaluated only on demonstrated pull (non-goal until then).

End of K-6.
