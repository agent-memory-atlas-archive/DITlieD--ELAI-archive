# Typed BeliefState handoff (I-11 / PABU)

- **ELAI ids:** I-11 / IC-8 / ID-3 / IN-2 / CL-1..CL-4 / SCHEMA ~248+ (plan baseline+1 from 221/532)
- **Spec status (as designed):** PARTIAL (`ELAI-FEATURE-MAP.md:1143-1148`; `BeliefState::new` + `escalation_waterfall.rs` Condensed wired; honesty / full I-7 payload-size gate still completing). Original plan WAVE 2 BUILD (`I-plan/I-11_pabu_belief_state_handoff.md:3`). J-83 **extends** this carrier into GRC-v3 claims+receipts (`ELAI-FEATURE-MAP.md:642-655`) — cite, do not rebuild J-83 here.
- **Crate / module (spec):** `crates/orchestrator/src/belief_state.rs` (`BeliefState`, `HonestyJob`); production: `escalation_policy.rs::condense_and_escalate` / live `escalation_waterfall.rs` + `session_dag.rs::resume` decode + `task_runner::run_honesty_job_if_due`
- **Purpose of this file:** enough architecture to rebuild the typed handoff condensate. Not a status ledger.

## 1. What it is

The **typed BeliefState** is a **versioned condensate** that replaces **raw session history** on every cross-model or cross-tier handoff (local→cloud escalation, architect→editor, session resume). It carries **tried / failed-because / current-AST-state / constraints** plus **DKG-keyed entities**, never a transcript dump (`I-plan/I-11_pabu_belief_state_handoff.md:9-18`, `I-plan/I-SUMMARY_INDEX.md:216-230`). PABU/Letta/MemGPT is the commodity shape; ELAI’s IC-8 package is **typed + DKG-linked + journaled (ID-3) + honestly measured** (`:10-12`). A schema-version mismatch on read **fails closed** (typed error, never silent parse) (`:14`). GRC-v3 (J-83) completes the same struct into **claims + external receipts** (confidence never model-set) — this organ already **forbids raw transcript** as the wire (`ELAI-FEATURE-MAP.md:642-650`).

## 2. Why it exists

Handoffs that ship the full session bloat the cloud payload and make compression cost **assumed**. If this organ is missing, I-7 escalation is a raw-history dump (≥20K-token control in the plan observable). If it is faked (a prose “summary” the model wrote, or a silent schema parse), ID-3 is dead: replay cannot reconstruct what the escalated model saw, and a version bump becomes a quiet wrong decode (`I-plan/I-11_pabu_belief_state_handoff.md:4-8,21-22,63-66`, `I-plan/I-0_INDEX.md:232-234`). Honesty without a ledgered full-history-vs-BeliefState delta is marketing.

## 3. How it works (runtime)

1. **Who calls it.** Orchestrator, at **three production sites**: (a) I-7 / H-7 local→cloud **condense-then-escalate**, (b) architect→editor routing, (c) session resume (`:15-16,59-61`). The model does not construct or version the struct.
2. **Input.** Post-`adaptive_reduce` slot snapshot (IS-6) — BeliefState **types** that condensate; it does not re-implement reduction (`:26,56`). Failed-branch set on I-7 (tried labels + structured failure reasons) (`escalation_waterfall.rs:228-231`).
3. **Encode (deterministic).** `BeliefState::new(...)` stamps `version = BELIEF_STATE_SCHEMA_VERSION`. Serialize MessagePack (`rmp_serde`). Hash **blake3 only** (`:33-34,68`; live `belief_state.rs:121-125,211-224`).
4. **ID-3 receipt (deterministic, mandatory).** Every condensation emits `BeliefStateEvent(Condensed)` with `schema_version` + `token_delta`. Missing row = FAIL (`:17-18,65`). Dispatch emits `BeliefStateEvent(Handoff)` (`:53`).
5. **Wire payload.** Escalation carries **prefix + BeliefState only**. Raw history is **structurally absent** from the cloud path (removed from escalation, **not** from session storage) (`:21-22,64,74-75`).
6. **Decode (deterministic, fail-closed).** Resume / receiving side: `BeliefState::decode`. Version ≠ current → typed mismatch error **before any model call** (`:14,66,75`). Then H-6 `recover_stubs` **re-expands** detail — I-11 **calls**, never reimplements (`:20,58`).
7. **Honesty job.** After every **N=10** escalations (configurable; `--force-honesty-check` fixture) re-run full-history vs BeliefState, ledger `HonestyDelta`, recomputable under `ELAI_SEED` (`:36,74`).
8. **Commit.** Condensed/Handoff/HonestyDelta rows in SSB. Artifact = encoded blob + receipts.
9. **Abstain / fail-closed.** Silent mismatch banned. Un-journaled condensation banned. Receiving model never sees a blob that failed version check (`:65-67,75`).
10. **The model may generate** approach labels and (on the far side) code given the condensate. It may **not** author `version`, `must_preserve` atoms (K-12/TD-3 live fill), confidence bits (J-83 / H-39), or sneak the transcript back onto the wire.

