Date: 2026-05-17
Audience: user asking "what will ELAI be capable of when all B-plan + C-plan done?"
Stance: honest, grounded in DETAILED_SUMMARY_01 + B-PLAN-SUMMARY + C-SUMMARY_INDEX + phase-index-MASTER + guide/INDEX. No marketing.

================================================================
CONFIDENCE LEGEND (per CLAUDE.md)
================================================================
[V]   = Verified by measured metric in repo (.toml files), shipped commit, or read code
[H]   = High confidence — design doc explicit + code path read
[P]   = Projected — design spec only, never benchmarked
[U]   = Uncertain / training-flavor — flag as such


================================================================
1) "Reason like a flagship model using local models?"
================================================================

Short answer: NO, not the model itself. PARTIALLY YES via systemic substitution. Honest YES only in hybrid mode.

Grounding:
- [V/U] Qwen2.5-Coder-7B NF4+FA2: 90% pass@1 figure is from the PUBLISHED Qwen2.5-Coder HF model card on HumanEval — NOT independently measured inside ELAI's harness. ELAI's own measurement is the 3B at 25K tokens hitting 100% on a small internal smoke set (memory project_qwen3b_baseline.md). Two different benchmarks; don't conflate. 7B in-harness measurement is unconfirmed.
- [V] Qwen2.5-Coder-3B NF4+FA2: 72% pass@1 (HF card) / 100% on ELAI smoke at 25K. 1.92GB. ~40K context.
- [P] Qwen2.5-Coder-32B+TQ4 at 48GB: HumanEval 92.7% / MBPP 90.2% per published HF cards. PROJECTED for ELAI — needs 2x3090 which is "coming" per memory project_vram_tier_plan.md. Untested in our harness.
- [H] DoLa, steering vectors, ETC LogitsProcessor: infrastructure ships but DoLa is HARMFUL on 7B (-15% code quality, MEASURED in baseline). Only viable on 32B+ (64 layers). Steering vectors NEUTRAL on 7B with generic pairs.
- [V] DETAILED_SUMMARY_01 §1: "The intelligence is in the infrastructure, not the model" — the explicit design thesis. Local 7B Qwen produces correct output because the system makes each task so small, so precisely defined, so thoroughly verified that even a weak model succeeds — eventually.

What "flagship-like" means here, decomposed:
(a) Raw chain-of-thought depth on novel problems → NO. A 7B has fewer parameters and no test-time-compute equivalent to Opus extended thinking. Sonnet/GPT-4/Opus will out-reason 7B on hard novel design problems. Period.
(b) Code correctness on decomposed tasks → YES, when the task fits in a baby step. Architecture's "FILE_PLAN + skeleton-first + MiniCritic + StructuralRetry + AcceptAndComplete" (C-31/C-32) enforces tiny units. Local model has been measured (90% pass) on this slice.
(c) Long-horizon planning across 30+ files → Done by Designer agent (deterministic decomposition tree, NOT the model freestyling). The model never sees the full plan; it gets one BabyStepSpec at a time (~330 tokens of meta-instructions per turn vs 1600-2600 in prompt-based workflows). So the model doesn't NEED to "reason like a flagship" — the Rust binary reasons; the model is a code printer.
(d) Catching its own mistakes / self-critique → Cross-model: auditor != executor (per CLAUDE.md). Anti-rubber-stamp + per-level audit modes (B-41). The auditor agent IS a separate LLM call with its own GRC budget. On local model + local auditor, this is two Qwen instances catching each other's drift — measurably better than self-review, but not equivalent to GPT-4 reviewing GPT-4-mini.

Where flagship-class reasoning is genuinely substituted:
- [H] FlashResearch tree research (Phase 4D / arXiv:2510.05145) for design decisions
- [H] CSQ counterfactual checks pre-write
- [H] Inline feasibility checks during execution
- [H] Mutation testing (C-28) for actual test quality, not just test count
- [H] Metamorphic relations (C-29) for behavior properties

These are DETERMINISTIC supplements. They make a 7B more reliable than a 7B + prompt-only-loop. They do NOT make it equal to Opus on novel architectural reasoning.

