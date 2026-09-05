# GrcHistory Context Slot (Phase 3E)

- **ELAI ids:** phase-3E / P3E-E001..P3E-E036 / P3E-I001 / P3E-A001 / P3E-A002
- **Spec status (as designed):** WIRED (8-slot allocator); GrcHistory is slot 7 of that allocator
- **Crate / module (spec):** `crates/orchestrator` context-budget module (`ContextSlot::GrcHistory`, `ContextBudget`, `AdaptiveLevel::FilteredByRelevance`)
- **Purpose of this file:** enough architecture to rebuild how **GRC traces sit in the 8-slot context budget** — floors, 20% split, FilteredByRelevance (current-topic only), and cache-prefix rules. Not a full 3E encyclopedia of RepoMap/BM25.

## 1. What it is

**GrcHistory** is **slot 7** of the fixed 8-slot context budget: “prior reasoning for the current topic” (`phase-3E.txt:21-29`). The allocator never exceeds `model_max_context`. Dynamic remainder after exact slots 1–3 and `OutputReserve` is split **RepoMap 15% / CodeContext 40% / SsbEntries 25% / GrcHistory 20%**, then floors are applied (`phase-3E.txt:103-118`). GrcHistory sits **after cache breakpoint 2**, so it is **recomputed every invocation** and is **not** part of the cached KV prefix (`phase-3E.txt:78-93`). Adaptive attempt 1 (`FilteredByRelevance`) keeps **only current-topic GRC entries** and drops cross-topic (`phase-3E.txt:147-149`).

## 2. Why it exists

