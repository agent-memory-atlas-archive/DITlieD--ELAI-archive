# Aletheia voluntary abort (I-8)

- **ELAI ids:** I-8 / RD-1 / ID-1 / IS-14 / IS-7 / WAVE 1 / Track 13 EVIDENCE-RANKED SEARCH
- **Spec status (as designed):** PARTIAL. FEATURE-MAP still labels **PLANNED(I-8)** (`ELAI-FEATURE-MAP.md:861-862`) — **stale** against live `acknowledge_infeasible` catalog+dispatch, `dead_end_cache.rs`, `TrajectoryAborted` SSB, `i8_voluntary_abort_contract.rs`. Original plan WAVE 1 BUILD (`I-plan/I-8_aletheia_voluntary_abort.md:1-3`, `I-plan/I-SUMMARY_INDEX.md:156-170`).
- **Crate / module (spec):** `crates/orchestrator/src/dead_end_cache.rs` (`DeadEndCache::check_and_insert`); `production_agent_rpc.rs` `TOOL_ACKNOWLEDGE_INFEASIBLE` catalog + dispatch; `crates/ipc-types` `AcknowledgeInfeasibleRequest`. SSB: `TrajectoryAborted` / `TrajectoryAbortedKind`. Credits H-39 `AbstentionScored` (compose, do not re-score).
- **Purpose of this file:** enough architecture to rebuild credited voluntary abort, dead-end hashing, and the 15 s check-runner halt. Not a status ledger.

## 1. What it is

**Aletheia voluntary abort** is the structural mechanism that lets a trajectory **declare infeasible** instead of spiraling until the window fills. Three parts compose shipped substrate (`I-plan/I-8_aletheia_voluntary_abort.md:4-10,12-20`):

1. **`acknowledge_infeasible` / abort_trajectory tool** in the production catalog (IS-14 3-arm). Calling it is **credited** by H-39 `AbstentionScored` (RD-1). Orchestrator rolls back to the last good trunk node via H-49 when that seam is live.
2. **Dead-end hashing:** blake3 fingerprint of the aborted `(task, approach)` is stored in a bounded in-session LRU. A future branch whose approach hash matches is **pruned BEFORE any model call** — zero tokens on isomorphic re-attempts.
3. **Empirical halting bound:** a deterministic check spinning past **15 s** (named I-17 config key, default `15_000` ms) auto-aborts and emits `HaltingBound`.

GRC-v3 **overthinking guard (J-85)** is the neighbor, not this organ: after **~3 failed receipted retries** it signals `MODALITY_CHANGE` → retrieve / I-7 escalate / **I-8 abstain** — **never** “reason more” (`J-85-grcv3-overthinking-guard.md` §3 step 9, `J-plan/J-85_grcv3_overthinking_guard.md:19,43`). I-8 is the **abstain destination** and the **dead-end cache** that makes “try the same approach harder” a zero-cost prune.

## 2. Why it exists

A system that cannot declare a task infeasible will spiral on impossible problems until context or budget dies. H-39 already **credits** abstention; what was absent is the **verb**, the **isomorphic-re-attempt prune**, and the **check-runner watchdog** (`I-plan/I-8_aletheia_voluntary_abort.md:4-10`). If this organ is missing, D-16/D-17 can refuse a **claim** while the **trajectory** still retries the same AST. If it is faked (prompt “you may give up”, or re-scoring abstention inside I-8, or resurrecting an aborted hash in-session), RD-1 and ID-1 are dead: the loop is rewarded for confident guessing, and failed paths re-enter context (`:23-25,86-89`, `H-39-abstention-aware-scoring.md` §2).

Honest residue: I-8 does **not** invent a learned infeasibility classifier and does **not** replace the global context-budget timeout (`I-plan/I-8_aletheia_voluntary_abort.md:20-23`).

## 3. How it works (runtime)

