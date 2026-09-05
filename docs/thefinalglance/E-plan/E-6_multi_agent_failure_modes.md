# E-6: MAST-grounded failure-mode defenses across the 8-role pipeline
Objective: harden ELAI's 8-role multi-agent substrate against the 14 MAST failure modes (Cemri et al. arXiv:2503.13657) and the 64.5% same-model blind-spot documented by Self-Correction Bench (arXiv:2507.02778), via (a) typed-contract role messaging, (b) iso-budget single-agent ablation harness, (c) cross-family Adversary enforcement on AUDITING, (d) persona-drift re-anchor at session midpoint, (e) MistakeBook provenance + cross-family re-verify on entry, and (f) MAST-mode SSB telemetry tags so failures are countable not anecdotal.
Estimated effort: 2-3 sessions, ~10-12 h
Pre-requisites: B-23 (self-mod substrate, gated), B-41 (per-level auditor, DONE), D-9 (L3 frozen surfaces, PLANNED), D-13 (Adversary role, PLANNED) — E-6 either lands before D-13 with cloud-Sonnet stub, or extends D-13 if D-13 ships first.
Parallel-safe with: E-7 (Darwinian substrate), E-8 (prompt evolution), E-9 (benchmarks), C-12 (per-level audit shipped).
Source report: workflow/improvement-sweep-2026-05-19/reports/06-multi-agent-failure-modes.md

## Context

MAST (arXiv:2503.13657) annotated 1600+ traces across 7 multi-agent frameworks (AutoGen, CrewAI, ChatDev, MetaGPT, Magentic-One, AG2, LangGraph) and produced a 14-mode taxonomy in 3 categories: specification (41.77% of failures), inter-agent misalignment (36.94%), task verification (21.30%). Self-Correction Bench (arXiv:2507.02778) measured 64.5% mean blind-spot rate across 14 non-reasoning LLMs — a model cannot see the error it just produced but corrects the same error from another source. Self-MoA (arXiv:2502.00674) shows heterogeneous LLM ensembling loses 6.6% on AlpacaEval 2.0 vs sampling the single best model with itself. Tran & Kiela (arXiv:2604.02460) under iso-budget thinking-token caps show single-agent Qwen3 / DeepSeek-R1 / Gemini-2.5 match or beat MAS on multi-hop tasks. ELAI runs 8 AgentRoles (Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator) and pays the MAST orchestration tax 8x — without empirical evidence of 8x lift. E-6 ships the measurement and defenses needed for ELAI to honestly claim the 8-role architecture earns its keep, with the cross-family Adversary as the hard structural fix for Auditor blindness.

## Existing state (B/C/D coverage)

- DELTA on B-41 (Per-Level Auditor Modes, shipped fd5c2e5): E-6 adds MAST-mode tagging to the audit verdict struct so AUDITING failures are countable per MAST FM-N category, not just BLOCK/PASS.
- EXTENSION of D-13 (Adversary role + cross-family verifier, PLANNED in D-SUMMARY_INDEX): E-6 mandates cross-family invocation on EVERY AUDITING transition for code/architecture claims, not just opt-in; if D-13 ships first E-6 extends it; if D-13 has not shipped E-6 lands a minimum-viable Adversary using cloud-Sonnet route.
- DELTA on anti-spiral skill + crates/orchestrator/src/anti_spiral.rs (SHIPPED): E-6 extends SpiralResult to count MAST FM-1.3 (step-repetition) per semantic step, not only per file edit.
- REPLACEMENT for free-text SSB inter-agent payloads: E-6 introduces TypedAgentEnvelope as the only legal inter-agent payload type for messages crossing AgentConnection; matches CAMEL JSON-contract pattern minus the two-agent echo failure (arXiv:2511.09710).
- EXTENSION of MistakeBook (C-21, shipped substrate): adds provenance + cross-family re-verify-on-entry so the same-model blind-spot doesn't self-reinforce.
- NEW: iso-budget ablation harness for FM-1 / FM-2 / FM-3 attribution; one new SsbActionType per MAST category (3 total).
- Persona-drift mitigation: extends existing anti-spiral 5-persona switch with a turn-N role-prompt re-anchor; this is novel to E-6.

## Reads (mandatory before coding)

