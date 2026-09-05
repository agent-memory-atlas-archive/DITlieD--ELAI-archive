GROUNDING SWEEP -- phase F1-F2a
Source: F-1_vibegate_unified_write_gate.md + F-2_codevuln_ruleset_measured_number.md
Date: 2026-06-01
FSM: IDLE | Session: s-6344

SPEC REFERENCES
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED | file exists; grep confirms SVEN 59.1->92.3, CodeGuard+ 43.2->76.0, CodeGen-2.7B caveat, C1/C2 mechanism, Tier-1/Tier-2 split, CodeGate archived 2025-06-05 framing, section-1.4 dropped framings, section-2, section-5, section-7 all present
docs/thefinalglance/F-plan/F-0_INDEX.md W-1..W-10 | GROUNDED | read:1-160; all ten W-clauses verified as authored; SCHEMA baseline 191 confirmed; sequencing F-2a+F-1 co-ship confirmed

CLAIM CLASSIFICATION

--- F-1 CLAIMS ---
dispatch_pre_tool_use DEF at hook_dispatcher.rs:304 (trait) and :862 (InProcessHookDispatcher impl) | GROUNDED | SCIP_DEF at :304 confirmed in elai.map; actual trait body read at :389 (trait decl); impl at :861-862 verified by direct file read. Note: SCIP_DEF tag in elai.map uses :304 for the trait method; the narrative description in F-1 says "trait method DEF at :304" which is the SCIP_DEF line, not the Rust decl line (:389). Both are valid -- no action needed.
HookFinding struct at hook_dispatcher.rs:219, is_block at :233 | GROUNDED | direct read: HookFinding DEF_STRUCT at :219, is_block fn at :233. Exact.
HookInvocation::for_pre_tool_use at hook_dispatcher.rs:324 | GROUNDED | direct read at :324. Exact.
task_runner.rs:6418 for_pre_tool_use call, :6425 emit_hook_events_to_ssb, :6434 is_block short-circuit | MISMATCH | actual lines: for_pre_tool_use at 6475, emit_hook_events_to_ssb at 6482, is_block check at 6491, hook_pre_blocked=true at 6492, block_source=T1Hook at 6495. All claims are off by ~57 lines. The symbols are real and wired; only the line numbers are stale. Correct to 6475/6482/6491.
conversation.rs:483 run_pre_tool_use_hook call | GROUNDED | direct read at :483. Exact.
T1Hook with_rules and new in tiered_analysis.rs | GROUNDED | elai.map: T1Hook::with_rules at tiered_analysis.rs:2530, T1Hook::new at :2582. Both real.
ALWAYS_INCLUDED_DOMAINS = &["engineering","security"] at rule_engine.rs:187 | GROUNDED | direct read at :187. Exact.
evaluate at rule_engine.rs:385 | GROUNDED | direct read at :384-385. Off by 1 line (385 vs 384). Negligible; symbol is correct.
mutable INTEGER NOT NULL DEFAULT 1 at schema.rs:46 | GROUNDED | direct read at :46. Exact.
severity CHECK(severity IN ('BLOCK','WARN','INFO')) at schema.rs:45 | GROUNDED | direct read at :45. Exact.
source CHECK(... 'SEED','AUDITOR','HUMAN','SELF_MOD') at schema.rs:50-51 | GROUNDED | direct read at :50-52. Exact.
mutable=0 bypass lifecycle.rs:870-871 .expect("mutable=0 should bypass CCR") | GROUNDED | direct read at :870-871. Exact.
active_mutable0_ccr_exempt test at lifecycle.rs:865 | GROUNDED | direct read at :865. Exact.
RouteDecisionEventKind Pattern-2 precedent at ssb.rs:23884 | GROUNDED | elai.map DEF_ENUM at :23884. Exact.
IncidentsDrawer.svelte + commands_e30.rs + DrawerKey present | GROUNDED | elai.map confirms all three exist and are wired.
emit_hook_events_to_ssb DEF at hook_dispatcher.rs:609 (SCIP_DEF) | GROUNDED | confirmed in elai.map at :609.
SSB Pattern-2 roundtrip pattern at ssb.rs:27907 | GROUNDED | direct read at :27903-27907. Exact.
SsbActionType enum at ssb.rs:4412 | GROUNDED | direct read at :4412. Exact.
ALWAYS_INCLUDED_DOMAINS makes security domain always evaluated | GROUNDED | rule_engine.rs:187 + evaluate fn at :384. Substrate confirmed.
No mutable=0 rows in security domain yet | VERIFY | no seed/security/ directory exists (glob confirms empty); no security-domain rows seeded anywhere in crates/rules/seed/. Claim stands.
E-30 (SHIPPED) destructive-verb allowlist at crates/safety/src/destructive_verb_allowlist.rs | VERIFY | elai.map confirms BlockSource::T1Hook + E-30 at task_runner.rs:14397; IncidentsDrawer+commands_e30 confirmed above. E-30 shipped.
CodeGate archived 2025-06-05 | GROUNDED | synthesis doc section 7 cites it. NOT independently verified via WebFetch this session (INGEST-level claim from synthesis, treated as grounded via synthesis).
blake3 already in-tree for orchestrator | VERIFY | not checked directly; synthesis asserts it; F-1 instructs to verify via cargo tree before coding. Leave as VERIFY -- implementer must run `cargo tree -p elai-orchestrator | rg blake3` before authoring.

