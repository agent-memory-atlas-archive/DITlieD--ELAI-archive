# Per-Agent GRC Config (Phase 2C)

- **ELAI ids:** phase-2C / P2C-E001..P2C-E020 / P2C-I001 / P2C-A001 / P2C-A002 / P2C-C001
- **Spec status (as designed):** WIRED
- **Crate / module (spec):** `crates/reasoning` (`reasoning/src/types.rs` `AgentGrcConfig`, `reasoning/src/config.rs`, `engine.rs` `with_agent_config`); TOML parsed in `core`/`orchestrator`, not in `reasoning/`
- **Purpose of this file:** enough architecture to rebuild role-routed GRC budgets, step whitelists, method policy, additive transition overrides, and auditor anti-rubber-stamp. Not a status ledger.

## 1. What it is

**Per-agent GRC configuration** gives each agent role its own `GrcBudget`, `required_steps`, minimum `grounding_level`, `allowed_methods`, CSQ flag, additive `transition_overrides`, and `require_scope_enumeration`. The engine installs one `AgentGrcConfig` at spawn via `with_agent_config`. Layering is `[grc.defaults]` plus field-by-field `[agents.{role}.grc]` overrides (underscore keys, not hyphens). Validation is fail-fast at load (`phase-2C.txt:7-12`, `phase-2C.txt:41-52`, `phase-2C.txt:54-66`).

## 2. Why it exists

A single 5/800 chain cannot serve auditor double-verify, context-compiler observe-only, and memory-writer 2/400. Without `transition_overrides`, auditor’s second Verify hits 2A `InvalidTransition` (no Verify→Verify in the default map) (`phase-2C.txt:28-40`). Without engine-enforced `FINDING:`, anti-rubber-stamp is a prompt suggestion and adversarial review dies (`phase-2C.txt:86-97`). Without load-time validation, `budget=1` with five required steps explodes mid-reasoning as `BudgetExhausted` (`phase-2C.txt:50-52`).

## 3. How it works (runtime)

1. **Who calls it.** Orchestrator at spawn: load config for the role, `engine.with_agent_config(config)`. All later `think` calls use that config (`phase-2C.txt:8-9`, `phase-2C.txt:368-372`).
2. **Load.** `AgentGrcConfigLoader::load_config(role, &GrcDefaults)` merges `[grc.defaults]` with `[agents.{role}.grc]` **field-by-field, not deep merge**. Missing role table → defaults only (dynamic roles ok). Role table present but invalid → **error, not silent fallback** (`phase-2C.txt:54-57`, `phase-2C.txt:360-366`, `phase-2C.txt:402-404`).
3. **Validate (fail-closed at load)** (`phase-2C.txt:42-48`, `phase-2C.txt:312-319`):
   - (a) `required_steps` non-empty
   - (b) `required_steps` is a valid chain under **merged** transitions
   - (c) `budget.max_calls >= required_steps.len()`
   - (d) `csq_required` ⇒ at least one `Verify` in `required_steps`
   - (e) `allowed_methods ⊆` known `VerificationMethod`
   First failure → `GrcError::ConfigValidation { role, reason }`.
4. **`with_agent_config`:** store config **and reset all topic state** (no budget leak across roles) (`phase-2C.txt:360`, `phase-2C.txt:399-401`).
5. **Each `think` — effective transitions** (`phase-2C.txt:321-335`):
   ```
   base      = GrcState::allowed_next(state)
   plus      = config.transition_overrides[state]   // additive union; NEVER remove
   merged    = base ∪ plus
   whitelist = required_steps ∪ {Status, Converge}  // Status always allowed (P2C-I001)
   effective = merged ∩ whitelist
   ```
   Context-compiler `required_steps=[Observe]` therefore cannot leak Hypothesize even though default `allowed_next(Observe)` contains it.
