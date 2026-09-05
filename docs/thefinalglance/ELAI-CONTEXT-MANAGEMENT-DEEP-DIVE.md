# ELAI — Context Management Deep Dive

A complete map of how ELAI manages context: in-session growth, the oversized-task problem (task bigger than the model window), cross-session handoffs, and reasoning-context control. Compiled from the design corpus (Phases 0-4M + B..Q plan families) and the harness tooling (.claude/). Every mechanism cites the file it lives in.

The thesis, restated once so the rest makes sense: the model is a replaceable cog that generates and selects; the deterministic harness owns the context. Three properties recur at every layer — off-agent evidence, fail-closed, determinism. Context is treated as a budgeted, journaled, reproducible resource the model never shapes on its own authority.

---

## TABLE OF CONTENTS

1. The four problems ELAI's context system solves
2. Layer map (which mechanism owns which concern)
3. Problem A — Fitting a single turn under the window: the context budget allocator (3E)
4. Problem B — Context growing across a long session: regime-aware reduction + compaction
5. Problem C — A task bigger than the window: decomposition + belief-state handoff
6. Problem D — Crossing a session/tier/model boundary: handoffs
7. Reasoning context (the GRC budget + belief-state carrier)
8. Memory tiers that feed context
9. Cache-stability as a context constraint (the prefix must not move)
10. The context-core inversion (L-16) — the end-state design
11. The harness's own context discipline (how the main model manages its own window)
12. Measurement — ELAI measures its own context behavior
13. Mechanism cross-reference index (the long table)

---

## 1. The four problems

ELAI's context system is built to answer four distinct questions, and conflating them is the main source of confusion. Keeping them separate:

- **A. Single turn > window.** One inference request would exceed the model's max context. Owned by the **context budget allocator (3E)** and the **decomposition node-admission gate (J-39)**.
- **B. Session grows over time.** A long interactive session accumulates turns until it crosses the threshold. Owned by **regime-aware reduction (H-50)**, the **6-tier compaction ladder (E-25)**, **observation masking (H-6)**, and **structurally-lossless trimming (3H)**.
- **C. Task is structurally bigger than any window.** No single turn can hold the task; it must be split into units small enough to fit. Owned by **recursive decomposition (4D / idea-intake)**, **hierarchical plan management (3G)**, and the **typed belief-state handoff (I-11 / J-83)** that carries state between units.
- **D. Crossing a boundary.** Session end, tier escalation (local->cloud), or model swap. Owned by **session handoff (3H)**, **sleep-time topic-digest handoff (H-51)**, **swap-restart (Q-4)**, and the **harness handoff protocol**.

---

## 2. Layer map

| Concern | Primary owner | Also involved |
|---|---|---|
| Per-turn budget allocation | 3E Context Budget Allocator (8 slots) | 3G (slot 3 load), 1D (repo map fit) |
| Staying under window in one turn | 3E adaptive reduction (3 levels) | 4G/B SPLIT_TASK (terminal) |
| Long-session growth | H-50 regime policy, E-25 6-tier ladder, H-6 masking | 3H trimming, Q-4 swap-restart |
| Oversized task | idea-intake decomposition (Concepts>Phases>Steps>BabySteps) | 3G token-bounded skeleton, J-39 admission gate |
| State across units/tiers | I-11 / J-83 typed BeliefState carrier | I-5 pristine-context injector, I-6 anchor pane |
| Cross-session resume | H-51 sleep-time topic-digest + tiered load | 3H DAG, harness handoff protocol |
| Reasoning depth bound | 2A GRC budget, 2C per-agent config, J-42 distill-reset | J-84/J-85 effort classifier |
| Cache stability (prefix must not move) | I-10 prefix invariance, H-2 breakpoints, L-16/17 fold+freeze | H-6 (cache-preserving reduction) |
| Memory feeding context | E-1 (A-MEM/MemGPT/sleep-time), C-48 trust tiers, L-24 conjecture tier | J-73 retrieval-abstain, J-104 live overlay |
| Compaction safety | P-20 pre-compaction flush, PreCompact hook | 3H recovery |
| Context-core design (end-state) | L-16 immutable transcript + pure fold-view | L-17 freeze epoch, N-11 journal-fold |

---

## 3. Problem A — fitting one turn under the window: the context budget allocator (Phase 3E)

Source: `docs/specs/guide/steps/phase-3E.txt`. This is the foundational mechanism; everything else composes with it.

**Fixed 8-slot layout, fixed order, never reordered (3E:21-30):**
```
Slot 1 SystemPrompt | Slot 2 Rules | Slot 3 PhaseSpec
--- cache breakpoint 1 (stable within a phase) ---
Slot 4 RepoMap | Slot 5 CodeContext
--- cache breakpoint 2 (stable within same file set) ---
Slot 6 SsbEntries | Slot 7 GrcHistory | Slot 8 OutputReserve
```

**Distribution (3E:101-119).** Fixed slots 1-3 measured exactly and subtracted from `model_max_context` first. `OutputReserve = max(output_reserve, 2000)` subtracted before the split. Remaining `dynamic_budget` split by fixed percentages: RepoMap 15%, CodeContext 40%, SsbEntries 25%, GrcHistory 20%. Invariant: `sum(all_slots) == model_max_context` — no waste, no overflow.

**Floors (3E:246-265).** SsbEntries >= 200, GrcHistory >= 500, OutputReserve >= 2000. If a slot falls below its floor after the percentage split, steal from CodeContext first, then RepoMap. If `dynamic_budget < sum(floors)=700`: `Err(BudgetInfeasible)`.

**Token counting (3E:64-73).** Allocation uses `token_count_estimate` (sampling-based, O(lines/N)) to avoid paying ~8x exact-tokenizer cost; final validation uses exact `token_count`. If the exact count exceeds `model_max_context` after allocation, adaptive reduction fires. Target latency for full budget computation < 50ms.

**3-level adaptive reduction (3E:142-166).** Each level logged to SSB `action_type='context_reduction'` with before/after counts:
- **L1 FilteredByRelevance** — raise SSB minimum overlap-score threshold; keep GRC history only for the current topic.
- **L2 ReducedGranularity** — RepoMap uses a higher PageRank density guard (fewer, higher-rank nodes); CodeContext uses signatures + doc comments instead of full chunks.
- **L3 ReducedScope** — RepoMap current file + direct deps only; CodeContext current file only; SSB entries the current agent's own only.
- **Terminal** — INTERACTIVE mode: `ESCALATE` to the user with a budget breakdown. AUTONOMOUS mode: `SPLIT_TASK` via PipelineSpec decomposition (Phase 4G; B-PLAN-SUMMARY:406 caps at 4 sub-tasks, depth 2). "Never silently truncate. Every reduction logged to SSB."

**Cache breakpoints (3E:49-53, 76-99).** Two `CacheBreakpoint { after_slot, prefix_hash, valid: bool }`. BP1 after PhaseSpec (slots 1-3); BP2 after CodeContext (slots 1-5). Hash is deterministic (FxHash/xxhash, NOT DefaultHasher). BP1 invalidated by phase transition / role change / rule-set change; BP2 by file-set change. Slots 6-8 recompute every turn. The allocator reports `CacheBreakpoint.valid` to the provider so cached KV is reused for the stable prefix.

