# Is ELAI the best agent harness? A market-grounded design evaluation

Date: 2026-07-10  
Evaluation mode: **AS-SPECIFIED** for architecture; implementation maturity is reported separately and does not contaminate the design verdict.  
Market evidence: current first-party documentation, official repositories, and vendor engineering reports only.  

## Executive verdict

**No: it is not defensible to call ELAI the best harness overall, or the best harness currently.** I found no controlled public benchmark covering ELAI, Claude Code, Codex, Hermes, Kilo, OpenHands, Copilot, Cursor, Devin, Kiro, Gemini CLI, Cline, and the smaller research harnesses while holding the model, task, repository, compute, and evaluation constant.

**A narrower claim is defensible:** as specified, ELAI is one of the most complete documented designs reviewed for **trust-evidenced autonomous execution**—an agent run whose important claims are checked by artifacts outside the generating model, whose side effects are journaled, and whose result can be independently inspected or replayed. That is a real architectural edge. It is not proof of best overall productivity, usability, reliability, or value.

**ELAI is strongly aligned with where the market is moving.** Major harnesses are converging on subagents, durable work items, worktree/container/VM isolation, repository-native guidance, skills, MCP, hooks, persistent memory, human interruption, and auditable artifacts. ELAI covers all of those and goes further on receipts, gate independence, taint tracking, replay, provenance, and verifier-owned completion.

**ELAI is not literally future-proof.** No harness is. Its stable ideas are future-resilient; its current breadth is not. The corpus has accumulated overlapping routers, claim types, provenance scales, context systems, gates, and product verticals faster than it has canonicalized them. The best future version of ELAI is therefore **smaller than the current plan corpus**: a narrow trust kernel that can wrap Claude Code, Codex, Hermes, Kilo, OpenHands, and future agents, with ELAI's full native orchestrator as an optional higher-assurance destination.

My recommendation is to make the already-specified external-agent wrapper and evidence format the first product:

1. freeze feature expansion;
2. fix the trust-critical contradictions listed below;
3. pull N's canonicalization and K/Q's proof economics forward;
4. ship the RRA/evidence standard, standalone verifier, full execution receipts, process confinement, and PR/CI evidence surface;
5. prove a same-model safety and cost advantage;
6. move research, trading, voice, canvas design, personal-world, and model-training features into optional packs until the proof spine wins.

## Plain-language summary

ELAI is trying to build the **black box recorder, safety interlocks, and independent inspector** for AI workers. Most competitors focus first on making the AI worker faster, smarter, or easier to use. That gives ELAI a meaningful and unusual strength: it is designed to show *why* a job should be trusted, not merely say the job is finished.

But ELAI is also trying to build the office, factory, research lab, trading desk, personal organizer, and app ecosystem around those workers. That breadth is now its largest risk. The core safety idea is excellent; the surrounding system is too large and contains some contradictions that weaken the very trust story it wants to sell.

So the non-technical answer is:

- **Best trust architecture on paper?** One of the strongest documented designs reviewed; an overall winner cannot be established.
- **Best complete coding tool today?** Not proven, and I would not claim it.
- **Going in the same direction as the major companies?** Yes—very clearly.
- **Future-proof?** Directionally resilient, but only if ELAI shrinks its permanent core and treats models and product surfaces as replaceable attachments.
- **What should happen next?** Finish and prove the safety recorder before building more rooms around it.

## 1. What was evaluated

### 1.1 The ELAI object under judgment

The overview defines ELAI as a local-first deterministic coding-and-reasoning harness where the model is a replaceable cog and the harness owns state, gating, and completion (`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:14-27`). The core phases add:

- hybrid code retrieval and a dependency knowledge graph;
- a typed Grounded Reasoning Chain;
- a nine-state lifecycle FSM;
- an append-only Shared State Board;
- deterministic parallel orchestration;
- process-isolated agents;
- tools, plugins, pipeline generation, autonomous execution, self-modification, media validation, a dashboard, and multiple runtime backends (`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:36-131`).

The letter plans then turn that substrate into the trust product:

- B-C activate and test it;
- D-E add reasoning improvement and broad hardening;
- F-G add fail-closed write gates, taint, RRA, SAFEDELTA, provenance, and the reviewer cockpit;
- H-I add efficiency, evidence-ranked search, context lifecycle, anti-gaming, and the external-agent wrapper;
- J adds the harness redesign and closes perception/control/journal gaps;
- K proves, distributes, and prices the wedge;
- L applies the trust spine to arbitrary claims;
- M packages the company/evidence standard;
- N canonicalizes fragmented subsystems;
- O-P add personal research and personal software-world surfaces;
- Q turns the spine on its own durability, cost, and gate integrity (`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:467-489`).

### 1.2 Evaluation rules

I used two distinct lenses:

1. **Design merit:** assume every specified mechanism is built correctly; do not mark ELAI down merely because an item is planned.
2. **Current market readiness:** ask whether a user today can rationally call it the best current product. This necessarily depends on shipping, usability, benchmarks, support, and ecosystem, but it is reported separately.

“Best” was decomposed into:

- verified correctness and anti-gaming;
- replay, provenance, and auditability;
- orchestration and long-horizon recovery;
- model/provider/runtime independence;
- security and permission boundaries;
- memory and context management;
- productivity, cost, and human review time;
- usability and distribution;
- ecosystem and maintainability;
- empirical proof.

No single harness wins all ten.

### 1.3 Market scope and limits

The detailed comparison covers Claude Code, OpenAI Codex and Symphony, Hermes Agent, Kilo Code, OpenHands, GitHub Copilot, Cursor, Devin, Kiro, Gemini CLI, Cline, mini-SWE-agent/SWE-agent, Aider, and the discontinued Roo Code. This is broad, not literally exhaustive. Closed internal harnesses cannot be evaluated, and current products change weekly.

## 2. What ELAI genuinely does better at the design level

### 2.1 Completion is owned by evidence, not the generator

The core doctrine is that important completion evidence is produced or captured outside the agent that authored the change. F-4 binds a PASS claim to orchestrator-captured output; I's search ranks candidates by receipts rather than an LLM judge; Q corrupts evidence deliberately to prove each gate really fails closed (`docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md:75-91`; `docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:91-108`; `docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md:51-56`).

Competitors increasingly support hooks that *can* run tests. ELAI's stronger idea is that the verifier and its evidence are structurally necessary. That is the clearest differentiator.

### 2.2 The enforcement plane can sit outside the model

I-1 specifies a process-separated supervisor as the only executor of side effects. Agents emit intents and hold no direct credentials or gate code. This is a stronger boundary than prompt instructions or an approval dialog (`docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:35-50`).

This matches Hermes' own security warning that in-process scanners and allowlists are heuristics, not containment; Hermes recommends whole-process OS isolation for untrusted inputs ([Hermes security policy](https://github.com/NousResearch/hermes-agent/blob/main/SECURITY.md)).

### 2.3 Evidence-ranked search is better than judge-ranked sampling

I-4 specifies a ladder from best-of-N to tournament/tree search where compile, tests, mutation coverage, progress, and other receipts determine which branch survives (`docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:91-108`, `:324-340`). The model proposes; hard vetoes and measured evidence rank.

This is more defensible than selecting the branch another model “likes,” although the expensive tree-search rungs must still earn their cost over the simple best-of-N rung.

### 2.4 RRA is a product artifact, not just a transcript

G-2 defines a portable run artifact containing the action stream, seed, model/hardware pins, and gate-verdict hashes; G-6 and I-38 use controlled gate-on/gate-off ablations; I-40 exposes the format and wrapper to external agents (`docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md:38-57`, `:111-127`; `docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:720-736`, `:755-773`).

Q-5 later states the correct honesty boundary: harness replay can be checked broadly; local tokens can be hash-verified only under a same-class condition; cloud execution supports action-stream authentication, not deterministic token replay (`docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md:44-49`).

