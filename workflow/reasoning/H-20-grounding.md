GROUNDING SWEEP -- phase H-20
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-20_leakage_taint_composite.md
Date: 2026-06-05
FSM: PLANNING | Session: s-8404

SPEC REFERENCES
_grounding/H-TRUST-GROUNDING-MAP.md: TS-2, TS-3, NN-4, TD-3, TD-5 | GROUNDED | H-TRUST-GROUNDING-MAP.md:39-60+110-111+131-151+191-199 -- all five tags present, on-topic for leakage COMPOSITE + taint primitive + G-1 prereq
design T-4 + PART 0 + PART 4 (R1-R5 -> T-4 mapping) | GROUNDED | D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md:128-141 (T-4 leakage), :10-14 (PART 0 north star), :292-354 (PART 4 R1-R5 -> T-4)
H-21_runtime_lookahead_differential.md | GROUNDED | file exists at docs/thefinalglance/H-plan/H-21_runtime_lookahead_differential.md
provenance.rs:131-200 (taint primitive) | GROUNDED | from_web at line 145; Provenance struct at 131; Provenanced/from_file/from_file_screened all present
capability_gate.rs:84-160 (allow_tool_with_origin) | GROUNDED | fn allow_tool_with_origin at line 96; docstring at line 83; CapabilityVerdict at line 96+
capability_flow.rs:180-540 (check_against_commitment / taint_text / taint_from_verdict) | GROUNDED | check_against_commitment:187, taint_from_verdict:219, taint_text:234; note: taint_text is at :234 not :534 (the grounding map's :534 is a test fn, not production -- see VALIDATION WARNINGS)
predicate.rs:100-120 (AstPattern + Composite + SchemaRequired) | GROUNDED | Predicate enum at line 100; AstPattern arm:102; SchemaRequired arm:121; Composite arm:139
G-1 plan at docs/thefinalglance/G-plan/G-1_taint_propagation_dataplane.md | GROUNDED | file exists; H-20 correctly flags G-1 as ABSENT in live code (NN-4 confirmed)
H-0_INDEX.md doctrine | GROUNDED | file exists at docs/thefinalglance/H-plan/H-0_INDEX.md

CLAIM CLASSIFICATION
taint primitive ships at provenance.rs:145, capability_gate.rs:84, capability_flow taint_text:534 | VERIFY | VERIFIED: from_web:145 MATCH; allow_tool_with_origin:96(not:84, :84=comment) NEAR-MATCH; taint_text:234(not:534, :534=test) LINE MISMATCH in grounding map -- production fn is at :234
rules engine AstPattern+Composite+SchemaRequired shipped (predicate.rs:102, rule_engine.rs:98) | VERIFY | VERIFIED: predicate.rs:100 enum Predicate; AstPattern:102; SchemaRequired:121; Composite:139; rule_engine.rs:98=CompiledPredicate enum -- all present
G-1 taint_flow ABSENT (NN-4/TD-5) | VERIFY | VERIFIED: no taint_flow.rs found; grounding map:198 explicitly marks absent; G-1 plan exists but G-1 not yet executed
leakage-specific predicate ABSENT (NN-4) | VERIFY | VERIFIED: grep found no leakage_composite.rs; grounding map:199 marks absent -- correctly stated BUILD surface
Subotic & Yang ASE'22: Datalog-over-SSA, 92.9% accuracy, ~30% of 100k notebooks leak | INGEST | paper cited by name+conf only; no arXiv ID given; NOT ingested in workflow/papers/ or .claude/papers/; exact arXiv ID unresolved by search
arXiv:2211.16073 (Drobnjakovic/Subotic/Urban, abstract interpretation proves absence across disjoint splits) | INGEST | arXiv:2211.16073 verified title = "Abstract Interpretation-Based Data Leakage Static Analysis" -- title MATCHES claim; paper NOT ingested in workflow/papers/
Kapoor & Narayanan "Leakage and the Reproducibility Crisis" (294 papers) | INGEST | cited by title; no arXiv ID in H-20; NOT ingested; paper is well-known (NeurIPS 2022 / Science paper); arXiv ID unresolved by automated search -- needs manual ingest
baseline+1 from LIVE 207/484 (SSB schema/variants for LeakageBlocked) | VERIFY | VERIFIED-STALE: live ssb.rs SCHEMA_VERSION=224 (ssb.rs:3010); TOTAL_VARIANTS=501 (action_type_lock.rs:645); e45:294 pins 501 not 484 -- H-20 must re-read at coding time (grounding map header explicitly warns 207/484 will move; H-20 correctly says "re-read + bump")
Abstract interpretation proves absence across disjoint splits (claim about soundness) | INGEST | soundness claim grounded in arXiv:2211.16073 (verified title); paper not ingested; downgrade to VERIFY if title-match is sufficient basis
FAIL-CLOSED suppression allowlist (registered row required; no silent skip) | GROUNDED | directly mirrors wiring-enforcement.md R1 (missing evidence = hard FAIL) + stub_ban in WIRING_CONTRACT; internally consistent
off-agent artifact = dataflow reachability result (R4/R5 wiring-enforcement) | GROUNDED | wiring-enforcement.md R4 (measure real objective) + R5 (sound reference not leaf-name grep); H-20 WIRING_CONTRACT matches
TD-3: target-derived/sampling-bias -> T3 advisory only (never faked sound verdict) | GROUNDED | H-TRUST-GROUNDING-MAP.md:142-149 TD-3 verbatim; design doc:319-339 matches
TD-5: honest residue, name undecidable classes | GROUNDED | H-TRUST-GROUNDING-MAP.md:149 TD-5; design doc:339-340 matches

ACTIONS REQUIRED
- ingest Subotic & Yang ASE'22 "Datalog-over-SSA data leakage detection in notebooks" via paper-ingestor mode=paper; search arxiv for "Subotic Yang ASE 2022 datalog notebook leakage" to resolve arXiv ID before coding the dataflow pass
- ingest arXiv:2211.16073 (Drobnjakovic/Subotic/Urban, abstract interpretation leakage) via paper-ingestor mode=paper arxiv_id=2211.16073 before TESTING_FIRST
- ingest Kapoor & Narayanan "Leakage and the Reproducibility Crisis" via paper-ingestor mode=paper; search for NeurIPS 2022 or Science 2023 publication + arXiv ID before coding the train/test-sink reachability predicate
- verify at coding time: taint_text is at capability_flow.rs:234 (not :534 as stated in the grounding map); Reads section cites :180-540 which covers both -- range is correct, but the grounding map's per-row :534 is wrong; update H-TRUST-GROUNDING-MAP.md:196 line number to :234 in a patch
- verify at coding time: re-read live SCHEMA_VERSION and TOTAL_VARIANTS before bumping (currently 224/501, not 207/484); H-20 correctly instructs "re-read + bump e45:294" but the stated baseline is stale

VALIDATION WARNINGS
- taint_text line mismatch: H-TRUST-GROUNDING-MAP.md:196 and H-20 both cite capability_flow.rs:534 for taint_text (production fn); actual production fn is at line 234; line 534 is a test body (taint_text_clean_input). The Reads range :180-540 is wide enough to cover the real fn, so EXECUTING is not blocked, but the grounding map row should be corrected.
- three data-leakage prior-art papers (Subotic & Yang ASE'22; arXiv:2211.16073; Kapoor & Narayanan) are not ingested -- they ground the COMPOSITE's soundness scope and the 92.9% accuracy claim; ingest before implementing the dataflow predicate.
- SCHEMA_VERSION/TOTAL_VARIANTS baseline in H-20 (207/484) is stale at time of sweep (live: 224/501). H-20 correctly instructs re-read at coding time; no blocking issue but executor must not use the stated baseline without re-reading.
