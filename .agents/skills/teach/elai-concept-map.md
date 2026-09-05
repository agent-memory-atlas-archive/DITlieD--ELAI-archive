ELAI CONCEPT MAP — the teach-skill curriculum spine for "teach me ELAI".
Built 2026-06-24 from a full read of docs/thefinalglance/ELAI-FEATURE-MAP.md
(all 2501 lines) + the phase-index. Dependency-ordered: teach TIER 0 before
TIER 1, etc. A concept lists the PROBLEM it solves (the WHY — test this first,
it is what vibe-coding skips) and the KEY FACTS to free-recall on. When a
lesson needs implementation detail below the feature-map altitude, drill with
codegraph_explore (one call, verbatim source) or pull the named plan slice;
do NOT pre-read those — fetch at the moment the lesson reaches them.

Spec-eval posture: every concept here is taught AS SPECIFIED. The STATUS tag
(WIRED / DARK / PARTIAL / PLANNED) is an implementation aside, never a
wrong-answer verdict and never part of the design question.

================================================================================
THE ONE-SENTENCE THESIS (the root every other concept hangs off)
================================================================================
ELAI is an autonomous coding harness built around a single problem: an LLM that
writes code, writes its own tests, and reports "done" is structurally
untrustworthy — it can satisfy its own success criteria from inside the boundary
the test was meant to police. So ELAI's entire design maximizes TRUST, not
capability: every claim of work must leave an off-agent artifact (a journal row,
a coverage hit, a captured stdout hash) that the agent which did the work did
NOT author, and missing evidence is a hard FAIL, never a pass. Slogan:
"Creator-Verifier on a Delegation backbone." Teach this first; everything else
is a mechanism that serves it.

================================================================================
TIER 0 — FOUNDATIONS (the substrate every other tier stands on)
================================================================================

C0-THESIS  trust-maximization, not capability-maximization
  WHY: "compiles + tests pass" is blind to wire-dark code — a unit test calling a
    function directly is indistinguishable from a missing production caller. The
    agent that writes impl + test + done-claim has gamed its own gate from inside.
  KEY FACTS: off-agent artifact is the load-bearing proof; missing evidence = FAIL
    (not SKIP/UNVERIFIED); the gate is frozen + read-only to the run being judged;
    measure the real objective never a proxy; ~40-45% measured wire-dark rate is
    the failure this exists to kill.
  DEPENDS: none (this is the root).

C0-SSB  the Shared State Board (the spine)
  WHY: trust needs a record an agent cannot forge or erase. One append-only log is
    that record AND the telemetry AND the plan-status source AND what makes a run
    replayable.
  KEY FACTS: single-writer SQLite in WAL mode, append-only (no UPDATE/DELETE),
    r2d2 read pool, ~80K inserts/sec, ClockProvider timestamps, broadcast-after-
    commit; action_type GOLDEN LOCK = a 9-tuple lockstep (variant + payload arm +
    struct + test payload + golden row + TOTAL_VARIANTS bump + SCHEMA_VERSION bump
    + match arms + ts-rs binding) so a partial add breaks the build; new types use
    "Pattern-2" (one variant with a Kind discriminator enum).
  DEPENDS: C0-THESIS.

C0-FSM  the 9-state finite state machine
  WHY: the trust checks have to happen in an enforced order — you cannot audit
    before you execute, cannot execute before tests exist. The FSM sequences them.
  KEY FACTS: IDLE / PLANNING / FEASIBILITY_PROBE / TESTING_FIRST / EXECUTING /
    AUDITING / MEMORIZING / COMPLETE / CONFIGURING; DAIRA evidence gates between
    states; phase-aware tool + rule loading; InteractiveGate (human) vs
    AutonomousGate (deterministic EvidenceCheck per transition); RiskLevel; every
    transition emits an SSB row. TESTING_FIRST before EXECUTING = TDD is structural.
  DEPENDS: C0-SSB.

