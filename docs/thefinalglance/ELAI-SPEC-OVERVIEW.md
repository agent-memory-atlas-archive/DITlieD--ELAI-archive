# ELAI — Systems Overview (spec-only)

What ELAI is, as designed. Every entry describes the CAPABILITY a plan adds, judged as if built.
No wiring state, no shipped/planned/dark status, no schema/SSB counts, no effort estimates. For
"is it wired / what's shipped", read the feature map and the wiring checkers, not this file.

Source of truth for the design: the MASTER phase index (Phases 0-4M) + the B..Q summary indexes.
This file is a topological map of the DESIGN, not a build ledger.

---

## The thesis

ELAI is a local-first, deterministic coding-and-reasoning agent whose product is not the model but
the HARNESS around it: a verification spine that makes an agent's claims checkable, reproducible, and
hard to fake. The model is treated as a replaceable cog — it generates and selects; the deterministic
harness sizes the work, owns the state, runs the gates, and decides "done" off artifacts the model did
not author. Everything composes toward one wedge: "trust-evidenced autonomous execution" — code (and
reasoning) you can audit and replay, on whatever model you point it at.

Three properties recur as the spine of every layer:
- **off-agent evidence** — a gate measures an artifact the generator did not write (an event row, a
  coverage hit, a hash-bound test outcome, a span), never the generator's own stdout.
- **fail-closed** — missing evidence is a FAIL, never a SKIP; the gate that judges a run is frozen and
  read-only to that run.
- **determinism** — a seeded clock + entropy source, an append-only journal, and a reproducible-run
  artifact make a whole session byte-recomputable.

---

# PART I — THE CORE ENGINE (Phases 0-4M)

The phased build is the actual ELAI system. The letter plans (Part II) extend, activate, harden, and
sell it.

## Phase 0 — Walking skeleton + determinism substrate

Forks a Rust coding-agent, strips known-bad code, and restructures the runtime into a 9-crate workspace
(core, providers, knowledge, reasoning, orchestrator, tools, plugins, rules, cli). Ships a one-model
one-FSM-loop one-tool base with msgpack session storage and a path-scoped visibility filter.
The load-bearing idea is **entropy quarantine**: all time and randomness flow through injected
`ClockProvider` + `EntropySource` traits, so a seeded run is byte-identical across executions — the
foundation every later "reproducible / replayable" claim rests on.

## Phase 1 — Knowledge / retrieval engine (`knowledge/`)

A hybrid code-retrieval stack, each layer a fallback for the last:
- **1A DKG** — tree-sitter parses source into a dependency knowledge graph in SQLite (symbols, typed
  edges: imports/contains/implements). Incremental content-hash updates.
- **1B BM25** — per-project tantivy full-text index, sub-10ms queries, precise per-file delete.
- **1C Vector search** — dual-model ONNX embeddings (code + text) with a per-model circuit breaker,
  content-hash vector cache, and graceful degradation to BM25-only on model failure.
- **1D PageRank repo map** — ranks symbols by importance and fits a token-budgeted repo map for context
  loading (symbol-level, density-guarded, task-personalized).
- **1E CatRAG + A-RAG** — assembles the full query pipeline (BM25 -> vector -> PageRank -> graph
  anchoring -> agentic snippet retrieval) for large token savings vs naive full-chunk retrieval.
- **1F/1G/1H** — speculative-decoding evaluation (dropped for Phase 1), a cross-encoder reranker, and a
  conditional SPLADE keyword-expansion evaluation.
- **1I Resilience** — per-model circuit breakers, a search-mode degradation cascade, query validation, a
  startup readiness canary, and staged query tracing.
- **3K RAG hardening** — AST-aware adaptive chunking, template multi-query expansion, HyDE, and
  tier routing (FAST/STANDARD/DEEP).

## Phase 2 — Reasoning engine (`reasoning/`)

The **Grounded Reasoning Chain (GRC)**: a budgeted state machine (Observe -> Hypothesize -> Plan -> Act
-> Verify) where every reasoning step must carry a typed grounding reference, classified into confidence
levels. Adds six binary **verification methods** (file/source/test/web/api/api-registry) that resolve
ungrounded claims, a plan-grounding sweep that blocks a phase transition on any ungrounded claim,
**per-agent GRC configuration** (each role gets its own budget, step sequence, allowed methods, and
anti-rubber-stamp rules), and **counterfactual safety questions** (CSQ) on functions touching
files/network/money/auth plus an inline feasibility check during execution.

## Phase 3 — Orchestration + resilience (`orchestrator/`, `rules/`)

- **3A FSM** — the full 9-state interactive machine (Idle/Planning/Feasibility/TestingFirst/Executing/
  Auditing/Memorizing/Complete/Configuring) with phase-aware tool/rule gating and evidence gates on
  every transition, each gate driven by a human-in-the-loop decision (approve/reject/modify/redirect)
  scaled by per-transition risk level.
- **3B SSB (Shared State Board)** — a single-writer append-only SQLite/WAL journal of typed inter-agent
  events (the observability + reproducibility backbone; every later "journaled row" lands here).
- **3C Agent system** — spawn / lifecycle / checkpoint / crash-recovery / Erlang-style supervision /
  anti-spiral (a content-hash loop detector).
- **3D BSP supersteps + channels** — deterministic macro-orchestration (barrier-synchronized parallel
  agents, typed channels, results applied in journal-sequence order).
- **3E Context budget allocator** — an 8-slot fixed budget that never exceeds the model window, with
  floors, deterministic cache breakpoints, and adaptive reduction.
- **3F Rules engine** — deterministic rule enforcement with rules stored as DATA (AST/grep/schema/metric/
  composite predicates), a proposed->shadow->active lifecycle with promotion gates, and self-discovery of
  recurring violations. Security/finance rules are immutable.
