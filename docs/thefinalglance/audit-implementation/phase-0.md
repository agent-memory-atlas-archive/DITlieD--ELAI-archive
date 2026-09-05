# Phase 0 Implementation Audit
Date: 2026-04-24
Overall verdict: PARTIAL

## Workspace structure

File: F:\.ELAI_workflow\.elai_cc\Cargo.toml
Members (16, not 9): core, rules, providers, tools, plugins, knowledge, reasoning, orchestrator, cli, contracts, agent, ipc-types, web_search, scaffold, web_validator, dashboard
All 9 spec-required crates (core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli) + contracts crate exist with Cargo.toml.
Leaf check: crates/core/Cargo.toml has no orchestrator/plugins/cli deps (OK).
Extras (agent, ipc-types, web_search, scaffold, web_validator, dashboard) are Phase 1+ additions, not Phase 0 violations per se, but exceed the 9-crate contract stated in phase-0 EXIT CRITERIA.

## Sub-step: 0.1 baseline measurement
Verdict: IMPLEMENTED
Evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_0_baseline.toml exists. compile_time_s=0.17, total_rs_files=69, total_loc=61420, workspace_members=9 (old layout). F:\.ELAI_workflow\.elai_cc\migration_manifest.toml present with 79 KEEP/STRIP/REPLACE/EXTEND/STRIPPED entries (spec check was 69 .rs; 79>=69 satisfies coverage).
Caveat: test_pass_total=93 recorded (not the 684 referenced in risk register). Baseline under-measured — only plugins+tools counted.

## Sub-step: 0.2 strip unnecessary code
Verdict: PARTIAL
Evidence strip_delta.toml: files_removed=6, loc_removed=3478, deferred=[oauth.rs, compact.rs, usage.rs]. STRIP acceptance required "All STRIP-classified files removed" and "TokenUsage replaced with minimal shim".
Failures:
  - F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\compact.rs: 696 LOC STILL PRESENT (elai.map L1060-11955, 50+ refs). Spec said STRIP.
  - F:\.ELAI_workflow\.elai_cc\crates\core\src\usage.rs: 279 LOC STILL PRESENT with full ModelPricing/UsageCostEstimate/format_usd/pricing_for_model (L8 ModelPricing, L57 pricing_for_model). Spec said strip billing, keep only minimal TokenUsage{input_tokens,output_tokens}.
  - F:\.ELAI_workflow\.elai_cc\crates\providers\src\oauth.rs: PRESENT. Spec said STRIP (DEFERRED per manifest, but 0.2 acceptance required removal).
  - F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\remote.rs: PRESENT. Spec required EVALUATE/STRIP.
  - F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\prompt.rs: PRESENT (hardcoded prompts decision not documented as resolved).
Successful strips (verified gone): lsp_client.rs, summary_compression.rs, telemetry crate dir, compat-harness crate dir, mock-anthropic-service crate dir.

## Sub-step: 0.3 verify strip
Verdict: PARTIAL
Evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_0_strip_delta.toml exists, tests_broken_expected=0, tests_broken_bug=0, cargo_check_errors=0.
Caveat: deferred block explicitly admits oauth/compact/usage were NOT stripped per acceptance "All BUG-classified test failures resolved" only; the "all STRIP-classified files removed" invariant is violated.

## Sub-step: 0.4a restructure leaf crates (core, rules, providers)
Verdict: IMPLEMENTED
Evidence: crates/core, crates/rules, crates/providers all exist with Cargo.toml.
SandboxConfig location verified: F:\.ELAI_workflow\.elai_cc\crates\core\src\sandbox.rs L28 pub struct SandboxConfig, L9 FilesystemIsolationMode — correctly moved to core/ (R22 resolved).
providers/ has OllamaProvider (providers/src/providers/ollama.rs L360) and LlmProvider trait (ollama.rs L352). Old `api::` imports: 0 hits workspace-wide.
core/ Cargo.toml has no orchestrator/plugins/cli deps — true leaf.

## Sub-step: 0.4b restructure mid-layer (tools, plugins)
Verdict: PARTIAL
Evidence: crates/tools, crates/plugins, crates/knowledge, crates/reasoning all present. tools/Cargo.toml has no `runtime` dep.
Failures:
  - file_ops.rs, bash_validation.rs, stale_branch.rs were supposed to move runtime -> tools/. Instead they remain in orchestrator/ (F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\file_ops.rs, bash_validation.rs, stale_branch.rs) AND have copies in tools/ (F:\.ELAI_workflow\.elai_cc\crates\tools\src\bash_validation.rs). Module mapping violated — duplicated/wrong crate.

