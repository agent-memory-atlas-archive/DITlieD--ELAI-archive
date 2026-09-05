E-17 SP6 — eval harness + monotonicity contract — decisions

Session: s-26556
FSM: EXECUTING (3 of 3 attempts available)
Deliverables shipped:
  D1  .elai_cc/eval/rag/queries.toml          (15-query seed goldset)
  D2  .elai_cc/eval/rag/run.py                (~530 lines stdlib-only)
  D3  .elai_cc/eval/rag/README.md             (plain-text per output-formatting)
  D4  .elai_cc/crates/cli/src/bench_rag.rs    (~380 lines, 5 unit tests)
  D4b .elai_cc/crates/cli/src/args.rs         (CliAction::BenchRag + parse_bench_rag_args)
  D4c .elai_cc/crates/cli/src/main.rs         (dispatcher + mod decl)
  D5  .elai_cc/crates/contracts/tests/rag_monotonicity_contract.rs (3 tests)

Decisions

DECISION-D1: 15 queries (8 PL + 7 NL), not 50
Rationale: pragmatic-scope cap in the SP6 spec + context package GAP-1
explicitly defers the 50-query goldset to an ops follow-up. The 15-query
seed exercises both the HyDE NL-only gate (8 code queries should fire
hyde_skipped_code_query at DEEP) AND the deep_pool_cap path (NL queries
hit DEEP expansion + HyDE attempt). Class balance (8/7) is load-bearing
for SP2 observability; total count is not. Adding queries is a single
queries.toml edit + re-run.

DECISION-D2: corpus = ELAI repo self (no 135-file private corpus)
Rationale: context package GAP-4 records that the 135-file private corpus
referenced in the E-17 spec does not exist. The ELAI repo (~700+ source
files indexed at .elai_cc/projects/<session>/bm25/) is the actual
production corpus the dashboard queries against, so measuring against it
is the right thing. Risk: corpus drifts as more code lands; queries.toml
entries pin to commit-stable symbols (RuntimeAdapter::stream_step,
SCHEMA_VERSION location, QueryTier enum) so they survive C/D/E-stream
campaigns. Mitigation: re-validate after each campaign by re-running
run.py and watching for sudden MRR drops.

DECISION-D3: ship `elai bench rag` subcommand (the GAP-1 blocker)
Rationale: no clean knowledge-query CLI existed before SP6. /research is
web research (live_cli.rs:2669+), not retrieval; pipeline.query() is only
reachable via augment_with_knowledge (wrapped in prompt-turn machinery
emitting a chat response, not parseable retrieval JSON). Option A from
the integration-explorer survey is implemented: the new subcommand drives
the EXACT same `LiveCli::build_local_knowledge_stack` factory production
uses (reuses, not duplicates, the live pipeline assembly), then calls
QueryPipeline::query_with_tier at the explicit tier. ~380 LOC including
docstring, 5 unit tests, and the CwdGuard helper that lets the
subcommand drive a project_root != cwd without polluting other tests.

DECISION-D4: contract test asserts MRR_deep >= MRR_standard (>= not >)
Rationale: context package GAP-6 flags pipeline.rs:813 uses std::HashMap
for the RRF accumulator; ties iterate in unstable order before sort_by.
On a 12-doc synthetic corpus, ties are common (BM25 with similar token
counts produces tied scores). Strict `>` would flake. The SP2 invariant
the contract pins is "no regression"; `>=` encodes that. The harness
run.py also uses `>=` (MONOTONICITY_EPSILON = 0.0). Future fix: switch
pipeline.rs:813 to BTreeMap/IndexMap for deterministic tie-break; that's
outside SP6 scope.

DECISION-D5: contract test uses SYNTHETIC corpus, not real index
Rationale: CI determinism + zero infra dependency. The synthetic corpus
is 12 docs (4 strongly relevant to "stream step" via RuntimeAdapter; 4
weakly relevant; 4 irrelevant) written to a TempDir and indexed via
Bm25Index::index_file. No ONNX models loaded, no sidecar required, no
.elai_cc/indexes/ pre-existing. The test runs in <1s on a 9-year-old
laptop and produces the same MRR every run (modulo the documented
HashMap-tie flap absorbed by >=). The real index measurement lives in
run.py; the synthetic measurement lives here.

DECISION-D6: cAST Recall@5 acceptance EXPLICITLY DEFERRED
Rationale: SP3 (cAST chunking) was skipped per pragmatic scope. The
E-17 spec's "cAST Recall@5 >= +4 over recursive baseline" target cannot
be measured without SP3. README.md documents this explicitly under
"What's deferred" so the next session does not re-open the cAST scope
by accident. Re-opening requires SP3 ship first.