- .elai_cc/crates/ipc-types/src/lib.rs (AgentRole enum at line 50; baseline for envelope type)
- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType at line 756; add 3 new variants)
- .elai_cc/crates/orchestrator/src/fsm.rs (FsmSubstate at line 124; AUDITING/Adversary transitions)
- .elai_cc/crates/orchestrator/src/agent_pool.rs (allowed_grounding_methods, AgentRole dispatch)
- .elai_cc/crates/orchestrator/src/ipc_gateway.rs (AgentConnection envelope serialization)
- .elai_cc/crates/orchestrator/src/anti_spiral.rs (SpiralResult + step normalization)
- .elai_cc/crates/orchestrator/src/per_role_runtime.rs (per-role backend mapping; needed for cross-family routing)
- F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\06-multi-agent-failure-modes.md
- arXiv:2503.13657 MAST PDF (already at .claude/papers/ per report)
- arXiv:2507.02778 Self-Correction Bench PDF (already saved)
- arXiv:2502.00674 Self-MoA PDF (already saved)
- arXiv:2604.02460 single-agent iso-budget
- arXiv:2412.00804 + arXiv:2601.04170 persona/identity drift
- arXiv:2511.09710 Echoing
- arXiv:2509.05396 Talk Isn't Always Cheap (conformity bias)
- docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md (if file present per D-SUMMARY)

## Tasks

1. Define TypedAgentEnvelope at .elai_cc/crates/ipc-types/src/envelope.rs (~120 LOC): enum payload variants (TaskHandoff, FindingReport, GroundingCitation, ConsensusVote, AdversaryVerdict, MistakeBookEntry) each as a tagged struct with sender_role: AgentRole, receiver_role: AgentRole, mast_tag: Option<MastMode>, content_hash: [u8;32] (blake3), ts_ms: u64. Add envelope_v1 wire format. Update AgentConnection in ipc_gateway.rs to refuse non-envelope payloads in debug, log+stat in release.

2. Add MastMode enum (.elai_cc/crates/orchestrator/src/mast.rs, ~80 LOC) modelling all 14 MAST modes (FM-1.1 DisobeySpec ... FM-3.3 IncorrectVerification) per the report's table. Add as_sql_str + serde. Audit verdicts that BLOCK now require an Option<MastMode> tag.

3. Wire MastMode into B-41 per-level auditor verdict at crates/orchestrator/src/per_level_audit_dispatch.rs (~60 LOC delta): when audit returns BLOCK or CASCADE, attribute to most likely MAST mode using a deterministic predicate (FindingReason::* -> MastMode). Emit new SsbActionType::AuditMastTagged with payload {verdict_id, mast_mode, confidence_band}.

4. Extend Adversary substrate (.elai_cc/crates/orchestrator/src/adversary.rs, ~250 LOC NEW or +150 LOC if D-13 file exists): require routing decision via PerRoleRuntime — Adversary MUST target a backend whose model_family != executor's model_family. If only one family available locally, route to cloud (Sonnet) with explicit env var ELAI_ALLOW_CLOUD_ADVERSARY=1; otherwise refuse to AUDITING and emit SsbActionType::CrossFamilyUnavailable so D-13's intent is honored. Add adversary_verdict_required(): bool predicate on PhaseKind (true for code/architecture/security claims).

