E-18 — Tool-use reliability gate decisions
Session: s-25120 session 2/4 (concurrent with session 3 s-31708)
Plan: docs/thefinalglance/E-plan/E-18_tool_use_reliability.md
Cross-check: workflow/sessions/s-25120/cross-check-E/E-18_tool_use_reliability-report.md
Context: workflow/sessions/s-25120/completion-drive/contexts/E-18_tool_use_reliability-context.md
Exploration: workflow/sessions/s-25120/completion-drive/exploration/E-18_tool_use_reliability-exploration.md

Status
total=18 (plan) -> implemented=18 missing=0 partial=0 (reuse-heavy)
SCHEMA bump 131 -> 132 atomic
TOTAL_VARIANTS 365 -> 369 atomic
all_test_payloads 304 -> 308 atomic
golden file regen alphabetical 369 entries
elai_cc_contracts e18_tool_use_reliability_contract 20/20 PASS

DECISION-REUSE-HEAVY
The cross-check report tagged 15 of 18 claims MISSING when read against the
literal plan WIRING_CONTRACT. Five sibling plans had already shipped most of
the underlying machinery:
  - D-11 RegistryCheck (anti-slopsquatting; checkers/registry/registry_check.py
    + CheckerKind::RegistryCheck wire-name "registry_check"). E-18's task 6
    package-verification path REUSES this checker via the existing checker_registry
    dispatch instead of authoring tools::package_verify::registry_check.
  - E-35 SearchReplaceMatcher + DelimiterCollisionGuard (in elai_tools::search_replace)
    + SidecarXGrammarHook (in elai_providers::runtime::sidecar_xgrammar_hook) +
    ToolParserVariant (in elai_ipc_types). E-18's tasks 1, 2, 3 REUSE these via
    cross-crate compile dependency; no E-18 module re-implements them.
  - C-47 ToolCalibrationRegistry (in elai_plugins::tool_calibration). E-18's
    reliability gate runs AFTER C-47 consult in both dispatch_tool_call_with_gate
    and dispatch_tool_call_with_authz, so the case base sees the same call shape
    regardless of E-18 verdict.
  - C-42 AuthzPolicy + C-35 ToolGate. Composition order locked at:
      AuthzPolicy -> ToolGate -> Calibration consult -> Reliability gate -> dispatch

The 14 plan-cited duplicate surfaces that would have been built were instead
composed. Concretely:
  - SearchReplaceMatcher: NOT re-authored (E-35 ships it).
  - DelimiterCollisionGuard: NOT re-authored (E-35 ships it).
  - XGrammarLogitsProcessor / compile_all_schemas_at_boot: deferred to a future
    sidecar-side session that owns the Python-side warmup loop. E-18 ships the
    host-side gate composing on E-35's wire-format hook contract; the Python
    side is RUNPOD-PENDING.
  - tools::package_verify::registry_check: NOT created (D-11 ships it).
  - SidecarXGrammarHook trait: NOT re-defined (E-35 ships it).
  - ToolParserVariant enum: NOT re-defined (E-35 ships it).

The MISSING -> implemented transition is by COMPOSITION (the reliability gate
consults the existing surfaces) rather than by RE-CONSTRUCTION (re-authoring
each surface inside the E-18 module). The contract test pins both directions:
G4 fences that E-35 SearchReplaceMatcher / DelimiterCollisionGuard remain the
canonical surface; G5 fences that D-11 RegistryCheck wire-name stays
"registry_check".

DECISION-4-VARIANT-SSB-CUT
Plan WIRING_CONTRACT enumerated 8 new variants:
  ToolRetryExhausted, ToolOneShotRecovered, ToolResultSpilled,
  PackageRefRejected, PathAllowlistPreFiltered, ProgressMarkerEmitted,
  DuplicateToolCallDeduped, XgrammarSchemaCompileFailed

E-35 sidecar_xgrammar_hook.rs:67-68 separately enumerated 10 deferred variants:
  XgrammarMaskApplied, InstructorModeActivated, GrammarModeActivated,
  CodeactActionExecuted, SearchReplaceTier1Hit, SearchReplaceCollisionFallback,
  CloudStrictSchemaApplied, GrammarCacheLoaded, StructuredEngineRejected,
  LegacyJsonObjectRefused

