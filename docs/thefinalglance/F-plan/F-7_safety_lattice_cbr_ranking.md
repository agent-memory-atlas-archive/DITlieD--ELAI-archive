# F-7: Safety-rule lattice completeness proof + outcome-utility CBR template ranking

Objective: ship the two separable halves of synthesis cluster C7 under the F-plan wiring doctrine. HALF A (lattice): amend E-53's formal-verify pipeline so the `mutable=0` (ImmutableRule) rule subset is PROVEN complete + monotone + conflict-free as a SET (not merely unit-tested per row), routed through the existing `VerifierKind::Z3Smt` path once the RUNPOD-PENDING stub resolves. HALF B (CBR): replace the current observational `first_try_success_rate` template ranking inside `CompiledExecutor::try_execute` with a Laplace-smoothed outcome-utility score computed over EXISTING SSB outcome events, surfaced live through a verification/templates dashboard panel. Both halves are wired to a real production entrypoint (CLI + FSM evidence gate + Tauri), and neither can be marked DONE while wire-dark (W-1..W-10).

Estimated effort: 2 sessions (~10-13 h). HALF A and HALF B are independently shippable; HALF B ships first (no RunPod dependency); HALF A's live proof is RUNPOD-gated and ships a deterministic fixture harness in the interim (W-5/W-10).

Pre-requisites:
- HALF A is an AMENDMENT to E-53 (formal verification). It is BLOCKED until E-53's `VerifierKind::Z3Smt` route stops returning `VerifierOutcome::Inapplicable { reason: "z3_stub" }` (orchestrator.rs:117-119). Until then HALF A ships ONLY: (1) the lattice-enumeration + encoder that PRODUCES the SMT-LIB2 obligation, (2) a deterministic fixture harness that exercises the real `formal_verify::orchestrator::run` path with a stubbed Z3 outcome, (3) the `safety_lattice_verified` SSB variant + CLI surface. The live Z3 proof is a SEPARATELY-GATED acceptance checkbox (W-5).
- HALF B has NO external prerequisite. `CompiledExecutor` + `TemplateCrystallizer` + `classify_task_for_template` are all SHIPPED-WIRED today (confirmed in elai.map; production call site `run_crystallizer_post_execute_hook` at task_runner.rs:3470, activated via runtime_setup.rs:316).

Parallel-safe with: F-5 (staleness cascade — disjoint crate surface; F-5 touches crates/memory + dkg, F-7 HALF B touches crates/orchestrator/src/compiled_pipeline.rs + pipeline_builder.rs). F-8 (SCF self-mod — both touch crates/orchestrator but disjoint modules; F-8 = self_mod_safety.rs, F-7 = compiled_pipeline.rs + formal_verify/). E-53 itself (HALF A is its amendment; coordinate the formal_verify/ + ssb.rs edit window so the SCHEMA bump is atomic).

Source report: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C7 (full grounded mechanism + REVISED SCOPE), section 1.2 (the Springdrift corrections already forced — do NOT re-introduce), section 5 TIER 3 sequencing. Format mirror: docs/thefinalglance/E-plan/E-30_public_agent_incidents.md. Doctrine: docs/thefinalglance/F-plan/F-0_INDEX.md (W-1..W-10).

## Context

Per C7: Springdrift demonstrates two separable mechanisms worth borrowing — an exhaustively-verified safety calculus and outcome-learning CBR ranking. ELAI already has the substrate for both: a rules-as-data engine whose `RuleRecord.mutable: bool` column (crates/rules/src/lifecycle.rs:126) marks the immune (ImmutableRule) subset, and a pipeline-template crystallizer (`TemplateCrystallizer`, compiled_pipeline.rs:292) that today ranks templates by a raw observational `first_try_success_rate`.

The gap is precise and was confirmed by the adversarial pass (verification run wf_b959963a-db6):
- (HALF A) E-53 proves FSM/IPC invariants per-target, but NOTHING proves the `mutable=0` rule SUBSET is complete (every security/finance class has a covering rule), monotone (adding a rule never weakens the lattice), and conflict-free (no two immune rules contradict). The contract tests at lifecycle.rs (`active_mutable0_ccr_exempt`, `retired_immutable_auto_blocked`) test individual rows, never the set. E-53's `VerifierKind::Z3Smt` route is the natural home for a set-level SMT obligation, but it is currently an `Inapplicable { reason: "z3_stub" }` stub (orchestrator.rs:117-119), RUNPOD-PENDING.
- (HALF B) The crystallizer ranks by `first_try_success_rate = successes / bucket.len()` (compiled_pipeline.rs:365-370) — a raw frequency with NO smoothing (a 1/1 bucket outranks a 95/100 bucket) and NO use of the richer outcome signals already journaled in the SSB. No plan does CBR (case-based-reasoning) outcome-utility ranking.

CRITICAL GROUNDING CORRECTION (synthesis 1.2, MANDATORY — do NOT re-introduce): the Springdrift "7,056 proposition pairs / 100% coverage" and "+10.7% P@4" numbers are SYNTHETIC / n=1 self-reported / UNREPRODUCED with confidence intervals overlapping the RAG baseline; the safety-calculus axiom constants were never source-verified. Both the lattice-completeness claim and the CBR uplift are DESIGN HYPOTHESES, never targets. The measurable claim for HALF B is a MEASURED delta after >=100 labelled sessions vs the current `first_try_success_rate` baseline — not an imported number. The DROPPED Springdrift +10.7% target stays dropped. Grounding for this F-N is PARTIALLY_GROUNDED with the synthetic-benchmark caveat explicit.

Why this is a TIER 3 / SPLIT item (W-10): HALF A needs a SUBPROCESS (the Z3 binary) at proof time and is therefore split out of the fail-closed hot path — it runs as an offline CLI verification + an FSM evidence gate, never inside a per-write hook. HALF B is pure in-memory arithmetic over already-journaled rows and ships now.

