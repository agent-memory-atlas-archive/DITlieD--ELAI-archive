# D-11: Constructive-checker registry — Mode A operational substrate

Objective: bootstrap .elai_cc/checkers/{math,code,text,graph,api,perf}/ with
6 initial typed checkers (sympy, z3, compile-rust, compile-py, registry-check,
regex); typed CheckerResult; orchestrator dispatch logic. Mode A constructive
proof becomes operational. PoT (arxiv:2211.12588, T1 TMLR 2023) + CodeAct
(arxiv:2402.01030, T1 ICML 2024) + AlphaProof Nature 2025 (T1) — all three
demonstrate: when claim is operationalizable, EXTERNAL CHECKER is ground truth,
not LLM confidence.
Estimated effort: 3-4 sessions.
Pre-requisites: D-10 (mode routing exists; checkers called via Mode A
  Constructive). spec-grounder verifies DONE.
Parallel-safe with: D-12 (operationalization predicate consumes checker
  registry as its target).

## Context

Real-problem §4.2 (lines 199-238) defines the checker layout:
```
.elai_cc/checkers/
  math/{sympy.py, z3.py, lean.py}
  code/{compile_rust.sh, compile_py.sh, compile_ts.sh,
        test_rust.sh, test_py.sh, test_ts.sh}
  text/{regex_assertion.py, schema_validate.py}
  graph/{networkx.py}
  api/{registry_check.py}
  perf/{timing_harness.py, asymptotic_curve_fit.py}
```

Each checker has typed signature:
```rust
fn check(claim: Claim, candidate: Candidate, sandbox: SandboxHandle) -> CheckerResult {
    Pass,
    Fail { evidence: Vec<String> },
    Inapplicable { reason: String },
}
```

D-11 ships 6 INITIAL checkers (lean + ts + perf + schema_validate + networkx
deferred to future sessions; not blocking):
1. **sympy** (math): symbolic + numeric verification of equalities.
2. **z3** (math): SAT/SMT for finite domains.
3. **compile_rust** (code): cargo check on candidate.
4. **compile_py** (code): py_compile + ruff on candidate.
5. **registry_check** (api): npm/pypi/crates.io existence check (anti-
   slopsquatting per CLAUDE.md).
6. **regex_assertion** (text): regex match against expected pattern.

PoT (arxiv:2211.12588, T1) +12% over CoT on numerical benchmarks — Python
interpreter is ground truth. CodeAct (arxiv:2402.01030, T1) +20% with Python
as agent's unified action space; error messages close the loop. AlphaProof
Nature 2025 (T1) — IMO silver medal using Lean kernel as formal verifier (the
verifier is a separate, sound checker, not an LLM).

Partial foundation present and usable:
- .elai_cc/python/ scaffold from existing sidecar (FastAPI patterns).
- ELAI sandbox (Tier C from B-18 + C-11) provides the SandboxHandle.
- D-10 GroundingMode::Constructive dispatch hook awaits this implementation.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.2 checker layout — lines 199-238; §1 Mode A grounding — lines 36-50)
- .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10 dispatch site)
- .elai_cc/crates/orchestrator/src/claim.rs (D-10 Claim + ClaimType)
- .elai_cc/crates/sandbox/ (existing Tier C sandbox from B-18 / C-11)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2211.12588 Program-of-Thought (PoT)
- arxiv:2402.01030 CodeAct
- AlphaProof Nature 2025 doi:10.1038/s41586-025-09833-y

Verify workflow/papers/{pot-2211.12588,codeact-2402.01030,alphaproof-nature-
2025}-interface.md.

## Tasks

