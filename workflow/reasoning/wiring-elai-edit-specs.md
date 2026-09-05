WIRING-FIX-ELAI EDIT SPECS — CONSOLIDATED EXECUTOR BLUEPRINT
Path: workflow/reasoning/wiring-elai-edit-specs.md
Generated: 2026-06-02 (WAVE-0 consolidation of 7 cluster verdicts: D-WC, D-EV, D-L3, D-L4, D-XL, D-EXH, D-FZ)

PURPOSE
This is the deterministic execution blueprint. Every edit is grouped by compile wave. wiring_contract.rs is one file (D-WC, SP1+SP2+SP4 merged). evidence.rs/task_runner.rs/autonomous_gate.rs are D-EV. wiring_coverage.rs + wiring_coverage_cmd.rs are D-L3. wiring_layer4_trace_contract.rs + tracing-test rule/checker are D-L4. cross_lang/CI/data are D-XL+D-FZ. The dispatch-enum lock test is D-EXH. Apply in wave order; cargo check at each wave boundary.

VERIFICATION STATUS (load-bearing facts re-confirmed this session)
- crate::symbol_graph::SymbolGraphCache EXISTS in the orchestrator crate (mod.rs:52 re-export; cache.rs:153 def). load() at cache.rs:167, iter_nodes()->impl Iterator<Item=(&String,&SymbolNode)> at cache.rs:350, SymbolNode{name,file,line,kind,id,calls} at cache.rs:61-76. The D-WC SP2 reachability integration is FEASIBLE. The grounding-doc elai_knowledge::symbol_graph path was WRONG; D-WC's crate::symbol_graph correction is correct. No new dep, no Cargo.toml change.
- .elai_cc/.git is a DEAD stub (hooks/ only, no HEAD/objects/refs). Outer core.hooksPath=.githooks. D-FZ's correction (enforce via .githooks, not .elai_cc/.git/hooks) is correct.
- TOTAL_VARIANTS live = 482 at crates/orchestrator/tests/action_type_lock.rs:534 (a TEST-file const, read as file-text by the e45 contract). D-XL 394->482 patch anchor is exact AND correct, conditional on no new action_types shipping this commit.
- WiringContractError enum anchor exact at wiring_contract.rs:172-181. WiringAuditError enum anchor exact at :1038-1046.
- evidence.rs DiagnosticProbeGate anchor exact at :695-698; AntiFabricationGate anchor exact at :763-768. EvidenceError{gate_name:&'static str, missing_artifacts:Vec<String>, suggestion:String} at :21. EvidenceGate trait name()/transition()/check() at :211-216.
- autonomous_gate.rs anchors exact: security_monotonicity field :298, default :326, all_passed :351, failed_checks :394.
- task_runner.rs predicate anchor exact at :13498-13499; derive_security_monotonicity tail at :13540.
- D-L4 SsbViewer::recent_phase_transitions(usize)->Result<Vec<(i64,String,String,i64)>,_> at ssb_viewer.rs:959 (query filters action_type IN phase_transition/fsm_transition). Fsm::new/with_ssb_writer/transition at fsm.rs:655/768/878 (transition takes Vec<EvidenceArtifact>). action_type strings phase_transition/fsm_transition/wiring_evidence/wiring_violation all live at ssb.rs:8376/8427/8598/8599.
- WiringAuditCmd literal blast radius: ONE production literal at cli/wiring_audit.rs:79; ZERO literals inside wiring_contract.rs. D-WC's 'tests/contracts construct others' risk is OVERSTATED.
- D-EV all_passed() blast radius: SIX contract files build all-true Evidence (the 3 D-EV named PLUS c10_anti_spiral_contract.rs, c9_autonomous_infra_contract.rs, phase_4g_extended_contract.rs).
- ANCHOR TO RE-READ AT EXECUTION (verified_anchor=false): D-L3 parse_wiring_coverage_subcommand_args. Real sibling is parse_archive_subcommand_args(rest:&[String]) at args.rs:2749; dispatch passes &rest[1..] (=&[String]). D-L3's &[&str] element type is WRONG; correct to (rest:&[String], output_format:CliOutputFormat), index args[i].as_str().

CROSS-CONTRACT CONFLICTS (MUST resolve before EXECUTING)
C1 D-EV<->D-L3: D-EV expects Layer3CoverageSummary{all_new_fns_covered,uncovered_new_fns} + derive_layer3_entrypoint_covered(&dyn SsbReader)->bool. D-L3 ships layer3_coverage_clean(&WiringCoverageCmd)->bool + WiringCoverageReport, NO SSB-reader helper, NO summary type. RECONCILE ownership/signature.
C2 D-EV<->D-L4: D-EV expects Layer4TraceSummary{observed_row_count} + derive_layer4_trace_present(&dyn SsbReader)->bool reading a new WireDarkTrace action_type. D-L4 ships neither; it asserts on existing phase_transition/wiring_evidence via SsbViewer. RECONCILE.
C3 SSB producer missing: D-EV requires new SsbActionType EntrypointCoverage+WireDarkTrace + SCHEMA/TV bump + golden append. No cluster owns this; D-L3 declines an SSB action_type. DECIDE owner+names.
C4 action_type name divergence: D-EV (EntrypointCoverage/WireDarkTrace) vs D-FZ (wiring_audit_completed/wiring_coverage_measured/wire_dark_rate_sampled) vs D-L4 (wiring_evidence). Pick ONE canonical set.
C5 TOTAL_VARIANTS: D-XL pins e45 to 482 (current). Any new action_type bumps it past 482 and re-breaks e45. If variants ship, re-target the patch + regen golden in the same commit.
C6 gate-manifest.txt: D-L4 (2 entries mid-list) + D-FZ (2 entries end) both edit it. Merge into ONE edit; re-lock ONCE.
C7 gate-manifest.lock: D-FZ edit replaces a real sha3 hash line with prose, corrupting the TSV. DROP it; re-lock via gate_freeze_check.py --lock post-commit.
C8 elai_cc.toml [wiring.entrypoints]: D-XL (6 symbols) vs D-FZ (2 symbols), same anchor. Merge to ONE edit (recommend D-XL 6 + D-FZ [wiring.layer3] block).
C9 entrypoint activation order: activating [wiring.entrypoints] before D-WC SP2 lands makes the reachability leg run-but-blind (self-certifying graph). Sequence SP2 first.
C10 D-L3 parse fn type: &[&str] -> &[String] (see verified_anchor=false above).

WAVE 0 — DECISION GATE (no code)
Resolve C1/C2/C3/C4/C5. Write one ADR block naming: the canonical Layer-3 surface D-EV consumes, the Layer-4 summary type+derive owner, the canonical SsbActionType evidence set, and whether new variants ship (and thus the correct TOTAL_VARIANTS target). Resolve C7 (drop lock edit), C6/C8 (merge plan).

WAVE 1 — FAIL-CLOSE-ONLY (self-contained, parallel-safe)
File wiring_contract.rs (D-WC SP1+SP4):
- :172-181 add WiringContractError::EmptyEntrypointSet(PathBuf) + AbsentEntrypointSet(PathBuf). Fail-close: typed errors so loader can FAIL. R1/R2.
- :251-267 flip WiringEntrypointSetLoader::load empty-branch and absent-branch from WARN+Ok to Err(EmptyEntrypointSet)/Err(AbsentEntrypointSet). Fail-close: empty/absent entrypoint set is missing evidence = FAIL. R1/R2.
- :1038-1046 add WiringAuditError::ZeroContracts(PathBuf) + ViolationsPresent(u32). R1 failOnEmptyShould + SP4 hard-fail surface.
- :879 insert zero-contracts guard after collection loop: all_contracts.is_empty() in a present dir -> Err(ZeroContracts). R1.
- :935-948 ObservableEffect leg: replace `let _ = runner.run_effect(...)` with failure-count + return Err(ViolationsPresent) on any BrokenObservable. Uses u32::try_from(...).unwrap_or(u32::MAX) (no panic). SP4(d) R4.
- :771-773 ssb_has_row db-missing comment re-doc (no behavior change).
- :718-732 run_effect expect_ssb loop: track db_ever_existed; at deadline distinguish 'db never appeared (entrypoint never ran)' from 'row absent'. SP4(e) R2.
File evidence.rs (D-EV self-contained):
- :763-768 AntiFabricationGate::check: None rows -> Err; empty rows -> Err (was Ok). R1/R2.
- :730-749 AntiFabricationGate doc-comment reconcile (remove stale W-2 default-degrade narrative).
- :695-706 DiagnosticProbeGate::check: None outcome -> Err; non-authoritative (TimedOut/ToolMissing) -> Err (was soft-pass Ok). R1/R2/R4.
- :838-850 check_transition: add const TRANSITIONS_REQUIRING_GATES (7 transitions); empty gate set on a member -> Err. R1 failOnEmptyShould.
File task_runner.rs (D-EV self-contained):
- :13497-13499 flip predicate: tests_fail_correctly = !test_rows.is_empty() && had_red; test_flipped = !test_rows.is_empty() && had_green (was vacuously true on empty). THE worst fail-open. R1/R2/R4.
File e45_cross_language_drift_contract.rs (D-XL, CONDITIONAL on no new action_types this commit):
- :293-297 assert TOTAL_VARIANTS==482 (was 394). Verified live 482.
File exhaustiveness_dispatch_lock.rs (D-EXH CREATE):
- New stdlib-only test sibling of action_type_lock.rs. 3 roundtrip + 4 grep tests asserting no wildcard arm on FsmSubstate(5)/SkillEventPhase(3)/RouteDecisionEventKind(3)/SsbActionType(482) dispatch matches. UNDER-SPECIFIED in source spec: executor must author real bodies; grep tests anchor on first-arm literals (all enums verified present). Note RouteDecisionEventKind has a test-local match at ssb.rs:29108.
Gate: cargo check -p elai-orchestrator -p elai-contracts. Loader/gate tests asserting Ok-on-empty WILL flip to Err — update them in this wave (expected, do NOT revert).

WAVE 2 — D-WC SP2 REAL REACHABILITY GRAPH (before entrypoint activation)
File wiring_contract.rs:
- :54-56 add `use crate::symbol_graph::SymbolGraphCache;` (CORRECTED path; same crate, no dep).
- :806-814 add WiringAuditCmd.graph_path: Option<PathBuf> + a second impl block with graph_path() accessor (resolves source_root/.claude/tools/elai_graph.json).
- :978 insert build_reachability_graph_from_source(graph_path,&entrypoints)->Result<ReachabilityGraph,WiringAuditError> + helpers entrypoint_match_key/symbol_trailing_name/is_test_source_path (reuse SKIP_DIR_FRAGMENTS/PREFIXES/SKIP_FILE_SUFFIXES). Name-keyed graph; test-file callers excluded (R7/W-8); missing graph -> Err (R2, no self-certifying fallback).
- :887-921 replace DkgReachability warn-and-skip/fabricated-graph block with debug_assert(!entrypoints.is_empty()) + build_reachability_graph_from_source(...)? + auditor run. Empty entrypoints now fail upstream at loader.
File cli/wiring_audit.rs:79 — add graph_path: None to the ONE WiringAuditCmd literal.
Gate: cargo check -p elai-orchestrator -p elai-cli. Residual unsoundness (name-collision in name-keyed graph) documented; Layer-3 runtime coverage is the load-bearing backstop.

WAVE 3 — D-L3 LAYER-3 RUNNER (new module, independent of WC)
- CREATE orchestrator/src/wiring_coverage.rs: SmokeManifestLoader, WiringCoverageCmd::run (build instrumented via cargo-llvm-cov subprocess, drive smoke scenarios, export --json, gate new-fn count==0). Fail-close: missing/zero coverage -> Err(MeasurementAbsent); static_only cannot emit PASS (R3); git-diff failure -> Err (R2). cfg(test)+WIRE-DARK suppression only. layer3_coverage_clean(&WiringCoverageCmd)->bool collapses Err to false.
- lib.rs: pub mod wiring_coverage (after :641) + pub use block (after :1465).
- CREATE cli/src/wiring_coverage_cmd.rs: shell over WiringCoverageCmd::run; every un-measurable error exits 1 (R2); not-clean exits 1.
- args.rs:362 add CliAction::WiringCoverage{manifest_path,diff_base,output_format}. args.rs:2284 add `"wiring-coverage" => parse_wiring_coverage_subcommand_args(&rest[1..], output_format)`. Add fn parse_wiring_coverage_subcommand_args(rest:&[String], output_format:CliOutputFormat)->Result<CliAction,String> (CORRECTED &[String]; index args[i].as_str(); use .ok_or for missing values, no unwrap). ANCHOR parse_archive_subcommand_args(rest:&[String]) at args.rs:2749 — re-read before applying.
- main.rs:71 add mod wiring_coverage_cmd. main.rs:775-782 add WiringCoverage dispatch arm (exhaustive match = compile-time wiring proof, R6).
- CREATE .claude/tools/knip_zero_module_wrapper.py (stdlib; knip --production --reporter json; zero-analyzed -> exit 2).
Gate: cargo check -p elai-orchestrator -p elai-cli.

WAVE 4 — SSB SCHEMA (conditional on WAVE 0 deciding new variants)
SSB owner adds the canonical evidence action_type variant(s) + action_type_lock.golden append + SCHEMA_VERSION/TOTAL_VARIANTS bump. Re-target e45 to the new count (supersedes WAVE 1c). Gate: action_type_lock test green at new count.

WAVE 5 — D-EV WAVE-2 CONSUMERS (after 3+4)
File evidence.rs:
- :196-202 add TransitionContext.layer3_coverage + layer4_trace (Option, serde default) using the RESOLVED summary types from WAVE 0.
- :790-815 add EntrypointAndTraceGate + register in all_gates (slot 11, order-stable). EXECUTING->AUDITING requires non-empty Layer-3 coverage AND >=1 Layer-4 row; absent/empty/zero -> Err.
File autonomous_gate.rs:
- :297-299 add Evidence.layer3_entrypoint_covered + layer4_trace_present (default FALSE).
- :323-328 Default sets both false + add default_false_* serde helpers.
- :351-367 all_passed() ANDs both new legs.
- :433-437 failed_checks() labels both.
File task_runner.rs:
- :13536-13541 populate both fields from the RESOLVED derive helpers.
CONTRACT UPDATES (same commit): all SIX all-true Evidence literals add layer3_entrypoint_covered:true + layer4_trace_present:true — autonomous_gate_contract.rs:52, autonomous_gate_with_framing_contract.rs:34, autonomous_gate_with_per_level_contract.rs:77, c10_anti_spiral_contract.rs, c9_autonomous_infra_contract.rs, phase_4g_extended_contract.rs.
Gate: cargo check -p elai-orchestrator -p elai-contracts.

WAVE 6 — D-L4 HARNESS + TRACING RULE (parallel with 5)
- CREATE contracts/tests/wiring_layer4_trace_contract.rs: drives REAL Fsm::transition, asserts off-agent phase_transition row via SsbViewer; expect_phase_transition_within fail-closes on db-not-found/zero-rows; layer4_missing_db_is_hard_fail self-test; #[ignore] D-WC dogfood placeholder (wiring_evidence). Uses verified APIs (Ssb::new/SsbConfig, Fsm::new/with_ssb_writer/transition, SsbViewer::open/recent_phase_transitions). multi_thread flavor (block_in_place).
- contracts/Cargo.toml: add tracing-test = "0.2.6" dev-dep (re-verify on crates.io at execution).
- CREATE .claude/rules/tracing-test-assert.md (T-1/T-2/T-3 mandatory-assert + Batch ban).
- CREATE .claude/tools/traced_test_assert_check.py (stdlib; brace-match #[traced_test] bodies; exit 1 BLOCK, exit 2 dead-evidence).
Gate: cargo check -p elai-contracts; pre-flight `python traced_test_assert_check.py --root .elai_cc/crates` to confirm no legacy #[traced_test] fails.

WAVE 7 — DATA / CI / FREEZE (no Rust compile)
- MERGED elai_cc.toml edit (resolve C8): uncomment [wiring.entrypoints] with D-XL's 6 verified symbols (cli main, fsm::transition, ipc_registry::dispatch, in_process::dispatch_tool, tauri main, tauri lib::run) + append D-FZ [wiring.layer3] block (enabled, fail_under_lines, new_fn_zero_hit_is_block, instrument_profile, llvm_cov_json_flag=--json). Lands AFTER WAVE 2 (real graph live).
- cross_lang_diff.py (5 coupled edits, apply together): --check-action-types -> --no-action-types (default RUN, R3); unconditional action-type leg; PAIRS->_PY_ONLY_PAIRS + _load_toml_pairs/_assemble_pairs (tomllib stdlib + _CANONICAL_FALLBACK); evaluate() iterates assembled pairs. SEARCH-FIRST: grep repo for any --check-action-types caller before the rename.
- cross-lang-pairs.toml header: document canonical-source authority.
- CREATE .github/workflows/wiring-audit.yml (ubuntu, no [ci skip], runs elai wiring audit + cross_lang_diff.py).
- windows.yml: remove the [ci skip] guard on the wiring step ONLY.
- wire-dark-allowlist.toml: populate 4 flags (E-38/E-10/E-43/E-31) with all six fields. NOTE smoke_test paths are SP3/SP5 forward-deps.
- smoke-manifest.toml: seed 3 gate-self features. NOTE expect_ssb names depend on C4 resolution.
- locked_keys_registry.toml: append [[locked_keys]] for smoke-manifest/wire-dark-allowlist/elai_cc.toml (+ self-freeze the registry). Schema reuses LockedKeyEntry (path/line_range/reason/owner_session/freeze_ts_ms/signature_blake3, empty sig ok).
- MERGED gate-manifest.txt edit (resolve C6): add .claude/rules/tracing-test-assert.md + .claude/tools/traced_test_assert_check.py + .elai_cc/elai_cc.toml + .elai_cc/scripts/locked_keys_registry.toml in ONE edit.
- DROP D-FZ gate-manifest.lock in-file edit (C7).
- CREATE .githooks/gate_and_locked_keys_check.sh (two-leg fail-closed: gate_freeze_check.py + locked-keys auditor).
- .githooks/pre-commit: wire in the new script before the success echo.

WAVE 8 — POST (human/controller only)
Run `python .claude/tools/gate_freeze_check.py --lock` ONCE to regenerate gate-manifest.lock over the full merged fileset. NEVER run by the executing plan-run (defeats R8). Verify gate_freeze_check.py --json reports OK.

NEW DEPENDENCIES (consolidated)
- tracing-test 0.2.6 (Rust dev-dep, D-L4) — re-verify crates.io at execution.
- cargo-llvm-cov 0.8.7 (subprocess tool, NOT a Cargo dep; --json flag) — CI must provision.
- knip 6.15.0 (subprocess CI tool, NOT a runtime dep) — JS CI must provision.
- tomllib (Python 3.11+ stdlib, no install).
- GitHub Actions checkout@v4/rust-toolchain@stable/setup-python@v5/cache@v4 (official, already used).
- ZERO net-new Cargo/pip/npm RUNTIME deps.

RESIDUAL RISKS
- Name-keyed reachability graph collapses same-named symbols (R5 unsound); Layer-3 runtime coverage is the load-bearing backstop.
- graph_path() default resolution depends on whether elai wiring audit runs at cwd F:/.ELAI_workflow vs .elai_cc; a wrong path -> Err (fail-close, correct) but blocks audits until configured.
- DiagnosticProbeGate now hard-FAILs on TimedOut/ToolMissing; confirm the E-52 LOCAL-ONLY soft-pass is intentionally tightened for the autonomous path.
- test_flipped predicate flip will surface more human-escalation events until callers reliably emit TestOutcome rows; verify emit_test_outcome fires on the normal path.
- traced_test_assert_check.py is regex-unsound (Layer-2, necessary-not-sufficient); the runtime assert! is the sound backstop.
- D-FZ self-lock needs an L3_OVERRIDE / ELAI_GATE_FREEZE_SKIP=1 documented escape for legitimate future registry edits.