GROUNDING SWEEP -- phase J-F7
Source: docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md
Date: 2026-06-01
FSM: EXECUTING | Session: s-6344

SPEC REFERENCES

No "Spec references:" lines cite a docs/specs/guide/steps/phase-X.txt file in this plan (F-plan tier; spec grounding is via the VIBESEC source doc and elai.map instead).
workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED | file confirmed present; section 4 item C7 + section 1.2 Springdrift correction + section 5 TIER 3 sequencing all present at lines 57,83,160,389-409,460-481,496
workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md (Pillar 5) | VERIFY_NEEDED | file not verified directly this sweep; referenced for git-whole-state framing only, not load-bearing for F-7 mechanism
arXiv:2501.13956 (Graphiti bi-temporal) | VERIFY_NEEDED | cited as reference-only (NOT load-bearing for F-7); not ingested this sweep; grounding not required since plan explicitly marks it non-load-bearing

CLAIM CLASSIFICATION

formal_verify::orchestrator::run exists at orchestrator.rs:111 | GROUNDED | source read: line 111 = "pub async fn run(request: VerifyRequest) -> PipelineResult" exact match
Z3 route returns Inapplicable{reason:"z3_stub"} at orchestrator.rs:117-119 | GROUNDED | source read lines 117-119 exact: "VerifierKind::Z3Smt => VerifierOutcome::Inapplicable { reason: "z3_stub".to_string(), }"
VerifierKind/VerifierOutcome/VerifyRequest in formal_verify/types.rs:105 | GROUNDED | source read lines 100-127: VerifierOutcome enum at line 105 with Verified/Refuted/Inapplicable/RunpodPending; VerifierKind at line 35-50
expected_local_install() returns false at types.rs:72-74 | GROUNDED | source read: all four variants return false
TemplateCrystallizer struct at compiled_pipeline.rs:292 | GROUNDED | elai.map DEF_STRUCT line 104050 + source read: struct at line 292 confirmed
first_try_success_rate at compiled_pipeline.rs:365-370 | GROUNDED | source read lines 365-370 exact match: raw frequency successes/bucket.len() with no smoothing, 0.0 on empty bucket
CompiledExecutor::try_execute at compiled_pipeline.rs:486 | GROUNDED | source read: function def at line 486, selects pipeline by cache key and confidence_threshold check against first_try_success_rate
CompiledPipeline::compute_eligibility at compiled_pipeline.rs:225 | GROUNDED | source read lines 220-232: function at line 220, param (steps, use_count) exact match
classify_task_for_template at pipeline_builder.rs:391 | GROUNDED | elai.map DEF_FN line 68659 + METHOD_CALLs confirmed at pipeline_builder.rs:391
run_crystallizer_post_execute_hook production wire at task_runner.rs:3470 | MISMATCH | plan claims :3470; actual DEF is at task_runner.rs:3529 (grep: "async fn run_crystallizer_post_execute_hook" = line 3529; line 3470 is a comment about D-15 novelty_runtime, unrelated); CALL site is at 7823
RuleRecord.mutable:bool at lifecycle.rs:126 | GROUNDED | source read line 126: "pub mutable: bool," exact match; SCIP_DEF confirmed
fetch_rule_record at lifecycle.rs:277 | GROUNDED | source read: "pub fn fetch_rule_record(conn: &Connection, rule_id: &str)" at line 277; mutable: row.4 != 0 at line 308 confirmed
insert_rule is a private test helper NOT a public symbol | GROUNDED | grep shows fn insert_rule at lifecycle.rs:589 inside #[cfg(test)] block; plan's WIRING_CONTRACT correctly uses fetch_rule_record; W-7 self-corrects this
FormalVerifyAction enum at args.rs:1331 | GROUNDED | source read lines 1330-1342: enum present, currently only Run{} and Status variants; no Lattice variant yet (new)
formal_verify_cmd::run at formal_verify_cmd.rs:31, main.rs:599 | GROUNDED | source read formal_verify_cmd.rs:31 = "pub(crate) fn run(action: FormalVerifyAction, ...)"; elai.map CALL at main.rs:599 confirmed
action_type_lock.rs ALL_VARIANTS + TOTAL_VARIANTS | GROUNDED | source read: TOTAL_VARIANTS const = 473 (line 490); current SCHEMA_VERSION = 196 (F-1/F-2a bump, 2026-06-01)
SCHEMA baseline 191/464 per CLAUDE.md | MISMATCH | CLAUDE.md states "SCHEMA_VERSION 191, TOTAL_VARIANTS 464" but the live tree is at SCHEMA_VERSION 196, TOTAL_VARIANTS 473 (F-1/F-2a bumped it, 2026-06-01); F-7 must use 196/473 as its baseline, not 191/464; plan text acknowledges this ("source tree carries higher in-flight comments; never hard-code a final number") so the plan protocol is correct but the CLAUDE.md stale reference needs awareness
TemplateCrystallizedPayload at ssb.rs:15291 | MISMATCH | plan cites ssb.rs:15291; actual DEF_STRUCT is at ssb.rs:15358 (elai.map line 28343 + source read line 15358); the struct itself matches the expected shape; line number is off by 67 lines
Tauri CmdResult + write_payload pattern in commands.rs | GROUNDED | source read commands.rs:465-481: #[tauri::command] + State<'_, AppState> + CmdResult<Value> + writer.write_payload("dashboard-v2",...) pattern confirmed; no get_template_ranking or get_lattice_status yet (new)
No VerificationPanel / TemplateRankingPanel exists today | GROUNDED | elai.map MonitorWindow search returns no TemplateRankingPanel or VerificationPanel; commands.rs search confirms get_template_ranking / get_lattice_status absent (new symbols to create)
Springdrift +10.7% P@4 numbers are SYNTHETIC/UNREPRODUCED | GROUNDED | VIBESEC section 1.2 lines 83-88 + section 4 C7 lines 396-408 both state explicitly: "SYNTHETIC / UNREPRODUCED with overlapping confidence intervals"; plan correctly drops this target
E-53 stubs NOT resolved | GROUNDED | source read orchestrator.rs:117-119: Z3 route still returns Inapplicable{reason:"z3_stub"}; types.rs:72-74: expected_local_install() returns false for all verifiers; HALF A lattice live proof remains RUNPOD-GATED
blake3 for obligation hash (not blake2/sha2) | GROUNDED | CLAUDE.md Anti-Stale-Training HARD RULE; plan cites this correctly; lattice.rs does not yet exist, implementer must enforce
no .unwrap()/.expect() in template_utility.rs / lattice.rs production | GROUNDED | CLAUDE.md HARD RULE; both files to-be-created; auditor enforces; plan correctly calls this out
Laplace formula (successes+alpha)/(bucket_len+alpha+beta) | VERIFY | standard Laplace smoothing formula for Bernoulli; no library needed (pure arithmetic); plan alpha=beta=1 uniform prior is the standard choice; no external paper needed; GROUNDED by mathematical definition
W-2 default-ON kill-switch constraint | GROUNDED | F-0_INDEX doctrine W-2 in F-plan; plan stub_ban explicitly lists the default-off terminal state as forbidden
CBR measurable = measured delta after >=100 labelled sessions | GROUNDED | VIBESEC section 4 C7 lines 407-408 explicit; plan acceptance criteria explicit; no numeric target imported

