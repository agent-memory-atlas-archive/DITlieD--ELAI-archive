B-PREP — Decision Doc
======================
Phase: B-PREP (preflight prep before B-STAGE-MODELS and PART B carryover)
Session: s-27276
Owner: orchestrator (this session)

WHAT
----
Three small, independent changes bundled into one FSM cycle:
1. Run cargo run --example measure_4e_tree → close phase 4E by appending [decomp_tree]
   (baby_steps_per_idea, bubble_up_rate, context_p50) to .elai_cc/metrics/phase_4E.toml.
2. Resolve ONNX path discrepancies P1 (models/ → models/onnx/ for graphcodebert + granite)
   and P2 (cross-encoder file name standardised to cross-encoder-l12.onnx; 4 edit sites total).
3. Declare cargo deps used by later phases:
   - nix 0.31 (features: mount, sched, signal, fs) under
     [target.'cfg(target_os = "linux")'.dependencies] in crates/orchestrator/Cargo.toml.
   - lru 0.16 in crates/plugins/Cargo.toml.
No nix / lru USAGE in this phase — only declarations so B-18B and B-19B can build on them.

KEY DECISIONS
-------------
D1. Approve 4E closure via measurement run rather than re-engineering the binary.
    The 779-LOC binary is already real code; the only gap is its non-execution.
D2. Spec wins over CLI on ONNX paths (P1, P2). User confirmed.
D3. nix dep gated to Linux target at Cargo.toml level because the "mount" feature is not
    Linux-gated inside nix itself (verified via nix master Cargo.toml). Cross-platform compile
    safety is the reason.
D4. lru placed in crates/plugins (the crate that owns the Ssb cache eviction logic per the
    A-VERIFY-4 finding).
D5. Latest stable versions chosen (nix 0.31.2, lru 0.16.3) — web-verified by context-compiler.

EXPECTED TEST OUTCOMES
----------------------
- After 4E run: phase_4E.toml grows a [decomp_tree] section with the 3 keys non-empty.
- After P1+P2 edits: cargo check --workspace succeeds; no test fixture intentionally testing
  the OLD path is broken (context-compiler flagged none).
- After dep adds: cargo check --workspace succeeds on Windows AND simulated Linux (cfg gate).
  On the actual host (Windows), nix is excluded so no compile-time dependency on glibc.
- check_anti_stale.sh / check_hashmap_ban.sh / check_entropy_quarantine.sh counters are
  unchanged (this phase introduces no new violations).

UNKNOWNS
--------
- Whether measure_4e_tree depends on a sidecar / external service. Context-compiler reports
  "graceful fallback built in" — auditor confirms by direct invocation.
- Whether nix 0.31 introduces unexpected MSRV bumps. Auditor confirms cargo check --workspace.
- Whether any test fixture deeper in tests/ or .elai_cc/tests/ pins the OLD ONNX paths.
  Context-compiler reports none found — auditor sweeps via grep.

ROLLBACK
--------
git restore for any modified file. No commits will be made (orchestrator-wide constraint).

ACCEPTANCE
----------
A1: phase_4E.toml has a [decomp_tree] section with 3 non-empty numeric keys.
A2: All 8 ONNX path edits applied; cargo check --workspace passes.
A3: nix and lru appear in their target Cargo.toml files at the correct version + features.
A4: No new violations introduced into the lint-script counters.
A5: No tests broken by the path rename (cargo test --no-run --workspace as a smoke check).
