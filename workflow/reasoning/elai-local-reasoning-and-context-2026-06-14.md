# Will ELAI improve local-model reasoning/capability, and how good is its cross-task context management?

Pass 1 of 2. Date: 2026-06-14. Author: main LLM (Opus 4.8).
Grounding: full read of workflow/research/small-models-long-horizon-findings.md +
docs/specs/guide/steps/phase-index-MASTER.txt + docs/thefinalglance/{B,C,D,E,F,G,H,I,J,K}-SUMMARY_INDEX.md
+ the oracle and idea-intake skills. External benchmark research (sonnet agent) is IN FLIGHT;
Pass 2 will revise the quantitative claims once it returns.

EVALUATION MODE (user direction 2026-06-14): evaluate ELAI AS A FULLY-SPECIFIED SYSTEM — the
design on its merits, considering the whole spec built. Shipped-vs-planned is NOT used to discount
the architecture; it is tracked separately as an IMPLEMENTATION-READINESS note at the end, never as
a reason to mark the design down. (The SUMMARY-INDEX status stamps are also known to drift — I-39
exists because of "five drifting sources of truth" — a second reason not to lean on them for the
design verdict.)

---

## Q1 — Will ELAI improve local-model REASONING and CAPABILITY?

"Reasoning" has TWO referents. Collapsing them is the most common error (I made it in the first
draft of this file). ELAI improves one of them heavily.

### Referent 1 — PARAMETRIC reasoning (what is baked into the fixed model's weights)
Unchanged at inference. ELAI runs a fixed open-weight model (canonically Qwen2.5-Coder-7B-abliterated;
3B/32B per VRAM tier) and does not alter its weights mid-run. The ONLY path that changes this is the
open-weight flywheel — I-30 (verifier-ranked SFT exhaust) then I-31 (GRPO on execution-grounded
rewards). As a SPECIFIED capability the flywheel does improve parametric reasoning (rejection-sampling
SFT on the harness's own receipt-ranked winning trajectories, then GRPO on compile/test/mutation
rewards), bounded by the convergence theorem: it only converges on a real un-gameable stronger-than-
generator filter (H Track 7). So even Referent 1 is a "yes, as designed," just the narrowest and most
guarded one.

### Referent 2 — SYSTEM / TEST-TIME reasoning (the quality + reliability of the reasoning TRAJECTORY)
This is where ELAI's reasoning-improvement engine lives, and as a specified system it is more
elaborate than what most agent harnesses attempt. GRC and the D-plan ARE this — the D-plan's own
title is "ELAI reasoning-improvement + real-problem-solving roadmap." The layers:

- GRC (Phase 2): forced observe -> hypothesize -> plan -> act -> verify, with MANDATORY grounding
  citations per step, per-role budgets, counterfactual-safety questions. A grounding-and-verification
  discipline on every reasoning step, not freeform CoT. Per-agent-role GRC configs (Phase 2C) shape
  the reasoning of every role.
- D-1 ReasoningTemplate: typed, archived SELF-DISCOVER-style reasoning structures selected per
  task/domain (not one fixed CoT).
- D-2 PPM: a process reward model scoring the reasoning STEP BY STEP (rStar-Math lineage) — a bad
  reasoning step is caught mid-chain, before it poisons the rest.
- D-3 multi-verifier consensus + D-13/E-6 cross-family adversary: a reasoning conclusion must survive
  a verifier from a DIFFERENT model family attempting to refute it (>=2 must concur).
- I-4 receipt-ranked test-time search + H-39 abstention credit + H-42 serial-depth-scaled budget:
  more compute on deeper sub-problems, candidates ranked by deterministic RECEIPTS (veto + mutation
  kills + progress) not by an LLM judge, and "I don't know" scored above confident-wrong.
- B-39 chat REFLECT (best-of-N + bounded REFINE), E-19 micro-reflection gated on self-certainty.

Why this is genuine reasoning improvement and not just plumbing: the test-time-compute /
verification-generation-gap literature says scaffolded reasoning DOES raise effective reasoning WHEN
verification is cheaper than generation. For code that is almost always true (compilers, tests,
type-checkers, SMT are sound oracles), and ELAI's D-10/D-11 deliberately route claims to SOUND
CHECKERS FIRST, OFF-AGENT. The 2507.07505 "verification is as hard as the task" pessimism holds only
for a CORRELATED verifier; ELAI's decorrelated, off-agent, cross-family verification is the
engineering refutation. So the system's reasoning trajectory is measurably better than the bare
model's single-pass reasoning — that is the whole point of the harness.

### CAPABILITY (effective task completion on real, long, multi-step work)
Here ELAI's thesis is strong and — importantly — it is aimed at exactly the right target. Our own
grounded research file (small-models-long-horizon-findings.md, web-verified 2026-06-14, citing
arXiv:2509.09677 + 2511.14777) establishes the mechanism by which small local models lose to
frontier models on agentic coding:

- The dominant cause is NOT reasoning-inability and NOT context-window size. It is EXECUTION
  RELIABILITY OVER A LONG HORIZON. Per-step accuracy p compounds as ~p^n over n dependent steps,
  so a tiny per-step gap explodes on long tasks (a 99.5% vs 97% per-step gap becomes 61% vs 5% at
  n=100). "Larger models execute more turns correctly EVEN WHEN small models have 100% single-turn
  accuracy."
- Small models additionally SELF-CONDITION: once their own error is in context, P(next error)
  rises — they spiral after the first mistake; frontier models recover.
- The one lever a harness (not the model) controls: shorten the EFFECTIVE HORIZON so n-per-
  unsupervised-chain stays small — aggressive decomposition, per-step verification, cheap rollback
  after the first error before the spiral sets in.

That last sentence is, almost verbatim, ELAI's architecture. The mapping is direct:

- HORIZON-SHORTENING: idea-intake decomposes Idea -> Concepts -> Phases -> Steps -> BabySteps,
  each baby step a <=2-file vertical slice (idea-intake SKILL.md; phase-4E DecompositionTree +
  TreeNavigator). Every unsupervised chain is therefore short.
- PER-STEP OFF-AGENT VERIFICATION: D-2 PPM step-scoring, D-3 multi-verifier consensus (>=2 must
  concur), I-17 tri-layer veto (L1 hard veto / L2 calibrated scorer / L3 cross-family adversary),
  the wiring/maintainability/production-caller gates. The verifier reads an artifact the generator
  did not author (an SSB row, a coverage hit) — the un-gameable design.
- CHEAP ROLLBACK BEFORE THE SPIRAL: I-8 voluntary abort + dead-end AST-hash pruning, I-22 first-
  error truncation salvaging the valid prefix, H-49 undo-tree time-travel, anti-spiral skill.
- ABSTENTION + DEPTH-AWARE COMPUTE: H-39 abstention-aware scoring (credit "I don't know" over
  confident-wrong), H-42 serial-depth-scaled reasoning budget (the formal "smarter orchestration,
  not a smarter model" result), D-16 conformal refusal.
- SEARCH AS RELIABILITY, NOT BRILLIANCE: I-4 evidence-ranked search ladder (best-of-N -> tournament
  -> PUCT tree) selects by RECEIPTS (deterministic veto + mutation kills + progress score), not by
  an LLM judge. This converts "one fragile long trajectory" into "many short ones, keep the one
  with receipts" — the exact p^n mitigation.

VERDICT (Q1, evaluated as a specified system): ELAI improves reasoning at the SYSTEM/test-time
level substantially (GRC grounding discipline -> reasoning templates -> process-reward step scoring
-> consensus + cross-family adversary -> receipt-ranked search -> abstention + depth budget), and it
improves PARAMETRIC reasoning only via the gated flywheel. On top of that it raises effective
CAPABILITY on long tasks by attacking the measured failure mechanism (horizon x per-step reliability
x self-conditioning) rather than waiting for a bigger model. On short, well-scoped tasks the local
model is already fine (Qwen-7B measured 100% pass / 25K tokens on the short task set —
project_qwen3b_baseline), so the gain there is marginal; the gain concentrates exactly where bare
small models collapse — and the reasoning-quality layers (PPM, consensus, adversary) are what keep
the longer trajectories from being confidently wrong.

THREE LOAD-BEARING CAVEATS (anti-sycophancy):
1. The convergence theorem (H Track 7 meta-finding): a self-improvement / search loop without a
   REAL, un-gameable, stronger-than-generator verification filter PROVABLY collapses. ELAI's
   shipped DGM filter is currently a PLACEHOLDER (derived_alpha = blake3(config_bytes); trap
   detection = keyword grep — both grep-confirmed live, both self-flagged "unsound"). H-43/H-44
   REPLACE them. Until that filter is real, the capability gain from the autonomous loop is not
   guaranteed — it could amplify confident-wrong output instead of suppressing it. The non-loop
   per-step verification (D-3/I-17) is the safer, nearer-term capability lever.
2. Decomposition has its own cost and its own failure modes (over-decomposition, contract drift,
   the cross-level cascade). It is not free reliability — the design pays for it with the per-level
   auditors (B-41) and the bubble-up/replan machinery.
3. The reasoning layers compose; they do not multiply. PPM + consensus + adversary + search each
   add latency/cost, so the design's own discipline (HD-1 measure-per-config, H-42 depth budget,
   K-4 composed gate-tax ceiling) exists precisely to stop the reasoning stack from becoming
   unaffordable. The design's quality claim is only as good as that affordability gate.

