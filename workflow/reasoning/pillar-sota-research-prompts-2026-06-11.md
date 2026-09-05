pillar SOTA research prompts, 2026-06-11
purpose: 8 standalone deep-research prompts (Perplexity deep research or GPT deep research), one per ELAI pillar. each is fully self-contained, paste-ready, and deliberately contains NO project-identifying details (no project name, no internal codenames, no file paths) per the external-prompt-anonymity rule. expected yield: ~50 papers/codebases total across all reports. triage pipeline for the returned reports: workflow/handoff/pillar-sota-recheck.md.

================================================================================
PROMPT 0 - TRUSTED RUN JOURNAL (already delivered to user in-chat 2026-06-11; repeated for completeness)
================================================================================

I'm building an autonomous AI coding-agent system in Rust and need to stress-test a core
architectural choice. Please do deep research (papers, GitHub codebases, industry systems,
2024-2026 preferred) on whether anything better exists.

WHAT I HAVE TODAY - a "trusted run journal" with these properties:
- single-writer, append-only SQLite database (WAL mode) recording every agent action, state
  transition, test verdict, gate decision, and spend as a typed, timestamped row (~80K inserts/sec)
- the event-type schema is frozen by a "golden lock" test: adding/renaming any event type without
  updating a locked manifest breaks the build, so the vocabulary can't drift silently
- runs can be exported as a sealed bundle: the full event stream + random seed + model pin +
  verdict hashes, with a blake3 root hash, replayable and verifiable offline
- the journal is the load-bearing evidence for anti-fabrication checks (an agent cannot report
  "tests passed" unless the orchestrator-captured output hash matches) and for reproducible
  benchmark claims

THE QUESTION: is there a meaningfully better design, library, or standard for this job -
a tamper-evident, append-only, high-throughput, locally-embedded event journal that serves as
cryptographic evidence of what an AI agent actually did?

Please cover, with links, licenses, maturity signals (stars/adoption/last release), and an honest
verdict per item:
1. Tamper-evident log structures: Merkle-tree / hash-chain transparency logs (e.g. Google
   Trillian, Sigstore Rekor, certificate-transparency-style logs), and whether any are practical
   EMBEDDED in a desktop app rather than as a server.
2. Event-sourcing / append-only embedded stores in Rust or embeddable from Rust (e.g. anything
   newer/better than SQLite-WAL single-writer for this: redb, fjall, sled successors, NATS
   JetStream embedded, EventStoreDB alternatives) - specifically for crash-safety + append-only
   enforcement at the storage layer, not just by convention.
3. AI-agent-specific work 2024-2026: papers or systems on verifiable/auditable agent execution,
   "attested" or "provable" agent runs, deterministic replay of agent trajectories, agent
   flight-recorders. Include arXiv papers and any production systems.
4. Provenance/attestation standards that could strengthen the sealed-bundle story: in-toto, SLSA,
   C2PA, W3C PROV, sigstore - which (if any) are worth adopting for signing/verifying agent-run
   bundles offline?
5. Schema-freeze techniques: better alternatives to a hand-rolled "golden lock" test for keeping
   an event vocabulary stable (e.g. buf breaking-change detection for protobuf, schema registries,
   typed event-sourcing frameworks).
6. Known weaknesses of my current design I should worry about: single-writer SQLite limits,
   hash-chain vs Merkle-tree trade-offs (inclusion proofs, partial verification), what happens
   if the writer process itself is compromised, and how others mitigate that.

End with a clear recommendation: KEEP the current design as-is, HARDEN it with specific named
additions, or REPLACE a component with a specific named alternative - and justify with evidence,
not vibes. Flag anything you couldn't verify. Prefer primary sources over blog posts.

================================================================================
PROMPT 1 - CODE RETRIEVAL
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: how the agent retrieves knowledge of the repository it edits. Papers (arXiv
2024-2026), GitHub codebases (stars/license/last release), and production systems. Actively
attack my design - list its documented failure modes and what others do about them.

WHAT I HAVE TODAY:
- a tree-sitter-parsed symbol dependency graph stored in SQLite (nodes = symbols, typed edges =
  imports/contains/implements), incrementally updated via content hashing, 18 language grammars
