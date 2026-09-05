# Output-side meta-commentary boundary (op-emission only)

- **ELAI ids:** J-106 / HD-3 / WAVE SDL-gap / I-32 twin
- **Spec status (as designed):** PLANNED (`ELAI-FEATURE-MAP.md:705-724`). Grep-confirmed absent product-side at authoring. Default-OFF `ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE` + allowlist; observe-only until reasoning-corpus false-positive rate is **0** (GD-4).
- **Crate / module (spec):** `crates/orchestrator/src/output_commentary_gate.rs` (new); consume I-32 classification via `report_fns.rs` / `role_contracts.rs`; scream path `crates/tools/src/prompt_hardening.rs`; eval metric on E-35/E-49 seam
- **Purpose of this file:** enough architecture to rebuild the output validator that forces work-logs on op-emission roles and **never** constrained-decodes reasoning roles. Not a status ledger.

## 1. What it is

The **output-side meta-commentary boundary** is a frozen, deterministic banned-phrase reject + bounded retry + work-log grammar (`READ / FOUND / DECISION / ACTION / RESULT`) applied **only** to **op-emission** roles (executor tool-call narration, status emission), plus an action-density **eval** metric (tool-actions per prose block / words-per-message) on the E-49 corpus. It is the OUTPUT twin of I-32 (which scopes roles and strips persona on the **input** side). The load-bearing constraint is **HD-3**: grammars and gates **only** on pure op-emission; **never** constrained-decode reasoning / synthesis roles — decoding is sacred for reasoning. Scope is the I-32 role classification, reused, not re-derived. A dropped/violating row uses the I-47 scream path (`J-plan/J-106_output_metacommentary_enforcement.md:1-12,31-35`, `ELAI-FEATURE-MAP.md:705-724`, `J-plan/J-SUMMARY_INDEX.md:2190-2201`).

## 2. Why it exists

I-32 stops narrative persona on the **input**. The model can still open an **output** with affirmation, hedging, or diligence-narration. ELAI had **no** runtime OUTPUT validator, **no** work-log grammar, **no** action-density metric (grep-confirmed gap) (`J-plan/J-106_output_metacommentary_enforcement.md:10,107`). House doctrine (`.claude/rules/output-formatting.md`: result-first openings, ~4 tool-actions per prose block, low words/message; “act, don’t narrate” / measured-Fable-signature) was convention, not a product check (`:7,109`). If this organ is missing, op-emission stays ceremony. If it is faked (grammar on a reasoner, force-JSON the token stream, LLM judges “ceremony”), HD-3 is violated and reasoning is damaged — **a single reasoning-role false-positive blocks default-ON** (`:12,70,114`). This is **house doctrine**, not a paper claim; nothing external to cite or fabricate (`:7,109`).

## 3. How it works (runtime)

1. **Who calls it.** The op-emission role **OUTPUT** path (executor/status emission) calls the validator on a live `elai run`. The eval harness records action-density on E-49. A contracts-only caller never satisfies R7 (`J-plan/J-106_output_metacommentary_enforcement.md:54-58`).
2. **Input.** The outbound text plus the I-32 role classification for that dispatch (`:49-53`).
3. **HD-3 scope (deterministic, first).** If the role is **reasoning / synthesis**, return the text **untouched**. This is the load-bearing exemption. A contract test feeds ceremony-laden prose as a reasoning role and asserts byte-identical passthrough (`:33,70,83`).
4. **Op-emission path — banned-phrase table (deterministic).** Frozen set of ceremony / affirmation / hedging openers from the output-formatting doctrine’s explicit ban list. `mutable=0`, gate-manifest-eligible (R8, I-46 frozen-fixup-table pattern). **No** LLM judge of “ceremony” (TD-3) (`:31,86,115`).
5. **Reject + bounded retry (not constrained-decode).** On hit: REJECT and retry-to-shape. **Not** a hard constrained-decode of the token stream, even for op-emission (HD-3: even here reject+retry, never force-JSON the stream) (`:32,61`).
6. **Work-log grammar.** Target shape `READ / FOUND / DECISION / ACTION / RESULT` for op-emission only. Retry-to-shape; on bounded-retry exhaustion, pass-through **with** an I-47 scream (observe-only first) (`:32,118`).
7. **Scream path.** A dropped/violating row is visible, never silently swallowed (I-47 `prompt_hardening.rs` dropped_row_count idiom) (`:35,63`).
8. **Action-density metric (eval-only).** `tool-actions per prose block` / `words-per-message` recorded on the E-49 eval set. **Not** a runtime gate (`:34,87`).
9. **Observe-only until FP=0.** Default-OFF `ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE` + `wire-dark-allowlist.toml` row (`promotion_plan_id=J-106-promotion`). Observe-only until the reasoning-corpus false-positive rate is bounded at **zero** (GD-4). Then enforce (`:64,89`).
10. **Optional journal.** Prefer-reuse an existing emission/violation row (net 0). `+OutputCommentaryRejected` Pattern-2 only if reject-audit is wanted; full 9-tuple lockstep; TOTAL_VARIANTS from LIVE at edit (W-9). Default = net 0 (`:36,68,116`).
11. **Fail-closed.** Reasoning-role output that gets constrained = **BLOCK**. Silent swallow = ban. Default-ON before FP=0 = ban (`:60-64,70`).

