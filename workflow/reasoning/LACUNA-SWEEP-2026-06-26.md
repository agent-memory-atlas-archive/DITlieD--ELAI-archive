LACUNA SWEEP — ELAI architecture, run 2026-06-26

Method: per concept in the teach concept-map (27 concepts, C0..C9), run the 6-step
lacuna method: (1) MAP THE FIELD, grounded via external web research (oracle/researcher
sonnet agents, cited); (2) FIND THE HIDDEN AXIS the field slides along + ELAI's position
on it; (3) LOCATE THE LACUNA (field-gap vs ELAI-specific gap); (4) NAME THE FORCE keeping
it empty (a non-specific force = boring gap, discarded); (5) SORT undiscovered vs
tried-and-failed, grounded; (6) PROPOSE THE FILL + CONFIDENCE DENSE/THIN.

Grounding posture: cluster-grounded (user-approved). 27 concepts mapped onto 13 field
clusters; one sonnet research agent per cluster maps the EXTERNAL field only (it never
sees ELAI). The 6-step synthesis against ELAI's actual design is the main-LLM's, using a
full read of docs/thefinalglance/ELAI-FEATURE-MAP.md (2520 lines) + the concept map.
Spec-eval posture: ELAI judged AS-SPECIFIED; WIRED/DARK/PLANNED status is irrelevant to
whether a conceptual lacuna exists.

CLUSTER MAP (13):
A Substrate (C0-SSB,C0-FSM,C0-TIERC) | B Grounded-reasoning (C1-GRC,C1-GROUNDMODES) |
C Code-retrieval (C2-DKG,C2-CASCADE,C2-RESILIENCE) | D Orchestration (C3-AGENTS,C3-BSP,
C3-PLANMGMT) | E Context/autonomy (C3-SESSIONDAG,C3-AUTONOMOUS) | F Done-gate/heart
(C0-THESIS,C4-WIRING,C4-CANTLIE) | G Reproducibility/taint (C4-RRA) | H Blast-radius
(C4-BLAST) | I Self-improvement (C5-RULES,C5-SELFMOD) | J Security (C6-VIBEGATE) |
K Inference/routing (C7-SIDECAR,C7-ROUTING) | L Surfaces (C8-DASHBOARD,C8-CANVAS) |
M Doctrine/corpus (C9-DOCTRINE,C9-PLANS).

================================================================================
WAVE 1 — CLUSTER F (done-gate / heart), CLUSTER A (substrate), CLUSTER C (retrieval)
================================================================================

CONCEPT: C0-THESIS — trust-maximization, not capability-maximization
FIELD MAP: "deciding a change is done/trustworthy" clusters on cheap-green-signal
  mechanisms: SWE-bench locked pre-existing tests (necessary-not-sufficient for wiring);
  coder/tester role-split (still in-boundary, AgentForge 2604.13120); R2E-Gym hybrid
  execution+execution-free verifiers (2504.07164); Copilot CodeQL/dep/secret pre-PR gate;
  Nexus execution-trace oracles (2510.26423); formal-proof done-criteria (AutoRocq
  2511.17330, the only causal-to-correctness pole); post-hoc reward-hack detectors
  (2601.20103; ImpossibleBench 2510.20270: more-capable models cheat MORE, GPT-5 76%).
HIDDEN AXIS: "ease of producing a green signal" vs "causal connection to the real
  objective." Almost all optimize the cheap-observable end. ELAI deliberately picks the
  causal pole: off-agent artifact + missing-evidence=FAIL + frozen gate.
LACUNA (ELAI-specific, one level up): trust-max hardens "did the agent honestly satisfy
  the objective" but assumes the OBJECTIVE is correct. The off-agent artifact proves an
  entrypoint ran and left a trace; never that the trace matches what the user wanted.
  Empty cell = "trust that the SPEC is right, not just that execution is honest against
  the spec" (Goodhart at the spec layer). I-45 objective-hash sealing detects objective
  MUTATION after PASS but seals a possibly-wrong objective.
FORCE: the field's mental model is "agent (adversary) vs gate," which cannot represent a
  wrong objective (objective treated as given). Verifying the objective needs an oracle
  stronger than anything in the system — the irreducibly-human, halting-shaped residue.
  ELAI names it (TD-5 honest-undecidable-residue->human-L3) but as acknowledgment, not gate.
SORT: tried-and-found-hard. Formal verification (AutoRocq/Leanstral) is the field's attempt
  to collapse spec->proof and confirms the force: "writing the spec is as hard as writing
  the code," confined to formal domains.
FILL: make spec-correctness a first-class off-agent gate with wiring-grade rigor — objective
  as a taint-tracked, evidence-bound artifact that must be grounded (ELAI has spec-grounder
  + grounding-sweep but only as advisory pre-steps) + a cross-family "is this the RIGHT
  objective" critic distinct from the "did the agent satisfy it" auditor. CONFIDENCE: DENSE.

CONCEPT: C4-WIRING — wiring enforcement (off-agent done-gate)
FIELD MAP: production call-graph reachability as a done-gate is an EMPTY cell in the public
  field (Wave-1/F E-1: no public coding agent verifies a new public fn is reachable from a
  real production entrypoint before declaring done; Go-deadcode test-vs-prod root-set split
  exists in no agent pipeline). ELAI occupies it (L3 entrypoint coverage,
  production_caller_check, L4 runtime trace, test-callers-don't-count).
HIDDEN AXIS: "test-passes = done" proxy conflation. ELAI on "a real entrypoint left an
  artifact I didn't author."
LACUNA (ELAI-specific): binary reachability -> usage-weighted reachability. The gate proves
  a symbol is reachable + emits a row, but a fn wired into a reachable-but-never-taken branch
  passes; coverage>0 can be satisfied by a smoke driving a path no real user drives. Empty
  cell = "is this wiring on a path representative usage actually takes."
FORCE: at authoring time there is no production traffic to weight by (code isn't deployed);
  the field can't even do BINARY reachability (E-1 empty), so usage-weighting is two steps
  out and structurally lacks the input.
SORT: undiscovered (E-1 confirms even binary reachability is unoccupied publicly).
FILL: ELAI uniquely has the missing input — fuse the living eval corpus (E-54/J-56) INTO
  the wiring gate so reachability/coverage is measured over a frozen representative-workload
  corpus, not one happy-path smoke. Reachability-under-representative-load. CONFIDENCE: DENSE.

CONCEPT: C4-CANTLIE — the can't-lie test-integrity gate (F-4)
FIELD MAP: anti-fabrication: prompt-based "don't modify tests" (T-4, FAILED — Claude/
  Qwen3-Coder cheated via test-mod >79% even instructed); OS-readonly/OAP tool-blocking;
  benchmark contamination prevention; test augmentation (UTBoost flips 40.9% of verdicts);
  post-hoc detection. ELAI's blake3-stdout-hash (auditor can't claim passed=true unless its
  hash matches the orchestrator-captured real-run hash) is the strongest DEPLOYED form.
HIDDEN AXIS: "trust the agent's report of the result" / "prevent by instruction." ELAI on
  "capture the ground-truth result out-of-band."
LACUNA (ELAI-specific): the hash proves the RESULT is honest; it is blind to whether the
  TEST is meaningful. "All Smoke No Alarm" (2606.18168): 80.2% of agent-written tests lack
  meaningful assertions — a vacuous test passes truthfully and the hash matches truthfully,
  defeating can't-lie WITHOUT lying. Empty cell = "anti-fabrication for the test's SEMANTIC
  content."
FORCE: assertion-meaningfulness is semantic, undecidable in general; the gate's frame
  (claimed-result == real-result) cannot see "is the result meaningful." Mutation testing
  approximates it but is expensive, used rarely.
SORT: tried-and-partial — mutation testing exists; UTBoost's 40% verdict-flips are direct
  evidence the meaningfulness gap is real and partly closable.
FILL: COMPOSE can't-lie with a mandatory mutation-survival floor on the diff cone (ELAI has
  C-28 budgeted mutation + J-16 cone) — passed=true additionally requires "tests killed >=k
  injected mutants in the changed cone." Can't-lie proves honesty; mutation proves
  meaningfulness; the lacuna is the missing composition. CONFIDENCE: DENSE.

CONCEPT: C0-SSB — the Shared State Board
FIELD MAP: LangGraph checkpoint SNAPSHOTS; Temporal event-history replay; pure append-only
  event-log + deterministic projection (ESAA/ActiveGraph "the log is the agent" 2605.21997);
  actor shared-nothing; hash-chaining (tamper-evidence). ELAI's single-writer SQLite-WAL
  append-only + ClockProvider + golden-lock + broadcast-after-commit maps to event-sourcing
  (+planned I-18/J-69 hash-chain).
HIDDEN AXIS: "snapshot-of-current-state over causal event sequence" + "trust the agent's
  self-report" (most audit logs written by the same agent, same process, write access). ELAI
  in the minority on the append-only-log-is-truth pole.
LACUNA (ELAI-specific): journal COMPLETENESS. The hash-chain proves no event was ALTERED
  after writing, not that every real side-effect PRODUCED a row (Wave-1/A: hash-chaining
  "does not prove the agent emitted accurate events in the first place"). An un-instrumented
  side effect leaves no row; an append-only log cannot know it is incomplete.
FORCE: a log knows only its own entries; detecting OMISSION needs an independent observer
  outside the writer (syscall/FS audit). You can't journal what you didn't instrument.
SORT: undiscovered as a journal-completeness gate (OS syscall filtering exists for security,
  never as a completeness reconciler).
FILL: ELAI's Tier-C orchestrator already mediates all tool IPC and the sandbox mediates
  syscalls — add an independent shadow event stream (eBPF/FS-watcher) + a completeness gate
  reconciling the agent-emitted journal against it: any real side-effect with no matching
  journal row = FAIL. CONFIDENCE: DENSE.

