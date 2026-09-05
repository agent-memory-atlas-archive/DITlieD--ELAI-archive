# Grounding Verification (Phase 2B)

- **ELAI ids:** phase-2B / P2B-E001..P2B-E023 / P2B-I001 / P2B-A001 / P2B-A002 / P2B-C001
- **Spec status (as designed):** WIRED
- **Crate / module (spec):** `crates/reasoning` (`reasoning/src/verification.rs`, `engine.rs` `verify_claim`/`ground_plan`); later IPC wrap is `GroundingRequest` in `crates/ipc-types`
- **Purpose of this file:** enough architecture to rebuild binary claim verification + the plan-grounding sweep that blocks PLANNING→EXECUTING. Not a status ledger.

## 1. What it is

**Grounding verification** resolves Phase 2A’s `type:reference` strings against real sources. Six `VerificationMethod` variants return a **binary** `passes: bool` plus an evidence string — no confidence scores (`phase-2B.txt:12-14`, `phase-2B.txt:44-54`). `ground_plan` decomposes a plan into those claims, classifies each GROUNDED / VERIFY_NEEDED / UNGROUNDED, auto-promotes resolvable `training:` (Uncertain) claims, and reports leftovers. Any remaining UNGROUNDED claim **blocks** PLANNING→EXECUTING (`phase-2B.txt:16-18`, `phase-2B.txt:9-10`).

## 2. Why it exists

Format-valid grounding is not evidence. Without this organ a plan can cite `file:nonexistent.rs` and still enter EXECUTING. Without binary composition (`all_grounded iff ungrounded.is_empty()`) a harness will invent a threshold. Without typed network stubs, unit tests grow live HTTP or panics (`phase-2B.txt:12-18`, `phase-2B.txt:136-138`, `ELAI-SPEC-OVERVIEW.md:68-69`).

## 3. How it works (runtime)

1. **Who calls it.** `GrcEngine::verify_claim` / `ground_plan` from the orchestrator (plan sweep before EXECUTING) and from `think` auto-resolution for Uncertain (`phase-2B.txt:16-23`, `phase-2B.txt:463-470`). After 4A, agents send `GroundingRequest`; the orchestrator runs the same methods (`phase-2B.txt:25-27`, `phase-4A.txt:168-180`).
2. **`verify_claim(&self, claim, source, context)`** — `&self`, **stateless per call** so 4A can wrap without a trait-signature change (`phase-2B.txt:141-143`, `phase-2B.txt:109-117`).
3. **Prefix map (deterministic, prefix always stripped before the Verifier)** (`phase-2B.txt:432-446`):

   | prefix | method | remainder used as |
   |---|---|---|
   | `file:` | FileCheck | path |
   | `grep:` | FileCheck | path (grep is file-based) |
   | `test:` | TestCheck | test name; `context` = crate name |
   | `search:` | WebCheck | query |
   | `tool:` | SourceCheck | symbol; `context` = file path |
   | `training:` | **no verify** | `VerificationResult { passes: false }` Uncertain; sweep may auto-resolve |

4. **LocalVerifier (implemented in 2B):**
   - **FILE_CHECK:** `tools::read_file(path, None, None)` after stripping `file:`/`grep:`. Exists → `passes=true`, evidence `"File exists: {path}, {total_lines} lines"`. `Error::NotFound` → `passes=false`, `"File not found: {path}"` (`phase-2B.txt:401-405`).
   - **SOURCE_CHECK:** `read_file` + regex `r"(fn|struct|enum|trait|impl|const|static|type|mod)\s+{symbol_name}\b"`. Found → `"Found {kind} {name} at {file}:{line}"`; else not found (`phase-2B.txt:407-412`).
   - **TEST_CHECK:** `TestRunner::run_test(name, crate)`. Production `CargoTestRunner` runs `BashSandbox::execute("cargo test --exact {name} -p {crate}", …)` **30s timeout**. Exit 0 pass; nonzero fail with stderr; timeout fail `"timed out after 30s"` (`phase-2B.txt:414-421`).