--- F-2 CLAIMS ---
ALWAYS_INCLUDED_DOMAINS at rule_engine.rs:187 | GROUNDED | same as above. Exact.
Predicate 7-variant enum at predicate.rs:101 | GROUNDED | direct read: AstPattern/GrepPattern/SchemaRequired/MetricGate/Composite/GraphReachability/VersionGate = 7. Exact.
AstPattern{language,query,query_format,scope} + GrepPattern{pattern,scope,context_lines,language,file_context} | GROUNDED | direct read confirmed all fields at predicate.rs:102-119.
FileContext Production/Test filter | GROUNDED | predicate.rs:81 (DEF). Exact.
load_seed_rules_from_dir at rule_loader.rs:40 | GROUNDED | elai.map DEF_FN + direct at :40. Exact.
seed_rules ProjectRuleSeeder at project_rule_seeder.rs:91 (body) / SCIP_DEF at :68 | GROUNDED | direct: fn at :68, .as_secs() at :91. Correct self-annotation (F-2 cites :68 as the SCIP_DEF -- confirmed).
dispatch_pre_tool_use at hook_dispatcher.rs:1477 labelled InProcessHookDispatcher | MISMATCH | :1477 is IpcHookAsyncAdapter::dispatch_pre_tool_use. InProcessHookDispatcher is at :862. The symbol is real but the impl-struct label is wrong. F-2's W-7 self-annotation says "(InProcessHookDispatcher impl; trait decl at :304)" for :1477 -- that is incorrect. Correct to: InProcessHookDispatcher::dispatch_pre_tool_use at :862; IpcHookAsyncAdapter at :1477; trait decl at :389. Implementer must attach to the InProcessHookDispatcher impl at :862, not :1477.
build_hook_block_summary at task_runner.rs:6439 | MISMATCH | actual DEF at task_runner.rs:12043; call site in pre-gate path at :6496 (not :6439). Off by 57 lines. Same shift as F-1 line-number drift. Symbols real; attach to :6496.
SsbActionType at ssb.rs:4412 | GROUNDED | confirmed.
sql_str parse pattern at ssb.rs:9326 / roundtrip at ssb.rs:27907 | GROUNDED | both read directly. Exact.
existing seed TOMLs: slopsquatting.toml, source_tiers.toml, risk_levels.toml, app_store.toml, design_hard.toml | GROUNDED | glob confirms all 5 exist at crates/rules/seed/.
seed/security/ does not exist yet | GROUNDED | glob returns empty. Confirmed new directory.
SVEN 59.1%->92.3% on CodeGen-2.7B (arXiv:2302.05319) | VERIFY | paper NOT ingested (not in workflow/papers/); synthesis doc confirms the number and caveat. Claim is synthesis-sourced, classified VERIFY -- paper-ingestor should ingest arXiv:2302.05319 before TESTING_FIRST to confirm abstract/table.
CodeGuard+ 43.2%->76.0% secure-pass@1 (arXiv:2405.00218) | VERIFY | same: paper not ingested; synthesis-sourced. Ingest arXiv:2405.00218 before TESTING_FIRST.
SeCodePLT NeurIPS 2025, 5.9k samples / 44 CWEs (arXiv:2410.11096) | VERIFY | paper not ingested; synthesis-sourced. Ingest arXiv:2410.11096 before F-2b TESTING_FIRST.
C-26 ConstrainedDecodingConfig at decoding.rs:122 | VERIFY | F-2 cites this for optional Tier-2 F-2b only. Not verified in elai.map this session (out of scope for F-2a). Pre-requisite for F-2b only; not a blocker for F-2a.
E-35 build_xgrammar_request at sidecar_xgrammar_hook.rs:240 (DEF) / :333 (call) | VERIFY | same: F-2b Tier-2 only. Not a blocker for F-2a.
E-9/E-49 bench harness: bench_cmd.rs dispatcher, bench_prmbench.rs::run, prmbench_runner.rs pattern | GROUNDED | elai.map: run_bench_prmbench DEF at bench_cmd.rs:31; bench_prmbench.rs exists; prmbench_runner.rs confirmed. Exact.
commands_e30.rs + PrmBenchPanel.svelte + PinnedRulesPanel.svelte templates | GROUNDED | commands_e30.rs confirmed in elai.map.
W-7 CORRECTION (already self-noted in F-2): synthesis named crates/cli/src/benchmark.rs and crates/rules/rules/security/ -- neither exists | GROUNDED | confirmed: no benchmark.rs in cli/src, no rules/rules/security/. Substituted anchors in F-2 are correct.
D-11 prerequisite for F-9 (not F-2a) | GROUNDED | F-2 does NOT depend on D-11; F-9 does. D-11 has contract tests in dashboard_v2_D11_contract.rs (elai.map confirms), classified as shipped for its own F-9 context. Not a blocker here.