C0-TIERC  Tier-C process isolation (what makes "off-agent" physically real)
  WHY: a verifier that shares the creator's memory/context can be talked around.
    Make it a separate OS process that literally cannot reach the creator.
  KEY FACTS: agents = minimal OS processes, NO network / NO LLM / NO tools of their
    own; talk only over a 15-message postcard IPC with a 10MB frame cap; seccomp
    (2 profiles), SO_PEERCRED; the orchestrator (not the agent) holds tools/creds
    and runs hooks — agents never see hooks. This is why the auditor's verdict is
    trustworthy: it ran somewhere the executor could not influence.
  DEPENDS: C0-FSM.

================================================================================
TIER 1 — REASONING & GROUNDING (how ELAI thinks, and proves it thought right)
================================================================================

C1-GRC  Grounded Reasoning Chain
  WHY: free-form chain-of-thought lets a model rationalize anything. Force every
    reasoning step to cite a grounding source and spend from a finite budget.
  KEY FACTS: built-in thinking DISABLED; all reasoning via mcp_think with a
    mandatory grounding citation ("type:reference"); per-TOPIC budgets (each topic
    its own call/char budget); state machine Observe -> Hypothesize -> PlanAction ->
    Act -> Verify -> Converge; must_converge when the budget runs out; grounding
    strength maps to {Verified, High, Uncertain}.
  DEPENDS: C0-FSM.

C1-GROUNDMODES  grounding verification + the A-E mode router
  WHY: not every claim needs the same proof; pick the cheapest sufficient checker,
    deterministically, with NO LLM input (an LLM choosing how hard to check itself
    is the same self-gaming problem).
  KEY FACTS: 6 binary pass/fail methods FILE / SOURCE / TEST / WEB / API /
    API_REGISTRY_CHECK (+CosmosCheck 7th, +VisualArtifactCheck 8th); MiniCheck-style
    binary verdict; a plan-sweep BLOCKS PLANNING->EXECUTING on any UNGROUNDED claim;
    select_grounding_mode (modes A-E) is a deterministic predicate, locked in a
    registry; per-agent GRC config gives each of 8 roles distinct budget/methods
    (auditor's Converge requires a FINDING: — anti-rubber-stamp).
  DEPENDS: C1-GRC.

================================================================================
TIER 2 — KNOWLEDGE / RETRIEVAL (how ELAI sees the codebase it edits)
================================================================================

C2-DKG  dependency knowledge graph
  WHY: an agent editing code blind breaks things it can't see. Give it a real
    symbol+edge graph of the repo.
  KEY FACTS: tree-sitter -> SQLite symbol graph (dkg_nodes / edges / files, WAL);
    typed edges imports / contains / implements; blake3 incremental update with
    CASCADE; 18 grammars + 4 resolvers (multi-language); underpins impact/blast-radius.
  DEPENDS: C0-SSB (shares the SQLite/WAL substrate idea).

C2-CASCADE  the retrieval cascade + RRF fusion
  WHY: no single retriever is enough; lexical, semantic, and graph-importance each
    miss what the others catch. Fuse them, then rerank.
  KEY FACTS: BM25 (tantivy, path field must be STRING not TEXT, 50-line windows) ->
    VECTOR (dual ONNX: GraphCodeBERT code 768d + Granite text 384d, usearch HNSW
    cosine, per-model circuit breaker, blake3 content-hash cache) -> PAGERANK repo
    map (aider-derived, symbol-level nodes) -> CatRAG -> A-RAG; fused by
    Reciprocal Rank Fusion k=60; CROSS-ENCODER reranker (ms-marco-MiniLM, CPU,
    <50ms/50, skip if <=10 candidates) is the high-value stage NEVER pruned.
  DEPENDS: C2-DKG.

C2-RESILIENCE  retrieval resilience (degrade, never crash)
  WHY: a missing model or a flaky index must not take the whole agent down.
  KEY FACTS: per-model circuit breaker (3 fail/60s, distinguishes transient vs
    permanent); SearchMode cascade HYBRID -> SPARSE_ONLY -> BM25_ONLY; missing model
    -> DEGRADED(BM25_ONLY) not a crash; query validator <1ms; 5s startup canary;
    9-stage QueryTrace; index_version migration.
  DEPENDS: C2-CASCADE.

================================================================================
TIER 3 — ORCHESTRATION (how many agents get coordinated deterministically)
================================================================================

C3-AGENTS  agent system + pool + roles
  WHY: one model in one context can't hold a large build AND stay objective about
    its own work. Split into role-scoped agents with supervision.
  KEY FACTS: AgentPool, 8 AgentRoles, lifecycle + checkpoint + crash recovery,
    Erlang-style supervision (max_restarts=3), anti-spiral blake3 3-identical-hash
    stop; ELAI's canonical agents (auditor, context-compiler, integration-explorer,
    memory-writer, ...) are dispatched as separate Tier-C processes.
  DEPENDS: C0-TIERC.

