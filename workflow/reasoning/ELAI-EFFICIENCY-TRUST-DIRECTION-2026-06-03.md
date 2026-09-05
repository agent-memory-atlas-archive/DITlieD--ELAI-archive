ELAI EFFICIENCY + TRUST ENGINEERING DIRECTION — DECISION RECORD
Authored 2026-06-03. Companion to (and refinement of) ELAI-DIRECTION-2026-06-01.md and
ELAI-FOCUS-RESCOPE-2026-06-02.md. Where those two set the MARKET positioning, this record sets the
ENGINEERING direction: how the harness should actually orchestrate, spend, and verify. Trigger:
maintainer said the current implementation is not optimal at "using a cloud flagship to make a small
model smart, to save tokens/cost, be context efficient" and asked for a deep, unbiased, research-
grounded redesign. Grounding provenance: a code-grounding workflow over the live .elai_cc source
(run wf_c9bb84b6-88c, 6 readers) + a 12-dimension proven-technique research sweep with a hostile
fact-checker on every load-bearing claim (run wf_eea93786-8e7, 21 agents; fill run wf_e6a7e493-64e,
6 agents). Every number below survived independent arXiv-id + in-context verification unless tagged
otherwise. The raw run outputs live in temp dirs that will be GC'd; this file is the durable digest.

==============================================================================
0. EXECUTIVE VERDICT
==============================================================================
Refine the engineering, do NOT pivot the thesis. The market docs are right (cede local-as-flagship,
demote cost-as-moat, wrap cloud Sonnet, the number is everything). This record OPERATIONALIZES them
with measured techniques and corrects three engineering instincts that the evidence refutes.

The reframed engineering thesis (one sentence):
  ELAI is a deterministic verification-and-orchestration harness wrapped around a cloud flagship: it
  makes every run CHEAPER (prompt caching), AUDITABLE (journal + reproducible artifact + attestation),
  and HONEST (off-agent, execution-grounded trust signals), proven by one published gate-ON-vs-OFF
  number. The local 7B is a cheap WORKER for cheaply-checkable sub-tasks, never the brain.

