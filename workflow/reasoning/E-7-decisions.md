E-7 Darwinian self-improvement decisions
session: s-25120 (completion-drive 4/4)
plan: docs/thefinalglance/E-plan/E-7_darwinian_self_improvement.md
status: scope-lean ship (substrate-only; live model evolution loops RUNPOD-PENDING)
schema bump: 153 -> 154 (atomic +1 single-variant per Pattern-2)
total_variants bump: 426 -> 427
fsm substate added: DarwinianGenerationEvaluating
env gate: ELAI_DARWINIAN_LOOP_ENABLE (default OFF; wire-dark)
feature gate: cfg(feature = "darwinian-evolve") on new modules

DECISION-1 paper re-anchor (correction over plan body)
   arXiv:2510.04399 is NOT AlphaEvolve and NOT DSPy-related.
   ACTUAL: "On The Statistical Limits of Self-Improving Agents"
   (Wang, Dorchen, Jin; Columbia; submitted 2025-10-05, revised 2026-02-01).
   PAC-theoretic foundation: distribution-free learnability is preserved
   under self-modification IFF the policy-reachable family has uniformly
   bounded VC dimension. Two-Gate guardrail = validation gate (margin tau)
   AND capacity gate (K[m]).
   Five modification axes (Theorem 11/12): Algorithmic, Representational,
   Architectural, Substrate, Metacognitive -- all reduce to the same
   capacity-supremum boundary.
   Interface map: workflow/papers/statistical-limits-self-improving-interface.md.
   USAGE in E-7: promotion_gate::two_gate_promote signature anchors tau + K_m
   against this paper. K_m default = 50 ships as config-knob (paper does not
   prescribe value). Local file: F:\.ELAI_workflow\.claude\papers\2510.04399.html.

DECISION-2 darwinian linkage for arXiv:2305.10718
   Discounted Thompson Sampling for Non-Stationary Bandit Problems
   (Qi, Wang, Zhu; arxiv 2023-05-18, revised 2023-05-22; cs.LG).
   Not an evolutionary paper -- bandit algorithm for non-stationary rewards.
   DARWINIAN LINKAGE: in MAP-Elites + island-model substrates, the per-cell
   arm reward distribution shifts as the population evolves; this is exactly
   the non-stationary bandit regime DS-TS targets. Discount factor gamma
   passively tracks the shift; this is the substrate fit for darwinian
   parent-selection inside a MAP-Elites cell.
   Regret bounds: O~(sqrt(T*B_T)) abrupt changes, O~(T^beta) smooth changes.
   USAGE in E-7: bai::DiscountedTs default discount = exp(-1/0.3) ~ 0.0357
   ships as config-knob (paper warns gamma must be tuned per regime; the
   exp(-1/0.3) derivation from MSI lambda=0.3 is design-hypothesis, not
   paper-quoted). Local file: F:\.ELAI_workflow\.claude\papers\2305.10718.pdf.

DECISION-3 REMOVE arXiv:2602.12413 from E-7 citations
   Verified content: "Soft Contamination Means Benchmarks Test Shallow
   Generalization" (Spiesberger et al., 2026-02-12, cs.LG). Benchmark
   integrity paper -- finds 78% CodeForces semantic duplicates in Olmo3
   corpus. ZERO relevance to darwinian self-improvement.
   ACTION: removed from in-code citations. The contam-filter design in
   E-7 task 6 keeps its paraphrase + cosine + canary GUID approach but
   does NOT cite 2602.12413 as the design basis (LiveCodeBench
   arXiv:2403.07974 remains the load-bearing cite for temporal cutoff;
   n-gram method is Shi/Riddell prior-work).

DECISION-4 primary darwinian-coding-agent anchor = arXiv:2506.13131
   AlphaEvolve (Google DeepMind 2025) -- real arxiv paper, NOT 2510.04399.
   Interface map: workflow/papers/alphaevolve-2506.13131-interface.md.
   USAGE: anchors MAP-Elites + island-model hybrid for template_archive;
   SEARCH/REPLACE diff format anchors AlphaEvolve's canonical diff shape.

DECISION-5 GEPA + OpenEvolve = background-only
   arXiv:2507.19457 (GEPA: Reflective Prompt Evolution; ICLR 2026 Oral)
   reports +6pp avg vs GRPO, up to +19pp, 35x fewer rollouts vs MIPROv2.
   USAGE: cited as background prior-art in template_archive module doc.
   NOT load-bearing; not ingested as interface map.
   OpenEvolve (github.com/jamesahou/openevolve, codelion/openevolve)
   open-source community implementations of AlphaEvolve. Background-only.
   Primary academic source remains arXiv:2506.13131.