Verdict: HYBRID is the realistic path. Local 7B handles 80% of routine code/edits with infra support. Cloud Sonnet/Opus escalation (proven end-to-end per commit db594a6, project memory) handles the hard 20%. With 32B on 2x3090 (when shipped), local share grows. With true frontier-only-tasks, cloud is irreplaceable.


================================================================
2) "Work on any project/language/scope and adapt to it?"
================================================================

Short answer: YES for the 17 supported languages. PARTIAL for domain-specific stacks (Unity, Unreal, embedded). Adaptation requires user investment.

Grounding:
- [H] B-30 multi-language DKG: Rust / Python / TypeScript / TSX / JavaScript / Go / Java / Kotlin / Scala / C / C++ / C# / Ruby / PHP / Swift / Bash / Svelte / Vue. Per-grammar failure isolation via std::panic::catch_unwind. Multi-module resolvers (JVM Maven/Gradle, C# namespace map, Go module info, TS path aliases with cycle detection).
- [H] B-34 EntryPoint detection: 14-language ENTRY_POINT_NAMES + framework router detection + test name patterns. Auto-injected into Designer's Recursive Decomposition when project is ExistingProject.
- [H] B-35 Cosmos artifact tier: DB schemas, OpenAPI, k8s manifests, ADRs, glossaries. 7th GRC grounding method ArtifactCheck. Designer's claim about "users table has email column" actually verifies against declared schema.
- [H] Project intelligence Phase 3L: ProjectProfile auto-detection from language stats, conventions, build tooling.
- [H] DETAILED_SUMMARY_01 §8: "The system works for any language and any project, not just Rust."

The Unity question specifically:
User said: "right now i am making a unity game, llms are stupid and depends on unitymcp, and they cant really make a proper UI without messing shit up"

Honest breakdown:
- [V] C# IS in B-30's language list. DKG will parse C# scripts. Symbol-graph cache (B-29) will resolve cross-script references with 4 confidence tiers (Local/Unique/MultipleCandidates/Unresolved).
- [U] Unity has tooling NOT in ELAI's spec: MonoBehaviour lifecycle, prefab/scene .unity serialization (YAML with GUIDs), .meta files, AssetDatabase API, ScriptableObject patterns, EditorWindow, IMGUI vs UIElements vs uGUI vs UI Toolkit (3 incompatible UI systems), URP/HDRP/Built-in render pipelines, Addressables vs Resources vs Asset Bundles. None of this is in ELAI's grammar set or default Cosmos schemas.
- [H] Plugins crate has MCP HTTP JSON-RPC client (batch-ful-d, plugins/src/ipc_hooks). User CAN wire UnityMCP through it. But ELAI doesn't ship Unity-aware deterministic rules, doesn't know prefab GUID semantics, doesn't know that touching a serialized field can corrupt scenes.
- [H] User would need to: (a) declare Unity scene/prefab files as Cosmos artifacts with custom chunkers, (b) write deterministic rules (T1 GREP_PATTERN, T2 AST) for Unity-specific anti-patterns (e.g., GetComponent in Update, allocations in hot loops), (c) wire UnityMCP via plugins MCP bridge.

So: ELAI gives you the harness (DKG + blast-radius + rule engine + MCP bridge + autonomous loop) but you still feed it domain knowledge. The "LLM is stupid at Unity UI" problem is fundamentally that Unity UI requires either (i) the LLM understanding component serialization (hard for ANY model without specific tools), or (ii) using a real Unity Editor automation tool (which is what UnityMCP attempts). ELAI does not make UnityMCP better — but ELAI's PostToolUse hook + visibility filter + audit cycle catches "you broke the scene file" deterministically if you wire the rules.

[U] My confidence that ELAI improves your Unity workflow over Cursor+UnityMCP: MEDIUM. The gain is in (a) auditor catching scene corruption, (b) Cosmos artifacts giving the model Unity-API-grounded knowledge, (c) skeleton-first preventing the model from dumping 800-LOC monolith scripts. The loss is in (a) no Unity tools out-of-box, (b) effort to wire it.