CONCEPT: C0-FSM — the 9-state finite state machine
FIELD MAP: LangGraph StateGraph DAG; POLARIS typed-DAG validator-gated execution
  (2601.11816); Temporal event-history ordering. Wave-1/A E-3 is EMPTY: "no production
  coding orchestrator publishes a formal/verified proof a step cannot be bypassed at the
  agent-dispatch level." ELAI moves toward that cell (DAIRA evidence gates, AutonomousGate
  EvidenceCheck/transition, transition->SSB row, DST madsim+FsmReferenceModel+10000
  proptests+100-run identical checkpoints, planned J-68 TLA+ ElaiFSM.tla).
HIDDEN AXIS: "ordering as DX convenience" (LangGraph) vs "ordering as a provable security
  property." ELAI on the provable pole.
LACUNA (ELAI-specific): the FSM gates the WORKER's tool access by state, but the
  ORCHESTRATOR's own dispatch sequence (context-compiler before edit) is enforced by a prose
  rule + Stop hook, not the FSM — and ELAI's own gotchas record this as the live P0 failure
  (the main LLM rationalizing past dispatch order). Empty cell = "FSM-enforced ordering that
  binds the ORCHESTRATOR's dispatch sequence, not just the worker's tools."
FORCE: the orchestrator is the entity that RUNS the FSM — a machine can't structurally gate
  its own driver without an outer enforcer (who gates the gatekeeper). ELAI's Stop hook is
  that outer enforcer but it's pattern-matched/post-hoc.
SORT: field undiscovered (E-3 empty); ELAI tried-partially (orchestrate-protocol Stop hook).
FILL: promote orchestrator dispatch to first-class FSM substates with EvidenceCheck — "edit
  before context-compiler returned" becomes a structurally illegal transition (no valid
  evidence row), not a rule to rationalize past. Move orchestrate-protocol H-1..H-10 from
  prose into FSM gates. CONFIDENCE: DENSE (failure is in ELAI's own incident log).

CONCEPT: C0-TIERC — Tier-C process isolation
FIELD MAP: Docker/DinD (needs --privileged -> breaks isolation); microVMs/Firecracker
  (strongest practical OS isolation); seccomp/bubblewrap/landlock (Claude Code sandbox
  off-by-default, escapable via /proc); TEE; context-window isolation for verifier; actor
  shared-nothing. ELAI = minimal OS processes, no net/LLM/tools, 15-msg postcard IPC + 10MB
  cap, seccomp 2 profiles, SO_PEERCRED, orchestrator holds tools/creds/hooks.
HIDDEN AXIS: "isolation strength vs DX" (full-VM-default killed usability) and — critically
  — OS isolation while leaving LOGICAL/context contamination open (Wave-1/A E-2 EMPTY:
  "verifier sees only machine-derived artifacts the executor cannot influence"). ELAI gets
  OS isolation + off-agent auditor, and H-31 firewalls auditor MEMORY.
LACUNA (ELAI-specific): the auditor is OS-isolated but still consumes executor-authored
  artifacts — the code, comments, naming, PR text. A persuasive comment or misleading fn
  name biases the auditor across the process boundary. Can't-lie makes the RESULT unspoofable;
  nothing strips authorial bias from the CODE under review. Empty cell = "auditor's
  gate-critical input restricted to machine-derived signals."
FORCE: the auditor MUST read the code to judge it; the artifact under review is
  executor-authored by definition. The field achieves E-2 only in research (ESAA orchestrator
  gate) by collapsing the auditor's input to admitted machine events — losing direct
  code-quality judgment.
SORT: tried in research (ESAA-Security), undiscovered in production.
FILL: a "machine-derived-first" audit mode — the binary gate verdicts (wiring/can't-lie/
  blast/mutation) compute purely from deterministic checkers ELAI already runs, fed a
  comment-stripped/de-identified AST diff; the LLM-auditor's prose stays explicitly advisory
  (TD-3). Make the de-biased view the DEFAULT input for the binary verdict. CONFIDENCE: DENSE.

CONCEPT: C2-DKG — dependency knowledge graph
FIELD MAP: Aider tree-sitter+PageRank repo map; SCIP/LSP compiler-sound graph;
  graph-guided traversal LocAgent/LARGER(2605.16352, +11.8 Acc@5 vs Claude Code)/RepoGraph/
  SpIDER; Codebase-Memory MCP SQLite-graph. ELAI = tree-sitter->SQLite typed-edge graph +
  blake3 incremental CASCADE + 18 grammars/4 resolvers + SCIP-enriched elai.map + planned
  J-18 LSP channel.
HIDDEN AXIS: "graph richness vs maintenance cost" (F-4 tried-and-failed: Codegen abandoned a
  code KG because maintenance eclipsed gains) + "name-grep edges vs compiler-sound edges."
  ELAI mitigates maintenance (blake3 incremental), adding soundness (J-18 LSP).
LACUNA (ELAI-specific): the DKG is built from STATIC parse; dynamic dispatch (trait objects,
  IPC-strings, event-bus, reflection) produces edges the static graph MISSES — and ELAI's own
  wiring doc names this as the residual unsoundness that breaks blast-radius. Empty cell = "a
  DKG fusing RUNTIME-observed edges with static ones."
FORCE: runtime edge capture needs the code run under instrumentation on representative
  workloads (expensive, needs the workloads); static graphs are cheap and "good enough" for
  the file-level recall the field optimizes.
SORT: undiscovered for code-agent DKGs (no runtime-fused graph in the field map; dynamic call
  graphs exist in classic program analysis, never wired into agent retrieval).
FILL: ELAI already captures an action-stream (RRA) + L3 coverage + L4 traces — feed
  runtime-observed call edges back into the DKG as a typed observed_calls edge kind, so
  blast-radius/wiring use static-union-runtime edges and close J-16's named cone
  false-negatives. CONFIDENCE: DENSE.

CONCEPT: C2-CASCADE — retrieval cascade + RRF fusion
FIELD MAP: BM25+reranker (SweRank 2505.07849); hybrid BM25+dense+RRF+cross-encoder;
  code-specific dense (CodeXEmbed/voyage-code-3); AST chunking (cAST); agentic-grep. F-1
  tried-and-failed: heavy embedding-RAG abandoned-as-primary for agent-managed pipelines
  (Anthropic: grep beat it "by a lot"), yet SweRank/SpIDER/LARGER show retrieve+rerank BEATS
  pure grep on localization at ~44x less cost. ELAI firmly on the heavy multi-channel pole
  (BM25->dual-ONNX vector->PageRank->CatRAG->A-RAG, RRF k=60, cross-encoder never pruned).
HIDDEN AXIS: "more retrieval channels over the RIGHT channel" (single-modality competition) +
  "file-level recall over function-precision." ELAI runs ALL channels always.
LACUNA: Cell 2 is field-EMPTY — "which-modality routing" (lexical->grep, structural->graph,
  semantic->dense). ELAI fuses everything unconditionally; never decides "this is an
  exact-symbol lexical query, skip the expensive dense+rerank." (ELAI's planned J-73 is
  WHEN-to-retrieve, not WHICH.)
FORCE: each research group optimizes its one method to win a benchmark; a router admits no
  single method wins (no headline). ELAI runs-all because RRF is simple and the reranker is
  cheap-enough — routing is unmeasured complexity.
SORT: undiscovered (Cell 2 explicitly empty in the field).
FILL: a deterministic query-modality classifier (from the query AST + C-29 intent-pair)
  selecting the channel SUBSET before fusion — grep-only on exact-symbol queries, dense+graph
  on "where is the logic that...". ELAI has the substrate (mode-routing predicate pattern,
  intent-pairs). CONFIDENCE: DENSE.

CONCEPT: C2-RESILIENCE — retrieval resilience (degrade, never crash)
FIELD MAP: Aider tree-sitter->raw-file fallback; grep-agents (no index to fail); hybrid
  pipelines degrade to BM25-only. Cell 1 is field-EMPTY: "no published system has a
  first-class tiered fallback with quality signals." ELAI explicitly occupies it: per-model
  circuit breaker (transient vs permanent), SearchMode cascade HYBRID->SPARSE_ONLY->BM25_ONLY,
  missing-model->DEGRADED not crash, 5s canary, 9-stage QueryTrace, index_version migration.
HIDDEN AXIS: "index freshness as an assumed invariant; degradation = DevOps, avoided in
  research." ELAI is essentially ALONE in the named field on this cell.
LACUNA (ELAI-specific): the degradation signal never reaches the trust-gate. A change authored
  under DEGRADED(BM25_ONLY) — provably missing semantically-related code the dense channel
  would surface — is judged by blast-radius/wiring exactly as a change made under full HYBRID.
  Empty cell = "trust-gate awareness of retrieval-mode degradation."
FORCE: degradation itself is unoccupied (Cell 1 empty), so propagating it into the VERDICT is
  two steps out; and it requires retrieval and verification to share a trust currency, which
  no system has (separate concerns everywhere).
SORT: undiscovered.
FILL: thread retrieval-completeness (QueryTrace/SearchMode) into SSB run metadata; the
  auditor/blast-radius gate reads it — a change made under DEGRADED gets a "context-incomplete"
  trust marker and escalates audit strictness or surfaces to human. ELAI's trust-marker
  precedent exists (J-54 free-pool non-RRA stamp, G-10 suppress-with-notice). CONFIDENCE: DENSE.

================================================================================
WAVE 2 — CLUSTER B (grounded reasoning), CLUSTER D (orchestration), CLUSTER E (context/autonomy)
================================================================================

