# ELAI — Systems Overview (concept map across the phase specs + B..M plans)

Purpose: read this instead of the 13 summary indexes (phase-index-MASTER + B..M) to know what every
ELAI system IS and how it works. Concepts only — no wiring/shipped status, no schema versions, no
session logs, no effort estimates. ELAI (`.elai_cc`) is a deterministic, Rust-first autonomous coding
agent: a knowledge engine + reasoning engine + orchestrator/FSM + sandbox + self-improvement loop +
a trust/verification spine + a Tauri dashboard, drivable on local (sidecar) or cloud (Anthropic) models.

The organising idea across all letters: push decisions out of model discretion into deterministic
harness machinery, leave the model only generation/selection, and gate even those against frozen,
measured, off-agent evidence. Every claimed number is measured from the system's own event journal;
every gate reads an artifact the generating agent did not author; missing evidence is a hard fail.

---

## 1. Foundation & determinism

- Entropy quarantine: no raw `std::time` / `thread_rng`. All time comes through a `ClockProvider`
  (OsClock in prod, SeededClock in test) and all randomness through an `EntropySource` (OsRng /
  SeededRng). A `RuntimeContext` bundles them. `ELAI_SEED` + single-threaded runs give byte-identical
  output across runs — the substrate everything reproducible rests on.
- Anti-stale-training stack: blake3 (never blake2/sha2) for hashes/cache-keys/fingerprints; tracing
  (not log); thiserror/anyhow (not Box<dyn Error>); FxHashMap/BTreeMap/IndexMap (not std HashMap);
  `.try_into()` over `as` casts; no `.unwrap()`/`.expect()` outside test/main/const-init.
- TDD red/green: every implementation phase writes failing tests before code.
- Deterministic Simulation Testing (DST): a reference FSM model (an "infallible sibling", <30% of the
  real FSM's LOC) drives property-based state-machine tests + IPC fault injection (packet loss,
  latency, SIGKILL) to prove FSM/SSB/BSP correctness and 100-run determinism.

## 2. Knowledge engine (retrieval pipeline, "Phase 1")

A layered code-retrieval pipeline over a per-project index:
- DKG (Dependency Knowledge Graph): tree-sitter parses source into a SQLite graph of symbols
  (`dkg_nodes`) + typed edges (imports/contains/implements). Multi-language via per-grammar
  `catch_unwind` isolation; composite languages (Svelte/Vue) re-parse `<script>` blocks. blake3
  incremental update.
- BM25 (tantivy): per-project full-text index; sliding-window or AST-aware chunks; atomic
  delete+insert; binary files skipped.
- Vector search: dual ONNX models — GraphCodeBERT (code, 768d) + Granite (text, 384d) — with a
  blake3-keyed SQLite vector cache, per-model circuit breaker, and a startup canary embed. Missing
  model → DEGRADED (BM25-only), never a crash.
- PageRank repo map: ranks symbols by importance (symbol-level nodes, sqrt edge-weight scaling,
  10× personalization for task-mentioned nodes, density guard) and fits a token-budgeted repo map via
  binary search.
- CatRAG + A-RAG: CatRAG does symbolic anchoring + key-fact enhancement over the DKG (heuristic
  edge scoring, no LLM in the hot path); A-RAG does keyword + 3× semantic-oversample retrieval with
  progressive snippet disclosure and a context-tracker dedup. RRF fusion (k=60) merges BM25 + vectors.
- Cross-encoder reranker: ms-marco-MiniLM ONNX (CPU), semaphore-1 + batched, circuit-breaker with an
  L-6 fallback ladder; skipped under a candidate threshold.
- SPLADE (conditional): keyword expansion behind a feature flag, early-dropped if code-vocab coverage
  is too low.
- Resilience: `SearchMode` cascade HYBRID → SPARSE_ONLY → BM25_ONLY, per-model breakers, a query
  validator, and a startup readiness gate (READY/DEGRADED/UNAVAILABLE).
- Production RAG hardening: adaptive AST-aware chunking, template-based multi-query expansion, HyDE
  (sidecar generates a hypothetical answer to embed), and query-tier routing FAST/STANDARD/DEEP.
- Knowledge extensions: symbol-level impact API (`compute_impact_radius`/`call_flow`/`symbol_context`
  over a sharded LRU symbol-graph cache with confidence tiers); resumable cross-process indexing with
  file-hash checkpoints + a PID-liveness lock + auto-resume; minified-file detection + universal char
  cap; entry-point detection (orphan/well-known-name/framework-route/test heuristics); an interactive
  symbol-graph viewer (Mermaid + cytoscape); and a second "artifacts" indexing pipeline for non-code
  docs declared in config (`GroundingMethod::ArtifactCheck`).
- Cosmos tier: a global, cross-project, user-scoped artifact store (`~/.elai/cosmos/`) of design/IP/
  app-store/monetization briefs, refreshed on a cadence by a FlashResearch worker, time-decayed to
  DECAYED, and compiled into a ≤1500-token briefing for the Designer/idea-framer
  (`GroundingMethod::CosmosCheck`).

