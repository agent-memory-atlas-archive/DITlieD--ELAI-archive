# Cross-family advisory calibration (J-45)

- **ELAI ids:** J-45 / DP-3 / EXP-DP3 / WAVE DPW-3 / D5 / C3 / JD-1 / Track J DOCTRINE-PROPAGATION
- **Spec status (as designed):** PLANNED BUILD. Grep-confirmed absent: no `doctrine-calibration` corpus, no kappa tool, no `cross_family_advisory` module (`J-plan/J-45_cross_family_advisory_calibration.md:1-4`, `J-plan/J-SUMMARY_INDEX.md:1199-1219`). Depends J-43, J-44, gpt-bridge MCP.
- **Crate / module (spec):** **harness leg**, net **0** SSB. Corpus `data/doctrine-calibration/*.jsonl`; kappa measurement (off-agent); gpt-bridge `gpt_chat` / `grok_chat` dispatch; escalation via `.claude/rules/handoff-protocol.md` + orchestrate handoff-mediation; degradation rows in `doctrine-debt-ledger.toml` (J-43). **Not** `crates/orchestrator/src/adversary.rs` (that is D-13).
- **Purpose of this file:** enough architecture to rebuild the calibrated D5 advisory — corpus, kappa gate, cross-family vote, escalation teeth. Not a status ledger. **Does not replace D-13.**

## 1. What it is

The **cross-family advisory calibration** is DP-3, the D5 **semantic leg**: for a judgement with **no mechanical oracle** (is this decision the model’s or the harness’s; is this a proxy or the objective; is this ATTACK section real or boilerplate), an LLM may **contribute** but is **never** the load-bearing decision. It is **one advisory vote**, off the critical path, and the critic is always a **different pretraining family** than the producer (`docs/strategy/ELAI-DETERMINISM-DOCTRINE.md:164-170`, `J-plan/J-45_cross_family_advisory_calibration.md:4`, `J-plan/J-SUMMARY_INDEX.md:1199-1210`).

C3 (deep-research vote) is blunt: an advisory leg is **decoration — routinely ignored** — unless (a) it is **calibrated** against **100–300** human-labeled traces to **Cohen kappa > 0.6** (uncalibrated = decoration; **kappa < 0.5 = unusable**), and (b) it has **human-escalation teeth** (flagged cases become an orchestrator/user **decision**, never a dashboard tile) (`J-plan/J-45_cross_family_advisory_calibration.md:4,6-10`). Until kappa > 0.6 on a **held-out** slice the leg is **admittedly decorative** and must be **labelled** so — it does **not** count as the satisfied D5 check (`:4,7`).

**D-13 is a different organ.** D-13 is **Mode B runtime grounding**: `AgentRole::Adversary` on a different family, structured `CounterEvidence`, auditor decides, `ELAI_ADVERSARY_ENABLE` (`D-13-adversary-role-cross-family.md` §1). J-45 **calibrates** the **doctrine-conformance advisory** that J-43’s floor and J-44’s panel **route unmechanizable findings to**. It **generalizes** D-13 / VD-4 / TD-3; it does **not** spawn Mode B, does **not** sit on `select_grounding_mode`, and does **not** replace D-13 (`J-plan/J-45_cross_family_advisory_calibration.md:42`, `D-13-adversary-role-cross-family.md` §1).

## 2. Why it exists

J-43’s floor catches **structural** anti-patterns (DD-1..DD-5). The semantic core (boundary classification, proxy detection, ATTACK substance) has **no mechanical oracle**; putting an LLM verdict on that gate would violate D5, and claiming the floor covers it would violate D3 (`J-plan/J-43_doctrine_propagation_keystone.md:4,17`, `:60`). J-44’s shape-check is **necessary-not-sufficient**; flagged substance routes **here** (`J-44-reasoning-method-artifact-shape.md` §3 step 5, `J-plan/J-44_reasoning_method_artifact_shape.md:48,59`).

