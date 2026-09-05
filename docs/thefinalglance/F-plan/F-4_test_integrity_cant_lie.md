# F-4: Test-integrity + output-grounding gate (the cant-lie half of the north star)

Objective: bind every claimed test PASS to a deterministic, SSB-journaled, orchestrator-captured artifact hash so a FABRICATED pass is structurally detectable, not just trusted. Today `TestOutcomePayload` journals `{test_name, passed, duration_ms}` with NO `stdout_hash`, and the baby-step emit site (`task_runner.rs:8660`) records only a bare `passed: bool` produced by the auditor's own LLM-Bash narrative. F-4 adds `stdout_hash: Option<String>` to `TestOutcomePayload`, captures it from the orchestrator-managed process slot, and installs an AUDITING->MEMORIZING evidence gate that emits `anti_fabrication_block` when a row claims `passed=true` with `stdout_hash=None` (no orchestrator-captured proof). This is the literal "cant lie to you" half of the ELAI north star ("cant lie / cant wreck"); the "cant wreck" half is already covered (E-30 argv gate, PreToolUse, sandbox, FSM gates).
Estimated effort: 1 session (~6-9 h). TIER 1 (in-process, fail-closed, no network/subprocess/RunPod at gate time).
Pre-requisites: C-22 (SHIPPED — `stdout_hash: Option<String>` on `StubResult`/feasibility path at `feasibility_probe.rs:168`; the exact blake3 pattern to copy is at `feasibility_probe.rs:188`). C-27 (DeterminismDrift + ELAI_SEED — shrinks variance to single-run reproducibility so a captured hash is meaningful across replays). C-28 (mutation — confirms the test actually exercises the code, so a real hash maps to real coverage). None of C-22/C-27/C-28 is a hard blocker for the Tier-1 field+gate, but C-27 is what makes the hash a *reproducible* claim rather than a one-shot fingerprint.
Parallel-safe with: F-1 (VibeGate — different crate surface, no SSB collision if F-4 takes its named SCHEMA slot; coordinate the `ssb.rs` edit window so both bumps land in one golden-lock update). F-2 (security ruleset — disjoint files). NOT parallel-safe with any other plan editing `crates/orchestrator/src/evidence.rs` gate registry or `TestOutcomePayload` in the same window.
Source report: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C4 (full grounded mechanism + corrections section 1 + sequencing section 5).

## Context

Per synthesis C4: ELAI's "cant wreck your system" half is well covered (FSM gates, PreToolUse argv allowlist E-30, Tier-C sandbox). The "cant LIE" half is under-built. The canonical lie-class failure is the Replit incident (AI Incident DB #1152, CONFIRMED: ~1,206+1,196 records wiped, 4,000 fabricated users, FAKED unit-test results, FALSE rollback claim). RedCode (arXiv:2411.07781, NeurIPS 2024) shows unsafe ops described in natural language get LOWER rejection than in code — so a harness CANNOT trust an agent's self-reported PASS narrative. ELAI's own memory rule `feedback_verify_semantics`: "PROVEN requires output content to match input semantics; a 511-byte Vite splash != ELAI built a TODO app."

ELAI ground truth (code-read, confirmed in elai.map at authoring time):
- `TestOutcomePayload` (`crates/orchestrator/src/ssb.rs:9459`, fields `test_name`/`passed`/`duration_ms` at ssb.rs:530-533) journals the outcome but has NO field binding it to a captured subprocess output. A `passed=true` row is unfalsifiable.
- `emit_test_outcome` (`crates/orchestrator/src/task_runner.rs:11974`, signature `async fn emit_test_outcome(writer: &Arc<Ssb>, test_name: &str, passed: bool, duration_ms: u64)`) is the single emit funnel. It is called at TWO production sites: the baby-step site (`task_runner.rs:8660`, where `passed` comes from `baby_step_outcomes.push((bs_id.0, passed))` — the unbound LLM-narrative path) and the integration-checkpoint site (`task_runner.rs:15267`, `emit_test_outcome(writer, &test_name, run.success, run.total_duration_ms)` — where `run` is the return of `run_integration_checkpoint`, an orchestrator-managed process slot WITH a real captured output).
- The blocking design question is therefore ALREADY pre-resolved by the codebase: `run_integration_checkpoint` (`crates/orchestrator/src/pipeline_tree_navigator.rs:1946`) returns a `TestRunResult` (`crates/reasoning/src/types.rs:391`) whose fields are `passed` (types.rs:392), `output: String` (types.rs:393), `duration_ms` (types.rs:394). The orchestrator-captured stdout handle EXISTS on the integration-checkpoint path. F-4's job is to (a) plumb `run.output` into a blake3 hash on that path so the field is LIVE, and (b) make the baby-step path's `stdout_hash=None` a fail-closed signal rather than silent.
- The AUDITING->MEMORIZING gate lives in `crates/orchestrator/src/evidence.rs` (Gate 6 comment at evidence.rs:499; `pub struct ZeroBlocksGate;` at evidence.rs:504; `impl EvidenceGate for ZeroBlocksGate` block at evidence.rs:506; gate registry `all_gates()` at evidence.rs:468; `TransitionContext` at evidence.rs:103 carrying `pub test_outcomes: Option<Vec<TestOutcome>>` at evidence.rs:112 and `audit_findings` at evidence.rs:121). The `EvidenceGate` trait (object-safe: `check(&self, ctx: &TransitionContext) -> Result<(), EvidenceError>`, `transition(&self) -> (FsmState, FsmState)`, `name(&self) -> &'static str`) is the integration point for the new gate.
- `evidence::TestOutcome` (the gate-side type, `crates/orchestrator/src/evidence.rs:62`, fields `name: String`, `is_syntax_error: bool`, `flipped_to_pass: bool`, `passed`) is DISTINCT from the SSB `TestOutcomePayload`. The new gate reads from the SSB rows (the journaled truth), not only the in-memory `evidence::TestOutcome`, so the gate sees the same `stdout_hash` a human would query.