(Implementation-readiness, tracked separately, NOT a design discount: the reasoning substrate
D-plan, I-4 search ladder, I-17 tri-layer stack, H-39/H-42 are PLANNED per their indices; B-39
REFLECT, B-40/B-41 framing+audits, B-37 backends are shipped. This is a build-status note for
sequencing, not an argument about the architecture's merit.)

---

## Q2 — How good is ELAI's context management BETWEEN tasks?

Read "between tasks" as cross-session / cross-task continuity (handoff, memory, resume), with
in-session window management as the adjacent concern. The honest answer has a sharp split between
a strong DESIGN and a currently-weak SHIPPED reality.

### IMPLEMENTATION-READINESS NOTE (build status, not a design discount)
Recorded only so the design verdict is not confused with the current binary; per the evaluation
mode above, this does NOT mark the architecture down.
- Cross-session resume is MANUAL (/continue) and reloads the PACKED, TRIMMED transcript — ~20-27K
  tokens on a local model, i.e. it re-pollutes the fresh window with the old one (H-51 grounding).
- SessionDag (parent_id DAG, branch/merge, 3-pass trimming) exists and is real (Phase 3H, B-9).
- The crates/memory bi-temporal substrate EXISTS but is largely WIRE-DARK: get_relevant_facts has
  zero production callers; conversation.rs has zero memory references; there is no memory ContextSlot
  (H-30/H-31/H-32 grounding). So durable USER/code memory is not actually feeding the prompt yet.
- The in-session policy is CLOUD-shaped applied to BOTH lanes, which actively MISHANDLES local
  (H-50 grounding, code-read 2026-06-05): flat 100K auto-compact never fires for a 32K-window model;
  the 85% gate reads a 64K non-opus window and ignores ELAI_LOCAL_MAX_CONTEXT; even when trim fires,
  recover_stubs re-inflates it to ~0 net; the live backstop is a HARD context_window_blocked error.
  Local prompt caching is also a no-op on the streaming path (sidecar PrefixKVCache bypassed).

So as-shipped, cross-task context management is roughly "a manual resume that reloads a trimmed
transcript, with a memory layer that is built but not wired, and a window policy that is wrong for
the local lane." That is BELOW commodity tools on the local lane today.

### What is DESIGNED (and why the design is strictly stronger than commodity)
The planned stack is genuinely good and well-grounded (Letta sleep-time, Mem0, A-MEM, Anthropic
note-taking, OpenHands dual-serialization — all cited):
- H-51 sleep-time cross-session handoff: author a ~40-100 line topic-keyed DIGEST from the SSB
  journal (off-agent ground truth, not recollection), auto-detect on a fresh session, tiered load
  ~1-3K tokens instead of the ~20-27K transcript. Reconstructs from journal rows, so it is
  reproducible and auditable, not a lossy summary.
- H-50 regime-aware in-session policy: OPPOSITE policies for cloud (keep cache prefix stable) vs
  local (window-MINIMIZATION: unload/keep-stubs-unloaded/selective-reload). Fixes the one-window-
  truth bug.
- I-11 typed BeliefState handoff: a versioned {tried / failed-because / current-AST-state /
  constraints, DKG-keyed} struct replaces raw history on every cross-model / cross-tier / resume
  handoff, and the compression cost is MEASURED (re-run from full history vs BeliefState, ledger
  the delta — "we know what our compression costs").
- I-14 deterministic context compression (LSP slicing + AST folding, reversible receipts), I-5
  pristine-context injector (hard-exclude failed-sibling tracebacks, keep a <=30-token contrastive
  constraint), I-6 anchor pane (immutable task-spec slot last, lost-in-the-middle defense).
- Two-tier durable memory: H-31 bi-temporal user-preference ledger (state once, honored across
  sessions, retired the instant contradicted — write-time supersession, never narrate stale), H-32
  code-grounded fact memory (validate-citation-before-use against the DKG). H-30 is an INSTRUMENT-
  FIRST keystone that MEASURES staleness before either is trusted. Auditor/critic roles get ZERO
  memory injection (Personalization-Trap defense) — protects the trust product.
- J-25 ports MiMoCode's exact production mechanics (typed checkpoint template, tool-pair boundary
  guard so trims never orphan a tool_use/tool_result, 4-level pressure ladder, newest-wins writer
  coalescing). J-34 replaces folklore context thresholds with MEASURED per-model degradation curves.

The differentiator vs commodity (Cursor/Claude Code/Mem0): ELAI's handoff/memory is JOURNALED
(SSB), REPRODUCIBLE (G-2 RRA), and grounded against off-agent rows rather than model recollection,
and it MEASURES its own compression cost. That is a real, defensible edge — IF it ships.

VERDICT (Q2, evaluated as a specified system): the cross-task context architecture is strong,
specific, and strictly stronger than commodity tools. Its defining edge is that the handoff and
memory are JOURNALED (SSB), REPRODUCIBLE (G-2 RRA), grounded against off-agent rows rather than
model recollection, and SELF-MEASURING (I-11 ledgers its own compression cost; J-34 replaces folklore
context thresholds with measured per-model degradation curves). It is also regime-correct: opposite
policies for cloud (cache-stable prefix) vs local (window-minimization), which is exactly the
distinction bare harnesses miss. The one design risk is affordability/complexity of the full memory
+ handoff + compression stack, which the design again gates (HD-1 measured, H-30 instrument-first).
Implementation-readiness (separate): the shipped binary is currently below par on the local lane
until H-50/H-51 + the memory wiring land — a sequencing fact, not an architecture flaw.

---

## What the in-flight external research will sharpen
Pass 1 is mechanism-grounded but not yet NUMBER-grounded. The sonnet research agent is fetching:
best local-model results on SWE-bench Verified / Terminal-Bench / METR horizon / Aider polyglot,
the harness/scaffold used, and the QUANTIFIED harness-vs-model delta (how much a scaffold alone
moves a fixed model). That converts the Q1 claim from "the design targets the right mechanism" to
"a harness historically closes X of the local-vs-frontier gap on benchmark Y," which is the
believe-the-number form. Pass 2 will fold those in and either strengthen or temper the verdict.

---

## Bottom line (Pass 1, to be confirmed in Pass 2)
- Q1: ELAI substantially improves SYSTEM/test-time reasoning (GRC + reasoning templates + process-
  reward step scoring + cross-family consensus/adversary + receipt-ranked search + abstention/depth
  budget — the D-plan IS a reasoning-improvement engine), improves PARAMETRIC reasoning only via the
  gated flywheel, and raises effective long-horizon CAPABILITY by attacking horizon x per-step
  reliability x self-conditioning. Bounded by the un-gameable-verifier convergence requirement and
  by the affordability of the composed reasoning stack — both of which the design itself addresses.
- Q2: Cross-task context management is excellently designed (journaled, reproducible, grounded
  handoff + two-tier measured memory) but currently under-shipped and locally mis-tuned; the fix
  is planned, not done.
- Both verdicts rest on SUMMARY-INDEX status claims that are known to drift — confirm against live
  code before this becomes a plan file.

================================================================================
## PASS 2 — folded in the external benchmark research (2026-06-14)
================================================================================
Evaluated under spec-eval (design-first; both sides grounded). The sonnet research agent returned
a benchmark survey; the numbers strongly STRENGTHEN Q1's design verdict and SHARPEN the ceiling.
Every figure below carries a source; treat vendor-self-reported scores as upper bounds (the report
flags contamination + scaffold non-standardization — OpenAI formally abandoned SWE-bench Verified
in early 2026 as a frontier measure).

### The harness-vs-model evidence is the headline, and it validates ELAI's core bet
The scaffold/harness moves a FIXED local model's score by huge margins on agentic coding:
- Same weights, scaffold-only swing: Qwen3-32B 28% (SWE-agent) -> 48% (Agentless) on SWE-bench Lite;
  GPT-4o-mini 10% -> 26% (SWE-Effi, arXiv:2509.09853). +10-20pp, i.e. +71-160% relative.
- 20% of reachable issues are missed by ANY single human-designed scaffold (ReCreate, arXiv:2601.11100:
  union of 5 GPT-4o agents solves 184 vs best-single 147). There is real, measured "scaffold headroom."
- Test-time compute: DeepSeek-Coder-V2 15.9% (pass@1) -> 56% (pass@250) on SWE-bench Lite, beating the
  then-frontier single-shot SoTA of 43% (Large Language Monkeys, arXiv:2407.21787). DeepSWE Qwen3-32B
  42% -> 59% with TTS; SWE-Master-32B TTS@8 -> 70.8%.
- Agentic POST-TRAINING (SFT on agent trajectories + RL on real execution feedback) is the single
  highest-return lever: SWE-Master Qwen2.5-Coder-32B ~5% -> 61.4% (SFT+RL) -> 70.8% (TTS@8)
  (arXiv:2602.03411); DeepSWE Qwen3-32B 23% -> 42% from RL alone. +50-65pp — LARGER than moving 7B->32B
  in a family (~5-15pp).