**SSB entry file-overlap scoring (3E:122-128).** `score = |intersection(entry.files, agent.files)| / |agent.files|`. Sorted score DESC then seq DESC (recency). Score 0.0 excluded entirely. This is how cross-agent / cross-turn state is filtered to what the current agent actually touches.

---

## 4. Problem B — context growing across a long session

### 4.1 The regime-aware doctrine (H-50) — the central in-session insight

Source: `docs/thefinalglance/H-plan/H-50_regime_aware_context_management.md`.

The core thesis (H-50:1-3): **a local model (25-32K window, NO cross-request cache in production) and a cloud model (200K, caching live) need OPPOSITE context policies, and ELAI was running ONE cloud-shaped policy for both, so a long LOCAL session was effectively unprotected.** Cloud caching amortizes a 1.25x write over many 0.1x reads — cloud WANTS a big stable prefix. Local has nothing to amortize and no room — it must MINIMIZE the prefix.

Mechanism (H-50:17-22): `effective_context_window(backend)` (local = ELAI_LOCAL_MAX_CONTEXT; cloud = cloud ceiling) replaces a flat `DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD = 100_000` (conversation.rs:22) and a 64K-non-opus gate that ignored the local ceiling. `ContextPolicy` is chosen by BackendMode:
- **LOCAL** runs the cheapest-first ladder in the interactive turn: drop dead trailing observations via H-6 masking -> mask/stub bulky tool results -> compact the oldest turns. Does NOT auto-recover; `recover_stubs` becomes SELECTIVE.
- **CLOUD** keeps the cache-stable prefix intact.

The bug it fixed (H-50:9-14): the old gate read a 64K window ignoring ELAI_LOCAL_MAX_CONTEXT, so a Qwen local got a ~54.4K trigger ABOVE its 32K window; and even when compaction fired, `recover_stubs` re-inflated every stubbed tool result to full bytes before the model call -> net trim ~0. `adaptive_reduce` (context_budget.rs:1525) had zero production callers.

Honest residue (H-50:40): "local caching stays a no-op until the Tier-2 unbypass ships... caching reduces cost-per-token, not token-count — a small window is still small; window-minimization is the primary local lever."

### 4.2 Deterministic observation masking (H-6) — cache-preserving reduction

Source: H-SUMMARY_INDEX.md:105-119. "Deterministic observation masking halves cost while matching LLM-summarization AND preserves the cache prefix (HD-2)." Makes `adaptive_reduce` the default long-session manager, wires `recover_stubs` and `get_relevant_facts`. A long real session masks trailing observations (ContextReduction rows land) WITHOUT busting the cache prefix.

### 4.3 The 6-tier compaction ladder (E-25)

Source: `docs/thefinalglance/E-plan/E-25_claude_code_architecture.md`. A `CompactionLadder` (compaction_ladder.rs, ~400 LOC) runs cheapest-first; each tier produces `(new_messages, freed_tokens, tier_id)`:
- **T1 Budget Reduction** — strip whitespace, dedupe identical tool calls.
- **T2 Snip** — drop oldest tool-result payloads > 2KB, keep their summaries.
- **T3 Microcompact** — summarize the 3 oldest user/assistant pairs into a bullet list per pair.
- **T4 Context Collapse** — fold all > 5-turn-old tool results into one "older work" summary.
- **T5 Auto-Compact** — invoke the sidecar with a "summarize this transcript preserving X" prompt.
- **T6 Narrative-summary** — last-resort full re-narrate by the sidecar.

Stops at the first tier that drops below target. Each tier emits a `compaction_tier_applied` SSB row. Cache-aware ordering preserves the first N tokens (system + CLAUDE.md + first user turn) to maximize prompt-cache hits. The PreCompact SESSION_HANDOFF write runs AFTER the ladder as a fall-through (the ladder runs first; it is not replaced).

### 4.4 Structurally-lossless trimming (Phase 3H)

Source: `docs/specs/guide/steps/phase-3H.txt:263-326`. `trim_context(messages, config)` with `target_tokens = model_context_window - allocated_slots_tokens - output_reserve`. Three passes:
- **Pass 1 identify** — stream messages; tool results > `stub_threshold` (default 500, range 100-1000) marked for stubbing. User/assistant messages NEVER marked.
- **Pass 2 stub** — marked content replaced with `Stub { original_type, original_tokens, summary_hash: blake3 of original, ssb_seq }` (msgpack); `is_stubbed=true`.
- **Pass 3 validate** — for each stub, verify `ssb_seq` exists in SSB; if missing, UNSTUB (restore original, since later recovery is impossible). If total trimmed > target: `Err(TrimBudgetExceeded)`.

**Cache awareness (3H:143-145):** trimming only operates on slot 6+ content (SSB entries, GRC history, tool results). Slots 1-3 (system prompt, rules, phase spec) are NEVER stubbed — preserves the BP1 cache prefix. Slots 4-5 regenerate per invocation anyway.

**Recovery (3H:389-430):** `recover_stubs` loads trimmed messages, parses each Stub, queries SSB by `stub.ssb_seq`, verifies `blake3(recovered) == stub.summary_hash`. Recovery failure modes: seq not found -> `Err(StubRecoveryFailed)`; hash mismatch -> integrity error. Because nothing was destroyed (only stubbed), recovery is always possible.

---

## 5. Problem C — a task bigger than the window

When no single turn can hold the task, ELAI does NOT try to cram it in. It splits the task into units, each fit to the window, and carries state between them as a typed object.

### 5.1 Recursive decomposition (Phase 4D / idea-intake skill)

Source: `.claude/skills/idea-intake/SKILL.md`. Any idea becomes Concepts -> Phases -> Steps -> BabySteps, each level grounded, each baby step an 8-block wargame spec.

The cascade (idea-intake:23-54):
```
1.  regen-map
2.  idea-framer  ->  2a. idea-critic loop (max 3)  ->  2b. plan-auditor concepts
3.  USER REVIEW
4.  researcher per concept (N parallel, batch 4)
5.  concept-decomposer per concept  ->  5a. plan-auditor phases
6.  USER REVIEW
7.  researcher per phase (conditional)
8.  phase-decomposer per phase  ->  8a. plan-auditor steps
9.  step-decomposer per step  ->  9a. plan-auditor babysteps
10. babystep-author per baby step (J massively parallel, batch 4) -> babysteps/{id}.md
10a. plan-auditor babystep-detail
11. assemble decomposition-tree.md (TreeNavigator walks depth-first from a cursor)
12. plan-level audit
13. USER APPROVAL
14. hand to planning-workflow (TESTING_FIRST)
```

**Why one agent per node per level (idea-intake:58-60) — the context-management rationale:** "A single designer agent doing all 200 nodes in one context window loses quality past ~30K tokens of working memory. Specialized per-level agents with fresh per-invocation context produce better-grounded, better-scoped output." Decomposition IS a context-management strategy: the tree exists so each node can be authored and executed in a fresh, small context.

**Baby step grain (~100 lines each):** each baby step is wargame-grade — EXPECTED_OBSERVATION + ON_FAIL per move, FORK triggers, RECON_NEEDED, ABORT, VERIFY with off-agent pass artifacts, and the PRODUCTION_HOOK seam. The TreeNavigator (tree-cursor.txt) walks the tree depth-first so execution never needs the whole tree in context at once — only the current baby step plus its parent chain.

