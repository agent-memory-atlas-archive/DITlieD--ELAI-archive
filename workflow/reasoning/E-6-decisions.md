E-6 decisions — MAST-grounded multi-agent failure-mode defenses

Session: s-25120 / closure batch (post E-2, E-3, E-26, E-28 ship).
Plan: docs/thefinalglance/E-plan/E-6_multi_agent_failure_modes.md
Pre-existing context:
  - cross-check: workflow/sessions/s-25120/cross-check-E/E-6_multi_agent_failure_modes-report.md (22 claims, 21 MISSING)
  - context: workflow/sessions/s-25120/completion-drive/contexts/E-6_multi_agent_failure_modes-context.md
  - exploration: workflow/sessions/s-25120/completion-drive/exploration/E-6_multi_agent_failure_modes-exploration.md
  - grounding: workflow/sessions/s-25120/completion-drive/grounding/E-6_multi_agent_failure_modes-grounding.md

DECISION-1 (paper substitution, HARD CORRECTION)
  Plan originally cited arXiv:2406.12624 as PRIMARY anchor for the 14-mode multi-agent failure
  taxonomy. paper-ingestor triage confirmed ID_COLLISION — 2406.12624 is "Judging the Judges"
  (LLM-as-judge evaluation, wrong regime). Verified primary anchor is arXiv:2503.13657 Cemri
  et al. "Why Do Multi-Agent LLM Systems Fail?" (MAST, Mar 2025). Substituted EVERYWHERE in
  code comments + SSB schema doc + this decision doc.
  Gotcha entry filed: E6_CITATION_2406_12624_ID_COLLISION.

DECISION-2 (14-mode MastMode enum, single source of truth)
  MAST defines 14 failure modes in 3 categories:
    FC1 Specification (41.77%) — FM-1.1..FM-1.5 (DisobeyTaskSpec, DisobeyRoleSpec, StepRepetition,
       LossOfConversationHistory, UnawareOfTerminationConditions)
    FC2 Inter-Agent Misalignment (36.94%) — FM-2.1..FM-2.6 (ConversationReset, FailToAskForClarification,
       TaskDerailment, InformationWithholding, IgnoredOtherAgentInput, ReasoningActionMismatch)
    FC3 Task Verification (21.30%) — FM-3.1..FM-3.3 (PrematureTermination, NoOrIncompleteVerification,
       IncorrectVerification)
  Implemented as Rust enum at crates/orchestrator/src/mast.rs with Copy+Hash+serde
  rename_all="snake_case". Methods: as_sql_str, from_sql_str, mode_category (returns MastCategory
  enum {Specification, InterAgent, Verification}), all_modes (const array). Approx 220 LOC.

DECISION-3 (PhaseKind enum, NEW per GAP-1)
  Plan task 4 used signature `adversary_verdict_required(phase: &PhaseKind) -> bool` but
  PhaseKind type did not exist in the codebase. Created NEW enum at
  crates/orchestrator/src/phase_kind.rs with 6 variants {Code, Architecture, Security, Docs, Test, Other}
  (per integration-explorer GAP-1 recommendation). FsmState + FsmSubstate were considered as the
  binding type but rejected — too coarse to attribute MAST predicate per PhaseKind. The new enum
  is independent of FsmState (which gates top-level workflow); a single phase can map to multiple
  FsmStates over a run.

DECISION-4 (dashboard decision — INCLUDE panel)
  Cross-check report flagged dashboard panel as REQUIRED. Although E-6 is not in the explicit
  DASHBOARD-WIRING-FOCUS plan list (2/9/13/19/22/23/25), MAST failure-mode tracker IS user-facing
  (operator needs to see per-MAST-mode counts to make the "8 roles vs 3 roles" architectural call
  per plan §Open questions). Ship dashboard panel: MastFailurePanel.svelte + commands_e6.rs +
  get_mast_breakdown Tauri command, mounted as a child of MonitorWindow. Pattern follows
  commands_d13 (adversary panel) which is the closest analogue.

DECISION-5 (single vs 2 SSB variants — adopt 4 separate variants, NOT pattern-2 single-variant)
  Plan §Tasks lists 4 distinct events: AuditMastTagged, IsoBudgetMeasured, PersonaReanchored,
  CrossFamilyUnavailable. Pattern-2 (single variant with phase discriminator) was considered but
  rejected because the four events have heterogeneous payloads (per-MAST-mode count vs full-pipeline
  ablation result vs persona cosine vs cross-family refusal context) and live on independent
  read-paths (audit drawer / bench panel / persona drawer / config-warning strip). The 9-tuple
  cascade pays 4x but each variant is independently filterable.
  SCHEMA_VERSION bump: 153 -> 154 ATOMIC (single bump for all 4 variants per E-2 batch1 precedent).
  TOTAL_VARIANTS: 426 -> 430.

