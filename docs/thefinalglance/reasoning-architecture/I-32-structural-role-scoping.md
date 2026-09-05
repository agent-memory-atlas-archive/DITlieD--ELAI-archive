# Structural role scoping (I-32)

- **ELAI ids:** I-32 / IC-17 / ID-5 / E-19 A/B
- **Spec status (as designed):** PLAN **PLANNED** (`ELAI-FEATURE-MAP.md:871-873`). Live `role_contracts.rs` + `elai role-ab-probe` + `RoleContractApplied` exist — implementation truth, not a map flip.
- **Crate / module (spec):** `crates/orchestrator/src/role_contracts.rs` (`RoleContract`, `RoleContractStore`, `apply_to_system_prompt`); injection at `agent_dispatch::build_system_prompt` (live: `cli/src/report_fns.rs`); A/B CLI.
- **Purpose of this file:** enough architecture to rebuild the persona-strip / persona-keep split. Not a status ledger.

## 1. What it is

**Structural role scoping** replaces **narrative persona** (“you are a X who Y”) on **factual/code** roles with **schema-first role contracts** (typed I/O, explicit constraints). **Alignment-dependent** roles **keep** hostile-critic / lateral framing. The production flip is **evidence-gated**: per-role A/B on E-19 fixtures must show fixture pass-rate **≥ baseline** (ID-5). Wholesale persona removal is **declined** (`I-plan/I-32_structural_role_scoping.md:5-28`, `I-plan/I-SUMMARY_INDEX.md:604-619`).

## 2. Why it exists

Persona prompting **hurts** factual/code (dossier MMLU −3.6..−5.3pp; MT-Bench coding −0.65 — **[DESIGN]**, not a gate) and **helps** alignment-enforcement. If this organ is missing, executors keep a story opener that costs accuracy with no compliance guarantee. If it is faked (strip **auditor** too, or flip without AbDelta ≥ 0), adversarial coverage dies or ID-5 is theater (`:11-28,71-73,80`). IC-17 is the correction to “strip all personas” (`:26-28`).

## 3. How it works (runtime)

1. **Who calls it.** Agent dispatch when assembling the system prompt (`:64,67-68`). A/B harness `elai run --role-ab-probe` (plan) / live `elai role-ab-probe` (`:69`; `role_ab_probe_cmd.rs:1`). The model does not pick strip vs keep.
2. **Input.** Role id + current system-prompt **sections**. Store lookup.
3. **Classify (deterministic, IC-17 table):**
   - **STRIP** (Tier-1): executor, context-compiler, researcher (`:13-15,39`).
   - **KEEP**: auditor, critic, adversary, anti-spiral’s **five lateral** personas (`:15-17,41`).
4. **Apply (deterministic).** STRIP: replace narrative block (`max-tokens=0` on persona; typed constraint list + I/O schema) via `apply_to_system_prompt` (`:19-20,50-51`). KEEP: sections **unchanged**; emit `PersonaRetained` as proof (`:41`). Live: first section containing `"You are"` replaced, else insert contract at 0 (`role_contracts.rs:288-316`).
5. **A/B (deterministic predicates, no live LLM in the live harness).** Old template vs new on E-19 fixtures; emit `RoleContractApplied(AbDelta)` per role under seed (`:20-22,77-79`). Live harness: string `expected_in_prompt` / `must_not_appear`; exit non-zero if `new_rate < old_rate` (`role_ab_probe_cmd.rs:8-16`).
6. **Evidence gate.** Production flip **only if** each affected role’s fixture pass-rate **≥ baseline**. Regression **BLOCKS** (R2) (`:42,73,80`).
7. **Commit.** `RoleContractApplied` Kind `{PersonaStripped, PersonaRetained, AbDelta}` (`:62`). Journal-grep executor outbound prompt: schema contract, **zero** persona narrative (`:80`).
8. **Fail-closed.** No flip without AbDelta ≥ 0. No persona narrative on STRIP roles after flip (`:72-73`).
9. **The model may generate** task output under the contract. It may **not** restore a “you are a senior engineer” opener or waive the A/B gate.

## 4. Types, enums, objects

Copy from `I-plan/I-32_structural_role_scoping.md:50-62` and live structs.