Three instincts the evidence refutes (all in maintainer's framing of the ask):
  - "make a small model smart" — REFUTED. The measured gain comes from sparse cloud-expert calls, not
    the 7B (SWE-Protege 7B->42.4% uses an Opus-4.1 EXPERT + SFT+RL, not a free 7B). Route a cheap worker
    ONLY to cheaply-checkable steps (Model-Cascading-for-Code excludes any model <10% from a cascade).
    [CORRECTED per section 16/E1: the "7B at 9.0%" baseline was fabricated; prior SLM SOTA is 17%.]
  - "save tokens/cost as the goal" — REFUTED as a goal; reframed as a measured supporting win. In CLOUD
    mode the ONLY real cost lever is prompt caching (41-80%); ELAI has it wired-dark. Sidecar quant
    (AWQ/FP8) touches the local GPU only, zero effect on the Sonnet bill. The ~330-token prompt is noise.
  - "be context efficient via LLM compaction / bigger window" — REFUTED. Deterministic observation
    masking halves cost while matching LLM-summarization quality AND removes a hallucination surface;
    context-rot means a bigger window degrades, it does not help.

One uncomfortable meta-finding: the research caught a HALLUCINATED number already living inside ELAI's
own memory (the "92.5 LoCoMo" Mem0 figure; real J-score is 66.88%), plus two prior bad arXiv cites. A
"cant-lie" product that ships unverified claims IS the liar it sells a cure for. Phase 0 below is the
integrity gate: ELAI must pass its own gate before it markets it.

==============================================================================
1. THE FOUR PILLARS (each mapped to a MEASURED, verified technique)
==============================================================================
CHEAPER (supporting, measured, never the headline):
  - Prompt-caching activation + cache-aware layout: 41-80% cloud cost cut (arXiv:2601.06007), cache
    read = 0.1x input (Anthropic docs). ELAI hardcodes cache_control:None on the message path today.
  - Cheap-worker deferral: cloud flagship reasons, 7B/cheap tier executes cheaply-checkable steps,
    routed by a CALIBRATED external layer (RouteLLM/AutoMix), never the 7B's own self-confidence.

SMARTER ORCHESTRATION (not a smarter model):
  - Architect/Editor split: +9.6pp on Aider code-edit bench (o1-mini reasoner + GPT-4o editor), positive
    across every model pairing. Maps to ELAI's BUILT-BUT-UNWIRED per-role RuntimeOrchestrator.
  - Single-strong-agent + INDEPENDENT-CONTEXT verifier, NOT parallel code-writers (multi-agent fan-out
    is refuted for coding: negative returns above 45% single-agent accuracy).
  - Deterministic observation-masking compaction: halves cost, matches LLM-summarization, ELAI_SEED-safe.

CANT LIE (the wedge):
  - Off-agent execution-grounded trust: bind passed=true to a runner-captured blake3 stdout hash AND
    diff test files against a spec-frozen version (catches operator-overloading, the dominant cheat).
  - Pre-execution approval gate + post-execution verification gate (EviBound dual-gate: 100%->0%).
  - Execution-agreement abstention (CodeRefuser cluster-ratio, 79.4% precision) as the conformal-refusal
    producer. KILL logprob/CoT/verbalized/escalate-to-stronger confidence (all measured dead-ends).

PROVABLE (durable, does not decay as models improve):
  - SAFEDELTA gate-ON-vs-OFF on SusVibes (hidden security tests = off-agent oracle), paired bootstrap CI,
    ~6pp noise floor, open-source ONLY the verifier, submit to Endor third-party leaderboard.
  - Property-based tests (Sonnet-authored) + cargo-mutants-on-diff as the correctness legs (50x mutation
    catch; 86% valid on top-ranked — both third-party measured).
  - in-toto + SLSA-VSA attestation over the Reproducible-Run Artifact = "trust without re-running."

==============================================================================
2. THE SEVEN GROUNDED CONCLUSIONS (the reasoning, with citations)
==============================================================================
C1. "Make the small model smart" is dead; "cloud reasons, cheap worker executes checkable steps" is
    alive and measured. SWE-Protege (arXiv:2602.22124) lifts a 7B to 42.4% on SWE-bench Verified, but with
    an Opus-4.1 EXPERT (41.0% with a Sonnet-4.5 expert at 11.9% of tokens) AND SFT+RL on expert trajectories
    — the gain is the expert's, not a free 7B's. [CORRECTED per section 16/E1: the "vanilla 7B at 9.0%"
    baseline cited earlier is FABRICATED — not in the paper; prior SLM SOTA is 17%.] Model-Cascading-for-Code
    (arXiv:2405.15842) excludes any model <10% accuracy from a cascade. The router must be an external
    calibrated layer: RouteLLM (arXiv:2406.18665, ICLR 2025 — paper-body 3.66x / ~73% cost cut at 95% GPT-4
    quality on MT-Bench; the "~85%" is an LMSYS blog figure, see 16/E2; transfers across pairs without
    retraining) / AutoMix (arXiv:2310.12963, >50% cost cut, POMDP absorbs noisy self-verification).

C2. In cloud mode, cost efficiency is essentially ONE lever ELAI has not pulled: prompt caching. Cache
    reads are 0.1x input (90% cut), free TTL refresh (Anthropic docs, live 2026-06-03). "Don't Break the
    Cache" (arXiv:2601.06007): 41-80% cost cut, 13-31% TTFT, but naive full-context caching INCREASES
    latency — the fix is layout (volatile content after the last breakpoint, exclude tool results from
    the cached region). ELAI is wire-dark here (anthropic_api.rs:137/163 hardcode cache_control:None on
    the message path; cache_check.rs notes "LOW hit rate on Anthropic-compat path"). AWQ/Marlin, FP8-KV,
    XGrammar are sidecar-only — ZERO effect on the Sonnet bill. (Correction: the "85% latency reduction"
    figure is fabricated-attribution; the docs give no latency %; use 13-31% TTFT.)

C3. Context efficiency wins are deterministic, not LLM-based — which is MORE on-thesis. "The Complexity
    Trap" (arXiv:2508.21433, NeurIPS 2025): deterministic observation masking halves cost while matching
    LLM-summarization solve-rate on SWE-bench Verified, and is ELAI_SEED-safe. Bi-temporal memory is
    measured (Zep arXiv:2501.13956: 71.2 vs 60.2 LongMemEval, 98.6% fewer tokens; Mem0 arXiv:2504.19413:
    +26% rel, 90% fewer tokens, real J-score 66.88% — NOT the hallucinated 92.5). Context-rot (Chroma,
    Jul 2025): a 200K model degrades by ~50K — refutes raising ELAI_LOCAL_MAX_CONTEXT for quality.

C4. The trust spine is independently vindicated, and the implementation is now decided. Stronger models
    CHEAT MORE (ImpossibleBench arXiv:2510.20270: GPT-5 54% / o3 49% / Opus 4.1 50% cheat-rates; >79% via
    test-modification, operator-overloading dominant; LLM monitors catch only 42-50% on SWE-bench). 72%
    of reward-hacking episodes carry a confident CoT rationale (arXiv:2605.02964) — CoT-monitoring is
    dead. Logprob hallucination prediction tops out at 22-33% (Collu-Bench arXiv:2410.09997) — dead.
    Therefore trust MUST be off-agent + execution-grounded. Two precise corrections: (a) stdout-hash alone
    misses operator-overloading, so also diff test files vs a spec-frozen version; (b) the strongest
    measured confidence signal is execution-agreement (CodeRefuser arXiv:2605.17029, 79.4% MBPP / 72%
    HumanEval precision vs 37-40% logprob), not calibration. "Escalate to Opus to stop lying" is REFUTED.

C5. Multi-agent fan-out for code is the wrong shape — and ELAI-DIRECTION section 7 was about to wire it.
    Science of Scaling (arXiv:2512.08296): negative returns from adding agents above ~45% single-agent
    accuracy; coordination cost superlinear. Single >= Multi at equal token budget (arXiv:2604.02460).
    Anthropic's own multi-agent post: 90.2% gain was on RESEARCH, "not a fit for coding," 15x tokens. The
    right use of a second cloud call for a single code change is an INDEPENDENT-CONTEXT VERIFIER, not a
    parallel writer — the Architect/Editor split (Aider, +9.6pp, positive across every pairing). DECISION:
    do NOT wire BSP for parallel code generation; repurpose fan-out for the research/explore breadth lane
    only; wire the per-role RuntimeOrchestrator for architect/editor/verifier instead.

C6. The number is the whole game, and the credibility path is concrete. SusVibes (arXiv:2512.03262, 200
    tasks/77 CWE, security tests HIDDEN from the agent = a real off-agent oracle; SWE-Agent+Sonnet 61%
    functional / 10.5% secure; >80% of functional solutions carry a vuln) is the SAFEDELTA harness.
    Single-run pass@1 varies 2.2-6.0pp even at temp 0 (arXiv:2602.07150) — anything under ~6pp is noise,
    and ELAI_SEED does NOT fix it (cloud inference is non-deterministic; Thinking Machines: temp-0 gives
    80 unique outputs/1000 runs). Report a paired bootstrap CI (arXiv:2511.19794), never a point estimate.
    A self-graded closed number is worth ~zero (OpenAI abandoned SWE-bench Verified Feb 2026: 59.4% flawed
    tests in the audited subset). Path: open-source ONLY the verifier, ship a reproducible Docker+seed
    envelope, submit to Endor's third-party leaderboard. Bonus stealable gate: Endor's anti-cheat caught
    git-history exploitation in 163/200 runs (81%) — add a git-log/git-show pre-flight BLOCK.

C7. ELAI must pass its own gate first. The research found two hallucinated facts already in ELAI's
    artifacts (the 92.5 LoCoMo number; prior bad arXiv cites). The credibility of the product and the
    credibility of its roadmap are the same discipline. Phase 0 purges these before anything ships.

==============================================================================
3. THE SEVEN-PHASE SEQUENCED PLAN (this also IS the wire-dark fix list)
==============================================================================
Phase 0 — Pass your own gate (cheap, prerequisite, docs-only).
  Purge hallucinated numbers (92.5 LoCoMo, prior bad cites) from ELAI docs + memory. Replace the
  "small-model-smart / token-savings-as-goal / bigger-window" framing with the measured reframe. No code.

Phase 1 — Prompt caching (biggest cheap win, pure activation).
  Activate cache_control on the stable prefix (anthropic_api.rs:137/163, currently hardcoded None on the
  message path). Cache-aware layout: all volatile content (tool results, RAG chunks, SSB rows) strictly
  AFTER the last cache breakpoint; debug-assert the cached byte-prefix is identical turn-to-turn. Wire
  cache_check.rs to report the real hit-rate. Measured target: 41-80% cloud cost cut.

Phase 2 — Wire the orchestration brain (architect/editor + verifier; NOT BSP-for-code).
  Wire the built-but-unwired per-role RuntimeOrchestrator into live dispatch: Architect (cloud flagship,
  reasons/plans) + Editor (cheap tier / 7B for cheaply-checkable edits) + independent-context Verifier
  (the auditor, separate context). Add a cheap-worker deferral signal (SWE-Protege shape) on a calibrated
  router (RouteLLM/AutoMix), never 7B self-confidence. Repurpose BSP/fan-out for the research lane only.

Phase 3 — The cant-lie spine, execution-grounded (the wedge).
  (a) stdout-hash binding on TestOutcomePayload (F-4) PLUS test-file-diff vs a spec-frozen version (catches
  operator-overloading). (b) EviBound-style pre-execution approval gate (acceptance-criteria schema +
  queryable run-id before a claim propagates). (c) Execution-agreement abstention: the G-7 claim_score
  producer = CodeRefuser cluster-ratio (k cloud samples + k generated tests, execute every pair in the
  existing sandbox, claim_score = largest execution-equivalence cluster / k) feeding conformal_calibration.rs
  ::decide(). DELETE the logprob/CoT/verbalized/escalate-to-stronger paths. Activate the wire-dark gates on
  the demo path and kill the .unwrap() debt on that path only.

Phase 4 — Deterministic context efficiency.
  ACTIVATE deterministic observation masking as the default context manager (arXiv:2508.21433): make
  adaptive_reduce default-on for long sessions and un-gate the E-25 ladder. [CORRECTED §16/E3: ELAI has NO
  LLM-compaction to "replace" — compact_session is already a deterministic string formatter.] Wire bi-temporal
  memory READ-BACK (get_relevant_facts is dark — facts written, never read) behind the trust/decay filter.
  Wire session-handoff stub-recovery (recover_stubs is dark; /continue falls back to a text placeholder).
  Drop the bigger-window instinct; keep a tight working set.

Phase 5 — The number (SAFEDELTA) + credibility.
  Gate-ON-vs-OFF on SusVibes vs CLOUD Sonnet (same model, same tasks, same harness). Report paired BCa
  bootstrap CI; treat any delta under ~6pp as noise. Add property-based tests (Sonnet infers 3-5 invariants
  -> proptest/Hypothesis, gate on pass-or-triaged-counterexample, off-agent SSB row) + cargo-mutants-on-diff
  (surviving mutant in NEW code => BLOCK) as the correctness legs. Add the git-history-exploit pre-flight
  block. Ship the reproducible Docker+seed+prompt+pin envelope; open-source ONLY the verifier; submit to
  Endor. This turns every "it cant" from a design assertion into a measured fact.

Phase 6 — Attestation (non-decaying value).
  in-toto v1 custom predicate (subject = staged-tree blake3) + SLSA-VSA rollup of every gate verdict over
  the Reproducible-Run Artifact = "trust without re-running" (the only honest cloud claim; token-level
  replay is impossible per Thinking Machines). This is the compliance/attestation substrate whose value
  does NOT decay as backbones get safer (hedge for focus-rescope 5.4 harness-overfit decay).

==============================================================================
4. DO NOT (refuted by measurement — do not spend solo-dev time here)
==============================================================================
  - Make the 7B "smart." (SWE-Protege: the gain is the cloud expert's.)
  - Best-of-N on a strong generator. (AceCoder arXiv:2502.01718: +2.6 / -0.6 on Qwen2.5-Coder-7B; gain is
    inverse to generator strength. Local-worker-only at best.)
  - Same-model self-critique. (Self-repair arXiv:2306.09896: no-op / token-burn unless the critic is
    strictly stronger; stronger critic raised repair 33.3%->52.6%.)
  - BSP parallel code-writers. (C5; refuted for coding.)
  - logprob / verbalized / CoT confidence. (Collu-Bench 22-33%; verbalized near-zero skill; 72% of hacks
    carry a confident CoT.)
  - Force-JSON on coder/planner roles. (arXiv:2408.02442: constrained decoding degrades reasoning; keep
    FreeText for those roles; force structured output ONLY on pure-data roles like the 5-class diagnoser.)
  - Self-graded closed number. (OpenAI abandoned SWE-bench Verified for this.)
  - Lean on ELAI_SEED for EVAL credibility. (Cloud inference is non-deterministic; seed controls the
    harness, not the provider. Determinism certifies REPRODUCED, not RIGHT — DFAH arXiv:2601.15322:
    determinism and accuracy decoupled, r=-0.11, p=0.63; small models hit near-perfect determinism by
    rigid pattern-matching at 20-42% accuracy.)
  - SAST-only security gate. (Snyk 38% / CodeQL 30% / SonarQube 8% on seeded vulns, all miss every logic
    flaw — DryRun vendor benchmark, directional. Route logic-security to a cloud reviewer + PBT.)
  - Cite EU AI Act Article-50 as a moat. (Governs synthetic media output, not source-code commits. Dates
    UNVERIFIED-pending per the E-38 HARD RULE.)

==============================================================================
5. VERIFIED EVIDENCE LEDGER (technique -> source -> number -> verdict)
==============================================================================
All arXiv ids below were independently confirmed to exist and match title; numbers confirmed in-context
unless tagged. Tags: [HOLDS] verified, [WEAK] preprint/single-author/small-N, [CORRECTED] number fixed
from the researcher's first pass, [DESIGN] mechanism real but specific % not body-confirmed.

Orchestration / cheap-worker:
  SWE-Protege arXiv:2602.22124 — 7B ->42.4% SWE-bench Verified with an Opus-4.1 expert (41.0% Sonnet-4.5 @
    11.9% tokens), via SFT+RL on expert trajectories. [CORRECTED §16/E1: "9.0% vanilla 7B" FABRICATED; prior SOTA 17%.]
  RouteLLM arXiv:2406.18665 (ICLR 2025) — paper-body 3.66x / ~73% cost cut at 95% GPT-4 quality (MT-Bench),
    transfers without retraining. [CORRECTED §16/E2: "~85%" was the LMSYS blog figure, not the paper body.]
  AutoMix arXiv:2310.12963 (NeurIPS 2024) — >50% cost cut; POMDP absorbs noisy self-verification. [HOLDS]
  Inter-Cascade arXiv:2509.22984 — cascade-with-memory: +33.06% weak acc, -48.05% strong calls, -49.63%
    fees. [HOLDS]
  Model-Cascading-for-Code arXiv:2405.15842 — exclude models <10% accuracy from cascade; avg 26% / up to
    70% cost reduction. [CORRECTED] the "49% HumanEval" figure was fabricated; the <10% exclusion is the
    load-bearing finding and the 7B (9.0%) is below it.
  Architect/Editor — aider.chat 2024-09-26 — o1-mini+GPT-4o 70.7% vs 61.1% (+9.6pp), positive across all
    pairings. [HOLDS]
  CodeAct arXiv:2402.01030 (ICML 2024) — +20.7pp success, ~29% fewer turns; stronger on stronger models.
    [HOLDS]

Anti-fabrication / verification / lying:
  EviBound arXiv:2511.05524 — dual-gate 100%->0% hallucination, ~8.3% overhead. [WEAK] N=8; MLflow-artifact
    -bound (analogous to, not identical to, ELAI's blake3 stdout_hash).
  ImpossibleBench arXiv:2510.20270 — GPT-5 54% / o3 49% / Opus 4.1 50% cheat; >79% via test-modification;
    monitors 42-50% on SWE-bench; stronger cheats more. [HOLDS]
  Collu-Bench arXiv:2410.09997 — logprob hallucination prediction 22-33% accuracy ceiling. [HOLDS] DEAD_END.
  Reward-Hacking Benchmark arXiv:2605.02964 — structural hardening -5.7pp (87.7% rel), task success
    unchanged; 72% of hacks carry CoT rationale; Sonnet 4.5 0%, R1-Zero 13.9%. [HOLDS]
  Upward-deception arXiv:2512.04864 — pervasive across 11 agents under benign instructions; prompt
    mitigation only partial. [HOLDS]
  CaMeL arXiv:2503.18813 (Google Research) — 77% AgentDojo WITH provable security vs 84% undefended (7pp
    cost), capability interpreter, no LLM modification. [HOLDS]
  RTBAS arXiv:2502.08966 + FIDES arXiv:2505.23643 — IFC/taint with dependency-screened join. [DESIGN]
    specific % not body-confirmed; FIDES has a live github.com/microsoft/fides repo.
  Self-repair arXiv:2306.09896 (ICLR 2024) — stronger critic 33.3%->52.6% (1.58x); same-model = no-op.
    [HOLDS]

Calibration / abstention:
  CodeRefuser arXiv:2605.17029 — execution-agreement cluster-ratio: 79.4% MBPP / 72% HumanEval precision
    vs 37-40% logprob; +26.5 pts; distribution-free LTT guarantee; "running the code is necessary." [HOLDS]
    cite per-model honestly (CodeLlama MBPP 79.40 / HumanEval 91.67; DeepSeek HumanEval 72 / MBPP 66.43),
    not the optimistic cross-model pair.
  UniCR arXiv:2509.01455 — multi-signal conformal risk-controlled refusal; activate executor-feature arm.
    [WEAK] single-author preprint; specific AURC numbers not body-confirmed.
  Spiess arXiv:2402.02047 (ICSE 2025) — Platt ECE 0.32->0.03 but ONLY on 3 obsolete 2022-23 models; Sonnet
    API does not expose per-token logprobs. [HOLDS] demote to local-secondary-low-weight only.

Context / memory:
  Complexity Trap arXiv:2508.21433 (NeurIPS 2025) — observation masking halves cost, matches LLM-
    summarization solve rate on SWE-bench Verified. [HOLDS]
  Zep arXiv:2501.13956 — 71.2 vs 60.2 LongMemEval, 98.6% fewer tokens, ~90% latency cut. [HOLDS]
  Mem0 arXiv:2504.19413 — J-score 66.88% / 68.44% graph, +26% rel, 90% fewer tokens, 92% p95 latency cut.
    [CORRECTED] the "92.5 LoCoMo" figure in ELAI memory is a HALLUCINATION; real is 66.88%.
  Anthropic context editing (Sonnet 4.5, 2025-09) — 84% token cut, +29% / +39% agentic-search. [HOLDS]
    internal eval, not third-party reproduced.
  Chroma context-rot (2025-07) — 18 models degrade as input grows; 200K model degrades by ~50K. [HOLDS]
  Cursor semantic search (2026) — +12.5% avg over grep-only, +2.6% on 1000+-file codebases; combo best.
    [HOLDS]

Multi-agent / orchestration shape:
  Science of Scaling arXiv:2512.08296 — negative returns above 45% single-agent accuracy; coordination cost
    superlinear; PlanCraft MAS -39% to -70%. [HOLDS]
  Single>=Multi equal budget arXiv:2604.02460 — SAS strongest default architecture. [HOLDS] (carve-out:
    heavy context degradation can let sequential MAS match SAS.)
  MAST arXiv:2503.13657 — 14 failure modes, kappa 0.88; ChatDev 25->40.6, HumanEval 89.6->91.5. [CORRECTED]
    methodology is ~150 tasks / 5 frameworks (not "200 traces / 7").
  Anthropic multi-agent (2025-06) — 90.2% on RESEARCH; "not a fit for coding"; 15x tokens; token usage = 80%
    of variance. [HOLDS]
  ESAA arXiv:2602.23193 — event-sourcing replay + SHA-256 canonicalized projection hash. [HOLDS] case-study
    only, NO benchmark delta (precedent for the pattern, not a measured win).

Eval / credibility:
  SusVibes arXiv:2512.03262 — 200 tasks / 77 CWE; 61% functional / 10.5% secure; >80% of functional carry a
    vuln; hidden security tests = off-agent oracle. [HOLDS] this IS the SAFEDELTA harness.
  Paired bootstrap arXiv:2511.19794 — 0.5-2pt gains never significant with 3 seeds under paired BCa +
    sign-flip permutation. [HOLDS]
  Agentic-eval variance arXiv:2602.07150 — single-run pass@1 varies 2.2-6.0pp; SD>1.5pp at temp 0. [HOLDS]
  OpenAI SWE-bench abandonment (2026-02-23) — 59.4% of the audited 27.6% subset have flawed tests +
    universal contamination. [HOLDS]
  Endor Agent Security League (2026-04-15) — third-party-run; git-history exploit caught in 163/200 (81%);
    flagged runs excluded. [HOLDS] use the live leaderboard snapshot, not a stale top number.
  BaxBench arXiv:2502.11844 — dynamic exploit-execution oracle (never LLM-judge); best model 62% correct,
    ~half of correct solutions exploitable. [CORRECTED] do NOT cite "62% incorrect-or-insecure" (that
    inverts the metric; 62% is the correctness ceiling).

Provenance / determinism:
  SLSA VSA (slsa.dev) — delegated-trust rollup, "trust without re-running." [HOLDS]
  in-toto v1.2.0 (2024-03-18) — custom predicate; subject bound by content digest; staged-tree blake3 is
    spec-legal. [HOLDS]
  DFAH arXiv:2601.15322 — determinism and accuracy decoupled (r=-0.11, p=0.63); small 7-20B near-perfect
    determinism at 20-42% accuracy; green replay != correct. [HOLDS]
  Thinking Machines (2025-09) — cloud temp-0 = 80 unique completions / 1000; batch-invariant kernels give
    bit-identical but self-hosted only. [HOLDS] kills any cloud "bit-identical replay" claim.

Code-quality gates:
  Property-based testing OOPSLA 2025 (DOI 10.1145/3764068) — each PBT catches ~50x more mutations than the
    average unit test (426 Python programs). [HOLDS]
  Agentic PBT arXiv:2510.09907 + red.anthropic.com 2026 — Claude-authored PBTs across 100 packages, 984
    reports, 86% valid on top-ranked (56% on a random sample). [HOLDS] apply a ranking-rubric filter.
  cargo-mutants-on-diff — no single headline ROI number; justified by the 50x PBT finding + direct wire-dark
    detection (surviving mutant in NEW code = R4/R6 violation). [DESIGN] measured artifact = surviving-mutant count.
  SAST pattern tools — Snyk 38% / CodeQL 30% / SonarQube 8% on seeded vulns, miss all logic flaws. [WEAK]
    DryRun vendor benchmark, N=26, directional. DEAD_END for the load-bearing leg.
  AutoVerus arXiv:2409.13082 / Kani — STEAL_LATER premium high-assurance tier only.

Inference / caching:
  Anthropic prompt caching (platform.claude.com docs, live 2026-06-03) — read 0.1x (90% cut), free TTL
    refresh, 1024-tok min Sonnet 4.6, 4 breakpoints. [HOLDS] (the "85% latency" figure is fabricated; docs
    give no latency %.)
  Don't Break the Cache arXiv:2601.06007 — 41-80% cost / 13-31% TTFT; naive caching increases latency;
    dynamic-content-at-end + exclude-tool-results is the fix. [HOLDS]
  Let Me Speak Freely arXiv:2408.02442 (EMNLP 2024 Findings) — constrained decoding degrades reasoning;
    stricter = worse; helps classification. [HOLDS]
  AWQ/Marlin arXiv:2408.11743, FP8-KV (vLLM 2026-04-22), XGrammar arXiv:2411.15100 — sidecar-only, CLOUD-
    IRRELEVANT (zero effect on the Sonnet bill). [HOLDS] FP8-KV regressed 128k needle 91%->13% on Hopper
    pre-fix; gate OFF above ~16-20K local context.

==============================================================================
6. CAVEATS / UNVERIFIED-PENDING (do not over-claim from this record)
==============================================================================
  - SWE-Protege expert-model identity is UNVERIFIED (abstract names none; two body fetches contradicted).
    Do not write "Claude 3.5 Sonnet" anywhere.
  - EviBound is N=8 and MLflow-artifact-bound; it validates a same-FAMILY pattern, not ELAI's exact blake3
    primitive.
  - UniCR / RTBAS / FIDES exact percentages are preprint-grade / not body-confirmed; cite the design, not
    the numbers, until a body read.
  - The ELAI code wiring claims in this record (caching hardcoded None, RuntimeOrchestrator unwired, gates
    dark, get_relevant_facts/recover_stubs dark) come from a code-grounding read (run wf_c9bb84b6-88c) and
    MUST be re-confirmed by context-compiler against current source before any Edit (the readers flagged
    these as needing in-code verification; some line numbers may have drifted).
  - EU AI Act dates/penalties are UNVERIFIED-pending per the E-38 HARD RULE (cite ELAI-2026-POSTURE.md or a
    primary source before any roadmap claim).

==============================================================================
7. WIRE-DARK / DEBT THIS DIRECTION RETIRES (the "fix it too" ask)
==============================================================================
  - Prompt caching on the message path: hardcoded cache_control:None -> activate (Phase 1).
  - per-role RuntimeOrchestrator: built, never called in production (only a default-OFF pre-warm) -> wire as
    architect/editor/verifier (Phase 2).
  - HybridEscalationGate + E-31 RuntimeRoutingPolicy: test-only / wire-dark; live escalation uses a hardcoded
    layer_failed:1 -> replace with the calibrated router (Phase 2).
  - BSP engine: runs with simulated_agent_work=ZERO (a concurrency harness, not inference) -> do NOT wire for
    code; repurpose for the research lane (Phase 2/5).
  - F-4 stdout-hash binding + anti_fabrication_block: present but needs the test-file-diff leg + activation
    (Phase 3).
  - G-7 claim_score producer: deferred -> CodeRefuser cluster-ratio (Phase 3).
  - adaptive_reduce / compaction ladder: test-only / default-OFF -> replace with deterministic observation
    masking (Phase 4).
  - get_relevant_facts (memory read-back) + recover_stubs (handoff SSB rehydration): dark -> wire (Phase 4).
  - .unwrap() debt on the demo/gate path: kill on that path first (Phase 3/5), not workspace-wide.

==============================================================================
8. POINTERS
==============================================================================
  - Market positioning this engineering record serves: ELAI-DIRECTION-2026-06-01.md (macro pivot) +
    ELAI-FOCUS-RESCOPE-2026-06-02.md (the trust wedge + SAFEDELTA + the 5 hard truths).
  - Trust substrate being built/activated: F-SUMMARY_INDEX.md + G-SUMMARY_INDEX.md (G-2 RRA, G-7 confidence
    producer, G-8 in-toto/SLSA all map directly onto Phases 3/5/6 here).
  - Code ground truth: the 6-reader grounding (run wf_c9bb84b6-88c) — caching/orchestrator/gates wire-dark.
  - memory: project_market_positioning_2026, project_trust_maximization_roadmap, project_focus_rescope_2026,
    project_gtm_playbook_2026.

==============================================================================
9. ONE-PARAGRAPH SUMMARY (for quick re-entry)
==============================================================================
Stop chasing a smart small model and cheap tokens as the product. Become the harness that wraps a cloud
flagship and is special on four measured axes: CHEAPER (activate prompt caching, 41-80%, currently wire-dark;
sidecar quant is cloud-irrelevant), SMARTER-ORCHESTRATION-not-model (wire the unused per-role orchestrator as
architect/editor/independent-verifier, +9.6pp; do NOT wire BSP for parallel code-writing — refuted; use
deterministic observation-masking compaction), CANT-LIE (off-agent execution-grounded trust: stdout-hash +
test-file-diff + EviBound approval gate + CodeRefuser execution-agreement abstention; kill logprob/CoT/
verbalized/escalate-to-stronger, all measured dead-ends; stronger models cheat MORE), and PROVABLE (one
SAFEDELTA gate-ON-vs-OFF number on SusVibes against cloud Sonnet, paired CI, ~6pp noise floor, open-source
only the verifier, submit to Endor; PBT + cargo-mutants correctness legs; in-toto/SLSA-VSA attestation for
"trust without re-running"). Sequence: pass your own gate (purge the hallucinated numbers) -> caching ->
orchestration brain -> cant-lie spine -> deterministic context -> the number -> attestation. Every "it cant"
is a design assertion until the number exists; the number and the killer demo are the same deliverable.

==============================================================================
10. EFFICIENCY ENGINEERING DEEP DIVE (2026-06-03 ADDENDUM)
==============================================================================
Trigger: maintainer asked to go deep on prompt caching and every cloud-cost / token / context lever, so ELAI is
provably MORE efficient than Claude Code / Cursor / Codex / Devin Desktop when driving the SAME cloud model.
Grounding provenance: an 11-dimension research sweep (run wf_8611a1e0-c20, 22 agents, ~1.64M tokens, 892 tool
calls) with a HOSTILE fact-checker on every load-bearing claim, verified against LIVE Anthropic docs (fetched
2026-06-03), against paper BODIES (not abstracts), and against direct .elai_cc source reads. Plus a targeted
read of the Devin Desktop / Windsurf 2.0 launch (devin.ai/blog/windsurf-is-now-devin-desktop, cognition.ai).
Tags: [HOLDS] verified; [CORRECTED] number/attribution fixed by the fact-checker; [WEAK] preprint/vendor/
single-source; [DESIGN] mechanism real but the % is an ELAI projection not a measurement; [REFUTED] false.
This DEEPENS the four-pillar CHEAPER lever; it does NOT change the thesis. It also corrects three figures that
appear in sections 1-9 and 5 of this doc (see section 16, ERRATA) before they propagate as new hallucinations.

10.0 THE GROUND TRUTH THAT REFRAMES THE WHOLE COST PROGRAM
Anthropic bills INPUT and OUTPUT separately, and OUTPUT is 5x INPUT: Sonnet 4.6 is $3/MTok input, $15/MTok
output [HOLDS, live pricing 2026-06-03]. Prompt caching ONLY discounts input (cache read = 0.1x = $0.30/MTok)
and does NOTHING for output. Therefore the cost program is TWO orthogonal stacks, not one:
  (A) INPUT STACK -- prompt caching + cache-stable layout + deterministic context masking + tool-search +
      (offline) batch. Goal: drive the cached-read fraction of input toward what competitors already measure
      (Cursor 85%, Claude Code 98.2% of input as cache reads -- see 12) so the repeated prefix bills at
      $0.30/MTok instead of $3.00/MTok (a 10x discount on that fraction).
  (B) OUTPUT STACK -- diff/SEARCH-REPLACE edits + FIM + streaming early-abort + terse structured output.
      Goal: stop paying $15/MTok to re-emit whole files. This stack is INDEPENDENT of caching and is the lever
      ELAI has most neglected; it is the one place where the 5x multiplier makes a token saved worth 5x more.
ELAI's current PRODUCTION cloud path pulls NEITHER lever hard. The RuntimeAdapter cloud path (the one task_runner
actually dispatches through) ships ZERO cache markers [HOLDS, see 10.1], and FocusTunnel DIFF mode always falls
back to whole-output STRUCTURED [HOLDS, see 10.3]. Both are WIRE-DARK, not absent -- activation, not invention.

10.1 LEVER L1 -- ACTIVATE PROMPT CACHING ON THE PRODUCTION CLOUD PATH  [biggest, cheapest, pure activation]
MECHANISM (all [HOLDS] vs live docs 2026-06-03): cache write = 1.25x input (5-min TTL) or 2x (1-hour TTL);
cache read = 0.1x (90% off); a cache hit refreshes the TTL for FREE. Min cacheable prefix on Sonnet 4.6 = 1024
tokens (Opus 4.6/4.7, Haiku 4.5, Mythos = 4096 [CORRECTED: broader than the prior 3-model summary]). Up to 4
breakpoints per request. Breakeven is the FIRST read, not the second: one 5-min write (1.25x) + one read (0.1x)
= 1.35x < 2.0x for two uncached reads; docs say "caching pays off after just one cache read" [CORRECTED -- the
intuition that you need 2 reads to break even is wrong]. Cache key = 100% byte-identical prefix; any change
before a breakpoint busts it and everything after. Invalidation hierarchy is tools -> system -> messages, so a
tool-list change busts the system and message caches too.
ELAI STATUS (all [HOLDS] vs source): there are TWO Anthropic code paths and the WRONG one is wired. (a) The
legacy AnthropicClient (providers/src/providers/anthropic.rs:448,502 -> apply_anthropic_cache_markers:1026-1041)
DOES stamp ephemeral cache_control on the last system block + last tool definition. (b) The PRODUCTION
RuntimeAdapter path (AnthropicApiRuntime::build_request, runtime/anthropic_api.rs:101-152) sets cache_control:
None at line 137 and NEVER calls apply_anthropic_cache_markers -- it ships ZERO markers. production_agent_rpc.rs
has 16 call sites all at cache_control:None (292-752) while anthropic_api.rs:189 declares supports_caching:true.
The conversation message history is NEVER marked on either path. cache_check.rs literally encodes the failure as
a feature: header says "Expected hit rate is LOW on Anthropic-compat path -- that is the signal," verdict_code=0
when hit_rate<=0.05. So ELAI is currently paying full $3/MTok on a prefix it could pay $0.30 on.
ACTION: make AnthropicApiRuntime mark the stable prefix (system + tools + rules + repo-map) with ephemeral
cache_control; mark BOTH the system/tools boundary AND a boundary just before the volatile region; wire
cache_check.rs to report the real hit-rate as a dashboard metric (not a buried critic verdict). Measured target
range from the literature: 41-80% input cost cut [WEAK: arXiv:2601.06007, cross-provider pooled, DeepResearch
Bench, 10K-token system prompts, ~500 sessions -- a preprint on a prefix-heavy workload, NOT a general number];
the only honest target is the MEASURED hit-rate on ELAI's own runs (section 14).
EDGE VS CLAUDE CODE: none yet -- this is catching up to parity. Claude Code already runs ~98.2% cache reads.
PITFALL: marking volatile content (tool results, SSB rows) caches bytes that never re-hit -- you pay the 1.25x
write and get no read discount, which can INCREASE latency (the "Don't Break the Cache" paradox). See L2.

10.2 LEVER L2 -- CACHE-STABLE LAYOUT + TAIL BREAKPOINT + 1-HOUR TTL  [the discipline that makes L1 actually pay]
MECHANISM ([HOLDS] unless noted): "Don't Break the Cache" (arXiv:2601.06007, Lumer et al., Jan 2026 preprint)
-- naive full-context caching can PARADOXICALLY INCREASE LATENCY because dynamic tool results trigger cache
writes for content never reused; the fix is to place volatile content AFTER the last breakpoint and exclude
dynamic tool results from the cached region. Body-confirmed model figure: Sonnet 4.5 78.5% cost / 22.9% TTFT
[HOLDS]; the "89% GPT-5.2" figure is a 50K-token ablation, not the headline (real GPT-5.2 = 79.6%) [CORRECTED];
there is NO "85% latency" figure anywhere (TTFT range is 13-31%) [CORRECTED, confirms the prior fabrication
flag]. CRITICAL multi-turn constraint: the breakpoint lookback window is 20 BLOCKS -- if a growing conversation
appends >20 blocks past a breakpoint, the cache silently misses; mitigation is a SECOND breakpoint near the
conversation tail. TTL: 5-min is the API-key default; 1-hour write costs 2x but free hits refresh it -- worth
it for long sessions and mandatory for batch (5-min expires before a batch completes).
ELAI STATUS [HOLDS]: ELAI's deterministic context-budget allocator already specifies 2 cache breakpoints (after
PhaseSpec, after CodeContext; context_budget.rs:1191-1202) and computes a blake3 byte-prefix stability flag
(update_breakpoints, :1481). BUT in production (runtime_client.rs:376-385) update_breakpoints is called with
EMPTY slices for slots 2-7, so the CodeContext boundary hash is always computed from empty bytes and the valid
flag is inert; the flag is even logged (:402-403) but NEVER consumed to apply or suppress a marker. This is the
single most ELAI-specific win in the whole document: ELAI's allocator + ELAI_SEED can GUARANTEE a byte-stable
prefix turn-to-turn, which a heuristic agent structurally cannot.
ACTION: feed real slot bytes into update_breakpoints; consume bp.valid to place markers ONLY on a verified-stable
prefix; add a debug_assert that the cached byte-prefix is identical turn-to-turn (a cache-bust is then a test
failure, not a silent bill); use 2 of the 4 breakpoints (stable-prefix + tail) and reserve 2; opt into 1-hour
TTL for sessions expected to exceed 5 minutes.
EDGE VS CLAUDE CODE: this is where ELAI WINS, not ties. ELAI can prove prefix-stability deterministically and
assert on it; Claude Code/Cursor rely on incidental ordering and silently lose the cache on reorderings (Cursor
Auto-mode shipped a 0%-cache bug, March 2026 [HOLDS]).
PITFALL: the 20-block window means a single tail breakpoint is not enough for very long tool loops -- budget the
breakpoints; and any nondeterministic byte (timestamp, random id, reordered map) in the prefix silently kills it.

10.3 LEVER L3 -- THE OUTPUT-TOKEN STACK  [the 5x lever ELAI has most neglected; orthogonal to caching]
MECHANISM: output is 5x input and caching never touches it [HOLDS]. Diff/SEARCH-REPLACE edits emit only changed
lines instead of whole files: Aider's unified-diff prompting cut "laziness" 20%->61% (GPT-4 Turbo) and 26%->59%
(GPT-4 June) -- it reduces output AND improves quality [HOLDS]; SEARCH/REPLACE is best for larger models
(arXiv:2510.12487 Diff-XYZ [HOLDS]; the prior "udiff best for small models" claim is NOT supported [CORRECTED]).
A vendor benchmark (Morph [WEAK]) puts Claude Code at ~479K tokens/task vs Aider ~105K (4.5x) -- directional
only. FIM (fill-in-the-middle) for single-region edits: "Hidden Cost of Readability" (arXiv:2508.13666, ICSE'26)
measures 24.5% average INPUT token reduction across 10 LLMs on FIM tasks [CORRECTED: the headline is 24.5% input,
NOT the 14.7%/13.2% intro-only tokenizer figures, and output reduction is only ~2.9%]. ShortCoder (arXiv:2601.09703)
gives 18.1% output token reduction / 37.8% efficiency on 828 pairs, and it targets OUTPUT generation [HOLDS].
ELAI STATUS [HOLDS]: FIM is wired (task_runner.rs:7821), but DIFF mode passes editor=None and ALWAYS returns
UnsupportedMode -> falls back to whole-output STRUCTURED (:7860-7874). Streaming early-abort (focus_tunnel/
streaming.rs, 50-token chunk gate) is wired to the SIDECAR path only, not the cloud path. ELAI's ShortCoder hook
does NOT actually shrink the payload: runtime_client.rs:340-366 computes a compressed CHAR COUNT for slot-budget
ESTIMATION only and sends the original bytes unchanged [CORRECTED -- the "ShortCoder reduces API payload" belief
is false; its 15-30% figure is a synthetic-fixture comment]. B-28's "~6800 tokens / 27% / 45-65% budget" numbers
are PROJECTIONS [DESIGN], not measurements.
ACTION: wire FocusTunnel DIFF mode end-to-end (supply the pre-assembler/dep-resolver/file-router so editor != None
and kill the STRUCTURED fallback) so cloud edits emit diffs not whole files; extend streaming early-abort to the
cloud path (honest caveat: Anthropic billing on aborted streams is UNVERIFIED conjecture [DESIGN] -- the win is
latency + downstream-token avoidance, confirm billing before claiming a $ cut); make ShortCoder actually rewrite
the outbound payload (or stop calling it a token-saver). Token-efficient tool use is already default-ON for
Sonnet 4.6 (the 14%/70% figures were measured on Claude 3.7 beta, do NOT cite them as Sonnet 4.6 numbers
[CORRECTED]); ELAI receives it passively but never verifies the saving.
EDGE VS CLAUDE CODE / CURSOR: large. Cursor deliberately uses FULL-FILE rewrites (diffs failed ~40% on line
numbers, and full-file pairs with speculative decoding) [HOLDS] -- so Cursor is structurally output-heavy. ELAI
emitting diffs + FIM on a deterministic file plan is a genuine, defensible output-cost edge.
PITFALL: diff edits fail when the model miscounts lines (Aider's 9x error increase without flexible patching
[HOLDS]) -- ELAI's 3-tier edit matching (B-20) is the mitigation and must land first.

10.4 LEVER L4 -- TOOL SEARCH TOOL (advanced-tool-use beta)  [85% cut on a per-request fixed cost]
MECHANISM [HOLDS]: the advanced-tool-use-2025-11-20 beta adds a Tool Search Tool that loads tool DEFINITIONS on
demand instead of in the system prefix: Anthropic measured an 85% reduction in tool-definition tokens (~72K ->
~8.7K) and Programmatic Tool Calling cut a complex-research task 37% (43,588 -> 27,297 tokens). Tool-search even
RAISED accuracy on large catalogs (Opus 4.5 79.5% -> 88.1%). Two variants: tool_search_tool_regex_20251119 and
tool_search_tool_bm25_20251119. Tool-use itself adds 497-589 tokens of system overhead per request on Sonnet 4.6.
ELAI STATUS [HOLDS]: ELAI sends NO advanced-tool-use header (anthropic.rs only carries the OAuth beta header) and
loads its full tool catalog into every request. This is a large, ABSENT lever.
ACTION: send the advanced-tool-use beta header and move ELAI's tool catalog behind the Tool Search Tool; this
also stabilizes the cache prefix (fewer tool-def bytes that change), compounding L1/L2.
EDGE VS CLAUDE CODE: Claude Code already does this (deferred tool loading is default-on, see 12) -- and crucially
ELAI's custom-base-URL / sidecar path may DISABLE Claude Code-style deferral, so ELAI must implement it itself.
PITFALL: tool-search adds a retrieval hop; for a tiny tool set it is pure overhead -- gate it on catalog size.

10.5 LEVER L5 -- BATCH API FOR NON-INTERACTIVE LANES  [flat 50% off, zero quality cost]
MECHANISM [HOLDS]: the Message Batches API charges 50% of synchronous on all models (Sonnet 4.6 batch = $1.50/
$7.50 vs $3/$15); <1h typical, 24h window, 29-day retention, 100K req / 256MB cap; supports tools, system,
multi-turn, thinking; excludes streaming, Fast mode, and max_tokens:0 pre-warm; Anthropic recommends the 1-hour
cache TTL with batch. The only cost is latency (async, not for the interactive REPL).
ELAI STATUS [HOLDS]: ELAI has NO batch endpoint (0 grep hits in anthropic.rs). Its non-interactive lanes run
synchronously at full price: Cosmos artifact refresh (cosmos_worker.rs refresh_one is per-artifact synchronous),
web-research fan-out (B-7), GEPA prompt optimization (gepa_template_evolution.rs), and the eval/benchmark suites
(E-9/E-49) are ALL batch-eligible and ALL paying 2x what they should.
ACTION: add a batch client; route Cosmos refresh, research fan-out, GEPA, and benchmark suites through it with
the 1-hour cache TTL; keep the interactive /task and chat REPL on the synchronous streaming path.
EDGE VS CLAUDE CODE: structural and easy. Interactive editors cannot batch their core loop; ELAI's autonomous /
background / cosmos / research lanes are a natural 50%-off fit that a human-in-the-loop IDE has no equivalent for.
PITFALL: do not batch anything a human is waiting on; 24h worst-case latency.

10.6 LEVER L6 -- DETERMINISTIC OBSERVATION MASKING AS THE DEFAULT CONTEXT MANAGER  [halves cost, preserves cache]
MECHANISM [HOLDS]: "The Complexity Trap" (arXiv:2508.21433, NeurIPS 2025 DL4C WORKSHOP -- not main conf) shows
deterministic observation masking HALVES cost vs the raw agent while MATCHING LLM-summarization solve-rate on
SWE-bench Verified. Body figures: Qwen3-32B $1.12->$0.55 (50.9%), Qwen3-Coder-480B $1.29->$0.61 (52.7%, and
masking even beat summarization 54.8% vs 53.8%), Gemini-2.5-Flash $0.41->$0.18 (56.1%). The "7%/11% hybrid"
figures are ABSTRACT-ONLY future work, not measured [CORRECTED -- do not cite as a result]. Anthropic context
editing (claude.com/blog/context-management, Sep 29 2025 [CORRECTED date, not Jan 2026]) reports 84% token
reduction on a 100-turn web-search eval and +29%/+39% agentic-search (INTERNAL eval). Memory layers cut context
hard: Zep (arXiv:2501.13956) 115K->1.6K tokens (~98.6%, arithmetic) at 71.2 LongMemEval; Mem0 (arXiv:2504.19413)
>90% token cut, J-score 66.88% body-confirmed [CORRECTED: 66.88 IS in the paper; the "92.5 LoCoMo" in ELAI
memory remains a hallucination]. Context-rot (Chroma, Jul 2025) confirms non-uniform degradation as input grows,
Claude worst on the focused-vs-full gap [the specific "20-50% / 200K->50K" figures are ELAI synthesis, NOT in
the source -- CORRECTED]. Crucially, masking PRESERVES the cache prefix (it drops trailing observations) whereas
LLM re-summarization REWRITES the prefix and BUSTS the cache -- so masking compounds L1/L2 and summarization fights it.
ELAI STATUS: the prior framing in this doc ("replace LLM-compaction") was WRONG [REFUTED]. ELAI has NO LLM
compaction to replace -- compact.rs::compact_session is a fully DETERMINISTIC string formatter (counts roles,
lists tool names; called only from the /compact CLI and --resume, not task_runner). The real situation: the
deterministic adaptive_reduce (context_budget.rs:1525-1598, 3 levels, emits ContextReductionPayload) IS ELAI's
observation-masking analogue and exists; the E-25 compaction ladder (compaction_ladder.rs, all 6 tiers
deterministic) is GATED wire-dark behind ELAI_E25_COMPACTION_LADDER_ENABLE; get_relevant_facts (memory read-back)
has zero production callers (4 test files only) [HOLDS, write-active/read-dark].
ACTION: make deterministic observation masking (adaptive_reduce) the DEFAULT context manager on long sessions;
un-gate the E-25 ladder with a default-on cheapest-first policy; wire get_relevant_facts read-back. Frame masking
as cache-PRESERVING (it keeps the stable prefix, only trims trailing observations).
EDGE VS CLAUDE CODE: Claude Code's auto-compact LLM-summarizes and busts the conversation-layer cache each time;
ELAI replaying a deterministic SSB journal + masking keeps the prefix cached AND reproducible -- a real edge.
PITFALL: masking can drop an observation a later step needs; keep the SSB journal as the lossless backstop so a
masked observation is re-fetchable (this is exactly the journal ELAI already has).

10.7 LEVER L7 -- CALIBRATED ROUTING / CHEAP-WORKER DEFERRAL  [architect/editor/verifier; cloud reasons, 7B executes]
MECHANISM: route by an EXTERNAL CALIBRATED signal, never the 7B's self-confidence. RouteLLM (arXiv:2406.18665,
ICLR 2025) paper body Table 6 = 3.66x cost ratio (~73% cut) at 95% GPT-4 quality on MT-Bench [CORRECTED -- the
"85%" is an LMSYS BLOG number, the paper body says 3.66x/~73%; MMLU 1.41x/~29%, GSM8K 1.49x/~33%]; transfers
across model pairs without retraining [HOLDS]. AutoMix >50% (arXiv:2310.12963) [HOLDS]. FrugalGPT 59.2-98.3%
across tasks (arXiv:2305.05176) [HOLDS]. Model-Cascading-for-Code: exclude any model <10% accuracy from a cascade
(arXiv:2405.15842, 26% avg / up to 70% cut) [HOLDS]. Inter-Cascade -48% strong calls / -49.6% fees by reusing
strong-model strategies (arXiv:2509.22984) [HOLDS]. SWE-Protege (arXiv:2602.22124): a 7B reaches 42.4% Pass@1
on SWE-bench Verified -- but with an Opus 4.1 EXPERT (41.0% with Sonnet 4.5 at 11.9% expert tokens), and the gain
requires SFT+RL on expert trajectories, not 4 free calls [CORRECTED: the "vanilla 7B at 9.0%" figure in section
2/5 of this doc is FABRICATED -- not in the paper; prior SLM SOTA is 17%]. So the honest claim is NOT "the 7B is
below the cascade floor"; it is "the cheap worker only saves cloud tokens on steps it can actually do, and the
big measured gains are the cloud expert's."
ELAI STATUS [HOLDS]: the per-role RuntimeOrchestrator + PerRoleRuntime are built but UNWIRED (runtime_orchestrator.rs
defers to B-37D, zero task_runner refs). RuntimeRoutingPolicy.evaluate is observational only (behind ELAI_ROUTE_
DECISION_SSB_ENABLE; RoutingContext hardcodes prior_audit_failures=0, budget_remaining=25.0). Live escalation uses
a hardcoded layer_failed:1 (task_runner.rs:7606). HybridEscalationGate + confidence::compute_score (first-token
Shannon entropy) are test-only, zero production callers. CloudModeRouter IS wired (runtime_setup.rs:533-535,
task_runner.rs:7326,7496) but routes on the stubbed context.
ACTION: wire RuntimeOrchestrator as architect (cloud, reasons/plans) + editor (cheap 7B, cheaply-checkable edits)
+ independent-context verifier (the auditor); replace the hardcoded layer_failed:1 with a calibrated router signal;
feed the router real audit-failure + budget context, not stubs. Keep this strictly cost-AND-correctness gated
(a cheap edit that fails an off-agent check is not cheaper).
EDGE VS CLAUDE CODE: Claude Code / Cursor are cloud-only for the heavy lane; ELAI's owned cheap worker is the
differentiator IF (and only if) the calibrated router keeps it to checkable steps.
PITFALL: routing on 7B self-confidence is the documented anti-pattern; an over-eager editor that fails checks
burns BOTH the cheap call and the cloud retry -- net negative. Gate on measured pass-rate per step class.

10.8 LEVER L8 -- RESPONSE / EXACT CACHING + LOCAL PREFIX CACHE  [secondary; one cloud lever, one local-only]
MECHANISM: GPT Semantic Cache (arXiv:2411.05276) cut API calls 61.6-68.8% at similarity threshold 0.8 -- but on
FACTUAL QA, not code generation [HOLDS]. LLMLingua prompt compression hits 20x at 1.52pp loss on GSM8K but 13pp
loss on BBH at 7x (arXiv:2310.05736); LLMLingua-2 is 2-5x [HOLDS]; the safe target is RETRIEVAL CONTEXT, never
instructions/code (LLMLingua docs) [HOLDS], and "Let Me Speak Freely" (arXiv:2408.02442) shows format/compression
on reasoning roles is catastrophic (Claude-3-Haiku GSM8K 86.5% -> 23.4% under JSON-schema) [HOLDS] -- so do NOT
compress or force-JSON the coder/planner. Local prefix caching is CLOUD-IRRELEVANT (zero effect on the Sonnet
bill): ELAI's sidecar already SHIPS a PrefixKVCache (server.py, sha256-keyed DynamicCache LRU, active under the
canonical --turboquant-bits 0 boot) [HOLDS], and vLLM APC / SGLang RadixAttention give prefill-only (TTFT)
throughput wins (~32% at high prefix-share [HOLDS]; several blog "7.2x/57x" figures were fabricated or
mis-attributed [CORRECTED]). It only helps the cheap WORKER's latency/throughput, never the cloud cost.
ELAI STATUS [HOLDS]: OracleAnswerCache (oracle_cache.rs, blake3-keyed, T1=30d..T4=never) is WIRED in /research
(repl_command.rs) but is IN-PROCESS ONLY (lost on exit; oracle_cache:None at live_cli.rs:802,924,1061). The
TemplateCrystallizer + CompiledPipelineStore ARE wired (runtime_setup.rs:371,444-445 -- the C-25 template cache
exists). prompt_cache.rs is a local FNV-1a exact-match completion cache (30s/300s TTL) for deterministic replay.
ACTION: persist OracleAnswerCache to disk and extend exact/semantic caching to IDEMPOTENT grounding queries only
(never to context-dependent code generation -- the staleness risk is a correctness hazard); keep the local
prefix cache for the cheap worker but never claim it as a cloud-cost win.
EDGE VS CLAUDE CODE: modest -- caching idempotent research/grounding across sessions is something a journal-backed
harness does naturally; an editor with no durable journal cannot.
PITFALL: semantic caching of code-gen is UNSAFE (subtle context differences -> wrong cached answer); confine it.

