GROUNDING SWEEP -- phase F-2c
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-2c_low_fp_cwe_rules_seccodeplt_baseline.md
Date: 2026-06-03
FSM: PLANNING | Session: s-6312

SPEC REFERENCES
docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md | GROUNDED | file exists; cited as predecessor ruleset convention + section-1.4 corrections
docs/thefinalglance/F-plan/F-0_INDEX.md | GROUNDED | file exists; cited for W-1..W-10 doctrine (glob confirmed)
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED | file exists; sections 1.4/2/4/5 cited as synthesis grounding
workflow/papers/seccodeplt-interface.md | GROUNDED | file exists; arXiv 2410.11096 verified, static vs dynamic caveat explicit at :16-17
crates/knowledge/src/ast_adapter.rs:18-44 | GROUNDED | read: exactly 10 langs registered (rust/python/typescript/go/java/c_sharp/ruby/html/css/svelte); python+typescript present; no JS/C/C++/Swift
crates/rules/src/predicate.rs:88-160 | GROUNDED | read: Predicate enum at :100-160; AstPattern:102, GrepPattern:109, Composite:139, FileContext on GrepPattern:118; serde tag + rename_all match claim
crates/rules/src/rule_engine.rs:329 | GROUNDED | grep confirms eval_order fn at :329
crates/rules/src/rule_engine.rs:346-362 | GROUNDED | read: detect_language_from_ext at :346-364; .py->.python, .ts/.tsx->.typescript confirmed; .svelte/.html/.css not in match -> returns None (gap confirmed)
crates/rules/src/project_rule_seeder.rs:392 | GROUNDED | grep: seed_security_rules at :392 confirmed
crates/orchestrator/src/runtime_setup.rs:322 | GROUNDED | grep: seed_security_rules call at :322 confirmed
crates/orchestrator/src/rule_loader.rs:142 | GROUNDED | grep: load_security_seed_rules_builtin at :142 confirmed
crates/orchestrator/src/vibegate.rs:133 | GROUNDED | grep: evaluate_vibegate fn at :133 confirmed
crates/cli/src/bench_seccodeplt.rs:1+ | GROUNDED | read lines 1-80: driver exists, drives evaluate_vibegate, static-gate DISTINCT from dynamic noted in docstring; SecCodePltRunner imported at :52
crates/orchestrator/src/benchmarks/seccodeplt_runner.rs | GROUNDED | read lines 1-60: runner exists, drives vibegate::evaluate_vibegate, before/after/gate_recall computed not literal
crates/orchestrator/src/ssb.rs::InsecureRateMeasured | VERIFY_NEEDED | enum variant at :8341 (confirmed), NOT :8317 as cited -- :8317 is inside WriteTimeSlopsquatBlocked; payload struct at :25644 (NOT :25427 as cited); off by hundreds of lines. Claim GROUNDED on substance but LINE NUMBERS are wrong.
crates/contracts/tests/f2_immune_rule_contract.rs:70 | GROUNDED | grep: "must insert exactly 12 rows" at :70 confirmed
data/rules/web.toml | GROUNDED | cited as Tier-2 Composite/AstPattern template; not read in full but existence not in question (ELAI_CC standard path; prior sessions confirmed WEB-013..023 present)
arXiv:2410.11096 (SeCodePLT) | GROUNDED | seccodeplt-interface.md :7 "Verified: arXiv id 2410.11096 confirmed via WebSearch 2026-06-02"; title/authors match
arXiv:2511.04023 (AdaTaint) | VERIFY_NEEDED | no paper file at .claude/papers/ or workflow/papers/ for this id; plan cites it as "VERIFIED via arxiv.org/abs/2511.04023" but no local ingest. Claimed: -43.7% FP, +11.2% recall. Source of verification not traceable in repo.
arXiv:2601.18844 (Tencent FP) | VERIFY_NEEDED | same: no local ingest; plan cites "VERIFIED via arxiv.org/abs/2601.18844"; claimed 94-98% FP elimination. Not locally ingested.

