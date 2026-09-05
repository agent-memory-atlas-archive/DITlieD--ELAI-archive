STEAL ANALYSIS — RecursiveMAS (arXiv:2604.25917) + RecursiveMAS-main repo
Date: 2026-06-20
Sources: paper https://arxiv.org/abs/2604.25917 (verified real via WebSearch, HF papers/2604.25917, alphaxiv); repo docs/.source/RecursiveMAS-main (official released inference source, MEDIUM-HIGH reliability).
Interface map: workflow/papers/recursivemas-interface.md ; repo index: docs/.source/RecursiveMAS-main-INDEX.md
Target family checked: K (highest 0_INDEX). Feature map read: 2026-06-19 stamp (fresh).

OUTCOME: ZERO new plan files. The paper's headline mechanism is REJECT (white-box, incompatible with ELAI's cloud wedge, infeasible on the local lane). Its portable arm is ALREADY-SHIPPED (B-39) + already-PLANNED-as-EXTENSION (J-15->D-11). Net result is CONFIRMATION + 3 ledger riders. This matches the precedent of the 2026-06-14, 2026-06-14b, and deer-flow 2026-06-19 runs (confirmation, zero new plan).

================================================================
WHAT RECURSIVEMAS IS (one paragraph)
================================================================
RecursiveMAS casts a multi-agent system as a single latent-space recursive computation. Heterogeneous small open-weight agents (e.g. Qwen3-1.7B planner, Llama3.2-1B critic, Qwen2.5-Math-1.5B solver) are wired by trained "RecursiveLink" adapters: an inner link (R_in(h)=h+W2*GELU(W1*h)) that re-feeds an agent's own last-layer hidden state for its next round, and an outer link (R_out(h)=W3*h+W2*GELU(W1*h)) that bridges one agent's hidden space into the next agent's input-embedding space. Agents exchange raw hidden-state TENSORS spliced into inputs_embeds (inference_mas.py:1140-1141), never decoded text; only the final agent in the final round decodes an answer. Four collaboration shapes: Sequential (planner->critic->solver, feedback edge solver->planner), Mixture (parallel math/code/science experts -> summarizer hub), Distillation (expert<->learner), Deliberation (reflector<->toolcaller with a nested real tool loop). All LLM backbones frozen; only the RecursiveLink adapters are trained (2-stage: cosine-align inner, then end-to-end CE outer).

================================================================
HEADLINE CLAIMS (verified against the paper, with the baseline they were measured on)
================================================================
- +8.3% avg accuracy over strongest baseline (TextGrad) at r=3: MATH500 88.0 vs 85.8, AIME2025 86.7 vs 73.3, GPQA-D 66.2 vs 62.5, LiveCodeBench 42.9 vs 39.8, MedQA 79.3 vs 77.2 (Table 3).
- Efficiency vs Recursive-TextMAS (the SAME recursive structure but with TEXT comms): r=1 1.2x/-34.6% tokens, r=2 1.9x/-65.5%, r=3 2.4x/-75.6% (Section 5.4).
- CRITICAL CAVEAT (load-bearing for the verdict): the 2.4x / 75.6% are measured against the text-comm variant of its OWN recursive MAS, NOT against a single-agent or a non-recursive baseline. The "win" is "latent handoff beats text handoff inside an identical recursive topology." It is not a free 75% token cut over a normal agent.

================================================================
THE DECISIVE PORTABILITY FACT (two independent confirmations)
================================================================
RecursiveMAS is fundamentally WHITE-BOX and train-time. It requires, all together: output_hidden_states=True reads of the last layer (inference_mas.py:858-869), inputs_embeds= injection of a foreign latent (inference_mas.py:859,1504), a hand-rolled autoregressive latent rollout replacing model.generate (inference_mas.py:846-886), trained per-model inner + per-edge outer adapter weights (modeling.py:121-159), and 2-4 full HF models co-resident on one GPU (system_loader.py:161-234). It CANNOT run over text-only Anthropic/OpenAI APIs (no hidden-state access). Confirmed independently by the paper-ingestor (paper text) and the repo mapper (source code).

================================================================
CANDIDATE STEALS
================================================================

S1 — Latent-space inter-agent communication (hidden-state passing; the core mechanism)
  What: replace decoded-text handoffs between agents with raw last-layer hidden-state tensors via trained RecursiveLink adapters. Paper Section 3-4; modeling.py:121-159; inference_mas.py:1024-1141.
  ELAI subsystem touched: ORCHESTRATION / AGENTS (feature-map 1.3); RUNTIME / sidecar.
  Verdict: REJECT. Three independent reasons, each sufficient:
   (a) Cloud wedge incompatibility. ELAI's wedge is trust-evidenced autonomous execution wrapped around CLOUD Sonnet/Opus reached only by text API (feature-map "wedge"; project_focus_rescope_2026). Latent passing needs hidden states the API never exposes. The entire primary lane is structurally ineligible. This is the same class as the existing "latent comms not portable" posture.
   (b) Local-lane infeasibility. The local lane is ONE remote 3090 sidecar with ~1.8GB free VRAM (memory project_gpu_context; project_vram_tier_plan). RecursiveMAS needs 2-4 models co-resident PLUS adapters. ELAI cannot hold planner+critic+solver simultaneously. And it would need TRAINED RecursiveLink adapters per model-pair — ELAI has DEFERRED all adapter/LoRA/RL training (ledger: LoRA DEFERRED catastrophic-forgetting; RL DEFERRED behind 3 unlock conditions; OPRO declined <100B). Reintroducing a per-pair adapter training run violates that settled posture.
   (c) Unproven win over ELAI's actual baseline. The efficiency number is vs Recursive-TextMAS, not vs a single agent. ELAI's existing inter-agent token-reduction levers are deterministic and cache-safe (HD-2 deterministic masking, microcompaction, context-package compression). A latent mechanism that busts determinism and the prompt cache trades a measured ELAI strength (deterministic, replayable, cache-warm cloud) for an unproven local-only gain.
  Aligned existing rejections: "All latent passing ... do NOT attempt to port" (repo INDEX); LLM re-summarization compaction REJECTED busts cache (HD-2); batch-racing REJECTED (IC-1).