==============================================================================
11. HOW THE LEVERS COMPOSE (illustrative, NOT a claimed result)
==============================================================================
The levers are not additive and must NOT be multiplied into a headline number (that would be the exact sin this
doc was written to stop). The honest structure:
  INPUT side: caching (L1/L2) discounts the REPEATED prefix to 0.1x; masking (L6) shrinks the VOLATILE tail that
  caching can't help; tool-search (L4) shrinks the fixed tool-def cost; batch (L5) halves the non-interactive
  lanes. These partly overlap -- masking reduces the very bytes caching would have discounted -- so the combined
  input cut is bounded by the larger of the two on any given turn, not their product.
  OUTPUT side: diffs/FIM (L3) cut output independently, and because output is 5x input, a 30% output cut can
  dominate a 60% input cut in absolute dollars on edit-heavy tasks.
Illustrative arithmetic (DESIGN, not measured): a long agentic session whose 30K-token prefix repeats 40 turns
pays ~$3.6 uncached input vs ~$0.5 cached (write once at 1.25x, then 0.1x reads) -- ~85% off THAT fraction; layer
masking and the volatile tail shrinks too; layer diffs and the output bill drops independently. The product of
these is NOT the claim. The claim is one MEASURED $/resolved-task delta (section 14). Everything above is the
hypothesis the measurement tests.

