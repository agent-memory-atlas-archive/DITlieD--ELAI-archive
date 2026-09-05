GROUNDING SWEEP -- phase E-F10
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-10_security_monotonicity_iter_guard.md
Date: 2026-06-01
FSM: EXECUTING | Session: s-6344

SPEC REFERENCES
No "Spec references:" block present. Source file cites synthesis doc + elai.map + arXiv as references.
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C10 | GROUNDED | confirmed: synthesis line 254-274 "C10. Per-iteration security-monotonicity gate" contains full mechanism + REVISED SCOPE + phase split
synthesis section 1 (corrections) | GROUNDED | confirmed: lines 71-113 record the v1 stale numbers correction (185/9x/20 -> 135/11x/12 v3) and discipline for C10
synthesis section 2 (tiered-gate rule) | GROUNDED | confirmed: lines 114-145 contain THE DESIGN RULE (Tier-1 in-process / Tier-2 post-write async) that governs F-10
synthesis section 5 (TIER-2 sequencing) | GROUNDED | confirmed: lines 450-456, C10 at position 7: "classifier first" TIER 2 ordering confirmed

CLAIM CLASSIFICATION

Code anchors (consumed symbols):

ReflectEventSink trait at chat_reflect.rs:294 | GROUNDED | elai.map: DEF_TRAIT at chat_reflect.rs:294 + confirmed via grep: line 294 "pub trait ReflectEventSink: Send + Sync {"
ChatReflectPipeline::new at chat_reflect.rs:314 | GROUNDED | elai.map: DEF_FN at 314; grep confirmed line 314 "pub fn new(config: ChatReflectConfig, sink: Option<Arc<dyn ReflectEventSink>>) -> Self {"
ChatReflectPipeline struct at chat_reflect.rs:306 | GROUNDED | grep confirmed "pub struct ChatReflectPipeline {" at line 306
parse_auditor_verdict at chat_reflect.rs:638 | GROUNDED | grep confirmed "fn parse_auditor_verdict" at line 638
live_cli.rs:1518 production ChatReflectPipeline instantiation | GROUNDED | elai.map CALL + CALL_RESOLVED at live_cli.rs:1518; grep confirmed "self.reflect_pipeline = Some(ChatReflectPipeline::new(cfg, sink));" at line 1518
task_runner.rs:13105-13168 AutonomyDecision evidence + GateDecision write | MISMATCH | plan cites 13105-13168; REAL anchors are: use crate::autonomous_gate::AutonomyDecision at line 13205; ("auto_advance"..., "all_checks_passed") at line 13208; error!(...write GateDecision failed) at line 13257. The range 13105-13168 was NOT the correct site (content there is B-41 derive_phase_zero_blocks and derive_framing_evidence helpers). Stale synthesis-time estimate (~3996) corrected in F-10 to "13105-13168" but real lines are 13205-13257.
autonomous_gate.rs::should_auto_advance at line 178 | GROUNDED | elai.map DEF_FN at autonomous_gate.rs:178; grep confirmed "pub fn should_auto_advance("
promotion_gate.rs:103 two_gate_promote | GROUNDED | grep confirmed "pub fn two_gate_promote(" at line 103
promotion_gate.rs:176-215 test coverage | GROUNDED | elai.map CALL entries at 176, 187, 201, 215
external_benches.rs:557 two_gate_promote re-export | GROUNDED | grep confirmed "crate::promotion_gate::two_gate_promote(&candidate, descriptor, archive, cfg)" at line 557
tripwire.rs::TripwireKind enum at line 53 | GROUNDED | grep confirmed "pub enum TripwireKind {" at line 53; variants: KlDivergence / CorrelationDrop / IqrOutlier / Perplexity / Ddm
tripwire.rs::TripwireMonitor struct at line 592 | GROUNDED | elai.map DEF_STRUCT at 592
tripwire.rs::perplexity_anomaly at line 532 | GROUNDED | elai.map DEF_FN at 532
tripwire.rs::kl_divergence at line 432 | GROUNDED | elai.map DEF_FN at 432
observe_post_consensus_tripwires called in tripwire_observable_smoke.rs:114 | GROUNDED | grep confirmed; BUT: function is defined in agent_pool.rs (not tripwire.rs) -- plan does not miscite the test file location (smoke.rs:114), only the function's home module is ambiguous in the plan
ssb.rs:8779 parse arms region | GROUNDED | elai.map CALL entries at 8779, 8793, 8817 etc all in "tionType::PhaseComplete)" arm region
reflect_bridge.rs:124-177 ReflectEvent::Triaged JSON | GROUNDED | grep + read confirmed: line 124 "ReflectEvent::Triaged { turn_id, class }" within fn emit; lines 124-177 cover the match arms
commands.rs:5657 reflect_pipeline construct | GROUNDED | elai.map CALL at 5657: "let client = std::sync::Arc::new(crate::reflect_bridge::HttpInferenceClient::new("

CWE classifier status:
cwe_classifier.rs does NOT exist | GROUNDED | ls safety/src/ shows: capability_gate.rs, destructive_verb_allowlist.rs, injection_classifier.rs, lib.rs, prompt_armor.rs, provenance.rs, spotlight.rs, tool_description_pin.rs, trust_dialog.rs -- NO cwe_classifier.rs. Plan correctly states "it does NOT exist yet"
safety crate exists (E-30) | GROUNDED | ls confirmed crates/safety/ with src/, benches/, policies/, Cargo.toml