### 5.2 Hierarchical plan management (Phase 3G) — the token-bounded skeleton

Source: `docs/specs/guide/steps/phase-3G.txt`. Two artifacts: a skeleton and per-phase detail.

**Skeleton (3G:37-57):** `PlanSkeleton { phases: IndexMap<PhaseId, PhaseEntry> }`. Each PhaseEntry is one-liner-only: `goal` max 120 chars, inputs/outputs as ContractRefs, depends_on, depended_by, version. TOML-authoritative at `.elai_cc/pipelines/{id}/skeleton.toml` + `.elai_cc/pipelines/{id}/phases/{phase_id}.toml`.

**Token bounding (3G:107-116):** skeleton budget uses `token_count_estimate` during editing, exact `token_count` on write. If exact > 4096 tokens -> `Err(SkeletonBudgetExceeded)`. A 20-phase plan fits ~1-2K; the 4096 cap has a 3600 safety margin that triggers the exact check early.

**Slot 3 load (3G:271-305):** `load_phase_context(skeleton, phase_id)` loads skeleton + active phase detail + extracted contracts, verified to fit Slot 3 (PhaseSpec). If it exceeds Slot 3, ReducedGranularity fires: load skeleton + contracts only, defer full phase detail. If `skeleton_tokens > slot3_budget` (tiny model): `Err(SlotBudgetExceeded)` — the system cannot operate without the skeleton.

**Contract extraction (3G:118-128):** before elaborating Phase X, input_contracts = union of dep.outputs, output_contracts = union of dep.inputs, shared_constraints = global + X.constraints. The model elaborates AGAINST contracts, not against full plan text. This is how a phase knows what its neighbors need without loading them.

### 5.3 Decomposition node-admission gate (J-39) — bound-and-escalate sizing

Source: `docs/thefinalglance/J-plan/J-39_decomposition_node_admission_gate.md`.

Pre-execution TOTAL token cost is provably unpredictable (Pearson r < 0.15, 30x same-task variance), but INPUT context is exactly countable. The model PROPOSES a decomposition; a deterministic admission gate measures each node's INPUT lower-bound (transitive import/call cone via B-29, tokenize each file, sum) against the measured per-(model,lane) ceiling and REJECTS oversized nodes (SplitForced at a graph-community boundary) or ESCALATES them (EscalatedToCloud via I-7). **"Bound-and-escalate, never predict-precisely."** This is the structural answer to "what if a task is bigger than the window": the gate refuses to let an oversized node execute and forces a split or an escalation instead.

### 5.4 The typed belief-state handoff (I-11 / J-83) — what crosses a unit boundary

Sources: `I-SUMMARY_INDEX.md:216-230`, `docs/thefinalglance/J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md`.

The unit of reasoning that crosses a model/tier/unload boundary is a typed BeliefState, never raw transcript. J-83 is the keystone that completes I-11:
- **CV-1 entry shape:** `{claim, grounding_receipt: Option<ReceiptRef>, confidence_bit (read from a receipt), provenance: DKG key + source SSB seq}`.
- **CV-2 per-step receipt gate:** a step is COMMITTED iff its domain receipt passes; a failing/missing receipt -> ABSTAINS, never commits on the chain's own say-so (the poison-cascade fix).
- **CV-3 commit-and-freeze:** a receipt-confirmed entry is simultaneously grounded, compressed-safe, and frozen against the generator second-guessing it.
- **CV-4 boundary transport:** on escalate (I-7) the carrier is condensed-then-handed (typed entries cross, not raw history); on local unload (H-50) the carrier is the thing unloaded and selectively reloaded.

Differentiator (I-11): "a local->cloud escalation carries ONLY prefix + BeliefState (journal-verified payload <= ~2K tokens vs the >= 20K raw history control)" — and it is HONESTLY MEASURED by periodically re-running a task from full history vs BeliefState and ledgering the success delta.

### 5.5 Pristine-context injector (I-5) — failed paths don't poison the next unit

Source: `I-SUMMARY_INDEX.md:110-124`. When expanding a branch, context is rebuilt from the parent chain; failed siblings + raw tracebacks are HARD-EXCLUDED; the negative signal survives only as a compressed contrastive constraint ("attempt A failed on line-42 type X — avoid X"). Measured: a sampled A/B (pristine vs accumulated context on the same decision) via the E-19 reasoning-quality probe — the first quantified vividness tax. "A retry branch's outbound payload contains ZERO bytes of the prior 5K-line traceback while carrying the <= 30-token contrastive constraint."

### 5.6 Anchor pane slot (I-6) — lost-in-the-middle defense

Source: `I-SUMMARY_INDEX.md:126-139`. An immutable PrivilegedAnchor slot appended LAST (after the final cache breakpoint — HD-2-safe) carrying the TaskSpec, unresolved claims, and active constraints; the model must emit a short acknowledgement tag reflecting the active constraint before generating (recency-bias grounding). Now shipped as the 11th ContextSlot (ordinal 10, per commit 4a8a43f).

---

## 6. Problem D — crossing a boundary (handoffs)

### 6.1 Session handoff (Phase 3H) — the DAG of snapshots

Source: `docs/specs/guide/steps/phase-3H.txt`. A SQLite `session_dag.db` (single writer, not WAL) holds two tables:
- `snapshots(snap_id PK, parent_id FK, session_id, fsm_state, phase, step, findings, code_changes, tests_written, commit_hashes, created_at, tag)` where tag is `"handoff" | "branch:{name}" | "abandoned:{name}" | NULL`.
- `snapshot_context(snap_id FK, seq, role, content, is_stubbed, stub_ssb_seq, PK(snap_id,seq))` — per-message-row (diverges from a BLOB-per-snapshot spec; enables selective stub recovery without deserializing the whole context BLOB).

**Triggers (3H:432-450):** checkpoint after every GRC step, tool call, phase transition. Handoff at token-budget threshold (default 85%). The compaction-boundary test: inject at exactly 85% -> handoff triggers; 84.9% -> no handoff; 100% -> triggers (late but not missed). Threshold reads `effective_max_context` from the sidecar, not a hard-coded constant.

**Lifecycle (3H:432-498):** `create_handoff` runs the 3-pass trim, saves a `tag="handoff"` snapshot. `load_handoff` finds the latest handoff; if a branch exists it loads the BRANCH TIP. `commit_hashes` persist across handoff. Full lifecycle test: 50 messages -> trim -> handoff -> load -> recover = byte-equal original.

**Branch/merge (3H:114-126, 328-388):** max 3 active branches. `create_branch(name, from_snap_id)` copies snapshot_context from parent. `merge_branch(winner_name)` creates a NEW trunk snapshot from the winner tip (snapshots immutable; parent_id never mutated), tags losers `abandoned:{name}`. Abandoned snapshots stay in DB, excluded from active queries via `WHERE tag NOT LIKE 'abandoned:%'`.

### 6.2 Sleep-time cross-session handoff (H-51) — topic-digest, not transcript reload

Source: `docs/thefinalglance/H-plan/H-51_sleeptime_handoff_digest_autodetect.md`.