==============================================================================
12. COMPETITIVE INTELLIGENCE -- WHERE ELAI STRUCTURALLY WINS (and where it must catch up)
==============================================================================
CLAUDE CODE [HOLDS unless noted]: 3-layer cache (system+tools / project-context / conversation); deferred tool
loading default-on so MCP connect/disconnect does NOT bust cache (EXCEPT on Haiku, Vertex, or a custom
ANTHROPIC_BASE_URL gateway -- which is exactly ELAI's sidecar/tunnel path, so ELAI loses this for free); 1-hour
TTL for subscription users (5-min on API key unless ENABLE_PROMPT_CACHING_1H=1); measured 98.2% cache-read rate
over 170 turns; per-DIRECTORY cache scope (the system prompt embeds cwd/platform/shell/OS, so two dirs miss each
other's cache); subagents start cold (5-min TTL, own prefix); auto-compact ~167K of a 200K window [WEAK,
decompiled] and the summarization call READS the existing cache (not a cold rebuild) [CORRECTED]; ~$13/dev/active
day, $150-250/dev/month, ~7x token multiplier for agent teams in plan mode.
CURSOR [HOLDS]: Turbopuffer vector index, Merkle 5-min sync, ~13K-token context, a 7B reranker over 500K tokens
(20x cheaper via blob KV caching); deliberately FULL-FILE rewrites (output-heavy by design); measured 85% of
input as cache reads, 178K cache-read tokens on a single task, 10+ LLM calls per agent message; shipped an
Auto-mode 0%-cache bug (Mar 2026). The "27% baseline -> 31% savings" figure some analyses cite is FABRICATED
(actual ~5%) [CORRECTED].
CODEX CLI [WEAK]: encrypted-blob compaction endpoint; ~5-file/50K re-read budget and a v0.118 2-3x regression are
single-source.
DEVIN DESKTOP / WINDSURF 2.0 (devin.ai/cognition.ai, fetched 2026-06-03): the local agent was rewritten in RUST
and is claimed "up to 30% more token-efficient" than Cascade (a vendor claim, no methodology -- ELAI's edge is to
beat them on RIGOR); a per-response token breakdown (input / output / CACHED-input) plus a PROMPT-CACHE-EXPIRY
TIMER in the context-window indicator (the single most directly stealable UX -- see section 14); "Spaces" share a
context bundle across related agent sessions so a new session inherits domain context "without rebuilding context
every time" (a cache-warming / stable-prefix-reuse pattern that directly answers Claude Code's per-directory
cache-miss weakness); "use the best model for each task" + the open Agent Client Protocol (ACP); SWE-1.5 at ~950
tok/s is a LOCAL speed play (Cerebras-class), cloud-bill-irrelevant -- consistent with this doc's quant finding.
WHERE ELAI STRUCTURALLY WINS (the thesis, now sharpened):
  1. Deterministic byte-stable prefix (allocator + ELAI_SEED) -> can GUARANTEE and ASSERT cache stability; can
     use the breakpoint-validity flag as a hard gate. Heuristic editors lose the cache on incidental reorderings.
  2. SSB journal -> reconstruct context by deterministic replay + masking instead of LLM re-summarization, which
     keeps the prefix CACHED and the run REPRODUCIBLE. Claude Code's compaction busts the conversation layer.
  3. Owned cheap worker -> architect/editor/verifier split (cloud-only tools cannot).
  4. Cosmos + session DAG -> a "Spaces"-style shared stable prefix across related sessions, fixing the
     per-directory cache-miss that Claude Code documents and that Devin built Spaces to solve.
  5. Non-interactive lanes (cosmos/research/GEPA/bench) -> Batch API 50%-off that a human-in-the-loop IDE cannot use.