C3-BSP  BSP supersteps (deterministic macro-orchestration)
  WHY: parallel agents writing to shared state non-deterministically is unreplayable.
    Borrow Bulk-Synchronous-Parallel: compute in parallel, then a barrier applies
    all writes in a fixed order.
  KEY FACTS: channel types LastValue / Topic / EphemeralValue / UntrackedValue;
    barrier -> apply-writes-by-SSB-seq (the journal sequence IS the total order);
    JoinSet spawn_parallel + VRAM-aware scheduling.
  DEPENDS: C3-AGENTS, C0-SSB.

C3-PLANMGMT  hierarchical plan management + decomposition
  WHY: a big task must break into a tree small enough that each leaf is a verifiable
    vertical slice (producer + consumer + smoke in ONE unit).
  KEY FACTS: skeleton <=2-4K tokens always resident (Slot3) + per-phase detail
    loaded on demand; recursive Idea -> Concepts -> Phases -> Steps -> BabySteps
    (8-block BabyStepSpec); contract extraction; Kahn cycle detection on the DAG;
    "add fn X" with no consumer is REJECTED at decomposition time (the walking-
    skeleton PRE_GATE).
  DEPENDS: C3-BSP.

C3-SESSIONDAG  session DAG + context lifecycle
  WHY: long sessions overflow the window; you need lossless history + a lossy view,
    and the ability to branch/rewind without clobbering the human.
  KEY FACTS: DAG snapshot versioning; 3-pass LOSSLESS trim (stubs recoverable from
    SSB seq — the store is lossless, the view is the projection); branch/merge max 3;
    85% handoff trigger; reversible edits / time-travel (CAS blake3+zstd object store,
    undo-tree HEAD, undo=back / redo=forward-edge / undo-then-edit=fork, snapshot-
    before-restore guard); regime-aware policy (cloud keeps a cache-stable prefix +
    late compaction; local minimizes the window + unloads early — OPPOSITE policies).
  DEPENDS: C3-PLANMGMT.

C3-AUTONOMOUS  autonomous mode + anti-spiral
  WHY: to run unattended it must self-advance on EVIDENCE, not vibes, and detect
    when it is stuck instead of looping forever.
  KEY FACTS: AutonomousGate wraps InteractiveGate; AutoAdvanceEvaluator runs a
    deterministic EvidenceCheck per transition; SPLIT_TASK (max 4 / depth 2),
    SUBSTITUTE_MODEL; tree-level risk (a Concept node is HIGH = always human);
    50-baby-step circuit breaker; anti-spiral = 4 StagnationPatterns + 5 personas +
    GoalDriftCanary, firing unconditionally in the drive loop.
  DEPENDS: C3-SESSIONDAG, C0-FSM.

================================================================================
TIER 4 — VERIFICATION / TRUST-GATES (the heart — the thesis made runnable)
================================================================================

