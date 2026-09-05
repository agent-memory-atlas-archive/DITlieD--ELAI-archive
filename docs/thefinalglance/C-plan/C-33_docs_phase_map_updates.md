# C-33: Documentation and Phase Map Updates
Objective: create 4 missing step files, update 2 existing step files, add B-37/38/40/41/42
status blocks to B-0_INDEX.md, add 3 FailureCategory variants to failure_taxonomy.rs,
reconcile B-39 REFINE_CAP and B-21 SSB action type names, and add per-command KEEP/STRIP
breakdown for 28+ slash commands to migration_manifest.toml.
Estimated effort: 1-2 sessions.
Pre-requisites: All other C-N files exist. C-30 (SurgicalDecomposer — implementation
of phase-4D-SURGICAL delivered there, this C-33 writes only the spec file).
Parallel-safe with: ALL other C-N (pure docs/spec; the only code touch is
failure_taxonomy.rs which no other C-N modifies).

NOTE: docs/specs/ is gitignored. The four new step files and all spec-file edits
are local-only. They are not committed to git. Flag this at session end.

## Context

update-phase-map.report.md (2026-05-17, status=PARTIAL) identifies:
- 4 step files that do not exist: phase-3L.txt, phase-3J-BASELINE.txt,
  phase-4D-SURGICAL.txt, phase-4D-ORACLE.txt.
- phase-3J.txt and phase-4G.txt each lack spec content for implementations
  that already shipped in code.
- B-0_INDEX.md last updated oracle-gate patch (2026-05-11); no status blocks
  for B-37, B-38, B-40, B-41, B-42; SCHEMA_VERSION reference is "26 confirmed"
  vs actual 46 in ssb.rs:184.

B-39_chat_reflect_pipeline.report.md (2026-05-17): REFINE_CAP is 3 in code
(chat_reflect.rs:48) and the contract test (chat_reflect_contract.rs:503-511
asserts == 3 citing RefineBench t=2->t=3 +12-20pp). The B-39 plan says cap=2.
Resolution: cap=3 is correct per implementation evidence; update the plan.

B-21_phase_4F_feasibility_probe.report.md (2026-05-17): SSB action type names
in B-21 WIRING_CONTRACT do not match the shipped code. Plan claims:
  feasibility_probe_started, feasibility_probe_complete, alternative_explored,
  feasibility_escalated.
Code ships: system_probe, feasibility_report, novel_step_detected, stub_run
(ssb.rs:531-540). Resolution: update B-21 WIRING_CONTRACT to match code names.

B-40_idea_framer_research_presets.report.md (2026-05-17): plan spec still carries
arXiv:2407.21787 (wrong paper); code substitutes arXiv:2409.04109 (Si et al. 2024).
Address as part of B-0_INDEX notes.

failure_taxonomy.rs:18-28 has 9-variant FailureCategory enum; LAZY_CODE,
EMPTY_BODY, FILE_SIZE_EXCEEDED are absent. LAZY_CODE_SCAN in
deterministic_auditor.rs:11 references the concept but the enum cannot classify
these failure types without the variants.

migration_manifest.toml (crates/commands/src/lib.rs entry at line 124) says
"28+ slash commands. Audit in 0.2g classifies KEEP/STRIP subset." but provides
no per-command breakdown. The per-command table must be explicit.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\update-phase-map.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-0_INDEX.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-39_chat_reflect_pipeline.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-40_idea_framer_research_presets.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-21_phase_4F_feasibility_probe.report.md
- .elai_cc/crates/tools/src/failure_taxonomy.rs (FailureCategory enum, lines 18-28)
- .elai_cc/crates/reasoning/src/chat_reflect.rs (REFINE_CAP, line 48)
- .elai_cc/crates/orchestrator/src/ssb.rs (B-21 action types, lines 531-540)
- .elai_cc/migration_manifest.toml (commands entry, lines 122-128)
- .elai_cc/crates/cli/src/commands.rs (slash_command_specs, line 4464)
- docs/thefinalglance/B-plan/B-0_INDEX.md
- docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md
- docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md

## Tasks

