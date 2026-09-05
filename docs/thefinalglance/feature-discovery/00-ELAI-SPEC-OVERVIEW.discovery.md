# ELAI feature discovery — systems overview

## Record purpose and authority

- Primary source read completely on 2026-08-11: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md (592 lines; SHA-256 ad1cb330bcec68caa7658c188f057032ca6a6e830d307a41767fed8aa08160f2).
- Scope: every capability, sub-capability, workflow, component, state, mode, artifact, integration, constraint, security/privacy behavior, lifecycle behavior, operational behavior, and explicitly named future or conditional capability stated in that overview.
- This is evidence capture from the overview, not a replacement for the MASTER phase index, B–Q summary indexes, or their original plan files.
- The source is explicitly spec-only: it describes capabilities as if built. It does not establish whether anything is wired, shipped, dark, complete, or tested. It intentionally omits schema/SSB counts and effort estimates. Wiring state must be determined from the feature map and wiring checkers.
- Design sources of truth named by the overview are the MASTER phase index (Phases 0–4M) and B–Q summary indexes. The overview is a topological design map, not a build ledger.
- Discovery IDs below are local trace labels only. Exact plan/phase identifiers and named terminology are retained in each entry.

## 1. Product thesis and system-wide invariants

### OVR-THESIS-001 — Product identity

ELAI is specified as a local-first, deterministic coding-and-reasoning agent. The product is the HARNESS around the model, not the model itself. The model is a replaceable cog that generates and selects. The deterministic harness sizes work, owns state, runs gates, and decides done using artifacts the model did not author.

### OVR-THESIS-002 — Product wedge

All layers compose toward trust-evidenced autonomous execution: code and reasoning that can be audited and replayed on any selected model backend.

### OVR-INV-001 — Off-agent evidence

- A gate measures an artifact the generator did not write.
- Explicit examples are an event row, a coverage hit, a hash-bound test outcome, and a span.
- The generator's own stdout is explicitly not sufficient evidence.

### OVR-INV-002 — Fail-closed verdicts

- Missing evidence is FAIL, never SKIP.
- The gate judging a run is frozen and read-only to that run.

### OVR-INV-003 — Determinism and reproducibility

- All time and randomness originate in controlled providers.
- The journal is append-only.
- A reproducible-run artifact records the run.
- Together these are intended to make an entire session byte-recomputable.

### OVR-INV-004 — Authority split

- Models: generate and select.
- Harness: size, gate, journal, own state, and decide completion.
- This split recurs at inference, tool execution, self-modification, evaluation, replay, and user-facing control surfaces.

## 2. Core engine — Phases 0–4M

### Phase 0 — Walking skeleton + determinism substrate

#### OVR-P0-001 — Runtime foundation

- Fork a Rust coding agent.
- Strip known-bad code.
- Restructure it into a nine-crate workspace: core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, and cli.

#### OVR-P0-002 — Minimal executable agent

- One model.
- One FSM loop.
- One tool.
- Msgpack session storage.
- Path-scoped visibility filter.

#### OVR-P0-003 — Entropy quarantine

- All time flows through an injected ClockProvider trait.
- All randomness flows through an injected EntropySource trait.
- A seeded run must be byte-identical across executions.
- This substrate is the dependency for every later reproducible/replayable claim.

### Phase 1 — Knowledge / retrieval engine (knowledge/)

#### OVR-P1-1A — DKG

- Tree-sitter parses source into a dependency knowledge graph in SQLite.
- Graph entities include symbols.
- Typed graph edges include imports, contains, and implements.
- Incremental updates are content-hash driven.

#### OVR-P1-1B — BM25

- Per-project Tantivy full-text index.
- Sub-10ms query target.
- Precise per-file deletion.

#### OVR-P1-1C — Vector search

- Dual-model ONNX embeddings: one code model and one text model.
- Per-model circuit breaker.
- Content-hash vector cache.
- Graceful degradation to BM25-only when a model fails.

#### OVR-P1-1D — PageRank repository map

- Rank symbols by importance.
- Fit the repository map to a token budget for context loading.
- Map is symbol-level, density-guarded, and task-personalized.

#### OVR-P1-1E — CatRAG + A-RAG

- Assemble the full query pipeline in this stated order: BM25 -> vector -> PageRank -> graph anchoring -> agentic snippet retrieval.
- Goal stated by the source: large token savings compared with naive full-chunk retrieval.

#### OVR-P1-1F/1G/1H — Retrieval evaluations and reranking

- 1F: speculative-decoding evaluation; explicitly dropped for Phase 1.
- 1G: cross-encoder reranker.
- 1H: conditional SPLADE keyword-expansion evaluation.

#### OVR-P1-1I — Retrieval resilience

- Per-model circuit breakers.
- Search-mode degradation cascade.
- Query validation.
- Startup readiness canary.
- Staged query tracing.

#### OVR-P1-3K — RAG hardening

- AST-aware adaptive chunking.
- Template multi-query expansion.
- HyDE.
- Tier routing with FAST, STANDARD, and DEEP tiers.

### Phase 2 — Reasoning engine (reasoning/)

#### OVR-P2-001 — Grounded Reasoning Chain (GRC)

- Budgeted reasoning state machine with the exact sequence Observe -> Hypothesize -> Plan -> Act -> Verify.
- Every reasoning step must carry a typed grounding reference.
- Grounding is classified into confidence levels.

#### OVR-P2-002 — Verification methods

Six binary methods resolve ungrounded claims:

1. file
2. source
3. test
4. web
5. api
6. api-registry

#### OVR-P2-003 — Grounding transition gate

- A plan-grounding sweep checks all claims.
- Any ungrounded claim blocks the phase transition.

#### OVR-P2-004 — Per-agent GRC configuration

Each role has its own:

- reasoning budget;
- step sequence;
- allowed verification methods;
- anti-rubber-stamp rules.

#### OVR-P2-005 — Counterfactual safety and feasibility

- Counterfactual safety questions (CSQ) apply to functions touching files, network, money, or auth.
- An inline feasibility check runs during execution.

### Phase 3 — Orchestration + resilience (orchestrator/, rules/)

#### OVR-P3-3A — Full FSM

- Nine interactive states: Idle, Planning, Feasibility, TestingFirst, Executing, Auditing, Memorizing, Complete, and Configuring.
- Tool and rule gating is phase-aware.
- Every transition has evidence gates.
- Each gate is driven by a human-in-the-loop decision: approve, reject, modify, or redirect.
- Human involvement scales by per-transition risk level.

#### OVR-P3-3B — Shared State Board (SSB)

- Single-writer, append-only SQLite/WAL journal.
- Stores typed inter-agent events.
- Acts as the observability and reproducibility backbone.
- Every later journaled row is intended to land here.

#### OVR-P3-3C — Agent system

- Agent spawn.
- Lifecycle management.
- Checkpointing.
- Crash recovery.
- Erlang-style supervision.
- Anti-spiral protection via a content-hash loop detector.

#### OVR-P3-3D — BSP supersteps + channels

- Deterministic macro-orchestration.
- Barrier-synchronized parallel agents.
- Typed channels.
- Results applied in journal-sequence order.

#### OVR-P3-3E — Context budget allocator

- Fixed eight-slot budget.
- Never exceeds the model context window.
- Slot floors.
- Deterministic cache breakpoints.
- Adaptive reduction.

#### OVR-P3-3F — Rules engine

- Deterministic rule enforcement.
- Rules are stored as data.
- Predicate forms: AST, grep, schema, metric, and composite.
- Lifecycle: proposed -> shadow -> active.
- Promotion gates govern lifecycle transitions.
- Self-discovery finds recurring violations.
- Security and finance rules are immutable.

#### OVR-P3-3G — Hierarchical plan management

- Token-bounded plan skeleton.
- Per-phase elaboration.
- Contract extraction between phases.
- Coherence checks.

#### OVR-P3-3H — Session handoff

- Session snapshots form a DAG.
- Structurally lossless trimming.
- Tool results may be stubbed in the snapshot but remain recoverable from the journal.
- Branch and merge operations.

#### OVR-P3-3I — Deterministic simulation testing (DST)

- Prove FSM, SSB, and BSP correctness.
- Compare behavior against an infallible reference model.

#### OVR-P3-3J — vLLM + advanced providers

- Production inference backend.
- Enables layer-level optimizations unavailable through an opaque API:
  - DoLa contrastive decoding;
  - TurboQuant KV compression;
  - ActAdd steering vectors.
- Composed local inference sidecar using the Anthropic protocol.
- CLI local mode and hybrid mode.

### Phase 4 — Production hardening + conditional extensions

#### OVR-P4-4A — Agent process isolation

- Agents run as isolated operating-system processes.
- Orchestrator is authoritative for GRC budget, inference, tools, and grounding.
- Those functions are proxied over a bounded 15-message IPC protocol.
- Agents hold no network access, LLM client, or tools.

#### OVR-P4-4B — Tools