If this organ is missing, DD-4 WARN and J-44 boilerplate flags die in a dashboard. If it is faked (same-family “second opinion”, uncalibrated advice cited as “D5 satisfied”, or an LLM boolean on a release gate), it is the failure C3 measured: same-family judges show **10–25% self-preference inflation**; uncalibrated advisory is ignored-by-construction (`J-plan/J-45_cross_family_advisory_calibration.md:12,33-34`).

Honest residue: the corpus is a **real recurring human cost**; gpt-bridge is **rate-limit-fragile**; the harness-domain composition remains an **unproven bet** that DP-5 (J-47) A/B/C resolves (`:22,46-50`, `J-plan/J-SUMMARY_INDEX.md:1215-1216`).

## 3. How it works (runtime)

1. **Who calls it.** Harness, not the GRC Verify loop and not D-13 `probe`:
   - J-43 floor **WARN** on DD-4 (LLM-verdict-on-gating-path) and other **unprovable** findings (`J-plan/J-45_cross_family_advisory_calibration.md:27,38`, `J-plan/J-43_doctrine_propagation_keystone.md:13,52,60`).
   - J-44 substance-audit panel routes boilerplate / ATTACK-theater judgements here (`J-plan/J-44_reasoning_method_artifact_shape.md:48`, `J-44-reasoning-method-artifact-shape.md` §3 step 5).
   - Design-time: oracle skill as a host for a doctrine diversity-vote prompt (DG-15) (`J-plan/J-45_cross_family_advisory_calibration.md:19`).
   A `#[cfg(test)]` or a unit mock of gpt-bridge does **not** satisfy JD-1.
2. **EXP-DP3 (observe-only first, the gate on the whole leg).** Build the corpus. Measure Cohen kappa between advisory verdicts and **human** labels on a **held-out** slice. Advisory is **non-load-bearing**, labelled **“uncalibrated”**, until **kappa > 0.6**. **kappa < 0.5 → unusable**, redesign prompt/lens. Nothing in DP-1/DP-2 may **cite** the advisory as the satisfied D5 check before this passes (`:6-7`).
3. **Calibration corpus.** **100–300** human-labeled doctrine-conformance examples: a feature/design/reasoning-artifact + ground-truth `{ doctrine-clean | named-violation(sub-doctrine) | debt }`. Drawn from DP-4 (J-46) baseline-sweep real findings + seeded synthetic violations (one per DD-* class + one boilerplate-ATTACK). Held-out un-gamed subset reserved for kappa. Recurring human cost, explicitly budgeted (`:8`). Lean: user labels the held-out slice (`:47`).
4. **Cross-family review (D5 / D-13 discipline, different grain).** Producer = session model. Advisory runs via gpt-bridge MCP: `gpt_chat` (second family) / `grok_chat` (third; effort `auto|fast|expert|heavy` — **no** grok deep-research; use expert/heavy) (`:9,16`, `CLAUDE.md:39`). Input: serialized **minimal snippet** + doctrine lens (J-43) + **DECISION-LEDGER** (J-44). **No repo access** (gpt-bridge constraint). Output: **findings**, never a release-gating boolean (`:9,12`). Same-family dispatch is **REFUSED** (DD-3 inverse at review grain) (`:9,12,28`).
5. **Escalation teeth (C3).** A flagged finding routes to the orchestrator/user as an explicit **DECISION** — handoff-protocol `BLOCKED=handoff-entry` shape, or orchestrate/auto-orchestrate **handoff-mediation** — with weighting/visibility. **Not** a dashboard tile that decays unread (`:10,18,29`).
6. **Degradation honesty.** gpt-bridge is occasional, `POOL_EXHAUSTED` possible. If the second family is unreachable, the review degrades to **single-family**, which **weakens D5**, so it records a **DOCTRINE-DEBT** row with `current_disposer = "single-family-degraded"`. **Never** pass single-family advice off as cross-family (`:11,12,28`). Default on prolonged outage: **proceed + debt** (advisory is non-load-bearing); BLOCK is over-strict — decide at edit (`:48`).
7. **Commit artifact (off-agent, JD-1).** kappa recorded on the held-out slice; flagged finding **grep-provable** in the handoff/run log; same-family negative arm REFUSED; degraded arm emits the debt row (`:29`). **Net 0 SSB.** The advisory **never** journals a verdict on a critical path (`:30`).
8. **Fail-closed / labelled decorative.** Uncalibrated ≠ D5 satisfied (`:12`). LLM-as-judge on a release-gating path banned (`:12`). Deep-research is **not** the advisory (`:12`).

