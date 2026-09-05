# ELAI cost / model-selection / context-management pipeline — full-detail map

Generated 2026-07-04 by reading phase-index-MASTER + the B..N SUMMARY_INDEX corpus end-to-end (J-plan read in full, 106 sessions). Purpose: document, as one grounded reference, the whole sequence from a task arriving to a model being chosen, escalated, budgeted, and billed — i.e. exactly how ELAI is designed to make running AI cheaper by delegating each unit of work to the smallest model that can do it, spending context only where it pays, and never paying twice. Then: judge the six cost-usage findings in the Claude-Code usage screenshot against this design.

Every mechanism below is cited to the plan unit that owns it. Status is tagged because a lot of this is SPECIFIED-and-partially-built, not shipped-and-wired — that distinction is the honest core of the answer and is called out explicitly per stage and in section 7.

Status legend (from the corpus's own tags):
- SHIPPED — code live, often "cloud Sonnet PROVEN" (B-37, B-10, base phases 0-4, B-40/41/42).
- WIRE-DARK — the machinery is BUILT but has no production caller / sits behind a default-OFF flag / emits to nothing. This is the single most common state of the efficiency substrate (H-plan Track 1's own premise: "mostly by ACTIVATING wire-dark caching/output/routing substrate").
- PLANNED — file authored, not executed.
- MEASURE-FIRST — ships a bench + fixture, the live number is RunPod/remote-sidecar-gated.

---

## 0. The one-paragraph thesis

ELAI's cost story is not "use a smaller model." It is **push every discretionary decision out of the model and into deterministic harness machinery, then route each unit of work to the cheapest lane that can prove it did the job.** Three independent per-turn cost sinks are attacked separately (J-55): the **tool-schema** (defer/shrink it), the **action envelope** (CodeAct vs constrained tool-call, per model strength), and the **edit output** (diff/FIM not whole-file). Input is made cache-stable so re-reads are ~free; output — which bills ~5x input and caching never touches — is minimized directly. Context is spent by measured positional value, not folklore. The model that runs each role is the cheapest one that clears an off-agent receipt; when it can't, the orchestrator escalates once, deterministically. And every token is captured in a per-lane ledger so the cost claim is falsifiable, not asserted.

---

## 1. The end-to-end sequence (task → done)

```
USER MESSAGE
  │
  ├─ Stage 1  FRONT DOOR: total-function lane routing (chat|task|fix|research|clarify)   [J-32, PLANNED]
  │            + intent-control: typed spec, premise-check, ask-only-if-it-helps          [J-35]
  │
  ├─ Stage 2  DIFFICULTY / AMBIGUITY estimate → how much machinery this earns
  │            2-axis reasoning classifier: trivial→zero, easy→depth, hard→width,          [J-84]
  │            very-hard→escalate-or-abstain; ambiguity/novelty → research depth           [N-10, B-38, D-15]
  │
  ├─ Stage 3  MODE: LOCAL (default, $0) | CLOUD | HYBRID  — per task                        [B-27, C-5]
  │            local-first waterfall: wide cheap branches local, escalate ONLY on failure   [I-7]
  │
  ├─ Stage 4  ROLE→MODEL routing (the delegation core)
  │            per-role runtime: architect=cloud flagship, editor=cheap 7B, verifier=alt    [B-37 SHIPPED, H-7 wires it]
  │            local serving tier router: warm/cold, per-tier concurrency, spill-to-cloud   [J-48]
  │            free-API rotation pool assignable per role; modality router per model strength[J-54, J-55]
  │
  ├─ Stage 5  CONTEXT BUDGET: 8 fixed slots, never exceeds window; measured positional value [3E SHIPPED, B-10 DONE, J-34]
  │            read-access ladder: read the cheapest rung that proves it (card→skeleton→file)[J-103]
  │            retrieval-abstention: don't retrieve when local context suffices             [J-73]
  │
  ├─ Stage 6  CACHE: prompt-cache markers on a byte-stable prefix, 1h TTL, hit-rate guard   [H-1/H-2/I-10]
  │            local prefix-KV reuse: freeze the head, re-prefill only the tail             [J-36]
  │
  ├─ Stage 7  EMIT CHEAP: diff/FIM not whole-file; Tool Search (77K→8.7K tool defs);        [B-28, H-3, H-4]
  │            streaming early-abort; deterministic token-efficient output format           [H-3, J-82]
  │
  ├─ Stage 8  ESCALATE (only on a failed off-agent receipt): condense to a typed BeliefState,│
  │            hand to the cloud architect once; substitute-model on repeated death          [I-7, I-11/J-83, B-22]
  │
  ├─ Stage 9  CONTEXT LIFECYCLE across the long session: regime-aware (cloud≠local),          [H-50/H-51, E-25, H-6]
  │            auto-compact/handoff at threshold, immutable-transcript projection             [3H, L-16]
  │
  ├─ Stage 10 OFFLINE / IDEMPOTENT lanes → Batch API (flat 50% off), oracle+semantic caches, │
  │            template crystallizer (repeat task → cached pipeline)                          [H-5, C-30/H-8, I-15, C-25]
  │
  └─ Stage 11 MEASURE: per-lane token ledger, live $/task cockpit, routing-economics bench    [J-28/J-29, H-9, K-2/K-4]
```

The rest of this doc details each stage with its grounding.

---

## 2. Stage 1-2 — Intake, difficulty, and "how much machinery does this earn"

The cheapest token is the one never spent. Before any model runs, ELAI is designed to size the work deterministically.

**Front door (J-32, PLANNED).** Every chat message is routed by a *total function* over a closed set of entrypoints (chat / task / fix / research / clarify) — not by asking a model "what did the user mean." Tiers: T0 slash-bypass → T1 deterministic regex matchers (the shipped `intent.ts` set promoted to one Rust `FrontDoorRouter`) → T2 a local-3B constrained label at temp 0 emitting `{lane, confidence, arg_spans}` as byte-spans of the user's own words. The point for cost: a "fix the failing test" message dispatches the *same* row sequence the `/fix` command would, with no extra cloud round-trip to classify it.

**Intent-control (J-35).** Reconstructs a typed low-entropy `IntentSpec` *before* decomposition and *before* retrieval (RAG masks goal-uncertainty), runs a deterministic premise-check against the DKG, and asks the user a question ONLY when it improves execution AND the action is costly/irreversible (over-asking is itself a tax). This stops mis-aimed work from decomposing into clean-but-wrong baby steps that burn tokens solving the wrong problem.

**Reasoning-effort classifier (J-84, EXTENDS H-42/I-26).** A deterministic 2-axis classifier: TYPE (reason vs retrieve-recall vs validate, reusing D-10) × difficulty proxy (H-39 confidence + case-base + retrieval coverage + intent-pairs) routes:
- trivial → ZERO reasoning tokens,
- easy/medium → serial DEPTH,
- hard → parallel WIDTH (candidate ladder, J-86),
- very-hard (proxy < ~0.05) → escalate (I-7) or abstain (I-8).

A recall query routed to a reasoning chain is a *confabulation bug* — the classifier's negative gate forbids it. The `lambda` effort-toggle (I-27/H-9) lets the user override the auto-tier. **The window is never a tier** — depth vs width vs window are kept orthogonal.

**Overthinking guard (J-85, NEW).** A per-call reasoning ceiling + 3-signal early-stop (answer oscillation / hesitation markers / declining confidence) that stops a chain *drifting off a correct answer* — distinct from anti-spiral (repetition). The ceiling tracks the K-15-*measured* per-(model, task-type) knee, never a paper constant.

Cost consequence: the trivial and recall paths never enter an expensive reasoning or search loop at all, and the hard paths get width only when a cheap deterministic proxy says they need it.

---

## 3. Stage 3 — Mode: local vs cloud vs hybrid ($0 by default)

**Three per-task modes (B-27 PLANNED, C-5 PLANNED):**
- **LOCAL** — default, 6 reasoning layers, **$0** (runs on the user's own model / remote 3090 sidecar).
- **CLOUD** — cloud generates, local L1 deterministic checks still run, L2-5 skipped.
- **HYBRID** — local first, auto-escalate to cloud on L3+ failure *for that step only*.

The cloud profile is deliberately different from local (B-27 D-2): 200K context vs 8-25K local, baby steps of 300-500 LOC vs 25-40, decomposition depth 2 vs 4, phase-split threshold 50K vs 3K, reasoning layers collapsed. So cloud does *bigger, shallower* units (fewer expensive round-trips) and local does *smaller, deeper* ones (cheap tokens, more of them).

**Local-first waterfall (I-7, corrected from the impossible "race the batch API").** Wide cheap branches run on the LOCAL lane first; deterministic checks run locally; only on local failure does the orchestrator *condense the failure set* into a typed BeliefState (I-11/J-83) and escalate **synchronously** to the cloud architect. A fixture task the local lane can solve *never leaves the machine* (`LocalWin` row, zero cloud spend in the journal). This is the load-bearing cost lever: the default cost of a solvable task is $0.

Hard ceiling on spend (E-39 PLANNED): the cost gate BLOCKS, not warns; one-shot retry max then escalate; per-tool spend gates; a pre-execution difficulty estimator before any cloud route. Explicitly closes the "$47K wallet-loop / 2.3M retry storm" incident class.

---

## 4. Stage 4 — Role→model routing: the delegation economics (the heart of "cheaper by delegation")

This is where "delegate to the necessary model, use cheaper models when you can" actually lives.

**Multi-runtime backend abstraction (B-37, SHIPPED — live cloud Sonnet PROVEN).** 11 backends behind a `RuntimeAdapter` trait + `PerRoleRuntime` + `RuntimeOrchestrator` + `/runtime` CLI + `RuntimePanel`. The key object: **a role picks its own backend.** Executor, auditor, context-compiler, researcher, designer, etc. each resolve to a model independently.

**Calibrated architect / editor / verifier routing (H-7, PLANNED — wires B-37's wire-dark orchestrator).** The doctrine "smarter orchestration, not a smarter model":
- **architect** = cloud flagship — reasons and plans (the expensive, rare step),
- **editor** = cheap 7B — cheaply-checkable edits (the frequent, cheap step),
- **verifier** = an independent-context auditor.

The architect/editor split is grounded at +9.6pp on Aider. Critically it is *cost-AND-correctness gated*: a cheap edit that fails an off-agent check is not cheaper, so the router only routes an edit to the editor when the check is cheap to run. The `layer_failed` signal is replaced with a *calibrated* external signal (RouteLLM/AutoMix-style), never the 7B's own self-confidence. Over-escalation rate is bounded on ≥3 runs before it goes default-on.

**Ledger-trained router (I-25, PLANNED).** Routing products train on proxy signals; ELAI's journal holds *ground-truth verifier outcomes* per (task features, lane, cost). A small router trains on ELAI's own outcomes; execution-cost profiling pre-routes known-expensive AST shapes to the deep lane *before* a cheap-lane failure burns tokens.

**Local serving tier router (J-48, PLANNED).** The outer layer B-37 lacks: when several local models share one or two GPUs, `LocalServingRouter::decide` returns `{ServeLocal, AdmitQueued, HibernateThenServe, SpillToCloud}` — per-tier concurrency caps, warm/cold whole-model lifecycle (pin small tiers; the 32B tier runs EXCLUSIVE because ~16GB weights + ~8GB KV @ 32K won't co-reside on one 24GB GPU), FIFO with ≤3 priority classes, busy/cold spill riding I-7. It deliberately does NOT reinvent continuous batching / paged-KV / preemption — the engine (vLLM, J-19) owns that.

**Free-API rotation pool (J-54, PLANNED).** A `FreePool` backend kind — a hand-rolled Rust port of per-(key,model) daily-quota persistence, 429-header cooldown, whole-pool exhaustion sleep, multi-key rotation, keyless overflow — **assignable per role** through the same PerRoleRuntime surface. So a user can put the draft/editor role on free-tier keys and keep the verifier on a paid model. Guardrail: a free-pool run on executor/auditor stamps the run non-RRA-reproducible and suppresses-with-notice the drift canary (surface, never silent swap).

**Model-strength modality router (J-55, KEYSTONE, PLANNED).** The "agentic needs tools → tools cost tokens → efficient means no tools" tension is *false*: per-turn cost is three independent sinks attacked separately. `ModalityRouter::resolve(profile, role, tier) → ActionModality`:
- **CodeAct** (tools as code imports, collapses schema + envelope) for capable lanes,
- **constrained single-tool-call under XGrammar + minimal compiled schemas** (NOT all-tools-every-turn) for the sub-14B local lane,
- a **universal SEARCH/REPLACE-diff edit path** (model-independent output-token win),
- progressive disclosure on the schema'd path.
The grounded keystone: naive "CodeAct everywhere" *loses* on ELAI's own Qwen-7B lane (open models collapse to ~12% on multi-tool code-mode; viability ~14B). The router is the load-bearing unit, not CodeAct. Crossover measured per-(model,lane) via `elai bench modality`.

**Tool-call adherence floor (J-51, PLANNED).** Measures per-local-model *semantic* tool-call reliability under the grammar mask (does the small model pick the RIGHT tool with RIGHT args?), writes it to `ModelCapabilityProfile`, and advisory-gates tool-heavy roles away from sub-floor models (spill to cloud on `FloorBreached`).

**Substitute-model on death (B-22 SHIPPED).** `SUBSTITUTE_MODEL` on max_restarts via a `ModelPool` swap chain; emits `model_swap` SSB. `BUDGET_EXCEEDED` does not increment restart count.

Net effect: the frequent, cheap, checkable work (edits) runs on a $0 local 7B; the rare, expensive reasoning (planning, architecture) runs on cloud flagship; verification runs in an independent cheap context; and the choice is data-driven off ELAI's own outcome ledger, not a static rule.

---

## 5. Stage 5 — Context budget: spend tokens only where they pay

**Fixed-slot allocator (Phase 3E, base — built; B-10 profile activation DONE).** 8 slots — SystemPrompt, Rules, PhaseSpec, RepoMap, CodeContext, SsbEntries, GrcHistory, OutputReserve — **never exceeds the model window.** Fixed slots measured exact; the dynamic remainder split by weight with floors; steal from CodeContext then RepoMap; `BudgetInfeasible` if below the floor sum. Per-tier slot weights differ (a 7B-12GB profile weights CodeContext at ~66.7%). B-10 stops the 5 task_runner sites that were passing `profile=None` (which zeroed the budget) and detects the model tier at bootstrap.

**Empirical context calibration (J-34, PLANNED — the anti-folklore unit).** The field's "when does context degrade" is a Reddit argument (42%? 67%?) and ELAI's own `CLOUD_TRIM_FRACTION=0.85` was the same folklore hardcoded. J-34 *measures* it: `elai bench context-fill` runs fixture tasks at stepped fill levels scored by *verified outcomes* (never LLM-judge) → a per-(model,lane) degradation curve + knee → `ModelCapabilityProfile.calibrated_trim_fraction` + `positional_value_curve`. The 3E allocator then orders slots by *measured* positional value (high-sensitivity slots at the measured edges, recoverable bulk in the trough) — cache-safe because the layout is fixed per model at calibration time, never per-call. This is the one axis where neither ELAI nor the field ships anything; ELAI measures its own.

**Read-access ladder (J-103, PLANNED).** Closes "read FILENAME, ELAI decides how much." A gated read-*escalation* ladder: `SymbolCard` (DKG) → `Skeleton` (B-29 traversal) → `HotPathExcerpt` (E-17 FAST/STANDARD) → `CodeWindow` (full file). Read the cheapest rung that proves it has what was asked, escalate on miss (a hallucinated identifier climbs to the full window, never returns a thin rung). Deny only on a security overlap, never on economy (downgrade-not-deny). The savings meter = would-have-read-vs-served tokens.

**Retrieval-abstention gate (J-73, PLANNED).** ELAI *always* retrieves in a retrieval tier; Repoformer shows a `should_retrieve` threshold yields >70% dense-setting speedup at >80% abstention precision — when local context suffices, retrieval is pure cost. J-73 repurposes ELAI's shipped abstention machinery to gate the cascade. Fail-closed: uncertain RETRIEVES, never silently skips.

**Retrieval that returns snippets, not dumps (Phase 1E, base).** The full pipeline (BM25 → dual-vector → PageRank → CatRAG → A-RAG) targets **55-87% token savings vs naive full-chunk retrieval** with sub-2s latency — progressive disclosure with "(previously read)" placeholders instead of re-pasting.

**Non-prompt steering + context-frugal tools (J-24, PLANNED).** Activates the dark sidecar `/v1/steering` ActAdd path — an activation-steering vector shapes behavior at *zero context tokens* (a residual-stream add, not text) — plus threads Tool Search (77K→8.7K tool-def) and MCP defer-loading. A deterministic observation-size pre-assessment (NONE/SELECTIVE/ALL) runs before any tool result is inserted.

**Deterministic context compression (I-14, PLANNED).** Tier-1 *deterministic*: headless-LSP slicing (extract exactly the AST node a compiler error references) + tree-sitter AST folding (irrelevant function bodies → `/* elided */`, signatures intact). A 10K-token compiler panic enters context as a ≤500-token slice with a reversible receipt. Tier-2 (prose only, VRAM-gated) LLMLingua-2-class. Cipher/abbreviation substitution BANNED.

---

## 6. Stage 6 — Caching: never pay to re-read the same prefix

**Prompt caching on the production path (H-1, PLANNED activation).** The main coding-agent loop *already caches* (ephemeral markers on the last system block + last tool def). Two gaps H-1 closes: the parallel `RuntimeAdapter` path hardcodes `cache_control:None` (ships zero markers for its secondary callers despite advertising `supports_caching:true`), and the hit-rate monitor is wire-dark. H-1 unifies marker logic and lights up live hit-rate measurement.

**Cache-stable layout + breakpoints + 1h TTL (H-2, PLANNED).** ELAI's allocator specifies 2 cache breakpoints (after PhaseSpec, after CodeContext) and computes a blake3 byte-prefix stability flag — but the whole mechanism is INERT (the flag was only `eprintln!`'d, fed empty slices, reachable only under a local budget policy). H-2 feeds real bytes, consumes the flag to *place* markers on the cloud path only on a verified-stable prefix, adds a second tail breakpoint, opts into 1h TTL, and adds a `debug_assert` that a cache-bust is a *test failure*. A deliberate prefix perturbation FAILS the assert.

**Cache-invariant economics middleware (I-10, PLANNED).** Byte-frozen prefix discipline end-to-end: no timestamps/UUIDs/mutable endpoints in the prefix; dynamic tool restriction via `tool_choice`/`allowed_tools` payload attributes, NEVER prompt-text mutation; TTL tiering (1h frozen prefix, 5m volatile tail) chosen by a cache-economics planner reading inter-call gap stats. The **hit-rate hard guard**: an unexplained miss-rate >10% on identical task classes HALTS and dumps a blake3 prefix diff naming the leaking mutation — the "March-6-class silent economics drift is caught by the harness, not the invoice."

**Local prefix-KV reuse (J-36, PLANNED).** The resident sidecar loads weights once and the head (system+rules+anchor) is byte-stable per session, but the shipped `PrefixKVCache` leases only on the non-streaming path, so the live streaming lane re-prefills the full head cold every step. J-36 extends the lease to streaming so a head hit prefills only the tail delta — with a per-model/quant eligibility guard (TQ-on / sliding-window / unknown-arch → ineligible → cold prefill, logged) so the cache is never wrongly assumed valid. Reconciles the H-50 tension: **freeze the head, minimize only the tail.**

**Immutable-transcript projection (L-16, PLANNED, structural).** Adopts Context-Warp-Drive's inversion: the raw transcript is permanent ground truth and the provider array is a *pure deterministic projection* of it (`fold(log, budget, policy, freeze) → view`). This makes cache-stability a *type property* not a doctrine, and collapses four mechanisms (mutate / recover_stubs / check_lossless / edit_checkpoints) into parameterizations of one projection. The E-25 ladder / H-6 masking / 3E allocator / H-50 policy become strategies inside `fold`.

---

## 7. Stage 7 — Emit cheap: output bills 5x, and caching never touches it

The corpus is explicit (H-3): **output bills ~5x input and caching never touches it**, so output is minimized directly.

**Focus Tunnel — FIM / DIFF / STRUCTURED (B-28 PLANNED; H-3 activates it).** Three deterministic emission modes:
- **FIM** (Qwen-native fill-in-middle, no tool schemas, ~350 tokens/step saved, single-region edits),
- **DIFF** (unified diff, ~30-50% smaller than full file, multi-region, reuses B-20's 3-tier edit matcher),
- **STRUCTURED** (fallback for new files + cloud).
H-3's finding: DIFF *never emits in prod* (the editor override was hardcoded `None` → always whole-file STRUCTURED), FIM is double-gated dark, and streaming early-abort is dark on all paths (the validator was hardcoded `None`). H-3 threads a real buffer editor so DIFF emits on cloud, constructs the streaming validator + calls `on_chunk` every ~50 tokens, and writes the early-abort SSB row. Streaming validation saves ~90% generation time on a bad gen caught at token 50 vs 500 (B-28).

**Tool Search Tool + advanced-tool-use header (H-4, PLANNED).** ELAI loads its full 16-tool catalog into every request and sends no advanced-tool-use header. H-4 moves the catalog behind the Tool Search Tool (loads defs on demand) — a **~77K→8.7K tool-def reduction** per request — gated on catalog size so a tiny tool set stays inline.

**Deterministic token-efficient emission (J-82, PLANNED, measure-first).** Three non-generative non-critical legs: a TOON-or-measured output format for the heaviest tool returns, a dictionary-encoding lane for repetitive logs (only where dropped volatiles are journal-archived), and a protected-span tokenizer (fenced/path/identifier spans preserved byte-for-byte) hardening the prose-stripping. Every number ELAI-measured, never the author's "40%/3x."

**Output meta-commentary enforcement at runtime (J-106, PLANNED).** The op-emission roles emit a work-log grammar (READ/FOUND/DECISION/ACTION/RESULT) with a banned-phrase reject + action-density metric — "act, don't narrate" operationalized so agents don't burn output tokens on ceremony. Hard boundary: op-emission roles only, NEVER constrained-decode reasoning roles.

---

## 8. Stage 8 — Escalation: one deterministic hop, condensed, not a re-dump

When a cheap lane fails an off-agent receipt, escalation is a bounded, typed hand-off — not "re-send the whole history to a bigger model."

**Typed BeliefState hand-off (I-11 / J-83, PLANNED).** A typed, versioned `BeliefState` (tried / failed-because / current-AST-state / constraints, DKG-keyed) *replaces raw history* on every cross-model or cross-tier hand-off. A local→cloud escalation carries only prefix + BeliefState (journal-verified ≤~2K tokens vs the ≥20K raw-history control). J-83 completes it into the reasoning carrier: each entry commits only if its off-agent receipt PASSES (else abstain), and a committed entry is *frozen* against later drift. The honesty job periodically re-runs a task from full history vs BeliefState and ledgers the success delta — "we know what our compression costs."

**Cascading escalation (I-7).** Strict waterfall (see Stage 3). The escalation target is H-7's cloud architect. `EscalationCascade` rows show the condensed payload size.

**Voluntary abort + dead-end hashing (I-8, PLANNED).** An explicit `acknowledge_infeasible` tool credited by the abstention score; the AST/approach hash of an aborted path is cached so an isomorphic future branch is pruned *before* an LLM call (zero model calls on a re-attempt of a known dead end). A 15s deterministic-check spin = abort + penalize.

---

## 9. Stage 9 — Context lifecycle across a long session (the >150K-context problem)

This is ELAI's single most-developed cost area, and it maps directly onto the screenshot's #1 finding.

**Regime-aware context lifecycle (H-50 / H-51, PLANNED — the corrected keystone).** The finding (user's catch, confirmed): **cloud and local need OPPOSITE context policies, and ELAI was running one cloud-shaped policy for both.** Local was unprotected (flat-100K auto-compact that never fires <32K; an 85% gate reading 64K while ignoring `ELAI_LOCAL_MAX_CONTEXT`; `recover_stubs` re-inflating trims). Fix: one-window-truth + `ContextPolicy`-by-`BackendMode` — local: unload / keep-stubs-unloaded / selective-reload; cloud: keep-cache-prefix (masking, no LLM re-summarize on the cached prefix, because that would bust the cache). H-51 adds cross-session sleep-time handoff: a grounded topic-digest from the journal, auto-detected, one-tap-confirm, tiered load ~20-27K → ~1-3K.

**Compaction ladder + observation masking (E-25 / H-6, PLANNED activation).** A 6-tier compaction ladder above the existing handoff. Deterministic observation masking halves cost while *preserving the cache prefix* — three real-but-never-called symbols (`adaptive_reduce`, `recover_stubs`, `get_relevant_facts`) get wired into the live read paths so a `/continue` rehydrates real content via `recover_stubs` (not a hardcoded `[stubbed]` placeholder).

**Local `_compact_history` recipe (J-76, PLANNED).** RLM's compaction as the H-50 *local-lane* recipe (trigger 85%-window, summarize preserving intermediate numerics, truncate to `[system+initial+summary+continue]`) — because ELAI's local lane has flat-100K auto-compact that never fires <32K. Cloud stays deterministic masking. Import the method, measure the own curve.

**Session handoff / DAG (Phase 3H, base; B-9 activation).** DAG-based session versioning + structurally-lossless 3-pass trimming; handoff fires at **85% token usage**; stubs always recoverable from the SSB by seq. Immutable DAG, branch/merge with file-overlap conflict detection.

**Context-activation wires (J-95, PLANNED).** Three dark call-site wires the deep-dive found: wire `adaptive_reduce` (relevance ladder) into the live turn-overflow branch *before* lossy compaction (drop bloat by relevance, not blind truncation); route slot assembly through the pristine-context filter (failed-path bursts become contrastive constraints, not raw text); verify+wire the BeliefState condense on escalation.

The reason this stage matters for the screenshot: ELAI's answer to "long sessions cost more" is to **automate** the `/compact` and `/clear` advice — auto-compact at the *measured* (not folklore) threshold, auto-handoff, and never LLM-re-summarize the cached prefix — instead of relying on the user to remember to run it.

---

## 10. Stage 10 — Offline / idempotent lanes and repeat-work caches

**Batch API for non-interactive lanes (H-5, PLANNED — none exists today).** A batch client (POST `/v1/messages/batches`, **flat 50% pricing**, 1h cache TTL) routes the non-interactive lanes — Cosmos refresh, web-research fan-out, bench/eval — through it, keeping the interactive `/task` + chat on the synchronous streaming path. A human-waited call never batches.

**Oracle + semantic caches (C-30 / H-8 / I-15, PLANNED).** `OracleAnswerCache` (blake3-keyed, tier-derived TTL: T1=30d … T4=never) for idempotent grounding answers, disk-backed so they survive restart (H-8). Confined to *idempotent* grounding queries only, never context-dependent code generation. I-15 caches intermediate summaries (DKG chunk summaries, research extracts) and serves them on semantically-similar queries with an exact-blake3 fast path before any vector math.

**Template crystallizer (C-25, PLANNED).** A repeated task (3 consecutive successes) crystallizes into a `CompiledPipeline` cached at `.elai/cache/compiled_pipelines.msgpack` (7-day TTL); the 4th run uses the template (`cache_read_tokens>0`, lower tokens) instead of re-decomposing from scratch. Plus a `CumulativeTokenTracker` that recalibrates the attention ceiling from actual response sizes.

---

## 11. Stage 11 — Measure it (so the cost claim is falsifiable)

The whole thing is instrumented, because "cheaper" that you can't recompute from the journal is a marketing claim, not an engineering one.

**Token ledger (J-28, PLANNED).** Fixes four grep-confirmed defects: the streaming-local capture bug (`input_tokens=0` on streaming local turns), per-lane semantics divergence (sidecar `input_tokens` = FULL prompt vs cloud = uncached suffix — *never naively summed*), sonnet-rates-for-every-model mispricing, and the missing rollup keys. Emits ONE uniform per-turn `UsageTurnEvent` from both lanes (identity/lane/backend/model/task/session + 4 counters + est_cost + elapsed) and an `elai usage` rollup verb.

**Usage window + live meter (J-29, PLANNED).** A dedicated `UsageWindow` (per-model/task/session/lane + cache effectiveness + tool/file-op counts) and a *live mid-stream token meter* in the chat status line (today it's a 5-10s poll *after* the turn). Local lane shows token counts not `$0.00`; a derived autonomy ratio.

**Cost cockpit (H-9, PLANNED).** The cloud spend ledger is dark end-to-end (SpendAccumulator + CloudSpend emits are gated `if let Some(ssb)`, the client ctor never chains the writer → CloudPanel renders all-zeros). H-9 attaches the writer, builds the cost feeder, and adds a per-response input/output/CACHED breakdown + cache-expiry timer + $/resolved-task + cache-hit-rate live — "the instrument that makes H-1/H-2 falsifiable."

**Routing-economics + gate-tax benches (K-2 / K-4, PLANNED).** `elai bench routing-economics` reports % steps offloaded, $ delta, and quality delta (verified-outcome rate) for cloud-only vs the H-7 hybrid — "the industry's first production hybrid number." `elai bench gate-tax` reports the composed friction (added wall-clock + over-block rate) so the safety machinery's cost is bounded, not hidden.

---

## 12. Does ELAI solve the six findings in the screenshot?

The screenshot is the user's *own Claude-Code usage while building ELAI*. So there are two honest framings, and both matter:
(A) does ELAI-the-product architecturally address each cost dimension, and (B) does ELAI's own dev harness (which generated this usage) currently eat its own dogfood. The answer to (A) is "yes, comprehensively, mostly in spec"; the answer to (B) is "no — the harness is itself a live example of the problem ELAI is built to solve."

| # | Screenshot finding | ELAI's design answer | Owning units | Verdict |
|---|---|---|---|---|
| 1 | **55% at >150K context** — "longer sessions cost more even cached; /compact, /clear" | Regime-aware auto-compaction (cloud≠local), auto-handoff at the *measured* 85% knee (not folklore 0.85), masking that preserves the cache prefix, immutable-transcript projection, deterministic compression. ELAI *automates* the /compact + /clear advice. | H-50/H-51, J-34, J-76, E-25/H-6, 3H, L-16, I-14 | **Directly addressed** — this is ELAI's most-developed cost area. Mostly PLANNED; 3E allocator SHIPPED. |
| 2 | **46% from subagent-heavy sessions** — "each subagent runs its own requests; configure a cheaper model for simpler subagents" | Per-role runtime is *exactly* "configure a cheaper model per subagent": architect=flagship, editor/simple-subagents=cheap 7B/free-pool, verifier=alt. Local-first waterfall runs subagents on the $0 lane first. Modality router shrinks each subagent's per-turn cost. | B-37 (SHIPPED), H-7, J-48, J-54, J-55, I-7 | **Directly addressed by design** — B-37 shipped, the cheap-model routing (H-7) is wire-dark/PLANNED. |
| 3 | **34% while 4+ sessions ran in parallel** — "all sessions share one limit; queue instead" | Partial: BSP VRAM-aware concurrency scheduling (3D), free-API rotation across keys/quota (J-54), burst-amortized scheduling (I-10), Batch API for non-interactive lanes (H-5), local-lane offload (I-7). ELAI spreads load off the shared paid limit onto local + free + batch. | 3D, J-54, I-10, H-5, I-7 | **Partially addressed** — mitigated via local/free/batch offload and rate-aware pooling; it cannot change Anthropic's shared account limit. Multi-seat fair-share (J-49) is the org-grain version. |
| 4 | **25% from 8+ hour sessions** — "background/loop sessions add up" | Watchdogs (idle/no-progress/safety timeouts, B-22b), voluntary abort + dead-end hashing (I-8), anti-spiral, background task manager with auto-detach (C-25), autonomous supervision hardening with mid-loop cost/iteration/wallclock caps (J-81), sleep-time cross-session handoff (H-51). | B-22b, I-8, C-25, J-81, H-51 | **Addressed** — long/loop sessions are bounded by deterministic watchdogs + a hard mid-loop cost cap. Mostly PLANNED. |
| 5 | **13% from general-purpose subagents** — "cheaper model or tighter prompts" | Same as #2: per-role runtime picks the cheapest capable model per role; the modality router + minimal compiled schemas (I-36) tighten each subagent's prompt; tool-call adherence floor keeps sub-floor models off tool-heavy roles. | B-37, H-7, J-55, J-51, I-36 | **Directly addressed by design** — the role→cheapest-model mapping is the literal answer. |
| 6 | **16% from MCP server local-rag** — "MCP results stay in context; /compact to flush or disable" | ELAI's retrieval is designed to *not* dump: snippet-based progressive disclosure (55-87% savings, 1E), deterministic tool-result compression (I-14), dual serialization (full to dashboard, condensed to context, I-13), observation masking (H-6), MCP tool token-budget enforcement + defer-loading (E-24, H-34), read-access ladder (J-103). | 1E, I-14, I-13, H-6, E-24/H-34, J-103 | **Architecturally addressed** — ELAI's whole retrieval design is "return the cheapest sufficient snippet, not the file." **The irony:** the harness's *own* local-rag MCP (16% of this usage) is the un-optimized version ELAI is built to replace. |

**Bottom line on the screenshot:** ELAI is, almost line-for-line, a system engineered to solve exactly these six cost dimensions — context bloat, subagent model tiering, shared-limit throughput, long-session drift, general-subagent cost, and MCP context accumulation. That is not a coincidence; the corpus was authored *against* these exact failure modes (E-39 names the wallet-loop incident; H-plan Track 1 is titled "make ELAI provably CHEAPER on the SAME cloud model than Claude Code / Cursor / Codex"). But the honest caveat below is load-bearing.

---

## 13. The honest caveat (grounded-by-default requires this)

**Most of the cost machinery is SPECIFIED and PARTIALLY BUILT, not SHIPPED-AND-WIRED.** H-plan Track 1's own premise: the efficiency gains come "mostly by ACTIVATING wire-dark caching/output/routing substrate." Concretely, from the plans' own status tags:
- **SHIPPED:** the multi-runtime backend (B-37, live cloud Sonnet proven), the context-budget profile activation (B-10), the base FSM/SSB/allocator/handoff (phases 0-4), the main-loop prompt caching (partial).
- **WIRE-DARK (built, no production caller):** the per-role `RuntimeOrchestrator` + `RouteDecisionEvent` (H-7 wires it), the `RuntimeAdapter` cache markers (H-1), the cache-breakpoint mechanism (H-2, was `eprintln!`-only), `adaptive_reduce`/`recover_stubs`/`get_relevant_facts` (H-6), the cloud spend ledger (H-9, renders all-zeros), the sidecar prefix-KV lease on streaming (J-36), the sidecar steering path (J-24), DIFF/FIM emission (H-3).
- **PLANNED / MEASURE-FIRST:** cloud/hybrid modes (B-27/C-5), the modality router (J-55), the reasoning classifier (J-84), the read-access ladder (J-103), the token ledger fixes (J-28), Batch API (H-5, no transport exists), local serving router (J-48), free-pool (J-54), context calibration (J-34).

So the design is real and unusually complete, the skeleton (B-37, B-10, base engine) is live, and the optimization layer is largely an *activation* program rather than an invention program — which is a good sign for feasibility (the substrate exists) but means the headline "cheaper by delegation" numbers are, per the corpus's own K-1/K-2 claims-policy, **BENCH-PENDING** — measured on ELAI's own fixtures, never asserted from a paper constant.

**And the reflexive point:** the screenshot is this repo's dev-session usage. The ELAI *harness* that produced it (subagent orchestration + the local-rag MCP + long autonomous sessions) is currently the *un-optimized* version of the very thing ELAI-the-product is designed to fix. Eating that dogfood — routing the harness's own subagents through a cheaper-model policy, flushing local-rag results deterministically, capping the long loops — is exactly the activation work H-1..H-9 / J-28 / J-48 describe, not yet applied to the workflow root itself.

---

## 14. Grounding index

Read end-to-end for this map: `docs/specs/guide/steps/phase-index-MASTER.txt`; the B..N `*-SUMMARY_INDEX.md` files under `docs/thefinalglance/`; the J-plan summary in full (106 sessions). Canonical live status is the SSB baseline (SCHEMA/TV counters at `ssb.rs` / `action_type_lock.rs`, ~268/577 at 2026-06-26) and `ELAI-FEATURE-MAP.md`; re-grep at edit per the corpus's W-7/W-9 rule. Every unit ID above (B-N/C-N/D-N/E-N/F-N/G-N/H-N/I-N/J-N/K-N/L-N/M-N/N-N) resolves to its detail file in the same tree.
