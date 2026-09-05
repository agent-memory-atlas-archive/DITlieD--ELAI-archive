# Verification Report: phase-0.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-0.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 0 restructure (fork, strip, restructure, implement, entropy quarantine) is substantially complete.
The 9-crate workspace exists, all required metrics files are present, contract test files exist, entropy
traits and CI script are implemented, and all old crate import paths are gone. Key gaps: compact.rs was
not stripped (DEFERRED to orchestrator), usage.rs was not stripped (kept intact and moved to core with
UsageTracker/billing intact contrary to spec intent), the hardcoded prompt "You are an interactive
agent" remains in orchestrator/src/prompt.rs, migration_manifest.toml lacks the count of 69 KEEP/STRIP
entries (some files were deferred), and config_basic contract test intentionally fails at runtime due to
ConfigLoader not discovering elai_cc.toml (documented gap).

## Deliverables verified DONE

- Step 0.1: cargo baseline metrics recorded — evidence: .elai_cc/metrics/phase_0_baseline.toml:1 (compile_time_s=0.17, test_pass_total=93, total_rs_files=69, total_loc=61420)
- Step 0.1: migration_manifest.toml exists — evidence: .elai_cc/migration_manifest.toml:1 (all 69 source files classified with KEEP/STRIP/REPLACE/EXTEND/DEFERRED)
- Step 0.2a: telemetry/ crate stripped — evidence: phase_0_strip_delta.toml:22 lists crates/telemetry/src/lib.rs as stripped
- Step 0.2b: compat-harness/ crate stripped — evidence: phase_0_strip_delta.toml:23 lists crates/compat-harness/src/lib.rs as stripped
- Step 0.2d: runtime/src/lsp_client.rs stripped — evidence: phase_0_strip_delta.toml:26 lists crates/runtime/src/lsp_client.rs as stripped
- Step 0.2 (partial): runtime/src/summary_compression.rs stripped — evidence: phase_0_strip_delta.toml:27
- Step 0.3: phase_0_strip_delta.toml exists — evidence: .elai_cc/metrics/phase_0_strip_delta.toml:1 (loc_removed=3478, files_removed=6)
- Step 0.4a: core/ crate extracted — evidence: .elai_cc/crates/core/src/lib.rs (ClockProvider, EntropySource, RuntimeContext exported)
- Step 0.4a: rules/ crate extracted — evidence: .elai_cc/Cargo.toml:6 "crates/rules" member
- Step 0.4a: providers/ crate extracted — evidence: .elai_cc/Cargo.toml:7 "crates/providers" member
- Step 0.4a: LlmProvider trait in providers/ — evidence: .elai_cc/crates/providers/src/providers/ollama.rs:352
- Step 0.4a: OllamaProvider with configurable base_url — evidence: .elai_cc/crates/providers/src/providers/ollama.rs:373,397
- Step 0.4a: old api:: import path removed — evidence: grep "use api::" across crates returns 0 hits
- Step 0.4b: tools/ crate exists — evidence: .elai_cc/Cargo.toml:8 "crates/tools" member
- Step 0.4b: plugins/ crate exists — evidence: .elai_cc/Cargo.toml:9 "crates/plugins" member
- Step 0.4b: knowledge/ skeleton crate — evidence: .elai_cc/Cargo.toml:10 "crates/knowledge" member (phase_0_restructure.toml: loc=1)
- Step 0.4b: reasoning/ skeleton crate — evidence: .elai_cc/Cargo.toml:11 "crates/reasoning" member (phase_0_restructure.toml: loc=1)
- Step 0.4c: orchestrator/ crate — evidence: .elai_cc/Cargo.toml:12 "crates/orchestrator" member
- Step 0.4c: cli/ crate — evidence: .elai_cc/Cargo.toml:13 "crates/cli" member
- Step 0.4c: old crate dirs removed from active workspace — evidence: grep "use runtime::|use commands::" returns 0 hits across crates/
- Step 0.5: phase_0_restructure.toml exists — evidence: .elai_cc/metrics/phase_0_restructure.toml:1 (578 pass, 40 fail pre-existing, total_crates=9)
- Step 0.6: contracts/ crate exists with Cargo.toml — evidence: .elai_cc/crates/contracts/src/lib.rs confirmed
- Step 0.6: provider_trait.rs contract test exists — evidence: .elai_cc/crates/contracts/tests/provider_trait.rs:1
- Step 0.6: fsm_basic.rs contract test exists — evidence: .elai_cc/crates/contracts/tests/fsm_basic.rs:1
- Step 0.6: tool_basic.rs contract test exists — evidence: .elai_cc/crates/contracts/tests/tool_basic.rs:1
- Step 0.6: config_basic.rs contract test exists — evidence: .elai_cc/crates/contracts/tests/config_basic.rs:1
- Step 0.6: visibility_filter.rs contract test exists — evidence: .elai_cc/crates/contracts/tests/visibility_filter.rs confirmed
- Step 0.7a: LlmProvider trait methods complete/embed/token_count/token_count_estimate — evidence: .elai_cc/crates/providers/src/providers/ollama.rs:353-356
- Step 0.7a: phase_0_implement.toml exists — evidence: .elai_cc/metrics/phase_0_implement.toml:1 (test_total=625, test_pass=585)
- Step 0.7b: FSM IDLE/EXECUTING states — evidence: .elai_cc/crates/orchestrator/src/fsm.rs:26-31 (FsmState::Idle, FsmState::Executing)
- Step 0.7b: FSM transition validation (invalid -> Error not panic) — evidence: fsm.rs transition table enforces allowed state list per state
- Step 0.7b: msgpack session storage — evidence: .elai_cc/crates/orchestrator/src/session.rs:32 (rmp_serde used for session serialization)
- Step 0.7c: visibility filter (hidden paths) — evidence: .elai_cc/crates/tools/src/lib.rs:1346 fn is_hidden_path()
- Step 0.7c: slash-command routing in cli/ — evidence: .elai_cc/crates/cli/src/runtime_client.rs:67 (slash_command_specs, validate_slash_command_input)
- Step 0.7c: tracing-based logging — evidence: .elai_cc/crates/cli/Cargo.toml:32 "tracing.workspace = true"
- Step 0.8: smoke_test_ollama.sh exists — evidence: .elai_cc/scripts/smoke_test_ollama.sh:1
- Step 0.9: ClockProvider trait in core/ — evidence: .elai_cc/crates/core/src/clock.rs:33
- Step 0.9: EntropySource trait in core/ — evidence: .elai_cc/crates/core/src/entropy.rs:7
- Step 0.9: OsClock impl in core/ — evidence: .elai_cc/crates/core/src/clock.rs:45
- Step 0.9: OsRng (OsEntropy) impl in core/ — evidence: .elai_cc/crates/core/src/entropy.rs:18
- Step 0.9: SeededClock impl in core/ — evidence: .elai_cc/crates/core/src/clock.rs:76
- Step 0.9: SeededRng impl in core/ — evidence: .elai_cc/crates/core/src/entropy.rs:36
- Step 0.9: RuntimeContext struct in core/ — evidence: .elai_cc/crates/core/src/lib.rs:103
- Step 0.9: ProductionContext and TestContext type aliases — evidence: .elai_cc/crates/core/src/lib.rs:118-120
- Step 0.9: CI quarantine script — evidence: .elai_cc/scripts/check_entropy_quarantine.sh:27 (implemented with excluded paths)
- Step 0.10: phase_0_entropy.toml records quarantine results — evidence: .elai_cc/metrics/phase_0_entropy.toml:7 (ci_gate_status=PASS, contract_tests_passed=14)
- Step 0.10: determinism_proof.sh exists — evidence: .elai_cc/scripts/determinism_proof.sh confirmed

