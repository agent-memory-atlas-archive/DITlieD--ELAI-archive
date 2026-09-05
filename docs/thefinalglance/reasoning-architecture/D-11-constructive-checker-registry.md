# Constructive-checker registry (D-11)

- **ELAI ids:** D-11 / RPS.II / D0-015 / D0-046 / D004-D11-001..061 / J-11 / J-15
- **Spec status (as designed):** DARK(`ELAI_CHECKER_ENABLE`) (`ELAI-FEATURE-MAP.md:585-589`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:86`). Live boot keeps `checker_runtime=None` until the env var is set (`task_runner.rs:3730-3746`, `runtime_config.rs:1270-1298`).
- **Crate / module (spec):** `crates/orchestrator/src/checker_registry.rs`; checker scripts under `.elai_cc/checkers/`; Mode A dispatch in `grounding_mode.rs`
- **Purpose of this file:** enough architecture to rebuild Mode A’s external-checker substrate. Not a status ledger.

## 1. What it is

The **constructive-checker registry** is Mode A’s operational substrate. When `select_grounding_mode` returns `Constructive`, the orchestrator runs an **external, deterministic checker** whose Pass/Fail/Inapplicable verdict is ground truth — not LLM confidence (`D-plan/D-11_constructive_checker_registry.md:1-9`, `ELAI-SYSTEMS-OVERVIEW.md:232-236`). Layout root is `.elai_cc/checkers/` with domain folders `{math,code,text,graph,api,perf}/`. D-11 ships **six initial** checkers: sympy, z3, compile_rust, compile_py, registry_check, regex_assertion.

## 2. Why it exists

PoT (+12% vs CoT), CodeAct (+20%), and AlphaProof (Lean kernel as verifier) all show: when a claim is operationalizable, the **checker** is the grade (`D-plan/D-11_constructive_checker_registry.md:47-53`). If this organ is missing, Mode A is a prompt. If it is faked (LLM “I ran sympy”), the optimizer grades itself. J-11 adds a register-time fixture gate and a demotion clause so a construction Fail cannot average into a false pass (`D-plan/D-11_constructive_checker_registry.md:330-359`).

## 3. How it works (runtime)

1. **Who calls it.** D-10 `dispatch_by_mode` Constructive branch → `CheckerRegistry::run` (`D-plan/D-11_constructive_checker_registry.md:161-165`, `D-plan/D-10_mode_routing_predicate.md:107-112`). Live later fill: `task_runner` post-task `dispatch_constructive_checkers_hook` when `ELAI_CHECKER_ENABLE=1` (`task_runner.rs:3730-3737`, `runtime_config.rs:1270-1298`).
2. **Input.** `claim: &Claim`, `candidate: &Candidate`, `ctx: &OrchContext` (sandbox handle). Checkers themselves read JSON on stdin.
3. **Load (deterministic).** `CheckerRegistry::load(root)` walks `root/*/*.toml`; each TOML is one `CheckerEntry` (`D-plan/D-11_constructive_checker_registry.md:89-91,99-108`).
4. **Applicable (deterministic).** `applicable(claim)` = entries whose `claim_types` contain `claim.claim_type` **AND** domain matches. Empty registry → empty `Vec`, not panic (`D-plan/D-11_constructive_checker_registry.md:92-93,222`).
5. **Run (deterministic dispatch, checker is the verdict).** `run` takes the **first** applicable entry, sandbox-executes the script, parses stdout JSON to `CheckerResult`. Timeout → `Inapplicable { reason: "timeout" }`. JSON parse failure → `Inapplicable`, not panic (`D-plan/D-11_constructive_checker_registry.md:94-97,219-224`).
6. **Checker process contract.** Stdin JSON in; stdout JSON `{ "result": "Pass" }` or `{ "result": "Fail", "evidence": ["…"] }` (plus Inapplicable). Exit 0 on Pass/Fail/Inapplicable; exit 1 only on script-level error (`D-plan/D-11_constructive_checker_registry.md:110-119`).
7. **Sandbox.** compile_rust / compile_py **must** run under Tier-C (seccomp + SO_PEERCRED + cap-drop). ComBench `reliability_guard` is **not** a sandbox and must not be copied (`D-plan/D-11_constructive_checker_registry.md:361-367`).
8. **J-11 register-time gate.** Before admission, each checker runs its bundled **reference fixture**. Fixture Fail → reject at registration, fail-closed (`D-plan/D-11_constructive_checker_registry.md:346-351`).
9. **J-11 demotion.** `Fail` caps the overall verdict (example: construction Fail ⇒ overall ≤ 0.3). **Averaging** a construction Fail with a proof score is banned (`D-plan/D-11_constructive_checker_registry.md:353-359`).
10. **J-15 one reformulation.** First Fail → structured “checker X failed for reason Y; reformulate” to the claimant **once** → same checker re-run. Second Fail → `Ungrounded`. Routing is deterministic; the LLM does not decide whether to retry (`D-plan/D-11_constructive_checker_registry.md:375-395`). Demotion still applies after reformulation (`:400-404`).
11. **Commit.** SSB `CheckerDispatched` plus exactly one of `CheckerPass` / `CheckerFail` / `CheckerInapplicable` (`D-plan/D-11_constructive_checker_registry.md:161-172`).
12. **The model may generate** the candidate body. It may **not** pick the checker, override Pass/Fail, skip the sandbox, or vote in an ensemble (ensemble is deferred).

## 4. Types, enums, objects

Copy from `D-plan/D-11_constructive_checker_registry.md:80-97,32-36`. Do not invent fields.

```
enum CheckerResult {
  Pass,
  Fail { evidence: Vec<BoundedString<2048>> },
  Inapplicable { reason: String },
}

fn check(claim: Claim, candidate: Candidate, sandbox: SandboxHandle) -> CheckerResult

struct CheckerEntry {
  name: String,
  claim_types: Vec<ClaimType>,
  domain: Domain,
  executable_path: PathBuf,
  language: CheckerLanguage,
  timeout_secs: u32,
}

enum CheckerLanguage { Python, Shell, Rust, Js }   // closed

struct CheckerRegistry { entries: Vec<CheckerEntry> }
  load(root: &Path) -> Self
  applicable(claim: &Claim) -> Vec<&CheckerEntry>
  run(claim, candidate, ctx) -> CheckerResult   // first applicable
```

**Registration TOML example** (`D-plan/D-11_constructive_checker_registry.md:99-108`):

```toml
name = "sympy"
claim_types = ["math"]
domain = "math"
executable_path = ".elai_cc/checkers/math/sympy.py"
language = "python"
timeout_secs = 30
```

**Live cousin:** `CheckerResult` adds `Error { source }` and Fail uses `reason: String` not `Vec<BoundedString<2048>>` (`checker_registry.rs:158-172`). `CheckerKind` closed enum `{Sympy, Z3, CompileRust, RegexMatch, JsonSchemaValidate, …}` (`checker_registry.rs:62-80`). Rebuild the **plan** triad + evidence vec; live `Error` is additive only if a later plan says so — original treats infra failure as Inapplicable.

## 5. Control flow / state machine

**Owner:** orchestrator registry + Tier-C sandbox. Checkers are subprocesses.

**Mode A path:** claim → D-10 Constructive → `applicable` → first entry → sandbox exec → parse JSON → CheckerResult → SSB → (J-15) maybe one reformulation → D-12/D-15 consume result.

**v1 dispatch:** single first-applicable checker. Multi-checker ensemble voting is **deferred** (`D-plan/D-11_constructive_checker_registry.md:323`).

**Forced:** timeout honors `timeout_secs`; no infinite wait. Empty `applicable` → do not panic (D-10 then sees no checker and must not pretend Mode A passed).

**J-15 loop bound:** exactly one reformulation pass. Not a retry-until-pass loop.

## 6. Grounding, receipts, gates

Checker stdout JSON **is** the receipt. LLM narrative is never a Mode A receipt.

**Never allowed as a confidence signal:** model self-eval, averaging Fail with unrelated scores (J-11 demotion), cached mock for production `registry_check`, Python `reliability_guard` as isolation.

**registry_check (anti-slopsquatting):** live npm / PyPI / crates.io. Pass iff hit **and** a release in the last **24 months**. Fail `"package not found"` or `"stale (last release > 24 months)"`. Unknown registry → Inapplicable (`D-plan/D-11_constructive_checker_registry.md:142-152`). Production must hit the live registry — no cached mock (`:221`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| sympy timeout (example) | 30 s | per `CheckerEntry.timeout_secs` |
| Fail evidence strings | `BoundedString<2048>` each | per Fail |
| initial checkers | 6 | this organ |
| registry freshness | last release ≤ 24 months | registry_check |
| J-15 reformulations | 1 | per claim/checker |
| demotion example cap | ≤ 0.3 on construction Fail | J-11; value otherwise UNCERTAIN |
| SCHEMA_VERSION (plan) | 56 → 57 | SSB |
| unit tests | ≥10 new checker tests | acceptance |
| observable | 6 synthetic claims, one per checker | dispatch + terminal row each |

## 8. Roles and neighbors

Mode A only. Consumes D-10 `GroundingMode::Constructive`, D-10 `Claim`/`ClaimType`, D-1/D-7 `Domain`, B-18/C-11 Tier C. Produces `CheckerResult` for D-12 `MEASURABLE_PROCEDURE` and D-15 `NoveltyClass` labeling.

FEATURE-MAP: consumes C-24 symbol_impact and C-30 surgical-mode DKG tooling (`ELAI-FEATURE-MAP.md:585-589`).

**Must not fork:** in-process PyO3 sympy (decision: subprocess JSON); a second LLM-as-checker; averaging Fail away.

## 9. Config / flags / defaults

- **`ELAI_CHECKER_ENABLE`** — opt-in. Truthy `"1"|"true"|"yes"|"on"` (case-insensitive). Absence → `CheckerRuntimeConfig::is_enabled() == false`; hook is no-op (`runtime_config.rs:1277-1298`, `wire-dark-allowlist.toml:821-826`). Copy this flag on rebuild.
- Optional `elai_cc.toml [checkers]` registration paths (`D-plan/D-11_constructive_checker_registry.md:218`).
- Python deps: pin `sympy` and `z3-solver` in `pyproject.toml` (`D-plan/D-11_constructive_checker_registry.md:120-128`).
- Default-OFF in live boot (`checker_runtime=None`). Original plan does not name the env var — FEATURE-MAP / runtime_config are the flag authority.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `checker_registry.rs` | `CheckerResult`, `CheckerEntry`, `CheckerLanguage`, `CheckerRegistry` |
| `.elai_cc/checkers/math/sympy.py` + `.toml` | identity simplify(LHS−RHS)==0 |
| `.elai_cc/checkers/math/z3.py` + `.toml` | unsat of negation |
| `.elai_cc/checkers/code/compile_rust.sh` + `.toml` | `cargo check` in Tier C |
| `.elai_cc/checkers/code/compile_py.sh` + `.toml` | `py_compile` + `ruff check` |
| `.elai_cc/checkers/api/registry_check.py` + `.toml` | live npm/pypi/crates.io |
| `.elai_cc/checkers/text/regex_assertion.py` + `.toml` | full\|contains |
| `grounding_mode.rs` Constructive branch | dispatch |
| `ssb.rs` 56→57 | four Checker* variants |
| `orchestrator/tests/checker_dispatch_integration.rs` | integration |
| `runtime_config.rs` `ELAI_CHECKER_ENABLE_ENV` | live gate |
| `agent_pool::dispatch_constructive_checkers` | live producer (`checker_registry.rs:33-36`) |

**Deferred (layout reserved, not this organ):** `math/lean.py`, `code/compile_ts.sh` + `test_*.sh`, `text/schema_validate.py`, `graph/networkx.py`, `perf/{timing_harness,asymptotic_curve_fit}.py` (`D-plan/D-11_constructive_checker_registry.md:318-323`).

## 11. Journal / SSB events

Plan SCHEMA_VERSION **56 → 57**:

```
CheckerDispatched { claim_id: Uuid, checker_name: String, audit_id: Uuid }
CheckerPass { claim_id: Uuid, checker_name: String, latency_ms: u64 }
CheckerFail { claim_id: Uuid, checker_name: String, evidence: Vec<String> }
CheckerInapplicable { claim_id: Uuid, checker_name: String, reason: String }
```

(`D-plan/D-11_constructive_checker_registry.md:167-172`). Live later fill uses `checker_result_emitted` (`task_runner.rs:3737`). Rebuild: one dispatch row + one terminal result row per run. J-11/J-15 add no extra SSB variants in the original files list (gap).

## 12. Tests and observables

| Case | Assert |
|---|---|
| applicable(math) | sympy + z3 |
| applicable(api) | registry_check |
| mock stdout Pass | `CheckerResult::Pass` |
| timeout | Inapplicable reason timeout |
| sympy `"x^2 = (x+1)*(x-1) + 1"` | Pass |
| sympy `"1 = 2"` | Fail + evidence |
| invalid Rust candidate | compile_rust Fail |
| npm `react` | registry_check Pass |
| `react-confetti-fake-malicious` | Fail (anti-slopsquat) |
| regex match / mismatch | Pass / Fail |
| empty registry | `[]`, no panic |
| cloud A | `/task` math identity → Mode A → sympy Pass → CheckerPass |
| cloud B | fake npm install blocked → CheckerFail |

Deterministic smokes: files exist + executable; ≥6 TOMLs; sympy stdin JSON round-trip; three paper interfaces (`pot-2211.12588`, `codeact-2402.01030`, `alphaproof-nature-2025`). ≥10 tests.

## 13. Rebuild contract

Another harness has **this** organ iff:

- `.elai_cc/checkers/` layout with six executable checkers + six registration TOMLs as named.
- `CheckerResult` triad Pass / Fail{evidence} / Inapplicable{reason}.
- `load` / `applicable` (type+domain) / `run` (first applicable, timeout, JSON parse → Inapplicable).
- Subprocess JSON stdin/stdout, exit 0 on the triad.
- compile_* under Tier-C sandbox, not a Python reliability_guard.
- Production registry_check is live + 24-month freshness.
- Mode A dispatch emits CheckerDispatched + one terminal row.
- Register-time reference-fixture gate (J-11).
- Fail demotes overall verdict; no averaging (J-11).
- Exactly one reformulation on first Fail; second Fail = Ungrounded (J-15).
- `ELAI_CHECKER_ENABLE` default-OFF isolation is allowed; Mode A must not silently pass when the flag is off (fail closed / Inapplicable, not fake Pass).

An LLM that “looks like sympy” is a cousin.

## 14. Non-goals / banned designs

- Lean, TS compile/test, schema_validate, networkx, perf harnesses, ensemble voting (`D-plan/D-11_constructive_checker_registry.md:318-323`).
- PyO3 in-process Python.
- Averaging construction Fail with proof score.
- Unbounded reformulation loops.
- Using ComBench `reliability_guard` as sandbox (`:361-367`).
- Cached mock registry in production.

## 15. Source citations

Primary: `D-plan/D-11_constructive_checker_registry.md` (full file, including J-11 `:330-367` and J-15 `:369-404`). Index: `D-plan/D-SUMMARY_INDEX.md:86-90`. Feature map: `ELAI-FEATURE-MAP.md:585-589`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:232-236`. Exhaustive: D0-015, D0-046, D004-D11-001..061. Live: `checker_registry.rs`, `runtime_config.rs:1270-1298`, `task_runner.rs:3730-3746`, `wire-dark-allowlist.toml:821-826`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- J-15 talks about CheckerResult `info`; declared Fail has `evidence`, not `info` (D004-D11-052).
- “First applicable” has no stable ordering (D004-D11-053).
- Reference-fixture path is required by J-11 but absent from `CheckerEntry`/TOML (D004-D11-054).
- Demotion cap value, per-checker config, aggregation type, events (D004-D11-055). 0.3 is an example only.
- Pass-after-reformulation vs construction-cap interaction (D004-D11-056).
- NL→Z3 translation grammar (D004-D11-057).
- Invalid TOML, duplicates, exit 1, oversized evidence, tmp cleanup, crash (D004-D11-058).
- Live registry URL escaping, allowlist, rate-limit vs Tier-C no-network (D004-D11-059).
- J-11/J-15 added no extra tests/SSB/files (D004-D11-060).
- Reload, concurrent runs, cancellation, idempotency (D004-D11-061).
- Live `CheckerResult::Error` vs plan Inapplicable-on-infra.
- `Candidate` type is named, never defined in D-11.
