# B-20: Tool Registry Unification
Objective: pick ONE tool dispatcher (InProcessToolDispatcher OR
IpcToolRegistry) — delete the other. Re-align failure taxonomy from
13 categories back to spec's 9.
Estimated effort: 1 session.
Pre-requisites: B-18 (Tier C IPC), B-19 (hook system pattern).
Parallel-safe with: B-15, B-16, B-17.

## REBASELINE STATUS (s-25120, 2026-05-23)

Per s-25120 SPEC_DRIFT sweep, the 5 detected drifts trace to two prior
documented decisions (B-20A canonical-list pick, C-33 taxonomy
extension). Resolutions below; full per-drift rationale in
workflow/reasoning/B-20-decisions.md (s-25120 appended section).

- DRIFT-1 (Delete InProcessToolDispatcher): DEFERRED-TO-B-18B-IPC.
  In-process dispatcher at crates/tools/src/dispatcher.rs has 0
  FsmState hits; FSM gate is replicated in IpcToolDispatcherAdapter at
  crates/tools/src/adaptive_tool.rs:396,442-446. The in-process
  dispatcher is RETAINED until B-18B IPC accept loop ships (acceptable
  fallback path during single-process bring-up).
- DRIFT-2 (Migrate FSM filtering + TestingFirst gate to IPC path):
  DEFERRED-TO-B-18B-IPC. Gate is already mirrored in the adaptive_tool
  adapter; in-process gate must remain wired until B-18B switches
  production callers to IPC end-to-end.
- DRIFT-3 (Delete InProcessToolDispatcher tests):
  DEFERRED-TO-B-18B-IPC. Tests stay green against the retained path.
- DRIFT-4 (12 variants vs spec 9): SHIPPED-ON-V2 via C-33. C-33
  extended FailureCategory legitimately from 9 to 12 (added LazyCode,
  EmptyBody, FileSizeExceeded) so the auditor classifies Zero-Lazy-
  Code / file-size gate hits without falling back to RuntimePanic.
  Spec updated below; stale "Expected: 9 distinct variants" line
  removed from DETERMINISTIC_CHECK.
- DRIFT-5 (PERMISSION_DENIED missing): SHIPPED-ON-V2 (comment-only).
  Per B-20A DECISION, PERMISSION_DENIED is intentionally folded into
  RuntimePanic via the default-fallback branch in
  failure_taxonomy.rs:190-191. One-line comment added above that
  fallback documenting the inclusion; no dispatch-logic change.

## Context

Per audit-implementation/phase-4B.md and audit-dashboard/tools-hooks-isolation-wiring.md:
- IpcToolRegistry + *Handler types in tools/src/ are DEAD CODE in production
- Orchestrator wires the parallel InProcessToolDispatcher
  (elai_core::ToolDispatcher) instead
- FSM filtering and TestingFirst gate live on the dead path
  (NOTE per s-25120 rebaseline: gate is in fact MIRRORED on the IPC
  adapter at crates/tools/src/adaptive_tool.rs:396,442-446 — both
  paths gate. In-process path remains live until B-18B.)
- Failure taxonomy DRIFT: spec demanded 9 categories, impl has 12
  post-C-33 (was 13; C-33 reduced + reorganised). C-33 extension is
  intentional. Spec list updated to 12 below.
- Missing dedicated PERMISSION_DENIED (folded into RuntimePanic per
  B-20A DECISION; documented at failure_taxonomy.rs fallback branch)
- Edit matching SequenceMatcher similarity hint missing on failure

## Reads

- audit-implementation/phase-4B.md
- audit-dashboard/tools-hooks-isolation-wiring.md
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4B.txt
- crates/tools/src/lib.rs (IpcToolRegistry — dead, kept for B-18B)
- crates/tools/src/dispatcher.rs (InProcessToolDispatcher — live;
  path corrected from crates/orchestrator/src/dispatcher.rs in v1
  plan; the dispatcher trait + InProcess impl live in the tools crate)
- crates/tools/src/adaptive_tool.rs (IpcToolDispatcherAdapter — FSM
  gate replication at L396, L442-446)
- crates/tools/src/failure_taxonomy.rs (current 12 categories;
  9-list extended by C-33)
- workflow/reasoning/B-20A-decisions.md (canonical-list pick)
- workflow/reasoning/C-33-decisions.md (LazyCode / EmptyBody /
  FileSizeExceeded extension)

## Tasks

### Tool registry decision