Thesis (H-51:3): today's cross-session resume is MANUAL (/continue) AND reloads the packed trimmed transcript (~20-27K tokens on local = the pollution). The harness handoff-protocol.md already produces the RIGHT artifact (~40-100-line topic-keyed curated digest) but only as prose. H-51 mechanizes it in three legs:
- **Sleep-time author:** at session end/idle a sleep-agent authors a TOPIC-keyed digest from the SSB journal — SUMMARY + DECISIONS (from intervention rows) + FILES (from FileWrite rows) + FSM/plan cursor, ~1-3K tokens, grounded in the journal not recollection.
- **Auto-detect + one-tap confirm:** a fresh REPL checks for {open topic-handoff + non-IDLE FSM + active DAG tip} keyed on cwd/project; surfaces ONE line.
- **Tiered load:** on confirm, load ONLY the ~1-3K digest by default; full transcript + tool-result stubs stay in SSB, lazy-pulled via H-50's SELECTIVE recover_stubs ONLY when the model asks. "Default local resume cost drops ~20-27K -> ~1-3K."

Prior art (H-51:6): Letta sleep-time (11K vs 20K, same accuracy); Mem0 (> 90% token savings, +26% LOCOMO); A-MEM (Zettelkasten linking). `HandoffLifecycleEvent` SSB rows (DigestWritten / ResumeOffered / ResumeConfirmed / TieredLoad). Default-ON detect + default-OFF auto-continue, kill-switch `ELAI_HANDOFF_AUTORESUME_DISABLE`.

### 6.3 Swap-restart (Q-4) — model substitution restarts the unit fresh

Source: `Q-SUMMARY_INDEX.md:32-37`. When the runtime swaps the model mid-unit (fallback/rotation/substitution), the loop must NOT keep iterating the new model on the old model's partial state (partial diffs, self-consistency, style commitments are model-conditioned; a swapped model inheriting them poisons per-model attribution). Q-4: restart fresh with ONLY the I-11 BeliefState + journal `ModelSwapRestart{discarded_turns}` so the routing-economics metric counts the restart tax. The context that crosses a model swap is the BeliefState, not the partial transcript.

### 6.4 The harness handoff protocol — the main LLM's own cross-session artifact

Source: `.claude/rules/handoff-protocol.md`. ONE topic-keyed file at `workflow/handoff/{topic-slug}.md` (topic-keyed, NOT session/date-keyed — this is what kills the sprawl of dated/per-session chains). GLOB for an existing handoff for this problem; APPEND if exists, CREATE from template if not.

**Anti-read-big when appending:** read only the HEAD (first ~40 lines: title + CURRENT STATUS + INDEX) to update the living fields, then append the new entry at the END. CURRENT STATUS + INDEX are the only living fields (overwritten on every append); prior entries are immutable history. Entry sections: SUMMARY / WORK DONE / FILES TOUCHED / USER CHOICES-DECISIONS / HOW TO PICK UP / KNOWN LIMITS-NEXT / GROUNDING-VERIFICATION. Authored by the main LLM directly (first-party tooling doc, NOT delegated to memory-writer); works in any FSM state.

---

## 7. Reasoning context (the GRC budget + distill-reset)

### 7.1 GRC budget with forced convergence (Phase 2A)

Source: `docs/specs/guide/steps/phase-2A.txt:39-44, 89-93`. `GrcBudget { max_calls, max_chars, calls_used, chars_used }` with `remaining_calls()`, `is_exhausted()`, `must_converge()` (true when `remaining_calls <= 1`). Default `max_calls=5, max_chars=800`. When `must_converge()` is true, only Converge steps are accepted; non-Converge returns `Err(BudgetExhausted)`. Budget decremented atomically per step. Per-topic isolation: exhausting topic "a" leaves topic "b" untouched. `think(Status)` is free (status is a free echo, never budgeted). Grounded in BudgetThinker (OpenReview 2025) + Token-Budget-Aware LLM Reasoning (ACL 2025).

### 7.2 Per-agent GRC config (Phase 2C)

Source: `docs/specs/guide/steps/phase-2C.txt`. `AgentGrcConfig { role, budget, required_steps, grounding_level, csq_required, allowed_methods, transition_overrides, require_scope_enumeration }`. Layered: global `[grc.defaults]` + per-role `[agents.{role}.grc]`, merged field-by-field. Per-role assignments (phase-index-MASTER:334-342):
- **Auditor:** budget (8, 1200), required_steps `[Observe, Hypothesize, Verify, Verify]` (double-verify), Verified-only, transition_overrides `{Verify: [Verify, Observe]}` (additive).
- **Context-compiler:** observe-only, `[Observe]`, `[FILE_CHECK, SOURCE_CHECK]` only.
- **Executor:** budget (8, 1200).
- **Memory-writer:** budget (2, 400).

**Anti-rubber-stamp (2C:86-97):** the auditor must find >= 1 issue per review. Enforced structurally: when the auditor's GRC session reaches Converge, the engine checks >= 1 prior ThinkStep has `step_type=Verify` with a `FINDING:` marker; if none -> `Err(InsufficientFindings)`.

**BUDGET_EXCEEDED is an agent death cause, NOT a restart (3C:169, 357-381):** GRC budget overflow detected -> inject convergence signal. Critically it does NOT increment restart_count and does NOT trigger restart — the agent is alive, just out of budget. Restarting would cause an infinite loop (budget exceeded -> restart -> exceeded again).

### 7.3 Within-step distill-reset (J-42) — bounded context, unbounded depth

Source: `docs/thefinalglance/J-plan/J-42_within_step_distill_reset_reasoning.md`. Today a single reasoning step runs straight-line until it hits the context wall. J-42: the model generates a reasoning segment AND proposes a compact progress-distillate; a DETERMINISTIC trigger (context-fill / GRC budget threshold) decides WHEN to checkpoint; a FROZEN admission check (bounded size + non-empty + uncertainty-preserved) decides whether the distillate is acceptable; then the harness RESETS the working context and re-seeds with `{pinned original goal + admitted distillate}`. InftyThink's sawtooth (bounded context, unbounded depth) within one step. The uncertainty-preservation check (arXiv:2603.24472): a distillate that strips all uncertainty FAILS admission. **"Bound-and-reset, never let-the-model-decide-when."**

### 7.4 Reasoning-budget measurement (K-15) + effort classifier (J-84/J-85)

Source: `K-SUMMARY_INDEX.md:388-411`. `elai bench reasoning-budget` — per-(model, task-type) verified-outcome-rate vs reasoning-compute curve + the measured peak/knee (a J-34 sibling, x-axis = reasoning budget not context-fill). The knee is the calibration source J-84/J-85's overthinking ceiling consumes. Differentiator: "receipt-grounded reasoning — every step's confidence is from an external receipt, never the chain self-grading."

---

## 8. Memory tiers that feed context

### 8.1 Infinite-context chat illusion (E-1)