## 3. Reasoning engine — GRC (Grounded Reasoning Chain, "Phase 2")

- GRC state machine: reasoning proceeds Observe → Hypothesize → PlanAction → Act → Verify (+ Status,
  Converge). Each `ThinkStep` carries a grounding string classified into levels
  (Uncertain/High/Verified). A per-topic budget (max_calls/max_chars) forces convergence near
  exhaustion. First step must be grounded observe.
- Grounding verification: six binary pass/fail methods — FILE_CHECK, SOURCE_CHECK, TEST_CHECK,
  WEB_CHECK, API_CHECK, API_REGISTRY_CHECK (later + Artifact/Cosmos/VisualArtifact = 8th…). A plan
  grounding sweep blocks PLANNING→EXECUTING on any UNGROUNDED claim.
- Per-agent GRC config: each role gets its own budget, required step sequence, grounding level, and
  allowed verification methods. Anti-rubber-stamp: an auditor's Converge is rejected if its history
  has zero `FINDING:` steps.
- CSQ (counterfactual safety questions) + inline feasibility: functions touching files/network/
  money/auth get generated CSQ checks ("if X is wrong, the code will Y — add a guard"); a bounded
  inline feasibility check runs during EXECUTING and can emit a Blocked verdict (`CONTRADICTS_PLAN:`).

## 4. Orchestration & FSM (interactive + autonomous, "Phase 3")

- Full FSM: IDLE, PLANNING, FEASIBILITY_PROBE, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING,
  COMPLETE, CONFIGURING. Phase-aware tool/rule filtering is enforced at the tool-registry level (a
  disallowed tool returns an error, not advice). Seven evidence gates guard the transitions (knowledge
  ready, zero ungrounded, tests fail for the right reason, a FAIL→PASS flip, zero BLOCKs, DKG+changelog
  written). Per-transition risk levels (LOW/MEDIUM/HIGH).
- Interactive gate: a human decision (APPROVE/REJECT/MODIFY/REDIRECT) at MEDIUM+HIGH transitions.
- Autonomous mode: an `AutonomousGate` WRAPS the interactive gate — in interactive it's a passthrough;
  in autonomous an `AutoAdvanceEvaluator` checks deterministic evidence per transition and only
  delegates to a human on HIGH-risk / missing-evidence / an escalation trigger (anti-spiral,
  ReplanMajor, circuit-breaker, substitute-model-failed). SPLIT_TASK and SUBSTITUTE_MODEL are its
  recovery moves. Tree-level auto-advance sets risk per decomposition level (BabyStep/Step=LOW,
  Phase=MEDIUM, Concept=HIGH, always-human at concept).
- SSB (Shared State Board): the append-only, single-writer SQLite WAL event journal — the causal
  record of the whole system. Every subsystem writes typed `action_type` rows; the dashboard and every
  gate read them; nothing UPDATEs or DELETEs. This journal is what makes every metric recomputable and
  every run reproducible.
- BSP supersteps + channels: deterministic macro-orchestration (snapshot → prepare → clear → parallel
  execute → apply-writes-in-SSB-seq-order → deferred → gate → checkpoint). Typed channels: LastValue,
  Topic (accumulate/reset), EphemeralValue (per-superstep), UntrackedValue (in-memory, never
  serialized).
- Agent system: spawn/lifecycle/checkpoint/crash-recovery/supervision. Eight roles (Executor, Auditor,
  PromptEngineer, ContextCompiler, MemoryWriter, Researcher, MediaValidator, Designer). Erlang-style
  supervision (max restarts, escalate/substitute). Anti-spiral: 3 consecutive identical blake3 step
  hashes → checkpoint + convergence signal; a 4-pattern StagnationDetector
  (Spinning/Oscillation/NoDrift/DiminishingReturns) switches among 5 lateral-thinking personas
  (Hacker/Researcher/Simplifier/Architect/Contrarian); a GenerationWatchdog (idle/no-progress/safety
  timeouts) + a goal-drift Jaccard canary.
- Rules engine: rules are DATA in SQLite, not prompt text. Predicate types AST_PATTERN / GREP_PATTERN
  / SCHEMA_REQUIRED / METRIC_GATE / COMPOSITE with verdicts ALLOW/WARN/BLOCK/ESCALATE. A lifecycle
  PROPOSED→CANDIDATE→SHADOW→ACTIVE→RETIRED with promotion gates (contract tests + fpr/tpr + causal
  rate). `mutable=0` rules (security/finance) are immune to auto-modification and self-improvement.
  Self-discovery auto-proposes rules from repeated violations.
- Hierarchical plan management: a small always-loaded skeleton (≤2-4k tokens) + per-phase elaborations,
  with contract extraction, coherence checks, and BFS change-propagation along dependency edges.

## 5. Context management & lifecycle