Why this matters for ELAI: the two empirically highest-return techniques are EXACTLY the two ELAI is
architected around. I-4 receipt-ranked test-time search IS the TTS lever; I-30/I-31 (SFT exhaust ->
GRPO on execution-grounded rewards) IS the post-training lever. And the literature names the precise
bottleneck of both: "the bottleneck is having a reliable verifier to select the best sample; current
model-as-verifier approaches leave a substantial gap to oracle-level verification." That bottleneck is
ELAI's central design obsession — SOUND, OFF-AGENT, sound-checker-first (D-10/D-11), CROSS-FAMILY
decorrelated verification (D-13/I-17), and the un-gameable-filter convergence requirement (H Track 7).
So ELAI is not only on the right two levers; it is built around the exact failure point the empirical
literature says limits those levers. As a specified design, that is a strong, evidence-aligned bet.

### The long-horizon ceiling is real, and the design must not over-claim past it
On single-issue SWE-bench Verified the local-vs-frontier gap is now small (Devstral-Small-2 24B 68%
on a consumer GPU; Qwen3.6-27B 77% vendor-scaffold; vs Claude Opus 4.6 ~80.8% — a 3-13pp gap, though
standardized Scale-SEAL numbers are markedly lower than vendor self-reports). But on genuinely
LONG-HORIZON tasks the gap is stark and scaffold-resistant:
- SWE-EVO (multi-file evolution, arXiv:2512.18470): GPT-5 21% vs its own 65% single-issue (ratio 0.32);
  120B open-weight models 2-6%; sub-32B effectively ~0-2%.
- SWE-bench Pro standardized (arXiv:2509.16941, Sep-2025): sub-32B 3.4-6.8% vs frontier ~23%.
- METR time-horizon (2026-01): NO open-weight model is even on the published curve; the note that
  mid-2025 DeepSeek (a 685B model) ~= late-2024 frontier implies the best OPEN-WEIGHT ~10-20 min 50%
  horizon vs Claude Opus 4.5's 320 min. Sub-32B is ~18-24 months behind on long-horizon autonomy.
- The MECHANISM is exactly our Pass-1 research file: small-model failures are 42% tool-use errors vs
  ~25% for frontier, and >60% of SWE-EVO failures are instruction-following over many steps — i.e.
  execution-reliability x horizon + self-conditioning, NOT reasoning-inability.

This is the crux of the honest verdict. ELAI's decompose -> verify -> rollback IS the correct (and
only) harness answer to that mechanism: it tries to convert one long-horizon task (where sub-32B scores
~3%) into many short sub-tasks (where the gap is only 3-13pp). The design is mechanism-sound. BUT:

VALIDATED vs EXTRAPOLATED (the line spec-eval requires me to draw honestly):
- VALIDATED by these numbers: a harness raises a fixed local model +10-40pp on short agentic tasks;
  TTS-with-a-verifier and SFT+RL are the top levers; the verifier is the bottleneck. ELAI is on all
  three. So ELAI WILL materially raise a local model's reliability/capability on decomposed, short,
  verifiable sub-tasks. High confidence.
- EXTRAPOLATED (not yet proven for ELAI's shape): the +20-40% scaffold figures are for MONOLITHIC
  single-agent loops (Agentless/SWE-agent/OpenHands), not a decompose-orchestrate harness. No published
  result shows a decomposition orchestrator recovering the long-horizon gap for a sub-32B model. Error
  also compounds ACROSS sub-tasks, and integration/cascade is its own horizon (SWE-EVO's 0.32 ratio is
  the warning). ELAI's per-step verification + first-error truncation + rollback are designed to fight
  exactly this, but "decomposition + sound verification beats a monolithic small-model agent on
  long-horizon" is a DESIGN HYPOTHESIS this corpus supports mechanistically but does not yet measure.
  ELAI's own SAFEDELTA/convergence-curve instruments (G-6, I-27, I-38) are the right way to falsify it.

### Q2 (context management) — the research reinforces the design choices
The report independently names the local-model context failure modes ELAI's context stack targets:
"context rot / lost-in-the-middle" worsened by sub-32B's smaller windows (32K-128K), and error
compounding ("models become more likely to make mistakes after their own mistakes"). ELAI's I-6 anchor
pane (lost-in-the-middle defense), I-5 pristine-context injector (hard-exclude failed-sibling
tracebacks = stop in-context error compounding), H-50 local window-minimization, I-14 deterministic
compression, and I-11 BeliefState handoff are each aimed at one of those named failure modes. So the
Q2 design verdict (strong, regime-correct, journaled/measured) holds and is reinforced by the data.

### REVISED bottom line (Pass 2)
- Q1: ELAI is a reasoning-AND-capability improvement system whose design sits on the two empirically
  highest-return local-model levers (test-time search with a sound verifier; SFT+RL post-training) and
  is built around the exact bottleneck the literature names (a reliable, un-gameable, off-agent
  verifier). On short/decomposed verifiable work the local-vs-frontier gap is already small (3-13pp) and
  ELAI's stack should capture most of the harness/TTS gains. On hours-long unsupervised autonomy the gap
  is fundamental for sub-32B and scaffold-resistant; ELAI's decomposition is the right answer but
  recovering that gap for a small model is a DESIGN HYPOTHESIS not yet measured — realistic framing is
  "far more reliable + captures the harness ceiling," and matching frontier long-horizon autonomy needs
  the flywheel (I-30/I-31) or the 24-32B tier (which the VRAM tier matrix + 2x3090 plan already target).
- Q2: cross-task context management is strongly designed and aimed squarely at the measured sub-32B
  context failure modes; journaled + reproducible + self-measuring is a real edge over commodity.
- Both verdicts are design-level (spec-eval); implementation-readiness is tracked separately and is not
  a mark against the architecture.

================================================================================
## PASS 2.5 — the three local-model intuitions resolved + "freeze the head, minimize the tail" (2026-06-14)
================================================================================
Triggered by the user's three direct questions (does ELAI solve context-degradation on long tasks; does
the tunnel solve tool-drift; how fast can the "generation cog" run with context offloaded and no weight
reload). Web-grounded on context rot. The three resolve cleanly and surface ONE design refinement.

External grounding (context rot is real, universal, and NOT just a window-size problem): every one of
18 frontier models (GPT-4.1, Claude Opus 4, Gemini 2.5) degrades as input grows — RoPE long-term decay
+ lost-in-the-middle (U-shaped, >30% accuracy drop for mid-context info) + attention dilution +
distractor interference. Sources: Chroma context-rot via morphllm.com/context-rot;
morphllm.com/lost-in-the-middle-llm; understandingai.org/p/context-rot; redis.io/blog/context-rot.
This nuances Pass-1 Q1's dismissal of "context-window size" as a cause: window SIZE is not the binding
constraint, but window OCCUPANCY and ROT are — and worse on sub-32B's 32K-128K windows. (Internal
inconsistency in the file: Q1 under-counts rot, Q2 gets it right via I-5/I-6. Q1 should read "size is
not the cause, rot and occupancy are," which strengthens Q2.)

INTUITION 1 — context degradation on long tasks: ELAI solves it by AVOIDANCE, not CURE. You cannot fix
RoPE decay in frozen weights; you refuse to operate in the regime where it bites. Decomposition means
no single unsupervised call accumulates a long/dirty context; each baby-step is short -> rot never
reaches damaging length. I-5 pristine-context (hard-exclude failed-sibling tracebacks = kills in-context
error compounding) + I-6 anchor pane (immutable task-spec LAST = lost-in-the-middle defense). THE CATCH:
decomposition does not delete rot, it RELOCATES it to the seams — long-horizon state must now survive
BETWEEN short calls via handoff/memory/BeliefState. If that carrier is lossy or bloats, the rot just
moved from inside-one-window to across-windows. That is why Q2's journaled/measured carrier (I-11
BeliefState, H-51 digest, I-14 reversible compression) is NOT polish — it is what makes decomposition
actually pay off.

INTUITION 2 — tool-drift ("loses itself on tools after a while"): NOT the tunnel's job. Transport !=
discipline. The fix is moving tool AUTHORITY out of the model's generation loop: I-1 directive/
supervisor plane (generator emits intent only; a separate process executes tools + holds credentials),
I-35 temporal tool-sequence enforcement, J-12 intervention harness (H3 embeds tool constraints in tool
DESCRIPTIONS = "constant attention," re-stated every call so they never fall into the rotting middle),
Tier-C process isolation (agents have NO net/LLM/tools; tools dispatched TO them). (Pass 3 develops the
Focus-Tunnel embodiment of this same inversion.)