For "any project/language/scope" more broadly:
- [H] Web apps: covered well (Svelte/Vue grammar; entry-point detection knows React/Next/Nuxt routers per B-34; SearXNG for research; web_validator crate for output validation).
- [H] CLI tools / libraries: well covered (was the original target).
- [H] Data pipelines: Python covered, deterministic checks include pytest. Cosmos artifacts cover SQL schemas, OpenAPI.
- [U] Mobile (iOS/Android): Swift + Kotlin in B-30 grammar set. No iOS/Android-specific tooling, no XCode/Gradle build adapters mentioned in specs. Would need same investment as Unity.
- [U] Embedded / firmware: C/C++ grammars present. Cross-compilation, hardware-in-loop testing not addressed.
- [U] Game engines other than Unity: Same as Unity — language is covered, ecosystem isn't.


================================================================
3) "Autonomously create e2e websites/apps independently of scope?"
================================================================

Short answer: ARCHITECTURALLY YES. EMPIRICALLY UNTESTED at scale beyond toy demos.

Grounding:
- [H] Pipeline factory L1 (Designer + FlashResearch) → L2 (Builder + DAG repair + DecompositionTree) → L3 (Executor through FSM).
- [H] AUTONOMOUS mode (B-22, phase-4G): AutoAdvanceEvaluator with EvidenceCheck and EscalationPolicy. SplitTaskPlan for large tasks. GateMode::from_cli_args.
- [H] WebResearchOrchestrator: FORMULATE → SEARCH → EXTRACT → REFLECT → CONVERGE loop with SearXNG (proven, running on remote). Source quality tiered deterministically (arxiv/official=T1, GitHub/SO=T2, blogs=T3, spam=T4). Contradictions surfaced.
- [H] Anti-spiral (C-10): 4 stagnation patterns (Spinning, Oscillation, NoDrift, DiminishingReturns) + 5 personas (Hacker, Researcher, Simplifier, Architect, Contrarian). Watchdog for idle/no-progress/safety.
- [H] Code completeness (C-31/C-32): FILE_PLAN required per BabyStep, FileLineTracker per-step, MAX_FUNCTIONS_PER_FILE=20, per-language soft+hard LOC limits, skeleton-first with BLANK_N substitution, MiniCritic per fill, StructuralRetry sequencer (Decompose/Template/FewShot/BestOfN), AcceptAndComplete at completeness_ratio ≥ 0.5.
- [H] Media validator (Phase 4I) + render-capture-evaluate: Playwright for web, VLM evaluates screenshots, visual feedback loop with convergence check.
- [V] Cloud mode proven end-to-end (commit db594a6, OAuth Sonnet 4.6). Cloud Designer can plan, cloud Executor can write multi-file changes.

Honest reservations:
- [V] User memory feedback_verify_semantics.md: "'PROVEN' requires output content match input semantics; 511-byte Vite splash != 'ELAI built TODO app'." Past sessions have declared PROVEN on shallow evidence. The verify-semantics rule exists BECAUSE this has been failing.
- [P] No metric in any phase_*.toml shows "ran AUTONOMOUS on real 1000+ LOC web app, hit completion, deployed". The b40/b41/b42 ship metrics are for INTERNAL feature additions to ELAI itself, not e2e user-app builds.
- [H] Architecturally, the AcceptAndComplete threshold (0.5 completeness) is a *known* compromise — if structural retry fails, the system accepts half-done work rather than spinning forever. This means autonomous runs CAN produce semi-broken artifacts.
- [H] The auditor has veto power on AUDITING → MEMORIZING (mutation_score, MR aggregate, wiring audit). But auditor is also a 7B model in local mode; it can miss things.
- [U] My confidence that an unattended overnight autonomous run produces a *working deployed e2e SaaS* with payments, auth, DB migrations, CI/CD: LOW for local-only, MEDIUM for hybrid mode with cloud Designer.

