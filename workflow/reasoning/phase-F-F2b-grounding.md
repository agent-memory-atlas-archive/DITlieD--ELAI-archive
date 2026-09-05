GROUNDING SWEEP -- phase F-F2b
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-2_codevuln_ruleset_measured_number.md
Date: 2026-06-01
FSM: EXECUTING | Session: s-6344

SPEC REFERENCES

F-plan/F-0_INDEX.md W-1..W-10 | GROUNDED | F-0_INDEX.md:39-115 contains all 10 doctrine clauses verbatim; each Wiring Done-Gate section of F-2 maps to a clause
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED | file exists at path; plan cites C2/section 0/1/2/5/7; synthesis doc is the primary mechanism source
predicate.rs:88-200 (Predicate enum + FileContext) | GROUNDED | FileContext DEF at predicate.rs:81, Predicate DEF at predicate.rs:101, AstPattern at :102, GrepPattern at :109, GrepPattern.file_context field at :118 -- all confirmed via elai.map DEF_ENUM rows
rule_engine.rs:180-200 + 380-520 (ALWAYS_INCLUDED_DOMAINS + evaluate) | GROUNDED | ALWAYS_INCLUDED_DOMAINS DEF at rule_engine.rs:187 (confirmed; snippet: &["engineering","security"]); evaluate DEF at :385 (private fn) and :267 (trait impl); evaluate_single at :467 -- confirmed via elai.map
rule_loader.rs:1-95 (load_seed_rules_from_dir) | GROUNDED | DEF_FN at rule_loader.rs:45 (plan cites :40 -- off by 5 lines; VERIFY_NEEDED inline); confirmed via elai.map row + direct read
project_rule_seeder.rs (seed_rules) | GROUNDED | DEF_FN at project_rule_seeder.rs:68 (plan body says :68 in citations block; Reads section says :91 -- off by 23; SCIP_DEF at :68 confirmed); plan's SCIP_DEF line is correct
hook_dispatcher.rs:1470-1600 (dispatch_pre_tool_use) | GROUNDED | DEF_FN IpcHookAsyncAdapter impl at hook_dispatcher.rs:1477 (confirmed); InProcessHookDispatcher impl at :862; HookDispatcher trait decl SCIP_DEF at :304 -- plan cite :304 is trait decl, matches
task_runner.rs:6420-6450 (build_hook_block_summary) | MISMATCH | DEF_FN at task_runner.rs:12043 (plan cites :11943, off by 100 lines); call site at :6496 (plan cites :6439, off by 57 lines); function exists and is real, line numbers are stale
bench_cmd.rs:1-160 (run_bench_prmbench) | GROUNDED | DEF_FN at bench_cmd.rs:31 (confirmed); run_bench_prmbench wrapper exists; patterns are real
prmbench_runner.rs (runner pattern) | GROUNDED | file exists at crates/orchestrator/src/benchmarks/prmbench_runner.rs; DEF_FN run_category at :588 confirmed via elai.map
commands_e30.rs + PrmBenchPanel.svelte + PinnedRulesPanel.svelte (frontend templates) | GROUNDED | DEF_FN canonical_kind at commands_e30.rs:117; DEF_FN reload at PrmBenchPanel.svelte:51; DEF_FN reload at PinnedRulesPanel.svelte:71 -- confirmed via elai.map
arXiv:2302.05319 SVEN (CCS 2023) | INGEST | not in RAG; claim: 59.1->92.3% secure on CodeGen-2.7B; plan correctly flags CodeGen caveat; abstract-VERIFIED claim in plan needs paper-ingestor confirmation
arXiv:2405.00218 CodeGuard+ | INGEST | not in RAG; claim: 43.2->76.0% secure-pass@1 on CodeGen-2.7B; plan correctly flags caveat; needs paper-ingestor
arXiv:2410.11096 SeCodePLT (NeurIPS 2025) | INGEST | not in RAG; claim: 5.9k samples / 44 CWEs / GPT-4o 55% secure; needs paper-ingestor before live run