INTUITION 3 — the "generation cog" + context-offload-without-weight-reload: architecturally correct,
CONFIRMED FROM CODE. The sidecar loads weights ONCE (server.py:2043-2050 from_pretrained), stays a
persistent resident process (NF4 ~1.92GB on the 3090); NO weight reload between calls. All durable state
(SessionDag, SSB journal, memory, BeliefState) lives OUTSIDE the model in Rust/SQLite — the model is a
hot stateless cog. Per-call cost = prefill (proc input context, parallel, cheap) + decode (gen output,
sequential, the bottleneck). Short per-call context wins three ways at once: dodges rot, near-free
prefill, max per-step accuracy p. The UNTAPPED SPEED LEVER is KV prefix reuse: the sidecar HAS a
PrefixKVCache (server.py:1819, LRU prefix reuse for DynamicCache) and the Rust side structures 2 xxhash
cache breakpoints — but it is bypassed on the live local streaming path (H-50 grounding) and disabled
under TurboQuant. Reusing the KV of the stable head (system + rules + anchor) drops fixed-prefix prefill
to ~0, so you only prefill the ~1-3K per-step delta instead of re-reading ~8K every step. At that point
the binding constraint stops being the model and becomes the SERIAL DEPTH of the decomposition (longest
chain of DEPENDENT calls) — literally H-42's serial-depth budget (cost scales with serial depth, not
window size). The orchestration engine's job in the cog model is therefore to MAXIMIZE PARALLEL
baby-steps (BSP supersteps, JoinSet spawn_parallel) and MINIMIZE the serial chain, or the resident-cog
throughput is wasted on a needlessly linear plan. On a 3B/7B NF4 model on a 3090 with weights resident +
prefix reuse + short contexts, each baby-step is sub-second-to-low-seconds (small-delta prefill + decode
of a few hundred output tokens).

THE REFINEMENT (the single cleanest one falling out of all three): commit explicitly to FREEZE THE HEAD,
MINIMIZE THE TAIL. The tension: H-50's local policy "unload-early / window-minimization" FIGHTS KV
prefix reuse — unload means a COLD prefix means re-prefill every call, throwing away the cache win. The
resolution is to keep the HEAD (system prompt + rules + anchor pane) hot and KV-cached and stable, and
minimize/swap only the small task-specific TAIL. This makes window-minimization and prefix-caching stop
fighting each other, and it is what lets the cog model hit both fastest AND most-reliable at once. It
composes H-50 (regime-aware policy) + I-15 (semantic cache / blake3 fast path) + the 3E context-budget
2-cache-breakpoint structure (xxhash prefix). OPEN FORK (for the user): make this a proper L-plan unit,
or keep as a reasoning artifact. (User leaning / status: pending decision at end of this file.)

================================================================================
## PASS 3 — the Focus Tunnel thesis: "minimal model kernel + maximal deterministic harness" (2026-06-14)
================================================================================
Triggered by the user's framing: "make ELAI smart without a model and use tools as needed for the model."
The referent is B-28_focus_tunnel.md (the GENERATION focus tunnel), NOT the SSH transport tunnel.
Evaluated under spec-eval (design-first; benchmark-grounded). This pass connects the Focus Tunnel to
the Pass-2 benchmark mechanism and resolves how far "smart without a model" actually goes.

### What the Focus Tunnel is (the labor-division mechanism)
It reduces the model to the smallest irreducible generative act and does everything else
deterministically in the harness (all per B-28):
- FIM mode: model outputs RAW CODE between prefix/suffix. No tool schemas, no JSON wrapper (B-28:17-19).
- Mode selection deterministic, "the LLM has no input" (B-28:24-29).
- Speculative file routing: "model never specifies path" (B-28:44).
- Dependency management: "model NEVER runs cargo add / pip install" (B-28:51,107).
- Streaming validation: tree-sitter partial parse kills a bad gen at token 50 not 500, ~90% time saved
  on bad gens (B-28:34-36).

### Why the thesis is GROUNDED, not wishful — it offloads the exact failure axis
The Pass-2 benchmark research is the load-bearing grounding: sub-32B models do NOT fail at raw code
generation — they fail at TOOL-USE (42% errors vs ~25% frontier) and INSTRUCTION-FOLLOWING over many
steps (>60% of SWE-EVO failures). Raw constrained code-gen is already near-frontier on short tasks
(Devstral-24B 68% SWE-bench Verified; Qwen-7B measured 100% on our short task set,
project_qwen3b_baseline). So the optimal division of labor for a FIXED small model is precisely: take
away what it is worst at (tool-calling, JSON formatting, path/dep decisions, knowing when to stop),
keep it only on what it is already good at (filling a short well-bounded code hole). The Focus Tunnel
does exactly that. FIM in particular DELETES THE ENTIRE TOOL-USE FAILURE CATEGORY — no JSON, no tool
schema, no path argument for the model to get wrong. You are not making the model better; you are
refusing to ask it the questions it answers badly. Evidence-aligned, strong design.

It stacks on the Pass-1/cog model: FIM is a LOWER-ENTROPY task than "write a function with tool calls,"
so per-step accuracy p rises -> p^n over the chain improves -> and context per call shrinks (~350
tokens/step) -> less rot, faster prefill. Same lever, three wins again.

### The precise reframing (anti-sycophancy correction to the user's phrase)
You cannot make ELAI smart WITHOUT a model — you make it need the SMALLEST IRREDUCIBLE MODEL KERNEL.
Novel code synthesis is irreducibly generative; no deterministic harness produces the CONTENT of the
hole. The model stays needed at exactly two points — high-level planning/decomposition (idea-intake)
and the FIM generative kernel — and everything BETWEEN them (retrieval, routing, mode, validation,
deps, context assembly, verification) is deterministic. That "between" layer is where ELAI is smart
WITHOUT a model; the Focus Tunnel is the seam. Correct framing: push the intelligence boundary as far
into the deterministic harness as it will go, leaving the model the smallest kernel it cannot be
replaced on.

### The inversion that makes it work: harness holds tool authority, model only fills
A local model "loses itself on tools after a while" because it is HOLDING tool authority — deciding
which tool, formatting the call, accumulating a tool transcript that rots. The Focus Tunnel (with the
I-1 directive/supervisor plane) takes tool authority OUT of the model loop: the harness decides the
mode, picks the file, resolves the deps, runs the validators; the model only ever emits content. Tools
are "used as needed for the model" — invoked AROUND the model, never BY it. That is the structural fix
for tool-drift (the transport tunnel has nothing to do with it).

### How far the thesis goes (adversarial — three load-bearing ceilings)
1. FIM only works for LOCAL edits with a clean prefix/suffix. Multi-region -> DIFF (weaker); new
   files / cloud -> STRUCTURED (weakest, full tool-use). The "model barely needed" win concentrates on
   edit-existing-code-in-a-known-spot — a large fraction of real coding but NOT the irreducibly-global
   multi-file evolution where the gap is worst (SWE-EVO 0.32 ratio). The tunnel narrows the MODEL'S
   JOB, not the TASK'S intrinsic globality.
2. You trade STOCHASTIC model errors for DETERMINISTIC harness errors — which are silent and CONSISTENT
   (same wrong answer every time), with no model judgment to catch them because you removed its
   authority. ELAI mitigates (AST-mismatch -> micro-step retry, the gates) but it is a real tradeoff:
   the smarter the deterministic layer, the more the HARNESS itself becomes the thing that must be
   verified. "Smart without a model" means the harness's correctness is now load-bearing.
3. The mode-selector deciding "single-region edit" does not guarantee the model fills it correctly.
   The tunnel raises p by lowering task entropy; it does not make p=1. Reliability still rests on
   per-step verification (streaming validation + audit gates), which is why those cannot be cut.