- Git operations.
- Five-tier validation.
- DAIRA trace markers.
- Three-tier edit matching.
- ShortCoder AST-equivalent code compression.
- Deterministic failure taxonomy.
- Async incremental compilation.

#### OVR-P4-4C — Plugins

- Hooks intercept IPC invisibly to agents.
- Tiered static analysis:
  - T1 rules;
  - T2 tree-sitter;
  - T3 pattern matching.
- No LLM runs in the static-analysis hot path.

#### OVR-P4-4D/4E — Pipeline factory

- A Research Agent and Designer derive a typed PipelineSpec from a task.
- Research uses tree-structured grounded research named FlashResearch.
- A deterministic builder creates the executable pipeline.
- Builder performs DAG repair and phase sizing.
- Recursive decomposition turns an idea into Concepts -> Phases -> Steps -> BabySteps.
- Every decomposition level is grounded.
- Every BabyStep is an eight-block specification.
- TreeNavigator is the primary execution artifact.

#### OVR-P4-4F — Feasibility probe

- Probe system resources.
- Run novel-step stubs in an isolated sandbox.
- Support branch-and-explore.

#### OVR-P4-4G — Autonomous mode

- AutonomousGate wraps the interactive gate.
- It advances on deterministic evidence.
- It delegates to a human on high risk, missing evidence, or escalation triggers.
- Fallbacks are task splitting and model substitution.

#### OVR-P4-4H — Self-modification (DGM-derived)

- Improve configs, rules, and data tables based on measured pipeline outcomes.
- DGM-style variant archive and parent selection.
- Modification-solidification index.
- Closed set of mutable configuration keys.
- Four safety-enforcement layers.
- The modifying agent never evaluates its own modification.

#### OVR-P4-4I — Media validator

- Validates non-code artifacts: images, 3D, audio, and UI.
- Deterministic Tier-1 checks have veto power.
- Advisory Tier-2 VLM.
- Render-capture-evaluate loop.

#### OVR-P4-4J — Empire integration

- Conditional integration connecting ELAI as a brain to an external ecosystem.

#### OVR-P4-4K — Dashboard

- Native Tauri desktop control surface.
- Polls the SSB viewer.
- Explicitly not a chatbot.
- Every input passes through FSM classification.

#### OVR-P4-4M — Multi-runtime backend abstraction

- One workflow specification drives interchangeable runtime backends.
- Named backends: local sidecar, cloud API, and vendored coding-agent CLIs.
- Single RuntimeAdapter trait.
- Per-role backend assignment.
- Backend fallback chains.

## 3. Capability layers — letter plans B–Q

### B — MVP completion + dashboard + knowledge extensions

#### OVR-B-001 — End-to-end activation and telemetry

- Path unification.
- FSM/SSB producer wiring.
- Every engine emits real telemetry per query.

#### OVR-B-002 — V2 dashboard and chat

- Tauri + Svelte desktop dashboard.
- Three-dimensional orbit visualizer that is the activity feed.
- Live token-streaming chat.
- Intent classification for chat.

#### OVR-B-003 — Harness hardening

- Phase-0 debt burndown.
- Anti-stale enforcement.
- Sandbox activation.
- Hook/tool unification.

#### OVR-B-004 — Cloud mode

- Per-task local, cloud, or hybrid routing.
- Spend ceilings.

#### OVR-B-005 — Focus Tunnel

- Deterministic emission modes: FIM, DIFF, and STRUCTURED.
- Streaming validation.
- Automated dependency resolution.
- Explicit goal: token efficiency.

#### OVR-B-006 — Activated final-product capabilities

- Autonomous mode.
- Self-modification.
- Media validation.

#### OVR-B-007 — Multi-runtime backend tier

- Swap the local inference engine behind a closed RuntimeAdapter trait.
- RuntimePanel surface.
- Makes the replaceable-model thesis literal at the runtime layer.

#### OVR-B-008 — Knowledge-engine extensions

- Symbol-level impact-analysis API with impact radius, call flow, and symbol context.
- Multi-language DKG with at least 13 grammars.
- Resumable indexing.
- Cross-process indexing locks.
- Adaptive minified-file chunking.
- Interactive symbol-graph viewer.
- Entry-point detection.
- Per-project context-artifacts pipeline.

#### OVR-B-009 — Reasoning, framing, and project-scale context

- Ambiguity-score continuous gate.
- Chat REFLECT pipeline: per-message triage -> best-of-N reasoning -> auditor -> synthesize.
- Idea-framer mode.
- Per-level auditor modes.
- Cross-project Cosmos artifact tier.

### C — Verification-gap closure + testing-trust + code-completeness

#### OVR-C-001 — Verification closure

- Close B's verification gaps.
- Prove each area with a real cloud smoke test.
- Full SSB emission across all producers.
- Dashboard command surface.
- Cloud-mode hardening.
- Tier-C sandbox completeness.
- IPC dispatcher production swap.
- Anti-staleness automation.
- Feasibility prober.

#### OVR-C-002 — Pipeline reuse and constrained generation

- Compiled pipeline.
- Template crystallizer caches a recurring pipeline shape.
- Constrained decoding.

#### OVR-C-003 — Testing-trust tiers

- T-1/T-0: determinism plus a wiring contract.
- T-1: mutation testing.
- T-2/T-3: intent pairs plus metamorphic relations.

#### OVR-C-004 — Code-completeness layer

- Per-file line budgets.
- Per-file function budgets.
- Skeleton-first generation.
- Mini-critic.
- Accept-and-complete generation loop.
- Explicit purpose: enable weak models to produce complete code.

#### OVR-C-005 — Later paper-grounded improvements

- Tool-attention gating.
- Mutation-operator catalogs.
- Zero-trust MCP fuzzing.
- Credit attribution.
- Parsimonious routing.

#### OVR-C-006 — Public benchmark publication harnesses

- SWE-bench.
- Aider Polyglot.
- Codebase-memory head-to-heads.

### D — Reasoning-improvement substrate + inference-time grounding

#### OVR-D-001 — Wave 1 typed self-improvement surface

- Typed ReasoningTemplate archive.
- Cryptographic hash chain for the archive.
- Process-reward-model sidecar.
- Multi-verifier consensus: promotion requires at least two verifiers to concur.
- Five-tripwire regression stack.
- Best-arm-identification promotion using Thompson sampling.
- Time-windowed held-out task store.
- Cross-domain transfer canary.
- L3 frozen-surface access control.

#### OVR-D-002 — Wave 2 five-mode inference-time grounding

- Five-mode grounding taxonomy; the overview does not enumerate the five names.
- Orchestrator-owned mode router.
- LLM has no input into mode selection.
- Constructive-checker registry with sympy, z3, compile, and registry checkers.
- Operationalization predicate.
- Cross-family Adversary role.
- Sycophancy probe.
- Discovery-vs-search labeling.
- Conformal refusal calibration.
- Honest-refusal SLA.
- Typed observable-effect predicate.

#### OVR-D-003 — Wave 3 measurement

- Pair every capability with a quantitative benchmark.
- Pair every capability with an adversarial probe.

### E — 55-report improvement sweep

#### OVR-E-001 — UX and memory

- Infinite-context chat illusion.
- Local-LLM speed tricks.
- Model-handoff UX: show role, hide model.
- Sycophancy detection.
- Persona-drift detection.
- Durable two-tier user memory.

#### OVR-E-002 — Orchestration and reasoning

- Multi-agent failure-mode classification.
- Darwinian self-improvement guardrails.
- Self-evolving prompts.
- Improved evaluation benchmarks.
- Adversarial self-critique of ELAI's design.

#### OVR-E-003 — Product-grade code generation

- Idea-to-SaaS framing.
- Production full-stack checklists.
- TDD verification agents.
- Browser end-to-end testing.
- Firecracker sandboxing.

#### OVR-E-004 — Retrieval and inference

- Activate the ONNX retrieval stack and code-vector path; called the single highest-leverage fix.
- Tool-use reliability improvements.
- Reasoning-quality improvements.
- Local/cloud routing.
- AWQ + Marlin quantization.
- FlashAttention 2 (FA2) attention kernels.
- Context-window lift.
- MLX adapter.
- llama.cpp adapter.

#### OVR-E-005 — Trust and security

- Prompt-injection defense.
- MCP hardening.
- Symbol-graph tools.
- LSP as verifier.
- Formal-verification code generation.
- Defenses derived from public agent incidents.
- Cost hard ceilings.
- Token hard ceilings.
- Evaluation-corpus design.

### F — Fail-closed write-gates (the cant-lie security spine)

#### OVR-F-001 — Real end-to-end done gate

F is described as the first plan whose done-gate is an actual end-to-end trigger rather than only compilation and passing tests.

#### OVR-F-002 — Unified pre-write gate

- Fail-closed, multi-axis, in-process write gate.
- Axes: code vulnerability, hallucinated dependency, secret, and malicious skill.
- Blocks before disk write.

