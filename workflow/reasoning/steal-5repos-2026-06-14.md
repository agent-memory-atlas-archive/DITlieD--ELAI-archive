STEAL ANALYSIS — 5 repos, incremental (one repo at a time, reason-then-wire-then-next)
Date: 2026-06-14
Rule (user, binding): treat ALL ELAI plans (shipped/DARK/PARTIAL/PLANNED) as DONE. A steal
  only counts if it is NET-NEW — not already shipped, planned, or in the REJECTED ledger
  (ELAI-FEATURE-MAP.md Section 4). Output is a NEW plan later, never an edit to existing plans.
Grounding: ELAI-FEATURE-MAP.md (canonical, read in full this session) + phase-index-MASTER.txt
  + the Nick Nisi/WorkOS "Case-on-Pi" transcript.
Verdict enum: NET-NEW | EXTENSION(plan-id) | ALREADY-SHIPPED(plan-id) | ALREADY-PLANNED(plan-id)
  | ALREADY-REJECTED(ledger) | REJECT(reason).

================================================================================
REPO 1 — earendil-works/pi   (opus explorer; verdict: REJECT / ALREADY-COVERED)
================================================================================
WHAT IT IS: generic provider-agnostic single-agent TS runtime + coding-agent CLI + unified
  multi-provider LLM API + TUI lib (Mario Zechner/badlogic, MIT, ~62k LOC, active 2026-06-13).
  It is the SUBSTRATE. Source = docs/.source/pi, map = docs/.source/pi-INDEX.md.

CRITICAL CORRECTION: the transcript's interesting "Case" harness (role-FSM
  implementer/verifier/reviewer/closer/retro + SHA256-of-test-output proof-of-work + retro
  memory) is GREP-CONFIRMED ABSENT from pi. Case is a separate product built ON pi via pi's
  tool-call block hook. So the steal surface here is pi's substrate mechanisms + the
  transcript's Case ideas (evaluated from memory of the talk, not from code).

PI MECHANISM -> ELAI COVERAGE (all 11, each already covered):
1. before/afterToolCall block+rewrite gate seam -> HOOK SYSTEM (phase-4C, WIRED) + J-12 H4
   per-tool result annotators. COVERED.
2. ~1200-line typed extension API (lifecycle events, registerTool/Command, prompt+context
   rewrite hooks) -> ELAI uses rules-as-data (3F) + hooks + skills; different model, same
   surface. COVERED.
3. Session DAG + fork + branch-summary -> SESSION DAG (WIRED) + H-49 time-travel + H-35
   /rewind. pi-INDEX itself flags this as "prior-art for ELAI time-travel". COVERED.
4. Split-turn iterative compaction w/ file-op tracking -> E-25 ladder + H-6 masking + H-50
   regime-aware + J-25 MiMoCode microcompaction. COVERED.
5. prepareNextTurn mid-run model/thinking ladder -> I-7 cascading escalation + E-3 handoff +
   I-26 dynamic thinking budgets. COVERED.
6. Steering vs follow-up message queues -> J-12 RUNTIME INTERVENTION HARNESS +
   intervention_respond (J-2). COVERED (marginal: see below).
7. Faux deterministic test provider (zero-token loop testing) -> DST (phase-3I) + RRA
   replay --verify (G-2); llm_recorder already REJECTED as RRA-subsumed (Section 4). COVERED.
8. Project-trust gate for project-local skills/extensions -> F-6 SkillTrustTier +
   skill-import-policy (skillspector). COVERED.
9. Unified multi-provider pi-ai -> B-37 MULTI-RUNTIME BACKEND + J-30 BYOK + I-49 lanes. COVERED.
10. Prompt-as-slash-command workflows -> slash REPL + skills. COVERED.
11. Multi-session git-safety doctrine (bans reset --hard/clean/stash) -> near-IDENTICAL to
   .claude/rules/agent-dispatch-preamble.md R-1 (convergent). COVERED.