Source: `docs/thefinalglance/E-plan/E-1_infinite_context_chat_illusion.md`. Replaces a flat 150-token injection with four interlocking primitives:
- **A-MEM notes (arXiv:2502.12110):** `memory_notes` SQLite table with bi-temporal columns (`valid_from`, `valid_to`, `trust_score`, `decay_rate`, `evidence_pointers`). Each note `{context, keywords, tags, links[]}`.
- **MemGPT Core+Archival split (arXiv:2310.08560):** a model-editable ~1500-token `CoreBlock { user_style, user_expertise, active_goal, opinions }` exposed via `core_memory_replace(section, new_text)` and `archival_search(query, top_k)` tool defs; the rest paged via tool calls.
- **Sleep-time compute (arXiv:2504.13171):** a `SLEEP_COMPUTE` FSM substate inside IDLE. After every successful turn, `should_reflect(turn_count, K=4)` decides whether to author notes + CoreBlock appends + decay ticks. ~50% test-time token reduction.
- **Zep bi-temporal decay (arXiv:2501.13956):** PPR-result post-filter drops rows where `valid_to IS NOT NULL`; reweights `trust_score *= exp(-decay_rate * (now - last_accessed))`. On contradiction (cosine < -0.4): emit `MemoryConflict`, do NOT auto-update.

**PPR-over-notes retargeting the PhaseSpec slot (E-1:65, 92-97):** selection is HippoRAG-style Personalized PageRank (arXiv:2405.14831) top-K=8 notes filtered to `fact_type IN ('mistake','preference','opinion','expertise') AND valid_to IS NULL`, reusing the 7-layer PageRank kernel at crates/knowledge/src/pipeline.rs:387.

### 8.2 Trust tiers control which memory feeds context (C-48)

Source: `C-SUMMARY_INDEX.md:691-695`. `TrustTier` enum (T0 user-direct / T1 codebase-derived / T2 web-sourced / T3 model-emitted). `MemoryView::T0_T1_only()` filter for safety-critical queries. Provenance propagation to prompts. This controls WHICH memory-tier rows feed context for safety-critical queries.

### 8.3 Conjecture-memory middle tier (L-24) — explore lane only

Source: `L-SUMMARY_INDEX.md:254-259`. The memory tier BETWEEN the append-only journal and the verified case graph: unproven conjectures preserved as first-class retrievable objects with a COMPUTED status (live / refuted / pending / suspended — a pure journal fold; refuting a premise SUSPENDS dependents; contradicted nodes QUARANTINED never deleted). EXPLORE LANE ONLY: a type-level lane token + journaled Retrieved audit row + negative contract test make an unproven conjecture structurally unretrievable into the proven code-fix path.

### 8.4 Retrieval-abstention gate (J-73) — don't retrieve when local context suffices

Source: `docs/thefinalglance/J-plan/J-73_retrieval_abstention_gate.md`. A `should_retrieve` gate in the pipeline BEFORE the BM25/ONNX cascade fires, reusing the shipped answer-abstention machinery. Repoformer (arXiv:2403.10059) shows a P(retrieve) threshold yields > 70% speedup in dense settings. A retrieval gated OFF is journaled so the saving is auditable and the false-positive (skipped-but-needed) rate is measurable.

### 8.5 Live-index overlay (J-104) — never retrieve a symbol you just edited

Source: `docs/thefinalglance/J-plan/J-104_live_index_overlay_idle_checkpoint.md`. B-31 indexing is incremental but BATCH, so an agent that just edited a file can retrieve a stale symbol until the next pass. J-104 adds an overlay-first read + an order-triggered atomic drain. Overlay-wins-on-conflict: a just-edited symbol is authoritative over the stale base; a deleted symbol is shadowed by a tombstone. The idle drain is ORDER-BASED, not wall-clock (keeps it replay-stable).

### 8.6 Read-access escalation ladder (J-103) — read the cheapest rung that proves it has what was asked for

Source: `docs/thefinalglance/J-plan/J-103_read_access_ladder_interceptor.md`. A raw-source read request is GATED — the agent must NAME the identifiers it expects; the enforcer verifies those identifiers exist in the served rung's real text BEFORE returning. Serves the cheapest rung (SymbolCard -> Skeleton -> HotPathExcerpt -> CodeWindow) satisfying the identifiers; climbs one rung on miss. Two load-bearing guards: BRITTLENESS (over-aggressive pruning flips correct to wrong -> must verify presence + escalate-on-miss) and OVER-BLOCK (an interceptor that denies a legitimately-needed read is catastrophic -> BIASES to DOWNGRADE + ESCALATE-ON-MISS, NEVER hard-deny on economy grounds).

### 8.7 Context-access spec per dispatch (J-78) — declared, not implicit

Source: `docs/thefinalglance/J-plan/J-78_context_access_spec.md`. Each agent dispatch carries an explicit access_spec naming which prior SSB rows / sibling outputs / memory namespaces it MAY read. The orchestrator builds the dispatch context from the access_spec ONLY — a read outside the spec is a violation. Makes I-5's pristine-context hard-exclude and the auditor firewall (zero memory) DECLARED rather than implicit.

---

## 9. Cache-stability as a context constraint (the prefix must not move)

On the cloud path, prompt caching amortizes a 1.25x write over many 0.1x reads — but ONLY if the prefix bytes do not change between turns. Several mechanisms exist solely to defend this.

### 9.1 Prefix invariance + TTL tiering + hit-rate guard (I-10)

Source: `I-SUMMARY_INDEX.md:198-214`. "Byte-frozen prefix discipline end-to-end: no timestamps/UUIDs/mutable endpoints in the prefix; dynamic tool restriction via tool_choice/allowed_tools payload attributes, NEVER prompt-text mutation." TTL tiering: 1h on the frozen prefix breakpoint, 5m on volatile tails, chosen by a cache-economics planner reading inter-call gap stats. The hit-rate HARD GUARD: "unexplained miss-rate > 10% on identical task classes HALTS and dumps a blake3 prefix diff naming the leaking mutation."

### 9.2 Cache-stable layout + breakpoints (H-2)

Source: `H-SUMMARY_INDEX.md:44-57`. The allocator specifies 2 breakpoints (after PhaseSpec, after CodeContext) and computes a blake3 byte-prefix stability flag (bp.valid). H-2 feeds real slot bytes, consumes bp.valid to PLACE cache_control markers on the cloud path, opts into 1h TTL, keys min-prefix on the exact model id, and adds the HD-2 debug_assert (a cache-bust is a test failure).

### 9.3 Deterministic observation masking is cache-preserving (H-6)

H-6 masking halves cost while preserving the cache prefix (HD-2 holds). LLM re-summarization on the cache prefix is REJECTED — it busts the prefix.

### 9.4 MCP/skill defer-loading — keep schemas out of the prefix until needed

Sources: H-34, E-25, P-20. The full inputSchema of every STDIO MCP server tool is the largest fixed tax on the 25-32K local window (mindstudio measured 55,000 tokens for a 93-tool GitHub MCP). Defer-load so only an always-floor + RAG-selected tools carry full schemas. P-20 generalizes: "a compact prompt index carrying name/description/path + a sha256 VERSION marker ~24 tokens/skill; the model reads the file only when needed, re-reads on version change; budget compact-degrade." ONE lazy-load mechanism for skills + rules + MCP.

---

## 10. The context-core inversion (L-16) — the end-state design

Sources: `docs/thefinalglance/L-plan/L-16_immutable_transcript_pure_fold_view_core.md`, `L-SUMMARY_INDEX.md:152-193`.