## Existing state (B/C/D/E coverage)

- E-53 (formal verification): SHIPPED with `VerifierKind` {Z3Smt, Dafny, Kani, Verus} + `VerifierOutcome` {Verified, Refuted, Inapplicable{reason}, RunpodPending} (crates/orchestrator/src/formal_verify/types.rs:105) + `counts_toward_consensus()` + `formal_verify::orchestrator::run(VerifyRequest) -> PipelineResult` (orchestrator.rs:111) + CLI `elai formal-verify <run|status>` (crates/cli/src/formal_verify_cmd.rs:31, wired at main.rs:599). The Z3/Kani/Verus routes are Inapplicable stubs (RUNPOD-PENDING). F-7 HALF A is the named AMENDMENT: it ADDS the lattice-completeness obligation to the Z3 route and a new `lattice` verb to the CLI; it does NOT duplicate the pipeline.
- C-25 (TemplateCrystallizer): SHIPPED. `TemplateCrystallizer` (compiled_pipeline.rs:292) with `record_execution` (:318), `check_crystallization` (:335), `CompiledExecutor::try_execute` (:486) cache-hit path, `CompiledPipeline { first_try_success_rate, use_count, compilation_eligible }` (:375), `compute_eligibility(&[CompiledStep], use_count)` (:225), `classify_task_for_template` (pipeline_builder.rs:391), `TemplateCrystallizedPayload` SSB (ssb.rs:15291), production hook `run_crystallizer_post_execute_hook` (task_runner.rs:3470). F-7 HALF B EXTENDS `CompiledExecutor::try_execute`'s selection arithmetic; it does NOT rewrite the crystallizer.
- crates/rules (lifecycle): SHIPPED. `RuleRecord.mutable: bool` (lifecycle.rs:126), `insert_rule` (:589), `LifecycleError::ImmutableRuleCannotAutoRetire` (:552), tests `active_mutable0_ccr_exempt` (:865), `retired_immutable_auto_blocked` (:912). F-7 HALF A READS this set to build the lattice; it does NOT mutate lifecycle semantics.
- SSB outcome events: SHIPPED. `ExecutionOutcome { success, concept_class, plan_skeleton_hash, token_count, timestamp }` (compiled_pipeline.rs:280); `first_try_success_rate` is computed inline (:365). Coverage/block signals exist as SSB action_types journaled per session (block events flow through the existing SsbActionType set; `as_sql_str` at ssb.rs:2505). F-7 HALF B reads these to compute Laplace utility; the action_type_lock golden at crates/orchestrator/tests/action_type_lock.rs is the W-9 gate.
- Dashboard: NO formal-verify or template-ranking panel exists today (grep of dashboard-v2 returns only MonitorWindow / drawers; no VerificationPanel). F-7 MUST add the frontend_pair (W-4). Tauri command pattern: `#[tauri::command]` + `State<'_, AppState>` + `CmdResult` + `.write_payload("dashboard-v2", ...)` in crates/dashboard-v2/src-tauri/src/commands.rs.
- SCHEMA: baseline SCHEMA_VERSION 191 per CLAUDE.md (the source tree carries higher in-flight comments; never hard-code a final number — this F-N names "baseline + 1" for HALF A's E-53 amendment and "baseline + 2" for HALF B, atomic where co-shipped).

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` section 4 item C7 + section 1.2 + section 5 TIER 3 (the scope + the Springdrift correction)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\formal_verify\orchestrator.rs:1-200` (the `run` pipeline + the z3_stub dispatch boundary HALF A swaps)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\formal_verify\types.rs:40-250` (`VerifierKind`, `VerifierOutcome`, `VerifyRequest`, `counts_toward_consensus`, `expected_local_install`)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\formal_verify_cmd.rs:1-130` (the CLI shape HALF A adds a `lattice` verb to)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\lifecycle.rs:120-130, 480-560, 580-600` (`RuleRecord.mutable`, `insert_rule`, the immune-row read path)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\compiled_pipeline.rs:225-240, 280-400, 452-540` (`compute_eligibility`, `TemplateCrystallizer`, `CompiledExecutor::try_execute`, `first_try_success_rate`)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\pipeline_builder.rs:391` (`classify_task_for_template`)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:3470-3540` (`run_crystallizer_post_execute_hook` — the production wire)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:104-300, 2505, 15291` (SsbActionType enum head, `as_sql_str`, `TemplateCrystallizedPayload`)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs:2100-2170` (golden-lock W-9 gate)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:1-60, 460-580` (Tauri `#[tauri::command]` + `CmdResult` + write_payload pattern)
- arXiv:2501.13956 Graphiti (cited via synthesis for the bi-temporal framing — NOT load-bearing for F-7; reference only)
- the springdrift dossier referenced in synthesis Pillar 5 (for the git-whole-state framing only; its numbers are SYNTHETIC per 1.2)

## Tasks

### HALF B — CBR outcome-utility template ranking (ships first, no RunPod) — TIER 3 standalone

1. **Add `crates/orchestrator/src/template_utility.rs` (~180 LOC).** New module computing a Laplace-smoothed outcome-utility score for a crystallized template bucket. Public symbol `pub fn laplace_utility(outcomes: &[ExecutionOutcome], block_count: u32, alpha: f32, beta: f32) -> f32` returning `(successes + alpha) / (bucket_len + alpha + beta)` down-weighted by `block_ratio = block_count / bucket_len.max(1)`. Named constants `LAPLACE_ALPHA: f32 = 1.0`, `LAPLACE_BETA: f32 = 1.0` (no magic numbers per CLAUDE.md). Also `pub struct TemplateUtility { pub concept_class: String, pub plan_skeleton_hash: u64, pub utility: f32, pub first_try_ratio: f32, pub block_ratio: f32, pub coverage: f32, pub sample_n: u32 }` — `first_try_ratio` is the EXISTING signal carried forward (so the dashboard can show the before/after), `block_ratio` + `coverage` are the new CBR inputs read from the SSB outcome rows. CSQ guard (CLAUDE.md): `// counterfactual: if alpha/beta are 0 the score collapses to raw frequency and a 1/1 bucket outranks 95/100 — guard alpha+beta > 0 at construction`. No `.unwrap()` in the production path; clamp utility to `[0.0, 1.0]`.

2. **Wire `laplace_utility` into `CompiledExecutor::try_execute` (~70 LOC delta in compiled_pipeline.rs:486-540).** Where `try_execute` today selects/returns a cached `CompiledPipeline`, compute `TemplateUtility::utility` over the observation bucket and use it as the tie-break / selection score INSTEAD of raw `first_try_success_rate` when multiple candidate templates match a `(concept_class, plan_skeleton_hash)` query. Keep `first_try_success_rate` as a recorded field (do not delete — W-2/no-removal-without-confirmation; it is the baseline the measurable compares against). Emit a `template_utility_ranked` SSB row carrying the chosen template's `TemplateUtility` snapshot. This is the W-8 in-plan caller of `laplace_utility`.

3. **Add the `template_utility_ranked` SSB action_type + payload (~70 LOC in ssb.rs).** New `SsbActionType::TemplateUtilityRanked` + `TemplateUtilityRankedPayload { concept_class: String, plan_skeleton_hash: u64, utility: f32, first_try_ratio: f32, block_ratio: f32, coverage: f32, sample_n: u32, chosen_pipeline_id: String }`. Register `as_sql_str` = `"template_utility_ranked"`, the SQL CHECK constraint arm, the parse arm, the round-trip golden fixture. Add the row to crates/orchestrator/tests/action_type_lock.rs ALL_VARIANTS (W-9). SCHEMA_VERSION baseline + 1 for HALF B if shipped alone, or baseline + 2 atomic if co-shipped with HALF A.

4. **CLI surface `elai template-ranking <list|show>` (~90 LOC in crates/cli/src/main.rs + new crates/cli/src/template_ranking_cmd.rs).** `list` prints every crystallized `(concept_class, plan_skeleton_hash)` bucket with its `TemplateUtility` (utility, first_try_ratio, block_ratio, coverage, sample_n) sorted by utility desc. `show --concept-class X` prints the single bucket detail. This is the W-3 production entrypoint for HALF B and the W-5 observable_effect trigger. `--json` flag per harness convention.

5. **Frontend pair for HALF B (~220 LOC svelte + ~120 LOC tauri) (W-4).** `crates/dashboard-v2/src/lib/components/TemplateRankingPanel.svelte` consuming a new Tauri command `get_template_ranking` in crates/dashboard-v2/src-tauri/src/commands.rs (pattern: `#[tauri::command] async fn get_template_ranking(state: State<'_, AppState>) -> CmdResult<Value>` reading the `template_utility_ranked` SSB rows + the live crystallizer buckets, returning a typed DTO array). Mount the panel in MonitorWindow.svelte (import alongside the existing ProposalsPanel import at MonitorWindow.svelte:35). The panel shows each template's utility WITH the old first_try_ratio side-by-side (the before/after the measurable needs). cross_lang_diff must pass.

### HALF A — safety-rule lattice completeness proof (E-53 amendment, RunPod-gated live, fixture-harness now) — TIER 3

6. **Add `crates/orchestrator/src/formal_verify/lattice.rs` (~200 LOC).** New module that READS the `mutable=0` rule subset and builds the completeness/monotonicity/conflict-freedom obligation. Public symbols: `pub fn enumerate_immutable_rules(conn: &rusqlite::Connection) -> Vec<ImmutableRuleRow>` (reuses the lifecycle.rs:308 row-decode shape: `mutable: row != 0`), `pub fn encode_lattice_obligation(rules: &[ImmutableRuleRow]) -> SmtLib2Obligation` producing an SMT-LIB2 string asserting (a) COMPLETE: every declared security/finance class has >=1 covering immune rule, (b) MONOTONE: rule-set union never reduces the blocked-action set, (c) CONFLICT-FREE: no two immune rules assert contradictory verdicts on the same argv/class. `pub struct SmtLib2Obligation { pub class_count: usize, pub rule_count: usize, pub smt2: String, pub obligation_blake3: String }` — the obligation hash uses blake3 (CLAUDE.md HARD RULE; NEVER blake2/sha2). No `.unwrap()` in production; propagate `rusqlite::Error` via `?`.

7. **Extend the E-53 Z3 route to dispatch the lattice obligation (~80 LOC delta in formal_verify/orchestrator.rs + types.rs).** Add `VerifierKind::Z3Smt` handling so that when `VerifyRequest.target_path` resolves to the rules DB (a new `VerifyRequest` discriminator field `pub lattice: bool` OR a dedicated `VerifyRequest::lattice(conn_path)` constructor), `run` calls `lattice::encode_lattice_obligation` then dispatches Z3. RUNPOD-PENDING boundary preserved: when the Z3 binary is absent (`expected_local_install()` false), return `VerifierOutcome::RunpodPending` (NOT a fabricated Verified — W-5/W-10, never advertise a guarantee not delivered). When present and the obligation discharges, return `VerifierOutcome::Verified`; on a counterexample, `VerifierOutcome::Refuted { counterexample_label }`. The dispatch boundary lands now; the live SMT shellout body is the RUNPOD-gated sub-phase.

8. **CLI verb `elai formal-verify lattice` (~70 LOC delta in crates/cli/src/formal_verify_cmd.rs).** Add `FormalVerifyAction::Lattice` to crates/cli/src/args.rs and a `run_lattice_dispatch` that builds the obligation from the rules DB, calls `formal_verify::orchestrator::run` with the lattice request, prints the `PipelineResult` (verifier=z3_smt, outcome, obligation hash, class_count/rule_count), and emits the `safety_lattice_verified` SSB row. This is the W-3 production entrypoint + W-5 observable_effect trigger for HALF A. When Z3 is absent the printed outcome is `runpod_pending` and the SSB row records `verified=false, runpod_pending=true` (the fixture harness asserts THIS path, the live proof is the separate gated checkbox).

9. **Add `safety_lattice_verified` SSB action_type as an E-53 amendment (~70 LOC in ssb.rs).** New `SsbActionType::SafetyLatticeVerified` + `SafetyLatticeVerifiedPayload { class_count: usize, rule_count: usize, obligation_blake3: String, outcome: String, verified: bool, runpod_pending: bool }`. Register `as_sql_str` = `"safety_lattice_verified"`, SQL CHECK arm, parse arm, round-trip golden, ALL_VARIANTS row in action_type_lock.rs (W-9). This is the named E-53 amendment SSB variant per F-0_INDEX SCHEMA bump-site table.

10. **Deterministic fixture harness for HALF A (~140 LOC at crates/contracts/tests/f7_lattice_fixture_contract.rs).** Per W-5: because the live Z3 proof is RUNPOD-PENDING, ship a fixture harness that exercises the REAL `formal_verify::orchestrator::run` path (NOT a mock of it) with a seeded in-memory rules DB containing (a) a COMPLETE conflict-free immune set -> asserts `encode_lattice_obligation` produces a well-formed obligation + `run` returns `RunpodPending` locally (binary absent) with the obligation hash present, (b) a DELIBERATELY INCOMPLETE set (a security class with no covering rule) -> asserts the obligation encodes the gap (the `smt2` string contains the uncovered-class assertion). This proves the production path end-to-end up to the Z3 boundary without RunPod. The LIVE z3-discharges-the-obligation assertion is a SEPARATE, explicitly RunPod-gated test (`#[ignore]` until a 2x3090 window) — W-5/W-10.

11. **Frontend pair for HALF A (reuse the HALF B panel or add a tab) (~80 LOC svelte + ~60 LOC tauri) (W-4).** Add a `lattice` section to `TemplateRankingPanel.svelte` (or a sibling `VerificationStatusPanel.svelte` mounted in MonitorWindow) consuming a Tauri command `get_lattice_status` that reads the latest `safety_lattice_verified` SSB row and renders {class_count, rule_count, outcome, verified/runpod_pending, obligation hash prefix}. This makes the W-6 dashboard E2E item satisfiable for HALF A (a human sees the lattice posture, including the honest "runpod_pending" state). cross_lang_diff must pass.

### Both halves

12. **Decision doc at `workflow/reasoning/F-7-decisions.md` (~120 LOC).** Covering: (a) why HALF A is an E-53 amendment not a standalone verifier (reuses VerifierKind::Z3Smt + the run pipeline — no duplicate toolchain), (b) why HALF B keeps `first_try_success_rate` as a recorded baseline (the measurable compares against it; no-removal rule), (c) the Springdrift correction (1.2) — both the lattice-completeness claim and the CBR uplift are DESIGN HYPOTHESES, the +10.7% target is DROPPED, the measurable is a MEASURED delta after >=100 labelled sessions, (d) the W-10 tier split: HALF A's Z3 subprocess is OUT of any fail-closed hot path (offline CLI + FSM evidence gate only), HALF B is pure in-memory arithmetic, (e) RUNPOD-PENDING posture for the live Z3 proof + the fixture-harness substitute, (f) the Laplace alpha/beta default rationale (1/1 = uniform prior, the smallest prior that prevents 1/1 buckets outranking 95/100).

## WIRING_CONTRACT

produces_symbols:
  # HALF B (SHIPPED, as-built). The CBR template-utility ranking ships in the
  # orchestrator crate + the CLI template-ranking subcommand + commands_f7.rs +
  # TemplateRankingDrawer.svelte. SSB action-type lines carry the ssb.rs file prefix.
  - crates/orchestrator/src/template_utility.rs::laplace_utility
  - crates/orchestrator/src/template_utility.rs::TemplateUtility
  - crates/cli/src/template_ranking_cmd.rs::run_template_ranking_subcommand
  - crates/orchestrator/src/ssb.rs::SsbActionType::TemplateUtilityRanked
  - crates/orchestrator/src/ssb.rs::TemplateUtilityRankedPayload
  - crates/dashboard-v2/src-tauri/src/commands_f7.rs::get_template_ranking
  - crates/dashboard-v2/src/lib/drawers/TemplateRankingDrawer.svelte
  # HALF A (INTENTIONALLY DEFERRED, NOT a wiring failure). The safety-lattice /
  # SMT formal-verify half (formal_verify/lattice.rs, SafetyLatticeVerified,
  # get_lattice_status) is deferred per the E-53 amendment, RunPod-pending; see
  # workflow/reasoning/F-7-decisions.md. These symbols do NOT exist as-built by
  # design; their absence is the documented deferral, not a wire-dark gap.
  # DEFERRED: crates/orchestrator/src/formal_verify/lattice.rs::enumerate_immutable_rules
  # DEFERRED: crates/orchestrator/src/formal_verify/lattice.rs::encode_lattice_obligation
  # DEFERRED: crates/orchestrator/src/formal_verify/lattice.rs::SmtLib2Obligation
  # DEFERRED: crates/orchestrator/src/ssb.rs::SsbActionType::SafetyLatticeVerified
  # DEFERRED: crates/orchestrator/src/ssb.rs::SafetyLatticeVerifiedPayload
  # DEFERRED: crates/dashboard-v2/src-tauri/src/commands.rs::get_lattice_status
consumes_symbols:
  - crates/orchestrator/src/compiled_pipeline.rs::CompiledExecutor::try_execute
  - crates/orchestrator/src/compiled_pipeline.rs::ExecutionOutcome
  - crates/orchestrator/src/compiled_pipeline.rs::CompiledPipeline
  - crates/orchestrator/src/pipeline_builder.rs::classify_task_for_template
  - crates/orchestrator/src/formal_verify/orchestrator.rs::run
  - crates/orchestrator/src/formal_verify/types.rs::VerifierKind
  - crates/orchestrator/src/formal_verify/types.rs::VerifierOutcome
  - crates/orchestrator/src/formal_verify/types.rs::VerifyRequest
  - crates/rules/src/lifecycle.rs::RuleRecord
  - crates/rules/src/lifecycle.rs::fetch_rule_record
  - crates/cli/src/formal_verify_cmd.rs::run
reachable_from:
  # HALF B (SHIPPED): the CLI subcommand is the real entrypoint.
  - crates/cli/src/main.rs:768 (elai template-ranking list -> template_ranking_cmd::run_template_ranking_subcommand -> emit_ranking_rows -> template_utility_ranked SSB row)
  - crates/orchestrator/src/task_runner.rs::run_crystallizer_post_execute_hook (live FSM path; CBR utility recorded when a template is selected during EXECUTING)
  - crates/dashboard-v2/src-tauri/src/commands_f7.rs::get_template_ranking (Tauri -> TemplateRankingDrawer.svelte)
  # HALF A (DEFERRED per F-7-decisions.md / E-53 amendment, RunPod-pending): the
  # formal-verify lattice entrypoint (elai formal-verify lattice -> safety_lattice_verified)
  # is not wired as-built by design; not a reachability failure.
stub_ban:
  - "TODO: lattice later" literal absent
  - "// CBR ranking placeholder" literal absent
  - "unimplemented!" / "todo!" absent in template_utility.rs and lattice.rs
  - No default-off env-gate as the terminal state for HALF B: ELAI_TEMPLATE_CBR_ENABLE (if present) is a KILL-SWITCH only; its DEFAULT is ON and CompiledExecutor::try_execute uses laplace_utility with NO env override required (W-2). The session is NOT DONE until the observable_effect runs with no env var set.
  - No LLM-bypass branch: neither laplace_utility nor encode_lattice_obligation may consult or be overridden by a model call — both are deterministic in-process functions (the ranking is arithmetic over journaled rows; the lattice obligation is a pure encode). A prompt cannot disable CBR ranking or fabricate a lattice Verified outcome.
  - No fabricated Verified: HALF A returns RunpodPending (never Verified) when the Z3 binary is absent — fail-honest, never advertise a proof not run (W-5/W-10).
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/TemplateRankingDrawer.svelte (HALF B, as-built — a drawer, not a MonitorWindow panel)
  tauri: crates/dashboard-v2/src-tauri/src/commands_f7.rs::get_template_ranking (HALF B; get_lattice_status is HALF-A DEFERRED, not shipped)
observable_effect:
  # Scoped to HALF B (the shipped half). HALF A (safety_lattice_verified) is
  # DEFERRED per F-7-decisions.md / E-53 amendment and is NOT asserted here.
  trigger: "cd .elai_cc && cargo run -p elai-cli -- template-ranking list --json"
  expect_ssb: "template_utility_ranked"
  expect_artifact: "stdout JSON matching \\\"utility\\\"\\s*:\\s*[01]?\\.[0-9]+ with a \\\"first_try_ratio\\\" sibling (HALF B). HALF-A artifact (lattice outcome/obligation_blake3) is DEFERRED, not asserted."

## DETERMINISTIC_CHECK

```bash
# W-1 WIRING DONE-GATE (the literal gate line — this is the done-gate, not cargo-green)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md --run-trigger

# Produced HALF-B symbols exist
rg "pub fn laplace_utility|pub struct TemplateUtility" .elai_cc/crates/orchestrator/src/template_utility.rs
rg "fn try_execute" .elai_cc/crates/orchestrator/src/compiled_pipeline.rs
rg "laplace_utility" .elai_cc/crates/orchestrator/src/compiled_pipeline.rs   # the in-plan caller (W-8)

# Produced HALF-A symbols exist
rg "pub fn enumerate_immutable_rules|pub fn encode_lattice_obligation|pub struct SmtLib2Obligation" .elai_cc/crates/orchestrator/src/formal_verify/lattice.rs
rg "FormalVerifyAction::Lattice|run_lattice_dispatch" .elai_cc/crates/cli/src/formal_verify_cmd.rs

# New SSB action types + golden lock (W-9)
rg "template_utility_ranked|safety_lattice_verified" .elai_cc/crates/orchestrator/src/ssb.rs
rg "TemplateUtilityRanked|SafetyLatticeVerified" .elai_cc/crates/orchestrator/tests/action_type_lock.rs

# blake3 (HARD RULE) — obligation hash must be blake3, never blake2/sha2
rg "blake3" .elai_cc/crates/orchestrator/src/formal_verify/lattice.rs
rg -n "blake2|sha2|sha256" .elai_cc/crates/orchestrator/src/formal_verify/lattice.rs && echo "FAIL: non-blake3 hash present" || echo "OK: no banned hash"

# No .unwrap()/.expect() in the two new production modules
rg -n "\.unwrap\(\)|\.expect\(" .elai_cc/crates/orchestrator/src/template_utility.rs .elai_cc/crates/orchestrator/src/formal_verify/lattice.rs && echo "FAIL: unwrap/expect in prod" || echo "OK"

# Frontend pair + cross-language drift (W-4)
test -f .elai_cc/crates/dashboard-v2/src/lib/components/TemplateRankingPanel.svelte
rg "get_template_ranking|get_lattice_status" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
python .claude/tools/cross_lang_diff.py

# Default-ON proof (W-2): the kill-switch, if present, defaults ON
rg -n "ELAI_TEMPLATE_CBR_ENABLE" .elai_cc/crates/orchestrator/src/compiled_pipeline.rs && echo "kill-switch present — verify default ON in code review" || echo "no env gate — default ON by construction"

# Decision doc
test -f workflow/reasoning/F-7-decisions.md
```

## ACCEPTANCE

- [ ] **W-1 WIRING DONE-GATE**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md --run-trigger` returns PASS (exit 0): every produces_symbol is a DEF/SCIP_DEF in elai.map, every consumes_symbol has a real CALL/REF, every reachable_from anchor reached by BFS over CALL_RESOLVED within depth 8, stub_ban finds zero forbidden literals, AND the observable_effect trigger RAN and emitted the expected SSB rows + artifact. cargo-green alone is NOT sufficient.
- [ ] **W-2 DEFAULT-ON**: the observable_effect trigger (`elai template-ranking list`) ranks via `laplace_utility` and emits `template_utility_ranked` with NO env override set (no `ELAI_TEMPLATE_CBR_ENABLE=1` needed). If a kill-switch env var exists it DISABLES; default is ON.
- [ ] **W-6 DASHBOARD E2E**: through the dashboard (dashboard-smoker), the TemplateRankingPanel renders >=1 template's utility WITH its first_try_ratio side-by-side, and the lattice section renders the latest `safety_lattice_verified` outcome (including an honest `runpod_pending` state). Verified through the live UI, not a unit test.
- [ ] **W-8 PRODUCES/CONSUMES SYMMETRY**: `laplace_utility` is called by `CompiledExecutor::try_execute` in-plan; `encode_lattice_obligation` is called by `run_lattice_dispatch` in-plan; both callers confirmed by the wiring check.
- [ ] HALF B: `laplace_utility` correctly outranks a 95/100 bucket over a 1/1 bucket (the smoothing acceptance — a unit assertion AND the live `template-ranking list` ordering).
- [ ] HALF B measurable: the decision doc records that the CBR-vs-`first_try_success_rate` delta is to be MEASURED after >=100 labelled sessions — NOT the dropped Springdrift +10.7% (synthesis 1.2 caveat present and explicit).
- [ ] **HALF A FIXTURE HARNESS (RunPod-substitute, ships now)**: `f7_lattice_fixture_contract.rs` exercises the REAL `formal_verify::orchestrator::run` path on a seeded rules DB; the complete-set case returns `RunpodPending` with a present `obligation_blake3`; the incomplete-set case encodes the uncovered-class gap in the `smt2` obligation.
- [ ] **HALF A LIVE RUN (separately gated, RUNPOD-PENDING)**: on a 2x3090 window with Z3 installed, `elai formal-verify lattice` discharges the obligation and returns `Verified` for a complete conflict-free immune set; a planted contradictory immune rule returns `Refuted`. This checkbox is SEPARATE from the fixture-harness checkbox and stays unchecked until the live window (W-5).
- [ ] **W-9**: both new SSB variants flow through `action_type_lock.rs` ALL_VARIANTS, round-trip golden green, cross_lang_diff green; SCHEMA_VERSION bump named (baseline + 1 for the E-53 amendment alone, baseline + 2 atomic if co-shipped).
- [ ] blake3 used for the obligation hash (no blake2/sha2); no `.unwrap()`/`.expect()` in template_utility.rs or lattice.rs production paths.
- [ ] `cargo check --workspace` GREEN; auditor (sole test runner) PASS on `-p elai-orchestrator -p elai-cli -p contracts`; no new dependency outside cargo verify (anti-slopsquatting).

## Tests to add

- `crates/contracts/tests/f7_template_utility_contract.rs`: `laplace_utility` smoothing (1/1 < 95/100), alpha/beta=0 counterfactual guard, clamp to [0,1], block_ratio down-weight; `template_utility_ranked` SSB round-trip.
- `crates/contracts/tests/f7_lattice_fixture_contract.rs`: the deterministic fixture harness (Task 10) — real `run` path on a seeded DB, complete-set RunpodPending + incomplete-set gap encoding; `safety_lattice_verified` SSB round-trip. Includes the `#[ignore]`-gated live Z3 test (Task 10 second half).
- extend `crates/orchestrator/tests/action_type_lock.rs`: both new variants in ALL_VARIANTS + golden match.
- HALF B CLI smoke (covered by observable_effect): `elai template-ranking list --json` produces a `template_utility_ranked` row.

## Files touched

- CREATE: `crates/orchestrator/src/template_utility.rs` (~180 LOC: laplace_utility + TemplateUtility)
- CREATE: `crates/orchestrator/src/formal_verify/lattice.rs` (~200 LOC: enumerate_immutable_rules + encode_lattice_obligation + SmtLib2Obligation)
- CREATE: `crates/cli/src/template_ranking_cmd.rs` (~90 LOC: elai template-ranking list|show)
- CREATE: `crates/dashboard-v2/src/lib/components/TemplateRankingPanel.svelte` (~220 LOC, includes the lattice section)
- CREATE: `crates/contracts/tests/f7_template_utility_contract.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/f7_lattice_fixture_contract.rs` (~140 LOC)
- CREATE: `workflow/reasoning/F-7-decisions.md` (~120 LOC)
- MODIFY: `crates/orchestrator/src/compiled_pipeline.rs:486-540` (~70 LOC: wire laplace_utility into CompiledExecutor::try_execute; keep first_try_success_rate as baseline field)
- MODIFY: `crates/orchestrator/src/formal_verify/orchestrator.rs:111-133` (~80 LOC: Z3 route dispatches the lattice obligation; RunpodPending preserved when binary absent)
- MODIFY: `crates/orchestrator/src/formal_verify/types.rs` (~20 LOC: VerifyRequest lattice discriminator/constructor)
- MODIFY: `crates/orchestrator/src/formal_verify/mod.rs` (~3 LOC: `pub mod lattice;` + re-exports)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~140 LOC: 2 new action_types + 2 payloads + sql_str + parse + CHECK + round-trip goldens)
- MODIFY: `crates/orchestrator/src/lib.rs` (~2 LOC: `pub mod template_utility;`)
- MODIFY: `crates/orchestrator/tests/action_type_lock.rs` (~6 LOC: 2 ALL_VARIANTS rows)
- MODIFY: `crates/cli/src/formal_verify_cmd.rs` (~70 LOC: Lattice verb + run_lattice_dispatch + safety_lattice_verified emit)
- MODIFY: `crates/cli/src/args.rs` (~10 LOC: FormalVerifyAction::Lattice + TemplateRanking subcommand args)
- MODIFY: `crates/cli/src/main.rs` (~20 LOC: dispatch template-ranking + formal-verify lattice)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~180 LOC: get_template_ranking + get_lattice_status Tauri commands)
- MODIFY: `crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte` (~10 LOC: mount TemplateRankingPanel)

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 C7 (PRIMARY scope + the REVISE verdict from wf_b959963a-db6) — PARTIALLY_GROUNDED per synthesis.
- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 1.2 (the Springdrift correction — 7,056-pair / +10.7% are SYNTHETIC/UNREPRODUCED; both halves are DESIGN HYPOTHESES; the +10.7% target is DROPPED). UNVERIFIED: any lattice-completeness coverage % or CBR uplift % stated as a target.
- workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md Pillar 5 (the git-whole-state + proven-complete-calculus + CBR framing). The git-whole-state mechanism is CONFIRMED real; the calculus axiom constants are CLAIMED-FROM-PAPER-ONLY (synthesis 1.2) — NOT source-verified, do NOT import as constants.
- ELAI ground truth (code-read 2026-06-01 via elai.map + source): RuleRecord.mutable (lifecycle.rs:126), VerifierKind/VerifierOutcome (formal_verify/types.rs:105), formal_verify::orchestrator::run (orchestrator.rs:111), z3_stub Inapplicable (orchestrator.rs:117-119), TemplateCrystallizer + first_try_success_rate (compiled_pipeline.rs:292,365), CompiledExecutor::try_execute (compiled_pipeline.rs:486), classify_task_for_template (pipeline_builder.rs:391), run_crystallizer_post_execute_hook (task_runner.rs:3470), formal_verify_cmd::run (formal_verify_cmd.rs:31, main.rs:599), action_type_lock golden (tests/action_type_lock.rs:2167). All W-7 anchors confirmed.
- arXiv:2501.13956 (Graphiti bi-temporal) — referenced by the synthesis for the broader C5/C7 cluster framing; NOT load-bearing for F-7's mechanism (reference only).
- CLAUDE.md HARD RULES applied: blake3 for the obligation hash (never blake2/sha2); no .unwrap()/.expect() in production; Pattern-2 SSB collapse considered (both variants are distinct single-field payloads, not a multi-arm Kind — no Pattern-2 collapse needed, but the golden-lock W-9 path is mandatory); anti-slopsquatting (no new dep — Z3 is a runtime `which`-probe per E-53, SMT-LIB2 is a hand-emitted string, no z3 crate added without confirmation).