```
RoleContract {
  role: AgentRole,
  typed_constraints: Vec<String>,
  io_schema: String,
  allowed_tool_categories: Vec<String>,  // live; empty = all tools
}
fn prompt_text(&self) -> String          // "# Role: {name}\n\n{io_schema}\n\nConstraints:\n- …"
fn tool_allowed(&self, tool_name: &str) -> bool

RoleContractStore { contracts: Vec<RoleContract> }
  Default = STRIP roles only; KEEP roles ABSENT from the store
fn get(role) -> Option<&RoleContract>    // None => retain persona

fn apply_to_system_prompt(sections: &mut Vec<String>, role, store) -> ApplyOutcome

ApplyOutcome { PersonaStripped, PersonaRetained }

RoleContractAppliedKind { AbDelta, PersonaStripped, PersonaRetained }

RoleContractAppliedPayload {
  kind,
  role: String,                    // snake_case
  fixture_pass_rate_old: f32,      // AbDelta; else 0
  fixture_pass_rate_new: f32,
  delta: f32,                      // new - old; production flip requires >= 0
  ts_ms: i64,
}

// IC-17 split (plan Tier-1)
STRIP: Executor, ContextCompiler, Researcher
KEEP:  Auditor, Critic, Adversary, anti-spiral 5 laterals
// I-21 evaluator persona: KEEP (compose I-21; :95)
```

**Live store extras (fills, not plan Tier-1):** Designer, MemoryWriter also have contracts (`role_contracts.rs:117-201`, payload comment `:16880-16881`). PromptEngineer / IdeaFramer / MediaValidator: **UNCERTAIN** vs plan table — live `get()` None ⇒ retain unless a contract exists.

**Live KEEP named:** Auditor*, IdeaCritic, InjectionClassifier (`:290-304`). Plan **Adversary** and **C-10 five laterals** are not `AgentRole` variants in this file — KEEP by **absence** from the store / by C-10 owning those prompts (`:15-17,90-95`).

**J-106 / HD-3 fill (neighbor):** `OutputChannelClass::{OpEmission, Reasoning}` in the same file (`role_contracts.rs:263-285`). Executor = OpEmission; **do not** use that class to decide persona strip. I-34/J-106 own constrained decode.

## 5. Control flow / state machine

**Owner:** orchestrator/CLI prompt builder. Generator never classifies the role.

```
load sections
  → apply_to_system_prompt
      store.get = Some → replace "You are…" → PersonaStripped
      store.get = None → unchanged → PersonaRetained
  → emit RoleContractApplied
A/B probe:
  → baseline vs contracted prompts on fixtures
  → AbDelta per role
  → if any new < old: exit non-zero; BLOCK production flip
```

**Forced:** no wholesale strip (`:23-24`). Cross-role chain effects deferred to I-21 context (`:46`). Per-commit A/B CI = Tier-2 (`:47`).

## 6. Grounding, receipts, gates

The **receipt** is the journaled system prompt + `RoleContractApplied` rows. Paper pp deltas are **hypotheses** (ID-5); the **gate** is ELAI fixture pass-rate (`:22,86`).

**Never allowed as the flip signal:** aesthetic “looks cleaner”; MMLU paper number as a threshold; LLM self-eval of the new prompt; stripping auditor because “personas are bad.”

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| persona block after STRIP | **max-tokens=0** (`:20`) | per prompt |
| module size (plan) | ~120 LOC (`:50`) | advisory |
| AbDelta | delta **≥ 0** to flip (`:73,80`) | per STRIP role |
| SSB | +1 type `RoleContractApplied` (`:62,88`) | organ |
| A/B seed | observable `--seed 42` (`:78`) | harness |

No GRC call budget here (2C). HD-3: do **not** force-JSON the reasoning roles this organ **keeps**.

## 8. Roles and neighbors

**STRIP vs KEEP is the whole organ.** Phase 2C dispatch configs = substrate, not rewritten (`:31,90-91`). `.claude/agents/*` modified **in place**, not forked (`:32,92`). C-41 skills routing composed (`:33,94`). E-19 probe reused (`:40,65,93`). I-21 evaluator KEEP (`:95`). C-10 laterals KEEP (`:16-17`). J-106 output meta-commentary is the **output-side twin** (`ELAI-FEATURE-MAP.md:706-715`) — cite.

**Must not fork:** a second persona policy in I-34 grammars; stripping C-10 laterals “for consistency.”

## 9. Config / flags / defaults