CLAIM CLASSIFICATION

ALWAYS_INCLUDED_DOMAINS = &["engineering","security"] | GROUNDED | elai.map DEF_CLASS rule_engine.rs:187 snippet matches exactly
Predicate enum: AstPattern + GrepPattern + FileContext variants | GROUNDED | elai.map SCIP_DEF predicate.rs:81/101/102/109/118 confirmed
load_seed_rules_from_dir at rule_loader.rs:40 | MISMATCH | actual DEF_FN at rule_loader.rs:45 (off by 5); function is real and correct
seed_rules at project_rule_seeder.rs:91 (Reads section) | MISMATCH | actual DEF_FN at project_rule_seeder.rs:68; plan's own Citations block says :68 -- Reads section line number is stale
build_hook_block_summary at task_runner.rs:11943, call at :6439 | MISMATCH | DEF at :12043, call at :6496; both off; likely due to F-2a insertions shifting line numbers
SsbActionType at ssb.rs:4412 | MISMATCH | actual enum at ssb.rs:4440 (off by 28); likely shifted by F-2a SCHEMA_VERSION 196 additions; variant SecurityRuleBlocked confirmed at :8046
dispatch_pre_tool_use IpcHookAsyncAdapter at hook_dispatcher.rs:1477 | GROUNDED | confirmed
SCHEMA_VERSION baseline 191 (CLAUDE.md baseline) | MISMATCH | CLAUDE.md says 191 but is acknowledged stale in task header; actual ssb.rs:2742 shows SCHEMA_VERSION = 196; F-2a already bumped it; TOTAL_VARIANTS 471 in task header also stale (golden now shows 473 after F-1+F-2a)
SVEN 59.1->92.3% secure rate on CodeGen-2.7B | INGEST | plan correctly caveat-flags as CodeGen-only; abstract-verified per plan but not RAG-confirmed; paper-ingestor required before live run
CodeGuard+ 43.2->76.0% secure-pass@1 on CodeGen-2.7B | INGEST | same caveat; same requirement
SeCodePLT 5.9k/44 CWEs (NeurIPS 2025) | INGEST | corpus design reference; paper-ingestor required before live run
ConstrainedDecodingConfig at decoding.rs:122 | GROUNDED | confirmed via elai.map + direct grep
build_xgrammar_request at sidecar_xgrammar_hook.rs:240, call at :333 | GROUNDED | DEF_FN at :240 confirmed; CALL at :333 confirmed via elai.map
SecurityRuleBlocked SSB variant shipped (SCHEMA 196) | GROUNDED | ssb.rs:8046 SecurityRuleBlocked DEF; action_type_lock.golden row 0368 security_rule_blocked; SCHEMA 196 confirmed at ssb.rs:2742
InsecureRateMeasured NOT yet in ssb.rs | GROUNDED | grep returns empty; confirmed absent; F-2b deliverable still required
bench_seccodeplt.rs NOT yet in crates/cli/src/ | GROUNDED | ls confirms absent; F-2b deliverable still required
SecurityRulesPanel.svelte NOT yet in dashboard-v2/src/lib/drawers/ | GROUNDED | ls confirms absent; F-2b deliverable still required
commands_f2.rs NOT yet in dashboard-v2/src-tauri/src/ | GROUNDED | ls confirms absent; F-2b deliverable still required
security seed dir + 4 TOMLs already exist | GROUNDED | ls confirms crates/rules/seed/security/{sqli,xss,weak_crypto,missing_authz}.toml all present; mutable=0 comment in sqli.toml:2 confirmed
load_security_seed_rules_builtin has zero production callers | GROUNDED | grep across crates/ shows only rule_loader.rs DEF + lib.rs re-export; no task_runner/hook_dispatcher caller yet; W-3/W-5 gap for F-2b
E-12 post-write semgrep vs F-2 pre-write BLOCK: orthogonal | VERIFY | E-12 auditor-fed async; F-2 PreToolUse boundary; distinct trigger sites; no source read needed, design claim verified via hook_dispatcher.rs + task_runner.rs trigger patterns
authz = WARN not BLOCK rationale (CodeGate FP lesson) | UNGROUNDED | CodeGate death-by-FP lesson cited in synthesis; synthesis doc is internal analysis (not external evidence); INGEST a primary source for FP rate data if used as a published claim
blake3 only (candidate + seed-integrity digests) | GROUNDED | plan explicitly forbids blake2/sha2; CLAUDE.md HARD RULE Anti-Stale-Training mandates blake3; existing codebase confirms blake3 in-tree
No new external deps (blake3 + E-9 harness deps already in-tree) | GROUNDED | anti-slopsquatting protocol satisfied; E-9 bench infrastructure confirmed shipped per RAG e9-agent-eval-benchmarks entry