The model (producer) may generate the artifact under review. The advisory model may generate **findings**. Neither may gate a release, pick D-13 Mode B, or mark D5 satisfied before kappa.

## 4. Types, enums, objects

Copy from the J-45 wiring contract and J-43 ledger row. Do not invent fields.

```
// Ground-truth / advisory verdict labels (corpus)
DoctrineConformanceLabel {
  DoctrineClean,
  NamedViolation { sub_doctrine },   // D1..D5 / DD-* as named
  Debt,
}

// Corpus record (path named; exact JSON schema UNCERTAIN §16)
data/doctrine-calibration/*.jsonl
  { artifact_or_snippet, human_label: DoctrineConformanceLabel, split: train|held_out, ... }

// Kappa gate (C3 prescription, PROVISIONAL on this domain until EXP-DP3)
kappa > 0.6     → may be cited as calibrated D5 advisory (still non-gating)
uncalibrated    → labelled decorative; NOT the D5 check
kappa < 0.5     → unusable; redesign prompt/lens
N_corpus        ∈ 100..=300 human-labeled traces

// J-43 ledger row J-45 writes on degradation (compose, do not fork)
DOCTRINE-DEBT {
  decision_id, feature, sub_doctrine,  // D1..D5
  why_unmechanized,
  current_disposer,                    // "single-family-degraded" | advisory-cross-family | none | manual
  owner, expiry_date, promotion_plan_id
}
// J-43_doctrine_propagation_keystone.md:15

// Escalation entry (host loop, not a new SSB type)
handoff-protocol BLOCKED = handoff-entry
OR orchestrate/auto-orchestrate handoff-mediation entry

// gpt-bridge carrier (capability, not a crate type)
gpt_chat, grok_chat { effort: auto|fast|expert|heavy }
// NO repo access; serialize snippet in
```

(`J-plan/J-45_cross_family_advisory_calibration.md:8-11,16,25-30`, `J-plan/J-43_doctrine_propagation_keystone.md:15`)

**Not this organ (D-13 types — cite only):** `AgentRole::Adversary`, `CounterEvidence`, `AdversaryVerdict { ClaimRobust, ClaimFalse, ClaimWeak }`, `CrossFamilyUnavailable`, `SameFamily` (`D-13-adversary-role-cross-family.md` §4). Rebuilding Mode B inside J-45 is a **cousin** and a **fork**.

## 5. Control flow / state machine

**Owner of the loop:** harness / auditor (≠ author, ≠ executor). No GRC step. No LLM inside a release predicate.

```
J-43 floor WARN (DD-4 / unprovable)
  or J-44 substance-audit flag
     │
     ▼
same-family?  → REFUSE (DD-3)
     │
     ▼
second family reachable? (gpt_chat / grok_chat)
     ├─ NO  → DOCTRINE-DEBT current_disposer=single-family-degraded
     │        proceed (default) ; never label "cross-family pass"
     │
     ▼ YES
serialize snippet + doctrine lens + DECISION-LEDGER
     │
     ▼
advisory findings (never a gating boolean)
     │
     ├─ EXP-DP3 kappa on held-out
     │     < 0.5  → unusable (redesign)
     │     < 0.6  → labelled uncalibrated / decorative
     │     > 0.6  → may be cited as calibrated D5 advisory
     │              STILL not a release gate
     │
     └─ flagged finding
              route to orchestrator/user DECISION (handoff)
              NOT a dashboard tile
```

**Forced transitions:**

