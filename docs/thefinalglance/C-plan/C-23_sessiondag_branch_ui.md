# C-23: SessionDag Branch UI — merge/close CLI + BranchTree mount + /continue wiring
Objective: complete the B-9 PARTIAL and MISSING items: add /session merge|close to the CLI
parser, mount BranchTree.svelte in SessionsDrawer, wire onfork/onmerge/onclose to real Tauri
commands, fix CommandWindow /continue to call load_handoff, fix parent_chain ordering bug,
add 100-run determinism test, verify SettingsWindow SessionHandoffEntry schema, and add
SSB-driven banners for branch operations.
Estimated effort: 1-2 sessions.
Pre-requisites: C-3, C-4 (Tauri commands path stable; AppState wiring proven).
Parallel-safe with: C-21, C-22, C-24.

## Context

B-9 shipped the SessionDag producer and BranchTree.svelte component but left six concrete gaps
confirmed by the B-9 verification report (B-9_sessiondag_activation.report.md, overall:
MOSTLY_DONE):

1. /session merge <branch_id> and /session close <branch_id> are absent from
   parse_session_command (commands.rs:1671). Only list/switch/fork are parsed.
2. BranchTree.svelte exists (dashboard-v2/src/lib/ui/BranchTree.svelte) with onfork/onmerge/
   onclose props but is not imported in any live Svelte screen (elai.map shows only self-refs).
3. fork_session_branch/merge_session_branch/close_session_branch Tauri commands
   (commands.rs:1853-1880) return CLI-delegation errors. The SessionDag is not in AppState.
   Comment: "B-9 DEFERRED: dag-write-in-dashboard".
4. CommandWindow.svelte executeSlash('/continue') only prepopulates input with "/continue ";
   it does not call load_handoff (confirmed: executeSlash at CommandWindow.svelte:253 only
   sets textarea content, no Tauri invocation).
5. parent_chain returns tip-to-root order. Spec (phase-3H.txt:232) says root-to-tip. Test at
   session_dag.rs:1966-1970 asserts chain[0]="snap-leaf" (tip), chain[2]="snap-root" — wrong
   order. Fix: chain.reverse() before Ok(chain) at session_dag.rs:861.
6. 100-run determinism test is absent. Only a 2-seed comparison exists (session_dag.rs:1803-1810).
   phase-3H.report.md confirms MISSING.

## Reads (mandatory)

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-9_sessiondag_activation.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-3H.report.md
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-3H.txt (section 3H.2 parent_chain, 3H.4 branch ops, 3H.6 handoff)

## Tasks

1. Add /session merge <branch_id> to parse_session_command
   File: .elai_cc/crates/cli/src/commands.rs:1671 (parse_session_command fn)
   Add arms: ["merge"] => usage_error, ["merge", target] => SlashCommand::Session with
   action="merge" + branch_id=Some(target). Mirror ["fork", target] pattern at line 1696.
   Also add ["close"] and ["close", target] arms for close action.
   Update the argument_hint in SlashCommandSpec (currently "[list|switch <session-id>|fork
   [branch-name]]") to include merge and close variants.
   Handle both commands in the live REPL dispatch (handle_slash_command match for
   SlashCommand::Session — verify action routing in commands.rs:3959 area).

2. Replace CLI-delegation stubs in Tauri commands
   File: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs:1851-1880
   Remove deferred comment "B-9 DEFERRED: dag-write-in-dashboard".
   Implement fork_session_branch: resolve SessionDag DB path from AppState
   (mirror get_session_branches pattern at commands.rs:1806), call dag.create_branch(name),
   emit SSB Checkpoint row via existing ssb_writer in AppState.
   Implement merge_session_branch: call dag.merge_branch_with_ssb(branch_id, ssb_writer).
   Implement close_session_branch: call dag.close_branch(branch_id), emit SSB Checkpoint row.
   All three must return typed DagBranchError, not raw string errors.

3. Import and mount BranchTree.svelte in SessionsDrawer
   File: .elai_cc/crates/dashboard-v2/src/lib/drawers/SessionsDrawer.svelte
   Add import at top of <script>: import BranchTree from '../ui/BranchTree.svelte';
   Mount below the existing branch list section (or replace the branch table placeholder).
   Pass branches={$sessionBranches} (derived from drawers.svelte.ts get_session_branches call).
   Wire onfork={(name) => invoke('fork_session_branch', {branchId: name})}
        onmerge={(id) => invoke('merge_session_branch', {branchId: id})}
        onclose={(id) => invoke('close_session_branch', {branchId: id})}
   On command success: refresh branch list via drawers.svelte.ts invalidation.

4. Wire /continue in CommandWindow to call load_handoff
   File: .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
   In executeSlash (line 253): add case for cmd === '/continue' — instead of prepopulating
   input, call invoke('load_handoff') then on success push a status card confirming branch
   resumed; on error fall back to the current prepopulate behavior with an error card.
   Import load_handoff from stub.ts (or add a direct invoke call if not already stubbed).
   Verify stub.ts exposes load_handoff; if absent, add the stub entry there first.