#### OVR-F-003 — Security measurement and content defense

- CWE ruleset.
- Produces the first measured pre-write insecure-rate.
- Data-plane tool filter blocks injected instructions found in untrusted content.

#### OVR-F-004 — Test integrity and output grounding

- Claimed PASS is bound to an orchestrator-captured hash.
- A fabricated PASS becomes detectable.

#### OVR-F-005 — Cross-system safety gates

- Cross-subsystem staleness quarantine.
- Learned-skill trust ladder gates autonomy.
- Safety-lattice completeness proof.
- Symbolic self-modification conflict detector.
- Write-time hallucinated-dependency gate.
- Per-iteration security-monotonicity gate blocks an iteration if N introduces a vulnerability class absent at N-1.

#### OVR-F-006 — Later F capabilities

- Close the first config-only DGM generation under the guard trio.
- Local deterministic NLI contradiction pre-filter.

### G — Trust-maximization

#### OVR-G-001 — Relationship to F

- Turns F's security spine into a distributable, reproducible trust product.
- Every G unit consumes an F-gate verdict instead of restating it.

#### OVR-G-002 — Taint propagation and runtime blast-radius control

- Harness-wide taint propagation.
- Deterministically block untrusted-content -> destructive-sink flows.
- Post-expansion filesystem gate.
- Egress IP-pin gate.
- Live MCP rug-pull re-pin gate.

#### OVR-G-003 — Reproducible-Run Artifact (RRA)

- Single portable file.
- Bundles action stream, seed, hardware pin, and every gate's verdict hash.
- Replay command: elai replay --verify.

#### OVR-G-004 — Reviewer-facing trust

- Reviewer cockpit.
- Diff-surface-to-review metric.
- Reviewer can skim vouched hunks.
- Per-change confidence score.
- Dual-LLM lethal-trifecta quarantine.

#### OVR-G-005 — SAFEDELTA

- Gate-ON versus gate-OFF ablation.
- Measures residual insecure diffs reaching a reviewer.
- Includes comparison against competitor harnesses.

#### OVR-G-006 — Provenance, redaction, and drift

- Signed in-toto provenance attestation.
- Offline verifier.
- Write-time secret/PII redaction gate.
- Model-drift regression canary monitors the exogenous underlying model.

### H — Efficiency + canvas + trust-hardening + grounded reasoning + reversibility + context lifecycle

#### OVR-H-001 — Efficiency

- Prove ELAI is cheaper on the same cloud model.
- Prompt caching on the production path.
- Cache-stable prompt layout.
- One-hour cache TTL.
- Output-token reduction stack:
  - diff emission;
  - FIM emission;
  - streaming early abort;
  - explicitly targets the 5x-billed lever.
- Tool Search tool.
- Batch API for non-interactive lanes.
- Deterministic observation masking.
- Calibrated architect/editor/verifier routing.
- Persisted oracle cache.
- Cost cockpit dashboard.

#### OVR-H-002 — Canvas workbench

- Replace the watch-only 3D orbit with a two-dimensional generative canvas.
- Model drives the canvas; user points at it.
- Read-only code graph.
- Live SSB overlay.
- File explorer.
- PLAN mode.
- RESEARCH mode.
- Model-driven canvas operations.
- App/website design mode with click-to-edit.
- General-purpose visual reply channel.
- Visual deixis is taint-gated.
- Retire the orbit visualizer.
- Underlying moat is a journaled, reproducible, taint-tagged operation stream.

#### OVR-H-003 — Autonomous-trading trust hardening

- Persisted, un-resettable trial ledger.
- Deflated-Sharpe, PBO, and MinBTL promotion.
- Reusable-holdout budgets.
- Leakage static gate.
- Runtime look-ahead gate.
- Trading metamorphic null tests.
- Same-seed rerun hash equality.
- Simulation-to-real parity.
- Result manifest.
- Shadow execution.
- Financial invariants.
- Gate self-validation (verify the verifiers).
- Reward-tampering gate.
- Fail-closed drift interlock.

#### OVR-H-004 — UX and user memory

- Staleness/intent-drift instrument.
- Two-tier user memory:
  - bi-temporal preferences;
  - code-grounded, validate-before-use facts.
- Auditor-objectivity firewall.
- Memory governance surface.
- MCP defer-loading.
- /rewind over the journal.
- Developer-event notifications.
- Plan-approval gate.
- Cost-confirmation gate on stop.

#### OVR-H-005 — Grounded-reasoning hardening

- Abstention-aware promotion credits an honest stop.
- Decorrelation-keyed verifier retention.
- Live checker-coverage instrument.
- Serial-depth-scaled reasoning budgets.

#### OVR-H-006 — DGM-verification hardening

- Replace placeholder byte-hash fitness with execution-grounded fitness.
- Replace placeholder keyword-grep behavior filtering with behavioral trap detection.
- Worst-case and lineage-diverse promotion scoring.
- Clade-metaproductivity parent selection.
- DGM evaluation firewall.
- Bandit driven by the real reward.
- Stated spine theorem: without an un-gameable filter stronger than the generator, a self-improvement loop collapses; with one, it converges.

#### OVR-H-007 — Reversibility

- Undo file time travel.
- Redo file time travel.
- Branching file time travel.
- Lossless paged transcript.

#### OVR-H-008 — Context lifecycle

- Regime-aware in-session context management.
- Local regime minimizes the context window.
- Cloud regime preserves cache stability.
- Cross-session sleep-time handoff.
- Grounded topic digest.

#### OVR-H-009 — Review anchoring and external steals

- Deterministic snippet-to-hunk finding anchoring; hallucinated line numbers become structurally impossible.
- Citation pre-binding on the answer path; ghost citations become impossible.
- Deterministic obligation-closure escape-hatch gate for the formal-proof lane.

### I — Fault isolation + evidence-ranked search + cache economics + anti-gaming + routing + flywheel

#### OVR-I-001 — Fault isolation

- Process-separated directive Supervisor.
- Supervisor is the only executor of side effects.
- Generator emits pure Intent structs.
- Generator cannot reach gate code.
- Copy-on-write workspace isolation for parallel branches.
- Hardened journal throughput.
- Atomic checkpoints.

#### OVR-I-002 — Evidence-ranked search

- Search ladder: best-of-N -> tournament -> tree search.
- Selection is by receipts, not by an LLM judge.
- Pristine-context injector.
- Anchor-pane slot.
- Cascading local-to-cloud escalation.
- Voluntary-abort tool.
- Dead-end hashing.
- Error-amplification telemetry.

#### OVR-I-003 — Cache-invariant economics middleware

- Byte-frozen prefix discipline.
- TTL tiering.
- Cache-hit hard guard.
- Typed belief-state handoff.
- Evidence-gated playbook curator.
- Memory guard.
- Dual serialization.
- Deterministic context compression.
- Semantic intermediate-summary cache.
- KV-cache/quantization evaluation.

#### OVR-I-004 — Deterministic gates and anti-gaming verification

- Tri-layer evaluation stack: hard veto -> calibrated scorer -> adversary.
- Tamper-evident hash chain.
- Attestation.
- AI-BOM.
- TDD preflight.
- Budgeted mutation scheduler.
- Confidence-interval scores designed to fit inside the loop.
- Sprint contracts.
- Progress-shaped rewards.
- Unified conformal calibration.
- Isomorphic anti-gaming canaries.

#### OVR-I-005 — Routing, budgets, and measurement

- Ledger-trained router trained on the journal's own verified outcomes.
- Dynamic thinking budgets.
- Budgeted-convergence instrument.
- DAG credit assignment.
- Diffusion draft lane behind the complete veto stack.

#### OVR-I-006 — Open-weight flywheel

- Verifier-ranked SFT exhaust from sealed runs.
- GRPO flywheel using execution-grounded rewards.
- Blocking safety guards protect the flywheel.

#### OVR-I-007 — Deterministic-over-prompt hardening

- Structural role scoping.
- Mechanical decision governance.
- Grammar pinning.
- Temporal tool-sequence enforcement.
- Cross-model prompt compilation.
- EARS requirement objects.
- QDELTA plus a statistical floor.
- Plan state derived from the repository, eliminating hand-maintained drift.

#### OVR-I-008 — Agent-agnostic trust-layer wrapper

- Run any external agent's tool traffic through ELAI.
- Apply ELAI taint controls.
- Apply ELAI write gates.
- Emit an RRA.
- This unbundles the trust wedge from ELAI's own agent.

#### OVR-I-009 — Later I tracks

- Design-phase critique lenses.
- Critique-only operating profile.
- Phase-0 kill-gate falsifier.
- Zero-retention export mode.
- Competitor-steal ports:
  - objective-hash sealed receipts;
  - bash interceptor;
  - verifier-input hardening;
  - provider protocol lanes;
  - auth broker;
  - unified model catalog;
  - checkpoint tool;
  - recipe runners;
  - structural AST edit tool.