1. **CheckerResult + CheckerRegistry types** in new file
   .elai_cc/crates/orchestrator/src/checker_registry.rs:
   - `enum CheckerResult { Pass, Fail { evidence: Vec<BoundedString<2048>> },
     Inapplicable { reason: String } }`
   - `struct CheckerEntry { name: String, claim_types: Vec<ClaimType>,
     domain: Domain, executable_path: PathBuf, language: CheckerLanguage,
     timeout_secs: u32 }`
   - `enum CheckerLanguage { Python, Shell, Rust, Js }` (closed enum)
   - `struct CheckerRegistry { entries: Vec<CheckerEntry> }`
   - `impl CheckerRegistry`:
     - `load(root: &Path) -> Self`: walk root/*.toml registration files;
       each TOML describes one CheckerEntry.
     - `applicable(claim: &Claim) -> Vec<&CheckerEntry>`: filter entries by
       claim_types contains claim.claim_type AND domain match.
     - `run(claim: &Claim, candidate: &Candidate, ctx: &OrchContext) ->
       CheckerResult`: dispatch first applicable; sandbox-execute; parse
       stdout JSON to CheckerResult; on timeout return Inapplicable with
       reason="timeout".

2. **Registration TOML format**:
   - .elai_cc/checkers/<domain>/<name>.toml describes the checker:
     ```toml
     name = "sympy"
     claim_types = ["math"]
     domain = "math"
     executable_path = ".elai_cc/checkers/math/sympy.py"
     language = "python"
     timeout_secs = 30
     ```

3. **sympy.py checker** (.elai_cc/checkers/math/sympy.py, new file):
   - Reads JSON from stdin: `{ "claim": "x^2 = (x+1)*(x-1) + 1", "candidate":
     null }` (claim_type=math, identity check).
   - Parses LHS + RHS as SymPy expressions; calls sympy.simplify(LHS - RHS);
     Pass if result is 0; Fail if non-zero with evidence = stringified
     simplified diff; Inapplicable on parse error.
   - Stdout: JSON `{ "result": "Pass" }` or `{ "result": "Fail", "evidence":
     ["..."] }`.
   - Exit 0 on Pass/Fail (Inapplicable too); exit 1 only on script-level
     error.
   - Verify sympy in pyproject.toml; if absent, document install
     requirement; CLAUDE.md anti-stale rule requires version pinning.

4. **z3.py checker** (.elai_cc/checkers/math/z3.py, new file):
   - For SAT/SMT claims like "for all x, y in Z: x + y = y + x", uses
     z3-solver Python package.
   - Translates claim to Z3 formula; calls z3.Solver.check; Pass if unsat
     on negation; Fail with counterexample evidence.
   - Verify z3-solver in pyproject.toml.

5. **compile_rust.sh checker** (.elai_cc/checkers/code/compile_rust.sh,
   new file):
   - Reads JSON from stdin: `{ "candidate": "<rust file content>", "kind":
     "compile" }`.
   - Writes candidate to tmpdir/lib.rs; runs `cargo check` (Tier C sandbox);
     Pass if exit 0; Fail with stderr snippet as evidence.

6. **compile_py.sh checker** (.elai_cc/checkers/code/compile_py.sh,
   new file):
   - Analogous: `python -m py_compile` + `ruff check` on candidate.

7. **registry_check.py checker** (.elai_cc/checkers/api/registry_check.py,
   new file; ANTI-SLOPSQUATTING per CLAUDE.md):
   - Reads JSON: `{ "package": "react-confetti", "registry": "npm" }`.
   - Per CLAUDE.md verification protocol:
     - npm: `npm view <pkg> version dist-tags time`; check for `latest` tag
       with publish date in last 24 months.
     - pypi: `curl -s https://pypi.org/pypi/<pkg>/json`; parse for version +
       license + recent release.
     - crates.io: `curl -s https://crates.io/api/v1/crates/<pkg>`.
   - Pass if registry hit + recent release; Fail with reason "package not
     found" or "stale (last release > 24 months)"; Inapplicable if registry
     unknown.

8. **regex_assertion.py checker** (.elai_cc/checkers/text/regex_assertion.py,
   new file):
   - Reads JSON: `{ "candidate": "<text>", "pattern": "<regex>",
     "match_mode": "full"|"contains" }`.
   - Compiles pattern; tests against candidate; Pass on match; Fail with
     position of mismatch; Inapplicable on invalid regex.

9. **Orchestrator integration in dispatch_by_mode** (D-10
   grounding_mode.rs):
   - GroundingMode::Constructive branch calls
     CheckerRegistry::run; emit CheckerDispatched +
     CheckerPass/Fail/Inapplicable SSB rows.

10. **SSB action types** in ssb.rs (SCHEMA_VERSION 56→57):
    - `CheckerDispatched { claim_id: Uuid, checker_name: String, audit_id: Uuid }`
    - `CheckerPass { claim_id: Uuid, checker_name: String, latency_ms: u64 }`
    - `CheckerFail { claim_id: Uuid, checker_name: String, evidence: Vec<String> }`
    - `CheckerInapplicable { claim_id: Uuid, checker_name: String, reason: String }`
    - Round-trip tests.

11. **Unit tests** in checker_registry.rs:
    - applicable(math claim) returns sympy + z3 entries.
    - applicable(api claim) returns registry_check.
    - run() with mock checker stdout pass → CheckerResult::Pass.
    - run() with timeout → CheckerResult::Inapplicable.

12. **Integration tests** in
    .elai_cc/crates/orchestrator/tests/checker_dispatch_integration.rs:
    - Math claim "x^2 = (x+1)(x-1) + 1" → sympy Pass.
    - Math claim "1 = 2" → sympy Fail with evidence.
    - Rust code claim with syntactically-invalid candidate → compile_rust
      Fail.
    - npm package "react" → registry_check Pass.
    - npm package "react-confetti-fake-malicious" → registry_check Fail
      (anti-slopsquatting demonstration).

13. **Decision doc** workflow/reasoning/D-11-decisions.md (3 decisions):
    - (a) Why Python for sympy/z3 (not Rust): Python ecosystem has mature
      SymPy + z3-solver bindings; reimplementing in Rust is wasted effort.
    - (b) Why subprocess + JSON stdin/stdout (not in-process Python via PyO3):
      subprocess isolates Python state; sandbox tier C constraints apply
      uniformly; matches existing python-extension pattern.
    - (c) Why registry_check is anti-slopsquatting: CLAUDE.md HARD RULE +
      arxiv:2507.05619 cryptographic-checksum spirit — verify package
      existence before agent imports unknown deps.

## WIRING_CONTRACT

produces_symbols:
  - CheckerResult + CheckerEntry + CheckerLanguage + CheckerRegistry
  - applicable + run methods
  - Six checker scripts (sympy.py, z3.py, compile_rust.sh, compile_py.sh,
    registry_check.py, regex_assertion.py)
  - SsbActionType::CheckerDispatched + ::CheckerPass + ::CheckerFail +
    ::CheckerInapplicable + payloads
  - SCHEMA_VERSION bump 56→57
  - .elai_cc/checkers/ directory layout + registration TOML files
consumes_symbols:
  - GroundingMode::Constructive dispatch (D-10)
  - Tier C sandbox (B-18 + C-11)
  - Claim + ClaimType (D-10)
  - Domain enum (D-1 + D-7)
reachable_from:
  - GroundingMode dispatch path
  - elai_cc.toml [checkers] config (optional registration paths)
stub_ban:
  - run() MUST timeout per checker.timeout_secs; no infinite wait
  - registry_check MUST hit live registry (no cached mock in production)
  - applicable() MUST handle empty registry (return empty Vec, not panic)
  - Each checker stdout MUST be valid JSON; CheckerResult parse failure
    returns Inapplicable, not panic
observable_effect:
  trigger: 6 synthetic claims, one per checker type
  expect_ssb: 6 CheckerDispatched rows + per-checker Pass/Fail/Inapplicable
              rows
  expect_artifact: .elai_cc/checkers/ contains 6 executable checker scripts;
                   each script's stdout is parseable JSON

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 57
rg "SCHEMA_VERSION: i64 = 57" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "CheckerDispatched|CheckerPass|CheckerFail|CheckerInapplicable" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 16

# 3. CheckerRegistry types
rg "struct CheckerRegistry|enum CheckerResult" \
  .elai_cc/crates/orchestrator/src/checker_registry.rs

# 4. 6 checker files present + executable
for f in math/sympy.py math/z3.py code/compile_rust.sh code/compile_py.sh \
         api/registry_check.py text/regex_assertion.py; do
  test -f ".elai_cc/checkers/$f" && echo "$f OK"
done

# 5. 6 registration TOMLs
ls .elai_cc/checkers/*/*.toml | wc -l
# Expected: >= 6