### The refinement that falls out (the actionable design lever)
The more steps route to FIM, the more "smart without a model" ELAI actually is. So the lever is to
MAXIMIZE THE FIM FRACTION — decompose and structure edits so the model's job is as often as possible
"fill this hole" rather than "author this file with tools." B-28 ALREADY tracks mode_distribution per
task in SSB (B-28:85) — that metric IS the measure of model-independence on a given task. Combined with
the resident-cog (no weight reload), KV-prefix reuse on a frozen head (Pass-2 "freeze the head,
minimize the tail"), streaming early-kill, and pipeline pre-assembly, a high-FIM workload is where the
local lane gets fastest AND most reliable simultaneously.

### Pass-3 bottom line
"Make ELAI smart without a model, use the model only as needed" is sound and is already the
architecture's spine, stated precisely as: MINIMAL IRREDUCIBLE MODEL KERNEL + MAXIMAL DETERMINISTIC
HARNESS + HARNESS HOLDS ALL TOOL AUTHORITY. It is grounded because it offloads exactly the axis the
benchmark data says sub-32B models fail on (tool-use, instruction-following), not the axis they are
already strong on (short constrained code-gen). The honest ceiling: FIM cannot shrink an
irreducibly-global task, and the correctness burden moves onto the harness itself. The single
actionable lever is to maximize and measure the FIM fraction (mode_distribution) as the operational
definition of how model-independent ELAI is on any given task.

================================================================================
## PASS 4 — prior-art grounding: the Focus Tunnel thesis is a NAMED, VALIDATED discipline with quantified ceilings (2026-06-14)
================================================================================
Source: one Sonnet research agent, 14 web searches / 18 web fetches, mostly T1 primary sources, written
to workflow/research/harness-as-pipe-prior-art-2026-06-14.md (read in full). This pass converts Pass-3's
"mechanism-sound bet" into "named-and-measured discipline," and — importantly — external evidence
INDEPENDENTLY CONFIRMS the exact ceilings Pass 2.5 / Pass 3 derived (triangulation, not coincidence).

### The thesis is real, validated, and already named "harness engineering"
The pattern — strip tool-selection/routing/planning/format off the model, do it deterministically, leave
the model a constrained generation slot — is a recognized 2024-2026 discipline with multiple MEASURED
instantiations:
- Agentless (arXiv:2407.01489): fixed 3-phase deterministic pipeline, model only ranks/generates within
  slots; SOTA-for-open-source on SWE-bench Lite at ~$0.34/issue (27.33% peer-reviewed -> 40.7%/50.8%
  Lite/Verified w/ Claude 3.5 Sonnet, README). This IS the thesis, published 2024.
- Life-Harness (arXiv:2605.22166): uses the EXACT phrase "frozen model as a reasoning kernel"; harness-
  only adaptation improved 92% of 126 model-env configs, avg +88.5% relative, transfers across 17 models.
- Compiled AI (arXiv:2604.05150): extreme version — invoke LLM ONCE at compile time, execute
  deterministically forever; 57x fewer tokens, 100% reproducibility.
- Grammar-constrained decoding (XGrammar arXiv:2411.15100 / SynCode arXiv:2403.01632): moves FORMAT
  correctness into the decoder; 96.07% Python/Go syntax errors eliminated, near-zero overhead.
- AHE (arXiv:2604.25850): measured head-on on Terminal-Bench 2 — harness moves beat model moves
  (memory +5.6pp, tools +3.3pp), and system-prompt-alone REGRESSED -2.3pp (more instruction text on the
  model made it worse — a direct datum for the "minimal model kernel" claim).
- LLMCompiler (arXiv:2312.04511, ICML 2024): model emits a one-shot DAG, deterministic scheduler runs it;
  3.7x latency / 6.7x cost / +9% accuracy vs ReAct. RIG (arXiv:2601.10112): deterministic repo-graph
  instead of model codebase-understanding; +12.2% accuracy / -53.9% time. DSPy (arXiv:2310.03714):
  compile-time prompt optimization makes T5-770M competitive with expert GPT-3.5 chains.

### The three counter-findings that bound the strongest version (anti-sycophancy)
Two of these independently CONFIRM Pass 2.5/Pass 3's derived ceilings.
1. THERE IS A MODEL-CAPABILITY FLOOR — the harness is not a substitute for the model. SWE-Effi
   (arXiv:2509.09853): GPT-4o-mini on a strong scaffold collapses to EuTB 5.1% where Qwen3-32B on the
   SAME scaffold hits 21.8%. So "make the model tiny, the harness huge" is REFUTED. Implication for ELAI:
   the benefit is largest for capable-but-not-frontier models (Qwen-7B/Coder is plausibly above the floor;
   GPT-4o-mini-class is below it). You cannot push the model arbitrarily small.
2. NO HARNESS ESCAPES CONSTRAINT-ACCUMULATION / MULTI-FILE GLOBALITY. Constraint Decay (arXiv:2605.06445):
   ~30pp average drop as stateful cross-file constraints pile up, IDENTICAL decay for a 100-line harness
   and full OpenHands; only 8.3% full compliance at L3. Agentless produced 6 multi-file patches out of 497
   successes; agentic beats deterministic on cross-file (OpenHands 25.3% vs Agentless 18.3% on multi-file,
   arXiv:2509.13941). This is EXACTLY the Pass-3 "FIM cannot shrink an irreducibly-global task" ceiling,
   now externally measured.
3. AT THE FRONTIER THE HARNESS BARELY MATTERS. Mini-SWE-Agent: 100 lines + a frontier model = >74%
   SWE-bench Verified. The thesis payoff is INVERSELY proportional to model strength — good for ELAI's
   local lane, irrelevant to its cloud lane.

### The genuine whitespace (where the "superpower" actually is)
The pieces all exist and are validated SEPARATELY, but the survey is explicit that the specific thing
ELAI is shaped to do is unstudied:
- THE INTEGRATED COMBINATION (Agentless-routing + grammar-constrained decoding + deterministic file-
  routing + compile-time optimization) measured ON A SMALL LOCAL MODEL — "genuinely unstudied." ELAI's
  Focus Tunnel + DKG repo-map + TemplateCrystallizer + rules-engine ARE that combination; nobody has
  assembled and measured it.
- THE DISPROPORTIONATE-SMALL-MODEL-BENEFIT PREDICTION is theoretically sound but NEVER cleanly measured.
  That is exactly the SAFEDELTA/believe-the-number experiment (G-6/I-27/I-38).
- SEMANTIC RULES ENFORCED IN THE HARNESS, NOT THE PROMPT — "nobody has built a harness where semantic
  codebase rules are enforced mechanically rather than prompted." ELAI's RULES ENGINE (rules as DATA in
  SQLite, AST/GREP/SCHEMA/COMPOSITE predicates -> BLOCK verdicts, phase-3F) is precisely that mechanism.
  The literature calls this empty space; ELAI already has the substrate. Real, named differentiator.

### Two concrete levers the survey hands ELAI for free
- GRAMMAR-CONSTRAINED DECODING is the upgrade to Focus Tunnel streaming validation. B-28 today DETECTS
  bad structure via tree-sitter at token 50; XGrammar/SynCode PREVENT it at the decoder with ~zero
  overhead (move format-correctness fully off the model). Sidecar-side change, strictly stronger.
- KIMI-DEV (arXiv:2509.23045) validates the flywheel's training-data shape: deterministic-pipeline
  trajectories give a clean 0/1 reward per modular step and the learned skills TRANSFER to agentic
  settings (60.4% SWE-bench Verified). Direct endorsement of I-30 verifier-ranked SFT exhaust — the Focus
  Tunnel does not just run the model, it GENERATES IDEAL TRAINING DATA for it.

### Pass-4 bottom line
Corrected framing: not "smart without a model" and not unprecedented — it is a NAMED, VALIDATED discipline
(harness engineering) whose payoff is LARGEST exactly where ELAI lives (a capable-but-not-frontier local
model) and HARD-CAPPED exactly where Pass 3 said (irreducibly-global multi-file tasks, regardless of
harness). The defensible "superpower" is narrower and more interesting than "make the model dumb": it is
THE INTEGRATED COMBINATION MEASURED ON THE SMALL LOCAL LANE + MECHANICAL ENFORCEMENT OF SEMANTIC CODEBASE
RULES — both of which the literature confirms are genuine whitespace nobody has occupied. Stale-risk
flagged in the source file: SWE-Effi 5.1/21.8 came from a snippet not the full PDF; Mini-SWE-Agent's >74%
is frontier-model-only (the small-model+minimal-harness experiment was never run).

================================================================================
## PASS 5 — the same repos read at SOURCE-CODE level (2026-06-15)
================================================================================
Triggered by the user dropping 9 repos + a paper for a fresh code-grounded steal
run. Where Passes 1-4 reasoned over PAPER ABSTRACTS of these systems, Pass 5
reasons over their actual SOURCE CODE. Inputs (all this session): code-maps of
xgrammar, syncode, Agentless, Kimi-Dev, meta-harness, agentic-harness-engineering
(AHE), Life-Harness/AgentBench, Life-Harness/TauBench, live-swe-agent
(workflow/papers/repomap-*.md) + the Compiled AI paper
(workflow/papers/paper-2604.05150-summary.md) + a full read of
ELAI-FEATURE-MAP.md incl. its Section-4 rejections ledger. spec-eval mode holds.
Standalone artifact: workflow/reasoning/elai-improvement-from-9repo-codemaps-2026-06-15.md.

### Headline: the CODE triangulates the design; the lever is SEQUENCING, not invention
ELAI's spec already contains essentially EVERY technique in all 9 repos, mapped to
plan IDs (grammar decoding E-35/I-34/J-15; Life-Harness 4-layer J-12; Agentless
E-46; DGM/AHE/meta-harness self-improvement H-43..H-48; Compiled AI -> C-25
TemplateCrystallizer + safety sandwich; Kimi-Dev SEARCH/REPLACE E-35; AgentBench
stagnation C-10). So at the code level there is NO missing-capability finding -
there is a sequencing + affordability finding. All 9 repos + the paper converge on
the SAME three primitives, and all three are already ELAI's central bets
(triangulation = the strongest evidence the architecture is aimed right):
1. UN-GAMEABLE OFF-AGENT VERIFICATION as the load-bearing primitive everywhere -
   Agentless (a repro test is trusted ONLY if it FAILS on the original buggy code
   first; selection by test-signal + AST-normalized majority vote, never LLM
   judge), Kimi-Dev (generator x verifier 40x40 + cheap fail-closed ast/flake8/
   no-op/git-apply gates), AHE (every harness edit carries a falsifiable
   predicted_fixes/at_risk_regressions checked against next-eval ACTUAL flips,
   auto-rollback), meta-harness (mutator/judge separation by TOOL DENY-LIST +
   off-agent fitness + (accuracy,COST) Pareto), Life-Harness/tau2 (end-state
   DB-hash via write-only replay vs gold + pass^k), Compiled AI (4-stage
   Security->Syntax->Execution->Accuracy + first-pass-rate metric). This IS
   wiring-enforcement R1-R9 / F-4 / D-3 / production-caller gate. Confirmed.