No public competitor source reviewed here describes this exact joined artifact: taint + side effects + gate receipts + replay verification + a standalone verifier.

### 2.5 The system tries to falsify its own value

SAFEDELTA measures dynamically exploitable changes reaching a reviewer with gates on versus off; QDELTA measures defective diffs; K-4 measures total gate latency and over-blocking; Q-6 measures evidence-corruption kill rate (`docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md:111-127`; `docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:720-736`; `docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md:180-195`; `docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md:51-56`).

That is excellent scientific hygiene. The spec does not merely say gates help; it defines ablations that can kill the claim.

### 2.6 ELAI treats plan-corpus integrity as a machine problem

N admits that the earlier corpus created four claim types, ten routers, six intensity deciders, five provenance scales, four counterfactual engines, and five context cores. It then specifies one canonical subsystem per concern and a plan-authoring validator that rejects re-forking (`docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md:10-18`, `:46-65`, `:228-248`).

Few harnesses try to make their own roadmap composition mechanically checkable. This is differentiated and necessary.

### 2.7 ELAI can become a trust layer for other agents

I-40 explicitly separates a thin wrapper/standard from the deep native orchestrator (`docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:755-773`). Strategically, this is ELAI's strongest move. Winning the plugin or foundation-model race against large vendors is unlikely; becoming the evidence format and independent verifier used *around* their agents is plausible.

## 3. Where competitors are better

### 3.1 Claude Code: native collaboration and focused ergonomics