S2 — Recursive forward-refine + backward-feedback orchestration topology (planner->refiner->solver, solver feedback re-enters planner, N rounds, only last round emits final)
  What: the round structure independent of latent encoding. inference_mas.py:2386-2488.
  ELAI subsystem touched: CHAT REFLECT PIPELINE (B-39, feature-map:210-212); ORCHESTRATE skill; DETERMINISM STEAL AMENDMENTS (J-15).
  Verdict: ALREADY-SHIPPED (B-39) + EXTENSION already PLANNED (J-15). B-39's CHAT REFLECT PIPELINE is exactly this shape over text: TRIAGE -> REASONER N=4 -> AUDITOR REFINE cap=2 -> SYNTHESIZE. The solver->planner backward-feedback edge is the Logic-LLM solver-error-feedback loop, already stolen and PLANNED into D-11 via J-15 (feature-map:1351-1355, 1694-1695). Re-proposing this re-litigates J-15 — a ledger violation. NO new unit.

S3 — The 4 named collaboration shapes as a topology catalog (sequential / mixture / distillation / deliberation)
  What: a reusable catalog of agent topologies with directed edges. system_loader.py:56-100.
  ELAI subsystem touched: ORCHESTRATION / FSM / AGENTS / BSP (1.3).
  Verdict: ALREADY-COVERED. Sequential = orchestrate chain + B-39. Mixture (parallel specialists -> aggregator) = BSP SUPERSTEPS spawn_parallel + JoinSet (feature-map:258-260) and synthesizer-elai. Distillation (expert->learner) = EXPERIENCE DISTILLATION [DARK ELAI_EXPERIENCE_DISTILL_ENABLE] (feature-map:530) + the cloud-teaches-local tiering; the latent version needs co-resident models + training (see S1). Deliberation (reflector + tool-caller) = CodeAct E-35 + reflect ticks + tool loop. The goose multi-agent-topology intake already established the principle: a static topology catalog is "strictly simpler than ELAI's orchestrate" (ledger:1787-1789). NO new unit.

S4 — Per-edge handoff policy (each directed agent edge A->B has its own specialized transform, e.g. outer_12 / outer_23 / outer_31)
  What: the design idea that a handoff is edge-specific, not a generic dump. system_loader.py:78-83.
  ELAI subsystem touched: CONTEXT-COMPILER / per-role context (feature-map:329); structured returns.
  Verdict: SUBSUMED. context-compiler already tailors context per sub-phase, and ELAI keeps per-role context A/B (auditor/critic/anti-spiral) while dropping it for executor/researcher (feature-map:329, IC-17). "Directed-edge-has-its-own-handoff-policy" over text is just per-edge prompt tailoring, which the context layer already does. Not worth a unit.

S5 — The empirical insight: compressing inter-agent handoffs (not full re-decoded text) cuts tokens 34-75% at no accuracy cost
  What: the paper's theoretical + measured result that intermediate-round text decoding is the dominant cost, and removing it is a large, safe lever (O(m*d_h^2) latent transform vs O(m*|V|*d_h) token decode, Appendix A.1).
  ELAI subsystem touched: deterministic masking (HD-2), microcompaction, context-package compression.
  Verdict: CONFIRMATION / external grounding rider. ELAI cannot pass latents on the cloud lane, but it ALREADY pursues the deterministic, cache-safe version of this lever. RecursiveMAS is third-party grounding that inter-agent text-handoff cost is a real, large, measurable target — it reinforces the existing direction. Recorded as a rider, NOT a build.

================================================================
LEDGER RIDERS (to add to feature-map SECTION 4)
================================================================
R1: RecursiveMAS latent-space inter-agent communication (white-box hidden-state passing + trained per-edge RecursiveLink adapters): REJECTED. Cloud wedge has no hidden-state access; local lane cannot co-resident 2-4 models in ~1.8GB free VRAM and all adapter/RL training is DEFERRED (LoRA/RL ladder); efficiency claim is vs its own Recursive-TextMAS, not vs single-agent. The portable arm (recursive text MAS) is B-39 + J-15->D-11. (steal-recursivemas-2026-06-20.md)
R2: RecursiveMAS 4-topology catalog (sequential/mixture/distillation/deliberation): ALREADY-COVERED — orchestrate+B-39 / BSP spawn_parallel / EXPERIENCE DISTILLATION / CodeAct E-35 respectively; same verdict class as the goose-topology NO-STEAL (strictly simpler than ELAI orchestrate). Do not re-propose as a catalog unit.
R3: RecursiveMAS efficiency result (token-free intermediate rounds cut 34-75% tokens at no accuracy loss) = external CONFIRMATION grounding for ELAI's deterministic inter-agent handoff compression (HD-2 masking / microcompaction / context-package), NOT a new build. Note: the 2.4x/75.6% is vs Recursive-TextMAS, never quote it as an ELAI number (HD-1/ID-5).

================================================================
PLAN IDS CREATED: none (zero NEW/EXTENSION verdicts; all candidates REJECT / ALREADY-SHIPPED / CONFIRMATION).
