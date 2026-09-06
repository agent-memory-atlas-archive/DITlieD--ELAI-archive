# Agentic RL + reward models for ELAI (2024-2026)

## TL;DR
- ELAI's current loop (BAI Thompson sampling on typed-ReasoningTemplate + Qwen2.5-Math-PRM-7B scoring + tripwires) already sits in the "soft-RL via posterior sampling" regime; published work treats contextual Thompson sampling as an RL policy (logarithmic regret), so the prompt/template layer does not need gradient updates to qualify as RL — but it DOES need a verifiable reward to escape reward hacking (RLVR pattern from DeepSeek-R1 / DAPO).
- Do NOT bolt RLHF/PPO onto the 7B base model. The 2025 evidence is mixed: "SFT memorizes, RL generalizes" holds for math/code with verifiable rewards (GRPO/DAPO + RLVR), but on agent traces (SWE-Gym) plain SFT on 491 trajectories already buys +12-14pp on SWE-Bench. Start with SFT-on-rollouts; only escalate to GRPO if the verifiable reward (compile/test/probe) is dense and the SFT plateau is reached.
- The right reward stack for ELAI is PRM (Qwen2.5-Math-PRM-7B for math-shaped trajectories) + a verifiable ORM (compile/test/build outcome) — hybrid Process+Outcome beats either alone (HRM/CRM/VisualPRM all show +5-12pp BoN). DPO/KTO are NOT the right primitive for a step-typed reasoning chain.
- RLAIF (cloud Sonnet as the judge, GEPA-style reflective feedback) is the highest-leverage low-risk addition. GEPA (ICLR 2026 oral) explicitly outperforms PPO/GRPO with 35x fewer rollouts on prompt-level optimization — that maps directly onto ELAI's D-5 template promotion.
- Hard anti-patterns: reward-hacking on PRM step-scores (composite-reward + MONA-style myopic approval needed), DPO's OOD generalization drop (3-7pp per Apple), over-sharpening in test-time training (90% Best-of-N vs 30% TTT on code), and skipping RLVR verifier when reward is learned (Goodhart guaranteed).

## Key sources
| URL | what | takeaway |
|---|---|---|
| https://arxiv.org/abs/2305.18290 | Rafailov DPO original (May 2023) | Verified citation — language model is secretly a reward model; eliminates explicit RM + PPO loop. |
| https://machinelearning.apple.com/research/reward-generalization | Apple 2025 — DPO implicit RM | DPO RM matches RLHF in-distribution, drops 3% mean / 7% max on 5 OOD settings. |
| https://arxiv.org/pdf/2502.20847 | Gradient Imbalance in DPO (Feb 2025) | DPO has structural gradient asymmetry vs RLHF reward signal. |
| https://arxiv.org/pdf/2402.01306 | KTO original (Ethayarajh) | Prospect-theory loss; works with binary thumbs-up/down — no pairwise. |
| https://arxiv.org/abs/2502.14187 | Federated KTO vs DPO (Feb 2025) | KTO (KTOO + KTOR) consistently beats DPO on MT-Bench-1 / Vicuna / AdvBench. |
| https://llm-stats.com/blog/research/post-training-techniques-2026 | Post-Training 2026 (GRPO/DAPO/RLVR) | DAPO drops KL penalty, adds clip-shifting + dynamic sampling + token-level loss for RLVR. |
| https://arxiv.org/abs/2412.21139 | SWE-Gym (ICML 2025) | 491 trajectories, +12.3 / +13.6pp on SWE-Bench Lite/Verified — SFT alone, no RL. |
| https://arxiv.org/abs/2410.01679 | VinePPO (NeurIPS 2024 / ICML 2025) | MC value estimates beat critic-network PPO; 3x wall-clock speedup on MATH/GSM8K. |
| https://arxiv.org/pdf/2501.13011 | MONA (Jan 2025) | Myopic optimization with non-myopic approval prevents multi-step reward hacking. |
| https://arxiv.org/pdf/2509.15557 | Verifiable Composite Rewards (Sep 2025) | RLVR + per-failure-mode penalties for medical reasoning reward-hacking. |
| https://arxiv.org/pdf/2501.07301 | Qwen2.5-Math-PRM lessons (Jan 2025) | MC estimation + LLM-judge consensus filter; PRM-7B > GPT-4o-0806 on ProcessBench. |
| https://qwenlm.github.io/blog/qwen2.5-math-prm/ | Qwen PRM blog | ELAI's PRM choice — design rationale + BoN evaluation methodology. |
| https://arxiv.org/pdf/2507.01352 | Skywork-Reward-V2 (Jul 2025) | 8B RM with 1.8% of curated data beats 70B INF-ORM-Llama3.1; preference-data curation matters more than scale. |
| https://www.nature.com/articles/s41586-025-09833-y | AlphaProof Nature (Nov 2025) | AlphaZero-RL on Lean 4; test-time RL on each problem variant; IMO 2024 silver. |
| https://arxiv.org/abs/2402.03300 / DeepSeek-R1 paper | DeepSeek-R1 GRPO | Group-relative advantage, no critic network, verifiable rewards on math/code. |
| https://arxiv.org/pdf/2507.19457 | GEPA (ICLR 2026 oral) | Reflective prompt evolution; 35x fewer rollouts than PPO/GRPO; Pareto frontier of candidates. |
| https://arxiv.org/html/2508.16546v1 | RL vs SFT panacea-or-mirage (Aug 2025) | Conditional on model capacity + data; SFT 2K can match RL 20K on weak models. |
| https://arxiv.org/pdf/2501.17161 | SFT Memorizes, RL Generalizes (Jan 2025) | RL leverages verification signal → better OOD; SFT distribution-bound. |
| https://arxiv.org/abs/2212.08073 | Constitutional AI / RLAIF | Anthropic original — AI feedback on principles replaces human raters. |
| https://arxiv.org/pdf/2309.00267 | RLAIF vs RLHF (Lee et al.) | Comparable or superior to RLHF on summ + dialogue at fraction of label cost. |
| https://github.com/SWE-Gym/SWE-Gym | SWE-Gym repo | 491 traces from GPT-4o+Sonnet → 32B Qwen-Coder, +14% absolute SWE-Bench Verified. |
| https://arxiv.org/html/2503.06639 | GRPO loss dynamics (Mar 2025) | Theoretical analysis of GRPO's success amplification. |