## Open questions / risks

- RISK (HALF A blocked on E-53): the live lattice proof CANNOT complete until E-53's Z3 route resolves (RUNPOD-PENDING). MITIGATION: the dispatch boundary + obligation encoder + fixture harness ship now and are fully wired (the CLI verb + SSB row + dashboard tile all work, showing the honest `runpod_pending` state); only the `Verified` discharge is gated. This is the W-5 "deterministic fixture harness exercising the REAL path + a separate gated live checkbox" pattern, NOT a stub.
- RISK (HALF B measurable is unbaselined): there is NO ELAI baseline for CBR-vs-frequency template-reuse quality today. MITIGATION: keep `first_try_success_rate` as a recorded field and surface both side-by-side; the delta is MEASURED after >=100 labelled sessions, never asserted from Springdrift's synthetic +10.7% (synthesis 1.2). Do NOT gate the F-N on a numeric uplift — gate it on the WIRING done-gate + the smoothing-correctness assertion.
- RISK (Laplace prior choice): alpha=beta=1 (uniform prior) is the smallest prior that prevents a 1/1 bucket outranking 95/100. Two approaches: (1) fixed 1/1 uniform prior (shipped — simplest, defensible, no per-class tuning); (2) per-concept-class empirical-Bayes prior fitted from the global success rate (DEFERRED — needs >=100 sessions of data that do not exist yet, and adds a fitting step with its own failure mode). Side effect of (1): under-smooths classes with genuinely high variance; acceptable until data exists to fit (2).
- RISK (SMT-LIB2 hand-emission correctness): emitting the obligation as a string (no z3 crate) means a malformed obligation could be silently accepted by a future Z3. MITIGATION: the fixture harness asserts the obligation STRUCTURE (uncovered-class assertion present in the incomplete case); the live Z3 run (gated) is the true check. Flag for the auditor: the hand-emitted SMT must be reviewed against the SMT-LIB2 grammar before the live run.
- RISK (W-2 escape hatch): if a staged-rollout kill-switch `ELAI_TEMPLATE_CBR_ENABLE` is added, it MUST default ON and the session is not DONE until the observable_effect runs with no env override (the exact night-run failure mode F-plan exists to prevent).
- Anti-slopsquatting: no z3 crate, no new dependency. Z3 is invoked via the E-53 `which`-style probe (RUNPOD-PENDING). If a Rust SMT library is later proposed, verify name + version + last-published before any cargo add, with user confirmation (CLAUDE.md HARD RULE).

