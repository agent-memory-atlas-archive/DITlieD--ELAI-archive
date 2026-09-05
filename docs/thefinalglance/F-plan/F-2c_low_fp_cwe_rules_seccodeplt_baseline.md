# F-2c: four low-FP CWE rules (798/321/532/95) on registered Py+TS via the existing Composite engine, then a SeCodePLT baseline before any multi-lang or ast-grep-core expansion

STATUS: PLANNED (promoted to canonical F-plan 2026-06-02, s-fnext-f2-ruleset). Successor increment to the SHIPPED F-2a ruleset + F-2b bench harness (both confirmed in-tree at authoring time — see Existing state). This is a wiring-doctrine-conformant F-plan part; implementation pending (another session orchestrates it).

Objective: extend ELAI's already-operational `security` rule domain with FOUR additional high-precision, low-false-positive vuln-class predicates — hardcoded credentials (CWE-798), hardcoded cryptographic key (CWE-321), secrets-in-logs (CWE-532), and eval/exec injection (CWE-95) — scoped to the ALREADY-REGISTERED Python + TypeScript grammars, authored as `GrepPattern -> AstPattern` Composite predicates in the existing on-disk seed convention (the `data/rules/web.toml` Tier-2 pattern proves Composite/AST rules already work with ZERO new dependency). Then, and only then, RUN the already-shipped `elai bench seccodeplt` harness to produce the FIRST measured before/after static-gate insecure-rate + gate-recall baseline over the now-16-rule corpus. F-2c ships NO multi-language expansion (C/C++/JS/Swift) and adds NO external crate. This is the disciplined "smallest increment" of the F-2 keystone: more coverage on the registered languages first, a measured number second, multi-lang and ast-grep-core deferred until the baseline justifies them.

Estimated effort: 1 session (~5-7 h). Effort S-M: four TOML rule files + the contract-test row-count update + a baseline bench RUN + the decision-doc addendum. NO ssb.rs SCHEMA change (the two action types `security_rule_blocked` + `insecure_rate_measured` already ship — F-2a/F-2b). NO new crate. NO ast_adapter.rs edit.

Pre-requisites (ALL confirmed SHIPPED in-tree at authoring time — this increment attaches to live code, no hallucinated integration points per W-7):
- F-2a SHIPPED: the `security` domain ruleset is real and operational. 12 rules confirmed on disk: `crates/rules/seed/security/{sqli.toml(3, CWE-89), xss.toml(3, CWE-79), weak_crypto.toml(4, CWE-327), missing_authz.toml(2, CWE-862 WARN)}`. The seed path `ProjectRuleSeeder::seed_security_rules` (project_rule_seeder.rs:392) is wired LIVE at `runtime_setup.rs:322`; the builtin loader `load_security_seed_rules_builtin` (rule_loader.rs:142) is re-exported at lib.rs:1751.
- F-2b SHIPPED (refines the prior verdict, which framed the bench as if no infra existed): `crates/cli/src/bench_seccodeplt.rs::run` + `crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::SecCodePltRunner` + `bench_cmd.rs::run_bench_seccodeplt` are real; the bench drives the REAL static gate `vibegate::evaluate_vibegate` (vibegate.rs:133) and emits `SsbActionType::InsecureRateMeasured` (ssb.rs:8317, payload struct :25427, SCHEMA_VERSION 201). The before/after harness exists and has NOT yet been RUN to a published baseline — running it on the 16-rule corpus IS the cheapest measured-number step.
- The AST substrate is real and registered for Python + TS: `TreeSitterAstEvaluator::new()` (ast_adapter.rs:18-44) registers python + typescript (among 10 total) — so AstPattern rules for the F-2c CWE classes need NO ast_adapter edit. `data/rules/web.toml` (WEB-013..023) is the working Tier-2 Composite/AST template, zero new dep.
- The eval-order is locked and real: MetricGate(0) -> VersionGate/GrepPattern(1) -> AstPattern/SchemaRequired(2) -> Composite(3) -> GraphReachability(4) (`rule_engine.rs:328-339`, doc-commented at :328). A `GrepPattern AND AstPattern` Composite is evaluated as a Tier-3 node whose children run cheap-grep-first, exactly the FP-bounding shape F-2c wants.

Parallel-safe with: any F-N that does NOT edit `crates/rules/seed/security/*.toml`, `crates/rules/src/project_rule_seeder.rs::seed_security_rules`, or `crates/contracts/tests/f2_immune_rule_contract.rs` in the same window. NOT parallel-safe with F-10 (security-monotonicity classifier) if F-10 also touches the `security` seed dir or the immune-rule row count. NOT parallel-safe with any plan editing `crates/orchestrator/src/ssb.rs` SsbActionType enum — but F-2c does NOT add an SSB variant, so the only collision is the row-count constant in the immune-rule contract test (a 12->16 update). Coordinate that single constant with any concurrent security-seed editor.

Source synthesis: the prior F-2 increment (`docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md`, item C2) and its grounding `workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` section 4 item C2 (mechanism, REVISED SCOPE, the no-perf-claim posture for the ruleset and the gated headline for the measured number). Doctrine: `docs/thefinalglance/F-plan/F-0_INDEX.md` W-1..W-10. F-2c inherits the section-1.4 corrections verbatim (NO "below 15%" target, NO GitGuardian-replay framing — the bench uses its OWN synthetic corpus).

## Context

ELAI's `security` domain is already operational, not a stub. The rules engine carries `ALWAYS_INCLUDED_DOMAINS` so a `security`-domain rule is evaluated on every phase; the `Predicate` enum (predicate.rs:101) ships `AstPattern`, `GrepPattern` (with a `FileContext::{Production,Test}` filter), and `Composite` (predicate.rs:139, `operator` + `children`); the gate `vibegate::evaluate_vibegate` (vibegate.rs:133) runs the compiled rules at write-time; a BLOCK verdict aborts the write through the PreToolUse path; every block is journaled to SSB as `security_rule_blocked` and is reproducible under `ELAI_SEED`. F-2a put 12 rules there (sqli/xss/weak-crypto BLOCK + authz WARN). F-2b shipped the `elai bench seccodeplt` harness that drives the REAL gate offline and emits an `insecure_rate_measured` before/after row.