ACTIONS REQUIRED

MISMATCH: load_seed_rules_from_dir cited at rule_loader.rs:40 (Reads section) but actual DEF at :45 -- update the Reads line numbers in F-2 file or confirm implementer reads the correct range; no functional impact but confusing
MISMATCH: seed_rules cited at project_rule_seeder.rs:91 (Reads section) but DEF at :68; plan's own Citations block already records :68 correctly -- Reads section is stale; no functional impact
MISMATCH: build_hook_block_summary cited at task_runner.rs:11943/:6439 but actual :12043/:6496 -- implementer must re-grep before any task_runner.rs edit to find real line numbers
MISMATCH: SsbActionType enum at ssb.rs:4412 but actual :4440 -- implementer must re-grep; already shifted by F-2a; no functional blocker
MISMATCH: SCHEMA_VERSION baseline 191 / TOTAL_VARIANTS 471 in plan text and CLAUDE.md are stale; actual is 196/473 post F-2a; F-2b must set its own bump relative to 196/473 not 191/471
INGEST: spawn paper-ingestor mode=paper arXiv:2302.05319 (SVEN CCS 2023) before F-2b live run; required for effect-size caveat verification
INGEST: spawn paper-ingestor mode=paper arXiv:2405.00218 (CodeGuard+ constrained decoding) before F-2b live run
INGEST: spawn paper-ingestor mode=paper arXiv:2410.11096 (SeCodePLT NeurIPS 2025) before F-2b live run; this is the corpus design reference
UNGROUNDED: authz=WARN CodeGate death-by-FP claim -- synthesis section 7 is internal analysis; if this becomes a published justification, a primary source (CodeGate post-mortem, FP-rate measurement) is required; for F-2a internal decision it is acceptable as design judgment
WIRE GAP (W-3/W-5): load_security_seed_rules_builtin has zero production callers outside rule_loader.rs definition; F-2b must wire this into at least one real production entrypoint (task_runner init path or hook_dispatcher setup) to satisfy W-3; current state = function exported but not called = wire-dark
WIRE GAP (F-2b missing deliverables): InsecureRateMeasured SSB variant, bench_seccodeplt.rs, seccodeplt_runner.rs, SecurityRulesPanel.svelte, commands_f2.rs, and F-2-decisions.md are all absent -- these are the required F-2b outputs; confirmed unshipped

VALIDATION WARNINGS

line numbers in Reads (mandatory) section have shifted since F-2a landed (rule_loader.rs:40 now :45; task_runner.rs:11943 now :12043 and :6439 now :6496; ssb.rs:4412 now :4440); implementer MUST re-grep before editing
SCHEMA_VERSION and TOTAL_VARIANTS in the plan and in CLAUDE.md are acknowledged stale by the task header (195/471 in header, 191/464 in CLAUDE.md); verified actual is 196/473; plan should treat these as reference points not hard-coded values
F-2a partial closure: seed TOMLs + SecurityRuleBlocked SSB variant + golden row shipped, but load_security_seed_rules_builtin has no production caller yet -- this is the open W-3 gap for the F-2a -> F-2b boundary
three paper INGESTs are pre-live-run gates per plan's own stated protocol; paper-ingestor must run before any F-2b live Qwen invocation