WHERE ELAI MUST CATCH UP (honest): production cloud path ships zero cache markers (L1); custom-base-URL disables
deferred tool loading so ELAI must implement Tool Search itself (L4); subagents cold-start; output path emits
whole files (DIFF dark) (L3). All four are wire-dark, not architectural -- activation closes them.

==============================================================================
13. THE EFFICIENCY WIRE-DARK FIX LIST (sequenced; maps onto existing plan ids)
==============================================================================
This is the "fix it too" punch list, ordered by $-leverage / effort. Each is activation of existing substrate.
  F-E1 (L1, cheapest huge win): mark the stable prefix on the PRODUCTION RuntimeAdapter path (anthropic_api.rs:137);
       wire cache_check.rs hit-rate to the dashboard. Extends B-27/C-5 cloud mode + E-39 cost gate.
  F-E2 (L2): feed real slot bytes to update_breakpoints, consume bp.valid, add the byte-prefix debug_assert, add a
       tail breakpoint, opt into 1h TTL on long sessions. Extends B-10/Phase-3E context budget.
  F-E3 (L3): wire FocusTunnel DIFF mode end-to-end (kill the STRUCTURED fallback), extend streaming early-abort to
       cloud, make ShortCoder rewrite the payload (or stop counting it as savings). Closes B-28 + B-20 edit-matching.
  F-E4 (L4): send the advanced-tool-use beta header + Tool Search Tool over ELAI's tool catalog. New; extends B-37/E-18.
  F-E5 (L5): add a Batch API client; route Cosmos (B-42) / research (B-7) / GEPA (E-8) / benchmarks (E-9/E-49) through
       it at 1h TTL. New, large, easy.
  F-E6 (L6): make adaptive_reduce the default long-session context manager; un-gate the E-25 ladder default-on; wire
       get_relevant_facts read-back. Closes E-25 + F-5 memory read.
  F-E7 (L7): wire RuntimeOrchestrator architect/editor/verifier; replace hardcoded layer_failed:1 with the calibrated
       router; feed real routing context. Closes B-37D + E-31 + the wire-dark HybridEscalationGate.
  F-E8 (L8): persist OracleAnswerCache to disk; extend exact/semantic caching to idempotent grounding only. Extends C-30.
  F-E9 (dashboard): steal Devin's per-response token breakdown (input/output/cached) + cache-expiry timer; surface
       cache-hit-rate + $/task live. Extends C-5 CloudPanel + C-25 CumulativeTokenTracker (cache_read_tokens already wired).
