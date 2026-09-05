# H-VERIFICATION-GROUNDING-MAP — Track 7 (DGM SELF-IMPROVEMENT VERIFICATION HARDENING) symbol map
Date: 2026-06-05. Author session: this thread, from ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md
  (6 background research agents, every arXiv id + repo WebFetch-verified by the agent) + E-19_reasoning_quality.md
  (the SE-Agent L4 recombination relocation). DGM code read/grepped DIRECTLY this session (anchors below).
Purpose: the W-7 grounded symbol map for H-43..H-48. Every consumes_symbol / reachable_from anchor in those
  files is grep-confirmed here against LIVE 2026-06-05 source (crates/orchestrator/**), SHIPPED-vs-ABSENT
  classified. Read your H-N's slice + the CITATIONS + the VD-1..VD-5 deltas before coding.

WHY TRACK 7 EXISTS (the meta-finding, one paragraph)
The self-verification dossier's six agents independently converge on ONE result: A RECURSIVE SELF-IMPROVEMENT
LOOP WITHOUT A REAL, UN-GAMEABLE VERIFICATION FILTER PROVABLY COLLAPSES; WITH ONE, IT PROVABLY CONVERGES
(Self-Verification-Prevents-Collapse, NeurIPS 2025 OpenReview X5Hk8aMs6w — naive recursive self-training ->
exponential error growth, a real verification filter -> finite-sample bound; Solver-Verifier Gap 2507.00075 —
gain bounded by how much STRONGER the verifier is than the generator; Skalse 2209.13085 — a proxy is unhackable
iff raising proxy-return never lowers true-return, RARE for soft proxies; Co-rewarding 2508.00410 — a SINGLE
self-judge collapses 44.81% vs 85.14% ground-truth). ELAI's DGM-as-shipped is the DIVERGENT branch because its
verification FILTER is the weak link: the fitness is a byte-hash (GAP-1) and the cheat-detector is a keyword
grep (GAP-2). Track 7 replaces the two placeholders with measured, code-backed mechanisms and upgrades the
scoring/selection/eval to the convergent regime. This is ACTIVATION + replacing two placeholders, NOT invention
of new doctrine: PART D of the dossier confirms ELAI's instincts (Two-Gate, cross-family adversary, reusable
holdout, Verus narrow slice, held-out store) are already right and already planned. The work is the FILTER.

================================================================================
DG — SHIPPED DGM-loop substrate (grep-confirmed 2026-06-05; the BUILD/REPLACE targets)
================================================================================
DG-1  darwin_admit_loop.rs:186 `Candidate::derived_alpha(&self) -> f64` — SHIPPED but a PLACEHOLDER.
      Body (read this session, lines 186-199): honors `config.force_alpha` (fixture), else
      `blake3::hash(config_blob)` first-8-bytes -> u64 -> /u64::MAX. A function of candidate BYTES, NOT
      performance. Doc-comment self-describes "the live multi-generation Python bench eval is the DEFERRED
      Tier-2." >>> GAP-1 (P0). H-43 REPLACES this with execution-grounded fitness. Callers: run_one_generation
      (darwin_admit_loop.rs:424, alpha bound at :447); score_candidate(&c, alpha, gen, cfg) is the guard chain.
DG-2  trap_tasks.rs:123 `score_candidate_for_traps(config_blob: &[u8], eval_log: &str) -> Option<TrapPattern>`
      — SHIPPED but a PLACEHOLDER. Body is substring grep over literal keyword arrays (trap_tasks.rs:139
      `["fake_test_log","fake test","fabricate test","forge test","spoof test log"]`, :143
      `["verifier_capture","capture verifier","prompt-inject the verifier","rubber-stamp","rubber stamp"]`).
      TrapPattern::{FakeTestLog (as_str :89), ...}. The dossier self-flags "name-grep is unsound; an adversary
      may evade it." >>> GAP-2 (P0). H-44 REPLACES with behavioral detection + measures recall on TRACE.
DG-3  promotion_gate.rs:109 `two_gate_promote(candidate, descriptor, archive, cfg)` — SHIPPED. tau=DEFAULT_TAU
      (promotion_gate.rs:50/60), k_m=DEFAULT_K_M (:54/61). Margin gate: `margin < cfg.tau` (:124) where margin
      is candidate_score - incumbent (incumbent often 0.0). Doc self-labels tau "UNCONFIRMED; carries forward
      bai.rs" (:39). Scores on the candidate's OWN alpha (mean), no ensemble/worst-case. >>> GAP-3 (P1). H-45
      adds worst-case/variance-penalized + lineage-diverse scoring + PRMBench-calibrated tau. EXTEND the score
      two_gate_promote consumes; do NOT rewrite the tau/k_m gate (GD-1) — same discipline H-39 uses.
DG-4  config_archive.rs:49 `sigmoid_perf_score(alpha) = 1/(1+exp(-lambda*(alpha-alpha_0)))` (lambda=10,
      alpha_0=0.5, DGM section 2.3) + :63 `sigmoid_perf_score(v.score)*exploration_bonus(children_count)` +
      :274 `select_parent(mode)` / :283 `select_parent_with<E:EntropySource>` + :198 `insert_variant` —
      SHIPPED. Parent selection weights by the node's IMMEDIATE score. >>> the HGM Metaproductivity-Performance
      Mismatch target. H-46 adds clade-metaproductivity (aggregate descendant subtree deltas) as the selection
      key. config_archive is otherwise solid (DB-enforced per-mode unique-active, archive cap, ancestors-of-
      active protected) — compose, do not rewrite the archive (GD-1).
DG-5  bai.rs:286 `sample(arms: &[ArmPosterior], rng) -> Option<usize>` + bai.rs:83 ArmPosterior (Beta(a,b)) +
      bai_hierarchical.rs (D-29 hierarchical; sample_theta :392, best-arm :512/:607/:661) — SHIPPED structs,
      but per the dossier darwinian_bai sample/update/run are SCAFFOLD-ONLY (the live regret-bounded selection
      on a REAL reward is RUNPOD-PENDING). >>> GAP-4 (P2). H-48 wires the bandit on H-43's real fitness.
DG-6  agent_pool.rs:1745 `sample_held_out_task(...)` — SHIPPED (a held-out sampling surface). The DGM-eval
      firewall (H-47) reads held-out items; D-6/E-54 own the store + the n-gram/cosine filter (compose).
DG-7  benchmarks/safedelta_runner.rs (SecCodePltRunner pattern, InsecureRateMeasured SSB) — SHIPPED FROZEN-
      runner + measured-off-agent-row pattern (RS-8 in H-REASONING-GROUNDING-MAP). Every Track-7 measured row
      (FitnessEvaluated, TrapBehaviorDetected, ...) MIRRORS this shape (GD-1; do NOT build a new runner shape).

================================================================================
PRIOR-PLAN COMPOSITION TARGETS (already authored; COMPOSE-WITH, do NOT restate — GD-1 P0)
================================================================================
PC-1  D-2_ppm_sidecar_wiring.md — PPM sidecar (Qwen2.5-Math-PRM-7B @ 8101). The CROSS-FAMILY verifier upgrade
      path for H-43/H-45 (GenPRM 2504.00891 / ThinkPRM 2504.16828 / rStar-Math 2501.04519 are its lineage).
      H-43 ships execution-grounded fitness NOW; D-2 upgrades the step-level signal LATER (same arc as E-19 PRM-lite).
PC-2  D-5_bai_thompson_promotion.md + D-29_hierarchical_bai_research.md — the BAI promotion bandit. H-48 ACTIVATES
      these on H-43's real reward (Discounted-TS 2305.10718 + Successive-Rejects Audibert COLT 2010; ExIt 2509.04575
      curriculum from partial histories). H-48 is the wiring, NOT a new bandit (GD-1).
PC-3  D-6_held_out_task_store.md + E-54_eval_corpus_design.md — time-windowed held-out store + 13-gram + embedding-
      cosine + cutoffs + eval-awareness probes (LiveCodeBench 2403.07974, Soft-Contamination 2602.12413, KOR 2410.06526,
      eval-awareness 2510.20487). H-47 COMPOSES these and adds ONLY net-new: GSM-Symbolic template regeneration at
      score-time (2410.05229) + the date-GATE wiring tying held-out dates to the darwin generation timestamp +
      binding the H-19 budget to the loop's adaptive queries. H-47 does NOT re-implement the filter or the store.
PC-4  H-19_reusable_holdout_budget.md (Track 3) — Thresholdout/Ladder DP query budget (1506.02629). H-47 BINDS
      H-19's HoldoutBudgetEvent to the DGM loop's adaptive "did my score go up?" queries (a self-improver firing
      thousands of holdout queries IS adaptive analysis; without Thresholdout it WILL overfit). REUSE H-19's variant.
PC-5  D-13_adversary_role_cross_family.md + E-6_multi_agent_failure_modes.md — cross-family adversary + iso-budget
      verifier retention. H-44/H-45 cross-view checks COMPOSE D-13 (different pretraining family = the Co-rewarding
      requirement). E-6's IsoBudgetReport is H-40's (Track 6) target, not Track 7's; do not collide.
PC-6  E-53_formal_verification_codegen.md — Verus/Kani/Z3 proof gate. The UN-GAMEABLE CEILING (VD-5). COMPOSE-NOT-
      AUTHOR: E-53 already plans it; Track 7 references it as the narrow CSQ-Critical slice (VERINA 4.9% / CLEVER
      0.6% measured pass rates = never routine). No Track-7 file re-authors a proof gate.
PC-7  E-19_reasoning_quality.md Task 11 — the SE-Agent L4 recombination operator (2508.02085), CONDITIONAL, cloud-
      hard-lane only, gated on an off-agent fitness. H-43's execution-grounded fitness IS that off-agent fitness;
      E-19 Task 11 becomes runnable once H-43 ships. COMPOSE (name the dependency); do NOT restate the operator.
PC-8  H-39_abstention_aware_promotion_eval.md (Track 6) — the abstention dimension on the SAME promotion reward
      H-43/H-45 touch. ORTHOGONAL LAYERS: H-43 = the correctness SIGNAL (execution-grounded), H-39 = credit
      abstention on it, H-45 = aggregate it worst-case+lineage. H-43 supplies the real "correct" bit H-39 assumes.

================================================================================
VD — TRACK 7 DELTA RULES (additional to W-1..W-10 + GD-1/2/4/5; all BLOCKING; canon: H-0_INDEX.md)
================================================================================
VD-1  FITNESS-IS-EXECUTION-GROUNDED-NOT-BYTES. Any signal the self-improvement loop OPTIMIZES (derived_alpha,
      the promotion reward, the bandit arm) MUST be a function of the candidate's MEASURED EXECUTION behavior
      (pass-rate over generated tests / dual-agreement mass), NEVER its bytes (hash) or its own confidence. A
      config producing FAILING code MUST score LOW. A byte-hash or self-reported fitness defaulted-ON is a FAIL
      (the divergent branch — Self-Verification-Prevents-Collapse + Skalse unhackable-proxy). [H-43]
VD-2  DETECTOR-IS-BEHAVIORAL-WITH-MEASURED-RECALL. Cheat/trap detection MUST be behavioral (execution, spec-test
      conflict, isomorphic perturbation), NOT keyword/substring grep; AND its recall MUST be MEASURED on a frozen
      out-of-band contrastive corpus (TRACE) and reported (GD-2 reproducible). A keyword-free hack that passes is
      a FAIL; an unmeasured detector advertises a guarantee it cannot prove (extends GD-5). [H-44]
VD-3  SCORE-WORST-CASE-NOT-MEAN + LINEAGE-DIVERSE. Promotion scores on the WORST-CASE / variance-penalized
      ensemble verdict (Coste 2310.02743 WCO/UWO), judges diversified by PRETRAINING lineage not seeds (Eisenstein
      2312.09244); cross-view disagreement BLOCKS (Co-rewarding 40-pt self-judge collapse). Promotion on the
      candidate's own mean alpha is a FAIL. tau is CALIBRATED on PRMBench (2501.03124), never a carried-forward
      UNCONFIRMED constant at default-ON. [H-45]
VD-4  STRONGER-THAN-GENERATOR VERIFIER. The loop's gain is bounded by the verifier-minus-generator capability gap
      (Solver-Verifier Gap 2507.00075); the verifier MUST be execution- or proof-grounded so the gap stays open,
      and the generator + cross-family verifier MUST run on DIFFERENT pretraining families (Co-rewarding collapse;
      the abliterated 7B may not verify its own family). A verifier weaker-than-or-equal-to the generator is not a
      valid gate. [H-43/H-45/H-46/H-48 — the cross-cutting rule]
VD-5  HONEST-UNDECIDABLE-RESIDUE. Name the irreducible residue and route it to human/proof Layer-3; never claim
      immunity. Proof gates rarely pass (VERINA 4.9% / CLEVER 0.6% — narrow CSQ-Critical slice only, E-53). A strong
      self-improver can SANDBAG its eval (eval-awareness 2505.23836 AUC 0.83 / 2510.20487 >80%) — mitigate via
      indistinguishability (H-47), do not cure. Cross-hardware FP bitwise + concept drift stay human-gated. Keep
      the whole self-improvement loop DARK/allowlisted (W-2 kill-switch); this makes it SAFER, not a sales headline.
      Extends GD-5 / TD-5. [all Track-7 H-N]

================================================================================
SCHEMA BLOCK (LIVE baseline 2026-06-05; NEVER hard-code; re-read at edit — W-9)
================================================================================
SCHEMA_VERSION = 224 (crates/orchestrator/src/ssb.rs:3010). TOTAL_VARIANTS = 501
  (crates/orchestrator/tests/action_type_lock.rs:645). AUTHORITATIVE, read this session.
  STALE — do NOT use: CLAUDE.md 191/464; F-plan 205/482; H-0 Track-1/2 snapshot 207/484; H-0 Track-4/6 220/497.
  LATENT DRIFT: e45_cross_language_drift_contract.rs:294 still pins ~482; the FIRST Track-7 H-N to bump SCHEMA
  also bumps that pin to LIVE (same instruction Track 4/6 carry — whichever ships first does it).
Each Track-7 H-N adds ONE off-agent-artifact action_type (Pattern-2 single-variant-with-Kind), baseline+1
EACH, SHIP-ORDER-CUMULATIVE; re-read the live integer at edit:
  H-43  +FitnessEvaluated (Pattern-2; Kind: ExecutionPassRate / DualAgreement / FitnessRegression). baseline+1.
  H-44  +TrapBehaviorDetected (Pattern-2; Kind: ImpossiblePassExploit / IsomorphMismatch / ExecutionHallucination /
        KeywordFallback); the recall roll-up rides this variant's payload {recall, n, corpus_hash} (no 2nd variant). baseline+1.
  H-45  +PromotionScoreAggregated (Pattern-2; Kind: WorstCaseScored / VariancePenalized / LineageDiverseDisagree). baseline+1.
  H-46  +CladeSelected (Pattern-2; Kind: CladeMetaproductivity / ImmediateScoreFallback). baseline+1.
  H-47  +EvalFirewallEvent (Pattern-2; Kind: PreUpdateItemRejected / TemplateRegenerated / IndistinguishabilityProbe);
        the holdout-budget leg REUSES H-19's HoldoutBudgetEvent (no new variant — GD-1). baseline+1.
  H-48  NO new action_type (default) — REUSE D-5's BAI promotion telemetry; if darwinian_bai has no emitter, the
        author may add +BanditArmSampled baseline+1 at edit (prefer reuse; minimise bumps). Conditional.
Net Track-7 delta if H-43..H-47 ship = +5 variants (224->229, 501->506); H-48 conditional. Confirm live at each edit.

================================================================================
CITATIONS (dossier-verified; the 3 keystones + 2 newest re-confirmed via WebSearch 2026-06-05 this session)
================================================================================
RE-CONFIRMED THIS SESSION (WebSearch, title+repo match):
  CodeRM 2501.01054 "Dynamic Scaling of Unit Tests for Code Reward Modeling" (ACL 2025; RUCKBReasoning/CodeRM;
    CodeRM-8B; +18.43% HumanEval Plus Llama3-8B). [H-43 TOP PICK for execution-grounded fitness]
  ImpossibleBench 2510.20270 (Zhong/Raghunathan/Carlini; safety-research/impossiblebench; spec-test conflict ->
    pass==cheat; "STOP if tests flawed" cut hack 93%->1%). [H-44]
  HGM 2510.21614 "Huxley-Godel Machine" (metauto-ai/HGM; Clade-Metaproductivity; Metaproductivity-Performance
    Mismatch; SWE-Verified-60 56.7 vs DGM 53.3 at ~2.38x less compute). [H-46] (uses GPT-5-mini naming — 2026 eco, expected)
  2604.15149 "LLMs Gaming Verifiers: RLVR Reward Hacking" (Isomorphic-Perturbation Testing) + 2605.02964 "Reward
    Hacking Benchmark" — both confirmed real arXiv (cross-ref'd in the ImpossibleBench result set). [H-44]
DOSSIER-VERIFIED (agent WebFetch, this-week, see ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md PART G):
  CodeT 2207.10397 (dual-agreement) · Sol-Ver 2502.14948 · CodeHalu 2405.00253 (yuchen814/CodeHalu) · TRACE
  2601.20103 (PatronusAI/trace-dataset) · Co-rewarding 2508.00410 (tmlr-group) · Coste 2310.02743 (WCO/UWO ensemble)
  · Eisenstein 2312.09244 (Helping or Herding?) · PRMBench 2501.03124 (ssmisya/PRMBench) · GenPRM 2504.00891 ·
  ThinkPRM 2504.16828 · rStar-Math 2501.04519 · GSM-Symbolic 2410.05229 · LiveCodeBench 2403.07974 · GSM1k
  2405.00332 · Thresholdout 1506.02629 · eval-awareness 2505.23836 · Solver-Verifier-Gap 2507.00075 · Skalse
  2209.13085 · Self-Verification-Prevents-Collapse (NeurIPS OpenReview X5Hk8aMs6w — NO arXiv id, do not invent) ·
  Discounted-TS 2305.10718 · ExIt 2509.04575 · Vericoding 2509.22908 / VERINA 2505.23135 / CLEVER 2505.13938 (E-53).
  MEDIUM (re-verify via paper-ingestor before load-bearing external cite): SCoRE 2603.24704, PSV-Verus 2512.18160.

End of H-VERIFICATION-GROUNDING-MAP.
