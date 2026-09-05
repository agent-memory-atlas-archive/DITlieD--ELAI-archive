# F-8: SCF symbolic semantic conflict detector (self-mod pre-execution first)

Objective: add a portable, LLM-free, rule-based symbolic conflict detector to ELAI's self-mod promotion path so a proposed config variant that CONTRADICTS an existing locked invariant or an active tripwire is flagged BEFORE it is admitted to the DGM archive. The detector resolves IntentNode-style pre/postconditions for a candidate `StructuralChange` / `ConfigVariant` against the current process model (locked-key set + active parameter bounds + last-observed tripwire state) and emits one of three conflict types (contradictory / resource-contention / causal-violation). It complements D-3 consensus + D-4 tripwires (which gate by score/drift, NOT by semantic conflict) and runs as a WARN-then-escalate pre-execution review on the SELF-MOD half only. This is the SHARD-comparison Pillar 4 synthesis target ("DGM archive search under guards PLUS a real semantic pre-execution review, which neither ships").
Estimated effort: 1 session (~6-9 h), self-mod half only. EFFORT M per synthesis decision table.
Pre-requisites: self-mod stack SHIPPED (D-3 consensus, D-4 tripwires, E-7 Two-Gate — all confirmed in elai.map: `tdd_verifier/consensus.rs::aggregate`, `agent_pool.rs::observe_post_consensus_tripwires` (NOT tripwire.rs — W-7 corrected), `promotion_gate.rs::two_gate_promote`). C-50 ContractSpec is AVOIDED by construction (it does not resolve in elai.map — PLANNED, not shipped — so attaching to the self_mod crate has NO precondition/postcondition namespace collision; this is the exact reason the synthesis scopes F-8 to self-mod-only).
Parallel-safe with: F-7 (safety-lattice proof — F-7 proves the locked-key SET is complete/monotone as a static property; F-8 checks a DYNAMIC candidate against that set at admission time; orthogonal, no schema overlap if F-8's `symbolic_conflict_detected` lands under its own SCHEMA bump). NOT parallel-safe with any plan editing `crates/orchestrator/src/self_mod_safety.rs` or `self_mod_layer1a.rs::apply_proposal` in the same window.
Source synthesis: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C8 (full mechanism + citations + REVISED SCOPE), section 1.6 (the 27.9%-precision correction — do NOT re-introduce "deliberately low-precision" framing), section 3 (SHARD verdict, Pillar 4), section 5 TIER 3 (sequencing — self-mod path only, inter-agent BSP half DEFERRED RunPod-pending).

## Context

SCF (Symbolic Conflict Framework) ships a portable, LLM-free detector: each agent intent is an IntentNode carrying preconditions and postconditions; the framework resolves them against a process model and reports three conflict types — contradictory (two intents assert mutually exclusive facts), resource-contention (two intents claim the same exclusive resource), causal-violation (an intent's precondition depends on a postcondition that a prior intent invalidates). GROUNDING STATUS (per synthesis section 1.6): the mechanism (IntentNode pre/postconditions, 3 conflict types) is PORTABLE and confirmed real; SCF's 27.9% precision is a SIMULATION-ONLY limitation (0 stars, empty tests/, no live LLM, no framework adapter — porting from a simulation scaffold, NOT adapting a library). The precision number has NEVER been validated outside SCF's own synthetic scenario set. We therefore treat the detector as a WARN-then-escalate signal, never a hard fail-closed BLOCK, on the one-promotion-per-session self-mod path where a 27.9%-precision false-positive is tolerable (one human escalation per session is bounded cost).

ELAI's self-mod loop today gates promotion by NUMBERS, not by SEMANTICS. `promotion_gate.rs::two_gate_promote` checks behaviour-descriptor margin (E-7 Two-Gate); D-3 `consensus.rs::aggregate` checks multi-verifier k-of-n; D-4 `tripwire.rs` checks score/perplexity drift and auto-reverts. `self_mod_safety.rs::check_locked_key` / `is_locked` refuse a change that TOUCHES a locked key by NAME. None of these answer the SCF question: does this candidate's POSTCONDITION CONTRADICT a locked invariant or an active tripwire's precondition, even when it touches no locked key name directly? Example: a variant that raises `self_mod_frequency` while D-4 has an active perplexity tripwire firing is a causal-violation (its postcondition — more self-mod ticks — invalidates the tripwire's precondition that the system is quiescent) — semantically conflicting, but name-distinct from any locked key, so it sails through `check_locked_key`. F-8 closes exactly that gap, at the `apply_proposal` admission boundary, before the variant enters `config_archive.rs::ConfigArchive`.

DEFERRED (explicit, per synthesis section 5 + section 7): the inter-agent BSP half (does agent A's planned postcondition contradict agent B's precondition before the parallel cap=3 superstep executes?) is RUNPOD-PENDING and OUT OF F-8 SCOPE. Rationale (synthesis section 7): 27.9% false-positives in a parallel dispatch loop degrade to noise, and the inter-agent variant WOULD collide with C-50's precondition/postcondition namespace in the same crate. The synthesis flags Graphiti's shipped LLM-judged contradiction detection (cloud-lane acceptable) as the higher-maturity alternative for that half — a future F-N, not this one.

## Existing state (B/C/D/E coverage)

- Self-mod safety (E-7 / Layer-3): `crates/orchestrator/src/self_mod_safety.rs` ships `apply_structural_change` (elai.map:270596), `check_locked_key` (471273), `is_locked` (280844), `is_known_parameter` (14598), `SafetyError` enum (147573), `check_lock_list_read` (51804). The locked-key set is enforced by NAME match. F-8 ADDS a semantic-contradiction layer that runs ALONGSIDE (not replacing) the name-match gate.
- Self-mod types: `crates/orchestrator/src/self_mod_types.rs` ships `StructuralChange` (line 421, carries `human_approved`), `SelfModProposal` (295), `ConfigVariant` (254), `ConfigVariantSimilarity` (443, the key-overlap struct), `ProjectScope` (286). F-8's IntentNode wraps a `&StructuralChange` / `&ConfigVariant`; it does NOT introduce a new proposal type.
- Promotion gate (E-7 Two-Gate): `crates/orchestrator/src/promotion_gate.rs::two_gate_promote` (line 103, elai.map:44513) + `PromotionDecision` (81) + `PromotionConfig` (46). Gates by behaviour margin. F-8 is a PRE-gate: it runs before two_gate_promote so a contradictory variant is flagged before margin is even computed.
- Layer-1A apply path (PRODUCTION ENTRYPOINT): `crates/orchestrator/src/self_mod_layer1a.rs::Layer1aProcessor::apply_proposal` (trait at line 323, `DefaultLayer1aProcessor` impl) is the canonical admission path — wired LIVE at `crates/orchestrator/src/runtime_setup.rs:299` (`DefaultLayer1aProcessor::new(rules_db, rules_db_path, ssb)`). `apply_proposal` writes the applied_proposals row + emits SSB (contract: `apply_proposal_contract.rs::apply_proposal_writes_applied_row_and_emits_ssb`). This is F-8's insertion point.
- DGM archive (the "archive admission" boundary): `crates/orchestrator/src/config_archive.rs::ConfigArchive` (struct line 148) with `get_variant`, `select_parent`, `revert`, `enforce_cap`. F-8 fires BEFORE a variant reaches the archive.
- D-3 multi-verifier consensus: `crates/orchestrator/src/tdd_verifier/consensus.rs::aggregate` (elai.map:7687), SSB SCHEMA_VERSION 69 (`VerifierConsensusPass`). Gates by k-of-n verifier agreement — score/vote, not semantic conflict.
- D-4 tripwires: `crates/orchestrator/src/tripwire.rs` ships `TripwireMonitor`, `TripwireKind` (IqrOutlier/Perplexity/Divergence), `TripwireFired`, auto-revert. `observe_post_consensus_tripwires` lives in `crates/orchestrator/src/agent_pool.rs:1622` (NOT tripwire.rs — confirmed elai.map DEF_FN, W-7 corrected). Gates by statistical drift. F-8 READS the last-observed tripwire state as part of its process model (an active tripwire is a precondition the candidate must not causally violate).
- Phase 3G coherence check (REFERENCE ONLY — do NOT extend): `crates/orchestrator/src/coherence_check.rs` ships `check_no_undeclared_interfaces`, `check_version_incremented`, `CoherenceError`. This is a `(name, type_sig)` SET-COMPARE over declared-vs-implemented phase interfaces — it answers "is interface X declared?", NOT "does postcondition X contradict invariant Y?". F-8 is semantically distinct; it does not touch coherence_check.rs.
- C-50 ContractSpec: does NOT resolve in elai.map (grep returned zero matches) — PLANNED, codegen-scoped, NOT shipped. F-8 lives in the self_mod crate and uses the names `SymbolicIntent` / `IntentCondition` (NOT `precondition`/`postcondition` as bare top-level symbols) to guarantee zero namespace collision when C-50 later lands its `ContractSpec`.
- Dashboard self-mod surface (frontend pair anchor): `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte` is the live REFLECT/self-mod drawer (subscribes to `reflect-progress`, shows mini-critic verdicts + `TripwireStatusStrip` + `ArchiveVerifyGauge`). `crates/dashboard-v2/src-tauri/src/commands.rs:1129 get_applied_proposals` (gated `#[cfg(feature = "self-modification")]`) reads `ssb_viewer.rs::applied_proposals_layer1a` -> `SelfModProposalRow` (elai.map:397644). F-8's flagged conflicts surface through this same drawer + a sibling Tauri command.
- SSB golden lock: `crates/orchestrator/tests/action_type_lock.rs::action_type_lock_golden_matches_runtime` (elai.map:11169) + `crates/orchestrator/src/cross_lang_drift/detector.rs` (SCHEMA_VERSION regex). Baseline SCHEMA_VERSION 191 per CLAUDE.md. F-8 adds exactly one action_type.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` section 4 item C8 + section 1.6 + section 3 Pillar 4 + section 5 TIER 3 + section 7 (the DEFER list)
- `F:\.ELAI_workflow\workflow\research\shard-prior-art\00-COMPARISON-shard-vs-priorart.md` (Pillar 2 + Pillar 4 — SCF source comparison, the 7/10 < 9/10 self-undercut caveat)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_safety.rs` (full — `apply_structural_change`, `check_locked_key`, `is_locked`, `is_known_parameter`, `locked_set`, `SafetyError`; the locked-key model F-8 reads)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_types.rs:254-500` (`ConfigVariant`, `StructuralChange`, `SelfModProposal`, `ConfigVariantSimilarity`, `ProjectScope` — the candidate shapes)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_layer1a.rs:300-1000` (`Layer1aProcessor` trait + `DefaultLayer1aProcessor::apply_proposal` — the production insertion point)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_gate.rs` (full — `two_gate_promote`, `PromotionDecision`, `PromotionConfig`; F-8 runs before this)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\tripwire.rs:1-200` (TripwireKind enum + TripwireFired struct — the types F-8 reads as precondition source); `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\agent_pool.rs:1622` (`observe_post_consensus_tripwires` — the function that produces the active-tripwire snapshot; NOT in tripwire.rs per elai.map W-7 correction)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\coherence_check.rs:1-200` (REFERENCE — confirm it is set-compare only; do NOT modify)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` (action_type enum + payload + sql_str + parse arms + CHECK constraint — the 5-site SSB pattern E-30 used)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\ReasoningDrawer.svelte` (the Svelte consumer to extend)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:1120-1180` (`get_applied_proposals` Tauri command pattern + `apply_proposal` confirm flow)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:4150-4520` (`applied_proposals_layer1a`, `SelfModProposalRow` — the viewer query pattern to mirror)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs` + `crates/orchestrator/tests/action_type_lock.golden` (W-9 golden lock)

## Tasks

### TIER 3 (in-process, no network/subprocess/RunPod at gate time — this is the whole F-8 shippable scope)

1. **Add `crates/orchestrator/src/symbolic_conflict.rs` (~320 LOC).** New module (NOT a new crate — extends orchestrator, consistent with self_mod_* siblings). Core types and functions:
   - `pub enum ConflictKind { Contradictory, ResourceContention, CausalViolation }` — the 3 SCF conflict types; each carries `&'static str as_str()` for SSB sql_str.
   - `pub struct SymbolicIntent { pub asserts: Vec<IntentCondition>, pub requires: Vec<IntentCondition> }` — IntentNode port; `asserts` = postconditions, `requires` = preconditions. Names chosen to AVOID C-50 `precondition`/`postcondition` collision (W-7 namespace safety).
   - `pub struct IntentCondition { pub key: ParameterKey, pub predicate: ConditionPredicate }` where `ParameterKey` is REUSED from `self_mod_safety.rs` (elai.map:193921, the existing typed key enum — no new key vocabulary). `ConditionPredicate` is an enum `{ Equals(f64), GreaterThan(f64), LessThan(f64), Locked, Unlocked, TripwireQuiescent }`.
   - `pub struct ProcessModel<'a> { pub locked_keys: &'a HashSet<&'static str>, pub active_tripwires: &'a [TripwireKind], pub current_bounds: &'a BTreeMap<String, (f64, f64)> }` — the snapshot the candidate is resolved against. `locked_keys` comes from `self_mod_safety::locked_set()`; `active_tripwires` from the last `observe_post_consensus_tripwires` outcome.
   - `pub fn detect_conflicts(candidate: &SymbolicIntent, model: &ProcessModel) -> Vec<SymbolicConflict>` — the resolver. Pure, deterministic, no I/O, no LLM. Returns ALL conflicts (a candidate may trip more than one type). `SymbolicConflict { kind: ConflictKind, key: String, detail: String }`.
   - `pub fn intent_from_structural_change(change: &StructuralChange) -> SymbolicIntent` and `pub fn intent_from_config_variant(variant: &ConfigVariant) -> SymbolicIntent` — the two adapters mapping ELAI's real candidate types into IntentNode form. These are the ONLY ELAI-specific glue; the resolver is portable.
   - Use `blake3` (NOT blake2/sha2 per CLAUDE.md) for the candidate fingerprint that ties the SSB row to the inspectable artifact: `pub fn conflict_fingerprint(candidate: &SymbolicIntent) -> String` (blake3 hex prefix).
   - NO `.unwrap()`/`.expect()` in any non-test path; propagate via `Result` / `Option`. NO magic numbers — all thresholds are named consts.

2. **Wire `detect_conflicts` into the self-mod admission path (~90 LOC delta in `crates/orchestrator/src/self_mod_layer1a.rs`).** Inside `DefaultLayer1aProcessor::apply_proposal` (the production path wired at `runtime_setup.rs:299`), BEFORE the proposal is recorded as applied: build the `ProcessModel` from `self_mod_safety::locked_set()` + the last tripwire outcome, call `intent_from_*` on the candidate, call `detect_conflicts`. If non-empty: emit `symbolic_conflict_detected` SSB row (one per conflict, fingerprint + kind + key + detail), and apply the WARN-then-escalate policy:
   - WARN-then-escalate is the TERMINAL behaviour (per synthesis 1.6, NOT a hard BLOCK because of the 27.9% sim-only precision): in interactive mode, surface the conflict and require operator confirm (reuse the existing `apply_proposal` confirm flow already in `commands.rs:1143`); in autonomous mode, HOLD the proposal as `pending` (do NOT auto-apply a flagged variant) and escalate to the operator queue. This matches "one promotion per session, 27.9%-precision WARN-then-escalate tolerable" exactly.
   - A flagged-and-held variant NEVER silently passes. There is NO LLM-bypass branch (W stub_ban): `detect_conflicts` cannot be disabled by any prompt or by the candidate's own content; the only override is the operator confirm row.

3. **Add the `symbolic_conflict_detected` SSB action_type (~70 LOC across the 5 ssb.rs sites, W-9).** Single new action_type (Pattern-2 single-variant — NO Kind-discriminator collapse needed here, only one variant). Touch all 5 sites consistently (E-30 pattern): (a) `SsbActionType` enum arm, (b) `sql_str`/`as_sql_str` value `"symbolic_conflict_detected"`, (c) parse arm, (d) `SsbPayload::SymbolicConflictDetected(SymbolicConflictPayload { fingerprint, kind, key, detail, candidate_id, fsm_substate })`, (e) SQL CHECK constraint + round-trip golden fixture. Named SCHEMA bump: baseline 191 + 1 (the WIRING_CONTRACT names the exact post-bump number at edit time; per CLAUDE.md never hard-code a final number in plan prose — say "baseline + 1"). Flow through `action_type_lock.golden` (W-9 golden lock) + `cross_lang_drift/detector.rs`.

4. **Add the ssb_viewer query (~40 LOC in `crates/orchestrator/src/ssb_viewer.rs`).** `pub fn symbolic_conflicts(&self) -> Result<Vec<SymbolicConflictRow>, SsbViewerError>` mirroring `applied_proposals_layer1a` (elai.map:6204). `SymbolicConflictRow { seq, created_at, fingerprint, kind, key, detail, candidate_id }`. This is the data the dashboard reads — it is what makes the signal NOT wire-dark (W-4).

5. **Frontend pair: Tauri command + ReasoningDrawer extension (~60 LOC tauri + ~50 LOC svelte, W-4).** (a) New `#[tauri::command] pub fn get_symbolic_conflicts(state: State<'_, AppState>) -> CmdResult<Value>` in `crates/dashboard-v2/src-tauri/src/commands.rs` (gated `#[cfg(feature = "self-modification")]` exactly like `get_applied_proposals`, with the `#[cfg(not(...))]` empty-array twin). Reads `viewer.symbolic_conflicts()`. (b) Extend `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte`: add a "Symbolic Conflicts" section (Card) listing flagged conflicts with a tone badge (Contradictory=fail, ResourceContention=warn, CausalViolation=warn), bound via `reasoningData` state + the new command. Add the command to the TS stub (`src/lib/api/stub.ts` `logStub('get_symbolic_conflicts')`) so `cross_lang_diff.py` sees the Rust<->TS pair. Register the command in the Tauri `invoke_handler` list (the `lib.rs`/builder generate-handler site).

6. **Process-model snapshot helper (~50 LOC in `crates/orchestrator/src/symbolic_conflict.rs`).** `pub fn current_process_model<'a>(locked: &'a HashSet<&'static str>, tripwires: &'a [TripwireKind], bounds: &'a BTreeMap<String,(f64,f64)>) -> ProcessModel<'a>` plus a `tripwire_quiescent_violated(intent: &SymbolicIntent, active: &[TripwireKind]) -> bool` predicate implementing the causal-violation rule (a candidate that asserts more self-mod activity while a tripwire is active violates the tripwire's quiescence precondition). This is the one ELAI-specific causal rule; the contradictory + resource-contention rules are generic over `IntentCondition`.

7. **Decision doc at `workflow/reasoning/F-8-decisions.md` (~110 LOC).** Records: (a) WARN-then-escalate (NOT hard-BLOCK) is the terminal policy and WHY (synthesis 1.6 — 27.9% precision is sim-only, one-promotion-per-session bounds the FP cost); (b) self-mod-only scope and the explicit DEFER of the inter-agent BSP half (RunPod-pending, Graphiti LLM-judged alternative); (c) `SymbolicIntent`/`IntentCondition`/`asserts`/`requires` naming chosen to avoid C-50 `ContractSpec` precondition/postcondition collision; (d) the 3 conflict types and the one ELAI causal rule (tripwire quiescence); (e) GROUNDING STATUS table: mechanism=PORTED (SHARD/SCF comparison), precision-number=UNVERIFIED-outside-SCF-sim (do NOT cite 27.9% as an ELAI target). This is the W-10 tier statement: F-8 is entirely Tier-1 in-process (no network, no subprocess, no RunPod at gate time) — there is no Tier-2 split because the detector is pure computation.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/symbolic_conflict.rs::detect_conflicts
  - crates/orchestrator/src/symbolic_conflict.rs::ConflictKind
  - crates/orchestrator/src/symbolic_conflict.rs::SymbolicIntent
  - crates/orchestrator/src/symbolic_conflict.rs::IntentCondition
  - crates/orchestrator/src/symbolic_conflict.rs::ProcessModel
  - crates/orchestrator/src/symbolic_conflict.rs::SymbolicConflict
  - crates/orchestrator/src/symbolic_conflict.rs::intent_from_structural_change
  - crates/orchestrator/src/symbolic_conflict.rs::intent_from_config_variant
  - crates/orchestrator/src/symbolic_conflict.rs::conflict_fingerprint
  - crates/orchestrator/src/symbolic_conflict.rs::current_process_model
  - crates/orchestrator/src/symbolic_conflict.rs::tripwire_quiescent_violated
  - crates/orchestrator/src/ssb_viewer.rs::symbolic_conflicts
  - crates/orchestrator/src/ssb_viewer.rs::SymbolicConflictRow
  - crates/orchestrator/src/ssb.rs::SsbActionType::SymbolicConflictDetected
  - crates/orchestrator/src/ssb.rs::SymbolicConflictPayload
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_symbolic_conflicts

consumes_symbols:
  - crates/orchestrator/src/self_mod_layer1a.rs::DefaultLayer1aProcessor::apply_proposal  (CONFIRMED elai.map: Layer1aProcessor trait line 323, apply_proposal impl; wired at runtime_setup.rs:299)
  - crates/orchestrator/src/self_mod_safety.rs::locked_set  (CONFIRMED elai.map:474605)
  - crates/orchestrator/src/self_mod_safety.rs::is_locked  (CONFIRMED elai.map:280844)
  - crates/orchestrator/src/self_mod_safety.rs::ParameterKey  (CONFIRMED elai.map:193921, self_mod_safety.rs:86)
  - crates/orchestrator/src/self_mod_types.rs::StructuralChange  (CONFIRMED elai.map:544399, line 421)
  - crates/orchestrator/src/self_mod_types.rs::ConfigVariant  (CONFIRMED elai.map:469038, line 254)
  - crates/orchestrator/src/tripwire.rs::TripwireKind  (CONFIRMED elai.map: tripwire.rs:53 DEF_ENUM TripwireKind::IqrOutlier/Perplexity/Divergence)
  - crates/orchestrator/src/agent_pool.rs::observe_post_consensus_tripwires  (CONFIRMED elai.map: agent_pool.rs:1622 DEF_FN — source of active_tripwires snapshot; NOT tripwire.rs — corrected W-7)
  - crates/orchestrator/src/ssb_viewer.rs::applied_proposals_layer1a  (CONFIRMED elai.map:6204 — viewer query pattern mirrored)
  - crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte  (CONFIRMED Glob — the Svelte consumer extended)

reachable_from:
  - PRODUCTION (CLI, as-built): crates/cli/src/self_mod_cmd.rs::run_apply (main.rs:566 CliAction::SelfMod -> run -> run_apply -> screen_on_runtime -> screen_config_variant_for_conflicts -> detect_conflicts) on a candidate that contradicts a locked invariant -> conflict flagged + held pending -> symbolic_conflict_detected SSB row
  - PRODUCTION (admission screen): crates/orchestrator/src/symbolic_conflict.rs::screen_config_variant_for_conflicts (symbolic_conflict.rs:546) runs detect_conflicts pre-admission and emits the symbolic_conflict_detected row on a flagged variant
  - DASHBOARD: ReasoningDrawer "Symbolic Conflicts" section -> get_symbolic_conflicts Tauri command (commands.rs:1175) -> ssb_viewer.symbolic_conflicts() -> live ssb.db rows rendered

stub_ban:
  - literal "TODO: symbolic conflict" absent
  - literal "// semantic check later" absent
  - NO LLM-bypass branch: detect_conflicts result cannot be overridden by any prompt, candidate content, or env var; only an explicit operator confirm row admits a flagged variant
  - NO default-off env terminal state: F-8 ships ENABLED on the self-mod path by default. If any ELAI_SYMBOLIC_CONFLICT_* var exists it is a kill-switch (disable-only); the DEFAULT is ON and the observable_effect MUST run with NO env override set (W-2). The session is NOT DONE while the detector is wired default-off.
  - NO silent pass on a detected conflict: a non-empty detect_conflicts result MUST emit at least one symbolic_conflict_detected SSB row before apply_proposal returns
  - NO .unwrap()/.expect() in symbolic_conflict.rs non-test paths

frontend_pair:
  svelte_consumer: crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte  ("Symbolic Conflicts" section)
  tauri_command: crates/dashboard-v2/src-tauri/src/commands.rs::get_symbolic_conflicts
  ts_binding: crates/dashboard-v2/src/lib/api/stub.ts  (logStub('get_symbolic_conflicts'))

observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- self-mod apply --proposal-fixture tests/fixtures/contradictory_locked_variant.json  (a self-mod proposal whose ConfigVariant raises a locked-invariant-governed parameter while a perplexity tripwire is active — exercises the REAL DefaultLayer1aProcessor::apply_proposal path, not a mock); OR in the running dashboard, open the Reasoning drawer and apply that fixture proposal via the apply_proposal confirm flow"
  expect_ssb: "symbolic_conflict_detected"
  expect_artifact: "the candidate is HELD (status=pending, not applied) and a row is queryable: cd .elai_cc && cargo run -p elai-cli -- ssb tail --action symbolic_conflict_detected emits >=1 row whose detail matches /CausalViolation|Contradictory/ and whose fingerprint matches /^[0-9a-f]{16}/ (blake3 prefix); the same row renders in the ReasoningDrawer 'Symbolic Conflicts' section"

## DETERMINISTIC_CHECK

```bash
# W-1 DONE-GATE (the literal line — wiring is a hard, executed done-gate, not cargo-green):
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md --run-trigger

# 1. The detector module + core symbols exist as real DEFs
rg "pub fn detect_conflicts|pub enum ConflictKind|pub struct SymbolicIntent|pub struct ProcessModel|pub fn intent_from_structural_change|pub fn intent_from_config_variant|pub fn conflict_fingerprint|pub fn tripwire_quiescent_violated" .elai_cc/crates/orchestrator/src/symbolic_conflict.rs

# 2. Wired into the production admission path (not test-only)
rg "detect_conflicts|symbolic_conflict|ProcessModel" .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs

# 3. The SSB action_type exists at all 5 sites (W-9)
rg "symbolic_conflict_detected|SymbolicConflictDetected|SymbolicConflictPayload" .elai_cc/crates/orchestrator/src/ssb.rs
rg "symbolic_conflict_detected" .elai_cc/crates/orchestrator/tests/action_type_lock.golden

# 4. The viewer query + row type (the data the dashboard reads — anti-wire-dark)
rg "pub fn symbolic_conflicts|SymbolicConflictRow" .elai_cc/crates/orchestrator/src/ssb_viewer.rs

# 5. Frontend pair (W-4): Tauri command + Svelte consumer + TS binding
rg "get_symbolic_conflicts" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
rg "Symbolic Conflicts|get_symbolic_conflicts|symbolic" .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte
rg "get_symbolic_conflicts" .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts

# 6. Cross-language drift clean (Rust SSB variant <-> TS binding parity)
python .claude/tools/cross_lang_diff.py

# 7. No default-off terminal state, no LLM-bypass (W-2 / stub_ban)
rg -n "ELAI_SYMBOLIC_CONFLICT" .elai_cc/crates/orchestrator/src/symbolic_conflict.rs .elai_cc/crates/orchestrator/src/self_mod_layer1a.rs || echo "no env gate present (acceptable — default-ON)"

# 8. Decision doc present
test -f workflow/reasoning/F-8-decisions.md
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md --run-trigger` returns PASS (exit 0): every produces_symbol resolves as a DEF/SCIP_DEF, every consumes_symbol has a real CALL/REF, the reachable_from anchor is reached over CALL_RESOLVED within depth 8, stub_ban finds zero literals, and the observable_effect trigger RAN and emitted the row. (cargo-green is necessary, NOT sufficient.)
- [ ] **W-2 default-ON verification**: the observable_effect trigger runs and emits `symbolic_conflict_detected` with NO env override set (no `ELAI_SYMBOLIC_CONFLICT_ENABLE=1` and no other flag). The detector is wired into apply_proposal by default; any env var present is a disable-only kill-switch and its absence does NOT disable detection.
- [ ] **W-3 production entrypoint**: the detector is reached from `DefaultLayer1aProcessor::apply_proposal` (live at runtime_setup.rs:299) and the dashboard apply-confirm flow — NOT from a `#[cfg(test)]`-only caller.
- [ ] **W-4 frontend pair + cross_lang_diff**: `get_symbolic_conflicts` Tauri command + ReasoningDrawer "Symbolic Conflicts" section + TS stub binding all present; `python .claude/tools/cross_lang_diff.py` passes (no Rust<->TS drift).
- [ ] **W-6 dashboard E2E**: with the dashboard running, applying the `contradictory_locked_variant.json` fixture proposal causes the flagged conflict to APPEAR in the Reasoning drawer "Symbolic Conflicts" section (verified THROUGH the dashboard by dashboard-smoker, not just a backend row).
- [ ] **W-9 SSB golden lock**: `symbolic_conflict_detected` flows through `action_type_lock.golden`; SCHEMA_VERSION bumped baseline 191 + 1 (named exactly in the executed plan); `action_type_lock` test + cross_lang_drift green; round-trip golden fixture for `SymbolicConflictPayload` passes.
- [ ] **3 conflict types**: a fixture per ConflictKind (Contradictory / ResourceContention / CausalViolation) is each detected; the causal-violation fixture (raise self_mod_frequency while perplexity tripwire active) is flagged.
- [ ] **WARN-then-escalate, not hard-BLOCK**: a flagged variant is HELD pending (autonomous) or requires operator confirm (interactive) — it is NOT silently applied and NOT hard-rejected without escalation. An operator confirm row admits it.
- [ ] **No C-50 collision**: `rg "ContractSpec" .elai_cc/crates/orchestrator/src/symbolic_conflict.rs` returns nothing; the module uses `SymbolicIntent`/`IntentCondition`/`asserts`/`requires`, not bare `precondition`/`postcondition` top-level symbols.
- [ ] **RunPod / inter-agent half DEFERRED**: F-8 ships NO inter-agent BSP conflict gate; the decision doc records the defer + the Graphiti alternative. (No separate live-RunPod checkbox needed — F-8 is entirely in-process; there is no RunPod-gated sub-phase.)
- [ ] `cargo check --workspace` GREEN; auditor (sole test runner) PASS on `-p elai-orchestrator` + the new contract tests + the dashboard svelte-check.
- [ ] No new external dependency (blake3 already in tree; anti-slopsquatting per CLAUDE.md — verify before any add).

## Tests to add

- `crates/contracts/tests/symbolic_conflict_contract.rs` (~180 LOC): one fixture per ConflictKind; assert `detect_conflicts` returns the right kind; assert a clean variant returns empty; assert the blake3 fingerprint is stable across runs (determinism under ELAI_SEED).
- `crates/contracts/tests/symbolic_conflict_apply_proposal_contract.rs` (~140 LOC): drive `DefaultLayer1aProcessor::apply_proposal` with a contradictory variant; assert a `symbolic_conflict_detected` SSB row is written AND the variant is held pending (not applied); assert a clean variant applies with NO conflict row. (Mirrors `apply_proposal_contract.rs::apply_proposal_writes_applied_row_and_emits_ssb`.)
- `crates/contracts/tests/symbolic_conflict_ssb_roundtrip_contract.rs` (~70 LOC): `SymbolicConflictPayload` round-trips through rmp; action_type sql_str == "symbolic_conflict_detected"; CHECK constraint accepts it.
- `crates/orchestrator/tests/symbolic_conflict_observable_smoke.rs` (~120 LOC): the W-5 fixture-harness exercising the REAL apply_proposal path end-to-end against a temp ssb.db + rules.db, asserting the row + the held-pending status + the fingerprint regex. (This is the deterministic harness; it is the SAME code path as the live trigger, not a mock.)

## Files touched

- CREATE: `crates/orchestrator/src/symbolic_conflict.rs` (~370 LOC: detector + 3 conflict types + 2 adapters + process-model helper + fingerprint)
- CREATE: `crates/orchestrator/tests/fixtures/contradictory_locked_variant.json` + `resource_contention_variant.json` + `causal_violation_tripwire_variant.json` + `clean_variant.json` (~4 small fixtures)
- CREATE: `workflow/reasoning/F-8-decisions.md` (~110 LOC)
- CREATE: `crates/contracts/tests/symbolic_conflict_contract.rs` (~180 LOC)
- CREATE: `crates/contracts/tests/symbolic_conflict_apply_proposal_contract.rs` (~140 LOC)
- CREATE: `crates/contracts/tests/symbolic_conflict_ssb_roundtrip_contract.rs` (~70 LOC)
- CREATE: `crates/orchestrator/tests/symbolic_conflict_observable_smoke.rs` (~120 LOC)
- MODIFY: `crates/orchestrator/src/self_mod_layer1a.rs` (~90 LOC: detect_conflicts call + WARN-then-escalate in apply_proposal)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~70 LOC: 1 new action_type across the 5 sites + payload + golden + CHECK)
- MODIFY: `crates/orchestrator/src/ssb_viewer.rs` (~40 LOC: symbolic_conflicts query + SymbolicConflictRow)
- MODIFY: `crates/orchestrator/src/lib.rs` (~3 LOC: `pub mod symbolic_conflict;` + re-export)
- MODIFY: `crates/orchestrator/tests/action_type_lock.golden` (~1 LOC: the locked variant)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~60 LOC: get_symbolic_conflicts + cfg twin)
- MODIFY: the Tauri builder generate-handler site (`crates/dashboard-v2/src-tauri/src/lib.rs` invoke_handler list) (~1 LOC: register get_symbolic_conflicts)
- MODIFY: `crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte` (~50 LOC: Symbolic Conflicts section)
- MODIFY: `crates/dashboard-v2/src/lib/api/stub.ts` (~6 LOC: get_symbolic_conflicts stub)
- MODIFY: `crates/dashboard-v2/src/lib/state/drawers.svelte.ts` (~10 LOC: reasoningData symbolic-conflicts binding; physical file is drawers.svelte.ts — SvelteKit imports it as $lib/state/drawers.svelte per W-7 correction)

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C8 (mechanism + REVISED SCOPE: self-mod-only, WARN-then-escalate, inter-agent DEFER) — PRIMARY synthesis source.
- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 1.6 — the 27.9%-precision correction (SIMULATION-ONLY limitation, NOT a design choice; precision UNVERIFIED outside SCF's own synthetic set). GROUNDING STATUS: mechanism PORTED / precision-number UNVERIFIED-as-ELAI-target. Do NOT cite 27.9% as a measured ELAI number.
- workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md Pillar 2 + Pillar 4 — SCF IntentNode pre/postcondition source; the DGM-archive-under-guards-PLUS-semantic-review synthesis target; the caveat that SHARD's own semantic detection (E2 7/10) underperformed its syntactic (9/10), so semantic conflict detection is a DESIGN HYPOTHESIS not a proven uplift.
- elai.map (F:\.ELAI_workflow\.claude\tools\elai.map) — every consumes_symbol confirmed AT AUTHORING TIME (W-7): self_mod_safety.rs (apply_structural_change:270596, check_locked_key:471273, is_locked:280844, is_known_parameter:14598, ParameterKey:193921, locked_set:474605), self_mod_types.rs (StructuralChange:544399, ConfigVariant:469038, SelfModProposal:544416, ConfigVariantSimilarity:68043), self_mod_layer1a.rs (Layer1aProcessor:323, apply_proposal impl:831, runtime_setup.rs:299 wiring), promotion_gate.rs (two_gate_promote:103), config_archive.rs (ConfigArchive:148), agent_pool.rs (observe_post_consensus_tripwires:1622 — CORRECTED from tripwire.rs, W-7 fix applied), tripwire.rs (TripwireKind:53 DEF_ENUM only), coherence_check.rs (reference-only set-compare), ssb_viewer.rs (applied_proposals_layer1a:4150, SelfModProposalRow:397644), commands.rs (get_applied_proposals:1129), ReasoningDrawer.svelte (confirmed via Glob).
- C-50 ContractSpec: `rg ContractSpec elai.map` returned ZERO matches — confirms C-50 is PLANNED/codegen-scoped and not shipped, so the self_mod-crate placement has no namespace collision (the exact reason synthesis scopes F-8 to self-mod-only).
- CLAUDE.md HARD RULES applied: blake3 (never blake2/sha2) for the conflict fingerprint; no .unwrap()/.expect() in non-test paths; Pattern-2 single-variant SSB (one action_type, no Kind collapse needed); SCHEMA_VERSION baseline 191 + 1 (named at edit time, never hard-coded final number in prose); no new external dep (blake3 in tree).

## Open questions / risks

- **FALSE-POSITIVE rate is the headline risk** (synthesis 1.6): SCF's 27.9% precision is sim-only and never validated outside its synthetic set. F-8 mitigates by (a) WARN-then-escalate not hard-BLOCK, (b) one-promotion-per-session bounding the operator-escalation cost, (c) the 3 conflict rules being conservative (they fire only on a concrete `IntentCondition` contradiction against a real locked-key/active-tripwire, not on heuristic similarity). RISK if the FP rate measured on ELAI's own self-mod history is high enough to make escalations noise — then the detector degrades to alarm fatigue. ACTION: the F-8-decisions doc must record a follow-up to measure FP rate on >=20 real self-mod proposals before considering any tightening to hard-BLOCK.
- **Process-model staleness**: the `active_tripwires` snapshot is read at apply_proposal time; if a tripwire fired-then-cleared between observe and apply, the causal-violation rule could flag a stale condition. Mitigation: read the LATEST tripwire outcome (not a cached one) inside apply_proposal; the smoke test asserts the snapshot is fresh.
- **Inter-agent BSP half is OUT OF SCOPE (DEFERRED)**: per synthesis section 5 + 7, the inter-agent conflict gate is RunPod-pending (27.9% FP in parallel cap=3 dispatch degrades to noise; C-50 namespace collision in the same crate). F-8 ships self-mod-only. The higher-maturity alternative for the inter-agent half is Graphiti's shipped LLM-judged contradiction detection (cloud-lane acceptable) — a future F-N, NOT this one. Do NOT scope-creep it in.
- **coherence_check.rs is NOT the place**: it is a `(name,type_sig)` set-compare over phase interfaces, a different question. Extending it would conflate "is interface declared" with "does postcondition contradict invariant". F-8 is a distinct module; the reads include coherence_check.rs only to CONFIRM the distinction at authoring time.
- **Grounding caveat (do NOT regress)**: semantic conflict detection is a DESIGN HYPOTHESIS, not a proven uplift — SHARD's own E2 semantic result (7/10) underperformed its syntactic (9/10). F-8 ships the mechanism as a guard signal, not as a claimed accuracy improvement. Any "F-8 catches X% of bad promotions" claim is UNGROUNDED until measured on ELAI's own history.

## WIRING DONE-GATE

How F-8 satisfies each doctrine clause (the implementer cannot skip wiring):

- **W-1 (no-wire-dark done-gate)**: the DETERMINISTIC_CHECK opens with the literal `wiring_contract_check.py --run-trigger` line; ACCEPTANCE item 1 makes its PASS a hard gate; the FSM may not reach COMPLETE without it. cargo-green alone does not close F-8.
- **W-2 (no default-off terminal state)**: `detect_conflicts` is wired into `apply_proposal` ENABLED by default. The stub_ban forbids a default-off flag as the terminal state; any `ELAI_SYMBOLIC_CONFLICT_*` var is disable-only (kill-switch); ACCEPTANCE W-2 verifies the observable_effect runs with NO env override.
- **W-3 (production entrypoint)**: reachable_from is `DefaultLayer1aProcessor::apply_proposal`, wired LIVE at `runtime_setup.rs:299` and invoked by drive_task + the dashboard confirm flow — a real self-mod admission path, not a `#[cfg(test)]` caller.
- **W-4 (frontend pair mandatory)**: `symbolic_conflict_detected` is dashboard-surfaceable, so frontend_pair names ReasoningDrawer.svelte + get_symbolic_conflicts + the TS stub; cross_lang_diff.py must pass. This is NOT a pure-internal gate (a human SHOULD see flagged conflicts), so the W-4 exception does not apply.
- **W-5 (observable_effect is a live trigger)**: the trigger is a real `cargo run -p elai-cli -- self-mod apply --proposal-fixture ...` (or dashboard apply) exercising the REAL apply_proposal path; expect_ssb is a live ssb.db row; expect_artifact is an inspectable `ssb tail` output + the held-pending status. The deterministic-fixture harness (`symbolic_conflict_observable_smoke.rs`) exercises the SAME production path (not a mock). F-8 is entirely in-process so there is no separate RunPod-gated live checkbox — the in-process trigger IS the live trigger.
- **W-6 (dashboard checkpoint)**: ACCEPTANCE W-6 requires the flagged conflict to APPEAR in the Reasoning drawer when the fixture proposal is applied — verified THROUGH the dashboard by dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol + reachable_from anchor + MODIFY target was confirmed in elai.map at authoring time (line numbers cited in Citations). C-50 ContractSpec's ABSENCE was likewise confirmed (zero grep matches) to justify the no-collision claim.
- **W-8 (produces/consumes symmetry)**: `detect_conflicts` is invoked in-plan by the apply_proposal wiring (Task 2); `symbolic_conflicts` viewer query is invoked in-plan by get_symbolic_conflicts (Task 5); `get_symbolic_conflicts` is invoked in-plan by ReasoningDrawer (Task 5). No produced symbol is left without an in-plan caller.
- **W-9 (SSB golden lock + named SCHEMA bump)**: one new action_type `symbolic_conflict_detected` flows through `action_type_lock.golden`; SCHEMA bump named as baseline 191 + 1 (exact post-bump number set at edit time); cross_lang_drift + action_type_lock must be green; Pattern-2 single-variant (no Kind discriminator needed for one variant).
- **W-10 (tiered-gate rule)**: F-8 is ENTIRELY Tier-1 — pure in-process computation, no network, no subprocess, no RunPod at gate time. There is NO Tier-2 to split out. The detector is genuinely deterministic and replayable under ELAI_SEED (the blake3 fingerprint is stable). The one network/RunPod-adjacent axis (the inter-agent BSP half) is explicitly DEFERRED out of F-8, satisfying the rule that no fail-open network axis advertises a fail-closed promise inside this plan.

## Next

F-7 (safety-lattice proof) is the natural composition: F-7 proves the locked-key SET is complete/monotone/conflict-free as a STATIC property (an E-53 amendment); F-8 checks a DYNAMIC candidate against that set at admission time. Once F-7's lattice proof lands, F-8's `ProcessModel.locked_keys` snapshot can cite the proven-complete set instead of `locked_set()` directly. The DEFERRED inter-agent BSP conflict half becomes a future F-N evaluating Graphiti's LLM-judged contradiction detection on ELAI's cloud lane (synthesis section 5 TIER 3 / section 7) — gated on a RunPod window and on a measured FP rate from F-8's self-mod history. F-10 (security-monotonicity iteration guard) also composes: F-8 catches a contradictory promotion at admission; F-10 catches a security-class regression across iterations — both are per-iteration self-mod guards, distinct axes (semantic conflict vs CWE-class monotonicity).

End of F-8.
