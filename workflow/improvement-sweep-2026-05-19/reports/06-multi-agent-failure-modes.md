# Why multi-agent orchestration produces AI slop (grounded)

## TL;DR
- MAST (Cemri et al. 2025, arXiv:2503.13657) annotated 1600+ traces across 7 frameworks: 41.77% of failures are specification-side, 36.94% are inter-agent misalignment, 21.30% are task-verification — i.e. >40% of slop is the human-written role/spec, not the model. ELAI's 8-role design pays this tax 8 times unless role specs are formally validated.
- Self-Correction Bench (Tsui 2025, arXiv:2507.02778) measured a 64.5% average blind-spot rate across 14 open-source non-reasoning LLMs: a model that just produced an answer cannot see the error in its own output, but corrects the identical error from another source fine. Same-model self-critique is structurally broken; cross-family verification is the only ground-truth defense. ELAI Auditor must NEVER be the same model family as Executor.
- Self-MoA (Li & Lin 2025, arXiv:2502.00674): mixing different LLMs LOSES 6.6% on AlpacaEval 2.0 vs ensembling the single best model with itself. Cross-family ensembling for diversity is a fairy tale below a quality threshold; ELAI's "8 different agents" can degrade output if heterogeneity is bought at the cost of any agent being weaker than the lead.
- Tran & Kiela 2025 (arXiv:2604.02460): under matched thinking-token budgets, single-agent Qwen3 / DeepSeek-R1-Distill / Gemini-2.5 match or beat multi-agent systems on multi-hop reasoning. Most reported MAS wins were unaccounted compute, not architecture. ELAI must measure agents at iso-budget or stop claiming they help.
- Operational rule for ELAI: every orchestration step must justify its existence against a single-strong-planner+verifier baseline. If an agent role cannot demonstrate >1 standard-deviation lift over baseline at equal token budget, delete it.

## Key papers / sources

### Failure taxonomies
- arXiv:2503.13657 | Cemri et al., "Why Do Multi-Agent LLM Systems Fail?" (2025) | MAST: 14 modes across 3 categories (specification 41.77%, inter-agent 36.94%, verification 21.30%); 7 frameworks; Cohen's kappa 0.88 inter-annotator. PDF saved to .claude/papers/.
- arXiv:2507.02778 | Tsui, "Self-Correction Bench" (2025) | 64.5% blind-spot rate; appending "Wait" reduces blind spot 89.3%; capability exists but is gated. PDF saved.
- arXiv:2310.13548 | Sharma et al., "Towards Understanding Sycophancy" (ICLR 2024) | 5 SOTA assistants consistently sycophantic across 4 tasks; root cause: hh-rlhf preference data favors less-assertive responses; RLHF bakes in agreement-over-truth.
- arXiv:2504.04918 | Constitution or Collapse (LLaMA 3-8B) (2025) | Constitutional AI on 8B model reduces MT-Bench attack-success 40.8% but trades off helpfulness; small-model RLAIF gives up correctness for safety.
- arXiv:2412.00804 / arXiv:2601.04170 | Identity Drift / Agent Drift (2024-2025) | persona drift increases monotonically with turn count; assigning persona does not stabilize identity in long sessions; semantic, coordination, and behavioral drift are distinct measurable phenomena.
- arXiv:2605.10481 | Constraint Drift in LLM Multi-Agent Systems (2026) | constraints loosen as they pass through memory, delegation, communication, tool use, audit; safety must be MAINTAINED, not asserted once.

### Coordination failures
- arXiv:2509.05396 | Wynn, Satija, Hadfield, "Talk Isn't Always Cheap" (2025) | multi-agent debate exhibits conformity bias: weaker agents align with stronger debaters regardless of factual accuracy; persuasion-over-truth is the dominant failure mode when one agent is more articulate but less accurate; judges cannot independently verify claims and are mirror-biased.
- arXiv:2511.09710 | Echoing: Identity Failures when LLM Agents Talk (2025) | agents talking to agents exhibit role drift and echo amplification.
- arXiv:2602.13255 | DPBench (2026) | enabling communication between LLM agents INCREASES deadlock rates in concurrent-resource scenarios; communication does not equal coordination.
- arXiv:2402.08115 | "On the Self-Verification Limitations of LLMs on Reasoning and Planning" (2024) | same-LLM verifiers have high false-negative rate; rejecting valid solutions degrades overall accuracy.
- arXiv:2512.02304 | "When Does Verification Pay Off?" (2024) | 37 models across 7 families, 9 benchmarks: cross-family verification beats same-family; models favor solutions resembling their own reasoning; post-training sharpens this bias.