# 6. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- checker 2>&1 | grep "test result"

# 7. Sympy round-trip smoke
echo '{"claim":"x^2 = (x+1)*(x-1) + 1"}' | python .elai_cc/checkers/math/sympy.py | python -c "import sys,json; print(json.load(sys.stdin))"
# Expected: {"result": "Pass"}

# 8. Registry-check anti-slopsquatting smoke
echo '{"package":"react","registry":"npm"}' | python .elai_cc/checkers/api/registry_check.py
# Expected: Pass

# 9. Paper-ingestor outputs
test -f workflow/papers/pot-2211.12588-interface.md && \
  test -f workflow/papers/codeact-2402.01030-interface.md && \
  test -f workflow/papers/alphaproof-nature-2025-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 57
- [ ] 4 SSB variants round-trip
- [ ] CheckerRegistry compiles; unit tests pass
- [ ] 6 checker scripts present + executable
- [ ] 6 registration TOMLs valid; load() parses
- [ ] Sympy passes on canonical identity; fails on false equality
- [ ] Z3 passes on tautology; fails on false statement
- [ ] compile_rust + compile_py Pass on valid code; Fail with evidence on
      syntax error
- [ ] registry_check Pass on "react" / "numpy" / "tokio"; Fail on synthetic
      "react-confetti-fake-malicious-xyz123"