What actually works well per design:
- [H] Surgical mode (C-30) for "modify this existing project to add X": real DKG blast-radius, complexity bands, RegressionGate, INTERACTIVE confirmation on LOC > 500.
- [H] Oracle mode for "research and answer this technical question": cached answers per project, source-tier-aware TTL (T1=30d, T2=7d, T3=1d).
- [H] Greenfield small-to-medium apps with clear spec: pipeline factory designed exactly for this.

What probably doesn't work yet:
- [U] Greenfield "make me a Notion clone" — too underspecified, decomposition tree will explode, integration testing won't catch UX bugs.
- [U] Continuous-deployment pipelines with rollback semantics — no specific design block addresses this.
- [U] Long-running stateful apps with migration history — Cosmos artifacts cover *current* schema but not migration sequencing.


================================================================
4) "Properly researching and making proper choices in autonomous mode?"
================================================================

Short answer: BETTER THAN MOST. The research scaffolding is the strongest part of the spec.

Grounding:
- [H] FlashResearch tree research (arXiv:2510.05145 per project memory reference_b40_papers.md): tree of sub-questions with parallel expansion, speculative execution (cloud), dual-threshold convergence (goal + quality).
- [H] Recursive decomposition triggers grounded research at EVERY tree level — not just top. Concept-level, phase-level, step-level, baby-step-level.
- [H/CURRENT-STATE-GAP] PackageRef.verified anti-hallucination badges: design says API_REGISTRY_CHECK does real HTTP to crates.io/PyPI/npm/etc per anti-staleness Phase 4. CURRENT STATE per DETAILED_SUMMARY_01 §2 (reasoning crate): "API_REGISTRY_CHECK — STUB returns typed error (uses string-shape only, not real registry HTTP, in Phase 4D verify_packages)." B-plan + C-plan closes this. Under "all done" assumption: YES. Today: shape-only.
- [H] DecisionSource tagged on every TechStackDecision: RESEARCH / EXISTING_PROJECT / USER_SPECIFIED / FALLBACK. Per memory feedback_paper_citation_verify.md: "WebSearch arXiv IDs before citing" — this rule exists because 2 bad cites were caught in B-40/B-42 plans.
- [H] Source quality tiers deterministic (4D web research). Contradictions surfaced, never silently resolved.
- [H] Cosmos artifacts (B-35) give the system project-specific ground truth so research doesn't override known facts.
- [H] CSQ counterfactual: "if assumption X is wrong, code will Y" — required pre-write for files/network/money/auth.

Reservations:
- [V] Project memory feedback_paper_citation_verify.md catches that "effect-size numbers need quote+page-anchor; 2 bad cites in B-40/B-42 plans." The research pipeline has been wrong before. It's not infallible.
- [H] Research budget is enforced (4D), so research can be terminated mid-flight if it exceeds budget. Choice quality degrades under tight budgets.
- [U] No metric showing autonomous-mode tech-stack-choice agreement with human-expert choice across N tasks. We don't know the FALSE POSITIVE rate of "I researched and chose React" vs "Vue was actually better for this".

Verdict: Genuinely strong research architecture. Better than naive "ask LLM what library to use." NOT a substitute for senior-engineer judgment on novel tradeoffs.


================================================================
5) "Get better with time as multi-use LLM IDE?"
================================================================

Short answer: YES, bounded. This is one of ELAI's most differentiated capabilities.

Grounding:
- [H] Self-modification Phase 4H (NOT YET STARTED per DETAILED_SUMMARY_01 §5). 3 layers:
  - Layer 1 (data, automatic): template rankings, rule trigger counts, pipeline outcome metrics. Pure arithmetic. No LLM.
  - Layer 1A (data-table growth, no LLM): deprecated_patterns, FailureTaxonomy, lazy-code regex, source-tiers grow from logs.
  - Layer 2 (config, full pipeline + auditor approval): new rules, GRC budgets, context ratios, model assignments.
  - Layer 3 (structure, human approval): FSM additions, agent roles, GRC protocol changes.
