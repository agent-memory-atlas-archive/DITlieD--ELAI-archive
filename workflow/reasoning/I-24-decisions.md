I-24 decisions (isomorphic canary anti-gaming)
2026-06-14

WHAT: scorer-side anti-gaming canary. Injects AST-preserving isomorphic code pairs
into the i-17 L2 scoring path; a scorer that rates semantically-identical variants
differently is flagged as style-biased and (after N consecutive detections) its weight
is zeroed for the run.

KEY DECISIONS

FORK-1 (controller resolved, A179 Option B):
  Ship a CPU canary SKELETON. The canary wraps the existing conformal-tau L2 stub
  (run_l2_score). The live dual-score through PpmClient is WIRE-DARK[I-24] until
  D-026 wires PpmClient into run_l2_score. Rationale: the conformal-tau stub returns
  a constant tau for all inputs (no surface-feature sensitivity), so Style- BiasDetected
  will never fire against real content on the current tree. The skeleton confirms SSB
  plumbing (PrmCanaryEvent), threshold logic, neutralize state machine, and the
  production wiring into run_verdict. The behavioral gate activates post-D-026.

FORK-2 (controller resolved, A179 reuse MUCOCO):
  Do NOT add duplicate MetamorphicRelation arms (ForWhileEquiv/RenameIdempotent/
  ConstantExtraction). The obfuscations already exist as MucocoOperator variants
  (for_to_while, constant_unfolding). At runtime the canary reads FROZEN pairs from
  fixtures/canary_pairs.json (authored using the MUCOCO operators out-of-band). No
  new relation dispatch layer needed. The frozen fixture approach is also the R8
  compliance path.

PRODUCTION_HOOK:
  CanaryInjector::maybe_inject is called from TriLayerStack::run_verdict at the L2
  call site (verifier_stack.rs, immediately after run_l2_score). Production caller
  chain: task_runner::dispatch_tri_layer_verdict -> TriLayerStack::run_verdict
  (gated ELAI_TRI_LAYER_ENABLE). The canary is gated by ELAI_CANARY_INJECTOR_ENABLE
  (default-OFF, R9); when the gate is off, maybe_inject is a no-op.

SCHEMA VERSION:
  Live SSB_ACTION_TYPES at HEAD = 574 entries. After PrmCanaryEvent: 575 entries.
  SCHEMA = -309 + 575 = 266. Controller instruction said 264/573->265/574 (stale).
  Actual: 265/574->266/575.

ID-4 COMPLIANCE:
  ScorerNeutralized zeroes the scorer weight for the run (L2-only effect). It cannot
  override or resurrect an L1 HardVeto. L1 HardVeto is structurally prior: run_verdict
  returns early at L1 and never reaches the L2 call site where the canary fires.

ID-5 COMPLIANCE:
  Bias threshold read from metrics/canary_bias_threshold.toml (bootstrap: 500 bp).
  Not a paper-derived constant. Pending re-measurement post-D-026 against live PPM scorer.

R8 COMPLIANCE:
  fixtures/canary_pairs.json added to gate-manifest.txt. File authored out-of-band
  with 6 pairs covering for_to_while / rename_idempotent / constant_unfolding operators
  in Python and Rust. The running system reads but never writes this file.

GD-1 COMPLIANCE:
  canary_injector.rs composes the existing run_l2_score closure (does not duplicate it).
  Does not re-implement H-44 TrapBehaviorDetected (candidate-side detection stays in
  trap_behavioral.rs; I-24 is scorer-side only).

UNKNOWN / RESIDUAL:
  - Live StyleBiasDetected on real content: blocked until D-026 (PpmClient into L2).
  - metrics/canary_bias_threshold.toml value: bootstrap 500 bp; re-measure post-D-026.
  - Frontend scorer-health badge: deferred (dashboard cross-lang pair); the SSB rows
    are produced; badge wiring is a follow-on dashboard plan.

EXPECTED TEST OUTCOME:
  cargo check --workspace --tests -> exit 0 (compile only).
  action_type_lock test: PASS (golden gains prm_canary_injected, exhaustive match arm added).
  e45 cross-language drift contract: PASS (new Rust SSB action_type, no TS counterpart
  needed since badge is deferred and the SSB emit is Rust-only for now).
  i24 contract (when authored by auditor): real-DB SSB row landing test via TempDir SQLite.
