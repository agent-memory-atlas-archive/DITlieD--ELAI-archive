Phase: UUID-v4-FINISH
Session: s-1327164
Date: 2026-04-29

What
  Replace 4 production Uuid::new_v4() call sites with explicit getrandom-fed UUID builds.
  Quarantines entropy: the v4 feature on uuid pulls a transitive entropy source we want to
  declare directly per workspace policy.

Sites (verified by context-compiler; line numbers updated +1 from HANDOFF):
  .elai_cc/crates/agent/src/main.rs:301
  .elai_cc/crates/orchestrator/src/sandbox.rs:600
  .elai_cc/crates/orchestrator/src/sandbox.rs:1059
  .elai_cc/crates/orchestrator/src/task_runner.rs:5760

Cargo.toml changes
  .elai_cc/crates/agent/Cargo.toml: add `getrandom = "0.3"` direct dep
  .elai_cc/crates/orchestrator/Cargo.toml: add `getrandom = "0.3"` direct dep
  Workspace .elai_cc/Cargo.toml: KEEP `features = ["v4"]` on uuid — test fixtures still use
    Uuid::new_v4(), CLAUDE.md test exception applies.

Key decision (pattern)
  Use uuid::Builder::from_random_bytes (NOT Uuid::from_bytes) so the resulting UUID has
  RFC 4122 v4 version/variant bits set. HANDOFF's literal Uuid::from_bytes(b) example
  would produce a non-v4-shaped UUID — that is a behavior change for any downstream code
  that parses the version field. Builder::from_random_bytes is the correct uuid-crate
  primitive for this conversion.

Pattern for Result-returning fns:
    let mut b = [0u8; 16];
    getrandom::fill(&mut b)?;            // getrandom 0.3 API; verify exact name
    Uuid::Builder::from_random_bytes(b).into_uuid()

Pattern for non-Result fns:
    let mut b = [0u8; 16];
    getrandom::fill(&mut b)
        .expect("getrandom: OS entropy source unavailable; process cannot continue");
    Uuid::Builder::from_random_bytes(b).into_uuid()

Per-site fn-return classification (execution agent verifies):
  agent/src/main.rs:301 — likely main fn or close; classify before edit.
  sandbox.rs:600, 1059 — context-compiler reports these may be non-Result.
  task_runner.rs:5760 — verify return type.

API verification: getrandom 0.3 renamed `getrandom()` to `fill()`. Execution agent must
verify the exact name (grep existing usage in workspace, or read getrandom-0.3.x docs in
target/doc/, or Cargo.lock for exact version → known API). If the API is `getrandom`, use
that instead. Auditor's cargo check catches mismatches.

Imports
  Each .rs file: ensure `use uuid::{Uuid, Builder};` (or full path Uuid::Builder; check
  current imports first). Add `getrandom` not needed if used as `getrandom::fill(...)`.

Drop TODO comments
  Each of the 4 sites has a TODO comment from B-RESIDUALS marking the deferral. Remove or
  update those comments after conversion.

Expected test outcome
  cargo check --workspace: PASS, 0 new warnings.
  No semantic change beyond the entropy provider declaration.

Unknowns
  - Exact getrandom 0.3 API name (`fill` vs `getrandom`) — execution agent verifies.
  - Whether agent crate has any other entropy uses to convert opportunistically (no — out of
    scope; only the 4 sites).

Out of scope
  - Test code Uuid::new_v4() usage (workspace v4 feature kept for tests).
  - Any other Uuid usage pattern.
  - Dropping the uuid v4 feature.
