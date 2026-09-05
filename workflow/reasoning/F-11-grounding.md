GROUNDING SWEEP -- phase F-11
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-11_close_one_config_generation.md
Date: 2026-06-03
FSM: PLANNING | Session: s-6312

SPEC REFERENCES
docs/specs/guide/steps/phase-4H.txt (no line range) | GROUNDED | phase-4H.txt:1-14 "DGM ConfigVariant archive (4H.3)"; phase-4H.txt:26-27 cites arxiv 2505.22954v3; on-topic for config archive, DGM math
workflow/papers/dgm-interface.md | GROUNDED | file exists at workflow/papers/dgm-interface.md; lines 1-40 confirm DGM loop (propose->evaluate->insert->select), lambda=10/alpha_0=0.5 formulas, SWE-bench 20.0->50.0 + Polyglot 14.2->30.7 paper numbers
arXiv:2505.22954 (DGM) | GROUNDED | phase-4H.txt:26-27 + dgm-interface.md:1 confirm paper resolves to Zhang/Hu/Lu/Lange/Clune; DGM math ported correctly to config_archive.rs:45-83
arXiv:2510.04399 (Two-Gate) | GROUNDED | promotion_gate.rs:34 "arXiv:2510.04399 does not prescribe a value"; :49 "per arXiv:2510.04399 §Theorem 2"; :52 "per arXiv:2510.04399 §Theorem 11" -- paper resolves to Wang/Dorchen/Jin bounded-capacity result