- a BM25 lexical index (tantivy) over 50-line sliding windows
- dual embedding models run locally via ONNX (one code-specialized ~768d, one text ~384d) with
  HNSW vector search and per-model circuit breakers
- a PageRank-style repo map that ranks symbols for context inclusion under a token budget
- reciprocal-rank-fusion across the retrievers + a CPU cross-encoder reranker (<50ms / 50
  candidates), AST-adaptive chunking, multi-query expansion, HyDE
- graceful degradation: missing embedding model -> BM25-only mode, never a crash

THE QUESTION: what is the 2025-2026 state of the art for code-aware retrieval in coding agents,
and does anything meaningfully beat this stack? Specifically:
1. compiler-grade indexes (SCIP, LSIF, rust-analyzer/LSP-backed) vs tree-sitter graphs - who has
   shipped agent retrieval on them and with what measured gains?
2. graph-RAG for code (2024-2026 papers/implementations) - real gains over RRF fusion or hype?
3. newer code embedding models (2025-2026) that materially beat GraphCodeBERT-era models at
   similar local-inference cost; ONNX/CPU-servable preferred.
4. "agentic retrieval" - agents that grep/navigate on demand instead of querying a prebuilt
   index (Claude Code-style); the evidence on when each approach wins, esp. for large repos.
5. repo-map/context-selection algorithms newer than PageRank-over-symbols.

FORMAT: (a) 3-6 strongest alternatives with one-line each + links + license + maturity;
(b) documented failure modes of MY stack; (c) verdict: KEEP / HARDEN (named additions) /
REPLACE (named alternative), evidence-justified; (d) top 3 highest-value specific changes,
ranked by evidence strength. Primary sources over blog posts. Flag anything unverified.

================================================================================
PROMPT 2 - DETERMINISTIC ORCHESTRATION
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: multi-agent orchestration. Papers (arXiv 2024-2026), GitHub codebases
(stars/license/last release), and production systems. Actively attack my design - list its
documented failure modes and what others do about them.

WHAT I HAVE TODAY:
- a deterministic finite-state machine drives every task (planning -> feasibility-probe ->
  test-first -> executing -> auditing -> memorizing), with evidence-gated transitions (a
  transition needs a concrete artifact, not model say-so)
- multi-agent work scheduled in BSP-style supersteps: deterministic barriers, writes applied in
  journal-sequence order, NO LLM anywhere in routing decisions
- worker agents are minimal OS processes: no network, no direct LLM access, syscall filtering
  (seccomp), typed binary IPC with frame caps, supervision trees with bounded restarts
- the whole orchestrator is tested under deterministic simulation (seeded fault injection,
  property tests, 100-run identical checkpoints)
- hierarchical plan decomposition (idea -> concepts -> phases -> steps -> baby steps) with cycle
  detection, and anti-stagnation detectors (loop/oscillation/no-progress patterns)

THE QUESTION: how do 2025-2026 agent frameworks and papers handle deterministic multi-agent
orchestration, and does anything beat FSM + supersteps + process isolation? Specifically:
1. published agent-orchestration architectures with determinism or replay guarantees (industry
   or academic), and what they give up for it
2. durable-execution engines (Temporal-style event-sourced workflows) applied to LLM agents -
   evidence of advantages over an in-process FSM + journal
3. multi-agent failure-mode research 2024-2026 (taxonomies, coordination failures) and which
   structural mitigations measurably help
4. sandboxing/isolation: is per-agent OS-process isolation with seccomp still the practical
   ceiling on consumer hardware, vs microVMs (Firecracker), WASM components, gVisor - measured
   startup/overhead trade-offs for short-lived agent workers
5. deterministic-simulation testing for agent systems (FoundationDB-style) - who else does it,
   any tooling newer than madsim-class simulators.

FORMAT: (a) 3-6 strongest alternatives with one-line each + links + license + maturity;
(b) documented failure modes of MY design; (c) verdict: KEEP / HARDEN (named additions) /
REPLACE (named alternative), evidence-justified; (d) top 3 highest-value specific changes,
ranked by evidence strength. Primary sources over blog posts. Flag anything unverified.