1. Create docs/specs/guide/steps/phase-3L.txt — 8-step Project Intelligence spec.
   Contents must cover the 8 implementation steps visible in conventions.rs,
   project_rule_seeder.rs, indexing_service.rs, and telescope.rs:
   Step 1: skeleton — ConventionStats / NamingConventions struct definitions,
     extract_conventions() public API, 4 SQL queries (naming, import, error-handling,
     commenting patterns), regex compilation at module init.
   Step 2: verify — conventions schema in DKG node metadata; migration from
     existing conventions.db schema if prior schema present.
   Step 3: tests — unit tests for extract_conventions() covering >=4 languages
     (Rust, Python, TypeScript, Svelte); deterministic output on identical input.
   Step 4: Tier1 — T1Hook loads conventions from rules.db; inserts CONVENTIONS
     block into agent prompt SYSTEM section via ConventionStats summary.
   Step 5: Tier2 — T2Hook with AST import extraction; feeds ProjectRuleSeeder to
     auto-generate project rules in rules.db; VERSION_GATE predicate wired.
   Step 6: Tier3 — ProjectProfile aggregation of ConventionStats, FreshnessConfig,
     ProjectClass; stored in knowledge/project_profile.rs.
   Step 7: telescope — context telescope integration (pi_telescope_contract,
     pi_tier3); priority ordering of convention signals by token cost vs recall.
   Step 8: measure — calibration script; convention extraction p50/p99 latency
     target <=50ms; metrics written to metrics/phase_3L.toml.
   Spec line count target: 80-120 lines.

2. Create docs/specs/guide/steps/phase-3J-BASELINE.txt — Token Efficiency
   benchmark harness spec.
   Contents:
   - Harness binary: benches/model_capability/run_benchmark.py; inputs: task
     fixture TOML with >=10 prompts per ModelCapabilityProfile tier.
   - 7 measurements per fixture run: total_tokens_in, total_tokens_out,
     slot_utilization_pct, compression_ratio (ShortCoder), ttft_ms (time-to-first
     token), p99_latency_ms, cost_per_run_usd.
   - ModelCapabilityProfile generation: bench runner reads elai_cc.toml
     [model_capability] section; emits profile to metrics/capability_profile.toml.
   - Baseline targets: micro_7b_12gb: slot_utilization >= 0.82,
     compression_ratio >= 0.65; cloud_200k: slot_utilization >= 0.90.
   - REPAIR_CONTEXT regression gate: bench run after repair must not increase
     total_tokens_in by >15% vs non-repair baseline on same fixture.
   - Constrained decoding gate: JSON schema fixture (20 prompts) must produce
     0 invalid-JSON outputs with ConstrainedDecodingConfig active.
   - Standalone binary requirement: run_benchmark.py must run without elai
     runtime (no SSB, no DKG) — reads fixtures, writes TOML results only.
   Spec line count target: 60-90 lines.

3. Create docs/specs/guide/steps/phase-4D-SURGICAL.txt — SurgicalDecomposer
   spec (implementation delivered in C-30; this file is the spec record).
   Cross-reference C-30 and the shipped implementation:
   - SurgicalPlan, AffectedSet, ComplexityClass, RegressionGate, TestBaseline,
     SurgicalStep types (as defined in surgical_decomposer.rs per C-30).
   - SurgicalDecomposer::decompose() steps: DKG blast-radius traversal,
     complexity classification thresholds (Simple: files<=3 AND loc<=200;
     Medium: files<=8 OR loc<=500; Complex: otherwise), constrained step
     generation (Simple=1, Medium=4, Complex=8 step cap), RegressionGate
     injection per step from DKG test-coverage metadata.
   - LOC threshold gate: >500 LOC emits SSB surgical_loc_warning; INTERACTIVE
     mode returns ConfirmationRequired; AUTONOMOUS >1000 LOC escalates.
   - CLI: --surgical flag captured by parse_task_flags (cli/src/commands.rs:2006).
   - SSB action types: surgical_plan_created (complexity_class, blast_radius_files),
     surgical_loc_warning (loc_estimate, threshold).
   - DETERMINISTIC_CHECK: rg "SurgicalDecomposer|SurgicalPlan" .elai_cc/crates.
   Spec line count target: 80-120 lines.