### J — Harness-redesign residue

The source calls J the largest plan and says its center of mass is the harness (.claude/ plus workflow tooling), followed by a large research-steal and idea-driven tail.

#### OVR-J-001 — Perception and documents

- Deterministic image-ingestion gateway.
- Perception scaffold.
- Image bytes are conditioned before any model sees them.
- Counts must come from AXTree/DOM, never a VLM.
- Hardware detection.
- Per-lane model-eligibility catalog.
- MinerU document-parsing lane.

#### OVR-J-002 — Control and front door

- ELAI as an MCP-server control plane.
- Total-function front-door router.
- Chat is ELAI, not a model.
- Every message resolves to a lane or a clarify chip.
- Intent-control layer verifies AIM, not merely fidelity.
- A mis-aimed premise is surfaced before decomposition.
- Per-session file backups and undo.
- Handoff-file ledger.
- Incremental-evaluation skill.

#### OVR-J-003 — Voice front door: common contract

- Voice is a transducer pair over the same front-door spine, not a new command surface.
- User speaks to one supervisor voice: the harness.
- Harness responds from its own journal.

#### OVR-J-004 — Voice inbound

- Local streaming ASR.
- Utterances land on the same front-door router as typed chat.
- Acoustic ingress is taint-tagged as untrusted.
- Before routing, journal transcript, audio hash, and ASR-model pin.
- A voice run replays like a typed run.
- Sessions are addressed by phonetically distinct call signs.
- Mutating lanes require spoken readback confirmation.
- Readback is generated from the routed typed intent, never a transcript echo, specifically to catch mistranscription.
- High-risk actions are never voice-only.

#### OVR-J-005 — Voice outbound

- Deterministic narration plane.
- Every narratable event family declares a spoken rendering of its journal row.
- Rendering uses a template, seeded variants, and local TTS.
- No model participates in the truth path, so narration cannot invent the event.
- Notification dispatcher handles narration.
- Priority preemption.
- Step-grain rate limits.
- User levels: silent, summary, and verbose.
- The only model slot is on-demand, cross-session status digests.
- Status digests carry row-ID receipts.
- A deterministic row-count footer is appended outside model text so summary drift is audibly self-contradictory.

#### OVR-J-006 — Token and provider economics

- Unified token ledger.
- Accounting granularity: per turn and per lane.
- Honest per-model pricing.
- Usage window.
- Live meter.
- BYOK.
- Custom endpoints.
- Free-API rotation pool.
- Model-strength modality router.
- CodeAct on capable lanes.
- Constrained minimal-schema tool calls on the weak local lane.

#### OVR-J-007 — NASA-grade determinism gates

- Power-of-ten generated-code gate.
- SSB temporal-monitor engine.
- Multi-mode gate actions: suppress, insert, and edit.
- Every gate action is journaled.
- Formal verification of ELAI's critical core with Kani and TLA+.
- SSB hash chain.
- Watermark monotonicity.
- Gate-manifest traceability.
- IV&V independence.
- Policy-as-code spike.

#### OVR-J-008 — Determinism doctrine propagation

- Structurally enforce the model-is-a-cog doctrine at every lifecycle seam.
- Components of enforcement:
  - thin frozen floor;
  - cross-family advisory;
  - fail-closed debt ledger;
  - measured rate;
  - program falsifier.
- DD triad plus reasoning axis:
  - deterministic node-admission sizing;
  - extractive research compression;
  - cheap three-tier grounding stack;
  - within-step distill-reset reasoning.

#### OVR-J-009 — Context retrieval

- Adapt one in-domain retriever using ELAI's own supervision.
- Retrieval-abstention gate prevents retrieval when local context is sufficient.
- Graph as a retrieval channel.
- Mid-task retrieval action.
- RRF ablation harness.
- Context-rot calibration.

#### OVR-J-010 — GRC-v3 reasoning engine

- Typed belief-state reasoning carrier.
- Per-step external-receipt gate.
- A step commits only if its receipt passes; otherwise it abstains.
- Two-axis effort classifier.
- Deterministic overthinking guard.
- Receipt-gated candidate ladder: receipt > vote over independent samples > never LLM self-evaluation.

#### OVR-J-011 — Lacuna fill

- Journal-completeness substrate proves every side effect produced a row, not merely that rows were not altered.
- Runtime-observed DKG edges.
- Objective + plan-DAG prevention gate.
- Cross-language blast cones.
- Dynamic blast cones.
- Re-executable RRA.
- Seam-composition wiring fills.

#### OVR-J-012 — Vision orchestration and memory

- Route perception faithfulness through the trust spine; a ghost region is treated as a ghost citation.
- Declared context-access specification per dispatch.
- Signal-triggered grill offers.
- Worktree-pool lifecycle.
- Autonomous-supervision hardening.
- Deterministic token-efficient emission.
- Per-seat memory.
- Per-seat trust isolation.

#### OVR-J-013 — Verified case graph

- Mint a typed case node only after a proven fix.
- Self-wire deterministic edges.
- Reuse proven cases only behind a strict execution-grounded filter.
- Reuse is default-off.
- Intended effect: ELAI becomes cheaper on the user's recurring bugs.

#### OVR-J-014 — Read access, time, and recursive reasoning

- Read-access escalation ladder reads the cheapest rung that can prove it has the requested material.
- Live-index overlay prevents retrieval of a symbol just deleted.
- Counterfactual memory-ROI attribution.
- Output-side act-don't-narrate gate.
- Two-plane time model: time-as-context versus time-as-control.
- Transformation-surviving taint binding.
- Config-ingress admission gate.
- Recursive-reasoning DEPTH operator:
  - deterministic multi-hop traversal;
  - abstain rather than fabricate.

### K — Prove / distribute / package the wedge

K is explicitly characterized as adding almost no new capability: it primarily proves, distributes, and makes the existing spine affordable.

#### OVR-K-001 — Claims, economics, and distribution

- Problem-quantification dossier.
- Claims-policy lint: no external claim without its measured number or an explicit tag.
- Routing-economics benchmark yielding a production hybrid dollars-per-verified-task number.
- PR/CI evidence emitter.
- Render deterministic, hash-bound, replayable evidence into GitHub check runs.

#### OVR-K-002 — Gate affordability and market assurance

- Composed gate-tax budget.
- Risk-tiered gating policy.
- Compliance export.
- AppSec/insurance brief.
- Absorbable-unit audit.
- Provider-check pre-gate.
- Model-refresh cadence.
- OWASP control mapping.

#### OVR-K-003 — Measurement and doctrine

- Time-determinism doctrine.
- Reasoning-budget benchmark.
- Context-management measurement.
- Case-reuse economics benchmark.
- Re-executable-RRA differentiator.
- Journal-completeness doctrine.
- Doctrine-coverage doctrine.
- Continuous absorption watchdog.

#### OVR-K-004 — Net-new exception: multi-repo/workspace memory

- Cross-repository graph.
- Federated retrieval.
- Cross-repository blast radius.
- Connected versus disconnected state must be represented honestly.

#### OVR-K-005 — Net-new exception: credit-based seats

- OpenRouter rail.
- Hash-chained credit ledger.
- Metered reserve/settle.
- Value proposition is more value per dollar, verified; explicitly not cheap.

### L — Causal grounding gate + output quality + context core + exploratory reasoning

#### OVR-L-001 — Claims grounding beyond ELAI's own code

- Trust spine points outward to verify any model's claims on any backend.
- Typed claim-extraction substrate.
- Unified ClaimRisk score.
- Convergence fuse: a high verdict requires at least two mechanistically distinct predictors.
- Evidence round-trip predictor.
- Semantic-entropy inter-seed predictor.
- Counterfactual do-test on claims:
  - execute the intervention;
  - observe whether the claimed effect changes;
  - only an executable pass labels the claim causal.
- Risk-as-curiosity targeted-effort router.
- Hallucination-policy gate.
- Working-memory hygiene.
- Research/math deep-think profile.
- Hallucination-delta benchmark.

#### OVR-L-002 — Output-quality enforcement

- SOLID/maintainability enforcement.
- Four-seam layered gate.

#### OVR-L-003 — Familiarity surface

- User-authorable rules.
- User-authorable hooks.
- User-authorable skills.
- One-shot verified builder.
- Explicit safety claim in the design: safer than the tools being imitated.

#### OVR-L-004 — Context-core inversion

- Immutable transcript.
- Pure fold-view projection.
- Makes cache stability structural.

#### OVR-L-005 — Evaluation supply

- Measured-competitor evaluation-supply harness.
- Weak-lane verification-intensity policy.

#### OVR-L-006 — Exploratory-reasoning mode

- Claimant-authored falsifiers.
- Conjecture-criticism map of rival explanations.
- Runs on the local lane.

#### OVR-L-007 — Conjecture-memory middle tier