The model may **generate** free prose on reasoning roles, and work-log-shaped text on op-emission roles. It may **not** classify its own role, edit the banned-phrase table, or decide that a reasoner “should” be gated.

## 4. Types, enums, objects

Copy from `J-plan/J-106_output_metacommentary_enforcement.md:31-36,40-47`. Do not invent I-32 role-enum variants beyond what J-106 names.

```
WorkLogShape {                       // reject-to-shape target; op-emission only
  READ,
  FOUND,
  DECISION,
  ACTION,
  RESULT,
}

BannedPhraseTable                    // FROZEN, mutable=0, gate-manifest R8
  // seed = output-formatting.md explicit ban list (ceremony / affirmation / hedging openers)
  // I-46 bash_validation.rs frozen-fixup-table pattern

fn output_commentary_gate(role, text) -> GateResult

GateResult {
  Untouched,                         // reasoning / synthesis — HD-3; byte-identical
  Accepted,                          // op-emission already compliant
  RejectedAndRetry,                  // banned-phrase or off-shape; bounded retry
  PassThroughWithScream,             // retries exhausted; I-47 scream; never silent
}

ActionDensity {                      // EVAL only, not a runtime gate
  tool_actions_per_prose_block,
  words_per_message,
}

// optional SSB (decide in J-106-decisions.md; default net 0):
OutputCommentaryRejected             // Pattern-2; what was rejected + retried

ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE   // default-OFF
promotion_plan_id = J-106-promotion

// REUSED, not re-authored:
I-32 role classification             // report_fns.rs / role_contracts.rs / i32_role_scoping_contract.rs
I-47 scream path                     // prompt_hardening.rs
I-46 frozen table pattern            // bash_validation.rs
E-35 / E-49                          // eval seam + coding metrics
```

**Op-emission (named):** executor tool-call narration, status emission (`:12`). **Reasoning / synthesis:** “a reasoner MUST be free to think in prose” (`:12`). I-32’s own split is executor / context-compiler / researcher **strip persona** vs auditor / critic / adversary / anti-spiral **keep** (`I-plan/I-32_structural_role_scoping.md:12-16,39-41`). Whether **researcher** is op-emission or reasoning for **this** gate is **UNCERTAIN** (§16) — J-106 names executor/status as op-emission and “reasoning / synthesis” as untouched; it does not publish a closed role table.

## 5. Control flow / state machine

**Owner of the loop:** `output_commentary_gate.rs` on the output path. I-32 owns classification. E-49 owns the eval metric. The model does not own the gate.

**Forced order:**

```
outbound text
  → I-32 role class
  → if reasoning/synthesis: return untouched          // HD-3, no table, no grammar
  → if flag OFF or observe-only: journal would-reject; do not block (GD-4)
  → if op-emission AND enforce:
       banned-phrase? → reject + bounded retry-to-WorkLogShape
       retries exhausted → pass-through + I-47 scream
       compliant → accept
```

**Forced BLOCK:** a reasoning-role output that is constrained, force-JSON’d, or retry-shaped (`:70,82`).

**No token-level XGrammar on reasoning** (E-35 already: NEVER force-JSON reasoning roles — this organ **reuses** that precedent, does not punch a hole in it) (`:19`).

## 6. Grounding, receipts, gates

This organ is a **form gate**, not a truth gate. It does not verify whether FOUND is true. Confidence still comes from external receipts (K-16 RR-1), not from a tidy work-log.