4. Create docs/specs/guide/steps/phase-4D-ORACLE.txt — Oracle Mode /research
   pipeline spec (implementation delivered in C-30; this file is the spec record).
   Cross-reference C-30 and the shipped implementation:
   - /research command entry point: live_cli.rs at run_research_pipeline call site.
   - OracleAnswerCache: CacheKey=(query_hash:[u8;32], project_hash:[u8;32]) both
     blake3; CachedAnswer { answer, cached_at, source_tier }.
   - TTL by SourceTier: T1Authoritative=2592000s, T2Strong=604800s, T3Weak=86400s,
     T4Rejected=0 (never cache).
   - Cache miss: call run_research_pipeline, determine tier from answer_confidence
     (HIGH->T1, MEDIUM->T2, LOW->T3), insert, emit oracle_cache_miss SSB row.
   - Cache hit: return cached answer, emit oracle_cache_hit SSB row (cached_age_secs,
     source_tier).
   - Eviction: LRU when cache len() > max_entries (elai_cc.toml configurable).
   - SSB action types: oracle_cache_hit, oracle_cache_miss.
   - DETERMINISTIC_CHECK: rg "OracleAnswerCache" .elai_cc/crates.
   Spec line count target: 70-100 lines.

5. Update docs/specs/guide/steps/phase-3J.txt — add block for shipped items.
   Add section "3J.ADDENDUM — Shipped extensions (post-3J-BASELINE)" covering:
   - ETCLogitsProcessor (A-9): server.py:956-1488; halts generation at entropy
     threshold; calibration: metrics/calibrate_etc_threshold.py.
   - GG signals (A-17/A-25): mean_logprob and entropy_mean in sidecar_client.rs
     :149-160 and server.py:1043-1086; consumed by research_pipeline confidence.
   - Constrained decoding (A-10): ConstrainedDecodingConfig in providers/src/
     runtime/vllm.rs; grammar-guided and JSON schema modes.
   - Tool routing (A-26): routing_rules.rs in crates/tools/.
   - Bash safety filter (A-27): safety_filter.rs in crates/tools/.
   - git_commit tool (A-28): git_commit.rs in crates/tools/.
   - Action classifier (A-31): action_classifier.rs in crates/tools/.
   - Steering vector batch mode (A-8): compute_steering_vector.py --batch flag
     (metrics/compute_steering_vector.py:228); contrast pairs in
     data/steering_contrast_pairs.toml.
   Do not remove or restructure existing 3J content; append only.

6. Update docs/specs/guide/steps/phase-4G.txt — add AutoAdvanceEvaluator and
   CloudSpendTracker block.
   Add section "4G.ADDENDUM — Shipped extensions" covering:
   - AutoAdvanceEvaluator reasoning exhaustion check: derive_framing_evidence
     (task_runner.rs:7162) as 7th evaluator; exhaustion detected when
     AutoAdvanceStats.evidence_rounds >= evidence_rounds_max.
   - CloudSpendTracker escalation policy: cloud_spend_ssb_contract.rs verifies
     thresholds; policy: at 50% of session budget emit cloud_spend_warning SSB;
     at 20% remaining emit cloud_spend_critical; below 20% escalate to INTERACTIVE
     or halt AUTONOMOUS mode. Budget threshold sourced from elai_cc.toml
     [cloud.session_budget_usd].
   Do not remove existing 4G content; append only.

7. Add LAZY_CODE, EMPTY_BODY, FILE_SIZE_EXCEEDED variants to FailureCategory
   enum in .elai_cc/crates/tools/src/failure_taxonomy.rs.
   Current enum (lines 18-28) has 9 variants ending at RuntimePanic.
   Add after RuntimePanic:
     LazyCode,       // placeholder return, TODO body, pass body, mock data in prod
     EmptyBody,      // function/impl block with no statements
     FileSizeExceeded, // file exceeds configured max-lines gate
   Update detect_category() match arms to detect these from output patterns:
   - LazyCode: regex r"TODO|placeholder|unimplemented!|todo!()" in output.
   - EmptyBody: regex r"empty function body|no statements" in output.
   - FileSizeExceeded: regex r"file exceeds|FILE_SIZE" in output.
   Update suggest_fix() match arms for all 3 new variants.
   Update the doc comment on line 11 to reflect 12 variants total.
   NOTE: ToolFailureCategory in elai_core must be kept in sync — grep for
   "ToolFailureCategory" in elai.map and add the same 3 variants there.

