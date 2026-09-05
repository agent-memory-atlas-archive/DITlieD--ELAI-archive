# C-38: PageRank Production Contract Test + DETAILED_SUMMARY Aider Spec Accuracy Fix

Objective: prove with a contract test that Phase 1D PageRank actually executes in
the production query path (not gated behind a never-fires dkg.db existence check),
AND correct DETAILED_SUMMARY_01_overview_and_architecture.md §10 which currently
misdescribes Aider's repomap as file-level PageRank when the actual implementation
at github.com/Aider-AI/aider/blob/main/aider/repomap.py is symbol-level.

Pre-requisites: B-1 (DKG runtime activation) confirmed SHIPPED.
Parallel-safe with: C-35, C-36, C-37, C-39, C-40 (orthogonal). NOT parallel-safe
with any C-N that mutates pipeline.rs gating logic.

Reference grounding (verified by prior oracle, 2026-05-17):
1. Aider's repomap.py direct read 2026-05-17 confirmed SYMBOL-LEVEL PageRank
   via NetworkX, self-loop weight=0.1 for definition-without-reference,
   50× chat-context multiplier. Entry points: get_repo_map, get_ranked_tags_map,
   get_ranked_tags. Path: github.com/Aider-AI/aider/blob/main/aider/repomap.py.
2. ELAI pre-B-1 had compute_repo_map at pipeline.rs:387 called but pipeline.dkg_engine
   gated on .elai/dkg.db existence which "never fires in production" per assessment §X.
   B-1 reportedly fixed the gating. C-38 verifies the fix held.
3. DETAILED_SUMMARY_01 §10 currently describes Aider's repomap as "file-level
   PageRank with 3 deliberate divergences." This is a docs-truth bug. The divergence
   list relies on a false premise (file-level) and must be re-audited.

## Context

Two debts, both rooted in the same area, batched into one C-N because the work
overlaps (read aider/repomap.py once, fix both).

**Debt 1: production-path proof**. Phase 1D PageRank was specced + tested at
24 tests but production wiring failed silently because the engine was gated
behind a database-existence check that never triggered. B-1 fixed the wiring.
A CONTRACT TEST is required to lock the fix — without it, a future refactor
could re-introduce the silent gate. The contract test asserts: (a) on any
prompt assembly path where the FSM state expects symbol-graph context,
compute_repo_map is invoked, (b) its result is non-empty given a non-empty
codebase, (c) the SSB emits a pagerank_run row.

**Debt 2: docs-truth correction**. DETAILED_SUMMARY_01 §10 describes Aider's
repomap as file-level. Direct read of aider/repomap.py 2026-05-17 confirms
symbol-level (NetworkX nodes are tag identifiers, not file paths). The §10
narrative about "3 deliberate divergences from Aider's file-level approach"
needs to be re-grounded against actual Aider behavior. Possible outcomes:
- the 3 divergences still hold but for different reasons (e.g. ELAI uses
  symbol-level over function-level instead of symbol-level over file-level)
- some of the 3 divergences are spurious and should be removed
- ELAI is closer to Aider than the prose currently claims

Either way, the spec should match reality. C-38 produces the corrected text
and a workflow/sessions/{id}/mistakes.md entry recording the discovery for
future spec-accuracy passes.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md §10 (Aider comparison section)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-1_dkg_runtime_activation.md
- .elai_cc/crates/orchestrator/src/pipeline.rs around line 387 (compute_repo_map call site, current state)
- .elai_cc/crates/knowledge/src/pagerank.rs (Phase 1D implementation)
- github.com/Aider-AI/aider/blob/main/aider/repomap.py (fresh fetch, do not trust prior reads)

## Tasks

1. **Fresh fetch of aider/repomap.py** — WebFetch the file at the URL above,
   compare get_ranked_tags / get_ranked_tags_map signatures against
   ELAI's compute_repo_map. Extract: graph node type, edge type, weighting
   scheme, self-loop policy, top-k selection, chat-context multiplier.