- **3G Hierarchical plan management** — a token-bounded plan skeleton plus per-phase elaboration, contract
  extraction between phases, and coherence checks.
- **3H Session handoff** — a DAG of session snapshots with structurally-lossless trimming (tool results
  stubbed, always recoverable from the journal) and branch/merge.
- **3I DST** — deterministic simulation testing that proves FSM/SSB/BSP correctness against an infallible
  reference model.
- **3J vLLM + advanced providers** — a production inference backend enabling layer-level optimizations
  (DoLa contrastive decoding, TurboQuant KV compression, ActAdd steering vectors) an opaque API cannot,
  plus the composed local inference sidecar (Anthropic-protocol) and the CLI local/hybrid mode.

## Phase 4 — Production hardening + conditional extensions

- **4A Agent process isolation** — agents run as isolated OS processes; the orchestrator is authoritative
  for GRC budget, inference, tools, and grounding, all proxied over a bounded 15-message IPC protocol.
  Agents hold no network, no LLM client, no tools.
- **4B Tools** — git ops, 5-tier validation, DAIRA trace markers, 3-tier edit matching, ShortCoder
  AST-equivalent code compression, a deterministic failure taxonomy, and async incremental compilation.
- **4C Plugins** — hooks intercept IPC invisibly to agents; tiered static analysis (T1 rules / T2
  tree-sitter / T3 pattern-match, no LLM in the hot path).
- **4D/4E Pipeline factory** — a Research Agent + Designer produce a typed PipelineSpec from a task via
  tree-structured grounded research (FlashResearch), then a deterministic builder assembles an executable
  pipeline with DAG repair and phase sizing. The **recursive decomposition** extension turns any idea
  into Concepts -> Phases -> Steps -> BabySteps, each level grounded, each babystep an 8-block spec, with a
  TreeNavigator as the primary execution artifact.
- **4F Feasibility probe** — system-resource probing + novel-step stubs in an isolated sandbox with
  branch-and-explore.
- **4G Autonomous mode** — an AutonomousGate wraps the interactive gate; it advances on deterministic
  evidence and delegates to a human only on high risk / missing evidence / escalation triggers, with
  task-splitting and model-substitution fallbacks.
- **4H Self-modification (DGM-derived)** — the system improves its own configs, rules, and data tables
  from measured pipeline outcomes, with a DGM-style variant archive + parent selection, a modification-
  solidification index, a closed set of mutable config keys, and four safety-enforcement layers. The
  modifying agent never evaluates its own modification.
- **4I Media validator** — validates non-code outputs (images/3D/audio/UI) via deterministic Tier-1 checks
  with veto power plus an advisory Tier-2 VLM, in a render-capture-evaluate loop.
- **4J Empire integration** — connects ELAI as a "brain" to an external ecosystem (conditional).
- **4K Dashboard** — a native Tauri desktop control surface polling the SSB viewer; not a chatbot, a
  structured control surface where every input routes through FSM classification.
- **4M Multi-runtime backend abstraction** — one workflow spec drives many runtime backends
  interchangeably (local sidecar, cloud API, and vendored coding-agent CLIs) via a single RuntimeAdapter
  trait, with per-role backend assignment and fallback chains.

---

# PART II — CAPABILITY LAYERS (the letter plans)

Each letter is a coherent capability wave over the core engine. The thesis of each is stated first;
the rest is what it adds.

## B — MVP completion + dashboard + knowledge extensions

Makes what the core engine builds actually run end-to-end and visibly. Adds: path unification and FSM/SSB
producer wiring so every engine emits real telemetry per query; the V2 dashboard (Tauri + Svelte, a 3D
orbit visualizer that IS the activity feed); live token-streaming chat with intent classification;
harness hardening (Phase-0 debt burndown, anti-stale enforcement, sandbox activation, hook/tool
unification); the **final-product features** — cloud mode (per-task local/cloud/hybrid routing with spend
ceilings), the **Focus Tunnel** (deterministic FIM/DIFF/STRUCTURED emission modes + streaming validation
+ automated dependency resolution for token efficiency), autonomous mode, self-modification, media
validation, and a **multi-runtime backend tier** (swap the local inference engine behind a closed
RuntimeAdapter trait, with a RuntimePanel surface — the "model is a replaceable cog" thesis made literal at
the runtime layer); and the **knowledge-engine extensions** — a symbol-level impact-analysis API (impact radius /
call flow / symbol context), multi-language DKG (13+ grammars), resumable indexing with cross-process
locks, adaptive minified-file chunking, an interactive symbol-graph viewer, entry-point detection, and a
per-project context-artifacts pipeline. Also the ambiguity-score continuous gate, the chat REFLECT
pipeline (per-message triage -> best-of-N reasoning -> auditor -> synthesize), the idea-framer +
per-level auditor modes, and the cross-project "Cosmos" artifact tier.

## C — Verification-gap closure + testing-trust + code-completeness

Closes B's verification gaps and proves each area with a real cloud smoke test. Adds: full SSB emission
across all producers; the dashboard command surface; cloud-mode hardening; Tier-C sandbox completeness;
the IPC dispatcher production swap; anti-staleness automation; the feasibility prober; the compiled-
pipeline + template crystallizer (cache a recurring pipeline shape); constrained decoding; and the
**testing-trust tiers** — determinism + a wiring contract (T-1/T-0), mutation testing (T-1), intent-pairs
+ metamorphic relations (T-2/T-3). The **code-completeness layer** enforces per-file line/function budgets
and a skeleton-first + mini-critic + accept-and-complete generation loop so weak models still produce
complete code. Later waves add paper-grounded improvements (tool-attention gating, mutation-operator
catalogs, zero-trust MCP fuzzing, credit attribution, parsimonious routing) and public-benchmark
publication harnesses (SWE-bench, Aider Polyglot, codebase-memory head-to-heads).