C4-WIRING  wiring enforcement (the off-agent done-gate)
  WHY: this is C0-THESIS turned into a concrete 4-layer gate the auditor runs.
  KEY FACTS: done-gate = wiring_contract_check.py --run-trigger PASS (cargo-green is
    REJECTED as a done-gate); 4 layers cheapest-first — L1 compile-time (dead_code /
    producer-without-consumer), L2 cross-language contract symmetry (Tauri<->TS,
    sidecar<->Rust, action_type golden lock), L3 entrypoint coverage (drive the real
    binary, a new fn with count==0 = BLOCK), L4 runtime trace assertion (assert the
    off-agent event row landed; db-not-found = FAIL); test-file callers do NOT count
    as production reachability; a new default-OFF flag needs a wire-dark-allowlist
    row (type/owner/expiry/promotion_plan_id/smoke) or it FAILS R9.
  DEPENDS: C0-THESIS, C0-SSB.

C4-CANTLIE  the cant-lie test-integrity gate (F-4)
  WHY: the most direct lie is "tests passed" when they didn't. Make that impossible.
  KEY FACTS: AntiFabricationGate in the drive loop; the orchestrator captures a
    blake3 stdout_hash of the real test run; the auditor CANNOT emit passed=true
    unless its claimed hash matches the orchestrator-captured one; emits
    TestIntegrityEvent. The proof is an artifact the auditor did not author.
  DEPENDS: C4-WIRING.

C4-RRA  Reproducible-Run Artifact + taint
  WHY: trust must be portable and replayable by a third party, and untrusted bytes
    must be tracked so they never silently reach a dangerous sink.
  KEY FACTS: RRA = a portable msgpack+zstd bundle: SSB stream + seed + HardwareClass
    + model pin + gate-verdict hashes + blake3 root; "cloud lane = reproducible
    ACTION-stream, not tokens"; replay --verify; TAINT (G-1) = Provenance built at 4
    ingest sites (WebFetch/MCP/file/sandbox), Provenanced<String> threaded, a
    capability_gate before destructive sinks; dependency-screened join (a global
    taint join was REJECTED — collapses utility to 33%).
  DEPENDS: C4-CANTLIE.

C4-BLAST  blast-radius / anti-regression gate
  WHY: the #1 real failure — "a small fix silently breaks something else."
  KEY FACTS: turns the shipped impact API into an ENFORCED auditor gate: reverse-dep
    cone + diff-scope (over-editing) + regression-test selection (a pass->fail =
    BLOCK) + silent-deletion flag; deterministic, the LLM never adjudicates;
    observe-only first until the false-positive rate is bounded on >=3 runs.
  DEPENDS: C4-RRA, C2-DKG.

================================================================================
TIER 5 — SELF-IMPROVEMENT / DGM (ELAI editing ELAI, safely)
================================================================================

C5-RULES  the rules engine (governance as data)
  WHY: hard-coded checks can't evolve or be A/B-promoted; make rules editable DATA
    with a measured lifecycle.
  KEY FACTS: rules live in SQLite; predicate kinds AST / GREP / SCHEMA / METRIC /
    COMPOSITE; verdicts ALLOW / WARN / BLOCK / ESCALATE; lifecycle PROPOSED ->
    CANDIDATE -> SHADOW -> ACTIVE -> RETIRED (promote only on fpr<5% + tpr>80% +
    causal>50%); a mutable=0 IMMUNE tier the system cannot weaken; >=33 seed rules.
  DEPENDS: C0-THESIS.

C5-SELFMOD  self-modification core + the DGM safety
  WHY: a self-improving loop with no verifier stronger than its generator PROVABLY
    collapses (it learns to game its own fitness).
  KEY FACTS: 3 levels L1 data-tables (no LLM) / L2 config params (10/50/200 staged
    eval) / L3 structural = human-only; DGM sigmoid parent-select; AllowedConfigKey
    is a CLOSED enum; auto-revert if >10% regression over 3; the two cardinal sins to
    NEVER reintroduce — fitness = blake3(config bytes) [bytes, not performance] and
    trap-detection = keyword grep [unsound]; the fix: execution-grounded fitness
    (pass-rate on frozen tests), behavioral trap detection with MEASURED recall, a
    CROSS-FAMILY verifier (different pretraining family from the generator, never
    same-model self-critique), and promote on worst-case/lineage-diverse verdicts.
  DEPENDS: C5-RULES, C4-WIRING.

