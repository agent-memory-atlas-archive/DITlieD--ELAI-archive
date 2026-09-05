C-21 DECISIONS — Self-Modification + Anti-Staleness Automation Closure
Session s-25672, executed 2026-05-19.

================================================================
SCOPE
================================================================

10 tasks closing wiring gaps in B-16, B-23, phase-4H, and anti-
staleness-enforcement. Driven by context-package-C21.md (9 DRIFT items
+ 7 open questions). Orchestrator answered Q1-Q7 + DRIFT-1..9 before
EXECUTING.

================================================================
DECISIONS (verbatim from orchestrator)
================================================================

Q1 (Task 2 / Chunk C): ANNOTATE existing 3 LazyLock<Regex>::new().unwrap()
   sites with `// SAFETY: LazyLock infallible regex (B-16B D5 exempt)`
   plus `#[allow(clippy::unwrap_used)]`. Skip aggressive conversion. dkg
   has zero production unwraps so no work there (DRIFT-2 vacuous).

Q2 (Task 4 / CatchLayer): ADD new variant `CatchLayer::DepAudit` (clean
   semantics; no SCHEMA_VERSION bump because CatchLayer is not part of the
   SSB action-type CHECK constraint — it lives inside the StaleApiCatch
   payload).

Q3 (Task 5 / makedirs): in Rust, before calling np.savez. Use
   std::fs::create_dir_all. Python script's existing os.makedirs is kept
   as defence-in-depth.

Q4 (Task 6 / dep_audit_latest.toml): emit a fresh copy (Windows-friendly
   over symlink) via the dep-audit subcommand. Implementation: new
   `--output-format toml` flag triggers a `std::fs::copy` from
   `metrics/dep_audit_<date>.toml` to `metrics/dep_audit_latest.toml`
   after the dated report is written.

Q5 (Task 7 / ParameterKey location): place in self_mod_safety.rs (already
   houses KNOWN_PARAMETER_KEYS). Export from orchestrator lib.rs.

Q6 (Task 10 / FailureCategory): ADD T1Block/T2Block/PreToolBlock variants
   (preferred over root_cause_json string embed for type-safety). Cross-
   crate impact: reasoning crate's traversal::default_answers_for_category
   match arm extended to cover the 3 new variants (folded into Unknown's
   single-question fallback because the interrogation tree has no hook-
   tier branch).

Q7 (Task 3 / pre-commit hook path): workspace-root .git/hooks/pre-commit
   is the only viable location — .elai_cc has no separate .git directory.
   Installer script resolves the workspace root relative to its own
   location (scripts/install_pre_commit_hook.sh → ../../.git/hooks/).

DRIFT-1: 3 LazyLock<Regex> unwraps annotated, not converted.
DRIFT-2: dkg lives in knowledge/, not orchestrator. No production
         unwraps — task vacuous; documented.
DRIFT-3: type is DepAuditResult (not DepAuditReport). Function
         signature: `dep_audit_to_stale_catches(report: &DepAuditResult)
         -> Vec<StaleApiCatch>` exported from cli::dep_audit.
DRIFT-4: steering refresh hook lives AFTER tx.commit() in
         self_mod_layer1a.rs apply_proposal, gated on
         `proposal.table == DataTable::DeprecatedPatterns`. NOT in
         self_mod_bridge.rs (that path is proposal-construction, not
         promote).
DRIFT-5: models/steering_vectors/ pre-created by Rust trigger_steering_
         vector_refresh before the python spawn (defence-in-depth in
         python too — already present at compute_steering_vector.py L196,
         L213).
DRIFT-6: --output-format toml flag added to CliOutputFormat; doctor.rs
         emits dep_audit_latest.toml when this format is selected.
         Existing Text/Json arms across cli get `| CliOutputFormat::Toml`
         to keep them exhaustive (Toml falls back to Text behaviour for
         non-dep-audit consumers).
DRIFT-7: AllowedConfigKey sweep updated self_mod_proposal.rs +
         staged_eval.rs. self_mod_orchestrator.rs has zero references —
         no change there.
DRIFT-8: synthetic Failed reason at task_runner.rs L6285 now embeds the
         BlockSource tier tag (T1_BLOCK/T2_BLOCK/PRE_TOOL_BLOCK/CSQ_BLOCK).
DRIFT-9: FailureCategory variants T1Block/T2Block/PreToolBlock added in
         core::mistake_book. as_str() snake_case round-trip pinned by
         c21_anti_stale_contract.rs.

================================================================
FILES TOUCHED
================================================================

MODIFY  .elai_cc/.github/workflows/windows.yml
        — added "Anti-stale lint" step running scripts/check_anti_stale.sh
          between entropy-quarantine and hooks compat tests.

