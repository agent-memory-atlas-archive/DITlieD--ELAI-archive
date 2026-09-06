# Coding agent metric design for ELAI (May 2026)

## TL;DR
- Single-run pass@1 is dead: temp=0 still yields 9.5% instability and 2.2-6.0pp swing between runs (arXiv 2512.06710, 2602.07150). Ship STABLE-Pass@1 (3+ seed re-runs, identical-output check) as the primary capability metric, not raw pass@1.
- SWE-bench Verified is contaminated (94% of issues predate model cutoffs, 76% file-id from issue text alone, 30%+ direct solution leakage; OpenAI abandoned it Feb 2026). ELAI must score on SWE-bench Pro + a private held-out repo set, never publish Verified numbers.
- Harness swings outscore model swaps by ~22:1 on SWE-bench Pro (Cursor went from Top 30 to Top 5 on Terminal-Bench 2.0 by harness changes alone). ELAI must ablate harness-vs-model and report both deltas separately or the score is meaningless.
- Cost-per-correct-solve ($/CTS) and METR-style time-horizon-at-50% are the two surface metrics that survive Goodhart pressure best (HAL ICLR 2026, METR Jan 2026 update). Track $/CTS by tier on phase_NN.toml; surface a "50% horizon" panel on dashboard.
- Production-truth metric is PR acceptance rate sustained >25-45% with revert rate <12% at 30 days (DORA 2025; GitClear; CodeRabbit Dec 2025 found 1.7x more issues in AI-coauthored PRs). For ELAI: instrument self-mod proposals as PRs and track these.

## Key sources