## Sub-step: 0.4c restructure top-layer (orchestrator, cli) + cleanup
Verdict: PARTIAL
Evidence: orchestrator/ holds conversation.rs (1726 LOC), session.rs, mcp_*.rs, fsm.rs — matches spec map. Old .elai_cc/rust/ is GONE (verified: test -d returned false).
No references to `use api::|use runtime::|use commands::|use rusty_claude` anywhere (grep = 0).
Failure: cli/src/main.rs is 11,181 LOC. Spec target: <500 LOC thin shell. Spec allowed documented debt but this exceeds target by 22x with no documented justification found. Per ADVERSARIAL clause: >1000 LOC flagged as technical debt — 11k LOC is massive technical debt.

## Sub-step: 0.5 verify restructure
Verdict: IMPLEMENTED
Evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_0_restructure.toml exists with compile_time_s=0.14, total_crates=9, test_pass=578, test_fail=40 (all 40 marked pre-existing). loc_per_crate populated. cargo_check clean.

## Sub-step: 0.6 contracts crate skeleton
Verdict: IMPLEMENTED
Evidence: F:\.ELAI_workflow\.elai_cc\crates\contracts\Cargo.toml present. All 5 required tests exist:
  - tests\provider_trait.rs (imports elai_providers)
  - tests\fsm_basic.rs (uses elai_tools registry, asserts IDLE initial state)
  - tests\tool_basic.rs
  - tests\config_basic.rs
  - tests\visibility_filter.rs (checks visibility_filtered error code)
All 5 import from elai_cc_* / elai_* crate names.
Note: Phase 0.6 expected "5 tests FAILED" (RED) — tests have since been implemented to pass (GREEN) in later phases.

## Sub-step: 0.7a implement provider + config
Verdict: IMPLEMENTED
Evidence: LlmProvider trait at providers/src/providers/ollama.rs L352. OllamaProvider L360 + OllamaProviderBuilder L366 with builder-configurable base_url. config_basic test present.

## Sub-step: 0.7b implement FSM + tool
Verdict: PARTIAL
Evidence: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\fsm.rs L24 FsmState enum, L332 Fsm struct. fsm_basic + tool_basic contract tests present.
Failure: msgpack session storage NOT implemented. grep for rmp_serde/msgpack in session.rs returned 0. Session persistence still uses serde_json (custom JsonValue). Spec 0.7b explicitly required "Replace JSON session storage with msgpack". Postcard is used for IPC wire protocol only (ipc-types crate), not for disk session storage.

## Sub-step: 0.7c implement interface hardening
Verdict: UNVERIFIED
Evidence: visibility_filter.rs contract test present. slash-command routing exists in cli/ (main.rs is 11k LOC, contains dispatch). No explicit verification of allowlist-based sanitization performed.

## Sub-step: 0.8 verify implementation
Verdict: PARTIAL
Evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_0_implement.toml exists (compile_time_s=24.17, test_total=625, test_pass=585, loc_delta=498).
Failure: F:\.ELAI_workflow\.elai_cc\scripts\smoke_test_ollama.sh MISSING. Only scripts/start-searxng.sh present.

## Sub-step: 0.9 entropy quarantine
Verdict: PARTIAL
Evidence:
  - F:\.ELAI_workflow\.elai_cc\crates\core\src\clock.rs L33 trait ClockProvider, L45 struct OsClock, L76 struct SeededClock. Only Instant::now() / SystemTime::now() are in clock.rs (L52, L67) — the allowed exception site.
  - F:\.ELAI_workflow\.elai_cc\crates\core\src\entropy.rs L7 trait EntropySource, L36 struct SeededRng.
  - F:\.ELAI_workflow\.elai_cc\crates\core\src\lib.rs L102 struct RuntimeContext<C,E>, L117 ProductionContext, L119 TestContext. Static-dispatch (not Arc<Mutex<>>), avoiding contention.
Failures (QUARANTINE LEAKING):
  - 40 raw Instant::now()/SystemTime::now() calls across 24 files in orchestrator/ alone (task_runner.rs:8, pipeline_tree_navigator.rs:4, session_control.rs:3, checkpoint.rs:2, indexing_service.rs:2, +19 others at 1 each). Spec 0.9 deterministic check: "hits ONLY in core/src/clock.rs".
  - Additional leaks in cli/main.rs, tools/lib.rs, tools/dispatcher.rs, providers, rules/rule_engine.rs, plugins, knowledge, web_validator, agent, dashboard.
  - thread_rng/OsRng::new/getrandom in rules/schema.rs, orchestrator/sandbox.rs, providers/oauth.rs (3 files beyond core).
  - F:\.ELAI_workflow\.elai_cc\scripts\check_entropy_quarantine.sh MISSING. CI enforcement not present.
Spec ACCEPTANCE explicitly required "Raw time/entropy ONLY in core/src/clock.rs and core/src/entropy.rs (plus TEST_KEEP)" — violated on 40+ sites.

