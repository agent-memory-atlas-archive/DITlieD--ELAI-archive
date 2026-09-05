# K-13: OWASP control-mapping dossier + injection-defense posture proof — answer "what is your OWASP LLM/Agentic Top-10 coverage?" with a tagged, honest, number-where-measured mapping

STATUS: PLANNED (WAVE KW-3; E-23-class PACKAGE + PROVE; NO runtime code, NO SSB; reuses the K-1 claims-policy lint; the injection-number row fills from G-6/A-2/A-3 gated by KD-1)

Objective: the org-side buyer's security questionnaire asks "OWASP LLM Top-10 / Agentic Top-10 coverage?"
(KS-3 report 16; the OWASP analysis was run 2026-06-25 precisely because no OWASP string existed anywhere
in the repo — grep-confirmed). This session ships TWO artifacts, both wedge/business-discipline, near-zero
new capability: (1) [PACKAGE / WI-5] a docs/strategy/ CONTROL-MAPPING that translates ELAI's already-built
trust spine onto OWASP LLM01-10 (2025) + Agentic ASI01-10 (2025), one cell per threat, each tagged
measured-vs-design with a code-or-plan citation (KD-1); it feeds K-6 compliance export (a new bundle clause)
and K-7 the AppSec/insurance brief (the buyer's question). (2) [PROVE / WI-4] bind the ONE OWASP axis that
needs a measured number not a design assertion — prompt-injection (LLM01/LLM05/ASI01) — to ELAI's
gate-ON-vs-OFF injection-defense delta from G-6 SAFEDELTA's AgentDojo arm, as a K-1 dossier row tagged
[BENCH-PENDING] until the run lands. Hard boundary it must NOT cross: it does NOT author the injection
HARDENING code (the transformation-surviving taint binding / cross-process taint / E-37 perimeter completion
= WI-1/WI-2/WI-3, a technical E-37/G-1 EXTENSION, J/E-bound, recorded below NOT authored here); it does NOT
author the config-ingress admission gate (WI-6 = a technical E-43 EXTENSION, J/E-bound); it does NOT run
G-6 SAFEDELTA / AgentDojo itself (the RunPod-gated A-2/A-3 + the AgentDojo harness; the number FILLS IN,
gated KD-1); it does NOT fabricate an injection-defense number before the bench produces it (KD-1); it does
NOT claim "OWASP certified" or "fully covered" — a PLANNED/DARK control is READINESS not a coverage gap, and
the cell is tagged with its honest implementation status.

Estimated effort: ~0.5-1 session (~2-3 h). Pure authoring (the mapping doc) + a one-row dossier binding +
a NARROW pattern-extension of the K-1 claims-policy lint to recognise OWASP coverage cells + one contract
test. No build, no SSB, no new dependency.
Pre-requisites: K-1 dossier + the K-1 claims-policy lint (the mapping's claims pass it; K-13 EXTENDS the
lint's recogniser, never forks it — GD-1). The OWASP LLM-2025 + Agentic-2025 lists (WebFetch-verified
2026-06-25 at genai.owasp.org; re-verify at authoring per KD-1, and NEVER cite the stale v1.1 owasp.org
landing list). G-6 SAFEDELTA substrate present (safedelta_runner.rs + the AgentDojo arm — the injection
number's source, BENCH-PENDING). The OWASP gap analysis (the mapping's source-of-truth, already grounded).
Parallel-safe with: K-8, K-9 (disjoint). Soft-depends K-1 (numbers + lint), K-6 (the bundle it adds a
clause to), K-7 (the brief it feeds). The injection-number row soft-depends A-2/A-3/the AgentDojo run.
Source: workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Sections B/C (the design-verdict
mapping), Section D (the live-code taint trace), Section E WI-4/WI-5 (the deliverables), Section H (config-
ingress context). KS-3 report 16 (org-side buyer). Doctrine: K-0_INDEX.md (KD-1, KD-2, E-23 precedent).

## Context
Why this matters: ELAI's verification spine already answers most of the OWASP rubric at design-SOTA grade
(the agentic rubric fits ELAI BETTER than the LLM list — it rewards supervisor/least-privilege, evidence-
bound approval, frozen gates, execution-grounded self-improvement; OWASP analysis Section C). What was
missing is the RENDER of that coverage in the language a procurement/AppSec/insurance buyer reads, and the
DISCIPLINE that a coverage claim carries its real status (measured vs design vs partial vs N/A) instead of
a marketing superlative. The OWASP analysis already did the mapping work (Sections B/C, live-code grounded);
K-13 promotes it into a maintained procurement artifact under the KD-1 claims-policy lint, and binds the one
genuinely-measurable axis (injection defense) to a real gate-ON-vs-OFF number. Mechanism: a one-row-per-
threat table where each cell is (ELAI control(s) + code/plan citation + status tag + measured-or-[DESIGN]),
the K-1 lint refusing any "covered" cell that lacks a measured number or a [DESIGN] tag; plus a single
dossier row pairing the OWASP injection threat with ELAI's SAFEDELTA AgentDojo delta, [BENCH-PENDING] until
the run lands. The PROVE half (the injection number) is the load-bearing claim; the PACKAGE half (the
mapping) is the distribution wrapper around the spine that already exists.

Standout already-covered axes the mapping records (NOT work — OWASP analysis Section C "STANDOUT STRENGTHS"):
ASI04 runtime MCP poisoning = G-4 live re-pin + E-24 pin + C-42 authz (the EXACT ASI04 control); ASI09
human-agent trust exploitation = receipts-not-prose (F-4 cant-lie + H-53 + G-5 + K-3/K-11); ASI10 rogue
agents = frozen gates R8 + execution-grounded self-improvement (H-43/H-44) + L1 HardVeto; LLM03 supply
chain = F-9 + anti-slopsquat + E-43 + I-18 + G-8; LLM09 misinformation = the grounding/cant-lie spine.

The N/A boundary the mapping states honestly (OWASP analysis Section B/F): model inversion / extraction /
theft (LLM02-tail, old LLM10) = N/A for single-user local (no proprietary training, own sidecar); they
RE-OPEN only under multi-seat, where J-59 per-seat isolation already owns them — so the mapping notes
"sequence J-59 BEFORE multi-seat" rather than authoring a service-provider defense.

## Existing state (A/B..J coverage — what the mapping cites, do NOT re-author, GD-1)
- G-1 TAINT [WIRED] (Provenance + capability_gate, 4 ingest legs wired, contract-tested) — LLM01/ASI01
  data-plane; the residual BINDING gap (substring not transformation-surviving) is WI-1, technical, J/E-bound.
- E-37 PROMPT INJECTION DEFENSE [PARTIAL] (Spotlighting + PromptArmor; output_sanitizer/classifier/egress
  allowlist/AgentDojo-CI absent) — the perimeter the WI-3 technical extension completes (J/E-bound).
- G-3 egress IP-pin, F-1 VibeGate secret/code gate, F-2 CWE gates, F-3 CaMeL tool-filter, F-9 hallucinated-
  dep gate, G-9 redaction, I-1 supervisor, I-50 auth broker, E-30 argv allowlist, Tier-C process isolation,
  F-4 cant-lie, H-53 citation pre-bind, G-5 reviewer cockpit, D-13 cross-family adversary, E-24/G-4/C-42 MCP,
  E-43 skill admission, B-22 autonomous gate — all WIRED/PARTIAL, each a cited OWASP cell (analysis B/C).
- G-6 SAFEDELTA [BENCH-PENDING] (SusVibes/BaxBench/SeCodePLT/AgentDojo, gated by F-2b, 0 measured rows) —
  the AgentDojo arm is the injection-number source the PROVE leg binds (gate-ON-vs-OFF, never a leaderboard).
- K-1 dossier + claims-policy lint (the number-binding pattern + the KD-1 enforcement K-13 reuses).
- K-6 compliance export (the bundle the OWASP mapping becomes a clause of) + K-7 brief (the pitch it feeds).
- K-11 human-viewable evidence render + K-3 PR emitter (the ASI09 "receipts-not-prose" distribution legs).
- E-23 positioning-artifact precedent (prose, near-zero code, no SSB).

## Reads (mandatory before coding)
1. workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md IN FULL (Sections B/C the mapping,
   D the taint trace, E WI-4/WI-5 the deliverables, F the do-not-re-propose ledger, G the residue, H config).
2. _grounding/K-GROUNDING-MAP.md (KD-1, KD-2) + the K-1 dossier + the K-1 claims-policy lint (extend it).
3. K-MASTER-BRIEF §3.2 (K-1/K-6/K-7 specs) + KS-3 report 16 (org-side buyer language).
4. The G-6 SAFEDELTA plan file (the AgentDojo arm shape + the gate-ON-vs-OFF delta the PROVE leg cites) —
   confirm BENCH-PENDING status + the F-2b RunPod gate live (W-7).
5. CLAUDE.md KD-1 + feedback_paper_citation_verify (re-verify the OWASP LLM/Agentic lists at authoring;
   genai.owasp.org canonical, owasp.org landing serves stale v1.1) + feedback_external_prompt_anonymity
   (if the mapping is ever externalised, neutral vocab).

## Tasks (WAVE KW-3 — the mapping + the injection-number binding + the lint extension)
1. [PACKAGE / WI-5] Author docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md: one row per threat for LLM01-10
   (2025) + ASI01-10 (2025); each cell = ELAI control(s) + code/plan citation + STATUS tag
   (WIRED-MEASURED | WIRED-DESIGN | PARTIAL | PLANNED(id) | N-A) + a measured number OR an explicit
   [DESIGN]/[EXTERNAL] tag (KD-1). The N/A boundary (model-inversion/extraction = single-user-local N/A,
   re-opens under multi-seat -> J-59) stated honestly. ~140-200 lines. Promotes the analysis Sections B/C;
   re-verify the OWASP lists at authoring (KD-1).
2. [PROVE / WI-4] Add ONE dossier row to the K-1 ELAI-PROBLEM-QUANTIFICATION-DOSSIER (or a clearly-marked
   K-13 sibling section): OWASP prompt-injection (LLM01/LLM05/ASI01) -> ELAI's gate-ON-vs-OFF injection-
   defense delta from G-6 SAFEDELTA's AgentDojo arm, recompute-pointer to the journal, tagged
   [BENCH-PENDING] until A-2/A-3/the AgentDojo run lands (KD-1, NO fabricated number; mirrors K-1's
   "PENDING — fills from A-N" rows and K-2's J-28-gated live leg).
3. EXTEND the K-1 claims-policy lint's recogniser (NOT a fork, GD-1) so an OWASP coverage cell that asserts
   "covered"/"protected" without a measured number AND without a [DESIGN] tag FAILS; a correctly-tagged cell
   passes. ~15-30 LOC added to claims_policy_lint.py + the conditional-routing trigger already loads it.
4. Record the COMPOSES/feeds relationship to K-6 (the OWASP mapping is a procurement clause K-6's export
   references when built) WITHOUT modifying K-6's plan file (new-plan-files-only) — a pointer line here +
   the feature-map cross-link.
5. NEGATIVE arm: a fixtures file with (a) an OWASP cell claiming "fully covered" with no number/tag and
   (b) an "OWASP certified" superlative — the lint must FAIL both.
6. workflow/reasoning/K-13-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md (the LLM01-10 + ASI01-10 control mapping, tagged)
  - the OWASP-injection dossier row in ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (K-1's file; [BENCH-PENDING])
  - the OWASP-cell recogniser extension in .claude/tools/claims_policy_lint.py (K-1's lint; extended)
consumes_symbols:
  - the K-1 dossier + claims-policy lint (EXTEND, do NOT fork; KD-1 enforcement; GD-1)
  - benchmarks::safedelta_runner AgentDojo arm (the injection-number source — INVOKE-when-it-lands, never rebuild)
  - the OWASP analysis Sections B/C/D (the mapping content; the analysis is the source, not re-derived)
  - K-6 compliance export + K-7 brief (the artifacts the mapping feeds; not modified here)
reachable_from:
  - the auditor's positioning-copy review runs claims_policy_lint.py over ELAI-OWASP-CONTROL-MAPPING.md [PRODUCTION audit gate]
  - conditional-routing loads the claims policy when docs/strategy/ positioning copy is edited (already wired by K-1)
  - K-6 compliance export references the OWASP mapping as a procurement clause when K-6 is built [feeds, not a caller]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO fabricated injection-defense number before G-6's AgentDojo arm produced it (KD-1; row stays [BENCH-PENDING])
  - NO OWASP coverage cell claiming "covered" without a measured number OR a [DESIGN]/[EXTERNAL] tag (KD-1)
  - NO "OWASP certified" / "fully covered" superlative (the brief/mapping sells honest status, never certification)
  - NO fork of the K-1 lint (extend the recogniser; GD-1) and NO main-LLM self-run of the lint (auditor-applied)
  - NO authoring of the technical injection-hardening (WI-1/WI-2/WI-3) or config-admission (WI-6) here (out-of-K)
frontend_pair:
  N/A — a positioning/procurement doc + a CI/audit lint extension, no dashboard surface. Justified (W-6).
observable_effect:
  trigger: "python .claude/tools/claims_policy_lint.py --files docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md"
  expect_ssb: "none (E-23-class; the lint is a deterministic check, not a journaled event)"
  expect_artifact: "the OWASP mapping exists and the K-1 lint PASSES it (every cell tagged); the injection
    dossier row is present and tagged [BENCH-PENDING]; a seeded 'fully covered, no number' cell and an
    'OWASP certified' superlative each FAIL the lint (the NEGATIVE arm that MUST fail)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-13_owasp_control_mapping_injection_posture.md
test -f docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md
python .claude/tools/claims_policy_lint.py --files docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md   # exit 0 (every cell tagged)
python .claude/tools/claims_policy_lint.py --files docs/thefinalglance/K-plan/_fixtures/owasp_negative.md ; test $? -ne 0   # 'fully covered' no-number + 'OWASP certified' MUST fail
rg -n "LLM0[1-9]|LLM10|ASI0[1-9]|ASI10" docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md   # all 20 threats present
rg -n "\[BENCH-PENDING\]" docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md   # the injection number is gated, not fabricated
rg -ci "OWASP certified|fully covered" docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md   # MUST be 0
test -f workflow/reasoning/K-13-decisions.md
```

## ACCEPTANCE
- [ ] all 20 OWASP threats (LLM01-10 2025 + ASI01-10 2025) have a row with a control citation + status tag.
- [ ] KD-1: every "covered" cell carries a measured number OR a [DESIGN]/[EXTERNAL] tag; the K-1 lint passes it.
- [ ] the injection-defense dossier row is present and [BENCH-PENDING] (no fabricated number) until G-6 AgentDojo lands.
- [ ] the N/A boundary (model-inversion/extraction = single-user-local N/A, re-opens under multi-seat -> J-59) is stated.
- [ ] the OWASP lists re-verified at authoring (genai.owasp.org canonical; the stale v1.1 list never cited).
- [ ] the K-1 lint is EXTENDED not forked (GD-1); no runtime code, no SSB, no new dependency (E-23-class).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the K-1 lint runs over the OWASP mapping + the negative fixture FAILS, not "the doc exists".
- W-2: N/A (no feature flag). W-3: N/A (no port — the analysis is promoted, not summary-ported). W-4/W-6: N/A dashboard; justified.
- W-5: trigger = the real lint command over the real mapping doc. W-7: G-6 AgentDojo + E-37 + K-1-lint anchors re-grepped at edit.
- GD-1: extend the K-1 lint + cite the analysis, never re-implement either. GD-2: the injection row carries a recompute pointer.
- GD-3: the mapping/dossier use SAFEDELTA gate-ON-vs-OFF framing, never a SWE-bench/leaderboard rank.
- KD-1: the lint IS the enforcement (no OWASP claim without a number or a tag). KD-2: the mapping is a buyer-facing render of the spine.

## Tests to add
- crates/contracts/tests/k13_owasp_claims_contract.rs (~50 LOC) OR a Python test for the lint extension:
  (a) a correctly-tagged OWASP cell passes; (b) a "fully covered" cell with no number/tag fails;
  (c) an "OWASP certified" superlative fails; (d) a [DESIGN]-tagged design-complete cell passes;
  (e) an empty/missing mapping input fails-loud (R1); (f) the [BENCH-PENDING] injection row does NOT count
  as a fabricated measured number (the lint accepts the explicit pending tag).

## Files touched
- CREATE: docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md (~180 LOC)
- MODIFY: docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 injection dossier row; K-1's file — a row add, not a rewrite)
- MODIFY: .claude/tools/claims_policy_lint.py (+~20 LOC OWASP-cell recogniser; K-1's lint, extended)
- CREATE: docs/thefinalglance/K-plan/_fixtures/owasp_negative.md (~12 LOC, the negative arm)
- CREATE: workflow/reasoning/K-13-decisions.md (~25 LOC)

## Citations / Grounding
- OWASP Top 10 for LLM Applications 2025 (genai.owasp.org/resource/owasp-top-10-for-llm-applications-2025),
  WebFetch-verified 2026-06-25: LLM01 Prompt Injection .. LLM10 Unbounded Consumption (LLM07 System-Prompt
  Leakage + LLM08 Vector/Embedding Weaknesses are the 2025-NEW entries). [EXTERNAL]
- OWASP Top 10 for Agentic Applications (genai.owasp.org, Dec-2025): ASI01 Agent Goal Hijack .. ASI10 Rogue
  Agents. [EXTERNAL] (re-verify both lists at authoring; never cite the stale owasp.org v1.1 landing list.)
- workflow/reasoning/owasp-llm-agentic-gap-analysis-2026-06-25.md Sections B/C (mapping), D (taint trace,
  codegraph file:line grounded), E WI-4/WI-5 (deliverables), F (do-not-re-propose), H (config-ingress).
- KS-3 report 16: org-side buyer, existing budget lines, security/insurance questionnaire. [EXTERNAL]
- ELAI ground truth: G-1/G-3/F-1/F-2/F-3/F-9/G-9/I-1/I-50/E-30/Tier-C/F-4/H-53/G-5/E-24/G-4/C-42/E-43/B-22
  WIRED-or-PARTIAL (feature-map Sections 1.6/1.8, code-grounded); G-6 SAFEDELTA AgentDojo arm BENCH-PENDING.
- CLAUDE.md: KD-1, GD-1, GD-3, feedback_paper_citation_verify, E-23 precedent.

## Open questions / risks
- Dossier-row home: a row in K-1's existing dossier vs a clearly-marked K-13 OWASP section within it —
  recommend a row IN the K-1 dossier (single source of truth for ELAI's measured numbers; K-13 owns the
  OWASP MAPPING doc, K-1 owns the number ledger). Decide at authoring; do not duplicate the number.
- The injection number is the load-bearing claim and it is BENCH-PENDING (G-6 gated on F-2b RunPod). The
  mapping ships honestly without it (the cell tagged WIRED-DESIGN + [BENCH-PENDING] number); do NOT let the
  mapping imply a measured injection result before the run lands (KD-1; the kill-switch-1 caveat in K-0 —
  a NULL/NEGATIVE safety delta re-aims the wedge — applies to this cell too).
- TECHNICAL COMPANION WORK, OUT-OF-K (recorded so nothing is lost; to be authored in a J/E pass, NOT here):
  WI-1 transformation-surviving taint binding (trifecta+window hybrid over the wired G-7 session gate) +
  WI-2 cross-process taint propagation + WI-3 E-37 perimeter completion = ONE technical EXTENSION on
  E-37/G-1 (the OWASP analysis Section E shape). WI-6 config-ingress admission gate (blake3 load-time
  admission manifest over .md instruction surface + MCP add-scan + skill hard-enforce) = a technical
  EXTENSION on E-43/E-24. Both are capability work (= J-family / E-extension), not wedge/business-discipline,
  so they are NOT K. K-13 PROVES + PACKAGES the EXISTING spine; the technical halves harden it.

## Next
This gives the org-side buyer the OWASP coverage answer in their language, under the KD-1 discipline, and
binds the one measurable axis (injection defense) to a real gate-ON-vs-OFF number when it lands. It pairs
with K-1 (the number ledger + lint it extends), K-6 (the compliance bundle it adds a clause to), and K-7
(the brief that cites it). The companion technical hardening (WI-1/WI-2/WI-3 on E-37/G-1, WI-6 on E-43) is
recorded for a separate J/E authoring pass — it is capability, not wedge.

End of K-13.
