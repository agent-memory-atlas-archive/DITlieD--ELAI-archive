# Pristine context injector (I-5)

- **ELAI ids:** I-5 / ID-1 / ID-5 / IS-6 / CL-1 / WAVE 2 / SCHEMA 247 / Track 13 EVIDENCE-RANKED SEARCH
- **Spec status (as designed):** PARTIAL (`ELAI-FEATURE-MAP.md:840-846` — `pristine_path_filter.rs` BUILT + `i5_pristine_context_contract.rs` proven; map’s older PLANNED label is stale). Original plan WAVE 2 BUILD (`I-plan/I-5_pristine_context_injector.md:1-3`, `I-plan/I-SUMMARY_INDEX.md:110-124`). I-4 production caller is DARK behind `ELAI_SEARCH_LADDER_ENABLE` (`I-04-evidence-ranked-search-ladder.md` header).
- **Crate / module (spec):** `crates/orchestrator/src/pristine_path_filter.rs` (`PristinePathFilter::build_branch_context`); production: `search/ladder.rs::SearchLadder::build_pristine_context` → `run_best_of_n`; observe-only cousin: `context_budget.rs::observe_pristine_slot_candidates` (J-95). SSB: `ContextPruned` / `ContextPrunedKind`.
- **Purpose of this file:** enough architecture to rebuild per-branch context rebuild that hard-excludes failed-sibling raw traceback. Not a status ledger.

## 1. What it is