- Sits between append-only journal and verified case graph.
- Stores unproven conjectures as first-class retrievable objects.
- Computed statuses: live, refuted, pending, and suspended.
- Status is a pure journal fold.
- Refuting a premise suspends dependents instead of refuting them.
- No status is absorbing.
- Store provenance.
- Attach criticisms with evaluator-version edges.
- Store declared missing tests.
- Explore-lane only.
- Structurally unavailable to the proven-only code-fix path.
- Purpose: retain ideas ahead of current receipts without contaminating the trust spine.

#### OVR-L-008 — Refusal, novelty, and training-exhaust controls

- Refusals include a typed unblock condition.
- Unblock condition records what was tried and what evidence would change the answer.
- It is composed from the receipt trail.
- Explore benchmark includes a meaning-cluster novelty instrument under grounding-gate ablation.
- Open-weight training exhaust has a diversity guard.
- Train on moves, not survivors.
- Labels are filtered by the exogenous-grounding floor.
- Every dataset row is bound to the evaluator version that vouched for it.

#### OVR-L-009 — Improvement-conjecture planning boundary

- Improvement-conjecture tool is explicitly not a new runtime reasoning organ.
- Maintains a separate append-only artifact archive.
- Computes status from edges.
- Reports archive-relative equivalence alongside groundedness, value, and cost.
- Caller selects a declared profile: SAFE, NOVEL-CREDIBLE, or visibly speculative upside.
- Explicit runtime exclusions: no FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, or bridge from the planning archive into L-23/L-24.
- Those excluded mechanisms failed evidence gates and are outside the specification.

#### OVR-L-010 — Separate Q-10 research-coupling proposal

- Does not reopen the excluded planning-to-runtime bridge.
- One-way input only.
- Verified research may be compressed into a typed relational packet.
- Packet fields/capabilities include:
  - source mechanisms;
  - object mappings;
  - shared relations;
  - invariants;
  - disanalogies;
  - contradictions;
  - provenance.
- Packet is always SOURCE_ONLY evidence.
- Every source mechanism and relation-source-predicate is support-edge bound.
- It may inspire a transfer but cannot verify a target claim.
- Target anchors, falsifiers, grounding receipts, and execution remain independent.
- Before reaching planning skills or later the explicit explore lane, the interface must beat direct, source-blind matched-scaffold, verified-evidence, and relation-shuffled controls under locked executable gates.

### M — Commercial multi-seat

#### OVR-M-001 — Offline-first commercial seat

- Make a single seat usable by a company without becoming an organizational SaaS.
- Signed, offline-verifiable seat entitlement.
- Purchase rail.
- Commerce failures fail open to a free tier.
- Safety gates must never brick.

#### OVR-M-002 — Organization controls and evidence

- Organization policy pack.
- Policy pack may only monotone-tighten policy.
- Distributed as a signed file.
- Fleet evidence rollup.
- Organization-grain answers are computed from seat journals.
- Zero telemetry.

#### OVR-M-003 — Distribution and trust continuity

- Signed release channel.
- Auto-updater.
- Support bundle.
- Commercial packaging.
- Standalone open-source verifier.
- Trust continuity.

#### OVR-M-004 — Open evidence standard

- Evidence format becomes a versioned open standard.
- Third-party tools can emit ELAI-verifiable evidence.

### N — Interlace / anti-fragmentation

N is described as inventing almost nothing and connecting fragmented mechanisms into one canonical subsystem per concern.

#### OVR-N-001 — Fragmentation diagnosis and anti-fork policy

- Named fragmentation examples: four claim types, ten routers, six intensity deciders, and five provenance scales.
- Declare one canonical subsystem for each fragmented concern.
- Mechanize an anti-fork rule.

#### OVR-N-002 — Canonical composition substrates

- Measurement spine as one subsystem.
- Corpus-grain composition validator.
- Validator is the first machine check that plans communicate with one another.
- Validator runs at plan-authoring time.
- Generalized anti-selection-bias promotion primitives.
- One Claim/Belief object plus a receipt memo.
- Policy/intensity arbitration plane.
- Provenance/trust lattice plus admission-gate family.
- Human-render required as a verdict field.
- Governance-coverage substrate.
- One counterfactual/ablation engine.
- Clarity/novelty-to-budget wiring.
- Journal-fold substrate.
- Consistency and deduplication bundle.

#### OVR-N-003 — External steals and governance closure

- Output-boundary registry.
- Self-improvement edit-loop instruments.
- Anti-affirmation stability measurement.
- Evaluator-refutation verdict collapse.
- Exogenous-grounding floor.

### O — Personal research machine

#### OVR-O-001 — Composition and differentiation

- TEIKOKU-style personal research machine on the ELAI canvas.
- Approximately 80% composition of existing capabilities.
- Differentiators: gated, journaled, replayable, and closed-vocabulary.

#### OVR-O-002 — Research daemon

- Generalizes Cosmos.
- User-editable typed topic set.
- Taint protection.
- Replay support.

#### OVR-O-003 — Scheduled TaskSpec substrate

- Typed cron schedules.
- Typed event schedules.
- Scheduled work enters an inbox.
- Product analogy stated in the source: n8n with receipts.

#### OVR-O-004 — Brain vault

- Brain-vault contract.
- Mount an existing Markdown vault.
- Namespace claim permits two harnesses to share one vault.

#### OVR-O-005 — Canvas workflow

- WORKFLOW mode over TaskSpecs.

### P — Personal software incarnation

#### OVR-P-001 — Grain lift

- Lift trust from changeset to app.
- Lift orchestration from task to fleet.
- Lift the commercial/user grain from seat to world.
- Intended product: a person's world of connected bespoke tools in which everything is protected, organized, and committed.

#### OVR-P-002 — App fleet

- The user's N bespoke apps become first-class objects.
- Health tick for each app.
- Gated automatic repair.

#### OVR-P-003 — World connectivity and portability

- Inter-tool ports.
- World-grain typed event bus.
- Persona/mechanism split.
- Hand-this-box-to-a-friend export.

#### OVR-P-004 — Living external brains

- Mount live Obsidian or Notion.
- Mounting is governed by a change-feed contract.

#### OVR-P-005 — Non-coder operation and scaled ceremony

- Non-coder operator surface.
- Graduated ceremony.
- Trust spine scales downward for low-risk tools.
- Explicit example: a toy runs gates in observe-only mode.

#### OVR-P-006 — Personal privacy and process adaptation

- Life-data privacy lattice.
- Described as taint's mirror: controls owned bytes flowing out.
- Per-user operating profile.
- Adapts process, not only content.
- Asymmetric policy: tightening is free; loosening requires measured evidence.

#### OVR-P-007 — Small-team collaboration

- Supports 2–10 humans plus agents in one world.
- Lossless, deterministic two-seat journal merge.

#### OVR-P-008 — Signed connector/stage PACK format

- Opens the command-center world to an unbounded connector surface without giving up receipts.
- Owns the capability format, not the external ecosystem.
- Signed and versioned connector/stage-PACK format.
- One generic http_call stage.
- http_call is parameterized by a declarative, admission-gated ConnectorSpec.
- Approximately 80% of connectors should be a data-only diff of the first pack, rather than a new build.
- Genuinely new behavior uses a closed code-pack lane.

#### OVR-P-009 — Ingress-adapter family

- Normalize any external event shape into one taint-tagged, journaled SsbEvent.
- Explicit ingress shapes:
  - inbound webhook push;
  - streaming/persistent listener;
  - outbound pull;
  - genuinely other device.
- Trigger enum never grows as connector count grows.
- Existing downstream trigger, replay, provenance, and privacy machinery continues unchanged.

#### OVR-P-010 — Connector auth plane

- First-class credential vault.
- Replaces scattered connector tokens with one authentication plane.

#### OVR-P-011 — Workflow from goal

- Synthesize user intent into a TaskSpec graph.
- Graph is constrained to closed enums.
- Nodes and wires may only use the existing typed stage/connector vocabulary.
- Render proposed workflow on the canvas.
- Any vocabulary gap is shown visibly disconnected rather than silently invented.
- Human confirmation is required before execution.
- This is the n8n-user front door while keeping every proposed wire typed, gated, and replayable.

#### OVR-P-012 — Desktop/GUI operator lane

- Operates in a disposable sandbox.
- Every coordinate action binds to the exact observation from which it was computed.
- Binding is the content hash of the delivered image.
- Binding is invalidated as soon as that image leaves context.
- It can never retarget to another display.
- Every action is journaled in a taint-tagged operator lane.
- A workflow's correctness must never depend on this operator lane.

#### OVR-P-013 — GUI perception trust boundary

- Model points by integer ID from a source-tagged element list instead of issuing raw coordinates.
- Every element declares provenance:
  - accessibility-exact;
  - OCR;
  - caption;
  - pixel.
- Pixel perception is a fallback only when no accessibility tree exists.
- Counts must never originate in a vision model.

#### OVR-P-014 — Explore then crystallize

