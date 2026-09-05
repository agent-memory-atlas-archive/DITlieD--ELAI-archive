# CRANE hybrid + grammar pinning (I-34)

- **ELAI ids:** I-34 / IC-18 / IS-12 / landmine #7 / landmine #8 / J-15 Option A (llguidance compile+pin)
- **Spec status (as designed):** PLAN **PLANNED** (`ELAI-FEATURE-MAP.md:2375-2376`). Live: `crane_adaptive` flag + `CraneAdaptiveConfig` **descriptor**, `re_validate_output`, `grammar_store`, `call_grammar_gate`, `GrammarGateEvent`. Sidecar **mask** CRANE = Tier-2. D-16 **enforcement** = `WIRE-DARK[I-34-enforce]` observe-only (`wire-dark-allowlist.toml:2054-2068`, `ELAI-FEATURE-MAP.md:5259-5260`).
- **Crate / module (spec):** `providers/src/decoding.rs` (`DecodingMode::Grammar { ebnf_path, crane_adaptive }`, `re_validate_output`); grammar pin store; orchestrator `grammar_gate.rs`.
- **Purpose of this file:** enough architecture to rebuild CRANE free-thinking + blake3-pinned grammars + post-parse re-validation. Not a status ledger.

## 1. What it is

**CRANE hybrid + grammar pinning** is three layers on the shipped constrained-decode substrate (IS-12 / C-26 / E-35):

1. **CRANE:** thinking tokens are **permitted between structural commits**; the grammar binds **FINAL output only** (structure-snowballing defense). The existing `crane_adaptive` flag gets a **runtime body**.
2. **Pin:** every grammar is a **blake3-pinned control-plane artifact** (E-24 MCP pattern). **No third-party grammar.** Enum-bearing schemas from **untrusted** sources **rejected**. Manifest is **R8-frozen**.
3. **Post-parse:** after the engine emits tokens, **re-parse against the schema**. Silent non-compliance → **D-16 refusal**, never downstream corruption.

(`I-plan/I-34_crane_hybrid_grammar_pinning.md:5-31`, `I-plan/I-SUMMARY_INDEX.md:641-660`, `I-plan/I-0_INDEX.md:562-567`)

HD-3 still holds: **do not force-JSON / grammar-wrap reasoning, coder, or planner roles**. J-15’s Rust `llguidance` leg is **compile+validate+pin at plan time**, not token masking on those roles (`:124-132`).

## 2. Why it exists

