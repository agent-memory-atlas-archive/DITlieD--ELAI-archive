# Idea-to-SaaS prior art (2024-2026)

Scope: survey idea-to-app systems 2024-2026, ground every claim, identify stealable patterns for ELAI's recursive decomposition + grounded research + auditor-gated FSM pipeline.

## TL;DR
- "Idea > working demo" is solved in narrow stacks (Next.js frontend, Supabase backend) by v0 / Bolt / Lovable; "idea > production SaaS" is NOT solved by anyone — 45% of AI-generated code fails basic security tests (Veracode 2025), 65% of vibe-coded production apps had security issues (Escape.tech scan of 1,400+ apps), 95% of GenAI pilots fail to reach production (MIT Sloan 2025).
- Frontier SWE-bench Verified scores (Claude Opus 4.7 87.6%, GPT-5.3-Codex SOTA) collapse on SWE-Bench Pro (best ~23-46% public, ~17.8% private commercial subset) — leaderboard performance does NOT predict production-ambiguity performance.
- Autonomous agents (Devin) fail 70-85% of real-world tasks per Answer.AI's January 2025 evaluation; Cognition's own 2025 retrospective admits Devin is "junior at execution" and "can't independently tackle an ambiguous coding project end-to-end."
- Systematic failure modes are well-documented and consistent across products: incomplete auth, missing CSRF/headers (100% of tested apps lacked CSRF — Tenzai Dec 2025), credential leaks (AI commits leak secrets at 2.1x human rate), hallucinated packages (19.7% of LLM package recommendations don't exist), demo-vs-deploy environment drift, scope drift on >3 hour autonomous runs.
- Architecture convergence in late 2025: parallel sub-agents (Cursor 2.0 cap=8, Replit Agent 3 spawns sub-agents), DAG-based research decomposition (FlashResearch arXiv:2510.05145, Flash-Searcher arXiv:2509.25301), "skills" / persistent SKILL.md behaviors (Codex Dec 2025, Claude Code), and harness-as-product (Claude Agent SDK rename from "Code SDK").

## Key sources

Products
- [v0.app review (Trickle, 2025)](https://trickle.so/blog/vercel-v0-review) | v0 capability matrix | UI 9.5/10, fullstack 6.8/10, token-based pricing eats fullstack budget
- [v0.dev > v0.app (Vercel blog)](https://vercel.com/blog/v0-app) | fullstack sandbox launch | v0 now imports GitHub, pulls Vercel env vars, runs full apps in browser sandbox
- [Bolt.new review (Trickle 2025)](https://trickle.so/blog/bolt-new-review) | WebContainer scope | JS-wrapped only, no non-JS backends; fails beyond prototype
- [Bolt status: DB claim issues](https://status.bolt.new/incidents/01KKV5QK8EX879EQKGQ33ZWZ9T) | production failure log | confirmed Supabase claim flow broken in prod
- [Lovable + Supabase launch (Supabase blog)](https://supabase.com/blog/lovable-cloud-launch) | architecture | Lovable Cloud auto-provisions Postgres + auth + edge functions
- [Lovable review (eesel)](https://www.eesel.ai/blog/lovable-review) | actual capability | 4.7-star for straightforward apps, ~3.5 for enterprise, "60-70% of work but falters on advanced logic"
- [Replit Agent 3 deep review (Skywork)](https://skywork.ai/blog/replit-agent-3-a-deep-dive-into-the-future-of-autonomous-coding/) | 200-min autonomous run | self-testing + sub-agent spawning; external API auth still consistently fails
- [Cursor 2.0 changelog](https://cursor.com/changelog/2-0) | parallel-8 agents | Composer model 4x faster, 8 parallel agents in isolated worktrees
- [Cursor Composer multi-agent (InfoQ)](https://www.infoq.com/news/2025/11/cursor-composer-multiagent/) | architecture | mixture-of-experts, 272k context, RL-trained in live coding harness
- [Cline Roo Code comparison (Qodo 2026)](https://www.qodo.ai/blog/roo-code-vs-cline/) | VS Code agent pattern | Roo adds modes (Code/Architect/Ask/Debug), diff editing saves 30% tokens
- [Aider leaderboard](https://aider.chat/docs/leaderboards/) | edit-format benchmark | polyglot bench 225 Exercism exercises across 6 langs; edit format matters more than raw model strength
- [Anthropic Claude Agent SDK](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk) | harness primitives | renamed from Code SDK; provides agent loop + context management as library
- [OpenAI Codex CLI changelog](https://developers.openai.com/codex/changelog) | Skills + remote control | SKILL.md persistent behaviors Dec 2025; codex remote-control daemon
- [GPT-5.3-Codex announcement](https://openai.com/index/introducing-gpt-5-3-codex/) | SOTA on SWE-Bench Pro + Terminal-Bench 2.0 | 3-4x fewer tokens/task, 200k context
- [GitHub Copilot Agent Mode (Feb 2025)](https://github.com/newsroom/press-releases/agent-mode) | multi-file agent | next-edit suggestions, agent mode for cross-file changes

Benchmarks
- [SWE-bench Verified leaderboard](https://www.swebench.com/verified.html) | 500 human-curated instances | Claude Mythos Preview 93.9%, Claude Opus 4.7 87.6% (2026)
- [SWE-Bench Pro leaderboard (Scale)](https://labs.scale.com/leaderboard/swe_bench_pro_public) | 1,865 tasks, 41 repos, multi-lang | Best models GPT-5 23.3% / Claude Opus 4.1 23.1% on public; ~17.8% on private commercial
- [SWE-Bench Pro blog (Scale)](https://scale.com/blog/swe-bench-pro) | what makes it hard | avg 107 LOC across 4.1 files per patch, contamination-resistant
- [SWE-Lancer arXiv:2502.12115](https://arxiv.org/pdf/2502.12115) | 1,488 Upwork freelance tasks, $1M payout | end-to-end e2e tests by qualified engineers, frontend+backend required
- [SWE-bench Multimodal](https://huggingface.co/datasets/SWE-bench/SWE-bench_Multimodal) | 617 visual tasks | UI mockups + screenshots + code
- [tau2-bench (Sierra Research)](https://github.com/sierra-research/tau2-bench) | dual-control retail/airline/telecom | pass@1 drops up to 40% when both user+agent manipulate shared state
- [FlashResearch arXiv:2510.05145](https://arxiv.org/html/2510.05145v1) | DAG+dynamic tree decomp | already cited in ELAI B-40; the original "parallel sub-questions" architecture
- [Flash-Searcher arXiv:2509.25301](https://arxiv.org/abs/2509.25301) | DAG-based parallel web agents | complementary to FlashResearch
- [Planner-Coder Gap arXiv:2510.10460](https://arxiv.org/abs/2510.10460) | multi-agent code gen robustness | information loss between planner > coder is the dominant failure mode

Critiques / post-mortems
- [Answer.AI: Thoughts on a Month with Devin](https://www.answer.ai/posts/2025-01-08-devin.html) | 3/20 success | 14 failures, 3 inconclusive; "Devin would spend days pursuing impossible solutions rather than recognizing fundamental blockers"
- [Cognition's Devin 2025 review](https://cognition.ai/blog/devin-annual-performance-review-2025) | own-data retrospective | 67% PR merge (vs 34% prior); admits "junior at execution, can't tackle ambiguity end-to-end"
- [Veracode 2025 GenAI Code Security](https://www.veracode.com/resources/analyst-reports/2025-genai-code-security-report/) | 100+ LLMs, 4 langs | 45% AI code fails basic security; Java 72% failure rate
- [Vibe Coding Security Crisis (CSA)](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-security-vibe-coding-202/) | Tenzai scan of 15 prod apps | 69 vulns, 100% missing CSRF, 100% missing security headers, every tool introduced SSRF
- [Escape.tech 1,400+ app scan (reported via Baytech)](https://www.baytechconsulting.com/blog/ai-vibe-coding-security-risk-2025) | 65% had security issues, 58% critical; 400+ exposed secrets, 175 PII leaks
- [arXiv:2510.26103: Vulns in AI-gen GitHub repos](https://arxiv.org/pdf/2510.26103) | large-scale empirical | confirms 2.74x vuln rate vs human-written
- [arXiv:2506.11022: Security Degradation in Iterative AI Gen](https://arxiv.org/html/2506.11022v2) | IEEE-ISTAS 2025 | each iteration of "refine this code" makes security worse not better
- [Slopsquatting / package hallucination (Help Net Security)](https://www.helpnetsecurity.com/2025/04/14/package-hallucination-slopsquatting-malicious-code/) | May 2025 academic paper | 19.7% of LLM recommended packages don't exist, OSS models 21.7%, 205k unique hallucinated names
- [VentureBeat: 43% AI code needs prod debugging](https://venturebeat.com/technology/43-of-ai-generated-code-changes-need-debugging-in-production-survey-finds) | survey | "works in dev, breaks in prod" is the dominant pattern
- [Qodo State of AI Code Quality 2025](https://www.qodo.ai/reports/state-of-ai-code-quality/) | quality on legacy code | 75% more logic/correctness issues on legacy vs greenfield
- [The Register: First AI SWE bad at job](https://www.theregister.com/2025/01/23/ai_developer_devin_poor_reviews/) | industry coverage of Answer.AI critique
- [MIT Sloan / Pertama: 95% pilot failure](https://www.pertamapartners.com/insights/ai-project-failure-statistics-2026) | failure stats | 95% pilots, 64% blame infra, 380% avg cost overrun at prod scale

## Product capability matrix

| Product | Backend? | Frontend? | Auth? | DB schema? | Deploy? | SOTA task it can finish | Cite |
|---|---|---|---|---|---|---|---|
| v0.app (Vercel) | partial (sandbox) | yes (best-in-class) | via integrations | via Supabase/Neon | one-click Vercel | dashboard + auth + Stripe (sandbox-grade) | [v0.app blog](https://vercel.com/blog/v0-app) |
| Bolt.new | JS only | yes | weak | weak (WebContainer) | Netlify/Vercel | prototypes, demos, hackathon apps | [Trickle review](https://trickle.so/blog/bolt-new-review) |
| Lovable.dev | yes (Supabase native) | yes | yes (Supabase Auth) | yes (auto-provision) | yes | "fully fledged SaaS" up to ~60-70% complete | [Lovable Cloud launch](https://supabase.com/blog/lovable-cloud-launch) |
| Devin (Cognition) | yes | yes | partial | partial | partial | brownfield refactor, migrations, sec-fixes; FAILS ambiguous greenfield | [Cognition 2025 review](https://cognition.ai/blog/devin-annual-performance-review-2025) |
| Replit Agent 3 | yes | yes | partial (external auth fails) | yes (Replit DB / Postgres) | yes (Replit Deployments) | mobile + web fullstack w/ 200-min autonomous run | [Skywork review](https://skywork.ai/blog/replit-agent-3-a-deep-dive-into-the-future-of-autonomous-coding/) |
| Cursor 2.0 / Composer | indirect (writes code) | indirect | indirect | indirect | indirect (uses your tooling) | large-codebase multi-file refactor, parallel-8 agents | [Cursor 2.0](https://cursor.com/changelog/2-0) |
| Cline / Roo Code | indirect | indirect | indirect | indirect | indirect | step-gated single-task execution | [Qodo comparison](https://www.qodo.ai/blog/roo-code-vs-cline/) |
| Aider | indirect | indirect | indirect | indirect | indirect | incremental edits on existing repo, 26.3% SWE-bench Lite | [Aider leaderboard](https://aider.chat/docs/leaderboards/) |
| smol-developer | yes (scaffolds) | yes | no | no | no | initial codebase scaffold from spec | [smol-ai/developer](https://github.com/smol-ai/developer) |
| Copilot Workspace + Agent Mode | yes | yes | depends | depends | via GitHub Actions | brainstorm > PR multi-file changes | [GitHub agent-mode press](https://github.com/newsroom/press-releases/agent-mode) |
| Claude Agent SDK | n/a (primitives) | n/a | n/a | n/a | n/a | toolkit for *building* the above | [Anthropic SDK doc](https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk) |
| Codex CLI (GPT-5.3-Codex) | indirect | indirect | indirect | indirect | indirect | SWE-Bench Pro SOTA + Terminal-Bench 2.0 77.3% | [Codex CLI features](https://developers.openai.com/codex/cli/features) |

## Systematic failure modes (grounded)

1. **Missing auth fundamentals.** 100% of 15 Tenzai-tested prod apps lacked CSRF protection; 100% lacked security headers; every tool introduced SSRF — [CSA / Tenzai Dec 2025](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-security-vibe-coding-202/).
2. **Credential leakage.** AI-assisted commits expose secrets at 3.2% vs 1.5% human rate (2.1x) — [Baytech 2025](https://www.baytechconsulting.com/blog/ai-vibe-coding-security-risk-2025); 400+ exposed secrets in Escape.tech's 1,400-app scan.
3. **Hallucinated packages / APIs.** 19.7% of LLM-recommended packages do not exist; OSS models 21.7%, proprietary 5.2%; 205k unique hallucinated names observed; "huggingface-cli" attack downloaded 30k+ times — [Help Net Security 2025](https://www.helpnetsecurity.com/2025/04/14/package-hallucination-slopsquatting-malicious-code/).
4. **Demo-vs-deploy environment drift.** Bolt's preview > prod gap: missing deps, env vars not transferred, build configs that pass preview fail prod — [YeasiTech Bolt problems](https://www.yeasitech.com/bolt-new-problems-solutions); confirmed by Bolt's own status page (DB claim flow broken).
5. **Database mock-as-real.** Lovable "handles 60-70% of work but falters on advanced logic, security, or scalability" — [eesel review](https://www.eesel.ai/blog/lovable-review); Bolt WebContainer cannot run real Postgres, only JS-wrapped stand-ins.
6. **Pursuit-of-impossible / no abort signal.** Devin "spent days pursuing impossible solutions rather than recognizing fundamental blockers" — [Answer.AI Jan 2025](https://www.answer.ai/posts/2025-01-08-devin.html).
7. **Tunnel vision in debugging.** Devin SSH debug "fixated on the script itself, never considering alternative root causes" — same source. Maps to ELAI's anti-spiral rule.
8. **Convention drift.** Devin "failed to adapt to project conventions" on modify-existing-projects (0-1/3 success in some categories) — Answer.AI.
9. **Code soup / overcomplicated.** Devin's Braintrust integration produced "layers of abstraction that made simple operations needlessly complex" — Answer.AI.
10. **Iterative security regression.** Each "refine this" iteration makes security worse, not better — [arXiv:2506.11022 IEEE-ISTAS 2025](https://arxiv.org/html/2506.11022v2).
11. **Production legacy collapse.** AI code has 75% more logic/correctness issues on legacy codebases than greenfield — [Qodo 2025](https://www.qodo.ai/reports/state-of-ai-code-quality/); 2.74x vuln rate overall — [CodeRabbit](https://www.coderabbit.ai/blog/2025-was-the-year-of-ai-speed-2026-will-be-the-year-of-ai-quality).
12. **Benchmark > production cliff.** Top models score 87-94% SWE-bench Verified but only 23% SWE-Bench Pro public, 14-17% on private commercial — [Scale SWE-Bench Pro](https://labs.scale.com/leaderboard/swe_bench_pro_public). Real-world ambiguity is the chasm.
13. **Pilot > prod cliff.** 95% of GenAI pilots fail production deployment — MIT Sloan; 64% blame infra, 380% cost overrun avg — [Pertama 2026](https://www.pertamapartners.com/insights/ai-project-failure-statistics-2026).
14. **External API auth fails consistently.** Replit Agent 3, despite 200-min autonomy, "external API integration particularly anything requiring authentication consistently caused problems" — [Skywork Replit review](https://skywork.ai/blog/replit-agent-3-a-deep-dive-into-the-future-of-autonomous-coding/).
15. **Planner-Coder information loss.** Multi-stage decomposition loses requirement detail; coder misinterprets intricate logic — [arXiv:2510.10460](https://arxiv.org/abs/2510.10460). Directly relevant to ELAI's Concepts > Phases > Steps > BabySteps tree.

## What ELAI can steal

Ranked by integration leverage to ELAI's existing FSM + decomposition pipeline.

1. **Tenzai/Escape.tech security checklist as a hard auditor gate.** Pattern: 100% CSRF-missing, 100% security-header-missing, 100% SSRF-prone is empirically reproducible. Grounding: [Tenzai Dec 2025](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-security-vibe-coding-202/), [Veracode 45%](https://www.veracode.com/resources/analyst-reports/2025-genai-code-security-report/). Integration hook: add a `security-gate` deterministic check to Phase 4 auditor — fail-closed if CSRF/CSP/SSRF detectors fire; embed in `wiring-contract-validator` next to existing stub-ban check. This is the single highest-leverage win.

2. **Slopsquatting / package-existence verification at edit time.** Pattern: every import line is verified against npm/PyPI/crates.io BEFORE the file is written, not after. Grounding: [arXiv 2025: 19.7% hallucination rate](https://www.helpnetsecurity.com/2025/04/14/package-hallucination-slopsquatting-malicious-code/); already in ELAI's CLAUDE.md anti-slopsquatting rule. Integration hook: enforce in `executor` agent and `auditor` — emit BLOCK if any import doesn't resolve to a registry entry with publish date <24mo. ELAI's rule exists; add the *automated* check at edit time, not just human-instruction.

3. **DAG-based parallel research at every decomposition level.** Pattern: FlashResearch (already cited in B-40) + Flash-Searcher prove DAG > sequential for research throughput. Grounding: [FlashResearch arXiv:2510.05145](https://arxiv.org/html/2510.05145v1), [Flash-Searcher arXiv:2509.25301](https://arxiv.org/abs/2509.25301). Integration hook: ELAI's `researcher` agent already does parallel sub-questions at Concept level — extend the SAME DAG pattern to Phase and Step levels (currently sequential per CLAUDE.md skill descriptions).

4. **Cursor 2.0's isolated git-worktree parallelism for sub-agent fanout.** Pattern: 8 parallel agents in isolated worktrees, conflict-free; "auto-pick the best" or merge. Grounding: [Cursor 2.0](https://cursor.com/changelog/2-0), [InfoQ Composer](https://www.infoq.com/news/2025/11/cursor-composer-multiagent/). Integration hook: ELAI's `parallel-cap=3` integration-explorer pattern (per orchestrate skill) could adopt git-worktree isolation for `executor` fanout when phases have orthogonal sub-tasks.

5. **Auto-abort signal for "Devin pursuing impossible solutions."** Pattern: detect stagnation in 3 hypothesis cycles AND escalate, not silently retry. Grounding: [Answer.AI Devin critique](https://www.answer.ai/posts/2025-01-08-devin.html) + ELAI's existing anti-spiral 4-pattern detector. Integration hook: tie ELAI's `anti-spiral` skill (already detects Spinning/Oscillation/NoDrift/DiminishingReturns) to a hard time-budget per phase that, once exceeded, ALWAYS escalates to HITL — currently anti-spiral only switches personas; add a hard abort lane.

6. **Codex/Claude Code SKILL.md persistent-behavior pattern.** Pattern: skills are .md files auto-loaded by trigger; same idea ELAI already uses. Grounding: [OpenAI Codex changelog Dec 2025](https://developers.openai.com/codex/changelog) + ELAI's `.claude/skills/`. Integration hook: ELAI already does this — the steal is the *trigger system* (Codex's auto-loaded SKILL.md based on context vs ELAI's trigger phrases). Already aligned; reinforces the architectural bet.

7. **Replit Agent 3's external-API-auth-fails warning as an ELAI Tier-C feasibility flag.** Pattern: known-failure-mode for external auth surfaces blocks the executor and demands HITL. Grounding: [Skywork Replit review](https://skywork.ai/blog/replit-agent-3-a-deep-dive-into-the-future-of-autonomous-coding/). Integration hook: ELAI Phase 4F Feasibility Probe — add `auth_complexity_class` as a constraint flag; if any concept requires OAuth2/SAML/external IdP, force HITL gate before EXECUTING.

8. **Cognition's "junior at execution" honesty as ELAI scope ceiling.** Pattern: explicitly declare what the pipeline CAN'T do (ambiguous greenfield, scope changes mid-task). Grounding: [Cognition 2025 review](https://cognition.ai/blog/devin-annual-performance-review-2025). Integration hook: ELAI idea-framer + idea-critic loop already exists — add a "scope-clarity score" the critic emits; below threshold = MANDATORY HITL questions before decomposing further.

9. **Aider's edit-format > model-strength insight.** Pattern: how the diff is formatted matters more than which LLM writes it. Grounding: [Aider polyglot leaderboard](https://aider.chat/docs/leaderboards/). Integration hook: ELAI's `executor` agent — adopt unified-diff or search/replace format with deterministic re-apply, instead of full-file rewrite. Saves tokens (Roo Code 30% savings on same pattern) and improves merge reliability.

10. **SWE-Lancer end-to-end test pattern for acceptance criteria.** Pattern: real-money freelance tasks scored by qualified-engineer-written E2E tests, not unit tests. Grounding: [SWE-Lancer arXiv:2502.12115](https://arxiv.org/pdf/2502.12115). Integration hook: ELAI's per-level auditor — promote E2E (Playwright/Cypress) tests over unit tests for the Phase 4I Media Validator render-capture-evaluate loop. Already partly there; formalize the SWE-Lancer scoring rubric (does the deliverable actually work in browser end-to-end?).

## Production-readiness gap

Demo-grade and production-grade are separated by these measurable criteria. Each is what *separates* the chasm, with grounded measurement.

- **CSRF + CSP + security headers present.** Tenzai found 100% absent in 15 tested apps. Measurable via `helmet`-style scanner. [CSA 2025](https://labs.cloudsecurityalliance.org/research/csa-research-note-ai-generated-code-security-vibe-coding-202/).
- **No exposed secrets in repo.** Escape.tech found 400+ exposed secrets in 1,400 apps. Measurable via gitleaks / trufflehog. [Baytech](https://www.baytechconsulting.com/blog/ai-vibe-coding-security-risk-2025).
- **All imports resolve to real packages with recent publish dates.** Slopsquatting 19.7% rate. Measurable via registry HEAD requests at edit time. [Help Net Security](https://www.helpnetsecurity.com/2025/04/14/package-hallucination-slopsquatting-malicious-code/).
- **WCAG accessibility coverage.** Automated tools cover ~30% of WCAG criteria; AI-augmented audits push to 55-60%; EU Accessibility Act enforcement live since June 2025. Measurable via Lighthouse score >=95 + axe-core run. [dev.to AI accessibility audit](https://dev.to/agentkit/i-built-an-ai-agent-that-runs-accessibility-audits-on-every-deploy-heres-what-it-catches-that-3h03).
- **Real DB schema with migrations.** Bolt/v0 prototype-DB pattern fails at this gate. Measurable: `prisma migrate diff` runs clean against the deployed environment. [Trickle Bolt review](https://trickle.so/blog/bolt-new-review).
- **External auth (OAuth/SAML) works end-to-end.** Replit Agent 3's documented consistent failure mode. Measurable: live OAuth round-trip test in CI. [Skywork](https://skywork.ai/blog/replit-agent-3-a-deep-dive-into-the-future-of-autonomous-coding/).
- **Build artifact in CI matches preview artifact.** Bolt preview > prod drift is the named failure. Measurable: hash-compare build outputs. [YeasiTech](https://www.yeasitech.com/bolt-new-problems-solutions).
- **E2E user flow passes on cold deploy.** SWE-Lancer's qualified-engineer E2E test pattern. Measurable: Playwright suite green on freshly-provisioned environment. [SWE-Lancer arXiv](https://arxiv.org/pdf/2502.12115).
- **No iterative security regression.** Refining code in a loop makes security worse, not better. Measurable: re-run Veracode/CodeQL scan after every refine cycle; reject deltas that decrease the score. [arXiv:2506.11022](https://arxiv.org/html/2506.11022v2).
- **Logic correctness on legacy/real schema.** 75% more issues on legacy than greenfield. Measurable: mutation testing against existing test suite. [Qodo 2025](https://www.qodo.ai/reports/state-of-ai-code-quality/).

## Anti-patterns

- **Autonomous-without-abort.** Devin's days-long impossible-task chase. [Answer.AI](https://www.answer.ai/posts/2025-01-08-devin.html). ELAI implication: anti-spiral must be HARD abort not just persona swap.
- **Iterative-refine-loops on security-sensitive code.** Worsens vulns per [arXiv:2506.11022](https://arxiv.org/html/2506.11022v2). ELAI implication: security-scan delta must be a non-decreasing invariant in the audit gate.
- **Trusting LLM-named packages.** Slopsquatting. [Help Net Security](https://www.helpnetsecurity.com/2025/04/14/package-hallucination-slopsquatting-malicious-code/). ELAI: rule exists; automate the check.
- **Unit tests as the only acceptance signal.** SWE-Lancer found E2E tests catch what units miss. [SWE-Lancer arXiv](https://arxiv.org/pdf/2502.12115).
- **Benchmark-tuned models in production-ambiguous scenarios.** Verified > Pro cliff of 60+ percentage points. [Scale SWE-Bench Pro](https://labs.scale.com/leaderboard/swe_bench_pro_public).
- **Single-LLM pipeline with no critic.** Planner-Coder info loss. [arXiv:2510.10460](https://arxiv.org/abs/2510.10460). ELAI already has idea-critic; extend per-level critic to Phase + Step + BabyStep.
- **WebContainer-class constraints sold as production.** Bolt's Node-in-browser hits a hard JS-only ceiling. [Trickle Bolt review](https://trickle.so/blog/bolt-new-review). ELAI: declare runtime tier explicitly.
- **HITL-optional in ambiguous tasks.** Cognition's own admission: ambiguity = senior judgment = human. [Cognition 2025](https://cognition.ai/blog/devin-annual-performance-review-2025).
- **Deploy = "we hit the button" rather than "it stayed up 72h."** Time-to-first-incident is the real metric, not "deploy success rate." Implied by 95% pilot-to-prod failure rate [Pertama](https://www.pertamapartners.com/insights/ai-project-failure-statistics-2026).

## Open questions

1. **What is ELAI's measurable security-gate threshold?** Target: 0 CSRF/CSP/SSRF findings on Tenzai-style scan AND <=Veracode median vuln density. Need a baseline run before Phase 4 audit upgrades.
2. **What's the empirical hallucination rate of ELAI's `executor` on dependency picks across npm/PyPI/crates.io?** Sample N=100 generated import lines, measure registry-resolve rate; target >=99%.
3. **Can ELAI's per-level critic close the Planner-Coder gap measurably?** Compare end-to-end success on a 20-task harness (Answer.AI-style) WITH vs WITHOUT idea-critic loop. Goal: >=2x improvement at 95% CI.
4. **What's ELAI's SWE-Lancer-equivalent E2E pass rate on the Phase 4I Media Validator's render-capture-evaluate loop?** Need 50-task internal eval set with Playwright assertions per task.
5. **Does anti-spiral hard-abort + HITL produce net-faster outcomes than current persona-swap?** A/B on a stagnation-prone task set; measure wall-clock-to-completion AND human-intervention-count.