5. **NetworkVerifier (stubbed in 2B):** WEB_CHECK, API_CHECK, API_REGISTRY_CHECK return `GrcError::VerificationError` — typed, no panic, no network (`phase-2B.txt:423-430`).
6. **`ground_plan(&self, plan) -> PlanGroundingResult`:**
   1. Regex-extract `type:reference` with type ∈ `{file,grep,test,search,tool,training}` (`phase-2B.txt:452-454`).
   2. Each claim → `verify_claim`.
   3. `passes=true` → GROUNDED; `passes=false` → UNGROUNDED; `training:` → VERIFY_NEEDED, attempt FILE_CHECK/SOURCE_CHECK auto-resolve; success → Verified in `claims` and **not** in `verify_needed`; failure stays Uncertain and lands in **both** `verify_needed` and `ungrounded` (`phase-2B.txt:176-186`, `phase-2B.txt:456-458`).
   4. Empty plan (0 claims) → `all_grounded=true` (trivial; must not panic) (`phase-2B.txt:139-140`, `phase-2B.txt:460`).
   5. `verified_at` from injected `core::entropy::TimeSource` as `u64` unix (`phase-2B.txt:75-80`, `phase-2B.txt:167-170`).
7. **Uncertain auto-promotion (I5).** On Uncertain, if the claim looks like a path → FILE_CHECK; like a symbol → SOURCE_CHECK. Pass → `GroundingLevel::Verified`. Fail or not resolvable → stay Uncertain (2C decides reject policy) (`phase-2B.txt:20-23`, `phase-2B.txt:463-470`). Spec also says this runs during `think()` (`phase-2B.txt:470`).
8. **FSM gate.** `all_grounded == false` blocks PLANNING→EXECUTING (`phase-2B.txt:9-10`, `phase-2B.txt:537-543`). The orchestrator evidence gate consumes the ungrounded list (`audit-implementation/phase-2B.md:284-317` — producer wiring is a later phase; the **rebuild contract** is: something must call `ground_plan` and refuse EXECUTING on leftovers).
9. **Fail-closed.** Network methods: `VerificationError`, not `passes=true`. Missing file: `passes=false`, not skip. Regex must not treat comment `file:fake_path` as a claim (T15) (`phase-2B.txt:349-355`, `phase-2B.txt:487-491`).

The model may **emit** claim strings. It may **not** grade them.

## 4. Types, enums, objects

```
VerificationMethod { FileCheck, SourceCheck, TestCheck, WebCheck, ApiCheck, ApiRegistryCheck }
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  requires_network() -> bool   // WebCheck, ApiCheck, ApiRegistryCheck = true
  requires_tools() -> bool     // FileCheck, SourceCheck, TestCheck = true

VerificationResult { claim: String, passes: bool, evidence: String, method_used: VerificationMethod }

GroundingResult {
  claim: String,
  method: VerificationMethod,   // NOT String (2A stub upgraded)
  level: GroundingLevel,
  source: String,
  verified_at: u64,             // TimeSource unix
}

PlanGroundingResult {
  claims: Vec<GroundingResult>,
  all_grounded: bool,           // == ungrounded.is_empty()
  ungrounded: Vec<String>,
  verify_needed: Vec<String>,   // subset of ungrounded; unresolved training:
}

GrcError::VerificationError { method: VerificationMethod, claim: String, reason: String }

trait Verifier: Send + Sync {
  fn verify(&self, method: VerificationMethod, claim: &str, context: &str)
    -> Result<VerificationResult, GrcError>;
  fn supported_methods(&self) -> &[VerificationMethod];
}

trait TestRunner: Send + Sync {
  fn run_test(&self, name: &str, crate_name: &str) -> TestRunResult;
}
TestRunResult { passed: bool, output: String, duration_ms: u64 }

CargoTestRunner { sandbox: Box<dyn BashSandbox>, timeout_ms: u64 }
MockTestRunner { results: HashMap<String, TestRunResult> }

LocalVerifier::new(Box<dyn TestRunner>)   // FILE/SOURCE/TEST
NetworkVerifier                            // WEB/API/API_REGISTRY stubs
```