8. Reconcile B-39 REFINE_CAP: update B-39 plan to reflect cap=3.
   In docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md:
   - Find the REFINE_CAP=2 claim (plan line mentioning RefineBench cap of 2).
   - Replace with: REFINE_CAP = 3 — RefineBench (arXiv:2511.22173) Table 3
     guided-mode t=2->t=3 yields +12-20pp; implementation shipped cap=3;
     contract test at chat_reflect_contract.rs:503-511 asserts == 3.
   Verify chat_reflect.rs:48 reads `pub const REFINE_CAP: u32 = 3` before editing.

9. Reconcile B-21 SSB action type names: update B-21 WIRING_CONTRACT to match code.
   In docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md,
   WIRING_CONTRACT produces_symbols action_types section:
   Replace:
     feasibility_probe_started -> system_probe
     feasibility_probe_complete -> feasibility_report
     alternative_explored -> stub_run (nearest shipped equivalent)
     feasibility_escalated -> (absent; note as DEFERRED)
   Add WIRING_DIVERGENCE note documenting that novel_step_detected matches plan;
   feasibility_escalated is absent from ssb.rs and deferred to C-22.
   Verify against ssb.rs:531-540 before editing.

10. Update docs/thefinalglance/B-plan/B-0_INDEX.md:
    Add status blocks for B-37, B-38, B-40, B-41, B-42 after the oracle-gate
    patch section. Each block format mirrors existing B-N entries:
      B-37 DONE — commit fd5c2e5. RuntimeAdapter trait + 9 adapters + PerRoleRuntime
        + RuntimeOrchestrator + /runtime CLI + RuntimePanel. SCHEMA_VERSION 46
        (final bump includes B-37C). Evidence: per_role_runtime.rs:169, runtime_
        orchestrator.rs, state.rs:488.
      B-38 — status: confirm from MEMORY.md (B-37 shipped block notes B-38 as
        "Ambiguity-score gate retrofit — small standalone"); add SHIPPED or
        PARTIAL pending auditor confirmation.
      B-40 DONE — commit logged in MEMORY.md. idea_framing.rs 1358 LOC,
        FsmSubstate::Framing, 5 SSB action types, IdeaFramingWindow.svelte,
        51 tests. SCHEMA_VERSION bump 40->41. arXiv citation corrected to
        arXiv:2409.04109 in code (plan still carries wrong ID — doc debt only).
      B-41 DONE — per-level auditor modes. auditor_mode + per_level_audit_dispatch
        + AgentRole+5 + Evidence+5 + AuditTimelineWindow. SCHEMA_VERSION 45.
        80 GREEN tests. navigator-spawn deferred.
      B-42 DONE — Cosmos Artifact Tier. cosmos_worker ~700 LOC, briefing compiler,
        VerificationMethod::CosmosCheck, SCHEMA_VERSION 44, CosmosDrawer.
        spawn_loop dispatcher deferred.
    Replace "SCHEMA_VERSION=26 confirmed" reference with:
        SCHEMA_VERSION=46 current (ssb.rs:184). Progression: B-6 bumped to 26;
        B-24 to 30; B-34 to 35; B-38/42/41 to 44/45/46. See ssb.rs:120-130
        inline comments for full history.

11. Add per-command KEEP/STRIP breakdown to migration_manifest.toml.
    Under the existing commands entry (lines 122-128), replace the "28+ slash
    commands" rationale line with a sub-table listing all 28+ slash commands.
    Format per command:
      [[file.commands]]
      name = "/task"
      action = "KEEP"
      target = "cli/src/commands.rs"
      rationale = "core task submission"
    Commands verified in cli/src/commands.rs (slash_command_specs assert 149):
    KEEP: /task, /research, /mode, /model, /runtime, /reason, /plan, /memory,
          /context, /search, /audit, /help, /status, /log, /export, /import,
          /reset, /history, /session, /config, /budget, /branch, /checkpoint,
          /supervise, /agents, /tools, /version, /health.
    STRIP: /debug (dev-only), /trace (dev-only), /benchmark (absorbed into
           run_benchmark.py standalone binary), /mock (test harness only).
    Flag /resume and /cancel as KEEP-pending (verify dispatch in live_cli.rs
    before finalising).
    Read crates/cli/src/commands.rs slash_command_specs() before writing to
    confirm exact command list; do not fabricate command names.