1. **Who calls it (production, R7).**
   - **Voluntary:** model tool-call → `build_tool_catalog` lists `acknowledge_infeasible` → `translate_tool_request` / dispatcher `TOOL_ACKNOWLEDGE_INFEASIBLE` arm (`I-plan/I-8_aletheia_voluntary_abort.md:79-81`, `production_agent_rpc.rs:1475-1504,3304-3391`, `ipc-types/src/lib.rs:1072-1098`).
   - **Dead-end:** **before every branch expansion**, `DeadEndCache::check_and_insert` — prune on hit, **zero model calls** (`I-plan/I-8_aletheia_voluntary_abort.md:16-18,66,79-81`). Live: cache + contract exist; a `task_runner` production call site for `check_and_insert` was **not** grepped in `orchestrator/src` outside `dead_end_cache.rs` (PARTIAL; rebuild the **plan** pre-dispatch hook).
   - **HaltingBound:** I-17 check-runner timeout → `DeadEndCache::check_halting_bound` (`I-plan/I-8_aletheia_voluntary_abort.md:82-83`, `dead_end_cache.rs:17-19,200-254`). Wave-1: config path + spawn-when-writer; Wave-3 I-17 fills the timer into the check runner (`dead_end_cache.rs:52-57`).
2. **Input (voluntary).** `AcknowledgeInfeasibleRequest { reason: String, trajectory_id: Option<String> }` postcard in `ToolRequest.arguments`. `reason` is required (`ipc-types/src/lib.rs:1090-1098`, `production_agent_rpc.rs:1489-1502`). The model supplies **reason text only**. It does **not** write the H-39 bit, the hash, or a freeze flag.
3. **Voluntary dispatch (deterministic host).** Decode wire → emit `TrajectoryAborted(Voluntary)` → call `abstention_aware_score(OutcomeTriple::new(RefusalDecision::Abstain, false), DEFAULT_CONFIDENCE_TARGET)` → emit `AbstentionScored` with `suite: "voluntary_abort"`, `mean_score` from H-39, `abstention_rate: 1.0` → skip H-49 rollback when no pre-abort snapshot (honest log) → `Err(reason)` so the runner maps `ToolOutcome::Failed` (`production_agent_rpc.rs:3309-3391`). **Do not re-implement the scorer** (`I-plan/I-8_aletheia_voluntary_abort.md:86-87`).
4. **Calibration hygiene (deterministic, I-23).** `voluntary_abort` rows are **excluded** from the conformal calibration corpus (`CALIBRATION_EXCLUDED_SUITES`). They carry synthetic `mean_score=0.0 / abstention_rate=1.0` and would down-bias tau (`abstention_score.rs:300-328`). Fail-closed: unknown suites excluded unless on `CALIBRATION_ALLOWED_SUITES`.
5. **Dead-end hash (deterministic).** `fingerprint(task_bytes, approach_bytes)` = blake3 of length-prefixed concat `[4-byte task-len LE][task][approach]` via `timetravel::blake3_hex` — **never sha2/blake2** (`dead_end_cache.rs:13-15,141-153`). `check_and_insert` returns **true** on collision (caller **must prune**, no model dispatch) and **false** on first insert (now recorded). Capacity eviction: oldest `VecDeque` front (`:155-175`). In-session only in v1 (`I-plan/I-8_aletheia_voluntary_abort.md:54-55`).
6. **Halting bound (deterministic).** Named key; default `DEFAULT_HALTING_BOUND_MS_FALLBACK = 15_000` from I-17 spec, **not a paper import**, **not a magic literal at call sites** (`dead_end_cache.rs:46-58`, `I-plan/I-8_aletheia_voluntary_abort.md:18-19,90-91`). `check_halting_bound` spawns **only** when both `halting_bound_ms` and `ssb_writer` are set; else honest no-op (`dead_end_cache.rs:210-228`). On breach: `TrajectoryAborted(HaltingBound)` as agent `halting_bound_watchdog` (`:60-63,234-246`). Caller should `.abort()` the `JoinHandle` on clean completion (`:205-208`).
7. **Rollback (H-49 compose).** Plan: restore last good trunk node (`I-plan/I-8_aletheia_voluntary_abort.md:15-16,76`). Live Wave-1: `timetravel::materialize()` **skipped** with info log — “H-49 Wave-2 will wire SessionDag HEAD restore” (`production_agent_rpc.rs:3314-3316,3382-3388`). Rebuild the **plan** rollback; do not treat the skip as the organ.
8. **J-85 modality-change (neighbor, cite).** Guard counts **failed receipted retries**. At **~3**, signal `MODALITY_CHANGE` to J-84/J-86. Acting options: retrieve / I-7 escalate / **I-8 `acknowledge_infeasible`**. **Banned:** more serial reasoning on the same modality (`J-85-grcv3-overthinking-guard.md` §3 step 9). I-8 does **not** own the counter.
9. **Fail-closed.** Same-session resurrection of a `DeadEndHash` is banned (`I-plan/I-8_aletheia_voluntary_abort.md:23-25,88-89`). SSB write on Voluntary is best-effort warn on failure (`production_agent_rpc.rs:3342-3349`) — contract still requires the row as R4 evidence (`i8_voluntary_abort_contract.rs:17`). Missing catalog entry would silently drop the tool to unknown fallback — IS-14 constant lives in `ipc-types`, not a match-arm literal (`ipc-types/src/lib.rs:1079-1082`).