1. **Decision: keep IpcToolRegistry (spec-compliant) IF B-18 lands.**
   Same rationale as B-19: Tier C IPC is the spec intent.
   Recommended: IPC path. Delete InProcessToolDispatcher.
   STATUS (s-25120): DEFERRED-TO-B-18B-IPC. The in-process path is the
   only live dispatcher until B-18B accept loop ships; deleting it now
   would break task_runner.rs:1804 wiring with no replacement.

2. **Migrate FSM filtering + TestingFirst gate to IPC path.** These
   currently live on the dead InProcessToolDispatcher. Move to
   IpcToolRegistry.
   STATUS (s-25120): SHIPPED-ON-V2 (partial — gate is REPLICATED, not
   migrated). IpcToolDispatcherAdapter at adaptive_tool.rs:396,442-446
   already gates on FsmState; migration-proper happens with B-18B.

3. **Delete InProcessToolDispatcher.** Update tests.
   STATUS (s-25120): DEFERRED-TO-B-18B-IPC.

### Failure taxonomy alignment

4. **Reduce 13 categories to spec 9.**
   STATUS (s-25120): SHIPPED-ON-V2 via C-33, count is 12 (not 9). Per
   C-33 DECISION the canonical list is the B-20 plan's 9-list extended
   with LazyCode / EmptyBody / FileSizeExceeded so the auditor can
   classify Zero-Lazy-Code and file-size gate hits without falling
   back to RuntimePanic. Phase-4B.txt's competing 9-list
   (PERMISSION_DENIED, TIMEOUT, RESOURCE_EXHAUSTED, EMPTY_OUTPUT) is
   rejected for this phase per B-20A; documented in
   workflow/reasoning/B-20A-decisions.md.

   Canonical 12-list (current impl, see failure_taxonomy.rs:22-38):
   1. SYNTAX_ERROR
   2. TYPE_MISMATCH
   3. UNDEFINED_SYMBOL
   4. MISSING_IMPORT
   5. LIFETIME_ERROR (Rust-specific; per spec)
   6. BORROW_ERROR (Rust-specific; per spec)
   7. UNUSED_WARNING
   8. TEST_ASSERTION
   9. RUNTIME_PANIC (also catches PERMISSION_DENIED, TIMEOUT, OOM
      via default-fallback per B-20A DECISION)
   10. LAZY_CODE (C-33: placeholder / TODO / pass body / mock-in-prod)
   11. EMPTY_BODY (C-33: no-statement function / impl block)
   12. FILE_SIZE_EXCEEDED (C-33: max-lines gate hit)

5. **Update auditor classification reading.** Auditor reads classification
   BEFORE raw output (saves 200-500 tokens per spec). Verify the read
   path uses the new 12-category set.

### Edit matching improvements

6. **Add SequenceMatcher similarity hint on edit failure.** Per spec:
   when 3-tier edit matching (exact → whitespace-normalized →
   ellipsis-elision) all fail, return a similarity hint pointing to the
   nearest match. Use difflib::SequenceMatcher (or Rust equivalent).

### File outline mode

7. **Add force_full flag + 50% budget guard at tool layer.** Per spec:
   when reading files >300 lines, return outline by default; user can
   force full read with force_full=true. Guard: full read can't exceed
   50% of context budget; if it would, fail with hint.

### IntegrationTestRunner success rows

8. **Emit integration_checkpoint_success per level.** Currently only
   failures emit. Required for B-7 dashboard surface. Add success rows
   with timing.

### WIP commits squashed after audit

9. **Per spec: WIP commits squashed after audit success.** Currently
   audit-flagged "WIP-commits-for-large-phases (>5 edits) NOT
   implemented". Add: when more than 5 edits land in one baby step,
   commit each WIP separately; squash into one phase-level commit on
   AUDITING → MEMORIZING transition.

## WIRING_CONTRACT

produces_symbols:
  - IpcToolRegistry
  - FailureCategory
  - SimilarityHint
  - best_line_match
  - enforce_file_read_budget_guard
  - INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY
  - squash_wip_commits_if_any
consumes_symbols:
  - task_runner.rs tool dispatch (migrated to IPC on B-18B)
  - auditor classification reader
  - dashboard failure taxonomy histogram (B-6 surface)
reachable_from:
  - dashboard task submit → agent → tool dispatch (in-process until
    B-18B; IPC thereafter)
stub_ban:
  - InProcessToolDispatcher absent (BLOCKED-BY-B-18B — retained until
    accept loop ships)
  - 13-category failure_taxonomy disallowed (12 is canonical per C-33)
  - Edit matching failure without similarity hint disallowed
frontend_pair: dashboard tool/hook drawers
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo test -p elai-orchestrator --test integration_checkpoint_success -- --nocapture
  expect_ssb: finding
  expect_artifact: dashboard shows 12-category histogram