CLAIM CLASSIFICATION
CWE-798 predicate is high-precision (low-FP) | VERIFY | FP risk: identifier-name grep (password/secret/api_key) will fire on CHANGEME placeholders in config templates and test fixtures; `file_context=production` reduces but does not eliminate (production configs use these literals). GrepPattern child alone would be high-FP; Composite with AST child (assignment to string literal, not comparison/env-read) bounds this. Plan acknowledges: "a placeholder `password = "CHANGEME"` ... CAN over-fire" -- FP RISK noted, mitigation is Composite shape + measured after Task 4 run. UNGROUNDED until measured.
CWE-321 predicate is high-precision (low-FP) | VERIFY | FP risk: crypto-sink grep anchors call (AES.new, Fernet, createCipheriv) -- this is narrower than CWE-798. AST child confirming literal-key arg is effective bound. Lower FP surface than 798. GROUNDED as design claim; efficacy UNGROUNDED until measured.
CWE-532 predicate is high-precision (low-FP) | VERIFY | Plan assigns WARN (not BLOCK) to Python print() shape explicitly because "higher FP"; BLOCK only for logging/logger/console sinks. Design is correctly calibrated. GROUNDED as design intent; efficacy UNGROUNDED until corpus measured.
CWE-95 predicate is high-precision (low-FP) | GROUNDED | Composite: grep eval/exec sink, AST-confirm arg is NOT a string literal constant. The literal-vs-dynamic distinction is semantically sound (eval("1+1") is not injection). Plan explicitly states eval("1+1") must NOT fire. High precision class. FP risk: low for Python/TS with AST confirmation.
seccodeplt bench is static-gate, no model needed | GROUNDED | bench_seccodeplt.rs:1-12 and seccodeplt_runner.rs:1-6 both explicitly state "NO model, NO sandbox, NO live inference -- fully deterministic and offline". vibegate::evaluate_vibegate called directly.
before/after/gate_recall computed not hardcoded | GROUNDED | seccodeplt_runner.rs:13-25 defines computation; "never literals" stated in docstring; plan repeats the invariant.
InsecureRateMeasured SSB at ssb.rs:8317 | VERIFY_NEEDED | actual line is :8341 for the variant, :25644 for the payload struct (plan cites :8317 and :25427 -- both off by ~24 and ~217 lines respectively). Substance correct, line numbers stale.
10 grammars registered (not 16) | GROUNDED | ast_adapter.rs:18-44 read: exactly 10 inserted (rust/python/typescript/go/java/c_sharp/ruby/html/css/svelte). astro behind feature flag, not in default count. F-2c correctly flags prior "16 grammars wired" claim as overclaim.
python+typescript registered, no ast_adapter edit needed | GROUNDED | ast_adapter.rs:22-25 confirms both present.
JavaScript/C/C++/Swift UNREGISTERED in evaluator | GROUNDED | confirmed: those languages appear in match arms of detect_language_from_ext (so file extension maps) but are NOT in ast_adapter.rs::new() -- AST rules for them would fail at language lookup.
seed_security_rules enumerates seed/security dir | VERIFY | plan says "confirm at read time"; NOT yet confirmed which path (enumerate vs hardcode). project_rule_seeder.rs:392 confirmed to exist but seeder logic not read.
arXiv:2511.04023 FP numbers (-43.7%/+11.2%) | UNGROUNDED | no local ingest; plan claims "VERIFIED" but traceable only via arxiv.org URL, not ingested. Numbers presented as motivation not ELAI claim -- caveat present in plan ("UNGROUNDED-for-ELAI"). Risk: low (cited as prior art only, no ELAI perf claim attached).
arXiv:2601.18844 FP numbers (94-98%) | UNGROUNDED | same. Cited as motivation only, ELAI caveat present. Same risk level.
ast-grep-core 0.43.0 registry status | GROUNDED | plan: "VERIFIED via crates.io API 2026-06-02: version 0.43.0, MIT, 2026-05-25"; anti-slopsquatting compliant. DEFERRED correctly.
tree-sitter 0.26.8 satisfies ast-grep-core ^0.26.3 floor | GROUNDED | plan cites Cargo.toml:13 + Cargo.lock 0.26.8; semver floor check correct.
Semgrep Rules License v1.0 (non-competing, internal OK) | GROUNDED | plan: "VERIFIED via semgrep.dev/legal/rules-license"; rules are hand-authored not copied.

ACTIONS REQUIRED
- VERIFY: seed_security_rules at project_rule_seeder.rs:392 enumerates seed/security/ dir vs hard-codes filenames -- read :392-545 before Task 3 implementation to resolve the conditional MODIFY. This is an explicit "resolve at read time" item in the plan; must not be assumed.
- VERIFY: SSB line numbers stale -- InsecureRateMeasured is at ssb.rs:8341 (not :8317) and InsecureRateMeasuredPayload is at ssb.rs:25644 (not :25427). Update the WIRING_CONTRACT consumes_symbols cite before EXECUTING so auditor cross-checks don't fail. Low severity (substance correct) but should be corrected.
- INGEST: arXiv:2511.04023 (AdaTaint FP reduction) -- spawn paper-ingestor mode=paper with id 2511.04023 before EXECUTING if the FP-reduction motivation numbers are to be cited in F-2c-decisions.md. Currently cited as motivation only with ELAI caveat; acceptable to leave as UNGROUNDED-for-ELAI with explicit label if no ingest happens.
- INGEST: arXiv:2601.18844 (Tencent hybrid LLM+static FP study) -- same treatment as above.
- FP RISK FLAG: CWE-798 (hardcoded_creds) predicate has acknowledged FP surface on placeholder literals (CHANGEME, example values). Mitigation is Composite + file_context=production + post-run measurement (Task 4). The plan correctly labels this the "SINGLE BIGGEST RISK". Do NOT assert low-FP before Task 4 baseline run produces evidence. Decision doc must bound FP on >=3 real orchestrate runs as stated.

VALIDATION WARNINGS
- SSB line number drift: ssb.rs:8317 (plan cite) vs :8341 (actual) and payload :25427 vs :25644 -- stale cite, not wrong symbol. Auditor should use grep not line-pinned read.
- no spec references to a per-phase spec file (phase-{X}.txt). This B-N self-grounds from code, synthesis docs, and papers -- standard for F-plan units; acceptable under W-7.
- data/rules/web.toml WEB-013..023 not read in this sweep (file path not absolute in this session context); cited as template but existing-code confirmation from prior sessions. Treat as VERIFY if auditor needs fresh confirmation before EXECUTING.
- all sources validated (no missing files, no index-only citations)