**Breaking vs 2A stubs** (`phase-2B.txt:96-117`):
- `verify_claim(&self, claim, source, context)` (3 args).
- `ground_plan(&self, plan) -> PlanGroundingResult` (not `Vec<GroundingResult>`).

**Object-safety:** `Box<dyn Verifier>` must compile (`phase-2B.txt:232-233`, T14). Send+Sync is inferred from 4A IPC, not an explicit sentence in reasoning.txt (`phase-2B.txt:245-249`, `P2B-I001`).

## 5. Control flow / state machine

This organ is not an FSM of its own. It is a **function** the GRC engine and the orchestrator FSM call.

**Classification after each verify:**

```
passes=true                → GROUNDED
passes=false (non-training)→ UNGROUNDED
training: auto-resolve ok  → GROUNDED, level=Verified, not in verify_needed
training: auto-resolve fail→ UNGROUNDED ∩ VERIFY_NEEDED, level=Uncertain
```

**Forced block:** any UNGROUNDED remaining → do not enter EXECUTING (`phase-2B.txt:199`, `phase-index-MASTER.txt:301-304`).

**Empty plan:** trivially GROUNDED (`phase-2B.txt:139-140`).

**Who owns the loop:** `BasicGrcEngine` holds `Box<dyn Verifier>` injected at construction (`phase-2B.txt:443-444`). Orchestrator owns *when* `ground_plan` is called (before EXECUTING).

## 6. Grounding, receipts, gates

Evidence is the `evidence: String` plus `verified_at`. Binary only (`phase-2B.txt:44-54`). MiniCheck-style labels, not RARR/SAFE NLI (`phase-2B.txt:50-63`).

**Never allowed as a confidence signal:**
- LLM “looks grounded”
- Scalar scores / thresholds (`phase-2B.txt:13-14`, `phase-2B.txt:48-54`)
- Semantic NLI decomposition (`phase-2B.txt:59-63`)
- Network success assumed because a stub returned `Ok`
- Agent-local memory that a file existed last turn without a fresh check

**Selector vs 2A levels:** 2A classified by prefix type. 2B can **promote** Uncertain→Verified when FILE/SOURCE pass (`phase-2B.txt:20-22`). Rejection of leftover Uncertain is **2C** (`phase-2B.txt:23`).

## 7. Budgets, limits, ceilings

| Limit | Value |
|---|---|
| FILE_CHECK latency | `< 2000` µs |
| SOURCE_CHECK latency | `< 10000` µs on a **≥1000-line** file |
| `ground_plan` 20 claims | `< 100000` µs |
| methods implemented in 2B | **3/6** (FILE, SOURCE, TEST) |
| methods stubbed | WEB, API, API_REGISTRY |
| TEST_CHECK timeout | **30s** |
| unit tests | **zero** network (`reqwest`/`hyper`/`TcpStream`/`http://`) |
| unit tests | **zero** `sleep`/`Duration`/`Instant`/`SystemTime` |
| TEST_CHECK in unit tests | MockTestRunner only — no cargo subprocess (`phase-2B.txt:381-382`) |

`verify_claim` does **not** consume GRC topic budget in the 2B phase file. After 4A, `GroundingRequest` **does** count toward the per-topic ThinkStep budget (`phase-4A.txt:178-180`). Rebuild 2B as `&self` and stateless; rebuild 4A as the counter.

## 8. Roles and neighbors

2B is method-complete, role-agnostic. 2C `allowed_methods` is the **policy** filter (what a role may request). 4A IPC is the **capability** filter (what the orchestrator will proxy). Policy ≤ capability (`phase-2C.txt:68-80`).

**Consumes:** `tools::read_file`, `BashSandbox::execute` (spec crate map). **Must not** create `tools → reasoning` cycle (`phase-2B.txt:82-94`, `phase-2B.txt:260-276`). Spec also says `cargo tree -p reasoning | grep knowledge` empty (`phase-2B.txt:270-272`).

**Produces:** `VerificationResult`, `PlanGroundingResult`. **Must not fork:** a second “semantic fact checker” beside these six methods for the same PLANNING→EXECUTING gate.

**Later neighbors (not this organ):** 7th CosmosCheck (B-42), 8th VisualArtifactCheck (E-36) (`ELAI-FEATURE-MAP.md:570-574`). Do not add them to the 2B enum.

