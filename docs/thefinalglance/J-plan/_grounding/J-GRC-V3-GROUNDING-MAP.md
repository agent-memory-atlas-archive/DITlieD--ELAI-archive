# J-GRC-V3 GROUNDING MAP — the reasoning-engine wave (J-83..J-86)
Authored 2026-06-26, main-LLM in-band, BEFORE the J-83..J-86 detail files (template R-a / section 4).
Source design doc: workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md (the spec-eval AS-SPECIFIED
reasoning run: ROUND-1 idea-to-map, ROUND-2 nine papers one-block-each, ROUND-3 the user's deep-research
report, ROUND-4 live elai.map/source grep, FINAL SYNTHESIS). Live SCHEMA re-derived this session.
The K-native half of the same run is K-15/K-16 (PROVE bench + DISCIPLINE doctrine); this map grounds the
technical half (J) only. Both halves cite the same anchors below.

CENTRAL GROUNDED FINDING (the reasoning run's headline, anti-sycophancy): GRC-v3 is NOT a new mechanism. It
is the UNIFICATION + ACTIVATION of seven pieces ELAI already has scattered across plan letters, plus ONE
genuinely-new deterministic component (the overthinking guard). The single highest-leverage move is to name
one coherent reasoning-state carrier (the typed BeliefState) and make the pieces compose, not to invent a
reasoning engine. The carrier is uniform; the win is domain-routed VALIDATORS (one carrier, one router, many
validators). The decisive correction to the user's framing: a model cannot reliably "ground itself" or judge
which of its own parts is "sure" (the self-correction blind spot + faithfulness results, three independent
2024-2026 papers) — so "the part the model is more sure about proceeds" MUST mean "the part an EXTERNAL
RECEIPT confirms proceeds". That single change is what makes the whole thing trustworthy, and it is exactly
ELAI's existing trust spine (F-4 cant-lie, D-13 cross-family, I-4 select-by-receipts, IC-6).

================================================================================
GS — SUBSTRATE-SHIPPED CORRECTIONS (the reasoning doc / older index labels are STALE; authors use THESE)
================================================================================
GS-1  I-11 PABU TYPED BELIEFSTATE — PARTIAL, not PLANNED. crates/orchestrator/src/belief_state.rs EXISTS;
      BeliefState::new + BeliefState::emit_condensed_receipt BUILT; wired at escalation_waterfall.rs:239
      (BeliefStateEventKind::Condensed); i11_belief_state_contract.rs present. The condense-on-escalate seam
      is LIVE; the typed-claim+receipt+abstention CARRIER unification (J-83) is what is absent (GA-1).
GS-2  H-50 REGIME-AWARE CONTEXT POLICY — WIRED. context_policy.rs is wired into live_cli.rs:50; the local
      lane gets its own window-minimization policy. The reasoning doc's "PARTIAL/uncommitted" is stale
      (code-grounded 2026-06-25, feature-map 1.5). The carrier's local-unload / cloud-cache-prefix legs ride
      this — already built, J-83 consumes it, does NOT re-author it (GD-1).
GS-3  I-5 PRISTINE CONTEXT INJECTOR — PARTIAL. pristine_path_filter.rs BUILT;
      i5_pristine_context_contract.rs (pristine_rebuild_zero_traceback_bytes proven). The "PLANNED" label is
      stale. Relevant because J-83's commit-and-freeze + the K-12 must-preserve atoms ride this.
GS-4  H-39 ABSTENTION-AWARE SCORING — WIRED (PARTIAL completeness). crates/orchestrator/src/abstention_score.rs
      parse_eval_log_outcome reads correct/wrong/abstain FROM THE EVAL LOG (a receipt, not self-report) +
      AbstentionScored SSB (218). This IS the blind-spot-safe confidence bit (the abstention/confidence bit
      each BeliefState entry carries is READ from here per RD-1, never re-derived).
GS-5  I-7 CASCADING ESCALATION — WIRED. i7_escalation_cascade_contract.rs, test fail_condenses_then_escalates;
      EscalationCascadeKind {LocalWin, Escalated, BatchRouted}; condense-on-escalate seam exists. J-83 threads
      the carrier through this; the seam is the named PRODUCTION_HOOK (escalation_waterfall.rs:239).
GS-6  I-4 EVIDENCE-RANKED SEARCH LADDER — DARK(ELAI_SEARCH_LADDER_ENABLE). search/ladder.rs (run_best_of_n at
      ladder.rs:237/277, also structural_retry.rs); SearchNodeEvent SSB; i4_search_ladder_contract.rs
      (branch_is_pruned_never_promoted). The substrate (prune/promote, best-of-N) is real; PUCT consumer +
      receipt-absent vote-fallback rung are the J-86 activation legs.
GS-7  H-42 SERIAL-DEPTH BUDGET — PARTIAL. h42_serial_depth_budget_contract.rs + ReasoningBudgetScaled SSB
      (220); depth-not-width guard. J-84 wires the 2-axis classifier into this + I-26.
GS-8  D-10 MODE-ROUTING PREDICATE — WIRED + LOCKED. select_grounding_mode (modes A-E), deterministic, LLM has
      no input; locked_keys_registry.toml. The TYPE axis of the J-84 classifier IS this router (reuse, not
      rebuild). I-26 DYNAMIC THINKING BUDGETS DARK(ELAI_DYNAMIC_BUDGET_ENABLE) reuses H-42 + effort field.
GS-9  D-2 PPM PROCESS REWARD SIDECAR DARK(ELAI_PPM_ENABLE) + I-24 ISOMORPHIC CANARY DARK + I-22 PROGRESS-SHAPED
      REWARDS DARK — the Tier-2 PRM step-scorer (under the Tier-1 receipt) + its anti-gaming canary. J-86
      activates D-2 as Tier-2-UNDER-receipt, guarded by I-24 (search over-optimizes past ~400 samples).
GS-10 J-34 EMPIRICAL CONTEXT CALIBRATION PLANNED — measures per-(model,lane) FILL->degradation curves; the
      reasoning-BUDGET knee (K-15 `elai bench reasoning-budget`) is its SIBLING bench (same harness shape,
      different x-axis: reason-budget not context-fill). I-27 BUDGETED CONVERGENCE INSTRUMENTATION
      DARK (ConvergenceCurveMeasured SSB; bench_convergence) = the verified-outcome-vs-compute harness K-15
      composes. H-49 SESSION DAG + REVERSIBLE EDITS (PARTIAL) = the backtrack substrate J-86 reuses (no new
      tree-search engine).

================================================================================
GA — ABSENT / NET-NEW (what the J wave BUILDS; confirmed not in code by the reasoning-doc ROUND-4 grep)
================================================================================
GA-1  the BeliefState CARRIER unification (typed entries {claim, grounding-receipt, abstention/confidence bit
      from GS-4, DKG provenance}; the per-step receipt gate = least-to-most-with-receipts; carried across
      GS-5 escalation + GS-2 local unload; commit-and-freeze) — NOT built; belief_state.rs is the
      condense-on-escalate seam only. J-83 [KEYSTONE].
GA-2  the 2-axis reasoning-effort classifier (TYPE axis = GS-8 D-10 router; COST/difficulty axis = a proxy
      from H-39 confidence + C-45/C-47 case-base hit + J-38 retrieval coverage + C-29 IntentPair count) →
      route zero/depth/width/abstain over GS-7 H-42 + I-26 — NOT in code. J-84.
GA-3  the deterministic OVERTHINKING GUARD (per-call ceiling + 3-signal early-stop: answer oscillation r=0.78
      lead / hesitation markers "wait/but/actually" r=0.71 / declining-confidence r=0.63; flip_ratio>1.0 =
      stop) — NOT present. grep "oscillation" hit only anti_spiral_extended.rs + telemetry, a DIFFERENT
      mechanism (anti-spiral catches REPETITION; overthinking catches the chain DRIFTING OFF a correct
      answer). The one genuinely-new component of the whole design. J-85.
GA-4  I-14 DETERMINISTIC CONTEXT COMPRESSION (LSP slicing + tree-sitter folding; ContextTransformReceipt;
      ciphers BANNED IC-3) — PLANNED, not found in code. The reasoning run adds only a RULE to it
      (preserve-receipts losslessly / fold-prose lossily, the ReAct compression rule) — recorded as a RIDER
      on the existing I-14 unit, NOT re-authored (GD-1, like the J-11/J-15 amendment riders).
GA-5  `elai bench reasoning-budget` — NOT present (J-34 `elai bench context-fill` is the fill-axis sibling).
      This is the K-15 PROVE leg (the GRC-v3 done-gate), composes I-27's bench_convergence harness shape.

================================================================================
GW — WEB / PAPER VERIFICATION ([EXTERNAL] — grounding for WHY, never an ELAI number; HD-1/ID-5)
================================================================================
All nine ROUND-2 papers + the deep-research report were read one-block-each in the reasoning doc; arXiv IDs
carried as cited there (CoT 2201.11903, zero-shot-CoT 2205.11916, self-consistency 2203.11171,
least-to-most 2205.10625, ToT 2305.10601, ReAct 2210.03629, Reflexion 2303.11366, test-time-compute
2408.03314, overthinking 2604.10739). Load-bearing 2026 corroborations cited in-doc: "Consensus is Not
Verification" (2603.06612, receipts > majority-vote), CGES early-stop (2511.02603), faithfulness (Lanham),
RULER (2404.06654, usable-not-available context). VERDICT pattern: the 2022 prompting papers are
PARTIALLY-STALE (their elicitation tricks are absorbed into instruction-tuned models; GRC-v3 does NOT
re-teach reasoning); the agentic/search papers converge FOUR independent times on receipts-not-self-eval;
the two scaling papers give the quantitative escalate/ceiling policy. The done-gate is ELAI-MEASURED, NEVER a
paper constant (ID-5; the overthinking paper 2604.10739 ITSELF forbids transferring its 6K/7K knee to a
smaller model — Qwen2.5-Coder-7B overthinks EARLIER). Every arXiv id is re-verified at detail-file authoring
(house rule; 2 bad cites in the B-40/B-42 era).

================================================================================
PER-SESSION SYMBOL TABLE (the file:line anchors each detail file uses; re-grep at edit, W-7)
================================================================================
J-83 (carrier): belief_state.rs (BeliefState::new / emit_condensed_receipt), escalation_waterfall.rs:239
  (BeliefStateEventKind::Condensed — the PRODUCTION_HOOK), abstention_score.rs (the abstention bit, GS-4),
  context_policy.rs + live_cli.rs:50 (GS-2 local unload), select_grounding_mode (GS-8 domain router), the
  reasoning crate GrcState (crates/reasoning/src/config.rs). SSB: prefer-reuse BeliefStateEvent (+Kind).
J-84 (classifier): grounding_mode.rs:157 select_grounding_mode (TYPE axis), abstention_score.rs (COST proxy
  input), C-29 IntentPair + J-38 retrieval coverage + C-45/C-47 case-base, h42_serial_depth_budget +
  ReasoningBudgetScaled (220), I-26 BudgetController/effort field, I-27 ConvergenceCurveMeasured + H-9 cost
  cockpit (the lambda/effort toggle). SSB: prefer-reuse ReasoningBudgetScaled (+Kind) else +ReasoningEffortRouted.
J-85 (overthinking guard): the GRC Verify step (crates/reasoning, phase-2A/2D), anti_spiral_extended.rs
  (the DISTINCT repetition detector it sits beside), I-26 budget ceiling; done-gate measured by K-15
  `elai bench reasoning-budget`. SSB: +OverthinkingGuardFired (Pattern-2) else reuse ReasoningBudgetScaled.
J-86 (receipt-gated ladder activation): search/ladder.rs:237/277 run_best_of_n + SearchNodeEvent (I-4),
  D-2 PPM sidecar (port 8101) + I-24 CanaryInjector + I-22 ProgressRewardScored, SessionDag branch/merge +
  H-49 backtrack, I-7 escalation. SSB: reuse SearchNodeEvent / ProgressRewardScored / PrmCanaryEvent (net 0).

================================================================================
CANONICAL CONSTANTS (re-derive at edit — W-9; both are DERIVED, never hand-counted)
================================================================================
SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH(-309) + SSB_ACTION_TYPES.len()  (ssb.rs:1503/1508).
TOTAL_VARIANTS = ALL_VARIANTS.len() = SSB_ACTION_TYPES.len()              (action_type_lock.rs:775/783).
=> TOTAL_VARIANTS = SCHEMA_VERSION + 309 always. LIVE at this authoring (grep 2026-06-26, last bump comment
"576 -> 577" = TokenAcquired, action_type_lock.rs:1494): SCHEMA_VERSION 268 / TOTAL_VARIANTS 577. The
236/545, 240/517, 268/577, 234/543 numbers in older indexes are all stale snapshots of this same derived
pair. Bump SEQUENTIALLY, one session at a time, 9-tuple golden lockstep (a partial add breaks the workspace,
the 91-error incident). The J wave is prefer-reuse-first: net 0..+1 SSB per session (J-83 reuse
BeliefStateEvent; J-86 reuse SearchNodeEvent net 0; J-84/J-85 at most +1 each, decide at edit).

================================================================================
DESIGN-DOC CORRECTIONS (reality flipped vs the reasoning doc's ROUND-1 labels — authors use these)
================================================================================
DC-1  ROUND-1 called I-11 the unbuilt keystone "PLANNED only". Live: PARTIAL (GS-1) — the condense seam is
      built; J-83 is a COMPLETION/EXTENSION, not a from-scratch build. Lowers J-83 risk.
DC-2  ROUND-1 called H-50 "PARTIAL/uncommitted" and the local-caching leg a NO-OP. Live: H-50 WIRED (GS-2);
      the local-KV-bypass (sidecar PrefixKVCache server.py:1819 bypassed on the streaming path
      runtime_client.rs:547) is still real, so the LOCAL lane's lever is unload/reload (H-50), NOT KV reuse.
DC-3  the "candidate system" must NOT be self-consistency majority-vote or MCTS-first (IC-6; the Section-4
      ledger). The selector is a LADDER (best-of-N → tournament → PUCT) where the SELECTOR is RECEIPTS. The
      receipt-absent VOTE rung (J-86) is the ONLY place statistical consensus is legal, and never for
      factual-recall (which routes to retrieval, not a chain — it confabulates).
DC-4  "lossless compression of the entire reasoning" = DETERMINISTIC + REVERSIBLE (I-14 / IC-3 ciphers
      BANNED), with the ReAct rule: preserve RECEIPTS/observations losslessly, fold reasoning PROSE lossily
      (rederivable). On cloud = deterministic masking of trailing observations behind the cache breakpoint
      (HD-2); NEVER an LLM re-summarization (busts the cache, unverifiable). Recorded as the I-14 rider (GA-4).
DC-5  depth-vs-width vs RD-4 (the honest tension T1): "depth-not-width" is right that WINDOW is never the
      lever (RD-4, RULER), but test-time-compute (2408.03314) shows HARD problems want parallel WIDTH not
      more revision-depth. Reconciliation (J-84): window→never; easy/medium→serial DEPTH (revise the
      beliefstate); hard→parallel WIDTH (I-4 best-of-N / BSP). State this explicitly, do not hide it.

================================================================================
HONEST RESIDUE (GD-5 — named near the claim in every detail file)
================================================================================
R-1  this is a UNIFICATION. The risk is not "will a piece work" (each is grounded) but "will the SEAMS
     between I-11/I-4/I-7/H-42/D-10 compose" — exactly the wiring-enforcement R10 integration-seam risk. Each
     J session ships its producer + a consumer + an integration smoke together; J-83 ships the carrier AND
     its I-7 consumer AND a multi-turn handoff smoke, never the carrier alone.
R-2  the classifier's difficulty proxy is itself imperfect (the papers warn a true pass@K oracle is
     expensive). It is OBSERVE-ONLY first (GD-4) before it gates anything, and MEASURED for calibration
     (K-15), never assumed.
R-3  the overthinking flip-ratio is a LAGGING indicator; the lead signal is rising oscillation — the guard
     acts on the lead and accepts some false-positive early stops (76.3% precision in the paper), tuned on
     ELAI's own data (K-15), never on the paper's threshold (the paper forbids transferring it).
R-4  the local Qwen2.5-Coder-7B lane overthinks EARLIEST and at the LOWEST budget studied, and its self-eval
     is noise — the local lane is the hardest case and the place "engine imposes structure + supplies
     receipts, model only generates" is mandatory not optional (the small model NEEDS the steps, Cobbe, but
     cannot be trusted to self-evaluate them).

End of J-GRC-V3 GROUNDING MAP.
