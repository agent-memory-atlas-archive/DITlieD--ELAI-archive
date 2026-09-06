# LLM reasoning quality for ELAI

Scope: 2024-2026 SOTA on CoT faithfulness, test-time compute scaling, tree search, reasoning verifiers, and when more reasoning hurts. Critique of ELAI's 6-layer pipeline against the literature. Sources are arxiv-anchored where possible. PDFs in ../papers/.

## TL;DR

- L4 AB-MCTS N=3 depth=3 is grounded (Misaki et al. arxiv:2503.04412 plus Sakana TreeQuest results), but the value-function gap is the load-bearing risk: AB-MCTS in the paper uses external feedback signals + Bayesian posteriors, while ELAI uses `mean_logprob + cosine` (GG intrinsic). Replace or augment with a verifier-style signal — process-reward style scoring or AB-MCTS Thompson sampling — before trusting selection. (arxiv:2503.04412, arxiv:2502.18581 self-certainty)
- L2 Micro-Reflection on a 7B is in the danger zone. Multiple 2025 papers show small/distilled reasoning models flip *correct* first answers to wrong ones during reflection: "First Try Matters" (arxiv:2510.08308) shows reflections are predominantly confirmatory and rarely flip wrong→right; ARLCP shows 7B reflection harms accuracy. ELAI's L2 should be gated by a confidence check on the L1 answer first; never auto-reflect when L1 is high-confidence.
- L3 Interrogation Tree (multiple-choice questions stored as data) is well-aligned with Self-Correction Bench (arxiv:2507.02778) finding: LLMs correct external-source errors but cannot correct their own — externalizing the question as a stored MCQ converts intrinsic self-critique into extrinsic verification, which IS supported. Keep it; consider promoting it ahead of L2 for low-confidence cases.
- L1 deterministic 60-70% capture rate is plausible-to-optimistic. The literature has no head-to-head number for "fraction of LLM code errors fixable by rule-based pattern match" — treat the 60-70% as an internal measurement, not a literature-grounded claim, and re-verify per release.
- Add a NEW intermediate layer between L1 and L2: a single-shot answer-confidence check (logit margin / self-certainty per arxiv:2502.18581) that decides whether to fire L2 at all. Inverse-scaling-in-TTC (arxiv:2507.14417) is the strongest argument for not always escalating.

## Key sources

