GROUNDING SWEEP -- phase F-2b
Source: docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md
Date: 2026-06-02
FSM: PLANNING (phase F-2b insecure-rate keystone) | Session: s-60824

SCOPE NOTE
This sweep is F-2b-targeted: the external-behavior claims about the SVEN / CodeGuard+ /
SeCodePLT prior art and the methodology of ELAI's own measured Qwen secure-rate number.
F-2a's in-tree code anchors (rule_engine, predicate, ssb, hook_dispatcher) are code-read
GROUNDED per the plan's Citations section and elai.map and are not re-litigated here.

SPEC REFERENCES (external-behavior / paper citations only)
arXiv:2302.05319 SVEN (CCS 2023) | GROUNDED | VIBESEC-PAPERS D1:286-295 STATUS=VERIFIED, all three % verbatim in abstract; backbone CodeGen-2.7B
arXiv:2405.00218 CodeGuard+ | GROUNDED | VIBESEC-PAPERS D2:298-307 STATUS=VERIFIED, Table 2 43.17->76.00 (+32.83) on CodeGen-2.7B
arXiv:2410.11096 SeCodePLT (NeurIPS 2025) | GROUNDED | VIBESEC-PAPERS A5:79-87 STATUS=VERIFIED, 5.9k samples / 44 CWEs, GPT-4o 55% secure (PRIMARY-PAGE seccodeplt.github.io, not arXiv abstract)
synthesis C2 + section 1.4 + section 7 | GROUNDED | VIBESEC-SHARD-...-2026-06-01.md:96-101 (1.4 corrections), 213-227 (C2 grounding+caveat), 494-496 (do-not-transfer rule)

CLAIM CLASSIFICATION
SVEN 59.1->92.3% prefix steering | GROUNDED | VIBESEC-PAPERS:293 abstract-verbatim; STATUS VERIFIED line 295
SVEN number is CodeGen-2.7B not Qwen | GROUNDED | VIBESEC-PAPERS:292 "Backbone: 2.7B CodeGen"; synthesis:218-219
CodeGuard+ 43.2->76.0% constrained decode | GROUNDED | VIBESEC-PAPERS:305 Table-2-verbatim 43.17->76.00; STATUS VERIFIED line 307
CodeGuard+ number is CodeGen-2.7B not Qwen | GROUNDED | VIBESEC-PAPERS:305 "on CodeGen-2.7B"; synthesis:218-219
SVEN/CG+ do NOT transfer to Qwen2.5-Coder-7B-abliterated | GROUNDED | synthesis:218-219, 494-496 explicit no-transfer rule; plan:25,255-256
SeCodePLT 5.9k samples / 44 CWEs | GROUNDED | VIBESEC-PAPERS:85 v2 abstract "more than 5.9k samples spanning 44 CWE-based risk categories"
SeCodePLT GPT-4o 55% secure rate | GROUNDED | VIBESEC-PAPERS:86 (seccodeplt.github.io primary-page, NOT arXiv abstract -- cite carefully)
SeCodePLT methodology = CWE-tagged prompt + deterministic secure/insecure oracle | GROUNDED | VIBESEC-PAPERS:85 "insecure-coding (instruction-gen + completion)"; plan:78 design intent matches
SeCodePLT tested Qwen2.5-Coder (base) | GROUNDED | VIBESEC-PAPERS:85 model list incl Qwen2.5-Coder; BUT no abliterated-variant secure rate published anywhere
F-2b DELIVERABLE = measured Qwen before/after insecure-rate | GROUNDED(as-target-shape) | plan:80,86,199; this is the number to PRODUCE, not a cited prior number
"below 15%" target / GitGuardian replay corpus | GROUNDED(as-prohibition) | synthesis:96-101,203 KEPT OUT; plan:25 confirms exclusion -- must NOT reappear
F-2b corpus is ELAI's OWN synthetic suite | GROUNDED | plan:78; synthesis:100-101 "must define its own synthetic benchmark, not borrow these as replay sets"

NON-TARGET FLAGS (mandatory -- every transferred number)
- SVEN 59.1->92.3% : NON-TARGET. VERIFIED on CodeGen-2.7B. MUST NOT be shipped as the F-2b number, a target, a ceiling, or an expected after-rate. It is directional prior-art evidence that secure generation is steerable without retrain -- nothing more for Qwen.
- CodeGuard+ 43.2->76.0% : NON-TARGET. VERIFIED on CodeGen-2.7B (Table 2). MUST NOT be shipped as the F-2b number/target/ceiling. Same status: directional only.
- SeCodePLT GPT-4o 55% / ~40-45% insecure : NON-TARGET as a Qwen expectation. It is the corpus-DESIGN reference and a different-model base rate; it is not Qwen2.5-Coder-7B-abliterated's rate and must not be quoted as F-2b's before-number.
- "below 15%" + GitGuardian 3.2% / Veracode 45% : PROHIBITED framings (section 1.4). Not targets, not corpora, not headline anchors.

ACTIONS REQUIRED
- Enforce in F-2b writeup (plan Task 11, decision-doc addendum): the ONLY shippable F-2b headline number is the MEASURED Qwen2.5-Coder-*-abliterated before/after from the live RunPod/remote-3090 run, with model_id + n_samples + ELAI_SEED + synthetic-corpus caveat. Until that run exists, NO percentage may be published as F-2b's number.
- Gate the F-2b LIVE-QWEN-RUN acceptance checkbox (plan:199) as the sole source of the keystone percentage; the deterministic-fixture harness (plan:198) ships NOW but emits the measured row only when run -- it may not hard-code or default any before/after value (stub_ban plan:138 "insecure_rate_before == insecure_rate_after hard-coded/mock absent").
- Carry the CodeGen-2.7B non-transfer caveat verbatim into SecurityRulesPanel.svelte copy + F-2-decisions.md so the dashboard before/after tile never implies the SVEN/CG+ numbers.
- When citing SeCodePLT GPT-4o 55%, tag it [PRIMARY-PAGE seccodeplt.github.io] not [arXiv abstract] -- the 55% is project-site, not in the arXiv abstract (VIBESEC-PAPERS:86).
- No INGEST needed: all three papers already VERIFIED in VIBESEC-PAPERS-2026-06-01.md (paper-ingestor equivalent already run). No UNGROUNDED claims remain.

VALIDATION WARNINGS
- The two transferred numbers are VERIFIED-true-but-non-transferable; the risk is misuse-as-target, not factual error. Treated as NON-target prohibitions above, not as VERIFY/INGEST.
- SeCodePLT 55% secure rate lives on the project site, not the arXiv abstract -- citation-surface mismatch noted; not a contradiction.
- No spec-file (docs/specs/guide/steps/phase-*.txt) references in F-2 -- F-plan is doctrine-driven (F-0_INDEX W-1..W-10), not phase-spec-driven; expected for the F series, not a traceability defect.
- All paper sources validated against VIBESEC-PAPERS-2026-06-01.md and the synthesis; no missing files.