## DETERMINISTIC_CHECK

```bash
# 1. InProcessToolDispatcher present (lives in tools crate, retained
#    until B-18B). Path corrected from orchestrator -> tools per
#    s-25120 rebaseline.
rg "InProcessToolDispatcher" .elai_cc/crates/tools/src/dispatcher.rs
# Expected (post-B-18B): zero hits. Until B-18B: ≥1 hit.

# 2. IpcToolRegistry has production callers
rg "IpcToolRegistry" .elai_cc/crates --type rust --glob '!*test*'
# Expected: ≥1 hit.

# 3. Failure taxonomy = 12 categories (was 9 in v1 spec; extended by
#    C-33 with LazyCode / EmptyBody / FileSizeExceeded; see
#    crates/tools/src/failure_taxonomy.rs:22-38 and
#    workflow/reasoning/C-33-decisions.md).
rg "FailureCategory::" .elai_cc/crates/tools/src/failure_taxonomy.rs
# Expected: 12 distinct variants.

# 4. FSM gate replicated on IPC adapter (s-25120: confirms DRIFT-2
#    SHIPPED-ON-V2 partial migration).
rg "FsmState" .elai_cc/crates/tools/src/adaptive_tool.rs
# Expected: ≥3 hits (gate at L396, L442-446).
```

## ACCEPTANCE

- [ ] InProcessToolDispatcher deleted (DEFERRED-TO-B-18B-IPC)
- [ ] IpcToolRegistry live in production (DEFERRED-TO-B-18B-IPC)
- [x] FSM filtering + TestingFirst gate on IPC path (SHIPPED-ON-V2,
      replicated on IpcToolDispatcherAdapter; verified
      s-B20CLOSURE: 8 FsmState hits in adaptive_tool.rs)
- [x] Failure taxonomy = 12 categories (SHIPPED-ON-V2 via C-33;
      spec-bump from 9 documented; verified s-B20CLOSURE: 12 distinct
      variants in failure_taxonomy.rs)
- [x] PERMISSION_DENIED documented as RuntimePanic fallback
      (SHIPPED-ON-V2 via comment, per B-20A DECISION)
- [x] Edit matching SequenceMatcher hint on failure (SHIPPED-ON-V2 via
      B-20A Task 6; verified s-B20CLOSURE: SimilarityHint 3 hits in
      core/tool_dispatcher.rs, best_line_match 5 hits in
      string_similarity.rs)
- [x] File outline mode force_full + 50% guard (SHIPPED-ON-V2 via
      B-20A Task 7; verified s-B20CLOSURE: enforce_file_read_budget_guard
      5 hits in task_runner.rs)
- [x] integration_checkpoint_success rows emit (SHIPPED-ON-V2 via
      B-20A Task 8; verified s-B20CLOSURE: integration_checkpoint_success
      3 hits in task_runner.rs)
- [x] WIP commits squashed on phase boundary (SHIPPED via C-13 Task 9;
      verified s-B20CLOSURE: post_transition_auditing_to_memorizing
      1 hit in task_runner.rs)

## Files touched

- DELETE: crates/tools/src/dispatcher.rs (DEFERRED-TO-B-18B; path
  corrected from crates/orchestrator/src/dispatcher.rs — dispatcher
  trait + InProcess impl live in the tools crate, not orchestrator)
- MODIFY: crates/tools/src/lib.rs (production callers of IpcToolRegistry)
- MODIFY: crates/tools/src/failure_taxonomy.rs (12 categories per
  C-33; s-25120 added PERMISSION_DENIED inclusion comment above
  RuntimePanic fallback)
- MODIFY: crates/tools/src/edit_matcher.rs (SequenceMatcher hint)
- MODIFY: crates/tools/src/file_ops.rs (force_full + 50% guard)
- MODIFY: crates/orchestrator/src/task_runner.rs (integration success
  emit + WIP squash)
- CREATE: workflow/reasoning/B-20-decisions.md (s-25120 section
  appended)

## GATE FOR PART III

After B-15 through B-20:
- Zero CLAUDE.md hard rule violations
- No dead/duplicate implementations remaining (IpcToolRegistry +
  InProcessToolDispatcher coexistence acceptable until B-18B IPC
  accept loop ships)
- Tier C isolation real
- Hook system unified (one path, IPC-based)
- Tool registry unified (deferred to B-18B for full unification)
- Failure taxonomy aligned with spec (12 categories per C-33 extension)

Production-grade hardening complete pending B-18B. Ready for ship
after PART III + B-18B.
PART IV (B-21..28) layers final-product features on top.

## Next

B-21 (Phase 4F Feasibility Probe).