## 4. Types, enums, objects

Copy from `I-plan/I-11_pabu_belief_state_handoff.md:13-18,50-53` plus live fields that name this organ. Do not invent.

```
BELIEF_STATE_SCHEMA_VERSION: u32 = 1

BeliefState {
  version: u32,                      // mismatch -> typed error
  tried: Vec<TriedApproach>,         // plan: tried
  failed_because: Vec<String>,       // plan: failed-because
  current_ast_state: Vec<u8>,        // plan: current-AST-state (opaque)
  constraints: Vec<String>,          // plan: constraints
  dkg_entities: Vec<i64>,            // plan: DKG-keyed entities (live: DkgNodeInfo node_id)
  token_delta: u32,                  // live: tokens saved by adaptive_reduce
  critical_atoms: Vec<CriticalAtom>, // K-12 fill; serde default
  quarantined_claims: Vec<String>,   // L-8 fill; serde default
}

TriedApproach { label: String, token_cost: u32 }

BeliefStateError {
  VersionMismatch { encoded, current },  // plan name: BeliefStateVersionMismatch
  Decode(...), Encode(...), Ssb(...),
  ReceiptRejected { reason },            // live; J-83-shaped commit gate
}

BeliefStateEvent.Kind { Condensed, Handoff, HonestyDelta }

BeliefStateEventPayload {
  kind: BeliefStateEventKind,
  schema_version: u32,
  token_delta: u32,     // 0 on HonestyDelta arm in live comment
  payload_bytes: u32,   // Handoff size; 0 on Condensed / HonestyDelta
  ts_ms: i64,
}

HonestyJob { interval_escalations: u32, seed: u64 }
HONESTY_JOB_INTERVAL: u32 = 10

HonestyDeltaPayload {
  full_history_tokens: u32,
  belief_state_tokens: u32,
  token_delta: u32,          // saturating sub
  compression_ratio: f64,    // belief / full; 1.0 if full==0
  seed: u64,
}

fn BeliefState::encode() -> Result<Vec<u8>, BeliefStateError>
fn BeliefState::decode(bytes: &[u8]) -> Result<BeliefState, BeliefStateError>
fn BeliefState::blake3_prefix() -> Result<String, BeliefStateError>   // live 8-byte / 16-hex
fn emit_condensed_receipt / emit_handoff_receipt
fn HonestyJob::run(...) -> Result<HonestyDeltaPayload, BeliefStateError>
fn run_honesty_job_if_due(..., force: bool)   // production; skip unless count%10==0 or force
```

**K-12 / J-83 fills (neighbors, same file, not extra organs):**

```
AtomKind { FilePath, Symbol, TestName, LineSpan, BugLocation, SafetyConstraint, Other }
CriticalAtom { kind, value, must_preserve: bool }  // LLM never sets must_preserve
ExternalReceipt { passed, abstained, evidence }    // live private; NO confidence field
```

J-83 entry shape `{claim, external receipt, abstention/confidence READ from H-39, DKG provenance}` is **that** organ (`ELAI-FEATURE-MAP.md:645-649`). Rebuild I-11 **without** treating a raw transcript as an allowed `current_ast_state`. AST bytes are a **snapshot**, not chat.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator middleware. Generator never owns canonical handoff state (ID-1 / ID-3) (`I-plan/I-0_INDEX.md:222-234`).