## WIRING DONE-GATE

This section states, clause by clause, how F-7 satisfies the F-plan doctrine W-1..W-10 so the implementer cannot mark it DONE while wire-dark.

- **W-1 (no-wire-dark done-gate)**: DONE requires `wiring_contract_check.py --run-trigger` PASS, NOT cargo-green. The observable_effect trigger (`elai template-ranking list` + `elai formal-verify lattice`) must actually RUN and emit `template_utility_ranked` + `safety_lattice_verified` rows into the live ssb.db, with the stdout artifact matching the regexes. The wiring-contract-validator agent runs this; the FSM may not reach COMPLETE without its PASS.
- **W-2 (no default-off terminal state)**: HALF B's CBR ranking is ON by default in `CompiledExecutor::try_execute` — no env var required to activate. Any `ELAI_TEMPLATE_CBR_ENABLE` is a kill-switch (default ON); stub_ban forbids the default-off terminal state; the W-2 acceptance item runs the trigger with NO override.
- **W-3 (production entrypoint)**: both halves are reachable from REAL production entrypoints — `elai template-ranking` and `elai formal-verify lattice` CLI subcommands (crates/cli/src/main.rs), the live FSM path `run_crystallizer_post_execute_hook` (task_runner.rs:3470) for CBR recording, and the Tauri commands for the dashboard. No symbol is test-only.
- **W-4 (frontend pair mandatory)**: both new SSB signals (`template_utility_ranked`, `safety_lattice_verified`) are human-surfaceable, so `TemplateRankingPanel.svelte` + `get_template_ranking`/`get_lattice_status` Tauri commands are mandatory and cross_lang_diff must pass. No backend-only invisible signal.
- **W-5 (live observable_effect, not a unit test)**: observable_effect is the two CLI invocations producing live ssb.db rows + inspectable JSON stdout. HALF A's live Z3 proof is RUNPOD-PENDING, so it ships the deterministic fixture harness (`f7_lattice_fixture_contract.rs`) exercising the REAL `formal_verify::orchestrator::run` path (not a mock) PLUS a separate `#[ignore]`-gated live checkbox — exactly the W-5 RunPod-gated pattern.
- **W-6 (dashboard E2E)**: the W-6 acceptance item is verified THROUGH the dashboard via dashboard-smoker — the panel renders utility-with-first_try_ratio and the honest lattice posture (including runpod_pending). Not N/A; F-7 has a real UI surface.
- **W-7 (no hallucinated integration points)**: every consumes_symbol, every reachable_from anchor, and every MODIFY target was GREPPED in elai.map at authoring time (CompiledExecutor::try_execute @compiled_pipeline.rs:486, formal_verify::orchestrator::run @orchestrator.rs:111, RuleRecord.mutable @lifecycle.rs:126, fetch_rule_record @lifecycle.rs:277 [corrected from insert_rule which is a test-only private helper], classify_task_for_template @pipeline_builder.rs:391, run_crystallizer_post_execute_hook @task_runner.rs:3470, formal_verify_cmd::run @formal_verify_cmd.rs:31, action_type_lock @tests/action_type_lock.rs, Tauri commands @dashboard-v2/src-tauri/src/commands.rs). The validator re-greps.
- **W-8 (produces/consumes symmetry)**: `laplace_utility` (produced) is called in-plan by `CompiledExecutor::try_execute` (Task 2); `encode_lattice_obligation` (produced) is called in-plan by `run_lattice_dispatch` (Task 8); `get_template_ranking` (produced) is consumed by `TemplateRankingPanel.svelte` (Task 5). No produced symbol is left caller-less.
- **W-9 (SSB golden lock + named SCHEMA bump)**: `TemplateUtilityRanked` + `SafetyLatticeVerified` flow through `action_type_lock.rs` ALL_VARIANTS + round-trip goldens + cross_lang_diff; SCHEMA_VERSION bump named baseline + 1 (E-53 amendment alone) or baseline + 2 (atomic co-ship). Neither variant is a multi-arm Kind, so no Pattern-2 collapse is forced; the golden-lock path is still mandatory.
- **W-10 (tiered gate — network/subprocess/RunPod split out of the fail-closed hot path)**: HALF A's Z3 SUBPROCESS is split entirely OUT of any fail-closed per-write path — it runs only as an offline CLI verification + an FSM evidence gate, and returns RunpodPending (fail-honest, never a fabricated Verified) when the binary is absent. HALF B is pure in-memory arithmetic over already-journaled rows — Tier-1, zero subprocess, ships now. Neither half advertises a guarantee it does not deliver.

## Next

E-53 is the natural composition: when its Z3/Kani/Verus routes resolve (RUNPOD), HALF A's lattice obligation discharges live and the `safety_lattice_verified` row flips from `runpod_pending=true` to `verified=true` with no code change — the wire is already complete. HALF B feeds the future per-concept-class empirical-Bayes prior (deferred until >=100 labelled sessions exist) and composes with F-6 (learned-skill library): once SkillTrustTier metadata lands, template utility can be a tier-promotion input. F-8 (SCF self-mod) can consume the lattice obligation as a pre-archive invariant check (does a proposed self-mod config contradict the proven-complete immune set?). The MEASURED CBR-vs-frequency delta becomes a reportable number once the labelled-session corpus exists — a real ELAI-engine metric, never a Springdrift import.

End of F-7.