- Same-family dispatch **cannot** count as D5 (`J-plan/J-45_cross_family_advisory_calibration.md:9,12`).
- Uncalibrated advisory **cannot** be cited by J-43/J-44 as the satisfied D5 check (`:7`).
- Advisory **cannot** sit on a release-gating path (`:12,30`).
- D-13 Mode B **cannot** be deleted or bypassed because J-45 exists (this file does not replace D-13).

**C-10 / H-39 / I-7 / I-5 / I-8 / I-11 / J-83:** not this organ. N-15 later permutes presentation across **J-45 samples** for framing-invariance (`ELAI-FEATURE-MAP.md:3372-3379`) — that is a **consumer**, not a replacement.

## 6. Grounding, receipts, gates

J-45 **is** the calibrated advisory. It is **never** the mechanical oracle.

| Rank | Signal | Legal here? |
|---|---|---|
| 1 | Mechanical floor (J-43 DD-*) / shape-check (J-44) | **gates** (those organs, not J-45) |
| 2 | Human-labeled held-out kappa | **the only “calibrated” signal** |
| 3 | Cross-family advisory finding + handoff decision | **advisory, off critical path** |
| 4 | Same-family critique / LLM release boolean | **BANNED** (D5, DD-3) |
| — | D-13 Mode B `CounterEvidence` | **D-13 organ**; J-45 does not consume or replace it |

**Never allowed as a confidence / D5-satisfied signal:** uncalibrated gpt-bridge one-shot; dashboard-only flag (`:12`); deep-research run as the advisory (`:12`); producer self-critique; quoting C3’s 10–25% inflation as ELAI’s measured kappa (ID-5: EXP-DP3 measures the real kappa, `:33-34`).

**D5 statement (doctrine, not a paper):** LLM in a gate is advisory **and** cross-family; never grounds a claim; never on the critical path (`ELAI-DETERMINISM-DOCTRINE.md:164-170`). Named in-repo instances J-45 **generalizes**: TD-3, **D-13 ADVERSARY**, VD-4, D-10 `select_grounding_mode` (`:172-180`). Generalize ≠ replace.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| corpus size | **100–300** human-labeled traces | EXP-DP3 (`J-plan/J-45_cross_family_advisory_calibration.md:8,33`) |
| kappa calibrated | **> 0.6** | held-out slice (`:7`) |
| kappa unusable | **< 0.5** | held-out slice (`:7`) |
| gpt-bridge | occasional, not routine; one vote; no repo | per review (`:16`, `CLAUDE.md:39`) |
| grok effort | `auto\|fast\|expert\|heavy` (no deep-research) | per `grok_chat` (`:12,16`) |
| SSB | **net 0** | harness leg (`:30`) |
| J-43 optional | +1 `DoctrineDebtEvent` only if runtime stream wanted; **default net 0** | `J-plan/J-43_doctrine_propagation_keystone.md:38` |
| kappa freshness | not one-time; stale kappa treated as debt | open question (`J-plan/J-45_cross_family_advisory_calibration.md:49`) |
| second health signal | escalation **ACCEPTANCE / override** rate | DP-4 (`:50`) |

**Overflow / scarcity:** gpt-bridge account pool / rate-limit → degrade + debt, do not silently skip the label (`:11`). Corpus labelling cost is budgeted human time, not a token cap (`:8,47`).

**Do not confuse with:** D-13 `ELAI_ADVERSARY_ENABLE` runtime probe; E-19 probe kinds; I-7 waterfall (lane change, not doctrine review).

## 8. Roles and neighbors

**Auditor / harness** applies the advisory. **Executor never self-applies** it as a pass (J-43 auditor-applied rule, `J-plan/J-43_doctrine_propagation_keystone.md:8`). Producer = session model; critic = gpt-bridge family.

**Consumes:** J-43 doctrine lens + `doctrine-debt-ledger.toml`; J-44 DECISION-LEDGER + substance flags; gpt-bridge `gpt_chat`/`grok_chat`; handoff-protocol / orchestrate mediation; J-46 sweep as corpus seed (`J-plan/J-45_cross_family_advisory_calibration.md:26,37-44`).