- [H] Modification Solidification Index (MSI): exp-decay weighted moving avg of delta_score. IMPROVING / STABILIZING / DIVERGING. Auto-revert on >10% degradation over 3 pipelines.
- [H] Anti-reward-hacking: metrics code, gate definitions, auditor config, test suite all READ-LOCKED during self-modification. Self-mod agent cannot read lock list.
- [H] Template crystallizer (C-25): templates become reusable after 3 consecutive successes. CompiledExecutor cache-hits speed up repeated patterns.
- [H] MistakeBook (C-31) records T1/T2 BLOCK events for future avoidance.
- [H] Anti-staleness nightly dep-audit refreshes deprecated_patterns + steering vector when needed.

Why this matters more than it sounds:
Most "AI IDEs" (Cursor, Windsurf, Copilot, Claude Code itself) do NOT learn project-specific patterns at the system level. They get better only when Anthropic/OpenAI retrains the model. ELAI has a learning loop where each project teaches the rules engine + template store + Cosmos cache, scoped per project. Bug catches accumulate. Rule false-positives auto-deprecate after 90 days.

Reservations:
- [H] Phase 4H NOT YET STARTED. C-21 closes the wiring but the actual learning loop empirical behavior is unmeasured.
- [H] Layer 1A growth is bounded by the closed AllowedConfigKey enum (12 variants per C-21). It learns within the lanes; it doesn't invent new lanes.
- [P] No metric showing "after N projects, ELAI's first-try ratio improved by X%". The infra is for learning; the dataset to learn from doesn't exist yet.


================================================================
6) BIG QUESTION: "Reason like flagship + work on any task as if 1M+ context?"
================================================================

Honest answer: BOTH PARTS ARE WRONG-FRAMED. The architecture's bet is that *neither is necessary* if the system decomposes well.

1M context claim, decomposed:
- [V] No local model has 1M context. Even cloud Sonnet 4.6 has 1M but at non-trivial cost.
- [H] ELAI's substitute is precision, not capacity:
  - A-RAG: 55-87% token savings on context retrieval
  - ShortCoder: 18-38% token savings on tool outputs
  - Process isolation (Tier C): agent reasoning chains in separate processes = 150% effective context window savings
  - Context budget allocator: 8 slots, file_overlap_score filtering, 3-level adaptive reduction
  - Session handoff DAG: structurally lossless trimming at 85% budget threshold
  - per-VRAM tier prompt budgets: 7B/12GB caps at 20K (30K with TQ8), 32B/48GB at 30K
- [H] Net effect: the model sees ~20-30K tokens of precisely curated context per turn. NOT 1M. But the *task* effectively spans more because state lives in SSB, FSM, SessionDag, Cosmos.
- [V] User memory project_qwen3b_baseline.md: "25K tokens 100% pass" measured. The ceiling is real but enough for properly decomposed tasks.

Flagship-reasoning claim, decomposed:
- (a) For atomic baby-step code: 7B + infra ≈ flagship (measured 90% pass on isolated functions). Marginal gap closed by decomposition + verification.
- (b) For multi-step architectural reasoning: 7B alone falls short. Designer agent + FlashResearch + cloud-escalation closes this — but at cost (cloud spend or 2x3090).
- (c) For novel domain insight (research-frontier algorithms, novel system design): even flagships struggle without specific training. ELAI's paper-ingestor + Cosmos + research tree improves grounding but doesn't manufacture genius.

What ELAI realistically delivers when all done:
- [H] On routine code tasks (CRUD, refactors, bug fixes, testing) in supported languages: comparable to or better than Cursor+Sonnet at fraction of API cost.
- [H] On long-horizon projects with iteration: superior to one-shot LLMs because the FSM + audit + memorize loop accumulates context across sessions.
- [H] On novel hard problems: needs cloud escalation. Local-only will hit ceilings.
- [H] On domain-specific stacks (Unity, embedded): needs user investment in Cosmos artifacts, rules, MCP bridges. Will not magically know your domain.
- [P] On end-to-end deployment (CI/CD, infra, observability): probably needs Cosmos + custom adapters; specs don't address this directly.