Schema version claims:
SCHEMA_VERSION 191 (CLAUDE.md) | MISMATCH | Real value: ssb.rs line 2742 "const SCHEMA_VERSION: i64 = 196;" -- the plan says "baseline 191 per CLAUDE.md" and maps baseline+1=192, but actual baseline is 196, so the bump should be 196->197. CLAUDE.md value (191) is stale.
TOTAL_VARIANTS 471 (plan W-7 comment) | MISMATCH | Real value: action_type_lock.rs line 490 "const TOTAL_VARIANTS: usize = 473;" -- plan cites W-7 note as "SCHEMA_VERSION is actually 195, TOTAL_VARIANTS 471 - CLAUDE.md 191/464 is stale" which is itself wrong (real: 196/473).

arXiv citations:
arXiv:2506.11022 Shukla +37.6% critical vulns / 5 iterations / 400 samples | VERIFY | synthesis VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md line 260 marks this VERIFIED per adversarial pass (run wf_b959963a-db6). Paper NOT locally ingested (not in workflow/papers/ or .claude/papers/). Synthesis-level VERIFIED is the grounding -- acceptable for motivation claim with explicit "MOTIVATION, never imported as ELAI target" discipline. INGEST recommended before phase-2 ships to upgrade to PRIMARY.
arXiv:2507.02976 v3 Sajadi ~11x vulns (135 vs 12) CWE-78/95/502/22 | VERIFY | synthesis line 261-262 marks VERIFIED. v3 numbers confirmed in synthesis (135/11x/12). v1 stale (185/9x/20) explicitly corrected in section 1 -- correction is grounded. Paper NOT locally ingested. Same as above -- synthesis-level VERIFIED, INGEST recommended.

Design claims:
Security degrades monotonically in AI self-improvement loops | GROUNDED | synthesis section 4 C10 lines 254-274 confirms GENUINELY_NEW + VERIFIED grounding from Shukla + Sajadi
CWE classifier is GREP/AST in-process (Tier-1) | GROUNDED | synthesis section 2 tiered-gate rule at lines 114-145 specifies exactly this design split
Post-write async feeding auditor (NOT blocking hot path) | GROUNDED | synthesis section 2 line 135 "run the CWE classifier on the delta as a post-write async check feeding the auditor"
FP<5% on 50 clean patches per language is design target, NOT transferred number | GROUNDED | synthesis section 4 C10 "measured FP<5% on 50 clean patches per language" -- plan correctly marks this as a DESIGN TARGET to be measured (phase-1 acceptance criterion)
Semgrep CWE rules sparse for Rust | GROUNDED | synthesis line 268 "ELAI's dominant language, where semgrep CWE rules are sparse" -- this is a KNOWN LIMITATION label, not a claim to verify
Pattern-2 single-variant SSB with loop_kind discriminator | GROUNDED | CLAUDE.md + F-0 W-9 mandate Pattern-2 collapse; plan correctly implements one SecurityClassRegressionBlocked variant

ACTIONS REQUIRED
- MISMATCH: task_runner.rs injection site -- plan cites lines 13105-13168 but real AutonomyDecision evidence block is at lines 13205-13257 (use statement at 13205, tuple at 13208, error! at 13257). Update the line range in WIRING_CONTRACT consumes_symbols and Reads section before TESTING_FIRST. The symbol names are correct; only the line numbers are wrong.
- MISMATCH: SCHEMA_VERSION -- plan says "baseline 191 -> baseline+1 = 192" but real SCHEMA_VERSION is 196; bump must be 196->197. Update Task 9 and ACCEPTANCE item and WIRING DONE-GATE W-9 note before TESTING_FIRST.
- MISMATCH: TOTAL_VARIANTS -- plan's W-7 parenthetical says "SCHEMA_VERSION is actually 195, TOTAL_VARIANTS 471" but real values are 196/473. The instruction to "never hard-code, say baseline+1" is correct but the stale parenthetical is misleading; remove or correct the parenthetical.
- VERIFY (upgrade recommended): ingest arXiv:2506.11022 (Shukla) via paper-ingestor mode=paper before phase-2 ships. Synthesis-level VERIFIED is acceptable for motivation claim but PRIMARY grounding strengthens the +37.6%/400-samples numbers.
- VERIFY (upgrade recommended): ingest arXiv:2507.02976 v3 (Sajadi) via paper-ingestor mode=paper. Confirm v3 is the current version and 135/12 numbers are stable before citing in decision doc.

VALIDATION WARNINGS
No "Spec references:" block in source -- consistent with F-plan format (cites synthesis doc + elai.map directly). Not a defect given the F-plan citation model.
The plan's W-7 parenthetical "(SCHEMA_VERSION is actually 195, TOTAL_VARIANTS 471 - CLAUDE.md 191/464 is stale)" contains stale values: real is 196/473 as of this sweep.
observe_post_consensus_tripwires is defined in agent_pool.rs (not tripwire.rs) -- plan references the smoke test at tripwire_observable_smoke.rs:114 correctly, but does not claim a tripwire.rs definition. Minor imprecision in "called in tripwire_observable_smoke.rs:114" (confirmed accurate per grep).
