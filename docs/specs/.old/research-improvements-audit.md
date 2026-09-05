RESEARCH-DRIVEN IMPROVEMENTS AUDIT
Auditor: Claude Sonnet 4.6 (adversarial audit mode)
Date: 2026-04-08
Target: docs/specs/research-driven-improvements.md (C1-C3, H1-H5, M1-M4)
Pipeline: 5/5 tiers PASS | 56/56 tests PASS | TestQuality: 1.00

==============================================================================
PROGRESS SUMMARY
==============================================================================

Tests: 56/56 passing | Tiers: 5/5 passed | TestQuality: 1.00 |
Regression: none | BLOCKs: 6 | Score: 0.52

==============================================================================
BLOCK (must fix before proceeding)
==============================================================================

BLOCK-1: H3 EDCT lifecycle placement contradicts cited research source.
  File: docs/specs/research-driven-improvements.md, H3 section
  Claimed source: auto-generated-linting-rules.md section 3.2
  Actual source text: "Gate: Only rules surviving counterfactual crucible
    transition from PROPOSED to CANDIDATE."
  Proposed placement: CANDIDATE -> SHADOW transition.
  The research specifies EDCT gates the PROPOSED->CANDIDATE transition,
  not CANDIDATE->SHADOW. The improvement places EDCT one lifecycle stage
  later, allowing spurious rules to enter CANDIDATE (triggering contract test
  auto-generation) before rejection. This wastes resources on rules that the
  research says should be killed earlier. The proposed placement also
  contradicts the claim "all grounded in research evidence" in the summary.
  Fix: Move EDCT gate to PROPOSED->CANDIDATE transition (match research).