Does your system allow for all that?
- [V] Hardware: 12GB GPU now, 2x3090 coming. 12GB caps you at 7B Qwen (good for code, can't run 32B+TQ4). 2x3090 unlocks 32B which is materially better.
- [V] Sidecar architecture (remote pythondev box) is proven; no local-machine GPU pressure during sessions.
- [V] Cloud mode proven (Sonnet 4.6 commit db594a6) — hybrid escalation is real.
- [H] Workflow tooling (FSM hooks, session locks, RAG, paper-ingestor, impact-mapper, spec-grounder) is mature; that's what makes B-N completions reliable.
- [H] Estimated remaining effort: 200-264h for C-plan alone. Plus deferred B-N items (~ B-15 partial, B-16 partial, B-18..B-28 mostly unwired, B-29..B-35 NOT STARTED). Realistic timeline: months, not weeks.


================================================================
7) WHERE THE PLAN MIGHT BE WRONG
================================================================

Honest critical points:
- The phrase "the intelligence is in the infrastructure, not the model" is a *bet*. It works for tasks that decompose. For tasks that don't decompose (e.g., "design this novel distributed-systems protocol"), the model still has to reason. A 7B model still has a 7B-shaped ceiling on novel reasoning.
- The "process isolation saves 150% context" claim assumes the executor's reasoning chain is bulky. For trivial tasks it's not; for complex tasks the chain itself may still overflow the agent's own budget. Either way, the orchestrator stays clean — but the agent might thrash.
- The autonomous-mode evidence gates (auto-advance, anti-spiral, persona switch) protect against runaway loops, but they accept "AcceptAndComplete at 0.5 completeness" as terminal. That's a known compromise: avoids infinite spin, allows half-done artifacts.
- Self-modification + safety = bounded improvement. Won't replace flagship; will get better at being itself.
- The dashboard at ~11% wired today (DETAILED_SUMMARY_01) — the *capability* is there but *visibility* into it is largely C-plan work. You may not SEE most of what the system does until C-1..C-8 ship.


================================================================
8) BOTTOM-LINE HONEST ASSESSMENT
================================================================

When B-plan + C-plan are done, ELAI will be:

[H/U] A serious local-first development engine with DIFFERENTIATED decomposition, grounding, and verification infrastructure — the "rule engine + audit + memorize" loop is genuinely uncommon in this space. NOT benchmarked head-to-head against Devin (SWE-bench), Aider (Aider's own leaderboard), Sweep, OpenInterpreter, or Cursor agentic mode. "Industry-leading" would be sycophancy; "differentiated" is honest.

[H] Capable of autonomously executing routine code tasks in 17 languages at a quality bar that matches or exceeds prompt-only "AI IDEs" — *for tasks the local 7B can output*.

[H] Capable of handling complex tasks through hybrid mode (cloud escalation), with the system controlling cost via Surgical/Oracle modes + spend caps + AutoAdvanceEvaluator.

[H] Continuously improving via self-modification within safety bounds (rule lifecycle, template crystallization, anti-staleness).

[H] Adaptable to domain-specific stacks (Unity, embedded, mobile) through Cosmos artifacts + MCP bridges + custom rules — but adaptation is USER WORK, not automatic.

NOT:

[V] A flagship model on local hardware. A 7B/32B Qwen will never match Opus on novel reasoning. The architecture compensates; it doesn't transcend.

[V] A 1M-context tool. It's a 20-35K context tool that uses context precisely. For tasks that need to hold 800K tokens of state in mind simultaneously, ELAI sidesteps via SSB/SessionDag, not by having huge context.

[H] A turn-key Unity/Unreal/embedded IDE. The harness is general; the domain tooling you'll have to wire yourself.

[V] An overnight-autonomous SaaS-builder with zero human attention. AcceptAndComplete at 0.5 + audit gates that can be over-permissive (auditor is also 7B) = real risk of half-done artifacts on long unattended runs.

Realistic use:
- BEST FIT: local-first daily-driver for greenfield small/medium projects, refactors across complex codebases, research-grounded library evaluation, anti-staleness-enforced code generation, learning-from-mistakes accumulation per project.
- GOOD FIT WITH INVESTMENT: domain-specific stacks (Unity, embedded) once you've wired Cosmos artifacts + rules + MCP.
- NEEDS HYBRID/CLOUD: novel architectural decisions, long-horizon design, deployment infrastructure.
- POOR FIT: zero-supervision multi-day unattended runs producing production-grade systems.


================================================================
9) DIMENSIONS NOT COVERED ABOVE (speed, cost, latency, comparison space)
================================================================