The model may **call** `acknowledge_infeasible` with a reason. It may **not** credit itself, clear the dead-end cache, raise the 15 s bound, or demand “try harder” on a hashed dead-end.

## 4. Types, enums, objects

Copy from the I-8 wiring contract plus live structs that name this organ. Do not invent fields.

```
TOOL_ACKNOWLEDGE_INFEASIBLE: &str = "acknowledge_infeasible"

AcknowledgeInfeasibleRequest {
  reason: String,                      // required
  trajectory_id: Option<String>,       // serde default
}

DEFAULT_DEAD_END_CACHE_CAPACITY: usize = 256
DEFAULT_HALTING_BOUND_MS_FALLBACK: u64 = 15_000
HALTING_BOUND_AGENT_ID: &str = "halting_bound_watchdog"

DeadEndCache {
  capacity, queue: VecDeque<String>, set: HashSet<String>,
  halting_bound_ms: Option<u64>,
  ssb_writer: Option<Arc<Ssb>>,
}
fn fingerprint(task_bytes, approach_bytes) -> String          // blake3 hex
fn check_and_insert(approach_hash) -> bool                    // true = prune
fn check_halting_bound() -> Option<JoinHandle<()>>
fn new / new_with_halting_bound / with_ssb_writer

TrajectoryAbortedKind { Voluntary, DeadEndHash, HaltingBound }

TrajectoryAbortedPayload {
  kind: TrajectoryAbortedKind,
  session_id: String,                  // serde default
  trajectory_id: Option<String>,
  approach_hash: Option<String>,       // Some for DeadEndHash; None Voluntary/HaltingBound
  abort_reason_text: Option<String>,   // Some for Voluntary; live HaltingBound also fills
  ts_ms: i64,
}

// H-39 READ/INVOKE, never re-derived (H-39-abstention-aware-scoring.md §4)
OutcomeTriple::new(RefusalDecision::Abstain, false)
fn abstention_aware_score(outcome, t) -> f64
DEFAULT_CONFIDENCE_TARGET t = 0.75 LOCKED
CALIBRATION_EXCLUDED_SUITES = ["voluntary_abort"]
```

(`ipc-types/src/lib.rs:1083-1098`, `dead_end_cache.rs:40-97`, `ssb/payloads.rs:16738-16784`, `production_agent_rpc.rs:3352-3369`, `abstention_score.rs:315`)

**Live docstring vs payload:** `approach_hash` / `abort_reason_text` comments say Voluntary/HaltingBound leave hash None and DeadEndHash leaves reason None (`ssb/payloads.rs:16775-16780`); live HaltingBound **does** set `abort_reason_text` (`dead_end_cache.rs:239-241`). Rebuild: allow reason on HaltingBound; do not invent extra fields.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator tool dispatcher + dead-end cache + I-17 watchdog. The model may request abort; the host journals, scores, prunes, rolls back.

```
branch expand
     │
     ├─ DeadEndCache::check_and_insert HIT
     │     emit TrajectoryAborted(DeadEndHash)
     │     ZERO model calls
     │     STOP (no resurrect in-session)
     │
     ├─ I-17 check spinning
     │     check_halting_bound timer
     │     on breach → TrajectoryAborted(HaltingBound) + abort_trajectory
     │
     └─ model calls acknowledge_infeasible
           TrajectoryAborted(Voluntary)
           H-39 AbstentionScored (suite=voluntary_abort)
           H-49 rollback when snapshot exists
           ToolOutcome::Failed(reason)

J-85 (neighbor, parallel):
  ~3 failed RECEIPTED retries → MODALITY_CHANGE
       retrieve | I-7 escalate | I-8 abstain
       NEVER more-reasoning
```

**Forced transitions:**

- Dead-end hit **cannot** dispatch the model (`I-plan/I-8_aletheia_voluntary_abort.md:16-18,88-89`).
- Voluntary abort **must** credit H-39; I-8 must not duplicate the formula (`:86-87`).
- 15 s bound is a **named config**, default from I-17 (`:90-91`).
- C-10 `StagnationPattern` is **repetition** (`C-10-anti-spiral-lateral-thinking.md` §1). J-85 is **drift**. I-8 is **infeasibility / isomorphic prune**. Do not merge the three (`J-85-grcv3-overthinking-guard.md` §3 step 7).