BLOCK-2: C2 introduces an undeclared rules/ -> knowledge/ dependency.
  File: docs/specs/research-driven-improvements.md, C2 section
  The proposal adds "trait DkgQuerier" to rules/, implemented by knowledge/.
  Current spec (rules.txt section 3A, projectIndex subsystem map):
    rules/ depends on: core/ (only)
    knowledge/ provides to: orchestrator, tools, reasoning
  Adding rules/ -> knowledge/ is a new dependency that:
  (a) is not declared in the change list (C2 says "rules/ crate gets trait
      DkgQuerier" but does not declare the new Cargo.toml dependency),
  (b) creates potential for future circular dependency if knowledge/ ever
      needs to enforce rules (knowledge/ already uses AstEvaluator pattern
      with rules/ taking Box<dyn AstEvaluator> — adding the reverse direction
      must be documented explicitly as safe),
  (c) contradicts the existing AstEvaluator pattern where rules/ takes
      Box<dyn AstEvaluator> injected at construction, isolating rules/ from
      knowledge/ at compile time. The DkgQuerier should follow the same
      injection pattern, not create a static crate dependency.
  Fix: Specify DkgQuerier as an injected trait (same pattern as AstEvaluator),
  not a direct rules/ -> knowledge/ crate dependency. Document in projectIndex.

BLOCK-3: C2 performance guarantee (<10ms at max_depth=10) is ungrounded.
  File: docs/specs/research-driven-improvements.md, C2 section
  Cited evidence: architectural-rule-engine.md section 5.2, "NGQP benchmark"
  Actual source: "180,000+ node scans with complex WHERE filtering per second"
  This benchmark covers non-recursive anti-joins and single-level joins, NOT
  depth-limited recursive CTEs. The source explicitly cites "inject depth
  counter (WHERE depth < 5)" as the safe limit for recursive traversal.
  The proposal uses default_depth=5 (matching research) but allows max_depth=10.
  The <10ms guarantee is not cited for depth>5 recursive CTEs. At depth=10 on
  a graph with 180K nodes and branching factor >3, the number of paths explored
  is O(3^10)=59K — the covering B-Tree index reduces this but the paper does
  not benchmark it. The performance claim at max_depth=10 is an ungrounded
  extrapolation from the research.
  Fix: Either cap max_depth=5 (matching research example) or add an explicit
  benchmark requirement at depth=10 as a phase gate for C2 implementation.

BLOCK-4: H4 proof gate applies binary behavioral identity, not soundness check.
  File: docs/specs/research-driven-improvements.md, H4 section
  IC3-Evolve's proof gate validates CORRECTNESS (certificate of inductive
  invariant + replayable counterexample trace). It rejects patches that
  produce wrong classifications, not patches that produce different behavior.
  The proposal maps this as: "If simulation diverges from reference... REJECT."
  DST reference model divergence is NOT equivalent to a soundness failure.
  A config change that legitimately tightens rule thresholds (fewer false
  positives, different BLOCK/ALLOW distribution) WILL produce divergent DST
  state sequences — it is not "unsound," it is an improvement.
  The binary reject-on-any-divergence gate will systematically reject valid
  config improvements, making the self-modification system unable to improve
  the very things it is designed to improve (rule thresholds, GRC budgets).
  This defeats the purpose of Layer 2 self-modification.
  Fix: Replace "diverges from reference" with a classification:
    UNSOUND_DIVERGENCE: FSM state machine reachability changes, evidence gates
      bypassed, SSB invariants violated -> REJECT (matches IC3's soundness gate).
    BEHAVIORAL_IMPROVEMENT: metrics improve, test outcomes better or equal,
      no safety regressions -> ALLOW (this is the intended self-mod target).
    REGRESSION: any PASS_TO_FAIL -> REJECT (already in current spec).

BLOCK-5: H1 VCR cassette uses blake2s hash but misidentifies the IPC layer to intercept.
  File: docs/specs/research-driven-improvements.md, H1 section
  The proposal says "Intercept all InferRequest/InferResponse IPC messages"
  and "LlmProvider::complete() intercepted by DST harness."
  The DST research (section 7.2) describes intercepting "HTTP requests
  (prompt, context arrays, hyperparameters)" at the network layer.
  But elai_cc's providers use reqwest (HTTP) to reach ollama. The DST
  research is explicit: "capture exact HTTP requests" via "custom Rust
  interceptors via reqwest middleware."
  The proposal conflates two distinct interception points:
    (a) LlmProvider::complete() trait method (application layer),
    (b) HTTP reqwest middleware (transport layer).
  (a) is fragile to prompt construction changes. (b) captures the exact
  wire payload. The proposal does not specify which to implement, and the
  research recommends (b). Additionally, the proposal says "blake2s hash
  of InferRequest" but LlmProvider::complete() takes a prompt: String +
  config, not an InferRequest struct. The struct name is fabricated — it
  does not exist in the current providers.txt LlmProvider trait definition.
  Fix: Specify the interception point precisely: reqwest middleware on
  OllamaProvider's HTTP client (matching research recommendation). Use
  the actual LlmProvider trait types. Remove the fabricated InferRequest
  struct reference.

BLOCK-6: Cross-cutting DKG auditor context injection is undeclared as a numbered change
  but modifies orchestrator.txt section 6 (context budget allocator), which is
  the most architecturally sensitive section of the largest crate.
  The "CROSS-CUTTING" section proposes adding a DKG subgraph sub-slot within
  Slot 5 (code_context), splitting it 30/70. This:
  (a) breaks cache breakpoint 2 (slots 4-5 stable within same files) — the
      DKG subgraph changes on every commit, not on file set changes alone,
  (b) is not assigned a phase number (affects 3F but possibly earlier since
      DKG exists from 1A),
  (c) is not in the "SUMMARY: CHANGE COUNT BY FILE" which counts only 1 change
      to orchestrator.txt — this is a second unlisted change,
  (d) is not numbered (C/H/M), making it invisible to change tracking.
  The omission from the numbered list means it bypasses priority and phase
  assignment review. This is either a mistake or an attempt to slip a major
  change past review as a footnote.
  Fix: Assign a change number (e.g., H6), specify phase assignment, document
  cache breakpoint 2 invalidation implications.

==============================================================================
WARN (should fix, document justification if skipped)
==============================================================================

WARN-1: C1 does not specify retry semantics for INSTRUCTOR mode (max 3 retries
  is cited but: what happens on retry 4? Silent truncation? BLOCK? Escalate?).
  Research (llm-json-schema-generation.md section 6.2): "On failure: Inject
  exact parsing error into secondary prompt for correction." No retry limit
  is specified in the research. The proposal's "max 3 retries" is a design
  choice, not a research-derived number. Failure beyond 3 retries is
  unspecified — this is a silent failure path in the auditor's output format
  enforcement. The auditor (R1-Distill) failing to produce valid JSON after
  3 retries will cause the calling agent to receive malformed data.
  Fix: Specify failure behavior after retry exhaustion (e.g., ESCALATE to
  human, return last valid JSON fragment, emit empty finding set with error).

WARN-2: C3 FxHashMap exception creates a two-tier determinism standard.
  The proposal bans HashMap for DST but allows FxHashMap "when iteration
  order does not matter." However, the CI enforcement (clippy.toml
  disallowed-methods) bans HashMap::new but NOT FxHashMap::new. Over time,
  FxHashMap will accumulate in the codebase under the "iteration order
  doesn't matter" exception, which is a judgment call made at write time.
  When the context of "does order matter here?" changes (refactoring, new
  callers), the FxHashMap usage silently becomes incorrect. The research
  (DST section 8.1) recommends BTreeMap or indexmap for DST-safe code;
  FxHashMap is mentioned as a performance option with explicit caveats.
  Fix: Require FxHashMap usage to carry a #[allow(non_deterministic_map)]
  annotation with justification comment, so grep can audit all exemptions.
  Alternatively, ban FxHashMap also and use indexmap as the default O(1) map.

