Phase: B-16B-FOLLOWUPS (W1 + W2)
Session: s-1327164 (continuation of s-330056)
Date: 2026-04-29

What
  W1: TaskRegistry + McpToolRegistry .lock().unwrap_or_else(|p| p.into_inner()) → .lock().expect("...")
      Files: .elai_cc/crates/orchestrator/src/task_registry.rs (11 sites)
             .elai_cc/crates/orchestrator/src/mcp_tool_bridge.rs (10 sites)
  W2: cli/main.rs:3143, 3176, 3184 — add rationale comments citing BuiltRuntime field setter site
      File: .elai_cc/crates/cli/src/main.rs
  W3: DEFERRED to B-15D (cli/input.rs:114 LineEditor::new — bootloader chain blast radius)

Key decision (W1)
  Option (a) chosen: .lock().expect("invariant: <registry> rejects partial writes; mutex poison = upstream bug, panic loud")
  Rejected option (b) [pre-return integrity check]: requires per-site analysis of what constitutes a "half-record" for each fn; not safely doable without user input on registry semantics; would also cascade scope.
  Rationale:
    1. std::sync::Mutex poison happens when a holder panics. For a registry, a poisoned writer means upstream invariant already broken — recovery is risky.
    2. unwrap_or_else(|p| p.into_inner()) silently absorbs the poison and continues with possibly-corrupt state — exactly the "silent half-write" risk W1 calls out.
    3. CLAUDE.md "panic = you'll see it immediately" semantic applies for critical infrastructure invariants.
    4. The s-330056 author originally had .expect() and relaxed to unwrap_or_else for B-16B Chunk A/B; W1 reverses that relaxation with stronger rationale doc-comments.

Key decision (W2)
  3 .expect() calls reference Option fields populated by BuiltRuntime construction (cli/main.rs:3116). Add a /// doc-comment block above each .expect() that cites the BuiltRuntime setter line. ~3 LOC each.

Expected test outcome
  cargo check --workspace: PASS, 0 new warnings.
  cargo test --no-run: PASS (no semantic change beyond panic-on-poison).
  No runtime test runs needed (per "minimize cargo" constraint).

Unknowns
  - Whether any callers of TaskRegistry/McpToolRegistry methods rely on the silent-poison behavior (i.e., they call into a poisoned registry and expect it to keep working). Auditor should grep for any retry/recovery pattern.
  - Whether the 21 sites enumerated in context-package-phase1.md exactly match — auditor verifies.

Out of scope
  - team_cron_registry.rs, worker_boot.rs (already done in s-330056 Chunk A; not in W1)
  - cli/input.rs:114 (W3, deferred)
  - Any cargo run by execution agent (auditor only, ONE pass)