- Agentic GUI exploration may mint a deterministic replayable script.
- Script is bound by a reproducible-run receipt to the exploration that produced it.
- Replay becomes a cheap, zero-reasoning execution mode.

#### OVR-P-015 — Verified-case GUI reuse

- Verified-case-graph reuse loop: retrieve, distill, and fuse.
- Save is allowed only after a proven fix.
- Tie-break judge scores journal rows, not screenshots.

#### OVR-P-016 — Gated install catalog

- Turns capability-on-demand requests such as download this app into named catalog entries.
- Catalog is admission-gated.
- Entries resolve to headless connector packs.
- GUI driving is a last resort, not the primary mechanism.

#### OVR-P-017 — Journaled co-browse

- Embedded browser pane where user and model can read and click through any site together.
- Pane is dumb glass over an engine-owned browser.
- Every navigation, click, and keystroke from either actor is journaled before dispatch.
- An unjournaled input route is structurally impossible.
- Session persistence is typed:
  - ephemeral by default;
  - named profiles stored under credential-vault custody.
- Outbound typed text passes through the life-data privacy lattice before leaving.
- Page content enters as tainted ingress.
- Tainted page content can never authorize a mutating lane.
- Product distinction stated by source: a co-browse pane with receipts.

#### OVR-P-018 — Smartphone as typed dimension

- Smartphone is a typed dimension of the existing design/test surface, not a parallel subsystem.
- Device-profile catalog.
- Render any app preview or browse pane in a phone frame.
- Run browser-E2E journeys across a device matrix.
- React Native apps enter click-to-edit design mode through their web render and must be labeled as such.
- Native emulator is another sandboxed operator backend.
- Emulator boots from a pinned snapshot.
- Snapshot hash is included in the reproducible-run pin.
- Emulator perception uses the platform accessibility tree.
- Exploration crystallizes into actuation-only replay flows.
- Verdicts still come from the off-agent evidence leg.
- Every mobile result records its render source.
- A cheap rendering rung may never silently satisfy a native gate.
- Works on the phone is a typed claim.

#### OVR-P-019 — Provider failover and billing truth

- Provider failover.
- Failure-class-specific cooldown lanes.
- Session stickiness preserves cache warmth.
- An explicit model choice must be honored strictly.
- Billed truth is typed separately from an estimate.

#### OVR-P-020 — Durable workflow runs and approval binding

- Durable workflow runs.
- Revision-counter concurrency.
- Sticky cancellation.
- Every approval binds a canonical hashed plan and its operands.
- Execution is denied if grant and execution have drifted.

#### OVR-P-021 — Configuration and operational lifecycle

- Fail closed when a policy object is absent.
- Preserve rejected writes.
- Environment allowlist.
- Operator doctor with stable exit codes.
- Verified backups of the SSB journal, explicitly described as the product.

#### OVR-P-022 — Context and memory lifecycle

- Pre-compaction flush.
- Guarantees nothing is destroyed before it has been offered to durable memory.
- Versioned, on-demand index.
- Keeps skills and rules out of the prompt until needed.

#### OVR-P-023 — Gate credibility and structural redaction

- Plant known-bad canaries through the real detectors.
- If a checker silently stops detecting, the entire gate fails.
- Redaction is structural.
- Shareable audit projection has no field in which a secret could reside.
- Shareable audit projection is a different type from the content-carrying debug bundle.

#### OVR-P-024 — Control-plane hardening

- A subagent return is fenced as inert evidence, not trusted instruction.
- Tool authority is depth-scoped.
- Device pairing is challenge-signed.
- Unrecognized event families are withheld rather than leaked.

### Q — Engineering-taste hardening + orchestration economics + research-conditioned creativity + research-instrument economics + autoresearch synergy

Q turns the trust spine onto cost and onto the harness itself. It is described as hardening rather than a new user surface.

#### OVR-Q-001 — Standing goals

- A completed unit's off-agent verification becomes a persistent scheduled invariant.
- The invariant is rechecked indefinitely.
- A once-passing gate that later rots re-enters the work pipeline.
- Doctrine stated by the source: a goal verified once is only an assumption with a timestamp.

#### OVR-Q-002 — Quorum wake gate

- Cheap cross-family voters decide whether an expensive lane wakes on a scheduled tick.
- A quiet tick should cost almost nothing.
- A false vote may delay a wake but may never suppress a verified result.

#### OVR-Q-003 — Advisor lane

- Cheap model executes every turn.
- Flagship model is consulted as a budgeted tool approximately once per task when judgment is needed.
- The same cheap executor continues after receiving advice.
- Cost is compared honestly with a same-lane baseline.

#### OVR-Q-004 — Swap-restart rule

- A model substitution in the middle of a unit restarts that unit from a fresh state.
- Reason: iteration state is model-conditioned.
- Discarded work is recorded.
- Keeps per-model credit attribution clean.

#### OVR-Q-005 — Per-turn local token hashing

- Hash emitted tokens for each local turn.
- Replay can localize divergence to the first divergent turn.
- Reproducibility claim is scoped to the layer that earns it:
  - harness action stream on any backend;
  - local tokens within the same class;
  - cloud action stream rather than cloud tokens.

#### OVR-Q-006 — Gate-wide evidence-corruption corpus

- Apply to every frozen checker in the gate manifest.
- Corruption cases: truncated, corrupt, missing, and flipped evidence.
- Every checker must be proven to fail closed against those cases.
- Missing evidence is a failure becomes a measured per-gate kill rate instead of an untested promise.
- Testing is concentrated on the gates because gates are the product.

#### OVR-Q-007 — Weak-lane code-style benchmark

- Measure whether code structured for weaker models increases task success.
- Promote a style rule into the output-quality gate only after a measured win.

#### OVR-Q-008 — Standalone-verifier ceiling

- Locked size ceiling.
- Locked dependency ceiling.
- Verifier must remain smaller and more dependency-light than what it audits.

#### OVR-Q-009 — Distribution hygiene

- Keep vendored external runtimes cheaply mergeable with fast-moving upstreams.
- Derive dashboard, CLI, canvas, and control-plane surfaces from one command registry.
- Prevent client drift.

#### OVR-Q-010 — Research-conditioned creativity

- Verified research becomes a source-scoped relational inspiration packet.
- Candidates expose their transfer edges.
- Separate target-side grounding and execution determine whether an analogy worked.
- Packet reaches paper-to-plan/idea-intake only after beating these control arms on real ELAI fixtures:
  - direct;
  - matched scaffold;
  - evidence only;
  - relation shuffled.
- Packet reaches /reason --profile explore only after an additional observe-only L/N ablation improves both receipt-valid meaning-cluster coverage and best verified outcome.
- That later admission must not harm PS-3, the exogenous-grounding floor, cost, or the wall between proven and unproven lanes.

#### OVR-Q-011 — Research instrument before search improvement

- Novelty classifier runs a known-answer canary every time.
- Canary proves the classifier can distinguish same from distinct.
- Verdict fails closed immediately if the classifier goes dead.
- Explore benchmark reports value per dollar.
- Spend cap is locked.
- Control arms are named.
- A policy that wins only by spending more is flagged, not promoted.
- Frozen pre-change baseline makes future deltas attributable.

#### OVR-Q-012 — Search economics after instrument validation

- Give each research operator only its minimal context.
- Spend the saved budget on additional iterations.
- A bandit allocates search across draft lineages.
- When one lineage stalls, fork a new strategic lineage from the best findings.
- Adopt only when it beats a plain deterministic scheduler at equal budget.
- A strategy may not win merely by spending more.

#### OVR-Q-013 — Receipt-bound briefing

- Every compiled briefing line binds to a resolvable receipt.
- Drop a line that cannot prove its provenance instead of asserting it.
- Closes the distinction between receipt-bound answers and previously unbound briefings.

#### OVR-Q-014 — Instrument the idea-selection engine

- Novelty leg runs a known-answer canary every time.
- Fail closed when it can no longer distinguish same from distinct.
- Do not silently fall back to a mechanism incapable of separating candidates.
- Every refutation records the loser's margin.
- A within-noise kill remains honestly reinstatable and distinct from decisive defeat.
- Tie-breaking favors the cheaper candidate on measured cost.
- If adjacent evidence partly contradicts a survival claim, attach the contradicting citation rather than silently relitigating it.

#### OVR-Q-015 — Research-benchmark protocol hardening

- Per-task Pareto retention preserves a candidate that wins at least one task even if aggregate score is dominated.
- A refutation must prove it had enough statistical power to refute.
- Underpowered failure is inconclusive, not disproven.
- Every benchmark fixture begins untrusted until a human establishes its baseline.
- One repeat-aware variance helper is shared across benches.
- Cost-normalized comparisons must align across benchmarks.
- Handoff resumes at the point where notes disagree with the live tree.
- This turns handoff drift into a measurement instead of silently reverifying it away.

## 4. Explicit composition and dependency topology

This section records relationships stated by the overview; it does not infer implementation wiring.