6. **Grounding floor.** Classify the step’s `GroundingLevel`. If `level < config.grounding_level` → reject. Ordering **must** be `Uncertain < High < Verified` (`phase-2C.txt:337-348`).
7. **Method policy.** A requested verification method not in `allowed_methods` is rejected in-process in 2C. 4A adds a second IPC layer (`phase-2C.txt:68-81`). Policy ≤ capability.
8. **Anti-rubber-stamp (auditor only).** On `think(Converge)` when `role == "auditor"`: scan history for `Verify` steps whose `content.trim_start()` **starts with** const `FINDING_PREFIX = "FINDING:"` (case-sensitive). Count `< 1` → `InsufficientFindings { role, expected_min: 1 }` (`phase-2C.txt:86-92`, `phase-2C.txt:350-358`).
9. **No config set:** engine uses global defaults — backward compatible with 2A (`phase-2C.txt:372`).
10. **Budget authority in 2C:** this in-process engine **is authoritative** through Phases 0–3. Phase 4A demotes agent-side to advisory (`phase-2C.txt:373-380`, `orchestrator.txt L440-444` as cited). Same `GrcBudget` struct; **who enforces** changes.

The model may generate FINDING lines and steps. It may not skip the prefix check or widen `allowed_methods` at runtime.

## 4. Types, enums, objects

**`AgentGrcConfig` — 8 fields** (`phase-2C.txt:111-125`, `phase-2C.txt:162-164`):

```
AgentGrcConfig {
  role: String,                                          // NOT an enum (cross-crate)
  budget: GrcBudget,
  required_steps: Vec<GrcState>,
  grounding_level: GroundingLevel,                       // minimum accepted
  csq_required: bool,
  allowed_methods: Vec<VerificationMethod>,
  transition_overrides: HashMap<GrcState, Vec<GrcState>>, // additive only
  require_scope_enumeration: bool,  // default true executor/designer, false memory_writer
}
```

```
GrcError::InsufficientFindings { role: String, expected_min: usize }
GrcError::ConfigValidation { role: String, reason: String }

trait AgentGrcConfigLoader {
  fn load_config(role: &str, global_defaults: &GrcDefaults) -> Result<AgentGrcConfig, GrcError>
  fn validate_config(config: &AgentGrcConfig) -> Result<(), GrcError>
}

GrcDefaults { budget, grounding_level, csq_required, allowed_methods }

GrcEngine += with_agent_config(&mut self, config: AgentGrcConfig)
GrcEngine += get_agent_config(&self) -> Option<&AgentGrcConfig>

const FINDING_PREFIX: &str = "FINDING:"
```

`reasoning.txt` still documents a 4-field `AgentGrcConfig`; 2C requires 8. Phase file flags reconciliation; it does not edit that owned spec (`phase-2C.txt:82-84`, `phase-2C.txt:468-469`, `P2C-C001`). Rebuild **8 fields**.

**`allowed_methods` vs `grounding_level`:** complementary, not redundant. Methods = **which** checkers; level = **minimum quality** (`phase-2C.txt:492-496`).

## 5. Control flow / state machine

2C does not replace 2A’s machine. It **intersects** it with a per-role whitelist and **unions** extra edges.

**Auditor double-verify:** `required_steps = [Observe, Hypothesize, Verify, Verify]` plus `transition_overrides = { Verify: [Verify, Observe] }` (`phase-2C.txt:29-37`, T1). Overrides cannot delete default edges (`phase-2C.txt:36-37`).

**Context-compiler observe-only:** `required_steps = [Observe]`; Hypothesize/PlanAction/Act/Verify → `InvalidTransition` (`phase-2C.txt:206-209`). Status/Converge remain reachable (`phase-2C.txt:327-333`).

**Owner of the loop:** still `GrcEngine::think`. Config is session-scoped on the engine, not per-topic.

**Forced fail-closed:** invalid TOML table → `ConfigValidation`; auditor Converge without FINDING → `InsufficientFindings`; level below minimum → reject; disallowed method → reject.

## 6. Grounding, receipts, gates