- Plan CLI: `elai run --role-ab-probe --seed 42` (`:78`).
- Live: `elai role-ab-probe` (`role_ab_probe_cmd.rs:1`, `cli/src/main.rs:1762-1764`).
- Decision doc: `workflow/reasoning/I-32-decisions.md` (`:43`).
- Frontend: **N/A** (W-6 justified); AbDelta in existing governance stream (`:55-56,75-76`).
- No env kill-switch in the plan; live apply runs at prompt build.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `role_contracts.rs::RoleContract` / `RoleContractStore` | `:60-61` |
| `apply_to_system_prompt` | `:51,68` |
| `RoleContractApplied` 3-arm | `:62` |
| `agent_dispatch.rs::build_system_prompt` | `:64` (live `report_fns.rs`) |
| E-19 fixtures | `:65` |
| `elai run --role-ab-probe` | `:69` |
| live `role_ab_probe_cmd.rs` | CLI |
| `i32_role_scoping_contract` | `role_ab_probe_cmd.rs:24` |
| `.claude/agents/*` | `:32,92` |

## 11. Journal / SSB events

**+RoleContractApplied** Kind `{PersonaStripped, PersonaRetained, AbDelta}` (`:62,88`). Payload §4 (`ssb/payloads.rs:16875-16908`). AbDelta carries old/new/delta. GD-2: recomputable under `ELAI_SEED` (`:79`).

## 12. Tests and observables

Named (`:71-80`, `I-SUMMARY_INDEX.md:616-619`):

| Id | Assert |
|---|---|
| STRIP grep | executor outbound system prompt has schema contract, **zero** persona narrative |
| PersonaStripped | executor / context-compiler / researcher rows |
| PersonaRetained | auditor / critic / adversary / anti-spiral laterals |
| AbDelta | per role, seed-stable; delta ≥ 0 or flip BLOCKED |
| Auditor still critic | retained framing present in journaled prompt |
| R2 | any role fixture regression **blocks** production flip |
| live store | Auditor `get()` is None; Executor has contract (`role_contracts.rs` unit tests) |

Adversarial: strip auditor; flip with negative delta; “You are” surviving in executor; using MMLU pp as the gate.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Closed IC-17 split: factual/code **schema contracts**; alignment roles **keep** persona.
- Apply is **host-side** (replace/omit the narrative block), not a polite system-prompt request.
- A/B on fixtures **before** production flip; **delta ≥ 0** or **no flip**.
- `RoleContractApplied` three kinds land (strip / retain / AbDelta).
- Journal-grep proves executor has **no** “you are a …” opener after flip.
- Wholesale persona deletion is **not** the design.

A “be concise” slogan, or stripping every role including auditor, is a **cousin**.

## 14. Non-goals / banned designs

- Strip-all personas (`:23-24,26-28`).
- Flip without AbDelta (`:73`).
- Reimplementing dispatch / E-19 loaders / C-41 (`:90-94`).
- Forking `.claude/agents/*` into a parallel tree (`:92`).
- Dashboard (`:75-76`).
- Cross-role interaction measurement in v1 (`:46`).
- Paper MMLU numbers as thresholds (ID-5) (`:86`).
- Constrained-decoding the **kept** reasoning roles (HD-3 / I-34 neighbor).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-32_structural_role_scoping.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:604-619`. Feature map: `ELAI-FEATURE-MAP.md:871-873`. Dossier (design numbers): `docs/prefableresearch/Research_Persona-Prompting.*` as cited `:84`. Live: `role_contracts.rs`, `ssb/payloads.rs:16875-16908`, `cli/src/role_ab_probe_cmd.rs`, `cli/src/report_fns.rs:1121-1261`. Neighbors: I-21, C-10, J-106, Phase 2C.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Plan STRIP set is three roles; live also Designer + MemoryWriter (`:39` vs `role_contracts.rs:117-201`).
- Plan KEEP **Adversary** / **five laterals** vs live AgentRole enum (no Adversary variant in `role_display_name`).
- `agent_dispatch.rs` vs live `report_fns.rs` injection site (`:52-53,64`).
- `--role-ab-probe` on `elai run` vs standalone `elai role-ab-probe` (`:69,78`).
- `max-tokens=0` on the persona block: specified (`:20`), not a field on `RoleContract`.
- Whether A/B uses live E-19 LLM probe or string fixtures (plan E-19; live no LLM — `role_ab_probe_cmd.rs:4`).
- PromptEngineer / IdeaFramer / MediaValidator strip vs keep (absent from plan table; live classification mixed).
- Exact anti-spiral five persona names (C-10 owns them; not listed here).
- SCHEMA_VERSION for RoleContractApplied (plan baseline+1 from 221/532; re-read at edit) (`:62,88`).