2. CONSTRAIN OUTPUT FORMAT MECHANICALLY not by prompting - xgrammar/syncode
   logits-layer grammar, Kimi-Dev/Agentless SEARCH/REPLACE DSL, Compiled AI
   Pydantic+auto-retry. ELAI plans all of it (E-35/C-26/I-34) but it is the most
   under-shipped high-leverage area.
3. FREEZE THE MODEL, EVOLVE THE HARNESS - AHE, meta-harness, Life-Harness state
   it explicitly; Compiled AI compile-once is the extreme. ELAI's spine. Confirmed.

### The three load-bearing DE-RISKS (these repos hand ELAI working reference code)
B.1 H-43 DGM execution-grounded fitness <- AHE + meta-harness (HIGHEST LEVERAGE).
   Pass-1 caveat #1 / VD-1/VD-2 name ELAI's fitness placeholder
   (derived_alpha=blake3(config_bytes) darwin_admit_loop.rs:186; trap=keyword grep
   trap_tasks.rs:123) as THE caveat on the autonomous-loop capability gain. AHE's
   evolve.py staggered-generation FALSIFICATION (predict flips -> verify against
   off-agent actual flips -> auto-rollback wrong predictions to best-ever) is a
   real stronger-than-generator filter; meta-harness's tool-DENY-LIST mutator/judge
   separation makes "agent runs its own gate" structurally impossible. Both are
   config/loop ports, no model, no RunPod. This un-blocks the convergence theorem
   gating the whole flywheel (I-30/I-31). H-43/H-44/H-45/H-47.
B.2 Grammar-constrained decoding INTO THE SIDECAR NOW <- xgrammar (+ syncode
   pattern). E-35 reads "XGrammarLogitsProcessor absent (RUNPOD-PENDING)" yet the
   sidecar is already model.generate()+LogitsProcessor chain listing "XGrammar
   mask". xgrammar is pip Apache-2.0 with a ready HF LogitsProcessor (contrib/hf.py)
   + get_model_structural_tag (consumes ELAI's OpenAI-shape tool catalog, emits the
   per-model wrapper grammar = a structurally-valid schema-correct tool call every
   time = DELETES the 42%-of-errors tool-use failure category = the decoder-level
   Focus Tunnel) + FindJumpForwardString (forced literal continuations, no forward
   pass). syncode adds the fallback pattern (offline mask store keyed by
   (tokenizer,grammar,mode) + byte-tokenizer abstraction). RUNPOD is only the EVAL;
   per W-10 the integration ships now behind a fixture. CAVEATS (no over-sell):
   local-lane only (cloud takes no logits processor), shapes SAMPLING not SEMANTICS
   (grounding/wiring gates stay orthogonal), HD-3 still bans grammar on reasoning/
   coder/planner roles. E-35/E-18/C-26/I-34/J-15.
B.3 Deterministic stagnation counters <- AgentBench. C-10 is PARTIAL with
   phase_4G.toml [stagnation] MISSING; J-12 H4 planned. AgentBench ships the exact
   CONTENT-AGNOSTIC counters mapping 1:1 onto StagnationPattern (exact-repeat->
   Spinning, A<->B->Oscillation, empty-error-streak->NoDrift, budget-force=GRC
   converge-or-escalate). Deterministic counts over executor+auditor history, no
   LLM. DISCIPLINE: binary counters only on deterministic seams (build/test/wiring/
   dup-action), never on NL taste/maintainability (over-fires). C-10/J-12.

### Cheap net-new wins
live-swe-agent (README + 3 YAMLs): per-TRAJECTORY $ cap ($3/task; E-39 has a
ceiling, not a per-task model cap), head+tail output truncation (5000+5000, middle
elided) for local-window protection (composes H-50), step-level reflection nudge +
in-trajectory DISPOSABLE tool synthesis (explorer-phase lever, A/B it). Kimi-Dev:
flake8-FATAL-only-DELTA gate (block only NEW fatal errors) - cheap auditor Layer-1.
Compiled AI: break-even n*=GENTOKENS/RUNTIMETOKENS_saved as the PRINCIPLED
crystallize threshold for C-25 (today an arbitrary 3-successes); "validation
first-pass rate per gate" as an AUDITING metric; the SAFETY SANDWICH (bounded
invocation -> hallucination range-check -> deterministic decision -> audit trail)
as the canonical bounded-invocation shape for the sidecar protocol + H-17..H-29;
DeBERTa-v3 injection classifier (95.8%/100% in their eval) as the E-37 Input Gate -
another small ONNX model on the existing vector/reranker infra, low risk
(anti-slopsquat: verify the HF model card before adding).

### The honest design critique (anti-sycophancy - the real improvement Pass 5 adds)
The code surfaces what the abstracts could not: EVERY harness here that WORKS is
RADICALLY SIMPLER than ELAI's spec. Agentless's published thesis is literally
"simple non-agentic beats agentic" (SOTA-for-open-source on SWE-bench Lite at
~$0.34/issue). AHE's DISCOVERED winning harness was "one shell tool + one regex
middleware + a verification-discipline system prompt", and its measured datum is a
direct warning to ELAI: adding system-prompt instruction text ALONE REGRESSED
-2.3pp (memory +5.6pp, tools +3.3pp were the wins; MORE INSTRUCTION made the model
WORSE). meta-harness keeps a (accuracy,COST) Pareto - complexity is a cost it
minimizes. Kimi-Dev's scaffold is a minimal two-call pipeline; its win was RL on a
72B model, not harness breadth. ELAI is the opposite shape (550 SSB variants,
~100 planned units, composed GRC+templates+PPM+consensus+adversary+search+
abstention+depth stacks). Pass-1 caveat #3 (layers compose, not multiply) + K-4
gate-tax ceiling already acknowledge this; the code makes it SHARP: payoff
concentrates in a SMALL number of load-bearing primitives, and instruction/
complexity past a point is NEGATIVE-yield on a small model.
ACTIONABLE (a sequencing+pruning discipline, not a feature):
1. Ship the MINIMAL load-bearing kernel FIRST and MEASURE before adding breadth:
   one un-gameable off-agent verifier loop [B.1] + grammar/edit-DSL output
   constraint on local [B.2] + deterministic stagnation counters [B.3] + the
   E-46 Agentless-shape decompose->repair->validate pipeline = ELAI's
   Agentless-equivalent spine.
