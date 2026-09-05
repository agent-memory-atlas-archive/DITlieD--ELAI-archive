B-16 Anti-Stale Enforcement -- Followup (regression + out-of-scope items)
==========================================================================

Session: s-25120 (completion-drive 1) closed B-16 against the v2 shipped
state and rebaselined the plan. This file captures the residual items
that COULD NOT be closed in B-16's scope and routes them to follow-up
phases. Mirrors the B-15-followup.md format.

Why a followup file
--------------------

B-16 was written against the 2026-04-26 anti-stale audit snapshot. The
core ship hit happened across B-16A (Box<dyn Error> removal in knowledge),
B-16B (.unwrap chunks A+B+D), B-16D (as-cast CAT-A categorisation),
B-16E (cast scan classification + HIGH-risk excludes). Between then and
2026-05-22, B-3 / B-4 / B-7 producer-wire ships re-introduced raw-grep
.unwrap() count growth (test code growth dominates; see (1) below).

The right move is not to expand B-16 scope mid-flight to re-baseline
the lint count -- that would conflate test-code growth (expected,
exempt) with prod-code regression (would be a real violation). Per-file-
cfg-test-aware audit this session confirms ZERO new production unwrap
violations were introduced; the regression is bookkeeping not behaviour.

(1) .unwrap() raw-count regression (test-code growth, NOT prod violation)
==========================================================================

Scope: scripts/check_anti_stale.sh REPORT count for .unwrap() grew from
1387 (s-91480 2026-05-09) to 1589 (s-25120 2026-05-22). Delta = +202 in
13 days = ~15.5 sites/day.

Per-file-cfg-test-aware audit (this session): ZERO new production
violations. ALL growth is in #[cfg(test)] mod tests blocks that the
bash grep cannot scope per-file (grep only excludes by PATH, not by
in-file cfg-test boundary).