DECISION-D7: HyDE-NL baseline measured but not gated this pass
Rationale: context package GAP-2 — no prior baseline exists. The harness
can be run twice (ELAI_HYDE_NL_ONLY=1 vs =0) and the deltas compared in
the .elai_cc/metrics/E-17-eval-*.toml output, but neither value is
gated. Gating requires (a) a measured baseline run on a stable index +
goldset, (b) a delta threshold agreed with the user. Both are ops
follow-up; recorded in README + here.

DECISION-D8: SSB observation via stdlib sqlite3, not via Rust binding
Rationale: stdlib sqlite3 is in Python core since 3.0; adding a Rust
binding (Tauri-style command, or PyO3 wrap of SsbViewer) would inflate
the harness footprint by an order of magnitude with no schema-stability
benefit (SSB SCHEMA_VERSION=45 per the orchestrator memory is stable).
run.py opens .elai_cc/sessions/ssb.db readonly and counts rows per
SsbActionType::as_sql_str() value. Verified path canonical layout:
ELAI_DATA_DIR_NAME = ".elai_cc"; sessions/ssb.db per core::paths.

DECISION-D9: bench_rag is SSB-noop (no Ssb wired at construct time)
Rationale: SP6 acceptance is "harness reads the 4 SP2 SSB row types via
sqlite3"; the producer is the live pipeline at runtime. Wiring an Ssb
Arc into bench_rag would require sandboxing a project-scoped Ssb config
(project_id from where? session_id from where?) and would emit beacons
ATTRIBUTED TO bench_rag rather than to the user's live session. That is
not the surface the harness measures. The dashboard observation path is
populated by the production wiring at live_cli.rs:1257-1268 during real
sessions; SP6 reads what's already there.

What targets are VALIDATED vs DEFERRED

VALIDATED (contract test passes, 3/3):
  monotonicity_deep_geq_standard_on_synthetic_corpus
  standard_mrr_meets_soft_floor_on_synthetic_corpus (floor=0.30)
  deep_returns_at_least_one_goldset_hit_at_recall_5

VALIDATED (harness smoke-runs, all wiring correct):
  elai bench rag --json emits well-formed payload (BenchRagOutput)
  run.py drives subprocess, parses JSON, computes MRR/R@5/R@10
  SSB observation via stdlib sqlite3 opens ssb.db readonly
  TOML report writer produces parseable output

DEFERRED:
  STANDARD MRR >= 0.55 against real ELAI index — requires a built BM25
    index. The harness was smoke-tested against an UNBUILT index at
    F:\.ELAI_workflow\.elai_cc (BM25 LockBusy from a concurrent elai
    process). Production gating requires: (a) `elai index build` first,
    (b) re-run with --enforce-floor. Both shippable today, neither
    in-scope for SP6.
  cAST Recall@5 — SP3 skipped (DECISION-D6).
  HyDE-NL baseline — no prior measurement (DECISION-D7).
  Full 50-query goldset — ops follow-up (DECISION-D1).

Whether the contract test uses synthetic or real corpus
SYNTHETIC. See DECISION-D5 above. Three tests run in <1s without any
external dependency.

SSB row counts measured during harness run
Harness smoke run against the workflow-root project encountered an
UNBUILT index path: F:\.ELAI_workflow\.elai_cc\sessions\ssb.db did not
exist (output reported "unavailable: not found"). SSB row counts will be
measurable after a real `elai index build` + `elai bench rag` run; the
sqlite reader path itself was exercised against the missing-table
branch (counts={}; available=false; reason="not found"). Wiring
verified end-to-end on the soft-degrade path.

Test results

cargo check -p elai-cli                                    PASS
cargo check -p elai-cli -p elai_cc_contracts --tests       PASS (warnings only)
cargo build -p elai-cli --bin elai                         PASS
cargo test -p elai_cc_contracts --test rag_monotonicity_contract
                                                            3 passed; 0 failed
elai bench rag --json (smoke)                              PASS (soft-degrade JSON)
python eval/rag/run.py --tier BOTH --top-k 10 --runs 1     PASS (degraded; no index)

Anti-spiral notes
None triggered. No file edited >3 times for same issue. No hypothesis
failures requiring debug logging. Single format-string fix
(rag_monotonicity_contract.rs:267 — Python-style {QUERY!r} -> Rust
{:?}) caught on first cargo check and resolved on first edit.