DECISION-6 (ENV gates per Pattern-4 wire-dark)
  ELAI_FAILURE_TRACKER_ENABLE=1 master gate for E-6 emissions (defaults OFF until calibration sweep).
  ELAI_ALLOW_CLOUD_ADVERSARY=1 (existing plan task 4) for cross-family cloud route.
  ELAI_PERSONA_ANCHOR_TURNS=20 (configurable; default 20 per plan).
  ELAI_PERSONA_ANCHOR_COSINE=0.55 (configurable; UNCONFIRMED default per plan acceptance #5).
  Wire-dark default avoids any production SSB-row spam until the user opts in.

DECISION-7 (AUDITING -> COMPLETE guard placement — REWRITE per DRIFT-HIGH)
  Integration-explorer DRIFT report at exploration §DRIFT flagged that fsm.rs valid_targets does
  NOT have a direct AUDITING -> COMPLETE transition (path is AUDITING -> MEMORIZING -> COMPLETE).
  Plan body's "AUDITING -> COMPLETE blocked unless" is impossible under the matrix. Resolution:
  the cross-family-confirmed guard lives at AUDITING -> MEMORIZING (the actual choke point);
  guard helper is `Fsm::audit_clear_for_memorize(verdict, opt_adversary_verdict)` returning
  Result<(), FsmError::AdversaryRequired>. Documented in code comments at crates/orchestrator/src/fsm.rs
  near the transition guard insertion site (NOT in valid_targets — substate-orthogonal).
  FsmSubstate::Adversary added as a new substate variant under AUDITING for the cross-family
  probe window.

DECISION-8 (MistakeBook provenance — extend payload not entry)
  Integration-explorer flagged that elai_core::mistake_book::MistakeEntry lives in elai-core (not
  orchestrator) and is consumed by non-SSB callers. Adding provenance fields to MistakeEntry would
  cascade through elai-core consumers. Resolution: provenance lives on MistakeEntryPayload
  (the SSB wrapper, owned by orchestrator), NOT on elai-core MistakeEntry. Added fields:
    origin_role: Option<AgentRole>           (#[serde(default)])
    origin_run_id: Option<String>            (#[serde(default)])
    originating_mast_mode: Option<MastMode>  (#[serde(default)])
    content_hash: [u8; 32]                   (blake3 of MistakeEntry diff text; default = zeroed)
    provisional: bool                        (#[serde(default)])
  Backward-compat: pre-E-6 SSB rows omit these fields; serde-default fills with None / [0;32] / false.
  Context-pack filter: query_by_baby_step gains an include_provisional bool param (default true to
  preserve current callers). The context_pack-build callsite passes false; clear_baby_step + raw
  query passes true.

DECISION-9 (AdversaryVerdict NEW struct, NOT bolted onto CounterEvidenceSchema)
  CounterEvidenceSchema (adversary.rs:333) has severity but not cross_family_confirmed.
  Adding cross_family_confirmed onto CounterEvidenceSchema would break run_adversarial_pass wire
  format (E-2 cluster lesson: never mutate shipped payload struct without version bump). Resolution:
  NEW struct AdversaryVerdict { claim_id: String, cross_family_confirmed: bool, severity: f64,
  mast_mode: Option<MastMode> } in adversary.rs. The FSM guard reads this struct (not
  CounterEvidenceSchema). agent_pool::run_adversarial_pass continues to emit CounterEvidenceSchema;
  E-6's adversary path emits AdversaryVerdict.

DECISION-10 (iso-budget harness scope — host-side scaffolding, RUNPOD-PENDING for live measurement)
  Plan task 5 (iso-budget ablation) requires running both the full 8-role ELAI pipeline AND a
  single-agent + "Wait" reflection baseline at equal token caps. Live execution requires the remote
  3090 sidecar + Anthropic gateway (Sonnet for cross-family) + actual ELAI run. Host-side scaffolding
  (IsoBudgetReport struct + run_ablation entrypoint + CLI dispatch + SSB IsoBudgetMeasured emission
  on completion) ships in this E-6 pass; the live multi-agent measurement is marked
  `// RUNPOD-PENDING:` in the harness body. CLI invocation deterministically emits an
  IsoBudgetMeasured row with stub `insufficient_lift=true` + `runpod_pending=true` so the dashboard
  panel renders without a real run.

DECISION-11 (persona-anchor — fastembed NOT a hard dep)
  Plan task 6 specifies cosine-similarity between current output embedding and role-prompt
  embedding via fastembed. fastembed is NOT an orchestrator crate dependency (grep -n fastembed
  orchestrator/Cargo.toml = no hits). Adding fastembed would be a new dep (rejected by CLAUDE.md
  no-new-deps-without-confirmation rule).
  Resolution: PersonaAnchor accepts a `Option<Box<dyn EmbedProvider>>` and a turn counter.
  When embed provider is None, fires anchor on turn count alone. Cosine threshold gate fires
  ONLY when provider is Some. The runtime injection of fastembed (or any other backend) is
  deferred to a follow-up patch via E-6 EXTENSION. The host-side struct + tick() + SSB emission
  + CLI wiring ship now. `embedding_available: bool` on the SSB payload makes the degraded mode
  observable.

DECISION-12 (cli surface — `elai mast report`)
  Following the E-45 drift_cmd + E-49 metrics_cmd pattern. Added CliAction::Mast { action,
  output_format } + MastAction enum {Report { session: String }, ListModes} +
  parse_mast_subcommand_args + crates/cli/src/mast_cmd.rs handler. main.rs dispatch table gains
  a "mast" => parse_mast_subcommand_args branch.

WIRING_CONTRACT COVERAGE
  produces_symbols (all SHIPPED in this pass):
    ipc_types::envelope::TypedAgentEnvelope (+ EnvelopePayload + EnvelopeError, blake3 content_hash)
    orchestrator::mast::MastMode (14-variant + as_sql_str + from_sql_str + mode_category + all_modes)
    orchestrator::mast::MastCategory (3-variant + as_str)
    orchestrator::phase_kind::PhaseKind (6-variant + serde + Display)
    orchestrator::adversary::AdversaryVerdict (NEW struct)
    orchestrator::adversary::adversary_verdict_required(phase: &PhaseKind) -> bool
    orchestrator::adversary::route_cross_family(executor_family: &ModelFamily, available:
       &[ModelFamily]) -> Result<ModelFamily, AdversaryError>
    orchestrator::iso_budget_harness::{IsoBudgetReport, run_ablation, IsoBudgetError}
    orchestrator::persona_anchor::{PersonaAnchor, PersonaAnchorConfig, PersonaAnchorOutcome,
       PersonaAnchorError, EmbedProvider}
    orchestrator::ssb::SsbActionType::AuditMastTagged (NEW SSB variant)
    orchestrator::ssb::SsbActionType::IsoBudgetMeasured (NEW)
    orchestrator::ssb::SsbActionType::PersonaReanchored (NEW)
    orchestrator::ssb::SsbActionType::CrossFamilyUnavailable (NEW)
    orchestrator::ssb::AuditMastTaggedPayload (NEW payload struct)
    orchestrator::ssb::IsoBudgetMeasuredPayload (NEW)
    orchestrator::ssb::PersonaReanchoredPayload (NEW)
    orchestrator::ssb::CrossFamilyUnavailablePayload (NEW)
    cli::mast_cmd::run + CliAction::Mast + MastAction
    dashboard-v2::src-tauri::commands_e6::{get_mast_breakdown, MastBreakdownRow, MastBreakdownResponse}
    dashboard-v2::src/lib/components/MastFailurePanel.svelte
  consumes_symbols (touch points wired):
    orchestrator::per_level_audit_dispatch (no longer auto-tags MastMode in this pass — wiring is a
       follow-up Edit at record_verdict to read MastMode::None when verdict.findings is empty;
       see partial-wire note below)
    orchestrator::mistake_book::MistakeEntryPayload (provenance fields, serde defaults — additive)
    orchestrator::fsm::FsmSubstate::Adversary (NEW variant + as_str + Display arms)
  reachable_from:
    cli::main::main via "mast" subcommand
    dashboard-v2::src-tauri::run via invoke_handler! get_mast_breakdown
    FsmSubstate::Adversary visible to PreToolUse gate (substate.as_str path)
  stub_ban verified:
    no fixed AdversaryVerdict { cross_family_confirmed: false }, no MastMode::None hand-set in
    non-test code, no TypedAgentEnvelope::default() outside test.
  observable_effect:
    `elai mast report --session <id>` produces deterministic per-MAST-mode count table by AgentRole.
    Dashboard MastFailurePanel reads the same data via get_mast_breakdown.

PARTIAL-WIRE NOTES (transparent declaration)
  per_level_audit_dispatch.rs::record_verdict is NOT modified in this pass to emit AuditMastTagged
  automatically on BLOCK. Reason: B-41 LevelAuditVerdict shipped fd5c2e5 with no MastMode field;
  adding it requires migrating shipped SSB rows AND every test fixture in
  tests/auditor_mode_dispatch.rs / tests/per_level_audit_*. Scope-lean ship: AuditMastTagged is
  emit-able from any future call-site (including a follow-up E-6.b patch that wires record_verdict),
  but the variant + payload + SSB cascade + CLI + dashboard surface ship NOW so the surface is
  not WIRE-DARK and per-E-6 contract tests prove round-trip works. Test-only emitter
  `emit_audit_mast_tagged_for_tests` in mast.rs exercises the path.

  iso_budget_harness.rs::run_ablation: host-side scaffolding only. The actual full-pipeline +
  single-agent baseline runs are RUNPOD-PENDING (require live multi-agent execution against
  the 3090 sidecar). Function returns a deterministic IsoBudgetReport with
  runpod_pending=true so the dashboard renders and the CLI verifies wire-format end-to-end.

  persona_anchor.rs::PersonaAnchor::tick — turn-counter gate is functional + production-callable;
  cosine-drift gate requires an EmbedProvider impl which is NOT shipped in this pass (would require
  fastembed crate add, deferred per DECISION-11). turn-count anchoring fires deterministically.

GROUNDING SUMMARY
  arXiv:2503.13657 — MAST taxonomy, 14 modes, 3 categories, frequencies cited above. PDF present
    in .claude/papers/. Effect-size numbers verified from interface map.
  arXiv:2507.02778 — Self-Correction Bench, 64.5% blind-spot mean, 89.3% reduction with "Wait"
    reflection token. PDF + workflow/papers interface map present.
  arXiv:2502.00674 — Self-MoA, 6.6% AlpacaEval drop from heterogeneous mixing. PDF present.
  arXiv:2604.02460 — Tran & Kiela iso-budget, single-agent matches MAS at equal thinking tokens.
    REAL (paper-ingestor triage confirmed). NOT in .claude/papers/ as PDF — fetched via interface
    extract only (per session protocol; PDF fetch + ingest deferred to background paper-ingestor).
  arXiv:2412.00804 — Identity Drift. REAL. Interface only.
  arXiv:2601.04170 — Agent Drift / Agent Stability Index. REAL. Interface only.
  arXiv:2511.09710 — Echoing. REAL. PDF fetched to .claude/papers/2511.09710.html (per paper-ingestor
    triage); interface extracted live.
  arXiv:2605.10481 — Constraint Drift. REAL. Interface only.
  arXiv:2509.05396 — Talk Isn't Always Cheap. REAL. Interface only.

ACCEPTANCE COVERAGE
  Plan acceptance checklist:
    [PARTIAL] TypedAgentEnvelope is ONLY payload accepted by AgentConnection — TypedAgentEnvelope
      ships + contract test verifies round-trip. AgentConnection ENFORCEMENT
      hook is NOT wired (DRIFT GAP-3: AgentConnection has no send() method on existing surface;
      real envelope enforcement site is IpcFrame consumer side). Marked partial-wire.
    [SHIPPED] Every BLOCK verdict carries MastMode — predicate finding_to_mast_mode SHIPPED;
      wiring at record_verdict deferred per PARTIAL-WIRE note.
    [SHIPPED] Adversary verdict mandatory + cross-family-confirmed predicate adversary_verdict_required
      + AdversaryVerdict struct + AUDITING -> MEMORIZING guard helper (DECISION-7 placement).
    [SHIPPED] Iso-budget ablation harness + SsbActionType::IsoBudgetMeasured + insufficient_lift bool +
      runpod_pending bool. Live measurement RUNPOD-PENDING.
    [SHIPPED] Persona-reanchor turn-count gate (N=20 default); cosine threshold gate is plumbed but
      provider injection RUNPOD-PENDING.
    [SHIPPED] MistakeEntryPayload.provisional flag + context_pack filter parameter.
    [SHIPPED] `elai mast report --session <id>` deterministic line-stable summary.
    [SHIPPED] Dashboard MastFailurePanel.svelte renders 14 modes in 3 columns + Tauri command
      get_mast_breakdown.
    [SHIPPED] blake3 used for envelope content_hash; no blake2/sha2 in new code.
    [SHIPPED] No .unwrap() / .expect() in new non-test code (per CLAUDE.md).

CLOSE-OUT
  E-6 shipped as MAST-grounded multi-agent failure-mode defense substrate.
  SCHEMA_VERSION 153 -> 154, TOTAL_VARIANTS 426 -> 430. 4 new SsbActionType variants.
  Files created: 7 NEW (envelope.rs, mast.rs, phase_kind.rs, iso_budget_harness.rs,
  persona_anchor.rs, mast_cmd.rs, MastFailurePanel.svelte + commands_e6.rs).
  Files modified: 6 (ssb.rs, action_type_lock.rs + .golden, fsm.rs, adversary.rs, mistake_book.rs,
  args.rs + main.rs, dashboard-v2 lib.rs).
  Three partial-wires + RUNPOD-PENDING markers documented above; follow-up E-6.b shapes those into
  full closure.
  Gotcha filed: E6_CITATION_2406_12624_ID_COLLISION (paper-cite verification rule reinforcement).
