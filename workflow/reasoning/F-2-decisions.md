F-2a code-vuln security ruleset decisions (session s-6344, Unit A co-ship with F-1)

What I will implement
F-2a only this unit: populate the always-on security rule domain with vuln-class patterns
(SQLi / XSS / weak-crypto / missing-authz), scoped to Python + TypeScript first to bound false
positives, that BLOCK at write-time on the PreToolUse path. New SSB SecurityRuleBlocked variant.
SecurityRulesPanel dashboard pair. NO performance claim (that is F-2b, gated).

Key decisions (grounded)
- Ruleset as data: crates/rules/seed/security/{sqli,xss,weak_crypto,missing_authz}.toml. Predicate
  shape per crates/rules/src/predicate.rs (GrepPattern/AstPattern; FileContext::Production at :83;
  confirm serde casing by reading the derive before authoring TOML). Loaded by
  rule_loader::load_seed_rules_from_dir (orchestrator) via a security/ dir path registered in
  project_rule_seeder::seed_rules. mutable=0 on every security row (cannot be disabled per-project).
- Write-time BLOCK: rows ride the existing DefaultRuleEngine evaluate() on the PreToolUse path; a
  security-domain BLOCK emits SsbActionType::SecurityRuleBlocked (payload: rule_id, cwe, file_path,
  language, candidate_blake3) alongside the existing hook block path. as_sql_str: security_rule_blocked.
- Shared SCHEMA bump with F-1: one atomic 195->196, TOTAL_VARIANTS 471->473 (VibeGateBlocked +
  SecurityRuleBlocked). Do not bump twice.
- Frontend pair (W-4): SecurityRulesPanel.svelte (DrawerKey 'security-rules', DrawerKeyUpper
  'SECURITY-RULES') + commands_f2.rs::get_security_blocks_recent. get_insecure_rate_latest is added in
  Tier-2 with F-2b (the panel shows blocks now, insecure-rate later).
- CWE mapping: SQLi=CWE-89, XSS=CWE-79, weak-crypto=CWE-327, missing-authz=CWE-862. Each TOML rule
  carries its cwe in metadata for the SSB payload + the F-2b benchmark later.

Grounding posture
- SVEN 59.1->92.3% and CodeGuard+ 43.2->76.0% are VERIFIED but on CodeGen-2.7B; they do NOT transfer
  to Qwen2.5-Coder-7B-abliterated. F-2a makes NO performance claim. The measured before/after
  insecure-rate is F-2b (RunPod-gated): ships as a deterministic SecCodePltRunner fixture + a separately
  gated live-run checkbox. SVEN/CodeGuard+/SeCodePLT paper-ingest is an F-2b pre-gate, not F-2a.

Expected test outcome
A generated Python f-string SQL concatenation Edit is refused pre-write with a security_rule_blocked
row (CWE-89). False-positive bound: 0 FP on a clean corpus of the 20 most-used real ELAI deps; bound
and measured before release. action_type_lock + golden green with the shared +2.

Unknowns / risks
- FileContext serde casing (production vs Production) - read predicate.rs derive first.
- Whether seed_rules accepts a mutable parameter or needs a low-level INSERT - read project_rule_seeder
  :68-160 first.
- FP rate on real TS/Py - the ruleset must be precise (anchored patterns), not broad regex.

================================================================================
F-2b - the measured insecure-rate keystone (session s-60824, continues s-6344)
================================================================================

What I will implement (F-2b deterministic half; F-2a above already SHIPPED + live)
A SeCodePltRunner (crates/orchestrator/src/benchmarks/seccodeplt_runner.rs) + a SeCodePLT-structured deterministic fixture corpus that drives the REAL evaluate_vibegate scorer OFFLINE (no live model) and computes a before/after insecure-rate, emits one InsecureRateMeasured SSB flat beacon + a seccodeplt-*.json report, surfaced as a before/after tile on the existing F-2a SecurityRulesPanel. CLI: `elai bench seccodeplt` (mirrors the prmbench bench_cmd.rs path; NO --suite flag). The LIVE Qwen2.5-Coder before/after number is a SEPARATE RunPod-gated checkbox (NOT run; user asleep + sidecar-boot HARD RULE).

