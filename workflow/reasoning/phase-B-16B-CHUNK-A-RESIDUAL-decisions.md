Phase: B-16B-CHUNK-A-RESIDUAL
Session: s-1327164
Date: 2026-04-29

What
  Close the gap discovered in Phase 1 audit: HANDOFF claimed B-16B Chunk A "done" for
  team_cron_registry.rs and worker_boot.rs but 21 .lock().unwrap_or_else(|p| p.into_inner())
  sites remain. Convert all 21 to .lock().expect("<rationale>") matching the W1 pattern.

Files
  .elai_cc/crates/orchestrator/src/team_cron_registry.rs (13 sites)
    lines: 69, 86, 91, 98, 109, 115, 156, 176, 181, 193, 202, 214, 227
  .elai_cc/crates/orchestrator/src/worker_boot.rs (8 sites)
    lines: 169, 205, 210, 349, 379, 437, 460, 485

Key decision
  Same as Phase 1 W1: option (a) panic-on-poison with .expect("<concrete rationale>") +
  ONE impl-level doc-comment per file explaining: poison = upstream invariant violated,
  panic loud per CLAUDE.md.

Expected test outcome
  cargo check --workspace: PASS, 0 new warnings.
  No semantic change beyond panic-on-poison.

Unknowns
  - Whether any of the 21 sites involve a tokio::sync::Mutex (which uses .await, not LockResult)
    — execution agent must skip those.
  - Whether any site is on the hot path where panic propagation breaks an invariant
    elsewhere — auditor verifies via cargo check.

Out of scope
  - cli/main.rs:945 and :3163 unrationaled unwrap_or_else (different file/pattern; logged
    as known-open in progress.md by Phase 1 memorize)
  - Any new file or new pattern type