- [ ] regex_assertion Pass on matching pattern; Fail on mismatch
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- checker: all pass, ≥10 new tests
- [ ] Cloud smoke test A: cloud /task math claim "x^2 = (x+1)*(x-1) + 1" →
      Mode A dispatched → sympy Pass → CheckerPass SSB row
- [ ] Cloud smoke test B: cloud /task fake npm package; agent attempts npm
      install → registry_check intervenes → CheckerFail; install blocked
- [ ] All 3 paper-ingestor outputs present
- [ ] Decision doc workflow/reasoning/D-11-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/checker_registry.rs
- ADD: .elai_cc/checkers/math/sympy.py + .toml
- ADD: .elai_cc/checkers/math/z3.py + .toml
- ADD: .elai_cc/checkers/code/compile_rust.sh + .toml
- ADD: .elai_cc/checkers/code/compile_py.sh + .toml
- ADD: .elai_cc/checkers/api/registry_check.py + .toml
- ADD: .elai_cc/checkers/text/regex_assertion.py + .toml
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 56→57;
    4 new variants)
- MODIFY: .elai_cc/crates/orchestrator/src/grounding_mode.rs (D-10 dispatch
    integration)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod checker_registry)
- MODIFY: .elai_cc/pyproject.toml (sympy, z3-solver dep declarations)
- ADD: .elai_cc/crates/orchestrator/tests/checker_dispatch_integration.rs
- ADD: workflow/reasoning/D-11-decisions.md
- ADD: workflow/papers/{pot-2211.12588,codeact-2402.01030,alphaproof-nature-
       2025}-interface.md

## Out-of-scope

- Lean (math/lean.py) — DEFERRED; high implementation cost, narrow domain
- TypeScript (code/compile_ts.sh + test_ts.sh) — DEFERRED to dashboard slot
- schema_validate.py + networkx.py + timing_harness.py — DEFERRED
- Multi-checker ensemble voting (DEFERRED; single applicable checker first)

## Next

D-12 (OperationalizationCheck) consumes the checker registry as its target —
every claim-emitting baby step must reference an applicable checker.

## J-11 AMENDMENT — S-7 ComBench CHECKER CONTRACT PRIOR ART (source-cited)
Grounding: ComBench `src/eval/prime_code_executor.py:19-43`, `src/eval/extractor.py:8-61`,
`src/eval/score_parsing.py:1-29`, `src/eval/evaluator.py:274-305`, `data_process/check_ref_construction.py:108-157`,
`src/prime_code/testing_util.py:676-694`, read directly per W-3.
ComBench's verifier architecture is a working existence proof of the checker contract D-11 needs. Three
stealable contract elements (GD-1 prior-art pointer, compose not restate):

### 1. Minimal reusable verifier contract (`prime_code_executor.py:19`)
`run_prime_code_verifier(verify_code: str, stdin_text: str, timeout: int) -> PrimeCodeExecutionResult`:
the verifier is a self-contained script that receives the candidate answer on stdin and prints "True" on pass
(`:20` test_cases = {"inputs": [stdin_text], "outputs": ["True"]}, `:36` passed = score == 1.0). The contract
is: (stdin payload, expected stdout) → (passed: bool, score: float, info: str, metadata: dict). D-11's
checker trait mirrors this: deterministic input → deterministic pass/fail + evidence, no LLM in the path
(TD-3). The `info` field (`:46` _build_info) carries the failure reason (status/error_message/traceback) —
this is the evidence the checker returns to the gate.