TRANSCRIPT "CASE" IDEAS -> ELAI COVERAGE:
- SHA256 proof-of-test-output (enforce-don't-instruct, "it stopped lying because i made it
  prove it") -> TEST-INTEGRITY CANT-LIE GATE F-4: blake3 stdout_hash, auditor cannot emit
  passed=true without the orchestrator-captured hash. ALREADY-SHIPPED. This is the single
  biggest idea in the talk and ELAI already has it.
- Retro agent mines its own Claude/Codex JSONL transcripts for doom-loops (same tool 3x) +
  writes per-stack memory -> J-12 offline failure-classifier feeding I-12 EVIDENCE-GATED
  PLAYBOOK CURATOR + anti-spiral blake3 3-identical-hash stop (phase-3C) + MAST E-6. PLANNED.
- "fix the harness not the code" + "measure don't pursue (evals)" -> ELAI doctrine
  (wiring-enforcement, SAFEDELTA G-6, the whole gate stack). COVERED as posture.

MARGINAL-ONLY (not plan-worthy alone, candidate refinement riders):
- Per-STACK partitioned failure memory (Case keeps separate Next.js / TanStack memory files):
  a partition refinement to I-12 / E-5 user-memory, not its own plan. Record as a possible
  one-line rider IF a later repo motivates a memory plan.
- Interactive mid-run STEERING of a live autonomous agent (inject redirection without killing
  the loop): ELAI's J-12 intervention is a pre-tool BLOCK that explicitly never blocks
  ambiguity; proactive natural-language steering of a running loop is thinner in ELAI. Still
  judged COVERED by J-12 + J-32 front-door + InteractiveGate; note as the weakest-covered item
  if the user wants a live-steering UX pass someday.

NET VERDICT: REJECT (zero net-new). pi's value is convergent-validation that ELAI's
  independently-arrived-at architecture (DAG, gate-seam, compaction, git-safety) is sound, and
  confirmation that the talk's headline (cant-lie test gate) is already ELAI's F-4.

================================================================================
REPO 2 — mandyyyii/HarnessBridge   (BLOCKED — URL does not resolve)
================================================================================
STATUS: BLOCKED-UNRESOLVED. github.com/mandyyyii/HarnessBridge returns 404 (git clone x2 +
  WebFetch confirmed by sonnet explorer). Independently verified: the OWNER handle "mandyyyii"
  itself 404s (no such GitHub user), and no repo named "HarnessBridge" is indexed by web search
  (WebSearch x4). So this is not a repo-name typo — the owner slug is wrong. Did not guess a
  substitute (anti-slopsquatting / abstention-over-guessing). Awaiting corrected URL from user.
  Closest-but-unconfirmed lead worth checking: a benchmark paraphrased in search as "evaluates
  complete harness CONFIGURATIONS not isolated models — 106 tasks / 8 backends / 6 harnesses /
  5,194 trajectories" (Agent = Model + Harness framing); if HarnessBridge is that artifact it is
  an EVAL-harness-comparison bench, which would map to ELAI's E-9 AGENT EVAL BENCHMARKS / E-49
  CODING METRICS / SAFEDELTA G-6 model-harness-config framing — but UNVERIFIED, do not act.

================================================================================
REPO 3 — r-three/risk-under-pressure   (verdict: EXTENSION, weak-to-medium)
================================================================================
WHAT IT IS: "Risk Under Pressure: Compute-Aware Evaluation of Adversarial Robustness in
  Language Models" (Ehghaghi/Ecsedi/Chechik/Raffel, arXiv:2606.11409, MIT). A MEASUREMENT-
  METHODOLOGY paper: replace fixed-budget attack-success-rate (ASR@N) with compute-aware
  risk-vs-attacker-TFLOPs CURVES. Source = docs/.source/risk-under-pressure, map = -INDEX.md.
  Grounding MEDIUM: code structure verified; paper Table-1 numbers ABSENT from repo (unverified).

WHAT IT OFFERS (net-new methodology, grep/code-verified):
- Risk-compute curve: risk(tau) plotted vs cumulative attacker TFLOPs (risk_curve.py,
  cost_mapper.py). Exact per-attack FLOP accounting (fwd = 2*N_B*L/1000 TFLOPs; GCG +128
  candidate passes + backward; PAIR + attacker-fwd) -> fair cross-attack cost comparison.
- 3 compute-space summary metrics: C@tau (min TFLOPs to reach risk tau), AE (risk gain per
  TFLOP), CAURC (area under risk-compute curve) — cost_summary.py, with bootstrap/t CIs.
- Reusable probe assets: LLMJudge rubric (fiction/roleplay/"As an AI" disclaimer ruled
  insufficient — judges actual content; llm_judge.py:12-37); 8-template jailbreak battery
  (prefix_injection/refusal_suppression/style_injection/role_play/AIM/dev_mode/base64/combo;
  jailbroken_attack.py:20-88); AttackPolicy ABC initialize/refine plug-in seam (attacks/base.py).

ELAI COVERAGE (all plans treated DONE): the COMPUTE-NORMALIZED robustness-CURVE framing is NOT
  in the feature map. ELAI's security/robustness evals are all fixed-budget / pass-rate:
  SAFEDELTA G-6 (exploitable-diffs/100 tasks, ON-vs-OFF), F-2 SeCodePLT gate_recall, F-3
  AgentDojo ASR, D-23 sycophancy, D-19 PRMBench. None cost-normalized. -> methodology = net-new.

WHY ONLY WEAK-TO-MEDIUM (honest caveats):
- ELAI's threat wedge = generated-code vulns + prompt-injection-via-tool-output, NOT direct
  GCG/PAIR jailbreaks of the underlying model (vendor's concern). GCG needs white-box gradients
  = local Qwen only, never cloud Sonnet.
- The paper's HEADLINE use (Base->SFT->DPO->RLVR safety-training robustness comparison) is
  OFF-LIMITS: ELAI defers LoRA + RL and HARD-AVOIDs DPO (Section-4 ledger). No SafeRL story to tell.

CLEANEST ELAI FITS (rider-grade, for the eventual new plan — NOT its own subsystem):
1. Cost/compute-normalized robustness metric (C@tau / CAURC) as an axis on SAFEDELTA G-6 +
   AgentDojo F-3 — strictly MORE honest than fixed-budget ASR, aligns with GD-3
   ("ablation-not-leaderboard; every number recomputable under ELAI_SEED").
2. Risk-compute curve as the METRIC for the J-20 / F-10 security-MONOTONICITY-per-evolution
   gate: does each DGM REFINE iteration lower attacker-compute-to-breach? Natural Misevolve fit
   (J-20 already mandates a CWE security-invariant gate; this upgrades binary->curve).
3. 8-template jailbreak battery + LLMJudge rubric = probe-library rider to F-3 / E-37
   prompt-injection eval (MIT, compact, portable).

NET VERDICT: EXTENSION(G-6 / J-20 / F-3-E-37), weak-to-medium. Carry the compute-aware metric +
  probe battery as candidate riders in the new plan; DECISION FOR USER = does the new plan want
  a robustness-measurement axis at all, or is this off-wedge? Not plan-headline on its own.

================================================================================
REPO 4 — NVlabs/SpatialClaw   (verdict: MOSTLY REJECT; 1 weak EXTENSION + 1 J-8 confirmation)
================================================================================
WHAT IT IS: "Rethinking Action Interface for Agentic Spatial Reasoning" (NVIDIA + KAIST AI,
  2026, no confirmed arXiv id, paper PDF at spatialclaw.github.io). Training-free inference
  harness: a VLM writes one Python cell per step into a STATEFUL Jupyter kernel, observing
  intermediate outputs before committing. Source = docs/.source/SpatialClaw, map = -INDEX.md.
  LICENSE = NVIDIA Source Code License-NC (NON-COMMERCIAL) -> IDEA/PATTERN-ONLY, reimplement
  from scratch, NEVER vendor code (stricter than odysseus's MIT). Claims 59.9% avg over 20
  spatial benches / +11.2 vs prior best (README; paper numbers UNVERIFIED).

DOMAIN MISMATCH: ~half the repo is computer-vision/3D spatial reasoning (SAM3, Depth-Anything-3,
  Pi3 reconstruction, gravity-aligned world frames) — entirely OFF ELAI's code-editing wedge.

MECHANISM -> ELAI COVERAGE (all plans DONE):
- 0-1000 coordinate normalization (vlm.locate standardized to 0-1000 int scale) = ALREADY-
  PLANNED(J-8): feature map J-8 literally says "0-1000 normalized coords schema-v2,
  counts-from-AXTree-never-VLM". Exact convergent confirmation -> grounding for J-8, not new.
- AST pre-execution security sandbox (blocks 19 module families + 7 builtins before run) =
  COVERED: VibeGate F-1 (AST/GREP PreToolUse) + F-2a Py BLOCK rules + E-30 destructive-verb
  argv allowlist + Tier-C seccomp isolation + I-46 bash interceptor.
- code-as-action (Python cells as actions) = COVERED (E-35 CodeAct; "LangCode = CodeAct"
  Section-4). The stateful-REPL-with-persistent-namespace variant is off-wedge for a
  code-editing harness (it suits exploratory data analysis, not repo edits).
- isolated VLM grounding sessions (probe runs with NO main-agent history + "Not visible"
  absence preamble) = ~COVERED: J-1 2-call probe router + H-31 AUDITOR-OBJECTIVITY-FIREWALL
  (verifier/probe gets zero memory injection). Small refinement at most.
- cross-validation doctrine ("no single source reliable; every spatial conclusion needs 2
  independent evidence lines; pixel-motion banned as evidence") = COVERED (D-3 >=2-verifier
  concur to promote; D-13 cross-family; grounding verification). Convergent doctrine validation.
- answer-gate reflection node (always runs on ReturnAnswer, can REJECT + force retry, typed
  ChecklistItem PENDING/VERIFIED/FLAGGED across steps) = COVERED (auditor gate + H-54
  obligation-closure BFS + I-37 EARS per-hunk VERIFIED/UNVERIFIED + B-39 reflect pipeline).
- three-service auto-discovery via shared JSON logs = infra detail, off-interest (B-37/sidecar).

THE ONE MARGINAL NET-NEW:
- ABLATION-AWARE NAMED PROMPT SECTIONS: every prompt section has a named key, is exclude/
  override-able from JSON config, and each TOOL/ROLE self-declares its own prompt section
  (aggregated at runtime). Generalizes I-32 (role-persona A/B measured) + I-48 (prompt file
  artifacts) into a full named-section registry enabling systematic prompt-SECTION A/B
  ablation. Aligns with ELAI measure-everything doctrine. Weak-to-medium; EXTENSION(I-32/I-48).

NET VERDICT: MOSTLY REJECT/ALREADY-COVERED. Carry into the new plan only: (a) the named
  ablatable prompt-section registry as a weak EXTENSION rider; (b) cite SpatialClaw as
  independent confirmation of J-8's 0-1000 coord schema. License = idea-only. Spatial/CV = no fit.

================================================================================
REPO 5 — Aiden0526/EvoArena   (verdict: EXTENSION, weak-medium; 1 conflict flag)
================================================================================
WHAT IT IS: EvoArena (arXiv:2606.13681, 2026) — a BENCHMARK + MEMORY paper, NOT evolutionary
  computation (the name is misleading from a DGM/MAP-Elites lens). It measures whether agents
  stay reliable when the ENVIRONMENT evolves across a task chain. 3 tracks (Terminal-Bench-Evo,
  SWE-Chain-Evo, PersonaMem-Evo) + EvoMem = git-diff-inspired append-only patch-history wrapper
  over existing memory (Mem0/A-Mem/Terminus). Source = docs/.source/EvoArena, map = -INDEX.md.
  License: PersonaMem-Evo subfolder MIT; top-level UNSET. Findings: baselines avg 39.6%; EvoMem
  +3-14pp chain accuracy; PASS_TO_PASS regression 9.09%->6.32% (numbers from docs HTML; arXiv
  UNVERIFIED). DGM relevance = LOW (it's an inference-time memory sidecar, no self-improve loop).

MECHANISM -> ELAI COVERAGE (all plans DONE):
- PASS_TO_PASS regression rate (do previously-passing tests regress after a new milestone) =
  ALREADY-PLANNED(J-16) BLAST-RADIUS/ANTI-REGRESSION GATE (the user's #1 keystone:
  "regression-test selection, pass->fail = BLOCK"). Convergent confirmation of J-16's value.
- chain accuracy (conjunctive success across all evolution steps) ~= pass^k (I-38 QDELTA).
- scratch git-diff change-capture (baseline pre-run, diff post-run, installs git if absent) =
  COVERED (H-49 file_ops.rs pre-image + CAS object store + RRA; ELAI doesn't depend on env-git).
- two-stage retrieval gating (summary gate "NEED_PATCH_DETAIL?" then full detail) = ~COVERED
  (FAST/STANDARD/DEEP tiers + J-25 section-aware budgeted read + H-6 masking).
- memory self-reference stripping = ~COVERED (I-5 pristine context injector + H-31 firewall).

GENUINE WEAK-MEDIUM CANDIDATES (net-new refinements):
1. TYPED CHANGE_TYPE taxonomy for preference/state transitions — 8 kinds (same_object_flip,
   object_replacement, conditional_preference, attribute_swap, temporal_validity, ask_to_forget,
   new_preference, strengthen) + typed conflict-resolution (recency > distinctiveness >
   frequency; NO compromise answers). Refines ELAI's coarser Mem0 ADD/UPDATE/DELETE/NOOP +
   bi-temporal supersession. EXTENSION(H-31 / E-5 / J-21).
2. ENVIRONMENT-COMPATIBILITY-TAGGED RECALL — probe the runtime for available tools, tag memory
   records with constraints (CAP_*/FORBID_*), validate a recalled solution's compatibility with
   the CURRENT environment before reuse. Extends H-32 validate-before-use + B-21 feasibility
   probe to the execution-environment axis. EXTENSION(H-32 / B-21). Small but clean.

*** CONFLICT FLAG (do NOT re-propose a rejected approach) ***
  EvoMem KEEPS the full superseded patch chain and reasons over it. ELAI's H-31 DELIBERATELY
  hard-filters superseded rows and BANS [OUTDATED]-narration (keeping superseded rows = the
  measured Graphiti +68%-token trap, in the Section-4 REJECTED ledger). Adopt the TYPED-TRANSITION
  model, NOT "keep superseded rows in the prompt"; ELAI's hard-filter decision stands. The
  evolving-environment EVAL scenario is borderline-new but eval-corpus work (lower priority).

NET VERDICT: EXTENSION(H-31/E-5/J-21 + H-32/B-21), weak-medium. Carry typed transition taxonomy +
  env-compat recall as candidate riders; confirm J-16 (PASS_TO_PASS) + I-38 (pass^k). Honor the
  H-31 hard-filter decision (conflict flag).

================================================================================
CROSS-REPO SYNTHESIS (after all 5; HarnessBridge still blocked)
================================================================================
HEADLINE: none of the 4 reachable repos yields a STRONG, plan-headline NET-NEW feature. ELAI's
  feature map (families A-J, 1746 lines) already covers, has-planned, or has-explicitly-rejected
  essentially everything these repos do. This matches the recent steal-run pattern (Section 4):
  mature steals now produce mostly CONFIRMATIONS + weak riders, not new subsystems. That is the
  grounded answer, not a failure to find a steal.

CONVERGENT VALIDATIONS (independent projects arriving at ELAI's own choices — grounding, not work):
  - pi: git-safety doctrine, session-DAG+fork, tool-call block-seam-as-gate, iterative compaction
    = all already ELAI. Talk headline "make it prove it" (SHA of test output) = ELAI F-4 cant-lie.
  - SpatialClaw: 0-1000 coord schema = exact match to planned J-8; "2 independent evidence lines,
    pixel-motion banned" = ELAI D-3 >=2-verifier doctrine.
  - EvoArena: PASS_TO_PASS regression = user's #1 keystone J-16; chain-acc = pass^k I-38.

CANDIDATE RIDERS (all genuinely net-new, all weak-to-medium, NO shared subsystem):
  R1 [risk-under-pressure] compute/cost-normalized robustness metric (C@tau/CAURC) + 8-template
     jailbreak probe battery + LLMJudge rubric -> SAFEDELTA G-6 / J-20 / F-3-E-37. (security-eval)
  R2 [SpatialClaw] named config-ablatable prompt-section registry, tools self-declare sections,
     A/B-measurable -> I-32 / I-48. (prompt-mgmt; NC license = idea-only)
  R3 [EvoArena] typed preference-transition taxonomy + typed conflict-resolution -> H-31/E-5/J-21;
     env-compatibility-tagged recall validation -> H-32/B-21. (memory)

RECOMMENDATION (honest, anti-manufacture): R1/R2/R3 are three weak-medium refinements in three
  DIFFERENT subsystems; none alone clears the bar for a headline plan; bundling = grab-bag. Options:
    OPTION A — author ONE small new plan track "external-steal riders 2026-06-14" with 3 vertical-
      slice units (R1/R2/R3), each extending its named subsystem. Cheap, keeps map honest.
    OPTION B — record R1/R2/R3 as one-line rider notes on the relevant existing planned units in
      the feature-map ledger (GD-1 "riders recorded, not re-authored"), author NO new plan now.
  Lean: OPTION B for R2/R3 (minor); R1 -> a plan only if the user wants a robustness-MEASUREMENT
  axis (most substantive, partly off-wedge). Either way = NEW plan/ledger action, never an edit to
  existing plan bodies (user never-amend rule).

OUTSTANDING: HarnessBridge (repo 2) unresolved — owner "mandyyyii" 404s; need corrected URL to
  complete the 5th data point before finalizing the new-plan decision.