| ID / URL | Title | Takeaway |
|---|---|---|
| arxiv:2307.13702 | Lanham et al., Measuring Faithfulness in CoT (Anthropic 2023) | CoT can be unfaithful; bigger models produce *less* faithful CoT on most tasks; perf boost is not purely from added test-time compute. |
| arxiv:2510.04040 | FaithCoT-Bench (ICLR 2026) | First instance-level unfaithful-CoT benchmark; counterfactual/logit/judge methods all leave gaps; detection is harder on advanced models. |
| arxiv:2510.01069 | Typed CoT / Curry-Howard | Typed Faithfulness Certificates + Certified Self-Consistency: 69.8% on GSM8K vs 19.6% baseline (same sampling budget). |
| arxiv:2410.21333 | Mind Your Step (by Step) | CoT *reduces* accuracy up to 36.3% absolute on implicit statistical learning, visual recognition, exception-pattern classification. |
| arxiv:2506.07142 | Wharton Prompting Science Report 2 | On modern reasoning-tuned models, CoT prompting is marginal or hurts; reasoning models do CoT internally; extra "think step by step" buys variance. |
| arxiv:2602.12113 | ARLCP (ICLR 2026) | DeepSeek-R1-Distill-Qwen-7B: 35.0% length reduction + 2.7% acc gain when reflection is penalized; reflection tokens negatively correlate with accuracy. |
| arxiv:2510.08308 | First Try Matters | 8 reasoning models, 5 math datasets: reflections are *confirmatory*; gain comes from first-answer accuracy; flips wrong→right are rare. |
| arxiv:2507.14417 | Inverse Scaling in Test-Time Compute (TMLR 2025) | Extending reasoning length deteriorates accuracy in 4 task categories; Claude becomes distractor-prone, o-series overfits framing, all degrade on deduction. |
| arxiv:2502.10954 | Learning to Stop Overthinking at Test Time | Easy problems overthink at 2K tokens, hard at 8K; inverted-U accuracy curve vs CoT length. |
| arxiv:2503.04412 | AB-MCTS (Misaki/Sakana 2025) | Bayesian posterior decides wider vs deeper; combining o4-mini + Gemini-2.5 + R1-0528 beats individual models on ARC-AGI-2. |
| arxiv:2305.14992 | RAP (Hao et al., Reasoning via Planning) | LLM as world model + MCTS; foundational for layered-reasoning trees. |
| arxiv:2310.04406 | LATS (Language Agent Tree Search) | MCTS + value function + self-reflection: 92.7% pass@1 on HumanEval with GPT-4. |
| arxiv:2310.01798 | LLMs Cannot Self-Correct Reasoning Yet (DeepMind) | Intrinsic self-correction often *degrades* reasoning unless external feedback present. |
| arxiv:2507.02778 | Self-Correction Bench (ICLR-era 2025) | 14 non-reasoning models, 64.5% blind-spot rate; LLMs correct external errors but miss their own. |
| arxiv:2305.14325 | Du et al. Multi-Agent Debate | Original "Society of Minds" debate paper. |
| arxiv:2502.08788 | Stop Overvaluing Multi-Agent Debate (2025) | Vanilla MAD often underperforms simple majority vote despite higher cost; needs diversity + calibrated confidence to beat. |
| arxiv:2502.18581 | Self-Certainty (BoN selection) | Replaces external verifier with model's own logit-derived self-certainty for best-of-N selection. |
| arxiv:2504.04823 | Quantization Hurts Reasoning? | W8A8 / W4A16 lossless for reasoning models; lower bit-widths (incl NF4 territory) introduce significant accuracy risk. |
| arxiv:2505.23422 | CTIM-Rover (REALM 2025) | Adapted CTIM episodic memory drops AutoCodeRover from 42%→40%; episodic NL memory introduces noise that small models can't filter. |
| arxiv:2501.12948 | DeepSeek-R1 technical report | Distilled-Qwen-7B: MATH-500 92.8%, AIME-2024 55.5%; distillation transfers reasoning patterns from large RL'd teacher. |
| Anthropic 2025 introspection | transformer-circuits.pub/2025/introspection | Claude shows *limited and unreliable* introspective awareness via activation injection — CoT-as-self-report is not ground truth. |
| arxiv:2510.01500 | Lateral ToT (LToT) | Standard ToT suffers breadth saturation + depth myopia; LToT separates utility from logical consistency, keeps "low-utility but consistent" candidates. |
| arxiv:2408.03314 | Snell et al., Test-Time Compute Scaling | Compute-optimal inference; smaller model + optimal TTC beats bigger model on a flop-matched basis for some regimes. |
| aclanthology:2025.acl-long.1230 | PRMBench (ACL 2025) | 6,216 problems / 83,456 step labels; PRMs evaluated on simplicity / soundness / sensitivity; Qwen2.5-Math-PRM-7B SOTA open PRM. |

26 sources cited.

## When to use what reasoning technique

