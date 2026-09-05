B-RESIDUALS — Decision Doc
==========================
Phase: B-RESIDUALS (small known-open follow-ups + 3 B-16E WARNs)
Session: s-330056

WHAT
----
Bundle of 7 applicable small fixes (2 of original 9 are FPs, drop):
  Item 1: ssb.rs:2328-2334 epoch try_from-ize for consistency with workload_profiling.rs:40
  Item 2: context_budget.rs:904-907 add debug_assert!(dynamic_budget < (1usize << 53))
  Item 3: dkg/mod.rs 5 child_count loop bounds → u32::try_from(...).unwrap_or(u32::MAX)
          (sites: 1256, 1438, 1577, 1722, 1726)
  Item 5: uuid v4 entropy-quarantine bypass (6 production hits) — replace Uuid::new_v4()
          with getrandom + Uuid::from_bytes; do NOT modify workspace Cargo.toml feature
          gate yet (defer feature drop to a later sweep so any remaining call sites
          would still compile)
  Item 6: crates/tools/src/dispatcher.rs:373-377 UnusedWarning pattern bug — add
          "warning:" prefix gate to 3 patterns
  Item 7: task_runner.rs:4230-4239 redundant `if let Some(first_file)` guard — replace
          with .first().expect("non-empty; checked above")
  Item 8: 3 list fns return unordered Vec from FxHashMap (TeamRegistry::list,
          CronRegistry::list, McpToolRegistry::list_servers) — add stable sort_by

DROPPED (false positives per context-compiler):
  Item 4: import_extractor.rs only has HashMap inside r#"..."# test fixture strings
  Item 9: stage_onnx_models.py:170 already SIM103-compliant

KEY DECISIONS
-------------
D1. Single execution agent for all 7 items (~27-31 Edits across 7 files). Sequential within
    the agent; no parallel.
D2. Item 7's .expect("non-empty; checked above") is acceptable per CLAUDE.md no-unwrap rule
    because the `is_empty()` check above is the proof of validity. Test/main/const init
    exception applies to provably-checked invariants like this.
D3. NO cargo runs during execution per user instruction (cargo lags PC). Auditor runs ONE
    final cargo check --workspace at end.
D4. Do NOT drop uuid v4 feature in this phase. Just convert call sites. Feature drop is
    a separate small phase later (touches workspace Cargo.toml; bigger blast radius).
D5. NO test changes. NO refactors. NO new deps.

EXPECTED TEST OUTCOMES
----------------------
A1. cargo check --workspace PASS (single auditor run, end of phase).
A2. cargo test --no-run -p elai-orchestrator -p elai-knowledge -p elai-tools — auditor
    runs ONLY if check passes and only if there's reason to suspect test compile breakage.
A3. No new compiler warnings beyond the pre-existing 2.
A4. Behavior preserved (deterministic-list change: list ordering becomes stable; any
    test asserting current undefined ordering would break — auditor must check).

UNKNOWNS
--------
- Whether sandbox.rs / task_runner.rs Uuid::new_v4 sites are inside Result-returning fns
  (preferred to use ? on getrandom error) vs need to fallback to Uuid::nil() / panic.
  Execution agent must inspect each site's surrounding fn and choose the minimal pattern.

ROLLBACK
--------
git restore on touched files.

CONSTRAINTS
-----------
- NO commits.
- NO cargo runs during execution.
- Single execution agent + single auditor.
