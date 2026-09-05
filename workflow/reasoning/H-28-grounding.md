GROUNDING SWEEP -- phase H-28
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-28_reward_tampering_harness_frozen.md
Date: 2026-06-06
FSM: PLANNING | Session: s-8404 lineage

SPEC REFERENCES
_grounding/H-TRUST-GROUNDING-MAP.md TS-7 | GROUNDED | file confirmed; TS-7 H-28 symbol table at lines 257-263: is_security_sensitive :383, two_gate_promote :109, darwin_admit_loop :298, AllowedConfigKey :203, LockedKeysAuditor :52/80/95, ArchiveVerifyResult ssb.rs:30840 (LINE STALE -- see MISMATCH below), NN-12 gap confirmed
_grounding/H-TRUST-GROUNDING-MAP.md NN-12 | GROUNDED | reward_tampering_counter + RF-INPUT-tamper flag ABSENT confirmed: grep of .elai_cc/crates/orchestrator/src returned zero hits for reward_tampering.rs
_grounding/H-TRUST-GROUNDING-MAP.md TD-2/TD-3/TD-5 | GROUNDED | TD-2 frozen-evaluator at map:139-141; TD-3 ordering at map:143-144; TD-5 Rice/Schmidhuber undecidable residue at map:152 -- cited by name only, no arXiv (see INGEST below)
promotion_gate.rs:100-230 | GROUNDED | two_gate_promote confirmed at :109 (fn signature), tests at :182/:193/:207/:221 -- on-topic for the gate H-28 fronts
darwin_admit_loop.rs:280-310 | GROUNDED | two_gate_promote call confirmed at :298 in live production path (not #[cfg(test)]) -- on-topic
self_mod_layer1a.rs:200-400 | GROUNDED | AllowedConfigKey enum at :203, is_security_sensitive at :383 -- both on-topic
locked_keys_observable_smoke.rs:40-100 | GROUNDED | LockedKeysAuditor observable pattern at :47/:75/:90 -- on-topic; lines match
gate_freeze_check.py | GROUNDED | file confirmed at F:\.ELAI_workflow\.claude\tools\gate_freeze_check.py; implements blake3 freeze check, exit 3 on BLOCK -- on-topic for TD-2
gate-manifest.txt | GROUNDED | file confirmed; current LOCKED set does NOT include H-17 ledger or eval harness -- confirms H-28 is the BUILD step to add them

MISMATCH
ssb.rs:30840 cited as ArchiveVerifyResult location | MISMATCH | ssb.rs:30840 is a TemplateCandidateRejectedPayload field (`rejection_reason: "below_floor"`). ArchiveVerifyResult variant is at ssb.rs:6767 (SsbActionType enum). The symbol is correct; only the line number is wrong in the plan + grounding map.

CLAIM CLASSIFICATION
Everitt current-RF: reward fn private + uninfluenceable | GROUNDED | everitt-reward-tampering-1908.04734-interface.md ingested; arXiv:1908.04734 confirmed; Claim 3 (Section 3.2) maps directly to the frozen-evaluator design
arXiv:1908.04734 verify before citing | GROUNDED | confirmed by interface file: paper is "Reward Tampering Problems and Solutions in RL: A Causal Influence Diagram Perspective", Everitt/Hutter/Kumar/Krakovna; published Synthese 2021
blake3 tripwire catches eval-harness edits (hash mismatch) | GROUNDED | ArchiveVerifyResult confirmed at ssb.rs:6767 (not :30840); blake3 used in darwin_admit_loop.rs:295 `blake3::hash(&blob)` -- on-topic
ArchiveVerifyResult SHIPPED (D-8) | GROUNDED | ssb.rs:6767 SsbActionType::ArchiveVerifyResult; ssb.rs:18878 ArchiveVerifyResultPayload struct
RF-INPUT tampering leg ABSENT (NN-12) | GROUNDED | grep of orchestrator/src returned zero hits for reward_tampering; reward_tampering.rs does not exist
DGM empirically faked test results + disabled detectors | GROUNDED | dgm-interface.md:49 "node 114 removed hallucination markers to achieve perfect score"; :60 "hallucination-checking functions MUST be hidden from coding agent" confirms disabled-detector pattern
two_gate_promote at promotion_gate.rs:109 | GROUNDED | code confirmed; fn two_gate_promote at :109
darwin_admit_loop.rs:298 live two_gate_promote driver | GROUNDED | confirmed at :298; not test-only
AllowedConfigKey closed enum at self_mod_layer1a.rs:203 | GROUNDED | enum AllowedConfigKey at :203 confirmed
is_security_sensitive at self_mod_layer1a.rs:383 | GROUNDED | fn is_security_sensitive at :383 confirmed
LockedKeysAuditor SHIPPED (D-9) | GROUNDED | locked_keys_observable_smoke.rs:47 test fn confirmed; LockedKeysAuditor::new called
gate_freeze_check.py + gate-manifest.txt SHIPPED (R8) | GROUNDED | both files confirmed present
reward_tampering_counter + SSB Pattern-2 "baseline+1 from LIVE 207/484" | UNGROUNDED (STALE) | LIVE SCHEMA_VERSION = 235 (ssb.rs:3182), TOTAL_VARIANTS = 512 (e45 contract :294). The plan says "207/484" baseline; those are stale by 28 schema versions and 28 variants. The bump target is 235->236 and 512->513. The NN-12 BUILD leg is correctly identified as absent; only the baseline numbers are stale.
Structural self-mod -> undecidable (Rice/Schmidhuber) | VERIFY | Rice's theorem is a foundational computability result (no arXiv needed); Schmidhuber self-improving machine undecidability is cited without paper ID. Standard CS reference, not load-bearing for the implementation design (the design consequence is: route to human Layer-3, which is correctly encoded). Classify VERIFY not INGEST -- the implementation conclusion (human Layer-3) does not depend on knowing Schmidhuber's exact paper; the undecidability is a background justification only.
Denison et al. Sycophancy-to-Subterfuge | INGEST | cited in grounding map :284 and H-28 Citations block; no arXiv ID in plan or map; paper not ingested (no file in workflow/papers). H-28 cites it in the Citations block without attaching a load-bearing claim to its specific findings. If any quantitative claim from Denison et al. is load-bearing (e.g., "82% acceptance of fabricated results"), INGEST is required. As cited in H-28, it is contextual (the DGM empirical incident is the load-bearing evidence; Denison adds corroboration). Risk: LOW but flag.
3a BLOCK never overridden by 3b green (TD-3 ordering) | GROUNDED | TD-3 explicitly encoded in H-28 Tasks 4 + stub_ban; WIRING_CONTRACT stub_ban clause confirms; grounding map TD-3 at :143-144
harness + thresholds + ledger in frozen LOCKED set (TD-2) | GROUNDED | gate-manifest.txt confirmed as the freeze mechanism; H-28 Task 1 adds these entries; gate_freeze_check.py enforces at exit 3

ACTIONS REQUIRED
- MISMATCH: ssb.rs:30840 cited in H-28 plan (lines 18, 40, 164, WIRING_CONTRACT :94) and H-TRUST-GROUNDING-MAP.md TS-7 (:263) as the ArchiveVerifyResult location. Actual location is ssb.rs:6767. Correct the line number in both files before EXECUTING. The symbol (ArchiveVerifyResult) and its behavior are correct; only the citation line is wrong.
- STALE BASELINE: H-28 plan line 77 says "baseline+1 from LIVE 207/484, re-read + bump e45:294". Live values at edit time are SCHEMA_VERSION=235, TOTAL_VARIANTS=512. The executor MUST re-read ssb.rs:3182 and action_type_lock.rs at edit time and bump to 236/513 (or whatever live value is at execution time). This is a VERIFY item, not a MISMATCH -- the plan already says "re-read", but the stale numbers in the plan text are a trap.
- INGEST: Denison et al. Sycophancy-to-Subterfuge -- spawn paper-ingestor mode=paper with search term "Sycophancy to Subterfuge Denison" (no arXiv ID known; search required). Required if any quantitative finding from this paper is cited as load-bearing before EXECUTING. Currently contextual only; risk LOW but recommend ingest to close the citation gap.
- verify Schmidhuber self-improving machine undecidability cite -- no action needed for implementation (Rice's theorem covers the structural undecidability claim; human Layer-3 routing is correctly encoded); document as "Rice's theorem is load-bearing; Schmidhuber is supplemental" in H-28-decisions.md.

VALIDATION WARNINGS
- ssb.rs:30840 line-number mismatch is the primary risk: it propagates to the grounding map (TS-7) and to the plan's WIRING_CONTRACT consumes_symbols block. An executor who searches for ArchiveVerifyResult at :30840 will find a TemplateCandidateRejectedPayload field and become confused. Elevating to MISMATCH per spec-grounder rule 4.
- SCHEMA_VERSION 207 / TOTAL_VARIANTS 484 in grounding map are stale (live: 235 / 512). Plan acknowledges "re-read at edit time" but the stale numbers will mislead if not caught.
- No "Spec references:" block in H-28 file; references are under "Reads (mandatory before coding)" and "Citations / Grounding" sections. Processed both. Count is non-zero (8 spec refs verified); no untraceable warning needed.
- Denison et al. has no arXiv ID in any plan or grounding file -- a future paper-ingest or WebSearch is needed to close this citation.