| Situation | Technique | Grounded support | Local-7B-friendly? |
|---|---|---|---|
| Deterministic error pattern, syntax-level | Rule-based / FailureTaxonomy (L1) | No literature head-to-head, but inverse-scaling-TTC (arxiv:2507.14417) and Mind-Your-Step (arxiv:2410.21333) say *do not* invoke CoT when a deterministic rule suffices | YES — zero LLM calls |
| Single-attempt code repair, ambiguous bug | Compressed-context single-shot (L2 style) | Wharton 2506.07142: CoT marginal on tuned models | YES — 1 LLM call, NF4 OK if W4A16 only (arxiv:2504.04823) |
| Self-doubt / "did I make a mistake somewhere" | Externalized MCQ / Interrogation (L3-style) | Self-Correction Bench (arxiv:2507.02778) confirms LLMs correct *external* errors fine, *intrinsic* poorly | YES — externalized check is short |
| 3 hypotheses failed, need branching exploration | AB-MCTS or LToT, N=3-15, depth ≤3 | arxiv:2503.04412 (AB-MCTS), arxiv:2510.01500 (LToT — keeps low-utility consistent branches) | RISKY — 7B value function unreliable; needs PRM or self-certainty verifier (arxiv:2502.18581) |
| Need calibrated confidence over alternative answers | Self-consistency with N≈10-15 | Self-consistency saturates around N=10-15 (Gemini-2.5 evidence); LToT extends usable budget | Maybe — 10x cost on 7B local |
| Reasoning trace must be auditable | Typed-CoT / TFCs | arxiv:2510.01069 — 50.3% improvement on GSM8K with same sample budget via typing constraints | YES — typing constraints are cheap |
| Long deductive chain, many constraints | DO NOT extend CoT; structured external scratchpad | arxiv:2507.14417 — all models degrade on deductive chains with more compute | YES via external scratchpad, NO via long CoT |
| Implicit statistical / pattern-with-exceptions | Direct answer, no CoT | arxiv:2410.21333 — up to 36.3% CoT penalty | YES |

## Faithfulness of reasoning traces

What CoT *reveals*:
- Functional decomposition of the prompt — what sub-questions the model produced. Useful as a planning artifact even when not causally faithful (Lanham, arxiv:2307.13702).
- Surface-level rule applications. Typed CoT (arxiv:2510.01069) makes the "what rule was applied" question first-class via TFCs.

What CoT *hides*:
- The actual computation. Lanham (arxiv:2307.13702): models often produce answers that survive paraphrasing or even mistake-injection in the CoT — the CoT is post-hoc rationalization.
- Self-correction blind spots. Self-Correction Bench (arxiv:2507.02778) shows 64.5% blind-spot rate: models who can fix an externally-supplied wrong step can't see the same wrong step in their own trace.
- Confirmation bias. "First Try Matters" (arxiv:2510.08308): reflection is mostly confirmatory; rarely flips wrong→right.
- Bigger models hide *more*. Lanham shows faithfulness *decreases* with model scale on most tasks.

Anthropic's 2025 introspection work (transformer-circuits.pub/2025/introspection) confirms: even when Claude can "notice" injected concepts in activations, the introspective signal is unreliable and limited — CoT cannot be trusted as a window into the model's actual reasoning.

For ELAI specifically: the MistakeBook (structured forbidden-diff list) is the right move because it externalizes verification — but the L2 micro-reflection that *generates* candidate diffs must not be treated as faithful self-critique. It's better framed as "structured re-sample with prior" than as "the model thinks about its mistake."

## Critique of ELAI's 6-layer pipeline