================================================================================
PROMPT 3 - OUTPUT VERIFICATION (the "can't lie" stack)
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: verifying the agent's output so it cannot claim false success. Papers
(arXiv 2024-2026), GitHub codebases, and production systems. Actively attack my design - list
its documented failure modes and what others do about them.

WHAT I HAVE TODAY (layered done-gates, fail-closed: missing evidence = FAIL, never SKIP):
- static layer: producer/consumer symmetry checks (new code with only test-file callers FAILS),
  cross-language wiring contracts, dead-code lints
- runtime layer: entrypoint-coverage proof - a new function must execute from a REAL production
  entrypoint (CLI/IPC/route), instrumented-binary hit counts; unit tests calling it directly do
  not count
- trace layer: assertions on an append-only event journal the implementing agent cannot write to;
  an auditor agent cannot report "tests passed" unless the orchestrator-captured output hash
  matches (fabricated success is structurally impossible)
- mutation testing gates; metamorphic relations + intent-pair tests; the evaluator/gate fileset
  is frozen and read-only to the run being evaluated (a run that edits its own gate = blocked)
- critique gates use a model from a DIFFERENT pretraining family (same-model self-critique is
  banned as a gate); calibrated abstention via conformal prediction instead of confident guessing

THE QUESTION: what is the strongest 2024-2026 work on verifying autonomous coding-agent output?
Specifically:
1. anti-reward-hacking / anti-gaming research for agent evaluators (incl. agents gaming their own
   tests, in-boundary artifact gaming, benchmark contamination)
2. execution-grounded verification systems (run-the-real-binary proof) - anyone with a stronger
   scheme than entrypoint coverage + off-agent trace assertions?
3. cross-model / committee verification: measured evidence on cross-family critique vs
   self-critique vs N-sample voting
4. mutation testing + metamorphic testing applied to LLM-generated code (2024-2026 results)
5. calibrated refusal/abstention for code agents (conformal methods, selective prediction) -
   newer/better than split-conformal baselines?

FORMAT: (a) 3-6 strongest alternatives with one-line each + links + license + maturity;
(b) documented failure modes of MY stack; (c) verdict: KEEP / HARDEN (named additions) /
REPLACE (named alternative), evidence-justified; (d) top 3 highest-value specific changes,
ranked by evidence strength. Primary sources over blog posts. Flag anything unverified.

================================================================================
PROMPT 4 - CONTEXT MANAGEMENT
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: managing the model's context window over long agent sessions. Papers (arXiv
2024-2026), GitHub codebases, and production systems. Actively attack my design - list its
documented failure modes and what others do about them.

WHAT I HAVE TODAY:
- a fixed-slot token-budget allocator: 8 named slots (system prompt / rules / task spec / repo
  map / code context / event history / reasoning history / output reserve) with per-slot floors
  and steal-priorities, adaptive 3-level reduction under pressure (<50ms)
- prompt-cache discipline: 2 cache breakpoints, byte-stable prefix (volatile content strictly
  after the last breakpoint); compaction NEVER rewrites the cached prefix - deterministic
  trimming drops trailing observations instead (LLM re-summarization of the prefix is banned
  because it busts provider prompt caches)
- a session DAG with lossless trim (dropped content recoverable from the journal), branch/merge,
  and a handoff digest authored from the journal for cross-session continuity
- OPPOSITE policies per backend, selected at runtime: cloud lane = maximize cache-stable prefix,
  late compaction; local small-window lane (32K) = minimize window, unload early, selective
  reload only
- two-tier memory: preference memory with bi-temporal supersession (superseded facts
  hard-filtered at retrieval, never narrated as "outdated"); code-grounded fact memory where
  every fact cites a code location re-validated before use, with TTL expiry

THE QUESTION: best 2025-2026 work on agent context management over long sessions - does anything
beat slot-budgeting + deterministic compaction + cache-prefix discipline? Specifically:
1. published context-management architectures from production coding agents (what do the major
   agent products actually do, where documented)
2. compression research 2024-2026: deterministic/structural approaches (AST folding, LSP slicing,
   observation masking) vs LLM summarization - measured quality AND cache-economics trade-offs