**Produces:** calibration corpus + kappa record; advisory findings; handoff **DECISION** entries; `single-family-degraded` debt rows.

**Must not fork:** a second D-13 adversary runtime; a release-gating LLM; a dashboard-only surface; Approach A comprehensive `doctrine_conformance_check.py` (already REJECTED as primary in J-43, `J-plan/J-43_doctrine_propagation_keystone.md:17`).

**Cite, do not re-author:**

- **D-13** — Mode B claim adversary. J-45 does **not** replace it (`D-13-adversary-role-cross-family.md`).
- **J-44** — method/shape + panel that **routes here** (`J-44-reasoning-method-artifact-shape.md`).
- **J-43** — floor; DD-4 stays WARN and leans on this advisory (`J-plan/J-43_doctrine_propagation_keystone.md:60`).
- **H-39 / C-10 / I-5 / I-7 / I-8 / I-11 / J-83** — runtime reasoning spine. J-45 is doctrine-propagation, not GRC-v3.

## 9. Config / flags / defaults

- No `ELAI_*_ENABLE` for a gating path (D5: no new gate on a critical path) (`J-plan/J-45_cross_family_advisory_calibration.md:22,30`).
- EXP-DP3 observe-only until kappa measured (`:6-7`).
- Degradation default: **proceed + debt** if gpt-bridge down (`:48`).
- Frontend: **no** dashboard-only surface (`:12`). Escalation is the handoff loop already run (`:10`).
- Oracle skill is a legitimate **design-time** host (DG-15) (`:19`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `data/doctrine-calibration/*.jsonl` | corpus (`J-plan/J-45_cross_family_advisory_calibration.md:25`) |
| kappa measurement (off-agent tool; name unspecified) | EXP-DP3 (`:6-7,25`) |
| gpt-bridge `gpt_chat` / `grok_chat` | carrier (`:16,26`) |
| J-43 lens + `doctrine-debt-ledger.toml` | consume (`:26`) |
| J-44 DECISION-LEDGER + panel | consume (`:26`, `J-44-reasoning-method-artifact-shape.md` §3) |
| `.claude/rules/handoff-protocol.md` + orchestrate handoff-mediation | escalation (`:18,26`) |
| `.claude/skills/oracle/SKILL.md` | optional design-time host (`:19`) |
| `crates/orchestrator/src/adversary.rs` | **D-13 — DO NOT implement J-45 here** |

No production `task_runner` hook is named. Reachable_from is J-43 WARN / J-44 flag → advisory → human decision (`:27`).

## 11. Journal / SSB events

**Net 0 new action_type.** Calibration corpus, kappa record, and escalation entries are **off-agent files / log rows** (JD-1). The advisory **never** journals a verdict on a critical path (`J-plan/J-45_cross_family_advisory_calibration.md:30`).

J-43 optional `DoctrineDebtEvent` (FloorFinding / DebtRegistered / DebtPromoted) is **not** the default and is **not** a J-45 gating row (`J-plan/J-43_doctrine_propagation_keystone.md:38`). If a degraded review writes a ledger row, that is **J-43’s file**, not a new SSB type.

## 12. Tests and observables

JD-1 off-agent (`J-plan/J-45_cross_family_advisory_calibration.md:22,29`, `J-plan/J-SUMMARY_INDEX.md:1217-1219`):

| Id | Assert |
|---|---|
| kappa | seeded doctrine-violation snippet → finding agrees with human label at the **measured** kappa (recorded) |
| teeth | flagged finding → real orchestrator/user **DECISION** entry, **grep-provable** in handoff/run log, not a dashboard write |
| same-family | dispatch **REFUSED** |
| degrade | second family unreachable → `single-family-degraded` debt row; **never** a clean cross-family pass |
| uncalibrated | DP-1/DP-2 artifacts **must not** cite J-45 as satisfied D5 before kappa > 0.6 |

**N-15 (later consumer):** framing-invariance permutes option order across B-41/**J-45**/L-11/I-41 samples (`ELAI-FEATURE-MAP.md:3372-3379`). Do not implement N-15 here.

Adversarial: LLM boolean in CI; quoting literature kappa as ELAI’s; silent single-family pass; replacing D-13 `probe` with gpt-bridge.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A **cross-family** doctrine advisory (producer family ≠ critic family) that emits **findings**, never a release-gating boolean.
- A **100–300** human-labeled corpus and a **held-out Cohen kappa**. Uncalibrated is **labelled decorative** and **does not** count as D5. kappa < 0.5 = unusable.
- **Escalation teeth:** flagged finding becomes a human/orchestrator **decision** in the existing handoff loop, not a dashboard.
- Same-family dispatch **refused**. Second-family outage records **debt**, never a silent cross-family pass.
- **D-13 Mode B remains** the runtime claim-adversary. This organ does **not** replace it, does not own `AdversaryConfig`, and does not sit on D-10 routing.
- J-43/J-44 **route** unmechanizable flags here; they do not grow an LLM gate.
- Net **0** SSB on the critical path. JD-1 evidence is off-agent.

A same-family “second opinion” prompt, an uncalibrated MCP one-shot cited as D5, or folding this into `adversary.rs` is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- LLM-as-judge on a release-gating path (D5) (`J-plan/J-45_cross_family_advisory_calibration.md:12`).
- Same-family self-critique counted as verification (DD-3) (`:12`).
- Dashboard-only surface with no escalation (C3 ignored-by-construction) (`:12`).
- Treating uncalibrated advice as the satisfied D5 check (`:4,7,12`).
- Deep-research as the advisory (`:12`).
- **Replacing D-13** Mode B / `select_grounding_mode` FalsifiabilityCrossFamily with this harness vote.
- Approach A comprehensive doctrine scorer (J-43 already rejected) (`J-plan/J-43_doctrine_propagation_keystone.md:17`).
- Promoting J-43 DD-4 from WARN to BLOCK on the floor alone (`:60`).
- New SSB verdict on a critical path (`J-plan/J-45_cross_family_advisory_calibration.md:30`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-45_cross_family_advisory_calibration.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:80-82,1199-1219`. Doctrine: `docs/strategy/ELAI-DETERMINISM-DOCTRINE.md:164-180,243-244`. J-43: `J-plan/J-43_doctrine_propagation_keystone.md:4,13,15,17,52,60`. J-44: `J-plan/J-44_reasoning_method_artifact_shape.md:48,59`, `reasoning-architecture/J-44-reasoning-method-artifact-shape.md` §3 step 5. D-13: `reasoning-architecture/D-13-adversary-role-cross-family.md`. Carrier: `CLAUDE.md:39`. Feature map (consumer only): `ELAI-FEATURE-MAP.md:3372-3379`.

C3 / 10–25% self-preference **[EXTERNAL, WHY not an ELAI number]:** deep-research governance vote; PROVISIONAL on this domain until EXP-DP3 (`J-plan/J-45_cross_family_advisory_calibration.md:33-34`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact JSONL schema / field names for `data/doctrine-calibration/*.jsonl` (path named; columns not).
- Tool filename for the kappa measurement (sibling of `doctrine_floor_check.py` / `doctrine_debt_rate.py` is implied, not named).
- Exact handoff entry **kind** string (plan: BLOCKED=handoff-entry **or** handoff-mediation; pick at edit).
- Proceed+debt vs BLOCK when gpt-bridge is down for a stretch (lean proceed+debt; decide at edit) (`J-plan/J-45_cross_family_advisory_calibration.md:48`).
- Kappa re-measure cadence (`:49`).
- Who labels besides “user labels the held-out slice” and how label quality is controlled (`:47`).
- Whether oracle/SKILL.md is mandatory or optional DG-15 (`:19`).
- Whether N-15 framing permutation is in-scope for v1 (FEATURE-MAP later unit; not in J-45 produces_symbols).
- Live code: **none** found this write (PLANNED).