5. Fix parent_chain root-to-tip ordering
   File: .elai_cc/crates/orchestrator/src/session_dag.rs:861 (end of parent_chain fn, after
   the while loop, before Ok(chain))
   Add: chain.reverse();
   Update test at session_dag.rs:1966-1970 to assert chain[0]="snap-root", chain[2]="snap-leaf".
   Update the full lifecycle integration test at session_dag.rs:3319 if it asserts chain order.

6. Add 100-run determinism test
   File: .elai_cc/crates/orchestrator/src/session_dag.rs (internal #[cfg(test)] module)
   or .elai_cc/crates/contracts/tests/session_dag_contract.rs
   Prefer contracts file per C-18 pattern.
   Test: seed a SeededRng with fixed value, run create_handoff + load_handoff 100 times in a
   loop on identical input; assert all snap_ids and trim output are identical across iterations.
   Check C-18 first — if it already covers determinism with 100 iterations, document DONE.

7. Verify SettingsWindow SessionHandoffEntry schema
   File: .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
   Grep for SessionHandoffEntry, HandoffEntry, or handoff_entry field refs. If no stale V1
   fields (e.g. chat_window_state) exist: document DONE in workflow/reasoning/C-23-decisions.md.
   If stale refs found: align field names to the Rust SessionHandoffEntry struct
   (session_dag.rs type definitions area).

8. Add SSB-driven banners for branch operations
   Produce SSB action_types branch_forked, branch_merged, branch_closed in the Tauri command
   implementations (Task 2). These must emit via the existing ssb_writer AppState handle
   (mirror checkpoint emit in merge_branch_with_ssb at session_dag.rs:1263-1312).
   In dashboard-v2, add ActivityWindow or SessionsDrawer renderer entries for these three
   new action_types. Use the existing branch_merge SSB type as the pattern (ssb.rs:465).

## WIRING_CONTRACT

produces_symbols:
  - SlashCommand::Session action="merge" and action="close" variants in parse_session_command
  - fork_session_branch, merge_session_branch, close_session_branch Tauri commands (real impl)
  - BranchTree mounted in SessionsDrawer with live IPC wiring
  - /continue -> load_handoff Tauri invocation in CommandWindow.svelte
  - parent_chain root-to-tip ordering (chain.reverse() fix)
  - SSB action_types: branch_forked, branch_merged, branch_closed
consumes_symbols:
  - SessionDag::create_branch, SessionDag::merge_branch_with_ssb, SessionDag::close_branch
    (session_dag.rs:1021, 1263, 1314)
  - SessionDag::load_handoff (session_dag.rs:1581)
  - get_session_branches Tauri command (commands.rs:1806) — already wired in drawers.svelte.ts:264
  - AppState ssb_writer handle (B-9 DEFERRED comment at commands.rs:1851 — requires AppState
    to expose SessionDag db path or a dag handle)
  - BranchTree.svelte onfork/onmerge/onclose props (already defined)
reachable_from:
  - CLI: /session merge <branch_id>, /session close <branch_id>
  - Dashboard: SessionsDrawer BranchTree component
  - Dashboard: CommandWindow /continue button -> load_handoff
stub_ban:
  - fork_session_branch/merge_session_branch/close_session_branch MUST NOT return
    "use CLI: /session fork" error strings after this plan; that stub pattern is the
    explicit target of Task 2
  - executeSlash('/continue') MUST NOT remain a textarea-prepopulate-only handler
frontend_pair:
  SessionsDrawer.svelte <-> fork_session_branch/merge_session_branch/close_session_branch
  CommandWindow.svelte <-> load_handoff
observable_effect:
  trigger: /session fork foo, then /session merge foo, then /session close foo via CLI
  expect_ssb: SSB Checkpoint rows emitted for each operation; branch_forked/branch_merged/
              branch_closed rows visible in ActivityWindow
  expect_artifact: BranchTree in SessionsDrawer reflects each state change; /continue in
                   CommandWindow triggers load_handoff (not textarea prepopulate)

## DETERMINISTIC_CHECK

```bash
# 1. merge and close arms in parse_session_command
rg "merge.*branch_id|close.*branch_id|action.*=.*merge|action.*=.*close" \
  .elai_cc/crates/cli/src/commands.rs

# 2. Tauri stub pattern gone from fork/merge/close commands
rg "use CLI: /session" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: zero hits

# 3. BranchTree mounted in SessionsDrawer
rg "BranchTree" .elai_cc/crates/dashboard-v2/src/lib/drawers/SessionsDrawer.svelte
# Expected: import line + mount line

# 4. /continue calls load_handoff (not just prepopulate)
rg "load_handoff" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
# Expected: at least one invoke('load_handoff') hit

# 5. parent_chain reverse fix
rg "chain.reverse()" .elai_cc/crates/orchestrator/src/session_dag.rs
# Expected: one hit inside parent_chain fn

# 6. parent_chain test order (root-to-tip)
rg "snap-root.*chain\[0\]|chain\[0\].*snap-root" .elai_cc/crates/orchestrator/src/session_dag.rs
# Expected: test assertion with root at index 0

# 7. 100-run determinism test
rg "100.*determinism|determinism.*100|for.*0\.\.100" \
  .elai_cc/crates/orchestrator/src/session_dag.rs \
  .elai_cc/crates/contracts/tests/session_dag_contract.rs
# Expected: at least one loop iterating 100 times on dag operations

# 8. SSB branch action_types
rg "branch_forked|branch_merged|branch_closed" .elai_cc/crates/orchestrator/src/

# 9. Contract tests pass (no regression on session_dag_activation tests a-k)
cargo test -p elai_orchestrator --test session_dag_activation 2>&1 | grep "test result"
cargo test -p elai_contracts --test session_dag_contract 2>&1 | grep "test result"
```

## ADVERSARIAL_CHECKS

- AppState may not expose a SessionDag handle or db path. Verify commands.rs AppState fields
  before Task 2. If absent, add a dag_db_path: Option<PathBuf> field to AppState and populate
  it at startup from elai_cc.toml session.dag_db_path. Do not hardcode a path.
- merge_branch_with_ssb is async (session_dag.rs:1263). The Tauri command must be #[tauri::command]
  async fn. Confirm existing fork/merge/close stubs are already async; if sync, convert.
- BranchTree onfork/onmerge/onclose are Svelte callback props. In Svelte 5 (runes mode),
  verify BranchTree uses $props() and that SessionsDrawer passes functions correctly; mismatched
  prop style (Svelte 4 vs 5) is a known cross-component issue in the dashboard.
- parent_chain fix changes observable output. Any caller that currently relies on tip-to-root
  order (e.g., the full lifecycle test at session_dag.rs:3319 chain assertion) will break.
  Grep all chain assertion sites before committing the fix.
- Determinism test must use a concrete SeededRng implementation. Verify EntropySource trait
  has a seeded constructor (session_dag.rs:9-10 imports); if not, use a fixed UUID list.

## ACCEPTANCE

- [ ] /session merge <branch_id> parsed correctly; /session close <branch_id> parsed correctly;
      both return SlashCommandParseError::Usage on missing arg
- [ ] fork_session_branch/merge_session_branch/close_session_branch return real results
      (not "use CLI: /session" errors); each emits SSB Checkpoint row
- [ ] BranchTree visible in SessionsDrawer in live dashboard; clicking fork/merge/close buttons
      triggers respective Tauri commands
- [ ] CommandWindow /continue button calls load_handoff Tauri command; status card shown on
      success; fallback error card on failure
- [ ] parent_chain returns [snap-root, snap-mid, snap-leaf] for a 3-level chain (root at [0])
- [ ] 100-run determinism test passes: same seed produces identical snap_ids and trim output
      across 100 iterations (or documented as covered by C-18)
- [ ] SettingsWindow SessionHandoffEntry field refs verified: no stale V1 fields present
- [ ] SSB action_types branch_forked, branch_merged, branch_closed rendered in dashboard
- [ ] cargo test -p elai_orchestrator --test session_dag_activation: all tests PASS (no
      regression from parent_chain reorder)
- [ ] cargo test -p elai_contracts --test session_dag_contract: all tests PASS
- [ ] Cloud Sonnet smoke test:
      Run `elai /task --mode cloud "implement and refactor a long-running task that touches 20
      files"` until >85% context budget; assert SSB row create_handoff fires.
      In new terminal: run /continue; assert SessionDag::load_handoff fires (SSB row
      load_handoff); BranchTree shows resumed branch with root-to-tip parent_chain.
      Run /session fork foo, /session merge foo, /session close foo; assert each emits SSB
      Checkpoint row and BranchTree reflects each state change.
      Dashboard: BranchTree visible in SessionsDrawer; fork/merge/close buttons functional.

## Files touched

- MODIFY: .elai_cc/crates/cli/src/commands.rs (parse_session_command merge+close arms,
  argument_hint update, REPL dispatch)
- MODIFY: .elai_cc/crates/orchestrator/src/session_dag.rs (parent_chain chain.reverse(),
  test order assertions, 100-run determinism test)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (real fork/merge/close
  implementations, SSB branch action_type emit)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/SessionsDrawer.svelte (import +
  mount BranchTree, IPC wiring)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (/continue ->
  load_handoff invocation)
- MODIFY or VERIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
  (Task 8 schema check)
- MAYBE ADD: .elai_cc/crates/contracts/tests/session_dag_contract.rs (100-run determinism
  test, if not in C-18)
- CREATE: workflow/reasoning/C-23-decisions.md

## Next

C-24 (if parallel; no ordering dependency) or C-18 (test-infra backfill) if determinism test
deferred there.