- Context budget allocator: fixed-slot budgeting (SystemPrompt, Rules, PhaseSpec, RepoMap, CodeContext,
  SsbEntries, GrcHistory, OutputReserve; later + Memory + AnchorPane) that never exceeds the model
  window, with floors, 3-level adaptive reduction, and deterministic cache breakpoints (xxhash prefix
  hashing) after PhaseSpec and CodeContext.
- Regime-aware policy: cloud and local need OPPOSITE context policies — cloud keeps a cache-stable
  prefix + late compaction; local minimizes the window + unloads early. One "window truth", a measured
  per-(model,lane) degradation knee replacing folklore window fractions.
- Compaction ladder + sleep-time handoff: a multi-tier compaction ladder above the session handoff;
  cross-session grounded topic-digest that auto-detects, one-tap-confirms, and tier-loads prior state.
- Immutable-transcript / pure-projection core (L-T3): the append-only transcript is the single source
  of truth; the provider array is a PURE deterministic projection `fold(log, budget, policy, freeze)`
  that never mutates the log. A freeze-epoch controller recomputes the prefix only on an enumerated,
  journaled cause. Losslessness and undo/redo/branch become "recompute the view at a log index".
- Deterministic context compression: LSP-slicing + tree-sitter AST folding (elide irrelevant function
  bodies, keep signatures) with reversible transform receipts; ciphers/abbreviations banned. Anchor-pane
  slot (immutable task/constraints appended last), pristine-context injection (failed-sibling tracebacks
  hard-excluded, kept only as a compressed contrastive constraint).

## 6. Isolation, tools, hooks (process boundary, "Phase 4A-4C")