2. Make SAFEDELTA/G-6 + convergence-curve instruments (I-27/I-38) the GATE that
   decides whether each ADDITIONAL reasoning layer earns its keep: gate-ON-vs-OFF
   ablation per layer, drop negative-yield layers (AHE's -2.3pp says they exist).
   HD-1 (measure per config) promoted from a doctrine line to the PRIMARY
   sequencing rule.
3. Treat the feature map's breadth as a MENU to prune against measured outcomes,
   not a backlog to fully build. "It is designed" is not "it should ship."

### Ceilings the code reconfirms (do not over-claim past them)
MODEL-CAPABILITY FLOOR: the harness is not a model substitute (SWE-Effi GPT-4o-mini
5.1% vs Qwen3-32B 21.8% same scaffold; Kimi-Dev's win was RL on 72B). ELAI's
local-lane bet rests on Qwen-7B being above the floor - plausible (100% on the
short set) but UNPROVEN for ELAI's decompose shape. MULTI-FILE GLOBALITY: Agentless
made 6 multi-file patches of 497 successes; agentic BEATS deterministic on
cross-file (OpenHands 25.3% vs Agentless 18.3%, arXiv:2509.13941) - the Focus
Tunnel narrows the MODEL's job, not the TASK's globality. "Decomposition recovers
the long-horizon gap for a sub-32B model" stays a DESIGN HYPOTHESIS this code
corpus supports mechanistically and does NOT measure. GRAMMAR DECODING is
local-only + sampling-not-semantics.

### What NOT to do (already in the Section-4 rejections ledger)
Temporal.io substrate (Compiled AI uses it) - self-REJECTED for local. The
constrained-gen alternatives (outlines/guidance/lm-format-enforcer/syncode-as-
framework) - REJECTED for settled XGrammar + planned llguidance (I-34); syncode is
a PATTERN reference, not a dep. Force-JSON on reasoning roles - BANNED (HD-3).
Same-model self-critique / blake3-bytes fitness / keyword-grep trap - BANNED
(VD-1/VD-2/D-13); B.1 REPLACES these. live-swe-agent's "make the model tiny"
extreme - refuted by the model floor; keep the SMALLEST IRREDUCIBLE kernel.

### Pass-5 bottom line
The source code does not reveal a capability ELAI lacks - it CONFIRMS, at code
level, that ELAI's design targets the three primitives every working harness here
converges on. Improvement = (1) de-risk the three load-bearing UNBUILT items with
these repos' working reference code [B.1 top priority, B.2, B.3], (2) add the cheap
wins [C], (3) adopt the minimality discipline the code makes undeniable [D]: ship
the minimal kernel first, gate every added layer on a measured delta, prune the
negative-yield ones. The believe-the-number SAFEDELTA run (G-6) is still the thing
that turns this whole design from "mechanism-sound bet" into "proven."

================================================================================
## PASS 6 — three memory papers, code-grounded: the ONE real gap is memory
## STALENESS, not memory retrieval (2026-06-15)
================================================================================
Triggered by the user dropping three memory papers (LongMemEval-V2 benchmark;
STALE arXiv:2605.06527; MemPro arXiv:2606.00619) for a steal-evaluation against
ELAI's cross-task memory stack. spec-eval mode holds (judge the design, track
readiness separately). Both papers ingested at SOURCE level by paper-ingestor
(workflow/papers/stale-2605.06527-interface.md + mempro-2606.00619-interface.md,
arXiv ids WebSearch+WebFetch-verified); the live ELAI supersession path was
code-read via codegraph (crates/memory/src/contradiction.rs). Section-4 ledger
checked first: none of the three is already rejected/planned by id; A-MEM/GEM/
MPBench (J-21's cites) are siblings; EvoArena R3 typed-transition rider exists;
"keep superseded rows in context" is a BANNED pattern (Graphiti +68%-token trap).
Standalone steal artifact NOT authored (user chose: deepen + add a pass here, no
new plan unless it survives grounding — it did, as ONE gap, recorded below).

### Headline: all three land on Q2; two confirm, one names a real, code-confirmed gap
This is the Q2 (cross-task memory) corpus, and it behaves like the Pass-4/Pass-5
runs: the design is aimed right and the lever is sharpening + activation, not a
missing capability — with ONE exception that the live code makes concrete.

### The gap — STALE (arXiv:2605.06527), the strongest finding, confirmed from code
STALE's contribution is the distinction ELAI's memory does NOT currently make:
IMPLICIT conflict (a new observation entails the old belief is wrong with NO
explicit negation) vs explicit contradiction, split into two subtypes —
  Type I (co-referential): same attribute, incompatible new value.
  Type II (PROPAGATED): m_n updates attribute A, which through a dependency
    invalidates attribute B that m_n never mentions ("moved to Arizona" silently
    invalidating "commutes by bike"). Catching this REQUIRES schema-guided
    dependency traversal — pairwise NLI provably cannot.
And the empirical core (LightMem diagnostic, verbatim from the paper): the right
updated evidence is in top-20 for 77.5% of cases, yet only 3.3% of old entries
get marked for update, so stale + current coexist unadjudicated and the model
fails at 56.1% (state-resolution) / 99.0% (premise-resistance) / 78.6% (policy-
adaptation) of those very cases. i.e. retrieval quality is NOT the bottleneck;
WRITE-SIDE ADJUDICATION is. Best plain frontier model 55.2% overall; CUPMem
(their fix) 68.0%; every other memory framework <=17.8%.

CUPMem's mechanism (note: WRITE-time, not read-time — corrects my pre-ingest
guess): at write, build a propagation-aware candidate set
C_t = Direct(touched slots) U Affected_theta(schema-dependent slots) U Global_k
(semantic fallback); adjudicate each entry into a 4-STATE typed verdict
{KEEP, STALE, REPLACE, UNKNOWN}; at read, do a CONSTRAINED readout gated by those
verdicts — active facts ground generation, STALE facts are history-only, UNKNOWN
BLOCKS the generator from using the old value as a premise. Data structure: a
two-level state schema Omega (domains -> slots) that supplies the dependency graph.

What the live ELAI code actually does (codegraph, contradiction.rs:64-160):
  - audit_conflicts() groups currently-valid rows by EXACT (project_id, scope,
    key) and only compares same-key pairs. There is NO state-schema dependency
    graph and NO Affected_theta leg -> Type-II propagated conflict is STRUCTURALLY
    uncatchable today.
  - the verdict enum ConflictAction is 3-state (Surface / SupersededByTier /
    NoOp), and supersession is driven by TRUST-TIER PRIORITY comparison
    (newer.trust_tier_priority() > older), not by semantic adjudication of whether
    the new observation invalidates the old. Equal-tier conflicting facts (the
    common case: two T1 or two T0 rows) just Surface and COEXIST — the exact
    LightMem failure mode. There is no UNKNOWN -> block-as-premise state, so no
    premise-resistance and no tie into D-16 conformal refusal.
  - the separate F-12 DeterministicNliJudge (live_cli post-reflect tick) catches
    semantic Type-I contradiction but is still pairwise over facts, not a schema
    traversal — it does not close the Type-II gap either.

So ELAI's H-31 instinct is VALIDATED by CUPMem (write-time supersession beats the
retrieval-heavy frameworks that score <=17.8%), but its current mechanism is
incomplete in two specific, code-confirmed ways: (a) no state-schema dependency
graph -> misses Type-II; (b) binary/tier verdict with no UNKNOWN-blocks-premise ->
no premise-resistance, no D-16 tie. This is a genuine EXTENSION of H-31/E-5 over
the contradiction.rs substrate, composing D-16 (UNKNOWN -> abstain over confident-
stale) and J-21 (STALE's state-schema is the non-code analog of J-21's "re-derive
a fact against commit+AST identity"; J-21 already cites GEM 2605.26252 for the code
case — STALE generalizes it to user/world facts). H-30 (the instrument-first
keystone) gets its missing concrete eval: STALE's 3-axis taxonomy (State-Resolution
/ Premise-Resistance / Implicit-Policy-Adaptation) is exactly the belief-revision
eval H-30 needs to be MEASURED rather than synthetic. VALIDATED (code-confirmed gap
+ verified paper mechanism); the only EXTRAPOLATION is that schema-propagation pays
off on ELAI's workload — falsifiable on the STALE corpus itself.

### MemPro (arXiv:2606.00619) — confirmation-by-NEGATIVE, a DO-NOT-COPY exhibit
MemPro makes the whole memory construction-retrieval pipeline an evolvable program
(version tree of runnable pipelines; an Evolving Agent selects/diagnoses/refines).
The idea rhymes with ELAI's spine (freeze model, evolve harness), but the SOURCE
read shows its fitness is the precise anti-pattern ELAI already bans: S_v is an
LLM-as-Judge score with the SAME model (gpt-4o-mini) judging and generating, NO
execution-grounded oracle, NO formal failure taxonomy (the "failure summary" A_v
is free-text LLM output), and the offline evolution cost (up to ~300 gpt-5.4 calls
at T=15/J=20) is EXCLUDED from the headline token numbers. That violates VD-1
(fitness execution-grounded, never self-confidence), VD-4 (verifier stronger-than-
generator + cross-family), HD-1 (measured-per-config, never a cost headline that
hides the overhead), and it is exactly the loop the H-Track-7 convergence theorem
says PROVABLY collapses without an un-gameable filter. Verdict: NOT a unit and NOT
ported. If ELAI ever evolves memory-pipeline CONFIG, it rides the existing self-mod
ladder (AllowedConfigKey + 10/50/200 staged eval, B-23) under H-43 execution-
grounded fitness + H-47 eval-firewall + a cross-family judge — i.e. the exact three
things MemPro lacks. The only positive transfers are minor confirmations of shipped
doctrine: "deploy argmax over the tree, not the latest node" and "tree beats chain,
multi-step edit beats single edit" echo H-45/H-46 lineage-diverse promotion; the
hidden-evolution-cost caveat is a DO-NOT-COPY honesty exhibit alongside the goose
cost-display entry. Rider on the Section-4 ledger, not a plan.