Auditor minimum is **Verified** (T8: `training:` Uncertain rejected) (`phase-2C.txt:237-241`). Context-compiler accepts High and is separately limited to FILE_CHECK+SOURCE_CHECK (`phase-2C.txt:451`, `phase-2C.txt:76-78`).

**Never allowed as a confidence signal:** prompt text “I found an issue” without the `FINDING:` prefix on a Verify step; case-folded `finding:`; FINDING on a non-Verify step (`phase-2C.txt:354-358`). The prefix is an **internal protocol**, not NLP (`phase-2C.txt:356-357`, `P2C-A001`).

## 7. Budgets, limits, ceilings

Pinned by tests / exit criteria (`phase-2C.txt:212-267`, `phase-2C.txt:445-455`, `phase-index-MASTER.txt:337-340`):

| Role (TOML key) | Budget (calls/chars) | required_steps (tests) | Notes |
|---|---|---|---|
| executor | **8/1200** | full chain (T3: 8 thinks, 9th `BudgetExhausted`) | `require_scope_enumeration=true` |
| auditor | **10/1200** | `[Observe, Hypothesize, Verify, Verify]` | Verified-only; FINDING; Verify→Verify override |
| context_compiler | observe-only `[Observe]` | FILE_CHECK+SOURCE_CHECK | numeric budget **UNCERTAIN** in phase-2C.txt (tests do not pin) |
| memory_writer | **2/400** | T4: 3rd think `BudgetExhausted` | `require_scope_enumeration=false` |
| prompt_engineer | **5/800** | `[Observe, Hypothesize, Converge]` (T13) | |
| designer | **8/1200** | `[Observe, Hypothesize, PlanAction, Verify, Converge]` (T14) | distinct from executor; produces PipelineSpec; same model **possible** |

Metrics: `config_load_us < 1000`; `routing_overhead_pct < 10`; memory per config `< 2KB` including overrides (`phase-2C.txt:414-421`). Measure validation separate from TOML I/O (`phase-2C.txt:433-434`). Pathological override size for the memory test: 7 states × 7 targets = 49 (`phase-2C.txt:435-437`).

**Feature-map “8 roles”** (`ELAI-FEATURE-MAP.md:575-577`) vs **2C “6 roles including Designer”** (`phase-2C.txt:445-447`): 2C’s organ is the six above. Researcher / MediaValidator appear on the orchestrator agent-pool (`audit-implementation/phase-2C.md:54-57`, `ELAI-SYSTEMS-OVERVIEW.md:105-106`). Those extras are **not specified in phase-2C.txt**. Do not invent their GRC tables here.

## 8. Roles and neighbors

**Consumes:** typed `AgentGrcConfig` from core/orchestrator (no `toml` crate in reasoning) (`phase-2C.txt:182-188`). **Produces:** effective transitions, rejections, `get_agent_config` for orchestrator inspection.

**Must not fork:** a second hardcoded role table in the orchestrator that disagrees with `TomlConfigLoader` (`audit-implementation/phase-2C.md:30-52` documents that bug). One source of truth.

**Neighbors:** 2A machine; 2B methods; 2D reads `csq_required`; 4A demotes budget authority and re-enforces `allowed_methods` on `GroundingRequest`; 3C spawn installs the config.

**Per-agent FSM sub-states** IDLE→LOADING→REASONING→ACTING→DONE are cited in 2C grounding summary from reasoning.txt §7 (`phase-2C.txt:467`) but are **not** implemented in 2C steps. Treat as neighboring role lifecycle, not this organ.

## 9. Config / flags / defaults

TOML surfaces (`phase-2C.txt:54-66`, `phase-2C.txt:445-447`):

```
[grc.defaults]
[agents.executor.grc]
[agents.auditor.grc]
[agents.context_compiler.grc]    # underscore, NOT context-compiler
[agents.memory_writer.grc]
[agents.prompt_engineer.grc]
[agents.designer.grc]
```

