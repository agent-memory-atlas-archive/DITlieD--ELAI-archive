C-50 — Contract-Coding STRUCTURAL contract test scaffolding (decisions)

Session: s-25120 session 2/4 of the s-25120 completion-drive cluster
(writing-session s-31708 paired in C-49 context).
Plan: docs/thefinalglance/C-plan/C-50_contract_coding_test_scaffolding.md
Paper: arXiv:2604.13100 (Lin, Zhao, Shi 2026 — "Contract-Coding:
Towards Repo-Level Generation via Structured Symbolic Paradigm").
Interface map: workflow/papers/contract-coding-interface.md.

WHAT SHIPPED (this session)
- New crate module crates/orchestrator/src/contract_coding/ with:
    mod.rs           (public re-exports + module header carrying
                      the load-bearing paper grounding caveats)
    spec.rs          (StructuralContract { namespace: Vec<String> N,
                      signatures: Vec<TypeSignature> Sigma,
                      state_space: Vec<StateConstraint> Delta } per
                      paper kernel K = <N, Sigma, Delta> §4.2;
                      validate() enforces disjointness + reference
                      invariants; SpecError taxonomy)
    emitter.rs       (emit_contract_test produces a Rust test-file
                      template scaffolding contract assertions over
                      N + Sigma + Delta; sanitize_ident guard so
                      arbitrary function names stay Rust-valid)
    discriminator.rs (audit_against_contract heuristic surface;
                      AuditResult { is_consistent, violations };
                      ContractViolation + ViolationKind taxonomy
                      MissingNamespace / MissingSignature /
                      UnrecordedStateConstraint per paper §4.4
                      Discriminator pattern)
    mutation.rs      (ContractMutation { Add(StateConstraint),
                      Update { target, new } } per paper §4.3
                      {Add, Update} closure; closure_check guard
                      against future Delete variant addition)
    scaffolder.rs    (scaffold_contract_test_from_signature entry
                      point; rejects empty function names; builds a
                      one-section / one-constraint kernel and
                      defers to emit_contract_test)
- ssb.rs +2 variants atomically with SCHEMA_VERSION 124 -> 125:
    ContractTestScaffolded(ContractTestScaffoldedPayload)
    ContractViolationDetected(ContractViolationDetectedPayload)
  9-tuple cascade applied (enum + SQL CHECK + as_sql_str + from_str
  + SsbPayload + action_type() + test_payload + all_test_payloads +
  count assert 287 -> 289 + golden file regen with 2 new
  alphabetical entries / TOTAL_VARIANTS 348 -> 350).
- CLI surface: `elai contract scaffold|audit|list` via
  crates/cli/src/contract_cmd.rs + CliAction::Contract +
  ContractAction { Scaffold, Audit, List } in args.rs;
  parse_contract_subcommand_args dispatcher + "contract" =>
  parse_contract_subcommand_args entry in the top-level subcommand
  match.
- Contract test crates/contracts/tests/c50_contract_coding_scaffolding_contract.rs
  (G1..G6 fences).

KEY DECISIONS + GROUNDING

1. PLAN REBASE: crates/code_completeness/ does not exist.
   The plan opens by declaring `crates/code_completeness/` as the
   home crate. The workspace has no such crate (sibling C-49 used
   the same relocation pattern — plan said crates/agent/, real
   home was crates/orchestrator/src/experience/). C-50 deliverables
   relocated to crates/orchestrator/src/contract_coding/ as a
   subdirectory module (mirrors credit_attribution/, experience/,
   routing/ layout shipped this session-cluster).

2. SCOPE CUT: STRUCTURAL contracts only, NOT pre/postcondition
   emission.
   The plan body describes a "pre/postcondition emitter" surface
   (ContractEmitter + ContractViolationCheck + MiniCritic
   extension + StructuralRetry::ContractViolationRetry variant).
   The paper (arXiv:2604.13100) has NO Hoare triples, NO temporal
   logic, NO Eiffel/JML-style assertions. The paper's "contract"
   is a STRUCTURAL artifact (kernel K = <N, Sigma, Delta>; Section
   4.2). C-50 ships the paper-grounded STRUCTURAL surface only;
   the pre/postcondition extension is rejected as ungrounded.
   This decision is logged as the 5th cite incident
   (real-but-misframed pattern) in workflow/sessions/s-25120/
   mistakes.md.

3. HEURISTIC Discriminator, NOT RL-trained.
   Paper Algorithm 1 (§4.4 + §4.5) describes a Generator Agent +
   Discriminator Agent pair trained end-to-end with GRPO-style RL.
   Reproducing it requires a feedback model M_FB and a scalar
   reward over kernel-violation counts we do not yet compute.
   C-50 ships the STRUCTURAL discriminator only — substring-match
   over the namespace / signature / constraint anchors. The RL
   trainer + Discriminator Agent rectification pass are
   RUNPOD-PENDING and documented as such in module headers.

4. CITATION DISCIPLINE — exact-quote required.
   Plan asserts "47% Greenfield-5 contract-test improvement". The
   exact paper phrasing (Section 5.2): Contract-Coding TIES Trae
   at 47% on the Roguelike subset (subset 5 of Greenfield-5);
   Gemini Studio beats Contract-Coding at 63%; OpenHands trails at
   30%. The paper argues a qualitative differentiator: failures
   are "strictly confined to Local Logic Errors, not architectural
   collapse". The corrected framing — including the Roguelike
   subset + TIES-Trae + structural-integrity qualifier — is
   embedded in:
     - the contract test header
     - the SsbActionType::ContractTestScaffolded + ::
       ContractViolationDetected payload docstrings
     - the orchestrator/src/contract_coding/mod.rs header
     - the contract_cmd.rs handler header
   Failure to thread this framing through to the producer surface
   would be a BLOCK-on-AUDIT per the wiring-contract-validator
   FUTURE-FIX in workflow/sessions/s-25120/mistakes.md.

5. MUTATION ALPHABET — {Add, Update} only, paper §4.3 closure.
   Paper deliberately excludes Delete to avoid kernel
   inconsistency (a Delete on Sigma would orphan StateConstraint
   references in Delta; a Delete on N would orphan signatures).
   ContractMutation enum has EXACTLY 2 variants and a
   closure_check helper that does an exhaustive match on the
   2-variant alphabet so a future contributor cannot silently add
   a Delete variant without surfacing a compile error.

6. SCHEMA cascade target: 124 -> 125.
   SCHEMA_VERSION fresh-read at session entry: 124 (post C-49 +
   D-14). C-50 lands at 125. Two new SsbActionType variants ship
   atomically with the SCHEMA bump per the 9-tuple cascade
   precedent (C-45 / C-46 / C-47 / C-49 quadruple-confirmed; C-50
   is the 5th application). The contract test G6 fence asserts
   SCHEMA_VERSION_PUBLIC >= 125 (floor, not equality — leaves
   headroom for concurrent sibling plans).

7. NO CLI new top-level deps.
   contract_cmd.rs reuses serde_json + serde (already in cli
   Cargo.toml) for spec encoding. No new external deps.

8. NO unwrap() / expect() in production paths.
   contract_coding module uses Result propagation throughout.
   The emitter does not call unwrap(); failures bubble through
   EmitError. The contract test helpers use expect("reason")
   per the test-code carve-out in CLAUDE.md "No .unwrap() in
   production" rule.

9. WIRE-DARK posture.
   C-50 CLI surface is observable but NOT auto-fired by any
   production code path. The two SSB row variants are emitted
   only when the CLI handler invokes the orchestrator's
   contract_coding module. The C-31/C-32 substrate (FILE_PLAN +
   MiniCritic + StructuralRetry) is NOT wired to emit C-50 rows
   in this session — that wire-up is RUNPOD-PENDING with the
   full Generator -> Discriminator pipeline. Matches the wire-
   dark precedent (D-5 / D-10 / D-11 / D-12 / D-13 / D-14 / C-49).

10. NO MiniCritic / StructuralRetry mutation this session.
    The plan body describes MiniCritic::ContractViolationCheck +
    StructuralRetry::ContractViolationRetry stage variant. Both
    are scope-cut per Decision 2 — the paper does not ground
    behavioral assertions, so no MiniCritic extension is
    paper-warranted. The C-32 contract pins (MiniCriticVerdict
    tri-state + StructuralRetryPlan stage order) remain unchanged.

RUNPOD-PENDING (deferred to future billable session)
- Full Contract-Coding Generator -> Discriminator RL training
  pipeline per paper Algorithm 1 (§4.4 + §4.5):
    contract_coding_gen_disc_rl_qwen3_8b
  Requires: a feedback model M_FB, GRPO-style RL infrastructure,
  scalar reward signal over kernel-violation counts, and live
  Generator Agent prompt orchestration. The C-50 heuristic
  Discriminator (substring-match) is the structural pre-stage
  that exists today.

CITATION VERIFICATION
- arXiv:2604.13100 — verified via paper-ingestor mode=paper output
  at workflow/papers/contract-coding-interface.md. The interface
  doc carries a "Citation-accuracy notice" section anchoring each
  numeric claim to its exact-quote + section anchor. CITE_PARTIAL
  flag on the 47% headline (subset-only + TIES Trae) and on the
  "pre/postcondition emitter" plan framing (unsupported by paper).

LOCKSTEP SITE CHECKLIST (9-tuple confirmed)
  [x] enum SsbActionType        (ssb.rs ~L3960 / appended at end)
  [x] SQL CHECK constraint      (ssb.rs ~L2280 / appended in block)
  [x] as_sql_str arm            (ssb.rs ~L4488 / contract_* lines)
  [x] from_str arm              (ssb.rs ~L4955 / contract_* lines)
  [x] SsbPayload enum variant   (ssb.rs ~L7820 / appended at end)
  [x] action_type() match arm   (ssb.rs ~L8354 / appended at end)
  [x] test_payload sample dispatch (ssb.rs ~L15737 / appended)
  [x] all_test_payloads vec     (ssb.rs ~L18261 / 287 -> 289)
  [x] action_type_lock.{rs, golden} (348 -> 350; golden regen ok)

NEXT STEPS (executor handoff)
- After audit-pass, memory-writer logs C-50 closure in
  memory-bank/activeContext.md + memory-bank/changelog.md.
- C-50 plan body retains its "pre/postcondition" framing in the
  prose; the implementation rebases to STRUCTURAL per Decision 2.
  Plan rewrite is OUT OF SCOPE this session — the corrected
  framing is documented in this decisions doc + the contract test
  header + the mistakes.md 5th incident note.
- The RUNPOD-PENDING entry is the load-bearing TODO for a future
  session.