### OVR-COMP-001 — Core dependency chain

1. Phase 0 provides the seeded clock/entropy and byte-identical execution foundation on which all replay and reproducibility claims depend.
2. Phase 1 provides hybrid retrieval and repository context.
3. Phase 2 makes reasoning a typed, grounded, budgeted state machine.
4. Phase 3 supplies state, orchestration, supervision, context budgets, rules, planning, handoff, simulation testing, and local inference.
5. Phase 4 isolates agents and hardens tools/plugins while adding pipeline construction, feasibility, autonomy, self-modification, media validation, dashboard control, and runtime interchangeability.

### OVR-COMP-002 — Activation and verification

- B makes the core run end to end, connects producers to FSM/SSB telemetry, and exposes it through chat/dashboard/knowledge/product surfaces.
- C closes B's verification gaps with real cloud smoke tests, producer-complete SSB emission, production IPC, testing-trust tiers, and code-completeness controls.
- E's ONNX/code-vector activation explicitly depends on the Phase-1 retrieval components existing but not yet being active on the production path.

### OVR-COMP-003 — Security-to-distribution chain

- F supplies the fail-closed pre-write and evidence-integrity verdicts.
- G must consume F verdicts rather than reimplementing them, then adds taint, RRA, runtime blast-radius controls, reviewer evidence, attestation, redaction, and drift monitoring.
- K distributes the composed trust result through PR/CI evidence, compliance artifacts, benchmarks, and a standalone verifier.
- M lifts distribution to offline-verifiable commercial seats, organization policy/rollup, signed releases, and an open evidence standard.

### OVR-COMP-004 — Self-improvement trust chain

- 4H creates DGM-derived configuration/rule/data-table modification with a separate evaluator.
- D adds a typed, hash-chained ReasoningTemplate mutation surface and multi-verifier promotion.
- F places config-only DGM behind security guards.
- H replaces placeholder DGM filters with execution-grounded and behavior-grounded filters and adds lineage-aware promotion/selection.
- I uses sealed, verified runs for SFT/GRPO exhaust behind blocking safety guards.
- N canonicalizes promotion, provenance, claim, arbitration, and measurement mechanisms to avoid competing forks.
- Q makes the improvement selector and research benchmark themselves measured and fail-closed.

### OVR-COMP-005 — Journal as shared substrate

The SSB append-only journal supports or feeds:

- typed inter-agent events and deterministic result ordering;
- lossless session handoff and /rewind;
- dashboard/canvas live overlays;
- RRA and replay;
- reviewer and provenance receipts;
- token/cost/router learning;
- voice input/output truth;
- journal completeness and case graphs;
- typed schedules and research topics;
- co-browse and GUI operator logs;
- multi-seat fleet rollups and journal merge;
- pure-fold claim/conjecture statuses;
- standing-goal rechecks and handoff-drift measurement.

### OVR-COMP-006 — Retrieval and memory composition

- Phase-1 BM25, vectors, PageRank, graph anchoring, and agentic retrieval form the initial retrieval cascade.
- B adds symbol impact, multi-language graphing, resumable indexing, viewers, entry points, and context artifacts.
- H/I/J add regime-aware context, caches, memory guards, deterministic compression, retrieval abstention, live-index overlays, and graph retrieval.
- J's verified case graph admits only proven fixes; L's conjecture tier is deliberately separated from that proven-only path.
- K federates retrieval and graph context across repositories.
- O generalizes Cosmos into topics/vault research.
- P mounts external brains and introduces pre-compaction durability guarantees plus on-demand skill/rule indexing.

### OVR-COMP-007 — User surface evolution

- 4K/B: structured dashboard and live chat, with the orbit as activity feed.
- H: retire orbit in favor of a model-driven/user-pointed 2D canvas with PLAN, RESEARCH, and design interactions.
- J: one total-function typed/voice front door over the same FSM, routing, journal, and notification spine.
- O: add WORKFLOW mode for scheduled personal research.
- P: lift the canvas/world to app fleet, connector workflows, GUI operation, co-browse, phone/device testing, and small-team use.
- Q: derive every control surface from one command registry to prevent drift.

### OVR-COMP-008 — Grain progression stated by the source

- Core/B–C: verified task and changeset.
- K: prove and distribute the single-seat wedge.
- M: company/multi-seat, still offline-first rather than org SaaS.
- O: personal research machine.
- P: changeset -> app, task -> fleet, seat -> world.
- P connector and operator waves extend the same receipts to external services, desktop/GUI, browser, and mobile rather than creating separate trust systems.

### OVR-COMP-009 — Whole-corpus sequence stated in “The shape of the whole”

1. Phases 0–4: deterministic hybrid-retrieval, grounded-reasoning coding agent with process isolation and self-modification.
2. B–C: end-to-end execution, verification, testing trust, and code completeness.
3. D–E: self-improvement substrate, inference-time grounding, and broad efficiency/security improvement.
4. F–G: fail-closed security spine and reproducible/distributable trust product using RRA, SAFEDELTA, attestation, and reviewer cockpit.
5. H–I: efficiency, generative canvas, autonomous-trading trust, evidence-ranked search, cache economics, anti-gaming verification, and open-weight flywheel.
6. J: harness redesign, perception, control plane, determinism gates, GRC-v3, verified case graph, and seam/lacuna fills.
7. K–M: prove, package, distribute, and sell the wedge from single seat to company, plus an open evidence standard.
8. N: consolidate fragmented concerns into canonical subsystems.
9. O–P: lift the spine to personal research and a personal software world; add typed connectors, sandboxed GUI operation, durable execution/failover/ops, and gate credibility without losing receipts.
10. Q: apply the trust spine to cost and itself through scheduled verification, economical orchestration, replay hashing, per-gate corruption testing, constrained verifier size, and benchmark-gated research transfer.

## 5. Exact state, mode, tier, role, and action vocabulary exposed by the overview

This inventory helps later detail passes detect missing names and resolve summary-only placeholders.

### States and lifecycles

- GRC states: Observe, Hypothesize, Plan, Act, Verify.
- FSM states: Idle, Planning, Feasibility, TestingFirst, Executing, Auditing, Memorizing, Complete, Configuring.
- Rule lifecycle: proposed, shadow, active.
- Conjecture computed statuses: live, refuted, pending, suspended; none is absorbing.
- Session history: snapshot DAG with branch and merge.
- Workflow run controls: revision-counter concurrency and sticky cancel.
- Co-browse persistence: ephemeral by default or a named profile in credential-vault custody.
- Trial/holdout state: persisted un-resettable trial ledger and reusable-holdout budgets.

### Human and gate actions

- FSM human decisions: approve, reject, modify, redirect.
- SSB temporal-monitor gate actions: suppress, insert, edit.
- Autonomous escalation reasons: high risk, missing evidence, escalation trigger.
- Autonomous fallbacks: task splitting and model substitution.
- Planning conjecture profiles: SAFE, NOVEL-CREDIBLE, visibly speculative upside.

### Runtime, reasoning, retrieval, and output modes

- Runtime/task placement: local, cloud, hybrid; plus vendored coding-agent CLI backends under RuntimeAdapter.
- Per-role backend assignment and fallback chains.
- Focus Tunnel emissions: FIM, DIFF, STRUCTURED.
- RAG tiers: FAST, STANDARD, DEEP.
- Dashboard/canvas: PLAN, RESEARCH, app/website design, visual reply, and later WORKFLOW.
- Voice narration: silent, summary, verbose.
- Critique-only operating profile.
- Research/math deep-think profile.
- /reason --profile explore.
- Graduated ceremony includes observe-only gates for a toy.
- Five-mode inference-time-grounding taxonomy exists, but its five mode names are not supplied in the overview.

### Named roles

- Research Agent.
- Designer.
- Architect, editor, verifier routing roles.
- Cross-family Adversary role.
- Directive Supervisor, the only side-effect executor.
- Model roles are shown in handoff UX while specific model identity is hidden.

### Tiers and trust orderings

- Five-tier tool validation.
- Three-tier edit matching.
- Plugin static analysis: T1 rules, T2 tree-sitter, T3 pattern match.
- Media: deterministic Tier 1 veto, advisory Tier 2 VLM.
- Testing trust: T-1/T-0 wiring/determinism, T-1 mutation, T-2/T-3 intent/metamorphic.
- Tri-layer evaluation: hard veto -> calibrated scorer -> adversary.
- GRC-v3 candidate priority: receipt > independent-sample vote > never self-evaluation.
- GUI element provenance: accessibility-exact, OCR, caption, pixel.
- Mobile render source must remain typed so a cheap rung cannot discharge a native gate.
- Research relational packets are SOURCE_ONLY evidence.

## 6. Explicit constraints, prohibitions, fail behavior, and privacy guarantees

### Model and agent authority constraints