Claude Code now exposes subagents and experimental agent teams with independent sessions, shared tasks, dependencies, peer messaging, and task claiming ([subagents](https://code.claude.com/docs/en/sub-agents), [agent teams](https://code.claude.com/docs/en/agent-teams)). It also ships hooks, permissions, checkpoints, memory, and mature terminal/editor interaction.

ELAI has the stronger evidence architecture. Claude Code has the more focused interaction design and a native peer-collaboration system that users can operate without understanding a large formal lifecycle.

### 3.2 Codex/Symphony: objective-driven issue orchestration and distribution

Current Codex releases ship subagents across app, CLI, and IDE, plus repository guidance, skills, MCP, opt-in local memories, sandboxing, and a broad local/cloud surface ([Codex subagents](https://developers.openai.com/codex/agent-configuration/subagents), [customization](https://developers.openai.com/codex/concepts/customization), [memories](https://developers.openai.com/codex/customization/memories), [security/sandbox](https://developers.openai.com/codex/agent-approvals-security)). Codex hooks currently have handler-specific limitations: command handlers are the dependable path; prompt/agent handlers and async/interception coverage should not be treated as universal enforcement ([hooks](https://developers.openai.com/codex/hooks)).

OpenAI's Symphony design turns an issue tracker into a durable control plane with isolated workspaces, retries, reconciliation, and observable worker runs ([announcement](https://openai.com/index/open-source-codex-orchestration-symphony/), [Draft v1 spec](https://github.com/openai/symphony/blob/main/SPEC.md)). The reference implementation is a low-key engineering preview for trusted environments, not a mature security baseline. Separately, OpenAI's engineering account reports that rigid agent nodes and strict transitions became too limiting as models improved, motivating objective-driven workers with tools and context; that lesson comes from the engineering account, not the normative Draft v1 spec.

ELAI should keep a deterministic safety envelope, but not encode model intelligence into an ever-more-detailed workflow graph.

### 3.3 Hermes: the strongest narrow-waist lesson

Hermes is the closest philosophical competitor: provider/local-model independence, persistent memory, agent-authored skills, delegation, cron, messaging gateways, plugins, MCP, searchable sessions, and RL tooling ([repository](https://github.com/NousResearch/hermes-agent), [tools](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/features/tools.md), [memory](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/features/memory.md)).

Its `AGENTS.md` states two valuable architectural rules: prompt caching is sacred, and the core should be a narrow waist with capabilities at the edges ([Hermes development guide](https://github.com/NousResearch/hermes-agent/blob/main/AGENTS.md)). ELAI's verification is much deeper; Hermes' permanent core and extension story are cleaner.

### 3.4 Kilo: provider-neutral practicality

Kilo supports 30+ providers, local and custom endpoints, context-isolated custom subagents, detailed permissions, skills/MCP, and an Agent Manager for parallel sessions that can use worktree-isolated Git/filesystem state, with live diffs and PR state ([providers](https://kilo.ai/docs/ai-providers), [subagents](https://kilo.ai/docs/customize/custom-subagents), [permissions](https://kilo.ai/docs/customize/agent-permissions), [Agent Manager](https://kilo.ai/docs/automate/agent-manager)). Ports, databases, containers, and external caches can still collide unless separately isolated.

Kilo deprecated its dedicated Orchestrator mode because capable primary agents can delegate directly ([deprecated Orchestrator](https://kilo.ai/docs/code-with-ai/agents/orchestrator-mode)). That is another signal that orchestration should be adaptive and task-shaped, not ceremony imposed on every job.

### 3.5 OpenHands: composable SDK and public research discipline

OpenHands provides a typed, serializable SDK with conversations, events, configurable tools, provider-independent LLMs, skills, MCP, local/Docker/remote/Kubernetes workspaces, security policies, and observability ([SDK](https://docs.openhands.dev/sdk/index), [architecture](https://docs.openhands.dev/sdk/arch/overview), [runtime](https://docs.openhands.dev/openhands/usage/architecture/runtime)).

ELAI's verifier-owned completion is stronger. OpenHands is currently the clearer reusable software-agent foundation and has a stronger public benchmark/research identity.

### 3.6 GitHub Copilot: repository and enterprise control plane

Copilot combines custom agents, subagent orchestration, MCP, repository-scoped cloud work, hooks, memory, PR/CI integration, firewall policy, RBAC, and audit events ([custom agents](https://docs.github.com/en/copilot/reference/custom-agents-configuration), [hooks](https://docs.github.com/en/copilot/reference/hooks-reference), [cloud agent](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent), [memory](https://docs.github.com/en/copilot/concepts/agents/copilot-memory)). Copilot CLI Fleet supplies an explicit task/dependency graph and dispatches ready work in parallel; Fleet SDK surfaces remain experimental ([Fleet](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/fleet)). The cloud-agent firewall is useful but does not mediate every setup or MCP path ([firewall](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/customize-the-agent-firewall)).

ELAI is more vendor-neutral and more ambitious about replay. GitHub owns the review/distribution surface where organizations already work. K-3's PR/CI evidence emitter is therefore strategically essential, not cosmetic.

### 3.7 Cursor and Devin: product polish and managed autonomy

Cursor has a polished foreground/background fleet experience, remote machines, rules/memory, worktrees, model choices, checkpoints, and remote steering—an analyst judgment about product integration, not a measured overall win ([background agents](https://docs.cursor.com/background-agent), [Cursor 2.5](https://cursor.com/changelog/2-5), [multitask/worktrees](https://cursor.com/changelog/04-24-26), [current changelog](https://cursor.com/changelog), [memories](https://docs.cursor.com/en/context/memories)). Its own documentation warns that always-on internet plus auto-running commands creates data-exfiltration risk.

Devin provides organization knowledge, reusable playbooks, schedules, full session timelines, and multiple managed sessions in isolated VMs ([advanced capabilities](https://docs.devin.ai/work-with-devin/advanced-capabilities), [playbooks](https://docs.devin.ai/product-guides/creating-playbooks)).

ELAI has the better auditability design. These competitors are better examples of reducing orchestration into an understandable product.

### 3.8 Gemini CLI, Cline, Aider, and mini-SWE-agent: simplicity matters

Gemini CLI ships MCP, extensions, memory, checkpointing, telemetry, and optional sandboxing; ACP/worktree features remain experimental, plan-mode material is still evolving, and arbitrary provider support is not a shipped general capability ([CLI reference](https://geminicli.com/docs/cli/cli-reference/), [memory](https://geminicli.com/docs/cli/tutorials/memory-management/), [checkpointing](https://geminicli.com/docs/cli/checkpointing/), [sandbox](https://geminicli.com/docs/cli/sandbox/), [ACP](https://geminicli.com/docs/cli/acp-mode/)). Cline offers provider choice, Plan/Act separation, rules, hooks, and persistent shadow-Git checkpoints ([checkpoints](https://docs.cline.bot/core-workflows/checkpoints), [rules](https://docs.cline.bot/customization/cline-rules)). Aider remains a compact terminal pair-programmer with a repo map, model-specific edit formats, architect/editor separation, and public editing benchmarks ([documentation](https://aider.chat/docs/), [architect/editor evidence](https://aider.chat/2024/09/26/architect.html)). mini-SWE-agent intentionally keeps a small, inspectable benchmark harness ([repository](https://github.com/SWE-agent/mini-swe-agent)).

Their lesson is not that ELAI's trust mechanisms are unnecessary. It is that every mechanism should have to beat a simpler arm under the same model and budget.

### 3.9 Kiro: strong spec-first workflow with important boundary gaps

Kiro is a strong commercial example of repository specs, steering, subagents, and enterprise governance. Its limits matter in this comparison: subagents cannot access Specs and hooks do not run inside them; models are curated rather than general BYOK; and stdio MCP subprocesses can execute outside the agent sandbox ([subagents](https://kiro.dev/docs/chat/subagents/), [MCP sandbox boundary](https://kiro.dev/docs/autonomous-agent/sandbox/mcp/), [models](https://kiro.dev/docs/models/), [governance](https://kiro.dev/docs/enterprise/governance/)). ELAI's advantage is a more unified verification boundary; Kiro's advantage is a more focused spec-first product.

### 3.10 Roo Code is no longer a current contender

Roo's repository was archived and the extension shut down on 15 May 2026 ([official repository notice](https://github.com/RooCodeInc/Roo-Code)). Its historical modes, MCP integration, and Boomerang orchestration remain useful prior art, but a current comparison should use Kilo and Cline instead.

## 4. Is ELAI moving where the large companies are moving?

### 4.1 Yes: the shared direction is unmistakable

Across first-party sources, the market is converging on:

1. isolated execution in worktrees, containers, or VMs;
2. durable tasks or issues rather than disposable chats;
3. parallel specialized agents;
4. repository-native instructions, skills, and specs;
5. MCP/plugins as external capability planes;
6. deterministic hooks and policy enforcement;
7. local/cloud/hybrid execution and remote steering;
8. persistent, inspectable memory;
9. OTel-compatible events, traces, and artifacts;
10. human approval, interruption, rewind, review, and escalation;
11. tests, CI, screenshots, videos, or other proof-of-work;
12. measured feedback loops that improve the environment around the model.

OpenAI describes the engineering shift as humans specifying intent and building environments and feedback loops while agents execute ([Harness engineering](https://openai.com/index/harness-engineering/)). Anthropic's 2026 harness work uses planner/generator/evaluator separation, structured handoffs, clean context resets, and a skeptical external evaluator ([long-running application harness](https://www.anthropic.com/engineering/harness-design-long-running-apps)). These are close to ELAI's central ideas.

### 4.2 ELAI is ahead of the public market on one axis

Commercial products increasingly produce proof-of-work, but their public designs usually treat verification as configured workflow. ELAI treats it as the product: independent receipts, a gate manifest, replay, controlled ablation, a standalone verifier, and an evidence interchange format.

That is forward-looking. It maps to the likely bottleneck as autonomous work scales: human attention cannot inspect every action, so machines need compact, trustworthy evidence about what happened and what remains uncertain.

### 4.3 ELAI diverges from the market in a risky way

The strongest 2026 product signals favor **more capable agents inside simpler, objective-oriented harnesses**:

- Symphony says rigid state-machine nodes did not age well;
- Kilo removed its dedicated orchestrator;
- mini-SWE-agent deliberately shrank the scaffold;
- Hermes keeps a narrow waist and pushes capability to extensions.

ELAI's nine-state lifecycle is useful as an audit and commitment envelope (`docs/specs/guide/steps/phase-index-MASTER.txt:391-415`). The risk is using states to prohibit useful actions or force ceremonial role transitions rather than merely controlling high-risk commitments. The future-proof form is:

- **strict at boundaries:** credentials, writes, network, irreversible actions, evidence admission, publication, and completion;
- **flexible inside the boundary:** planning, exploration, decomposition, testing order, and agent topology selected dynamically for the objective.

## 5. Why the broad “best harness” claim is not defensible

### 5.1 N is both the cure and the confession

N's quantified fragmentation is not a hypothetical future problem; it is evidence that the roadmap process already created competing machinery (`docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md:10-18`). A trust product cannot ask users to trust ten routers and several overlapping claim/provenance systems whose precedence is defined later.

N should be a prerequisite, not cleanup after J-M.

### 5.2 Gate proliferation can make the safe system unusable

K explicitly says false-positive friction killed prior gate generations and adds a whole-system gate-tax ceiling plus risk-tiered gating (`docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md:101-112`, `:180-215`). That is the correct diagnosis.

The doctrine should not be “everything fails closed.” It should be:

- high-risk, irreversible, security, privacy, finance, credential, and publication boundaries fail closed;
- quality and productivity checks can quarantine, retry, request evidence, or remain advisory according to measured risk;
- missing telemetry is itself an error, but not every telemetry error should destroy low-risk exploratory work.

OpenAI's harness report makes the opposite economic point for high-throughput, low-risk changes: waiting can cost more than cheap correction, so it uses minimal blocking merge gates while mechanically enforcing a small set of boundaries ([Harness engineering](https://openai.com/index/harness-engineering/)). ELAI's K-5 risk tiers should be foundational.

### 5.3 The corpus opens too many product fronts

The trust spine is mixed with:

- finance-specific autonomous trading gates;
- a generative canvas and app-design mode;
- voice ingress/narration;
- a personal research daemon;
- a personal app fleet and world event bus;
- connector packs and workflow-from-goal;
- company licensing/billing;
- SFT/GRPO self-training;
- diffusion drafting;
- external Empire integration.

These can be valuable products, but they are not all load-bearing to the trust wedge. O and P explicitly add surfaces rather than proof-spine numbers (`docs/thefinalglance/O-plan/O-SUMMARY_INDEX.md:9-13`; `docs/thefinalglance/P-plan/P-SUMMARY_INDEX.md:9-17`). K says the number is the product and other work should wait behind proof/distribution (`docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md:588-598`). The roadmap should obey that statement.

### 5.4 Model power still matters

The harness can expose, verify, route, and compound a model's capability; it cannot eliminate the model's ceiling. A frontier model in a thin harness can outperform a weak local model behind perfect governance on a genuinely novel task.

ELAI's defensible thesis is not “the model does not matter.” It is:

> The harness determines how much model capability can be trusted, reviewed, recovered, and compounded per dollar.

That is a strong thesis and does not require pretending models are interchangeable in raw capability.

### 5.5 Empirical superiority is a separate readiness claim

K's design correctly refuses an unqualified external claim without measured proof (`docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md:101-112`). SAFEDELTA, QDELTA, gate-tax, routing economics, reviewer time, and hallucination delta make the architecture stronger; whether current implementations establish superiority belongs to the market/readiness axis, not the as-specified design score.

## 6. Trust-critical design flaws to fix before a public claim

These are design defects, not “not built yet” complaints.

### P0-1. A journal failure cannot be allowed to disappear

B-2 says an SSB write failure must not block an FSM transition (`docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md:114-124`). The core contract says the SSB write completes before the result is dispatched and the system's thesis says missing evidence fails closed (`docs/specs/guide/steps/phase-index-MASTER.txt:431-439`; `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:21-27`).

A transition without its journal row breaks replay and completeness. Use a transactional outbox/commit protocol or fail/quarantine the transition.

### P0-2. G-3's shell-expansion plan is unsafe and technically wrong

The plan says `std::fs::canonicalize` resolves `~`, globs, and symlinks (`docs/thefinalglance/G-plan/G-3_blast_radius_gates.md:44-49`, `:106-117`). It resolves symlinks and filesystem paths; it does not perform shell tilde or glob expansion. The plan also admits the source of post-expansion argv is unresolved (`:292-294`).

Letting a shell perform expansion before the gate can execute command substitution. The fix is a side-effect-free shell AST evaluator, explicit rejection/containment of command substitutions, canonical allowed-path resolution, and an OS-enforced boundary after resolution.

### P0-3. A Merkle root is not a signature

G-8 describes a local BLAKE3 Merkle proof as the offline signing default (`docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md:145-159`). A Merkle root proves internal integrity relative to itself; an attacker who changes the artifact can recompute it.

Use a keyed signature such as Ed25519, bind signer identity and gate-manifest version, and externally checkpoint the signed root. The hash chain remains useful, but it is not authentication by itself.

### P0-4. The full receipt contract is fragmented rather than canonical

F-4 binds a PASS to `stdout_hash` (`docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md:75-91`). Later plans add much of the missing envelope in separate places: G-2 adds run/model/hardware/evidence pins, H-25 adds code/data/environment/result provenance, I-18 adds the row chain, and I-45 adds the objective hash (`docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md:38-57`; `docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md:401-413`; `docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:342-357`, `:888-901`). The design problem is fragmentation: one canonical `TestExecutionReceipt` should bind all of these, including:

- objective/task hash;
- source-tree/working-copy hash;
- exact command and arguments;
- working directory;
- environment allowlist hash;
- toolchain/container image and lockfile;
- stdout and stderr;
- exit status and timeout reason;
- test discovery/list;
- gate/checker versions;
- timestamp from the controlled clock;
- signer/verifier identity.

Otherwise individual real hashes can still be assembled incorrectly or attached to the wrong command, source state, or environment.

### P0-5. `elai wrap` is bypassable without whole-process confinement

I-40 proposes an MCP/proxy wrapper around an external agent's tool traffic (`docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:755-773`). An external CLI with direct filesystem, process, credential, or network access can bypass that proxy.

The wrapper must run the full process tree under the I-1 supervisor/OS boundary, with credentials withheld and filesystem/egress controls enforced outside the agent. MCP interception alone is not a security boundary.

### P0-6. The determinism claim needs one normative matrix

The overview's whole-session byte-recomputation wording (`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:21-27`) is stronger than the detailed qualifications: live LLM BSP participation can vary (`docs/specs/guide/steps/phase-index-MASTER.txt:474-478`), and Q scopes cloud to action-stream fidelity (`docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md:44-49`).

Publish one canonical table:

| Layer | Honest claim |
|---|---|
| Pure harness/FSM/journal fold | Byte-recomputable under pinned version and seed |
| Local inference, same model build/hardware class | Token-hash verifiable if deterministic kernels pass |
| Local inference, different hardware class | Divergence-localizable, not necessarily token-identical |
| Cloud model | Action/receipt stream authenticated; model tokens not replay-guaranteed |
| External live world | Historical re-execution only against captured bytes/cassette; current-world equivalence not claimed |

### Normative cleanup. Make H-49 authoritative for rewind

H-35's older journal-replay wording conflicts with H-49, which specifies the content-addressed snapshot store needed to preserve file bytes (`docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md:566-572`, `:824-840`). Under the full as-specified design, H-49 repairs the mechanism. Declare H-49 authoritative and delete stale claims that hashes/journal rows alone reconstruct files.

## 7. What I would keep, improve, move, and remove

### 7.1 Keep as the permanent trust kernel

Keep these concepts central:

- supervisor-owned side effects and credentials;
- isolated workspaces/process trees;
- canonical append-only event journal;
- taint/provenance on untrusted inputs;
- frozen gate manifest and independently versioned verifier;
- complete execution receipts;
- RRA plus open evidence schema;
- Claim/Belief + evidence receipt;
- evidence-ranked best-of-N;
- risk-tier policy arbitration;
- human-rendered verdicts and interruption;
- PR/CI evidence emitter;
- gate-on/off and gate-tax measurement;
- standing re-verification goals;
- provider/runtime capability negotiation.

This is the moat.

### 7.2 Improve the architecture

#### A. Move N, K, and Q ahead of new surfaces

Pull forward:

- N-1 measurement spine;
- N-2 corpus composition validator;
- N-4 canonical Claim/Belief;
- N-5 policy/intensity arbitration;
- N-6 trust lattice/admission family;
- N-11 journal fold;
- N-12 duplicate deletion;
- K-1 claims lint;
- K-3 PR emitter;
- K-4 gate tax;
- K-5 risk tiers;
- Q-6 gate corruption corpus;
- Q-8 small verifier ceiling.

No new plan should be accepted if it creates a second router, evidence object, provenance scale, fold path, or benchmark runner.

#### B. Make orchestration adaptive by risk

Use three operating tiers:

1. **Fast:** one agent, minimal receipts, low-risk reversible work.
2. **Standard:** plan -> implement -> independent verify, with normal receipts.
3. **High assurance:** full decomposition, isolated parallelism, mutation/security/causal checks, formal gates, and human approval.

Keep the FSM as a commitment/audit envelope. Let the agent choose the internal reasoning topology. Benchmark the current nine-state/tool-blocking design against a smaller objective-driven envelope under the same model.

#### C. Make capabilities negotiable, not falsely interchangeable

`RuntimeAdapter` should expose a capability matrix: tool calling, constrained decoding, prompt caching, context size, vision, local determinism class, pricing freshness, batch, streaming, logprobs, and security boundary. A backend without a capability must cause a typed downgrade/refusal, not silent lowest-common-denominator behavior.

#### D. Require four numbers before any gate becomes blocking

Every gate needs:

1. corrupt-evidence kill rate;
2. marginal verified-outcome or escaped-defect improvement;
3. false-positive/over-block rate;
4. latency and cost contribution.

If benefit is within noise or the composed K-4 ceiling breaks, the system should propose demotion or deletion. Human policy confirms the change.

#### E. Replace hard-coded global constants with calibrated policies

Several plans use fixed refusal bands, seed counts, completeness thresholds, rationale-length proxies, and one calibration threshold for different consumers. Share calibration machinery and provenance, but use task/risk-specific loss functions, power analysis, and named-distribution coverage.

#### F. Separate integrity, authenticity, completeness, and correctness

These are distinct:

- hash chain/Merkle tree -> internal integrity;
- digital signature/external checkpoint -> authenticity;
- mediated side effects + completeness monitor -> journal completeness;
- tests/checkers/receipts -> correctness evidence.

Never use one as proof of another.

### 7.3 Move out of the trusted core into signed packs

Move, do not necessarily delete:

- autonomous trading H-17..H-29;
- app-design canvas mode (keep the generic journaled canvas client);
- voice front door/narration;
- personal research machine O;
- personal app world/fleet P;
- broad connector UX and life-data systems;
- domain-specific formal-verification packs;
- provider-specific inference optimizations;
- company billing/commerce.

Each pack should consume the same stable trust-kernel contracts and declare its additional threat model, dependencies, gates, and proof numbers.

### 7.4 Remove or abandon

I would remove these from the active core roadmap:

- Empire integration B-25/C-34 until a real accessible dependency and user exist;
- the claim that a local Merkle proof is a signature;
- universal fail-closed treatment for low-risk quality/efficiency signals;
- any blocking role for the 27.9%-precision semantic-conflict detector until calibrated;
- global fixed refusal-rate bands;
- lexical diversity/length as a load-bearing rationale-quality test;
- one shared calibration threshold across unrelated consumers;
- prompt tagging described as an enforcement boundary;
- PUCT/tree search until receipt-ranked best-of-N wins materially;
- diffusion drafting until it beats ordinary drafting behind identical gates;
- GRPO/self-training from the active build sequence until the simpler SFT rung produces a durable measured lift;
- duplicate routers, claim types, provenance scales, context cores, and legacy fold/compaction paths after N parity tests.

“Deferred” items still impose reading and coordination cost. Items without a live prerequisite or proof-spine contribution should be marked abandoned or moved to an external idea archive, not carried through every summary.

## 8. The recommended product strategy

### Strategy A: build the full ELAI cathedral first

Advantages:

- maximum vertical integration;
- every surface demonstrates the same trust doctrine;
- a differentiated complete product if all pieces compose.

Disadvantages:

- enormous coordination and verification cost;
- upstream vendors absorb features while they are being built;
- fragmentation and stale summaries grow;
- proof of the wedge arrives late;
- users must replace their existing agent to receive value.

### Strategy B: trust-kernel first (recommended)

Ship:

1. `elai wrap` with whole-process confinement;
2. canonical complete receipts;
3. signed RRA v1 and conformance suite;
4. small standalone verifier;
5. taint + pre-write/security gates;
6. independent test/audit runner;
7. PR/CI evidence renderer;
8. SAFEDELTA/QDELTA/gate-tax benchmark suite;
9. adapters for Codex, Claude Code, Hermes, Kilo, and OpenHands.

Advantages:

- users keep the agent they already prefer;
- ELAI competes on its actual wedge;
- same-model comparisons become possible;
- the evidence format can create ecosystem effects;
- every later native feature must prove it beats the wrapper composition.

Disadvantages:

- external agents are harder to mediate completely;
- lowest-level integrations need maintenance;
- less immediate “all-in-one” product spectacle.

The spec already chooses the two-tier version of Strategy B in I-40. The improvement is sequencing and confinement, not inventing a new direction.

## 9. How to prove or falsify “best”

Run a same-model harness tournament. At minimum include:

- ELAI native;
- the same model under Claude Code where possible;
- the same model under Codex where possible;
- Hermes;
- Kilo;
- OpenHands;
- Cline/Aider or mini-SWE-agent as simple controls;
- `elai wrap` around each compatible external harness;
- ELAI gates-off ablation.

Control:

- model build/provider/reasoning settings;
- repository commit and dependency cache;
- task prompt and acceptance criteria;
- compute, time, token, and dollar ceilings;
- tool/network availability;
- number of attempts/seeds;
- hidden tests and git-history stripping;
- blind human review.

Measure:

1. verified task success;
2. escaped defects and dynamically exploitable diffs per 100 tasks;
3. human minutes to approve;
4. percentage of diff needing full attention;
5. dollars and wall-clock per verified task;
6. false blocks and operator interventions;
7. recovery after interruption/context reset;
8. replay/receipt completeness;
9. cross-repo generalization;
10. maintenance burden after a model/provider upgrade.

Pre-register kill rules. Examples:

- if a gate does not improve escaped defects beyond noise and adds material friction, demote/remove it;
- if a specialist role does not change decisions or outcomes, merge it;
- if tree search does not beat best-of-N per dollar, do not build the next rung;
- if a local lane cannot reach a minimum verified-outcome rate, restrict it to mechanical work;
- if `elai wrap` cannot prove complete mediation, narrow its security claim rather than market it as complete.

Until this experiment exists, the right public wording is:

> ELAI is designed to maximize independently verifiable trust per dollar. Its superiority is benchmark-pending.

## 10. Implementation-readiness note (separate from design merit)

This evaluation did **not** perform a live code-level completion audit. Summary/index status stamps are known to drift, and the overview explicitly says it is not a build ledger (`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:1-8`). Therefore:

- the design verdict assumes the full specification works;
- the phrase “best currently” cannot be established from the specification;
- any release or purchase decision requires live code, benchmark, packaging, security, and UX verification;
- competitor features marked preview/experimental must also be rechecked at decision time.

The gap is timing/maturity, not automatically a design gap—but it still matters to the word “currently.”

## 11. Final answers

### Is ELAI the best harness currently?

**No defensible evidence supports that broad claim.** It may have the strongest public *design* for verifier-owned, replayable agent evidence. Other harnesses are better in native collaboration, polished UX, issue/PR distribution, provider ecosystems, SDK simplicity, or current empirical proof.

### Is it future-proof and moving where the major companies are aiming?

**It is moving in the correct broad direction and, on evidence/provenance, arguably further. It is not future-proof.** Future resilience depends on keeping a small stable trust kernel, making providers/capabilities replaceable, measuring every scaffold, and deleting mechanisms vendors absorb or benchmarks do not justify.

### What is the single biggest improvement?

**Make the external-agent trust kernel the first product and force the full roadmap to earn its way back in through measured proof.**

### What is the single biggest removal?

**Remove breadth from the trusted core.** Trading, voice, app-design canvas, personal research/world, connector UX, and self-training should be optional packs or research branches until the safety/evidence wedge is proven.

### What should never be removed?

The core idea: **the model proposes; an isolated, deterministic, independently verifiable harness owns side effects, evidence, and completion.**

## 12. Primary external sources

All external capability claims above were refreshed from first-party sources on 2026-07-10.

- [OpenAI — Harness engineering](https://openai.com/index/harness-engineering/)
- [OpenAI — Symphony announcement](https://openai.com/index/open-source-codex-orchestration-symphony/)
- [OpenAI — Symphony Draft v1 specification](https://github.com/openai/symphony/blob/main/SPEC.md)
- [Codex — Subagents](https://developers.openai.com/codex/agent-configuration/subagents)
- [Codex — Customization](https://developers.openai.com/codex/concepts/customization)
- [Codex — Memories](https://developers.openai.com/codex/customization/memories)
- [Codex — Hooks](https://developers.openai.com/codex/hooks)
- [Codex — Security and approvals](https://developers.openai.com/codex/agent-approvals-security)
- [Codex — Record & Replay](https://developers.openai.com/codex/extend/record-and-replay)
- [Anthropic — Claude Code agent teams](https://code.claude.com/docs/en/agent-teams)
- [Anthropic — Claude Code subagents](https://code.claude.com/docs/en/sub-agents)
- [Anthropic — Harness design for long-running application development](https://www.anthropic.com/engineering/harness-design-long-running-apps)
- [Anthropic — Infrastructure noise in agentic coding evals](https://www.anthropic.com/engineering/infrastructure-noise)
- [Hermes Agent repository](https://github.com/NousResearch/hermes-agent)
- [Hermes security model](https://github.com/NousResearch/hermes-agent/blob/main/SECURITY.md)
- [Hermes development guide / narrow waist](https://github.com/NousResearch/hermes-agent/blob/main/AGENTS.md)
- [Kilo providers](https://kilo.ai/docs/ai-providers)
- [Kilo custom subagents](https://kilo.ai/docs/customize/custom-subagents)
- [Kilo Agent Manager](https://kilo.ai/docs/automate/agent-manager)
- [Kilo Orchestrator deprecation](https://kilo.ai/docs/code-with-ai/agents/orchestrator-mode)
- [OpenHands SDK](https://docs.openhands.dev/sdk/index)
- [OpenHands architecture](https://docs.openhands.dev/sdk/arch/overview)
- [OpenHands runtime](https://docs.openhands.dev/openhands/usage/architecture/runtime)
- [GitHub Copilot custom agents](https://docs.github.com/en/copilot/reference/custom-agents-configuration)
- [GitHub Copilot hooks](https://docs.github.com/en/copilot/reference/hooks-reference)
- [GitHub Copilot cloud agent](https://docs.github.com/en/copilot/concepts/agents/cloud-agent/about-cloud-agent)
- [GitHub Copilot memory](https://docs.github.com/en/copilot/concepts/agents/copilot-memory)
- [GitHub Copilot CLI Fleet](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/fleet)
- [GitHub Copilot cloud-agent firewall](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/customize-the-agent-firewall)
- [Cursor background agents](https://docs.cursor.com/background-agent)
- [Cursor 2.5](https://cursor.com/changelog/2-5)
- [Cursor multitask/worktree release](https://cursor.com/changelog/04-24-26)
- [Devin advanced capabilities](https://docs.devin.ai/work-with-devin/advanced-capabilities)
- [Kiro subagents](https://kiro.dev/docs/chat/subagents/)
- [Kiro MCP sandbox boundary](https://kiro.dev/docs/autonomous-agent/sandbox/mcp/)
- [Gemini CLI reference](https://geminicli.com/docs/cli/cli-reference/)
- [Gemini CLI sandbox](https://geminicli.com/docs/cli/sandbox/)
- [Cline checkpoints](https://docs.cline.bot/core-workflows/checkpoints)
- [Aider documentation](https://aider.chat/docs/)
- [mini-SWE-agent repository](https://github.com/SWE-agent/mini-swe-agent)
- [SWE-agent development notice](https://github.com/SWE-agent/SWE-agent)
- [Roo Code archive/shutdown notice](https://github.com/RooCodeInc/Roo-Code)

## 13. Local evidence anchors

Most important local anchors used in the decision:

- thesis and topological map: `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:12-27`, `:467-489`;
- lifecycle and journal contract: `docs/specs/guide/steps/phase-index-MASTER.txt:391-439`, `:474-496`;
- test-integrity receipt: `docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md:75-91`;
- RRA/SAFEDELTA/provenance: `docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md:38-57`, `:111-159`;
- blast-radius unresolved expansion: `docs/thefinalglance/G-plan/G-3_blast_radius_gates.md:44-49`, `:292-294`;
- evidence-ranked search and external wrapper: `docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md:91-108`, `:755-773`;
- proof/distribution/gate tax: `docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md:101-112`, `:156-215`, `:588-598`;
- anti-fragmentation: `docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md:10-18`, `:46-65`, `:228-248`;
- scoped determinism and gate self-tests: `docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md:44-73`;
- previous evaluation and absorbed improvements: `workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md:188-248`.

## 14. Fresh-research caveat

The research floor was exceeded (25+ searches, 35+ primary-source opens). The meaningful caveat is market velocity: preview/experimental labels, product availability, model support, and provider pricing change quickly. Re-check those facts before external publication. The architectural conclusions—narrow waist, risk-proportional orchestration, isolated side effects, independent evidence, and same-model ablation—are substantially less time-sensitive.

## Appendix A — External harness market dossier (verbatim)

The following dossier is reproduced verbatim from the dedicated fresh-research pass. It is retained separately so the external evidence and its caveats are not diluted by the synthesis above. **The edited synthesis and normative source list above are authoritative wherever this raw dossier differs. Its “Best” labels are analyst judgments among the reviewed public mechanisms, not benchmark-established winners.**

### External harness market dossier — 10 July 2026

I completed well over the requested floor: 25+ searches and 35+ primary-source opens. I used vendor documentation, official repositories, and first-party engineering posts. No secondary comparison articles were used for core conclusions.

#### Bottom line

There is no defensible, evidence-based “best harness” today. No independent benchmark compares all major harnesses while holding model, repository, prompt, compute, and infrastructure constant. Anthropic found infrastructure alone can move agentic benchmark results by up to six percentage points, while [SWE-bench now holds the mini-SWE-agent harness fixed](https://www.swebench.com/) when comparing models. Vendor demos therefore cannot establish an overall winner.

The market has several different leaders:

- Best native collaborating-agent UX: [Claude Code agent teams](https://code.claude.com/docs/en/agent-teams), though still experimental.
- Best issue-driven continuous orchestration design: [OpenAI Symphony](https://openai.com/index/open-source-codex-orchestration-symphony/), though its reference implementation is an engineering preview and its [spec is Draft v1](https://github.com/openai/symphony/blob/main/SPEC.md).
- Best enterprise repository control plane: GitHub Copilot, especially its explicit [Fleet dependency graph](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/fleet) and cloud-agent session management.
- Best provider-independent personal learning agent: [Hermes Agent](https://github.com/NousResearch/hermes-agent).
- Best open composable agent SDK foundation: [OpenHands SDK](https://docs.openhands.dev/sdk/index).
- Best provider-neutral multi-session IDE workflow: [Kilo Agent Manager](https://kilo.ai/docs/automate/agent-manager).
- Best formal commercial spec-first workflow: [Kiro](https://aws.amazon.com/documentation-overview/kiro/).
- Best polished local/cloud fleet IDE: Cursor.
- Best transparent minimal research harness: [mini-SWE-agent](https://github.com/SWE-agent/mini-swe-agent).

#### Mechanism-by-mechanism comparison

| Harness | Orchestration | Memory/context | Models | Planning and verification | Security, trace, human control | Important limitation |
|---|---|---|---|---|---|---|
| Claude Code | Independent peer agents, shared tasks, dependencies, messages, file locking | CLAUDE.md plus local auto-memory and per-agent memory | Claude only, through several clouds | Plan approval and deterministic hooks; tests are configurable, not mandatory | Strong permissions, sandbox, checkpoints, OTel, teammate messaging and interruption | [Agent teams are experimental](https://code.claude.com/docs/en/agent-teams); remote control is preview |
| Codex + Symphony | Native subagents; Agents SDK pipelines; Symphony maps every tracker issue to an isolated persistent worker | AGENTS.md plus opt-in local evidence-backed memory | OpenAI models | Strong hooks, review and test surfaces; Symphony has explicit claims, retries, reconciliation and stall handling | OS sandbox, network controls, approvals, cloud isolation, OTel/traces | Symphony is a [trusted-environment engineering preview](https://github.com/openai/symphony); several Codex hook handler types are parsed but not yet executed |
| GitHub Copilot | Fleet stores tasks and dependencies and dispatches ready work in parallel; cloud sessions centrally managed | Repository instructions, skills and persistent sessions | CLI supports BYOK; cloud agent remains more curated | PR-native tests, CI, review and artifacts | Excellent RBAC, audit and live logs; firewall and approval controls | Fleet SDK is experimental; cloud agent is repository-centric; firewall does not cover every MCP/setup path |
| Cursor | Async and nested subagents, worktrees, cloud agents, multi-repo and mobile steering | Rules, AGENTS.md and history retrieval | Broad curated model menu | Plan/todo modes and cloud-generated test/demo artifacts | Hooks, network/file allowlists, checkpoints and human handoff | Closed platform; model choice is not true runtime/provider independence; cloud internet access carries stated exfiltration risk |
| Hermes | Parallel isolated subagents and batch workers; no documented peer task graph | First-class persistent memory, agent-authored skills, session search and skill curator | Very broad provider/local-model support | Limited formal spec machinery; tests and review depend on configured workflow | Approval modes, containers, remote runtimes, searchable trajectories | Default local process is not a security boundary; MCP/plugins/hooks/in-process code escape terminal-backend isolation |
| Kilo | Parallel custom subagents plus 1–20 isolated Agent Manager sessions and multi-version runs | AGENTS.md and session history; former Memory Bank deprecated | 30+ providers, BYOK, local and OpenAI-compatible | Plan/code/debug agents and review flow, but no mandatory verifier | Worktree isolation, permission dock, stop/cancel, live diffs and PR status | Dedicated Orchestrator was deprecated; agents mostly return summaries rather than collaborate through a shared graph |
| OpenHands | Multi-agent-capable SDK, but mainly a substrate for building orchestration | Serializable conversations and compression | Strong provider independence through LiteLLM-style adapters | Planning and security-analyzer components; verification is application-defined | Excellent OTel, local/Docker/cloud/Kubernetes runtimes and approval policies | Less turnkey than Claude/Codex/Kilo; no default peer-team or verifier-owned completion protocol |
| Cline | Experimental parallel read-only research subagents | Rules, skills, sessions | Strong provider independence | Clear Plan/Act split and checkpoints | Hooks, OTel events, command policies and human approvals | Subagents cannot edit, browse, use MCP or nest |
| Kiro | Parallel subagents plus local/web automation | Steering, repository specs and knowledge | Curated models; no general BYOK | Strongest spec-first commercial workflow | Enterprise model/MCP controls and sandboxed web agent | Subagents cannot see Specs and hooks do not run inside them; MCP subprocesses can sit outside the agent sandbox |
| Gemini CLI | Custom subagents, extensions and experimental worktrees/ACP | GEMINI.md hierarchy and persistent facts | Primarily Gemini; arbitrary provider support is not shipped | Plan mode exists but official configuration material still describes parts as under development | Strong sandbox choices, approvals, checkpoints and detailed telemetry | Model lock and less mature peer/shared-task orchestration |
| Devin | Multiple managed sessions, knowledge, playbooks and schedules | Strong organization knowledge and reusable playbooks | Closed managed platform | Managed cloud validation and human review | Strong session timeline, RBAC and audit logs | Parallel sessions are not a documented live peer fleet; no local/provider-neutral runtime |
| mini-SWE-agent | Deliberately single-agent and linear | Serialized conversation trajectory | Broad LiteLLM-compatible models | Reproducible coding benchmark harness | Simple local/container execution and exact trajectory inspection | Not an enterprise harness: no MCP, durable learning, multi-agent system or governance |
| Roo Code | Historical Boomerang parent/subtask model | Rules and checkpoints | Broad providers | Architect/code modes | Human approvals and shadow-Git checkpoints | No longer a current contender: [the repository was archived and the extension shut down on 15 May 2026](https://github.com/RooCodeInc/Roo-Code) |

#### Important 2026 shifts

- Claude now has the market’s clearest native peer-agent system: independent sessions, shared tasks, dependency blocking, peer messaging and teammate steering. It is nevertheless explicitly experimental.
- OpenAI moved beyond chat-session orchestration with Symphony. Its issue tracker becomes the durable control plane; each issue gets a workspace, worker, retries, continuation turns, stall detection and reconciliation. OpenAI reports up to a 500% increase in landed PRs on some internal teams, but that is a vendor result, not an independent benchmark.
- Kilo replaced its dedicated Orchestrator with general agents spawning isolated subagents. Separately, its [Agent Manager](https://kilo.ai/docs/automate/agent-manager) runs parallel worktree sessions, live diffs, PR/CI state, human approvals, cancellation and multi-model variants.
- Hermes explicitly follows a [“narrow waist, capability at the edges” design](https://github.com/NousResearch/hermes-agent/blob/main/AGENTS.md): keep the permanent model tool schema small and add capabilities through CLI-plus-skill, gated services, plugins or MCP. This is a strong future-proofing pattern.
- Roo Code is not merely falling behind; it has been discontinued and archived.
- The original SWE-agent is maintenance-only and has been superseded by mini-SWE-agent. That team deliberately reduced scaffold complexity as models improved.

#### Where the whole market is heading

The convergence is unusually clear:

1. Isolated agents in worktrees, containers or VMs.
2. Durable work items rather than disposable chats.
3. Repository-native guidance through AGENTS.md, skills and specs.
4. MCP and plugins as external capability planes.
5. Hooks and policy engines for deterministic enforcement.
6. Local/cloud/hybrid execution with remote or mobile steering.
7. OTel-compatible event streams, traces and auditable artifacts.
8. Human plan approval, interruption, queueing, rewind and review.
9. Explicit tests, CI, screenshots, videos or other proof-of-work.
10. Persistent memory—but increasingly with provenance, limits and user inspection.

This strongly validates ELAI’s broad direction if its specification really implements durable tasks, isolated workers, explicit state, human gates, memory provenance, verification, telemetry and replaceable execution backends.

#### Likely ELAI differentiators

Conditional on the local specification matching its claims, ELAI appears unusually strong in areas most competitors leave optional:

- An independent auditor rather than the implementing agent declaring itself finished.
- Claim-to-test attribution.
- Wiring-contract verification across subsystem boundaries.
- Impact mapping before edits.
- Explicit FSM transitions and recovery states.
- A dedicated memory writer after verification.
- Separation between context compilation, implementation, audit and durable learning.

Most commercial products provide hooks that *can* run tests. Very few make independent verification and provenance structurally necessary. That is probably ELAI’s strongest defensible advantage—not raw agent count.

#### What I would improve or remove

- Add adaptive operating tiers. Use a fast single-agent lane for trivial work, a planned implement-and-review lane for normal work, and the full multi-agent FSM only for high-risk or cross-system work.
- Keep the core a narrow waist. Task, workspace, event, artifact, policy and verification contracts should remain stable while models, MCP servers, memory backends and executors remain replaceable.
- Make the durable task/lease/retry/reconciliation layer as explicit as Symphony’s control-plane spec.
- Treat worktrees or containers as the default execution boundary. Explicitly document that MCP servers, plugins and in-process extensions may bypass a shell sandbox.
- Standardize an evidence bundle for every completion: diff, tests, claim coverage, logs, artifacts, unresolved uncertainty and verifier identity.
- Support OTel plus deterministic replay of state transitions—not merely chat transcript replay.
- Permit self-improvement only through measured promotion: proposed memory or skill → sandbox evaluation → regression comparison → human/policy approval → versioned activation → rollback.
- Collapse roles whose outputs are repetitive or do not change decisions. Repeated prompt reformatting, full-context recompilation and multiple ceremonial sign-offs can become a cost and latency tax.
- Avoid shared-context “agent theatre.” Spawn another agent only when it has an independent context, evidence source, permission boundary or verification responsibility.
- Preserve provider and runtime independence. A future-proof harness should survive model leadership changing every few months.

The strongest warning from the market is that sophistication can become drag. Kilo removed its dedicated orchestrator, and the SWE-agent team moved toward a much smaller harness. ELAI should retain its rigorous mechanisms while making them proportional to task risk.

#### Primary source set

- [Claude agent teams](https://code.claude.com/docs/en/agent-teams)
- [Claude subagents](https://code.claude.com/docs/en/sub-agents)
- [Claude memory](https://code.claude.com/docs/en/memory)
- [Claude monitoring](https://code.claude.com/docs/en/monitoring-usage)
- [Codex security](https://developers.openai.com/codex/agent-approvals-security)
- [Codex MCP](https://developers.openai.com/codex/extend/mcp)
- [Codex hooks](https://developers.openai.com/codex/hooks)
- [Symphony announcement](https://openai.com/index/open-source-codex-orchestration-symphony/)
- [Symphony specification](https://github.com/openai/symphony/blob/main/SPEC.md)
- [GitHub Copilot Fleet](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/fleet)
- [Hermes repository](https://github.com/NousResearch/hermes-agent)
- [Hermes security model](https://github.com/NousResearch/hermes-agent/blob/main/SECURITY.md)
- [Kilo Agent Manager](https://kilo.ai/docs/automate/agent-manager)
- [Kilo custom subagents](https://kilo.ai/docs/customize/custom-subagents)
- [OpenHands architecture](https://docs.openhands.dev/sdk/arch/overview)
- [OpenHands observability](https://docs.openhands.dev/sdk/guides/observability)
- [Cline subagents](https://docs.cline.bot/features/subagents)
- [Kiro subagents](https://kiro.dev/docs/chat/subagents/)
- [mini-SWE-agent](https://github.com/SWE-agent/mini-swe-agent)

The evidence floor is comfortably met. The only meaningful staleness caveat is that this market is changing weekly; “preview,” “experimental,” and model/provider availability should be rechecked before publication.

### Editorial errata for the verbatim dossier

- The six-percentage-point infrastructure result is supported by [Anthropic's infrastructure-noise report](https://www.anthropic.com/engineering/infrastructure-noise).
- The official SWE-agent wording is that mini-SWE-agent has superseded SWE-agent and receives most development effort; “maintenance-only” in the raw dossier is stronger than the source.
- Read “only meaningful staleness caveat” as **main time-sensitive caveat**. Additional limitations are first-party-source bias, inaccessible closed/internal harnesses, and the absence of a controlled cross-harness benchmark.
- Canonical, current product URLs and boundary qualifications in Sections 3 and 12 supersede raw-dossier shorthand.

## Addendum — Native ELAI is the product; the wrapper is the on-ramp

Date: 2026-07-10  
Status: **corrective product-strategy addendum**. This addendum supersedes the wrapper-first recommendation in Sections 7.3, 8, and 11 where they conflict. It does not change the report's benchmark caution, trust-contract findings, canonicalization requirements, risk-tier recommendation, or argument for modular vertical packs.

### A. The benchmark verdict stands; the wrapper-first conclusion does not

The absence of a controlled cross-harness benchmark still means ELAI cannot honestly claim to be the best overall harness today. ELAI can aim to win without claiming that it has already won.

That evidence gap does **not** imply that ELAI should stop competing as a native harness and redefine itself primarily as middleware around Claude Code, Codex, Hermes, Kilo, or OpenHands. Product ambition and present proof are separate questions.

### B. I-40 was interpreted too broadly

The detailed I-40 plan defines two tiers but does not make the wrapper the primary product:

- “wrapper = wedge + standard track”;
- “the deep ELAI orchestrator with full wiring stays the premium destination”;
- “The wrapper is the on-ramp, not the product.”

See `docs/thefinalglance/I-plan/I-40_trust_layer_wrapper_rra_spec.md:34-35`.

Therefore, the earlier statement that the specification already chooses wrapper-first Strategy B is too strong. The specification chooses an exportable wrapper **alongside** a first-party orchestrator and explicitly preserves native ELAI as the destination.

The overview reinforces that identity:

- Phases 0–4 are “the actual ELAI system” (`docs/thefinalglance/ELAI-SPEC-OVERVIEW.md:31-34`);
- that system owns retrieval, GRC reasoning, lifecycle control, deterministic orchestration, context allocation, isolated execution, autonomous advancement, validation, runtime negotiation, and the native control surface (`:36-131`);
- the full-corpus invariant remains a native deterministic harness that sizes, gates, journals, and decides completion from off-agent evidence (`:467-489`).

### C. A compact trust kernel is not a wrapper-first product

The previous recommendation collapsed three independent architectural decisions:

1. **Small trusted computing base:** keep the supervisor, journal, policy arbitration, receipts, verifier, confinement, and completion contract compact and independently testable.
2. **Exportable trust protocol:** let foreign agents emit or be evaluated through RRA-compatible evidence and provide `elai wrap` as a compatibility surface.
3. **Product identity:** decide whether ELAI itself remains a native reasoning and execution harness.

The first two do not require surrendering the third.

Hermes' own narrow-waist rule makes this distinction explicit: “Smallest footprint” governs how capability is wired into the core, **not** whether the product may grow; Hermes is “expansive at the edges and conservative at the waist” ([official `AGENTS.md`](https://github.com/NousResearch/hermes-agent/blob/main/AGENTS.md)). The appropriate lesson for ELAI is a small permanent kernel with rich native capabilities at typed, measurable edges—not a recorder-only product.

### D. Why wrapper-only ELAI would discard part of the advantage

A proxy around an external agent can mediate observable tool traffic, enforce some side-effect policy, and capture receipts. Unless the foreign agent is subordinated as an ELAI runtime backend under whole-process confinement, the proxy does not own that agent's internal:

- reasoning and grounding protocol;
- task decomposition and search topology;
- context allocation and compaction;
- branch selection and evidence-ranked search;
- recovery, handoff, and anti-spiral behavior;
- verifier/worker separation.

The earlier P0-5 finding already shows the enforcement side of this limit: MCP or stdio interception is bypassable when an external process retains direct filesystem, process, network, or credential access. Native ELAI's process model is stronger because agents hold no independent tools, network access, or credentials and the orchestrator remains authoritative (`ELAI-SPEC-OVERVIEW.md:101-105`).

The wrapper can export part of ELAI's trust spine. It cannot, by itself, express the full native capability architecture.

### E. “Simpler orchestration” does not mean “no native orchestrator”

The current market evidence supports flexibility **inside** deterministic boundaries:

- OpenAI's Symphony announcement rejects treating workers as narrow rigid nodes and gives them objectives, tools, and room to choose their path. Its specification nevertheless retains one authoritative orchestrator for dispatch, retries, reconciliation, workspace isolation, and explicit state transitions ([announcement](https://openai.com/index/open-source-codex-orchestration-symphony/), [specification](https://github.com/openai/symphony/blob/main/SPEC.md)).
- Kilo removed a dedicated user-selected Orchestrator mode because ordinary agents gained native delegation; orchestration moved into the primary agent rather than disappearing ([Kilo documentation](https://kilo.ai/docs/code-with-ai/agents/orchestrator-mode)).
- Anthropic's long-running application harness is an integrated planner/generator/evaluator architecture with structured handoffs and independent evaluation, not merely an evidence wrapper ([Anthropic engineering report](https://www.anthropic.com/engineering/harness-design-long-running-apps)).

The resulting correction is narrower than deleting ELAI's FSM: retain strict state and enforcement around commitments, credentials, writes, publication, evidence admission, and completion; allow objective-driven flexibility in planning, exploration, decomposition, testing order, and agent topology.

### F. Corrected product strategy

The earlier report presented a false binary between building the entire ELAI “cathedral” first and making the wrapper the first/primary product. The missing strategy is:

1. **Compact trust kernel** — stable receipts, journal, confinement, policy, provenance, independent verification, rollback, and evidence schemas.
2. **Native ELAI flagship** — the competitive first-party coding-and-reasoning harness using GRC, retrieval, orchestration, recovery, runtime control, and the complete trust kernel.
3. **Wrapper and adapters** — compatibility, adoption, conformance testing, and evidence interchange for external agents.
4. **Signed capability packs** — trading, research, canvas, voice, personal-world, and provider-specific extensions outside the permanent trusted core.

In short: **kernel-first engineering, native-first product identity, wrapper-assisted distribution**.

This preserves the strongest part of the original recommendation—canonicalize and prove the trust spine before expanding the active build sequence—without reducing ELAI to a building inspector or flight recorder. ELAI's closer role is a native flight-control system with an open recorder and conformance standard that other systems can also adopt.

### G. The experiment that decides the strategy

Do not settle native-versus-wrapper by analogy. Add a three-arm comparison to the proposed tournament:

1. the competitor harness unmodified;
2. the same competitor under `elai wrap`;
3. native ELAI using the same underlying model.

Hold repository, task, model, model settings, compute budget, tool access, and evaluation constant. Measure:

- verified task completion;
- escaped defect and security-failure rate;
- evidence-corruption kill rate;
- reviewer minutes;
- recovery after interruption or worker failure;
- wall time, token/compute cost, and gate tax;
- false-positive and over-block rate.

This separates two causal questions:

- how much value comes from the portable trust kernel alone;
- how much additional value comes from ELAI's native reasoning and orchestration.

If wrapped competitors match native ELAI across the declared target domains at lower cost, wrapper-first becomes evidence-backed. If native ELAI produces materially better verified outcomes, recovery, or reviewer economics, the native orchestrator earns its position as the flagship. Until that test exists, declaring the wrapper ELAI's best future is premature.

### H. Corrected final answer

ELAI should be a **native, model- and runtime-agnostic autonomous harness with an exportable trust kernel**, not a trust wrapper with an optional native harness. The wrapper is the on-ramp and compatibility layer; the deep ELAI orchestrator remains the product and premium destination.