- Agent process isolation (Tier C): agents run as minimal OS processes with NO network/LLM/tool deps;
  the orchestrator is authoritative for everything and proxies via a 15-message IPC protocol (versioned
  frames, 10MB cap). Linux seccomp/namespaces/rlimits/hidepid; Windows AppContainer. Hidden paths
  return NotFound identical to missing (agents can't distinguish hidden from nonexistent). Firecracker/
  gVisor snapshot-restore pool for per-baby-step forks; overlayfs/worktree-pool CoW workspaces for
  parallel branches with a deterministic merge barrier.
- Tools: git2 ops, a 5-tier validation ladder (determinism→quality/AST→unit→integration→stress),
  DAIRA trace markers, 3-tier edit matching (exact→whitespace→ellipsis), ShortCoder (AST-equivalent
  code compression), a deterministic failure taxonomy (regex categories), file-outline mode, and async
  incremental compilation.
- Hooks + tiered static analysis: PreToolUse/PostToolUse/Stop/PreCompact hooks intercept IPC in the
  orchestrator (agents never see them). T1 = rule predicates (<1ms), T2 = tree-sitter syntax/dead-code/
  lazy-code (<10ms), T3 = pattern library (fires only on WARN-without-BLOCK). No LLM in the hot path.
  A directive supervisor (I-1) makes the generator emit pure Intent→Directive structs; a
  process-separated supervisor is the only executor of side effects and holds the credentials.

## 7. Reversibility, sessions, handoff

- Session handoff (DAG): snapshot-based session versioning with structurally-lossless 3-pass trimming
  (tool results > threshold get stubbed with a blake3 summary hash + SSB seq; user/assistant messages
  never stubbed; stubs always recoverable from the SSB). Branch/merge (max active branches, immutable
  DAG, file-overlap conflict detection), handoff triggered at 85% token usage.
- Reversibility / time-travel (H-49): undo/redo/branching file time-travel over a CAS object store
  wrapping edit checkpoints; an undo-tree HEAD over the session DAG (redo = forward edge, undo-then-edit
  = fork); a divergence guard refuses to restore if on-disk no longer matches the recorded post-image;
  lossless paged session history.
- Harness undo (J-3): a PreToolUse hook copies each file's pre-image to a per-session content-addressed
  backup before the write; an `undo.py` list/diff/restore/prune tool; last-good = the version preceding
  an audit PASS.
- Re-executable RRA (Reproducible-Run Artifact, G-2): a portable msgpack+zstd bundle of the SSB action
  stream + seed + hardware class + system fingerprint + model pin + an evidence manifest of every
  gate's verdict hash + a blake3 root. `elai replay --verify` re-runs the action stream and reports
  PASS or the first divergence. Honest tiers: action-faithful (cloud) vs token-faithful (local where
  determinism caps pass) vs score-faithful. A VCR "cassette" records the taint-ingested external bytes
  so a third party re-runs against the recorded world under a pinned environment.

## 8. Pipeline factory & recursive decomposition ("Phase 4D/4E")

- Research + Designer (L1): a GRC-grounded tree-structured research agent (FlashResearch: breadth/depth,
  φ/ψ convergence thresholds, info-gain pruning) produces a `PipelineSpec` + `QualitySpec`; the Designer
  is a distinct role from the Executor. Source-quality is a deterministic 5-tier lookup; packages carry
  an API-registry check to prevent hallucinated deps.
- Recursive decomposition: fractal Idea → Concepts → Phases → Steps → BabySteps, each level triggering
  grounded research. A `BabyStepSpec` has 8 blocks (GROUNDING, PRE_GATE, TASK, TDD, DETERMINISTIC_CHECK,
  ACCEPTANCE, INTEGRATION, EXIT_CRITERIA).
- Builder (L2): deterministic pipeline assembly (no LLM) with 8-point structural validation, bounded
  DAG repair (deterministic REORDER for cycles / SUBSTITUTE for missing tools), and phase sizing
  (split if complexity > 0.6 of budget). A `TreeNavigator` walks concepts→phases→steps→baby-steps
  depth-first; each baby step runs an 8-stage cycle; a `BubbleUpHandler` escalates with per-level retry
  thresholds; integration checkpoints at 4 levels (baby-step cargo-check → workspace tests).
- Idea-framer (B-40): a FRAMING substate before decomposition classifies the product idea (regex first,
  LLM fallback), runs domain research presets, and a 10-category critic loop (STALE/SLOP/POLICY/
  FEASIBILITY/MARKET/SCOPE/…) that blocks on unresolved findings; emits tradeoff cards.
- Per-level auditor (B-41): 6 auditor modes (Concepts/Phases/Steps/BabySteps/BabyStepDetail/Plan), each
  with its own GRC budget and rule cluster; BLOCK respawns the level-N decomposer once, then cascades
  to the parent tier.
- Ambiguity score (B-38): a continuous `1 − Σ clarity·weight` metric over 4 dimensions
  (goal/constraint/success/context) with per-dimension floors and an auto-complete streak requirement.
- Chat REFLECT (B-39): a per-message TRIAGE → REASONER (best-of-N) → AUDITOR (bounded REFINE) →
  SYNTHESIZE reasoning pipeline for the chat REPL, opt-in via `/reason`.

## 9. Self-modification & the self-improvement flywheel

- DGM-style self-modification ("Phase 4H"): the system improves its own configs/rules/data-tables from
  `PipelineOutcome`, never evaluating its own modifications. Class A = additive data-table rows (no
  LLM, no staged eval); Class B = parameter changes (AllowedConfigKey closed set, 10/50/200 staged
  eval); Layer 3 = structural = human-only. Score normalization to [0,1], MSI (exponential-decay moving
  average → IMPROVING/STABILIZING/DIVERGING), DGM sigmoid parent selection from an archive, auto-revert
  on degradation, and 4 safety-enforcement layers. The modifying agent cannot read the lock list.
- Verification hardening (the convergence theorem): a self-improvement loop without a real,
  un-gameable, stronger-than-generator filter provably collapses. So: execution-grounded DGM fitness
  (a gate-PROVEN pass on a held-out recurrence, never a byte-hash), behavioral trap detection with
  measured recall (not keyword-grep), worst-case + lineage-diverse promotion scoring, clade
  meta-productivity parent selection, an eval firewall (date-gate + template-regen + holdout-budget),
  and a reward-tampering gate frozen into the read-only gate set.
- Reasoning-improvement substrate (D-plan Wave 1): typed `ReasoningTemplate` records with blake3
  content-hash + archive lifecycle; a PPM (process reward model) sidecar for step-level scoring; a
  multi-verifier consensus rule (≥2 verifiers concur); a 5-signal tripwire stack (KL/Δρ/IQR/perplexity/
  DDM) with auto-revert; Beta-posterior Thompson-sampling promotion (BAI); a time-windowed held-out
  task store (9→13-gram contamination filter, 6-month auto-retire); a 5-family cross-domain transfer
  canary; a cryptographic append-only archive with `elai archive verify`; and L3 frozen-surface access
  control.
- Open-weight flywheel (Track 17): the paid-for search compute produces a dataset — extract verified
  winning paths from sealed RRAs into SFT JSONL, record negative pairs (highest-scoring-but-vetoed =
  the canonical "fluent but wrong"), do rejection-sampling SFT (LoRA) first; GRPO on execution-grounded
  rewards only comes LAST behind blocking verification prerequisites and a kill-switch. DAG credit
  assignment walks backward from COMPLETE to credit chunks/sub-plans/templates.

## 10. Real-problem grounding & mode routing (D-plan Wave 2)

- Mode routing: an orchestrator-side (not LLM) `select_grounding_mode` routes each claim to A/B/C/D/E:
  A = constructive checker (sympy/z3/compile/registry), B = cross-family adversary, C = discovery-vs-
  search labeling, D = observable-effect / sandbox experiment, E = conformal refusal. Modes map to a
  checker registry (`.elai_cc/checkers/`), an operationalization predicate (a claim-emitting baby step
  without a MEASURABLE_PROCEDURE is blocked), an `Adversary` role with mandatory cross-family
  invocation, a sycophancy probe (inverted-framing re-run), a `NoveltyClass` label, UniCR conformal
  refusal calibration, an honest-refusal SLA monitor (<5% or >50% is flagged), and a typed
  `ObservableEffect` (≥2 per claim, sandbox-vs-production labeled).

## 11. Multi-runtime backends, cloud, cost, efficiency

- Multi-runtime abstraction ("Phase 4M", B-37): one `RuntimeAdapter` trait over many backends —
  Sidecar, AnthropicApi, ClaudeCode, CodexCli, OpenCode, Hermes, GeminiCli, KiroCli, CopilotCli,
  Ollama, Vllm, SGLang, MLX, LlamaCpp, plus OpenRouter. `PerRoleRuntime` gives each agent role its own
  backend + fallback chain; CLI subprocesses run inside Tier-C isolation.
- Cloud mode: per-task LOCAL / CLOUD / HYBRID; hard spend ceilings that BLOCK (not warn); per-tool
  spend gates; a difficulty estimator before a cloud route; privacy redaction + air-gapped mode; BYOE
  custom endpoints.
- Local serving: a Python sidecar speaking the Anthropic protocol; NF4→AWQ+Marlin quantization, FP8
  KV cache, mandatory FA2, StreamingLLM sinks, 32K native context on Qwen2.5-Coder; DoLa/ActAdd/
  TurboQuant layer-level levers where the backend exposes logits; local-lane token determinism
  (batch-invariant ops + seeded Gumbel).
- Caching + output-token efficiency: prompt-cache markers on a byte-stable prefix with breakpoints +
  1h TTL (a cache-bust is a test failure); a cache-hit hard guard (unexplained >10% miss dumps a prefix
  diff); Tool-Search Tool + advanced-tool-use header to shrink per-request tool-def cost; Batch API for
  non-interactive lanes (50% off); deterministic observation masking that preserves the cache prefix;
  diff/FIM output emission + streaming early-abort (output bills 5×).
- Routing: architect (cloud flagship, plans) / editor (cheap local, cheaply-checkable edits) /
  independent-context verifier, on a calibrated external signal (never a 7B's self-confidence), gated on
  cost-AND-correctness; a small router trained on the journal's ground-truth verifier outcomes; dynamic
  thinking/serial-depth budgets per node with an 85% collapse rule; a cascading local-first waterfall
  that condenses failures into a typed BeliefState before escalating.

## 12. The trust/verification spine (F/G/H-Track-3 + the gate stack)

- Wiring enforcement: "compiles + tests pass" is blind to wire-dark code (a unit test calling a
  function is indistinguishable from a missing caller). The gate drives a REAL entrypoint, measures an
  artifact the agent did not author (an SSB row / coverage hit / span), treats missing evidence as a
  hard FAIL, and is frozen read-only to the run being evaluated. A layered stack: L1 compile-time dead-
  code/producer-without-consumer; L2 cross-language contract symmetry (Tauri↔TS, sidecar↔Rust); L2b
  frontend mount-reachability; L3 instrumented-binary entrypoint coverage; L4 runtime trace assertion
  (a tracing-test `logs_contain` MUST be inside an `assert!`). Production-caller gate: every new public
  symbol must have a real production caller in the same changeset or a registered WIRE-DARK marker.
- VibeGate (F-1): one PreToolUse handler fans a file-write across four fail-closed axes — code-vuln
  (AST/GREP), hallucinated-dep, secret, malicious-skill — backed by a `mutable=0` "vibesec" rule domain
  so per-project edits can't disable security. F-2: a code-vuln ruleset that BLOCKs at write + the first
  measured pre-write insecure-rate. F-9: a write-time hallucinated-dependency existence+age gate. F-10:
  a per-iteration security-monotonicity gate (block if iteration N introduces a CWE class absent at
  N-1). F-4: test-integrity — a blake3 stdout_hash binds every claimed test outcome to an
  orchestrator-captured artifact so a fabricated PASS is detectable ("can't-lie").
- Taint / data-plane defense (G-1, CaMeL/RTBAS/FIDES): `Provenance`-labelled data threaded from the
  four ingest sites (web/MCP/file/sandbox) so a capability gate blocks untrusted-content→destructive-
  sink flows; dependency-screened join (not global, which over-taints). F-3 Tool-Filter locks the
  researcher's tool set to a minimal subset before any untrusted byte is seen. G-3: post-shell-expansion
  filesystem/egress blast-radius gates (canonicalize globs/symlinks before the verb; pin egress IPs;
  reject null-byte hosts). E-30: a non-LLM PreToolUse argv destructive-verb allowlist. G-4: live MCP
  rug-pull re-pin on `tools/list_changed` with a description diff.
- Provenance & attestation (G-8): a signed in-toto attestation (model id, seed, an SLSA-VSA rollup of
  every gate verdict, evidence hashes) in a git trailer + sidecar file, verifiable OFFLINE via
  `elai verify-attestation` without re-running ELAI; SSB per-row chain hashes; opt-in Rekor
  transparency log; an AI-BOM (CycloneDX-ML) entry. G-9: a write-time secret/PII redactor on the SSB
  commit path (append-only means redact-at-write), file-granularity retention TTL.
- Reviewer cockpit (G-5) + DSR: a per-hunk view rendering the proof tokens ELAI already emits so a
  reviewer skims vouched hunks; DSR (diff-surface-to-review = lines needing full attention / total
  changed) is the headline review-time number, recomputed from the journal. H-52: deterministic
  snippet→hunk finding-anchoring + a line-less finding schema (hallucinated line numbers made
  impossible). PR/CI evidence emitter (K-3/K-11): renders gate verdicts + stdout-hash-bound outcomes +
  DSR bands + anchored findings + attestation into GitHub check-runs; a no-evidence hunk renders
  UNVOUCHED, never clean.
- Adaptive gate economy (K-4/K-5): a composed gate-tax budget (whole-system friction ceiling, release-
  blocking) + a risk-tiered gating policy (cheap deterministic checks always-on; expensive legs —
  mutation, differential, VLM, formal — fire only on risk triggers: blast-radius, CWE surface, autonomy
  level, novelty, criticality). The LLM never picks the tier; the policy table is frozen.
- Autonomous-trading-trust gates (H-Track-3): a persisted, agent-unresettable cumulative trial count N
  (the max of N noisy Sharpe ratios is upward-biased); a Deflated-Sharpe/PBO/MinBTL promotion gate; a
  reusable-holdout DP query budget; a leakage static-analysis composite over the taint graph + a runtime
  look-ahead/causality differential backstop (run the pipeline twice, full vs future-truncated); trading
  metamorphic null-tests (edge must collapse under a shuffled future); same-seed re-run hash-equality.
  The irreducible undecidable residue is named and routed to a human.

## 13. Search, anti-gaming verification, memory

- Evidence-ranked search ladder (I-4, Track-13 keystone): nothing selects by an LLM judge — ELAI
  selects BY RECEIPTS. v1 best-of-N (candidates in isolated workspaces → tri-layer veto + budgeted
  mutation + progress score → a RANKING with journaled reasons); v2 tournament; v3 PUCT tree over the
  session DAG. The generator proposes K divergent candidates; the orchestrator owns the tree.
- Tri-layer evaluation stack (I-17): L1 HARD VETO (compile/test/VibeGate/dep-gate — any fail = 0.0,
  never overridable) → L2 calibrated scorer on survivors only (PPM/PRM-lite) → L3 disagreement escalation
  to the cross-family adversary; a 15s halting bound; flake quarantine.
- Budgeted mutation scheduler (I-20): mutation testing INSIDE the loop — mutants only on the diff's
  data-flow cone, time-boxed, sampled with a confidence-interval score + early stop, cached by content-
  hash; survivors become structured counterexample directives to the generator. Progress-shaped rewards
  (Δφ from deterministic observables, first-error truncation, decay on no-progress). Isomorphic canary
  anti-gaming (AST-preserving perturbations detect a style-biased scorer). Sprint contracts (generator/
  evaluator with zero shared context, bounded Ralph loop). Aletheia voluntary abort (an
  acknowledge-infeasible tool credited by the abstention score + dead-end AST-hash pruning).
- Insight backpropagation (J-10, Arbor): extract a 1-3 sentence insight from each audit/experiment,
  re-synthesize each ancestor's insight from its children up to the root, inject ancestor-lineage
  insights down into executor prompts; each insight carries a provenance hash so a fabricated lesson is
  detectable.
- User & fact memory (E-5 etc.): a `memory_facts` SQLite table with Mem0-style ADD/UPDATE/DELETE/NOOP
  dispatch, Zep bi-temporal scaffolding, trust-score + temporal decay, and a MINJA poisoning defense;
  A-MEM Zettelkasten + MemGPT core/archival + a sleep-time digest layer; Tier-A code-grounded
  validate-citation-before-use facts; a write-time memory guard (injection/invisible-Unicode/secrets);
  an auditor-objectivity firewall (model-emitted memory never enters the auditor's context).
- Learned-artifact admission pipeline (J-109): the convergence-theorem filter applied at ADMISSION
  grain, not just promotion — every candidate memory fact / playbook bullet / case / skill passes one
  ordered gate at the write seams: a span index over the immutable instruction surface rejects
  re-learned doctrine ("don't re-learn what the rules already say"); a two-threshold dedup band splits
  on trigger context (merge vs coexist); and a pre-write counterfactual replay fetches the byte-exact
  originating prompt from the SSB, injects the candidate, re-decides N times, and admits only on
  zero-harmful + supermajority flips — flip judged execution-grounded where a runnable gate exists, any
  LLM judge advisory-only. Embedder failure degrades to lexical with a labeled verdict, never a blind
  accept; every decision is a journaled row.
- Verified Case Graph (VCG): mint-on-proof typed case nodes (a bug + its verified fix, F-4-proven),
  deterministic self-wiring edges, recurrence-hit-rate (does a new failing task match an existing
  trigger signature), and a gated retrieval-time reuse loop + episodic→skill→rule promotion ladder —
  fitness is execution-grounded only, never LLM-judged-helpful or "it-was-retrieved". This is the
  per-customer compounding-state moat ("gets faster/cheaper on your recurring bugs over time").
- Multi-repo / workspace memory (K-23): a thin cross-REPO layer over deep per-root substrate — a
  workspace manifest, cross-repo derived edges (Sound LSP/SCIP edges gate, heuristic name/import edges
  only advise), federated retrieval (fan out + RRF merge with per-repo provenance), cross-repo blast
  radius, and a workspace FSM above the per-root FSM. Honest CONNECTED vs DISCONNECTED modes; never a
  fabricated connection.

## 14. Grounding gates & hallucination (L-plan Causal Grounding Gate)

The trust spine turned OUTWARD — a model-agnostic gate over ANY model's CLAIMS on ANY backend:
- Typed claim extraction: any model output → a `ClaimSet` of typed `Claim{kind, span, refs}`
  (File/Symbol/Dependency/Api/Behavior/Causal/Reachability), parsed structurally, never by asking a
  model what it claimed.
- Unified ClaimRisk + convergence fuse: three mechanistically-distinct predictors — P1 evidence
  round-trip (does the claim survive re-grounding), P2 oscillation, P3 semantic entropy (resample N,
  cluster by MEANING via a deterministic NLI judge, entropy over clusters) — and a high/critical verdict
  needs ≥2 to agree. Cost-gated: expensive predictors fire only after a cheap signal flags medium risk.
- Counterfactual do-test (L-5, the causal core): for a Causal/Behavior claim, run the intervention
  (revert only A / disable the flag / patch only the claimed function) and re-run the F-4 outcome or a
  mutation/metamorphic relation; only an interventional PASS labels a claim "causal" — an "A imports B"
  claim is scored at its weaker static tier.
- Risk-as-curiosity router: a high/critical claim auto-fires targeted effort at exactly the weak claim
  (mid-task retrieval / read-ladder / a targeted test / escalation), then re-scores. A hallucination
  policy gate maps risk to action (allow / caveat / force-effort / BLOCK-or-abstain), wired to the
  autonomous gate + conformal refusal, observe-only first. Working-memory hygiene quarantines
  contradicted assumptions from the live belief graph. Honest limit: catches coverage-gap hallucination,
  not confident-shared-wrong beliefs (mitigated, not eliminated, by the cross-family adversary).

## 15. Perception, media, dashboard, canvas

- Media validator ("Phase 4I", B-24): Tier-1 deterministic checks (ImageMagick/FFmpeg/Tesseract/Blender
  CLI, SSIM/LPIPS on CPU for determinism) with veto power over an advisory Tier-2 VLM (constrained-JSON
  output). A Render-Capture-Evaluate loop: executor produces → adapter renders → VLM evaluates →
  structured feedback → iterate.
- Perception gateway (J-1): a deterministic image pipeline every screenshot/diagram passes before any
  model — preprocess (resize to the model's sweet spot, tile dense UIs), a hybrid composer (UI images
  get AXTree/DOM text alongside; documents route to a vision-RAG leg), a two-call perception scaffold
  (call 1 = literal-elements-to-JSON at temp 0 with 0-1000 normalized coords, call 2 = the coding model
  reasons over description+image), and a small-VLM router. Counts come from the AXTree, never the VLM.
  A hardware-model-eligibility catalog resolves what THIS box can run per lane (coding/vision/OCR/
  embedding). Document lane via MinerU.
- Dashboard: Tauri v2 + Svelte 5 (runes), no HTTP/REST/WebSocket — it polls a read-only SSB viewer via
  `invoke()`. Teal-on-charcoal, JetBrains Mono, information-density, outlined-buttons-only. Windows
  (Command/Activity/Monitor/Explore/Review/Plan/Settings + drawers per subsystem), keyboard shortcuts,
  intervention cards, a cost cockpit ($/resolved-task + cache-hit-rate + per-response token breakdown).
  A visual-UI loop drives screenshot→deterministic-diff→schema-constrained VLM critique with a two-pass
  stop gate.
- Canvas workbench (H-Track-2 + J-31/J-60): a 2D generative canvas replacing the 3D orbit — a WindowHost
  window rendering the codebase call/PageRank graph (dead code visibly disconnected), a live SSB activity
  overlay + file explorer, PLAN/RESEARCH/ARCHITECTURE modes, and the model DRIVES it via `canvas_*` ops
  (createNode/connect/annotate/fork/spawn_view) through a dispatcher with mode-namespace gating. The moat
  is the journaled + reproducible (RRA) + taint-tagged op-stream underneath. Altitudes: system-composition
  (nodes = bound buildable units, edges = verified wiring claims), process observability (ELAI's own FSM +
  agents as a journal-derived graph), and memory/case (VCG). App/website DESIGN mode: a live-preview page
  node, click-an-element-to-edit via opaque build-time AST ids, deterministic recast writeback for style/
  text and the LLM only for structural.
- Front door (J-32/J-35): chat = ELAI, not a raw model — a typed intent spec + premise-check + route +
  a decision card (user > elai > model).

## 16. Benchmarks, eval, measurement (the believe-the-number layer)

- Every claimed number is gate-ON-vs-OFF ABLATION framing (never a leaderboard rank), under `ELAI_SEED`,
  recomputable from the journal, with receipts (verified-outcome rate) on the y-axis, not pass@k.
- SAFEDELTA: a competitive safety-delta benchmark measuring the gate's marginal residual-insecure-rate
  reaching a human, on git-stripped held-out tasks (SUSVIBES/BaxBench/SeCodePLT/AgentDojo), with a
  cheat-detector (git-history strip, dynamic exploit exec, held-out CWE split). QDELTA is the quality
  sibling; gate-tax is the friction denominator; routing-economics, case-reuse, reasoning-budget,
  context-fill, hallucination-delta, convergence-curve, error-amplification are the other measured arms.
- Eval corpus: time-windowed per-family held-out shards, 13-gram contamination filter, matched-pair
  (eval-vs-deploy) sycophancy probes, commit-mined task supply, and a competitor-arm paired harness over
  the agent-CLI bridges (LLM judge advisory-only, ≥5-seed paired floor).
- Abstention timeliness (J-110): the WHEN axis on top of abstention-correctness scoring — SPL
  (oracle_step / max(first_abstain_step, oracle_step)), timely-recall and pass@k folded off-agent from
  the SSB trajectory over abstain-warranted tasks only, with the false-abstain rate on solvable twins
  reported alongside. The denominator is constructed, not labeled: deterministic mutators derive
  should-abstain variants from commit-mined tasks (exactly one defect — DKG-witnessed false premise,
  contradiction, or underspecified intent — plus a delayed pack whose infeasibility only surfaces after
  acting), each recording its oracle step by construction. Run as a gate-ON-vs-OFF bench arm, it gives
  the feasibility probe / ambiguity score / critic / policy gate their measured earliness number and
  calibrates the escalate-or-abstain effort tier; late-abstention episodes feed stopping-rule playbook
  candidates through the J-109 admission pipe.
- Metrics: STABLE-Pass@1 (n=5), $/correct-task-solved, METR 50% horizon, harness-vs-model attribution
  tuple, prompt-injection ASR, eval-awareness, context-rot, a 30-day revert canary. Model-drift regression
  canary watches the EXOGENOUS model version (pin the model+fingerprint+lane; re-run a canary on a
  detected change; surface, never auto-swap). Quarterly model-refresh cadence over the hardware catalog.

## 17. Deterministic-over-prompt hardening + doctrine

- Deterministic mechanisms replacing prompt-shaped control planes (I-Track-18, J): temporal tool-sequence
  enforcement, EARS requirement objects, cross-model prompt compilation, plan-state-from-repo (I-39),
  Crane-hybrid grammar pinning, a power-of-ten NASA-grade generated-code gate, an SSB temporal-monitor
  engine, multi-mode gate actions (Accept/Suppress/Insert/Block), SSB log-integrity hash chain +
  watermark monotonicity, gate-manifest traceability + IV&V independence, and formal verification (Kani/
  SPARK/TLA+) of ELAI's own critical code.
- Determinism doctrine propagation: the general law is "measure the objective not the proxy; freeze the
  gate; missing evidence = fail; the model generates/selects, the harness decides/gates". Applied to the
  rulebook itself via a doctrine-enforcement-coverage ledger tagging every rule clause {frozen-checker /
  auditor-advisory / unenforced}. Time-determinism: volatile time is strictly after the last cache
  breakpoint, journaled as model-input, and NEVER a gate/transition/promotion input; wall-clock kill is
  supervisor-plane liveness outside the replay set.
- Maintainability review: an 8-dimension catalog (structural simplification, file-size boundary,
  spaghetti, directness, type/boundary clarity, canonical layering, orchestration atomicity) with a
  frozen deterministic checker (file-size / type-erasure / long-fn / branching-density), auditor-applied,
  never executor-self-applied.

## 18. Commercial layer (M-plan)

- Makes a trustworthy single seat purchasable by a company without becoming an org SaaS. Signed offline-
  verifiable seat entitlements (14-day grace → degrade-to-Free, never brick; safety gates untouched); a
  credit-based seat model on an OpenRouter rail (balance = a fold over Granted/Reserved/Settled/Expired,
  never a mutable counter; the local free-lane is the margin engine; the pitch is "more resolved tasks
  per dollar, verified", gated on a measured SAFEDELTA number — never "cheap"/"unlimited"); a signed
  file-distributable org policy pack (monotone-tighten, drift-detected); a pull-model fleet evidence
  rollup (`elai org report` — numbers + hashes, never transcripts); a signed release channel + auto-
  updater + local redacted support bundle; a standalone engine-free `elai-verify` OSS binary (the engine
  stays closed); EULA/DPA/trust-continuity (escrow-or-convert-to-OSS on abandonment). Commerce is the one
  deliberately fail-OPEN layer; every safety gate stays fail-closed.