**Never allowed as a ceremony signal:** LLM “this looks like narration,” a learned classifier, constrained-decode of a reasoner, using action-density as a **runtime** kill switch (`:34,61-62`).

**HD-3 is the gate.** Grammars/gates only on pure op-emission. The reasoning-corpus FP=0 **done-gate** is how HD-3 is proven, not a vibe (`:70,92`).

Table seed too broad over-blocks legitimate op-emission; too narrow misses ceremony. Expansion **only** via gate-manifest review (R8) (`:115`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| work-log labels | exactly `{READ, FOUND, DECISION, ACTION, RESULT}` | op-emission retry target |
| retry | bounded (count UNCERTAIN §16) | per output |
| FP done-gate | reasoning-corpus **0** before default-ON | GD-4 |
| doctrine density (DESIGN) | ~4 tool-actions per prose block; low words/message | **eval** metric, not runtime threshold |
| flag | `ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE` default-OFF | process |
| allowlist | type=experiment, owner, expiry, `promotion_plan_id=J-106-promotion`, smoke | R9 |
| table | frozen in `.claude/gate-manifest.txt` mutable=0 | R8 |
| contract tests | ~120 LOC `j106_output_commentary_contract.rs` | (a)–(g) |
| SSB | net 0 default; +1 if `OutputCommentaryRejected` | W-9 LIVE ~268/577 |
| new deps | none | (`:89`) |

**Overflow:** retries exhausted → scream + pass-through, not an infinite retry, not a force-decode (`:118`).

Do not confuse with: E-35 constrained-decode (this is reject+retry); I-32 input scoping; I-47 input scream; K-16 trace-as-confidence (different grain).

## 8. Roles and neighbors

**Op-emission only.** Reasoning/synthesis **provably untouched**.

**Consumes:** I-32 classification (`report_fns.rs`, `role_contracts.rs`); I-47 scream; E-35 eval seam; E-49 corpus; I-46 table pattern; op-emission OUTPUT path (`:48-53`).

**Produces:** gate module; scoped call from `report_fns.rs`; eval metric; optional reject row; `J-106-decisions.md`; allowlist + gate-manifest table (`:99-105`).

**Must not fork:** a second role taxonomy; XGrammar on reasoners; LLM ceremony judge; a runtime action-density blocker.

**House-doctrine source:** `.claude/rules/output-formatting.md` + global act-don’t-narrate (`:7,109`).

**Frontend:** none required at runtime; metric on E-49 surface; tile vs W-6 N/A at edit (`:65-66`).

## 9. Config / flags / defaults

- `ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE` default-OFF + complete wire-dark-allowlist row (W-2/R9) (`:64,89`).
- Observe-only until reasoning-corpus FP=0 (`:64`).
- Banned-phrase table in gate-manifest, mutable=0 (R8) (`:105`).
- SSB: net 0 default; journal only if reject-audit wanted; resolved in `J-106-decisions.md` (`:116`).
- Table seed: doctrine ban list, expanded only via manifest review (`:115`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/output_commentary_gate.rs` | table + reject + retry + grammar + HD-3 scope (`:42,100`) |
| `report_fns.rs` | classification consumer + output-path call (`:101`) |
| `role_contracts.rs` / `i32_role_scoping_contract.rs` | I-32 (`:17,110`) |
| `prompt_hardening.rs` | I-47 scream (`:51`) |
| `bash_validation.rs` | I-46 pattern (`:53`) |
| E-35 / E-49 eval seam | action-density (`:52,102`) |
| `wire-dark-allowlist.toml` | flag row (`:105`) |
| `.claude/gate-manifest.txt` | frozen table (R8) (`:105`) |
| `ssb.rs` + payloads + lock + `.golden` | only if +`OutputCommentaryRejected` (`:103`) |
| `crates/contracts/tests/j106_output_commentary_contract.rs` | (`:97, not listed in Files but Tasks/Tests name it`) |
| `workflow/reasoning/J-106-decisions.md` | SSB + table seed + FP=0 finding (`:36,104`) |

Plan “Files touched” omits the contract file that “Tests to add” names (`:96-104`). Rebuild includes the contract. UNCERTAIN whether that omit is intentional (§16).

## 11. Journal / SSB events

Prefer-reuse existing emission/violation row (`:68`):

```
// default NET 0
// optional:
OutputCommentaryRejected { /* rejected text + retry outcome; GD-2 recomputable */ }
```

LIVE ~268/577 at authoring; bump TOTAL_VARIANTS only if the optional type is chosen (W-9) (`:22,68,93`). Observe-only still needs a visible scream (I-47), which may ride existing rows.

## 12. Tests and observables

Named contract `j106_output_commentary_contract.rs` (`:97`):

| Id | Assert |
|---|---|
| (a) | op-emission ceremony fixture (banned-phrase opener + narration) rejected + retried to work-log shape |
| (b) HD-3 | **same** prose from a **reasoning** role passes **UNTOUCHED** (zero FP) |
| (c) | a reasoning-role output that gets constrained is a **BLOCK** |
| (d) | banned-phrase reject deterministic under `ELAI_SEED` (frozen table) |
| (e) | retry is bounded |
| (f) | dropped/violating row uses I-47 scream |
| (g) | action-density recorded on a fixture corpus |

Production observable (`:67-70`): live op-emission ceremony → reject+retry; live reasoning same prose → untouched; eval-corpus run records density.

`rg` (`:76-79`): HD-3 scope from I-32 classification; banned/work_log/reject-retry present; action_density wired off the test tree.

W-1: live reject+retry **and** live reasoning untouched, not “the table exists” (`:92`). production_caller on `output_commentary_gate.rs` (`:75`).

Negative: reasoning-role constrained = BLOCK (`:70`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Frozen deterministic banned-phrase table (no LLM judge), gate-manifest-frozen.
- Op-emission outputs: reject + **bounded retry-to-work-log**, not constrained-decode of the token stream.
- Work-log target uses `{READ, FOUND, DECISION, ACTION, RESULT}` (or a documented equivalent closed set).
- Reasoning / synthesis outputs are **byte-identical** through the gate. FP on a reasoning corpus must be **0** before default-ON.
- Role scope is the harness’s existing role classification, not a second taxonomy.
- Violations scream; they are not swallowed.
- Action-density is an **eval** signal, not a runtime veto.
- Default-OFF + allowlist; observe-only first.

A system prompt “don’t narrate,” XGrammar on a reasoner, or an LLM style-judge is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Constraint on a reasoning / synthesis role’s output (HD-3) (`:60`).
- Hard constrained-decode of the op-emission token stream / force-JSON (`:61`).
- LLM judging whether output is “ceremony” (`:62`).
- Silent swallow of a violating row (`:63`).
- Default-ON before reasoning-corpus FP=0 (`:64`).
- Action-density as a runtime gate (`:34`).
- Re-authoring I-32 / I-47 / E-35 (`:16-19`).
- Paper numbers (there are none; house doctrine) (`:7`).
- Copying sdl-mcp code (license ban; FEATURE-MAP 4883 neighbor, not this organ’s body).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-106_output_metacommentary_enforcement.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:2190-2201`. Feature map: `ELAI-FEATURE-MAP.md:705-724`. SDL reasoning: `workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md` §4. House doctrine: `.claude/rules/output-formatting.md`. I-32: `I-plan/I-32_structural_role_scoping.md:12-16,39-41`. Neighbors: I-32, I-47, I-46, E-35, E-49. Hard rules: HD-3, TD-3, GD-4, GD-2, W-1/W-2/W-7/W-9, R7/R8/R9.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Closed map from I-32 roles → `{op-emission, reasoning/synthesis}`. Executor/status are named op-emission; researcher is I-32 strip-persona but not named here (`:12` vs `I-32:14`).
- Bounded retry **count** (`:31,85` say bounded, no N).
- Exact banned-phrase strings (seed from doctrine ban list at edit; not copied into the plan) (`:115`).
- Observe-only journal shape when net-0 (I-47 scream on which existing row) (`:36,68`).
- Whether `j106_output_commentary_contract.rs` is in-scope given “Files touched” omit (`:96` vs `:99-105`).
- Action-density runtime **thresholds** — doctrine cites ~4 tool-actions/block as house convention; the metric is recorded, not gated (`:7,34`).
- Call-site symbol on the op-emission OUTPUT path (plan says re-grep W-7) (`:29,100`).
- LIVE schema / TOTAL_VARIANTS at implement time (W-9).
- Dashboard tile vs W-6 (`:66`).
- Interaction with E-35 per-mode grammars if an op-emission role already has a decode grammar (compose vs double-gate unspecified).
