B-16B — Decision Doc
====================
Phase: B-16B (.unwrap()/.expect() → ? cleanup)
Session: s-330056

WHAT
----
Convert production-path .unwrap() and .expect() to ? / Result/Option propagation.
Re-baselined via context-compiler enumeration: ~114 true production hits (NOT 1381 as
HANDOFF.md estimated — that was raw unclassified count; ~95% are test/LazyLock/main
exempt per CLAUDE.md).

CHUNK PLAN (re-baselined):
  Chunk A (LOW, this session)    Mutex registries: team_cron + task_registry + worker_boot
                                  ~32 hits; pattern: .unwrap_or_else(|p| p.into_inner());
                                  no signature change.
  Chunk D (LOW, this session)    Skip 28 LazyLock<Regex> static init (CLAUDE.md exception);
                                  convert 8 inline fn-body hits in tiered_analysis.rs.
  Chunk B (LOW-MEDIUM, this session)  mcp_bridge + cli + lane_events; ~27 hits;
                                       no signature change expected.
  Chunk C (MEDIUM, NEXT SESSION)  deterministic_auditor + dkg + misc; ~12 hits;
                                   3 inline regex::Regex::new().unwrap() in non-Result fns
                                   require caller-chain signature change. NEEDS USER APPROVAL.
                                   DEFERRED to next session HANDOFF.

KEY DECISIONS
-------------
D1. This session executes Chunks A+D (1 agent) and Chunk B (1 agent) sequentially.
D2. Chunk C deferred for explicit user approval (signature changes cascade).
D3. NO cargo runs during execution; auditor runs ONE cargo check at end.
D4. Mutex.lock() poisoned-pattern: .unwrap_or_else(|p| p.into_inner()) (recover-on-poison)
    is the canonical fix; matches the "tokio::sync::Mutex in async / std::sync::Mutex only
    if provably sync-only" rule because these are sync-only registries.
D5. LazyLock<Regex>::new(...).unwrap() in static init: per CLAUDE.md "const/static init
    with provably valid literals" exception, leave as-is. Add `// CLAUDE-md-EXCEPTION:
    static-init-with-literal` comment if the auditor flags it.
D6. .expect("rationale") with strong rationale (e.g. "previously checked at line N", or
    "literal regex compile") stays. Weak rationale gets either a stronger reason or a
    proper conversion. Agent decides per-site, conservative default = strengthen rationale.
D7. core/json.rs custom .expect() is a parser method, NOT std Result — do NOT touch.

EXPECTED TEST OUTCOMES
----------------------
A1. cargo check --workspace PASS (single auditor run at end).
A2. ~67 production hits converted across Chunks A+D+B.
A3. ~28 LazyLock<Regex> sites annotated (exception comments) but not converted.
A4. No new warnings beyond the pre-existing 2.
A5. Behavior preserved (Mutex poison recovery is benign for these registries; original
    .expect("poisoned") would have panicked anyway).

UNKNOWNS
--------
- Whether any Mutex registry call site actually depends on panic-on-poison as a defense
  invariant. If yes, conversion to .unwrap_or_else(|p| p.into_inner()) silently masks the
  bug. Mitigation: agent reads the surrounding fn doc/comments; if doc says "panics on
  poison by design", leaves the .expect with strengthened rationale instead.

ROLLBACK
--------
git restore on touched files.

CONSTRAINTS
-----------
- NO commits.
- NO cargo runs during execution (only auditor at end).
- 2 sequential execution agents (A+D combined, then B). No parallel.
- Chunk C explicitly DEFERRED.