## D — Reasoning-improvement substrate + inference-time grounding

Extends ELAI from a verified system into a self-improving + grounded-reasoning one. Wave 1 builds the
typed mutation surface DGM-class self-improvement needs: a typed ReasoningTemplate archive with a
cryptographic hash chain, a process-reward-model sidecar, a multi-verifier consensus rule (promotion
needs ≥2 verifiers to concur), a five-tripwire regression stack, best-arm-identification promotion
(Thompson sampling), a time-windowed held-out task store, a cross-domain transfer canary, and L3
frozen-surface access control. Wave 2 builds the **five-mode inference-time grounding** taxonomy — an
orchestrator-owned mode router (the LLM has no input to selection), a constructive-checker registry
(sympy/z3/compile/registry), an operationalization predicate, a cross-family Adversary role, a sycophancy
probe, discovery-vs-search labeling, conformal refusal calibration, an honest-refusal SLA, and a typed
observable-effect predicate. Wave 3 pairs each capability with a quantitative benchmark + adversarial
probe.

## E — 55-report improvement sweep

A broad grounded sweep, each session one recommendation. Adds, across UX/memory: an infinite-context chat
illusion, local-LLM speed tricks, model-handoff UX (show role, hide model), sycophancy/persona-drift
detection, and a durable two-tier user memory. Across orchestration/reasoning: multi-agent failure-mode
classification, Darwinian self-improvement guardrails, self-evolving prompts, better eval benchmarks, and
an adversarial self-critique of ELAI's own design. Across product-grade codegen: idea-to-SaaS framing,
production fullstack checklists, TDD verification agents, browser E2E, and Firecracker sandboxing. Across
retrieval/inference: **the single highest-leverage fix — activating the ONNX retrieval stack and code-vec
path** — plus tool-use reliability, reasoning quality, local/cloud routing, quantization (AWQ+Marlin),
attention kernels (FA2), context-window lift, and MLX/llama.cpp adapters. Across trust/security:
prompt-injection defense, MCP hardening, symbol-graph tools, LSP-as-verifier, formal-verification codegen,
public-agent-incident defenses, cost/token hard ceilings, and eval-corpus design.

## F — Fail-closed write-gates (the "cant-lie" security spine)

The first plan whose done-gate is a real end-to-end trigger, not "compiles + tests pass". Adds a unified
fail-closed multi-axis in-process **write-gate** (code-vuln + hallucinated-dep + secret + malicious-skill,
blocking before disk), a CWE ruleset that produces the first measured pre-write insecure-rate, a
data-plane tool-filter defense against injected instructions in untrusted content, a **test-integrity +
output-grounding gate** (a claimed PASS is bound to an orchestrator-captured hash so a fabricated PASS is
detectable), cross-subsystem staleness quarantine, a learned-skill trust ladder that gates autonomy, a
safety-lattice completeness proof, a symbolic self-mod conflict detector, a write-time hallucinated-
dependency gate, and a per-iteration security-monotonicity gate (block if iteration N introduces a vuln
class absent at N-1). Later parts close the first config-only DGM generation under the guard trio and add
a local deterministic NLI contradiction pre-filter.

## G — Trust-maximization

Turns the security spine into a distributable, reproducible trust product; every unit consumes an
F-gate's verdict rather than restating it. Adds: **harness-wide taint propagation** (untrusted-content ->
destructive-sink flows blocked deterministically), the **Reproducible-Run Artifact (RRA)** — a single
portable file bundling the action stream + seed + hardware pin + every gate's verdict hash, with
`elai replay --verify`; deterministic runtime blast-radius gates (post-expansion FS gate + egress IP-pin);
a live MCP rug-pull re-pin gate; a **reviewer cockpit** with a diff-surface-to-review metric (skim the
vouched hunks); per-change confidence scoring + a Dual-LLM lethal-trifecta quarantine; **SAFEDELTA** — a
gate-ON-vs-OFF ablation measuring the residual insecure diffs that reach a reviewer, vs competitor
harnesses; signed in-toto provenance attestation with an offline verifier; a write-time secret/PII
redaction gate; and a model-drift regression canary that watches the exogenous model underneath.

## H — Efficiency + canvas + trust-hardening + grounded-reasoning + reversibility + context-lifecycle

The largest and most multi-track letter.
- **Efficiency** — makes ELAI provably cheaper on the SAME cloud model: prompt caching on the production
  path, cache-stable layout + 1h TTL, an output-token stack (diff/FIM emission + streaming early-abort,
  the 5x-billed lever), the Tool Search tool, the Batch API for non-interactive lanes, deterministic
  observation masking, calibrated architect/editor/verifier routing, a persisted oracle cache, and a cost
  cockpit dashboard.
- **Canvas-workbench** — replaces the watch-only 3D orbit with a 2D generative canvas the model DRIVES and
  the user points at: a read-only code-graph, a live SSB overlay + file explorer, PLAN/RESEARCH modes,
  model-driven canvas ops, an app/website design mode with click-to-edit, a general-purpose visual reply
  channel with taint-gated deixis, and retirement of the orbit. The moat is the journaled + reproducible +
  taint-tagged op-stream underneath.
- **Trust-hardening (autonomous trading)** — makes an autonomous strategy-search loop trustable by
  deterministic gates: a persisted un-resettable trial ledger, deflated-Sharpe/PBO/MinBTL promotion,
  reusable-holdout budgets, leakage static + runtime look-ahead gates, trading metamorphic null-tests,
  same-seed re-run hash-equality, sim-to-real parity, a result manifest, shadow execution + financial
  invariants, gate self-validation (verify-the-verifiers), a reward-tampering gate, and a fail-closed
  drift interlock.
