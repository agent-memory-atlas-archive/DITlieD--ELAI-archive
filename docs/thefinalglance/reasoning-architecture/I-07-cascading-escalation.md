# Cascading escalation + offline batch (I-7)

- **ELAI ids:** I-7 / IC-1 / ID-3 / IS-18 / WAVE 4 / SCHEMA 262 / Track 13 EVIDENCE-RANKED SEARCH
- **Spec status (as designed):** DARK(`ELAI_ESCALATION_WATERFALL_ENABLE`) live; FEATURE-MAP row **WIRED** (`ELAI-FEATURE-MAP.md:2351-2354`) **and** letter-ledger **DARK(gated)** (`:3437`) — treat as **gated-wired**: module + `drive_task` hook exist, default-OFF, GD-4 measured-first-win (`ssb/payloads.rs:17916-17922`, `I-plan/I-7_cascading_escalation_offline_batch.md:1-3`). Original plan WAVE 4 BUILD, depends I-11.
- **Crate / module (spec):** `crates/orchestrator/src/escalation_waterfall.rs` (`EscalationWaterfall::run`); production: `task_runner::maybe_run_escalation_waterfall`. Composes I-4 `SearchLadder`, I-11 `BeliefState`, H-7 `CloudModeRouter`. SSB: `EscalationCascade` / `EscalationCascadeKind`.
- **Purpose of this file:** enough architecture to rebuild the local-first waterfall that condenses local failure into BeliefState then escalates. Not a status ledger.

## 1. What it is

The **cascading escalation waterfall** is a **strict local-first dispatcher**: fan out K cheap branches on the **local** lane, run the I-17 tri-layer check on each, and **only when every local branch fails** does the orchestrator **CONDENSE** the failure set into an I-11 `BeliefState` and escalate **SYNCHRONOUSLY** to the cloud architect. The LLM does **not** pick the backend. The Anthropic/OpenAI **Batch API is confined to H-5 offline lanes** (Cosmos, research, bench) and is **structurally unreachable** from this module (IC-1) (`I-plan/I-7_cascading_escalation_offline_batch.md:4-11,13-25`, `escalation_waterfall.rs:1-18`, `I-plan/I-SUMMARY_INDEX.md:141-154`).

GRC-v3 (J-83) **threads** this organ as the I-7 consumer of the typed carrier: escalate hands **CLAIMS+RECEIPTS**, never raw history (`J-83-grcv3-beliefstate-carrier.md` §3 steps 1, 8; `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:56-57`). I-11 already ships `BeliefState::new` / `emit_condensed_receipt`; this organ **calls** them (`I-11-typed-belief-state.md` §3, `escalation_waterfall.rs:25-26,197-207`).

## 2. Why it exists