5. Add iso-budget ablation harness at .elai_cc/crates/orchestrator/src/iso_budget_harness.rs (~180 LOC): given a task fixture + a thinking-token budget B, run (A) full 8-role ELAI pipeline capped at B tokens total; (B) single-agent baseline (one Executor with reflection token "Wait" appended per arXiv:2507.02778's 89.3% blind-spot reduction) at same B. Record per-role token spend, pass-rate, and per-MAST-mode failure counts; emit SsbActionType::IsoBudgetMeasured. CLI surface: `elai bench iso-budget --task <fixture> --budget <tokens>`.

6. Add persona re-anchor at midpoint (.elai_cc/crates/orchestrator/src/persona_anchor.rs, ~120 LOC): every N turns (default N=20, configurable) re-injects the role prompt + grounding instruction + last successful FindingReport as the first message of the agent's next turn; emits SsbActionType::PersonaReanchored. Drift metric: cosine similarity between current output embedding and role-prompt embedding via local fastembed; if cosine < 0.55 trigger anchor regardless of turn count. (Cosine threshold UNCONFIRMED; calibration is part of acceptance criteria #5.)

7. Harden MistakeBook entry path (.elai_cc/crates/orchestrator/src/mistake_book.rs DELTA ~80 LOC): every entry gets provenance {origin_role, origin_run_id, originating_mast_mode: Option<MastMode>, content_hash, timestamp_ms}; on entry, if origin_role == AgentRole::Auditor AND no cross-family Adversary verdict attached, mark entry as PROVISIONAL until next session's Adversary pass corroborates. PROVISIONAL entries do NOT influence in-context prompt during execution; corroborated entries do. This implements the report's "MistakeBook contamination" mitigation.

8. Add typed-contract enforcement test at crates/contracts/tests/typed_envelope_contract.rs (~90 LOC): assert all 10 most-common AgentConnection write paths use TypedAgentEnvelope and reject raw String payloads. Tests written first per TDD.

9. Update FSM transition matrix at crates/orchestrator/src/fsm.rs (~30 LOC delta): AUDITING -> COMPLETE blocked unless verdict carries either MastMode::None OR an attached AdversaryVerdict with cross_family_confirmed=true.

10. Dashboard panel (crates/dashboard-v2/src/lib/components/MastFailurePanel.svelte, ~220 LOC) listing per-session MAST-mode counts, with the 14 modes in 3 grouped columns; live via SSB subscription on AuditMastTagged. Tauri command get_mast_breakdown returns Vec<MastModeCount> grouped by session.

11. Add `elai mast report --session <id>` CLI command (crates/cli/src/mast_cmd.rs, ~140 LOC) producing a deterministic text summary of MAST-mode counts per session and per role; runs against SSB.

## WIRING_CONTRACT

produces_symbols:
  - ipc_types::envelope::TypedAgentEnvelope
  - orchestrator::mast::MastMode (14-variant enum + as_sql_str)
  - orchestrator::adversary::adversary_verdict_required(phase: &PhaseKind) -> bool
  - orchestrator::adversary::route_cross_family(executor_role, adversary_role) -> Result<RuntimeBackend>
  - orchestrator::iso_budget_harness::run_ablation
  - orchestrator::persona_anchor::PersonaAnchor::tick
  - orchestrator::mistake_book::Entry::provenance (field added)
  - SsbActionType::AuditMastTagged
  - SsbActionType::IsoBudgetMeasured
  - SsbActionType::PersonaReanchored
  - SsbActionType::CrossFamilyUnavailable
consumes_symbols:
  - crates/orchestrator/src/per_level_audit_dispatch.rs::record_verdict (tags MastMode)
  - crates/orchestrator/src/agent_pool.rs (allowed_grounding_methods routes via TypedAgentEnvelope)
  - crates/orchestrator/src/ipc_gateway.rs::AgentConnection::send (refuses non-envelope in debug)
  - crates/orchestrator/src/fsm.rs::transition (AUDITING -> COMPLETE gated on Adversary)
  - crates/orchestrator/src/mistake_book.rs::insert (provenance required)
reachable_from:
  - FsmSubstate::Auditing (verdict path)
  - FsmSubstate::Adversary (new substate variant for cross-family probe)
  - cli::main::main (bench iso-budget + mast report subcommands)
  - dashboard-v2/src-tauri::get_mast_breakdown
stub_ban:
  - "AdversaryVerdict { cross_family_confirmed: false }" — fixed-true stubs absent in non-test source
  - "MastMode::None" — must not be hand-set for any BLOCK verdict outside test code
  - "TypedAgentEnvelope::default()" — default impl absent or test-only
frontend_pair: crates/dashboard-v2/src/lib/components/MastFailurePanel.svelte
observable_effect:
  trigger: "elai bench iso-budget --task fixtures/multi-hop-3-step.toml --budget 4000"
  expect_ssb: "SsbActionType::IsoBudgetMeasured within 60000ms with per_role_tokens + pass_rate + mast_mode_counts populated"
  expect_artifact: "stdout includes a 2-row table (full-pipeline vs single-agent-with-Wait), per-row pass_rate and MAST-mode breakdown; dashboard MastFailurePanel renders the same counts"

## DETERMINISTIC_CHECK

```bash
# 1. Typed envelope replaces free-text payloads everywhere
rg "AgentConnection::send\(.*&String" .elai_cc/crates --type rust --glob '!**/tests/**'
# Expected: zero hits.

# 2. MAST mode is set on every BLOCK verdict
rg "verdict.*BLOCK" .elai_cc/crates/orchestrator/src/per_level_audit_dispatch.rs -n
rg "mast_mode: Some\(" .elai_cc/crates/orchestrator/src -n
# Expected: count(mast_mode: Some) >= count(verdict BLOCK paths).

# 3. AUDITING -> COMPLETE requires cross_family_confirmed
rg "AUDITING.*COMPLETE.*cross_family_confirmed" .elai_cc/crates/orchestrator/src/fsm.rs -n
# Expected: at least one match in the transition guard function.

# 4. Iso-budget harness exists and is bench-callable
cargo run -p elai-cli -- bench iso-budget --task .elai_cc/tests/fixtures/iso_budget_smoke.toml --budget 2000 --json
# Expected: JSON with full_pipeline + single_agent_wait keys; exit 0.

# 5. Persona-reanchor fires at N=20 turns
cargo test -p elai-orchestrator persona_anchor::tests::reanchor_at_default_turn_count
# Expected: PASS.

# 6. MistakeBook PROVISIONAL entries skipped in context build
cargo test -p elai-orchestrator mistake_book::tests::provisional_excluded_from_context_pack
# Expected: PASS.

# 7. blake3 used for envelope content_hash (anti-stale rule)
rg "blake2|sha2" .elai_cc/crates/ipc-types/src/envelope.rs
# Expected: zero hits.
```

## ACCEPTANCE

- [ ] TypedAgentEnvelope is the ONLY payload type accepted by AgentConnection in debug mode; release mode logs a counter on any rejected non-envelope.
- [ ] Every BLOCK verdict emitted by per_level_audit_dispatch carries a Some(MastMode); >=12 of 14 MAST modes have at least one emission across the iso-budget ablation suite.
- [ ] Adversary verdict is mandatory and cross-family-confirmed for any AUDITING -> COMPLETE transition where adversary_verdict_required(phase) returns true; FSM rejects otherwise with explicit error.
- [ ] Iso-budget ablation shows full-pipeline pass-rate either >=1 stdev above single-agent-with-Wait, or, if not, emits SsbActionType::IsoBudgetMeasured with insufficient_lift=true so the user sees the architectural cost on the dashboard.
- [ ] Persona-reanchor fires at default N=20 and the drift cosine threshold gates the anchor; cosine values logged so threshold is later calibrated empirically.
- [ ] MistakeBook provisional flow demonstrated: an Auditor-originated entry without cross-family corroboration does not enter context_pack on the next session.
- [ ] CLI command `elai mast report --session <id>` produces a deterministic, line-stable summary with 14 MAST modes and 8 roles.
- [ ] Dashboard MastFailurePanel renders live counts from at least one real session and passes svelte-check.
- [ ] blake3 used everywhere for envelope content hashing; no blake2 / sha2 in new code.
- [ ] No .unwrap() in new non-test code (per CLAUDE.md).

## Tests to add

- crates/contracts/tests/typed_envelope_contract.rs: every AgentConnection write site round-trips a TypedAgentEnvelope; raw String rejected
- crates/contracts/tests/mast_mode_tagged_contract.rs: 14 MAST modes serialize-roundtrip; verdict_with_block_carries_mast_mode = true
- crates/contracts/tests/cross_family_adversary_contract.rs: same-family executor+adversary pair rejected at FSM transition
- crates/orchestrator/tests/iso_budget_harness_smoke.rs: 2-row ablation result populated, total tokens within budget +- 5%
- crates/orchestrator/tests/persona_anchor_reanchor.rs: N=20 default; cosine override also fires
- crates/orchestrator/tests/mistake_book_provenance.rs: PROVISIONAL excluded from context_pack
- crates/orchestrator/tests/mast_report_cli.rs: deterministic output across 3 invocations on fixed SSB seed
- crates/dashboard-v2/src/lib/components/MastFailurePanel.test.ts: panel renders 14 modes from mock SSB payload

## Files touched

- CREATE: crates/ipc-types/src/envelope.rs (~120 LOC)
- CREATE: crates/orchestrator/src/mast.rs (~80 LOC)
- CREATE: crates/orchestrator/src/adversary.rs (~250 LOC if new; else delta ~150 LOC)
- CREATE: crates/orchestrator/src/iso_budget_harness.rs (~180 LOC)
- CREATE: crates/orchestrator/src/persona_anchor.rs (~120 LOC)
- CREATE: crates/cli/src/mast_cmd.rs (~140 LOC)
- CREATE: crates/dashboard-v2/src/lib/components/MastFailurePanel.svelte (~220 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (+4 SsbActionType variants, ~40 LOC delta)
- MODIFY: crates/orchestrator/src/fsm.rs (transition guard, ~30 LOC)
- MODIFY: crates/orchestrator/src/per_level_audit_dispatch.rs (MastMode attribution, ~60 LOC)
- MODIFY: crates/orchestrator/src/ipc_gateway.rs (envelope enforcement, ~50 LOC)
- MODIFY: crates/orchestrator/src/agent_pool.rs (allowed_grounding_methods, ~25 LOC)
- MODIFY: crates/orchestrator/src/mistake_book.rs (provenance + provisional, ~80 LOC)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (get_mast_breakdown command, ~40 LOC)

## Citations / Grounding

- arXiv:2503.13657 (MAST, Cemri et al. 2025) — 14-mode taxonomy with frequencies; grounds MastMode variants and frequency-band defaults
- arXiv:2507.02778 (Self-Correction Bench, Tsui 2025) — 64.5% blind-spot rate; grounds cross-family Adversary requirement and "Wait" reflection token used in single-agent baseline
- arXiv:2502.00674 (Self-MoA, Li & Lin 2025) — 6.6% AlpacaEval loss from heterogeneous mixing; grounds the "do not add agent role without iso-budget lift" rule the harness measures
- arXiv:2604.02460 (Tran & Kiela, iso-budget) — single-agent matches MAS at equal thinking tokens; grounds the ablation baseline
- arXiv:2412.00804 (Identity Drift) + arXiv:2601.04170 (Agent Drift) — monotonic persona drift with turn count; grounds re-anchor mechanism
- arXiv:2511.09710 (Echoing) — agent-to-agent echo amplification; grounds typed-envelope rather than free-text inter-agent messaging
- arXiv:2605.10481 (Constraint Drift) — constraints loosen across memory + delegation; grounds MistakeBook provenance + provisional state
- arXiv:2509.05396 (Talk Isn't Always Cheap) — conformity bias in MAS debate; grounds Adversary cross-family requirement, not just second-voice critic

## Open questions / risks

- Persona-drift cosine threshold (0.55 default) is UNCONFIRMED; needs calibration sweep across 50+ real ELAI sessions. Acceptance criterion logs cosine values so calibration data exists post-ship.
- Adversary cloud-route requires an explicit env var; without 2x3090 dual-GPU local pair (per memory:project_vram_tier_plan), cross-family is cloud-only and adds latency + cost — surface via dashboard so the user can opt out per session.
- MastMode attribution from FindingReason is a deterministic predicate, not a learned classifier; mis-tagging is possible. Mitigation: dashboard panel surfaces low-confidence band; auditor can override on next pass.
- Iso-budget harness depends on token-counter accuracy across multiple backends; tokenizer divergences across providers will skew the ablation. Use ELAI's existing token counter from PerRoleRuntime metrics, do not normalize across providers.
- Report flagged "echo amplification" risk as severe but did not prescribe; E-6 lands typed envelopes as the structural fix per CAMEL pattern, but does not solve the deeper issue that agents read each other's outputs at all — that requires a re-grounding rule for every agent on every turn (out of scope here, captured for E-7/E-8).

## Next

- E-6 unblocks E-9 (agent eval benchmarks) — MAST-mode tags feed into the failure-attribution column the benchmark report demands
- E-10 (hostile critic) recommends collapsing 8 roles to 3; E-6 delivers the iso-budget harness that lets the user make that call empirically
- D-13 (Adversary role, PLANNED) should be subsumed/extended by E-6's adversary module if not yet started; if already started, merge per D-13 schema and E-6 enforcement
- Persona-reanchor data unblocks a future E-N that decides whether ELAI should hard-cap session length per role
