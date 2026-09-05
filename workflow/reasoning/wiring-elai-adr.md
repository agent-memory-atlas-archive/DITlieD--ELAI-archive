WIRING-FIX-ELAI ADRs (WAVE-0 conflict reconciliation)
Written 2026-06-02. Resolves the 10 cross-cluster conflicts + 6 blockers from the WAVE-0 design consolidator
(wf_40de6607-2e8). Authoritative over the per-cluster specs in workflow/sessions/s-wiring-fix-elai/specs/*.json
and over workflow/reasoning/wiring-elai-edit-specs.md where they disagree. Executors apply specs AS AMENDED here.

ADR-1 (Layer-3/Layer-4 evidence surface; resolves CONFLICT-1/2/3/4/5, BLOCKER-2/3): NO new SsbActionType variants.
  Reuse the EXISTING action_types wiring_evidence (clean) and wiring_violation (verified live ssb.rs:8598/8599).
  The Layer-3 coverage runner and the Layer-4 trace check each emit, on completion, a wiring_evidence row carrying a
  payload leg discriminator (leg = "layer3_coverage" with {clean:bool, uncovered:[...]} ; leg = "layer4_trace" with
  {present:bool, observed_rows:n}), or a wiring_violation row when they fail. CONSEQUENCES: TOTAL_VARIANTS stays 482;
  the D-XL e45 patch 394->482 is correct as written; WAVE-4 (SSB schema bump + golden regen + SCHEMA_VERSION) is
  DROPPED entirely; no action_type_lock.golden change. Drop the D-EV names (EntrypointCoverage/WireDarkTrace) and the
  D-FZ trio (wiring_audit_completed/...). Canonical = wiring_evidence + wiring_violation.

ADR-2 (autonomous gate consumption; CONFLICT-1/2): Evidence (autonomous_gate.rs) gains layer3_entrypoint_covered: bool
  and layer4_trace_present: bool, both default FALSE. They are derived by reading the most-recent in-window
  wiring_evidence rows (by leg) via the real &dyn SsbReader; absent row OR clean/present=false => field=FALSE. A new
  EntrypointAndTraceGate (impl EvidenceGate, transition Executing->Auditing, name "entrypoint_and_trace") FAILS when
  either field is FALSE (R1 missing-evidence=FAIL, R4 measure-off-agent-artifact). Evidence::all_passed() ANDs both new
  fields. derive_* fns take &dyn SsbReader (NOT &WiringCoverageCmd), so the gate reads the off-agent SSB artifact, not a
  re-run. (Supersedes the consolidator's CONFLICT-1 recommendation to call layer3_coverage_clean(&Cmd) directly.)

ADR-3 (gate-manifest; CONFLICT-6/7, BLOCKER-5): .claude/gate-manifest.txt gets ONE merged edit adding D-L4's two
  entries (tracing-test-assert.md + traced_test_assert_check.py) AND D-FZ's two entries (elai_cc.toml +
  locked_keys_registry.toml). DROP D-FZ's gate-manifest.lock in-file content edit entirely (it corrupts a real
  sha3_256 hash line). The lock is regenerated ONCE post-commit by the auditor/controller via
  `python .claude/tools/gate_freeze_check.py --lock`, NEVER by the executing run (R8).

ADR-4 (elai_cc.toml entrypoints; CONFLICT-8, BLOCKER-6): ONE merged edit to [wiring.entrypoints] = D-XL's 6-symbol set
  (cli main, fsm::transition, ipc_registry::dispatch, in_process::dispatch_tool, tauri main, tauri lib::run) PLUS
  D-FZ's [wiring.layer3] block appended after it.

ADR-5 (sequencing; CONFLICT-9): D-WC SP2 (real SymbolGraphCache reachability graph) MUST land before the elai_cc.toml
  [wiring.entrypoints] activation, else the reachability leg runs live-but-blind. Enforced by wave order (WAVE 2 before
  WAVE 6).

ADR-6 (D-L3 parse signature; CONFLICT-10, BLOCKER-4): parse_wiring_coverage_subcommand_args(rest: &[String],
  output_format: CliOutputFormat) -> Result<CliAction, String>; body indexes via args[i].as_str(). The specced
  &[&str] is WRONG (sibling parse_archive_subcommand_args(rest:&[String]) at args.rs:2749). Re-read anchor at exec.

ADR-7 (SymbolGraphCache path; BLOCKER-1): use crate::symbol_graph::SymbolGraphCache (load() cache.rs:167, iter_nodes()
  cache.rs:350, SymbolNode cache.rs:61-76). The grounding-doc elai_knowledge::symbol_graph path was WRONG. Same crate,
  no Cargo.toml change, no new dep. Feasibility VERIFIED.

ADR-8 (Evidence-literal blast radius): six contracts crate files build all-true Evidence literals and MUST add
  layer3_entrypoint_covered:true + layer4_trace_present:true in the SAME wave as the field additions:
  autonomous_gate_contract.rs:52, autonomous_gate_with_framing_contract.rs:34, autonomous_gate_with_per_level_contract.rs:77,
  c10_anti_spiral_contract.rs, c9_autonomous_infra_contract.rs, phase_4g_extended_contract.rs.

ADR-9 (D-EXH under-specified): the spec ships placeholder bodies. Executor authors the real test: 4 round-trip tests
  (every variant through the dispatch fn) + 4 grep contract tests asserting no wildcard arm at the canonical dispatch
  site. Note RouteDecisionEventKind has a test-local match at ssb.rs:29108 (do not anchor the grep there).

REVISED WAVE PLAN (WAVE-4 dropped per ADR-1)
  WAVE 1 (fail-close, file-disjoint parallel): D-WC SP1+SP4 (wiring_contract.rs) + D-EV 5 self-contained gate flips
    (evidence.rs/task_runner.rs) + D-XL e45 394->482 + D-EXH exhaustiveness test CREATE. Gate: cargo check -p
    elai-orchestrator -p elai-contracts; loader/gate tests asserting Ok-on-empty flip to expect-Err (update same wave).
  WAVE 2 (D-WC SP2 reachability): crate::symbol_graph::SymbolGraphCache graph builder + WiringAuditCmd.graph_path field
    + cli/wiring_audit.rs:79 literal +graph_path:None. Gate: cargo check -p elai-orchestrator -p elai-cli.
  WAVE 3 (D-L3 Layer-3 runner): orchestrator/src/wiring_coverage.rs + lib.rs pub mod + cli subcommand (CORRECTED sig) +
    args/main wiring; runner emits wiring_evidence(leg=layer3_coverage)/wiring_violation. Gate: cargo check -p
    elai-orchestrator -p elai-cli.
  WAVE 4 (D-EV consumers): Evidence fields + EntrypointAndTraceGate + derive_* (read SSB rows) + 6 contract updates.
    Gate: cargo check -p elai-orchestrator -p elai-contracts.
  WAVE 5 (D-L4 harness, parallel-able with 4 but different crate): contracts/tests/wiring_layer4_trace_contract.rs +
    contracts/Cargo.toml tracing-test dev-dep + .claude/rules/tracing-test-assert.md + traced_test_assert_check.py;
    harness emits wiring_evidence(leg=layer4_trace). Gate: cargo check -p elai-contracts.
  WAVE 6 (data/CI/freeze, no compile): merged elai_cc.toml (ADR-4) + merged gate-manifest.txt (ADR-3) +
    cross_lang_diff.py unconditional + cross-lang-pairs.toml + wiring-audit.yml + windows.yml [ci skip] removal +
    wire-dark-allowlist + smoke-manifest population + locked_keys_registry append + githooks. DROP gate-manifest.lock edit.
  WAVE 7 (audit + freeze-lock + commit): auditor (cargo check --workspace + scoped tests + dogfood elai wiring audit) +
    gate_freeze_check.py --lock regen + memory-writer + ONE commit (maintainer voice, no AI trailer).