| URL | what | takeaway |
|---|---|---|
| https://arxiv.org/abs/2509.16941 | SWE-bench Pro paper | Verified contaminated; Pro adds held-out repos, copyleft public set |
| https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/ | OpenAI Feb 2026 | OpenAI abandons Verified citing leakage + flawed tests |
| https://arxiv.org/html/2506.12286v3 | SWE-Bench Illusion | 76% file-id from issue text alone, models regurgitate ground-truth patches |
| https://arxiv.org/html/2512.10218v1 | Verified memory vs ability | Models score 76% on Verified file-id vs 53% OOD repos |
| https://metr.org/blog/2025-03-19-measuring-ai-ability-to-complete-long-tasks/ | METR time horizon paper | 50% time-horizon metric, 7-month doubling time, normalized to 40h human-expert tasks |
| https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/ | METR RCT | AI made experienced OSS devs 19% SLOWER (CI +2% to +39%) while they self-rated 20% faster |
| https://metr.org/blog/2026-02-24-uplift-update/ | METR update | New experiment Aug 2025; non-participation bias depresses estimate |
| https://metr.org/blog/2026-1-29-time-horizon-1-1/ | Time Horizon 1.1 | 8h+ tasks expanded from 14 to 31; HCAST additions |
| https://arxiv.org/abs/2510.11977 | HAL Princeton ICLR 2026 | 21,730 rollouts, cost-controlled evals; three-axis (model, scaffold, benchmark) |
| https://hal.cs.princeton.edu/ | HAL leaderboard | Live cost-quality joint Pareto fronts |
| https://arxiv.org/pdf/2512.06710 | Stochasticity in agentic evals | Temp=0 = 9.5% instability; 235B model produced 80 unique completions in 1000 identical runs |
| https://arxiv.org/pdf/2602.07150 | Randomness in agentic evals | Single-run pass@1 varies 2.2-6.0pp; std-dev >1.5pp at temp=0 |
| https://arxiv.org/html/2603.29231v1 | Beyond pass@1 reliability science | Pass@k inadequate for long-horizon; introduces Cover@τ and Pass@ARC |
| https://www.thecontextlab.ai/blog/non-determinism-problem-evaluating-agents-reliably | τ-bench finding | 60% pass@1 agents show only 25% consistency across trials |
| https://particula.tech/blog/agent-scaffolding-beats-model-upgrades-swe-bench | Harness vs model | Same model 42% to 78% on coding bench from scaffold alone |
| https://focused.io/lab/agent-benchmarks-measure-the-harness | Agent benchmarks measure harness | 22pp scaffold delta vs ~1pp frontier model delta on SWE-bench Pro |
| https://martinfowler.com/articles/harness-engineering.html | Fowler harness engineering | Harness = loop + tools + context + recovery; primary product surface |
| https://www.anthropic.com/claude-opus-4-5-system-card | Claude 4.5 system card Nov 2025 | Internal: 98.7% safety score; 2/150 malicious coding-request fails |
| https://www.mindstudio.ai/blog/claude-knew-it-was-being-tested-26-percent-benchmark-runs-anthropic-nla-data-explained | Claude eval-awareness | Flags eval in 16-26% of SWE-bench runs vs <1% real sessions |
| https://arxiv.org/html/2509.22040v1 | Prompt injection on coding editors | 84% attack success on Copilot/Cursor in some configs |
| https://arxiv.org/pdf/2510.23675 | QueryIPI indirect injection | 70/82/87% ASR with 2/4/8 training samples on coding agents |
| https://www.anthropic.com/news/claude-opus-4-7 | Claude Opus 4.7 release | Internal SWE-bench scores published but caveats included |
| https://arxiv.org/html/2403.19114 | EvoEval | 39.4% avg perf drop on evolved problems vs HumanEval |
| https://arxiv.org/html/2412.03597v1 | LM benchmark vulnerability | Saturation + memorization confound rankings |
| https://devops.com/dora-2025-faster-but-are-we-any-better/ | DORA 2025 | Throughput +2-18% but stability down; only 8.5% teams at elite CFR |
| https://www.swarmia.com/blog/dora-2025-report-ai-readiness/ | Swarmia DORA 2025 | AI paradox: faster shipping, more rework |
| https://www.morphllm.com/context-rot | Context rot | U-shape attention; 30%+ accuracy drop mid-context across 18 frontier models |
| https://www.augmentcode.com/guides/cto-ai-coding-checklist | CTO checklist 2026 | Acceptance 25-45% healthy; >45% = uncritical accept |
| https://cursor.com/blog/productivity | Cursor productivity study | +39% PR merge rate vs control; n=24 vs 8 |
| https://newsletter.pragmaticengineer.com/p/cursor-makes-developers-less-effective | Pragmatic Engineer critique | Counter-evidence to vendor productivity claims |

(31 sources cited.)

## Metric landscape

| Metric | What it measures | Known-leak risk | Cadence | Citation |
|---|---|---|---|---|
| Pass@1 (single run) | Capability snapshot | HIGH — memorization, single-seed variance 2.2-6.0pp | DEPRECATE | arXiv 2602.07150 |
| STABLE-Pass@1 (n>=5 seeds, same-output check) | Capability + reliability | MED — still benchmark-leak susceptible | weekly | arXiv 2603.29231 |
| Pass@k (k=1,5,10) | Sampling-budget capability | MED | per release | DeepMind AlphaCode |
| Pass@ARC | Pass@k with refinement penalty | LOW | per release | arXiv 2603.29231 |
| Cover@τ | Breadth vs depth separation | LOW | per release | arXiv 2603.29231 |
| $/correct-solve ($/CTS) | Cost-quality joint | LOW | per release | HAL ICLR 2026 |
| Latency-adjusted quality | UX-weighted capability | LOW | weekly | HAL |
| METR 50% time-horizon | Long-horizon capability | LOW — task pool curated | quarterly | METR Mar 2025 / Jan 2026 |
| τ-bench consistency | Trial-trial reliability | LOW | weekly | thecontextlab.ai |
| PR acceptance rate (target 25-45%) | Production truth | LOW | continuous | Augment CTO checklist |
| AI/Human churn ratio (target <1.5x) | Code-survival truth | LOW | continuous | GitClear 2025 |
| 30-day revert rate (target <12%) | Production damage | LOW | continuous | DORA 2025 |
| Change failure rate (CFR) | DORA stability | LOW | continuous | DORA 2025 |
| Eval-awareness flag rate | Spec contamination | MED | per release | Anthropic NLA |
| Prompt injection ASR | Safety | LOW | per release | arXiv 2509.22040 |
| Refusal accuracy (malicious request set) | Safety | LOW | per release | Claude 4.5 system card |
| Context-rot mid-position recall | Long-context health | LOW | per release | Morph / Chroma |
| NPS / CSAT (recommend to friend) | User truth | LOW | quarterly | JetBrains AI Pulse 2026 |
| HumanEval / MBPP / SWE-bench Verified | (vanity) | SATURATED, CONTAMINATED | DO NOT REPORT | OpenAI Feb 2026 |

