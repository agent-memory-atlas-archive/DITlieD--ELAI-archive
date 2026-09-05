# Mechanical decision governance (I-33)

- **ELAI ids:** I-33 / IN-11 / ID-5 / E-25 `prompt_decision`
- **Spec status (as designed):** PLAN **PLANNED** (`ELAI-FEATURE-MAP.md:866-868`). Live `decision_governance.rs` + `elai` governance-probe / `bench_governance.rs` exist — implementation truth. Kill-switch `ELAI_GOVERNANCE_DISABLE` (`wire-dark-allowlist.toml:1716-1725`).
- **Crate / module (spec):** `crates/orchestrator/src/decision_governance.rs` (`CeflRequest`, `I6qValidator`, `CommitRevealRecord`, `governed_prompt_decision`); wrapped around E-25 `prompt_decision`.
- **Purpose of this file:** enough architecture to rebuild CEFL + I6Q + commit-reveal on **decision-type** LLM calls. Not a status ledger.

## 1. What it is

**Mechanical decision governance** wraps orchestrator **decision** calls (mode picks, gate advisories, classification) with three host primitives: **CEFL** (orchestrator authors the candidate set; the model only ranks it — invented options are **unparseable**), **I6Q** (deterministic min-length + lexical-diversity check; vacuous rationales **rejected**), **commit-reveal** (`blake3(seed || choice_index)` **before** reveal; SSB order is the proof) (`I-plan/I-33_mechanical_decision_governance.md:5-29`, `I-plan/I-SUMMARY_INDEX.md:621-639`). Metrics **CDL / DIU / FSR** are ledgered under `ELAI_SEED` (GD-2). Santander decoupling is a **design target**, measured locally (ID-5) (`:31-33`).

## 2. Why it exists

Verbal “please pick fairly” does not stop framing or “this is best because it is best.” If this organ is missing, the model can invent a candidate it prefers, emit a vacuous rationale, or exploit option order. If it is faked (prompt-only CEFL, model-settable I6Q thresholds, reveal-before-commit), selection-by-negation and ordering attacks remain (`:5-8,27-29,80-84`).

## 3. How it works (runtime)

1. **Who calls it.** Every **decision-type** `prompt_decision` site (gate advisory, mode-pick, classification) (`:47,58-60,77-78`). Production entry: `governed_prompt_decision` (`decision_governance.rs:24,335-350`). Not GRC / search CoT (HD-3: those stay unconstrained; I-34).
2. **CEFL (deterministic host).** Orchestrator builds `CeflRequest` from an exhaustive candidate list. Schema domain = that set. Model scores/ranks **only**. A model-invented label **fails parse** (`:13-16,81`). Live: JSON Schema `{type:string, enum:[...]}` + prompt framing; empty list → `None` (`decision_governance.rs:93-135,144-157`).
3. **Commit (deterministic, stochastic gates).** Before the model sees the prompt, write commit row with `blake3(seed || choice_index)` (`:23-25`). Live hash: `blake3(seed.to_le_bytes() || choice_index.to_le_bytes())` → `[u8;32]` (`:259-270`).
4. **Model call.** E-25 `prompt_decision` (compose, do not reimplement) (`:72,101`). Optional CEFL `decoding_config` attached (`:169-173`).
5. **I6Q (deterministic).** If a rationale is present: min length + unique-word count. Fail → `GovernancePrimitiveFired(I6qRejected)` and route **D-16 refusal / Abstain** (`:18-21,74`; live `RefusalDecision::Abstain` comment `:15-16`). Thresholds **orchestrator-owned**, not model-settable (`:21`). Live defaults: `min_len=20`, `min_unique_words=5`; length first (`:192-220`).
6. **Reveal.** Write reveal row; `verify_order(commit_row_id < reveal_row_id)`. Out-of-order = **R2 FAIL** (`:25,82,88-90`).
7. **Metrics.** CDL (cosmetic-deadlock rate), DIU (deferral information utilisation), FSR (framing-success rate) from SSB under seed (`:31-33,84`). Framing-inversion fixture: **same decision** despite reversed field order; FSR **~0** on the fixture set (`:90`).
8. **Commit artifacts.** `GovernancePrimitiveFired` Kind `{CeflExternalized, I6qRejected, CommitRevealSealed}` (`:48,70`). Candidate list in Cefl payload (`:89`).
9. **Fail-closed.** Schema that parses an invented option = **broken CEFL** (`:81`). Reveal-before-commit = broken (`:82`).
10. **The model may generate** a rank **inside** the enum and a rationale **for I6Q**. It may **not** extend the candidate set, set I6Q thresholds, or see the choice before commit.

## 4. Types, enums, objects

Copy from `I-plan/I-33_mechanical_decision_governance.md:44-48,66-70` and live module.

