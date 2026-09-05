GROUNDING SWEEP -- phase F-10
Source: docs/thefinalglance/F-plan/F-10_security_monotonicity_iter_guard.md
Date: 2026-06-02
FSM: IDLE | Session: s-60824

SCOPE NOTE
This sweep targets the three external-behavior claims named in the task, not the
full F-10 spec-reference set. The plan cites code anchors (chat_reflect.rs:294/314,
live_cli.rs:1518, task_runner.rs:13105-13168, etc.) as ELAI ground-truth; those are
internal and out of scope for this sweep (they are W-7 wiring anchors, validated by
wiring_contract_check.py at execution, not paper claims).

SPEC REFERENCES (paper-citation provenance for the 3 claims)
arXiv:2506.11022 (Shukla, +37.6%) | GROUNDED | VIBESEC-RESEARCH-...LANDSCAPE.md:114-120 records title/authors/all-4-figures CONFIRMED; live WebFetch arxiv.org/abs/2506.11022 confirms title "Security Degradation in Iterative AI Code Generation", authors Shukla/Joshi/Syed, abstract verbatim "37.6% increase in critical vulnerabilities after just five iterations", 400 samples, 40 rounds, 4 prompting strategies.
arXiv:2507.02976 v3 (Sajadi, ~11x) | GROUNDED | VIBESEC-PAPERS-...md:269-278 records 135 vs 12 / ~11x / CWE-78=79,95=39,502=17,22=14 as VERIFIED against full paper; live WebFetch confirms title, authors Sajadi/Damevski/Chatterjee, current version v3 (last revised 29 Dec 2025), abstract corroborates "Llama introduces many new vulnerabilities". v1 numbers (185/9x/20) flagged STALE.
FP<5% / 50-clean-patches-per-language target | VERIFY_NEEDED (design target, not a paper claim) | synthesis section 4 C10 (lines 270-272) states "measured FP<5% on 50 clean patches per language"; this is a CALIBRATION TARGET to be MEASURED in F-10 phase 1, NOT a cited external result. No paper backs the 5% number for Rust; it is a chosen acceptance threshold.

CLAIM CLASSIFICATION
security degrades +37.6% over 5 AI-improvement iterations | GROUNDED | arXiv:2506.11022 abstract VERIFIED (WebFetch + LANDSCAPE.md:114). Plan compresses "5 iterations / 40 rounds / 400 samples"; the F-10 line "5 iterations / 400 samples" is faithful but DROPS the "40 rounds, 4 prompting strategies" qualifier present in the source.
standalone LLM injects ~11x more vulns than humans | GROUNDED | arXiv:2507.02976 v3, 135 vs 12, VIBESEC-PAPERS:276 VERIFIED. "more-autonomous frameworks inject MORE" = directional, also in v3 abstract. MUST cite v3; v1 (185/9x/20) is stale.
CWE-class set CWE-78/95/502/22 dominant | GROUNDED | arXiv:2507.02976 v3 top-CWE table VIBESEC-PAPERS:276 (78=79,95=39,502=17,22=14).
CWE-classifier is deterministic (GREP/AST, in-process) | GROUNDED | design decision, internally consistent with safety-crate pattern style (synthesis C10 line 257-259, F-10 Task 1); deterministic-by-construction, not an external behavior claim. Feasibility for Rust is the open risk, not determinism.
classifier is L-effort, "ships now" | VERIFY | synthesis C10 (lines 268-271) calls the classifier itself L-effort BECAUSE it does not exist and Rust semgrep coverage is sparse. F-10 honors this (two-phase split). "ships now" is the plan's framing of phase-1 buildability, NOT a measured fact -- it is a scoped estimate (2 sessions / 14-18h, F-10 line 4).
calibrated to FP<5% on 50 clean patches per language | UNGROUNDED-AS-FACT / VALID-AS-TARGET | NO external source measures a Rust CWE-grep classifier at <5% FP. Synthesis line 271 + F-10 Task 3 both label it a TARGET to be MEASURED on ELAI's own corpus. F-10 line 204 + Open-questions explicitly mark Rust portability UNVERIFIED until phase-1 measures it. Correctly NOT presented as proven.
Rust semgrep CWE coverage is sparse | VERIFY | asserted in synthesis C10 (line 269-270) and F-10 (lines 15,46,204,209) without a primary cite. Plausible and load-bearing (drives the L-effort split + Rust-WARN-only fallback). Recommend a 1-source web confirmation before phase-1 if it gates the Python/TS-first fallback decision.