2. **Diff matrix authoring** — produce a per-attribute comparison table in
   workflow/sessions/{id}/aider-elai-pagerank-diff.md. Columns: attribute,
   Aider behavior, ELAI behavior, divergence-intent, divergence-impact.
   This artifact is the source-of-truth that DETAILED_SUMMARY §10 will cite.

3. **DETAILED_SUMMARY §10 rewrite** — replace the "file-level" claim with
   the verified symbol-level description. Rewrite the divergences list to
   match the diff matrix. Add a footnote: "Verified against
   github.com/Aider-AI/aider/blob/main/aider/repomap.py on YYYY-MM-DD via
   C-38 fresh fetch; see workflow/sessions/{id}/aider-elai-pagerank-diff.md."

4. **mistakes.md entry** — append a row recording the original spec error
   pattern: "Spec described external dependency behavior without fresh-fetch
   verification; bug pattern: stale-external-reference."

5. **Contract test (4D.X.1)** — tests/contracts/pagerank_production_contract.rs:
   - 5.a: synthetic 50-file codebase fixture under tests/fixtures/repomap_synth/
     with a deliberate hub file imported by 20 others; PageRank top-3
     should include the hub
   - 5.b: invoke the production query path (NOT the test-only direct call)
     and assert compute_repo_map fires (via SSB pagerank_run row, NOT via
     mock-inspection — must observe through the production side-effect)
   - 5.c: when .elai/dkg.db is ABSENT, compute_repo_map MUST trigger a
     transparent rebuild (not silently skip); SSB emits dkg_rebuild_started
     and dkg_rebuild_complete
   - 5.d: result top-k is deterministic across 10 repeated invocations
     against the same fixture

6. **SSB action types** — add: pagerank_run (existing? confirm), dkg_rebuild_started,
   dkg_rebuild_complete. Each row carries fixture_or_repo_id and elapsed_ms.
   SCHEMA_VERSION bump only if dkg_rebuild_* are new.

7. **Production wiring lock (4D.X.2)** — if the B-1 fix is intact, the
   gate logic at pipeline.rs:387 should now read something like:
   `let dkg = self.ensure_dkg_or_rebuild()?;` (no silent skip). Verify the
   actual code matches. If pipeline.rs:387 still has a silent-skip pattern,
   file a separate bug, do NOT close C-38; treat as evidence B-1 didn't
   actually land what it claimed.

8. **Documentation** — update DETAILED_SUMMARY_01 §10 (per Task 3) AND
   add a short "verifying external library claims" subsection to
   .claude/rules/token-discipline.md or equivalent rule file at the
   apps_workflow side referencing this pattern.

## WIRING_CONTRACT