Tight grammars on **intermediate** reasoning cost ~10–15pp (structure-snowballing — **[DESIGN]**, measure locally, ID-5). XGrammar-class engines **silently** emit invalid JSON (landmine #7). Grammars are a **CDA attack surface** (landmine #8). If this organ is missing, `crane_adaptive` is a dead flag and the mask is trusted. If it is faked (`--trust-mask`, loadable third-party EBNF, JSON-schema on a reasoner), IC-18 and HD-3 are dead (`:5-9,30-31,82-85`).

## 3. How it works (runtime)

1. **Who calls it.** Local `DecodingMode::Grammar` path (`:78`). Init: grammar manifest load. After every constrained generation: `re_validate_output` (`:47,80`). C-37 pilot **consumes** CRANE (`:18,62,109`). Orchestrator `call_grammar_gate` (`grammar_gate.rs:1-12`). Model does not choose the grammar file.
2. **Eligibility (deterministic).** `crane_adaptive_eligible` iff `Grammar { crane_adaptive: true }` (`decoding.rs:183-185`). Else skip pin/CRANE; live `NotEligible` ≡ pass for the caller (`grammar_gate.rs:71-74`).
3. **CRANE runtime (plan).** Between commit points, **free** tokens; constraint at **commit / FINAL** only. Reflection loops: constraint on **final** output only (`:14-18,85`). Live **descriptor** `CraneAdaptiveConfig { thinking_tokens_permitted: true, constrain_final_only: true, ebnf_path }` via `from_mode` only (`decoding.rs:191-239`). **Sidecar logits-processor free-token pass = Tier-2** (`:53-54`, `decoding.rs:202-204`).
4. **Pin (deterministic, fail-closed).** Load `.elai/grammar_pins/manifest.toml`. Re-hash each EBNF; mismatch → `GrammarGateEvent(PinViolation)` (`:20-23,79`; `grammar_store.rs:5-26,115-125`). Running process **must not** rewrite the manifest (R8) (`:22-23`, `grammar_gate.rs:18-19`).
5. **Untrusted enums (deterministic).** Enum-bearing schemas from untrusted sources **rejected at load** (CDA) (`:21-22`, landmine #8).
6. **Generate** via E-35/E-18 XGrammar **mask** (inference-time; unchanged by J-15) (`:125-127`).
7. **Post-parse (deterministic, mandatory).** `re_validate_output(raw, schema)` — **never** trust the mask (landmine #7) (`:25-28,84`). Fail → `PostParseRejected` → D-16 (`:80`). `--trust-mask` broken arm **FAILS** R2 (`:28,92`).
8. **J-15 Option A (plan-time).** `llguidance` `ParserFactory` compiles+validates+blake3-pins the **spec** at **plan** time; RRA carries spec+pin; executor **recompiles** on load (compiled state **not** serializable) (`:117-139`). **Option B** (Rust token mask) **REJECTED** (`:128-132`). **New dep requires user confirmation + anti-slopsquat re-verify** (`:141-149`).
9. **Commit.** `GrammarGateEvent` Kind `{CranePassed, PinViolation, PostParseRejected}` (`:48,70`).
10. **The model may generate** free thinking between commits and a structured finale. It may **not** load a grammar, skip re-validation, or constrain its own CoT (HD-3).

## 4. Types, enums, objects

Copy from `I-plan/I-34_crane_hybrid_grammar_pinning.md:14-28,66-70` and live types.

```
DecodingMode::Grammar { ebnf_path: PathBuf, crane_adaptive: bool }  // IS-12 :73-82

fn crane_adaptive_eligible(mode) -> bool
CraneAdaptiveConfig {
  thinking_tokens_permitted: bool,   // MUST true in CRANE
  constrain_final_only: bool,        // MUST true in CRANE
  ebnf_path: PathBuf,
}
fn CraneAdaptiveConfig::from_mode(mode) -> Option<Self>  // only Grammar+crane_adaptive

fn re_validate_output(output: &str, schema_json: &str) -> Result<(), ValidationError>

GrammarPinRecord { file_path, blake3_hex, pinned_at_ms }
GrammarPinManifest { entries: Vec<GrammarPinRecord> }
fn load_and_verify(manifest_path) -> Result<GrammarPinManifest, GrammarPinError>
GrammarPinError { Io, TomlDe, PinViolation { path, expected, actual }, FileNotFound }

fn call_grammar_gate(...) -> GrammarGateResult
GrammarGateResult { Passed, NotEligible, PinViolation {..}, PostParseRejected { message } }

GrammarGateEventKind { CranePassed, PinViolation, PostParseRejected }
GrammarGateEventPayload {
  kind,
  grammar_path: String,
  pin_violation_expected: Option<String>,
  pin_violation_actual: Option<String>,
  session_id: String,
  ts_ms: i64,
}

// J-15
ParserFactory   // llguidance; plan-time compile+validate+pin; NOT inference mask
```

Manifest path live: `.elai/grammar_pins/manifest.toml` (`grammar_store.rs:6-16`).

## 5. Control flow / state machine

**Owner:** providers decode + orchestrator gate. Generator never owns the grammar bytes.

```
DecodingMode::Grammar { crane_adaptive }
  → [J-15] plan-time compile+pin (spec in RRA)
  → load_and_verify manifest          → PinViolation → stop
  → sidecar XGrammar mask (inference)
      CRANE: free tokens between commits; constrain FINAL only
  → re_validate_output                → PostParseRejected → D-16
  → CranePassed
```

**Forced:** no third-party load (`:82`). No mask-without-revalidate (`:84`). No tight grammar on intermediate reasoning (`:85`). Python sidecar CRANE mask = Tier-2 (`:53-54`). I-34-enforce D-16 **Err** path still dark; live observe+warn+Ok (`wire-dark-allowlist.toml:2057-2063`). Rebuild the **plan**: PostParseRejected **routes to refusal**, not a silent continue.

## 6. Grounding, receipts, gates

**Receipts:** blake3 pin, `GrammarGateEvent` rows, post-parse Result. The mask is **not** evidence.

**Never allowed:** third-party grammar (`:30,82`); `--trust-mask` (`:31,84`); constraints on intermediate CoT (`:85`); force-JSON of reasoning roles (HD-3, J-15 `:131-132`); paper 10–15pp as a gate (ID-5) (`:49,91`).

Selector: pinned manifest **is** the sole grammar authority (`:82`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| CRANE vs FreeText | pass-rate **≥ FreeText** on reasoning fixture, **measured** (`:49,91`) | ID-5 |
| pin store size (plan) | ~80 LOC (`:59`) | advisory |
| llguidance | v1.7.6 snapshot **re-verify at exec**; compile sub-second (README, not a gate) (`:136-148`) | plan-time |
| retries | C-26 `max_retries_on_invalid` default 3 (`decoding.rs:118-131`) | constrained calls |
| hash | blake3 only (`:86`) | pins |

**Overflow:** pin mismatch fail-fast first record (`grammar_store.rs:110-111`). Invalid JSON output → PostParseRejected (`decoding.rs:291-295`).

## 8. Roles and neighbors

**CRANE/grammar on op-emission / structured-output roles** (C-26 per-mode). **Reasoning / coder / planner: FreeText** (HD-3) (`ELAI-FEATURE-MAP.md:2374-2375`, J-15 `:131-132`). I-33 CEFL JSON-schema is **decision-type**, not this CRANE loop. I-32 does not pick grammars. N-13 OutputBoundary **inherits** I-34 pins (`ELAI-FEATURE-MAP.md:3335-3346`). C-37 consumes CRANE (`:109`). E-24 is the pin **pattern** (`:107`).

**Must not fork:** a second XGrammar in Rust (Option B rejected) (`:128-132,147-149`).

## 9. Config / flags / defaults

- IS-12 `crane_adaptive: bool` on `DecodingMode::Grammar` (`:72-82` of decoding.rs as cited in plan `:34`).
- Manifest R8-frozen; `gate_freeze_check.py` covers it (`:59`).
- `WIRE-DARK[I-34-enforce]`: observe-only until FP measured (`wire-dark-allowlist.toml:2054-2068`).
- Decision doc: `workflow/reasoning/I-34-decisions.md` (`:51`).
- Frontend: grammar-pin chip on existing decode panel; `cross_lang_diff.py` (`:50,63,87-88`).
- CLI: `elai run --crane-fixture --seed 42` (`:90`); live `bench_crane.rs`.
- `llguidance` **not** added without user confirm (`:141-144`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `decoding.rs` CRANE runtime / `crane_adaptive` | `:67,72` |
| `re_validate_output` | `:68` |
| grammar pin manifest | `:69` |
| `GrammarGateEvent` 3-arm | `:70` |
| E-35/E-18 XGrammar | `:73` |
| C-26 DecodingMode | `:74` |
| E-24 pin pattern | `:75` |
| D-16 refusal | `:76` |
| live `CraneAdaptiveConfig` | `decoding.rs:206-239` |
| live `grammar_store.rs` | pin load |
| live `grammar_gate.rs::call_grammar_gate` | orchestrator |
| live `bench_crane.rs` | three arms |
| J-15 `llguidance` ParserFactory | `:117-139` |

## 11. Journal / SSB events

**+GrammarGateEvent** Kind `{CranePassed, PinViolation, PostParseRejected}` (`:70,102`). Payload §4 (`ssb/payloads.rs:20279-20324`). Wire: `crane_passed` / `pin_violation` / `post_parse_rejected`. GD-2 under `ELAI_SEED` (`:91`).

## 12. Tests and observables

Named (`:49,82-92`, `I-SUMMARY_INDEX.md:656-660`):

| Id | Assert |
|---|---|
| CRANE vs FreeText | reasoning fixture pass-rate **≥** FreeText; tax of tight-grammar arm **recorded** (ID-5) |
| CranePassed | compliant + clean manifest |
| PinViolation | mutated grammar file refused at load |
| PostParseRejected | seeded silent-invalid output caught; **not** shipped downstream |
| `--trust-mask` | skips re_validate → **FAIL** R2 |
| pin chip | decode panel + cross_lang_diff |
| live `from_mode` | only `crane_adaptive: true` yields config (`decoding.rs:771-783`) |
| live `re_validate` | missing required / wrong type / enum / non-JSON fail (`decoding.rs:794-833`) |

Adversarial: third-party EBNF; enum schema from untrusted plugin; JSON-schema on executor CoT; trusting XGrammar stdout.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `crane_adaptive` **does something**: free tokens **between** structural commits; grammar on **FINAL** only.
- Grammars load **only** from a **blake3 pin manifest**; tamper → PinViolation; process cannot rewrite the manifest.
- **Every** constrained output is **re-parsed**; fail → refusal, not silent accept.
- Reasoning/coder/planner are **not** force-JSON’d (HD-3).
- `GrammarGateEvent` three kinds exist.
- `--trust-mask` is a **failing** arm.
- Optional: plan-time `llguidance` compile+pin **without** replacing the inference mask (Option A). Option B is not this organ.

A tight JSON-schema around CoT, or “the mask said OK,” is a **cousin**.

## 14. Non-goals / banned designs

- Third-party grammars (`:30,82`).
- Trusting the mask (`:31,84`; landmine #7).
- Constraints on intermediate reasoning (`:85`; IC-18).
- Force-JSON reasoning roles (HD-3; J-15 `:131-132`).
- Option B Rust token masking (`:128-132`).
- Reimplementing XGrammar / E-24 pin protocol / D-16 (`:104-108`).
- Adding `llguidance` without user confirm (`:141-144`).
- Serializing llguidance compiled parser (not supported; recompile) (`:134-139`).
- Paper pp / ASR numbers as gates (ID-5).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-34_crane_hybrid_grammar_pinning.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:641-660`. Landmines: `I-plan/I-0_INDEX.md:562-567`. Feature map: `ELAI-FEATURE-MAP.md:2374-2376`, `:5259-5260`. Live: `providers/src/decoding.rs`, `providers/src/grammar_store.rs`, `orchestrator/src/grammar_gate.rs`, `ssb/payloads.rs:20279-20324`, `cli/src/bench_crane.rs`, `wire-dark-allowlist.toml:2054-2068`. HD-3: `H-plan/H-0_INDEX.md:291-297`. Neighbors: C-26, C-37, E-24, E-35, D-16, I-33 (decision CEFL, not CRANE).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `decoding.rs:62-63` still says CRANE runtime “not yet implemented” while `CraneAdaptiveConfig` exists as a **descriptor** — sidecar mask vs descriptor split.
- Untrusted **enum-bearing schema** reject: specified (`:21-22`); **no** matching check in `grammar_store.rs` grep.
- `re_validate_output` validates **JSON Schema subset**, not EBNF of `Grammar.ebnf_path` (`decoding.rs:289-315` vs plan “re-parsed against the schema” after Grammar mode).
- I-34-enforce observe-only vs plan D-16 route (`:80` vs allowlist `:2057-2063`).
- `--crane-fixture` vs live `bench_crane` CLI name (`:90`).
- Manifest path `.elai/grammar_pins/` vs plan “path TBD W-7” (`:69`).
- Whether `NotEligible` should emit CranePassed (live comment `:6-7`) or emit nothing.
- J-15 dep pin **v1.7.6** is a 2026-06-11 snapshot; must re-verify (`:143-147`).
- How commit **points** are detected in the token stream (plan: “between structural commitments”; no tokenizer rule).
- Paper arXiv IDs to WebSearch-confirm before headlines (`:99-101`).