## RL technique landscape
| technique | mechanism | works on 7B | gain over SFT (verified) | citation |
|---|---|---|---|---|
| RLHF (PPO + RM) | learned RM, PPO update with KL anchor to SFT | yes but 4-model footprint (policy + ref + RM + critic) | ~70% over SFT on summarization (Lee 2023, replicated in RLAIF paper) | arxiv 2309.00267 |
| DPO | closed-form policy gradient from preference pairs; no explicit RM | yes (2-model: policy + ref) | matches RLHF in-distribution; -3pp mean / -7pp max OOD | 2305.18290 + Apple 2025 |
| KTO | prospect-theory loss on binary signals | yes | beats DPO on MT-Bench-1 / Vicuna / AdvBench (Feb 2025 federated study) | 2402.01306, 2502.14187 |
| ORPO | odds-ratio normalization, no reference model | yes (1-model) | competitive with DPO at lower compute | aman.ai primer |
| SimPO | reference-free, length-normalized reward | yes | +6.4 AlpacaEval2 / +7.5 Arena-Hard vs DPO | medium DPO-isnt-enough |
| GRPO | group-relative advantage, no critic | yes (DeepSeek-Math 7B was first deploy) | unlocks reasoning on verifiable rewards; basis of R1 | DeepSeek-Math + R1 |
| DAPO | GRPO + drop-KL + clip-shift + dynamic sample + token-level loss | yes | exceeds GRPO on long-CoT reasoning (RLVR settings) | llm-stats 2026 post-train review |
| VinePPO | MC value estimates replace critic network | yes | up to 3x faster than PPO on MATH/GSM8K, better generalization | 2410.01679 |
| RLAIF | AI judge replaces human rater on RLHF loop | yes | matches/exceeds RLHF on summ + dialogue | 2309.00267 |
| GEPA | reflective natural-language feedback evolves prompts (no weight update) | YES, model-agnostic | 35x fewer rollouts than GRPO/PPO at same/better task perf | 2507.19457 |
| Thompson sampling (ELAI's BAI) | Bayesian posterior over arm rewards | n/a — operates above the model | logarithmic regret (Agrawal-Goyal); equivalent to soft contextual RL when arms are templates | 2512.00930 + Wikipedia |

## DPO/KTO/ORPO/SimPO/RLAIF comparison
For ELAI's use case (single 7B local Qwen base, no preference-pair dataset, verifiable rewards available via build/test/PPM, cloud Sonnet available as judge):

- **DPO is the WRONG primitive.** Needs paired (chosen, rejected) data ELAI does not collect, and Apple 2025 shows 3-7pp OOD drop — ELAI's whole point is OOD plan execution.
- **KTO works on unpaired binary signals** (matches the way audit BLOCK/PASS rows naturally land), beats DPO across MT-Bench-1/Vicuna/AdvBench in Feb 2025 federated study. If ELAI ever weight-updates, KTO > DPO.
- **SimPO is reference-free** so it skips the ELAI complication of maintaining a frozen reference 7B; +6-7pp over DPO is the headline gain.
- **ORPO eliminates the reference model entirely** — useful when VRAM is the constraint (matches ELAI's 12GB local ceiling). Tradeoff: less mature than DPO/KTO.
- **RLAIF is the highest-leverage path** because the AI judge (cloud Sonnet 4.6, already wired per memory:project_b37_shipped.md) replaces the human preference-data bottleneck entirely. Anthropic's CAI shows RLAIF can match/exceed RLHF on alignment tasks (2212.08073). For ELAI: cloud-Sonnet-as-rater on local-Qwen rollouts is the GEPA / RLAIF pattern combined, and report 08's "self-evolving prompts" idea lands here.

Winner for ELAI today: **RLAIF-style judge + GEPA-style reflective prompt evolution** > any weight-update method. If/when weights must move, **KTO + RLVR** (binary verifier signals) > DPO.

## Reward modeling SOTA
- **PRM (Process Reward Model)** — scores each reasoning step. ELAI uses Qwen2.5-Math-PRM-7B. Best practice: bootstrap with Monte Carlo + LLM-judge consensus filter (drop steps where MC vote and LLM-judge disagree on first-error location) — Zhang 2501.07301 reports this is the dominant data-quality lever. PRM-7B exceeds GPT-4o-0806 on ProcessBench; Math-Shepherd lineage.
- **ORM (Outcome Reward Model)** — scores final answer only. Skywork-Reward-V2-Llama-3.1-8B is current open-source SOTA across 7 benchmarks. Skywork's lesson: 290K curated pairs (1.8% of 40M raw) beats 70B models — quality > volume.
- **Hybrid PRM + ORM** — wins. HRM and CRM (Conditional Reward Modeling, Sep 2025) fuse step-level + outcome-level. VisualPRM-8B reports +5.9pp BoN over ORM-only. OR-PRM reports +12.5pp BoN absolute on operations research.
- **PRM training pitfalls** — Math-Shepherd-style MC labels are noisy; the consensus filter is non-negotiable. Adaptive MC search (AMCS, Sep 2025) reduces samples on easy steps and concentrates on uncertain ones.

For ELAI: keep Qwen2.5-Math-PRM-7B for math-shaped trajectories; ADD a verifiable ORM (build/test/probe outcome) and combine via weighted average or HRM-style hierarchical fusion. Do NOT train a fresh PRM from scratch — pretrained Qwen + RLVR outcome is the cheaper SOTA path in 2025.

## Inference-time alternative
The 2025 evidence is sharp:
- **Best-of-N + verifier at inference > RL fine-tune** when the verifier is reliable. JitRL paper: code generation BoN-64 = 90% pass, test-time training = 30.6% pass (over-sharpening collapses diversity).
- **AlphaProof model** (Nature 2025) does the opposite: per-problem test-time RL on Lean variants. This works because Lean's verifier is perfect — RLVR with binary ground truth.
- **CMU meta-RL view** (blog.ml.cmu.edu Jan 2025): optimizing test-time compute IS a meta-RL problem; budget allocation between sequential (longer CoT) and parallel (k samples) scaling is the lever.
- **GEPA** explicitly: reflective prompt evolution outperforms PPO/GRPO at 35x lower rollout cost on the prompt-optimization frontier.

Implication for ELAI: every dollar of RL-fine-tune budget should first be spent on (a) better BoN with PRM+ORM verifier, (b) test-time search with the existing template space, (c) reflective prompt evolution. RL fine-tune of the 7B Qwen base only after these saturate.

## BAI vs RL — where ELAI is now
ELAI's D-5 BAI Thompson sampling on typed-ReasoningTemplate promotion is, theoretically, contextual RL. Wikipedia entry on Thompson sampling explicitly calls it "a reinforcement learning policy"; Agrawal-Goyal proved logarithmic regret for the stochastic MAB. So D-5 already does an RL update — at the template-arm level, not the weight level.

Does ELAI need to escalate to gradient updates? The 2025 evidence says NO until two conditions hold:
1. The template space saturates (BAI converges to a single arm and the tripwire stack stops firing, but performance still misses target). At that point the template language is the bottleneck — adding more arms (Pareto-frontier expansion, à la GEPA) is the FIRST escalation, not gradient updates.
2. There exists a dense verifiable reward (compile/test/probe outcome on the inner trajectory, not just the final answer) — DeepSeek-R1's whole thesis. Without that, weight-level RL goes to Goodhart.

Concrete trigger to escalate to gradient (KTO + RLVR via GRPO/DAPO): ≥3 sessions where (a) BAI converged to a stable arm, (b) PPM step-score plateaued above baseline, and (c) outcome-verifier reward is still <target. Before that point: spend rollouts on GEPA-style prompt evolution + better PRM+ORM fusion.

## Best additions for ELAI (if adopting)
Ranked by ROI given ELAI's current state:

1. **RLAIF judge with cloud Sonnet 4.6 as the rater** | Anthropic CAI 2212.08073, Lee 2309.00267 | Hook: D-13 Adversary already cloud-calls; extend to D-5 BAI by feeding Sonnet-rated trajectory scores into the Beta posterior alongside PPM scores. Zero weight updates. Risk: cloud cost; mitigated by sampling.

2. **GEPA-style reflective prompt evolution on templates** | 2507.19457 (ICLR 2026 oral) | Hook: D-5 BAI promotion candidate set — instead of hand-curated templates, generate new templates via Sonnet reflection on bottom-quartile trajectories. Pareto-frontier retention keeps complementary strategies alive. 35x fewer rollouts vs GRPO for same gain.

3. **Hybrid PRM + verifiable ORM in BAI reward** | Skywork-RM-V2 2507.01352 + VisualPRM 2503.10291 + OR-PRM | Hook: D-2 PPM step-scores already wired; ADD outcome verifier (build green / test pass / probe semantic match) and fuse via weighted sum or HRM-style two-tower. +5-12pp BoN in published settings.

4. **MONA + composite-reward guards against reward hacking** | 2501.13011 + 2509.15557 | Hook: tripwire stack — add explicit penalty terms for non-reasoning answers, format gaming, sycophancy on PPM. Composite reward = base_reward - penalty_format - penalty_length - penalty_sycophancy.

5. **VinePPO-style MC credit assignment on agent trajectories** (if/when gradient updates) | 2410.01679 | Hook: sidecar fine-tune phase. Skip the critic network; use rollout MC estimates. 3x cheaper than PPO on math-style reasoning.

6. **SFT-on-rollouts BEFORE any RL** | SWE-Gym ICML 2025 | Hook: collect 200-500 GREEN sessions, fine-tune Qwen 7B LoRA, measure delta. SWE-Gym reports +12-14pp from 491 traces — almost certainly the highest LoRA-budget ROI before any RL.

7. **KTO + RLVR (NOT DPO) for any later weight update** | 2402.01306 + 2502.14187 + RLVR/DAPO | Binary thumbs-up/down on audit BLOCK/PASS rows is the natural ELAI signal. KTO ingests it directly; DPO requires synthetic pair construction.

## Anti-patterns
- **DPO on ELAI's data shape.** No paired (chosen, rejected) corpus; OOD drop 3-7pp (Apple 2025); gradient imbalance vs RLHF (2502.20847). KTO or SimPO instead.
- **Weight-level RL without a verifiable reward.** DeepSeek-R1's whole insight is RLVR. Learned RM + PPO on agent traces invites reward hacking — Lilian Weng's 2024 survey + IR³/BNRM 2026 papers all converge on this.
- **Test-time training without diversity preservation.** JitRL evidence: 90% BoN-64 vs 30.6% TTT on code due to over-sharpening; gradient updates kill the diversity that BoN exploits.
- **Skipping PRM data curation.** Math-Shepherd MC labels are noisy without the MC-vs-LLM-judge consensus filter (Qwen 2501.07301). Bootstrapping a fresh PRM without that filter underperforms the pretrained PRM-7B baseline.
- **Treating Thompson sampling as "not real RL" and adding gradient anyway.** Logarithmic regret guarantee is an RL theorem; if BAI has not yet converged, RL fine-tune is premature optimization.
- **Citing papers you have not read.** 2305.18290 (DPO original) is confirmed real and verified in this report; per memory:reference_b40_papers.md, prior B-plan cites have been wrong — always WebSearch the arXiv ID.

## Open questions
1. **Measurable**: On ≥3 paired ELAI runs (template-X vs template-Y on same prompt), does Sonnet-judge ranking match PPM-PRM-7B ranking? If correlation < 0.6, RLAIF judge is adding signal PPM is missing — proceed; if > 0.85, PPM alone suffices, skip RLAIF cost.
2. **Measurable**: Does composite-reward (PPM step-score + verifiable build/test ORM) lift Best-of-N pass rate by ≥3pp vs PPM-only on a held-out 50-task set? Below 3pp → fusion not worth wiring.
3. **Measurable**: After GEPA-style template evolution for 100 rollouts (cloud cost ≈ $20-50), does BAI's converged arm change AND does the new arm beat the prior arm on a held-out task set by ≥5pp? Below 5pp → template space already saturated; spend budget elsewhere.
4. **Measurable**: SFT-LoRA on 200 GREEN session trajectories — does Qwen 7B local pass rate on a held-out 50-task ELAI bench rise ≥5pp? Below 5pp → SFT-on-rollouts not the bottleneck (likely template/prompt-engineering bottleneck per GEPA).
5. **Open**: At what BAI-convergence horizon does template-arm RL stop returning marginal gains and weight-level KTO+RLVR become preferred? No published curve for this regime; ELAI would need to instrument it.