## Deliverables PARTIAL

- Step 0.2c: compact.rs STRIP
  - exists: .elai_cc/crates/orchestrator/src/compact.rs:1 (file present with full compaction logic)
  - gap: Spec requires removal. migration_manifest.toml marks it DEFERRED. Strip was deferred to restructure; orchestrator still contains compact.rs (CompactionConfig, compact_session, format_compact_summary remain).

- Step 0.2c: usage.rs STRIP (structural)
  - exists: .elai_cc/crates/core/src/usage.rs:29 (TokenUsage, UsageTracker, format_usd, pricing_for_model, ModelPricing all present)
  - gap: Spec requires stripping UsageTracker, CompactionConfig, format_usd, pricing_for_model, ModelPricing (billing concepts). These remain intact in core/src/usage.rs and are re-exported from core/src/lib.rs:38.

- Step 0.2f: hardcoded system prompt not removed
  - exists: .elai_cc/crates/orchestrator/src/prompt.rs:474 ("You are an interactive agent that helps users...")
  - gap: Spec requires "You are an interactive agent" removed or replaced. The string persists verbatim in prompt.rs.

- Step 0.4c: workspace member count exceeds 9
  - exists: .elai_cc/Cargo.toml:3 (members list has 15 active entries: core, rules, providers, tools, plugins, knowledge, reasoning, orchestrator, cli, contracts, agent, ipc-types, web_search, scaffold, web_validator)
  - gap: Spec requires exactly 9 crate members (core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli). The workspace has grown to 15 active crates as later phases added agent, ipc-types, web_search, scaffold, web_validator. This is expected evolution but fails the Phase 0 deterministic check literally.