DECISION-6 Krakovna spec-gaming spreadsheet -- trap-task seed list
   Real, verified via vkrakovna.wordpress.com and DeepMind blog.
   Direct Google-Sheets URL is auth-walled (404 on direct fetch);
   gwern.net mirror is 10MB+ overflow. Content grounded via blog post +
   DeepMind sources.
   Confirmed categories: RL reward hacking (Coast Runners green-block loop,
   Lego stacking flip, robot grasp-camera hover, simulated-walking
   leg-hook), NLP ROUGE gaming, code-model unit-test edit, simulation
   physics exploitation, reward tampering.
   USAGE: trap_tasks::TrapPattern enum variants seed off this list.
   Trap fixtures land in .elai_cc/held-out/traps/ as RUNPOD-PENDING
   for the full 50-fixture set (substrate-only this session; live trap
   replay needs model evolution loop).

DECISION-7 TripwireKind closed-enum: FEATURE-GATE, not remove/expand
   Per re-anchor instruction: TripwireKind::Perplexity variant STAYS.
   In-loop emit path of Perplexity is gated behind
   cfg(feature = "darwinian-evolve") (NOT cfg(feature = "perplexity_external")
   as the plan body wrote -- the substrate is darwinian-evolve, perplexity
   stays as a tripwire kind whose in-loop dispatch toggles with that gate).
   PageHinkley + Adwin + EstInvarianceViolation variants ADDED behind the
   same feature gate (additive to TripwireKind; SCHEMA bump covers it).
   IMPLEMENTATION REVISION: this session ships the substrate WIRE-DARK
   only; the actual additive enum variants land in a follow-on patch
   once the feature gate is exercised (avoids cascading payload struct
   churn under a closed-enum invariant).

DECISION-8 FsmSubstate rename to avoid collision with FsmState::Auditing
   Per re-anchor instruction: FsmSubstate::Auditing would collide with
   top-level FsmState::Auditing. New substate named
   FsmSubstate::DarwinianGenerationEvaluating (NOT Promoting -- the
   evaluation phase is the gated boundary; promotion is a state of the
   archive, not the FSM).
   Placement: orthogonal to top-level Auditing state (PreToolUse hook
   keys on (state, substate) tuple). Restricts edits to template_archive
   admit + archive_chain append paths while in this substate.

DECISION-9 Pattern-2 single SSB variant
   Per re-anchor instruction: ONE new SSB variant carrying a phase
   discriminator, NOT 8 variants.
   Variant: DarwinianGenerationCompleted
   Payload struct DarwinianGenerationCompletedPayload carries:
     - phase: snake_case discriminator {mutation, selection, evaluation,
       two_gate_blocked, est_invariance_measured, contam_filter_rejected,
       transfer_regression_blocked, trap_tripped, archive_sig_verified,
       archive_sig_broken, archive_cell_elevated}
     - generation_index: u32
     - cell_blake3_prefix: [u8; 8] (first 8 bytes of the BehaviourDescriptor
       hash; full blake3 stored only in archive_chain)
     - tau: Option<f64> (margin gate value when phase == two_gate_blocked OR
       archive_cell_elevated)
     - k_m: Option<u32> (capacity cap when phase == two_gate_blocked)
     - reason_hash: Option<[u8; 8]> (blake3 prefix of human-readable reason
       string when phase is a blocked/broken variant)
     - ts_ms: i64
   Cascade cost: 1 variant x 10 sites = 10 edit points (vs 8 x 10 = 80).
   Dashboard reads by phase discriminant; per-phase counters are derived.

DECISION-10 ENV gate: ELAI_DARWINIAN_LOOP_ENABLE
   Precedent: D-29 ELAI_BAI_MODE_ENV (BaiMode::Flat default).
   Default OFF; MAP-Elites + Two-Gate + EST probe + adversarial farming
   are wire-dark until env set.
   Tests with the gate set fire the new SSB variant; tests without the
   gate confirm no emission (existing bai.rs Thompson sampler stays
   untouched as the legacy default).