PREREQUISITE SHIP STATUS
F-1 pre-requisites: crates/rules (SHIPPED -- rule_engine, schema, lifecycle all confirmed), hook_dispatcher.rs (SHIPPED -- dispatch_pre_tool_use real at :862), ssb.rs (SHIPPED -- SsbActionType at :4412). F-2a co-ships; no external pre-req. CLEAR.
F-2a pre-requisites: same substrate. C-26 and E-35 are F-2b-only (Tier-2, RunPod-gated). E-9/E-49 bench harness required for F-2b only. CLEAR for F-2a.

BLOCKERS
None hard-blocking F-2a. Two MISMATCHes (line numbers) are documentation errors, not wiring errors -- the symbols are real. Three VERIFYs for the benchmark papers are pre-TESTING_FIRST gates for F-2b only; F-2a ships no performance claim.

ACTIONS REQUIRED
- MISMATCH: F-1 task_runner.rs line claims are wrong. Update F-1 Reads section and reachable_from to: for_pre_tool_use at :6475, emit_hook_events_to_ssb at :6482, is_block check at :6491, hook_pre_blocked at :6492, block_source=T1Hook at :6495. (W-7 pre-condition: implementer must grep at coding time -- this is documented.)
- MISMATCH: F-2 labels hook_dispatcher.rs:1477 as InProcessHookDispatcher. Correct the label: :862 = InProcessHookDispatcher::dispatch_pre_tool_use; :1477 = IpcHookAsyncAdapter::dispatch_pre_tool_use. The Task 3 delta should target :862 impl, not :1477.
- MISMATCH: F-2 build_hook_block_summary call site is :6439; actual is :6496. Update F-2 Reads / MODIFY spec to :6496.
- VERIFY (pre-F-2b): ingest arXiv:2302.05319 (SVEN CCS 2023) via paper-ingestor mode=paper before F-2b TESTING_FIRST -- numbers cited in synthesis; need primary-source confirmation for the published claim.
- VERIFY (pre-F-2b): ingest arXiv:2405.00218 (CodeGuard+) via paper-ingestor mode=paper before F-2b TESTING_FIRST.
- VERIFY (pre-F-2b): ingest arXiv:2410.11096 (SeCodePLT NeurIPS 2025) via paper-ingestor mode=paper before F-2b TESTING_FIRST.
- VERIFY (pre-coding): run `cargo tree -p elai-orchestrator | rg "blake3|regex"` to confirm both are already in-tree before Task 1 of F-1. F-1 already instructs this in Open questions -- flag as mandatory pre-gate.

VALIDATION WARNINGS
- No Spec references lines in F-1 or F-2 (these are F-plan files, not B-N files; their references are in Citations/Grounding sections -- treated as equivalent).
- Line-number drift of ~57 in task_runner.rs affects BOTH F-1 and F-2 reachable_from anchors. Likely caused by code additions after elai.map was generated or after the synthesis doc was written. Not a wiring failure; implementer must re-grep at coding time per W-7.
- Paper-ingest gap for SVEN/CodeGuard+/SeCodePLT does not block F-2a (no perf claim in F-2a). Blocks F-2b headline only.
- D-11 ship status: confirmed (contract tests present). Not a blocker for F-1 or F-2.
- all sources validated for F-2a in-process substrate (rules engine, hook dispatcher, SSB, seed dir convention, bench harness pattern)