- **UX + user-memory** — a staleness/intent-drift instrument, a two-tier user memory (bi-temporal
  preferences + code-grounded validate-before-use facts) with an auditor-objectivity firewall, a memory
  governance surface, MCP defer-loading, /rewind over the journal, dev-event notifications, a plan-approval
  gate, and a cost-confirmation gate on stop.
- **Grounded-reasoning hardening** — abstention-aware promotion (credit the honest stop), decorrelation-
  keyed verifier retention, a live checker-coverage instrument, and serial-depth-scaled reasoning budgets.
- **DGM-verification hardening** — replaces two placeholder filters with real ones: execution-grounded
  fitness (not a byte hash), behavioral trap detection (not a keyword grep), worst-case + lineage-diverse
  promotion scoring, clade-metaproductivity parent selection, a DGM eval firewall, and a bandit on the real
  reward. The spine theorem: a self-improvement loop without an un-gameable stronger-than-generator filter
  provably collapses; with one, it converges.
- **Reversibility** — undo/redo/branching file time-travel + a lossless paged transcript.
- **Context lifecycle** — regime-aware in-session context management (local window-minimization vs cloud
  cache-stable) + cross-session sleep-time handoff with a grounded topic-digest.
- **Review-anchoring + external steals** — deterministic snippet->hunk finding-anchoring (a hallucinated
  line number is structurally impossible), citation pre-binding on the answer path (ghost citations
  impossible), and a deterministic obligation-closure escape-hatch gate for the formal-proof lane.

## I — Fault-isolation + evidence-ranked search + cache economics + anti-gaming + routing + flywheel

The first letter authored after the wiring bedrock was reached; every session is feature engineering.
- **Fault-isolation** — a process-separated directive Supervisor (the only executor of side effects; the
  generator emits pure Intent structs and cannot reach gate code), copy-on-write workspace isolation for
  parallel branches, and hardened journal throughput + atomic checkpoints.
- **Evidence-ranked search** — the keystone: a best-of-N -> tournament -> tree search ladder that selects
  by RECEIPTS, not by an LLM judge, with a pristine-context injector, an anchor-pane slot, cascading
  local->cloud escalation, a voluntary-abort tool with dead-end hashing, and error-amplification telemetry.
- **Cache-invariant economics middleware** — byte-frozen prefix discipline + TTL tiering + a cache-hit
  hard guard, a typed belief-state handoff, an evidence-gated playbook curator, a memory guard + dual
  serialization, deterministic context compression, a semantic intermediate-summary cache, and a KV-cache/
  quant evaluation.
- **Deterministic gates + anti-gaming verification** — a tri-layer evaluation stack (hard veto ->
  calibrated scorer -> adversary), a tamper-evident hash chain + attestation + AI-BOM, a TDD preflight, a
  budgeted mutation scheduler (confidence-interval scores that fit inside the loop), sprint contracts,
  progress-shaped rewards, unified conformal calibration, and isomorphic anti-gaming canaries.
- **Routing + budgets + measurement** — a ledger-trained router (trained on the journal's own verified
  outcomes), dynamic thinking budgets, a budgeted-convergence instrument, DAG credit assignment, and a
  diffusion draft lane behind the full veto stack.
- **Open-weight flywheel** — verifier-ranked SFT exhaust from sealed runs, and a GRPO flywheel on
  execution-grounded rewards behind blocking safety guards.
- **Deterministic-over-prompt hardening** — structural role scoping, mechanical decision governance,
  grammar pinning, temporal tool-sequence enforcement, cross-model prompt compilation, EARS requirement
  objects, QDELTA + a statistical floor, plan-state derived from the repo (kill hand-maintained drift), and
  the **agent-agnostic trust-layer wrapper** — run any external agent's tool traffic through ELAI's taint
  + write-gates + RRA emission (the wedge unbundled). Later tracks add design-phase critique lenses, a
  critique-only operating profile, a Phase-0 kill-gate falsifier, a zero-retention export mode, and
  competitor-steal ports (objective-hash sealed receipts, a bash interceptor, verifier-input hardening,
  provider protocol lanes, an auth broker, a unified model catalog, a checkpoint tool, recipe runners, and
  a structural AST edit tool).

## J — Harness-redesign residue (the grab-bag; the largest plan)

The first letter whose center of mass is the HARNESS (`.claude/` + workflow tooling), plus a very large
tail of research-steal and idea-driven waves. Highlights of what it adds:
- **Perception + document** — a deterministic image-ingestion gateway + perception scaffold (condition
  bytes before any model sees them; counts from AXTree/DOM never a VLM), a hardware-detection + per-lane
  model eligibility catalog, and a MinerU document-parsing lane.
- **Control + front-door** — ELAI-as-an-MCP-server control plane, a total-function front-door router
  (chat is ELAI, not a model; every message resolves to a lane or a clarify chip), an intent-control layer
  (verify AIM not just fidelity — surface a mis-aimed premise before decomposition), and harness safety
  (per-session file backups + undo, a handoff-file ledger, an incremental-eval skill).