Top producer-wire attribution (by file, with prod_unwraps measured as
hits BEFORE the first #[cfg(test)] line):

  B-3 wires (ssb + navigator producer):
    - orchestrator/session_dag.rs   raw=171  prod=0  (171 in test block from L1706)
    - orchestrator/ssb.rs           (numerous test-block hits, no prod)
    - orchestrator/plan_skeleton.rs raw=156  prod=0  (156 in test block from L1180)
    - orchestrator/mistake_book.rs  raw=40   prod=0  (40 in test block from L155)

  B-4 wires (rules + research producer):
    - rules/contract_tests.rs       raw=60   (entire file is tests by name)
    - rules/rule_engine.rs          raw=56   prod=0  (test block from L890)
    - rules/lifecycle.rs            raw=53   prod=0  (test block from L572)
    - rules/self_discovery.rs       raw=44   prod=0  (test block from L828)
    - rules/predicate.rs            raw=31   prod=0  (test block from L496)

  B-7 wires (knowledge + reranker producer):
    - knowledge/reranker.rs         raw=47   prod=1  (1 = static LazyLock-regex exemption)
    - knowledge/freshness.rs        raw=42   prod=0  (test block from L396)
    - knowledge/degradation.rs      raw=31   (test block dominant)
    - knowledge/vector/cache.rs     raw=25   (test block dominant)
    - knowledge/vector/hnsw_index.rs raw=27  (test block dominant)
    - knowledge/index_version.rs    raw=23   (test block dominant)

  Plugins T2/T3 lint (own LazyLock-regex pattern set):
    - plugins/tiered_analysis.rs    raw=85   prod=37 (ALL 37 are static LazyLock<Regex>
                                                       in default_patterns()/etc with
                                                       CLAUDE-md-EXCEPTION comments)

The fix shape per producer: NONE NEEDED at the producer level. Test code
.unwrap() is CLAUDE.md-exempt by design (no panic risk in unit tests).
The actionable item is at the LINT level only.

Two paths to close the bookkeeping gap:

  Path A -- lint refinement (low effort, high value):
    Teach check_anti_stale.sh to per-file-scope #[cfg(test)] boundaries
    when counting .unwrap(). Approach: for each .rs file, find the first
    `^#\[cfg(test)\]` line and only count unwraps BEFORE that line.
    Bash impl is ~15 lines. Risk: false-negatives if a file has
    multiple cfg(test) blocks (rare; current first-occurrence heuristic
    catches the common-case). Refinement: post-process the raw scan
    output through an awk-stage that splits per-file then per-line.

  Path B -- shifted lint surface (deferred, more work):
    Move the .unwrap() count from bash grep to the T2 AST lint at
    plugins/tiered_analysis.rs (which already has tree-sitter cfg(test)
    awareness via is_test_file + scope traversal). Cost: ~1-2 hours
    plugin work + new SsbAction surfaces. Defer to a future B-N.

Recommended owner: a new B-16-lint-refinement plan (path A is the right
trade). NOT a B-16 closure item -- moves the regression from "raw count
growth" framing to "per-file-cfg-test-aware count" framing.

(2) ort 2.0.0 stable publish status (PENDING-UPSTREAM)
========================================================

Scope: Cargo.toml pins in knowledge:54 + orchestrator:62 remain on
"=2.0.0-rc.12" because ort 2.0.0 stable is STILL not on crates.io as of
2026-05-22 (last WebSearch verify 2026-05-09; rc.12 itself was published
2026-03-05).

Waiver: workflow/reasoning/B-16A-decisions.md DECISION-A active. Lint
ORT_FAIL=0 while waiver file exists.

Bump trigger checklist (when stable ships):
  1. WebSearch confirms ort 2.0.0 (no -rc suffix) on crates.io.
  2. Update knowledge/Cargo.toml:54        ort = "=2.0.0-rc.12" -> ort = "2"
  3. Update orchestrator/Cargo.toml:62     ort = { version = "=2.0.0-rc.12", ... } -> ort = { version = "2", ... }
  4. cargo update -p ort
  5. cargo check -p elai-knowledge -p elai-orchestrator (fix any API breaks).
  6. Delete B-16A-decisions.md DECISION-A waiver block.
  7. Remove ORT-PENDING markers from check_anti_stale.sh + .bat output.

Not actionable until upstream ships. Re-check cadence: monthly
WebSearch (next: 2026-06-22).

(3) Future-lint extension (CLAUDE.md rules not yet covered by lint)
=====================================================================

CLAUDE.md Anti-Stale-Training block lists rules beyond what
scripts/check_anti_stale.sh currently enforces. These are deferred to a
future B-N (anti-stale-lint-extension):

  - rand >= 0.9 (never 0.8)
      Lint shape: grep Cargo.toml for `rand = "0.8` / `rand = "0`
  - getrandom >= 0.3
      Lint shape: grep Cargo.toml for `getrandom = "0.2` / `getrandom = "0.1`
  - native async fn in traits (never async-trait crate)
      Lint shape: grep .rs for `use async_trait::` (zero hits today;
      no lint enforces the constraint going forward)
  - deadpool/bb8 over r2d2
      Lint shape: grep Cargo.toml for `r2d2 = `
  - globset/ignore over glob
      Lint shape: grep Cargo.toml for `^glob = ` (the `glob` crate, not
      `globset`)
  - tokio::sync::Mutex in async context vs std::sync::Mutex sync-only
      Lint shape: context-sensitive (cannot grep deterministically; AST
      tier with await-traversal needed)
  - tracing-vs-log macro usage (use log:: is enforced; tracing macro
      usage in code already migrated, no enforcement to PREVENT future
      backsliding)

Recommended treatment: extend check_anti_stale.sh with a Cargo.toml
scan pass (rand/getrandom/r2d2/glob version pins) + a .rs scan pass
(async_trait import). The Mutex async/sync split needs AST traversal
and belongs in the plugins T2 tier.

Owner: a new B-16-lint-extension plan. Estimated effort: 1 session for
the Cargo.toml scans + lint script tightening + bench/contract tests.

(4) Dashboard-v2 BaselineRunCompleted + DeterministicAuditRun
================================================================

Out-of-scope for B-16 but flagged for visibility:

B-3 shipped SsbActionType variants DepAuditCompleted + BaselineRunCompleted +
DeterministicAuditRun + RuleTransition in crates/orchestrator/src/ssb.rs.
Only RuleTransition has a TS mirror in dashboard-v2/src/lib/api/types.ts
(line 2245). DepAuditCompletedPayload, BaselineRunCompletedPayload, and
DeterministicAuditRunPayload exist in Rust but no TS mirror exists, so
AntiStalenessDrawer.svelte cannot render them.

Recommended treatment: dashboard-followup plan to add TS row mirrors
+ AntiStalenessDrawer panels for the missing three payloads. Estimated
effort: 1 session. NOT a B-16 item (B-16 ships the rule-enforcement
events; dashboard rendering is a separate consumer wire).

End of followup doc.