Origin (L-16:18-37): Context Warp Drive (measured ~89-93% provider cache-read) proved that a deterministic, never-mutate, pure-projection context core keeps the provider cache hot. ELAI's spec independently arrived at every neighboring piece but kept a mutate-then-recover core: compact_session rewrites the prefix; recoverability is bolted on by SSB stubs + blake3. "That is 'destroy then restore'. CWD is 'never destroy, project'."

The CC delta family (L-16:41-54):
- **CC-1 raw is immutable** — no context-shaping operation may mutate or delete an entry from the append-only transcript log.
- **CC-2 view is pure** — the provider-facing array is `fold(log, budget, policy, freeze) -> view`, a deterministic pure function; identical inputs MUST produce byte-identical output; reading wall-clock/RNG/process-global is a violation.
- **CC-3 prefix moves only on a named cause** — prefix bytes may change only at a freeze epoch (L-17) carrying an enumerated cause, journaled; an unexplained change is a hard gate failure.
- **CC-4 lossless by construction** — because the log is never destroyed, "lossless" reduces to a view-level assertion; the archive-before-drop gate is subsumed.

The build (L-16:58-82): all existing reducers become STRATEGIES inside fold — the E-25 ladder = budget strategy, H-6 masking = trailing-observation strategy, 3E allocator = per-slot budget, H-50 ContextPolicy = strategy-set selector. `compact_session` is re-specified: instead of building a new messages vector, the runtime holds the log and calls `fold()` each turn. `recover_stubs` becomes "recompute the view with a larger budget for the requested span." **STRUCTURAL HD-2:** because fold is pure and the log prefix immutable, the view prefix cannot move between turns unless budget/policy/freeze change — HD-2 stops being doctrine and becomes a type property.

**Freeze-epoch controller (L-17):** `evaluate_freeze -> Reuse | Recompute(cause)` over an enumerated cause set (FirstCall / ColdGapTtl / TailOverflow / BoundaryMismatch / HistoryRewound / PolicyChanged / PressureCeiling); every recompute journals its cause (CC-3).

**Projection corollaries (L-18):** losslessness is checked at the VIEW (assert K-12 critical atoms present, exact-match) and fail-closed safely (nothing is dropped from the log, so a fail means widen the budget, not data loss). Undo/redo/branch become recompute-the-view-at-a-log-index. `recover_stubs` collapses to project-at-span-with-larger-budget.

**Ambient page-in budget ladder (L-19):** a pressure ladder scaling page-in budget by the L-17 window band (healthy full / warning halved / critical quartered / overflow hints-only); matched content injected as APPEND-ONLY cards riding the L-17 tail (CC-2, never the prefix); failed-path recalls use the I-5 contrastive constraint shape, not a raw paste.

**N-11 generalizes the fold core beyond context (N-SUMMARY:211-224):** five other surfaces (canvas altitudes, plan status, billing balance, fleet rollup, coverage) re-derive-from-journal ad hoc; N-11 extends L-16's fold core so all five use one fold, covered by one Kani correctness proof instead of five separate audits.

---

## 11. The harness's own context discipline (how the main model manages its own window)

ELAI-the-product is distinct from the harness (`.claude/`) that runs its build. The harness has its own context rules, and they are the practical answer to "how does the main model avoid blowing its own context."

### 11.1 Anti-read-big (token-discipline.md)

The main LLM hard-refuses Read > 200 lines OR Grep with > 50 lines of output without spawning a subagent first. Agents have their own contexts that garbage-collect on return; the main LLM context window does not. Reference docs in docs/ + memory-bank/ are always delegated to context-compiler or paper-ingestor. EXCEPTION: deliberate scoped Reads of target files about to be Edited are allowed (authoring reads, not research reads).

### 11.2 Context delegation — main model never reads plan/context/memory directly (main-llm-role.md)

The main LLM does NOT read context-package.md, exploration-plan.md, workflow/sessions/{id}/exploration/*.md, or memory-bank files in HEAVY flows. Sub-agents do that. CLAUDE.md restates: "NEVER read plan files, memory-bank/, or context-package.md directly. Sub-agents do that. No exceptions."

### 11.3 The prompt-transform -> context-compiler -> integration-explorer -> memory-writer flow

Sources: `prompt-transform/SKILL.md:17-22`, `planning-workflow/SKILL.md:15`, `.claude/agents/context-compiler.md`, `.claude/agents/memory-writer.md`.
- **context-compiler** (sonnet) receives a task and writes a structured context package to `workflow/sessions/{sid}/context-package.md`, returns ONLY a one-line confirmation. "The main model never reads this file. The prompt-engineer agent reads it directly. This saves tokens in the main model's context window."
- **integration-explorer-elai** runs in parallel with context-compiler (cap 3), returns `workflow/sessions/{sid}/exploration/{sub_phase_id}-{subsystem}.md`.
- **prompt-engineer / prompt-transform** reads the context package directly (not inline) and transforms the task into a grounded prompt.
- **memory-writer** (haiku) executes all memory-bank updates for the MEMORIZING phase; operates in three sub-states (READ_TEMPLATES -> WRITE_UPDATES -> MEMORIZE_RAG) enforced by a hook.

### 11.4 FSM compaction resilience (PreCompact hook)

Source: `C:\Users\user\.claude\rag\hooks\pre_compact_check.py` (151 lines), registered in `.claude/settings.local.json`. Fires before Claude Code truncates history. Reads `.fsm-state.json` (state, phase, attempts, audit_cycles, agent_substate), reads activeContext.md, builds a `SESSION_HANDOFF:` block, replaces the existing one (or appends). Writes to BOTH the resolved activeContext.md AND the per-session `workflow/sessions/{sid}/activeContext.md`. Exit 0 always (never blocks compaction). The block instructs the post-compaction model: (1) re-read activeContext.md and the SESSION_HANDOFF, (2) call mcp_fsm_get_state to verify FSM position, (3) continue from the recorded state/phase.

**Manual handoff (planning-workflow:179):** if context is running low, immediately fill the SESSION_HANDOFF section before doing anything else — what was in progress, current hypothesis, things tried that failed and why, unresolved questions, exact next steps, files open. "This is the only information the next session gets."

### 11.5 Pre-compaction flush (P-20) — nothing destroyed that was never offered to durable memory

Source: `P-SUMMARY_INDEX.md:413-435`. The pre-compaction memory flush is an ORDERED journaled step: a silent turn writes durable notes BEFORE compaction, once per cycle. Invariant: "no compaction may destroy context that was never offered to durable memory." This is IN-SESSION, distinct from H-51's cross-session sleep-time handoff. observable_effect: "a memory_flush_event row lands BEFORE the compaction row (ordered); a compaction that fires before its flush FAILS."

### 11.6 Per-turn local token-hashing (Q-5) — replay scoping

Source: `Q-SUMMARY_INDEX.md:44-49`. Journal `blake3(emitted_token_id_seq)` per LOCAL turn on the J-28 usage row. Payoff: replay localizes divergence to the FIRST divergent turn, a same-class replay becomes token-verifiable, a cross-class replay reports precisely where the exogenous model diverged. Honest scoping: harness anywhere / local same-class hash-verified / cloud action-stream.

### 11.7 Advisor lane (Q-3) — cheap model every turn consults flagship as a budgeted tool

Source: `Q-SUMMARY_INDEX.md:25-30`. The executor stays the CHEAP model every turn; the flagship is a budgeted TOOL called ~once/task when judgment is needed; advice flows back; the SAME executor continues. Payload = the I-11 BeliefState (<= 2K). Advisory-never-the-gate.

---

## 12. Measurement — ELAI measures its own context behavior

### 12.1 Empirical context calibration (J-34)

Source: `docs/thefinalglance/J-plan/J-34_empirical_context_calibration.md`. The field's SOTA on "when does context degrade" is a Reddit argument (42%? 67%?); ELAI's own numbers were the same kind (CLOUD_TRIM_FRACTION=0.85 hardcoded). Four legs:
1. **Fill-degradation curve** (`elai bench context-fill`): fixture tasks at stepped fill levels (10%..95%) with deterministic real-repo content, scoring verified-outcome rate; output per (model, lane) a fill->outcome curve + its knee.
2. **ModelCapabilityProfile** gains `fill_degradation_curve` + `calibrated_trim_fraction`; context_policy prefers the measured value over 0.85.
3. **Positional placement policy:** a positional-value probe -> slot ORDER chosen by (slot recall-sensitivity x measured positional value) — high-sensitivity slots pinned to high-recall edges, bulk content in the trough. Cache-safe by construction (layout fixed per model at calibration time).
4. G-10 model-drift marks artifacts stale.

### 12.2 Context-rot ceiling + placement + local _compact_history (J-76)

Source: `docs/thefinalglance/J-plan/J-76_context_rot_calibration_riders.md`. Three knobs grounded by empirical findings: (a) cap retrieval chunk COUNT per query; (b) PLACE top chunks at head/tail, never middle ("lost in the middle" 30%+ loss); (c) prefer a focused pre-filtered prompt. Plus RLM's `_compact_history` (triggers at 85% of window, summarizes preserving intermediate numerics, truncates to `[system + initial + summary + "continue from summary"]`) as H-50's LOCAL compaction policy.

### 12.3 Context-management measured (K-17)

Source: `K-SUMMARY_INDEX.md:441-461`. Banks the differentiator: "ELAI MEASURES its per-(model,lane) context-degradation knee against gate verdicts (J-34) and runs OPPOSITE cloud/local context policies (cache-stable-prefix + late-compaction vs window-min + unload-early); the folklore window-fraction (0.75 local/0.85 cloud) and the slot ORDER are replaced by a measured knee." No new bench — J-34 IS `elai bench context-fill`. KD-4 framing: caching/compaction are provider-absorbed, so the durable claim is the verification-coupled MEASUREMENT, not the bare feature.

### 12.4 Deterministic context compression (I-14)

Source: `I-SUMMARY_INDEX.md:267-283`. Tier-1 DETERMINISTIC: headless-LSP slicing (extract exactly the AST node/declaration/failing line a compiler error references) + tree-sitter AST folding (bodies of irrelevant functions -> "/* elided */", signatures/types intact — BPE priors preserved); Tier-2 (gated, prose-only) LLMLingua-2 token classification. Cipher/abbreviation substitution BANNED. EVERY transform lands a reversible ContextTransformReceipt so replay reconstructs what the model saw. "A 10K-token compiler panic enters context as a <= 500-token slice with a receipt row whose reverse pointer re-expands to the original bytes (blake3 match)."