3. long-session memory architectures (MemGPT lineage, A-MEM, newer 2025-2026 work) with evidence
   of task-success gains, not just token savings
4. KV-cache-aware context policies for SMALL local windows (<=32K) - anyone doing regime-aware
   (cloud vs local) policy switching?
5. cross-session handoff/continuity designs that are journal-derived (not model-recalled).

FORMAT: (a) 3-6 strongest alternatives with one-line each + links + license + maturity;
(b) documented failure modes of MY design; (c) verdict: KEEP / HARDEN (named additions) /
REPLACE (named alternative), evidence-justified; (d) top 3 highest-value specific changes,
ranked by evidence strength. Primary sources over blog posts. Flag anything unverified.

================================================================================
PROMPT 5 - LOCAL INFERENCE LANE
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: the local model-serving layer. Papers, GitHub codebases, benchmarks
(2024-2026). Actively attack my design - list its documented failure modes and what others do
about them.

WHAT I HAVE TODAY:
- a Python sidecar process serving a 7B code model via HuggingFace transformers
  (model.generate + a custom LogitsProcessor chain), exposing an Anthropic-Messages-compatible
  HTTP API with SSE streaming
- 4-bit NF4 quantization (bitsandbytes), bfloat16 compute, SDPA or FlashAttention-2, single
  consumer GPU (24GB class), ~32K context ceiling
- custom logits processors are load-bearing: contrastive-decoding variants, entropy monitors,
  prompt-lookup, grammar masking (XGrammar) used ONLY for structured tool-op emission - never
  for free reasoning text
- planned: batch-invariant kernels + seeded sampling for run-to-run token determinism

THE QUESTION: in 2026, is HF transformers still defensible as the local serving layer at this
scale, vs the alternatives? Evaluate vLLM, SGLang, llama.cpp/llama-server, mistral.rs, candle,
TensorRT-LLM, and anything newer, SPECIFICALLY on these axes (my real constraints, in priority
order):
1. single-GPU 7B-class throughput/latency at batch size 1-4 (agent workloads, not bulk serving)
2. custom logits-processor extensibility (can I port a chain of stateful Python processors? at
   what cost?)
3. deterministic / reproducible decoding support (batch-invariance, per-request seeds)
4. NF4-or-equivalent 4-bit quality at 7B (AWQ/GPTQ/GGUF quality comparisons, 2025-2026 measured)
5. structured-output/grammar support, OpenAI/Anthropic-protocol-compatible serving, and
   KV/prefix-cache reuse across requests
6. operational simplicity for a desktop-deployed sidecar (install footprint, cold start, VRAM
   headroom behavior)

Also: 2025-2026 evidence on whether 7B coder models have been displaced for local agent use
(newer 7-14B code models with materially better SWE-style benchmark results at the same VRAM).

FORMAT: (a) per-alternative one-line verdicts + links + license + maturity; (b) documented
failure modes of MY setup; (c) verdict: KEEP / HARDEN (named additions) / REPLACE (named
alternative), evidence-justified; (d) top 3 highest-value specific changes, ranked by evidence
strength. Primary sources and reproducible benchmarks over blog posts. Flag anything unverified.

================================================================================
PROMPT 6 - SAFE SELF-IMPROVEMENT LOOP
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: the system's self-improvement loop. Papers (arXiv 2024-2026), GitHub
codebases, and production systems. Actively attack my design - list its documented failure modes
and what others do about them.

WHAT I HAVE TODAY:
- behavior rules stored as DATA (SQLite rows with typed predicates), never as code edits, with a
  measured promotion lifecycle: proposed -> candidate -> shadow-mode -> active, gated on observed
  false-positive rate (<5%), true-positive rate (>80%), and causal contribution; an immune tier
  of rules that can never be auto-modified
- config-parameter evolution: only a closed enum of parameters is touchable; staged evaluation
  (10/50/200 tasks), auto-revert on >10% regression over 3 evaluations; structural/code changes
  are human-only
- bandit-based promotion decisions (Thompson-sampling class) being wired to replace a hand-tuned
  sigmoid