CONCEPT: C1-GRC — Grounded Reasoning Chain
FIELD MAP: budget-forcing (s1 2501.19393, reasoning_effort); RLVR external-oracle reward
  (but 2504.13837: RLVR doesn't add new reasoning, narrows pass@k); PRMs step-level
  (AgentPRM); claim-citation grounding (eTracer 2601.03669, CiteGuard 2510.17853);
  semantic-entropy+conformal abstention; activation probing (2603.01437: answer predictable
  pre-CoT @0.9 AUC); CoT monitoring; self-consistency; VerifiAgent tool-verify; typed CoT;
  calibrated verbalized confidence; self-verification suppression.
HIDDEN AXIS: "post-hoc faithfulness checking over forced grounding at generation time" +
  "more tokens = better." ELAI deliberately picks forced-grounding-at-generation: built-in
  thinking DISABLED, every mcp_think step MUST carry a grounding citation, finite per-topic
  budget, must_converge. ELAI occupies the field's under-occupied "force grounding during
  generation + bounded budget" cell.
LACUNA (ELAI-specific): GRC forces each step to CITE a grounding ("type:reference") — but the
  citation is self-asserted; not verified to actually entail/support the step. The field's
  killer finding (Correctness != Faithfulness 2412.18004; activation probes; TF-2): a model
  produces a citation that didn't causally inform the step (post-rationalized). Empty cell =
  "verify the grounding link is CAUSAL/entailing, not just present." ELAI verifies CLAIMS
  (C1-GROUNDMODES) but GRC's per-STEP citations aren't run through that verifier.
FORCE: checking every step's citation entails it is expensive (a MiniCheck call per step) and
  the entailment check is itself an LLM (in-boundary; who verifies the citation-verifier).
SORT: tried-and-partial — eTracer/CiteGuard verify claim-citation faithfulness post-hoc
  (12.3% gain, ~human level); the lever exists but is post-hoc/imperfect.
FILL: route GRC's per-step grounding tags through C1-GROUNDMODES' cheapest-sufficient checker
  — a step tagged "type:source" gets its citation verified against the actual source span
  (SOURCE_CHECK is non-LLM, entailing), not just accepted. The lacuna is the COMPOSITION of
  GRC (forces citation) with GROUNDMODES (verifies it) at STEP granularity, currently only at
  CLAIM granularity. CONFIDENCE: DENSE.

CONCEPT: C1-GROUNDMODES — grounding verification + A-E mode router
FIELD MAP: Wave-2/B E-1 is EMPTY: "deterministic verification METHOD selection (not just
  budget scaling) — VerifiAgent does it but with an LLM router; a rule-based dispatch with no
  LLM in the routing path is genuinely underexplored." ELAI occupies it: select_grounding_mode
  is a deterministic predicate, LLM has no input, locked in a registry, sound-checker-first.
HIDDEN AXIS: "letting the model/sibling self-assess how hard to verify." ELAI deliberately
  removes the LLM from routing (an LLM choosing how hard to check itself = the same self-gaming
  problem). ELAI essentially alone on the deterministic-method-router pole.
LACUNA (ELAI-specific): the router is a deterministic predicate over claim FEATURES -> picks
  FILE/SOURCE/TEST/WEB/API. But the predicate is hand-authored/frozen; no meta-check that the
  chosen method was SUFFICIENT for the claim's RISK (a security-critical claim mis-routed to a
  weak FILE check passes a weak gate). Wave-2/B E-4: "no system uses difficulty as a covariate
  in a calibrated verification budget." ELAI routes by claim TYPE, not by risk-calibrated
  sufficiency.
FORCE: assessing whether a method was SUFFICIENT needs the ground truth (did the weak check
  miss what the strong check catches) — circular. The deterministic predicate can only encode
  the authors' priors.
SORT: undiscovered (E-1 cell is ELAI's; the sufficiency-calibration sub-cell E-4 is
  field-empty).
FILL: an OFFLINE calibration loop — periodically re-run a sample of claims through ALL methods
  on a held-out set (D-6), measure method-disagreement per claim-type, tighten the router
  predicate where the cheap method disagrees with the expensive one too often
  (route-by-measured-sufficiency, not by-authored-prior). CONFIDENCE: DENSE.

CONCEPT: C3-AGENTS — agent system + pool + roles
FIELD MAP: MetaGPT SOP roles; OpenAI Swarm handoffs (abandoned for prod); CrewAI hierarchical
  manager (FAILED — degrades to sequential, TDS post-mortem); actor shared-nothing (AutoGen
  v0.4); MAST 14 failure modes (2503.13657: 41-87% failure across 7 frameworks; semantic not
  crash). ELAI = AgentPool + 8 roles + lifecycle/checkpoint/crash-recovery + Erlang
  supervision max_restarts=3 + anti-spiral blake3 3-identical-hash + Tier-C isolation/agent.
HIDDEN AXIS: "more agents over fewer regardless of error amplification" (17x error trap) +
  "verification as social convention not hard gate." ELAI on "few well-supervised isolated
  roles + hard gate"; deliberately rejects the bag-of-agents.
LACUNA (ELAI-specific): Erlang supervision restarts on process DEATH (max_restarts=3). But
  MAST shows most failures are SEMANTIC, not crashes (FM-2.6 reasoning-action mismatch 13.98%,
  FM-1.3 step-repetition, FM-3.x verification). Erlang supervision is blind to an agent that's
  ALIVE but producing wrong work. Empty cell = "supervision that restarts on SEMANTIC failure
  (alive but off-task), not just process death." ELAI's semantic check is the auditor — but it
  fires at AUDITING, AFTER the work, not in-loop.
FORCE: detecting "alive but off-task" mid-run needs a semantic judge watching the agent (an
  LLM; cost; who watches the watcher). Erlang supervision is cheap/deterministic (liveness);
  semantic supervision is expensive. Field (SupervisorAgent 2510.26585) is just starting.