produces_symbols:
  - tests/contracts/pagerank_production_contract.rs (new)
  - tests/fixtures/repomap_synth/* (synthetic codebase)
  - workflow/sessions/{id}/aider-elai-pagerank-diff.md (artifact)
  - SSB action_types: dkg_rebuild_started, dkg_rebuild_complete (new if not present)
  - Updated text in F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md §10
consumes_symbols:
  - compute_repo_map (knowledge crate)
  - pipeline.rs:387 production path
  - SSB event bus
reachable_from:
  - /explore command (production path, end-to-end)
  - test harness cargo test --test pagerank_production_contract
stub_ban:
  - contract test MUST observe the production SSB row, not a mock
  - if pipeline.rs:387 still has silent-skip pattern, C-38 closure is BLOCKED
    pending the actual fix
  - DETAILED_SUMMARY §10 update MUST cite the fresh-fetch date and the diff
    artifact path — drift-protection
frontend_pair: (none — knowledge layer)
observable_effect:
  trigger: cargo test -p elai_orchestrator --test pagerank_production_contract
  expect_ssb: pagerank_run rows for each invocation; dkg_rebuild_* rows when
              the fixture starts with absent dkg.db
  expect_artifact: aider-elai-pagerank-diff.md present and cross-referenced
                   in DETAILED_SUMMARY §10

## DETERMINISTIC_CHECK

```bash
# 1. Diff artifact exists
test -f workflow/sessions/$(cat workflow/sessions/current.txt)/aider-elai-pagerank-diff.md

# 2. DETAILED_SUMMARY §10 cites the artifact
grep -c "aider-elai-pagerank-diff" F:/.ELAI_workflow/docs/thefinalglance/summary/DETAILED_SUMMARY_01_overview_and_architecture.md

# 3. Contract test exists and passes
test -f .elai_cc/crates/orchestrator/tests/contracts/pagerank_production_contract.rs
cargo test -p elai_orchestrator --test pagerank_production_contract 2>&1 | grep "test result.*ok"

# 4. Fixture exists with deliberate hub
test -d .elai_cc/crates/orchestrator/tests/fixtures/repomap_synth
ls .elai_cc/crates/orchestrator/tests/fixtures/repomap_synth | wc -l | awk '$1 >= 50 {exit 0} {exit 1}'

# 5. SSB action types
rg "pagerank_run|dkg_rebuild_started|dkg_rebuild_complete" .elai_cc/crates/orchestrator/src

# 6. pipeline.rs:387 (or wherever compute_repo_map is called) does NOT have
#    the old silent-skip pattern
rg "if.*dkg\.db.*exists" .elai_cc/crates/orchestrator/src/pipeline.rs
# expect: NO matches, OR a match that is now gated by ensure_or_rebuild

# 7. mistakes.md has the stale-external-reference entry
grep "stale-external-reference" workflow/sessions/$(cat workflow/sessions/current.txt)/mistakes.md
```

## ACCEPTANCE

- [ ] Fresh fetch of aider/repomap.py recorded with timestamp
- [ ] aider-elai-pagerank-diff.md authored with full attribute table
- [ ] DETAILED_SUMMARY_01 §10 rewritten to match the diff artifact
- [ ] Footnote in §10 cites the artifact path + fetch date
- [ ] mistakes.md entry added: stale-external-reference pattern
- [ ] tests/contracts/pagerank_production_contract.rs implemented per
      tasks 5.a-5.d
- [ ] Synthetic fixture (50+ files, deliberate hub) present
- [ ] Contract test passes — top-3 PageRank includes hub
- [ ] Contract test passes — SSB pagerank_run observed on production path
- [ ] Contract test passes — absent-dkg triggers rebuild, not skip
- [ ] Contract test passes — top-k deterministic across 10 invocations
- [ ] If pipeline.rs:387 still has silent-skip: C-38 closure BLOCKED;
      file separate B-1-followup bug instead

## Files

- ADD: .elai_cc/crates/orchestrator/tests/contracts/pagerank_production_contract.rs
- ADD: .elai_cc/crates/orchestrator/tests/fixtures/repomap_synth/ (50+ synthetic files)
- ADD: workflow/sessions/{id}/aider-elai-pagerank-diff.md
- MODIFY: F:\.ELAI_workflow\docs\thefinalglance\summary\DETAILED_SUMMARY_01_overview_and_architecture.md §10
- MODIFY (if needed): .elai_cc/crates/orchestrator/src/pipeline.rs (gate fix if not already present)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (dkg_rebuild_* if new)
- MODIFY: workflow/sessions/{id}/mistakes.md (add stale-external-reference row)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-1_dkg_runtime_activation.report.md
  — annotate C-38 production-path verification on closure.
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-1D.report.md
  — annotate C-38 contract-test lock.

## Next

C-38 closure unlocks honest claims about ELAI's PageRank behavior in any
external communication (blog, paper submission, README). Until §10 is corrected,
any "ELAI improves on Aider via X" claim is grounded in a spec error and
should be treated as unverified.