The **pristine context injector** is the per-branch context rebuilder that enforces the ID-1 **stateless-generator / trajectory-pruning** doctrine: when a branch expands, context is **rebuilt from the parent chain**, not truncated from an accumulated window. Failed-sibling payloads and their **raw tracebacks are HARD-EXCLUDED**. The negative signal survives **only** as a compressed **contrastive constraint** of `<=30` tokens (`"attempt A failed at line-42 type X — avoid X"`). A sampled A/B (pristine rebuild vs accumulated context on the same decision) is measured via the existing E-19 probe and ledgered so the vividness-tax claim is recomputable under `ELAI_SEED` (`I-plan/I-5_pristine_context_injector.md:4-17`, `I-plan/I-SUMMARY_INDEX.md:110-116`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:193-195`).

GRC-v3 (J-83) **rides** this organ so a failed-path traceback cannot poison the next unit’s carried prefix (`J-83-grcv3-beliefstate-carrier.md` §3 step 10, `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:29`). I-11’s `failed_because` condensate is a **different** encoding of the same doctrine on the **escalate** wire — cite, do not rebuild (`I-11-typed-belief-state.md` §1, `belief_state.rs:129`).

## 2. Why it exists

When a branch expands, accumulated failed-sibling tracebacks occupy context and pollute the generator’s next step — the **attention-sink / vividness-trap** (ID-1). ELAI’s verifier stack makes the tax **measurable**: a sampled A/B on the same decision via E-19, not a paper import (`I-plan/I-5_pristine_context_injector.md:4-9`, `:100-101`). If this organ is missing, a retry prompt contains the prior 5K-line traceback and the model over-anchors on the failed path. If it is faked (truncate the window, or “the model should see what failed”), truncation **preserves the sink at a shrunken size**; carrying raw traceback **is** the anti-pattern (`I-plan/I-5_pristine_context_injector.md:18-22`).

This is not information suppression: it is a **different encoding** that keeps the causal signal without the attention sink (`I-plan/I-5_pristine_context_injector.md:15-17`). I-4 **calls** this filter per branch and must not re-accumulate raw tracebacks (`I-04-evidence-ranked-search-ladder.md:14`, `I-plan/I-4_evidence_ranked_search_ladder.md:7-8,58-59`).

## 3. How it works (runtime)

1. **Who calls it.** Production (R7), not a `#[cfg(test)]` stand-in:
   - I-4 `SearchLadder::run_best_of_n` → `build_pristine_context` → `PristinePathFilter::build_branch_context` (`pristine_path_filter.rs:13-15,100-102`, `search/ladder.rs:315-340`, `I-plan/I-5_pristine_context_injector.md:70-72`).
   - Reached from `task_runner::dispatch_search_ladder` when `ELAI_SEARCH_LADDER_ENABLE` is truthy (`pristine_path_filter.rs:13-15`, `I-04-evidence-ranked-search-ladder.md` §3 step 1).
   - I-7 waterfall constructs the same ladder (and therefore this filter) at `task_runner::maybe_run_escalation_waterfall` (`task_runner.rs:14296-14298`).
2. **Input.** `branch_id`, `session_id`, `sibling_payloads: &[SsbPayload]` (same parent, different `branch_id`), optional `traceback_bytes`, `BackendMode`, and a mutable `ContextBudget` (`pristine_path_filter.rs:103-111`). The **caller** is supposed to populate per-slot contents **before** the call (`pristine_path_filter.rs:86-90`). The model does **not** choose what is excluded.
3. **Regime (deterministic, H-50 compose).** `regime_policy(mode)` selects the regime-aware window. I-5 **calls**; it does not re-author the policy (`pristine_path_filter.rs:20-21,112`, `I-plan/I-5_pristine_context_injector.md:30-31,67`).
4. **Sibling hard-exclude (deterministic, ID-1).** If `sibling_payloads` is non-empty: sum exclusion weight, emit `ContextPruned(SiblingExcluded)` with `excluded_bytes`. Raw sibling payloads **must not** appear in the rebuilt branch context (`pristine_path_filter.rs:118-136`, `ssb/payloads.rs:17153-17155`).
5. **Traceback contrast (deterministic, ID-1).** If `traceback_bytes` length `>= TRACEBACK_COMPRESS_THRESHOLD_BYTES` (512): build a `<= MAX_CONSTRAINT_TOKENS` (30) contrastive phrase from a 120-byte preview, word-split truncated. Emit `ContextPruned(TracebackContrasted)` with `original_traceback_bytes` + `constraint_phrase`. The raw traceback is **absent** from the outbound payload (`pristine_path_filter.rs:39-46,138-168`, `I-plan/I-5_pristine_context_injector.md:14-16,90-92`). Payloads **below** 512 bytes are not compressed (`pristine_path_filter.rs:43-46`).
6. **Vividness A/B (sampled, E-19 compose).** When `umbrella_enabled()` (`ELAI_REASONING_QUALITY_PROBE_ENABLE`), emit `ContextPruned(VividnessProbe)` carrying `outcome_delta`. Plan: both paths on a sampled fraction; delta = pristine_score − accumulated_score, recomputable under `ELAI_SEED` (GD-2, ID-5) (`I-plan/I-5_pristine_context_injector.md:16-17,43-44`, `ssb/payloads.rs:17159-17161,17195-17197`). Live emit currently stamps `Some(0.0)` as a placeholder until I-4 wires real scoring (`pristine_path_filter.rs:95-97,171-175,246-254`).
7. **Prefix re-anchor (deterministic, HD-2 / IS-6).** After filtering, `budget.update_breakpoints(&slot_contents)` re-anchors blake3 cache breakpoints. Named `ContextSlot` discriminants, never raw ints (`pristine_path_filter.rs:92-93,178-199`, `context_budget.rs:1335-1362,1850`). Plan pin `context_budget.rs:1507` / `:1079` is **stale** against current source (`I-plan/I-5_pristine_context_injector.md:25-26`).
8. **Commit artifact.** `BuildBranchContextOutcome { excluded_bytes, original_traceback_bytes, constraint_phrase }` plus the `ContextPruned` SSB row(s) (`pristine_path_filter.rs:263-274`).
9. **Abstain / fail-closed.** SSB write failure → `PristineFilterError::SsbWrite` (no silent skip of the receipt) (`pristine_path_filter.rs:54-57,229-233`). A zero `ContextPruned` row-count after a compressing traceback is **FAIL**, never UNVERIFIED (`i5_pristine_context_contract.rs:121-133`). Claiming a numeric vividness-tax improvement **without** the A/B row is banned (ID-5) (`I-plan/I-5_pristine_context_injector.md:21-22,78-79`).
10. **I-4 live v1 cousin.** `build_pristine_context` currently passes `&[]` siblings and `None` traceback (“generator owns sibling exclusion upstream”) and still re-anchors breakpoints (`search/ladder.rs:315-333`). That is a **partial freeze of the telemetry call**, not the full plan filter. Rebuild the **plan** organ: caller supplies sibling payloads + failed traceback; the filter excludes them.

The model may **generate** candidate source on a branch. It may **not** decide what stays in context, carry raw traceback, set `must_preserve` (K-12 / I-11), or self-grade the vividness tax.

## 4. Types, enums, objects

Copy from the I-5 plan wiring contract plus live structs that name this organ. Do not invent fields.

```
MAX_CONSTRAINT_TOKENS: usize = 30
TRACEBACK_COMPRESS_THRESHOLD_BYTES: usize = 512

PristinePathFilter { ssb: Arc<Ssb> }
fn build_branch_context(
  budget: &mut ContextBudget,
  branch_id, session_id,
  sibling_payloads: &[SsbPayload],
  traceback_bytes: Option<&[u8]>,
  mode: BackendMode,
) -> Result<BuildBranchContextOutcome, PristineFilterError>

BuildBranchContextOutcome {
  excluded_bytes: u64,                 // SiblingExcluded weight
  original_traceback_bytes: u64,       // 0 if no traceback above threshold
  constraint_phrase: Option<String>,   // <=30-token phrase; None if no compression
}

PristineFilterError { SsbWrite(String) }

ContextPrunedKind { SiblingExcluded, TracebackContrasted, VividnessProbe }

ContextPrunedPayload {
  kind: ContextPrunedKind,
  branch_id: String,
  session_id: String,
  excluded_bytes: u64,                 // SiblingExcluded; 0 otherwise
  original_traceback_bytes: u64,       // TracebackContrasted; 0 otherwise
  constraint_phrase: Option<String>,   // TracebackContrasted only
  outcome_delta: Option<f64>,          // VividnessProbe only (pristine - accumulated)
}

ProbeKind::VividnessProbe              // E-19 6th arm; primary row is still ContextPruned
                                       // (pristine_path_filter.rs:241-245)

ContextSlot { SystemPrompt=0, Rules, PhaseSpec, RepoMap, CodeContext,
              SsbEntries=5, GrcHistory, OutputReserve, Memory,
              Evidence, AnchorPane=10 }   // SLOT_COUNT = 11
```

(`pristine_path_filter.rs:39-46,52-57,68-70,263-274`, `ssb/payloads.rs:17151-17198,11848-11852`, `context_budget.rs:23,1335-1362`)

**Live cousins (do not rebuild as this organ):**

- J-95 `observe_pristine_slot_candidates` — default-OFF (`ELAI_J95_PRISTINE_ON_BUDGET=1`); **emits accounting, never hard-strips slots** (`context_budget.rs:1406-1438`).
- I-11 `BeliefState.failed_because` / `constraints` — escalate-wire condensate, not per-branch prompt rebuild (`I-11-typed-belief-state.md` §4).
- K-12 `CriticalAtom.must_preserve` — orchestrator-set; LLM never sets (`belief_state.rs:78-86`, `J-83-grcv3-beliefstate-carrier.md` §5).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator. The model proposes a branch; the host rebuilds context, then the generator runs. No LLM inside the filter (`I-plan/I-5_pristine_context_injector.md:70-72`).

```
branch expansion (I-4 SearchLadder / I-7 waterfall ladder)
     │
     ▼
regime_policy(mode)                         // H-50; window only
     │
     ├─ sibling_payloads nonempty
     │     HARD-EXCLUDE from rebuilt context
     │     emit ContextPruned(SiblingExcluded)
     │
     ├─ traceback_bytes >= 512
     │     compress to <=30-token phrase
     │     RAW traceback NEVER in outbound payload
     │     emit ContextPruned(TracebackContrasted)
     │
     ├─ umbrella_enabled()                  // E-19
     │     emit ContextPruned(VividnessProbe)  // delta ledgered
     │
     └─ update_breakpoints(slot_contents)   // HD-2 blake3 prefix
              return BuildBranchContextOutcome
```

**Forced transitions:**

- Failed-sibling raw traceback **cannot** enter the next branch’s context (ID-1, stub_ban) (`I-plan/I-5_pristine_context_injector.md:74-75`).
- Negative signal **only** as the `<=30`-token phrase (`I-plan/I-5_pristine_context_injector.md:14-16`).
- Truncation of an accumulated window is **not** a legal substitute for rebuild (`I-plan/I-5_pristine_context_injector.md:19-20`).
- Prefix breakpoints must be re-anchored after rebuild (HD-2) (`pristine_path_filter.rs:92-93,178-199`).

**I-4 prune vs I-5 exclude:** L1 HardVeto **kills** a branch (`I-04-evidence-ranked-search-ladder.md` §3 step 4). I-5 **strips that branch’s traceback** from a **sibling’s** prompt. Do not merge the two.

## 6. Grounding, receipts, gates

This organ **is** the ID-1 exclusion gate at branch-expansion grain. The vividness-tax **number** is an **external A/B receipt**, never a model self-grade (ID-5).

| Rank | Signal | Legal here? |
|---|---|---|
| 1 | Deterministic exclusion + journaled `ContextPruned` row | **the only rebuild signal** |
| 2 | E-19 A/B `outcome_delta` under `ELAI_SEED` | **measurement of the tax**, not a license to carry traceback |
| 3 | LLM “I learned from the error dump” / raw traceback as teaching material | **BANNED** (`I-plan/I-5_pristine_context_injector.md:18-19,74-75`) |

**Never allowed as a confidence / quality signal:** carrying the failed sibling’s full error output; window fill; LLM re-summarization of the traceback (that is still an attention sink, just shorter); a literature vividness-tax percentage quoted as ELAI’s number before the A/B row exists (`I-plan/I-5_pristine_context_injector.md:21-22,78-79`).

**E-19 compose:** `ProbeKind::VividnessProbe` is the 6th E-19 arm; the **primary** SSB row I-5 emits is still `ContextPruned(VividnessProbe)`, not a `ReasoningQualityProbe` row (`pristine_path_filter.rs:241-245`, `E-19-reasoning-quality-hooks.md` header, `ssb/payloads.rs:11829,11848-11852`). Do not re-author E-19 (`I-plan/I-5_pristine_context_injector.md:32-33,68`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| contrastive constraint | `<= 30` tokens (`MAX_CONSTRAINT_TOKENS`) | per compressed traceback (`pristine_path_filter.rs:39-41`) |
| traceback compress threshold | `512` bytes | per traceback (`pristine_path_filter.rs:46`) |
| preview window | first `120` bytes (~30 tokens × ~4 B/token proxy) | live compressor (`pristine_path_filter.rs:146`) |
| observable broken-arm traceback | prior **5K-line** / **>=5K tokens** when `--carry-tracebacks` | plan R2 (`I-plan/I-5_pristine_context_injector.md:90-92`) |
| SCHEMA | `ContextPruned` SCHEMA_VERSION **247** (2026-06-13) | `lib.rs:237-241`, `action_type_lock.rs:725-727` |
| Plan SSB baseline | +1 from LIVE **221/532** as of 2026-06-10; **re-read at edit (W-9)** | `I-plan/I-5_pristine_context_injector.md:105-106` |
| dashboard tile | wire-dark until **>=3** measured A/B runs with a consistent delta (GD-4) | Tier-2 (`I-plan/I-5_pristine_context_injector.md:47-49`) |

**Overflow:** a phrase that would exceed 30 tokens is truncated to the limit (`pristine_path_filter.rs:39-41,151-156`). Exact tokenizer vs word-split proxy: **UNCERTAIN** (§16).

**Do not confuse with:** Phase-2A per-topic `GrcBudget` 5/800; I-11 `CONDENSED_PAYLOAD_MAX_BYTES = 8_192` (escalate condensate, `escalation_waterfall.rs:70-75`); 3E window slots; H-6 `adaptive_reduce` (window compression — live comment treats it as **orthogonal**, `pristine_path_filter.rs:17-19`).

## 8. Roles and neighbors

Role-agnostic filter. I-4 / I-7 own **when** it runs.

**Consumes:** IS-6 `ContextSlot` / `update_breakpoints` (GD-1); H-50 `regime_policy` (GD-1); E-19 umbrella + `ProbeKind::VividnessProbe` (GD-1); I-4 per-branch expansion as the production caller (`I-plan/I-5_pristine_context_injector.md:64-68,108-113`).

**Produces:** rebuilt slot contents (plan: `SsbEntries` replaced); `BuildBranchContextOutcome`; `ContextPruned` rows.

**Must not fork:** a second masking layer that bypasses H-6 turn-drop (`I-plan/I-5_pristine_context_injector.md:76-77`); a prompt-only “please ignore prior errors”; carrying traceback on the I-7 / J-83 escalate wire (that is I-11 typed condensate — `J-83-grcv3-beliefstate-carrier.md` §3 step 8).

**Cite, do not re-author:**

- **I-11** BeliefState — `failed_because` is the **escalate** encoding of negative signal (`I-11-typed-belief-state.md`).
- **J-83** — commit-and-freeze + I-5/K-12 guard so failed-path traceback does not poison the next unit (`J-83-grcv3-beliefstate-carrier.md` §3 step 10).
- **I-4** — receipt-ranked ladder **calls** this filter (`I-04-evidence-ranked-search-ladder.md` §3 step 3).
- **H-39 / C-10 / D-13** — not this organ. H-39 scores abstention; C-10 catches **repetition**; D-13 is Mode B adversary. I-5 is prompt hygiene at branch expand.

**Plan vs live on H-6:** plan says I-5 **EXTENDS** H-6 turn-drop machinery and must not duplicate it (`I-plan/I-5_pristine_context_injector.md:20-21,28-29,109`). Live module header says H-6 `adaptive_reduce` is **orthogonal** (window compression vs per-branch exclusion) (`pristine_path_filter.rs:17-19`). Rebuild: **one** shared masking engine, no parallel turn-drop; do not invent a second H-6.

## 9. Config / flags / defaults

- I-5 filter itself has **no** `ELAI_*_ENABLE`. Production reach is gated by the **caller**: `ELAI_SEARCH_LADDER_ENABLE` (I-4) and `ELAI_ESCALATION_WATERFALL_ENABLE` (I-7) (`pristine_path_filter.rs:13-15`, `ssb/payloads.rs:17916-17922`).
- VividnessProbe emit: `ELAI_REASONING_QUALITY_PROBE_ENABLE` via `umbrella_enabled()` (`reasoning/mod.rs:30,35`, `ssb/payloads.rs:17159-17161`).
- J-95 observe-only: `ELAI_J95_PRISTINE_ON_BUDGET=1` default-OFF; **not** the I-5 organ (`context_budget.rs:1407-1414`).
- Frontend: vividness-tax tile in ReasoningDrawer — **wire-dark** until GD-4 bound; W-6 justified for v1 (`I-plan/I-5_pristine_context_injector.md:47-49,81-83`).
- Decisions doc required: `workflow/reasoning/I-5-decisions.md` (`I-plan/I-5_pristine_context_injector.md:46`).
- `lib.rs:237-241` still says “Wire-dark until I-4 SearchLadder ships”; module header says **WIRED (I-4)**. Treat `lib.rs` comment as **stale** against `pristine_path_filter.rs:13-15,100-102`.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/pristine_path_filter.rs` | new module (`I-plan/I-5_pristine_context_injector.md:53,61-63`) |
| `PristinePathFilter::build_branch_context` | produces_symbols (`:62`) |
| `search/ladder.rs` `SearchLadder::build_pristine_context` / `run_best_of_n` | PRODUCTION caller (`:70-71`, `ladder.rs:315-340`) |
| `context_budget.rs` `ContextSlot` / `update_breakpoints` | IS-6 (`context_budget.rs:1335,1850`) |
| `context_policy::regime_policy` | H-50 (`pristine_path_filter.rs:112`) |
| `reasoning::umbrella_enabled` | E-19 gate (`pristine_path_filter.rs:172`) |
| `ssb.rs` + `action_type_lock.rs` `ContextPruned` | SCHEMA 247 (`action_type_lock.rs:725-727`) |
| `crates/contracts/tests/i5_pristine_context_contract.rs` | Tier-1 contract |
| `workflow/reasoning/I-5-decisions.md` | CREATE (`I-plan/I-5_pristine_context_injector.md:46`) |
| `context_budget.rs::observe_pristine_slot_candidates` | J-95 cousin, not this organ |

## 11. Journal / SSB events

**`ContextPruned`** Pattern-2, action_type string `"context_pruned"` (`action_type.rs:4310-4319,6687`, `ssb/payloads.rs:17151-17198`):

```
ContextPrunedPayload {
  kind: SiblingExcluded | TracebackContrasted | VividnessProbe,
  branch_id, session_id,
  excluded_bytes, original_traceback_bytes,
  constraint_phrase, outcome_delta,
}
```

E-19 companion (schema extend, not a second primary row): `ProbeKind::VividnessProbe` (`ssb/payloads.rs:11848-11852`). I-5 **must not** depend on a new action_type beyond `ContextPruned` (`I-plan/I-5_pristine_context_injector.md:63`).

Plan baseline +1 from 221/532 (2026-06-10) is a **snapshot**; live SCHEMA_VERSION 247. Rebuild **emits these payload shapes**; do not rewind a live schema number (`I-plan/I-5_pristine_context_injector.md:105-106`).

## 12. Tests and observables

Named contract `i5_pristine_context_contract.rs` (`I-plan/I-5_pristine_context_injector.md:84-92`, live tests):

| Id | Assert |
|---|---|
| branch-1 | 5 KB traceback → `original_traceback_bytes` above threshold; `constraint_phrase` present; word count `<= 30`; SSB `ContextPruned(TracebackContrasted)` row in the **database** (R2) (`i5_pristine_context_contract.rs:65-133`) |
| branch-2 | no siblings, no traceback → `original_traceback_bytes == 0`, no phrase, `excluded_bytes == 0` (`:136-167`) |
| constant | `MAX_CONSTRAINT_TOKENS == 30` (`:170-179`) |
| unit | `build_branch_context_traceback_compressed` word-count cap (`pristine_path_filter.rs:322-354`) |

**Plan observable (HARD input):** `elai run --task fixtures/search/retry_branch.toml --search-ladder` (after I-4 v1). Retry branch outbound payload contains **ZERO bytes** of the prior 5K-line traceback while the `<=30`-token phrase is present. `--carry-tracebacks` broken arm re-inflates to `>=5K` tokens and **FAILS** (R2/ID-1) (`I-plan/I-5_pristine_context_injector.md:84-92`). Sampled expansion: `VividnessProbe` row with measured `outcome_delta` recomputable under `ELAI_SEED`.

**DETERMINISTIC_CHECK (from stub_ban):**

- `rg` of raw-traceback carry into the next branch’s context **MUST be 0** on the production rebuild path.
- No new masking layer that duplicates H-6 (`I-plan/I-5_pristine_context_injector.md:76-77`).
- No `.unwrap()`/`.expect()` on prod paths; blake3 only (`:80`).
- A contract test is **not** a production caller (R7) (`i5_pristine_context_contract.rs:3-7`).

Adversarial: truncating accumulated context instead of rebuilding; claiming a vividness-tax % from literature (ID-5).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- On branch expand, context is **rebuilt from the parent chain**. Failed-sibling payloads and raw tracebacks are **HARD-EXCLUDED**.
- The only permitted negative signal is a **`<=30`-token** structured contrastive constraint. Not a summary of the dump. Not a truncated dump.
- `ContextPruned` rows: `SiblingExcluded` / `TracebackContrasted` / `VividnessProbe`.
- A/B vividness tax is **measured** via E-19 (or the named equivalent) and ledgered under a seed **before** any improvement claim (ID-5).
- Blake3 prefix breakpoints are re-anchored after rebuild (HD-2). Regime-aware window via H-50 compose.
- Production caller is the search-ladder per-branch expand (I-4), not a unit test.
- H-6 masking is **extended or composed**, not forked.
- The `--carry-tracebacks` (or equivalent) broken arm **fails**.

A “please ignore previous errors” prompt, a shrunken window that still contains the traceback, or I-11 `failed_because` **without** per-branch exclusion is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Carrying raw tracebacks so “the model can see what failed” (`I-plan/I-5_pristine_context_injector.md:18-19,74-75`).
- Truncating context rather than rebuilding it (`:19-20`).
- A new masking layer that conflicts with / duplicates H-6 (`:20-21,76-77`).
- Claiming a numeric vividness-tax improvement before the A/B measurement (ID-5) (`:21-22,78-79`).
- Contrastive-constraint auto-tuning per task class — Tier-2 deferred (`:50`).
- Dashboard tile before `>=3` measured A/B runs (`:47-49`).
- Re-authoring E-19, H-50, I-4, I-11, or J-83 (GD-1).
- SHA-2 / blake2; `.unwrap()` on prod (`:80`).
- Using I-11 condensate **as** the per-branch prompt filter (wrong grain).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-5_pristine_context_injector.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:110-124`. Feature map: `ELAI-FEATURE-MAP.md:840-846`. Context: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:46,193-195,302,346,454`. Live: `crates/orchestrator/src/pristine_path_filter.rs`, `search/ladder.rs:315-340`, `context_budget.rs:1335-1362,1406-1438,1850`, `ssb/payloads.rs:11848-11852,17151-17198`, `lib.rs:237-241`, `contracts/tests/i5_pristine_context_contract.rs`. Neighbors: `reasoning-architecture/I-04-evidence-ranked-search-ladder.md`, `I-11-typed-belief-state.md`, `J-83-grcv3-beliefstate-carrier.md`, `E-19-reasoning-quality-hooks.md`.

arXiv / review **[EXTERNAL, WHY not an ELAI number]:** Consolidated Part II review 1.4-4 vividness/attention-sink (`I-plan/I-5_pristine_context_injector.md:11-12,103-104`). WebSearch-confirm before headline if cited in the decision doc.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Whether live I-4 v1 (`sibling_payloads=&[]`, `traceback_bytes=None`, `ladder.rs:330-332`) satisfies ID-1 or only the telemetry call. Plan requires the caller to pass failed-sibling payloads + traceback.
- Whether `build_branch_context` must **mutate** `ContextSlot::SsbEntries` (plan/docstring `pristine_path_filter.rs:86-90`) vs the live empty-placeholder `update_breakpoints` (`:184-199`).
- Tokenizer vs whitespace word-split for the 30-token cap (`pristine_path_filter.rs:144-156` vs plan “tokens”).
- Sampled **fraction** of expansions for VividnessProbe (plan: sampled fraction; live: every call when umbrella is on).
- Real `outcome_delta` wiring (live `0.0` placeholder, `pristine_path_filter.rs:253`).
- H-6 “extend turn-drop” (plan) vs “orthogonal adaptive_reduce” (live header) — which H-6 symbol is the shared engine.
- `--carry-tracebacks` CLI / `fixtures/search/retry_branch.toml` existence (named in the plan observable; not verified here).
- Whether `excluded_bytes` must be serialized sibling **payload** size (plan) vs `action_type().as_sql_str().len()` proxy (live, `pristine_path_filter.rs:121-124`).
- Frontend tile vs justified N/A timing (W-4 / W-6).