## 9. Config / flags / defaults

- Network backends gated `#[cfg(feature="network")]` for integration only (`phase-2B.txt:136-138`).
- No confidence threshold config — binary composition (`phase-2B.txt:53-54`).
- `TimeSource` injected for `verified_at` (`phase-2B.txt:75-79`).
- Deps allowed in 2B: `core`, `tools`, `thiserror`, `serde` (`phase-2B.txt:266-268`). `providers/` not yet.

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `reasoning/src/verification.rs` | Verifier, TestRunner, LocalVerifier, NetworkVerifier |
| `GrcEngine::verify_claim` / `ground_plan` | `phase-2B.txt:203-208`, `:432-461` |
| `tools::read_file` | FILE_CHECK / SOURCE_CHECK |
| `BashSandbox::execute` | CargoTestRunner |
| tests `reasoning/tests/verification_tests.rs` | ≥16 tests |
| metrics `.elai_cc/metrics/phase_2B.toml` | `file_check_latency_us`, `source_check_latency_us`, `plan_sweep_latency_us`, `methods_implemented` |
| 4A `GroundingRequest` / `GroundingResult` IPC | wrap, do not change trait (`phase-2B.txt:25-27`) |

**Audited tree (not spec):** skipped `tools/` dep; used internal `FileReader`; `CargoTestRunner` not in reasoning/; `ground_plan` regex anchored on `Grounding:` prefix; Uncertain auto-resolve in `ground_plan` **not** in `think()`; `GroundingSweepGate` exists but no production caller of `ground_plan` (`audit-implementation/phase-2B.md:12-18`, `:213-223`, `:273-317`). Those are deviations. Rebuild the **spec** unless an owner ratified the FileReader inversion.

## 11. Journal / SSB events

2B specifies **no** SSB action_type. The orchestrator evidence gate later consumes `TransitionContext.ungrounded_claims` (`audit-implementation/phase-2B.md:287-292`). 3B `reasoning_trace` / `claim` action types are neighbors (`phase-index-MASTER.txt:421-423`).

## 12. Tests and observables

File: `reasoning/tests/verification_tests.rs` (`phase-2B.txt:284-360`):

| Id | Name | Assert |
|---|---|---|
| T1 | `file_check_existing_file` | temp file → `passes==true`, evidence contains path |
| T2 | `file_check_nonexistent_file` | `passes==false` |
| T3 | `source_check_function_exists` | file with `fn hello_world()` → pass, evidence contains name |
| T4 | `source_check_function_missing` | `passes==false` |
| T5 | `test_check_passing_test` | mock runner; reflects configured result |
| T6 | `web_check_stub_returns_error` | `VerificationError` or network-unavailable stub; **no network** |
| T7 | `api_check_stub_returns_error` | same |
| T8 | `api_registry_check_stub_returns_error` | same |
| T9 | `plan_grounding_sweep_blocks_ungrounded` | existing + missing file → `all_grounded==false`, ungrounded contains missing |
| T10 | `plan_grounding_sweep_passes_all_grounded` | only valid files → `all_grounded==true` |
| T11 | `plan_grounding_sweep_empty_plan` | `ground_plan("")` → `all_grounded==true` |
| T12 | `uncertain_claim_auto_resolution` | `training:` + existing file → promoted Verified |
| T13 | `verification_method_requires_network` | WebCheck true, FileCheck false |
| T14 | `verifier_trait_object_safety` | `Box<dyn Verifier> = LocalVerifier` compiles |
| T15 | `plan_sweep_adversarial_regex` | `// see file:fake_path` **not** extracted; real `file:src/main.rs` is |
| T16 | `verify_claim_strips_prefix` | `verify_claim("file:src/main.rs",…)` Verifier sees `src/main.rs` |

Exit: 6 methods at type level; 3 implemented; 3 typed stubs; prefix strip; empty plan safe; 16+ tests; 2A tests still pass; zero `todo!`/`unimplemented!`/`unwrap()`/`panic!` in `verification.rs` (`phase-2B.txt:537-548`).

## 13. Rebuild contract