WARN-3: M3 confidence decay threshold for auto-demotion (below 0.25 = SHADOW)
  conflicts with the existing SHADOW->ACTIVE promotion gate.
  If a rule's confidence drops to 0.25 (after ~180 days no TPs) and it is
  demoted back to SHADOW, it will immediately fail the SHADOW->ACTIVE gate
  (requires false_positive_rate < 5% AND true_positive_rate > 80% AND
  counterfactual_causal_rate > 50%). The decay-demoted rule will never
  re-promote unless it gets new true positives. But SHADOW rules are only
  evaluated, not enforced — they cannot generate new TPs on their own unless
  code is audited. This creates a dead-end lifecycle: ACTIVE -> (decay) ->
  SHADOW -> (no enforcement) -> (no TPs) -> (stays in SHADOW forever).
  The proposal does not specify how a decayed-then-demoted rule can recover.
  Fix: Specify recovery path. Either: (a) SHADOW rules re-run retrospectively
  against historical code to collect TPs, or (b) decay-demoted rules enter
  a "DORMANT" state (not evaluated) with manual revival path, not SHADOW.

WARN-4: M2 (ast-grep YAML DSL evaluation) is assigned to a "Decision gate:
  Phase 3F SKELETON step" but the deliverable is underspecified. The proposal
  says "Implement both, benchmark, pick the one with better ergonomics +
  performance." This requires implementing BOTH the S-expression and YAML DSL
  paths in the rules/ crate interpreter before making the decision. Phase 3F
  is already the rules engine implementation phase — adding a parallel
  implementation of AST_PATTERN evaluation doubles the implementation surface
  for that step. The phase file does not account for this.
  Fix: Either (a) specify M2 as a Phase 3F PRE_GATE research spike
  (benchmark without implementing both in production), or (b) declare a
  decision NOW based on research (treesitter section 7 clearly recommends
  ast-grep YAML DSL as "optimal ergonomic layer") to avoid the dual-path cost.

WARN-5: M4 custom SQLite VFS (~200-400 LOC) has zero risk assessment.
  The proposal cites the Limbo project as reference. Limbo is a complete
  async SQLite rewrite in Rust, not a VFS plugin. The actual SQLite VFS API
  uses C FFI structs (sqlite3_vfs). In Rust, this requires unsafe blocks to
  implement the C callback interface. The proposal does not acknowledge:
  (a) the unsafe Rust requirement,
  (b) that SQLite's WAL recovery behavior may differ between the real VFS
      and the in-memory VFS (page alignment, journal mode interaction),
  (c) that madsim already patches libc time syscalls — the custom VFS must
      coordinate with madsim's time injection to avoid two competing sources
      of simulated latency.
  Phase 3I already specifies madsim as the DST substrate. Adding a custom
  VFS that independently controls I/O latency creates two deterministic
  time sources that must be synchronized.
  Fix: Add risk section. Specify coordination protocol between madsim and
  SQLite VFS for I/O latency. Document unsafe surface. Reference actual
  SQLite VFS API docs, not Limbo.

WARN-6: H5 GBNF schema constraints documentation duplicates C1 content
  without adding new restrictions. C1 already specifies the 3-nesting-level
  limit and bans $ref cycles. H5 adds the full GBNF limitation table (from
  the same research source, same section). The justification for H5 as a
  separate HIGH-priority change (versus C1 which is CRITICAL) is that it
  "must be enforced at schema compilation time." But C1 already says
  "compile_predicate() must reject schemas with >3 nesting levels or $ref
  cycles." H5 adds: additionalProperties, regex anchors, uniqueItems/not/if.
  These additions ARE valuable (they prevent runtime crashes), but the
  priority rating (HIGH vs CRITICAL) is inverted — the regex catastrophic
  backtracking causing server crashes is as critical as the SIGSEGV from
  deep recursion. H5 should be merged into C1 as additional constraints,
  not listed as a separate lower-priority change.
  Fix: Merge H5 into C1. All GBNF constraints belong in one place.