| Layer | Claim | Grounding strength | Weakness | Suggested adjustment |
|---|---|---|---|---|
| L1 Deterministic | 60-70% of errors caught, sub-100ms, 9 FailureTaxonomy categories | LOCAL (no literature benchmark for this %) | Number is an internal metric, not externally grounded | Re-measure per quarter; track per-category capture rate (some categories saturate, others drift) |
| L2 Micro-Reflection | 20-25%, 1 LLM call JSON-constrained, 500-token compressed context | WEAK against arxiv:2510.08308 ("First Try Matters"), arxiv:2602.12113 (ARLCP shows 7B reflection harms), arxiv:2310.01798 (DeepMind: intrinsic self-correct degrades reasoning) | A 7B "reflecting on a mistake" is the classic failure case — confirmatory reflection that doesn't flip wrong→right | (a) Gate L2 with answer-confidence check from L1 (self-certainty per arxiv:2502.18581). (b) Make L2 a *re-sample with prior* (the MistakeBook is the prior), not a "reflection." (c) Cap reflection tokens hard (ARLCP-style length penalty). |
| L3 Interrogation Tree | 5-8%, 3-7 MCQs stored as data in rules.db | STRONG vs Self-Correction Bench (arxiv:2507.02778) — externalized MCQs convert intrinsic→extrinsic | Question set quality decides everything; stale MCQs become noise (CTIM-Rover, arxiv:2505.23422) | Keep. Add: relevance/recency filter on MCQs at retrieval time. Consider running L3 *before* L2 when L1 confidence is low — it's a cheaper, more grounded escalation. |
| L4 AB-MCTS | 3-5%, N=3 branches at temps [0.7, 1.0, 0.3], MistakeBook forbidden diffs, max depth 3, max 15 branches, GG intrinsic (mean_logprob + cosine) | MEDIUM — AB-MCTS itself is well-grounded (arxiv:2503.04412), but the paper assumes *external feedback*; ELAI uses intrinsic GG | mean_logprob + cosine is a weak value function. Lanham (arxiv:2307.13702): logit signal is not a clean proxy for correctness. arxiv:2510.01500 (LToT) shows breadth saturation can happen at very low N (3 is risky) | Add a process-reward-style signal (PRM-lite using `auditor` agent or a deterministic syntactic verifier) for node scoring. Consider LToT's mainline+lateral split — keep "low-utility but logically consistent" branches one extra ply before pruning. |
| L5 Cloud Diagnostic | $0.002-$0.05, diagnoses only — cloud never generates code | NOVEL design; not directly in literature but consistent with extrinsic-verifier story (arxiv:2507.02778) | Cloud diagnostic still suffers same overthinking pathology if asked to think longer (arxiv:2507.14417: Claude becomes distractor-prone with more compute) | Force cloud diagnostic to short / capped outputs. Prefer Haiku for triage, Sonnet only when L1-L4 deltas disagree. Do NOT let Opus run long deductions — inverse scaling. |
| L6 Cloud Generation | ~0.5-1%, validated by same local pipeline | OK — generation by stronger model, verification by cheaper local pipeline, is exactly the "verifier-generator gap" pattern (arxiv:2502.18581) | If the same local pipeline that failed at L1-L4 verifies L6, the verifier signal is the same noisy signal | Add a fresh-state validation pass: re-run L1 + a *different* L3 question set on L6 output. Verifier diversity matters more than verifier strength here. |

Cross-cutting concerns:

- REPAIR_CONTEXT 1400 tokens per attempt (fresh wipe) is broadly correct: avoids CTIM-Rover (arxiv:2505.23422) noise accumulation and Kimi-Dev's stateless reset is well-validated. Stateless is *better than* episodic memory on small models, which is what literature supports.
- Generation params (0.3/0.9 first attempt, 0.3/0.9 reflection, 0.1/0.9 interrogation): temperature schedule is reasonable. The temp=0.1 for interrogation may be too low if the question is genuinely ambiguous — consider 0.2-0.3 for the MCQ-generation step, 0.1 for the answer-choice step.
- NF4 4-bit on Qwen2.5-Coder-7B is in the "accuracy at risk" zone per arxiv:2504.04823 (W8A8 / W4A16 lossless, lower bit-widths risky). Measure per-layer impact: L1 unaffected, L2/L4 most exposed (require reasoning), L3 partially protected (MCQs are short).

## Best techniques to adopt

Ranked by expected ELAI benefit:

1. **Pre-L2 confidence gate via self-certainty (arxiv:2502.18581)** — integrate at L1→L2 boundary. If L1 already accepted with high logit margin, skip L2 entirely. Cheapest single change with strongest grounding (inverse-scaling-TTC + First-Try-Matters).
2. **PRM-lite scoring for L4 AB-MCTS nodes (PRMBench, R-PRM)** — replace or augment GG intrinsic. Even a 1B distilled PRM beats logprob-cosine for branch selection. Integration: L4 value function.
3. **LToT mainline/lateral split (arxiv:2510.01500)** — within L4: keep N=3 mainlines + N=2 laterals (consistent-but-low-utility), prune laterals after one extra ply only. Integration: L4 expansion policy.
4. **Reflection token cap (ARLCP arxiv:2602.12113)** — hard token limit on L2 output proportional to L1 confidence. Integration: L2 generation params.
5. **Typed-CoT lite (arxiv:2510.01069)** — emit `{rule_applied, types_in, types_out}` triples in L2 output. Cheap; gives wiring-contract-validator something concrete. Integration: L2 JSON schema.
6. **Verifier diversity for L6 validation (arxiv:2502.08788)** — current "same local pipeline" is single-verifier. Add a separately-seeded L1 pass + a fresh MCQ from L3 catalog. Integration: post-L6.
7. **Difficulty-conditional TTC budget (arxiv:2502.10954 + arxiv:2408.03314)** — easy problems get smaller depth/N, hard problems get bigger. Need a difficulty estimator (FailureTaxonomy category × historical retry count is a starting signal). Integration: AB-MCTS hyperparams.
8. **Skip L2 entirely for known no-CoT-helps task classes (arxiv:2410.21333)** — pattern-classification with exceptions, implicit-statistical-learning style fixes. Route directly L1 → L3. Integration: L1 routing logic.