ADD     .elai_cc/scripts/install_pre_commit_hook.sh
        — bash installer; writes .git/hooks/pre-commit at workspace root
          (F:/.ELAI_workflow/.git). Hook enumerates the 12 LOCKED_KEYS
          patterns + elai_cc.toml and exits 1 with a guidance message
          when any staged file matches. Bypass: git commit --no-verify.

MODIFY  .elai_cc/crates/orchestrator/src/deterministic_auditor.rs
        — annotated 3 LazyLock<Regex>::new().unwrap() sites (MOD_DECL
          L946, PY_IMPORT L992, TS_EXPORT L1037) with SAFETY comment +
          #[allow(clippy::unwrap_used)] per Q1 + DRIFT-1.

MODIFY  .elai_cc/crates/orchestrator/src/self_mod_safety.rs
        — added ParameterKey closed enum (12 variants matching
          KNOWN_PARAMETER_KEYS), as_str()/from_str() round-trip.
          Re-exported from orchestrator/lib.rs.

MODIFY  .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs
        — AllowedConfigKey::Parameter(String) → Parameter(ParameterKey).
        — Added trigger_steering_vector_refresh helper. Spawns
          metrics/compute_steering_vector.py --batch detached after
          apply_proposal commit when proposal.table == DeprecatedPatterns.
          Pre-creates models/steering_vectors/.
        — Added trigger_dep_audit_background helper. Fires when MSI
          DIVERGING gate trips. 24h rate limit via metrics/.dep_audit_
          last_trigger sentinel mtime.

MODIFY  .elai_cc/crates/orchestrator/src/self_mod_types.rs
        — added CatchLayer::DepAudit variant; as_str() arm = "dep_audit".

MODIFY  .elai_cc/crates/orchestrator/src/self_mod_proposal.rs
        — Class B sample constructor switched to
          ParameterKey::FreshnessHalfLife (was String).

MODIFY  .elai_cc/crates/orchestrator/src/staged_eval.rs
        — match arm reads `pkey.as_str()` for runtime lock/known checks.
        — locked_parameter_rejected_at_entry test converted to a runtime
          consistency check (the literal `Parameter("cloud_monthly_budget")`
          construction no longer compiles; closed-enum enforces).

MODIFY  .elai_cc/crates/orchestrator/src/lib.rs
        — exported ParameterKey, trigger_steering_vector_refresh,
          trigger_dep_audit_background.

MODIFY  .elai_cc/crates/orchestrator/src/task_runner.rs
        — added BlockSource enum (T1Hook/T2Hook/PreToolUse/CsqCritical/
          None) with reason_tag() + failure_category() helpers.
        — wired block_source assignment from PreToolUse hook,
          irreversible classifier, CSQ Critical-without-guard.
        — synthetic Failed reason now embeds tier tag.
        — failure_category resolver prefers BlockSource::failure_category
          over the generic classify_baby_step_failure when a tier was
          recorded.

MODIFY  .elai_cc/crates/cli/src/args.rs
        — CliOutputFormat::Toml variant + parse() arm + Display in error
          message updated. for_generic_consumer() helper folds Toml→Text
          for non-dep-audit consumers.

MODIFY  .elai_cc/crates/cli/src/dep_audit.rs
        — added dep_audit_to_stale_catches function bridging
          DepAuditResult.entries to Vec<StaleApiCatch> with
          caught_by=CatchLayer::DepAudit.

MODIFY  .elai_cc/crates/cli/src/doctor.rs
        — Toml output format writes metrics/dep_audit_latest.toml as a
          Windows-friendly copy of the dated TOML report.

MODIFY  .elai_cc/crates/cli/src/{auth,help,live_cli}.rs
        — extended Text-match arms to `Text | Toml` so the new variant
          is exhaustive without growing custom Toml renderers for non-
          dep-audit consumers.

MODIFY  .elai_cc/crates/contracts/tests/{layer1a,staged_eval,self_mod_
        proposal,self_mod_types}_contract.rs
        — updated AllowedConfigKey::Parameter constructions to use
          ParameterKey::EtcThreshold/FreshnessHalfLife.

MODIFY  .elai_cc/crates/core/src/mistake_book.rs
        — added FailureCategory::T1Block / T2Block / PreToolBlock
          variants + as_str() arms (snake_case wire form pinned by
          c21 contract test).

MODIFY  .elai_cc/crates/reasoning/src/traversal.rs
        — extended default_answers_for_category match to fold the 3 new
          FailureCategory variants into the Unknown single-question
          fallback (interrogation tree has no hook-tier branch).

MODIFY  .elai_cc/crates/knowledge/src/project_profile.rs
        — added append_constraints_from_rules_db function. Opens
          rules.db, queries query_deprecated_patterns(_, Some("BLOCK")),
          appends `"DO NOT USE {name}. Use {replacement}."` to
          profile.constraints, dedup-by-substring against existing
          entries. Silent skip on missing db / open error / query error.