- **Voice front door** — the front door spoken: a transducer pair over the same spine, never a new command
  surface. Inbound, local streaming ASR lands utterances on the front-door router exactly like typed chat —
  taint-tagged as untrusted acoustic ingress, journaled with transcript + audio-hash + ASR-model pin before
  routing (a voice run replays like a typed one), addressed to sessions by phonetically-distinct call-signs,
  with mutating lanes confirmed by a spoken readback built from the ROUTED typed intent (never a transcript
  echo — the mis-transcription catch) and high-risk never voice-only. Outbound, a deterministic narration
  plane: every narratable event family declares a spoken render of its journal row (template + seeded variants
  + local TTS; no model in the truth path, so the narrator structurally cannot lie), dispatched through the
  notification dispatcher with priority preemption, step-grain rate limits, and silent/summary/verbose levels;
  the one model slot — on-demand cross-session status digests — carries row-id receipts and a deterministic
  row-count footer appended outside the model text, so a drifted summary contradicts itself audibly. One
  supervisor voice: the user talks to the harness, and the harness talks back off its own journal.
- **Token + provider economics** — a unified token ledger (per-turn, per-lane, honest per-model pricing),
  a usage window + live meter, BYOK + custom endpoints, a free-API rotation pool, and a model-strength
  modality router (CodeAct on capable lanes, constrained minimal-schema tool-calls on the weak local lane).
- **Determinism gates (NASA-grade)** — a power-of-ten generated-code gate, an SSB temporal-monitor engine,
  multi-mode gate actions (suppress/insert/edit, all journaled), formal verification of ELAI's own critical
  core (Kani + TLA+), an SSB hash chain + watermark monotonicity, gate-manifest traceability + IV&V
  independence, and a policy-as-code spike.
- **Determinism doctrine propagation** — the "model is a cog" doctrine made structurally enforced at every
  lifecycle seam (a thin frozen floor + a cross-family advisory + a fail-closed debt ledger + a measured
  rate + a program falsifier), and the DD triad + reasoning axis: deterministic node-admission sizing,
  extractive research compression, a cheap three-tier grounding stack, and within-step distill-reset
  reasoning.