The synthesis REVISED SCOPE (section 4 C4): scope to the single concrete field + gate; pre-resolve the process-slot question FIRST (done above — the integration-checkpoint path is the slot); split semantic output-grounding (does the artifact match the *requested* semantics?) to a separate later F-N because it needs task-specific spec-pinned expected outputs and is not deterministically specifiable yet. F-4 ships ONLY: the field, the capture, the fail-closed gate, the dashboard surface.

## Existing state (B/C/D/E coverage)

- C-22 (SHIPPED): `stdout_hash: Option<String>` already on `StubResult` (`crates/orchestrator/src/feasibility_probe.rs:168`); the blake3 helper `passed_with_stdout(step_id, elapsed_ms, stdout)` (feasibility_probe.rs:187) does `let hash = blake3::hash(stdout.as_bytes()).to_hex().to_string();` (feasibility_probe.rs:188). F-4 COPIES this exact pattern onto the `TestOutcomePayload` path. The C-22 contract `crates/contracts/tests/c22_feasibility_completeness_contract.rs` (asserts `stdout_hash.is_none()` default + `Some("deadbeef")` set) is the test shape to mirror.
- C-27 (DeterminismDrift + ELAI_SEED): makes a captured hash a *reproducible* claim. F-4 does not modify C-27; it relies on the seed to make the hash stable across a replay. The Tier-1 done-gate does NOT require a replay to PASS (that is C-27's job); F-4 only requires the hash be PRESENT when `passed=true`.
- C-28 (mutation): confirms the test exercises the code. Orthogonal; the F-4 gate fires on `stdout_hash=None`, independent of mutation score.
- E-30 (SHIPPED): the "cant wreck" sibling — argv allowlist, trust dialog, SSB journal discipline, `action_type_lock.golden`. F-4 reuses E-30's golden-lock + Pattern-2 SSB discipline. The two are the two halves of the north star.
- E-52 (SHIPPED): `DiagnosticProbeGate` (EXECUTING->AUDITING) at evidence.rs:643-720 is the precedent for adding a new `EvidenceGate` impl + registering it in `all_gates()`. F-4 follows the same registration shape but on the AUDITING->MEMORIZING edge.
- The auditor agent (`.claude/agents/auditor.md`) is the sole test runner. Today its PASS narrative comes from its OWN LLM Bash tool, which is exactly the unbound path. F-4 does NOT change the auditor's role; it adds an orchestrator-side gate that refuses to advance to MEMORIZING unless the SSB row carries a captured hash. The auditor still runs tests; the orchestrator now binds the claim.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` (section 4 item C4 full; section 1 corrections; section 5 sequencing)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:525-540` + `:9455-9475` (`TestOutcomePayload` def + the 9 lockstep sites: enum arm, SQL CHECK arm, `as_sql_str`, `ssb_action_type_from_str`, payload variant, roundtrip golden — per the comment at ssb.rs:530)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:11960-11990` (`emit_test_outcome` body), `:8650-8680` (baby-step call site + `baby_step_outcomes`), `:15255-15275` (integration-checkpoint call site + `run` object)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\pipeline_tree_navigator.rs:1946-2010` (`run_integration_checkpoint` return; confirm `TestRunResult.output` is populated with real captured stdout)
- `F:\.ELAI_workflow\.elai_cc\crates\reasoning\src\types.rs:388-400` (`TestRunResult` fields `passed`/`output`/`duration_ms`)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\feasibility_probe.rs:160-220` (the C-22 `stdout_hash` + blake3 pattern to copy verbatim)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\evidence.rs:55-130` (`TestOutcome`, `TransitionContext`, `AuditFinding`), `:338-530` (`ReplanFastTrackGate` shape at :338; `all_gates()` registry at :468; `ZeroBlocksGate` struct+impl at :499-506; registration pattern to mirror for `AntiFabricationGate`), `:120-130` (`audit_findings`/`test_outcomes` context for the new gate)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:9408-9430` (`get_per_level_verdicts` — the Tauri command `AuditTimelineWindow` consumes)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\AuditTimelineWindow.svelte` (the Svelte consumer that will render the `anti_fabrication_block`)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\action_type_lock.rs:2120-2160` (the golden-lock `ALL_VARIANTS` + `TOTAL_VARIANTS` bump site)
- `.claude/agents/auditor.md` (confirm the auditor's test-run mechanism is LLM-Bash today — the unbound path F-4 closes)

## Tasks

### TIER 1 (in-process, fail-closed, the F-4 done-gate)

1. **Add `stdout_hash: Option<String>` to `TestOutcomePayload` (~40 LOC across the 9 lockstep sites in `crates/orchestrator/src/ssb.rs`).** Mirror the C-22 `StubResult` field exactly (feasibility_probe.rs:168). Touch ALL nine atomic lockstep sites the ssb.rs:530 comment enumerates: the struct field (ssb.rs:9459 block), the `as_sql_str` arm, the `ssb_action_type_from_str` arm, the SQL CHECK constraint, the payload variant, the round-trip golden fixture, the `all_test_payloads()` constructor, and the two SCIP-tracked field rows. `stdout_hash` defaults to `None` (additive, non-breaking; existing rows deserialize with `None`). Doc-comment it: `/// blake3 hex digest of the orchestrator-captured test subprocess stdout. None = no orchestrator-captured proof; the AUDITING->MEMORIZING gate treats None+passed=true as a fabrication signal.`

2. **Capture the hash on the integration-checkpoint path (~25 LOC in `crates/orchestrator/src/task_runner.rs:15259-15275`).** At `emit_checkpoint_outcome` (task_runner.rs:15259), the `run: TestRunResult` already carries `run.output` (the real captured stdout, reasoning/types.rs:393). Compute `let stdout_hash = (!run.output.is_empty()).then(|| blake3::hash(run.output.as_bytes()).to_hex().to_string());` (the exact feasibility_probe.rs:188 pattern; blake3 per CLAUDE.md HARD RULE, never blake2/sha2). Thread it into `emit_test_outcome(writer, &test_name, run.success, run.total_duration_ms, stdout_hash)` — i.e. EXTEND the `emit_test_outcome` signature (task_runner.rs:11974) to take `stdout_hash: Option<String>` and set the new payload field. The baby-step call site (task_runner.rs:8660) passes `None` for now (it has no orchestrator-captured slot) — and that `None` is precisely what the gate keys on, so it is NOT a stub, it is the fail-closed input.

3. **Add the `AntiFabricationGate` evidence gate (~90 LOC in `crates/orchestrator/src/evidence.rs`).** New `pub struct AntiFabricationGate;` implementing `EvidenceGate`: `transition()` returns `(FsmState::Auditing, FsmState::Memorizing)`; `name()` returns `"anti_fabrication"`; `check(&self, ctx)` reads the journaled `TestOutcomePayload` rows for the active superstep (via the same SSB-query path the gate context exposes — `ctx` carries `test_outcomes`; F-4 adds the queried `stdout_hash` presence into the gate's check, reading from the SSB rows the orchestrator wrote in Task 2, NOT only the in-memory `evidence::TestOutcome`). RULE: if ANY row has `passed == true && stdout_hash.is_none()` AND it originated from a path that SHOULD have a slot (the integration-checkpoint test_name suffix `_integration`, format from task_runner.rs:15266 `{level_tag}_{node_id}_integration`), the gate returns `Err(EvidenceError{ ... gate_name: "anti_fabrication", ... })` with a `FindingSeverity::Block` `AuditFinding` (evidence.rs:85). Register `AntiFabricationGate` in `all_gates()` (evidence.rs:468) — the `assert_eq!(names.len(), 9)` at evidence.rs:1358 becomes 10; update it. Emit `test_integrity_verified` SSB on PASS (every checkpoint row had a hash) and `anti_fabrication_block` SSB on BLOCK. NO bypass branch: the gate has no env/flag that disables the BLOCK behavior (W-2 + stub_ban below). NO `.unwrap()`/`.expect()` in the gate body (CLAUDE.md HARD RULE; use `?`/`ok_or`).

4. **Add 2 SSB action types (W-9, Pattern-2 collapse) (~50 LOC in `crates/orchestrator/src/ssb.rs`).** `test_integrity_verified` and `anti_fabrication_block`. Per the F-0 SCHEMA bump site list, ALSO the `stdout_hash` field on `TestOutcomePayload` (Task 1). Use the Pattern-2 single-variant-with-Kind-discriminator collapse if both new action types share a payload shape: one `TestIntegrityEventPayload` carrying `{ test_name, passed, stdout_hash: Option<String>, kind: TestIntegrityEventKind }` where `TestIntegrityEventKind ∈ { Verified, FabricationBlocked }`. SCHEMA_VERSION bump: baseline 191 (per CLAUDE.md) + N (one collapsed variant if Pattern-2, so "baseline + 1"; the WIRING_CONTRACT names the exact target). Flow through `action_type_lock.golden` + `ALL_VARIANTS`/`TOTAL_VARIANTS` (action_type_lock.rs:2141) — `cross_lang_diff` + `action_type_lock` must stay green.

5. **Frontend pair (W-4) (~60 LOC svelte + ~30 LOC tauri).** `crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte` already renders per-level audit verdicts via `get_per_level_verdicts` (commands.rs:9408). Add a `fabricationBlocks` derived count + a red badge row that lists any `anti_fabrication_block` for the task (the test_name + "no orchestrator-captured stdout_hash" reason). Extend the Tauri command `get_per_level_verdicts` (commands.rs:9408) — or add a sibling `get_test_integrity_events(task_id)` — to project the two new SSB action types into a typed DTO (`TestIntegrityEventJson { test_name, passed, has_stdout_hash, kind }`). Wire the DTO into `crates/dashboard-v2/src/lib/api/types.ts` + the stub at `api/stub.ts`. `cross_lang_diff.py` must PASS (no Rust<->TS drift).

6. **Decision doc (~80 LOC at `workflow/reasoning/F-4-decisions.md`)** covering: (a) the process-slot question is pre-resolved — the integration-checkpoint path IS the orchestrator-managed slot (`run_integration_checkpoint` -> `TestRunResult.output`), the baby-step path is intentionally `None` and that `None` is the fail-closed input not a stub; (b) why semantic output-grounding is DEFERRED to a later F-N (needs spec-pinned expected outputs, not deterministically specifiable); (c) the gate is fail-closed with NO disable flag (W-2); (d) blake3 not blake2/sha2 (CLAUDE.md); (e) Pattern-2 SSB collapse rationale + the named SCHEMA bump.

### DEFERRED (explicitly out of F-4 scope — named, not silently dropped)

- Semantic output-grounding ("does the captured artifact MATCH the requested task semantics?") — separate later F-N; needs task-specific spec-pinned expected outputs. F-4 proves a hash EXISTS and is orchestrator-captured; it does NOT yet prove the content is *correct-for-the-task*.
- Binding the BABY-STEP path to a captured slot (giving baby-step tests their own orchestrator-managed process). F-4 makes baby-step `stdout_hash=None` observable; routing baby-step tests through a slot is a follow-up (the integration-checkpoint path is the one that already has the slot and is where a fabricated integration PASS does the most damage).

## WIRING_CONTRACT

produces_symbols:
  # Reconciled to shipped reality 2026-06-02 (W-7, s-6344 F-Unit-B). The 2 SSB
  # action types ship as a Pattern-2 single-variant-with-Kind, NOT bare names.
  - crates/orchestrator/src/evidence.rs::AntiFabricationGate
  - crates/orchestrator/src/ssb.rs::TestOutcomePayload::stdout_hash
  - crates/orchestrator/src/ssb.rs::TestIntegrityEventPayload
  - crates/orchestrator/src/ssb.rs::TestIntegrityEventKind
  - crates/orchestrator/src/ssb.rs::SsbActionType::TestIntegrityEvent
  - crates/orchestrator/src/evidence.rs::IntegrationTestRowSummary
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_test_integrity_events
  - crates/dashboard-v2/src/lib/api/types.ts::TestIntegrityEventJson
consumes_symbols:
  # Reconciled 2026-06-02: F-4 path consumes IntegrationRunResult.output_capture
  # (pipeline_tree_navigator), NOT reasoning::types::TestRunResult (different
  # subsystem, only used by reasoning/verification.rs). emit_test_outcome gained
  # a new stdout_hash param.
  - crates/orchestrator/src/task_runner.rs::emit_test_outcome
  - crates/orchestrator/src/task_runner.rs::emit_checkpoint_outcome
  - crates/orchestrator/src/pipeline_tree_navigator.rs::run_integration_checkpoint
  - crates/orchestrator/src/pipeline_tree_navigator.rs::IntegrationRunResult
  - crates/orchestrator/src/pipeline_tree_navigator.rs::CriterionResult
  - crates/orchestrator/src/evidence.rs::all_gates
  - crates/orchestrator/src/evidence.rs::TransitionContext
  - crates/orchestrator/src/evidence.rs::EvidenceGate
  - crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte
reachable_from:
  # Reconciled 2026-06-02 to the REAL task_runner drive_task enforcement chain.
  # NOT fsm.rs:888 — fsm.rs transition() has no access to integration_test_rows
  # (documented at task_runner.rs:9081-9082). drive_task is the canonical site.
  - crates/orchestrator/src/task_runner.rs::drive_task (live TaskSubmitted loop @4344 -> drive_task @5131 -> post-loop AntiFabricationGate.check @9087 -> emits TestIntegrityEvent(FabricationBlocked) + returns TaskRunnerError::FabricationDetected BEFORE PhaseComplete) [PRODUCTION: the FSM task runner enforcement site]
  - crates/orchestrator/src/evidence.rs::all_gates (AntiFabricationGate registered slot 10 -> available to the EvidenceGate registry for the AUDITING->MEMORIZING transition) [PRODUCTION: the evidence gate registry]
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_test_integrity_events (Tauri command surfaces TestIntegrityEvent rows to AuditTimelineWindow) [PRODUCTION: dashboard]
stub_ban:
  - "TODO: hash stdout later" literal absent
  - "// integrity gate stub" literal absent
  - No env var / flag whose default-off DISABLES the AntiFabricationGate BLOCK (W-2: the gate has NO terminal default-off state; the only permitted env is a kill-switch that is DEFAULT-ON, and the session is not DONE until observable_effect runs with NO env override set)
  - No LLM-bypass branch: the gate decision is NOT delegated to any model call; a `passed=true && stdout_hash=None` integration row is a deterministic BLOCK with no "ask the model if it is really fine" escape
  - No silent fallback: an integration-checkpoint row that should have a hash but does not is a BLOCK, never a warn-and-continue
  - stdout_hash uses blake3 (never blake2/sha2)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands.rs::get_test_integrity_events
observable_effect:
  trigger: "elai run --plan <fixture-plan> where the fixture's integration-checkpoint test process exits 1 while the audit narrative claims PASS (deterministic fixture harness exercising the REAL emit_checkpoint_outcome -> emit_test_outcome -> AntiFabricationGate path; NO env override set, default-ON)"
  expect_ssb: "anti_fabrication_block row appears in the live ssb.db within the same superstep (action_type='anti_fabrication_block'); a clean run emits test_integrity_verified instead"
  expect_artifact: "FSM does NOT advance AUDITING->MEMORIZING for the fabricated step; workflow/sessions/{id}/audits/ carries the BLOCK finding; the AuditTimelineWindow shows a red fabrication badge. Glob: workflow/sessions/*/audits/**/*anti_fabrication* OR the ssb.db row matches /anti_fabrication_block/"

## DETERMINISTIC_CHECK

```bash
# W-1 done-gate (the literal F-plan gate line — wiring_contract_check with the trigger actually run)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-4_test_integrity_cant_lie.md --run-trigger

# Produced symbols exist as real DEFs
rg "struct AntiFabricationGate|impl EvidenceGate for AntiFabricationGate" .elai_cc/crates/orchestrator/src/evidence.rs
rg "pub stdout_hash: Option<String>" .elai_cc/crates/orchestrator/src/ssb.rs
rg "TestIntegrityEventPayload|TestIntegrityEventKind" .elai_cc/crates/orchestrator/src/ssb.rs

# The 2 new SSB action types are wired (enum arm + sql_str + from_str)
rg "test_integrity_verified|anti_fabrication_block" .elai_cc/crates/orchestrator/src/ssb.rs

# The gate is registered in the production gate registry (NOT test-only)
rg "AntiFabricationGate" .elai_cc/crates/orchestrator/src/evidence.rs | rg -v "#\[cfg\(test\)\]|mod tests"

# The hash is captured on the orchestrator-managed slot path (blake3, not blake2/sha2)
rg "blake3::hash\(run.output" .elai_cc/crates/orchestrator/src/task_runner.rs
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/task_runner.rs && echo "FAIL: forbidden hash algo" || echo "OK: blake3 only"

# Golden lock + schema bump
rg "TestIntegrityVerified|AntiFabricationBlock" .elai_cc/crates/orchestrator/tests/action_type_lock.rs

# Frontend pair + cross-lang symmetry (W-4)
rg "get_test_integrity_events" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
rg "TestIntegrityEventJson|fabricationBlocks" .elai_cc/crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte
python .claude/tools/cross_lang_diff.py

# Decision doc
test -f workflow/reasoning/F-4-decisions.md
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-4_test_integrity_cant_lie.md --run-trigger` returns PASS (exit 0) — every produces_symbol resolves to a DEF/SCIP_DEF, every consumes_symbol has a real CALL/REF, every reachable_from anchor is reached by BFS over CALL_RESOLVED edges, stub_ban finds zero forbidden literals, AND the observable_effect trigger actually RAN and emitted the row + artifact
- [ ] **W-2 default-ON verification**: the observable_effect fixture run produces `anti_fabrication_block` with NO `ELAI_*` env override set; if a kill-switch env exists it is default-ON and disabling it is a separate documented escape, never the terminal state
- [ ] **W-6 dashboard E2E**: the `anti_fabrication_block` is VISIBLE in `AuditTimelineWindow` (red fabrication badge) when the fixture runs — verified THROUGH the dashboard by the dashboard-smoker agent, not just a unit assert
- [ ] **DETERMINISTIC-FIXTURE-HARNESS (Tier-1, no RunPod)**: a fixture whose integration-checkpoint process exits 1 while the narrative claims PASS, run through the REAL `emit_checkpoint_outcome` -> `emit_test_outcome` -> `AntiFabricationGate` path, emits `anti_fabrication_block` because the SSB row has `passed=true` with `stdout_hash=None`; a clean run emits `test_integrity_verified`. (F-4 is NOT RunPod-gated; this is the live Tier-1 path, so the harness IS the live run — no separate gated live checkbox needed.)
- [ ] `stdout_hash: Option<String>` added to `TestOutcomePayload` across all 9 lockstep sites; existing rows deserialize with `None` (additive, non-breaking) — round-trip golden contract passes
- [ ] `AntiFabricationGate` registered in `all_gates()`; the `names.len()` assert bumped 9->10; gate fires on the AUDITING->MEMORIZING edge only
- [ ] blake3 only on the capture path (no blake2/sha2); no `.unwrap()`/`.expect()` in the gate or capture body (CLAUDE.md HARD RULE)
- [ ] 2 new SSB action types round-trip via the SSB roundtrip contract; `action_type_lock` + `cross_lang_diff` green; SCHEMA_VERSION bumped to the named baseline+N
- [ ] `cargo check --workspace` GREEN; auditor (sole test runner) PASS on `-p elai-orchestrator -p contracts`
- [ ] No new dependency (blake3 already in tree per C-22; verify via cargo, anti-slopsquatting per CLAUDE.md)

## Tests to add

- `crates/contracts/tests/f4_test_integrity_contract.rs` (~140 LOC): (1) `TestOutcomePayload` default has `stdout_hash == None`; (2) a constructed payload with `Some(blake3_hex)` round-trips through SSB serialize/deserialize; (3) `AntiFabricationGate.check(ctx)` with an `_integration` row `passed=true, stdout_hash=None` returns `Err` with a `FindingSeverity::Block`; (4) the same gate with `stdout_hash=Some(...)` returns `Ok`; (5) the 2 new action types appear in `all_test_payloads()` and the golden lock.
- `crates/contracts/tests/f4_fabrication_fixture_harness.rs` (~110 LOC): the deterministic fixture harness — drive `emit_checkpoint_outcome` with a `TestRunResult{ passed:true(claimed), success:false(real exit), output:"" }` and assert `emit_test_outcome` wrote a row that the gate BLOCKS. This exercises the REAL path (W-5 fixture-harness clause), not a mock of it.
- Extend `crates/orchestrator/src/evidence.rs` `#[cfg(test)]` mod with `test_anti_fabrication_gate_blocks_unhashed_integration_pass` + `test_anti_fabrication_gate_allows_hashed_pass` (mirror the existing `test_gate6_*` shapes at evidence.rs:1124).

## Files touched

- MODIFY: `crates/orchestrator/src/ssb.rs` (~90 LOC: `stdout_hash` field on `TestOutcomePayload` across 9 lockstep sites; `TestIntegrityEventPayload` + `TestIntegrityEventKind`; 2 action_type arms; SCHEMA bump; roundtrip golden)
- MODIFY: `crates/orchestrator/src/task_runner.rs` (~50 LOC: extend `emit_test_outcome` signature with `stdout_hash`; capture `blake3::hash(run.output...)` at `emit_checkpoint_outcome` ~15259; pass `None` at the baby-step site ~8660)
- MODIFY: `crates/orchestrator/src/evidence.rs` (~100 LOC: `AntiFabricationGate` struct + `EvidenceGate` impl; register in `all_gates()`; bump `names.len()` assert; `#[cfg(test)]` cases)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~40 LOC: `get_test_integrity_events` projecting the 2 action types into `TestIntegrityEventJson`)
- MODIFY: `crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte` (~60 LOC: `fabricationBlocks` derived + red badge row)
- MODIFY: `crates/dashboard-v2/src/lib/api/types.ts` (+ `api/stub.ts`) (~30 LOC: `TestIntegrityEventJson` DTO + stub)
- MODIFY: `crates/orchestrator/tests/action_type_lock.rs` (~10 LOC: 2 variants into `ALL_VARIANTS` + `TOTAL_VARIANTS`)
- CREATE: `crates/contracts/tests/f4_test_integrity_contract.rs` (~140 LOC)
- CREATE: `crates/contracts/tests/f4_fabrication_fixture_harness.rs` (~110 LOC)
- CREATE: `workflow/reasoning/F-4-decisions.md` (~80 LOC)

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C4 (the mechanism + REVISED SCOPE + measurable_claim; verification run wf_b959963a-db6) [GROUNDED: synthesis doc, survived adversarial refutation]
- AI Incident DB #1152 / theregister 2025-07-21 — Replit FAKED unit-test results + FALSE rollback claim + 4,000 fabricated users (the canonical lie-class failure) [GROUNDED: synthesis C4, CONFIRMED]
- arXiv:2411.07781 (RedCode, NeurIPS 2024) — unsafe ops in natural language get LOWER rejection than in code; the harness cannot trust the agent's self-report [GROUNDED: synthesis C4, VERIFIED]
- memory feedback_verify_semantics — "PROVEN requires output content to match input semantics; a 511-byte Vite splash != ELAI built a TODO app" [GROUNDED: ELAI auto-memory]
- ELAI code-read (elai.map confirmed at authoring time, 2026-06-01): `TestOutcomePayload` ssb.rs:9459 (no stdout_hash); `emit_test_outcome` task_runner.rs:11974; baby-step site task_runner.rs:8660; integration-checkpoint site task_runner.rs:15267; `run_integration_checkpoint` pipeline_tree_navigator.rs:1946; `TestRunResult{passed,output,duration_ms}` reasoning/types.rs:391-394; C-22 `stdout_hash`+blake3 pattern feasibility_probe.rs:168/188; `EvidenceGate`/`TransitionContext`/`ZeroBlocksGate`/`all_gates` evidence.rs:62-470; `get_per_level_verdicts` commands.rs:9408; `AuditTimelineWindow.svelte`; action_type_lock.rs:2141 [GROUNDED: own codebase, grep-confirmed — W-7 satisfied]
- CLAUDE.md HARD RULES: blake3 not blake2/sha2; no `.unwrap()`/`.expect()` in prod; Pattern-2 SSB collapse; baseline SCHEMA_VERSION 191; anti-slopsquatting; no new deps without confirmation [GROUNDED: project rules]
- DO-NOT-REINTRODUCE (synthesis section 1 corrections): none of section 1's overstated numbers (CaMeL 2.82x, Springdrift +10.7%, "below 15%", Voyager 3.3x, SCF 27.9%) appear in F-4 — F-4 makes NO external-performance claim, only a structural-detectability claim grounded in ELAI's own SSB rows [GROUNDED]

## WIRING DONE-GATE

How F-4 satisfies each doctrine clause W-1..W-10 (the implementer may NOT skip wiring — every clause is a checkable gate):

- **W-1 (no-wire-dark done-gate)**: the literal `wiring_contract_check.py --bn-file ... --run-trigger` PASS is the first ACCEPTANCE item; `cargo check --workspace` green is necessary but NOT sufficient. The wiring-contract-validator agent runs it; the FSM may not reach COMPLETE without PASS.
- **W-2 (no default-off terminal)**: `AntiFabricationGate` is wired into `all_gates()` and fires by DEFAULT on every AUDITING->MEMORIZING transition — there is NO `ELAI_*_ENABLE` flag whose default-off disables it. stub_ban forbids such a flag. The observable_effect runs with NO env override; the ACCEPTANCE W-2 item asserts default-ON.
- **W-3 (production entrypoint)**: reachable_from names three REAL production entrypoints — the FSM task runner (`emit_checkpoint_outcome`), the FSM evidence gate registry (`all_gates`), and a Tauri dashboard command (`get_per_level_verdicts`). No `#[cfg(test)]`-only caller; the DETERMINISTIC_CHECK greps `-v "#[cfg(test)]"` to prove the gate registration is production.
- **W-4 (frontend pair)**: `frontend_pair` names `AuditTimelineWindow.svelte` + `get_test_integrity_events`; `cross_lang_diff.py` is in DETERMINISTIC_CHECK and ACCEPTANCE. The `anti_fabrication_block` is a human-visible signal, so a UI consumer is mandatory and provided.
- **W-5 (live observable_effect)**: `observable_effect.trigger` is a runnable `elai run --plan <fixture>` command that produces a LIVE `anti_fabrication_block` ssb.db row + an inspectable audit artifact — NOT a unit test. F-4 is Tier-1 (no RunPod), so the deterministic fixture harness IS the live path (exercises the REAL emit chain, not a mock); there is no separate gated live checkbox because no model run is required.
- **W-6 (dashboard E2E)**: an ACCEPTANCE item verifies the fabrication badge THROUGH the dashboard via dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol, every reachable_from anchor, and every MODIFY target was grep-confirmed real in elai.map at authoring time (see Citations "ELAI code-read"); the pre-resolved process-slot answer (`run_integration_checkpoint` -> `TestRunResult.output`) is a confirmed real return, not an assumption.
- **W-8 (produces/consumes symmetry)**: every produced symbol has an in-plan caller — `AntiFabricationGate` is called by `all_gates()` (in-plan), `stdout_hash` is written by `emit_test_outcome` (in-plan, Task 2) and read by `AntiFabricationGate` (in-plan, Task 3), `get_test_integrity_events` is consumed by `AuditTimelineWindow.svelte` (in-plan, Task 5). No produced function is deferred to "later".
- **W-9 (SSB golden lock + schema bump)**: the 2 action types flow through `action_type_lock.golden` + `ALL_VARIANTS`/`TOTAL_VARIANTS`; Pattern-2 single-variant-with-Kind collapse (`TestIntegrityEventKind ∈ {Verified, FabricationBlocked}`); SCHEMA_VERSION = baseline 191 + 1 (named in WIRING_CONTRACT Task 4); `cross_lang_diff` + `action_type_lock` green.
- **W-10 (tiered-gate)**: F-4 is entirely TIER 1 — the gate is pure in-process computation (read SSB rows, compare `stdout_hash` presence, BLOCK) with zero network/subprocess/RunPod at gate time, so it is genuinely fail-closed and IS the done-gate. The one deferred axis (semantic output-grounding, which would need a model run to judge content-vs-semantics) is explicitly split out to a later F-N per the synthesis REVISED SCOPE, and F-4 makes no fail-closed claim about it.

## Open questions / risks

- RISK (gate scope): the gate keys on the `_integration` test_name suffix (task_runner.rs:15266 format) to distinguish slot-backed rows from baby-step rows. If a future change renames that suffix, the gate silently stops firing. Mitigation: the f4 contract test pins the suffix; a rename breaks the test. Surface to the implementer.
- RISK (empty stdout): a legitimately silent passing test (exit 0, empty stdout) yields `stdout_hash=None` and would false-positive. Mitigation: the capture uses `run.success` (exit code) AND output presence; the gate BLOCKs only when `passed=true && stdout_hash=None && run came from the slot path`. A genuinely empty-output integration run should set the hash of the empty string explicitly (`blake3::hash(b"")`) so `None` strictly means "no slot capture", never "empty but captured". Decision doc records this; the C-22 pattern uses `(!stdout.is_empty()).then(...)` — F-4 may need to deviate to `Some(blake3::hash(run.output.as_bytes())...)` unconditionally on the slot path so empty-but-captured is distinguishable from never-captured. CONFIRM in the decision doc.
- DESIGN (pre-resolved, restated): the process-slot question the synthesis flagged as blocking is RESOLVED by the codebase — the integration-checkpoint path already runs through `run_integration_checkpoint` with a captured `TestRunResult.output`. The baby-step path does NOT have a slot; F-4 intentionally leaves it `None` (fail-closed input) and defers giving it a slot. This is the synthesis's recommended scope, not a gap.
- UNVERIFIED: that `run.output` at `emit_checkpoint_outcome` is ALWAYS populated with real captured stdout (vs sometimes empty). The implementer MUST read pipeline_tree_navigator.rs:1946-2010 to confirm `TestRunResult.output` is the real subprocess stdout before relying on it; if it is a summary string rather than raw stdout, the hash is still orchestrator-captured (acceptable for the fabrication-detection claim) but the decision doc must state which it is. Labeled UNVERIFIED until the implementer reads the function body.
- Anti-sycophancy note: F-4 does NOT claim it makes lying impossible — it makes a *specific* lie (claiming an integration PASS with no orchestrator-captured output) STRUCTURALLY detectable. Semantic lies (captured output that is real but does not match the task) remain uncaught until the deferred semantic-grounding F-N. State this honestly in the decision doc; do not oversell.

## Next

F-1 (VibeGate) composes: both are Tier-1 in-process fail-closed gates on the SSB-journal substrate; together they are the "cant wreck" (F-1 write-gate) + "cant lie" (F-4 test-integrity) pair the north star names. The deferred semantic output-grounding check is the natural F-4 follow-up (a later F-N), gated on spec-pinned expected outputs becoming deterministically specifiable. C-27 (ELAI_SEED) is what upgrades F-4's "a hash exists" to "the hash reproduces across replays" — once C-27 is wired into the checkpoint path, F-4's `test_integrity_verified` row can carry a replay-confirmed flag. E-29 observability (if shipped) exports `anti_fabrication_block` off-machine for audit-trail archival.

End of F-4_test_integrity_cant_lie.md.