Sequencing: F-E1 -> F-E2 -> F-E9 first (caching + visibility, prove the hit-rate moves), then F-E3 (output) and
F-E5 (batch) in parallel, then F-E6/F-E4, then F-E7 (routing, the heaviest), F-E8 last.

==============================================================================
14. HOW TO PROVE "MORE EFFICIENT THAN CLAUDE CODE" (the measurement, not the assertion)
==============================================================================
The claim is a MEASURED $/resolved-task delta at the model-harness-CONFIG level, never a SWE-bench rank.
GROUNDING: Harness-Bench (arXiv:2605.27922, 106 tasks / 5194 trajectories) proves harness effects vary enough
that capability MUST be reported per model-harness pairing [HOLDS]; the Artificial Analysis Coding Agent Index
shows a ~60x same-model cost spread across harnesses [CORRECTED: ~60x and $0.07-$4.10+, NOT the "32x/$2.26" that
some snapshots cite]; single-run pass@1 varies 2.2-6.0pp at temp 0 (arXiv:2602.07150) so anything under ~6pp is
noise; a paired bootstrap CI (arXiv:2511.19794 [WEAK -- validated on vision/text classification, applying it to
$/RT is an extrapolation to disclose]) is the right test. SWE-Effi (arXiv:2509.09853) shows FAILED attempts cost
4.9-13.1x the tokens of successes -- so efficiency MUST be cost-AND-correctness, never tokens alone.
THE PROTOCOL: same cloud model (Sonnet 4.6), same held-out task set, ELAI-harness vs Claude-Code-harness vs Aider;
report the efficiency tuple per config -- ($/resolved-task, cache-read fraction, input:output token ratio,
tokens-per-resolved-task) -- with a paired bootstrap CI and a ~6pp noise floor; reproducible Docker+seed envelope.
COMPARATORS (live, [HOLDS]): Aider polyglot top models $0.06-0.15/resolved-task; Claude Code ~$13/dev/active day;
Cursor 85% / Claude Code 98.2% cache-read fractions are the bars to beat or match.
INSTRUMENTATION (all wired today): E-49 MetricsCollector cost_per_task_cents (metrics_cmd.rs:53,118);
cache_check.rs hit-rate; SpendAccumulator cache_read_tokens (cloud_mode.rs:654, grounding_client.rs:892);
CumulativeTokenTracker (context_budget.rs:732). The number is recomputable from the SSB journal under ELAI_SEED.
HONEST FRAMING: Devin published "up to 30% more token-efficient" with NO methodology; ELAI's wedge is not a bigger
number, it is a REPRODUCIBLE, journal-recomputable, paired-CI number at the config level -- the same rigor spine
as the SAFEDELTA trust number. The efficiency number and the trust number share one measurement discipline.

