E-17 SP6 — RAG eval harness

This directory holds the retrieval-quality eval harness shipped by E-17
SP6. It measures STANDARD vs DEEP MRR + R@5 + R@10 against the ELAI
repo (used as its own corpus) and feeds the result into the
monotonicity contract test at:

  .elai_cc/crates/contracts/tests/rag_monotonicity_contract.rs

Files in this directory

  queries.toml   15-query seed goldset (8 code / 7 NL).
                 Class balance is load-bearing because the HyDE NL-only
                 gate (E-17 SP2) is only exercised when code-symbol
                 queries hit DEEP tier (then the `hyde_skipped_code_query`
                 SSB row fires).

  run.py         Python 3.11+ stdlib-only driver.
                 Invokes `elai bench rag --json` once per (query, tier),
                 reads the JSON payload from stdout, computes MRR + R@5
                 + R@10 per tier, then opens the SSB sqlite store
                 out-of-band to count the 4 SP2 row types as a wiring
                 observable. Exit 0 iff DEEP MRR >= STANDARD MRR.

  README.md      this file

How to run

  cd <project-root containing .elai_cc/>
  # build the elai binary so target/debug/elai exists
  cargo build -p elai-cli --bin elai
  # full BOTH-tier sweep over the 15 queries, median-of-3 to absorb
  # the std::HashMap-tie flap risk in pipeline.rs:813:
  python .elai_cc/eval/rag/run.py --tier BOTH --top-k 10 --runs 3

  # standalone STANDARD pass:
  python .elai_cc/eval/rag/run.py --tier STANDARD --top-k 10 --runs 1

  # production-floor gating (E-17 spec STANDARD MRR >= 0.55):
  python .elai_cc/eval/rag/run.py --tier BOTH --enforce-floor

Prerequisites

  1. The BM25 index must exist under <project-root>/projects/<session-id>/bm25/
     (created by `elai index build` during normal use). If the harness
     reports "PIPELINE_UNAVAILABLE: BM25 index could not be opened ...",
     run an index build first then re-run the harness.

  2. ONNX models in .elai_cc/models/onnx/ (cross-encoder, graphcodebert,
     granite-embed) and .elai_cc/models/splade/ are optional. When
     absent the pipeline degrades gracefully (BM25_ONLY mode + warn
     line) and the harness still produces an MRR number — but DEEP and
     STANDARD will be nearly identical because the only real difference
     becomes query expansion + HyDE attempts. Use this to detect a
     model-load regression in the harness output.

  3. No sidecar required. The reranker is in-process ONNX
     (cross-encoder-l12.onnx). The sidecar at localhost:8100 is only
     consulted for HyDE (sidecar_probe_ok); when offline, HyDE is
     skipped with a tracing::debug line and the harness still runs.

Targets (acceptance)

  STANDARD MRR >= 0.55  (E-17 spec; --enforce-floor activates this gate)
  DEEP MRR     >= STANDARD MRR  (monotonicity invariant; always gated)

The contract test asserts only the monotonicity invariant on a
synthetic in-memory corpus (so it runs deterministically in CI). The
production MRR floor is checked at harness time against the live index.

What's deferred

  cAST recall@5 acceptance — E-17 SP3 was skipped per pragmatic scope.
    The "cAST Recall@5 >= +4 over recursive baseline" target from
    docs/thefinalglance/E-plan/E-17_rag_retrieval_quality.md is NOT
    measured by this harness. Re-opening that scope requires shipping
    SP3 (cAST chunking) first.

  Full 50-query goldset — the spec calls for 50 queries; this seed
    ships 15 (8 code / 7 NL). The remaining 35 are an ops follow-up.
    Adding queries requires: (a) edit queries.toml, (b) re-run with
    --runs 3 to confirm no MRR regression, (c) update the
    `total_queries` line in [meta].

  HyDE-NL-only A/B baseline — the harness can be run twice (with
    ELAI_HYDE_NL_ONLY=1 and =0) and the deltas compared, but neither
    mode is gated yet. The spec line "HyDE-NL recall@5 >= baseline"
    needs a baseline number first (no prior measurement exists).

Runtime-load failure pathway (SP1 finding)

  RerankerConfig::default() returns relative PathBufs that resolve
  against process CWD. If a caller uses `default()` instead of
  `with_project_root()`, the reranker silently fails to load when CWD
  != .elai_cc. SP2 fixed this for the production wiring at
  live_cli.rs:1336 (uses with_project_root). The `bench rag` subcommand
  reuses `LiveCli::build_local_knowledge_stack` which uses the fixed
  path, so the harness inherits the fix. If you see a "cross-encoder
  reranker not loaded" warn in the bench output, the model files are
  missing — not a path-resolution bug.

Cross-reference (dashboard observation)

  The 4 SP2 SSB row types surfaced by this harness are also rendered in
  the dashboard's KnowledgeDrawer.svelte (E-17 SP4 wired the 4 Tauri
  reader commands). Running the harness against a live index and
  opening the dashboard's Knowledge drawer is the end-to-end
  observable_effect smoke for the SP2 + SP4 + SP6 chain.

Cross-reference (contract test)

  .elai_cc/crates/contracts/tests/rag_monotonicity_contract.rs holds
  the synthetic-corpus monotonicity assertion. It runs in CI without
  any on-disk index, sidecar, or model. See its top-of-file docstring
  for the exact synthetic corpus design.

Exit codes

  run.py exits:
    0 — STANDARD MRR meets floor (when --enforce-floor) AND DEEP MRR
        >= STANDARD MRR
    1 — monotonicity regression OR STANDARD floor failure
    2 — harness error (subprocess crash, missing toml lib, etc.)