- Exactly six `VerificationMethod` variants with `requires_network` / `requires_tools` as specified.
- Binary `passes` + evidence. No scores.
- `Verifier` + `TestRunner` object-safe, Send+Sync, DI at `LocalVerifier::new`.
- FILE_CHECK / SOURCE_CHECK / TEST_CHECK real; network three return `VerificationError` without sockets.
- SOURCE_CHECK regex exactly `(fn|struct|enum|trait|impl|const|static|type|mod)\s+{name}\b`.
- TEST_CHECK via sandbox `cargo test --exact {name} -p {crate}` with **30s** timeout in production; mocks in unit tests.
- `verify_claim` 3-arg, strips `type:` **before** Verifier, prefix table as §3.
- `ground_plan` returns `PlanGroundingResult` with `verify_needed ⊆ ungrounded` and `all_grounded = ungrounded.is_empty()`.
- Empty plan grounded. T15 does not harvest comment `file:`.
- Uncertain `training:` attempts FILE/SOURCE; unresolved stays Uncertain for 2C.
- `&self` / stateless so 4A IPC can wrap without changing the trait.
- PLANNING→EXECUTING refuses leftover UNGROUNDED.
- No `tools→reasoning` cycle.

A “let the model grade citations” checker, an NLI decomposer, or a six-method enum that always returns `passes=true` is a cousin.

## 14. Non-goals / banned designs

- Scalar confidence / threshold tuning (`phase-2B.txt:13-14`, `:48-54`).
- LLM/NLI claim decomposition (`phase-2B.txt:59-63`).
- Live network in unit tests (`phase-2B.txt:136-138`).
- Panic/`todo!` on WEB/API/API_REGISTRY (`phase-2B.txt:423-430`, `:483-485`).
- Passing `file:src/main.rs` **with prefix** into `read_file` (`phase-2B.txt:486-488`).
- Greedy regex that matches `file:` inside comments (`phase-2B.txt:489-491`, T15).
- TEST_CHECK without timeout (`phase-2B.txt:492-493`).
- Hard-wired Verifier (no `Box<dyn Verifier>` inject) (`phase-2B.txt:494-495`).
- Adding Cosmos/Visual methods into this organ’s enum (`ELAI-FEATURE-MAP.md:573-574` are later).
- Changing `verify_claim` to `&mut self` (breaks 4A wrap) (`phase-2B.txt:141-143`).

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-2B.txt`. Index: `phase-index-MASTER.txt:301-330`. Overview: `ELAI-SPEC-OVERVIEW.md:68-69`. Feature map: `ELAI-FEATURE-MAP.md:570-574`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:73-75`. Exhaustive: P2B-E001..E023, P2B-I001, P2B-A001, P2B-A002, P2B-C001. Audit: `audit-implementation/phase-2B.md`. 4A wrap: `phase-4A.txt:168-180`, `phase-2B.txt:25-27`.

## 16. Gaps

- **Exact plan-extraction regex** beyond “type:reference in the six-type set” vs T15’s need for context (`P2B-A001`, `phase-2B.txt:349-355`, `:452-454`). One tree used `^(?:\s*Grounding:\s*)((file|grep|…):[^\s]+)` (`audit-implementation/phase-2B.md:194-198`). That prefix is **not** in I4.
- SOURCE_CHECK “looks like a path” vs “looks like a symbol” heuristic (`P2B-A002`, `phase-2B.txt:463-469`).
- Exact stub reason string: `"network verification not implemented in phase 2B"` vs MASTER `"not impl until Phase 4A IPC proxy"` — behavior is typed error, not the string (`P2B-C001`, `phase-2B.txt:425`, `phase-index-MASTER.txt:325`).
- Whether I5 auto-resolve **must** run inside `think()` as well as `ground_plan` (`phase-2B.txt:470` vs audit partial).
- FILE_CHECK evidence line-count omitted in one tree (`audit-implementation/phase-2B.md:414-416`) — spec text includes `{total_lines}`.
- Who populates orchestrator `ungrounded_claims` from `PlanGroundingResult` (2B says the sweep blocks; does not name the struct) (`phase-2B.txt:16-18`).