### 12.5 Dual serialization (I-13)

Source: `I-SUMMARY_INDEX.md:250-265`. OpenHands-style: `to_ui_dict` (full, dashboard) vs `to_llm_content` (condensed, context); plus a TraceQuery tool so the model can expand exact line ranges on demand (prevents condensation starvation). "A 500-file directory listing reaches the dashboard complete but enters context as the condensed form."

### 12.6 Context-window lift (E-32)

Source: `docs/thefinalglance/E-plan/E-32_context_window_extension.md`. Qwen2.5-Coder-7B is natively trained at 32,768 tokens; raising the documented ceiling from 25K to 32K is free (KV cache at NF4 grows ~700MB; use 25000-28000 in production for headroom). Found-in-the-Middle calibration (arXiv:2406.16008): a LogitsProcessor on requests with `prompt_len > 8K AND ELAI_FITM_ENABLED=1` runs one extra forward pass with a dummy-doc baseline and subtracts the positional prior — +15pp RAG accuracy training-free; default OFF until internal RAG-eval shows >= +5pp recall@3. Deep mode (ELAI_DEEP_MODE=1, >= 24GB VRAM) loads the 1M model with Dual Chunk Attention (arXiv:2501.15383). YaRN explicitly REJECTED (15-20% short-prompt quality drop). The SessionDag handoff threshold must read `effective_max_context` from the sidecar dynamically, not a hard-coded constant.

### 12.7 Local KV prefix reuse (E-2)

Source: `docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md`. Pre-compute KV for the immutable system prefix once at boot (keyed by blake3 of the prefix). On each request: slice cached KV into a fresh StaticCache, append per-request suffix KV via normal generate. Gate `ELAI_PREFIX_KV_REUSE=1`; default off until a determinism test passes 3 consecutive runs (byte-equivalent output at temp=0).

---

## 13. Mechanism cross-reference index

Format: `PLAN/PHASE — name — file — one-line what`.

Budget / single turn:
- `3E — Context Budget Allocator — docs/specs/guide/steps/phase-3E.txt — 8 fixed slots, % split, floors, 3-level adaptive reduction, 2 cache breakpoints.`
- `3G — Hierarchical Plan Management — docs/specs/guide/steps/phase-3G.txt — token-bounded skeleton (4096 cap) + per-phase detail + contract extraction.`
- `1D — PageRank repo map fit — docs/specs/guide/steps/phase-1D.txt — binary-search fit to max_map_tokens within +15% (undershoot guard 850).`
- `J-39 — Node-admission gate — H-plan/J-39...md — measure node INPUT lower-bound, SplitForced / EscalatedToCloud if oversized.`
- `B / 4G — SPLIT_TASK terminal — B-PLAN-SUMMARY:406 — autonomous terminal on budget exhaustion, max 4 sub-tasks depth 2.`

Long-session growth:
- `H-50 — Regime-aware context — H-plan/H-50...md — local window-min vs cloud cache-stable; effective_context_window(backend).`
- `H-6 — Observation masking — H-SUMMARY:105-119 — adaptive_reduce as default; cache-preserving reduction (HD-2).`
- `E-25 — 6-tier compaction ladder — E-plan/E-25...md — T1..T6 cheapest-first, cache-preserving ordering.`
- `3H — Structurally-lossless trim — docs/specs/guide/steps/phase-3H.txt:263-326 — 3-pass trim, stub tool results, slots 1-3 never stubbed.`
- `J-76 — Context-rot ceiling — J-plan/J-76...md — chunk-count cap, head/tail placement, _compact_history local policy.`
- `I-14 — Deterministic compression — I-SUMMARY:267-283 — LSP slice + AST fold, reversible ContextTransformReceipt.`
- `I-13 — Dual serialization — I-SUMMARY:250-265 — to_ui_dict vs to_llm_content + TraceQuery.`
- `J-42 — Within-step distill-reset — J-plan/J-42...md — sawtooth: checkpoint distillate, reset context, re-seed.`
- `P-20 — Pre-compaction flush — P-SUMMARY:413-435 — ordered memory-flush before compaction (nothing destroyed never-offered).`

