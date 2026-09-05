# GRC-v3 BeliefState reasoning-state carrier (J-83)

- **ELAI ids:** J-83 / I-11 / GA-1 / CV-1..CV-5 / WAVE GRC-1 / IMPL 10/11/18/20/31
- **Spec status (as designed):** PLANNED (KEYSTONE of GRC-v3). I-11 condense-on-escalate seam is PARTIAL/live; the typed-entry carrier is the completion (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:1-3`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:23-26`, `ELAI-FEATURE-MAP.md:642-655`).
- **Crate / module (spec):** `crates/orchestrator/src/belief_state.rs` (Entry + per-step receipt gate + commit-and-freeze); threads `escalation_waterfall.rs` I-7 seam; consumes `abstention_score.rs`, `select_grounding_mode`, `context_policy.rs`; gate home is GRC Verify in `crates/reasoning`.
- **Purpose of this file:** enough architecture to rebuild the claims+receipts carrier that crosses a model/tier/unload boundary. Not a status ledger.

## 1. What it is

The **GRC-v3 BeliefState reasoning-state carrier** is the typed unit of reasoning that crosses a model, tier, or context-unload boundary as **CLAIMS+RECEIPTS**, never as a raw transcript. GRC-v3 is **UNIFICATION + ACTIVATION** of pieces ELAI already ships, plus **ONE** net-new organ (the overthinking guard, J-85). J-83 is the keystone of that unification: each carried entry is `{claim, grounding-receipt, abstention/confidence bit READ from H-39, DKG provenance}`; a step commits iff its D-10-routed **external receipt** passes; a committed entry is **frozen**. The carrier is uniform; only the domain validator that closes a candidate changes (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:8-18`, `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:5-7`, `J-plan/J-SUMMARY_INDEX.md:1848-1861`).

I-11 already ships the **condense-on-escalate seam** (`BeliefState::new`, `emit_condensed_receipt`). J-83 **completes** that seam into the carrier. It is not a from-scratch engine (DC-1) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:23-26,130-132`).

## 2. Why it exists

ELAI splits reasoning, testing, and execution across different models and contexts (cloud vs local generator; auditor vs executor; per-role). The local lane has **no** cross-request KV reuse (PrefixKVCache is bypassed on the streaming path, DC-2). Reasoning state must therefore survive a model boundary **and** a context unload as a typed, loss-controlled object (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:11-12`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:133-135`).

If this organ is missing, a later step inherits a prior step’s ungrounded prose (the least-to-most **poison cascade**). If it is faked (the model grades which of its own parts is “sure”, or a raw chain is handed across escalate), the self-correction blind spot is back in the loop: a model cannot reliably ground itself. “The part the model is sure about proceeds” **must** mean “the part an **EXTERNAL RECEIPT** confirms proceeds” (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:13-18`, `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:108-109`, `ELAI-FEATURE-MAP.md:645-650`).

If commit-and-freeze is missing, the generator can oscillate a receipt-confirmed entry away (J-85’s defense has nothing to freeze). If receipts are folded lossily with the prose, the next model cannot re-verify (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:140-142`).