## WIRING_CONTRACT

produces_symbols:
  - docs/specs/guide/steps/phase-3L.txt (new)
  - docs/specs/guide/steps/phase-3J-BASELINE.txt (new)
  - docs/specs/guide/steps/phase-4D-SURGICAL.txt (new)
  - docs/specs/guide/steps/phase-4D-ORACLE.txt (new)
  - docs/specs/guide/steps/phase-3J.txt (3J.ADDENDUM block appended)
  - docs/specs/guide/steps/phase-4G.txt (4G.ADDENDUM block appended)
  - FailureCategory::LazyCode, FailureCategory::EmptyBody,
    FailureCategory::FileSizeExceeded (3 new variants)
  - detect_category() and suggest_fix() arms for 3 new variants
  - B-39 plan: REFINE_CAP corrected to 3
  - B-21 plan: WIRING_CONTRACT action type names aligned to code
  - B-0_INDEX.md: status blocks for B-37/38/40/41/42, SCHEMA_VERSION=46
  - migration_manifest.toml: per-command [[file.commands]] sub-table
consumes_symbols:
  - chat_reflect.rs:48 (REFINE_CAP = 3)
  - ssb.rs:531-540 (B-21 actual action type names)
  - ssb.rs:184 (SCHEMA_VERSION = 46)
  - failure_taxonomy.rs:18-28 (current 9-variant enum)
  - commands.rs slash_command_specs() (exact command list for manifest)
reachable_from:
  - spec_link_check.py against all 4 new step files
  - wiring_contract_check.py against B-21 and B-39 plan files
  - B-0_INDEX.md grep for B-37/38/40/41/42 and SCHEMA_VERSION
stub_ban:
  - Do NOT invent command names for the manifest. Read commands.rs first.
  - Do NOT modify any phase spec files (1A, 1E, 3A, 3C, 3E, 3F, 4B, 4C, 4E,
    4H) — those belong to separate C-N tasks not scoped here.
  - Do NOT touch ToolFailureCategory without grepping elai.map first.
observable_effect:
  trigger: spec_link_check.py --file docs/specs/guide/steps/phase-3L.txt
  expect: exit 0 (all Spec references: entries map to real content)
  trigger: wiring_contract_check.py --bn-file docs/thefinalglance/B-plan/
           B-21_phase_4F_feasibility_probe.md
  expect: no WIRING_DIVERGENCE for SSB action type names

## DETERMINISTIC_CHECK

```bash
# 1. New step files exist
for f in phase-3L phase-3J-BASELINE phase-4D-SURGICAL phase-4D-ORACLE; do
  test -f "docs/specs/guide/steps/$f.txt" && echo "OK: $f" || echo "MISSING: $f"
done

# 2. FailureCategory has 12 variants
grep -c "^\s*[A-Z][a-zA-Z]*,$\|^\s*[A-Z][a-zA-Z]*$" \
  .elai_cc/crates/tools/src/failure_taxonomy.rs

# 3. New variants present
grep -E "LazyCode|EmptyBody|FileSizeExceeded" \
  .elai_cc/crates/tools/src/failure_taxonomy.rs

# 4. B-0_INDEX has status blocks for shipped B-plans
grep -c "B-37\|B-40\|B-41\|B-42" \
  docs/thefinalglance/B-plan/B-0_INDEX.md

# 5. SCHEMA_VERSION=46 in B-0_INDEX
grep "SCHEMA_VERSION=46\|SCHEMA_VERSION 46" \
  docs/thefinalglance/B-plan/B-0_INDEX.md

# 6. REFINE_CAP=3 in B-39 plan
grep "REFINE_CAP.*3\|cap=3\|cap = 3" \
  docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md

# 7. B-21 WIRING_CONTRACT uses code names
grep "system_probe\|feasibility_report" \
  docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md

# 8. migration_manifest has per-command entries
grep -c "name = \"/task\"\|name = \"/research\"" \
  .elai_cc/migration_manifest.toml

# 9. spec_link_check passes on phase-3L
python .claude/tools/spec_link_check.py \
  --file docs/specs/guide/steps/phase-3L.txt

# 10. spec_link_check passes on phase-3J after update
python .claude/tools/spec_link_check.py \
  --file docs/specs/guide/steps/phase-3J.txt

# 11. wiring_contract_check: B-21 no SSB-name WIRING_DIVERGENCE
python .claude/tools/wiring_contract_check.py \
  --bn-file docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md

# 12. wiring_contract_check: B-39 REFINE_CAP matches code
python .claude/tools/wiring_contract_check.py \
  --bn-file docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md
```