DECISION-11 Dashboard panel re-anchor
   Plan cites crates/dashboard-v2/src/lib/components/MutationArchivePanel.svelte
   ACTUAL: crates/dashboard-v2/src/lib/drawers/MutationArchivePanel.svelte
   (D-26 polling viewer). E-7 delta targets the drawers/ path.
   This session: NO dashboard edits land; the 3D-grid + Two-Gate + EST
   banner + trap-trip surface marked RUNPOD-PENDING for follow-on.
   Rationale: dashboard delta requires the new Tauri commands first; the
   substrate ships now, the surface lands when the evolution loop is live.

DECISION-12 archive_cmd path drift
   Plan cites crates/cli/src/archive_cmd.rs -- does not exist.
   ACTUAL: crates/cli/src/archive_verify.rs (canonical walker re-export).
   This session: no CLI edits. The map-elites-stats subcommand + Ed25519
   sig walk extension marked RUNPOD-PENDING for follow-on.

DECISION-13 bai.rs doc/impl mismatch already documented in D-29 -- leave alone
   Per re-anchor instruction. This session does NOT touch bai.rs.
   DiscountedTs + SuccessiveRejects scaffolds land in a new sibling
   module under feature gate (orchestrator/src/darwinian_bai.rs), not as
   a delta to bai.rs. The existing Thompson sampler remains the legacy
   default; darwinian_bai is reachable only via the env gate +
   feature gate.

DECISION-14 SCHEMA head verified
   At executor time grep "const SCHEMA_VERSION: i64" -> 153.
   This session bumps to 154 (atomic +1 for the single DarwinianGenerationCompleted
   variant). TOTAL_VARIANTS in action_type_lock.rs 426 -> 427.

DECISION-15 Anti-slopsquatting deferred
   ed25519-dalek crate verification deferred (no Cargo.toml change this
   session). archive_crypto.rs scaffold uses blake3-only signature
   placeholder; ed25519 layer marked RUNPOD-PENDING with explicit
   `cargo info ed25519-dalek` step gating the follow-on.

PAPER GROUNDING STATUS
   arXiv:2510.04399 -- interface map present + html local. Load-bearing.
   arXiv:2305.10718 -- interface map present + pdf local. Load-bearing.
   arXiv:2506.13131 -- interface map present. Load-bearing.
   arXiv:2507.19457 (GEPA) -- background-only; no ingest needed.
   arXiv:2505.22954 (DGM) -- interface map at .claude/papers/2505.22954v3.pdf;
                              previously ingested.
   arXiv:2507.05619 (EST) -- interface map present
                             (reward-hacking-detection-2507.05619).
   arXiv:2403.07974 (LiveCodeBench) -- interface map present.
   arXiv:2501.03124 (PRMBench) -- interface map present.
   Krakovna spreadsheet -- content grounded via blog + DeepMind sources;
                            direct URL auth-blocked.
   arXiv:2602.12413 -- REMOVED from E-7 citations (wrong paper).
   arXiv:2410.06491 + 2402.06627 + 2310.09144 -- referenced in plan body
   but not ingested; flagged in workflow/sessions/s-25120/mistakes.md for
   follow-on if the live evolution loop ships.

LIVE-MODEL EVOLUTION LOOPS RUNPOD-PENDING
   - MAP-Elites generation step with real model proposals + verifier scoring
   - EST probe paraphrase mode (cloud-Sonnet route)
   - Trap-task fixture replay against real proposals
   - Ed25519-signed merkle chain second-party signer (auditor in distinct
     sandbox)
   - Dashboard 3D-grid + Two-Gate + EST banner + trap-trip surface
   - CLI `elai archive map-elites stats` + `elai archive verify` (Ed25519)
   - 50 hand-crafted trap fixtures (~1500 LOC)

STUB-BAN REGEX STATUS
   "fitness_proportional|softmax_parent" -- grep returns 0 hits in
   .elai_cc/crates/orchestrator/src/ at executor time. Per re-anchor
   instruction "outside this plan's scope" -- no action required this
   session (the 3 pre-existing file hits flagged in context were stale
   or already removed in prior cluster).