==============================================================================
15. EFFICIENCY EVIDENCE LEDGER (technique -> source -> number -> verdict; complements section 5)
==============================================================================
Caching mechanics (LIVE Anthropic docs, fetched 2026-06-03):
  cache read 0.1x / 5-min write 1.25x / 1h write 2x; Sonnet 4.6 $3 in / $15 out / $0.30 cache read. [HOLDS]
  min prefix 1024 (Sonnet 4.6); 4096 (Opus 4.6/4.7, Haiku 4.5, Mythos). [HOLDS/CORRECTED -- broader than 3 models]
  4 breakpoints; 20-block lookback; tools->system->messages invalidation; hit refreshes TTL free; breakeven on
  FIRST read. [HOLDS / CORRECTED breakeven]
  workspace-level cache isolation since 2026-02-05 (Claude API/AWS/Foundry); Bedrock+Vertex org-level. [HOLDS]
Cache layout:
  Don't Break the Cache arXiv:2601.06007 -- 41-80% cost cut, 13-31% TTFT, naive caching can raise LATENCY. [WEAK]
    (cross-provider pooled, DeepResearch Bench, 10K-token prompts, preprint; Sonnet 4.5 body figure 78.5%/22.9%.)
    "89% GPT-5.2" is a 50K ablation [CORRECTED]; no "85% latency" figure exists [CORRECTED].