GRC traces are the working memory of 2A–2D. If they share an undifferentiated window with code and SSB, a long topic either blows the model cap or is silently truncated. If GrcHistory is placed **before** a cache breakpoint, every ThinkStep busts cloud prefix cache (`phase-3E.txt:90-97`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:306-316`). If FilteredByRelevance is implemented as a **token shrink** instead of a **topic filter**, a cousin organ keeps the wrong traces (`phase-3E.txt:147-149` vs `audit-implementation/phase-3E.md:211-212`). If the 500-token floor is stolen from, small models get **zero** prior reasoning (`phase-3E.txt:216-217`).

## 3. How it works (runtime)

1. **Who calls it.** Orchestrator, **per agent invocation inside a BSP superstep**, after `prepare_next_tasks()` and before `execute_tasks()` (`phase-3E.txt:133-139`).
2. **Slot order (never reordered)** (`phase-3E.txt:78-88`, `phase-3E.txt:216`):

   ```
   1 SystemPrompt     // GRC structured prompt per role
   2 Rules
   3 PhaseSpec
   --- BP1 (stable within a phase) ---
   4 RepoMap
   5 CodeContext
   --- BP2 (stable within same file set) ---
   6 SsbEntries
   7 GrcHistory       // this organ
   8 OutputReserve    // never allocated to content
   ```

   Discriminants: `SystemPrompt as u8 == 0`, `OutputReserve as u8 == 7` (`phase-3E.txt:216`). Therefore **GrcHistory as u8 == 6**.

3. **Compute (deterministic)** (`phase-3E.txt:101-113`, `:226-237`):
   - Measure slots 1–3 **exactly**; subtract from `model_max_context`.
   - `output_reserve = max(configured, 2000)`; subtract **before** the percentage split (`phase-3E.txt:229`, `P3E-E025`).
   - `dynamic_budget = model_max_context - sum(slots 1-3) - output_reserve`
   - `GrcHistory` allocation = `floor(dynamic_budget * 0.20)`
   - Remainder after floors of the four dynamic slots goes to the **largest** slot (CodeContext) so `sum(all_slots) == model_max_context` (`phase-3E.txt:231-238`).
   - Worked example: `compute(8192, [100,200,300], 2000)` → dynamic=5592 → GrcHistory=`floor(5592*0.20)=1118` (`phase-3E.txt:231-237`).

4. **Floor (`GrcHistory >= 500`)** (`phase-3E.txt:115-118`, `:250-254`):
   - If after the split GrcHistory < 500, **steal from CodeContext first, then RepoMap**.
   - Do **not** steal from SsbEntries or GrcHistory floors to feed code.
   - If donors cannot cover the deficit, or `dynamic_budget < 700` (`200+500` floors; OutputReserve already subtracted), `Err(BudgetInfeasible)`.
   - Example: `compute(4000, [100,200,300], 2000)` → dynamic=1400; GrcHistory floor 500 forces steal from CodeContext (`phase-3E.txt:256-257`).
   - `compute(3000, …, 2000)` → dynamic=400 < 700 → `BudgetInfeasible` (`phase-3E.txt:258`).
   - If CodeContext < 200 after steal: **SSB warning, not error** (`phase-3E.txt:261`).

5. **Fill GrcHistory content.** Prior `ThinkStep` / reasoning-trace rows for the invocation’s **current topic**, fitted with `token_count_estimate` during allocation and exact `token_count` at validation (`phase-3E.txt:66-72`). If exact assembled prompt > `model_max_context` → adaptive reduction (`phase-3E.txt:71`, `:144`).

6. **Adaptive attempt 1 — `FilteredByRelevance` (this organ’s reduction)** (`phase-3E.txt:147-150`):
   - SSB: raise minimum overlap threshold (configurable, **e.g. 0.3** — example, not a hard default: `P3E-I001`).
   - **GRC history: keep only entries for the current topic; drop cross-topic.**
   - Recount. If fits: stop.
   - Attempts 2–3 coarsen RepoMap/CodeContext / shrink scope; they are neighbors. Attempt 3 does **not** name further GrcHistory surgery (`phase-3E.txt:152-161`).
   - Max 3 attempts. Each logs SSB `action_type='context_reduction'` with before/after tokens (`phase-3E.txt:144-145`). **Those log rows must not enter the current invocation’s context** (`phase-3E.txt:327`).
   - Terminal: INTERACTIVE `ESCALATE` with budget breakdown; AUTONOMOUS `SPLIT_TASK` (Phase 4G). **Never silently truncate** (`phase-3E.txt:163-166`).

7. **Cache prefix rules (GrcHistory is outside the prefix):**
   - BP1 after PhaseSpec (slots 1–3). Invalidated by phase transition, role change, rule-set change (`phase-3E.txt:90-91`).
   - BP2 after CodeContext (slots 1–5). Invalidated by file-set change (`phase-3E.txt:92`).
   - Slots 6–8 **recomputed every invocation** because SSB/GRC change each turn (`phase-3E.txt:93`).
   - Changing GrcHistory **must leave BP1/BP2 valid** when slots 1–5 are unchanged (`phase-3E.txt:279`: same 1–5, changed slot 6 → BP2 still valid; slot 7 is likewise after BP2).
   - When BP1 valid: provider reuses KV for slots 1–3. When BP2 valid: slots 1–5 (`phase-3E.txt:95-97`). GrcHistory is **never** in that cached prefix.
   - `prefix_hash`: FxHash or xxhash, **NOT `DefaultHasher`** (`phase-3E.txt:51`, `:281`).

8. **Prompt assembly.** Slots 6–7 are **not** in the base prompt; they are injected as **structured context after `task_spec`**. Slot 8 is not in the prompt at all (`phase-3E.txt:171-177`). Allocator tracks 8 slots independently; assembler may merge 4+5 for the model (`phase-3E.txt:183-186`).

9. **Handoff / trim (3H neighbor).** Structurally lossless trimming preserves slots 1–3 (BP1). Trimming operates on **slot 6+** (SSB, **GRC history**, tool results) (`ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:126-127`, `phase-3E.txt:191-196`). BP2 is **recomputed** on new session; do not assume it survives (`phase-3E.txt:195-197`).

10. **Fail-closed.** `BudgetInfeasible` is a `Result` the caller must handle — never swallow (`phase-3E.txt:262`). Hard truncation after failed adaptive is **terminal**, not silent cut (`phase-3E.txt:72`, `:166`).

The model does not choose which GRC steps stay. The allocator + topic filter do.

## 4. Types, enums, objects

From `phase-3E.txt:21-61`:

```
enum ContextSlot {
  SystemPrompt,   // 1: GRC structured prompt per agent role
  Rules,          // 2
  PhaseSpec,      // 3
  RepoMap,        // 4
  CodeContext,    // 5
  SsbEntries,     // 6
  GrcHistory,     // 7: prior reasoning for current topic
  OutputReserve,  // 8: min 2000, never content
}

struct SlotAllocation {
  slot: ContextSlot,
  allocated_tokens: usize,
  floor_tokens: usize,     // 0 for slots 1-3; GrcHistory floor = 500
  actual_tokens: usize,
  source_truncated: bool,
}

struct ContextBudget {
  model_max_context: usize,          // ModelCapabilityRegistry
  slots: [SlotAllocation; 8],
  cache_breakpoints: [CacheBreakpoint; 2],
  total_allocated: usize,            // == model_max_context
  total_used: usize,                 // <= total_allocated
  adaptive_level: AdaptiveLevel,
}

struct CacheBreakpoint {
  after_slot: ContextSlot,           // BP1 PhaseSpec, BP2 CodeContext
  prefix_hash: u64,                  // FxHash / xxhash
  valid: bool,
}

enum AdaptiveLevel {
  Normal,
  FilteredByRelevance,   // attempt 1: SSB/GRC by relevance; GRC current-topic only
  ReducedGranularity,
  ReducedScope,
  Terminal,              // INTERACTIVE ESCALATE / AUTONOMOUS SPLIT_TASK
}
```

`ContextBudget` implements `Display` for breakdown (`phase-3E.txt:210`). `ContextBudget::new(model_max_context)` starts all allocations at 0 (`phase-3E.txt:209`).

GrcHistory **content type** (the traces themselves): 2A `ThinkStep` history / 3B `reasoning_trace` rows. Exact struct copied into the slot is **UNCERTAIN** in 3E (slot is a token budget, not a new GRC type).

## 5. Control flow / state machine

```
Normal
  exact token_count(assembled) > model_max_context?
    → FilteredByRelevance   // GRC: current topic only
         still over? → ReducedGranularity
           still over? → ReducedScope
             still over? → Terminal (Escalate | SplitTask)
```

**Owner:** orchestrator context allocator. GRC engine does not trim itself into the window (that later cousin is J-42 distill-reset — **not** this organ; `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:258-260`).

**Forced:** slot order immutable; GrcHistory never moved before BP2; floors never violated in a successful `compute`.

## 6. Grounding, receipts, gates

GrcHistory carries **already-grounded** 2A/2B steps; 3E does not re-verify claims. Selection key for attempt 1 is **topic equality**, not an LLM relevance score (`phase-3E.txt:149`). SSB slot 6 uses file-overlap `|{intersection}| / |agent.files|`; score 0 excluded (`phase-3E.txt:124-127`). That formula is **not** the GRC filter.

**Never allowed as a confidence signal:**
- LLM “drop the old Observe, keep my last paragraph”
- `DefaultHasher` prefix validity (`phase-3E.txt:281`)
- Window-as-tier (using raw context size instead of the 8-slot math)
- Silent truncate of GrcHistory to protect CodeContext without going through steal-then-adaptive-then-terminal (`phase-3E.txt:166`)
- Treating attempt-1 as `allocated * (1 - threshold)` without dropping cross-topic rows (implementation cousin: `audit-implementation/phase-3E.md:211-212`)

## 7. Budgets, limits, ceilings

| Knob | Value |
|---|---|
| GrcHistory % of `dynamic_budget` | **20%** |
| GrcHistory floor | **≥ 500** tokens |
| SsbEntries floor (neighbor that shares steal) | ≥ 200 |
| OutputReserve | **≥ 2000**, `max(configured, 2000)` |
| Infeasible if `dynamic_budget < 700` | 200+500 |
| Steal order | CodeContext → RepoMap |
| Adaptive attempts | **3** max |
| Assembly latency | **< 50ms** via `ClockProvider`, not `Instant::now` in production (`phase-3E.txt:73`, `:351`) |
| Invariant | `sum(all_slots) == model_max_context` (`phase-3E.txt:238`) and `<= model.max_context` (`phase-3E.txt:359`) |
| Determinism | 100× byte-identical assembly (`phase-3E.txt:348`) |

**Later overlay (not this organ):** B-10 7B/12GB weights `12.5/66.7/12.5/8.3` for slots 4–7 (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B10-009). Rebuild **3E** as 15/40/25/20 unless a later locked plan replaces it.

**Later extra slots** Memory / AnchorPane / 11th PrivilegedAnchor (`ELAI-SYSTEMS-OVERVIEW.md:122-123`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:199`) are **not** 3E. This organ is the original 8.