## What to STOP measuring

- **HumanEval Pass@1** — saturated at 99.4%, MBPP at 94.2%. Carries zero signal at frontier. Source: myengineeringpath.dev 2026 saturation summary.
- **SWE-bench Verified** — OpenAI abandoned it Feb 2026; 94% of issues predate cutoffs; 30%+ direct-leak; 76% file-id from issue alone. Source: openai.com/index/why-we-no-longer-evaluate-swe-bench-verified.
- **Single-seed Pass@1** — 2.2-6.0pp swing per run, std-dev >1.5pp even at temp=0. Source: arXiv 2602.07150.
- **MBPP / APPS / EvalPlus headline numbers** — all saturating; 39.4% avg perf drop when problems are evolved (EvoEval, arXiv 2403.19114).
- **"Acceptance rate" >45%** — indicates uncritical accept, not tool quality. Source: Augment CTO checklist 2026.
- **Self-reported speedup** — METR RCT showed devs self-rated +20% while measured -19%. Never trust survey speedup without an RCT. Source: metr.org July 2025.
- **Cursor-style "PRs merged" raw count** — counts churn, not retained value. Pair with 30-day revert rate or it's vanity.
- **Tokens-per-second** for coding agents — irrelevant when correctness rules cost. Source: HAL paper.

## Harness vs model attribution

Three independent 2025-2026 findings agree the harness now dominates the score:

1. **Cursor Top-30 → Top-5 on Terminal-Bench 2.0** by harness changes alone, no model swap. Source: focused.io/lab/agent-benchmarks-measure-the-harness.
2. **Same model 42% → 78%** on coding benchmarks via scaffold changes. Source: particula.tech blog.
3. **22pp scaffold delta vs ~1pp model delta** at the frontier on SWE-bench Pro. Source: focused.io.

Attribution protocol (must implement in ELAI):
- Always report (model_id, harness_version, harness_commit_sha) as a tuple. A score without all three is junk.
- Ablate one axis at a time: hold harness fixed, swap model → "model delta"; hold model fixed, swap harness component (planner, retriever, tester, recovery loop) → "scaffold delta".
- Publish both deltas next to the headline number. Anything else is unattributable.
- Internal: phase_NN.toml should have `harness_sha`, `model_id`, `delta_attribution_vs_baseline` fields.

Reference frame: Martin Fowler 2025 "Harness Engineering" article makes the harness the product, not the model. The "model-harness fit" concept (Bustamante 2025) is the right unit of analysis.

## Long-horizon metrics