Honest residue (R-1): the risk is **seam composition** (I-11 / I-4 / I-7 / H-42 / D-10), not “will a piece work”. J-83 ships the carrier **and** its I-7 consumer **and** a multi-turn handoff smoke, never the carrier alone (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:151-155`, `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:89-90,115`).

## 3. How it works (runtime)

1. **Who calls it.** Production callers, not a `#[cfg(test)]` stand-in (R7):
   - GRC **Verify** step (`crates/reasoning`) → per-step receipt gate (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:55-56`).
   - I-7 waterfall after local-all-fail → condense the typed carrier and hand it on escalate (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:56-57`). Live PRODUCTION_HOOK: `escalation_waterfall.rs` `condense` + `BeliefState::emit_condensed_receipt` (`escalation_waterfall.rs:197-207`). The grounding-map pin `escalation_waterfall.rs:239` is **stale** against current source (that line is now `TriedApproach.label` assignment inside `condense`, `escalation_waterfall.rs:239-241`).
2. **Input.** A model-emitted **claim** (text) plus the orchestrator’s current `Claim`/`OrchContext` for D-10 routing. The model does **not** supply the receipt, the abstention/confidence bit, or a freeze flag (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:15,59`).
3. **Domain route (deterministic).** `select_grounding_mode(claim, ctx)` picks the validator. Same carrier for a Unity-render bug, a Sharpe-ratio bug, and a codebase bug; only the closing receipt differs (CV-5 / C4). Plan validators: code F-4/test, math D-11, visual J-1/B-24, trading H-17..H-29 (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:19`, `J-plan/J-SUMMARY_INDEX.md:1860-1861`). Live `select_grounding_mode` is at `grounding_mode.rs:110` (sound-checker Constructive branch `grounding_mode.rs:165`); that live enum is the D-10 **cousin** — rebuild the **plan** A–E router as the TYPE axis, do not fork a second selector (`D-10-mode-routing-predicate.md` §4 live-cousin note, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:47-49`).
4. **Run the domain validator (external).** Six grounding methods + F-4 cant-lie + CosmosCheck/VisualArtifactCheck are the receipts that close a candidate (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:27,52`). J-83 does not re-author those checkers (GD-1).
5. **Per-step receipt gate (CV-2, deterministic).** A step is **COMMITTED** to the carried prefix **iff** the domain receipt **PASSES**. Fail or missing → entry **ABSTAINS** (stays open), may take more depth or I-7 escalate. It is **never** committed on the chain’s own say-so (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:16,82`). Corrupt/empty receipt → ABSTAIN, fail-closed (R1), never silent commit (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:99`).
6. **Fill the entry (engine, not model).** `claim` from the generator. `grounding_receipt: Option<ReceiptRef>` from the validator. `confidence_bit` **READ** from H-39 `AbstentionScored` / eval-log outcome (`correct=+1` / `abstain=0` / `confident-wrong=-t/(1-t)`), never re-derived (RD-1, GS-4). `provenance` = DKG key + source SSB seq, hashed **blake3** not sha2 (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:15,23,62`).
7. **Commit-and-freeze (CV-3).** Committing a receipt-confirmed entry **FREEZES** it for the rest of the run. A later generator rewrite is **dropped + logged**. One operation, three payoffs: grounding (receipt confirmed), compression-safe (committed = foldable; receipts lossless, prose lossy, DC-4), overthinking defense (frozen against oscillation; J-85 consumes this) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:17,42,83`).
8. **Boundary transport (CV-4).**
   - **Escalate (I-7):** condense-then-hand **typed entries**, not raw history. Keep `BeliefState::new` / `emit_condensed_receipt` (GD-1). Live waterfall: local `run_best_of_n` → `LocalWin` or `condense` → `emit_condensed_receipt` → `EscalationCascadeKind::Condensed` then `Escalated` (`escalation_waterfall.rs:180-224`, `ssb/payloads.rs:17862-17879`).
   - **Local unload (H-50):** the carrier is what is unloaded and selectively reloaded. Cloud lane sits behind the cache-stable prefix. J-83 **rides** `context_policy.rs`; it does **not** re-author the policy (GS-2, DC-2). Live: `ContextPolicy::{Local,Cloud}` at `context_policy.rs:110-127`; kill-switch `ELAI_CONTEXT_POLICY_DISABLE` at `context_policy.rs:19-20,325`. Grounding-map pin `live_cli.rs:50` is **stale**; live consume is `context_policy_disabled` / window policy around `live_cli.rs:63,1440-1448` and the module header’s `live_cli.rs:1185` note (`context_policy.rs:32-34`).
9. **Compression rider (DC-4 / GA-4, not re-authored here).** Preserve RECEIPTS/observations losslessly; fold reasoning PROSE lossily (rederivable). Cloud: deterministic masking of trailing observations behind the cache breakpoint (HD-2). **Never** LLM re-summarization. Recorded as an I-14 rider, not a J-83 rewrite (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:73-78,140-142`).
10. **Pristine / must-preserve.** I-5 pristine injector + K-12 must-preserve atoms guard the commit so failed-path traceback does not poison the next unit (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:29`, `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:181-195`).
11. **Commit artifact.** Prefer-reuse `BeliefStateEvent` (+Kind). Plan Kind names if added: `EntryCommitted` / `EntryAbstained` / `CarrierHandedOnEscalate`. Decide at edit (W-9); net 0..+1 (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:67-68`).
12. **Abstain / fail-closed.** Missing receipt, failed receipt, corrupt/empty receipt → ABSTAIN, not commit. Model-set confidence bit is banned (stub_ban count of `set_confidence` in `belief_state.rs` MUST be 0) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:59-62,74`). Version mismatch on I-11 decode already fails closed (`belief_state.rs:40-48,192-199`).

The model may **generate** claim text. It may **not** write the confidence/abstention bit, adjudicate a commit, hand raw transcript across a boundary, or compact via LLM summary.

## 4. Types, enums, objects

Copy from `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:15-19,47-52` and I-11 live structs that J-83 **extends**. Do not invent fields.

**J-83 carrier entry (the completion; not yet the live I-11 struct):**

```
Entry {
  claim,                                          // model-emitted text; engine does not invent it
  grounding_receipt: Option<ReceiptRef>,          // None => open/abstained
  confidence_bit,                                 // READ from H-39 AbstentionScored
                                                  // correct=+1 / abstain=0 / confident-wrong=-t/(1-t)
                                                  // model NEVER writes this (TD-3, RD-1)
  provenance: { dkg_key, source_ssb_seq },        // blake3, not sha2
  frozen,                                         // set on commit; generator rewrite dropped+logged
}

BeliefState  // ordered set of Entry; UNIFORM across domains
  // EXTENDS I-11 wire; KEEP BeliefState::new / emit_condensed_receipt (GD-1)

ReceiptRef    // named; exact shape UNCERTAIN — see §16
              // plan preference: pointer to the SSB row that closed the claim (GD-2)

fn per_step_receipt_gate(claim, ctx) -> Commit | Abstain
  // select_grounding_mode → domain validator → commit iff PASS
```

**H-39 bit source (READ, never re-derived):**

```
OutcomeTriple { answered, correct, refusal: RefusalDecision }   // abstention_score.rs:51-58
fn parse_eval_log_outcome(eval_log: &str) -> Option<OutcomeTriple>  // :267
AbstentionScored { mean_score, abstention_rate, confident_wrong_rate, t, n, seed }
DEFAULT_CONFIDENCE_TARGET t = 0.75 LOCKED                    // abstention_score.rs:33-38
```

(`H-39-abstention-aware-scoring.md` §4; `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:23`)

**I-11 live wire (EXTEND, do not replace) — `belief_state.rs:122-145`:**

```
BeliefState {
  version: u32,                    // BELIEF_STATE_SCHEMA_VERSION = 1
  tried: Vec<TriedApproach>,       // { label, token_cost }
  failed_because: Vec<String>,
  current_ast_state: Vec<u8>,
  constraints: Vec<String>,
  dkg_entities: Vec<i64>,
  token_delta: u32,
  critical_atoms: Vec<CriticalAtom>,      // K-12; LLM never sets must_preserve
  quarantined_claims: Vec<String>,        // L-8
}

BeliefStateEventKind { Condensed, Handoff, HonestyDelta }  // live; ssb/payloads.rs:17214-17221
BeliefStateError { VersionMismatch, Decode, Encode, Ssb, ReceiptRejected }
```

**Live cousin (do not rebuild as this organ):** `ExternalReceipt { passed, abstained, evidence }` plus process-local `committed_steps()` HashSet and private `commit_external_receipt`, gated by `per_step_source_check_enabled()` inside `emit_condensed_receipt` (`belief_state.rs:89-105,227-266,275-298`). That is a **partial freeze ledger on I-11 constraints**, not the typed `Entry` carrier, not D-10-routed per-step commit, and not H-39 bit-read. Rebuild the **plan** Entry + gate.

**I-7 kinds the carrier threads (reuse, do not fork):** `EscalationCascadeKind { LocalWin, Condensed, Escalated, BatchRouted }` (`ssb/payloads.rs:17862-17879`). Plan text also names `{LocalWin, Escalated, BatchRouted}` (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:24`); live adds `Condensed` as the condensation beacon.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator + GRC engine. The model proposes a claim/step; the engine routes, verifies, commits or abstains, freezes. No token-level FSM. No LLM inside the gate (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:41,59`).

```
model emits claim
     │
     ▼
select_grounding_mode(claim, ctx)          // D-10; LLM has no input
     │
     ▼
domain validator runs (F-4 / D-11 / J-1 / H-trading / 6 methods)
     │
     ├─ PASS  → fill Entry{receipt=Some, bit=READ H-39, provenance}
     │            COMMIT to prefix → FREEZE
     │            generator rewrite of frozen entry → drop+log
     │
     ├─ FAIL or MISSING or corrupt/empty
     │            ABSTAIN (entry stays open)
     │            more serial depth  |  I-7 escalate  |  stay open
     │            NEVER commit on chain say-so
     │
     └─ I-7 local-all-fail
              condense typed carrier (not raw history)
              emit BeliefStateEvent(Condensed)     // ID-3; un-journaled = FAIL
              hand carrier on Escalated
              local unload/reload rides H-50
```

**Forced transitions:**

- First commit of a receipt-passing entry freezes it for the rest of the run (CV-3).
- Failed/missing receipt **cannot** enter the carried prefix as committed (CV-2).
- Escalation hands the **typed** carrier; raw-history passthrough is stub_ban (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:61`, `escalation_waterfall.rs:227-231`).
- `BatchRouted` is **not** a live waterfall arm (IC-1); do not emit it from this seam (`ssb/payloads.rs:17875-17879`).

**Must-preserve atoms (K-12):** orchestrator sets `CriticalAtom.must_preserve`; LLM never does (`belief_state.rs:78-86`). Live `condense` maps non-empty `failed_because` into `AtomKind::BugLocation` must-preserve atoms (`escalation_waterfall.rs:244-252`).

## 6. Grounding, receipts, gates

This organ **is** the per-step receipt gate. Confidence is an **EXTERNAL RECEIPT**, never model self-grade.

**Selector hierarchy (wave-wide; J-83 is the receipt rung at step grain):**

| Rank | Signal | Legal here? |
|---|---|---|
| 1 | Deterministic **external receipt** (D-10-routed domain validator) | **the only commit signal** |
| 2 | Vote over **independent** samples | **not this organ** — J-86 receipt-**absent** fallback only, never recall |
| 3 | LLM self-eval / “I am sure” / chain self-grade | **BANNED** (IC-6, TD-3, RR-1) |

(`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:13-18,136-138`, `ELAI-FEATURE-MAP.md:19` of the pack INDEX, `ELAI-FEATURE-MAP.md:685-687`)

**Never allowed as a confidence signal:** model-written `confidence_bit`; scalar 0–1 from the generator; majority vote on a factual-recall chain; raw CoT/L2M prefix as “the model progressed”; window fill; LLM re-summarization of the carrier (HD-2); SHA-2 provenance keys.

**Abstention bit source:** H-39 reads correct/wrong/abstain **from the eval log** (a receipt), plus `AbstentionScored` SSB (218). J-83 **reads** that bit; it does not re-derive it (GS-4, RD-1) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:34-37`, `abstention_score.rs:17-20,267-293`).

**I-11 ID-3:** every condensation emits `BeliefStateEvent(Condensed)`; un-journaled condensation is FAIL (`belief_state.rs:268-271`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| I-11 differentiator payload | journal-verified condensate **≤ ~2K tokens** vs **≥ 20K** raw-history control | per escalate (`ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:191`) |
| Live byte ceiling | `CONDENSED_PAYLOAD_MAX_BYTES = 8_192` (~2K tokens × ~4 B/token, labelled `[DESIGN]`) | per escalate (`escalation_waterfall.rs:70-76`) |
| Honesty job interval | `HONESTY_JOB_INTERVAL = 10` escalations | periodic full-history vs BeliefState delta (`belief_state.rs:342-345`) |
| I-11 wire version | `BELIEF_STATE_SCHEMA_VERSION = 1`; mismatch → `VersionMismatch` fail-closed | decode (`belief_state.rs:38`) |
| Provenance hash | blake3 | DKG key (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:36,62`) |
| SSB | prefer-reuse `BeliefStateEvent` (+Kind); net **0..+1** from snapshot LIVE **268/577** (2026-06-26); 9-tuple lockstep if a Kind is added | W-9 (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:118-126`) |
| Live schema | `SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH(-309) + SSB_ACTION_TYPES.len()`; do **not** rewind to 268 | `ssb.rs:1674-1689` |
| Feature flag | **none**; carrier is on the live reasoning path. If gated: default-ON behind `*_DISABLE` kill-switch + allowlist row | W-2 (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:91`) |
| Effort estimate | ~1.5 sessions; ~280 LOC `belief_state.rs` + ~30 LOC waterfall + ~80 LOC contract | plan (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:7,100-105`) |

**Overflow:** encoded condensate truncated to `CONDENSED_PAYLOAD_MAX_BYTES` on the live waterfall (`escalation_waterfall.rs:200`). Exact J-83 Entry size cap beyond that I-11 ceiling: **UNCERTAIN** (§16).

**Do not confuse with:** Phase-2A per-topic `GrcBudget` 5/800 (that is the step loop); 3E window slots; I-26 thinking-token budget; J-84 effort **tier** (this organ carries state, it does not pick depth vs width).

## 8. Roles and neighbors

Role-agnostic carrier. D-10 may special-case live roles in the **cousin** predicate (`grounding_mode.rs:115-128`); J-83 consumes the **plan** router, does not add a second role table.

**Consumes:** I-11 `BeliefState::new` / `emit_condensed_receipt` (EXTEND); H-39 bit; I-7 `EscalationCascadeKind` + waterfall; D-10 `select_grounding_mode`; 6 grounding methods + F-4; H-50 `context_policy.rs`; SessionDag branch/merge (candidate substrate for J-86); I-5 pristine + K-12 atoms.

**Produces:** ordered committed/abstained entries; freeze log; `BeliefStateEvent` rows; the object handed on escalate / local unload.

**Unlocks:** J-84 (sizes reasoning that **fills** the carrier), J-85 (protects **frozen** entries), J-86 (receipt-gated **candidate branches** over it) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:119`).

**Must not fork:** a second BeliefState engine inside the agent; a prompt-only “carry a summary”; MCTS as a replacement for the carrier; re-implementation of I-11 / H-50 / I-7 / D-10 (GD-1, IC-16).

**Parallel-safe with:** J-84 / J-85 (they read the carrier, land in budget/verify seams). **Sequence:** J-86 **after** J-83 (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:8`).

## 9. Config / flags / defaults

- No new `ELAI_*_ENABLE` for the carrier (W-2). If a gate appears: **default-ON**, `*_DISABLE` kill-switch, allowlist row (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:91`).
- H-50 already has `ELAI_CONTEXT_POLICY_DISABLE=1` (flat legacy trim). J-83 must not invent a second context policy (`context_policy.rs:19-21`).
- Live partial freeze is behind `per_step_source_check_enabled()` (`belief_state.rs:281`) — that flag is **not** the J-83 organ; the plan carrier is on the live reasoning path.
- Frontend: existing ContextStatePanel / GRC history surfaces read committed/abstained counts via SSB poll. Exact new tile **decide at edit** (W-4); per-entry receipt drawer is later UX. Justified N/A is allowed (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:64`).
- Decisions doc required: `workflow/reasoning/J-83-decisions.md` (entry shape; C4 one-carrier domain-routed validators; commit-freeze triple-payoff) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:44`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/crates/orchestrator/src/belief_state.rs` | I-11 live + J-83 Entry / gate / freeze (**EXTEND**) |
| `BeliefState::new` / `encode` / `decode` / `blake3_prefix` | keep; do not re-implement (`belief_state.rs:147-225`) |
| `BeliefState::emit_condensed_receipt` / `emit_handoff_receipt` | ID-3 receipts (`belief_state.rs:275-335`) |
| `HonestyJob` / `run_honesty_job_if_due` | I-11 honesty delta (`belief_state.rs:347-448`) |
| `escalation_waterfall.rs` `condense` + emit at local-all-fail | I-7 PRODUCTION_HOOK (`escalation_waterfall.rs:197-280`) |
| `abstention_score.rs` `parse_eval_log_outcome` | CV-1 bit source |
| `grounding_mode.rs::select_grounding_mode` | CV-5 domain router |
| `context_policy.rs` `regime_policy` / `ContextPolicy` | CV-4 lane policy (ride, do not rewrite) |
| GRC Verify (`crates/reasoning`) | CV-2 gate home |
| `crates/contracts/tests/i11_belief_state_contract.rs` | existing I-11 contract |
| `crates/contracts/tests/j83_beliefstate_carrier_contract.rs` | J-83 contract (~80 LOC; CREATE) |
| `workflow/reasoning/J-83-decisions.md` | CREATE |
| `ssb.rs` + `action_type_lock.rs` + golden | only if a Kind is added; 9-tuple lockstep |

**Live neighbors (implementation truth, not the rebuild names):** `commit_external_receipt` private (`belief_state.rs:230`); `ExternalReceipt` crate-private (`belief_state.rs:94`); waterfall `BeliefState::new` at `escalation_waterfall.rs:268`.

## 11. Journal / SSB events

Prefer-reuse **`BeliefStateEvent`** (Pattern-2). Live kinds: `Condensed`, `Handoff`, `HonestyDelta` (`ssb/payloads.rs:17214-17239`):

```
BeliefStateEventPayload {
  kind: BeliefStateEventKind,
  schema_version: u32,     // BELIEF_STATE_SCHEMA_VERSION
  token_delta: u32,
  payload_bytes: u32,
  ts_ms: i64,
}
```

Plan optional Kind add (decide at edit, W-9): `EntryCommitted` / `EntryAbstained` / `CarrierHandedOnEscalate` (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:67`).

I-7 companion rows (reuse, net 0): `EscalationCascade` with `EscalationCascadeKind` (`LocalWin` / `Condensed` / `Escalated`; live waterfall never emits `BatchRouted`).

H-39 companion (read, do not bump): `AbstentionScored` (SCHEMA 218 snapshot in feature-map).

GD-2: committed/abstained verdicts **recomputable from the SSB stream** (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:95`). Snapshot **268/577** in the grounding map is a 2026-06-26 derived pair; live `SCHEMA_VERSION` is `-309 + len(SSB_ACTION_TYPES)`. Rebuild **emits these payload shapes**; do not rewind a live schema number (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:118-126`).

## 12. Tests and observables

Named contract `j83_beliefstate_carrier_contract.rs` (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:97-99`):

| Id | Assert |
|---|---|
| (a) | receipt PASSES → entry **commits** |
| (b) | receipt FAILS or absent → **ABSTAINS**, not committed (poison-cascade fix) |
| (c) | model **cannot** set the confidence bit (READ from H-39) |
| (d) | committed entry is **frozen**; rewrite dropped |
| (e) | escalation hands the **typed carrier**, not raw history |
| (f) | corrupt/empty receipt → ABSTAIN fail-closed, never silent commit |

**W-1 observable (R10 integration smoke, HARD input):** multi-turn task that escalates local→cloud mid-reasoning with a **strict validator** and a step whose receipt **FAILS**. After escalation the cloud lane receives committed entries + receipts, **not** raw history; failed-receipt step is ABSTAINED; generator rewrite of a committed entry is dropped+logged (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:65-68,89`).

**DETERMINISTIC_CHECK** (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:70-77`):

- `rg "grounding_receipt|confidence_bit|commit.*freeze|frozen" belief_state.rs` present
- `rg -c "set_confidence|model.*sets.*bit|self.*confidence" belief_state.rs` **MUST be 0**
- `rg "escalation_waterfall|EscalationCascadeKind" belief_state.rs` threaded
- `production_caller_check.py` 0 DARK
- `workflow/reasoning/J-83-decisions.md` exists

**ACCEPTANCE CV-1..CV-5 + GD-1:** cargo check `--workspace` GREEN; auditor PASS on `-p orchestrator`; I-11/H-50/I-7/D-10 composed not re-implemented (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:80-86`).

Adversarial: condensing without `BeliefStateEvent(Condensed)` is FAIL (ID-3); a `#[cfg(test)]` caller does **not** satisfy reachable_from (R7).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- GRC-v3 is treated as **unification + activation** of existing pieces plus **one** net-new organ (overthinking guard). This file is the **carrier**, not a new reasoning engine.
- An ordered BeliefState of typed entries `{claim, optional external receipt, H-39-read confidence/abstention, DKG+SSB provenance}`.
- The model emits claim text only. It **never** writes the confidence bit or adjudicates commit.
- Per-step gate: commit **iff** D-10-routed domain receipt PASSES; fail/missing/corrupt → ABSTAIN, never chain-say-so.
- Selector: **external receipt** is the only commit signal. Vote is not used here. LLM self-eval is banned.
- Commit-and-freeze: a committed entry is immutable; rewrite dropped+logged.
- Boundary transport: escalate/unload carries **typed entries + receipts**, not raw transcript. Receipts lossless; prose lossy-foldable; **no** LLM re-summarization; **no** ciphers.
- Existing I-11 condense seam / H-50 policy / I-7 waterfall / D-10 router are **composed**, not rewritten.
- Production caller on the GRC Verify path **and** the I-7 escalate path. A unit test is not a production caller.
- Multi-turn local→cloud smoke with a **failed-receipt** negative arm.
- Provenance hash is blake3. No `.unwrap()`/`.expect()` on prod paths.

A prompt wrapper (“summarize your reasoning for the next model”), a raw-history handoff, or the live I-11 `tried`/`failed_because` condensate **without** typed receipt-gated entries is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-authoring I-11, H-50, I-7, D-10, I-14, F-4, D-11, J-1, H-trading validators (GD-1, IC-16) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:62`).
- Model-set confidence / self-grade commit (TD-3, self-correction blind spot) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:13-18`).
- Commit on the chain’s own say-so (L2M poison cascade) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:60`).
- Raw-transcript handoff (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:61`).
- LLM re-summarization compaction (HD-2) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:140-142`).
- Ciphers as compression (IC-3) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:73-74`).
- Majority-vote or MCTS-first as the candidate selector (that is J-86 / IC-6 / DC-3; J-83 is the carrier those candidates sit on) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:136-138`).
- Treating I-11 PARTIAL as “J-83 already done” (DC-1) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:116`).
- Shipping the carrier **without** the I-7 consumer and the multi-turn smoke (R-1 / R10) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:151-155`).
- SHA-2 provenance; `.unwrap()` on prod (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:62`).
- Window size as a quality/confidence signal (RD-4; that confusion is J-84’s to refuse).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md` (full file). Grounding map: `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md` GS-1/GS-2/GS-4/GS-5/GS-8, GA-1, DC-1/DC-2/DC-4, R-1. Index: `J-plan/J-SUMMARY_INDEX.md:1845-1869`. Feature map: `ELAI-FEATURE-MAP.md:642-655`. Context: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:33,46,181-191`. Design: `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` FINAL SYNTHESIS §1/§4/§8 + IMPLICATION 10/11/18/20/31 (as cited by the J-83 plan). Live code: `crates/orchestrator/src/belief_state.rs`, `escalation_waterfall.rs:180-280`, `abstention_score.rs`, `grounding_mode.rs:110`, `context_policy.rs`, `ssb/payloads.rs:17214-17239,17862-17879`. Pack quality bars: `reasoning-architecture/02A-grc-state-machine.md`, `D-10-mode-routing-predicate.md`.