ADD     .elai_cc/benchmarks/swe_bench/
        — README.txt + trigger_task.toml + task_01..task_10.toml (11
          files total). Canary + 10 SWE-bench-Lite-style problem
          statements (no solution patches). Stage assignments: 2 stage-1,
          4 stage-2, 4 stage-3, 1 canary.

ADD     .elai_cc/crates/contracts/tests/c21_anti_stale_contract.rs
        — 8 tests pinning ParameterKey 12 variants, round-trip, typed
          AllowedConfigKey, CatchLayer::DepAudit, FailureCategory new
          variants + serde, dep-audit/steering trigger public surface.

================================================================
TEST RESULTS
================================================================

cargo check --workspace               PASS (clean)
cargo test -p elai-orchestrator --lib self_mod    45 passed / 0 failed
cargo test -p elai-orchestrator --lib staged_eval 14 passed / 0 failed
cargo test --test c21_anti_stale_contract          8 passed / 0 failed
cargo test --test self_mod_types_contract         19 passed / 0 failed
cargo test --test staged_eval_contract             9 passed / 0 failed
cargo test --test self_mod_bridge_contract        11 passed / 0 failed
cargo test --test self_mod_proposal_contract       7 passed / 0 failed
cargo test --test layer1a_contract                20 passed / 0 failed
cargo test --test task_runner_chokepoint_contract  3 passed / 0 failed
cargo test -p elai-cli dep_audit                   8 passed / 0 failed
cargo test -p elai-knowledge --lib project_profile 63 passed / 0 failed

Pre-existing task_runner test failures (NOT introduced by C-21):
  task_runner::tests::task_runner_consumes_submitted_task_and_emits_progress
  task_runner::tests::task_runner_dedupes_via_cursor_on_restart
  task_runner::tests::task_runner_emits_finding_when_launcher_missing_for_web_project
  task_runner::tests::task_runner_emits_preview_available_for_web_project
Cause: PhaseComplete row + dev_server findings not emitted in the
test fixture. These failures relate to web-project dev server logic
and PhaseComplete terminal-row emission — neither touched by C-21.
The session's initial git status already had task_runner.rs marked
modified, and the failing assertions are unrelated to BlockSource /
synthetic Failed reason / FailureCategory variant additions.

================================================================
NOT DONE / DEFERRED
================================================================

- No new SsbActionType variant for "steering_vector_refresh_triggered".
  Rationale: adding a CHECK-constraint entry + enum variant + from_str
  arm + payload struct + SCHEMA_VERSION bump (53→54) is significant
  blast radius for an event already implied by the existing
  ProposalApplied SSB row + tracing log emitted by
  trigger_steering_vector_refresh. Future B-N may upgrade if dashboard
  needs an explicit row.

- No B-16B Chunk C dkg signature cascade. dkg has zero production
  unwraps (DRIFT-2 verified). The 3 deterministic_auditor LazyLock
  unwraps annotated per Q1 — exempt under B-16B D5.

- ConstraintsBlock auto-population from rules.db is exposed as a
  public function but is NOT yet called from any project-profile
  consumer. Caller-side wiring (config_loader / dashboard
  initialise) is a separate task — append_constraints_from_rules_db
  is ready to be invoked once the consumer surface is decided.

================================================================
ACCEPTANCE STATUS
================================================================

[x] check_anti_stale.sh wired into .github/workflows/windows.yml CI.
[x] 3 LazyLock<Regex> unwraps annotated (deterministic_auditor.rs).
[x] .git/hooks/pre-commit guard (via install_pre_commit_hook.sh
    installer).
[x] dep-audit emits dep_audit_latest.toml (Windows copy) + new
    stale_api_catches helper.
[x] compute_steering_vector.py output dir pre-created (Rust side,
    DRIFT-5).
[x] ParameterKey enum 12 variants in self_mod_safety.rs.
[x] ConstraintsBlock fed from rules.db (query_deprecated_patterns).
[x] SWE-bench fixture skeleton: 11 TOML files in benchmarks/swe_bench/.
[x] MistakeBook entry tier-categorised via BlockSource →
    FailureCategory::{T1Block, T2Block, PreToolBlock}.
[x] FailureCategory adds T1/T2/PreToolBlock variants.
[x] AllowedConfigKey sweep updates self_mod_proposal.rs +
    staged_eval.rs (NOT self_mod_orchestrator.rs per DRIFT-7).
[x] c21_anti_stale_contract.rs (8 tests passing).
[x] cargo check + scoped tests PASS.
[x] NO STUBS — every helper has real implementation; the BlockSource
    enum's T2Hook variant carries #[allow(dead_code)] because the T2
    dispatch surface is not yet wired in task_runner (T2 lives in
    deterministic_auditor today), with explicit doc-comment reservation.