CLAIM CLASSIFICATION
config_archive insert_variant/select_parent/sample_variant have ZERO production callers | GROUNDED | grep **/*.rs returns exactly 3 files: config_archive.rs + config_archive_contract.rs + lib.rs; no darwin_loop_cmd.rs or other production caller
darwin_loop_cmd.rs:104 drives synthetic alternating reward | GROUNDED | source line 104: `raw = if (arm_idx + gen_idx as usize) % 2 == 0 { 1.0 } else { 0.0 }` -- confirmed
darwin_loop_cmd.rs:127 prints RUNPOD-PENDING terminus | GROUNDED | source line 127: `println!("darwin-loop COMPLETE (RUNPOD-PENDING live evolution loop)")` -- confirmed
main.rs:874 wires CliAction::DarwinLoop | VERIFY | actual line is 889 not 874; wiring is real and confirmed (main.rs:889 CliAction::DarwinLoop -> darwin_loop_cmd::dispatch); line number is stale by 15 lines
is_security_sensitive at self_mod_layer1a.rs:383 | GROUNDED | source line 383: `pub fn is_security_sensitive(row: &serde_json::Value) -> bool` with 10-keyword NEEDLES array -- confirmed
is_security_sensitive on SEPARATE path from config_archive admit | GROUNDED | call sites at :789 + :846 both inside DefaultLayer1aProcessor filter_proposals / apply_proposal; no call exists in config_archive.rs -- bypass risk is real and correctly identified
promotion_gate DEFAULT_TAU=0.05 at :42, K_m UNCONFIRMED at :34 | GROUNDED | source :34-37 `pub const DEFAULT_K_M: u32 = 50` labeled "UNCONFIRMED for ELAI"; :42 `pub const DEFAULT_TAU: f64 = 0.05` labeled "UNCONFIRMED; carries forward bai.rs DEFAULT_POSTERIOR_MARGIN"
two_gate_promote at promotion_gate.rs:109 | GROUNDED | source line 109: `pub fn two_gate_promote(candidate: &ArchiveEntry, ...)` -- confirmed
blake3_event_id at tripwire.rs:254 | VERIFY | function exists at line 254 but is PRIVATE (fn not pub fn); the driver cannot call it directly from darwin_loop_cmd.rs; public tripwire API exists (TripwireMonitor, build_tripwire_fired_payload etc) -- plan must use a public path, not blake3_event_id
insert_variant at config_archive.rs:198 (score-range validation ONLY) | GROUNDED | source line 198-201: `pub async fn insert_variant` checks `!(0.0..=1.0).contains(&v.score)` only; no is_security_sensitive call -- killer-risk #1 bypass confirmed real
select_parent at :274, select_parent_with at :283 | GROUNDED | source lines 274 and 283 confirmed, EntropySource-seeded variant at :283 correct
BenchmarkResult at external_benches.rs:136, BenchTarget at :87 | GROUNDED | source line 136: `pub struct BenchmarkResult`, line 87: `pub struct BenchTarget` -- confirmed
get_suite at external_benches.rs:459, list_suites at :440 | GROUNDED | source line 440: `pub fn list_suites()`, line 459: `pub fn get_suite(name: &str)` -- confirmed
DGM SWE-bench 20.0->50.0, Polyglot 14.2->30.7 performance numbers | VERIFY | dgm-interface.md confirms these are paper-reported figures from the DGM multi-iteration coding-agent run; plan correctly labels them NOT F-11 targets; vendor/paper-reported, not measured by ELAI
tau/K_m design-tunable not calibrated, labeled UNCONFIRMED in source | GROUNDED | promotion_gate.rs:34-42 self-labels both constants as UNCONFIRMED; docstring at :39 "until the held-out canary calibration yields a paper-grounded value" -- correctly flagged as design-tunable
single closed generation positive delta UNGROUNDED (killer-risk #4) | GROUNDED | plan explicitly labels this UNGROUNDED and cites DGM deceptive-dip caveat; dgm-interface.md confirms DGM ran T=80 iterations; acceptance criterion is loop-closure+guard-fire not uplift
config-only mutant avoids Layer-3 human_approved by construction | GROUNDED | self_mod_safety.rs:247-254 `apply_structural_change` only fires on StructuralChange type; ConfigVariant is a different type; plan's claim is correct by type-level construction
arXiv:2510.04399 bounded-capacity requirement justifies config-only scope | GROUNDED | promotion_gate.rs:34,49,52 confirm the paper is cited in source; docstring states "capacity is the supremum boundary across five modification axes" -- consistent with plan's bounded-capacity justification
blake3 for candidate fingerprint (not blake2/sha2) | GROUNDED | tripwire.rs:34 Blake3Hasher; config_archive.rs header cites blake3; CLAUDE.md HARD RULE confirmed; plan's blake3 requirement is correct

ACTIONS REQUIRED
- verify blake3_event_id visibility: tripwire.rs:254 `fn blake3_event_id` is PRIVATE; the darwinian_admit driver cannot call it directly. Before TESTING_FIRST the implementer must identify the public tripwire API surface (TripwireMonitor::observe, build_tripwire_fired_payload, or a new pub wrapper) and update the WIRING_CONTRACT consumes_symbol from `tripwire.rs::blake3_event_id` to the actual public callable. The stub_ban DETERMINISTIC_CHECK step `rg blake3_event_id .elai_cc/crates/cli/src/darwin_loop_cmd.rs` will FAIL unless the function is made public or the call site uses a different public API.
- verify main.rs DarwinLoop line number: plan cites main.rs:874 for CliAction::DarwinLoop; actual line is 889 (stale by 15 lines). Update in F-11-decisions.md and WIRING_CONTRACT comment before EXECUTING so the context-compiler reads the correct range.
- DGM SWE-bench/Polyglot numbers (20.0->50.0, 14.2->30.7): paper-reported, not ELAI-measured. Plan correctly labels them as NOT F-11 targets. No action required beyond confirming the caveat is preserved in F-11-decisions.md.

VALIDATION WARNINGS
- blake3_event_id private-visibility finding is the only substantive blocking issue; all other spec refs GROUNDED or minor line-number drift
- dgm-interface.md exists and is the grounding anchor for DGM math; RAG has no indexed entry for arXiv:2505.22954 or arXiv:2510.04399 but both papers are verified via source-code citations in config_archive.rs and promotion_gate.rs respectively
- No index file references found in the plan (all refs are per-phase or per-source); no VERIFY_NEEDED from index-only citations
- all sources validated except the blake3_event_id visibility issue noted above
