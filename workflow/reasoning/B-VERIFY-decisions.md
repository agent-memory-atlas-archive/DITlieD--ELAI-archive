B-VERIFY — Decision Doc
=======================
Phase: B-VERIFY (real-model end-to-end + 18 per-stage probes)
Session: s-27276

WHAT
----
Run elai end-to-end against a real-model sidecar and a real test corpus, and validate 18
spec acceptance criteria via per-stage probe scripts.

KEY DECISIONS
-------------
D1. Test corpus: F:\.ELAI_workflow\docs\.source\nextest-main (957 files, ~722 Rust).
    User confirmed source codebases under .source/ are valid test material.
D2. Non-interactive entry: `elai.exe -p "<task>"` via CliAction::Prompt (cli/main.rs:551-564).
    No need to drive interactive stdin.
D3. Sidecar: bind to default port. Model points to .elai_cc/models/Qwen2.5-Coder-7B-Instruct-abliterated.
    ONNX models already staged at .elai_cc/models/onnx/ + /splade/ from prior B-STAGE-MODELS phase.
D4. Probe layout: 18 verify_stage_<id>.sh + verify_stage_<id>.bat scripts in
    .elai_cc/scripts/. Each emits a single line "PASS|<measured-value>" or
    "FAIL|<reason>". An aggregator script reads all and writes B-VERIFY-report.md.
D5. ELAI_SSB_PATH explicitly exported to absolute path before any probe runs (showstopper #3
    from context-compiler).
D6. Build cargo --release once at start (showstopper #2 from context-compiler), ~5 min cost.
D7. Sidecar startup: wait for "Application startup complete." marker (~30-60s for 7B model).
D8. Showstopper #1 from context-compiler is STALE — ONNX models WERE staged in B-STAGE-MODELS.
    The execution agent must verify file presence at start (sanity gate) but should expect
    files to be present.
D9. Probes that need specific triggers (CSQ destructive call, SessionDag 85% budget, Stop hook
    Ctrl+C, ruff PostToolUse syntax error, 9-cat taxonomy fixtures) get bespoke trigger
    recipes per Section G of the context package.
D10. graphcodebert kill test (1I): rename graphcodebert.onnx → graphcodebert.onnx.disabled,
    re-run a query, observe SearchMode cascade in tracing log, then rename back.

EXPECTED TEST OUTCOMES
----------------------
- Real-model run completes; SSB has rows; tracing log has stage entries.
- Most probes PASS. Some may FAIL — those become spec-citation BLOCK items the orchestrator
  records as known-broken stages (do NOT silently skip per user instruction).
- B-VERIFY-report.md aggregates all 18 with measured values + pass/fail.

UNKNOWNS
--------
- Whether sidecar actually loads on this machine without GPU memory pressure (user noted
  10.2/12GB used, ~1.8GB free). 7B 4-bit Qwen + 5 ONNX models may exceed budget.
  Mitigation: load models lazily; run with --model 3B fallback if 7B OOM.
- Whether all 18 probes can run in a single session without state cross-contamination.
  Mitigation: each probe gets its own SSB query window or separate elai run.
- 5 minute cargo build + 30-60s sidecar start = ~6 min before first probe. Acceptable.

ROLLBACK
--------
git restore on probe scripts. Models/SSB DB are gitignored. graphcodebert rename test must
restore the file at the end (or on probe failure).

ACCEPTANCE
----------
A1: 36 probe scripts written (.sh + .bat each for 18 checks).
A2: Sidecar started successfully and responded to a sanity request.
A3: elai.exe -p "<task>" ran end-to-end and emitted SSB rows + tracing entries.
A4: All 18 probes ran without crashing the harness.
A5: B-VERIFY-report.md aggregates results with PASS/FAIL + measured value per check.
A6: cargo check --workspace remains PASS (no source changes).