Not adopting:
- Multi-agent debate (arxiv:2502.08788 shows vanilla MAD often loses to majority vote; not worth the cost on local 7B).
- Long-CoT reasoning models for L2 (arxiv:2507.14417 inverse scaling + ELAI's stateless-per-call design conflict with thinking-mode warm context).

## When MORE reasoning hurts

- Tasks where humans do worse with deliberation: implicit statistical learning, visual recognition, exception classification — up to 36.3% CoT penalty (arxiv:2410.21333).
- Distractor-rich prompts: Claude models distractor-prone with more compute (arxiv:2507.14417).
- Long deductive chains: all tested models degrade with more compute on deductive constraint-tracking (arxiv:2507.14417).
- Small reasoning models reflecting on their own answer: ARLCP arxiv:2602.12113 shows 7B reflection token count negatively correlates with accuracy.
- Hard problems near 8K-token threshold: inverted-U; easy problems overthink at 2K, hard at 8K (arxiv:2502.10954).
- Confirmatory reflection: First Try Matters (arxiv:2510.08308) — reflection mostly affirms first answer; rarely flips wrong→right.
- Multi-agent debate without diversity / calibrated confidence: arxiv:2502.08788 — vanilla MAD underperforms majority vote.
- Episodic NL memory on small SE agents: CTIM-Rover (arxiv:2505.23422) drops 42%→40%.

## Anti-patterns

- "Just ask the model to think more" — Wharton 2506.07142 and inverse-scaling-TTC (arxiv:2507.14417).
- "Have the model critique itself" without external signal — arxiv:2310.01798, arxiv:2507.02778.
- Trusting CoT as faithful explanation — arxiv:2307.13702 + Anthropic 2025 introspection.
- Episodic NL memory across tasks for small models — arxiv:2505.23422.
- Aggressive low-bit quantization on the reasoning steps — arxiv:2504.04823 (NF4 is borderline).
- Long deductive CoT — arxiv:2507.14417 hits *all* models, not just small ones.
- Multi-agent vanilla debate as a free-lunch quality boost — arxiv:2502.08788.

## Open questions

1. Empirical L1 capture rate on the *current* Qwen2.5-Coder-7B-abliterated FailureTaxonomy: is 60-70% accurate at this model + this taxonomy version? (Measurable: log + classify 1000 production errors over a week.)
2. Does L2 reflection on a 7B *ever* flip wrong→right in ELAI logs, or is it purely confirmatory like First-Try-Matters predicts? (Measurable: pair every L2 invocation with its L1 candidate; count wrong-L1→right-L2.)
3. Does adding LToT lateral branches at L4 reduce expansions-to-first-hit on ELAI's bug corpus, the way arxiv:2510.01500 shows on GSM-Hard / HumanEval? (Measurable: A/B with same seed, compare ELAI's branch-budget consumption.)
4. Does self-certainty (arxiv:2502.18581) outperform GG mean_logprob+cosine as an L4 value function on the ELAI MistakeBook corpus? (Measurable: rank-correlation on held-out repair pairs.)
5. At what NF4 / W4A16 boundary does L2 reasoning quality collapse on local Qwen2.5-Coder-7B vs the abliterated variant? (Measurable: run L2-only ablation on a fixed corpus across W8A8 / W4A16 / NF4 quantizations — arxiv:2504.04823 protocol.)