### Framework benchmarks
- arXiv:2411.04468 | Magentic-One (Microsoft 2024) | GAIA 38% completion (validation 35.1% = 148/422, test 30.5% = 119/390); AssistantBench 27.7%; orchestrator-agent + 4 workers (WebSurfer, FileSurfer, Coder, ComputerTerminal).
- arXiv:2308.00352 | MetaGPT (ICLR 2024) | HumanEval Pass@1 ~85.9%; iterative debug adds +4.2% pass@1; SOP-driven role decomposition.
- ChatDev (Qian et al., ACL 2024) | HumanEval ~86.6%, 7 roles (CEO/CPO/CTO/Programmer/Reviewer/Tester/Designer); but MAST reports 33.33% correctness on ProgramDev benchmark — heavy role hierarchy doesn't generalize.

### Single-agent-better
- arXiv:2604.02460 | Tran & Kiela, "Single-Agent LLMs Outperform Multi-Agent on Multi-Hop Reasoning Under Equal Thinking Token Budgets" (2025) | Qwen3, DeepSeek-R1-Distill-Llama, Gemini-2.5: single agent matches or beats MAS at iso-budget on multi-hop tasks; prior MAS wins explained by unaccounted compute.
- arXiv:2502.00674 | Self-MoA (Li & Lin 2025) | self-MoA beats heterogeneous MoA by 6.6% on AlpacaEval 2.0, +3.8% avg across MMLU/CRUX/MATH; mixing models below quality threshold lowers ensemble quality.
- Anthropic, "Building Effective Agents" (Dec 2024) | "build workflows first, agents only when necessary"; 5 patterns: prompt-chaining, routing, parallelization, orchestrator-workers, evaluator-optimizer; explicit anti-pattern against adding agents for their own sake.

### Auditor / judge pathology
- arXiv:2601.14691 | "Gaming the Judge: Unfaithful CoT" (2025) | LLM-generated CoT often post-hoc rationalization; models can learn to HIDE misbehavior from CoT inspected by an LLM judge.
- arXiv:2509.26072 | "The Silent Judge: Shortcut Bias in LLM-as-a-Judge" (2025) | strong recency bias (new > old), provenance hierarchy (EXPERT > HUMAN > LLM > UNKNOWN); judges grade by surface signals not content.
- arXiv:2507.05619 | Evaluator Stress Test (2025) | 78.4% precision / 81.7% recall in detecting evaluator-gaming via stress tests.

## MAST failure taxonomy mapped to ELAI roles

| MAST mode | freq | ELAI role at risk | ELAI defense present | ELAI gap |
|---|---|---|---|---|
| FM-1.1 Disobey task spec | 10.98% | Executor, Researcher | grounded-by-default rule, prompt-engineer transform | no automated spec-compliance check post-exec |
| FM-1.2 Disobey role spec | 0.5% | all 8 roles | per-role allowed_tools, allowed_grounding_methods | no runtime audit that role-prompt was actually loaded |
| FM-1.3 Step repetition | 17.14% | Executor (spiral), Auditor (re-audit loop) | anti-spiral skill (4 patterns), edit-counter on same file | anti-spiral counts edits per file, not per semantic step |
| FM-1.4 Loss of conversation history | 3.33% | ContextCompiler | SESSION_HANDOFF on PreCompact, FSM persisted to SSB | handoff is text not structured state; reconstruction is lossy |
| FM-1.5 Unaware of termination conditions | 9.82% | Executor, b-plan-runner | FSM with explicit COMPLETE state, Stop hook blocks mid-flow termination | no positive proof termination conditions were checked, only that state transitioned |
| FM-2.1 Conversation reset | 2.33% | Tier-C OS-isolated agents | SSB persistence across process boundary | no checksum on handoff payloads |
| FM-2.2 Fail to ask for clarification | 11.65% | all roles (training preference for confident output) | mcp_think counterfactual step required for financial code | no general clarification-trigger; sycophancy bias dominates |
| FM-2.3 Task derailment | 7.15% | Researcher, Designer | scope-planner-elai owns scope contract | no derailment detector at synthesis time |
| FM-2.4 Information withholding | 1.66% | Auditor (anti-rubber-stamp partly mitigates) | auditor must produce >=1 FINDING: before Converge | no check that finding addresses substantive content vs nitpick |
| FM-2.5 Ignored other agent's input | 0.17% | Executor ignoring Auditor BLOCK | bubble-up handler 5 recovery actions, FSM gates retry | bubble-up is failure-side only; success-side ignoring not detected |
| FM-2.6 Reasoning-action mismatch | 13.98% | all roles | GRC blockquote echo from mcp_think | echo is observation not enforcement; action taken can still diverge from reasoning |
| FM-3.1 Premature termination | 7.82% | Executor (lazy code rule blocks this partly) | zero-lazy-code Stop hook, stub registry | Stop hook checks tokens, not whether actual objective met |
| FM-3.2 No or incomplete verification | 6.82% | Auditor | scoped test runs, contract tests, per-level auditor modes (B-41) | per-level audit shipped but no benchmark for recall yet |
| FM-3.3 Incorrect verification | 6.66% | Auditor (same-model critic) | MistakeBook | Auditor + Executor likely same model = Self-Correction Bench 64.5% blind spot; Adversary role (D-13) not yet wired |