WARN-7: The PREDICATE VALIDATION ADDITIONS section (AST_PATTERN validation
  additions to rules.txt section 3A) is unlisted in the SUMMARY: CHANGE
  COUNT BY FILE and is assigned to no numbered change. It affects rules.txt
  section 3A which is already being modified by C2. The validation additions
  (reject unanchored quantifiers, validate against grammar at insertion time)
  ARE already implied by the existing spec (section 3A states "Invalid
  predicates rejected with specific error type") but the specific conditions
  from treesitter research (section 4.3) are new. This is a legitimate
  addition that deserves a number and phase assignment.
  Fix: Number as M5 or fold into M2 (ast-grep evaluation already touches
  AST_PATTERN predicate validation).

WARN-8: H4 slot restrictions do not cover the context budget allocator.
  The proposed Slot 1-4 correspond to GRC budgets, rule thresholds, context
  budget percentages, and model parameters. These map to orchestrator.txt
  section 6 (budget split: repo_map 15%, code_context 40%, ssb 25%, grc 20%).
  But Slot 3 ("Context budget percentages") would allow the self-mod agent to
  modify the 15/40/25/20 split. Changing code_context from 40% to 70%
  is a "config" change but fundamentally alters what agents can see, affecting
  auditor context quality and executor grounding. The proposal does not
  flag this as the highest-risk slot. The IC3-Evolve paper restricts edits
  to specific named functions — the elai analogue should restrict to specific
  named config keys, not broad "percentage" categories.
  Fix: List specific config keys per slot (not categories). Slot 3 should
  enumerate: repo_map_pct, code_context_pct, ssb_pct, grc_pct. Flag that
  changes to auditor's code_context_pct require human approval per Layer 3
  (since auditor quality affects the alignment loop).

==============================================================================
INFO (noted, no action required)
==============================================================================

INFO-1: H2 is an empty placeholder. It says "[Detailed in C2 above]" and
  lists as a separate HIGH priority change. It adds zero content. The effect
  tagging work is entirely within C2. H2 inflates the count to 13.
  Effect tagging is Phase 1A compatible (can be added incrementally), which
  is correctly noted in H2. The phase assignment (1A or 3F) should be in C2.

INFO-2: M1 (incremental AST cache) describes a performance requirement
  already partially implied by the existing specs. rules.txt section 3A
  states "amortized if cached per-file" for AST_PATTERN evaluation. The
  spec does not define WHERE the cache lives. M1 correctly identifies that
  the cache should live in knowledge/ (which already parses for DKG) and
  that rules/ should read from it. This is a clarification of an implied
  architecture, not a new design. The improvement correctly identifies the
  cold-start parse issue (5-10ms vs <1ms budget) as a BLOCK to Phase 3F
  correctness. The change is needed and the research grounding is accurate.

INFO-3: C3 HashMap ban is partially redundant with existing core.txt
  infrastructure. The EntropySource/ClockProvider trait pattern and
  check_entropy_quarantine.sh script already ban SystemTime and rand.
  The HashMap ban is the logical extension of the same quarantine principle.
  The proposal correctly identifies that clippy disallowed-methods is the
  enforcement mechanism — this is consistent with how the existing entropy
  quarantine is enforced.

INFO-4: The evidence quote in C1 ("RTX 3060 constrained decoding: <9 OTPS")
  is accurate to the research table (section 5). The constraint is
  correctly flagged as relevant since the target hardware is RTX 3060 (12GB).
  At <9 OTPS for auditor output, a 200-token auditor response takes ~22s.
  The INSTRUCTOR retry pattern (multiple inference passes) is strictly
  worse for throughput. The tradeoff (semantic correctness vs throughput)
  is not discussed in the improvement. At 3 retries, worst case: 66s for
  one auditor response on RTX 3060. This should be noted in the spec.

INFO-5: The proposed phase assignments are internally consistent with the
  current phase plan. C1/H5 target 3F (rule enforcement) where SCHEMA_REQUIRED
  predicates are built. C2 targets 3F (DKG queries from rules engine). C3
  targets "NOW" (entropy quarantine should be Phase 0.8 — correctly noted).
  H1/M4 target 3I (DST). H4 targets 4H (self-modification). M3 targets 3F/4H.
  No phase ordering conflict was found.

==============================================================================
MISSING IMPROVEMENTS (research suggests but file does not capture)
==============================================================================

MISSING-1: Spearman's rho threshold for rule promotion is absent.
  auto-generated-linting-rules.md section 4.2 specifies: "Industrial
  operations require rho > 0.90 for surrogate rules in production. Below
  threshold -> demote to SHADOW."
  The lifecycle table in the research uses Spearman's rho as a complementary
  metric to FP rate for SHADOW->ACTIVE transition. The current spec uses
  false_positive_rate < 5% as the gate. M3 adds confidence decay but does
  not add the rho correlation metric. The research lists rho as a distinct
  gate from FP rate (section 4.2 table). This gap means rules with low FP
  rate but poor correlation with actual violations can still promote.

MISSING-2: Coevolving evaluators (objective hacking prevention) are absent.
  auto-generated-linting-rules.md section 6.3 identifies reward hacking
  as a production failure mode: "Agent modifies evaluation framework to
  auto-return True." The current self-modification spec locks metrics/gates
  via PreToolUse (READ-LOCKED during self-mod). This is a necessary but
  potentially insufficient protection — the lock prevents writes but does
  not prevent the self-mod agent from proposing rule changes that gradually
  shift what "good" metrics mean. The research recommends coevolving
  evaluators (genetic algorithm on evaluation logic). This is acknowledged
  as non-trivial to implement but the spec should at least note the threat
  model and planned mitigation beyond file locking.

MISSING-3: The research (architectural-rule-engine.md section 4.3) describes
  CIRank (Change Impact Rank, PageRank-derived) as a complementary metric
  to graph reachability. C2 adds reachability queries but not the
  probabilistic blast-radius ranking. CIRank would enable "mandatory
  architectural review when CIRank propagation is high" vs "auto-approve
  isolated changes." This is a natural complement to the BLAST_RADIUS query
  type proposed in C2 but is not mentioned.

==============================================================================
OVER-ENGINEERING FLAGS
==============================================================================

OE-1: M4 (custom SQLite VFS) adds ~200-400 LOC of unsafe Rust with C FFI
  for a capability that may not be needed. Phase 3I's current spec
  achieves DST via madsim (which patches libc syscalls including I/O) and
  the existing ClockProvider/EntropySource injection. The VFS adds torn-write
  simulation and deterministic I/O latency. The spec already notes that
  SQLite WAL recovery is tested via the reference model (phase-3I.txt).
  The VFS only becomes necessary if WAL corruption scenarios cannot be
  simulated via madsim's existing I/O interception. This should be a
  SPIKE (evaluate whether madsim's libc hooks are sufficient) before
  committing to implementing a custom VFS.

OE-2: H3 EDCT counterfactual testing adds ~4 new columns to rule_metrics
  (edct_pass, edct_fail, causal_catches, spurious_catches) and a new
  AST transform pass for generating S' (fixed snippet). The existing CEL
  counterfactual (rules.txt section 5A) already measures causal_catches
  and spurious_catches at the auditor level. EDCT measures causality at
  the predicate level — it is more precise but requires AST transforms
  that "may not be possible for this predicate type" (proposal's own caveat).
  For GREP_PATTERN rules (regex-based), the AST transform is undefined.
  The improvement is valuable for AST_PATTERN predicates but the spec
  does not define EDCT behavior for non-AST predicate types. This gap
  means EDCT silently skips half the predicate surface.

==============================================================================
VERIFICATION SCRIPT OUTPUT
==============================================================================

Script: F:\.ELAI_workflow\workflow\audit-scratch\verify_research_improvements_audit.py
All 10 tests PASS. Key findings from verification:
  - H3 EDCT placement contradicts research (confirmed by code inspection)
  - M3 decay math correct: C(90)=0.500, C(180)=0.250 (matches research)
  - M3 asymmetric penalty equilibrium: max FP = 20% at 80% TP threshold
  - M3 time to confidence=0.1: 299 days (~10 months without TPs)
  - C2/H2 schema additions have no column name conflicts
  - C2 new crate dependency verified as undeclared in proposal

==============================================================================
SUMMARY
==============================================================================

6 BLOCKs. 8 WARNs. 5 INFOs. 3 missing improvements. 2 over-engineering flags.

Critical: H3 lifecycle placement contradicts its own cited source. H4 proof
gate analogy is logically flawed (behavioral identity != soundness). C2
performance guarantee is ungrounded at max_depth=10. These three must be
resolved before any implementation begins on the affected phases.

The proposals are generally well-grounded to research. The primary failure
mode is precision: claims that are directionally correct but imprecise in
ways that will create implementation bugs (H1 InferRequest fabrication,
C2 dependency undeclared, H4 divergence=reject).