## ACCEPTANCE

- [ ] phase-3L.txt exists; spec_link_check.py exits 0
- [ ] phase-3J-BASELINE.txt exists; spec_link_check.py exits 0
- [ ] phase-4D-SURGICAL.txt exists (cross-references C-30 types correctly)
- [ ] phase-4D-ORACLE.txt exists (cross-references C-30 OracleAnswerCache)
- [ ] phase-3J.txt: grep for ETCLogitsProcessor and mean_logprob returns hits
- [ ] phase-4G.txt: grep for AutoAdvanceEvaluator and cloud_spend_warning returns hits
- [ ] FailureCategory enum has 12 variants (9 original + LazyCode + EmptyBody
      + FileSizeExceeded); cargo check -p elai-tools passes
- [ ] detect_category() and suggest_fix() have arms for all 12 variants
- [ ] B-39 plan: REFINE_CAP=3 present; wiring_contract_check exits 0
- [ ] B-21 plan: WIRING_CONTRACT lists system_probe, feasibility_report, stub_run,
      novel_step_detected; wiring_contract_check exits 0 for action type names
- [ ] B-0_INDEX.md: blocks for B-37, B-38, B-40, B-41, B-42 present;
      SCHEMA_VERSION=46 reference present; grep count >= 5 for those B-numbers
- [ ] migration_manifest.toml: [[file.commands]] sub-entries for >=10 named commands;
      KEEP/STRIP action on each; no invented command names (verified vs commands.rs)
- [ ] docs/specs/ gitignore confirmed: new step files are local-only; flagged in
      session handoff
- [ ] Optional: elai /research --mode cloud "what does phase 3L do?" returns answer
      that references phase-3L.txt content (proves discoverable via cloud-Sonnet)

## Files touched

- CREATE: docs/specs/guide/steps/phase-3L.txt (local-only, gitignored)
- CREATE: docs/specs/guide/steps/phase-3J-BASELINE.txt (local-only, gitignored)
- CREATE: docs/specs/guide/steps/phase-4D-SURGICAL.txt (local-only, gitignored)
- CREATE: docs/specs/guide/steps/phase-4D-ORACLE.txt (local-only, gitignored)
- MODIFY: docs/specs/guide/steps/phase-3J.txt (append 3J.ADDENDUM, local-only)
- MODIFY: docs/specs/guide/steps/phase-4G.txt (append 4G.ADDENDUM, local-only)
- MODIFY: .elai_cc/crates/tools/src/failure_taxonomy.rs (3 new enum variants +
          detect/suggest arms)
- MODIFY: docs/thefinalglance/B-plan/B-39_chat_reflect_pipeline.md (REFINE_CAP)
- MODIFY: docs/thefinalglance/B-plan/B-21_phase_4F_feasibility_probe.md
          (WIRING_CONTRACT action type names)
- MODIFY: docs/thefinalglance/B-plan/B-0_INDEX.md (5 status blocks + SCHEMA_VERSION)
- MODIFY: .elai_cc/migration_manifest.toml ([[file.commands]] sub-table)
- CREATE: workflow/reasoning/C-33-decisions.md

## Next

C-34 or next outstanding C-N per bn_status_report.py --filter todo output.
After all C-N complete: final FSM transition MEMORIZING -> COMPLETE -> IDLE.