The original I-7 design (Batch API speculative racing vs parallel local branches) is **architecturally impossible** for live loops. Batch lanes run **10–45 min** with a **24 h SLA**; cancels **do not refund** queued work. That design is **DEAD** and must not be resurrected (IC-1, carried landmine #5) (`I-plan/I-7_cascading_escalation_offline_batch.md:4-6,14,78-80,103-107`). If this organ is missing, a task the local lane could solve still leaves the machine, or a live search loop submits un-refundable batch jobs. If it is faked (prompt “try the cloud”, or the model chooses Local vs Cloud vs Batch, or escalate ships **raw history**), ID-3 and E-39 are dead: replay cannot reconstruct the cloud payload, and cost is unbounded (`I-plan/I-7_cascading_escalation_offline_batch.md:20-25,84-85`).

A LocalWin **never** spends cloud (E-39 compose). A local-all-fail escalates **once**, with a typed condensate `<= ~2K` tokens, not a `>=20K` transcript dump (`I-plan/I-7_cascading_escalation_offline_batch.md:16-18,91-97`, `I-11-typed-belief-state.md` §2).

## 3. How it works (runtime)

1. **Who calls it.** `task_runner::drive_task` → `maybe_run_escalation_waterfall` (after `dispatch_search_ladder`). Two gates, both required: (a) `ELAI_ESCALATION_WATERFALL_ENABLE` truthy (`1|true|yes|on`), (b) D-11 `checker_runtime` plumbed. Else the hook is a **strict no-op**; linear retry stays (GD-4) (`task_runner.rs:14225-14280`, `escalation_waterfall.rs:338-350`, `I-plan/I-7_cascading_escalation_offline_batch.md:74-76`). The model does **not** call `EscalationWaterfall::run`.
2. **Input.** `candidates: &[BranchInput]`, `session_id`, structured `failed_because: Vec<String>` ( **no** raw traceback — I-5 doctrine on the escalate wire), optional `CloudCredentials` (`escalation_waterfall.rs:166-177`). Isolation: I-2 workspaces via the I-4 ladder (`I-plan/I-7_cascading_escalation_offline_batch.md:72`).
3. **Local fan-out (deterministic host, I-4 compose).** `SearchLadder::run_best_of_n(..., BackendMode::Local)` — per-branch I-5 pristine rebuild + I-17 tri-layer + veto/quarantine prune + receipt rank (`escalation_waterfall.rs:180-184`, `I-04-evidence-ranked-search-ladder.md` §3). I-7 **does not** re-implement the ladder (`escalation_waterfall.rs:20-24`).
4. **LocalWin (deterministic).** If `ladder_outcome.winner.is_some()`: emit `EscalationCascade(LocalWin)` best-effort; return `WaterfallOutcome { local_win: true, routed: None, condensed_bytes: 0 }`. Cloud arm **never runs** (E-39) (`escalation_waterfall.rs:186-195`).
5. **All-local-fail → condense (deterministic, ID-3).** `winner == None` (BudgetCollapsed / every candidate pruned) **is** the escalate signal — not an LLM judgment (`escalation_waterfall.rs:23-24,197-209`). `condense`:
   - `tried` = branch ids as `TriedApproach { label, token_cost: 0 }`
   - `failed_because` + `constraints` = the structured reason vec
   - non-empty reasons → `CriticalAtom { kind: BugLocation, must_preserve: true }` (K-12; LLM never sets)
   - `quarantined_claims` from session `StalenessQuarantineBroadcast` (`claim:` prefix)
   - `current_ast_state` / `dkg_entities` empty in v1
   - `enforce_belief_handoff_faithfulness` — missing must-preserve atoms → `Faithfulness` error
   (`escalation_waterfall.rs:227-279`, `J-83-grcv3-beliefstate-carrier.md` §5)
6. **Encode + receipt.** `belief.encode()`; `condensed_bytes = encoded.len().min(CONDENSED_PAYLOAD_MAX_BYTES)`; `emit_condensed_receipt` (**un-journaled condensation = FAIL**, I-11 ID-3); emit `EscalationCascade(Condensed)` best-effort (`escalation_waterfall.rs:198-209`, `belief_state.rs` emit path cited in `J-83-grcv3-beliefstate-carrier.md` §3 step 8).
7. **Synchronous cloud route (deterministic decision, not billed call).** `CloudModeRouter::route(ExecMode::Cloud, credentials)` — H-7 seam. The **billed** cloud-architect inference that consumes the condensate is **D-026 PENDING** and is **not** made here. Emit `EscalationCascade(Escalated)` (write **is** propagated — off-agent evidence) (`escalation_waterfall.rs:31-38,211-224,283-287`). The LLM does **not** choose Local / Cloud / Batch.
8. **Offline BatchRouted (H-5 only).** `EscalationCascadeKind::BatchRouted` exists for **journal completeness on the offline scheduler**. The live waterfall **never emits it** and has **zero import path** to `batch_client` / `MessageBatch` / `anthropic_batch` (`escalation_waterfall.rs:13-18`, `ssb/payloads.rs:17875-17879`, `I-plan/I-7_cascading_escalation_offline_batch.md:78-83`).
9. **Abstain / fail-closed.** Ladder / condense / route / faithfulness / Escalated SSB errors abort the waterfall; caller keeps linear retry (`escalation_waterfall.rs:102-121`). Raw history on the cloud payload is stub_ban (`I-plan/I-7_cascading_escalation_offline_batch.md:84-85`). `BatchRouted` from this module is a **compile-time / grep structural impossibility**, not a runtime convention (`:98-101`).

The model may **generate** local candidate source. It may **not** pick the backend, submit batch on a live loop, hand raw transcript to cloud, or skip condensation.

## 4. Types, enums, objects

Copy from the I-7 wiring contract plus live DTOs that name this organ. Do not invent fields.

```
CONDENSED_PAYLOAD_MAX_BYTES: usize = 8_192   // [DESIGN] ~2K tokens × ~4 B/token
ELAI_ESCALATION_WATERFALL_ENABLE_ENV = "ELAI_ESCALATION_WATERFALL_ENABLE"
ESCALATION_WATERFALL_AGENT_ID = "escalation_waterfall"

EscalationWaterfall { ladder: Arc<SearchLadder>, router: CloudModeRouter, ssb: Arc<Ssb> }
fn run(candidates, session_id, failed_because, credentials) -> Result<WaterfallOutcome, EscalationWaterfallError>
fn condense(candidates, failed_because) -> Result<BeliefState, EscalationWaterfallError>
fn escalation_waterfall_enabled() -> bool
fn default_escalation_router() -> CloudModeRouter

WaterfallOutcome {
  local_win: bool,
  pruned_count: usize,
  routed: Option<RoutedProvider>,   // Some only on all-fail; DECISION not billed response
  condensed_bytes: usize,           // 0 on LocalWin; <= CONDENSED_PAYLOAD_MAX_BYTES
}

EscalationWaterfallError {
  Ladder(SearchLadderError),
  Condense(BeliefStateError),
  Route(CloudModeError),
  Faithfulness(usize),              // missing must-preserve atoms
  Ssb(SsbError),
}

EscalationCascadeKind { LocalWin, Condensed, Escalated, BatchRouted }
  as_sql_str:
    escalation_cascade_local_win
    escalation_cascade_condensed
    escalation_cascade_escalated
    escalation_cascade_batch_routed

EscalationCascadePayload {
  kind: EscalationCascadeKind,
  condensed_bytes: u32,             // 0 on LocalWin
  ts_ms: i64,
}
```

(`escalation_waterfall.rs:66-122,338-350`, `ssb/payloads.rs:17854-17914`)

**Reuse, do not fork (I-11 / J-83):** `BeliefState`, `TriedApproach`, `CriticalAtom` / `AtomKind::BugLocation`, `BeliefStateEventKind::Condensed` (`I-11-typed-belief-state.md` §4, `J-83-grcv3-beliefstate-carrier.md` §4). Plan text `BeliefState::condense` (`I-plan/I-7_cascading_escalation_offline_batch.md:59,70`) is the **seam name**; live symbol is `EscalationWaterfall::condense` + `BeliefState::new` (`escalation_waterfall.rs:232-277`).

**Live cousin:** J-83 plan Kind names `{LocalWin, Escalated, BatchRouted}` (`J-83-grcv3-beliefstate-carrier.md` §4); live adds `Condensed` as the condensation beacon. Rebuild **live four-arm** enum; waterfall never emits `BatchRouted`.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `EscalationWaterfall`. The model proposes local candidates; the engine ranks, condenses, routes. No token-level FSM. No LLM inside the waterfall (`escalation_waterfall.rs:128-131,155-164`).

```
maybe_run_escalation_waterfall
     │
     ├─ flag OFF or no checker_runtime → no-op (linear retry stays)
     │
     ▼
SearchLadder::run_best_of_n (Local)
     │
     ├─ winner Some  → EscalationCascade(LocalWin)
     │                 routed=None, condensed_bytes=0
     │                 STOP (never cloud, never batch)
     │
     └─ winner None  → condense typed BeliefState (not raw history)
                       encode, cap bytes
                       BeliefStateEvent(Condensed)     // ID-3; missing = FAIL
                       EscalationCascade(Condensed)
                       CloudModeRouter::route(ExecMode::Cloud)
                       EscalationCascade(Escalated)
                       return routed DECISION
                       billed inference = D-026 PENDING (not here)
```

**Forced transitions:**

- Local success **cannot** open the cloud arm (`escalation_waterfall.rs:186-195`).
- Live-loop path **cannot** import or call the H-5 batch client (IC-1) (`:13-18,98-101`).
- Escalate payload **must** be I-11 BeliefState, not transcript (`I-plan/I-7_cascading_escalation_offline_batch.md:84-85`).
- `BatchRouted` **must not** be emitted from `escalation_waterfall.rs` (`ssb/payloads.rs:17875-17879`).

**J-85 neighbor (cite, do not author):** after **~3 failed receipted retries**, GRC-v3 overthinking guard signals `MODALITY_CHANGE` → retrieve / **I-7 escalate** / I-8 abstain — **never** “reason more” (`J-85-grcv3-overthinking-guard.md` §3 step 9). I-7 is a **destination**, not the retry counter.

**C-10 neighbor:** anti-spiral catches **repetition** inside a loop (`C-10-anti-spiral-lateral-thinking.md` §1). I-7 is **lane change after local-all-fail**, not a persona switch.

## 6. Grounding, receipts, gates

Local “did it work?” is the **I-17 tri-layer receipt** (via I-4), never model self-grade. Escalate **is** condensation + ID-3 receipt + H-7 route decision.

| Rank | Signal | Legal here? |
|---|---|---|
| 1 | I-4/I-17 survivor vs all-pruned | **the only LocalWin vs escalate signal** |
| 2 | I-11 `BeliefStateEvent(Condensed)` + `EscalationCascade` | **the only cloud-payload receipt** |
| 3 | LLM “send this to Sonnet / Batch” | **BANNED** (IC-1, TD-3) |

**Never allowed as a confidence / routing signal:** model-chosen backend; speculative local+cloud race; heartbeat-primary (IC-4, not this organ) (`I-plan/I-7_cascading_escalation_offline_batch.md:22-25`); raw CoT as the cloud prompt; window size as “we tried enough locally.”

**D-13:** `maybe_run_escalation_waterfall` constructs `TriLayerStack` with `AdversaryConfig::default_for_local_worker()` (`task_runner.rs:14288-14295`). That is D-13 **config plumbing**, not Mode B. Do not replace D-13 with this waterfall (`D-13-adversary-role-cross-family.md` §1).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| condensed payload | `CONDENSED_PAYLOAD_MAX_BYTES = 8_192` (~2K tokens, `[DESIGN]` ID-5) | per escalate (`escalation_waterfall.rs:69-75,200`) |
| plan observable | condensate **<= ~2K tokens** vs **>= 20K** raw-history control | per escalate (`I-plan/I-7_cascading_escalation_offline_batch.md:16-17`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:191`) |
| SCHEMA | EscalationCascade **262** | `ssb/payloads.rs:17854-17856` |
| Plan SSB baseline | +1 from LIVE **221/532** as of 2026-06-10; re-read at edit (W-9) | `I-plan/I-7_cascading_escalation_offline_batch.md:112-113` |
| I-11 honesty job | every **10** escalations (I-11 organ, not re-authored) | `I-11-typed-belief-state.md` §7 |
| flag | default-**OFF**; truthy `1\|true\|yes\|on` | per process (`escalation_waterfall.rs:338-350`) |
| multi-hop cascade | **DEFER** until `>=3` measured runs show single-hop insufficient | Tier-2 (`I-plan/I-7_cascading_escalation_offline_batch.md:52-53`) |

**Overflow:** encoded condensate truncated to `CONDENSED_PAYLOAD_MAX_BYTES` (`escalation_waterfall.rs:200`). `emit` uses `u32::try_from(condensed_bytes).unwrap_or(u32::MAX)` for the row field (`:294`) — production path still has that saturate, not a panic.

**Do not confuse with:** Phase-2A `GrcBudget` 5/800; I-26 thinking-token budget; J-84 effort **tier** (sizes reasoning; does not pick Local vs Cloud).

## 8. Roles and neighbors

Role-agnostic dispatcher. Cloud **architect** is the H-7 escalate **target**, not a model-chosen role (`I-plan/I-7_cascading_escalation_offline_batch.md:118-119`).

**Consumes:** I-4 `SearchLadder` (fan-out + I-17 + I-5); I-11 `BeliefState::new` / `encode` / `emit_condensed_receipt`; H-7 `CloudModeRouter`; E-39 cost gate (LocalWin ⇒ zero cloud rows); I-2 isolation via the ladder (`I-plan/I-7_cascading_escalation_offline_batch.md:68-73,115-121`).

**Produces:** `WaterfallOutcome`; `EscalationCascade` rows; the typed condensate object handed to the (pending) cloud call.

**Must not fork:** a second BeliefState encoder; a prompt “summarize for the cloud”; a live `MessageBatch` client; a parallel race.

**Cite, do not re-author:**

- **I-11** — typed condensate (`I-11-typed-belief-state.md`).
- **J-83** — carrier completion + this waterfall as PRODUCTION_HOOK (`J-83-grcv3-beliefstate-carrier.md` §3 step 1; live `condense` at `escalation_waterfall.rs:232`, not the stale grounding-map `:239`).
- **I-5** — per-branch prompt hygiene inside the ladder (`I-05-pristine-context-injector.md`).
- **I-4** — receipt ranker (`I-04-evidence-ranked-search-ladder.md`).
- **H-39** — not this organ (I-8 credits abort; I-7 condenses failures).
- **C-10** — repetition detector, not lane change.
- **D-13** — Mode B adversary; I-7 only plumbs `AdversaryConfig` into the local tri-layer.

**H-5:** compose offline batch; **do not** re-implement the client (`I-plan/I-7_cascading_escalation_offline_batch.md:33-34,82-83,116-117`).

## 9. Config / flags / defaults

- `ELAI_ESCALATION_WATERFALL_ENABLE` default-OFF; promotion plan `I-7-promotion`; wire-dark-allowlist row (`ssb/payloads.rs:17916-17922`).
- Second gate: `resolved.checker_runtime` must be Some (`task_runner.rs:14271-14279`).
- No dashboard cascade tile in v1 — Tier-2 behind GD-4; H-9 cost-cockpit family (`I-plan/I-7_cascading_escalation_offline_batch.md:53-54,87-90`).
- Decisions doc: `workflow/reasoning/I-7-decisions.md` (`:50`) — **not present** at authoring of this architecture file (path read failed).
- `CloudConfig::default()` Auto mode, Anthropic Sonnet; waterfall **forces** `ExecMode::Cloud` (`escalation_waterfall.rs:328-335`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/escalation_waterfall.rs` | new module (`I-plan/I-7_cascading_escalation_offline_batch.md:42,57,65-66`) |
| `EscalationWaterfall::run` | produces_symbols (`:66`) |
| `task_runner::maybe_run_escalation_waterfall` | PRODUCTION (`:75`, `task_runner.rs:14250`) |
| `BeliefState::new` / `encode` / `emit_condensed_receipt` | I-11 (`escalation_waterfall.rs:198-207,268`) |
| `CloudModeRouter::route` | H-7 (`escalation_waterfall.rs:215`) |
| `SearchLadder::run_best_of_n` | I-4 (`escalation_waterfall.rs:181-184`) |
| `PristinePathFilter::new` at dispatch | I-5 compose (`task_runner.rs:14296-14298`) |
| `ssb` `EscalationCascade` | SCHEMA 262 |
| `crates/contracts/tests/i7_escalation_cascade_contract.rs` | contract |
| H-5 batch client | **no import** in this module |

## 11. Journal / SSB events

**`EscalationCascade`** Pattern-2 4-arm (`ssb/payloads.rs:17854-17914`):

```
EscalationCascadePayload {
  kind: LocalWin | Condensed | Escalated | BatchRouted,
  condensed_bytes: u32,
  ts_ms: i64,
}
```

**I-11 companion (mandatory on all-fail):** `BeliefStateEvent(Condensed)` — distinct from `EscalationCascade(Condensed)` (`escalation_waterfall.rs:202-209`). Un-journaled condensation is FAIL (`J-83-grcv3-beliefstate-carrier.md` §6 ID-3).

`BatchRouted` is **offline H-5 only**. Snapshot 221/532 in the plan is 2026-06-10; live SCHEMA 262. Rebuild payload **shapes**; do not rewind schema numbers.

## 12. Tests and observables

Named contract `i7_escalation_cascade_contract.rs` (`I-plan/I-7_cascading_escalation_offline_batch.md:91-101`, live header `:1-24`):

| Id | Assert |
|---|---|
| (a.1) LocalWin | empty-registry fixture → `local_win`; `routed=None`; `condensed_bytes=0`; exactly one `EscalationCascade(LocalWin)` in **real DB**; **zero** Escalated (`i7_escalation_cascade_contract.rs:11-14,122-147`) |
| (a.2) all-fail | always-veto registry → Condensed then Escalated; `RoutedProvider::Cloud` decision (`:15-17`) |
| (b) IC-1 | `rg 'batch_client\|MessageBatch\|anthropic_batch' escalation_waterfall.rs` **MUST be 0**; injected batch call **fails to compile** (`:18-21`, `I-plan/I-7_cascading_escalation_offline_batch.md:98-101`) |
| (c) default-OFF | env unset → `escalation_waterfall_enabled()==false` (`:22-23`) |

**Plan CLI observable:** `elai run --task fixtures/search/local_solvable.toml --waterfall` (LocalWin, zero cloud-API journal rows) **and** `local_fail.toml` (Condensed then Escalated, payload `<=2K` from journal) (`I-plan/I-7_cascading_escalation_offline_batch.md:91-97`).

Adversarial: parallel local+cloud race; batch on interactive; raw-history escalate; `#[cfg(test)]` as the only caller (R7).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- **Waterfall, not a race.** Local K-branch + deterministic checks first. Cloud only on **all-local-fail**.
- **LLM does not pick the backend.** Route is a host decision (H-7 / equivalent). Batch is **structurally** unreachable from the live module.
- Local failure **condenses into BeliefState** (tried / failed-because / constraints / must-preserve atoms), then escalates that object. **No raw history.**
- ID-3: every condensation journals a receipt. Missing row = FAIL.
- LocalWin ⇒ **zero** cloud spend in the journal (E-39).
- `EscalationCascade` kinds `LocalWin` / `Condensed` / `Escalated`; `BatchRouted` reserved for **offline** only.
- Production caller on the task-runner. A unit test is not enough (R7).
- Default-OFF until a measured first-win (GD-4), unless the host’s equivalent promotion already landed.

A prompt “escalate if unsure”, a Batch-API race, or shipping the transcript to the cloud is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Batch API on any interactive or live search-loop path (IC-1) (`I-plan/I-7_cascading_escalation_offline_batch.md:78-80`).
- Async / speculative parallel local+cloud submission that depends on cancel-refunds (`:22-25,81-82`).
- Re-implementing the H-5 Batch client (`:82-83`).
- Raw history in the cloud payload (`:84-85`).
- Heartbeat-primary as this organ (IC-4 elsewhere) (`:23`).
- Multi-hop local → cloud-small → cloud-large in v1 (`:52-53`).
- Re-authoring I-11 / I-4 / I-5 / H-7 / E-39 / J-83 (GD-1).
- Treating I-11 PARTIAL as “I-7 already done” (J-83 DC-1 analog).
- SHA-2; `.unwrap()` on prod paths (`:86`) — live `u32` saturate on emit is the documented cousin (`escalation_waterfall.rs:294`).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-7_cascading_escalation_offline_batch.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:141-154`. Feature map: `ELAI-FEATURE-MAP.md:2351-2354,3437`. Live: `escalation_waterfall.rs`, `task_runner.rs:14225-14348`, `ssb/payloads.rs:17854-17922`, `contracts/tests/i7_escalation_cascade_contract.rs`. Neighbors: `I-11-typed-belief-state.md`, `J-83-grcv3-beliefstate-carrier.md`, `I-04-evidence-ranked-search-ladder.md`, `I-05-pristine-context-injector.md`, `J-85-grcv3-overthinking-guard.md` §3 step 9, `D-13-adversary-role-cross-family.md`.

IC-1 sources named by the plan (WHY, not an ELAI number): Review_ADECO-Adversarial-Critique §1/§4-2 + Research_RateLimits (`I-plan/I-7_cascading_escalation_offline_batch.md:104-107`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Whether `maybe_run_escalation_waterfall` currently builds a **real** candidate set from the completed task or a “small deterministic candidate set” stub (`task_runner.rs:14244-14247`) — exact construction after `ladder = SearchLadder::new` not re-copied here.
- D-026: **when** the billed cloud-architect call consumes the condensate (not in this organ; do not fake it) (`escalation_waterfall.rs:31-38`).
- v1 empty `current_ast_state` / `dkg_entities` / `token_cost: 0` vs I-11 plan fields (`escalation_waterfall.rs:227-231,237-242`).
- FEATURE-MAP “PABU leg still PLANNED” (`:2353`) vs I-11 PARTIAL live — do not treat either as J-83 typed `Entry` completion.
- `workflow/reasoning/I-7-decisions.md` missing at this write.
- Exact E-39 symbol that **asserts** zero cloud-API rows on LocalWin (plan-asserted; compose, do not invent a second meter).
- Whether `emit_best_effort` on Condensed (non-fatal write) conflicts with ID-3 fail-closed (I-11 `emit_condensed_receipt` **does** propagate; cascade Condensed beacon does not) (`escalation_waterfall.rs:202-209,303-320`).
- Multi-hop thresholds (Tier-2).
