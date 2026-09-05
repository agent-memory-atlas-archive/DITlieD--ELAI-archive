E-13 — TDD verification agents decisions

Session: s-25120 (completion-drive session 2/4 — concurrent with s-31708 on SCHEMA bump)
Date: 2026-05-24
Plan: docs/thefinalglance/E-plan/E-13_tdd_verification_agents.md

DECISION-1: cite-discipline deferral for 11 arXiv IDs

The E-13 plan cites 11 arXiv IDs across its motivation + grounding
sections. The cluster-wide cite-incident taxonomy in
workflow/sessions/s-25120/mistakes.md records 5 PRIOR bad-cite
incidents (B-40 2407.21787, B-42 2305.18290 DPO substitute, C-43
arXiv:2604.21725, C-44 shallow-cite-fit, C-50 2604.13100). The
discipline rule is: WebFetch ONLY the 2-3 most load-bearing cites
inline; the rest are referenced by author/title with cite
verification DEFERRED.

For E-13 the two load-bearing cites are:
  - arXiv:2404.09952 LLMorpheus — LLM-mutated source finds faults
    Stryker can't. Inline-cited in
    crates/orchestrator/src/tdd_verifier/agent.rs doc header AND
    crates/orchestrator/src/ssb.rs TddCycleStarted variant docstring
    (grounding for the cycle-anchor row).
  - arXiv:2504.16828 ThinkPRM — process reward model evidence.
    Inline-cited in the TddTestPassed variant docstring (grounding
    for the "test flipped" signal as the canonical non-LLM verifier
    under D-3 aggregation).

The remaining 9 plan-cited IDs are referenced by author/title in
THIS document only:
  - PGS (Property-Generated Solver, +13.4% pass@1) — plan task 1
    rationale.
  - Agentic-PBT (56% valid bug rate) — plan task 1 rationale.
  - UTBoost (24.4% Verified leaderboard inflation) — plan tasks 1
    + 8 anti-vacuous rationale.
  - Meta ACH (73% engineer acceptance) — plan task 5 mutant-hunter
    rationale.
  - Hora & Robbes 2026 (36% vs 26% mock-add rate) — plan task 3
    anti-over-mock rationale.
  - No Free Labels (TPR>96% / TNR<25%) — plan task 4 non-LLM
    verifier requirement.
  - PRMs for SWE agents — plan general context.
  - Less Is More / CleanTest (43.52% Methods2Test noise) — plan
    general quality > volume context.
  - Fuzz4All (98 bugs / 64 novel across 6 languages) — plan task 6
    fuzz gate rationale (TASK DEFERRED in this session).

Verification of all 9 deferred IDs is logged as a follow-up to the
paper-ingestor agent; this session does NOT inline-cite any of
them in code. The e13_tdd_verification_contract::d7 test enforces
non-inlining at compile time.

DECISION-2: compose with D-3, do NOT ship a parallel aggregator

The plan §4 says "If D-3 has not shipped, ship the constraint as a
local addition to `auditor_mode.rs::aggregate_verifiers` and
migrate to D-3 when it lands." The re-check report's exploration
verified that D-3 IS SHIPPED at
crates/orchestrator/src/verification_result.rs with `aggregate`
(line 322), `ConsensusRule::VerifierKindHeterogeneous` (line 248),
and `AggregateVerdict` (line 286).

Per the project's no-parallel-surfaces rule, E-13 composes with
D-3 instead of shipping a second aggregator. The TDD verifier
agent emits a `Vec<VerificationResult>` (one per signal source —
example flip + cargo-mutants outcome + anti-mock scan) via
`tdd_verifier::consensus::tdd_results_to_verifier_results`, and
the caller hands those rows to D-3's
`aggregate(&ConsensusRule::VerifierKindHeterogeneous {
required_kinds: vec![VerifierKind::Deterministic] }, &rows)`.