Oversized task / decomposition:
- `4D / idea-intake — Recursive decomposition — .claude/skills/idea-intake/SKILL.md — Concepts>Phases>Steps>BabySteps, one agent per node, TreeNavigator cursor.`
- `3G — Token-bounded skeleton — docs/specs/guide/steps/phase-3G.txt — 4096-token skeleton cap, Slot 3 load with ReducedGranularity fallback.`
- `J-39 — Node-admission gate — H-plan/J-39...md — bound-and-escalate.`
- `I-5 — Pristine-context injector — I-SUMMARY:110-124 — hard-exclude failed siblings, contrastive constraint only.`
- `I-6 — Anchor pane slot — I-SUMMARY:126-139 — 11th slot, immutable, appended last (HD-2-safe).`
- `J-40 — Extractive research compression — J-SUMMARY:1080-1095 — model selects / harness fetches+compresses (~200 tok, verbatim).`

Boundary handoffs:
- `3H — Session DAG — docs/specs/guide/steps/phase-3H.txt — snapshot DAG, branch/merge (max 3), stub recovery from SSB.`
- `H-51 — Sleep-time topic-digest — H-plan/H-51...md — ~1-3K journal-grounded digest, auto-detect, tiered load.`
- `Q-4 — Swap-restart — Q-SUMMARY:32-37 — restart fresh on model swap, BeliefState only + ModelSwapRestart row.`
- `I-11 / J-83 — Typed BeliefState — I-SUMMARY:216-230 / J-plan/J-83...md — claim+receipt carrier, per-step receipt gate, commit-and-freeze.`
- `handoff-protocol — Harness handoff — .claude/rules/handoff-protocol.md — one topic-keyed file, head-only read + append.`
- `pre_compact_check.py — FSM compaction resilience — C:\Users\user\.claude\rag\hooks\ — SESSION_HANDOFF autosave before truncation.`

Reasoning context:
- `2A — GRC budget — docs/specs/guide/steps/phase-2A.txt — max_calls=5/max_chars=800, must_converge, per-topic isolation.`
- `2C — Per-agent GRC config — docs/specs/guide/steps/phase-2C.txt — layered defaults, auditor double-verify, anti-rubber-stamp.`
- `3C — BUDGET_EXCEEDED — docs/specs/guide/steps/phase-3C.txt:357-381 — agent death, NOT a restart.`
- `K-15 — Reasoning-budget bench — K-SUMMARY:388-411 — elai bench reasoning-budget, peak/knee calibration.`
- `J-84/J-85 — Effort classifier — J-plan — overthinking ceiling consumes the K-15 knee.`

Memory tiers:
- `E-1 — Infinite-context illusion — E-plan/E-1...md — A-MEM notes + MemGPT core/archival + sleep-time + Zep decay.`
- `C-48 — Trust tiers — C-SUMMARY:691-695 — T0..T3, T0_T1_only filter for safety-critical.`
- `L-24 — Conjecture-memory — L-SUMMARY:254-259 — explore-lane only, computed status, quarantined not deleted.`
- `J-73 — Retrieval-abstain gate — J-plan/J-73...md — should_retrieve before cascade.`
- `J-104 — Live-index overlay — J-plan/J-104...md — overlay-wins-on-conflict, order-based atomic drain.`
- `J-103 — Read-access ladder — J-plan/J-103...md — cheapest rung that proves identifiers, escalate-on-miss.`
- `J-78 — Context-access spec — J-plan/J-78...md — declared per-dispatch read whitelist.`
- `N-4 — One Claim/Belief + receipt memo — N-SUMMARY:85-103 — unifies four objects, case-graph receipt cache.`

Cache stability:
- `I-10 — Prefix invariance — I-SUMMARY:198-214 — byte-frozen prefix, TTL tiering, hit-rate hard guard.`
- `H-2 — Cache-stable layout — H-SUMMARY:44-57 — bp.valid consumed to place cache_control, HD-2 assert.`
- `H-6 — Cache-preserving masking — H-SUMMARY:105-119 — masking not re-summarization.`
- `H-34 / E-25 / P-20 — MCP/skill defer-loading — keep schemas out of prefix until needed.`
- `L-16 — Context-core inversion — L-plan/L-16...md — immutable transcript + pure fold-view (CC-1..CC-4).`
- `L-17 — Freeze-epoch controller — L-SUMMARY:152-164 — enumerated recompute causes, journaled.`
- `N-11 — Journal-fold substrate — N-SUMMARY:211-224 — one fold core, five surfaces, one Kani proof.`

Measurement:
- `J-34 — Empirical context calibration — J-plan/J-34...md — elai bench context-fill, fill->outcome curve + knee.`
- `K-17 — Context-management measured — K-SUMMARY:441-461 — measured knee replaces folklore fraction.`
- `E-32 — Context-window lift — E-plan/E-32...md — 32K native, FITM calibration, DCA deep mode, YaRN rejected.`
- `E-2 — Local KV prefix reuse — E-plan/E-2...md — StaticCache prefix-KV reuse.`
- `Q-5 — Per-turn token-hash — Q-SUMMARY:44-49 — blake3(emitted_token_id_seq), replay divergence scoping.`

---

## Sources

Design corpus (read in full or via cited line ranges by the research agents):
- docs/thefinalglance/ELAI-SPEC-OVERVIEW.md
- docs/specs/guide/steps/phase-index-MASTER.txt
- docs/specs/guide/steps/phase-1D.txt, phase-2A.txt, phase-2C.txt, phase-3C.txt, phase-3E.txt, phase-3G.txt, phase-3H.txt
- docs/thefinalglance/{B,C,D,E,F,G,H,I,J,K,L,N,O,P,Q}-plan/*-SUMMARY_INDEX.md
- docs/thefinalglance/E-plan/E-1, E-2, E-25, E-32 .md
- docs/thefinalglance/H-plan/H-50, H-51 .md
- docs/thefinalglance/J-plan/J-34, J-39, J-42, J-73, J-76, J-78, J-83, J-95, J-103, J-104 .md
- docs/thefinalglance/L-plan/L-16 .md

Harness tooling:
- .claude/rules/token-discipline.md, main-llm-role.md, handoff-protocol.md
- .claude/skills/prompt-transform/SKILL.md, planning-workflow/SKILL.md, idea-intake/SKILL.md
- .claude/agents/context-compiler.md, memory-writer.md
- C:\Users\user\.claude\rag\hooks\pre_compact_check.py
- .claude/settings.local.json (hooks block)

Items marked UNVERIFIED in the source research (cited but not located in a backing file by the research agents): the TreeNavigator implementation body in planning-workflow SECTION 1 (TREE-DRIVEN EXECUTION) — named by idea-intake step 14 but the implementation file was not located; `cache_edits` api.anthropic.com parameter (E-25 flags this as server-side-only / theoretical for local); phase-3H MergeResult::Conflict file-overlap detection (the phase file labels it UNGROUNDED); the exact hash algorithm for 3H stub integrity (phase file labels UNGROUNDED, blake3 assumed as fast-and-common). These are spec-author caveats, not gaps in this document.