The overlap (XgrammarSchemaCompileFailed ~ StructuredEngineRejected) plus the
sidecar-specific nature of most E-35 deferred variants meant a one-bump batched
shipment of all 18 would have surfaced a noisy SCHEMA bump and tied E-18 to a
sidecar follow-on that is not yet wired.

Scope rebase trimmed to the 4 essentials distinct from BOTH lists:
  - ToolReliabilityGatePassed  (Pass verdict)
  - ToolReliabilityGateRejected (Reject verdict; reject_reason_hash carries
    the snake_case reason label hashed via blake3-prefix u64)
  - ToolCallSpilled (body > inline cap; spill_path_hash + payload_bytes)
  - ToolCallRetryEscalated (Escalate verdict; attempt + escalate_to_role_hash)

The remaining 4 plan-WIRING variants + the 10 E-35-deferred variants are tracked
for a future E-18.B / E-35.B session when the sidecar Python side lands.

DECISION-SPILL-PATH
Plan cites .elai_cc/sessions/{id}/spill/{uuid}.txt.
Context-compiler flagged this as path-drift: the canonical project state dir is
.elai_cc/.elai_cc/ (ssb.db + grammar_cache live there per E-35).
Decision: spill at .elai_cc/.elai_cc/spill/<ts_ms>-<blake3-prefix>.json,
matching the sibling-convention. Filename uses timestamp + blake3 prefix so
simultaneous spills at the same millisecond do not collide. The contract test
G2 fences both: spill_root_default ends with /spill AND does NOT contain
"sessions" in the path.

DECISION-NO-FIXER-ROLE
Plan §4 says "second failure escalates to fixer agent (existing ELAI agent at
.claude/agents/fixer.md)". The Rust AgentRole enum (elai_ipc_types::AgentRole)
does NOT carry a Fixer variant. RetryDecision::Escalate { to_role } uses
AgentRole::Auditor as the destination — closest semantic match for "cycle the
audit pass when retry is exhausted". The contract test G6 fences this.

The .claude/agents/fixer.md agent is a Claude Code sub-agent (markdown role),
not a Rust enum variant. The mapping happens at orchestrator dispatch time
(spawn_general_purpose_agent("fixer.md") under AgentRole::Auditor escalation).

DECISION-AGENTDRAWER-RENAME
Plan WIRING_CONTRACT.frontend_pair cites
crates/dashboard-v2/src/lib/components/AgentDrawer.svelte.
Cross-check + Glob confirmed: AgentDrawer.svelte DOES NOT EXIST.
Sibling components in same dir (ToolGateInspector / AuthzPanel / CaseBaseInspector)
all mount under SettingsWindow, NOT a separate drawer.

Decision: extend SettingsWindow with a new ToolReliabilityInspector.svelte
component matching the sibling shape (CaseBaseInspector precedent). Same
header / summary / table / recent-rows layout. The plan-cited AgentDrawer file
is NOT created; the cited file path drifts to ToolReliabilityInspector under
the same components/ directory.

DECISION-COMPOSITION-ORDER
The plugins-crate dispatch_tool_call_with_gate + dispatch_tool_call_with_authz
methods are extended to call self.reliability_gate_check(tool_name, shape_hash,
&args) AFTER self.consult_calibration(tool_name, shape_hash).await and BEFORE
self.http_call_tool(url, tool_name, args.clone()).await.

Locked order:
  1. AuthzPolicy (C-42) — composed_with_tool_gate
  2. ToolGate (C-35) — check_tool_allowed
  3. ToolCalibrationRegistry::consult (C-47) — hint emission
  4. ToolReliabilityGate::check (E-18) — Pass / Reject / Spill verdict + SSB emit
  5. http_call_tool — actual tool dispatch
  6. ToolCalibrationRegistry::record_call (C-47) — post-call audit

Contract test composition_retry_budget_takes_precedence_over_allowlist +
composition_package_registry_reject_when_disallowed fence the gate's own
internal precedence (retry > allowlist > package > inline cap).