Matches existing `[agents.{role}.supervision|.slo_tier|.sandbox|.role_prompt]` pattern (`phase-2C.txt:58-65`).

**`require_scope_enumeration` defaults:** true executor/designer, false memory_writer (`phase-2C.txt:120-125`, `phase-2C.txt:455`).

Unknown role → defaults, no error. Malformed known table → error (`phase-2C.txt:365-366`, `:402-404`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `reasoning/src/types.rs` `AgentGrcConfig` | 8 fields |
| `reasoning/src/config.rs` | loader + `validate_config` + `TomlConfigLoader` |
| `engine.rs` `effective_allowed_next` / FINDING scan / level check | I2–I4 |
| tests `reasoning/tests/agent_config.rs` | T1–T15 |
| metrics `.elai_cc/metrics/phase_2C.toml` | `config_load_us`, `routing_overhead_pct` |
| `GrcEngine::with_agent_config` / `get_agent_config` | `phase-2C.txt:134-136` |

**Audited tree:** `TomlConfigLoader` hardcoded, no `[grc.defaults]` in `elai_cc.toml`; orchestrator `default_grc_config` disagrees on Designer / PromptEngineer / ContextCompiler; `routing_overhead_pct=28.5` vs `<10` (`audit-implementation/phase-2C.md:15-52`, `:180-187`). Rebuild the **spec** (real TOML merge + one table + observe-only compiler).

## 11. Journal / SSB events

None specified in 2C. Orchestrator may inspect `get_agent_config`. FINDING lines are ThinkStep content, not an SSB schema in this phase.

## 12. Tests and observables

`reasoning/tests/agent_config.rs` ≥15 (`phase-2C.txt:196-282`):

| Id | Name | Assert |
|---|---|---|
| T1 | `auditor_gets_double_verify` | Observe, Hypothesize, Verify, Verify all succeed |
| T2 | `context_compiler_observe_only` | Observe ok; Hypothesize → `InvalidTransition` |
| T3 | `executor_budget_8_1200` | get_budget 8/1200; 9th think `BudgetExhausted` |
| T4 | `memory_writer_budget_2_400` | 3rd think `BudgetExhausted` |
| T5 | `config_validation_empty_steps_rejected` | `ConfigValidation` mentions empty steps |
| T6 | `config_validation_budget_too_small` | max_calls=2 vs 5 steps → `ConfigValidation` |
| T7 | `auditor_anti_rubber_stamp` | Converge without FINDING → `InsufficientFindings`; Verify `"FINDING: unused variable"` then Converge ok |
| T8 | `auditor_rejects_uncertain_grounding` | Observe `training:…` with min Verified → error; also `assert!(Verified > Uncertain)` |
| T9 | `context_compiler_allowed_methods` | WEB_CHECK request rejected |
| T10 | `global_defaults_with_per_role_override` | defaults 5/800; executor override 8/1200; other fields inherit |
| T11 | `transition_overrides_are_additive` | Observe override `[Verify]` **still includes Hypothesize** |
| T12 | `auditor_budget_10_1200` | 11th think `BudgetExhausted` |
| T13 | `prompt_engineer_budget_5_800` | steps Observe/Hypothesize/Converge; 6th think exhausted |
| T14 | `designer_config_distinct_from_executor` | role name, required_steps, role_prompt differ; PipelineSpec vs code |
| T15 | `require_scope_enumeration_per_role` | executor Observe requires scope; memory_writer does not |

Zero timing-dependent tests (`phase-2C.txt:302`). Cover FINDING exact prefix, leading whitespace, missing prefix (`phase-2C.txt:358`).

Exit criteria: six roles including Designer; additive overrides; anti-rubber-stamp; grounding floor; five validation rules; field-by-field merge; 2A+2B still pass; 15+ tests; zero `todo!`/`unwrap` in `engine.rs`/`config.rs` (`phase-2C.txt:445-457`).

## 13. Rebuild contract

- 8-field `AgentGrcConfig` + `GrcDefaults` + loader trait + `ConfigValidation` / `InsufficientFindings`.
- Role as `String` validated against known names at load; no hyphen keys.
- Field-by-field overlay, not deep merge. Missing table → defaults. Bad table → error.
- `validate_config` rules (a)–(e) before any `think`.
- `effective = (base ∪ overrides) ∩ (required_steps ∪ {Status, Converge})`; overrides cannot remove base edges.
- Auditor: 10/1200, double Verify, `{Verify:[Verify,Observe]}`, Verified-only, `FINDING:` on Converge.
- Context-compiler: observe-only + FILE_CHECK/SOURCE_CHECK only.
- Executor 8/1200; memory-writer 2/400; prompt-engineer 5/800 with `[Observe,Hypothesize,Converge]`; designer 8/1200 distinct chain/output.
- `with_agent_config` resets topics.
- In-process engine is budget authority until 4A.
- `GroundingLevel` Uncertain < High < Verified.
- T1–T15 pass.

A single global GRC config, a replace-not-union override map, or prompt-only “auditor please find bugs” is a cousin.

## 14. Non-goals / banned designs

- Deep TOML merge (`phase-2C.txt:56-57`).
- `[agents.context-compiler.grc]` hyphen keys (`phase-2C.txt:63-66`).
- `AgentRole` enum inside reasoning (use String) (`phase-2C.txt:152-156`).
- Overrides that **remove** default transitions (`phase-2C.txt:36-37`, `:157-159`).
- Advisory-only anti-rubber-stamp (`phase-2C.txt:94-97`).
- `reasoning/` parsing TOML itself (`phase-2C.txt:182-185`).
- Silent fallback on malformed role table (`phase-2C.txt:402-404`).
- Leaving 2A `GroundingLevel` declaration order as `{Verified, High, Uncertain}` with derived Ord (`phase-2C.txt:342-347`).
- Treating 2C agent-side budget as authority **after** 4A (`phase-2C.txt:373-380`).
- Using `grounding_level=High` as a substitute for `allowed_methods` (`phase-2C.txt:492-496`).
- NLP / case-insensitive FINDING matching as the contract (prefix is case-sensitive after trim) (`phase-2C.txt:354-357`). T7 adversarial note allowed discussing case-insensitive; **I4 contract is case-sensitive**.

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-2C.txt`. Index: `phase-index-MASTER.txt:332-362`. Overview: `ELAI-SPEC-OVERVIEW.md:70-71`. Feature map: `ELAI-FEATURE-MAP.md:575-577` (8-role claim — see §7). Systems: `ELAI-SYSTEMS-OVERVIEW.md:76-78`. Context §7.2: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:246-256` (**auditor budget listed there as 8/1200 — contradicts T12 10/1200; T12 / MASTER win**). Exhaustive: P2C-E001..E020. Audit: `audit-implementation/phase-2C.md`.

## 16. Gaps

- Full authoritative TOML **sample tables** for all six roles are test-described, not printed as one config block (`P2C-A002`, `phase-2C.txt:198-277`).
- Context-compiler **numeric** budget not pinned in phase-2C.txt tests (audit claimed 3/600; that number is **not** in `phase-2C.txt`. 4A feasibility override **is** 3/600 — do not conflate).
- Researcher / MediaValidator GRC tables: not in 2C (`ELAI-FEATURE-MAP.md:575` vs `phase-2C.txt:445`).
- Structured `FindingMarker` type deferred; prefix is interim (`P2C-A001`, `phase-2C.txt:394-398`).
- Whether Status is a GRC step type in `required_steps` or only the always-allowed extra (`P2C-I001`).
- Exact error variant for level-insufficient (new variant vs extend `UngroundedStep`) (`phase-2C.txt:339-340`).
- How T14 asserts `role_prompt` when `AgentGrcConfig` has no `role_prompt` field (T14 names it; the 8-field struct does not). **UNCERTAIN** — likely compared via neighboring agent record, not this struct.
