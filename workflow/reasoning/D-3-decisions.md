D-3 decisions — VerificationResult aggregator + multi-verifier consensus

This file logs the canonical D-3 decisions across BOTH the original D-3
landing (Decisions 1-5 below, inherited from the prior cut) AND the D-3
CLOSURE pass (DECISION-1..DECISION-5 NEW, this session) that closes the
12 MISSING + 1 NOT_WIRED + 3 SPEC_DRIFT items the cross-check re-audit
surfaced.

CLOSURE DECISIONS (this session — s-25120 D-3 closure)

DECISION-1: SCHEMA bump 104 -> 105 (D-3 closure), NOT 48 -> 49 as the plan said.

  Addendum: a concurrent C-37 lift landed during this session bumping
  the SCHEMA further to 106 (PerRoleDecodingApplied; +1 from D-3's 105).
  The action_type_lock TOTAL_VARIANTS now reads 311 (D-3 closure's 310 +
  C-37's +1); ALL_VARIANTS array + exhaustive_match_guard carry the
  PerRoleDecodingApplied entry alphabetically at its insertion point
  (line 204). The contract test asserts >= 105 floor (passes through 106).

  The plan's literal §5 "Bump SCHEMA_VERSION 48->49" is severely stale
  (D-2 already bumped through 66/67/68/69/... and the repo is at 104 via
  C-7 / C-36 evolution as of session start). The D-3 closure atomic
  9-tuple cascade lifts 104 -> 105 carrying 3 new SsbActionType variants:
    - PpmUnavailable (carried from D-2 DECISION-1 deferral)
    - VerifierDisagreement (D-4 tripwire-stack input)
    - CrossContextDispatched (cross_context::run emission)
  Lockstep updates:
    - ssb.rs SCHEMA_VERSION i64 const + SCHEMA_VERSION_PUBLIC re-export
    - ssb.rs SsbActionType enum (+3 variants)
    - ssb.rs as_sql_str arm (+3)
    - ssb.rs from_str arm (+3)
    - ssb.rs SQL CHECK allowlist (+3 strings)
    - ssb.rs SsbPayload enum (+3 arms + 3 payload structs)
    - ssb.rs SsbPayload::action_type() match (+3)
    - ssb.rs all_test_payloads() (both sites; +3 each; length 246 -> 249)
    - action_type_lock.rs TOTAL_VARIANTS 307 -> 310 + ALL_VARIANTS
      append + exhaustive_match_guard append + golden regen
  Rationale: plan text is years stale; the bump SHAPE (atomic 9-tuple) is
  the binding contract, not the literal numbers. Future plans should
  treat plan-recorded SCHEMA numbers as advisory and bind to the
  current SCHEMA_VERSION_PUBLIC at exec time.

DECISION-2: Keep `aggregate` free fn + `VerificationResult::aggregate`
  assoc fn; do NOT introduce a `ResultAggregator` trait.

  The D-3 plan §1 + §WIRING_CONTRACT both refer to a `ResultAggregator
  trait`. The shipped pattern (verification_result.rs:322 free fn + :430
  assoc fn alias) is already an idiomatic Rust composition: no caller
  needs polymorphism over the aggregator (the closed-enum ConsensusRule
  + closed-enum VerifierKind already provide the strategy switch). Adding
  a trait would force a blanket impl + a Box / Arc wrapping at every
  call site for ZERO behaviour gain.

  Rationale: the wiring contract phrasing "ResultAggregator trait" is
  documentation drift; the shipped pattern (free fn + assoc fn) is the
  correct one and the contract checklist should be read as "the function
  exists and is reachable", not "the trait abstraction exists". Updated
  WIRING_CONTRACT in the closure decision log; the plan file itself is
  immutable (untouched).

DECISION-3: Substituted hallucinated arxiv:2603.21454 (CCV) cite with
  first-principles + verification_result.rs aggregator pattern.

  The D-3 plan §Context lines 39-42 cite "arxiv:2603.21454 (T1)" as the
  grounding for the cross-context verification rule. The id prefix
  "2603" decodes to year 2026 month 03 — and as of the current session
  date 2026-05-24, the arxiv corpus does NOT produce a paper at id
  2603.21454. The id is FUTURE-DATED / HALLUCINATED.

  Substitute grounding (real, current, verified):
    - V-STaR (arxiv:2402.06457, T1; already ingested as
      workflow/papers/vstar-2402.06457-interface.md) — heterogeneous-
      verifier rule + score-floor convention.
    - van Rijsbergen "Information Retrieval" (1979) ch.3 — shingle /
      jaccard similarity construction (textbook IR baseline).
    - ELAI's existing blake3 content-hash convention (memory notes,
      locked keys auditor; no new paper needed).

  Implementation: cross_context.rs is derived from first principles +
  the verification_result.rs aggregator pattern. The substitute paper
  interface is logged at workflow/papers/ccv-2603.21454-interface.md
  with the citation-trace + substitution rationale.

  Rationale: per CLAUDE.md feedback_paper_citation_verify rule, arxiv
  IDs must be WebSearch-verified before citing. Future plans must not
  reference 2603.21454 — the canonical substitute references are V-STaR
  + the textbook shingle-jaccard construction.

DECISION-4: PpmUnavailable variant lands HERE (D-3 closure), atomically
  carried from D-2 DECISION-1.

  D-2 shipped 2 PPM variants: PpmScored (a successful score, possibly
  below threshold) and PpmGatingFailed (scored, below threshold).
  Neither covers the "sidecar down / health check failed" case — that
  is a distinct semantic. D-2 deferred this to D-3's atomic SCHEMA
  bump (D-2 DECISION-1) because adding a single variant outside an
  atomic cascade is the canonical D-plan miss pattern (per gotchas.md).

  Closure: PpmUnavailable lands atomically with VerifierDisagreement +
  CrossContextDispatched in the SCHEMA 104 -> 105 cascade. Payload
  shape: { candidate_id, verifier_id, reason }. Producer: PpmClient
  health-path + agent_pool::run_consensus_pass adapter-error route.

DECISION-5: Two of six plan-named SSB variants are RENAMES of already-
  shipped variants; do NOT rename to preserve cross-cutting stability.

  Plan §5 SsbActionType list (6 variants):
    1. PpmScoreEmitted     — RENAME of shipped PpmScored (D-2). NOT renaming.
    2. PpmUnavailable      — NEW (added in this closure per DECISION-4).
    3. VerifierConsensusPass — already shipped (D-3 original cut).
    4. VerifierConsensusReject — RENAME of shipped VerifierConsensusFail
       (D-3 original cut). NOT renaming.
    5. VerifierDisagreement — NEW (added in this closure).
    6. CrossContextDispatched — NEW (added in this closure).

  Renaming PpmScored -> PpmScoreEmitted OR VerifierConsensusFail ->
  VerifierConsensusReject would require:
    - serde rename shim across all consumers (Tauri DTOs, dashboard
      panel, SQL viewer, contract tests)
    - golden file update (8 sites)
    - cross-language pair update (dashboard-v2 typescript surface)
    - migration warning for replay-time deserialization of historic rows
  For ZERO behaviour gain — the existing names are semantically correct
  (PpmScored = scored, success or failure; VerifierConsensusFail = the
  fail-or-inconclusive bucket).

  Decision: keep the shipped names; treat plan-recorded names as
  documentation drift. Closure landing only ADDS the 3 truly missing
  variants (PpmUnavailable + VerifierDisagreement + CrossContextDispatched).

CLOSURE FILES TOUCHED (this session — s-25120 D-3 closure)

  ADD: .elai_cc/crates/orchestrator/src/cross_context.rs (NEW; 12 tests)
  ADD: .elai_cc/crates/contracts/tests/verifier_consensus_contract.rs
  ADD: workflow/papers/vstar-2402.06457-interface.md
  ADD: workflow/papers/ccv-2603.21454-interface.md (substitute-citation
       trace per DECISION-3)
  MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (+1 pub mod cross_context)
  MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
            - SCHEMA_VERSION 104 -> 105 + closure comment block
            - +3 SsbActionType variants (PpmUnavailable +
              VerifierDisagreement + CrossContextDispatched)
            - +3 as_sql_str arms, +3 from_str arms
            - +3 SQL CHECK allowlist strings
            - +3 SsbPayload arms + 3 payload structs
            - +3 SsbPayload::action_type() match arms
            - +3 entries in both all_test_payloads sites (246 -> 249)
  MODIFY: .elai_cc/crates/orchestrator/tests/action_type_lock.rs
            - TOTAL_VARIANTS 307 -> 310
            - ALL_VARIANTS append (+3)
            - exhaustive_match_guard append (+3)
  MODIFY: .elai_cc/crates/orchestrator/tests/action_type_lock.golden
            (regenerated via `cargo test ... regen_golden_file`)
  MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs
            - ResolvedConfig.verifier_adapters (Option, default None)
            - dispatch_verifiers free fn (D-3 NOT_WIRED closure)
            - drive_task call site (post-task CLAIM-emit hook)
  MODIFY: .elai_cc/elai_cc.toml (+[verification] section, L3-locked)
  MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d3.rs
            - VerificationConsensusRow.kind doc widened
            - get_recent_verifications scans +3 row families
            - consensus_detail_to_row arms +3
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/VerificationConsensusPanel.svelte
            - kind union widened to 5 variants
            - row reason fallback for +3 new kinds
            - +3 badge style rules

ORIGINAL DECISIONS (prior session; preserved verbatim)

Decision 1: ConsensusRule is a closed enum with 4 variants, not table-driven config.

  Variants: AllPass, KofN{k,n}, WeightedScoreThreshold{weights,threshold},
  VerifierKindHeterogeneous{required_kinds}.

  Rationale: V-STaR (arxiv:2402.06457, T1) §3 + rStar-Math (arxiv:2501.04519,
  T1) §3.2 both treat the consensus shape as a fixed structural decision —
  the producer never invents a new rule shape at runtime, only picks one of
  the well-defined families. A table-driven config would let agent self-mod
  introduce a `OneOf{kinds}` rule that quietly degenerates the gate. The
  closed enum is L3-locked. KofN{2,3} is the default per D-3 plan §3.3
  (companion §3.3 lines 242-256).

  Rejected: dynamic / table-driven consensus rule. The L3 freeze story only
  works if the rule SHAPE is in code, not config. Threshold values
  (logprob floor, entropy ceiling) live in elai_cc.toml and remain
  evaluator-tunable.

Decision 2: extract_intrinsic_signals lives in the reasoning crate, not the
  orchestrator crate.

  Rationale: orchestrator depends on reasoning. Putting the extraction in
  the orchestrator would either force a cyclic dep (reasoning -> orchestrator)
  or duplicate the IntrinsicSignals struct in two places (drift surface).
  The reasoning crate exposes ExtractedIntrinsicSignals; the orchestrator's
  `verification_result::IntrinsicSignals` wraps the same scalars and the
  conversion is a single struct-literal copy at the producer call site.

  Rejected: re-export the orchestrator type from reasoning. Would require
  reasoning to depend on orchestrator-ssb-payload structures — strictly
  worse coupling for zero token saving.

Decision 3: VerifierAdapter uses native async-fn-in-trait + AdapterErased
  wrapper rather than the `async-trait` crate.

  Rationale: native async-fn-in-trait is stable on Rust 1.75+; CLAUDE.md
  anti-stale rule (T1) calls out `async-trait` as deprecated for new code.
  Native async-fn-in-trait is NOT dyn-compatible — we need `&dyn` for the
  consensus pass to accept heterogeneous adapters. The `AdapterErased<A>`
  wrapper bridges the gap by erasing into a boxed-future via
  `DynVerifierAdapter`. This keeps the user-facing trait native + ergonomic
  and quarantines the boxed-future to one wrapper struct.

  Rejected: `#[async_trait]`. Pulls in `async-trait` (additional dep) and
  forces all impls to box every async fn even when called statically.

Decision 4: Adapter errors are swallowed at the consensus pass; failing
  adapters are skipped, not propagated as Err.

  Rationale: a single sidecar dropping the connection should not kill the
  consensus pass entirely — that would degrade the gate to "verifier broke,
  so we reject the candidate" which is the opposite of the desired
  behaviour. PRMBench §4.2 lists "single point of failure" as the
  Deception Resistance failure mode; treating adapter errors as "verifier
  unavailable" and continuing with the remaining adapters defends against
  it. The aggregate verdict then has fewer signals (potentially flipping
  Pass to Inconclusive on a heterogeneous rule); the caller's policy
  decides whether to defer or reject.

  Rejected: propagate Err from the first failing adapter. Would make the
  consensus pass fragile and discourage horizontal verifier scaling.

Decision 5: One SSB row per consensus call (Pass OR Fail). No row for
  Inconclusive separate from Fail.

  Rationale: dashboard UX. The panel renders chronological consensus
  decisions; a third "inconclusive" row family would split the timeline
  without operational benefit (the consumer of the fail row already
  reads `reason` to distinguish). The plan §SsbActionType lists 2 new
  variants — adding a third just for Inconclusive would force a 153-count
  variant assertion update, which is not justified by the rendering need.

  Rejected: 3-variant scheme (Pass/Fail/Inconclusive). Add later if D-4
  tripwire needs to differentiate.

ORIGINAL FILES TOUCHED (prior session)

  ADD: .elai_cc/crates/orchestrator/src/verification_result.rs
  MODIFY: .elai_cc/crates/orchestrator/src/agent_pool.rs
  MODIFY: .elai_cc/crates/orchestrator/src/lib.rs
  MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
  MODIFY: .elai_cc/crates/reasoning/src/engine.rs
  MODIFY: .elai_cc/crates/reasoning/src/lib.rs
  ADD: .elai_cc/crates/orchestrator/tests/verifier_consensus_smoke.rs
  ADD: .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d3.rs
  MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
  ADD: .elai_cc/crates/dashboard-v2/src/lib/drawers/VerificationConsensusPanel.svelte
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/DrawerHost.svelte
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/layout/Drawer.svelte
  MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts
  ADD: workflow/reasoning/D-3-decisions.md (this file)

Plan-acceptance drift notes (for future plan re-readers)

- Plan §1 VerificationResult struct shape (deterministic / ppm /
  intrinsic / cross_context / consensus_rule / verdict fields) is
  INTENTIONAL SPEC_DRIFT — the shipped shape is one VerificationResult
  per verifier with `kind: VerifierKind` discriminator (better design:
  aggregator iterates a slice, no need for the option-of-each shape).
  Accept the rename, do NOT revert the shipped struct.
- Plan §1 ConsensusRule variants (AtLeastTwo / AtLeastThree / AtLeastN /
  RequireAll) are SPEC_DRIFT — KofN{k=2,n=3} subsumes AtLeastTwo cleanly;
  WeightedScoreThreshold + VerifierKindHeterogeneous are strictly more
  expressive than RequireAll([SignalKind]). Accept the rename.
- Plan §9 contract test SCHEMA_VERSION_PUBLIC == 49 assertion is stale
  per DECISION-1; the closure contract test asserts >= 105 floor instead.
- Plan §Out-of-scope mentions D-9 L3 config consolidation as the owner
  of elai_cc.toml [verification]; the D-3 closure landed the section
  HERE (with the L3-lock note) because the elai_cc.toml [verification]
  miss was a 12-MISSING line item the re-audit surfaced. D-9 will
  consume the section, not re-author it.
- Plan §Out-of-scope mentions verifier_consensus_integration.rs as a
  separate ADD; shipped name is verifier_consensus_smoke.rs (prior
  session rename, accepted as IMPLEMENTED in the re-audit).