arXiv **[EXTERNAL, WHY not an ELAI number]:** least-to-most 2205.10625; Reflexion 2303.11366 / faithfulness; self-consistency 2203.11171; “Consensus is Not Verification” 2603.06612 (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:110`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:85-96`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact `ReceiptRef` type: pointer to the closing SSB row vs inlined verdict. Plan **prefers** a pointer (GD-2, small carrier) and defers the decision to authoring against F-4 / D-11 / J-1 verdict row shapes (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:113-114`).
- Rust type of `confidence_bit`: spec says “bit” but values are `+1` / `0` / `-t/(1-t)` (an H-39 **scalar**). Whether the Entry stores `f64`, an enum `{Correct, Abstain, ConfidentWrong}`, or a pointer to `AbstentionScored` is unspecified.
- Whether `Entry` is a new struct **inside** `BeliefState` or a parallel collection beside I-11 `tried` / `failed_because` / `constraints` (plan says EXTEND the live seam; live wire has no `entries` field, `belief_state.rs:122-145`).
- Whether `frozen` is a per-entry flag on the wire, or the live process-local `committed_steps()` HashSet (`belief_state.rs:100-105`) is an acceptable freeze ledger. Plan says “frozen flag” (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:42`).
- Precise GRC Verify hook symbol in `crates/reasoning` (plan names the Verify **step**, not a function).
- Exact frontend tile vs justified N/A (W-4 decide-at-edit) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:64`).
- Whether to add `BeliefStateEventKind` arms or reuse Condensed/Handoff only (W-9 decide-at-edit) (`J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:67`).
- Per-entry size / count ceiling beyond I-11 `CONDENSED_PAYLOAD_MAX_BYTES`.
- How `quarantined_claims` (L-8, already on the I-11 wire) interacts with abstained vs committed entries.
- Live `commit_external_receipt` returns `Err(ReceiptRejected)` on fail, while the plan says ABSTAIN (stay open). Fail-closed vs open-abstain on the I-11 cousin is unresolved (`belief_state.rs:246-254` vs `J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md:16`).
- Grounding-map PRODUCTION_HOOK line `escalation_waterfall.rs:239` and `live_cli.rs:50` are stale vs current files (see §3). Rebuild to **symbols**, not those line numbers.