================================================================================
TIER 6 — SECURITY / SANDBOX / INJECTION-DEFENSE
================================================================================

C6-VIBEGATE  the unified write gate + dependency defense
  WHY: the model can be tricked (prompt injection) or hallucinate a malicious
    dependency; the write path must fail closed before bytes land.
  KEY FACTS: VibeGate = fail-closed multi-axis PreToolUse — code-vuln (AST/GREP) +
    secret (GREP) in-process Tier-1, dep-existence + skill-scan Tier-2; mutable=0
    "vibesec" domain; HALLUCINATED-DEP gate checks existence+age+provenance at
    write-time incl cargo (Spracklen 19.7% hallucination rate); CaMeL data-plane
    locks researcher tools before the first untrusted byte; hook system T1<1ms /
    T2<10ms tree-sitter / T3<5ms pattern — NO LLM in the hot path.
  DEPENDS: C0-TIERC, C5-RULES.

================================================================================
TIER 7 — INFERENCE (where the tokens actually come from)
================================================================================

C7-SIDECAR  the local Python sidecar
  WHY: a private, steerable, deterministic local model lane that speaks the same
    protocol as the cloud so the rest of the system doesn't care which it talks to.
  KEY FACTS: FastAPI Anthropic-compatible /v1/messages + /v1/steering/*;
    model.generate() + a LogitsProcessor chain (DoLa, ETC, prompt-lookup, XGrammar
    mask); canonical deployment is the REMOTE 3090, never local; Qwen2.5-Coder
    (3B/7B abliterated) NF4 4-bit, FA2 mandatory on 7B (FA3/FA4 are Hopper-only,
    blocked on Ampere); KV prefix cache is a no-op on the live streaming path
    (honest residue).
  DEPENDS: C0-FSM (callers gate through it).

C7-ROUTING  multi-runtime backend + cloud routing
  WHY: different tasks deserve different models/lanes; abstract them behind one
    trait so role-based routing is a config decision, not a code change.
  KEY FACTS: RuntimeAdapter trait, 11 backends, PerRoleRuntime, resolve_backend;
    CloudModeRouter with a HARD cost ceiling (blocks, not warns); cloud lane =
    Sonnet workhorse / Opus confirmation-gated / cheap floor for volume; cloud
    prompt CACHING is live (cache markers on the production path); the cant-lie /
    RRA spine treats the cloud lane as an action-stream because tokens aren't
    reproducible; the agent-CLI bridge can "ride" a vendor login (claude_code /
    codex_cli / gemini_cli ...) through the same spawn seam under Tier-C.
  DEPENDS: C7-SIDECAR, C7? (B-37 trait).

================================================================================
TIER 8 — SURFACES (how a human watches and steers)
================================================================================

C8-DASHBOARD  dashboard-v2 + the activity orbit
  WHY: the trust story is worthless if a human can't SEE the evidence. The dashboard
    is a read-only VIEW over the SSB journal — never a second source of truth.
  KEY FACTS: Tauri 2.10 + Svelte 5 runes + Vite 5 + three.js orbit; 9 windows + 10+
    drawers; ~98 Tauri commands; SsbViewer is a read-only SQLite WAL poll; engine
    crates are deliberately KEPT OUT of the dashboard's Cargo.toml; the 3D orbit IS
    the activity feed (PLAN/MAP/RESEARCH modes).
  DEPENDS: C0-SSB.

C8-CANVAS  the canvas workbench
  WHY: a 2D work-surface the model drives and the user points at — the moat is that
    every op is a journaled, reproducible, taint-tagged op-stream, not free-form UI.
  KEY FACTS: model drives canvas_* ops through the same dispatcher pattern; bindings
    + verdicts persist ONLY as SSB rows (the journal is the store, the canvas
    re-derives — no parallel state); "system" mode binds nodes to plan units / wiring
    contracts; deixis (user clicks an element -> handed to the model) is taint-gated;
    build still dispatches through the normal FSM — there is no canvas-side runtime
    (the deliberate "anti-n8n" line).
  DEPENDS: C8-DASHBOARD, C4-RRA.

================================================================================
TIER 9 — DOCTRINE & PLAN STRUCTURE (the laws, and how the repo is organized)
================================================================================

C9-DOCTRINE  the doctrine families (the binding laws of plan authoring)
  WHY: consistency across hundreds of plans needs named, citable invariants.
  KEY FACTS: W-1..10 wiring (off-agent done-gate, default-off is never terminal);
    GD-1..5 trust-compose (COMPOSE-NOT-RESTATE — consume prior verdicts; every number
    recomputable from the journal under ELAI_SEED; ablation framing never a
    leaderboard rank); HD efficiency (output bills 5x, never constrained-decode
    reasoning roles, never bust the cache prefix); TD trading-trust (agent-
    unresettable trial count, LLM is advisory never the gate); RD reasoning (credit
    abstention, penalize confident-wrong, budget scales with serial depth not window
    size); VD verification (fitness execution-grounded, verifier stronger-than-
    generator + cross-family); RV reversibility (never clobber the human, lossless
    store / lossy view); CL context (one window truth per backend, regime-opposite
    policy); ID/IC I-plan (generator proposes / orchestrator owns state; ELAI-measured
    baselines never paper constants; search is a LADDER not MCTS-first; SFT-before-
    GRPO; ciphers BANNED); JD harness (.claude legs have no SSB — evidence is the
    hook-log/state.json).
  DEPENDS: all prior tiers (it abstracts them).

C9-PLANS  the plan-letter map (how to navigate the corpus)
  WHY: orienting in the doc tree — which letter holds what.
  KEY FACTS: base 0-4 = walking-skeleton -> production; B = MVP completion +
    dashboard + harness hardening; C = closes B's verification gaps + paper-grounded
    waves + benchmarks; D = reasoning-improvement substrate + 5-mode grounding;
    E = a 55-report research sweep (one steal per report); F = VibeSec/security gates
    + the ENFORCED done-gate doctrine origin; G = trust-maximization composing F;
    H = efficiency/canvas/trading-trust/UX-memory/grounded-reasoning/DGM-verification/
    reversibility/context-lifecycle; I = fault-isolation/evidence-ranked-search/
    cache-economics/anti-gaming/routing/flywheel; J = paper+repo steals/perception/
    hardware/determinism/harness tooling. The SSB SCHEMA_VERSION / TOTAL_VARIANTS
    counters are the live "where are we" stamp (grep ssb.rs, never trust a doc number).
  DEPENDS: C9-DOCTRINE.

================================================================================
TEACHING ORDER (default walk; interleave older concepts every 2-4 new ones)
================================================================================
1. C0-THESIS (the root — over-invest here; it reframes everything else)
2. C0-SSB -> C0-FSM -> C0-TIERC (the substrate triangle)
3. C1-GRC -> C1-GROUNDMODES
4. C2-DKG -> C2-CASCADE -> C2-RESILIENCE
5. C3-AGENTS -> C3-BSP -> C3-PLANMGMT -> C3-SESSIONDAG -> C3-AUTONOMOUS
6. C4-WIRING -> C4-CANTLIE -> C4-RRA -> C4-BLAST  (the payoff tier — ties back to C0)
7. C5-RULES -> C5-SELFMOD
8. C6-VIBEGATE
9. C7-SIDECAR -> C7-ROUTING
10. C8-DASHBOARD -> C8-CANVAS
11. C9-DOCTRINE -> C9-PLANS

The single most important retrieval to drill repeatedly, from EVERY angle, is the
C0-THESIS <-> C4 link: "why does "tests pass" not mean done, and what makes an
ELAI done-claim trustworthy instead." If the user owns that, they understand ELAI.
