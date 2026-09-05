Tranche O decisions — orchestrate s-22600 (B-15 cli-regrowth)
Date: 2026-06-01
Source ctx: workflow/sessions/s-22600/context-package-trancheO.md

WHAT SHIPPED

ITEM O.1 — cli/src helpers extraction
  NEW: .elai_cc/crates/cli/src/cli_helpers.rs (281 LOC)
  Hosts five top-level helpers moved out of main.rs:
    1. shortcoder_compress_text (pub(crate)) — externally consumed by runtime_client.rs:348+352 via use crate::*; glob
    2. CharEstimatorProvider (pub(crate) struct + LlmProvider impl)
    3. check_anthropic_gateway (pub(crate))
    4. build_pr_review_action (pub(crate))
    5. cmd_incident_record (pub(crate))
  Visibility note: ctx-package recommended cmd_incident_record + build_pr_review_action as module-private. Executor chose pub(crate) to keep them re-exportable via the glob at main.rs without forcing main.rs to qualify calls. Same net visibility surface (crate-scoped).
  Constants kept in main.rs: FENCE_LANG_MAP + TOKEN_CHARS_PER_TOKEN. cli_helpers.rs imports them via use crate::{FENCE_LANG_MAP, TOKEN_CHARS_PER_TOKEN};

ITEM O.2 — args.rs bench parser block extraction
  NEW: .elai_cc/crates/cli/src/args_bench.rs (743 LOC)
  Hosts the 8 bench parsers + validate_bool_arg helper moved out of args.rs L3358-L4084 (722 LOC):
    parse_bench_subcommand_args (pub(crate); ENTRY)
    parse_bench_mode_routing_args (private)
    parse_bench_conformal_args (private)
    parse_bench_arc_args (private)
    parse_bench_rag_args (private)
    parse_bench_sycophancy_args (private)
    parse_bench_canary_args (private)
    parse_bench_tamper_args (private)
    validate_bool_arg (private)
  Call site change: args.rs L2018 — was parse_bench_subcommand_args(&rest[1..]); now crate::args_bench::parse_bench_subcommand_args(&rest[1..])
  CliAction enum UNCHANGED. 8 Bench* variants still live in args.rs L568-L674.
  Eq derive preserved trivially (no enum touch).
  No test impact: bench smoke tests (bench_arc_smoke / bench_conformal_smoke / bench_mode_routing_smoke / bench_prmbench_smoke) build CliAction directly OR call parse_args end-to-end, not parse_bench_*_args directly.

WIRING IN main.rs

  Added two mod decls (after dkg_lifecycle):
    pub(crate) mod cli_helpers;
    pub(crate) mod args_bench;
  Added one re-export (after pub(crate) use crate::commands::{...};):
    pub(crate) use crate::cli_helpers::*;  // load-bearing for runtime_client.rs:72 use crate::*; glob
  Removed 5 helper bodies in 4 Edit blocks (235 LOC removed).
  main.rs delta: 1221 -> 1009 LOC (-212).

WIRING IN args.rs

  Updated single L2018 call site to crate::args_bench::parse_bench_subcommand_args.
  Removed L3358-L4084 (the entire bench parser block).
  Added one comment block at the deletion site noting Tranche O.2 relocation.
  args.rs delta: 8146 -> 7431 LOC (-715).

LOC ACCOUNTING

  main.rs:        1221 -> 1009 (-212)
  args.rs:        8146 -> 7431 (-715)
  cli_helpers.rs:    0 ->  281 (+281)  NEW
  args_bench.rs:     0 ->  743 (+743)  NEW
  Net crate:       -927 + 1024 = +97 LOC
  Hygiene-only delta per gotchas.md [J_CLI_EXTRACTION_WRAPPER_BOILERPLATE_COST]; ctx-package projected ~+103, actual +97. Within INFO band.

VERDICT

  cargo check -p elai-cli --tests: GREEN (28.25s; pre-existing warnings only — unexpected_cfgs/empire feature)
  cargo check --workspace --tests:  GREEN (4.16s incremental; pre-existing warnings only)
  Wiring evidence:
    grep shortcoder_compress_text -> cli_helpers.rs:31 (def) + main.rs:258/262/294/321 (mod-decl + re-export comments) + runtime_client.rs:348+352 (consumer call sites unchanged) + bench_cmd.rs:23 (Tranche J doc-comment reference)
    grep parse_bench_subcommand_args in args.rs -> only the single call-site at L2018 (cleanly dispatching via args_bench:: qualifier)

DECISIONS DEVIATED FROM CTX-PACKAGE

  D-1. Visibility of cmd_incident_record + build_pr_review_action.
       Ctx: module-private. Executor: pub(crate).
       Rationale: cli_helpers.rs is re-exported by main.rs via pub(crate) use crate::cli_helpers::*;. Making them module-private at cli_helpers means they would NOT be surfaced through the glob, so main.rs would need either crate::cli_helpers::cmd_incident_record(...) or a separate use crate::cli_helpers::{cmd_incident_record, build_pr_review_action};. Both forms are mechanically uglier than letting the glob re-export work uniformly across all 5 helpers. Net effective visibility unchanged (still crate-scoped; nothing leaks to outside the cli crate).

UNCHANGED ITEMS

  CliAction enum (args.rs L75-L973): no variants touched.
  SCHEMA_VERSION 195: no SSB / schema modifications.
  Existing tests: bench smoke tests + cli_tests + sandbox_report_tests all compile (workspace cargo check --tests GREEN).
  pr_review_cmd::PrReviewAction shape: preserved (helper extraction is body-for-body move).
  shortcoder_compress_text algorithm: preserved (byte-identical body).
  CharEstimatorProvider::LlmProvider trait impl: preserved (4 methods, native async fn in trait surface via Pin<Box<dyn Future>>).

CARRY-OVER

  Tranche P (next): per ctx INFO-O-3 the natural follow-on is Candidate D — pair PR/pr-review parser extraction (~298 LOC) with the build_pr_review_action helper move just completed. Symbol-graph cluster (Candidate B) and admin cluster (Candidate C) remain deferred.
  elai.map staleness: INFO-O-1 still applies; map regen via .claude/tools/codebase_mapper.py should happen before next context-compiler dispatch since main.rs and args.rs both shifted significantly.
  Optional check_anthropic_gateway unit tests: INFO-O-4 — auditor's call whether to require; executor did not add them (kept scope minimal per fixer-agent quality rule #2 "Never widen the scope").