## Sub-step: 0.10 determinism proof
Verdict: PARTIAL
Evidence: F:\.ELAI_workflow\.elai_cc\metrics\phase_0_entropy.toml records quarantine_check="PASS" for the tests that pass, contract_tests_passed=14, seeded_determinism tests pass (3/3), clock_entropy_contracts (14/14) pass.
Failures (per metrics file's own audit section):
  - determinism_proof_script = "FAIL — set -euo pipefail causes script to exit at Step 2".
  - determinism_diff_lines = -1 (could not run).
  - timestamp_violations = -1 (could not run).
  - 4 WARNs from cycle-2 audit: determinism_proof.sh structural flaw; SeededClock not seeded by ELAI_SEED value; LogicalInstant cross-provider Ord hazard; OS_CLOCK_TICK global shared across instances.
  - scripts/determinism_proof.sh MISSING at expected path.

## Module mapping check
Verdict: PARTIAL
69 .rs files were supposed to be mapped. Migration manifest has 79 entries (covers all + extras). Several files landed in wrong crates relative to the spec mapping:
  - file_ops.rs, bash_validation.rs, stale_branch.rs, branch_lock.rs: spec said tools/, actually in orchestrator/ (some duplicated in tools/).
  - session_control.rs: spec said SPLIT (identity -> core/, state -> orchestrator/). Currently entirely in orchestrator/ (session_control.rs present there).
Final file count: 378 .rs under .elai_cc/crates (5.5x the 69 baseline due to Phase 1+ additions).

## Entropy quarantine check
Violations:
  - SystemTime::now() / Instant::now() outside core/clock.rs: 40+ sites across orchestrator, cli, tools, providers, plugins, rules, knowledge, web_validator, agent, dashboard crates (`Grep` returned 40 files limit).
  - thread_rng / OsRng::new / getrandom outside core/entropy.rs: rules/src/schema.rs, orchestrator/src/sandbox.rs, providers/src/oauth.rs.
  - CI script scripts/check_entropy_quarantine.sh MISSING.
Verdict: FAIL spec acceptance. Quarantine exists as traits but is not enforced.

## HashMap ban check
use std::collections::HashMap: 84 occurrences.
use std::collections::HashSet: 10 occurrences.
Verdict: Spec "HashMap ban: std HashMap/HashSet replaced with BTreeMap/IndexMap/FxHashMap" is NOT enforced. 94 std HashMap/HashSet imports remain. No migration evidence.

## Stripped code check
Still present (should be gone or minimal per 0.2 acceptance):
  - crates/orchestrator/src/compact.rs (696 LOC, full claw-code compaction logic + tests).
  - crates/core/src/usage.rs (279 LOC with ModelPricing, UsageCostEstimate, pricing_for_model, format_usd — spec said strip billing, keep only minimal TokenUsage struct).
  - crates/providers/src/oauth.rs (OAuth code; spec said STRIP, DEFERRED in manifest but never removed).
  - crates/orchestrator/src/remote.rs (spec required EVALUATE/STRIP).
  - crates/orchestrator/src/prompt.rs (hardcoded prompts resolution not documented).
Correctly stripped: telemetry crate, compat-harness crate, mock-anthropic-service crate, lsp_client.rs, summary_compression.rs. Old .elai_cc/rust/ tree gone.

## Summary
- Implemented: 4 / 15 (0.1, 0.4a, 0.5, 0.6, 0.7a)
- Partial: 9 (0.2, 0.3, 0.4b, 0.4c, 0.7b, 0.8, 0.9, 0.10, module mapping)
- Missing: 0 (0.7c listed UNVERIFIED rather than missing)
- Dead/unstripped: compact.rs (696 LOC), usage.rs billing types (~250 LOC of 279), oauth.rs, remote.rs, prompt.rs — ~1700+ LOC of code the spec required removed.
- Confidence: HIGH (direct grep of elai.map, src files, Cargo.toml, metrics TOMLs)
- Critical gaps:
  1. Entropy quarantine leaks on 40+ sites; scripts/check_entropy_quarantine.sh missing; 0.9 acceptance violated.
  2. HashMap ban not enforced (94 std imports).
  3. compact.rs, usage.rs billing code, oauth.rs NOT stripped as required by 0.2.
  4. cli/main.rs is 11,181 LOC vs <500 LOC target — 22x over budget.
  5. Session persistence still JSON-based; msgpack per 0.7b not done.
  6. scripts/check_entropy_quarantine.sh, scripts/determinism_proof.sh, scripts/smoke_test_ollama.sh all MISSING.
  7. Workspace has 16 members, not the 9 the EXIT CRITERIA state (extras are Phase 1+, but Phase 0 exit was never re-validated against the 9-crate invariant).
  8. file_ops/bash_validation/stale_branch landed in orchestrator/ instead of tools/ per module mapping; some duplicated across crates.