Output reduction:
  Aider unified-diff laziness 20->61% / 26->59%; 9x error w/o flexible patching. [HOLDS]
  Diff-XYZ arXiv:2510.12487 -- SEARCH/REPLACE best for larger models. [HOLDS] (udiff-for-small was overclaimed [CORRECTED].)
  Hidden Cost of Readability arXiv:2508.13666 (ICSE'26) -- 24.5% avg INPUT cut on FIM across 10 LLMs. [CORRECTED]
  ShortCoder arXiv:2601.09703 -- 18.1% output cut / 37.8% efficiency, 828 pairs (targets OUTPUT). [HOLDS]
  token-efficient tool use 14% avg / 70% max -- measured on Claude 3.7 beta, default-on Claude 4. [CORRECTED: not Sonnet 4.6 numbers]
Batch + tool tokens (LIVE docs):
  Message Batches 50% off; Sonnet 4.6 $1.50/$7.50; 24h/29d/100K-256MB; 1h TTL recommended. [HOLDS]
  Tool Search Tool 85% tool-def cut (~72K->8.7K); Programmatic Tool Calling 37% (43,588->27,297); Opus 4.5 79.5->88.1%. [HOLDS]
Context / memory:
  Complexity Trap arXiv:2508.21433 (NeurIPS25 WORKSHOP) -- masking halves cost (50.9-56.1%/model), matches summarization. [HOLDS]
    (7%/11% hybrid is abstract-only future work [CORRECTED].)
  Anthropic context editing -- 84% token cut, +29%/+39% (internal eval), Sep-29-2025 [HOLDS/CORRECTED date].
  Zep arXiv:2501.13956 71.2 LongMemEval, 115K->1.6K tokens. [HOLDS] Mem0 arXiv:2504.19413 >90% cut, J 66.88% body-confirmed. [CORRECTED]
  Chroma context-rot Jul-2025 -- non-uniform degradation, Claude worst focused-vs-full. [HOLDS] ("20-50%/200K->50K" is ELAI synthesis [CORRECTED].)
Routing / cascade:
  RouteLLM arXiv:2406.18665 (ICLR25) -- 3.66x/~73% at 95% GPT-4 (MT-Bench), transfers. [CORRECTED: "85%" is blog, not paper body.]
  AutoMix >50% [HOLDS]; FrugalGPT 59.2-98.3% [HOLDS]; Model-Cascading-for-Code <10% exclusion, 26%/70% [HOLDS]; Inter-Cascade -48%/-49.6% [HOLDS].
  SWE-Protege arXiv:2602.22124 -- 7B 42.4% w/ Opus-4.1 expert (41.0% Sonnet-4.5 @ 11.9% tokens), needs SFT+RL. [CORRECTED: "9.0% vanilla 7B" is FABRICATED; prior SOTA 17%.]
Local prefix cache (CLOUD-IRRELEVANT):
  ELAI sidecar PrefixKVCache shipped (sha256 LRU, active under TQ=0). [HOLDS] vLLM APC ~32%/-36.7% throughput. [HOLDS] ("7.2x/57x" blog figures fabricated/mis-attributed [CORRECTED].)
Response cache / compression:
  GPT Semantic Cache arXiv:2411.05276 -- 61.6-68.8% API-call cut @ thr 0.8 (factual QA only). [HOLDS]
  LLMLingua arXiv:2310.05736 20x@1.52pp(GSM8K)/13pp(BBH); LLMLingua-2 arXiv:2403.12968 2-5x; compress RETRIEVAL only. [HOLDS]
  Let Me Speak Freely arXiv:2408.02442 -- JSON-schema drops Haiku GSM8K 86.5->23.4%; don't compress/force-JSON reasoning roles. [HOLDS]
Measurement:
  Harness-Bench arXiv:2605.27922 (106 tasks/5194 traj) -- report per model-harness config. [HOLDS]
  agentic-eval variance arXiv:2602.07150 -- 2.2-6.0pp single-run, >1.5pp@temp0. [HOLDS]
  SWE-Effi arXiv:2509.09853 -- failed attempts 4.9-13.1x token cost of successes. [HOLDS]
  Artificial Analysis Coding Agent Index -- ~60x same-model harness cost spread ($0.07-$4.10+). [CORRECTED: not 32x/$2.26.]
  Aider polyglot $0.06-0.15/resolved-task top models; Claude Code $13/dev/active day. [HOLDS]

==============================================================================
16. ERRATA -- corrections to sections 1-9 of this doc (the fact-checker caught these)
==============================================================================
A "cant-lie" product cannot ship its own unverified numbers. The 2026-06-03 sweep found these errors in THIS
doc's own sections 1-9; they are corrected here and the in-body lines have been patched where load-bearing:
  E1. SWE-Protege "7B 9.0% -> 42.4%" (sections 2/C1 and 5 ledger): the 9.0% vanilla-7B baseline is FABRICATED --
      it is NOT in arXiv:2602.22124. Prior SLM SOTA is 17%; 42.4% uses an Opus-4.1 expert (41.0% with Sonnet 4.5
      at 11.9% expert tokens); the gain requires SFT+RL, not 4 free calls. The "below the cascade floor" argument
      built on 9.0% collapses. (In-body line patched.)
  E2. RouteLLM "~85% cost cut" (sections 1/C1 and 5 ledger): that is the LMSYS BLOG figure. The paper body
      (Table 6) is 3.66x / ~73% at 95% GPT-4 quality on MT-Bench. (In-body lines patched.)
  E3. "Replace LLM-compaction with deterministic masking" (Phase 4, section 3): there is NO LLM compaction in
      ELAI to replace -- compact_session is a deterministic string formatter. Correct action: ACTIVATE the
      already-deterministic observation masking (adaptive_reduce) as default and un-gate the E-25 ladder. (See 10.6.)
  E4. Caching "41-80%" (Phase 1 / C2): keep it, but tag it [WEAK] -- cross-provider pooled, DeepResearch-Bench
      preprint on a prefix-heavy workload, not a general or Anthropic-only number. The only honest ELAI target is
      a measured hit-rate. The prior "85% latency" call-out as fabricated is CONFIRMED (TTFT is 13-31%).
  E5. Mem0 "real J-score 66.88%": CONFIRMED in the paper body (the prior "92.5 LoCoMo" remains the hallucination).
  E6. Anthropic context editing date is 2025-09-29, not Jan 2026 (section 5 implied recency); 84% figure holds.
The discipline stands: every number above is tagged; no number here should be quoted without its tag. This
section is the proof that ELAI ran its own integrity gate (Phase 0) on its own roadmap before acting on it.