Key technical decision
The keystone number is ELAI's REAL static write-gate measured on a SeCodePLT-structured CWE corpus ("a real number on its own engine", R1) - NOT a reimplementation of SeCodePLT's dynamic-test methodology. before = corpus ground-truth insecure-rate (all candidate completions); after = insecure-rate of the gate-APPROVED subset (what reaches disk) = the static gate's false-negative residual; gate_recall = (before-after)/before. Computed by the production scorer, deterministic, no model/no sandbox (true W-5).

Expected test outcome
cargo check --workspace --tests GREEN; f2_seccodeplt_fixture_smoke + the contract assert the corpus is non-degenerate (before > after, both COMPUTED not literal) and the SSB round-trips; live `elai bench seccodeplt --json` emits a real insecure_rate_measured row + a report JSON with insecure_rate_before/after.

Unknowns to verify empirically
1. The measured before/after + gate_recall on the corpus (workflow/reasoning/F-2b-calibration.md; must be non-degenerate, no hard-coded literal).
2. Live all_test_payloads / TOTAL_VARIANTS counts (RE-COUNT; baseline 477->478, SCHEMA 200->201, all_test_payloads 395->396).
3. Whether `elai bench` already has a --json flag / report-dir convention to reuse (mirror bench_prmbench).

FORK (anti-sycophancy, grounded-by-default) - static gate vs SeCodePLT dynamic tests
The paper-ingestor (workflow/papers/seccodeplt-interface.md, arXiv:2410.11096 VERIFIED) established SeCodePLT measures insecure-rate via DYNAMIC unit-test execution: 3-way label (incorrect = fails functionality; correct-insecure = passes functionality but triggers the vuln; correct-secure = passes both). insecure-rate = correct_insecure/total, and CRITICALLY insecure-rate != 1 - secure_rate. SeCodePLT explicitly REJECTS regex/rule + LLM-judge scoring. ELAI's actual production control is the F-1 STATIC regex/rule write-gate (evaluate_vibegate); these measure different things.
  (a) reimplement SeCodePLT's dynamic harness (multi-language capability+safety test execution + sandbox) - REJECTED for the deterministic ship: measures something ELAI's gate does NOT do, needs untrusted-code execution + per-language toolchains, and cannot produce the headline number without a live model (gated). Misdirects effort from the keystone thesis.
  (b) measure ELAI's REAL static write-gate on a SeCodePLT-STRUCTURED corpus - CHOSEN. Deterministic, exercises the real production scorer (true W-5), and is exactly the pre-write before/after the keystone sells.
HONESTY: the shipped number is named a STATIC-gate insecure-rate, DISTINCT from SeCodePLT's dynamic secure-coding rate. The dynamic 3-way metric + functional pass@1 + the live Qwen number are the RunPod-gated live-run checkbox; the pi's dynamic methodology + the "insecure-rate != 1 - secure_rate" rule are captured in seccodeplt-interface.md as the reference for that gated run.