SORT: tried-and-hard (SupervisorAgent; SWE-agent loop-detect ABANDONED on false-positives —
  can't distinguish refined-retry from degenerate-loop syntactically).
FILL: ELAI already has the MAST 14-mode classifier (E-6) but POST-HOC — wire it as an IN-LOOP
  deterministic supervisor over the live SSB action stream (FM-2.6 = stated-plan vs actual-tool
  -calls diverge; FM-1.3 = action_type sequence repetition, semantic not byte) that
  redirects/restarts mid-run, not at audit. The SSB action stream is the ideal substrate.
  CONFIDENCE: DENSE.

CONCEPT: C3-BSP — BSP supersteps (deterministic macro-orchestration)
FIELD MAP: LangGraph Pregel supersteps (same idea); CAID git-worktree isolation; CRDT
  (CodeCRDT — structural convergence, 5-10% semantic incoherence); Atomix transactional
  epoch-gated commits (2602.14849); Compiled-AI (LLM out of runtime, 57x token cut). ELAI =
  BSP supersteps + channel types (LastValue/Topic/Ephemeral/Untracked) + barrier->
  apply-writes-by-SSB-seq (journal seq IS total order) + JoinSet + VRAM scheduling. (RRA
  "cloud lane = reproducible ACTION-stream not tokens" sidesteps the field's bit-replay gap.)
HIDDEN AXIS: "emergent/narrative coordination over deterministic replayable coordination."
  ELAI deliberately picks deterministic (barrier + journal-seq total order = replayable); goes
  further than Pregel (SSB seq gives a replayable total order, not just superstep atomicity).
LACUNA (ELAI-specific): Wave-2/D Cell 3 is EMPTY — "intra-agent barrier synchronization
  (sub-BSP at agent granularity)." ELAI's BSP is at the agent-pool level (across agents);
  within ONE agent's multi-tool-call run there's no superstep barrier. An agent editing file A
  then B in one invocation has no intra-invocation checkpoint/validation between them. Empty
  cell = "BSP-inside-an-agent: checkpoint+validate each tool-call phase within a single agent."
FORCE: the field treats a single agent's tool loop as atomic-ish; intra-agent barriers mean
  checkpointing mid-agent (state explosion) and validating partial work (what gates half-done
  work?). The barrier model is naturally coarse-grained.
SORT: undiscovered (Cell 3 explicitly empty).
FILL: ELAI has H-49 reversible edits (CAS, grains edit/turn/babystep) + edit_checkpoints — use
  the edit/babystep grain as an INTRA-agent superstep boundary: between tool-call phases apply
  a mini-barrier (checkpoint + cheap deterministic validation, e.g. J-52 per-edit LSP daemon)
  before the next phase. Substrate exists; the lacuna is using it as a barrier not just undo.
  CONFIDENCE: THIN (value to ELAI's already-bounded Tier-C agents is moderate; extrapolated).

CONCEPT: C3-PLANMGMT — hierarchical plan management + decomposition
FIELD MAP: AOP pre-dispatch subtask screening (solvability/completeness/non-redundancy reward
  model 2410.02189 — closest to producer-without-consumer prevention); Meta-Agent I/O-contract
  DAG with execution-time re-decomposition (2605.25233); TDAG dynamic re-decomposition; MetaGPT
  SOP assembly-line. Wave-2/D Cell 1 EMPTY: "formally verified task decomposition — no system
  applies formal methods (session types / Petri-net reachability) to guarantee a decomposition
  is complete AND free of producer-without-consumer gaps before execution." ELAI's
  walking-skeleton PRE_GATE ("add fn X with no consumer REJECTED at decomposition") is the
  closest DEPLOYED thing — but a rule, not a formal check.
HIDDEN AXIS: "more agents / social verification." ELAI on "every leaf a verifiable vertical
  slice (producer+consumer+smoke in one unit)"; partially occupies Cell 1.
LACUNA (ELAI-specific): the PRE_GATE rejects bare "add fn X" at AUTHORING (planner LLM + rule),
  not via a FORMAL completeness/reachability check over the DAG. ELAI has contract extraction +
  Kahn cycle detection (catches cycles) but NOT a producer-consumer TYPE-completeness proof
  over the whole tree. Empty cell = "static type-check of the decomposition DAG: every produced
  symbol/artifact has a declared downstream consumer; every consumer's input is produced
  upstream — proven, not rule-enforced."
FORCE: task-graph subtasks carry natural-language I/O specs, not typed interfaces;
  type-checking needs machine-checkable I/O contracts planners don't emit (AOP/Meta-Agent are
  research). The force: "decomposition output is prose, not types."
SORT: tried-in-research (AOP solvability screening, Meta-Agent I/O contracts),
  undiscovered-in-production-coding.
FILL: ELAI's BabyStepSpec is an 8-block typed schema + it does contract extraction — extend the
  contract to a typed produces/consumes interface per node, then run a deterministic
  completeness/reachability check over the tree (its wiring_contract_check producer-consumer
  symmetry, but at PLAN time not code time): any node whose produces has no downstream consumes
  = REJECT, before any code. The runtime analog already exists; lift it to the PLAN DAG.
  CONFIDENCE: DENSE.

CONCEPT: C3-SESSIONDAG — session DAG + context lifecycle
FIELD MAP: LLM-summarization compaction (Claude Code/Codex/OpenCode, lossy); manual handoff
  (Amp, RETIRED 2026); DAG-based lossless trimming (CMV research 2602.22402); demand paging
  (research 2603.09023, 21.8% context recoverable waste); git-worktree isolation; multi-tier
  memory (Mem0/Zep); checkpoint time-travel + forking (LangGraph, Claude Code /rewind —
  CAN'T undo side effects). ELAI = DAG snapshot + 3-pass LOSSLESS trim (stubs recoverable from
  SSB seq = store lossless, view lossy) + branch/merge + 85% handoff + CAS time-travel
  (undo-tree, RV-1 snapshot-before-restore) + regime-aware cloud/local OPPOSITE policy.
HIDDEN AXIS: "lossy summarization over lossless history, everywhere" + "one context policy for
  all backends." ELAI deliberately picks BOTH opposite poles: lossless store/lossy view (the
  CMV research cell, ELAI ships it) AND regime-OPPOSITE policy by backend (the exact thing the
  field says nobody does). Occupies the field's empty Cell A and refutes its hidden axis B.
LACUNA (ELAI-specific): time-travel CANNOT undo external side effects (shell, network, git
  push, API) — the field's killer finding (TF-5; /rewind's "biggest limitation"). ELAI's RV-4
  doctrine NAMES this ("honest irreversibility — surfaced from the saga log, never
  auto-reverted"). But surfacing != handling. Empty cell = "compensating actions for external
  side effects" (saga compensation: git push -> revert; file create -> delete) rather than just
  surfacing irreversibility. ELAI has the saga LOG, no compensation generator.
FORCE: external side effects are not generally invertible (can't un-send an email; a DELETE may
  be unrecoverable); the field punts to ephemeral environments (fresh container/session)
  instead. A correct compensating action is as hard as the original, sometimes impossible.
SORT: tried-and-hard — saga/compensation is a known distributed-systems pattern; Atomix's
  "externalized effects compensated on abort" (2602.14849) is the closest, still research.
FILL: ELAI's Tier-C orchestrator mediates and journals every side-effecting tool — register a
  COMPENSATION per side-effecting tool where invertible (git push -> revert + force-with-lease
  guard, file create -> delete), explicitly mark non-invertible for human (it has the saga log
  + RV-4 honesty). A compensation registry keyed by tool, driving /rewind to actually reverse
  invertible external effects, not just conversation/file state. CONFIDENCE: DENSE.

CONCEPT: C3-AUTONOMOUS — autonomous mode + anti-spiral
FIELD MAP: Codex /goal token-budget loops (stops on test-pass or budget — neither is evidence
  of correct reasoning; 25h/13M-token runaway); learned stuck/milestone classifiers (ModernBERT
  2604.27151, research); deterministic circuit breakers (identical-call/resource caps; the
  $437-bill incident); goal-anchoring/subgoal decomposition; AutoGPT/BabyAGI (FAILED — no
  termination criteria, looped). Wave-2/E Cell B EMPTY: "evidence-gated autonomous self-advance
  — stop when EVIDENCE is absent, not when budget is gone; no production system gates
  self-advance on off-agent evidence." ELAI OCCUPIES Cell B: AutoAdvanceEvaluator runs a
  deterministic EvidenceCheck per transition; advances on EVIDENCE not vibes/budget.
HIDDEN AXIS: "detecting stuckness after the fact over preventing it" + "keeping going over
  stopping." ELAI on "advance on off-agent evidence + tree-risk gating (Concept=HIGH
  always-human) + 4 StagnationPatterns + GoalDriftCanary + 50-baby-step circuit breaker."
LACUNA (ELAI-specific): ELAI has GoalDriftCanary, partially filling the field's empty Cell D
  (goal-drift as a production telemetry signal). But 2026 research (2603.03258/2603.03456) shows
  goal drift is partly a PREFERENCE CONFLICT — "an agent is more likely to drift from a goal its
  values are at odds with" (agents abandon privacy goals for utility). ELAI's GoalDriftCanary
  treats drift as capability/stagnation, not as a value-conflict SAFETY signal. Empty cell =
  "drift detection that distinguishes capability-drift (lost the thread) from preference-drift
  (trained values resist the goal)" — the latter needs escalation, not a persona-switch.
FORCE: preference-drift requires knowing the model's "values" (opaque, the activation-probing
  cell); distinguishing "I forgot" from "I don't want to" is mechanistically hard. The field
  just discovered preference-drift exists; no one detects it in production.
SORT: undiscovered (Cell D nearly empty; preference-vs-capability drift brand-new in literature).
FILL: extend objective-hash sealing (I-45) + GoalDriftCanary — when drift fires, deterministically
  check WHICH constraint was dropped against the sealed typed objective; if a SAFETY/privacy/
  security constraint was dropped (vs a feature), escalate to human as preference-drift (HIGH),
  not persona-switch. The sealed typed objective (I-45) gives the deterministic foothold.
  CONFIDENCE: DENSE.

================================================================================
WAVE 3 — CLUSTER G (reproducibility/taint), CLUSTER H (blast-radius), CLUSTER I (self-improvement)
================================================================================

CONCEPT: C4-RRA — Reproducible-Run Artifact + taint
FIELD MAP: structured trace bundles + deterministic replay stubs (AgentRR 2505.17716);
  Compiled-AI (LLM out of loop, 2604.05150); HDP cryptographic delegation provenance
  (2604.04522); SLSA/in-toto/sigstore AIBOM (only 1 of 30 surveyed agents does crypto
  signing); Faramesh control plane w/ Canonical Action Representation + signed PERMIT/DENY
  (2601.17744); dual-LLM/CaMeL; dynamic taint IFC (RTBAS 2502.08966/FIDES/NeuroTaint);
  AgentArmor program-analysis-on-trace; capability confinement (Progent/Sandlock);
  MemLineage; counterfactual provenance (ARM 2604.04035). ELAI = RRA portable msgpack+zstd
  bundle (SSB stream + seed + HardwareClass + model pin + gate-verdict hashes + blake3 root,
  replay --verify, "cloud lane = reproducible ACTION-stream not tokens") + TAINT G-1
  (Provenance at 4 ingest sites, Provenanced<String>, capability_gate before sinks,
  dependency-screened join; global join REJECTED — collapses utility to 33%).