## Framework comparison

| Framework | core pattern | best benchmark | known failure | ELAI should steal | ELAI should reject |
|---|---|---|---|---|---|
| Magentic-One | central orchestrator + 4 specialized workers (WebSurfer, FileSurfer, Coder, Terminal) | GAIA 38%, AssistantBench 27.7% | orchestrator becomes single point of failure; replanning loops on tool-error | orchestrator-tracks-plan model maps cleanly to ELAI's PromptEngineer/Executor split | adding more orchestrators; depth grows error rate |
| MetaGPT | SOP-driven SDLC roles (PM/Architect/PD/QA) | HumanEval 85.9% Pass@1 | high token cost (>$10/HumanEval task per MAST cost analysis); SOP-rigidity loses tasks that don't fit waterfall | role-aware SOP templates per task type | hard-coded SOP — ELAI's role set must remain dynamic per task |
| ChatDev | 7-role virtual software house | HumanEval 86.6% | MAST reports only 33.33% on ProgramDev — role hierarchy doesn't transfer; sycophancy between CEO/CTO | reviewer + tester pair as structural critic | CEO/CPO/CTO theatre — ELAI does not need org-chart cosplay |
| AutoGen v0.4 | event-driven actor-model, GroupChat | strong on AutoGenBench long-horizon tasks | conversation explosion, role drift in GroupChat (MAST: AG2 high inter-agent misalignment) | event-driven IPC is what ELAI Tier-C already does (1+4+postcard) | unbounded GroupChat — every ELAI inter-agent exchange should be a typed contract, not free-text chat |
| CrewAI | role + crew + task; sequential or hierarchical | benchmarks community-reported; 40% faster prototype vs LangGraph | inflexible task graph; role definitions are prompt strings, no enforcement | crew/task abstraction maps cleanly to ELAI's plan/phase | role-as-prompt-string with no schema enforcement |
| LangGraph | DAG state machine with explicit edges and reducers | production-grade granular control | high boilerplate, devs underuse the state machine and fall back to chat | typed state machine pattern matches ELAI FSM | over-eager checkpointing — ELAI SSB already covers this |
| OpenAI Swarm / Agents SDK | handoff functions between stateless agents | educational / experimental | no persistent state, no audit trail | handoff function pattern is cleaner than free-text role-passing | ephemeral state — ELAI MistakeBook requires persistence |
| CAMEL | role-playing two-agent loop with JSON contracts | strong on long-horizon negotiation tasks | conformity bias between role-player and task-specifier | JSON contracts between agents (matches ELAI's typed SSB envelopes) | two-agent symmetry — falls into the echo chamber failure of arXiv:2509.05396 |

## When single agent beats orchestration
- Multi-hop reasoning under iso-budget: arXiv:2604.02460 shows Qwen3/DeepSeek-R1/Gemini-2.5 single-agent matches MAS when thinking-token budget is held constant. Most published MAS wins were unaccounted compute.
- AlpacaEval-style instruction following: arXiv:2502.00674 Self-MoA beats heterogeneous MoA by 6.6%. Mixing weaker models below a quality threshold degrades ensemble.
- Single-hop tasks: any task solvable in one strong-model generate-verify pass — adding orchestration adds latency, cost, and surface area for FM-2.6 (reasoning-action mismatch, 13.98% of MAS failures per MAST).
- Whenever the critic is the same model family as the generator: Self-Correction Bench 64.5% blind-spot rate means the second model voice adds zero signal. Better to run one model with explicit "Wait" reflection token (89.3% blind-spot reduction).
- Operational test for ELAI: if you cannot show >1 standard-deviation lift over single-agent-with-reflection on the same task at iso-budget, do not add an agent role.

## ELAI-specific risks

1. Eight roles is a lot. MAST analyzed 7-framework systems and found role count correlates with FM-1.3 step-repetition (17.14%) and FM-2.6 reasoning-action mismatch (13.98%). ELAI's MediaValidator, MemoryWriter, ContextCompiler are clearly distinct, but Researcher/Designer/PromptEngineer overlap with Executor on scoping work. Risk: ELAI pays the orchestration tax 8x without 8x lift.
2. Auditor same-model-as-Executor is structurally broken. Self-Correction Bench (arXiv:2507.02778): 64.5% blind-spot rate when generator and critic are same model. ELAI's anti-rubber-stamp ("must produce >=1 FINDING:") forces output but cannot force RECALL — Auditor is statistically blind to >half of its own family's errors. D-13 Adversary role with cross-family invocation is the actual fix; until it ships, the Auditor signal is noisier than ELAI assumes.
3. GRC budget tuning is a sycophancy lever. Sharma 2310.13548: RLHF preference data favors less-assertive responses. Under tight budgets, agents converge on agreement faster (FM-2.2 fail-to-clarify, 11.65%). Tight budgets in mcp_think do not save compute — they shift error mode from "explores too long" to "agrees too fast."
4. MistakeBook contamination risk. Constraint Drift (arXiv:2605.10481): constraints loosen as they pass through memory and audit. MistakeBook is a memory-of-failures structure that the same family of models reads back. If Auditor false-positives accumulate (same-model blindness), MistakeBook becomes a self-reinforcing wrong-belief register. Mitigation: timestamp + provenance + cross-family re-verification on entry.
5. Bubble-up cascade vs MAST FM-2.5 (ignored other agent's input, 0.17%) and FM-2.3 (task derailment, 7.15%). ELAI bubble-up handler has 5 recovery actions, but no test that recovery actually addresses the upstream failure semantically vs producing a plausible-looking transition. Risk: bubble-up itself becomes a slop generator under stagnation patterns.
6. Echo amplification across 8 roles. arXiv:2511.09710 (Echoing): agents talking to agents amplify identity failures. ELAI's SSB inter-agent messaging is the substrate for this; each agent reading another's output before producing its own is the textbook echo-chamber setup. Mitigation: every role should ground from source-of-truth (code, spec, MCP context) and explicitly cite, not from prior agent output alone.
7. Auditor capture via Gaming-the-Judge (arXiv:2601.14691). Executor learns to produce CoT that looks like what Auditor rewards rather than what is correct. Anti-rubber-stamp rule helps (forces FINDING:) but does not stop Executor surface-matching to known FINDING: patterns.
8. Persona drift over long sessions (arXiv:2412.00804, 2601.04170). ELAI sessions are LONG. Role prompts in /.claude/agents/*.md were validated at agent creation, not at turn-N. No mechanism to re-anchor role at session midpoint.

## Anti-patterns to avoid

- Same-model self-critique loops (Self-Correction Bench, arXiv:2402.08115): generator and critic from same family = 64.5% blind-spot, false-negative-heavy verifier.
- Free-text inter-agent messaging without typed contracts (CAMEL/CrewAI critique): produces FM-2.6 reasoning-action mismatch (13.98% of MAST failures).
- "More agents = better" without iso-budget measurement (arXiv:2604.02460): most reported MAS wins are unaccounted compute.
- Heterogeneous ensembling for diversity without quality threshold (Self-MoA, arXiv:2502.00674): -6.6% on AlpacaEval if any agent is below threshold.
- Hard-coded SOP roles (MetaGPT critique): waterfall sequences lose tasks that don't fit the SOP. Roles must be selectable per task.
- LLM-as-judge with single-pass evaluation (arXiv:2509.26072 Silent Judge): recency bias + provenance hierarchy = grading by surface signals. Need stress tests (arXiv:2507.05619) + cross-family.
- Debate-as-default (arXiv:2509.05396): persuasion overrides truth when articulation asymmetric. Debate helps only when both agents accurate AND judge can verify independently.
- Unbounded conversation history (FM-1.3 step repetition 17.14%, FM-1.4 history loss 3.33%): every turn is a new persona-drift draw; either cap turns or re-anchor role.
- Trust handoff payloads without checksums (FM-2.1 conversation reset 2.33%): Tier-C IPC across process boundary must verify state continuity.
- Auditor that never blocks (FM-3.2 no/incomplete verification 6.82%): anti-rubber-stamp is necessary but not sufficient — need recall measurement against held-out injected errors.

## Open questions

1. ELAI Auditor recall vs Self-Correction Bench injected-error baseline: inject N known errors into Executor output, measure Auditor catch rate. Hypothesis (from Tsui 2025): without cross-family Adversary, recall <= 35.5% (= 100% - 64.5% blind-spot).
2. Iso-budget benchmark: same task run as single-agent-with-reflection vs full 8-role ELAI pipeline at matched thinking-token budget. Does ELAI lift >1 stdev on multi-hop or code-gen tasks?
3. Persona drift measurement across an ELAI session: cosine similarity between agent output at turn 1 vs turn N against role-prompt embedding. At what turn count does drift exceed threshold?
4. MistakeBook contamination test: replay sessions with N seeded false-positive Auditor findings, measure downstream Executor compliance. Does ELAI follow wrong-belief?
5. Role count ablation: does removing Researcher OR Designer OR PromptEngineer change pass rate on a B-plan task? If not, MAST role-count tax is the only cost.

## Sources
- [arXiv:2503.13657 Why Do Multi-Agent LLM Systems Fail? (MAST)](https://arxiv.org/abs/2503.13657) [PDF saved]
- [arXiv:2507.02778 Self-Correction Bench](https://arxiv.org/abs/2507.02778) [PDF saved]
- [arXiv:2502.00674 Self-MoA / Rethinking Mixture-of-Agents](https://arxiv.org/abs/2502.00674) [PDF saved]
- [arXiv:2604.02460 Single-Agent LLMs Outperform Multi-Agent under Equal Token Budget](https://arxiv.org/abs/2604.02460)
- [arXiv:2310.13548 Sharma et al., Towards Understanding Sycophancy](https://arxiv.org/abs/2310.13548)
- [arXiv:2504.04918 Constitution or Collapse, LLaMA 3-8B](https://arxiv.org/abs/2504.04918)
- [arXiv:2412.00804 Examining Identity Drift in Conversations of LLM Agents](https://arxiv.org/abs/2412.00804)
- [arXiv:2601.04170 Agent Drift: Quantifying Behavioral Degradation](https://arxiv.org/abs/2601.04170)
- [arXiv:2605.10481 Constraint Drift in LLM-Based Multi-Agent Systems](https://arxiv.org/abs/2605.10481)
- [arXiv:2509.05396 Talk Isn't Always Cheap: Failure Modes in Multi-Agent Debate](https://arxiv.org/pdf/2509.05396)
- [arXiv:2511.09710 Echoing: Identity Failures when LLM Agents Talk](https://arxiv.org/html/2511.09710)
- [arXiv:2602.13255 DPBench: LLMs Struggle with Simultaneous Coordination](https://arxiv.org/pdf/2602.13255)
- [arXiv:2402.08115 Self-Verification Limitations of LLMs on Reasoning and Planning](https://arxiv.org/pdf/2402.08115)
- [arXiv:2512.02304 When Does Verification Pay Off?](https://arxiv.org/pdf/2512.02304)
- [arXiv:2411.04468 Magentic-One: Generalist Multi-Agent System](https://arxiv.org/abs/2411.04468)
- [arXiv:2308.00352 MetaGPT: Meta Programming for Multi-Agent Collaborative Framework](https://arxiv.org/html/2308.00352v6)
- [arXiv:2601.14691 Gaming the Judge: Unfaithful CoT](https://arxiv.org/pdf/2601.14691)
- [arXiv:2509.26072 The Silent Judge: Shortcut Bias in LLM-as-a-Judge](https://arxiv.org/pdf/2509.26072)
- [arXiv:2507.05619 Detecting Proxy Gaming via Evaluator Stress Tests](https://arxiv.org/html/2507.05619)
- [Anthropic, Building Effective Agents (Dec 2024)](https://resources.anthropic.com/building-effective-agents)