- Agent processes hold no network, LLM client, or tools in Phase 4A.
- Generator emits pure Intent and cannot reach gate code in I.
- Modifying agent cannot evaluate its own modification.
- LLM cannot influence D's grounding-mode selection.
- LLM self-evaluation is never the last resort in the GRC-v3 candidate ladder.
- Static-analysis hot path contains no LLM.
- Voice narration truth path contains no model.
- GUI workflow correctness may never rest on the operator lane.

### Evidence and failure constraints

- Missing evidence is FAIL, never SKIP.
- Gates are frozen and read-only to the run they judge.
- Fabricated test PASS is detectable through orchestrator-captured hash binding.
- Every side effect must have a journal row.
- Unjournaled co-browse input is structurally impossible.
- Briefing lines without resolvable provenance are dropped.
- A goal that once passed is periodically reverified rather than assumed forever.
- Evidence corruption must produce a measured fail-closed kill rate for every gate.

### Security and privacy constraints

- Security and finance rules are immutable.
- Untrusted content cannot flow to a destructive sink.
- Tainted page content cannot authorize a mutating lane.
- Outbound user text/life data crosses the life-data privacy lattice.
- Shareable audit data is structurally unable to contain a secret and is type-separated from debug content.
- Organization fleet rollup uses seat journals with zero telemetry.
- Zero-retention export mode exists.
- Unrecognized event families are withheld rather than leaked.
- Tool authority is depth-scoped; subagent output is inert evidence.

### Safety and operational failure behavior

- Commerce failures fail open to a free tier; safety gates never brick.
- Missing policy objects fail closed.
- Rejected configuration writes are preserved.
- Spoken high-risk operations are never voice-only.
- Model swap mid-unit discards recorded work and restarts fresh.
- A false quorum vote may delay work but cannot suppress a verified result.
- An underpowered research refutation is inconclusive, not disproven.
- Pixel perception is fallback-only and vision models never supply counts.
- GUI coordinates expire with their source observation and cannot retarget.
- GUI automation and connector code use closed/admission-gated lanes for genuinely novel behavior.

## 7. Explicitly conditional, gated, future, dropped, or non-runtime items

- Phase 1F speculative decoding was evaluated and explicitly dropped for Phase 1.
- Phase 1H SPLADE is a conditional keyword-expansion evaluation.
- Phase 4J Empire integration is conditional.
- Autonomous execution delegates on high risk, missing evidence, or escalation.
- Case-graph reuse is default-off and gated by execution-grounded proof.
- Diffusion drafting sits behind the full veto stack.
- Open-weight SFT/GRPO is behind blocking safety guards.
- Q-10 relational research packets cannot enter planning/idea intake until they beat four controls, and cannot enter explore until an additional L/N ablation passes without harming named safety/cost walls.
- L's planning improvement-conjecture archive is not a runtime reasoning organ and has no bridge to L-23/L-24.
- FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, and the archive-to-runtime bridge are explicitly outside the specification.
- A source-only research packet may inspire a target transfer but cannot verify the target.
- Style rules for weak models promote only after measured task-success improvement.
- Search/bandit changes adopt only at equal budget against a deterministic scheduler.
- Connector behavior beyond the declarative vocabulary must use the closed code-pack lane.
- GUI driving is a last resort after headless connector-pack resolution.
- Native/mobile claims cannot be discharged by a cheaper web-render rung.
- Policy-as-code in J is described as a spike, not an unconditional mature capability.

## 8. Cautious implications — not explicit feature claims

The following are interpretation aids and must not be treated as requirements unless confirmed in detail sources:

- Because many layers share SSB rows, Claim/Belief objects, receipts, taint, RuntimeAdapter, TaskSpec, and RRA, stable versioned contracts are likely necessary; the overview does not specify their schemas.
- Terms such as byte-identical, structurally impossible, fail closed, and never exceed are design requirements in this spec-only document, not proof that current code satisfies them.
- “Sub-10ms,” “large token savings,” “5x-billed lever,” “approximately once per task,” “approximately 80%,” and similar statements need benchmark definitions and hardware/workload conditions from detailed plans.
- The five grounding modes, five regression tripwires, four safety-enforcement layers, guard trio, safety lattice, PS-3, DD triad, exact plan identifiers such as L-23/L-24, and many tier semantics are named but not fully enumerated here.
- The overview sometimes describes later layers as activation, closure, hardening, replacement, or wiring of earlier designs. Only detail sources and wiring checkers can distinguish duplicate wording, planned replacement, migration, and actually composed implementation.

## 9. Source coverage and traceability

### Primary-file coverage

The source was read in full in three contiguous passes: lines 1–240, 241–480, and 481–592. Section coverage:

| Source lines | Overview section | Discovery coverage |
|---:|---|---|
| 1–30 | status disclaimer, thesis, three invariants | Sections 1, 6, 8 |
| 31–134 | Phases 0–4M | Section 2 |
| 135–157 | B | OVR-B-* |
| 158–170 | C | OVR-C-* |
| 171–184 | D | OVR-D-* |
| 185–198 | E | OVR-E-*; the source describes 55 reports, one recommendation per session |
| 199–211 | F | OVR-F-* |
| 212–224 | G | OVR-G-* |
| 225–261 | H | OVR-H-* |
| 262–293 | I | OVR-I-* |
| 294–350 | J | OVR-J-* |
| 351–366 | K | OVR-K-* |
| 367–409 | L plus planning boundary and Q-10 coupling | OVR-L-* and Section 7 |
| 410–418 | M | OVR-M-* |
| 419–432 | N | OVR-N-* |
| 433–440 | O | OVR-O-* |
| 441–508 | P, connector, computer-use, mobile, and orchestration-mechanics waves | OVR-P-* |
| 509–563 | Q, including fourth/fifth/sixth tracks | OVR-Q-* |
| 564–592 | whole-system shape and invariant | Sections 1 and 4 |

### Detail indexes that later agents must inspect completely

Core phases:

- docs/specs/guide/steps/phase-index-MASTER.txt

Letter summaries:

- docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md
- docs/thefinalglance/C-plan/C-SUMMARY_INDEX.md
- docs/thefinalglance/D-plan/D-SUMMARY_INDEX.md
- docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md
- docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md
- docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md
- docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md
- docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md
- docs/thefinalglance/J-plan/J-SUMMARY_INDEX.md
- docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md
- docs/thefinalglance/L-plan/L-SUMMARY_INDEX.md
- docs/thefinalglance/M-plan/M-SUMMARY_INDEX.md
- docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md
- docs/thefinalglance/O-plan/O-SUMMARY_INDEX.md
- docs/thefinalglance/P-plan/P-SUMMARY_INDEX.md
- docs/thefinalglance/Q-plan/Q-SUMMARY_INDEX.md

### Original/detail-file families that remain mandatory after the summary pass

- Every Phase 0–4M leaf/original plan referenced by phase-index-MASTER.txt, under docs/specs/guide/steps/.
- Every original session, wave, report, benchmark, appendix, and amendment referenced by each B–Q summary index under its matching docs/thefinalglance/[LETTER]-plan/ directory.
- Detail files are especially required wherever the overview provides only a collective name or count rather than enumeration: D's five modes/five tripwires/L3 surface; E's 55 recommendations; F's guard trio and safety lattice; H's trading statistics and DGM filters; J's power-of-ten gate, DD triad, context-access spec, and case graph; L's four-seam quality gate, L-23/L-24, and Q-10 controls; N's canonical old-to-new subsystem mappings; P's PACK/ConnectorSpec/SsbEvent/TaskSpec schemas and mobile/operator claims; and Q's PS-3, evidence-corruption corpus, admission benches, statistical-power rules, and per-task Pareto protocol.
- Feature map and wiring checker files must be inspected separately to establish implementation status. The overview does not identify their exact paths.

### Explicit non-coverage

- No implementation source code was inspected.
- No wiring, shipped/dark, test-passing, schema-count, SSB-count, or effort claim is made.
- No B–Q summary or original plan was opened during this pass; those are intentionally delegated to later discovery agents.
- No capability was inferred merely from a likely implementation requirement. Interpretations are isolated in Section 8.

## 10. Exhaustiveness checkpoint for the compiler agent

Before the final super-feature list is considered complete, reconcile this overview record against detail-agent output for:

1. all exact IDs beneath Phases 0–4M and B–Q;
2. every named mode/tier/state and the missing member names noted above;
3. every producer/consumer of SSB, RRA, Claim/Belief, receipt, taint, TaskSpec, PipelineSpec, RuntimeAdapter, and gate verdicts;
4. all replacement relationships so obsolete placeholders are not presented as coexisting final behavior;
5. all conditional admission tests, default-off behaviors, and explicit exclusions;
6. every quantitative threshold, benchmark control, timeout/TTL/budget, cardinality, and fail behavior;
7. final cross-layer behavior after N's anti-fragmentation canonicalization;
8. actual implementation/wiring status only from the appropriate feature map and wiring checkers, clearly separated from this spec-as-designed catalog.