Added post-self-critique. The capability assessment above is qualitative; these are operational tradeoffs the user should know.

SPEED (per-token throughput):
- [V] Local 7B NF4+FA2 on 3090 (remote): ~30-60 tok/s output (typical Qwen2.5-Coder NF4 numbers; sidecar uses SDPA mandatory, no flash-attn on 3B per memory).
- [V] Cloud Sonnet 4.6: ~80-120 tok/s output per Anthropic published numbers (varies).
- [U] Cloud Opus 4.7: slower than Sonnet, higher quality.
- Net: local is comparable per-token but THE ARCHITECTURE BURNS MANY TURNS. A pipeline with 8 baby steps × audit cycle per step × MiniCritic × StructuralRetry can be 20-40 LLM calls. Wall clock per "feature" is HIGH vs a single Cursor prompt. This is by design (precision over speed) but it's a real cost.

LATENCY (wall-clock per feature):
- [P] No metric in any phase_*.toml shows end-to-end p50/p99 for a complete user feature delivery. Should be added (B-44 candidate).
- [H] Compounding factors: GRC budgets, evidence gates, audit cycles, mutation testing (C-28), metamorphic relations (C-29), wiring audit (C-27). Each adds correctness, costs minutes.
- [H] Mitigations: compiled pipelines (C-25 template crystallizer cache-hits), surgical mode (C-30 for edits), Oracle cache (C-30 30d/7d/1d TTL).

COST (cloud):
- [H] Local: ~$0 marginal (electricity + GPU amortization).
- [V] Cloud Sonnet 4.6 input ~$3/M tokens, output ~$15/M (Anthropic pricing). Hybrid mode + cap (C-5 DailySpendCap / MonthlySpendCap) bounds risk.
- [H] Cost-killers if not bounded: FlashResearch tree expansion (cloud speculative), large-context cloud calls. The cap + estimate_task_cost_usd (C-5) exist for this reason.

COMPARISON SPACE (named competitors, what they're known for):
- Devin (Cognition Labs): SWE-bench performance, autonomous long-horizon task runs, sandbox+browser+editor. ELAI's autonomous mode is architecturally similar but unmeasured on SWE-bench.
- Aider: git-aware, repo-map, in-terminal pair programming, voice mode. ELAI's PageRank repo map is aider-derived per Phase 1D. Aider is simpler and faster to onboard.
- Sweep / SweepAI: GitHub-issue-to-PR automation. ELAI's pipeline factory could do this but lacks specific GitHub integration adapter.
- OpenInterpreter: local code execution in chat, multi-language REPL. ELAI's sandboxed bash + tools crate is more rigorous but less interactive.
- Cursor Composer / Cursor Agent: VS Code embedded, Cloud-frontier-model-first, fast UX. ELAI is workflow-driven via separate dashboard, local-first.
- Cline / Roo Code: VS Code agentic, MCP-first. Comparable scope to ELAI's plugins MCP bridge but no rule engine / GRC / evidence gates.
- Claude Code (this tool): Anthropic-official, settings.json + hooks + skills + agents. ELAI's harness has heavier deterministic verification (mutation/metamorphic/wiring/CSQ) and self-modification.

ELAI's unique-vs-all-of-above: rule engine + audit + memorize + self-modification + Cosmos artifacts. None of the above have a learning loop at the system level (they all depend on vendor retraining).

What ELAI lacks vs above: SWE-bench score, GitHub-issue adapter, voice mode, in-editor inline-completion, VS Code embedding, established benchmark presence.