## 8. Roles and neighbors

Every agent invocation gets a GrcHistory slice of **its** topic. 2C role does not change the 20%/500 math; it changes **how much GRC text exists** (auditor 10/1200 vs memory-writer 2/400).

**Consumes:** topic id, prior ThinkSteps / `reasoning_trace`, `model_max_context`, exact sizes of slots 1–3. **Produces:** `SlotAllocation` for `GrcHistory`, `adaptive_level`, BP validity **unchanged by this slot**.

**Must not fork:** a 9th “scratch reasoning” slot that sits in the prefix; letting the model compact GrcHistory with T5 auto-compact (E-25 ladder is session-growth, not this per-turn slot — `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:107-117`).

**3G:** PhaseSpec (slot 3) load is a neighbor; ReducedGranularity on slot 3 if skeleton+detail overflow (`ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:170-171`) — does not resize GrcHistory’s percentage.

**3H:** may stub GrcHistory content; must not stub slots 1–3.

## 9. Config / flags / defaults

- Percentages and floors are **spec constants**, not per-role TOML in 3E.
- Attempt-1 overlap raise “e.g. 0.3” is **example**, not a pinned default (`P3E-I001`, `phase-3E.txt:148`).
- `output_reserve` argument to `compute`, floored at 2000 (`phase-3E.txt:229`).
- No dedicated cockpit; `Display` + INTERACTIVE escalate breakdown (`P3E-E035`, `P3E-A002`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `ContextSlot::GrcHistory` | `phase-3E.txt:28` |
| `ContextBudget::compute(model_max_context, fixed_slot_sizes, output_reserve)` | `phase-3E.txt:226` |
| `AdaptiveLevel::FilteredByRelevance` | `phase-3E.txt:57`, `:147-150` |
| `CacheBreakpoint` BP1 after PhaseSpec, BP2 after CodeContext | `phase-3E.txt:50`, `:82-85` |
| steps 3E.1–3E.7 | types, compute, floors, breakpoints, SSB filter, adaptive, e2e (`P3E-E036`) |
| production: **no** `Instant::now` in the context-budget module | `phase-3E.txt:351` |

## 11. Journal / SSB events

```
action_type = 'context_reduction'
payload: level, before_tokens, after_tokens, slots_affected
```

(`phase-3E.txt:144-145`, `:323`). Visible **next** invocation only (`phase-3E.txt:327`). GrcHistory **content** may also appear as 3B `reasoning_trace` (`phase-index-MASTER.txt:422`). Do not double-count reduction logs into slot 7 of the turn that logged them.

## 12. Tests and observables

Pinned checks that **mention GrcHistory** or bind it:

- Discriminant order: `SystemPrompt==0`, `OutputReserve==7` (`phase-3E.txt:216`).
- `compute(8192,[100,200,300],2000)` → GrcHistory **1118** (`phase-3E.txt:235`).
- Remainder assigned to CodeContext so sums match (`phase-3E.txt:236-237`).
- `compute` fuzz: 100 different `model_max_context` values, `sum==dynamic_budget` for dynamic slots / `sum(all)==model_max_context` (`phase-3E.txt:240`, `:244`).
- `output_reserve` arg 500 still yields 2000 (`phase-3E.txt:241`).
- `compute(4000,[100,200,300],2000)`: GrcHistory **≥500** via steal (`phase-3E.txt:256-257`).
- `compute(3000,…)` → `BudgetInfeasible` (`phase-3E.txt:258`).
- Same slots 1–5, changed slot 6 → BP2.valid still true (`phase-3E.txt:279`) — GrcHistory similarly after BP2.
- Adaptive force overflow: levels 1→2→3; SSB rows contain level/before/after/slots_affected (`phase-3E.txt:321-323`).
- INTERACTIVE terminal `Err(Escalate(breakdown))`; AUTONOMOUS `Err(SplitTask)` (`phase-3E.txt:324`).
- Gemma 4 E2B 128K; also 4096 and 131072; assembly <50ms via ClockProvider; 100× identical (`phase-3E.txt:337-348`).
- Inject estimate 20% low → adaptive fires (`phase-3E.txt:350`).
- Grep production module: zero `Instant::now` (`phase-3E.txt:351`).

Done-gates: floors ssb≥200 / grc≥500 never violated; never silent truncate; deterministic (`phase-3E.txt:358-366`).

## 13. Rebuild contract

- Exactly 8 `ContextSlot` variants in this order; GrcHistory is 7th (index 6).
- 20% of `dynamic_budget` after exact 1–3 and `max(reserve,2000)`.
- Floor 500; steal CodeContext then RepoMap; `dynamic < 700` → `BudgetInfeasible`.
- `sum(slots) == model_max_context`; no waste, no overflow.
- GrcHistory **after BP2**; every-turn recompute; **must not** participate in provider prefix KV.
- `FilteredByRelevance` **drops cross-topic GRC rows**, not merely scales the token cap.
- Adaptive ≤3, SSB `context_reduction`, no current-turn circularity, never silent truncate.
- Deterministic hasher for `prefix_hash`.
- No `Instant::now` in production allocator.
- Prompt assembler injects slots 6–7 as structured context after task_spec; OutputReserve never in the prompt.

A sliding window of raw transcript labeled “history”, or a prefix-cache that includes the latest Verify step, is a cousin.

## 14. Non-goals / banned designs

- Reordering slots (`phase-3E.txt:78`).
- Silent truncation (`phase-3E.txt:166`).
- `DefaultHasher` (`phase-3E.txt:51`, `:281`).
- Putting GrcHistory in the BP1/BP2 prefix (`phase-3E.txt:86-97`).
- Stealing from the GrcHistory floor to save CodeContext (steal is the other direction) (`phase-3E.txt:251-252`).
- LLM-chosen drops from GrcHistory.
- Window-as-tier (raw `max_context * 0.2` without subtracting slots 1–3 and OutputReserve).
- `Instant::now` in the production module (`phase-3E.txt:351`).
- Feeding `context_reduction` SSB rows into the same invocation’s SsbEntries/GrcHistory (`phase-3E.txt:327`).
- Assuming BP2 survives 3H handoff (`phase-3E.txt:195-197`).
- Implementing attempt 1 as `GrcHistory.allocated *= (1-threshold)` **without** the current-topic filter (`phase-3E.txt:149` vs audit scale-only cousin).
- Collapsing this organ into J-42 distill-reset or GRC-v3 BeliefState (those are later, PLANNED; `ELAI-FEATURE-MAP.md:642-650`).

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-3E.txt` (types L21-61, order L76-98, distribution L101-119, adaptive L142-166, assembly L169-186, handoff L189-197, compute example L226-237, floors L246-265, BP tests L277-279, exit L358-366). Index: `phase-index-MASTER.txt:505-515` (8 slots, 15/40/25/20, floors, FilteredByRelevance “higher overlap + same-topic GRC only”). Overview: `ELAI-SPEC-OVERVIEW.md:86-87`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:120-124`. Context deep dive §3 + §7 + §9: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:56-81`, `:240-256`, `:306-316`. Exhaustive: P3E-E001..E036, P3E-I001. Audit (impl cousin on L1): `audit-implementation/phase-3E.md:195-221`. Feature map 1.2 does not list 3E; 3E lives under context (`ELAI-FEATURE-MAP.md:563-580` vs systems §5).

## 16. Gaps

- Exact serialization of a GrcHistory **item** (full `ThinkStep` vs SSB `reasoning_trace` vs both) — 3E only says “prior reasoning for current topic” (`phase-3E.txt:28`).
- Sort key inside the topic (seq DESC? calls_used order?) — **UNCERTAIN**. SSB slot specifies score DESC then seq DESC (`phase-3E.txt:126`); GrcHistory attempt 1 does not.
- Whether Normal-mode GrcHistory already excludes other topics, or only FilteredByRelevance does (`phase-3E.txt:28` “current topic” vs `:149` “keep only current topic, drop cross-topic” implying Normal might have been mixed).
- Attempt-1 overlap threshold default (`P3E-I001`).
- `model_max_context` source values (`P3E-A001`).
- How `source_truncated=true` on GrcHistory interacts with “never silently truncate” (truncating to allocation after a declared adaptive level vs truncating with no log). Spec forbids **silent** truncate; fitted-to-allocation with `source_truncated` is a `SlotAllocation` field (`phase-3E.txt:37`). **UNCERTAIN** whether GrcHistory may set that flag in Normal or only after logged adaptive.
- B-10 percentage overlay vs 3E 15/40/25/20 — which wins at runtime: **UNCERTAIN** in 3E (3E is 15/40/25/20).
- 11th AnchorPane slot vs this 8-slot organ (`ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:199`) — later; do not add it while rebuilding 3E GrcHistory.