ACTIONS REQUIRED

MISMATCH: run_crystallizer_post_execute_hook is at task_runner.rs:3529 (DEF) and 7823 (CALL site), NOT :3470 as cited in plan (line 3470 is an unrelated D-15 comment). Update all three reachable_from references in WIRING_CONTRACT and any executor reads to use :3529 for the DEF and :7823 for the call. This does not invalidate the wiring (the hook is real and wired); it is a stale line-number only.

MISMATCH: TemplateCrystallizedPayload is at ssb.rs:15358 NOT :15291 as cited in the Reads section. Update the mandatory-reads entry. Impact is cosmetic (the struct shape is correct); the implementer must read the correct range.

VERIFY (pre-TESTING_FIRST): confirm SCHEMA_VERSION and TOTAL_VARIANTS at implementation start via grep in action_type_lock.rs header before naming the "baseline + 1 / +2" bump. At authoring time: SCHEMA_VERSION 196, TOTAL_VARIANTS 473. F-7 HALF B will be 197/474; F-7 HALF A (co-ship) will be 198/475. The plan's "baseline 191/464" text is from a stale CLAUDE.md reference; plan protocol ("never hard-code a final number") is correct.

VERIFY: workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md Pillar 5 cited for git-whole-state framing. Verify this file exists before executor reads it. Not load-bearing for the mechanism; low risk if absent (Pillar 5 is supplementary framing only).

all other claims grounded or verified -- clear to TESTING_FIRST with the two MISMATCH line-number corrections above applied

VALIDATION WARNINGS

No docs/specs/guide/steps/phase-X.txt Spec references in this F-plan file -- expected for F-plan tier (VIBESEC source doc + elai.map serve as primary grounding); not an error.
CLAUDE.md SCHEMA_VERSION 191/TOTAL_VARIANTS 464 is stale vs live tree (196/473). This is a known mid-session drift acknowledged in the orchestrate prompt ("SCHEMA_VERSION is now ~196-197 mid-session, TOTAL_VARIANTS ~473+, CLAUDE.md 191/464 stale"). Plan protocol handles this correctly; no action on the plan file itself.
FSM state is EXECUTING (phase F-Unit-A), not IDLE/PLANNING -- this grounding sweep is mid-orchestrate-run; session s-6344 matches input.