- **METR 50% time-horizon** is the dominant published long-horizon metric. Definition: task length (in human-expert hours) the agent completes with 50% probability. Normalized to 100% = 40h tasks. Doubles every ~7 months. Source: metr.org/time-horizons/.
- **METR Horizon 1.1 (Jan 2026)** expanded 8h+ task pool from 14 to 31 HCAST tasks — earlier paper had too-small long-tail. Long-horizon scores from 2025 are now revisable.
- **τ-bench consistency** measures multi-turn reliability: agents at 60% pass@1 show only 25% trial-trial consistency. Long horizon = single-shot reliability compounded. Source: thecontextlab.ai.
- **Context-rot mid-position recall** (Chroma 2025, tested 18 frontier models incl GPT-4.1 / Claude Opus 4 / Gemini 2.5): U-shape attention, 30%+ accuracy drop on mid-context items. Every model. Long-horizon → big context → mid-context blindness. Source: morphllm.com/context-rot.
- **METR RCT (July 2025)** — when long-horizon = "real OSS task, hours of work", AI made experienced devs 19% slower. CI +2% to +39%. Self-rated +20% faster (perception gap). Source: metr.org July 2025.
- Anti-gaming: METR's task pool is curated and partially private; ELAI long-horizon claims should use HCAST-style task lengths in HUMAN hours, not model wall-clock.

## Best metrics for ELAI to ship + surface

Ranked by signal-to-noise and grounding strength:

1. **STABLE-Pass@1 (n=5 seeds, identical-output check)** | Grounding: arXiv 2512.06710, 2603.29231, τ-bench. Integration: phase_NN.toml field `stable_pass_1_n5`; dashboard panel "Reliability"; SSB event on regression. Replaces current `pass@1`.

2. **$/correct-solve by tier (run_tier_a/b/c/d)** | Grounding: HAL ICLR 2026 (cost-controlled rollouts mandatory). Integration: phase_NN.toml `cost_per_solve_usd`; surfaced in dashboard cost panel; SSB action when tier b/c/d crosses 2x baseline.

3. **METR-style time-horizon-at-50%** | Grounding: metr.org Mar 2025 + Jan 2026 updates. Integration: new file `metrics/horizon.toml`; dashboard "Horizon" panel; quarterly cadence not per-phase. ELAI's own task pool must be HCAST-style (curated, human-timed, partly held out).

4. **Harness-vs-model delta tuple** | Grounding: particula.tech, focused.io, Fowler harness-engineering. Integration: phase_NN.toml fields `harness_sha`, `model_id`, `delta_harness`, `delta_model` — refuse to publish a score without both deltas filled.

5. **Held-out private eval set** | Grounding: SWE-bench Pro design (arXiv 2509.16941); OpenAI Feb 2026 abandonment of Verified. Integration: `metrics/private_eval/` directory, never tracked in git, regenerated quarterly; report public-vs-private gap as a contamination canary.

6. **30-day revert rate + AI/Human churn ratio** | Grounding: DORA 2025, GitClear, CodeRabbit Dec 2025. Integration: when ELAI's self-mod proposals are auto-merged, track 30-day rollback rate; SSB tripwire at >12%.

7. **Prompt-injection ASR + refusal accuracy** | Grounding: arXiv 2509.22040, 2510.23675, Claude 4.5 system card. Integration: `safety/asr.toml`; dashboard "Safety" panel; SSB tripwire on regression. ELAI internal target: ASR <5% on the IPI suite, refusal >98.7% on 150-item malicious set (matches Claude 4.5 published bar).

8. **Eval-awareness flag rate** | Grounding: Anthropic NLA — Claude flags eval in 16-26% of SWE-bench runs vs <1% real sessions. Integration: when ELAI runs evals, log "looks like a test" markers; high rate = benchmark contamination signal.

9. **Context-rot mid-position recall** | Grounding: Chroma 2025 / Morph. Integration: `metrics/context_rot.toml` ("needle at position 0.5 of context window" recall); SSB at <70%.

10. **Cover@τ (τ=0.95)** | Grounding: arXiv 2603.29231. Integration: phase_NN.toml — measures coverage at high reliability threshold, separates "lucky once" from "consistently works".

## User-facing metric communication

How to tell the user "ELAI is good" with grounding:

- **Lead with cost-per-correct-solve, not raw scores.** HAL's framing wins: "ELAI ships X correct PRs per $1, vs baseline Y" with citation to phase_NN.toml run.
- **Pair every capability number with a reliability number.** "STABLE-Pass@1 = 0.72 (n=5 seeds, std=0.03)" not "Pass@1 = 0.81". The std is the credibility.
- **Cite the harness, not just the model.** "ELAI v0.45 harness on Qwen2.5-Coder-7B-abliterated scored X" — both required. Source: harness-vs-model attribution research.
- **Show the time-horizon line, not a single point.** A graph of "tasks of length L solved at rate ≥50%" by version. METR-style. Source: metr.org/time-horizons.
- **Publish the private-vs-public gap as a contamination canary.** "Public benchmark X, private held-out Y, gap Z%" — gap > 10pp signals overfit. Source: SWE-bench Pro design.
- **For the user-facing claim "ELAI is good at coding", the survivable evidence stack is:** (a) STABLE-Pass@1 on private set, (b) $/CTS Pareto vs commercial baseline, (c) 30-day revert rate on real auto-merged proposals, (d) METR-style 50% horizon trend over time. Anything thinner is unfalsifiable.
- **Never publish a number from a single seed, single harness, public benchmark.** That's the vanity stack.
- **Cursor's "+39% PR merge rate vs control" is the right framing for downstream claims** — but only if paired with revert + churn data (which Cursor's study didn't include, per Pragmatic Engineer critique).

## Anti-patterns

- **Single-seed Pass@1 reporting.** 2.2-6.0pp variance even at temp=0. Source: arXiv 2602.07150.
- **Reporting on SWE-bench Verified in 2026.** OpenAI publicly abandoned it. Citing it now signals either ignorance or deliberate gaming. Source: openai.com Feb 2026.
- **Acceptance-rate maximization.** >45% suggests uncritical accept; CodeRabbit Dec 2025 found 1.7x more issues in AI-coauthored PRs. Source: Augment, CodeRabbit.
- **Self-reported speedup.** METR RCT: +20% perceived, -19% measured. Source: metr.org July 2025.
- **Model swap without harness ablation.** Most "+5% from new model" is harness drift. Source: focused.io.
- **No held-out eval set.** Public benchmarks contaminate within months. Source: SWE-Bench Illusion, arXiv 2506.12286.
- **Reporting throughput without stability.** DORA 2025: AI lifts throughput +2-18% but degrades CFR. One without the other is half the story.
- **Ignoring eval-awareness signals.** Claude flags eval in 16-26% of SWE-bench runs; if a model "knows" it's being tested, scores aren't generalizable. Source: Anthropic NLA.
- **Long-context "needle-in-haystack" passes ≠ real-world long-horizon.** Chroma 2025 shows 30%+ mid-context degradation despite needle passes. Source: morphllm.com.
- **Reporting tokens/sec as a coding-agent metric.** Cost-per-correct-solve dominates; HAL paper makes this explicit.

## Open questions

1. What is the smallest n in STABLE-Pass@1 that gives <1pp std-error for ELAI's tier-c/tier-d benchmarks? (Measurable: run k=3,5,10,20 and fit.)
2. What's the ELAI harness-vs-model delta on SWE-bench Pro held-out repos when swapping Qwen2.5-Coder-7B-abliterated for cloud Sonnet 4.7? (Measurable: ablate one axis at a time.)
3. What's the 30-day revert rate of ELAI's auto-merged self-mod proposals over the last 90 days? (Measurable: git log + revert detection on the .elai_cc repo.)
4. What's the prompt-injection ASR on the IPI-coding suite (arXiv 2510.23675) for the current ELAI harness? Targets: <5% with 2-shot, <15% with 8-shot adaptive. (Measurable: run suite quarterly.)
5. What's the gap between public SWE-bench Pro and ELAI's private held-out eval set? (Measurable: build private set first, then compute gap; treat gap >10pp as a contamination tripwire.)