| Site | Encode / decode | Event |
|---|---|---|
| I-7 local-fail escalate | encode after adaptive_reduce | Condensed then Handoff |
| H-7 architect→editor | encode | Condensed / Handoff |
| session resume | decode then H-6 recover_stubs | (no new condensation) |
| every 10 escalations or `--force-honesty-check` | HonestyJob::run | HonestyDelta |

**Forced transitions:**

- Version mismatch → **stop before model call** (`:75`).
- Un-journaled encode → FAIL ID-3 (`:65`).
- Raw-history broken arm on escalation → payload-size gate FAIL (`:75`).
- Schema **migration tooling** is Tier-2; v1 is fail-closed only (`:41-42`).
- DKG-link **staleness detection** is Tier-2 (`:41`).

**H-51 split:** same digest **engine**, different **scope**. I-11 = live handoff; H-51 = sleep-time / cross-session. Do not reimplement H-51 (`:16,19,57,67`).

## 6. Grounding, receipts, gates

BeliefState **is** the receipt-bearing handoff object. It is **not** a chat log.

**Selector hierarchy for what may enter the blob:**

| Allowed | Source |
|---|---|
| tried / failed-because / constraints | structured orchestrator fields (SSB Finding / branch ids) |
| current_ast_state | SessionDag snapshot bytes |
| dkg_entities | DKG node ids, reachable at decode (`:88`) |
| token_delta | adaptive_reduce measurement |
| (J-83) claim + **external** receipt | D-10-routed domain receipt must **pass** to commit |
| (K-12) critical_atoms | orchestrator / structured; `must_preserve` not LLM-set |

**Never allowed as the handoff body or as a confidence signal:** raw transcript; model-written “summary memory”; LLM self-rating that a condensation is faithful; silent decode of a foreign schema; un-journaled compress (`:21-22,64-66`). Confidence/abstention is **read from H-39**, never stored as a generator field (J-83; live `ExternalReceipt` comment at `belief_state.rs:89-92`).