```
CeflRequest { candidates: Vec<String>, schema_json: String }
fn new(candidates) -> Option<Self>     // None if empty
fn apply(&self, prompt: &str) -> String
fn schema_json(&self) -> &str
fn decoding_config(&self) -> ConstrainedDecodingConfig
fn candidates(&self) -> &[String]

I6qValidator { min_len: usize, min_unique_words: usize }
  Default { 20, 5 }                    // live fill; plan: orchestrator config
fn validate(rationale) -> Result<(), I6qRejection>
I6qRejection { TooShort { actual, required }, TooVacuous { actual, required } }

CommitRevealRecord { seed: u64, choice_index: usize }
fn commit_hash(&self) -> [u8; 32]      // blake3 only
fn verify_order(commit_row_id, reveal_row_id) -> Result<(), CommitOrderError>
CommitOrderError { OutOfOrder { commit_row_id, reveal_row_id } }

fn governed_prompt_decision(...)       // live production wrapper
fn emit_governance_fired(ssb, kind, session_id, details_json, clock)

GovernancePrimitiveFiredKind {
  CeflExternalized, I6qRejected, CommitRevealSealed
}
GovernancePrimitiveFiredPayload {
  kind,
  session_id: Option<String>,
  details_json: String,                // live: must not exceed 4 KiB
  ts_ms: i64,
}

GovernanceError { PromptHandler, I6qRejected, SsbWrite }

// Metrics (named, not a struct in the plan)
CDL  cosmetic-deadlock rate
DIU  deferral information utilisation
FSR  framing-success rate              // ~0 on inversion fixture = framing failed to change the decision
```

Plan hash: `blake3(seed || choice_index)` (`:24`). Live encodes both as LE bytes (`:267-270`).

## 5. Control flow / state machine

**Owner:** orchestrator `governed_prompt_decision`. Model only ranks inside CEFL.

```
build CeflRequest (host)
  → emit CeflExternalized (candidates in details_json)
  → CommitRevealRecord.commit_hash → SSB commit row  (stochastic gates)
  → prompt_decision (E-25)
  → I6qValidator.validate? else I6qRejected → D-16 Abstain
  → SSB reveal row
  → verify_order (commit_id < reveal_id) else R2 FAIL
  → ledger CDL/DIU/FSR
```

**Forced:** model-authored candidates excluded (`:27-28`). Reveal-before-commit excluded (`:28-29`). Adaptive I6Q threshold = Tier-2 (`:55`). Cross-gate DIU trends need ≥3 real runs (`:54`).

**Live opt-in:** module always compiled; primitives fire when caller passes `Some(cefl)` / `Some(i6q)` (`decision_governance.rs:26-28`). Kill-switch `ELAI_GOVERNANCE_DISABLE` bypasses in emergency (`wire-dark-allowlist.toml:1718-1719`). Rebuild: **default ON** for decision sites; disable is a kill-switch, not an experiment flag.

## 6. Grounding, receipts, gates

CEFL **is** the schema receipt (candidate list journaled). I6Q is a **deterministic** checker, not an LLM judge. Commit-reveal receipt is **row order** + blake3.

**Never allowed as a decision signal:** model-invented options; vacuous rationale; reveal-before-commit; model-set I6Q thresholds; fabricated CDL/DIU/FSR (`:80-84`); paper MCC 0.43→0.88 as a gate (ID-5) (`:93-95`).

Phase 3F routes `GovernancePrimitiveFired` as verdict plumbing (`:60,73`). C-26 `DecodingMode` hosts the CEFL schema (`:75,104`). **HD-3:** this is **decision/op-emission**, not a reasoning-role CoT wrap. Do not JSON-schema GRC Observe.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| I6Q min_len | live **20** chars | per rationale |
| I6Q unique words | live **5** | per rationale |
| `details_json` | live **≤ 4 KiB** | per event |
| module size (plan) | ~200 LOC (`:58`) | advisory |
| FSR target | **~0** on inversion fixture (`:90`) | measured, not a paper constant |
| hash | blake3 only (`:83`) | commit |

**Overflow:** empty CEFL set → no request (`new` → None). I6Q fail → Abstain, not a silent yes.

## 8. Roles and neighbors

Applies to **decision-type** calls, not executor CoT. E-25 `prompt_decision` is the **only** LLM machinery (`:72,101`). B-22 / H-37 are named call sites (`:39`). D-16 is the I6Q rejection target (`:74,103`). I-34 supplies grammar pinning for the CEFL schema (neighbor). I-32 does not replace CEFL (persona ≠ candidate set).

**Must not fork:** a second `prompt_decision`; prompt-only “choose from this list” without schema unparseability.

## 9. Config / flags / defaults

