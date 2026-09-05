NASA-GRADE DETERMINISTIC GATES — steal analysis + gate-proposal plan

Source: deep-research-report(1).md (2026-06-21, "Deterministic LLM Harness Rule
  Enforcement Research Report", 31-work catalogue). 13 of the 31 works ingested
  faithfully (copy-not-type, source>text) into workflow/papers/*-interface.md by
  13 sonnet paper-ingestor agents, this session.
Goal: turn the ingested works into NEW or EXTENDED deterministic harness GATES so
  that code ELAI GENERATES is NASA-grade, and ELAI's OWN gate machinery is itself
  verified. These are harness gates (siblings of maintainability_check.py /
  production_caller_check.py / traced_test_assert_check.py), NOT .md coding rules.
Status: PLANNED. Reasoning artifact + plan units. Index/feature-map upsert is a
  follow-up the user approves separately (paper-to-plan phase 5 not yet run).
Grounding discipline: every gate cites the paper section it derives from; verdicts
  are checked against docs/thefinalglance/ELAI-FEATURE-MAP.md (read this session,
  section 1.6 verification/trust-gates + 1.2 reasoning + 1.3 orchestration).

================================================================================
PART 1 — PER-WORK STEAL VERDICT (what each ingested work yields)
================================================================================

Interface maps on disk (all 13, faithfully extracted, RAG-unindexed — see Part 4):
  workflow/papers/power-of-ten-interface.md
  workflow/papers/enforceable-security-policies-interface.md
  workflow/papers/edit-automata-interface.md
  workflow/papers/efficient-monitoring-safety-interface.md
  workflow/papers/rv-survey-interface.md
  workflow/papers/cbmc-interface.md
  workflow/papers/do333-case-studies-interface.md
  workflow/papers/tla-specifying-systems-interface.md
  workflow/papers/cedar-interface.md
  workflow/papers/spark-gnatprove-interface.md
  workflow/papers/nasa-std-8739-8b-interface.md
  workflow/papers/raft-interface.md
  workflow/papers/monitoring-rewriting-interface.md

W1 Power of Ten (Holzmann 2006) -> NEW GATE. A generated-code rule checker. The
  existing maintainability_check.py (MR-1..MR-4: file-size, casts, long-fn,
  branching) does NOT cover the P10 rule set. Net-new: acyclic call graph, every
  loop statically bounded, assertion density >=2, every return value checked,
  zero-warnings-as-CI-break, no dyn-alloc in hot loops, cfg-complexity<=2.
  Verdict: NEW (group A), sibling of maintainability_check.py. Headline item.

W2 Enforceable Security Policies (Schneider 2000) -> DOCTRINE (not a checker). The
  safety-property feasibility test (3 conditions) decides which gates CAN be
  runtime SSB monitors vs which must move to admission/plan time, and forces
  step-count-not-wall-clock predicates. Grounds groups B and C.
  Verdict: NEW doctrine note + feasibility lint (group B).

W3 Edit Automata (Ligatti 2005) -> EXTENSION(J-12 RuntimeInterventionHarness). ELAI
  gates today only BLOCK (truncate). The 4-class hierarchy adds suppress / insert /
  edit gate actions. J-12's H2 pre-tool gate is "Block(msg)" = truncation-only;
  this upgrades it to a typed GateAction {Accept, Suppress, Insert, Block}.
  Verdict: EXTENSION(J-12) — typed gate-action taxonomy (group C).

W4 Efficient Monitoring of Safety Properties (Havelund/Rosu 2004) -> EXTENSION(I-35
  TemporalToolSequenceEnforcement) + NEW monitor-synthesis engine. Past-time-LTL
  online monitor compiled to O(1)-per-event recurrences over SSB rows. The agent's
  Gate-3 ("TestOutcome passed=true preceded by stdout_hash row") is ALREADY-SHIPPED
  as F-4 cant-lie gate (confirms the method). Gate-1/Gate-2 are net-new SSB temporal
  gates. Verdict: EXTENSION(I-35) (group B).

W5 A Brief Account of Runtime Verification (Leucker/Schallhart 2009) -> DOCTRINE. The
  layer-assignment table (proof=invariants, RV=live policy, tests=behavior) + the
  monitorability / 3-valued caveat that prevents misapplying W3/W4/W13.
  Verdict: NEW doctrine note (group B guardrail).

W6 CBMC (Clarke/Kroening/Lerda 2004) -> NEW GATE via KANI (Rust analog, verified
  active kani-verifier 0.67.0). Bounded model check on small critical ELAI Rust
  modules: ssb/payloads.rs serializer round-trip, FSM transition fn, gate predicates
  fail-closed, wiring_contract parser no-panic, budget counters no-underflow.
  Verdict: NEW (group D). Distinct from D-11 constructive-checker (z3/sympy on
  generated math) — this verifies ELAI's OWN code.

W7 Formal Methods Case Studies for DO-333 (Cofer/Miller 2014) -> NEW gate-manifest
  traceability schema. Each frozen gate gets: which verification objective it covers,
  which technique (proof/model-check/abstract-interp/RV/test), what evidence it
  produces, and a REPLACE/AUGMENT/CANNOT-SATISFY classification. Indeterminate is
  never a pass. Verdict: NEW (group E), enriches .claude/gate-manifest.txt.

W8 Specifying Systems / TLA+ (Lamport 2003) -> NEW spec-time gate. Formally spec the
  9-state FSM (ElaiFSM.tla sketched by the agent) + model-check invariants via TLC;
  a privileged FSM change ships a spec delta + passing TLC run, counterexample=BLOCK.
  Partial overlap with phase-3I DST (madsim proptest 10000) — TLA+ adds EXHAUSTIVE
  BFS over the small finite state space + a concrete counterexample trace.
  Verdict: NEW spec artifact / EXTENSION(phase-3I) (group D).

W9 Cedar (OOPSLA 2024, verified cedar-policy 4.11.1) -> SPIKE-FIRST architecture
  candidate. Express transition/capability gates as analyzable policy-as-code over
  (principal=agent-role, action=tool/transition, resource=file/SSB-scope,
  context=FSM-state); buys SMT policy-equivalence/regression analysis (prove a gate
  refactor changed no permissions). Bigger move; tradeoff vs frozen-Python checkers.
  Verdict: NEW-but-SPIKE (group G), relates to I-1/I-35. Do not commit before a spike.

W10 SPARK/GNATprove (verified Kani primary, Creusot ok, Prusti REJECTED unmaintained)
  -> EXTENSION(I-1 supervisor plane + W6 Kani gate). Contract-first (pre/post) +
  AoRTE proof obligation on privileged side-effecting Supervisor-plane fns (file ops,
  egress, SSB write, creds). The CLAUDE.md no-.unwrap()/no-panic rule IS the runtime
  AoRTE guarantee; Kani makes it proof-time. Verdict: EXTENSION(I-1 + group D).

W11 NASA-STD-8739.8B (2022) -> CONFIRMATION + gap riders. Auditor!=executor +
  frozen gate already satisfy IV&V technical independence + records-retention. NET-NEW
  GAPS found: (1) financial independence absent — auditor & executor share one token
  budget; (2) managerial independence partial — orchestrate H-10 exceptions let the
  orchestrator skip the auditor. Verdict: NEW doctrine note + 2 gap riders (group E).

W12 Raft (Ongaro/Ousterhout 2014) -> EXTENSION(I-18 + J-22), CONFIRMATION. SSB is
  already single-writer WAL, append-only, RRA-replayable (= single-sequencer +
  Leader-Append-Only + State-Machine-Safety). NET-NEW gap: SSB rows have NO prev_hash
  — a blake3 hash-chain (Log-Matching analog) detects tamper/reorder in an O(n) scan
  before replay. This is EXACTLY planned I-18 (per-row chain_prev_hash) + J-22
  (writer-isolation test). Plus a watermark-monotonicity gate (Leader-Completeness).
  Verdict: EXTENSION(I-18 + J-22) — confirms those are correct (group F).

W13 Monitoring by Rewriting (Havelund/Rosu 2001) -> EXTENSION(I-35), the future-time
  SESSION-END companion to W4. Future-obligation gate by formula-rewriting; an
  unresolved obligation at session-end = BLOCK (fail-closed end-of-trace verdict, not
  "inconclusive"). E.g. "every spawned agent emits a return row before session-end",
  "every BLOCK finding reaches closure". Verdict: EXTENSION(I-35) (group B).

================================================================================
PART 2 — GATE PROPOSALS (vertical-slice units per wiring-enforcement PRE_GATE:
  each ships producer + in-harness consumer + a smoke through a real entrypoint)
================================================================================

GROUP A — GENERATED-CODE POWER-OF-TEN GATE  [headline; NEW; W1]
  WHAT: .claude/tools/power_of_ten_check.py — a deterministic checker the AUDITOR
    runs over ELAI-generated Rust/Python/TS in a changeset. Rules (from
    power-of-ten-interface.md DETERMINISTIC-GATE RULES, all 10 verbatim there):
    P10-1 acyclic call graph (no direct/indirect recursion in agent/task loops);
    P10-2 every loop statically bounded (flag `loop{}`/`while cond{}`/`while True:`
      without a literal/`.len()` bound or break-after-N); P10-3 no heap alloc in the
      hot dispatch path; P10-4 fn <=60 lines (tighten the existing MR-3 80->60 for
      flagged safety-critical modules only); P10-5 assertion density >=2/fn + ban
      trivially-true asserts; P10-7 every non-void return checked (Rust: clippy
      -D unused_must_use; Python: mypy strict); P10-8 cfg-complexity <=2 non-test;
      P10-9 no fn-pointer/`dyn Fn` dispatch where a concrete type fits; P10-10
      zero-warnings-as-BLOCK (cargo clippy -D warnings, ruff/tsc/eslint --max-warnings 0).
  PRODUCER: power_of_ten_check.py (--files <changed> --base HEAD --json), named
    constants for every threshold, frozen in .claude/gate-manifest.txt.
  CONSUMER: auditor invokes it in AUDITING over the changeset (off-agent, like
    maintainability_check.py); a P10-BLOCK class fails the audit verdict.
  SMOKE: run on a hand-seeded violating fixture (unbounded loop, 80-line fn,
    assert(true), `let _=` on a Result) -> expect exit 1 with the right tags.
  TAGS: C-P10-LOOP / C-P10-RECURSION / C-P10-ASSERT / C-P10-RETURN / C-P10-WARN /
    C-P10-FNLEN / C-P10-CFG. BLOCK on loop/recursion/warn/return; WARN otherwise.
  CALIBRATION: ship OBSERVE-ONLY first (WARN-only) for 1-2 plan cycles, measure the
    false-positive rate on real ELAI diffs, then promote the hard-BLOCK subset.
  RISK: P10 was written for C; Rust/Python analogs are approximations (P10-2 loop
    bound, P10-9 dyn-dispatch). Keep those WARN, keep the binary ones (P10-4/7/10)
    BLOCK. Do NOT auto-tighten MR-3 globally to 60 — only flagged modules.

GROUP B — SSB TEMPORAL-MONITOR ENGINE  [EXTENSION(I-35); W2+W4+W5+W13]
  WHAT: a deterministic monitor-synthesis engine over the append-only SSB stream.
    Two legs:
    B1 past-time-LTL ONLINE monitors (W4): compile a ptLTL safety formula over SSB
      action_types into the O(1)-per-event recurrences (efficient-monitoring-safety-
      interface.md Part B). Fires per-row. Concrete gates given by the agent:
        G1 always(write_tool -> diamond policy_allow)   [write-after-allow]
        G2 always(egress -> not [scope_widen, declassify)_s)  [no-egress-after-widen]
      (G3 "test_passed -> diamond stdout_hash" is ALREADY F-4 — do not re-build.)
    B2 future-time SESSION-END monitors (W13): formula-rewriting; an obligation
      undischarged at session close = BLOCK (monitoring-rewriting-interface.md). E.g.
        Oblig1 []( agent_spawned -> <> agent_returned )   checked at session-end
        Oblig2 []( finding_block -> <> finding_closed )
  FEASIBILITY PRE-LINT (W2, Schneider): before any temporal gate is added, assert its
    predicate is a SAFETY property (3-condition test) AND uses STEP/EVENT counts not
    wall-clock. A liveness/wall-clock predicate is rejected at authoring (moved to a
    watchdog, not an SSB gate). The RV-survey layer table (W5) is the routing doctrine.
  PRODUCER: crates/orchestrator/src/ssb_monitor/ — a ptLTL compiler + a rewriting
    evaluator; gate predicates registered against SsbActionType. Deterministic, LLM
    never adjudicates (TD-3).
  CONSUMER: online monitors run in the SSB broadcast-after-commit path; session-end
    monitors run in the MEMORIZING->COMPLETE transition. Verdict feeds the FSM gate.
  SMOKE: drive the REAL entrypoint on a HARD input that fires a write-tool with no
    prior allow row (expect G1 BLOCK) and a session that spawns an agent that never
    returns (expect Oblig1 BLOCK at session-end).
  SSB: +SsbMonitorVerdict action_type (or reuse I-35's). Wire-dark allowlist row +
    promotion plan if shipped default-OFF.
  NOTE: this is the concrete compilation algorithm I-35 (Pro2Guard DTMC) lacked.

GROUP C — MULTI-MODE GATE ACTIONS  [EXTENSION(J-12); W3]
  WHAT: replace the boolean BLOCK with a typed GateAction enum from the Edit-Automata
    hierarchy (edit-automata-interface.md): {Accept, Suppress(drop the action, continue),
    Insert(remediation actions before the action), Block(truncate)}. Edit = suppress+insert.
  PRODUCER: a GateAction type + a dispatcher in the PreToolUse / J-12 H2 chain that
    can drop a dangerous tool-call (Suppress), inject a mandatory approval/declassify
    step (Insert), or redact a secret from a tool result (Edit) — not only kill.
  CONSUMER: the existing pre-tool gate chain (rules-3F / hooks-4C / J-12) returns a
    GateAction instead of bool; the orchestrator applies it.
  SMOKE: a tool-call that should be SUPPRESSED (continues) and one that should INSERT
    an approval step (the approval row appears before the side effect) through the
    real PreToolUse path.
  FEASIBILITY: per W2, classify each policy's enforceability class BEFORE choosing its
    mode (suppressible vs insertion-based vs not-runtime-enforceable). Do not promise
    "edit" semantics for a non-safety property.
  RISK: Insert/Edit change the event stream — they MUST be journaled (the inserted/
    suppressed actions are SSB rows) or replay/audit breaks (ties to group F).

GROUP D — FORMAL VERIFICATION OF ELAI'S OWN CRITICAL CODE  [NEW; W6+W8+W10]
  D1 KANI BOUNDED-MODEL-CHECK GATE (W6/W10): #[kani::proof] harnesses on the small
    critical modules named in cbmc-interface.md — ssb/payloads.rs serialize/deserialize
    round-trip, the FSM transition fn (no panicking next-state), gate predicates are
    fail-closed (evidence-absent -> Block), wiring_contract parser never panics on
    bounded input, budget counters never underflow. Auditor runs `cargo kani` in CI;
    an unwinding-assertion failure OR a found counterexample = BLOCK; HARNESS_NOT_FOUND
    = BLOCK (fail-closed). UNSOUND beyond the unwind bound — a Layer-2 leg, NOT a
    replacement for runtime gates.
  D2 SPARK-STYLE CONTRACTS ON PRIVILEGED ADAPTERS (W10): every side-effecting
    Supervisor-plane fn (file ops, egress, SSB write, credential handling) carries
    pre/post (assert! at entry, debug_assert! before return) + an AoRTE obligation
    discharged by a D1 harness, before it is allowed to be a production side-effect
    sink. Pairs with the no-.unwrap() rule (panic = Constraint_Error = AoRTE failure).
    Crates: kani (primary), creusot (optional gold); Prusti REJECTED (unmaintained).
  D3 TLA+ FSM SPEC + TLC GATE (W8): commit ElaiFSM.tla (sketched in
    tla-specifying-systems-interface.md) with invariants Inv2 (AUDITING only from
    EXECUTING), Inv3 (no EXECUTING without evidence_gate=TRUE), Inv4 (COMPLETE only
    after MEMORIZING). A privileged FSM change (new state/transition/gate condition)
    ships a spec delta + a passing TLC run; an invariant counterexample = BLOCK.
    Complements phase-3I DST (runtime sim) with design-time exhaustive BFS.
  PRODUCER/CONSUMER/SMOKE: harnesses + a .tla + a CI invocation; auditor re-runs them
    off-agent; smoke = a deliberately broken transition fn / a deliberately bad FSM
    edit -> expect the respective BLOCK.
  RISK: Kani/TLC are toolchain deps (CI cost). Scope to the SMALL critical core only,
    never the whole workspace (cbmc-interface.md is explicit: do not prove the world).

GROUP E — ASSURANCE DISCIPLINE  [NEW doctrine + manifest enrichment; W7+W11]
  E1 GATE-MANIFEST TRACEABILITY SCHEMA (W7/DO-333): extend .claude/gate-manifest.txt
    so every frozen gate row declares {objective_covered, technique
    (proof|model-check|abstract-interp|RV|test|static), evidence_artifact
    (SSB-row|coverage|proof|hash), satisfaction (REPLACE|AUGMENT|PARTIAL)}. An
    indeterminate/UNKNOWN gate result is never a pass (mirrors maintainability
    unknown=FAIL). Gives a requirements->gate->evidence trace matrix.
  E2 IV&V INDEPENDENCE GAP-CLOSURE (W11/NASA-STD): two measured gaps to close —
    (a) FINANCIAL independence: give the auditor a SEPARATE token/cycle budget the
      orchestrator cannot raid to "save cost" (the exact rationalization
      orchestrate-protocol.md already bans; this makes it structural).
    (b) MANAGERIAL independence: the auditor authority path must not be short-circuitable
      by the orchestrate H-10 skip exceptions for safety-critical changesets.
  These are doctrine + small structural changes, not big builds. Mostly CONFIRMATION
    of the existing auditor!=executor + frozen-gate posture.

GROUP F — SSB LOG-INTEGRITY HASH CHAIN  [EXTENSION(I-18 + J-22); W12]
  WHAT: add prev_hash = blake3(canonical_prev_row_bytes) to each SSB row (Raft
    Log-Matching analog) so a tampered/reordered/truncated log fails an O(n) chain
    scan BEFORE the O(n) RRA replay. Plus a watermark-monotonicity check (a new
    session must start at >= the prior committed watermark; Leader-Completeness analog).
  STATUS: this is ALREADY PLANNED as I-18 (UPIP per-row chain_prev_hash, 5-layer stack
    hash, offline blake3-Merkle default) and J-22 (writer-isolation contract test,
    PunkGo sole-commit-point). W12 is independent CONFIRMATION those are the right
    design; fold the watermark-monotonicity gate in as an I-18 rider, do NOT author a
    competing plan. blake3 not sha256 (CLAUDE.md anti-stale).
  CONSUMER: the chain scan runs at session-open and in RRA replay --verify; a broken
    chain = BLOCK. SMOKE: hand-mutate one row's payload -> expect chain BLOCK.

GROUP G — POLICY-AS-CODE SPIKE (CEDAR)  [SPIKE-FIRST; W9]
  WHAT: a time-boxed SPIKE (not a commitment) evaluating whether ELAI's transition/
    capability gates should be expressed as Cedar policies over (principal=agent-role,
    action=tool/transition, resource=file/SSB-scope, context=FSM-state). The payoff is
    Cedar's SMT policy-equivalence analysis: prove a gate refactor or a new wire-dark
    exemption changed NO permissions (cedar-interface.md Theorem 4.2). cedar-policy
    4.11.1 verified (Apache-2.0). TRADEOFF: adds an SMT dep + a second source of gate
    truth vs the current frozen-Python checkers. Decision deferred to spike output.
  DO: a 1-agent spike that re-expresses ONE existing capability gate as a Cedar policy
    + schema, runs the equivalence check across a refactor, and reports
    keep-Python vs adopt-Cedar with measured numbers. No production wiring until then.

================================================================================
PART 3 — SEQUENCING + CROSS-CUTTING
================================================================================

RECOMMENDED ORDER (cheapest-highest-leverage first; matches the report's own build
  order: eliminate ambiguity, enforce structure, qualify the path, expand):
  1. GROUP A (Power-of-Ten gate) — highest leverage, fully deterministic, the
     user's headline "NASA-grade generated code", ships OBSERVE-ONLY then promotes.
  2. GROUP E1 (gate-manifest traceability) — cheap, makes every other gate auditable;
     do alongside A.
  3. GROUP F (hash chain) — fold into I-18/J-22 (already planned); confirms not invents.
  4. GROUP B (temporal monitors) — extends I-35; the Schneider/RV doctrine (W2/W5)
     lands here as the authoring pre-lint.
  5. GROUP D1 (Kani gate) — start with the SSB serializer + FSM transition harnesses
     (smallest, highest-value); D2 contracts + D3 TLA+ follow.
  6. GROUP C (multi-mode gate actions) — needs B+F first (journaled inserts/suppresses).
  7. GROUP E2 (independence gaps) — doctrine + small structural change.
  8. GROUP G (Cedar spike) — last; gated on a real spike result, may be declined.

ALREADY-SHIPPED / DO-NOT-REBUILD (verified against ELAI-FEATURE-MAP.md):
  - "test_passed preceded by stdout_hash" temporal gate = F-4 cant-lie gate (WIRED).
  - single-writer append-only replayable log = SSB (phase-3B) + RRA (G-2) (WIRED).
  - untrusted-planner-in-trusted-envelope = DIRECTIVE SUPERVISOR PLANE I-1 (PLANNED).
  - per-row chain hash + writer-isolation = I-18 + J-22 (PLANNED) — group F folds in.
  - z3/sympy/compile checkers = CONSTRUCTIVE-CHECKER REGISTRY D-11 (DARK) — Kani is
    distinct (verifies ELAI's Rust, not generated math).
  - obligation-DAG closure (sorry/admit disqualifies) = OBLIGATION-CLOSURE GATE H-54
    (PLANNED) — group B's future-obligation leg is adjacent, keep them separate.

CROSS-CUTTING NOTES (surface to user):
  N1 RAG INGEST FAILED ON ALL 13 — the local-rag primary is down (known reranker
     crash, project_rag_reranker_crash.md). The interface maps are on disk and are
     the ground truth, but they are NOT indexed into RAG. Recovery: clear primary.pid
     + restart, then re-ingest the 13 .claude/papers/*_extracted/*.md. Offer pending.
  N2 SCOPE TOUCH — the Cedar agent wrote .elai_cc/tests/contracts/test_cedar_contract.py
     (premature; we are at plan stage). Uncommitted, harmless. Remove or keep as a
     ready stub — user's call.
  N3 RETRACTED (was: "product has no first-class faithful source-ingest feature").
     WRONG — corrected 2026-06-22 after grepping the plan indexes (the original claim
     came from a partial feature-map read without searching the indexes). ELAI-the-
     product DOES have the source-grounded copy-not-fabricate substrate:
       - SOURCE_CHECK grounding method (phase 2B, crates/reasoning) — binary pass/fail
         symbol/substring match against the ACTUAL source.
       - source_ingest = a registered PRODUCT skill in the SkillRegistry (C-plan SkillRegistry,
         distinct from paper_ingest) — not just the harness agent.
       - A-RAG extractive snippets (sentence-level +-2 lines, phase 1E.5) — copies the span.
       - CITATION PRE-BINDING (H-53): EvidenceAnchorSet frozen BEFORE generation, [N] =
         closed-set membership, out-of-set -> D-16 refusal; ghost citations structurally
         impossible. This is the product-level anti-fabrication "can only cite what was
         fetched" mechanism.
       - SOURCE QUALITY TIERS (.claude/rulebook/source-tiers.md + product SourceTierEntry /
         source_tier_distribution): the "approved websites" — a seeded deterministic
         trusted-domain list (tier-1 official/canonical; tier-4 never in plans; adding a
         tier-1 domain needs user approval). Researcher stamps every fetched URL's tier.
     The ONLY narrow residue: the specific "academic paper/SDK -> frozen interface-map
     artifact" end-to-end shape is most fully realized in the HARNESS paper-ingestor; the
     PRODUCT substrate (grounding + tiers + pre-binding + extractive snippets) exists.
     NOT a steal candidate. Do not re-propose.
  N4 Prusti REJECTED (unmaintained, last release 2024-03); Kani 0.67.0 + Creusot
     0.12.0 are the live Rust proof tools (anti-slopsquat verified 2026-06-21).
     cedar-policy 4.11.1 verified. blake3 not sha256 for the chain (anti-stale).

================================================================================
PART 4 — OPEN DECISIONS FOR THE USER
================================================================================
Q1 Which groups to author into formal plan-family files (index + ELAI-FEATURE-MAP
   upsert via paper-to-plan phase 5)? Recommend: A + E1 + B + D1 first.
Q2 Re-ingest the 13 maps into RAG once the reranker is restarted? (recommended)
Q3 Remove the premature test_cedar_contract.py, or keep it as a stub?
Q4 (VOID — N3 retracted; the product already has the source-ingest substrate. No steal.)

================================================================================
OUTCOME (2026-06-22, paper-to-plan phase 5, main-LLM in-band)
================================================================================
Authored as the NASA-GATES wave J-65..J-71 (1:1 group->file), NOT gated on shipped-status (the J-plan
is sequential; the substrate lands before the wave is reached). Files created:
  J-65 power_of_ten_generated_code_gate.md            (GROUP A; headline)
  J-66 ssb_temporal_monitor_engine.md                 (GROUP B; EXTENSION I-35)
  J-67 multimode_gate_actions.md                      (GROUP C; EXTENSION J-12)
  J-68 self_code_formal_verification.md               (GROUP D; Kani+SPARK+TLA+)
  J-69 ssb_log_integrity_hash_chain.md                (GROUP F; EXTENSION I-18+J-22)
  J-70 gate_manifest_traceability_ivv_independence.md (GROUP E)
  J-71 cedar_policy_as_code_spike.md                  (GROUP G; SPIKE)
  _grounding/J-NASA-GATES-GROUNDING-MAP.md            (the W-7 anchor; ND family + anti-slopsquat)
Updated: J-0_INDEX.md (title 64->71, Date line, ND delta family, sessions table + SSB delta rows),
  J-SUMMARY_INDEX.md (title + 7 entries + wave cross-cutting), ELAI-FEATURE-MAP.md (header upsert +
  7 section-1.6 entries + section-4 ledger rider). No commit (docs/ + workflow/ gitignored).
Verdict tally: 7 NEW/EXTENSION authored (A/B/C/D/E/F/G); GROUP F folded as a NEW file EXTENDING I-18/J-22
  (never a competing chain); 0 re-proposed ledger entries; N3 stays RETRACTED. ND doctrine family added.