NON-TARGETS (sg GROUNDED - must NOT appear as F-2b's number/target/ceiling/panel copy)
- SVEN 59.1->92.3% (arXiv:2302.05319, prefix steering) - VERIFIED but CodeGen-2.7B, no transfer to Qwen.
- CodeGuard+ 43.2->76.0% (arXiv:2405.00218, constrained decode) - VERIFIED but CodeGen-2.7B, no transfer.
- SeCodePLT GPT-4o 55% (cite seccodeplt.github.io primary page) - reference only.
- "below 15%" + GitGuardian replay-corpus framing - PROHIBITED per synthesis 1.4.
The only shippable F-2b percentage is a MEASURED Qwen-abliterated run (gated). The deterministic fixture ships now and reports ELAI's static-gate before/after.

CORPUS + GROUNDING
SeCodePLT-structured fixture (>=40 CWE-tagged samples, Python/TS) each: task spec + vulnerable_code + patched_code + ground-truth label + CWE (mirrors SeCodePLT per-sample STRUCTURE, NOT its 5.9k scale). Calibration (mirrors F-10): evaluate_vibegate must FLAG every vulnerable variant (recall) and PASS every patched variant (FP) - grounds the labels + measures gate recall. Record measured before/after/gate_recall/recall/FP in F-2b-calibration.md (all COMPUTED, no literals).

SSB + WIRING (mirror the F-10 cascade)
InsecureRateMeasured flat beacon { suite, model_id, n_samples:u32, insecure_rate_before:f64, insecure_rate_after:f64, gate_recall:f64, seed:u64, corpus_blake3:String }. 11-site ssb.rs lockstep + action_type_lock.rs (ALL_VARIANTS + exhaustive_match_guard) + golden ("insecure_rate_measured" alpha-inserted between injection_fixture_replayed and instructor_mode_activated, renumber). Counters LOCKSTEP: SCHEMA 200->201, TOTAL_VARIANTS 477->478, all_test_payloads 395->396 (RE-COUNT). blake3 corpus digest. Ssb::new inside rt.block_on. W-2: bench beacon always-emitted on run; NO new ELAI_*_ENABLE (do NOT pattern off the legacy wire-dark ELAI_SECURITY_RULE_BLOCKED_ENABLE).
CLI `elai bench seccodeplt [--json]` (mirror bench_prmbench.rs/run_bench_prmbench/CliAction::BenchPrmbench + args_bench.rs if-ladder). Dashboard EXTEND the shipped F-2a pair: commands_f2.rs get_insecure_rate_latest (mirror get_security_blocks_recent) + lib.rs register + SecurityRulesPanel before/after tile + types.ts InsecureRate DTO; cross_lang_diff pair; DrawerKey 'security-rules' reused.

OPERATIONAL NOTE: local-rag RAG/reranker subsystem degraded this session (IPC timeout on ingest/memorize/search; known Windows reranker crash). mcp_think + mcp_fsm + memory-bank direct writes still work. seccodeplt-interface.md is complete on disk; RAG re-index + mcp_memorize of it are deferred recovery items (clear primary.pid + restart).

================================================================================
F-2b Slice A3 - bench CLI + W-5 live trigger (session s-60824)
================================================================================
SHIPPED: `elai bench seccodeplt [--json]` end-to-end wiring (cargo check -p elai-cli --tests GREEN, 1m44s).
New file: crates/cli/src/bench_seccodeplt.rs (run/run_inner/write_report/emit_insecure_rate_measured).
  - Constructs the REAL SecCodePltRunner::from_embedded(), runs the gate, writes a timestamped
    seccodeplt-<unix_ms>.json report under <cwd>/benchmarks/seccodeplt/ (mirrors farm_adv_rm benchmarks/<bench>/
    + drift_cmd timestamped-json + handoff.rs UNIX_EPOCH idiom).
  - Emits EXACTLY ONE SsbActionType::InsecureRateMeasured beacon: Ssb::new constructed INSIDE rt.block_on
    (no-reactor panic only catchable at live run), write_payload("cli:bench-seccodeplt", 0, payload, vec![]).
    W-2: always-on bench beacon, NO ELAI_*_ENABLE gate (did NOT copy legacy wire-dark ELAI_SECURITY_RULE_BLOCKED_ENABLE).
  - thiserror BenchSeccodepltError (mirrors bench_prmbench); no .unwrap()/.expect() outside #[cfg(test)].
5 wiring sites (mirror prmbench chain):
  - bench_cmd.rs::run_bench_seccodeplt(json) wrapper (after run_bench_tamper ~L137).
  - args.rs CliAction::BenchSeccodeplt { json } variant (after BenchTamper ~L739).
  - args_bench.rs `if sub == "seccodeplt"` arm + parse_bench_seccodeplt_args (only --json, NO --suite);
    guard error msg extended to list seccodeplt.
  - main.rs `mod bench_seccodeplt;` decl (after mod bench_tamper ~L230) + CliAction::BenchSeccodeplt dispatch (~L808).
SSB substrate (variant/payload/test_payload/all_test_payloads) already landed in Slice A1 (SCHEMA_VERSION 201);
A3 only consumes it (InsecureRateMeasuredPayload at elai_orchestrator::ssb, SsbPayload::InsecureRateMeasured).
Live `elai bench seccodeplt --run-trigger` smoke = wiring-contract-validator job (NOT run here; binary not executed).