The plan §4 stand-in branch (`aggregate_verifiers` in
auditor_mode.rs) is therefore OBSOLETE and was NOT shipped. The
cross-check report's `[MISSING] aggregate_verifiers function in
auditor_mode.rs` is acknowledged but the resolution is "don't
ship; compose with D-3" rather than "ship the stand-in".

DECISION-3: cargo-mutants 27.0.0 has no `--shard` flag

The plan §2 calls for `cargo mutants --in-place --baseline skip
--shard <touched_files>`. The cargo-mutants 27.0 CLI surface
(github.com/sourcefrog/cargo-mutants README + `cargo-mutants
--help`) does NOT document a `--shard` flag. Per-file scoping is
expressed via repeated `--file <PATH>` argv pairs.

The corrected argv shape lives in
`crates/orchestrator/src/tdd_verifier/mutator.rs::cargo_mutants_argv`:
  cargo-mutants --in-place --baseline=skip --no-shuffle
                --file <P1> --file <P2> ...

The contract test `e13_tdd_verification_contract::d3` enforces
this shape (asserts `--file` present, `--shard` absent).

DECISION-4: SSB scope cut — 7 variants -> 3 essential signals

The plan §11 calls for 7 new SSB action types (property_gate_pass,
property_gate_vacuous_rejected, mutation_gate_survivor_block,
anti_mock_block, fuzz_panic, integration_property_coverage,
mutant_hunter_spawned). The scope-lean re-check trimmed this to
the 3 essential signals tied to the actual TDD cycle:
  - TddCycleStarted (cycle entry anchor)
  - TddTestFailed (Red-phase observation)
  - TddTestPassed (Red->Green flip with optional mutation_score)

Rationale:
  - PropertyGate / FuzzGate / IntegrationTestRunner / mutant-hunter
    agent are DEFERRED (Task 1, 6, 9, 5 from the plan). These
    surfaces require their own design pass + paper-grounding;
    shipping the variants without the producers is wire-dark
    pollution.
  - AntiMockHook ships as a pure-function heuristic
    (`tdd_verifier::anti_mock_scan`) rather than a PostToolUse-hook
    integration. The `anti_mock_block` SSB variant is therefore
    deferred to the hook-integration follow-up.

SCHEMA bump: 129 -> 130 atomic +3. TOTAL_VARIANTS 358 -> 361.

DECISION-5: 1 new FsmSubstate variant (Tdd) — not MutantHunting

The plan §5 calls for `FsmSubstate::MutantHunting` to mark the
mutant-hunter spawn. Since the mutant-hunter agent is deferred
(see DECISION-4), the substate variant is renamed to `Tdd` and
marks an active Red->Green->Refactor cycle driven by the
TddVerifierAgent. Future plan iterations can add `MutantHunting`
as a sub-variant when the agent file ships.

Snake-case label: `tdd`.

DECISION-6: no `properties.md` harness check, no `mutant-hunter`
agent file, no Dashboard panels in this scope

These three plan items (Task 7 + Task 5 + Task 11) are deferred to
follow-up sessions. Rationale:
  - properties.md per-crate file authoring is a one-time setup
    cost that benefits future plans more than E-13; it ships under
    a separate "harness" plan.
  - mutant-hunter agent requires local Qwen 7B context + a careful
    prompt; the agent file authoring belongs to the
    scope-planner-elai partition, not the executor.
  - Dashboard panels (MutationGateGauge / PropertyCoverageGauge /
    OverMockHistogram) require the corresponding SSB variants from
    Task 1, 2, and 3 to be SHIPPED first; since those are
    DEFERRED, the panels are too.

DECISION-7: live cargo-mutants run is RUNPOD-PENDING

The local dev host may not have `cargo-mutants 27.0.0` on PATH.
The mutator wrapper returns
`MutatorError::ToolchainMissing` when the binary is absent
(wire-dark posture per E-12 Tier-B + C-52 dafny precedents). Live
mutation-score evaluation on a representative crate is logged as
RUNPOD-PENDING.

Files shipped this session

CREATE:
  - .elai_cc/crates/orchestrator/src/tdd_verifier/mod.rs (~70 LOC)
  - .elai_cc/crates/orchestrator/src/tdd_verifier/cycle.rs (~180 LOC)
  - .elai_cc/crates/orchestrator/src/tdd_verifier/anti_mock.rs (~170 LOC)
  - .elai_cc/crates/orchestrator/src/tdd_verifier/mutator.rs (~310 LOC)
  - .elai_cc/crates/orchestrator/src/tdd_verifier/consensus.rs (~130 LOC)
  - .elai_cc/crates/orchestrator/src/tdd_verifier/agent.rs (~210 LOC)
  - .elai_cc/crates/cli/src/tdd_cmd.rs (~230 LOC)
  - .elai_cc/crates/contracts/tests/e13_tdd_verification_contract.rs (~230 LOC)
  - workflow/reasoning/E-13-decisions.md (this file)
MODIFY:
  - .elai_cc/crates/orchestrator/src/lib.rs (+8 LOC; pub mod tdd_verifier)
  - .elai_cc/crates/orchestrator/src/fsm.rs (+15 LOC; FsmSubstate::Tdd)
  - .elai_cc/crates/orchestrator/src/ssb.rs (+150 LOC; 3 variants + payloads + cascade)
  - .elai_cc/crates/orchestrator/tests/action_type_lock.rs (+10 LOC; lock cascade)
  - .elai_cc/crates/orchestrator/tests/action_type_lock.golden (regenerated 358 -> 361 entries)
  - .elai_cc/crates/cli/src/args.rs (+115 LOC; CliAction::Tdd + TddAction + parser)
  - .elai_cc/crates/cli/src/main.rs (+10 LOC; mod tdd_cmd + dispatch arm)
  - workflow/sessions/s-25120/mistakes.md (+entry; E-13 cluster cite-deferral)

Deferred to follow-up sessions:
  - properties.md harness check
  - mutant-hunter agent file
  - PropertyGate / FuzzGate / IntegrationTestRunner modules
  - 4 plan-proposed SSB variants beyond the 3 essential
  - Dashboard MutationGateGauge / PropertyCoverageGauge /
    OverMockHistogram components
  - 3 Tauri commands (get_mutation_gate_status,
    get_property_coverage, get_over_mock_stats)
  - 3 elai_cc.toml tables ([testing.mutation_targeted],
    [testing.property], [testing.anti_mock])
  - 9 deferred arXiv ID verifications (paper-ingestor follow-up)