### 2. Register-time reference-fixture gate (`check_ref_construction.py:108-157`)
At dataset-build time, the REFERENCE answer is run through the verifier (`:144` run_prime_code_verifier on
verify_code + ref_construction). If the reference fails, the BUILD FAILS CLOSED (`:209` sys.exit(1)). This is
D-11's register-time gate: a checker that cannot pass its own reference fixture is REJECTED at registration
time, not at runtime. D-11's registry must run each checker against its bundled reference fixture before
admitting it to the registry — a checker that fails its own reference is a broken checker, not a deployed one.

### 3. Demotion clause — averaging is NOT demotion (`evaluator.py:274-305`, M-2)
ComBench's `_build_case_result` (`:274`) averages the non-skipped component scores (`:287`
`total_score = sum(normalized_score) / len(applicable)`). This is a **flaw** D-11 must explicitly refuse: a
construction failure (code doesn't compile) does NOT demote the proof score — it's averaged in, diluting the
signal. D-11's checker registry must support a **demotion clause**: if a checker returns Fail, the overall
verdict is at most the checker's demotion cap (e.g., construction Fail => overall ≤ 0.3 regardless of proof
score), NOT an average. A construction failure that averages to 0.5 with a perfect proof is a false pass.

### DO-NOT-COPY — ComBench's reliability_guard is NOT a sandbox (`testing_util.py:676`)
ComBench's `reliability_guard` (`:676`) explicitly warns (`:682`): "This function is NOT a security sandbox.
Untrusted code, including model-generated code, should not be blindly executed outside of one." D-11's
compile_rust.sh / compile_py.sh checkers MUST run under Tier-C sandbox (crates/agent + crates/ipc-types),
NOT under reliability_guard. The guard disables destructive functions (`:688` resource.setrlimit) but does
not prevent network access, filesystem writes outside the limit, or process spawning. ELAI's sandbox is
seccomp + SO_PEERCRED + cap-drop (C-11/C-13) — use that, not a Python function that warns it isn't a sandbox.

## J-15 AMENDMENT — Logic-LLM solver-error-feedback bounded reformulation loop (source-cited)
Grounding: determinism triage ledger footnote rider + agent det-repos-H4 verdict
(workflow/reasoning/steal-determinism-repos-2026-06-11.md); teacherpeterpan/Logic-LLM
solver-error-feedback pattern (the only stealable mechanism from that repo; the rest is
subsumed by D-11's existing checker contract + the J-11 ComBench amendment above).

### The steal: ONE bounded reformulation pass on checker FAIL
On a checker FAIL, the solver's error message (the `info` field from `CheckerResult` — see
J-11 amendment section 1: `status/error_message/traceback`) is fed back to the claimant for
ONE bounded reformulation pass BEFORE the claim is ruled UNGROUNDED. The mechanic:
1. Checker returns `Fail` with `info = "error message"` (deterministic, TD-3).
2. The error message is routed back to the claimant as a structured "your claim failed
   checker X for reason Y; reformulate" prompt (deterministic routing, not a free retry).
3. The claimant gets ONE reformulation attempt (bounded — not unbounded retries, not a loop).
4. The reformulated claim is re-checked by the SAME checker.
5. If the reformulated claim passes: the claim is `Verified` (the checker verdict is
   load-bearing throughout — TD-3). If it fails again: the claim is `Ungrounded` (two
   failures = the claim is wrong, not the checker).

### Bound: ONE pass, deterministic routing, checker verdict stays load-bearing
This is NOT an unbounded retry loop. It is ONE bounded reformulation pass with deterministic
routing (the error message is the sole feedback signal, the checker is the sole verdict
source). The checker's verdict is load-bearing at every step (TD-3): the reformulation does
NOT bypass the checker, it gives the claimant one chance to fix a fixable error. The bound
(ONE pass) prevents the claimant from gaming the checker through brute-force reformulation.
The routing is deterministic (no LLM decides whether to retry — the checker FAIL always
triggers one reformulation pass, the second FAIL always rules Ungrounded).

### Composes with the J-11 ComBench amendment (not a restatement)
The J-11 amendment (section 1) established the checker contract: deterministic input ->
deterministic pass/fail + evidence. This amendment adds the feedback loop: on FAIL, the
evidence (error message) is routed back for one reformulation pass. The two compose: the
checker contract produces the evidence; the feedback loop consumes it. Neither restates the
other (GD-1). The demotion clause (J-11 section 3) is UNCHANGED: a construction failure
still caps the overall verdict regardless of reformulation — the reformulation pass applies
to the FAILED checker's specific claim, not to the overall verdict calculation.