FILES TOUCHED THIS SESSION
   - workflow/reasoning/E-7-decisions.md (this file, NEW)
   - .elai_cc/crates/orchestrator/src/ssb.rs (+1 variant, +1 payload,
     +cascade sites; SCHEMA 153 -> 154)
   - .elai_cc/crates/orchestrator/tests/action_type_lock.rs
     (TOTAL_VARIANTS 426 -> 427 + variant insert + golden recompute)
   - .elai_cc/crates/orchestrator/src/fsm.rs
     (+FsmSubstate::DarwinianGenerationEvaluating + as_str arm + Display)
   - .elai_cc/crates/orchestrator/src/lib.rs
     (+pub mod template_archive, promotion_gate, est_probe,
      cross_domain_canary, trap_tasks, archive_crypto, l3_lock,
      darwinian_bai; all behind cfg(feature = "darwinian-evolve"))
   - .elai_cc/crates/orchestrator/src/template_archive.rs (NEW, WIRE-DARK)
   - .elai_cc/crates/orchestrator/src/promotion_gate.rs (NEW, WIRE-DARK)
   - .elai_cc/crates/orchestrator/src/est_probe.rs (NEW, WIRE-DARK)
   - .elai_cc/crates/orchestrator/src/cross_domain_canary.rs (NEW, WIRE-DARK)
   - .elai_cc/crates/orchestrator/src/trap_tasks.rs (NEW, WIRE-DARK)
   - .elai_cc/crates/orchestrator/src/archive_crypto.rs (NEW, WIRE-DARK,
     blake3 placeholder; Ed25519 RUNPOD-PENDING)
   - .elai_cc/crates/orchestrator/src/l3_lock.rs (NEW, WIRE-DARK)
   - .elai_cc/crates/orchestrator/src/darwinian_bai.rs (NEW, WIRE-DARK,
     DiscountedTs + SuccessiveRejects scaffolds; bai.rs untouched)
   - .elai_cc/crates/orchestrator/Cargo.toml (add feature
     "darwinian-evolve"; ed25519-dalek NOT added yet -- anti-slopsquatting)

NOT TOUCHED THIS SESSION
   - bai.rs (per DECISION-13)
   - tripwire.rs (per DECISION-7; closed-enum churn deferred)
   - cli/src/archive_verify.rs (per DECISION-12)
   - dashboard-v2/src/lib/drawers/MutationArchivePanel.svelte (per
     DECISION-11)
   - self_mod_bridge.rs (no parent-selection swap; loop reachable only
     via env gate which is OFF by default)
   - held_out.rs (per DECISION-3; existing 9-gram + temporal filter
     stays, no soft-contam reference added)

ACCEPTANCE STATUS (vs plan)
   [SCAFFOLD] MAP-Elites archive: type signatures + WIRE-DARK; live admit
              loop RUNPOD-PENDING
   [SCAFFOLD] Two-Gate guardrail: two_gate_promote signature + tests
              compile; live promotion loop RUNPOD-PENDING
   [SCAFFOLD] Discounted-TS + Successive Rejects: scaffolds in
              darwinian_bai.rs; live regret bound RUNPOD-PENDING
   [SCAFFOLD] EST invariance probe: probe surface + perturbation modes;
              cloud-Sonnet paraphrase route RUNPOD-PENDING
   [DEFERRED] Tripwire rebuild: TripwireKind churn deferred (DECISION-7)
   [SCAFFOLD] Contam filter: existing held_out.rs unchanged; paraphrase
              + canary RUNPOD-PENDING
   [SCAFFOLD] Transfer-regression threshold: scaffold + tests; family
              selection RUNPOD-PENDING
   [SCAFFOLD] Trap fixtures: trap_tasks::TrapPattern enum + registry;
              50-fixture set RUNPOD-PENDING
   [SCAFFOLD] Ed25519 merkle chain: blake3 placeholder; ed25519 dep
              RUNPOD-PENDING
   [SCAFFOLD] L3 lock-list extension: 4 lock entries + assert_lock_intact;
              pre-commit hook RUNPOD-PENDING
   [PENDING]  CLI map-elites-stats + verify (DECISION-12 RUNPOD-PENDING)
   [PENDING]  Dashboard panel delta (DECISION-11 RUNPOD-PENDING)
   [PARTIAL]  No .unwrap()/.expect() in non-test code -- enforced in new
              modules; no blake2/sha2/async-trait introduced.

NOTES
   This session ships the SUBSTRATE; the LIVE EVOLUTION LOOP is gated
   by the env var + cargo feature. The substrate compiles, the SSB
   variant is registered, the FSM substate is wired into PreToolUse hook
   gating. No live model evolution runs occur.
   Per CLAUDE.md "no destructive ops without explicit user approval" --
   the adversarial farming retraining loop (D-28 original Wave-5 scope)
   remains deferred. E-7 lands the 50-trap evaluation SURFACE only as
   scaffold; the actual 50 hand-crafted fixture .toml files land when
   the live loop is exercised.