DECISION-ANTI-STALE
- blake3 for every digest (tool_name_hash, reject_reason_hash, spill_path_hash,
  escalate_to_role_hash, spill body integrity). NEVER blake2 / sha2.
- thiserror enums for ToolReliabilityError (NEVER Box<dyn Error>).
- tracing for the warn fallback in the orchestrator-side adapter (NEVER log).
- No .unwrap() / .expect() outside #[cfg(test)] and the const SCHEMA_VERSION
  bootstrap. blake3 prefix narrowing uses .try_into().unwrap_or([0u8; 8]) to
  surface the impossible case as a deterministic zero rather than a panic.
- u32::try_from / i64::try_from for narrow boundaries instead of `as` casts.

DECISION-CLI-NAMESPACE
Plan does not enumerate a CLI verb. Decision: `elai reliability <sub>` with
three sub-actions (status / list-spills / replay-spill) matching the existing
CliAction precedent (C-46 Routing, C-47 Calibration, D-16 ConformalCalibration,
C-49 Experience, C-50 Contract, C-52 Dafny, E-13 Tdd, E-14 Browser — the 10th
CliAction:: variant). The handler at crates/cli/src/reliability_cmd.rs is
synthetic + GPU-free; live SSB integration is via the dashboard side
ToolReliabilityInspector.

DECISION-RUNPOD-PENDING
None. The host-side reliability gate composes only on already-shipped surfaces.
The Python-side XGrammarLogitsProcessor + Hermes envelope grammar is the
genuine "tool reliability" Python work — deferred to a future sidecar-side
session (was already RUNPOD-PENDING in E-35's TODO list per
sidecar_xgrammar_hook.rs:32-77).

Files touched
=============
NEW
  .elai_cc/crates/orchestrator/src/tool_reliability/mod.rs
  .elai_cc/crates/orchestrator/src/tool_reliability/gate.rs
  .elai_cc/crates/orchestrator/src/tool_reliability/spill.rs
  .elai_cc/crates/orchestrator/src/tool_reliability/retry.rs
  .elai_cc/crates/orchestrator/src/tool_reliability/types.rs
  .elai_cc/crates/orchestrator/src/tool_reliability_ssb_adapter.rs
  .elai_cc/crates/cli/src/reliability_cmd.rs
  .elai_cc/crates/contracts/tests/e18_tool_use_reliability_contract.rs
  .elai_cc/crates/dashboard-v2/src/lib/components/ToolReliabilityInspector.svelte
  .elai_cc/crates/dashboard-v2/src-tauri/src/commands_e18.rs
  workflow/reasoning/E-18-decisions.md
MODIFY
  .elai_cc/crates/orchestrator/src/lib.rs (mod tool_reliability + mod tool_reliability_ssb_adapter)
  .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA 131->132 + 4 SsbActionType variants + 4 payload structs + 4 test arms + count assertion bump)
  .elai_cc/crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS 365->369 + 4 ALL_VARIANTS entries + 4 exhaustive_match_guard arms)
  .elai_cc/crates/orchestrator/tests/action_type_lock.golden (regen alphabetical 369 entries)
  .elai_cc/crates/plugins/src/mcp_bridge.rs (ReliabilityGateSsbEmitter trait + NoopReliabilityEmitter + reliability_emit_kinds + reliability_reject_reasons + reliability_gate_check helper + wire into both dispatchers)
  .elai_cc/crates/plugins/src/lib.rs (re-export reliability surfaces)
  .elai_cc/crates/cli/src/args.rs (CliAction::Reliability + ReliabilityAction + parse_reliability_subcommand_args + top-level "reliability" dispatch)
  .elai_cc/crates/cli/src/main.rs (mod reliability_cmd + CliAction::Reliability dispatch arm)
  .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte (import ToolReliabilityInspector + section mount)
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (mod commands_e18 + 2 invoke_handler entries)

cargo check
===========
cargo check --workspace : clean (9 pre-existing warnings, no new errors).
cargo test -p elai_cc_contracts --test e18_tool_use_reliability_contract: 20/20 PASS.
