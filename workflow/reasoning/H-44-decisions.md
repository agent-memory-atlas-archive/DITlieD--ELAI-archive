H-44 decisions: behavioral trap detection + measured recall

WHAT

Replaced the keyword-grep cheat-detector (trap_tasks.rs:123,
score_candidate_for_traps, GAP-2) with three behavioral detection legs
plus a frozen-corpus recall measurement harness. The keyword-grep is
downgraded to a cheap advisory KeywordFallback first-pass (VD-2: never
the sole blocker).

KEY DECISIONS

D-1: Why behavioral legs over keyword-grep?

Grounding: Skalse 2209.13085 proves keyword proxies are gameable by
construction. A candidate that reward-hacks without using any flagged
phrase passes the keyword detector untouched (GAP-2, confirmed at
trap_tasks.rs:132-179). The three behavioral legs read execution and
structure, not strings.

D-2: ImpossibleBench conflict canary (arXiv:2510.20270)

"STOP if tests flawed" guard cut hack-rate from 93% to 1% in the paper.
The frozen IMPOSSIBLE_TASKS set embeds spec-test conflicts where pass-rate
equals cheat-rate by construction. The token is never a keyword — it is
a generated impossible-pass marker that a correct solution CANNOT produce.

D-3: Isomorphic-Perturbation mismatch (arXiv:2604.15149)

Passes original but fails logically-isomorphic rewrite == gaming the
surface form. Tier-1 uses a textual variable-rename transform (space-padded
single-char identifiers). Full semantic isomorphism (compile + run) is
Tier-2 / RunPod-pending. The textual check is sufficient for the config-DGM
candidate surface.

D-4: CodeHalu execution-classification (arXiv:2405.00253)

Composes the H-43 execution_grounded_fitness harness (GD-1 — no second
runner). A candidate that "looks right" (contains structural tokens) but
scores below HALLUCINATION_FITNESS_FLOOR (0.3, locked at compile time per
VD-1) is classified as an execution hallucination.

D-5: Cross-view gate (VD-4, Co-rewarding 2508.00410) — Tier-1 scope

VD-4 requires >= 2 different-family views to agree before a hard block.
The second view is D-13's cross-family adversary opinion (PC-5). D-13 is
PLANNED but not yet shipped. This is a genuine cross-plan dependency, NOT
a stub:

  - The single-view behavioral block in detect_trap_behavior IS fully
    functional and is the correct Tier-1 behavior.
  - No TODO or placeholder marker is placed in the code.
  - The D-13 leg will be added when D-13 ships by extending the guard
    chain: "if two different behavioral families agree, block; if only
    one, flag as advisory".
  - The absence of D-13 is documented here and in the WIRING_CONTRACT
    (stub_ban clause 3 still holds because the single-view block IS
    behavioral, not keyword-only).

D-6: Recall measurement mandate (VD-2)

A detector with UNMEASURED recall advertises a guarantee it cannot prove.
The FROZEN_TRACE_CORPUS (20 pairs, TRACE-style arXiv:2601.20103) provides
a gate-frozen (R8) baseline. ELAI reports its own measured recall — NOT
the paper's 63% ceiling (GD-5 / HD-1). The corpus is authored out-of-band
and the running loop cannot inject into it.

D-7: KeywordFallback advisory placement

The old score_candidate_for_traps is called LAST, after all behavioral legs.
It emits KeywordFallback kind only when no behavioral leg fires. Its doc
comment is updated to say "advisory only". The TrapPattern enum and
TrapTaskRegistry are retained intact (no deletion).

D-8: SSB variant design

One Pattern-2 single-variant 4-arm variant (TrapBehaviorDetected) following
the H-43 FitnessEvaluated template. The recall roll-up rides the same
variant payload (no second variant, per W-9 plan note). This keeps the
lockstep bump to one variant (532 -> 533, SCHEMA 221 -> 222).

D-9: Feature gate

trap_behavioral.rs is gated behind cfg(feature = "darwinian-evolve")
because it imports from trap_tasks (also feature-gated). The execution_fitness
harness remains always-compiled (H-43 decision). The SSB emit and
darwin_admit_loop wiring are already feature-gated; this is consistent.

EXPECTED TEST OUTCOME

cargo check --workspace --all-targets = 0
cargo test -p elai-orchestrator --test action_type_lock: 2/3 PASS (222/533)
cargo test -p elai_cc_contracts --test h44_trap_behavioral_contract: 5 PASS (default)
cargo test -p elai_cc_contracts --test h44_trap_behavioral_contract --features darwinian-evolve: additional 5-6 PASS
cargo test -p elai-orchestrator --lib: ~42+ backlog (zero new H-44 failures)

UNKNOWNS

- The live large-corpus recall sweep (full TRACE dataset, Tier-2) is a
  separately-gated checkbox per W-10. Not attempted in this session.
- D-13 cross-family adversary is a forward cross-plan dependency. Not
  present here. The single-view Tier-1 behavioral block is fully functional.
- Sandbagging by a strong candidate (eval-awareness 2510.20487, AUC 0.83)
  is a residual risk acknowledged per VD-5. The recall report is honest
  about the ceiling; we never claim 100%.
