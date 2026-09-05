# K-7: AppSec / insurance positioning brief — sell the evidence pack into existing AppSec/SCA budgets and answer underwriting questionnaires

STATUS: PLANNED (WAVE KW-3; positioning-only, E-23-class; NO runtime code, NO SSB; depends K-1's numbers)

Objective: the buyer is org-side and buys from existing budget lines (KS-3 report 16). This session writes
the BRIEF that positions ELAI's evidence pack (K-6 compliance export + the trust spine) into AppSec/SCA
budgets and insurance underwriting: F-9 hallucinated-dependency gate vs the USENIX numbers (KW-1), F-2 CWE
gates vs the 11x-vulns number (KW-2), and the controls/logs/rollback/independent-evidence answers an
insurance questionnaire asks. Hard boundary: PURE PROSE + the K-1 dossier numbers; NO runtime code, NO
SSB; every number tagged measured-vs-external (KD-1); user-reviewed before any external use.

Estimated effort: ~0.5-1 session (~2-3 h). Pure authoring; no build.
Pre-requisites: K-1 dossier (the verified number source) + the K-1 claims-policy lint (the brief's
numbers pass it). F-9 hallucinated-dep gate + F-2 CWE gates (the controls the brief sells). K-6
compliance export (the artifact the brief points at).
Parallel-safe with: K-8, K-9 (disjoint). Soft-depends K-1 (numbers) + K-6 (the pack it sells).
Source: K-MASTER-BRIEF §3.2 K-7 [KS-1]; KS-3 report 16; grounding map KW-1/KW-2, KD-1. Doctrine: K-0_INDEX.md.

## Context
Why this matters: developers don't pay for safety — orgs do (KS-3 report 16); the brief is the artifact
that maps ELAI's controls onto the AppSec/SCA budget owner's and the insurer's existing questions.
Mechanism: a prose brief structured as (problem-with-a-number) -> (the ELAI control that answers it) ->
(the evidence the compliance export provides). F-9 vs KW-1 (hallucinated-package supply-chain risk),
F-2 vs KW-2 (CWE-class vulns from LLM patches), the audit trail + attestation + rollback vs the
underwriting questionnaire. External numbers are [EXTERNAL]; ELAI controls are cited to shipped code.

## Existing state (A/B..J coverage)
- K-1 dossier + claims-policy lint (the number source + the KD-1 enforcement the brief's numbers pass).
- F-9 hallucinated-dependency gate (the F-9/Spracklen grounding matches KW-1; re-grep the live gate).
- F-2 CWE gates (vs the KW-2 11x-vulns / CWE-95/502 finding).
- K-6 compliance export (the artifact bundle the brief points the buyer at).
- E-23 positioning-artifact precedent (prose, no code, no SSB).

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KW-1/KW-2, KD-1) + the K-1 dossier (the verified numbers).
2. The F-9 hallucinated-dep gate + F-2 CWE gate plan files (the controls the brief sells; confirm live status).
3. K-6's compliance-mapping doc (the procurement-language bundle the brief references).
4. memory project_gtm_playbook_2026 (the GTM stance: first buyer = vibe-cleanup AGENCIES @ $39-49/seat,
   NOT enterprise; kill the Article-50 hook; believe-the-number via reproducible run) — the brief honors it.
5. CLAUDE.md KD-1 + feedback_external_prompt_anonymity (if the brief is ever externalized, neutral vocab).

## Tasks (WAVE KW-3 — the brief)
1. Author the brief (docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md): the problem-with-a-number -> the
   ELAI control -> the evidence sections. F-9 vs KW-1, F-2 vs KW-2, the audit/attestation/rollback vs the
   underwriting questionnaire. Honor the GTM stance (agencies-first, no Article-50-certified claim).
   ~120-180 lines.
2. Every number tagged measured-vs-external (KD-1); run it through the K-1 claims-policy lint.
3. workflow/reasoning/K-7-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md (the positioning brief)
consumes_symbols:
  - the K-1 dossier (the verified number source) + the K-1 claims-policy lint (the brief passes it)
  - F-9 hallucinated-dep gate + F-2 CWE gates (the controls the brief sells; cited to shipped code)
  - K-6 compliance export (the artifact the brief points at)
reachable_from:
  - the K-1 claims-policy lint runs over the brief [PRODUCTION audit/lint gate]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO runtime code, NO SSB (positioning-only; E-23-class)
  - NO untagged number (every figure measured-vs-external; KD-1; the lint enforces)
  - NO Article-50-certified claim (GTM-killed; "designed to support" only)
  - NO enterprise-first framing that contradicts the GTM stance (agencies-first)
frontend_pair:
  N/A — a positioning doc. Justified (W-6).
observable_effect:
  trigger: "python .claude/tools/claims_policy_lint.py --files docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md"
  expect_ssb: "none (E-23-class)"
  expect_artifact: "the brief exists; the K-1 claims-policy lint PASSES it (every number tagged); a
    seeded untagged number FAILS the lint (the NEGATIVE arm, shared with K-1)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-7_appsec_insurance_positioning_brief.md
test -f docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md
python .claude/tools/claims_policy_lint.py --files docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md   # exit 0 (every number tagged)
rg -c "Article.50.certified|certified compliant" docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md   # MUST be 0
test -f workflow/reasoning/K-7-decisions.md
```

## ACCEPTANCE
- [ ] KD-1: every number tagged measured-vs-external; the K-1 lint passes the brief.
- [ ] the GTM stance honored (agencies-first; no Article-50-certified claim).
- [ ] user-reviewed before any external use.
- [ ] No runtime code, no SSB, no new dependency (E-23-class).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the brief passes the K-1 claims-policy lint (not just "the file exists").
- W-2/W-3/W-4/W-6: N/A (positioning doc); justified. W-5: the lint is the real trigger.
- W-7: F-9/F-2 control anchors re-confirmed at authoring. KD-1: the lint enforcement.

## Tests to add
- shares the K-1 claims-policy lint test (no separate runtime test; E-23-class). The brief's compliance
  is the lint pass + the user review.

## Files touched
- CREATE: docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md (~150 LOC)
- CREATE: workflow/reasoning/K-7-decisions.md (~20 LOC)

## Citations / Grounding
- KW-1 USENIX 2025 packages (5.2%/21.7%, 58%/43% recurrence) vs F-9. [EXTERNAL]
- KW-2 arXiv:2507.02976 11x-vulns / CWE-95/502 vs F-2. [EXTERNAL]
- KS-3 report 16: org-side buyer, existing budget lines, insurer AI sublimits. [EXTERNAL]
- GTM record (memory project_gtm_playbook_2026): agencies-first, Article-50 hook killed.
- CLAUDE.md: KD-1, E-23 precedent, feedback_external_prompt_anonymity (if externalized).

## Open questions / risks
- Externalization: if the brief is ever shared outside, apply feedback_external_prompt_anonymity (neutral
  vocab, no codenames/paths) — but the internal brief keeps real names.
- Number freshness: the brief's external numbers must match the K-1 dossier's re-verified values (KW set);
  if K-1's dossier-build re-verification changed a value, the brief inherits it (single source of truth).

## Next
This is the sales artifact for the org-side buyer. It pairs with K-6 (the bundle it sells) and K-1 (the
numbers). After it, the evidence pack has both the artifact (K-6) and the pitch (K-7) for an AppSec/GRC/
insurance budget owner.

End of K-7.