**Honesty job is measurement, not a grade.** It ledgers success/token delta. It does **not** let the model certify the condensate (`:36`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| escalation payload | **≤ ~2K tokens** vs **≥20K** raw-history control (`:74-75`, `I-plan/I-SUMMARY_INDEX.md:227-228`) | per handoff |
| raw-history broken arm FAIL | payload **>10K** tokens (`:75`) | R2 |
| honesty interval | **10** escalations default; configurable (`:36`, `HONESTY_JOB_INTERVAL`) | per session |
| schema version | **1** (`BELIEF_STATE_SCHEMA_VERSION`) | wire |
| module size (plan) | ~200 LOC (`:45`) | advisory |
| blake3 prefix (live) | 8 bytes / 16 hex (`belief_state.rs:211-224`) | identity |

**Overflow:** plan does not name truncate-vs-reject when the condensate exceeds ~2K. **UNCERTAIN** (§16). Live I-7 condense may leave `current_ast_state` / `dkg_entities` **empty in v1** (`escalation_waterfall.rs:228-231`) — that shrinks the blob; it does not license raw history.

CL-1..CL-4 bind **local vs cloud** policy; BeliefState sits at that boundary (`:29`, `I-plan/I-0_INDEX.md` CL family). Do not treat context-window size as the condensation budget (RD-4 / 3E neighbor).

## 8. Roles and neighbors

Handoff is **orchestrator middleware**, role-agnostic. Architect/editor (H-7) **consume** the blob; they do not schema it.

**Consumes:** `context_budget.rs::adaptive_reduce` (IS-6); `escalation_policy.rs` handoff sites; `session_dag.rs` resume; H-6 `recover_stubs`; DKG at decode; I-7 waterfall (`:54-58,84-89`).

**Produces:** encoded BeliefState; `BeliefStateEvent` rows; `HonestyDeltaPayload`. SessionsDrawer inspector row (`:38,70-71`).

**Must not fork:** a second “memory layer” that ships transcript; Letta/MemGPT as a drop-in (missing IC-8); reimplementation of H-51; J-83 as a **second** struct (J-83 **extends** this file). N-4 later unifies Claim/Belief objects (`ELAI-FEATURE-MAP` N-4) — cite, do not merge here.

**I-7:** local win never leaves the machine; local-fail condenses **≤2K** then escalates **once** (`I-plan/I-SUMMARY_INDEX.md:151-154`). I-4 search failures feed tried/failed-because, not traces (I-5 still strips tracebacks on branch prompts).

## 9. Config / flags / defaults

- Original I-11: **no env kill-switch**. Condensation is the escalation path.
- Honesty: interval default 10; `--force-honesty-check` fixture (`:36,74`).
- `--local-first --emit-ssb` on the observable trigger (`:73`).
- Live `run_honesty_job_if_due`: skip if `escalation_count==0 && !force`; seed from `ELAI_SEED` else 0; SSB failure **warns, does not abort the task** (`belief_state.rs:450-489`). Plan ID-3 is **fail** on **missing condensation receipt** — do not generalize live honesty non-fatal to Condensed.
- Frontend: BeliefState inspector in SessionsDrawer **required** (HonestyDelta is an economics signal) (`:70-71`).
- Serialization: MessagePack; **no** blake2/sha2 (`:68-69`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/belief_state.rs::BeliefState` | `I-plan/I-11_pabu_belief_state_handoff.md:50` |
| `HonestyJob` | `:51` |
| `BeliefStateEvent` Condensed/Handoff/HonestyDelta | `:53` |
| `escalation_policy.rs::condense_and_escalate` → `encode` | `:60` |
| `session_dag.rs::resume` → `decode` | `:61` |
| `context_budget.rs::adaptive_reduce` | `:56` |
| H-6 `recover_stubs` | `:58` |
| live `task_runner::run_honesty_job_if_due` | `belief_state.rs:22-26,451` |
| live `escalation_waterfall.rs` condense | `:228-253` |
| `crates/contracts/tests/i11_belief_state_contract.rs` | F1–F5, W1–W3 |
| SessionsDrawer inspector | `:47,70-71` |

**Live neighbors:** `lib.rs` re-exports `BeliefState`, `HonestyJob`, `run_honesty_job_if_due` (`lib.rs:1520`). J-83 continues in the **same** `belief_state.rs` (`ELAI-FEATURE-MAP.md:654`).

## 11. Journal / SSB events

Plan: **+BeliefStateEvent** Kind `{Condensed, Handoff, HonestyDelta}` (`:53,82`). Observable expects `schema_version: 1` and `token_delta` on Condensed (`:74`).

Live payload: `BeliefStateEventPayload` in §4 (`ssb/payloads.rs:17204-17242`). Pattern-2 single-variant 3-arm (`ssb/action_type.rs:4321-4328`).

Replay must reconstruct **exactly what the escalated model received** (ID-3) (`I-plan/I-0_INDEX.md:232-234`). The encoded blob is the artifact; the row is the receipt. Do not store the transcript on the event (live redaction note: BeliefState bytes are opaque `Vec<u8>`, not a top-level String — `redaction.rs:477-481`).

## 12. Tests and observables

Named requirements (`:63-75`, `I-plan/I-SUMMARY_INDEX.md:227-230`, live F1–F5 / W1–W3):

| Id | Assert |
|---|---|
| encode/decode | lossless MessagePack over tried / failed_because / AST bytes / constraints / dkg ids / token_delta (F2) |
| version fail-closed | foreign `version` → `VersionMismatch`, no silent parse (F3) |
| blake3 stable | same state → same 16-hex prefix (F4) |
| SSB 3 arms | Condensed / Handoff / HonestyDelta round-trip real sqlite (F5) |
| ID-3 | every condensation has a Condensed row (missing = FAIL) |
| payload size | prefix+BeliefState ≤~2K tokens vs ≥20K raw control |
| R2 raw-history arm | bypass → >10K tokens, FAIL |
| mismatch-before-model | version-bumped blob errors before any model call |
| honesty interval | `escalation_count==10`, `force=false` → HonestyDelta (W3) |
| `--force-honesty-check` | HonestyDelta without waiting for 10 |
| GD-2 | HonestyDelta recomputable under `ELAI_SEED` |
| W1/W2 | `evaluate_handoff` + `adaptive_reduce` reachable (production symbols) |

Trigger (plan): `cargo run -p elai-cli -- run --task 'implement error handling for a Rust function' --local-first --emit-ssb` (`:73`).

Adversarial: silent schema parse; un-journaled `encode`; transcript on the cloud request; LLM-authored `must_preserve`; honesty job that calls an LLM.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A **typed, versioned** `BeliefState` with tried / failed-because / current-AST / constraints / DKG keys — **not** a transcript.
- Encode/decode with **fail-closed** version mismatch **before** any model call.
- Escalation / cross-tier handoff payload is **prefix + BeliefState only**.
- Every condensation emits `BeliefStateEvent(Condensed)` (ID-3). Handoff and HonestyDelta exist.
- Honesty job: full-history vs BeliefState, default every 10 escalations, `ELAI_SEED`-recomputable, **no LLM**.
- Receiving side calls H-6-style stub recovery; does **not** reimplement H-51 sleep digest.
- Hash is blake3. No `.unwrap()` on production decode.
- Raw-history passthrough on escalation is a **failing broken arm**.

A Letta/MemGPT blob, a model-written summary, or J-83’s per-step receipt gate **without** this condensate is a **cousin** (J-83 is an extension of **this** type, not a replacement name).

## 14. Non-goals / banned designs

- Raw history on the escalation path (`:21-22,64`).
- Silent schema mismatch (`:66`).
- Un-journaled condensation (`:65`).
- Reimplementing H-51 (`:67`).
- Automated DKG-link staleness (Tier-2) (`:41`).
- Schema migration beyond fail-closed (Tier-2) (`:42`).
- blake2/sha2 (`:69`).
- Model-set confidence on the carrier (J-83 / H-39; live ExternalReceipt has none).
- Using window size as the 2K budget (3E / RD-4).
- Treating `current_ast_state` as a place to smuggle chat.

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-11_pabu_belief_state_handoff.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:216-230`. Doctrine: `I-plan/I-0_INDEX.md:232-234` (ID-3), IC-8 in the I-11 file. Neighbors: I-7 `I-SUMMARY_INDEX.md:141-154`; J-83 `ELAI-FEATURE-MAP.md:642-655`; K-12 atoms `ELAI-FEATURE-MAP.md:1072-1080`. Feature map: `ELAI-FEATURE-MAP.md:1143-1148`. Live: `crates/orchestrator/src/belief_state.rs`, `escalation_waterfall.rs:228-253`, `ssb/payloads.rs:17204-17242`, `crates/contracts/tests/i11_belief_state_contract.rs`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact token tokenizer for “≤~2K tokens” vs byte size (`payload_bytes` is bytes) (`:74-75` vs live payload field).
- What **prefix** is (frozen cache prefix? H-51 digest prefix?). Named, not specified.
- `condense_and_escalate` vs live `escalation_waterfall` / `evaluate_handoff` symbol names (`:60` vs live).
- Whether `session_dag.rs::resume` vs `load_handoff` is the decode site (file header names `load_handoff`; wiring names `resume`) (`belief_state.rs:24` vs `:61`).
- Honesty **success-rate** delta (plan `:7,36`) vs live **token** delta only (`HonestyDeltaPayload` has no success field).
- Live honesty SSB failure is non-fatal (`:450-489`); plan ID-3 fail-closed is written for **condensation** receipts — honesty miss policy unspecified.
- v1 waterfall leaves AST/DKG empty (`escalation_waterfall.rs:228-231`) vs plan fields required at schema level.
- `BeliefStateVersionMismatch` as a distinct type vs `BeliefStateError::VersionMismatch`.
- Whether frontend inspector is W-4 required or deferrable (plan says required `:70-71`).
- How DKG “reachable at decode time” fails (hard error vs drop links) (`:88`).
- `commit_external_receipt` process-local freeze ledger is **not** in the I-11 wire (`belief_state.rs:100-101`) — J-83 territory.
- SCHEMA_VERSION_PUBLIC floor in F1 (≥248) vs plan “baseline+1 from 221/532” (`:82` vs contract F1).