HIDDEN AXIS: "trusting the agent's own log of what it did" + "reproducibility of TOKENS vs
  reproducibility of EFFECTS." ELAI deliberately picks effect/action-stream reproducibility
  (the sound pole the field's HA-3 says most slide away from) and on taint rejects the coarse
  global join (the utility-collapse trap) for dependency-screened join. ELAI occupies the
  field's near-empty E-1 cell (third-party-verifiable run bundles).
LACUNA (ELAI-specific): the RRA is REPLAYABLE (replay recorded actions, check verdict-hashes
  match) but not RE-EXECUTABLE. It verifies INTERNAL CONSISTENCY (the bundle reproduces its
  own verdicts) not EXTERNAL FAITHFULNESS (the bundle matches what happened on the real
  machine) — and if the recording omitted a side effect (the C0-SSB completeness lacuna), the
  RRA faithfully reproduces an incomplete record. Field's E-4 EMPTY: "effect-level snapshot +
  restore — no system combines the action-stream with an environment snapshot so an external
  verifier can RE-RUN against the same environment and confirm effects match." Empty cell =
  "environment-snapshotted, re-executable RRA, not just re-playable."
FORCE: full environment snapshots per action are heavy (storage); external API state is not
  generally capturable. RRA chose action-stream replay precisely because env snapshot is
  expensive/impossible for live external APIs.
SORT: tried-and-hard — Compiled-AI/Faramesh defer actions, Atomix snapshots INTERNAL effects;
  full re-executable env snapshot is unsolved (external APIs can't be snapshotted).
FILL: ELAI already taint-captures the 4 ingest sites (WebFetch/MCP/file/sandbox) — capture the
  ingested untrusted BYTES into the RRA as a content-keyed VCR cassette (it has J-14 ReproPack
  content-keyed cassette + I-49 fixture planned). That makes the RRA RE-EXECUTABLE against
  recorded external responses, not just replayable. Both halves exist (taint ingest + RRA +
  planned cassette); the lacuna is composing them for re-executability. CONFIDENCE: DENSE.

CONCEPT: C4-BLAST — blast-radius / anti-regression gate
FIELD MAP: SWE-bench PASS_TO_PASS (measured but leaderboard-ignored -> perverse incentive,
  SWE-CI 2603.03823); SWE-CI continuous-regression benchmark (zero-regression rate; most
  models <0.25, Opus 4.6 0.76); TDAD graph impact + targeted test selection (advisory, 70%
  regression cut, 2603.17973); TestPrune (prune suite to context); STING/SWE-ABS
  mutation-guided strengthening (top agents drop 4-16pp); Meta Predictive Test Selection (HARD
  gate, >99.9% catch — but NOT an agent system); Ekstazi/STARTS static RTS (deterministic,
  safety-by-construction — used by NO agent); diff-coverage gating; AgentAssay statistical
  regression for non-det workflows; harness-engineering deterministic outer-loop; SlopCodeBench
  structural erosion; over-editing studies (2601.15195). FIELD CROSS-CUTTING FINDING: "no
  deployed, enforced, deterministic, off-agent gate that computes blast radius, selects
  impacted tests, runs them, and hard-blocks a patch if any fail — for autonomous coding
  agents." ELAI's J-16 (planned KEYSTONE, user's #1) is exactly this: reverse-dep cone +
  diff-scope (over-editing) + regression-test selection (pass->fail=BLOCK) + silent-deletion
  flag, deterministic, LLM never adjudicates, OBSERVE-ONLY first, consumes B-29.
HIDDEN AXIS: "advisory over enforcement" + "running more tests vs reasoning about the change"
  + "trusting the agent to self-limit its diff." ELAI deliberately on enforcement +
  deterministic; occupies the field's empty Cell A.
LACUNA (ELAI-specific): J-16's cone uses B-29's STATIC SINGLE-LANGUAGE impact API. Field Cell D
  EMPTY: "cross-language, cross-layer dependency tracing — TDAD is Python-AST-only; no system
  traces a changed Rust symbol through its IPC boundary into the TS consumer." So J-16's cone
  has false-NEGATIVES on (a) cross-language seams (Rust<->TS Tauri, Rust<->Python sidecar) and
  (b) dynamic dispatch (the C2-DKG static-graph gap) — a change that breaks via a cross-lang or
  dynamic edge the static cone can't see passes the blast gate. Empty cell = "a blast-radius
  cone that spans IPC/cross-language seams + dynamic dispatch."
FORCE: cross-language impact tracing needs a unified cross-language graph (almost nobody has;
  CPGs are single-language, field Cell D empty); dynamic edges need runtime observation. The
  default is static single-language graphs.
SORT: undiscovered (Cell D explicitly empty; no agent does cross-lang impact).
FILL: ELAI uniquely has elai.map (all langs, SCIP-enriched, .rs/.py/.ts/.svelte) +
  cross_lang_diff.py (Rust<->Python, Tauri<->Svelte pairs) + the C2-DKG runtime-edge fill —
  feed elai.map's cross-language edges + runtime-observed edges INTO J-16's cone so blast-radius
  spans IPC/Tauri seams and dynamic dispatch, not just single-language static. All pieces exist;
  the lacuna is composing them into the cone. CONFIDENCE: DENSE.

CONCEPT: C5-RULES — the rules engine (governance as data)
FIELD MAP: OPA/Rego policy-as-code at tool-call layer (Gatekeeper dryrun = shadow); Cedar (AWS
  Bedrock AgentCore, deterministic default-deny, formally analyzable); Microsoft Agent
  Governance Toolkit (YAML+Rego+Cedar, <0.1ms p99); Diogenesis immutable-axiom tier + learnable
  rules; ShieldAgent verifiable rule-circuits (90.1% TPR/4.8% FPR); layered norm-to-control
  (2604.05229). ELAI = rules-as-DATA in SQLite + predicates AST/GREP/SCHEMA/METRIC/COMPOSITE +
  verdicts ALLOW/WARN/BLOCK/ESCALATE + lifecycle PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED
  (promote on fpr<5% + tpr>80% + causal>50%) + mutable=0 IMMUNE tier + >=33 seed rules.
HIDDEN AXIS: "capability-additive rules over measured-lifecycle rules" — the field ADDS rules
  on break, never retires on measured evidence; "shadow->active->retired is desired; most ship
  shadow on paper but lack the measurement infra to promote quantitatively." Field Cell 2 EMPTY:
  "policy lifecycle DB with measured promotion criteria — no system gates promotion on measured
  TPR/FPR on a held-out stream." ELAI OCCUPIES Cell 2 exactly; mutable=0 immune tier = Diogenesis
  pattern + the measured lifecycle Diogenesis lacks.
LACUNA (ELAI-specific): ELAI promotes on measured fpr/tpr/causal — but WHERE does the labeled
  ground-truth come from? Measuring a rule's false-positive rate needs to know which BLOCKS were
  WRONG, which needs an oracle that knows the right answer the rule approximates. If the labeler
  is an LLM judge it's in-boundary (TD-3 violation). Empty cell = "the non-LLM ground-truth
  labeling source for rule fpr/tpr in production."
FORCE: labeling a block false-positive requires an oracle that knows the thing the rule was
  approximating — circular (same force as C1-GROUNDMODES sufficiency); production rarely has
  clean labels; human labeling doesn't scale.
SORT: undiscovered/hard (Cell 2 is empty precisely because the measurement infra/labels don't
  exist anywhere).
FILL: ELAI uniquely can source labels NON-LLM from its own off-agent trust spine — a rule BLOCK
  is a false-positive if the SAME change later passed all the deterministic gates (wiring/
  cant-lie/mutation/blast) and shipped with no downstream regression (caught by the regression
  gate); derive fpr/tpr from the OUTCOME recorded in the SSB journal, not an LLM judge. The
  append-only journal IS the labeled outcome stream. The lacuna is wiring rule-lifecycle
  promotion to the journal's recorded downstream outcomes as the label source. CONFIDENCE: DENSE.

CONCEPT: C5-SELFMOD — self-modification core + DGM safety
FIELD MAP: DGM empirical archive evolution (Sakana 2505.22954, fitness=external benchmark,
  acknowledges benchmark-gaming risk, NO explicit anti-reward-hacking beyond benchmark
  comprehensiveness); DARWIN genetic; MOSS production-failure-anchored fitness + health-probe
  rollback (only one anchoring fitness to non-gameable live failures); STOP/STO (DOCUMENTED
  editing OUT its own budget constraints, 2310.02304); MAV cross-family verification
  (cross-family > intra-family > self, unambiguous, 2502.20379); formal-proof immune fitness
  (Lean/AlphaVerus); EvoFSM structural constraint. KEY: alphaT->0 PROVABLY collapses (2601.05280
  entropy-decay + variance-amplification); RewardHackingAgents 50% evaluator-tampering in mutable
  workspaces (2603.11337); alignment-tipping (safety erodes under self-evolution, 2510.04860).
  ELAI = L1 data-tables(no LLM)/L2 config(10/50/200 staged)/L3 structural=human-only; DGM sigmoid
  parent-select; AllowedConfigKey CLOSED enum; auto-revert >10% over 3; Track-7 (H-43..H-48)
  replaces the two LIVE placeholders: fitness=blake3(config bytes) -> execution-grounded fitness;
  trap=keyword-grep -> behavioral trap w/ MEASURED recall; + CROSS-FAMILY verifier (D-13) +
  worst-case/lineage-diverse promotion + eval firewall (H-47).
HIDDEN AXIS: "self-assessed fitness over execution-grounded" + "same-model self-critique over
  cross-family." ELAI deliberately on BOTH correct poles; its VD doctrine literally names the
  field's two cardinal sins (fitness=bytes/self-confidence; same-model self-critique) as
  never-reintroduce. ELAI is the field's prescription made into doctrine — one of its most-covered
  concepts.
LACUNA (ELAI-specific): cross-family verifier (D-13/VD-4) needs a DIFFERENT pretraining family
  than the generator. The cloud lane has it (gpt-bridge: GPT/Grok/Gemini). But the LOCAL/offline
  self-mod lane is Qwen-only with ~1.8GB free VRAM — it CANNOT co-resident a second family
  (ELAI's own feature map rejects RecursiveMAS for exactly this), so a local self-mod loop has
  only same-family self-critique, the banned configuration. Empty cell = "cross-family
  verification when only ONE model family is available locally."
FORCE: cross-family needs >=2 independent families resident; the local box physically can't
  (VRAM). MAV proves intra-family/self is structurally weakest, but the second family is
  unaffordable on the constrained lane.
SORT: tried-and-hard (MAV proves the necessity; the resource constraint is real and ELAI-noted).
FILL: for the local lane, substitute cross-FAMILY with cross-COMPUTATIONAL-FAMILY — a
  deterministic non-LLM verifier (H-43 execution-grounded fitness on frozen tests + J-68
  Kani/TLA formal checker) IS a different computational family than the generator and is
  un-gameable (the field's "immune fitness function" pole). Explicitly designate the deterministic
  checker stack as the local-lane verifier-stronger-than-generator and forbid local L2 promotion
  without it. ELAI nearly has this (H-43 + J-68); the lacuna is naming/binding the
  checker-as-cross-family-substitute for the resource-constrained lane. Secondary check: ensure
  the mutable=0 immune-tier MEMBERSHIP lives in the frozen gate-manifest, not in mutable L1 data
  (else L1 self-mod could re-flag a safety rule as mutable). CONFIDENCE: THIN (ELAI is largely
  ahead here; this is a reframe + an edge-case hardening, not a wide-open gap).

================================================================================
WAVE 4 — CLUSTER J (security/write-gate), CLUSTER K (inference/routing), CLUSTER L (surfaces)
================================================================================

CONCEPT: C6-VIBEGATE — the unified write gate + dependency defense
FIELD MAP: PreToolUse hooks (Claude Code, deterministic, opt-in); pre-execution firewall proxy
  (AEGIS 2603.12621 / AgentTrust 2605.04785 / Invariant Gateway — regex+schema+YAML, LLM-judge
  only OUT of hot path); capability IFC/taint (CaMeL 2503.18813 / Fides); static SAST+secret
  (LlamaFirewall CodeShield, VibeGuard 2604.01052); registry verification for deps (Socket/
  Aikido; Spracklen slopsquatting 19.7%, 51% pure fabrications); masked re-execution (MELON);
  MCP schema allowlist (mcp-scan); classifier injection guard (PromptGuard2); sandbox;
  human-approval escalation. ELAI = VibeGate fail-closed multi-axis PreToolUse (code-vuln
  AST/GREP + secret GREP Tier-1; dep-existence + skill-scan Tier-2; mutable=0 "vibesec" domain)
  + HALLUCINATED-DEP gate (existence+age+provenance at WRITE-time incl cargo) + CaMeL data-plane
  + hooks T1<1ms/T2<10ms tree-sitter/T3<5ms NO LLM in hot path.
HIDDEN AXIS: "detect-after vs block-before (SIEM mentality)" + "trusting the name the model
  emitted" + "outsourcing the guard to another LLM." ELAI deliberately on block-before
  (fail-closed PreToolUse, no LLM in hot path) + verifies the dep name at write-time +
  deterministic. ELAI occupies the field's strongest pole on all three; it also occupies the
  field's empty Cell 1 (write-time-import hallucinated-dep detection, vs install-time elsewhere).
LACUNA (ELAI-specific): field Cell 2 EMPTY: "integrity-labeled data flow that survives multi-step
  agent MEMORY — taint works within ONE context window; once stored to memory + retrieved in a
  new session, the label is LOST; no system tracks integrity labels across session/memory/
  delegation boundaries" (persistent memory-poisoning, verified real for Bedrock). ELAI's G-1
  taint is Provenanced<String> threaded WITHIN a run, but does it survive into crates/memory
  (E-5) and across the H-51 cross-session handoff? Not stated. Empty cell = "taint labels that
  survive the memory write->retrieve->inject cycle and cross-session handoff."
FORCE: taint systems reset labels at session/memory boundaries because the memory store (often a
  vector DB) is label-blind; threading the IFC label through serialize->store->retrieve->reinject
  requires every hop to carry it, and most memory subsystems don't.
SORT: undiscovered (Cell 2 explicitly empty; ELAI has J-21 memory-poisoning hardening + MINJA
  partial, but taint-LABEL persistence specifically isn't stated).
FILL: thread G-1 Provenance THROUGH the crates/memory write path (store the label with the fact)
  and the H-51 handoff digest (carry labels across sessions), so capability_gate sees the taint
  of a memory-retrieved fact, not just a same-run ingest. Pieces exist (G-1 + crates/memory +
  J-21 + MINJA); the lacuna is composing G-1's label with the memory lifecycle. CONFIDENCE: DENSE.

CONCEPT: C7-SIDECAR — the local Python sidecar
FIELD MAP: vLLM (PagedAttention/continuous batching); SGLang (RadixAttention prefix-share);
  llama.cpp/Ollama/GGUF (consumer/edge; Ollama serializes at concurrency); quantization NF4/
  AWQ-Marlin/FP8 (FP8 Hopper-only, Ampere=INT8/INT4); XGrammar structured decode (<40us/token);
  batch-invariant deterministic inference (Thinking Machines; NO framework ships it; temp=0 NOT
  deterministic due to BF16 non-associativity, 2506.09501); activation/logits steering (local-
  only, weight access). ELAI = FastAPI Anthropic-compat /v1/messages + /v1/steering, generate()
  + LogitsProcessor chain (DoLa/ETC/prompt-lookup/XGrammar), REMOTE 3090 canonical, Qwen2.5-Coder
  NF4, FA2 mandatory on 7B (FA3/4 Hopper-blocked), PrefixKVCache BYPASSED on streaming path.
HIDDEN AXIS: "throughput/latency over determinism and auditability — NO framework ships
  batch-invariant kernels by default; temp=0 non-determinism is a silent correctness hazard for
  agents producing diffs." ELAI deliberately on the determinism pole (ELAI_SEED, RRA, J-13
  batch_invariant_ops + SeededGumbel) and uses the local-only steering path the field underuses.
  ELAI partially occupies the field's empty E-1 (deterministic-by-default local serving).
LACUNA (ELAI-specific): J-13 covers mm/addmm/log_softmax batch-invariance but EXPLICITLY NOT
  attention (SDPA/FA2 "NOT covered — graded opt-in, honest residue") — and attention is where BF16
  non-associativity bites hardest (the field's root cause). Empty cell = "batch-invariant ATTENTION
  on the constrained Ampere/FA2 lane." ELAI's own residue names it.
FORCE: HARDWARE — batch-invariant attention needs FP32 attention (2x mem/latency, infeasible on
  ~1.8GB-free 3090) or a custom batch-invariant FA kernel; the Thinking-Machines/Hopper kernels
  don't cover FA2/Ampere, and FA3/4 are Hopper-only (ELAI Ampere-blocked).
SORT: tried-and-hard (batch-invariant kernels exist Hopper-class; LayerCast 2506.09501 lighter,
  unshipped); Ampere/FA2 batch-invariant attention is blocked.
FILL: since the kernel is hardware-blocked, make the determinism CLAIM lane-accurate — bind the
  attention-determinism residue to the J-14 ReproPack tier so the local lane records token-faithful
  (T1) replay ONLY when attention is batch-invariant, else downgrades to action-faithful (T0). This
  is exactly the CL-4/GD-5 doctrine ("never advertise a guarantee a path can't deliver"). The
  lacuna is wiring the attention residue into the RRA tier so the local lane never over-claims.
  CONFIDENCE: DENSE.

CONCEPT: C7-ROUTING — multi-runtime backend + cloud routing
FIELD MAP: RouteLLM preference-router (binary strong/weak, 75-85% cost cut); FrugalGPT cost
  cascade (double-billing penalty); architect/editor role-routing (Aider); domain/task-feature
  routing (Triage code-health, BEST-Route 60% cut <1% drop); API-unifying gateway (LiteLLM).
  ELAI = RuntimeAdapter trait + 11 backends + PerRoleRuntime + CloudModeRouter HARD cost ceiling
  (blocks not warns) + EscalationCascade local->cloud + H-7 calibrated routing (architect/editor/
  verifier) + J-55 model-strength modality router (CodeAct vs constrained-call, planned KEYSTONE).
HIDDEN AXIS: "query-text signals over task-structural signals" + "cloud-frontier as the routing
  target / local-only strong-side baseline NOT measured — cost savings are relative to a billing
  ceiling, not relative to what a capable local model could eliminate." ELAI deliberately
  local-first (EscalationCascade, VRAM-tier, the whole sidecar lane) and routes by task-structural
  signals (role/VRAM-fit/modality). ELAI occupies the field's neglected local-baseline pole.
LACUNA (ELAI-specific): field E-4 EMPTY: "task-phase-aware routing WITHIN a single agentic trace
  — plan=architect-class, retrieval=long-context, edit=cheap, verify=cheap; no productized system
  switches the model mid-trace by detected phase; Aider's architect/editor is a static
  approximation." ELAI routes per AGENT ROLE (PerRoleRuntime/H-7) but role != phase; within one
  agent's run (plan->read->edit->verify) the model is fixed. Empty cell = "phase-keyed mid-trace
  re-routing."
FORCE: switching models mid-trace busts the byte-stable cache prefix (ELAI's own HD-2 doctrine
  forbids busting it) and adds re-prefill/handoff cost; the field's static role-split exists
  precisely because mid-trace swaps are cache-expensive.
SORT: undiscovered (E-4 explicitly empty).
FILL: use the FSM phase + the proposed C3-BSP intra-agent superstep boundary as the routing key —
  at the superstep barrier (a natural cache breakpoint, so HD-2-safe), J-55 ModalityRouter +
  PerRoleRuntime re-resolve the backend for the next phase, condensing context via I-11 PABU to
  bound re-prefill cost. Pieces exist (FSM phases, J-55, I-11); the fill must only switch at a
  cache breakpoint or it violates HD-2 — that tension is what makes it risky. CONFIDENCE: THIN
  (cache-bust economics may not pay off; gated by the C3-BSP intra-agent barrier first).

CONCEPT: C8-DASHBOARD — dashboard-v2 + the activity orbit
FIELD MAP: external trace-store platforms (LangSmith/Langfuse/Arize/Braintrust — a SECOND store,
  can drift on dropped spans); framework-native dashboards (OpenAI Traces, AutoGen Studio,
  OpenHands); APM-native (Datadog/Honeycomb); OTel GenAI semconv (interop, still Development not
  Stable); governance-enforcement observability (Zenity, GAAT 2604.05119); event-sourced-log-as-
  truth (ActiveGraph "the log is the agent" 2605.21997 — research only); Devin metrics. ELAI =
  Tauri+Svelte+three.js, SsbViewer READ-ONLY SQLite WAL poll, engine crates deliberately OUT of
  the dashboard Cargo.toml, 3D orbit IS the activity feed.
HIDDEN AXIS: "external store vs the projection problem — almost every platform is an external
  store receiving async traces and can drift; ActiveGraph is the ONLY source framing the dashboard
  as a deterministic PROJECTION of the authoritative log." ELAI deliberately on the projection
  pole (read-only view over the SSB, engine crates kept OUT so it CAN'T write); occupies the
  field's empty Cell A that only the ActiveGraph research prototype else occupies.
LACUNA (ELAI-specific): field Cell B EMPTY: "observability that distinguishes PLAN DRIFT from
  execution failure — no platform ships a 'stated plan vs executed steps' diff view." ELAI has the
  substrate (sealed objective I-45, plan skeleton, SSB action stream, GoalDriftCanary, plan-status-
  from-journal I-39) but the dashboard doesn't render a plan-vs-execution diff. Empty cell = "a
  dashboard projection diffing the agent's sealed/stated plan against its journaled executed
  actions" (the visual surfacing of the C3-AUTONOMOUS drift lacuna).
FORCE: rendering drift needs the plan to be a persisted machine-comparable artifact (most systems'
  plan is prose in the trace); dashboard teams optimize cheap-visible aggregates (HA-3 passive-
  observation creep: dashboards show green while agents loop).
SORT: undiscovered (Cell B explicitly empty field-wide).
FILL: render a read-only projection diffing planned units (plan skeleton / sealed objective I-45)
  against journaled executed actions, flagging skipped/substituted/drifted steps and consuming the
  C3-AUTONOMOUS constraint-class-diff fill (safety-class drift = red). Pure view over the SSB,
  consistent with ELAI's view-not-store doctrine. CONFIDENCE: DENSE.

CONCEPT: C8-CANVAS — the canvas workbench
FIELD MAP: canvas-as-runtime no-code (n8n/Langflow/Flowise/Dify — canvas IS execution, ~20-node
  complexity ceiling); canvas-as-composition over a code runtime (Sim Studio); generative-pipeline
  canvas (ComfyUI); model-driven canvas + human pointing (Claude Design, Orca CHI'26); deixis
  research. ELAI = 2D work-surface the model drives + user points at; MOAT = journaled reproducible
  taint-tagged op-stream; bindings+verdicts persist ONLY as SSB rows (journal is store, canvas
  re-derives, no parallel state); build dispatches through the FSM — NO canvas-side runtime (the
  anti-n8n line). J-31 system mode binds nodes to plan-units/wiring-contracts with verdicts from
  checker+observable_effect rows; J-60 layered observability (semantic-zoom altitude); J-61
  redirect loop (journaled CanvasRedirectIntent); H-15 taint-gated deixis.
HIDDEN AXIS: "canvas-as-runtime vs canvas-as-view confusion — n8n/Langflow treat the canvas as
  both definition AND runtime, hence the complexity ceiling (the canvas can't be richer than what
  it visually represents); the anti-n8n design (canvas reflects a code-first runtime) is the
  escape." ELAI deliberately on the anti-n8n pole (canvas is a re-derived VIEW, build goes through
  the FSM); it occupies the field's escape cell and (via J-31/J-60/J-61/H-15) already covers
  deixis-in, model-drives-out, journaled steering, and the visual ceiling (altitude/zoom). This is
  one of ELAI's most-covered concepts.
LACUNA (ELAI-specific, derivative): the canvas re-derives from the SSB, so it inherits the C0-SSB
  journal-COMPLETENESS gap — a canvas op-node shows what was journaled; a side-effect that left no
  row (the completeness lacuna) is INVISIBLE, giving a false sense of "this is everything that
  happened." The canvas's trustworthiness is bounded by journal completeness. Empty cell = "canvas
  surfacing of un-journaled/shadow-stream activity so the view doesn't imply a completeness it
  can't deliver."
FORCE: the canvas can only render journal rows; un-journaled side effects are invisible by
  construction (same force as C0-SSB) — and rendering "what's missing" requires the independent
  shadow stream that doesn't exist yet.
SORT: undiscovered (derivative of the C0-SSB completeness lacuna).
FILL: once the C0-SSB shadow-stream completeness gate exists, render shadow-stream side-effects
  with no matching op as ANOMALY nodes on the canvas — the view then honestly shows both the
  journaled op-stream AND the gaps. CONFIDENCE: THIN (derivative of C0-SSB; C8-CANVAS is otherwise
  well-covered by J-31/J-60/J-61/H-15).

================================================================================
WAVE 5 — CLUSTER M (doctrine + plan-corpus)
================================================================================

CONCEPT: C9-DOCTRINE — the doctrine families (binding laws of plan authoring)
FIELD MAP: prose rule files (AGENTS.md/CLAUDE.md, probabilistic, 60K+ repos); lifecycle hooks
  as deterministic gates (Claude Code/Kiro); ContextCov (COMPILE prose rules -> executable AST/
  shell/arch checks; 88.3% compliance vs 50.3% prose, 2603.00822); harness-engineering ratchet
  (Hashimoto — each mistake -> structural prevention; "feature list is JSON not Markdown because
  agents rewrite prose"); NLAH executable harness rep; formal-verification gates; AI-SDLC
  protocol DSL (policy-vs-mechanism separation); Microsoft Agent Governance Toolkit. ELAI =
  named citable doctrine families W/GD/HD/TD/RD/VD/RV/CL/ID-IC/JD, PARTLY prose (.md rules) and
  PARTLY executable (wiring_contract_check / production_caller_check / maintainability_check /
  traced_test_assert_check / gate_freeze_check, the frozen gate-manifest R8).
HIDDEN AXIS: "prose the model may follow vs machinery the model cannot bypass" — the field STARTS
  prose (cheap) and MIGRATES to machinery because prose fails under context pressure (ETH Zurich:
  LLM-gen context files REDUCE success 3% +20% cost, 2602.11988; OctoBench: systematic
  task-solving-vs-scaffold-compliance gap, 2601.10343; ContextCov 50.3% prose vs 88.3% compiled).
  ELAI deliberately on the machinery pole for load-bearing invariants (frozen deterministic
  checkers + freeze-the-gate) while keeping prose as the human-readable layer. ELAI is the
  field's prescription (compile invariants into off-agent gates) made into a discipline.
LACUNA (ELAI-specific): which doctrine families are ENFORCED by a frozen checker vs only by prose?
  W/maintainability/tracing-test have executable checkers; but GD (compose-not-restate), HD
  (cache/efficiency), RD (reasoning), much of ID/IC appear PROSE-ONLY — enforced by the auditor
  reading the rule (~50% compliance per ContextCov). Field Cell E-2: "machine-readable invariants
  with a lifecycle" is empty. Empty cell = "a doctrine-ENFORCEMENT coverage ledger: per clause,
  is it frozen-checker-enforced / auditor-advisory / unenforced, and what % of binding doctrine
  is prose-only (i.e. at ~50% compliance risk)."
FORCE: the fuzzy families (GD-1, HD-aesthetic, RD-reasoning-quality) resist deterministic checking
  (Stability-Trap: 35-45% even for LLM-judge stability) — they're prose-only BECAUSE they're not
  mechanically checkable (same undecidability force as C1-GROUNDMODES/C5-RULES; TD-3 routes them
  to LLM-advisory). Some doctrine is irreducibly judgment.
SORT: tried-and-partial (ContextCov compiles many prose rules at 88% compliance; the residue is
  the genuinely-fuzzy families).
FILL: maintain an explicit DOCTRINE-ENFORCEMENT LEDGER — per clause, tag {frozen-checker /
  auditor-advisory / unenforced} and compute a coverage metric (% of binding clauses with an
  executable gate); surface the prose-only clauses (at ~50% compliance risk) for ContextCov-style
  compilation. ELAI's own "measure the real objective" turned on its rulebook; it has the checker
  infra + gate-manifest, the lacuna is the coverage ledger over the doctrine itself.
  CONFIDENCE: DENSE.

CONCEPT: C9-PLANS — the plan-letter map (navigating the corpus)
FIELD MAP: linear markdown phase files (BMAD 655 files, hand-maintained status, sprawl);
  spec-driven four-phase cycle (GitHub Spec Kit / OpenSpec / Kiro — status in hand-maintained
  checkboxes/issues, NO auto code-to-spec reconciliation); skills-as-packages w/ evaluated
  lifecycle (Tessl); event-sourced/git-derived status (Git Context Controller 2508.00031, Kage —
  status from git log not hand-annotation); Codified Context hot/cold memory (retrieve on-demand,
  not load-all, 2602.20478). ELAI = plan-letter map (base 0-4, B..J) + SSB SCHEMA_VERSION/
  TOTAL_VARIANTS counters as the live "where are we" stamp ("grep ssb.rs, never trust a doc
  number") + ELAI-FEATURE-MAP.md living source-of-truth + I-39 "plan status DERIVED from the
  journal, kills hand-maintained drift."
HIDDEN AXIS: "hand-maintained plan status vs status derived from the artifact graph" — every major
  SDD tool tracks status in hand-maintained checkboxes; code doesn't update the plan; spec written
  once then diverges; no major public SDD tool auto-derives completion from git diff/tests. ELAI
  deliberately on the derived pole (I-39 derives status from the journal; counters grepped from
  source; feature-map upsert uses wiring/audit gate verdicts as ground truth, never executor
  self-assessment). ELAI occupies the field's empty derived-status pole.
LACUNA (ELAI-specific): I-39 derives plan STATUS, but the feature-map ITSELF is HAND-UPSERTED
  (its header literally lists STALE counters: "191/464, 207/484... every other number is STALE")
  and its content (composition edges, status flags) is agent-maintained prose with the documented
  "agents rewrite prose specs inappropriately" risk. Field Cell E-1 ("spec freshness as a
  first-class CI metric") empty. Empty cell = "the feature-map's STATUS + COMPOSITION layers
  derived from the journal + code, not hand-upserted" (I-39 does status; not the composition graph
  or the narrative).
FORCE: composition edges + the "why" narrative are SEMANTIC (design intent: which feature serves
  which goal), not derivable from the journal (the SSB records WHAT happened, not DESIGN INTENT).
  The narrative layer is irreducibly hand-authored; the numeric/status layer IS derivable and
  ELAI does it.
SORT: tried-and-partial (I-39 + Git Context Controller/Kage derive STATUS; no system derives the
  composition/intent graph — that's design knowledge).
FILL: auto-derive the MECHANICAL half of the feature-map — status flags from I-39 journal +
  SCHEMA/TV from a grep + composition EDGES from elai.map/codegraph wiring contracts (consumes/
  produces) — leaving ONLY the design-intent "why" hand-authored. Shrinks the stale-prone
  hand-maintained surface to the irreducible intent narrative. ELAI has I-39 + elai.map +
  codegraph + wiring contracts; the lacuna is auto-deriving the status+composition layers.
  CONFIDENCE: DENSE.

================================================================================
FINAL SYNTHESIS
================================================================================

THE META-FINDING (the single most important output of this sweep):
ELAI is, across nearly every concept, the system that OCCUPIES THE EMPTY CELL. On concept after
concept the grounding agents independently reported a field-wide empty cell or a hidden axis the
whole field slides along, and ELAI is sitting on the contrarian pole by deliberate design:
  cheap-green-signal -> CAUSAL connection (C0-THESIS, C4-WIRING)
  snapshot-of-state -> append-only EVENT LOG (C0-SSB)
  ordering-as-DX -> PROVABLE ordering (C0-FSM)
  trust-self-report -> OFF-AGENT artifact (C0-TIERC, C4-CANTLIE, C4-RRA)
  advisory warnings -> ENFORCED deterministic gates (C4-BLAST, C6-VIBEGATE)
  prose rules -> compiled MACHINERY (C9-DOCTRINE)
  lossy summarization -> LOSSLESS store / lossy view (C3-SESSIONDAG)
  hand-maintained status -> DERIVED-from-journal status (C9-PLANS)
  canvas-as-runtime -> canvas-as-VIEW / anti-n8n (C8-CANVAS)
  external trace store -> read-only PROJECTION (C8-DASHBOARD)
  self-assessed fitness -> EXECUTION-GROUNDED fitness + cross-family verifier (C5-SELFMOD)
  add-rules-on-break -> MEASURED-lifecycle rules (C5-RULES)
  index-freshness-assumed -> DEGRADE-gracefully first-class (C2-RESILIENCE)
  LLM-chooses-verification -> DETERMINISTIC method router (C1-GROUNDMODES)
Consequence: ELAI has very few FIELD-WIDE lacunae left that it is not already aimed at. Its real
remaining lacunae are of two recursive shapes:
  (1) COMPOSITION gaps — ELAI built both halves and has not wired them together.
  (2) ONE-LEVEL-UP gaps — ELAI secured layer N, so the residual untrusted-ness moved to layer N+1
      ("who guards the guard" applied recursively): trust the execution -> now the OBJECTIVE is
      unverified (C0-THESIS); complete the journal -> now journal COMPLETENESS is unproven
      (C0-SSB); the FSM gates the worker -> now the ORCHESTRATOR that runs the FSM is ungated
      (C0-FSM); verify the change is wired -> now whether it's wired on a REAL-USAGE path is
      unverified (C4-WIRING); verify the result is honest -> now the test's MEANINGFULNESS is
      unverified (C4-CANTLIE).

RANKED REAL LACUNAE (force named; THIN/boring excluded from the top tiers).
Ranking = (a) load-bearing-ness of the concept x (b) DENSE confidence first.

TIER 1 — load-bearing root/spine/heart, DENSE, act first:
  L1. C0-THESIS  spec/objective-correctness gate (Goodhart at the spec layer). The root concept's
      own blind spot: ELAI proves the agent honestly satisfied the objective, never that the
      objective is right. FILL: spec-as-taint-tracked-evidence-bound artifact + a cross-family
      "is this the right objective" critic distinct from the auditor. [FIELD-WIDE]
  L2. C0-SSB  journal COMPLETENESS via an independent shadow (eBPF/FS) stream + reconcile gate;
      an un-journaled side-effect is currently invisible and the append-only log can't know it's
      incomplete. The entire trust spine rests on the journal being complete. [FIELD-WIDE]
  L3. C4-RRA  RE-EXECUTABLE (not just replayable) run bundle — capture the taint-ingest bytes as a
      content-keyed VCR cassette (J-14) so a third party can re-run, not just replay-and-trust.
      This is the WEDGE/GTM artifact; "reproducible action-stream" currently means replayable, not
      re-executable. [FIELD-WIDE]
  L4. C4-WIRING  usage-weighted reachability — fuse the living eval corpus (E-54/J-56) into the
      wiring gate so reachability is measured over representative workloads, not one happy smoke.
      [FIELD-WIDE]
  L5. C4-BLAST  cross-language + dynamic-dispatch blast cone — feed elai.map cross-lang edges +
      runtime-observed edges (C2-DKG fill) into J-16's cone; today it's single-language static and
      blind to IPC/Tauri seams + dynamic dispatch. User's stated #1 priority. [FIELD-WIDE]
  L6. C0-FSM  bind the ORCHESTRATOR's own dispatch order as first-class FSM substates (not prose +
      Stop-hook). ELAI's own logged P0 incident: the controller rationalizes past dispatch order.
      [ELAI-SPECIFIC — the field's POLARIS/hooks bind dispatch]
  L7. C4-CANTLIE  compose can't-lie with a mutation-survival floor on the diff cone (C-28 + J-16);
      a vacuous test passes truthfully and the hash matches truthfully. [FIELD-WIDE-adjacent]
  L8. C3-PLANMGMT  formal producer->consumer TYPE-completeness check over the plan DAG (lift
      wiring_contract_check's symmetry to PLAN time) — prevention-grade, beats detection.
      [FIELD-WIDE — Cell 1 empty]

TIER 2 — important, DENSE:
  C0-TIERC  machine-derived-first audit input (comment-stripped AST diff for the binary verdict;
    LLM prose advisory). [ELAI-SPECIFIC]
  C3-AGENTS  in-loop MAST supervisor over the live SSB stream (E-6 classifier is post-hoc); restart
    on SEMANTIC failure, not just process death. [ELAI-SPECIFIC]
  C6-VIBEGATE  taint labels that survive the memory write->retrieve->inject cycle + H-51 handoff
    (persistent memory-poisoning). [FIELD-WIDE — Cell 2 empty]
  C1-GRC  verify each per-step grounding tag is ENTAILING via C1-GROUNDMODES' deterministic
    checker (compose GRC + GROUNDMODES at step granularity). [ELAI-SPECIFIC]
  C3-AUTONOMOUS  distinguish preference-drift from capability-drift via constraint-class diff vs the
    sealed objective; route safety-class drift to human. [FIELD-WIDE — Cell D barely explored]
  C3-SESSIONDAG  compensation registry for invertible external side-effects (git push->revert),
    explicit non-invertible->human; today ELAI surfaces irreversibility but doesn't compensate.
    [ELAI-SPECIFIC]
  C5-RULES  source rule fpr/tpr LABELS non-LLM from journal OUTCOMES (a BLOCK was a false-positive
    if the same change later passed all deterministic gates + shipped without regression).
    [FIELD-WIDE — Cell 2 empty]
  C2-DKG  fuse runtime-observed call edges into the DKG (observed_calls), closing dynamic-dispatch
    false-negatives that break blast-radius. [FIELD-WIDE] (enabler for L5)
  C9-DOCTRINE  doctrine-enforcement coverage ledger (which clauses are checker-enforced vs prose-
    only at ~50% compliance). [FIELD-WIDE — E-2 empty]
  C9-PLANS  auto-derive the feature-map's status + composition layers from journal + elai.map,
    hand-author only design intent. [ELAI-SPECIFIC]

TIER 3 — DENSE but narrower load-bearing:
  C1-GROUNDMODES route-by-measured-sufficiency calibration; C2-CASCADE which-modality retrieval
  router; C2-RESILIENCE retrieval-degradation -> trust-gate marker; C7-SIDECAR bind attention-
  determinism residue to the RRA tier (honesty fix); C8-DASHBOARD plan-vs-execution drift view.

THIN (extrapolated from sparse pattern; verify before acting — flagged honestly):
  C3-BSP intra-agent superstep barrier; C7-ROUTING phase-keyed mid-trace routing (tension with the
  HD-2 cache-prefix doctrine — only safe at a cache breakpoint); C5-SELFMOD local-lane
  checker-as-cross-family substitute (ELAI largely already ahead here); C8-CANVAS surface
  un-journaled shadow activity (derivative of the C0-SSB fill).

FIELD-WIDE vs ELAI-SPECIFIC (separated, per the brief):
  FIELD-WIDE lacunae ELAI is uniquely positioned to fill (nobody in the field occupies these, and
  ELAI's substrate — append-only journal, off-agent gates, RRA, eval corpus, cross-language map —
  is exactly what's needed): C0-THESIS spec-gate, C0-SSB completeness, C4-RRA re-executable,
  C4-WIRING usage-weighted, C4-BLAST cross-lang+dynamic, C3-PLANMGMT plan-DAG type-check,
  C6-VIBEGATE taint-through-memory, C5-RULES journal-outcome labels, C3-AUTONOMOUS preference-drift,
  C2-CASCADE modality router, C2-RESILIENCE degradation-trust-marker, C8-DASHBOARD plan-exec drift,
  C9-DOCTRINE enforcement-coverage. These are the highest-leverage: ELAI could be FIRST.
  ELAI-SPECIFIC lacunae (the field has a solution ELAI's design hasn't adopted/composed): C0-FSM
  orchestrator-dispatch binding (POLARIS/hooks), C0-TIERC machine-derived audit (ESAA), C1-GRC
  citation-entailment (eTracer/CiteGuard), C3-AGENTS in-loop supervisor (SupervisorAgent),
  C3-SESSIONDAG compensation (Atomix), C9-PLANS derived feature-map (Git Context Controller).

ONE-LINE TAKEAWAY: ELAI has already built the off-agent, fail-closed, journal-grounded trust
substrate the 2025-2026 field is independently converging toward (the grounding agents kept
naming, as the field's empty cell, the exact thing ELAI ships). Its remaining real lacunae are not
"missing capabilities" but recursive residue — every trust layer it closed pushed the untrusted-ness
up one level (objective, journal-completeness, usage-representativeness, the orchestrator itself) —
plus a handful of composition gaps where both halves exist unwired. The highest-value next moves are
TIER 1: the spec-correctness gate (L1), journal completeness (L2), and the re-executable RRA (L3).