## 6. Grounding, receipts, gates

Abstention **credit** is H-39 over `RefusalDecision::Abstain` — a **receipt-shaped outcome**, not “the model said IDK” (`H-39-abstention-aware-scoring.md` §3 steps 3–4, `production_agent_rpc.rs:3352-3358`). Dead-end prune is a **blake3 cache hit**, not an LLM similarity judge.

| Rank | Signal | Legal here? |
|---|---|---|
| 1 | Tool-call + `TrajectoryAborted` row + H-39 `AbstentionScored` | **voluntary abort** |
| 2 | blake3 fingerprint collision | **DeadEndHash prune** |
| 3 | I-17 watchdog elapsed | **HaltingBound** |
| 4 | LLM “I should keep thinking” / self-scored abstain | **BANNED** (RD-1, TD-3) |

**Never allowed as a confidence signal:** resurrecting a hashed dead-end “in case it works now”; a learned infeasibility classifier in v1 (`I-plan/I-8_aletheia_voluntary_abort.md:20-21`); hard whole-task timeout as a substitute (`:21-23`); including `voluntary_abort` suite in tau calibration (`abstention_score.rs:300-306`).

**D-16/D-17:** I-8 **extends** the abstention substrate with the tool; it does not re-implement refusal scoring or SLA (`I-plan/I-8_aletheia_voluntary_abort.md:122-124`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| LRU capacity | `256` entries (~16 KiB of 64-byte hex) | per task-runner instance (`dead_end_cache.rs:40-44`) |
| halting bound default | `15_000` ms | per deterministic check (`dead_end_cache.rs:58`) |
| J-85 neighbor | **~3** failed **receipted** retries → modality change | per GRC call (`J-85-grcv3-overthinking-guard.md` §3 step 9) |
| H-39 `t` | `0.75` LOCKED | scoring (`H-39-abstention-aware-scoring.md` §4) |
| Plan SSB | +`TrajectoryAborted` from LIVE **221/532** as of 2026-06-10; re-read W-9 | `I-plan/I-8_aletheia_voluntary_abort.md:119-120` |
| Contract floor | `TOTAL_VARIANTS >= 549` (lock comment) | `i8_voluntary_abort_contract.rs:177-183` |
| persistence | in-session v1; cross-session store **DEFER** until `>=3` sessions show repeats | `I-plan/I-8_aletheia_voluntary_abort.md:54-55` |

**Overflow:** at capacity, oldest hash evicted; a previously-pruned approach can re-enter after eviction (`dead_end_cache.rs:166-174,288-291`). That is LRU behavior, **not** in-session resurrection of a still-cached hash.

**Do not confuse with:** Phase-2A `GrcBudget`; I-7 `CONDENSED_PAYLOAD_MAX_BYTES`; C-10 spin_threshold=3 (consecutive **error class**, not receipted retries).

## 8. Roles and neighbors

Any role that can call tools can call `acknowledge_infeasible`. Scoring is **host-side**.

**Consumes:** IS-14 `build_tool_catalog`; D-16/D-17 refusal schema; H-39 `abstention_aware_score` / `AbstentionScored`; H-49 rollback (when live); I-17 check-runner watchdog hook; E-35 tool schema (`I-plan/I-8_aletheia_voluntary_abort.md:72-77,122-130`).

**Produces:** `TrajectoryAborted` rows; H-39 credit row (`suite=voluntary_abort`); prune-before-dispatch; (plan) trunk restore.

**Cite, do not re-author:**

- **H-39** — the scorer (`H-39-abstention-aware-scoring.md`).
- **J-85** — overthinking guard; I-8 is the abstain **sink** for `MODALITY_CHANGE` (`J-85-grcv3-overthinking-guard.md`).
- **J-83** — frozen receipt-confirmed entries; abort does not unfreeze by rewriting the chain (`J-83-grcv3-beliefstate-carrier.md` §3 step 7).
- **I-7** — the **other** modality (escalate) when local retries fail (`I-07-cascading-escalation.md`).
- **I-5** — aborted paths stay **out of the next prompt** (ID-1); dead-end cache is the mechanical prune (`I-05-pristine-context-injector.md`).
- **I-11** — escalate condensate is not the abort verb.
- **C-10** — repetition; distinct (`C-10-anti-spiral-lateral-thinking.md` §1).
- **D-13** — Mode B adversary; not an abort classifier.

## 9. Config / flags / defaults

- No `ELAI_ALETHEIA_ENABLE` in the plan. Tool is on the production catalog (`production_agent_rpc.rs:1475-1504`).
- Halting bound: `TaskRunnerConfig::halting_bound_ms` named in the TODO; fallback constant `DEFAULT_HALTING_BOUND_MS_FALLBACK` (`dead_end_cache.rs:52-57`). Plan forbids an unnamed `15000` literal (`I-plan/I-8_aletheia_voluntary_abort.md:90-91`).
- Watchdog dark unless `new_with_halting_bound` **and** `with_ssb_writer` (`dead_end_cache.rs:115-118,210-214`).
- Frontend abort/dead-end strip in ReasoningDrawer — wire-dark GD-4; W-6 justified v1 (`I-plan/I-8_aletheia_voluntary_abort.md:56,94-96`).
- Decisions doc: `workflow/reasoning/I-8-decisions.md` (`:52`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/dead_end_cache.rs` | produces `DeadEndCache::check_and_insert` (`I-plan/I-8_aletheia_voluntary_abort.md:45,69`) |
| `production_agent_rpc.rs` catalog + dispatch | IS-14 3-arm (`:70,1475-1504,3322-3391`) |
| `ipc-types` `TOOL_ACKNOWLEDGE_INFEASIBLE` / `AcknowledgeInfeasibleRequest` | wire (`ipc-types/src/lib.rs:1083-1098`) |
| `abstention_score.rs` `abstention_aware_score` | H-39 invoke (`production_agent_rpc.rs:3352-3358`) |
| `ssb` `TrajectoryAborted` | Pattern-2 3-arm (`ssb/payloads.rs:16738-16784`) |
| `architecture::timetravel::blake3_hex` | hash (`dead_end_cache.rs:33,152`) |
| `crates/contracts/tests/i8_voluntary_abort_contract.rs` | contract |
| H-49 `SessionDag` rollback | Wave-2; live skip log (`production_agent_rpc.rs:3382-3388`) |
| I-17 check runner | Wave-3 watchdog home (`dead_end_cache.rs:17-19`) |

Plan pin `production_agent_rpc.rs build_tool_catalog :657` is **stale** (`I-plan/I-8_aletheia_voluntary_abort.md:28`); live catalog entry is `:1481`. Rebuild to **symbols**.

## 11. Journal / SSB events

**`TrajectoryAborted`** Pattern-2 (`ssb/payloads.rs:16738-16784`):

```
TrajectoryAbortedPayload {
  kind: Voluntary | DeadEndHash | HaltingBound,
  session_id, trajectory_id, approach_hash, abort_reason_text, ts_ms,
}
```

**H-39 companion (mandatory on Voluntary):** `AbstentionScored` with `suite="voluntary_abort"` (`production_agent_rpc.rs:3359-3369`). Do **not** ingest that suite into I-23 calibration (`abstention_score.rs:300-315`).

Net +1 action_type from the I-8 plan (`I-plan/I-8_aletheia_voluntary_abort.md:49,119-120`). Snapshot 221/532 is 2026-06-10; live lock comments I-8 around SCHEMA 247-era + later bumps. Rebuild **shapes**, not a frozen schema integer.

## 12. Tests and observables

Named contract `i8_voluntary_abort_contract.rs` (`I-plan/I-8_aletheia_voluntary_abort.md:97-107`, live header):

| Id | Assert |
|---|---|
| Voluntary | `acknowledge_infeasible` → `Err` (`ToolOutcome::Failed`); SSB `TrajectoryAborted(Voluntary)`; `abort_reason_text` from request (`i8_voluntary_abort_contract.rs:17,359-396`) |
| DeadEndHash | second isomorphic approach → `TrajectoryAborted(DeadEndHash)` **instead of** a model dispatch; `fingerprint` stable; `check_and_insert` true on duplicate (`:10-11,245-283`) |
| HaltingBound | variant reachable; named `DEFAULT_HALTING_BOUND_MS_FALLBACK`; Wave-1 honest stub (`:14,294-326`) |
| lock | `TrajectoryAborted` in `SSB_ACTION_TYPES`; `TOTAL_VARIANTS >= 549` (`:177-190`) |

**Plan CLI observable:** `fixtures/abort/impossible_task.toml` → Voluntary + trunk rollback journal-verified + `AbstentionScored abort_reason=Voluntary`; second isomorphic run → DeadEndHash and **journal token count = 0**; `spin_task.toml --halting-bound 15s` → HaltingBound within budget. Broken arm without the cache spirals past 3 iterations and fails the spiral-detector (R2) (`I-plan/I-8_aletheia_voluntary_abort.md:97-107`).

Adversarial: re-scoring inside I-8; sha2 fingerprint; resurrecting a live cache hit; treating C-10 oscillation as I-8 abort.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- An explicit **acknowledge_infeasible** (or equivalent) **tool** on the production catalog, IS-14 3-arm, not a prompt suggestion.
- Calling it emits a **journaled abort** and **credits H-39 abstention** (compose the scorer; do not copy the formula).
- **Dead-end blake3** cache: isomorphic re-attempt pruned **before** the next model call; in-session prune is final while the hash is cached.
- **Named** check-runner halt (~15 s from I-17 spec), configurable, default sourced — not a magic literal.
- Neighbor composition with J-85: **~3 failed receipted retries → modality change**, not more reasoning; I-8 is a legal **abstain** sink.
- Production callers on the tool-dispatch path **and** (when I-17 is live) the check-runner path. A unit test is not enough (R7).
- No learned infeasibility classifier as v1.

A “you may stop if stuck” system prompt, a global wall-clock on the whole task, or C-10 persona rotation **without** the tool+hash is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-scoring abstention / duplicating H-39 (`I-plan/I-8_aletheia_voluntary_abort.md:86-87`).
- Resurrecting an aborted trajectory or overriding a DeadEndHash prune in-session (`:23-25,88-89`).
- Hard-coding 15 s without a named config key (`:90-91`).
- blake2/sha2 dead-end hash (`:92`).
- Learned infeasibility classifier in v1 (`:20-21`).
- Whole-task hard timeout as this organ (`:21-23`).
- Cross-session durable store in v1 (`:54-55`).
- Merging with C-10 anti-spiral or J-85 overthinking guard (`J-85-grcv3-overthinking-guard.md` §3 step 7).
- “Reason more” after ~3 failed receipted retries (`J-85-grcv3-overthinking-guard.md` §3 step 9).
- `.unwrap()` on prod (`I-plan/I-8_aletheia_voluntary_abort.md:93`) — live watchdog `ts_ms` uses `unwrap_or(i64::MAX)` (`dead_end_cache.rs:242`).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-8_aletheia_voluntary_abort.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:156-170`. Feature map: `ELAI-FEATURE-MAP.md:861-862`. Live: `dead_end_cache.rs`, `production_agent_rpc.rs:1475-1504,3304-3391`, `ipc-types/src/lib.rs:1072-1098`, `ssb/payloads.rs:16738-16784`, `abstention_score.rs:300-328`, `contracts/tests/i8_voluntary_abort_contract.rs`. Neighbors: `H-39-abstention-aware-scoring.md`, `J-85-grcv3-overthinking-guard.md`, `J-83-grcv3-beliefstate-carrier.md`, `I-07-cascading-escalation.md`, `I-05-pristine-context-injector.md`, `C-10-anti-spiral-lateral-thinking.md`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Production `task_runner` / search-ladder call site for `DeadEndCache::check_and_insert` (plan reachable_from; live grep confined to `dead_end_cache.rs` + contracts).
- H-49 trunk-restore wiring (Wave-2; live honest skip) (`production_agent_rpc.rs:3382-3388`).
- I-17 spawning the watchdog at the **check-runner** vs only exposing `check_halting_bound` (`dead_end_cache.rs:17-19,52-57`).
- Whether DeadEndHash path also emits `AbstentionScored` (plan “abort handler emits … abort_reason=Voluntary/DeadEndHash/HaltingBound”; live Voluntary emit is explicit; DeadEndHash/HaltingBound scoring **not** shown in `dead_end_cache.rs`).
- Plan `AbortScored.abort_reason` field vs live `AbstentionScoredPayload` having `suite` not `abort_reason` (`production_agent_rpc.rs:3359-3368` vs `I-plan/I-8_aletheia_voluntary_abort.md:101-102`).
- Exact `TaskRunnerConfig::halting_bound_ms` field existence (named in TODO).
- Fixtures `impossible_task.toml` / `spin_task.toml` existence.
- Whether J-85’s “~3” is a LOCKED constant or K-15-measured (J-85 organ; do not freeze it here).