- I6Q thresholds: orchestrator config (`:21`); live struct fields, Default 20/5.
- Production seed for commit-reveal: **PENDING** (U-2, D-4) (`decision_governance.rs:252-253`, allowlist `:1716-1717`). Bench uses `--seed` (default 42) (`:88`; `bench_governance.rs:22`).
- Kill-switch: `ELAI_GOVERNANCE_DISABLE` (`wire-dark-allowlist.toml:1720`).
- Decision doc: `workflow/reasoning/I-33-decisions.md` (`:51`).
- Frontend: CDL/DIU/FSR tile in MonitorWindow / SettingsWindow — **W-6 mandatory** (`:49,62-63,85-86`). `cross_lang_diff.py` PASS. Live pair: `dashboard-v2/src-tauri/src/commands_i33.rs` (`decision_governance.rs:30`).
- CLI: `elai run --governance-probe --seed 42` (`:88`); live `bench_governance.rs` / main.rs `:1710`.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `decision_governance.rs::CeflRequest` | `:67` |
| `I6qValidator` | `:68` |
| `CommitRevealRecord` | `:69` |
| `GovernancePrimitiveFired` 3-arm | `:70` |
| E-25 `prompt_decision` | `:72` |
| 3F verdict plumbing | `:73` |
| D-16 refusal | `:74` |
| C-26 scoring schema | `:75` |
| live `governed_prompt_decision` | `decision_governance.rs:335` |
| `bench_governance.rs` | CLI probe |
| `i33_governance_contract.rs` | allowlist smoke |
| `commands_i33.rs` | dashboard pair |

## 11. Journal / SSB events

**+GovernancePrimitiveFired** Kind `{CeflExternalized, I6qRejected, CommitRevealSealed}` (`:70,96`). Payload §4 (`ssb/payloads.rs:16843-16869`). Observable: CeflExternalized **per decision** with candidate list; I6qRejected on vacuous fixture; CommitRevealSealed with `commit_row_id < reveal_row_id` (`:89`).

## 12. Tests and observables

Named (`:50,80-90`, `I-SUMMARY_INDEX.md:635-639`):

| Id | Assert |
|---|---|
| CEFL | journaled request shows **host** candidate list; invented option unparseable |
| I6Q | vacuous rationale → I6qRejected |
| framing-inversion | reversed field order → **same** decision; FSR ~0 measured |
| commit-reveal | commit row id **<** reveal id; out-of-order fixture **nonzero exit** |
| GD-2 | metrics recomputable under seed |
| tile | CDL/DIU/FSR render; cross_lang_diff PASS |
| live probe | three arms emit three kinds (`bench_governance.rs:9-16`) |

Adversarial: schema that accepts extra enums; I6Q thresholds in the model prompt; reveal row written first; fabricated FSR.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Decision calls go through a host wrapper that **externalizes** the candidate enum (CEFL). Invented options **cannot parse**.
- Vacuous rationales fail a **deterministic** I6Q checker (min length + lexical diversity); rejection is journaled and routes to refusal/abstain.
- Stochastic choices **commit blake3 before reveal**; SSB order proves it; reverse order **fails**.
- `GovernancePrimitiveFired` three kinds exist.
- Framing inversion does not change the decision (measured FSR).
- No LLM inside I6Q / commit-reveal / candidate authorship.

A prompt “pick from A/B/C,” or JSON-schema on a **reasoning** role, is a **cousin**.

## 14. Non-goals / banned designs

- Model-authored candidate sets (`:27-28,81`).
- Reveal-before-commit (`:28-29,82`).
- Reimplementing E-25 / 3F / D-16 / C-26 (`:100-104`).
- Adaptive I6Q in v1 (`:55`).
- Fabricated CDL/DIU/FSR (`:84`).
- Paper MCC as a gate (`:93-95`).
- blake2/sha2 (`:83`).
- Applying CEFL JSON-schema to GRC/reasoning CoT (HD-3 / I-34).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-33_mechanical_decision_governance.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:621-639`. Feature map: `ELAI-FEATURE-MAP.md:866-868`. Dossier (design): `docs/prefableresearch/Research_NaturalLanguage-vs-Deterministic-Constraints.*` (`:93`). Live: `decision_governance.rs`, `ssb/payloads.rs:16843-16869`, `cli/src/bench_governance.rs`, `wire-dark-allowlist.toml:1716-1725`. Neighbors: E-25, D-16, C-26, I-34, Phase 3F.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric I6Q thresholds not in the plan (live 20 / 5) (`:18-21` vs Default).
- CEFL `apply` text forbids explanation (`decision_governance.rs:153-154`) while I6Q **requires** a rationale — one-call vs two-phase unspecified.
- Production commit **seed** source PENDING (`:252-253`).
- When `choice_index` is known at **commit** time if the model has not chosen yet (plan: commit before reveal; live struct includes `choice_index` in the pre-image — may bind a host-sampled index, not the model’s pick). **UNCERTAIN** binding.
- CDL / DIU formulas unnamed.
- `elai run --governance-probe` vs live `bench_governance` CLI name (`:88`).
- Whether `governed_prompt_decision` is the **only** `prompt_decision` caller (module claims it `:337`; not in the plan).
- `details_json` 4 KiB cap live-only.
- SCHEMA 240 in allowlist vs plan baseline+1 from 221/532 (`:70` vs `:1716`).
- Kill-switch vs plan “always wrap every decision call” (`:47` vs `ELAI_GOVERNANCE_DISABLE`).