The gap F-2c closes is COVERAGE on the languages already registered, plus the FIRST published baseline number — NOT multi-language reach. The four CWEs added here are the highest-ROI, lowest-false-positive classes still uncovered: hardcoded credentials (CWE-798), hardcoded crypto key (CWE-321), secrets/credentials written to logs (CWE-532), and eval/exec code-injection (CWE-95). Each is expressible as a high-precision `GrepPattern -> AstPattern` Composite on Python + TS, mirroring `web.toml`'s Tier-2 rules byte-for-byte in convention. None requires a new grammar, a new crate, or an `ast_adapter.rs::new()` edit.

The measured number is the keystone (ELAI-DIRECTION R1: one real number on ELAI's own engine; R2: the anti-Replit demo "the gate physically blocked the SQLi/secret before it was written, here is the before/after rate"). F-2b built the harness; nobody has RUN it to a published baseline. F-2c's second half is exactly that run: the >=40-sample synthetic SeCodePLT-structured corpus through the REAL `evaluate_vibegate` gate over the now-16-rule ruleset, under `ELAI_SEED`, emitting one `insecure_rate_measured` row + a `seccodeplt-<unix_ms>.json` report. This is a STATIC-gate insecure-rate (the gate's false-negative residual), explicitly DISTINCT from SeCodePLT's dynamic sandbox-executed metric and from any live-model number — the live-Qwen run stays a separately-gated RunPod checkbox per W-5/W-10, NOT a claim made here.

Why this scope and not more (the disciplined increment, per the prior verdict's killer-risks):
- The prior verdict's headline correction: `TreeSitterAstEvaluator::new()` registers exactly 10 languages (rust/python/typescript/go/java/c_sharp/ruby/html/css/svelte; astro behind `web-astro`), NOT "16 grammars wired" (ast_adapter.rs:18-44, read at authoring time — CONFIRMED). JavaScript/C/C++/Swift/Kotlin/Scala/PHP/Bash are pinned in `crates/knowledge/Cargo.toml` (lines 14-47) but UNREGISTERED in the evaluator. A Tier-2 AST rule for those languages requires editing `ast_adapter.rs::new()` + verifying each grammar's ABI — it is NOT a free registry insert. F-2c therefore scopes ALL new rules to the already-registered Python + TS and DEFERS multi-language expansion to a separate F-N.
- The prior verdict's second correction: `ast-grep-core` (Approach B) would add a new MIT crate whose only win over the existing engine is a richer YAML DSL. F-2c uses the EXISTING engine (Approach A — zero new dep) for all four rules. ast-grep-core is DEFERRED; see Open questions for its verified registry status.
- The prior verdict's efficacy caveat (correct): no SeCodePLT run has produced FP/recall numbers, so any "reduces FP" / "adds languages" claim is UNGROUNDED until measured. F-2c makes NO efficacy claim for the four rules; it RUNS the baseline to produce the number, and labels it explicitly as ELAI's own synthetic-corpus static-gate metric.

## Existing state (B/C/D/E coverage)

- F-2a (SHIPPED): `crates/rules/seed/security/{sqli,xss,weak_crypto,missing_authz}.toml` — 12 rules total (3 CWE-89 BLOCK, 3 CWE-79 BLOCK, 4 CWE-327 BLOCK, 2 CWE-862 WARN), Python + TS, `file_context = "production"` to exclude test code. Seeded `mutable = 0` (immune) via `ProjectRuleSeeder::seed_security_rules` (project_rule_seeder.rs:392), wired LIVE at runtime_setup.rs:322. The immune-rule contract `crates/contracts/tests/f2_immune_rule_contract.rs:70` asserts "seed_security_rules must insert exactly 12 rows" — F-2c updates this constant to 16 (the only test-count edit).
- F-2b (SHIPPED): `crates/cli/src/bench_seccodeplt.rs::run` (driver), `crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::SecCodePltRunner` (runner), `bench_cmd.rs::run_bench_seccodeplt` (dispatcher arm). Drives the REAL gate `vibegate::evaluate_vibegate`; computes before = corpus ground-truth insecure-rate, after = the gate's static false-negative residual, gate_recall = (before - after)/before; emits one `SsbActionType::InsecureRateMeasured` row (ssb.rs:8317; payload `InsecureRateMeasuredPayload` ssb.rs:25427; SCHEMA_VERSION 201) + a `seccodeplt-<unix_ms>.json` report. NOT yet run to a published baseline.
- Rules engine (`crates/rules`, SHIPPED): `Predicate` 7-variant enum (predicate.rs:101 — AstPattern:102, GrepPattern:109, SchemaRequired:121, MetricGate:126, Composite:139, GraphReachability:144, VersionGate:158); `eval_order` (rule_engine.rs:329) locks MetricGate(0)->GrepPattern/VersionGate(1)->AstPattern/SchemaRequired(2)->Composite(3)->GraphReachability(4); `detect_language_from_ext` (rule_engine.rs:346-362) maps `.py/.ts/.tsx` -> python/typescript (F-2c's target langs are covered) but returns None for `.svelte/.html/.css` (gap CONFIRMED real, OUT OF F-2c scope — flagged in Open questions). `NoOpAstEvaluator` (rule_engine.rs:912) is inside `#[cfg(test)]` (mod at :902) — test-only, NOT the production evaluator.
- AST evaluator (`crates/knowledge`, SHIPPED): `TreeSitterAstEvaluator` (ast_adapter.rs:14) is a genuine tree-sitter impl (set_language + Query::new + cursor.matches at :53-102), NOT a stub. `::new()` (ast_adapter.rs:18-44) registers 10 languages; python + typescript are both present, so F-2c's AST children need NO evaluator edit.
- `data/rules/web.toml` (SHIPPED): WEB-013..023 are real Tier-2 AST predicates against tree-sitter-html/css/svelte-ng, proving Composite/AST rules load + evaluate today with zero new dependency. This is F-2c's authoring template.
- E-12 (post-write semgrep, PLANNED/partial): runs CWE detection AFTER the file is written. F-2c is the PRE-write complement — same orthogonality F-2a already established; no overlap.
- F-10 (security-monotonicity iteration guard, PLANNED): per-iteration CWE-class regression guard + Rust coverage. F-2c does NOT touch the monotonicity axis and does NOT widen to Rust; the two are distinct axes. Coordinate only on the `security` seed-dir row count if both land in the same window.

## Reads (mandatory before coding)

- `docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md` (the predecessor — the ruleset convention, the no-perf-claim posture, the Tier-1/Tier-2 split, the section-1.4 corrections to NOT re-introduce)
- `docs/thefinalglance/F-plan/F-0_INDEX.md` (W-1..W-10 doctrine — the done-gate this file is written to satisfy)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\seed\security\weak_crypto.toml` + `sqli.toml` + `xss.toml` (the EXACT on-disk `[[rules]]` shape F-2c mirrors: rule_id, domain, tier, severity, title, predicate_json, phase_mask)
- `F:\.ELAI_workflow\.elai_cc\data\rules\web.toml:1-120` (the working Composite/AstPattern Tier-2 template — confirm the `predicate_json` AST + Composite serde shape before authoring)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:88-160` (the `Predicate` enum: AstPattern + GrepPattern + Composite + FileContext serde tags — `#[serde(tag = "type", rename_all = "snake_case")]`)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:320-365` (eval_order + detect_language_from_ext — confirm `.py/.ts` covered, Composite tier-3 ordering)
- `F:\.ELAI_workflow\.elai_cc\crates\rules\src\project_rule_seeder.rs:380-545` (seed_security_rules — the loader the new TOMLs plug into; confirm it enumerates the seed/security dir so 4 new files load with NO loader change)
- `F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\f2_immune_rule_contract.rs:60-75` (the 12-row count assertion to update to 16)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\bench_seccodeplt.rs:1-120` + `crates\orchestrator\src\benchmarks\seccodeplt_runner.rs` (the REAL bench path to RUN for the baseline — confirm the corpus + the before/after/gate_recall computation before running)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\ast_adapter.rs:18-44` (CONFIRM only 10 langs registered + python/typescript present — the basis for the "no ast_adapter edit needed, multi-lang deferred" decision)
- `F:\.ELAI_workflow\workflow\papers\seccodeplt-interface.md` (the SeCodePLT corpus design reference, if present — the synthetic-corpus methodology)

## Tasks

### TIER 1 — F-2c-i: four low-FP CWE rules on registered Python + TS (in-process, fail-closed, NO new dep, NO ast_adapter edit, NO perf claim)

1. **Create the four CWE rule files under the EXISTING `crates/rules/seed/security/` dir (~140 LOC across 4 files).** Mirror the exact on-disk convention of `weak_crypto.toml` / `sqli.toml` (CONFIRMED present): each `[[rules]]` carries `rule_id`, `domain = "security"`, `tier = 1`, `severity`, `title` (with the CWE id in the text), `predicate_json`, `phase_mask = 4294967295`. Every `predicate_json` is validated against the real `Predicate` serde shape (`#[serde(tag = "type", rename_all = "snake_case")]`). The four files, all SCOPED to the already-registered Python + TypeScript:
   - `seed/security/hardcoded_creds.toml` — string literal assigned to a `password`/`passwd`/`secret`/`api_key`/`token`/`apikey` identifier (CWE-798). Composite `GrepPattern AND AstPattern`: the grep child anchors the suspicious identifier-equals-string-literal shape cheaply; the AST child confirms it is an assignment to a string literal (NOT a comparison, NOT a function call arg read from env) to bound the FP surface. Python + TS variants. `file_context = "production"`.
   - `seed/security/hardcoded_key.toml` — a literal byte/hex/base64 key passed to a crypto constructor (`AES.new`, `Fernet(`, `createCipheriv`, `crypto.subtle.importKey` with a literal) (CWE-321). Composite, anchored to the crypto sink + a literal-key argument; complements (does NOT duplicate) `weak_crypto.toml` which gates the ALGORITHM, not the KEY-PROVENANCE. Python + TS. `file_context = "production"`.
   - `seed/security/secrets_in_logs.toml` — a `password`/`secret`/`token`/`api_key` identifier interpolated into a logging sink (`logging.info/debug/warning`, `logger.*`, `console.log/info/error`, `print(` with a secret var) (CWE-532). Composite: grep the log-sink call, AST-confirm a secret-named identifier in the argument. WARN (not BLOCK) for the Python `print(` shape (higher FP); BLOCK for explicit `logging`/`logger`/`console` sinks where the signal is high-precision. Python + TS.
   - `seed/security/eval_injection.toml` — `eval(`/`exec(`/`compile(` on a non-literal (Python) and `eval(`/`new Function(`/`setTimeout(`-with-string (TS/JS-via-ts) (CWE-95). Composite: grep the eval/exec sink, AST-confirm the argument is NOT a string literal constant (a literal `eval("1+1")` is not injection; `eval(user_input)` is). BLOCK — eval-on-dynamic is a high-precision class. Python + TS.
   Each `predicate_json` cites its CWE + the SeCodePLT class in a TOML comment, exactly as the existing four files do. Use the `Composite` operator form already serialized by `web.toml` (confirm the `{"type":"composite","operator":"and","children":[...]}` shape against predicate.rs:139 before writing).

2. **Update the immune-rule row-count contract from 12 to 16 (~2 LOC delta in `crates/contracts/tests/f2_immune_rule_contract.rs`).** The test at :70 asserts `seed_security_rules` inserts exactly 12 rows; with four new files (each carrying multiple `[[rules]]`) the count rises. Compute the exact post-add total from the authored TOMLs (12 existing + the F-2c rules) and set the constant + the assertion message. This is the ONLY existing-test edit; it is a count constant, not a logic change. (Per W-7, the new total is named at edit time, not guessed in this draft.)

3. **Confirm the four new files load through the EXISTING seed path with NO loader change (~0 LOC, verification task).** `seed_security_rules` (project_rule_seeder.rs:392) enumerates the `seed/security/` directory; if it enumerates the dir (confirm at read time), the four new TOMLs are picked up automatically and seeded `mutable = 0` + blake3-integrity-checked like the existing 12. If — and only if — it hard-codes the four existing filenames, add the four new filenames to that list (a small explicit delta). Resolve which at read time; the WIRING_CONTRACT names `seed_security_rules` as the consumed loader either way.

### TIER 1 — F-2c-ii: RUN the SeCodePLT baseline over the 16-rule corpus (in-process, deterministic, the measured number — RunPod-INDEPENDENT)

4. **RUN the already-shipped bench to produce the FIRST published static-gate baseline (~0 new LOC; this is an EXECUTION + a decision-doc writeup).** Invoke `elai bench seccodeplt --json` (the F-2b driver `bench_seccodeplt.rs::run`) over the >=40-sample synthetic corpus through the REAL `vibegate::evaluate_vibegate` gate now carrying 16 rules, under a fixed `ELAI_SEED`. This emits one `insecure_rate_measured` SSB row `{suite, model_id, n_samples, insecure_rate_before, insecure_rate_after, gate_recall, seed}` + a `seccodeplt-<unix_ms>.json` report. Capture before/after/gate_recall as the FIRST measured number. CAVEAT (mandatory, section 1.4): this is ELAI's OWN synthetic corpus, a STATIC-gate insecure-rate (the gate's false-negative residual), DISTINCT from SeCodePLT's dynamic sandbox metric and from SVEN/CodeGuard+ CodeGen-2.7B numbers. NO transfer claim. NO live-model claim (the live Qwen run stays a separate RunPod-gated checkbox per W-5/W-10 — NOT made here).
   - If the corpus does not yet exercise the four new CWE classes (798/321/532/95), add fixtures for them to the existing `seccodeplt_fixtures/` so the baseline reflects the 16-rule ruleset (each fixture is a `{prompt, cwe, secure_oracle_pattern}` triple — the EXISTING corpus shape; no new harness code).

5. **Decision doc `workflow/reasoning/F-2c-decisions.md` (~100 LOC).** Records: (a) the four-CWE scope bound + why these four (highest-ROI, lowest-FP still-uncovered classes); (b) the EXPLICIT multi-language DEFER with the grounded reason (ast_adapter.rs:18-44 registers 10 langs not 16 — C/C++/JS/Swift Tier-2 needs an evaluator edit + per-grammar ABI verification, OUT OF SCOPE); (c) the ast-grep-core DEFER with its verified registry status (0.43.0, MIT, last published 2026-05-25, crates.io — clean but unneeded for this batch; the only win is a richer DSL); (d) the measured static-gate baseline (before/after/gate_recall/n_samples/seed) with the synthetic-corpus + static-vs-dynamic caveat; (e) the section-1.4 corrections kept out (no "below 15%", no GitGuardian-replay); (f) the W-10 statement: F-2c is ENTIRELY Tier-1 in-process (no network, no subprocess, no RunPod at gate time OR at bench time — the bench runs offline against the static gate); the live-model run is a separately-gated future checkbox, not part of F-2c.

## WIRING_CONTRACT

produces_symbols:
  # F-2c adds NO new Rust symbols and NO new SSB variant — it adds DATA (TOML rule
  # files) consumed by the SHIPPED loader, RUNs the SHIPPED bench, and updates one
  # contract-test constant. Every "produced" artifact below is a seed file consumed
  # by an existing loader, satisfying W-8 (produces/consumes symmetry) by construction.
  - crates/rules/seed/security/hardcoded_creds.toml
  - crates/rules/seed/security/hardcoded_key.toml
  - crates/rules/seed/security/secrets_in_logs.toml
  - crates/rules/seed/security/eval_injection.toml
  - workflow/reasoning/F-2c-decisions.md   (the measured-baseline writeup — the R1/R2 keystone artifact)

consumes_symbols:
  # ALL confirmed in source at authoring time (W-7) — file:line cited.
  - crates/rules/src/project_rule_seeder.rs::seed_security_rules        (CONFIRMED project_rule_seeder.rs:392; wired LIVE at runtime_setup.rs:322 — the production seed path that loads seed/security/*.toml mutable=0 + blake3-integrity)
  - crates/orchestrator/src/rule_loader.rs::load_security_seed_rules_builtin (CONFIRMED rule_loader.rs:142; re-exported lib.rs:1751 — the builtin loader)
  - crates/rules/src/predicate.rs::Predicate                            (CONFIRMED predicate.rs:101 — AstPattern:102, GrepPattern:109, Composite:139, the variants the new predicate_json deserializes to)
  - crates/rules/src/predicate.rs::FileContext                          (CONFIRMED predicate.rs:118 — the production/test scoping the new rules set to "production")
  - crates/rules/src/rule_engine.rs::eval_order                         (CONFIRMED rule_engine.rs:329 — locks Composite at tier 3, GrepPattern child at tier 1)
  - crates/rules/src/rule_engine.rs::detect_language_from_ext           (CONFIRMED rule_engine.rs:346-362 — maps .py->python, .ts/.tsx->typescript; the new rules target these)
  - crates/orchestrator/src/vibegate.rs::evaluate_vibegate             (CONFIRMED vibegate.rs:133 — the REAL static write-gate the new rules evaluate through and the bench drives)
  - crates/cli/src/bench_seccodeplt.rs::run                            (CONFIRMED bench_seccodeplt.rs:1 + run() — the SHIPPED bench driver RUN for the baseline)
  - crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::SecCodePltRunner (CONFIRMED imported at bench_seccodeplt.rs:52 — the SHIPPED runner)
  - crates/orchestrator/src/ssb.rs::SsbActionType::InsecureRateMeasured (CONFIRMED ssb.rs:8317; payload InsecureRateMeasuredPayload:25427; sql_str:9118; parse arm:9797 — the SHIPPED beacon the baseline emits)
  - crates/contracts/tests/f2_immune_rule_contract.rs                   (CONFIRMED f2_immune_rule_contract.rs:70 — the 12->16 row-count constant updated here)
  - crates/knowledge/src/ast_adapter.rs::TreeSitterAstEvaluator        (CONFIRMED ast_adapter.rs:14; ::new():18-44 registers python+typescript — the AST children resolve with NO evaluator edit)

reachable_from:
  # Every produced rule is reached over a REAL production chain at authoring time (W-3/W-7).
  - crates/orchestrator/src/runtime_setup.rs::seed_security_rules call site (CONFIRMED runtime_setup.rs:322 — seeds the seed/security/*.toml set, INCLUDING the 4 new files, mutable=0 on the sole production rules.db init path)
  - crates/orchestrator/src/vibegate.rs::evaluate_vibegate            (CONFIRMED vibegate.rs:133 — the PreToolUse static gate evaluates the security domain, including the 4 new rules, on every write-candidate; production entrypoint per W-3)
  - crates/cli/src/bench_seccodeplt.rs::run                           (CONFIRMED — `elai bench seccodeplt` CLI subcommand drives the 16-rule corpus through evaluate_vibegate; CLI production entrypoint per W-3)

stub_ban:
  - "TODO: add CWE rule" literal absent in the 4 new TOMLs
  - "// rule body later" / "placeholder pattern" literal absent
  - NO bypass branch: the new rules evaluate through the EXISTING vibegate path; there is no prompt-/env-controlled flag that disables the security domain (inherits F-2a's no-bypass guarantee) [W-2]
  - NO default-OFF env gate as the terminal state: the 4 rules are seeded + enforced by DEFAULT (no ELAI_*_ENABLE); the observable_effect MUST run with NO env override set [W-2]
  - NO mock/hard-coded baseline number: insecure_rate_before/after/gate_recall are COMPUTED by the real bench run, never literals; "insecure_rate_before == insecure_rate_after" hard-coded absent [section 1.4]
  - NO ast_adapter.rs::new() edit in this F-N (multi-lang is DEFERRED; if a diff touches ast_adapter.rs::new the scope has crept — STOP)
  - NO new external crate (ast-grep-core DEFERRED; if Cargo.toml gains a dep the scope has crept — STOP)

frontend_pair:
  # W-4: the two SSB signals this increment touches (security_rule_blocked, insecure_rate_measured)
  # ALREADY have their frontend pair shipped under F-2a/F-2b. F-2c adds NO new SSB action_type and
  # therefore NO new frontend pair — it surfaces through the EXISTING SecurityRulesPanel.
  svelte: crates/dashboard-v2/src/lib/drawers/SecurityRulesPanel.svelte   (EXISTING — F-2a/F-2b; lists security_rule_blocked rows + the before/after insecure-rate tile; the 4 new rules' blocks + the new baseline render here with no UI change)
  tauri:  crates/dashboard-v2/src-tauri/src/commands_f2.rs::get_security_blocks_recent (+ get_insecure_rate_latest)  (EXISTING — F-2a/F-2b)
  justification: no new SSB variant => no new Rust<->TS DTO => cross_lang_diff has nothing new to check beyond the existing pair; the W-4 obligation is satisfied by the SHIPPED pair surfacing the new data.

observable_effect:
  trigger: "a PreToolUse Edit carrying a Python `password = \"hunter2\"` hardcoded-credential literal (CWE-798) into a production .py file is REFUSED at write-time via evaluate_vibegate; AND `elai bench seccodeplt --json` is RUN over the 16-rule corpus under ELAI_SEED"
  expect_ssb: "a security_rule_blocked row carrying the new rule_id (e.g. SEC-CREDS-001) + CWE-798 + a blake3 candidate digest appears in the live ssb.db from the write-time block; AND one insecure_rate_measured row with computed before/after/gate_recall + n_samples>=40 + the seed appears from the bench run"
  expect_artifact: "the write-time block leaves the candidate .py file UNWRITTEN and the FSM does NOT advance to EXECUTING for the refused step; the bench writes a report matching glob benchmarks/seccodeplt/seccodeplt-*.json containing /insecure_rate_(before|after)/ AND /gate_recall/; the measured before/after is recorded in workflow/reasoning/F-2c-decisions.md with the synthetic-corpus + static-gate caveat"

## DETERMINISTIC_CHECK

```bash
# W-1 wiring done-gate (the literal line; PASS required, NOT cargo-green-alone)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-2c_low_fp_cwe_rules_seccodeplt_baseline.md --run-trigger

# 1. the 4 new CWE rule files exist under the EXISTING security seed dir
test -f .elai_cc/crates/rules/seed/security/hardcoded_creds.toml
test -f .elai_cc/crates/rules/seed/security/hardcoded_key.toml
test -f .elai_cc/crates/rules/seed/security/secrets_in_logs.toml
test -f .elai_cc/crates/rules/seed/security/eval_injection.toml

# 2. each new rule carries the right CWE + domain + a real predicate_json (Composite/Grep/Ast)
rg "CWE-798|CWE-321|CWE-532|CWE-95" .elai_cc/crates/rules/seed/security/
rg "domain = \"security\"" .elai_cc/crates/rules/seed/security/hardcoded_creds.toml .elai_cc/crates/rules/seed/security/eval_injection.toml
rg "\"type\":\"(composite|grep_pattern|ast_pattern)\"" .elai_cc/crates/rules/seed/security/hardcoded_creds.toml

# 3. the rules are Python/TS-only (no unregistered grammar leaked in — multi-lang deferred)
! rg "\"language\":\"(c|cpp|swift|javascript|scala|php|bash|kotlin|go|java|ruby|rust)\"" .elai_cc/crates/rules/seed/security/{hardcoded_creds,hardcoded_key,secrets_in_logs,eval_injection}.toml

# 4. NO ast_adapter edit, NO new crate (scope-creep tripwires)
git diff --name-only | grep -qv "ast_adapter.rs" && echo "ast_adapter untouched OK" || echo "SCOPE CREEP: ast_adapter touched"
! git diff .elai_cc/crates/knowledge/Cargo.toml .elai_cc/crates/rules/Cargo.toml | rg "^\+.*ast-grep-core|^\+.*tree-sitter-"

# 5. the immune-rule row count updated 12 -> 16 (the only existing-test edit)
rg "must insert exactly 1[0-9] rows|16 rows" .elai_cc/crates/contracts/tests/f2_immune_rule_contract.rs

# 6. the baseline was RUN: an insecure_rate_measured row + a report exist (the measured number)
rg "insecure_rate_measured" .elai_cc/crates/orchestrator/tests/action_type_lock.golden   # variant already locked (F-2b)
ls benchmarks/seccodeplt/seccodeplt-*.json 2>/dev/null || echo "RUN elai bench seccodeplt to produce the baseline report"

# 7. blake3 (never blake2/sha2) on the candidate digest (inherited from F-2a)
! rg "blake2|sha2::Sha256" .elai_cc/crates/rules/seed/security/ 2>/dev/null

# 8. compile gate (necessary, NOT sufficient per W-1)
cd .elai_cc && cargo check --workspace
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-2c_low_fp_cwe_rules_seccodeplt_baseline.md --run-trigger` returns PASS (exit 0) — every produces artifact exists, every consumes_symbol has a real CALL/REF, every reachable_from anchor reached by BFS over CALL_RESOLVED within depth 8, stub_ban finds zero forbidden literals, AND the observable_effect trigger RAN and emitted the rows + artifact. cargo-green alone is NOT sufficient.
- [ ] **W-2 default-ON verification**: the observable_effect (the CWE-798 hardcoded-cred Edit is refused; `elai bench seccodeplt` runs) executes with NO `ELAI_*_ENABLE` / env override set — the 4 new rules are seeded + enforced by default.
- [ ] **F-2c-i four-rule BLOCK/WARN**: a Python `password = "hunter2"` candidate (CWE-798) is BLOCKED at write-time; a `secrets_in_logs` `print(password)` is WARN-class; an `eval(user_input)` (CWE-95) is BLOCKED; each produces a `security_rule_blocked` row with the right CWE + a blake3 digest; a clean candidate passes; test-context code (file_context=test) does NOT fire.
- [ ] **F-2c-i immune-rule count**: the security ruleset loads with `mutable = 0` for all 16 rows; `f2_immune_rule_contract.rs` asserts the new exact count (16) and passes; a tampered seed file still fails the blake3 integrity check (F-2a guarantee unchanged).
- [ ] **F-2c-i no-perf-claim for the rules**: the decision doc makes NO before/after percentage claim for the four rules in isolation — the only number is the corpus-level baseline from Task 4, explicitly caveated.
- [ ] **F-2c-ii measured baseline (RunPod-INDEPENDENT)**: `elai bench seccodeplt` RUNS the >=40-sample corpus through the REAL `evaluate_vibegate` gate over the 16-rule ruleset under `ELAI_SEED`, emits one `insecure_rate_measured` row + a `seccodeplt-*.json` report; the before/after/gate_recall is recorded in `workflow/reasoning/F-2c-decisions.md` with model_id (offline/static), n_samples, seed, and the synthetic-corpus + static-vs-dynamic caveat. This is the FIRST published baseline.
- [ ] **multi-lang + ast-grep-core DEFERRED**: NO `ast_adapter.rs::new()` edit; NO new external crate; the decision doc records the defer + the grounded reason (10 langs registered not 16; ast-grep-core's only win is a DSL).
- [ ] **W-4 frontend**: the new rules' blocks + the new baseline surface through the EXISTING SecurityRulesPanel + commands_f2 (no new SSB variant => no new pair); `python .claude/tools/cross_lang_diff.py` PASS (nothing new to drift).
- [ ] **W-6 dashboard E2E**: through the running dashboard, SecurityRulesPanel shows a CWE-798 block from the trigger above (dashboard-smoker / a live tauri:dev launch), not just a unit assertion.
- [ ] **No new dependency** (anti-slopsquatting per CLAUDE.md): F-2c adds NO external crate; blake3 + the E-9/F-2b harness deps are already in-tree.
- [ ] `cargo check --workspace` GREEN; auditor PASS (sole test runner; main LLM never runs tests).

## WIRING DONE-GATE

How F-2c satisfies each doctrine clause (the implementer cannot skip wiring):

- **W-1 (no-wire-dark done-gate)**: the literal `wiring_contract_check.py --run-trigger` line opens DETERMINISTIC_CHECK and is the first ACCEPTANCE checkbox; the FSM may not reach COMPLETE without its PASS. The new rules are wired through the SHIPPED `seed_security_rules` -> `evaluate_vibegate` path, so they are reachable by construction.
- **W-2 (no default-off terminal state)**: the 4 rules are seeded + enforced by DEFAULT (no `ELAI_*_ENABLE`), inheriting F-2a's no-bypass guarantee; the observable_effect runs with NO env override.
- **W-3 (production entrypoint)**: reachable_from names three REAL production entrypoints — the `seed_security_rules` call at runtime_setup.rs:322 (rules.db init), the `evaluate_vibegate` PreToolUse static gate, and the `elai bench seccodeplt` CLI subcommand. No `#[cfg(test)]`-only caller (NoOpAstEvaluator at rule_engine.rs:912 is test-only and is NOT used here — the production `TreeSitterAstEvaluator` evaluates the new AST children).
- **W-4 (frontend pair)**: F-2c adds NO new SSB action_type, so it inherits the SHIPPED SecurityRulesPanel + commands_f2 pair; the new rule blocks + the new baseline render there with no UI change; cross_lang_diff has no new DTO to check. The W-4 obligation is met by the existing pair surfacing the new data (justified explicitly in WIRING_CONTRACT).
- **W-5 (live observable_effect)**: observable_effect.trigger is a real write-time block + a runnable `elai bench seccodeplt` CLI run producing a live ssb.db row + an inspectable JSON report — never "cargo test passes". The live-Qwen run is a SEPARATE, explicitly-gated future checkbox (W-10), NOT made here; the static-gate baseline is RunPod-INDEPENDENT and IS the F-2c measured number.
- **W-6 (dashboard checkpoint)**: ACCEPTANCE includes a dashboard-verified item (SecurityRulesPanel shows a CWE-798 block) via dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol + reachable_from anchor + MODIFY target was confirmed in source AT AUTHORING TIME (2026-06-02), file:line cited in the WIRING_CONTRACT — seed_security_rules:392, runtime_setup.rs:322, load_security_seed_rules_builtin:142, Predicate:101/Composite:139, eval_order:329, detect_language_from_ext:346, evaluate_vibegate:133, bench_seccodeplt.rs:1/52, SecCodePltRunner, InsecureRateMeasured ssb.rs:8317/25427, f2_immune_rule_contract.rs:70, ast_adapter.rs:18-44. The prior verdict's "16 grammars" claim was CORRECTED here to the real 10 (ast_adapter.rs:18-44 read directly).
- **W-8 (produces/consumes symmetry)**: every produced TOML is consumed by `seed_security_rules` (the loader); the decision-doc baseline is produced by the `elai bench seccodeplt` run which consumes the 16-rule ruleset. No produced artifact lacks an in-plan consumer.
- **W-9 (SSB golden + named bump)**: F-2c adds NO new SSB action_type — `security_rule_blocked` + `insecure_rate_measured` already ship (F-2a/F-2b) and already flow through `action_type_lock.golden`. NO SCHEMA_VERSION bump (the baseline is 201 from F-2b's InsecureRateMeasured; F-2c does not touch it). This is a legitimate no-bump increment.
- **W-10 (tiered-gate)**: F-2c is ENTIRELY Tier-1 — the four rules are pure in-process AST/GREP (zero network/subprocess), and the baseline bench runs OFFLINE against the static gate (no model, no sandbox, no RunPod at bench time). There is NO Tier-2 in F-2c. The live-model secure-rate run + multi-language expansion + ast-grep-core are explicitly DEFERRED to future F-Ns and never advertise fail-closed for the model-run axis.

## Tests to add

- `crates/contracts/tests/f2c_cwe_rule_block_contract.rs` (~160 LOC): one production-context positive + one clean negative + one test-context (must-not-fire) per new CWE class — a Python `password = "hunter2"` (798) BLOCKs; a literal AES key (321) BLOCKs; a `logging.info(f"token={tok}")` (532) BLOCKs; a `print(password)` (532) WARNs; an `eval(user_input)` (95) BLOCKs; an `eval("1+1")` literal does NOT fire; a clean candidate passes; `file_context=test` code does NOT fire. Asserts the verdict + the CWE on each.
- `crates/contracts/tests/f2c_predicate_shape_contract.rs` (~70 LOC): each new `predicate_json` deserializes to the expected `Predicate::Composite { operator, children }` with `GrepPattern` + `AstPattern` children and `file_context = Production`; the AST child's `language` is one of {python, typescript} (registered-grammar guard — fails if an unregistered language leaks in).
- UPDATE `crates/contracts/tests/f2_immune_rule_contract.rs`: the row-count constant 12 -> 16 (+ the assertion message); the mutable=0 + idempotence + blake3-integrity assertions continue to pass over the larger set.
- `crates/orchestrator/tests/f2c_seccodeplt_baseline_smoke.rs` (~90 LOC): the deterministic fixture harness runs the >=40-sample corpus (now including 798/321/532/95 fixtures) through the real `evaluate_vibegate` path under ELAI_SEED and asserts one `insecure_rate_measured` row with before/after/gate_recall computed (not literal) + n_samples>=40 + a stable result under a fixed seed. (No live model — the offline static-gate path.)
- `crates/dashboard-v2` svelte-check on SecurityRulesPanel (unchanged) + cargo check on commands_f2 (dashboard-smoker) — confirm the existing pair still renders the new data.

## Files touched

- CREATE: `crates/rules/seed/security/hardcoded_creds.toml` (~38 LOC, CWE-798, Composite, Py+TS)
- CREATE: `crates/rules/seed/security/hardcoded_key.toml` (~36 LOC, CWE-321, Composite, Py+TS)
- CREATE: `crates/rules/seed/security/secrets_in_logs.toml` (~36 LOC, CWE-532, Composite, Py+TS, mixed BLOCK/WARN)
- CREATE: `crates/rules/seed/security/eval_injection.toml` (~34 LOC, CWE-95, Composite, Py+TS)
- CREATE: `workflow/reasoning/F-2c-decisions.md` (~100 LOC, the measured-baseline writeup + the defer rationale)
- CREATE: `crates/contracts/tests/f2c_cwe_rule_block_contract.rs` (~160 LOC)
- CREATE: `crates/contracts/tests/f2c_predicate_shape_contract.rs` (~70 LOC)
- CREATE: `crates/orchestrator/tests/f2c_seccodeplt_baseline_smoke.rs` (~90 LOC)
- MODIFY: `crates/contracts/tests/f2_immune_rule_contract.rs` (~2 LOC: row-count constant 12 -> 16 + message)
- MODIFY (CONDITIONAL — only if `seed_security_rules` hard-codes filenames rather than enumerating the dir): `crates/rules/src/project_rule_seeder.rs` (~4 LOC: add the 4 new filenames to the seed list). Resolve at read time; if the loader enumerates the dir, this file is NOT touched.
- MODIFY (CONDITIONAL — only if the existing corpus lacks the new CWE classes): `crates/orchestrator/src/benchmarks/seccodeplt_fixtures/` (~4 fixture triples for 798/321/532/95) so the baseline reflects the 16-rule ruleset. No harness code change.
- NO MODIFY: `crates/orchestrator/src/ssb.rs` (no new variant), `crates/knowledge/src/ast_adapter.rs` (no new grammar), any `Cargo.toml` (no new dep). These three NON-edits are scope-creep tripwires (DETERMINISTIC_CHECK step 4).

## Citations / Grounding

- `docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md` (predecessor — ruleset convention, no-perf-claim posture, Tier-1/Tier-2 split, the section-1.4 corrections) — PRIMARY plan grounding.
- `workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` section 4 item C2 + section 1.4 (the "below 15%"/GitGuardian corrections KEPT OUT) + section 2 (Tier rule) + section 5 (sequencing) — synthesis grounding (inherited via F-2).
- ELAI ground truth (code-read, CONFIRMED in source at authoring 2026-06-02): `ProjectRuleSeeder::seed_security_rules` project_rule_seeder.rs:392 (live at runtime_setup.rs:322); `load_security_seed_rules_builtin` rule_loader.rs:142 (re-export lib.rs:1751); `Predicate` enum predicate.rs:101 (AstPattern:102, GrepPattern:109, Composite:139, FileContext:118); `eval_order` rule_engine.rs:329 (Composite=tier3, GrepPattern=tier1); `detect_language_from_ext` rule_engine.rs:346-362 (.py->python, .ts/.tsx->typescript; None for .svelte/.html/.css — gap confirmed real, OUT OF SCOPE); `evaluate_vibegate` vibegate.rs:133; `bench_seccodeplt.rs::run` + `SecCodePltRunner` (imported bench_seccodeplt.rs:52); `SsbActionType::InsecureRateMeasured` ssb.rs:8317 (payload :25427, sql_str :9118, parse :9797, SCHEMA_VERSION 201); `f2_immune_rule_contract.rs:70` ("must insert exactly 12 rows" — updated to 16); the 12 existing rules confirmed exactly (sqli 3 / xss 3 / weak_crypto 4 / missing_authz 2).
- AST grammar count CORRECTION (CONFIRMED at authoring, contradicts the prior research report's "16 grammars wired"): `TreeSitterAstEvaluator::new()` ast_adapter.rs:18-44 registers exactly 10 languages (rust/python/typescript/go/java/c_sharp/ruby/html/css/svelte; astro behind `web-astro` feature). JavaScript/C/C++/Swift/Kotlin/Scala/PHP/Bash are pinned in `crates/knowledge/Cargo.toml:14-47` but UNREGISTERED in the evaluator — a Tier-2 AST rule for those needs an `ast_adapter.rs::new()` edit + per-grammar ABI verification. F-2c targets only the registered python+typescript, so NO evaluator edit is needed.
- `data/rules/web.toml` (WEB-013..023) — the working Composite/AstPattern Tier-2 template proving structural rules load + evaluate with zero new dependency (read at authoring).
- ast-grep-core registry status (anti-slopsquatting, VERIFIED via crates.io API 2026-06-02): version 0.43.0, license MIT, last published 2026-05-25. Clean + current — but DEFERRED (its only win over the existing engine is a richer YAML DSL; F-2c uses the existing Composite engine, zero new dep). tree-sitter pin: Cargo.toml:13 `"0.26"`; Cargo.lock resolves `tree-sitter 0.26.8` (>= the ast-grep-core 0.43 `^0.26.3` floor) — so a future ast-grep-core add is semver-compatible against the CURRENT lockfile; this refines the prior verdict's "patch-floor glossed" risk (the floor IS met at 0.26.8).
- arXiv:2410.11096 = SeCodePLT (NeurIPS 2025; Python/C/C++/Java; 44 CWEs; dynamic 3-way label) — VERIFIED via arxiv.org/abs/2410.11096 (the synthetic-corpus DESIGN reference; F-2c's static-gate metric is DISTINCT from SeCodePLT's dynamic metric — caveat carried).
- arXiv:2511.04023 = AdaTaint, "LLM-Driven Adaptive Source-Sink Identification and False Positive Mitigation for Static Analysis" (-43.7% FP, +11.2% recall; Juliet 1.3 + C benchmarks) — VERIFIED via arxiv.org/abs/2511.04023 (prior-art on FP reduction; an UNGROUNDED-for-ELAI target until measured on ELAI's own corpus — cited as motivation, NOT as an ELAI claim).
- arXiv:2601.18844 = "Reducing False Positives in Static Bug Detection with LLMs: An Empirical Study in Industry" (Tencent; 433 alarms; hybrid LLM+static eliminates 94-98% FP) — VERIFIED via arxiv.org/abs/2601.18844 (industry FP-reduction prior-art; same UNGROUNDED-for-ELAI caveat).
- Semgrep Rules License v1.0 (Dec 13 2024; non-SaaS / non-competing; internal use OK) — VERIFIED via semgrep.dev/legal/rules-license. The four F-2c rule patterns are HAND-AUTHORED (mirroring the existing 12), NOT copied from Semgrep-maintained rules, so no license entanglement; the citation is for the CWE taxonomy + the FP-discipline precedent only.
- CLAUDE.md HARD RULES applied: blake3 only (candidate digest, inherited from F-2a); no new external dep (ast-grep-core DEFERRED, verified-but-unused); the synthesis section-1.4 corrections NOT re-introduced; Python/TS-only scope (Rust deferred to F-10, multi-lang deferred to a future F-N).

## Open questions / risks

- **THE SINGLE BIGGEST RISK — false-positive UX friction on the four new classes (the documented CodeGate killer, synthesis section 7).** CWE-798/321/532 are absence/provenance-shaped (a literal assigned to a secret-named var; a secret in a log arg) and CAN over-fire (e.g. a placeholder `password = "CHANGEME"` in a config template, a logged hash that is not a secret). Mitigation: (a) each rule is a `GrepPattern AND AstPattern` Composite so the cheap grep gates the AST confirm and the AST bounds the shape; (b) `file_context = "production"` excludes test code; (c) the `secrets_in_logs` `print(` shape and the authz-like classes are WARN not BLOCK; (d) the Task-4 baseline RUN measures the actual FP/false-negative residual on the >=40-sample corpus BEFORE any "reduces FP" claim is published. ACTION (mandatory in the decision doc): bound the FP rate on >=3 real orchestrate runs as a release gate before promoting any WARN class to BLOCK. Until measured, NO efficacy claim is made.
- **Measured-number honesty (static vs dynamic).** The F-2c baseline is a STATIC-gate insecure-rate (the gate's false-negative residual over a synthetic corpus), NOT SeCodePLT's dynamic sandbox-executed secure-coding rate and NOT a live-model number. The decision doc states this explicitly; the panel tile must label it "static-gate baseline (synthetic corpus)". Conflating it with a live-model secure-rate would be a section-1.4-class overstatement.
- **Loader-enumeration uncertainty (resolve at read time).** Whether `seed_security_rules` enumerates the `seed/security/` directory or hard-codes the four existing filenames determines if Task 3 is a 0-LOC verification or a 4-LOC list edit. The WIRING_CONTRACT names `seed_security_rules` as the consumed symbol either way; the conditional MODIFY in Files-touched covers the second case. Do NOT assume; read project_rule_seeder.rs:392 first.
- **multi-language expansion is OUT OF SCOPE (DEFERRED, with a grounded reason).** C/C++/JS/Swift Tier-2 rules require an `ast_adapter.rs::new()` edit (the evaluator registers 10 langs, not 16) + per-grammar ABI verification — a separate, larger F-N. F-2c ships python+typescript only. The DETERMINISTIC_CHECK has an explicit tripwire (step 4) that FAILS if ast_adapter.rs::new is touched. Do NOT scope-creep multi-lang in.
- **ast-grep-core is OUT OF SCOPE (DEFERRED, verified-but-unused).** Verified clean (0.43.0, MIT, 2026-05-25, semver-compatible against the 0.26.8 lockfile) but its only win is a richer DSL; the existing Composite engine covers all four CWE classes. Adding it now would buy nothing for this batch and violate the no-new-dep-without-confirmation rule. The DETERMINISTIC_CHECK step 4 FAILS if any Cargo.toml gains a tree-sitter/ast-grep dep.
- **Rust coverage remains DEFERRED to F-10** (semgrep CWE rules sparse for Rust — synthesis C10). F-2c does NOT widen to Rust; flagged explicitly so scope does not silently drift.
- **`.svelte/.html/.css` language gap (CONFIRMED real, OUT OF SCOPE).** `detect_language_from_ext` returns None for these three extensions (rule_engine.rs:346-362), so a `security`-domain GrepPattern with `language: Some("...")` never fires on them. This is real but irrelevant to F-2c (the four new CWE classes are Python/TS code-level, not markup) — noted so a future F-N owns it, not silently widened here.
- **SCHEMA coordination: none needed.** F-2c adds no SSB variant and no SCHEMA bump (baseline 201 from F-2b is untouched). The only cross-plan coordination point is the immune-rule row-count constant (12->16) if another plan edits the security seed dir in the same window.

## Next

Once F-2c lands, the published static-gate baseline is the input to: (1) a future multi-language F-N that edits `ast_adapter.rs::new()` to register C/C++/JS/Swift + adds their Tier-2 rules (gated on the baseline showing the registered-language coverage is solid first); (2) the live-Qwen secure-rate run (RunPod / 2x3090 window) that produces the dynamic, model-dependent number for the ELAI-DIRECTION R1/R2 release artifact (the anti-Replit demo) — F-2b's harness already supports it, gated on a remote-sidecar window; (3) F-10 (security-monotonicity iteration guard) which consumes the now-16-rule `security` domain as its per-iteration CWE-class regression baseline. The ast-grep-core DSL evaluation becomes a separate, evidence-driven decision only if the measured FP rate on the existing Composite engine motivates a richer matcher — not before.

End of F-2c.