- **Context-retrieval** — in-domain retriever adaptation (adapt ONE retriever on ELAI's own supervision),
  a retrieval-abstention gate (don't retrieve when local context suffices), graph-as-a-retrieval-channel +
  mid-task retrieval action, an RRF ablation harness, and context-rot calibration.
- **GRC-v3 reasoning engine** — a typed belief-state reasoning carrier with a per-step external-receipt
  gate (a step commits only if its receipt passes, else abstains), a 2-axis effort classifier, a
  deterministic overthinking guard, and a receipt-gated candidate ladder (receipt > vote over independent
  samples > never LLM self-eval).
- **Lacuna-fill** — a journal-completeness substrate (prove every side-effect produced a row, not just
  that no row was altered), runtime-observed DKG edges, an objective + plan-DAG prevention gate,
  cross-language + dynamic blast cones, a re-executable RRA, and the seam-composition wiring fills.
- **Vision-orchestration + memory** — perception faithfulness routed through the trust spine (a ghost
  region is a ghost citation), a declared context-access spec per dispatch, signal-triggered grill offers,
  worktree-pool lifecycle, autonomous-supervision hardening, deterministic token-efficient emission, per-seat
  memory + trust isolation, and a **verified case graph** — mint a typed case node only on a proven fix,
  self-wire deterministic edges, and (behind a strict execution-grounded filter, default-off) reuse proven
  cases so ELAI gets cheaper on YOUR recurring bugs.
- **Read-access + time** — a read-access escalation ladder (read the cheapest rung that proves it has what
  it asked for), a live-index overlay (never retrieve a symbol you just deleted), counterfactual memory-ROI
  attribution, an output-side act-don't-narrate gate, a two-plane time model (time-as-context vs
  time-as-control), transformation-surviving taint binding, a config-ingress admission gate, and a
  recursive-reasoning DEPTH operator (deterministic multi-hop traversal, abstain rather than fabricate).

## K — Prove / distribute / package the wedge

Adds almost no new capability; it PROVES the spine sits on the externally-demanded axis, DISTRIBUTES it,
and makes it affordable. Adds: a problem-quantification dossier + a claims-policy lint (no external claim
without its measured number or an explicit tag), a routing-economics bench (the first production hybrid
$/verified-task number), a **PR/CI evidence emitter** (render deterministic, hash-bound, replayable
evidence into GitHub check-runs — the distribution leg), a composed gate-tax budget + a risk-tiered gating
policy (the affordability gate the prior generations failed), a compliance export + an appsec/insurance
brief, an absorbable-unit audit + provider-check pre-gate, a model-refresh cadence, an OWASP control
mapping, a time-determinism doctrine, a reasoning-budget bench, a context-management measurement, a
case-reuse economics bench, a re-executable-RRA differentiator, journal-completeness + doctrine-coverage
doctrines, and — the two net-new-capability exceptions — **multi-repo/workspace memory** (a cross-repo
graph + federated retrieval + cross-repo blast radius, connected vs disconnected honestly) and
**credit-based seats on an OpenRouter rail** (a hash-chained credit ledger, metered reserve/settle,
sold as "more value per dollar, verified", never "cheap"), plus a continuous absorption watchdog.

## L — Causal grounding gate + output-quality + context-core + exploratory reasoning

The first plan that points the trust spine OUTWARD — from verifying the agent's own code to verifying ANY
model's CLAIMS on any backend. Adds a typed claim-extraction substrate, a unified **ClaimRisk** score with
a convergence fuse (a high verdict needs ≥2 mechanistically-distinct predictors), an evidence round-trip
predictor, a semantic-entropy inter-seed predictor, a **counterfactual do-test on claims** (run the
intervention and observe whether the claimed effect changes — only an executable pass labels a claim
causal), a risk-as-curiosity targeted-effort router, a hallucination policy gate, working-memory hygiene,
a research/math deep-think profile, and a hallucination-delta bench. Also: an **output-quality enforcement
system** (SOLID/maintainability via a 4-seam layered gate), a **familiarity surface** (user-authorable
rules/hooks/skills + one-shot verified builder, safer than the tools they imitate), a **context-core
inversion** (an immutable transcript + a pure fold-view projection, making cache-stability structural), a
measured-competitor eval-supply harness + a weak-lane verification-intensity policy, and an
**exploratory-reasoning mode** — claimant-authored falsifiers + a conjecture-criticism map of rival
explanations on the local lane, backed by a **conjecture-memory middle tier** between the append-only
journal and the verified case graph: unproven conjectures preserved as first-class retrievable objects
with a computed status (live / refuted / pending / suspended — a pure journal fold; refuting a premise
suspends dependents rather than refuting them, and no status is absorbing), provenance, attached
criticisms carrying evaluator-version edges, and declared missing tests — explore-lane only, structurally
unretrievable into the proven-only code-fix path, so memory can hold ideas ahead of the current receipt
plane without contaminating the trust spine. Refusals gain a typed unblock condition (what was tried,
what evidence would change the answer, composed from the receipt trail), the explore bench gains a
meaning-cluster novelty instrument under the grounding-gate ablation, and the open-weight flywheel's
training exhaust gains a diversity guard — train on moves not survivors, labels filtered by the
exogenous-grounding floor, every dataset row bound to the evaluator version that vouched for it.

Planning-harness boundary: the improvement-conjecture tool is not a new runtime reasoning organ. It
keeps a separate append-only artifact archive, computes status from edges, reports archive-relative
equivalence beside groundedness/value/cost, and lets callers choose a declared `SAFE`,
`NOVEL-CREDIBLE`, or visibly-speculative upside profile. ELAI's runtime design deliberately contains
no FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, or bridge from
this planning archive into L-23/L-24; those mechanisms failed their evidence gates and are outside
the specification.

A separate Q-10 research-coupling plan does not reopen that bridge. It proposes a materially different,
one-way input: verified research may be compressed into a typed relational packet containing source
mechanisms, object mappings, shared relations, invariants, disanalogies, contradictions, and provenance.
The packet is always `SOURCE_ONLY` evidence — every source mechanism/relation-source-predicate is support-edge-bound, and it
may inspire a transfer but cannot verify the target claim. Target anchors, falsifiers, grounding receipts,
and execution remain independent, and direct, source-blind matched-scaffold, verified-evidence, and
relation-shuffled controls must lose under locked executable gates before the interface may
reach planning skills or, later, the explicit explore lane.

## M — Commercial multi-seat

Makes the sellable single seat sellable to a company, offline-first, without becoming an org SaaS. Adds:
signed offline-verifiable seat entitlement + a purchase rail (commerce fails OPEN to a free tier; safety
gates never brick), an org policy pack (monotone-tighten, distributed as a signed file), a fleet evidence
rollup (org-grain answers computed from seat journals, zero telemetry), a signed release channel +
auto-updater + support bundle, commercial packaging + a standalone OSS verifier + trust-continuity, and an
**evidence format as a versioned open standard** so third-party tools can emit ELAI-verifiable evidence.

## N — Interlace / anti-fragmentation

Invents almost nothing; CONNECTS everything. Because prior letters were authored against summaries,
same-concern machinery fragmented (four claim types, ten routers, six intensity deciders, five provenance
scales). N declares ONE canonical subsystem per fragmented concern and mechanizes the anti-fork rule: a
measurement spine as one subsystem, a corpus-grain composition validator (the first machine check that
plans talk to each other — it runs at plan-authoring time), generalized anti-selection-bias promotion
primitives, one Claim/Belief object + a receipt memo, a policy/intensity arbitration plane, a provenance/
trust lattice + admission-gate family, human-render as a required verdict field, a governance-coverage
substrate, one counterfactual/ablation engine, clarity/novelty->budget wires, a journal-fold substrate,
and a consistency + dedupe bundle. External steals add an output-boundary registry, self-improvement
edit-loop instruments, anti-affirmation stability measurement, and governance closure (evaluator-refutation
verdict-collapse + an exogenous-grounding floor).

## O — Personal research machine

A TEIKOKU-style personal research machine on the ELAI canvas, ~80% composition, differentiated by being
gated + journaled + replayable + closed-vocabulary. Adds: a research daemon (generalize Cosmos to a
user-editable typed topic set with taint + replay), a scheduled TaskSpec substrate (typed cron/event
schedules -> inbox, "n8n with receipts"), a brain-vault contract + mounting (mount an existing markdown
vault, namespace-claimed so two harnesses share one vault), and a canvas WORKFLOW mode over the TaskSpecs.

## P — Personal software incarnation

Lifts the trust spine one grain: changeset->app, task->fleet, seat->world. The way a person builds their
own WORLD of connected bespoke tools, everything protected/organized/committed. Adds: an app-fleet layer
(the user's N bespoke apps as first-class objects with health-tick + gated auto-repair), inter-tool ports
+ a world-grain typed event bus, a persona/mechanism split + a hand-this-box-to-a-friend export, living
external brains (mount live Obsidian/Notion under a change-feed contract), a non-coder operator surface +
graduated ceremony (the trust spine scales DOWN — a toy runs gates observe-only), a life-data privacy
lattice (taint's mirror: owned bytes flowing OUT), a per-user operating profile (adapt PROCESS not just
content, asymmetrically — tighten free, loosen only on measured evidence), and a small-team collaboration
grain (2-10 humans + agents in one world, with a lossless deterministic two-seat journal merge).

Then a command-center distribution wave opens that world onto an unbounded connector surface without
forfeiting receipts — own the FORMAT not the ecosystem, applied to CAPABILITY. Adds: a signed, versioned
connector/stage-PACK format (one generic `http_call` stage parameterized by a declarative, admission-gated
`ConnectorSpec`, so ~80% of connectors are a data-diff of the first pack not a build; genuinely-new
behavior takes the closed code-pack lane), an ingress-adapter family that normalizes any external shape
(inbound webhook push, streaming/persistent listener, outbound pull, or genuinely-other device) into one
taint-tagged journaled SsbEvent — so the trigger enum never grows and every downstream trigger / replay /
provenance / privacy mechanism works unchanged — a first-class credential vault giving that connector
surface an auth plane instead of scattered tokens, and workflow-from-goal: an intent synthesized into a
closed-enum-constrained TaskSpec graph drawn ONLY from the existing typed stage/connector vocabulary,
rendered on the canvas with any gap shown visibly disconnected rather than silently invented, and
human-confirmed before anything runs (the n8n-user front door where every proposed wire stays typed,
gated, and replayable).

A computer-use wave then lifts the same spine onto the desktop/GUI grain the browser-E2E and media-validation
lanes never reached. Adds: a desktop/GUI operator lane that actuates inside a disposable sandbox where every
coordinate action is bound to the exact observation it was computed from (a content hash of the delivered image,
invalidated the moment that image leaves context, never retargeting to another display) and is journaled as a
taint-tagged operator lane a workflow's correctness never rests on; a perception layer where the model points by
integer id off a source-tagged element list rather than raw coordinates, each element carrying its provenance
(accessibility-exact, OCR, caption, or pixel) as the trust boundary, with pixel perception the fallback only where
no accessibility tree exists and counts never coming from a vision model; an explore-then-crystallize lane where an
agentic exploration run mints a deterministic replayable script bound by a reproducible-run receipt to the run that
produced it, so replay becomes the cheap zero-reasoning execution mode; a verified-case-graph reuse loop
(retrieve/distill/fuse) whose save is gated on a proven fix and whose tie-break judge scores journal rows, not
screenshots; and a gated install catalog that turns capability-on-demand ("download this app") into a named,
admission-gated catalog resolving to headless connector packs, with GUI driving the last resort rather than the
mechanism. The same wave adds a journaled co-browse surface: an embedded browse pane where user and model read and
click through any site together, rendered as dumb glass over an engine-owned browser so that every navigation, click,
and keystroke — from either actor — is journaled before it is dispatched and an unjournaled input path is structurally
impossible; session persistence is a typed choice (ephemeral by default, named profiles held in credential-vault
custody), outbound typed text passes the life-data privacy lattice before it leaves, and page content enters as
tainted ingress that can never authorize a mutating lane — a co-browse pane with receipts, where the field ships the
pane without them. And it extends the design/test surface to the smartphone grain as a typed dimension rather than a
parallel subsystem: a device-profile catalog renders any app preview or browse pane in a phone frame and runs the
browser-E2E journeys across a device matrix; React-Native apps enter the click-to-edit design mode through their web
render, labeled as such; and a native emulator becomes just another sandboxed operator backend — booted from a pinned
snapshot whose hash lands in the reproducible-run pin, perceived through the platform accessibility tree, its
explorations crystallized into actuation-only replay flows whose verdicts still come from the off-agent evidence leg.
Every mobile result carries its render source, so a cheap rung can never quietly discharge a native gate — "works on
the phone" becomes a typed claim.

An orchestration-mechanics wave hardens the boring correctness a trust product cannot fake: provider failover with
failure-class cooldown lanes, session-stickiness for cache warmth, strict honouring of an explicit model choice, and
billed truth kept typed-separate from estimate; durable workflow runs (revision-counter concurrency, sticky cancel)
where every approval binds a canonical hashed plan and its operands and denies on any drift between grant and
execution; a config/ops lifecycle that fails closed when a policy object is absent, preserves rejected writes,
allowlists environment, and ships an operator doctor with stable exit codes plus verified backups of the journal
that is the product; a context/memory lifecycle where a pre-compaction flush guarantees nothing is destroyed that
was never offered to durable memory and a versioned on-demand index keeps skills and rules out of the prompt until
needed; a gate-credibility layer that plants known-bad canaries through the real detectors so a checker that has
silently stopped detecting fails the whole gate, and makes redaction structural (the shareable audit projection has
no field a secret could live in, kept separate by type from the content-carrying debug bundle); and control-plane
hardening where a subagent's return is fenced as inert evidence rather than trusted instruction, tool authority is
depth-scoped, device pairing is challenge-signed, and an unrecognised event family is withheld rather than leaked.

## Q — Engineering-taste hardening + orchestration economics + research-conditioned creativity + research-instrument economics + autoresearch synergy

The capabilities that make the deterministic harness cheaper to run and harder to let rot — the trust
spine turned on cost and on itself, rather than a new user surface. Adds: standing goals (a completed
unit's own off-agent verification becomes a persistent scheduled invariant, re-checked forever, so a
once-passing gate that later rots re-enters the work pipeline instead of silently lying — a goal verified
once is only an assumption with a timestamp), a quorum wake-gate (cheap cross-family voters decide whether
the expensive lane wakes on a scheduled tick, so a quiet tick costs almost nothing while a false vote only
ever delays a wake, never suppresses a verified result), an advisor lane (the cheap model executes every
turn and consults the flagship as a budgeted tool about once per task when judgment is needed, the same
executor continuing after the advice, priced honestly against the same-lane baseline), a swap-restart rule
(a mid-unit model substitution restarts the unit fresh because iteration state is model-conditioned, and
records the discarded work, keeping per-model credit attribution clean), per-turn local token-hashing (each
local turn's emitted tokens are hashed so a replay localizes divergence to the first divergent turn and the
reproducibility claim is scoped exactly to the layer that earns it — harness stream anywhere, local tokens
same-class, cloud action-stream), a gate-manifest-wide evidence-corruption corpus (every frozen checker is
proven to fail-closed against truncated, corrupt, missing, or flipped evidence, so "missing evidence is a
failure" becomes a measured per-gate kill-rate rather than a promise — testing concentrated on the product,
which is the gates), a weak-lane code-style bench (measure whether structuring code for weaker models
actually raises their task-success, and promote a style rule into the output-quality gate only on a measured
win), and a locked size-and-dependency ceiling on the standalone verifier (the tool that audits every run
stays smaller and more dependency-light than what it audits). A distribution-hygiene coda keeps the vendored
external runtimes cheaply mergeable against their fast-moving upstreams and derives every surface — dashboard,
CLI, canvas, control-plane — from one command registry so clients cannot drift. A fourth, benchmark-first
track couples the research system to creativity without coupling their authority: verified research becomes a
source-scoped relational inspiration packet, candidates expose their transfer edges, and separate target-side
grounding/execution decides whether the analogy worked. The packet reaches paper-to-plan/idea-intake only after
it beats direct, matched-scaffold, evidence-only, and relation-shuffled controls on real ELAI fixtures; it reaches `/reason --profile
explore` only after a further observe-only L/N ablation improves receipt-valid meaning-cluster coverage and best
verified outcome without harming PS-3, the exogenous-grounding floor, cost, or the proven/unproven lane wall. A fifth,
instrument-before-improvement track turns that same discipline on the research lane's own measuring apparatus before it
touches search: the novelty classifier proves itself live on every run through a known-answer canary that fail-closes the
verdict the moment it can no longer tell same from distinct (so a silently-dead safeguard screams instead of shipping dead
labels hidden by aggregate scores), the explore bench reports value per dollar at a locked spend cap with named control
arms so a policy that merely spends more is flagged rather than promoted, and a frozen pre-change baseline makes every later
delta attributable. Only then is search itself made cheaper and smarter — each research operator receives only its minimal
context and the saved budget buys more iterations, and the freed search is allocated across draft lineages by a bandit that
forks a new strategic lineage from the best findings when one stalls, adopted only if it beats a plain deterministic
scheduler at equal budget rather than by spending more. And every compiled briefing line is bound to a resolvable receipt so
a line that cannot prove its provenance is dropped rather than asserted, closing the gap between a receipt-bound answer and an
unbound briefing. A sixth track turns that same measuring discipline inward — onto the machinery that picks ELAI's own
improvements and the bench that scores its research. The idea-selection engine is instrumented like any other measuring surface:
its novelty leg proves itself live on every run through a known-answer canary that fail-closes the moment it can no longer tell
same from distinct rather than quietly degrading to a fallback that cannot separate candidates at all, every refutation records
how far the loser lost so a within-noise kill stays honestly reinstatable rather than indistinguishable from a decisive one, the
selector breaks ties toward the cheaper candidate on measured cost, and a survival claim that adjacent evidence half-contradicts
carries the contradicting citation instead of being silently re-litigated. The research bench and its protocols stop discarding
specialists the aggregate hides — a per-task pareto retention policy keeps a candidate that wins some task even when its total is
dominated — require a refutation to prove it had the statistical power to refute so an underpowered failure is recorded as
inconclusive rather than disproven, stamp every bench fixture untrusted until a human baselines it, unify variance behind one
repeats-aware helper so cost-normalized comparisons line up across benches, and make a handoff resume record where its notes
disagreed with the live tree so drift becomes measurable rather than silently re-verified away.

---

# The shape of the whole

Read top to bottom, the corpus is one argument:
1. **Phases 0-4** build a deterministic, hybrid-retrieval, grounded-reasoning coding agent with process
   isolation and self-modification.
2. **B-C** make it run, verify it end-to-end, and add testing-trust + code-completeness.
3. **D-E** add self-improvement substrate, inference-time grounding, and a broad efficiency/security sweep.
4. **F-G** build the fail-closed security spine and turn it into a reproducible, distributable trust
   product (RRA, SAFEDELTA, attestation, reviewer cockpit).
5. **H-I** add efficiency, the generative canvas, autonomous-trading trust, evidence-ranked search, cache
   economics, anti-gaming verification, and the open-weight flywheel.
6. **J** is the harness redesign + a very large research-steal tail: perception, control plane, determinism
   gates, GRC-v3, the verified case graph, and lacuna-fills.
7. **K-M** prove the wedge, package it, and sell it (single seat -> company -> open evidence standard).
8. **N** connects the fragmented pieces into canonical subsystems.
9. **O-P** lift the whole spine to the personal-research and personal-world grain, and P's command-center
   wave opens that world to an unbounded connector surface while keeping every wire typed, gated, and replayable;
   P's computer-use wave then lifts the spine onto the desktop/GUI grain (a sandboxed operator lane, provenance-
   tagged perception, explore-then-crystallize replay), and its orchestration-mechanics wave hardens the boring
   durable-execution / failover / ops / gate-credibility correctness a trust product cannot fake — the two things
   the field's shipped agent infrastructure has and ELAI's corpus specced thinly, absorbed without forfeiting receipts.
10. **Q** turns the spine on cost and on itself: standing goals, a wake-gate and advisor lane and swap-restart
    rule for cheap-but-honest orchestration, per-turn replay hashing, a measured fail-closed kill-rate for every
    gate, a weak-model code-style bench, a size-capped standalone verifier, and a benchmark-gated research-to-
    explore interface whose source-scoped analogy packets must earn separate target receipts — hardening and
    measurement, not unsupported surface or citation authority.

The invariant across all of it: the model generates and selects; the deterministic harness sizes, gates,
journals, and decides "done" off evidence the model did not author — measured, fail-closed, replayable.