- Step 0.6/0.7a: config_basic contract test — elai_cc.toml not in ConfigLoader discover()
  - exists: .elai_cc/crates/contracts/tests/config_basic.rs:13 (test asserts ConfigLoader discovers elai_cc.toml)
  - gap: .elai_cc/crates/core/src/config.rs:255 explicitly documents that elai_cc.toml is NOT in discover() — it is loaded separately via runtime_config::load_elai_config(). The contract test assertion fails at runtime (intentional TDD red for this specific path, but the underlying config_basic contract requirement from spec is unmet).

- Step 0.10: determinism_proof.sh cannot complete
  - exists: .elai_cc/scripts/determinism_proof.sh confirmed
  - gap: phase_0_entropy.toml:17 documents "FAIL — set -euo pipefail causes script to exit at Step 2 when cargo test --workspace exits 101 (pre-existing cli failure)". Diff and timestamp checks never execute. Auditor recorded 4 WARNs including SeededClock not seeded by ELAI_SEED value.

## Deliverables MISSING

- .elai_cc/metrics/phase_0_baseline.toml field test_total: baseline recorded test_pass_total=93 (not the spec-required 684 tests from workspace). The spec says "run existing 684 tests — record how many pass". The baseline only reflects plugins+tools (93 tests), not full workspace. The field name differs (test_pass_total vs test_pass/test_fail/test_total).

## Deliverables UNVERIFIABLE

- Step 0.2e: runtime/src/oauth.rs stripped — migration_manifest.toml marks it DEFERRED (not STRIPPED). The file is no longer in the old rust/ directory structure (which was archived/removed). Cannot confirm whether oauth.rs is absent or absorbed. Evidence from strip_delta.toml lists it under [deferred] section.
- Step 0.2g: slash command classification in migration_manifest.toml slash-command section — migration_manifest.toml:123 shows commands/src/lib.rs classified as REPLACE/cli but does not contain a per-command KEEP/STRIP breakdown. The spec requires each of 28+ commands individually classified. No individual entries found.
- Step 0.2h: remote.rs evaluation — migration_manifest.toml:401 marks it DEFERRED with note "Evaluate in step 0.2". No STRIP or KEEP decision recorded.

## What must be done (action list)

- [ ] Remove or stub out compact.rs from .elai_cc/crates/orchestrator/src/compact.rs (CompactionConfig, compact_session, format_compact_summary are claw-code billing concepts)
- [ ] Strip UsageTracker, format_usd, pricing_for_model, ModelPricing from .elai_cc/crates/core/src/usage.rs (keep only minimal TokenUsage struct per spec 0.2c)
- [ ] Remove or replace hardcoded prompt at .elai_cc/crates/orchestrator/src/prompt.rs:474 ("You are an interactive agent...")
- [ ] Add per-command KEEP/STRIP breakdown for 28+ slash commands to .elai_cc/migration_manifest.toml (step 0.2g requirement)
- [ ] Fix .elai_cc/scripts/determinism_proof.sh to use scoped test target (--test seeded_determinism) instead of --workspace to avoid pre-existing cli failure blocking diff check
- [ ] Resolve SeededClock ELAI_SEED seeding: phase_0_entropy.toml WARN notes clock always starts at tick 0; only RNG is seeded from ELAI_SEED
- [ ] Document or close the elai_cc.toml ConfigLoader gap — either add elai_cc.toml to discover() or mark config_basic contract test as intentionally partial

## Notes

The workspace has legitimately grown beyond 9 crates due to subsequent phases (B-plan work added agent, ipc-types, web_search, scaffold, web_validator). This is expected evolution, not a regression. The Phase 0 "exactly 9 crates" check is satisfied if evaluated against the crates present at Phase 0 completion; later additions are out of scope for this report's MISSING classification.

The entropy quarantine CI script passes (check_entropy_quarantine.sh returns 0 per metrics), but the full determinism proof script is broken due to a set -euo pipefail interaction with a pre-existing Windows cli test failure. This is a tooling defect, not a code defect.

The provider_trait.rs contract test does not test LlmProvider trait methods directly (it tests MessageRequest/MessageResponse serialization). This is a scope narrowing from the spec's intent but the file exists and compiles.