### LongMemEval-V2 (xiaowu0162) — adopt as a memory BENCHMARK + one missing axis
A memory benchmark (451 Q, haystacks <=115M tokens, web/enterprise). Two useful
pieces: its 5-capability taxonomy (static-state / dynamic-state / workflow-knowledge
/ environment-gotchas / PREMISE-AWARENESS — note premise-awareness independently
converges with STALE's Premise-Resistance, two papers naming the same failure) as an
eval corpus for H-30/E-9/C-56; and its accuracy-vs-LATENCY metric, a real axis
ELAI's memory design lacks — ELAI measures memory TOKEN cost (the +68% trap) and
context-FILL (J-34) but never retrieval latency-vs-accuracy. Its AgentRunbook-C
(code/runbook-as-memory) is direct prior art for I-12 (evidence-gated playbook
curator -> CLAUDE.md/AGENTS.md/skills). Riders on H-30/E-9/C-56/I-12, not a unit.

### Pass-6 bottom line + the one open fork for the user
Net of the three papers: the memory DESIGN is again at/beyond the comparators, with
exactly ONE concrete, code-confirmed gap worth building — STALE-style staleness
adjudication: a state-schema dependency graph (closes Type-II propagated conflict,
which audit_conflicts cannot reach today) + a 4-state {KEEP/STALE/REPLACE/UNKNOWN}
verdict replacing the 3-state tier-only model, with UNKNOWN wired to D-16 refusal
(premise-resistance). It is an EXTENSION of H-31/E-5 composing D-16/J-21/H-30, not a
new subsystem; everything else is benchmark-grounding (LongMemEval-V2 + STALE's eval
into H-30) or a DO-NOT-COPY confirmation (MemPro's gameable fitness). This does NOT
move the standing priority: the highest-leverage memory work remains ACTIVATING
H-50/H-51 + the crates/memory wiring (get_relevant_facts still has only report_fns
+ test callers; the whole substrate is PARTIAL/DARK) — a staleness adjudicator on an
unwired memory layer is premature. OPEN FORK (user's call): author the staleness-
adjudication finding as a small EXTENSION plan unit on the J/K family (vertical
slice: state-schema + 4-state verdict + D-16 tie + the STALE corpus as its done-gate
eval), OR keep it as this pass + a Section-4 ledger rider until the memory substrate
is wired. Recommendation: ledger-rider now, promote to a unit at the same time
crates/memory gets its activation pass, so the adjudicator lands on a live store.

================================================================================
## PASS 7 — ponytail (code-minimalism ruleset): right GOAL, weak MECHANISM;
## the user's "infra not a skill" instinct is exactly ELAI doctrine (2026-06-15)
================================================================================
Triggered by the user dropping github.com/DietrichGebert/ponytail with two
worries: (1) it "wouldn't work in every scenario", (2) baking it into the harness
means "the models will always work this way" — and the lean "maybe not a skill,
could be done in the infra". spec-eval mode. No code ported; this is a category
verdict, consistent with the Section-4 skills-repos ledger.

### What it is
A coding-minimalism BEHAVIORAL RULESET: an ordered decision ladder (YAGNI ->
stdlib -> native platform -> existing dep -> one-liner -> minimal custom; "stop at
the first rung that holds"), shipped as PROSE — AGENTS.md, .cursor/rules/, skills
(/ponytail /ponytail-review /ponytail-audit), hooks — across 11 agents. JS/Node,
nothing portable code-wise. Benchmark (80-94% less code, 3-6x faster, 47-77%
cheaper) is on FIVE trivial single-function tasks (email validator, debounce, CSV,
timers, rate limiters) — exactly the cases where one-liner/stdlib trivially wins.

### The goal is correct and ALREADY ELAI doctrine
"Prefer the simplest rung, do not over-engineer" is maintainability-review.md
dimension 0 (structural simplification) + the Pass-5 finding that every working
harness in the 9-repo run was RADICALLY SIMPLER than ELAI's spec (Agentless's
thesis is literally "simple non-agentic beats agentic"). No disagreement on intent.

### The mechanism is the WEAK form ELAI deliberately rejected (two grounded reasons)
- It enforces by prose the model must follow. That is precisely the Section-4
  skills-repos verdict (mattpocock/skills, cursor team-kit, obra/superpowers):
  "PURE PROSE, no runtime gate ... ELAI's enforcement is OUTSIDE the context window
  (FSM/rules/VibeGate/wiring gates); it does NOT depend on the model following .md
  prose." 
- Hard datum against prose rules: AHE (arXiv:2604.25850) measured that adding
  system-prompt instruction text ALONE REGRESSED -2.3pp on Terminal-Bench 2 (memory
  +5.6 / tools +3.3 were the wins; MORE INSTRUCTION made the model worse). A global
  minimalism preamble is exactly that kind of always-on text.

### The skill-vs-infra question resolves the user's "always works this way" worry
The distinction the ponytail packaging conflates and the user correctly separated:
  - PROSE-IN-PROMPT = an always-on GENERATION-time bias (the bad, AHE-regressing
    thing; also wrong on the tasks below).
  - OFF-AGENT AUDITOR GATE = fires ONLY when a diff crosses a boundary, never
    touches how the model generates, can't be gamed, main model never runs it.
ELAI already does the latter: MR-1..MR-4 (1000-line boundary, long-fn, branching
density, as-any/cast flags) + the qualitative catalog, auditor-applied + frozen in
the gate manifest. So "in the infra, not a skill" is exactly right — and done that
way it does NOT make the model "always work this way"; it rejects over-engineered
OUTPUT after the fact. That is the line.

### "Wouldn't work in every scenario" — grounded, and it's why always-on is wrong
The 5-trivial-task bench says nothing about multi-file/global tasks where ELAI's own
grounding (Pass-5: SWE-EVO 0.32 ratio, Constraint Decay -30pp) shows structure is
REQUIRED. An always-on YAGNI bias would fight decomposition, the production-wiring
requirement (preamble R-8: every new pub fn needs a real caller — YAGNI says "you
aren't gonna need it"), and structured SaaS scaffolds (E-12). A per-diff gate that
fires on violation has no such conflict. So a GLOBAL minimalism prose is wrong; a
gate is right — the user's worry is correct and is itself the argument for the gate
form over the prose form.

### The one genuine steal (small; rider, not a plan)
Ponytail's ORDERED reuse ladder is more specific than ELAI's current catalog, and
the top rungs are partly mechanizable in the INFRA the user is pointing at:
  - reuse-first ("existing dep / existing helper before custom") = a PRE-GENERATION
    DKG/retrieval check ("does a symbol or installed dep already do this?") over the
    shipped B-29 impact API + J-18 retrieval, instead of a prose reminder. The real
    infra embodiment; partly maps to maintainability C-MAINTAINABILITY-DUP.
  - stdlib-over-custom / native-platform = explicit auditor maintainability LENS
    items, advisory (TD-3, irreducibly judgment), folded into the off-agent catalog.
DO NOT add it as an always-on generation prompt (AHE -2.3pp).

### Pass-7 bottom line
Don't import the plugin or the skill. Fold the one sharpening as a rider on
maintainability-review (reuse-rung auditor lens) + a DKG pre-gen "existing-impl?"
check (B-29/J-18). Matches the skills-repos ledger posture exactly: ELAI owns the
"is it maintainable / is it over-built" axis OFF-AGENT and FROZEN; it does not
delegate that axis to .md prose the model may or may not honor.

================================================================================
## PASS 8 — the mkdir fumble: a visceral proof of the tool-authority thesis
## (Claude Code, observed 2026-06-15)
================================================================================
Triggered by the user showing a Claude Code transcript (D:\.liteGAME_workflow root,
an adversarial-auditor running a ScatterField/GrassRenderer bake-invariants stress
test). Captured as a concrete grounding for the I-46 / Focus-Tunnel tool-authority
thesis (Pass 3) — the cheapest possible demonstration of why "harness holds tool
authority" pays.

### The observation
To create ONE directory (workflow/audit-scratch), the model burned THREE Bash
calls, two of them failures, because it guessed the shell wrong twice:
  1. `if not exist "..." mkdir "..." echo exists`  -> exit 2, bash syntax error
     (CMD syntax run in bash).
  2. `New-Item -ItemType Directory -Force -Path "..." | Out-Null; Write-Host ...`
     -> exit 127, "New-Item: command not found" (PowerShell run in bash).
  3. `mkdir -p "D:/.liteGAME_workflow/workflow/audit-scratch" && echo "ok"` -> ok
     (POSIX, finally).

### The cost (honest accounting)
~2 wasted round-trips. Each is a full model turn (emit -> execute -> read error ->
re-reason), so roughly 400-1200 wasted tokens (two IN commands + two OUT error
blocks + the recovery reasoning) and 2 extra SEQUENTIAL latency round-trips on the
critical path before any real work. The larger hidden cost is self-conditioning:
two failures now sit in context, and "once their own error is in context,
P(next error) rises" (Pass-1, arXiv:2511.14777). Pure self-inflicted pollution for
a mkdir. Root cause: the model does not know the platform/shell and GUESSES; it is
HOLDING tool authority for a deterministic op and getting it wrong.

### Why ELAI's design removes this class (the same inversion as ponytail + STALE)
1. The harness owns deterministic, platform-aware file/dir ops. TOOLS LAYER [WIRED]
   = git2 + typed file operations in Rust; system_probe.rs already knows it is
   win32. "Ensure dir X" is a deterministic harness op, NEVER a model-guessed shell
   string — so CMD-vs-PowerShell-vs-POSIX cannot be guessed wrong because the model
   never emits the shell.
2. Tier-C agents have NO raw shell [WIRED core] — minimal OS processes, no
   net/LLM/tools, tools dispatched TO them via typed postcard IPC. An ELAI auditor
   running this very stress test would request a registered file-op, not run mkdir
   bash. This is Pass-3 made literal: harness holds tool authority, model emits only
   intent; the mkdir fumble is precisely a tool-authority failure.
3. The targeted belt-and-suspenders for a STRAY bare command — I-46 BASH
   INTERCEPTOR + FIXUP [PLANNED] — is a frozen deterministic fixup table (bare
   command -> REFUSE-WITH-REDIRECT to the exact registered-tool call). A wrong-shell
   mkdir is exactly its job.

### Readiness (spec-eval, separate from the design verdict)
Load-bearing legs shipped: the deterministic platform-aware Tools layer + Tier-C
no-shell isolation already remove this class for AGENT work. Not yet wired: (a) the
I-46 interceptor that would catch a stray bare command in the interactive loop and
redirect it; (b) the interactive main loop still permits free bash the way Claude
Code does here. So as an architecture ELAI removes the wrong-shell-mkdir class by
never letting the model hold shell authority for a deterministic op; the file-ops +
Tier-C legs are wired, the stray-command interceptor (I-46) is planned.

### Pass-8 bottom line (the through-line for Passes 5-8)
ponytail (don't over-build), the STALE adjudicator (don't act on stale memory), and
this mkdir fumble (don't guess the shell) are the SAME principle: push authority out
of the model's generation loop into the deterministic, platform-aware harness, and
leave the model only the irreducible content. The mkdir image is the most visceral
proof of it — three tool calls and a polluted context for an operation that should
cost zero model decisions.