NUMBERS THAT MUST NOT BECOME TARGETS (Goodhart guard)
+37.6% (critical-vuln increase / 5 iters): MOTIVATION ONLY. F-10 line 49(d) + 200 + GROUNDING-STATUS(204) already forbid importing it as an ELAI target. Do NOT wire any threshold, reward, or pass/fail bound to 37.6 or a derived "<37.6%" budget. The gate is BINARY (new CWE class present vs absent), NOT a percentage budget -- keep it that way.
~11x / 135 / 12 (Sajadi v3): MOTIVATION ONLY. Same discipline (F-10 line 49(d)). Do NOT set an "allow up to N injected vulns" or "stay under 11x" target. The invariant is class-presence monotonicity, not a vuln-count ratio.
FP<5% (50 clean patches/lang): this IS a legitimate acceptance target (it is ELAI's OWN measured calibration, not a transferred external number) -- but it must stay an acceptance GATE measured per-language, never a knob the classifier optimizes toward by loosening patterns (anti-spiral: F-10 Task 3 already says tighten+re-measure, escalate after 3 passes; do NOT game FP down by under-detecting real CWEs -- track recall alongside, or the 5% becomes a Goodhart target that hides misses).

ACTIONS REQUIRED
- v3-LOCK (highest): every place F-10 / executor cites Sajadi MUST say "v3" and use 135/~11x/12 + CWE 78=79,95=39,502=17,22=14. Re-introducing v1 (185/9x/20) is a citation-hygiene regression (VIBESEC-PAPERS:543-544). F-10 already pins v3 in 6 places -- preserve on every edit.
- KEEP +37.6% and ~11x as MOTIVATION-ONLY strings in code comments / decision doc; assert in F-10-decisions.md part A(d) that NO numeric threshold derives from either. Add a contract-test-level assertion or reviewer note that the gate logic contains no "37.6" / "11" magic constant.
- verify "Rust semgrep CWE coverage is sparse" with 1 web source (e.g. semgrep registry Rust ruleset count) before phase-1 IF the Python/TS-first fallback gets triggered; it currently rides as an internally-asserted premise.
- TRACK RECALL alongside FP in F-10-classifier-calibration.md: a <5% FP achieved by under-firing is a hidden failure. The 50-clean-patch corpus measures FP; add at least a few KNOWN-VULN fixtures (the cwe78_regression.json + siblings) so recall is non-zero-checked, else FP<5% is gameable.
- Sajadi-abstract caveat: live WebFetch of 2507.02976 returned only the directional abstract claim ("Llama introduces many new vulnerabilities"); the 135/12 counts live in the PDF body. The source-of-record (VIBESEC-PAPERS:276-278) marks them VERIFIED against the full paper -- trust that, but if an executor needs the raw number it should read the PDF, not the abstract page.

VALIDATION WARNINGS
- Shukla 2506.11022 is NOT in VIBESEC-PAPERS-2026-06-01.md (the dedicated papers file); its provenance is VIBESEC-RESEARCH-...LANDSCAPE.md:114 only. Both F-10 (line 38) and the synthesis cite it. Provenance is real and now WebFetch-confirmed, but the paper is single-sourced inside the repo -- LANDSCAPE, not the curated PAPERS card.
- "5 iterations" vs source "5 iterations over 40 rounds, 4 prompting strategies": F-10's compression is accurate but lossy; the 40-rounds/4-strategies context matters if anyone re-derives the rate.
- FP<5% Rust portability is UNVERIFIED by design (correctly labelled in F-10:204 + Open-questions). Not a defect -- a known phase-1 measurement gate.
- All three task-named claims resolved; no MISMATCH between F-10 and its source-of-record. F-10's v3-discipline and motivation-not-target discipline are intact in the plan as written.