- HARD DOCTRINE: fitness must be execution-grounded (real frozen-test pass rates) - never model
  self-assessment, never a hash of the config bytes, never an LLM grading its own output; trap
  tasks / canary tasks to detect gaming; held-out task stores with contamination filtering
- the whole loop ships dark behind flags until each gate's false-positive rate is bounded on
  real runs

THE QUESTION: strongest 2024-2026 work on SAFE self-improving agent systems. Specifically:
1. Darwin-Gödel-machine-style / self-modifying agent loops (2024-2026): published designs, and
   especially their documented COLLAPSE and GAMING failure modes (fitness hacking, eval
   overfitting, degenerate convergence)
2. verifier-gated evolution: what makes the verifier trustworthy when the generator improves
   (stronger-than-generator arguments, cross-family verification, eval firewalls / held-out
   hygiene)
3. measured evidence on rules-as-data / prompt-policy evolution vs weight-level methods (SFT/RL)
   for improving agents WITHOUT retraining
4. bandit / evolutionary promotion schemes for agent configs with non-IID task streams
5. security-monotonicity: evidence that iterative self-refinement degrades safety/security
   properties, and gate designs that prevent it.

FORMAT: (a) 3-6 strongest alternatives with one-line each + links + license + maturity;
(b) documented failure modes of MY loop; (c) verdict: KEEP / HARDEN (named additions) /
REPLACE (named alternative), evidence-justified; (d) top 3 highest-value specific changes,
ranked by evidence strength. Primary sources over blog posts. Flag anything unverified.

================================================================================
PROMPT 7 - CROSS-SESSION AGENT MEMORY
================================================================================

I'm building an autonomous AI coding-agent system (Rust orchestrator + Python inference sidecar,
runs locally on consumer hardware, also routes to cloud LLM APIs). Deep research request on ONE
architectural pillar: agent memory across sessions. Papers (arXiv 2024-2026), GitHub codebases,
and production systems. Actively attack my design - list its documented failure modes and what
others do about them.

WHAT I HAVE TODAY (two tiers + a firewall):
- TIER A, code-grounded fact memory: every stored fact must cite a code location (file/symbol);
  the citation is RE-VALIDATED against the live codebase before the fact is ever injected into a
  prompt; facts carry a TTL (~28 days) and expire
- TIER B, user-preference memory: explicit-capture (/remember-style), bi-temporal supersession
  applied at WRITE time, and retrieval HARD-FILTERS superseded rows (never injects them with an
  "outdated" narration - measured elsewhere to inflate tokens ~68% for zero gain); deletion is
  banned for auditability (superseded rows stay in the store, invisible to retrieval)
- write-time poisoning defenses (injection-pattern scan on memory writes; trust tiers on sources)
- AUDITOR-OBJECTIVITY FIREWALL: reviewer/critic/adversary agents receive ZERO memory injection,
  so accumulated beliefs can never soften an audit
- a staleness instrument measuring how often a stale fact actually reaches a prompt
  (measure-before-trust: retrieval expansion is gated on that number)

THE QUESTION: best 2025-2026 agent-memory designs and evidence. Specifically:
1. the Mem0 / Zep / Letta(MemGPT) lineage and NEWER 2025-2026 systems - measured task-success
   gains (not just token savings), and their consistency/staleness handling vs bi-temporal
   supersession
2. graph/temporal-knowledge-graph memories for coding agents specifically - does code-citation
   grounding like mine appear anywhere else, and is there a better validation scheme?
3. memory poisoning & persuasion attacks on agent memory (2024-2026): attack papers, measured
   defenses, and whether write-time scanning + trust tiers is still adequate
4. the objectivity question: any published work on memory/persona contamination of evaluator
   agents (evidence for or against my zero-memory-for-auditors firewall)
5. forgetting policies: TTL vs usage-based decay vs learned forgetting - measured comparisons.

FORMAT: (a) 3-6 strongest alternatives with one-line each + links + license + maturity;
(b) documented failure modes of MY design; (c) verdict: KEEP / HARDEN (named additions) /
REPLACE (named alternative), evidence-justified; (d) top 3 highest-value specific changes,
ranked by evidence strength. Primary sources over blog posts. Flag anything unverified.

end of prompts file.
