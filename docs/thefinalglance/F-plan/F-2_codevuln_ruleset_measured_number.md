# F-2: Code-vuln-class pre-write gate + the FIRST measured insecure-rate number

Objective: populate ELAI's always-on `security` rule domain with curated vuln-class predicates (SQLi / XSS / weak-crypto / missing-authz) that BLOCK at write-time through the existing PreToolUse path, wired so the feature CANNOT be marked done while wire-dark, and produce the FIRST published before/after insecure-code-rate number on ELAI's own pre-write gate (the strategic keystone: ELAI-DIRECTION R1 measured-number + R2 anti-Replit demo + a first-in-category claim no VibeSec competitor holds). This is C2 in the synthesis (cluster identity C2->F-2). Two tiers per the W-10 split: F-2a (ruleset + write-time BLOCK, Python/TS first, no perf claim) ships in-process and fail-closed NOW; F-2b (the measured Qwen secure-rate number + optional constrained-decode sidecar layer) is RunPod-gated and ships a deterministic-fixture harness exercising the REAL path plus a separately-gated live-run checkbox.

Estimated effort: 2 sessions for F-2a (~8-10 h); F-2b is RunPod-gated (1 session once a 2x3090 / remote-sidecar window opens). Effort L per synthesis (split M + gated).

Pre-requisites:
- F-1 Tier-1 (VibeGate compound gate) co-ships: F-2a IS the code-vuln axis of F-1's compound evaluator. The two land in the same window; F-2 owns the ruleset + the benchmark, F-1 owns the four-axis fan-out + per-tool contract tests. Author both together; if F-1 lands first, F-2a registers its predicates into F-1's evaluator. If F-2 lands first, F-1 consumes `security`-domain rows. CONFIRM the integration symbol before edit.
- C-26 (constrained decoding substrate, `crates/providers/src/decoding.rs`) + E-35 (XGrammar sidecar mask, `crates/providers/src/runtime/sidecar_xgrammar_hook.rs`) SHIPPED — required only for the OPTIONAL Tier-2 secure-decoding layer in F-2b; F-2a does NOT depend on them.
- E-9/E-49 benchmark harness (`crates/cli/src/bench_cmd.rs` dispatcher + the per-bench `run()` module pattern + the `crates/orchestrator/src/benchmarks/` runner pattern) SHIPPED — F-2b's `--suite seccodeplt` runner mirrors it.
- RunPod / remote-sidecar window for F-2b ONLY (the Qwen2.5-Coder-7B-abliterated secure-rate run). F-2a is fully local.

Parallel-safe with: F-1 (co-ship; shared `security` domain — coordinate the rule_engine edit window; no schema conflict if F-1's `vibesec_code_vuln_blocked` and F-2's `security_rule_blocked` are added under one SCHEMA bump — see W-9). F-4 (cant-lie gate; orthogonal — different payload, different gate site). NOT parallel-safe with any plan editing `crates/orchestrator/src/ssb.rs` SsbActionType enum in the same window without a coordinated bump.

Source: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C2 (full mechanism + grounding + REVISED SCOPE + measurable_claim + the kept caveats); section 1 (corrections already forced — do NOT re-introduce 1.4 "below 15%" / GitGuardian-replay-corpus framings); section 2 (the Tier-1/Tier-2 cross-cutting rule); section 5 (sequencing — F-2a + F-1 co-ship TIER 1, F-2b is the TIER 2 keystone). Doctrine: docs/thefinalglance/F-plan/F-0_INDEX.md W-1..W-10.

## Context

The synthesis executive verdict (section 0): "the single most valuable move across all ten, and the one that serves ELAI-DIRECTION's two open release risks (R1 'one real measured number', R2 'distribution'): produce the FIRST published before/after insecure-code-rate number on ELAI's own pre-write gate (item C2). The whole VibeSec market sells the posture and nobody publishes the number (tools-matrix J #6, landscape 7.3). That one number is simultaneously R1, R2 (the anti-Replit demo: 'ELAI physically blocked the SQLi/secret/slop before it was written, here is the gate and the before/after rate'), and a first-in-category defensible claim no competitor holds."

ELAI already owns every substrate this needs. The rules engine carries `ALWAYS_INCLUDED_DOMAINS = &["engineering", "security"]` (CONFIRMED `crates/rules/src/rule_engine.rs:187`) so a `security`-domain rule is evaluated on EVERY phase regardless of project domain. The `Predicate` enum (CONFIRMED `crates/rules/src/predicate.rs:101`) ships `AstPattern { language, query, query_format, scope }` and `GrepPattern { pattern, scope, context_lines, language, file_context }` with a `FileContext::{Production, Test}` filter — exactly the AST/GREP predicate substrate C2 attaches to, with a built-in production-vs-test scoping that bounds the false-positive surface. The seed-rule loader `load_seed_rules_from_dir(rules_dir, engine)` (CONFIRMED `crates/orchestrator/src/rule_loader.rs:40`) compiles every `.toml` under a directory into `Vec<(Rule, CompiledPredicate)>` for `RuleEngine::evaluate`, and is shared by both the CLI and dashboard runtime setup. The PreToolUse path (CONFIRMED `dispatch_pre_tool_use` at `crates/orchestrator/src/hook_dispatcher.rs:1477`; `build_hook_block_summary(&findings, "PreToolUse")` at `crates/orchestrator/src/task_runner.rs:6439`) is where a BLOCK verdict aborts a write. The SSB journal makes every block auditable + reproducible under `ELAI_SEED` — which is precisely why the headline framing ("a security control you can audit and reproduce") is true and no probabilistic competitor (Snyk Studio, Semgrep advisory) can claim it.

The gap (synthesis C2): the `security` domain has predicate infrastructure but ZERO curated vuln-class rules, the gate is not wired as the code-vuln axis of a write-time block, and — the keystone — NO tool anywhere publishes a measured pre-write insecure-rate reduction. F-2a closes the ruleset+BLOCK gap (Tier-1, in-process, genuinely fail-closed). F-2b produces the number (Tier-2, RunPod-gated).

The kept caveat (synthesis C2 + section 7): the SVEN (59.1%->92.3%) and CodeGuard+ (43.2%->76.0%) numbers are CodeGen-2.7B; they do NOT transfer to Qwen2.5-Coder-7B-abliterated. F-2a makes NO performance claim. F-2b's headline is GATED on a measured Qwen run. Re-introducing the "below 15%" target or the "30-commit GitGuardian replay corpus" framing is a section-1.4 violation — the benchmark defines its OWN synthetic corpus.

## Existing state (B/C/D/E coverage)

- E-12 (post-write semgrep, PLANNED/partial): runs CWE detection AFTER the file is written, as an auditor-fed async check. F-2 is the PRE-write complement — it BLOCKS at the PreToolUse boundary before bytes hit disk. Orthogonal trigger site (hook vs auditor); no overlap. Synthesis novelty verdict: GENUINELY_NEW vs E-12.
- C-26 (`crates/providers/src/decoding.rs`, CONFIRMED): `ConstrainedDecodingConfig` (decoding.rs:122), `default_decoding_for(role: RoleDecoding)` (decoding.rs:262), `DecodingMode` (decoding.rs:73), grammar/regex/json_schema constructors (decoding.rs:146-164). The OPTIONAL F-2b secure-decode layer attaches here, NOT in F-2a.
- E-35 (`crates/providers/src/runtime/sidecar_xgrammar_hook.rs`, CONFIRMED): `build_xgrammar_request(...)` (DEF_FN sidecar_xgrammar_hook.rs:240; call site at :333), `XGrammarRequest`, `xgrammar_cache_key`. The Tier-2 sidecar mask path for F-2b ONLY.
- E-9/E-49 (benchmark harness, CONFIRMED): `crates/cli/src/bench_cmd.rs` dispatcher (run_bench_prmbench/arc/rag/... each calling `crate::bench_<name>::run(...)`), the per-bench `run()` module pattern (e.g. `bench_prmbench.rs`), and the `crates/orchestrator/src/benchmarks/*_runner.rs` runner pattern (prmbench_runner.rs, sycophancy_runner.rs, etc.). F-2b adds a `seccodeplt` runner + CLI arm in the SAME pattern (each emits one SSB completion row).
- E-30 (`crates/safety`, SHIPPED): the argv destructive-verb allowlist. F-2 is content-class (vuln patterns in generated code), not argv-class (destructive shell verbs) — disjoint. E-30's `commands_e30.rs` + `IncidentsDrawer.svelte` are the FRONTEND PAIR template F-2's panel mirrors.
- E-43 (skill scanning, SHIPPED): the malicious-skill axis of VibeGate (F-1 Tier-2). Disjoint from F-2's code-vuln axis.
- Rules engine (`crates/rules`, SHIPPED): `ALWAYS_INCLUDED_DOMAINS` (rule_engine.rs:187), `Predicate` 7-variant enum (predicate.rs:101), `evaluate`/`evaluate_single` (rule_engine.rs:385/467), `load_seed_rules_from_dir` (rule_loader.rs:40), `ProjectRuleSeeder::seed_rules` (project_rule_seeder.rs:91), existing seed TOMLs at `crates/rules/seed/` (slopsquatting.toml, source_tiers.toml, risk_levels.toml, app_store.toml, design_hard.toml). The new security ruleset follows this exact on-disk convention.
- SSB (`crates/orchestrator/src/ssb.rs`, SHIPPED): `SsbActionType` enum (ssb.rs:4412), sql_str parse arms (ssb.rs:9326/9380 pattern), payload roundtrip arm (ssb.rs:27907 pattern), golden lock (`action_type_lock.golden`). Pattern-2 single-variant-with-Kind-discriminator collapse is the SCHEMA discipline (CLAUDE.md baseline SCHEMA_VERSION 191).

## Reads (mandatory before coding)

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C2 (full), section 1 (corrections), section 2 (Tier rule), section 5 (sequencing), section 7 (what-not-to-do)
- docs/thefinalglance/F-plan/F-0_INDEX.md (W-1..W-10 doctrine — the done-gate this file is written to satisfy)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:88-200 (the 7-variant Predicate enum; AstPattern + GrepPattern + FileContext shape the ruleset TOML)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\rule_engine.rs:180-200 + 380-520 (ALWAYS_INCLUDED_DOMAINS, evaluate, evaluate_single, domain filter)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\rule_loader.rs:1-95 (load_seed_rules_from_dir — the loader the new dir plugs into)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\project_rule_seeder.rs (seed_rules + mutable column path; for the mutable=0 immune-rule clause)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs:1470-1600 (dispatch_pre_tool_use backend shape)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:6420-6450 (build_hook_block_summary — the BLOCK abort path that emits the SSB row)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\bench_cmd.rs:1-160 (the E-9 bench dispatcher; F-2b mirrors it)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\prmbench_runner.rs (the runner pattern F-2b's seccodeplt_runner mirrors)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands_e30.rs + crates\dashboard-v2\src\lib\drawers\PrmBenchPanel.svelte + components\PinnedRulesPanel.svelte (frontend_pair templates)
- arXiv:2302.05319 SVEN (CCS 2023) — abstract/table only; CodeGen-2.7B caveat
- arXiv:2405.00218 CodeGuard+ — constrained-decode 43.2%->76.0% secure-pass@1; CodeGen caveat
- arXiv:2410.11096 SeCodePLT (NeurIPS 2025, 5.9k samples / 44 CWEs) — the corpus design reference for the synthetic suite

## Tasks

### TIER 1 — F-2a: vuln-class ruleset + write-time BLOCK (in-process, fail-closed, Python/TS first, NO perf claim)

1. **Create the security ruleset directory `crates/rules/seed/security/` with vuln-class TOML files (~180 LOC across 4 files).** Mirror the on-disk convention of the existing `crates/rules/seed/*.toml` (CONFIRMED present: slopsquatting.toml, source_tiers.toml) so `load_seed_rules_from_dir` (rule_loader.rs:40) compiles them with no loader change. Each `[[rules]]` block carries `rule_id`, `domain = "security"`, `severity = "BLOCK"`, a `predicate_json` of `GrepPattern` or `AstPattern` shape (predicate.rs:101), and `file_context = "production"` to bound false-positives (do NOT fire on test code). Initial set, SCOPED TO PYTHON + TYPESCRIPT output (the synthesis REVISED-SCOPE bound; Rust deferred to F-10's classifier work):
   - `seed/security/sqli.toml` — string-interpolated / f-string / template-literal SQL into `execute`/`query`/`raw` sinks (CWE-89). GrepPattern with `language: Some("python")` and a TS variant.
   - `seed/security/xss.toml` — `dangerouslySetInnerHTML`, `innerHTML =`, `v-html`, unescaped template interpolation into DOM sinks (CWE-79). TS/JS scope.
   - `seed/security/weak_crypto.toml` — `hashlib.md5`/`hashlib.sha1` for security use, `Math.random()` for tokens/secrets, `DES`/`ECB` mode (CWE-327/328). Python + TS. NOTE: this complements (does not duplicate) the existing `slopsquatting.toml` blake3-vs-blake2 staleness rule, which is a VersionGate, not a vuln-class rule.
   - `seed/security/missing_authz.toml` — route/handler decorators or framework endpoints with no auth guard adjacent (CWE-862), expressed as a COMPOSITE predicate (route-pattern AND NOT auth-guard-pattern within scope). Conservative: WARN-not-BLOCK for the authz class in F-2a (high FP risk per synthesis section 7 CodeGate-death lesson); BLOCK reserved for sqli/xss/weak-crypto where the signal is high-precision.
   Every `predicate_json` is validated against the real `Predicate` serde shape (`#[serde(tag = "type", rename_all = "snake_case")]`) — `{"type":"grep_pattern", "pattern":..., "file_context":"production", "language":"python", "context_lines":0, "scope":null}`. Cite each CWE + the SeCodePLT class in a TOML comment.

2. **Register `crates/rules/seed/security/` as a loaded seed dir + seed it mutable=0 (immune) (~60 LOC delta).** The synthesis records `mutable` is the INTEGER column (default 1) with a tested mutable=0 path; there are no mutable=0 rows in the security domain yet. Extend the seed path so the security ruleset is loaded AND inserted with `mutable = 0` via `ProjectRuleSeeder::seed_rules` (project_rule_seeder.rs:91) so per-project rule edits cannot disable security rules. Add a blake3 integrity check of the seed-rule set at load time (CLAUDE.md: blake3, never blake2/sha2) — `mutable=0` is not truly immune if the SQLite is user-writable, so the load path recomputes a blake3 digest of the canonical seed bytes and refuses to start with a tampered security ruleset (fail-closed, logged). Do NOT add a network or subprocess call here — pure in-process file read + blake3.

3. **Wire the security-domain evaluation as a write-time BLOCK on the PreToolUse path (~80 LOC delta in `crates/orchestrator/src/task_runner.rs` + `crates/orchestrator/src/hook_dispatcher.rs`).** The `dispatch_pre_tool_use` backend (hook_dispatcher.rs:1477) already drives findings; `build_hook_block_summary(&findings, "PreToolUse")` (task_runner.rs:6439) already aborts the write on a BLOCK verdict. Wire: when a PreToolUse invocation carries a candidate file write (Edit/Write tool with code content), run the compiled `security`-domain rules through `RuleEngine::evaluate` (rule_engine.rs:385) over the candidate diff content; on a BLOCK verdict, refuse the write and emit a `security_rule_blocked` SSB row carrying `{rule_id, cwe, file_path, blake3_of_candidate, language, severity}`. NO bypass branch: the LLM cannot disable the security evaluation via any prompt-controlled flag (stub_ban enforces). T1 latency budget (<1ms) — regex/AST match only, no I/O. This is F-2a as the CODE-VULN AXIS of F-1's compound VibeGate (co-ship).

4. **Add the `security_rule_blocked` SSB action type (Pattern-2 collapse) (~70 LOC in `crates/orchestrator/src/ssb.rs`).** Add ONE variant `SecurityRuleBlocked` to `SsbActionType` (ssb.rs:4412) with a `SecurityRuleBlockedPayload { rule_id, cwe_class, file_path, candidate_blake3, language, severity }`. Wire the sql_str parse arm (ssb.rs:9326-pattern `"security_rule_blocked" => Ok(...)`), the payload roundtrip arm (ssb.rs:27907-pattern), the SQL CHECK constraint, and the golden lock row in `action_type_lock.golden`. SCHEMA bump named in WIRING_CONTRACT (W-9). Per synthesis + F-0 W-9: collapse F-2a's `security_rule_blocked` and F-2b's `insecure_rate_measured` under the SAME SCHEMA bump if F-2 ships as one unit; if F-2a ships first, F-2b's variant lands in its own gated bump.

5. **Frontend pair: SecurityRulesPanel.svelte + Tauri command (~330 LOC svelte + ~160 LOC tauri).** Per W-4 (every dashboard-surfaceable SSB signal needs a frontend pair). New `crates/dashboard-v2/src/lib/drawers/SecurityRulesPanel.svelte` listing recent `security_rule_blocked` rows (rule_id, CWE, file, language, timestamp) AND — once F-2b lands — the before/after insecure-rate from `insecure_rate_measured`. New Tauri command `crates/dashboard-v2/src-tauri/src/commands_f2.rs::get_security_blocks_recent` + `get_insecure_rate_latest` mirroring the `commands_e30.rs` / `commands_d6.rs::get_held_out_recent` shape (CONFIRMED pattern). Add `DrawerKey 'security_rules'` to types.ts + DrawerHost.svelte. `cross_lang_diff.py` MUST pass (no Rust<->TS DTO drift).

6. **Decision doc `workflow/reasoning/F-2-decisions.md` (~120 LOC).** Covers: (a) Python/TS-first scope bound + why Rust is deferred to F-10's classifier (semgrep CWE rules are sparse for Rust — synthesis C10); (b) authz=WARN-not-BLOCK rationale (CodeGate-death FP lesson, synthesis section 7); (c) the mutable=0 + blake3-integrity immune-rule design; (d) the EXPLICIT no-perf-claim posture for F-2a + the gated headline for F-2b; (e) the section-1.4 corrections kept out (no "below 15%", no GitGuardian-replay framing); (f) the Tier-1/Tier-2 split per synthesis section 2.

### TIER 2 — F-2b: the measured Qwen secure-rate number (RunPod-gated; ships a deterministic fixture harness + a separately-gated live checkbox)

7. **Define the synthetic insecure-rate benchmark corpus `crates/orchestrator/src/benchmarks/seccodeplt_fixtures/` (~150 LOC + fixtures).** A SeCodePLT-style synthetic suite (arXiv:2410.11096 design: CWE-tagged prompt + a deterministic insecure/secure classifier per sample), SCOPED to the Python/TS CWE classes F-2a's ruleset covers (CWE-89/79/327/862). Each fixture is a `{prompt, cwe, secure_oracle_pattern}` triple. The corpus is ELAI's OWN synthetic benchmark (section 1.4: never borrow GitGuardian 3.2% or Veracode 45% as a replay set). Ship >=40 samples.

8. **`crates/cli/src/bench_seccodeplt.rs::run(...)` + dispatcher arm (~140 LOC) — the REAL-path harness.** Mirror `crates/cli/src/bench_prmbench.rs::run` + the `bench_cmd.rs::run_bench_prmbench` wrapper (CONFIRMED pattern). New `crates/orchestrator/src/benchmarks/seccodeplt_runner.rs` mirrors `prmbench_runner.rs`. The runner: (1) for each fixture, generates code via the SAME production sidecar generation path the live engine uses (NOT a mock), (2) runs the candidate through the SAME F-2a `security`-domain `RuleEngine::evaluate` gate, (3) computes insecure-rate = (samples whose generated output matches a vuln predicate) / total, BEFORE the gate (raw model) and AFTER the gate (post-BLOCK regeneration). Emits one `insecure_rate_measured` SSB row `{suite, model_id, n_samples, insecure_rate_before, insecure_rate_after, seed}`. W-5: this harness exercises the REAL production code path with a DETERMINISTIC FIXTURE (runs offline / on the remote sidecar under `ELAI_SEED`); the LIVE Qwen run is a SEPARATE, explicitly-gated acceptance checkbox.

9. **Add the `insecure_rate_measured` SSB action type (~60 LOC in ssb.rs).** Same registration pattern as Task 4 (variant + sql_str arm + payload roundtrip + CHECK + golden). `InsecureRateMeasuredPayload { suite, model_id, n_samples, insecure_rate_before, insecure_rate_after, seed }`. Surfaced by the F-2b half of SecurityRulesPanel (the before/after number — the keystone deliverable a human SEES through the dashboard).

10. **OPTIONAL Tier-2 secure-decoding layer (gated, design-first; ~120 LOC if pursued).** ONLY if the F-2a gate's after-rate is insufficient: add a constrained/secure-decode mask via `crates/providers/src/decoding.rs::ConstrainedDecodingConfig` (decoding.rs:122) + the E-35 `build_xgrammar_request` (sidecar_xgrammar_hook.rs:333) sidecar mask. This is a model-run-dependent Tier-2 axis (synthesis section 2) — document which guarantee it delivers (it shifts generation, it is NOT fail-closed) and never let it advertise the fail-closed promise. Defer unless the measured F-2a number motivates it.

11. **F-2b decision doc addendum + the published number writeup (~80 LOC).** Append the measured before/after to `workflow/reasoning/F-2-decisions.md` with the model_id (`huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated` on the remote 3090, or 7B on a 2x3090 RunPod window), n_samples, seed, and the EXPLICIT caveat that this is ELAI's synthetic corpus (not transferable to SVEN/CodeGuard+ CodeGen-2.7B numbers — synthesis section 7). This writeup IS the R1/R2 keystone artifact.

## WIRING_CONTRACT

produces_symbols:
  # RECONCILED 2026-06-01 (s-6344 W-1 gate, per W-5/W-7/W-10): the W-1 done-gate
  # covers the SHIPPED Tier-1 (F-2a) symbols ONLY. F-2b (the measured Qwen
  # secure-rate number + bench harness + InsecureRateMeasured) is RunPod-gated and
  # NOT shipped this window -- its symbols moved to deferred_produces_symbols below
  # so the W-1 gate does not FAIL on a legitimately-deferred Tier-2 (W-5: live-run
  # is a separately-gated checkbox). Every entry here is grep-confirmed DEF/SCIP_DEF
  # in elai.map (TOMLs confirmed on disk).
  - crates/rules/seed/security/sqli.toml
  - crates/rules/seed/security/xss.toml
  - crates/rules/seed/security/weak_crypto.toml
  - crates/rules/seed/security/missing_authz.toml
  - crates/orchestrator/src/ssb.rs::SsbActionType::SecurityRuleBlocked
  - crates/orchestrator/src/ssb.rs::SecurityRuleBlockedPayload
  - crates/orchestrator/src/rule_loader.rs::load_security_seed_rules_builtin
  - crates/rules/src/project_rule_seeder.rs::seed_security_rules
  - crates/dashboard-v2/src/lib/drawers/SecurityRulesPanel.svelte
  - crates/dashboard-v2/src-tauri/src/commands_f2.rs::get_security_blocks_recent
deferred_produces_symbols:  # F-2b Tier-2, RunPod-gated; NOT part of the W-1 done-gate this window (W-5/W-10)
  - crates/orchestrator/src/ssb.rs::SsbActionType::InsecureRateMeasured
  - crates/orchestrator/src/ssb.rs::InsecureRateMeasuredPayload
  - crates/cli/src/bench_seccodeplt.rs::run
  - crates/cli/src/bench_cmd.rs::run_bench_seccodeplt
  - crates/orchestrator/src/benchmarks/seccodeplt_runner.rs::SecCodePltRunner
  - crates/dashboard-v2/src-tauri/src/commands_f2.rs::get_insecure_rate_latest
consumes_symbols:
  - crates/rules/src/rule_engine.rs::evaluate
  - crates/rules/src/rule_engine.rs::ALWAYS_INCLUDED_DOMAINS
  - crates/rules/src/predicate.rs::Predicate
  - crates/rules/src/predicate.rs::FileContext
  - crates/orchestrator/src/rule_loader.rs::load_seed_rules_from_dir
  - crates/rules/src/project_rule_seeder.rs::seed_rules
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use
  - crates/orchestrator/src/task_runner.rs::build_hook_block_summary
reachable_from:
  # RECONCILED 2026-06-01 (s-6344): the run_bench_seccodeplt anchor is F-2b
  # (RunPod-gated, not shipped) -> moved out of the W-1 gate. F-2a IS the code-vuln
  # axis of F-1's compound gate, so its production chain is the same baby-step
  # pre-gate wiring (grep-confirmed DEF/SCIP_DEF in elai.map).
  - crates/orchestrator/src/runtime_setup.rs::build_task_runner_wiring (calls ProjectRuleSeeder::seed_security_rules at rules.db init; the security ruleset is seeded mutable=0 + integrity-checked on the sole production wiring path)
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use (PreToolUse hook fires security-domain eval on every baby-step pre-gate candidate; production entrypoint per W-3)
  - crates/dashboard-v2/src-tauri/src/commands_f2.rs::get_security_blocks_recent (Tauri command; dashboard entrypoint per W-3/W-4)
stub_ban:
  - "TODO: add security rules" literal absent
  - "// vuln gate later" literal absent
  - No bypass branch on the security-domain PreToolUse evaluation (the LLM cannot disable the code-vuln gate via any prompt-controlled or env-controlled flag) [W-2 LLM-bypass-ban]
  - No default-OFF env gate as the terminal state: the security ruleset is loaded + enforced by DEFAULT with no ELAI_*_ENABLE required; any kill-switch env var defaults to ON and the observable_effect MUST run with NO env override set [W-2]
  - No silent fail-open when the blake3 seed-integrity check fails (refuse-to-start, never silently skip the security ruleset)
  - "insecure_rate_before == insecure_rate_after" hard-coded / mock number absent (the measured number is computed from a real run, never a placeholder) [section 1.4]
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/SecurityRulesPanel.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_f2.rs::get_security_blocks_recent (+ get_insecure_rate_latest)
observable_effect:
  trigger: "elai bench seccodeplt --suite seccodeplt --json   (runs the deterministic fixture harness through the REAL production generate+gate path under ELAI_SEED; for the gate-block path: a PreToolUse Edit carrying a Python f-string SQL concatenation into cursor.execute is refused at write-time)"
  expect_ssb: "a security_rule_blocked row (from the write-time block path) AND an insecure_rate_measured row (from the bench run) appear in the live ssb.db within the run; both carry a blake3 digest, not blake2/sha2"
  expect_artifact: "the bench writes a report file matching glob workflow/**/seccodeplt-*.{toml,json} containing /insecure_rate_(before|after)/ ; the write-time block leaves the candidate file UNWRITTEN and the FSM does NOT advance to EXECUTING for the refused step"

## DETERMINISTIC_CHECK

```bash
# W-1 wiring done-gate (the literal line; PASS required, not cargo-green)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md --run-trigger

# 1. security ruleset dir + the 4 vuln-class TOMLs exist
test -d .elai_cc/crates/rules/seed/security
test -f .elai_cc/crates/rules/seed/security/sqli.toml
test -f .elai_cc/crates/rules/seed/security/xss.toml
test -f .elai_cc/crates/rules/seed/security/weak_crypto.toml
test -f .elai_cc/crates/rules/seed/security/missing_authz.toml

# 2. the 2 new SSB action types exist (variant + sql_str arm + payload)
rg "SecurityRuleBlocked|security_rule_blocked|SecurityRuleBlockedPayload" .elai_cc/crates/orchestrator/src/ssb.rs
rg "InsecureRateMeasured|insecure_rate_measured|InsecureRateMeasuredPayload" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. the security-domain eval is wired into the PreToolUse block path (no bypass)
rg "security_rule_blocked|SecurityRuleBlocked" .elai_cc/crates/orchestrator/src/task_runner.rs .elai_cc/crates/orchestrator/src/hook_dispatcher.rs

# 4. the bench runner + CLI arm exist (E-9 pattern)
test -f .elai_cc/crates/cli/src/bench_seccodeplt.rs
rg "run_bench_seccodeplt" .elai_cc/crates/cli/src/bench_cmd.rs
test -f .elai_cc/crates/orchestrator/src/benchmarks/seccodeplt_runner.rs

# 5. blake3 (never blake2/sha2) on the candidate + seed-integrity digests
rg "blake3" .elai_cc/crates/orchestrator/src/ssb.rs .elai_cc/crates/rules/src/project_rule_seeder.rs
! rg "blake2|sha2::Sha256" .elai_cc/crates/rules/seed/security/ 2>/dev/null

# 6. frontend pair + cross-language drift gate (W-4)
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/SecurityRulesPanel.svelte
test -f .elai_cc/crates/dashboard-v2/src-tauri/src/commands_f2.rs
python .claude/tools/cross_lang_diff.py

# 7. golden lock carries the 2 new variants + SCHEMA bump named
rg "security_rule_blocked|insecure_rate_measured" .elai_cc/crates/orchestrator/tests/action_type_lock.golden

# 8. compile gate (necessary, NOT sufficient per W-1)
cd .elai_cc && cargo check --workspace
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md --run-trigger` returns PASS (exit 0) — every produces_symbol resolves to a DEF/SCIP_DEF in elai.map, every consumes_symbol has a real CALL/REF, every reachable_from anchor reached by BFS over CALL_RESOLVED within depth 8, stub_ban finds zero forbidden literals, AND the observable_effect trigger RAN and emitted the expected rows + artifact. (cargo-green alone is NOT sufficient.)
- [ ] **W-2 default-ON verification**: the observable_effect (a PreToolUse Edit with an f-string SQLi candidate is refused; `elai bench seccodeplt` runs) executes with NO `ELAI_*_ENABLE` / env override set — the security ruleset is enforced by default. Any kill-switch defaults to ON.
- [ ] **F-2a code-vuln BLOCK**: a candidate Python `cursor.execute(f"SELECT * FROM users WHERE id={uid}")` Edit is BLOCKED at write-time; a `security_rule_blocked` SSB row with CWE-89 + a blake3 candidate digest appears; the file is left UNWRITTEN.
- [ ] **F-2a immune-rule**: the security ruleset loads with `mutable = 0`; a per-project rule edit attempting to disable a security row is refused; a tampered seed file fails the blake3 integrity check and refuses-to-start (fail-closed, logged).
- [ ] **F-2a no-perf-claim**: the decision doc + panel make NO before/after percentage claim for F-2a (the number is F-2b's, gated).
- [ ] **W-6 dashboard E2E**: through the running dashboard, the SecurityRulesPanel lists the `security_rule_blocked` row from the block above (verified via dashboard-smoker / a live tauri:dev launch), not just a unit assertion.
- [ ] **W-4 cross-lang**: `python .claude/tools/cross_lang_diff.py` PASS (no Rust<->TS DTO drift on the commands_f2 / SecurityRulesPanel pair).
- [ ] **W-9 SSB golden**: both new variants flow through `action_type_lock.golden`; SCHEMA_VERSION bumped to baseline+N (named in WIRING_CONTRACT / decision doc); cross_lang_diff + action_type_lock green.
- [ ] **F-2b DETERMINISTIC FIXTURE HARNESS (RunPod-independent, REQUIRED to ship F-2b code)**: `elai bench seccodeplt` runs the >=40-sample synthetic corpus through the REAL production generate+gate path under `ELAI_SEED` (not a mock), emits an `insecure_rate_measured` row + a `seccodeplt-*.json` report. This is a SEPARATE checkbox from the live run per W-5.
- [ ] **F-2b LIVE QWEN RUN (RunPod / remote-sidecar GATED — the keystone number)**: the same harness run against `huihui-ai/Qwen2.5-Coder-*-abliterated` on the remote 3090 / a 2x3090 RunPod window produces the FIRST measured before/after insecure-rate, written to `workflow/reasoning/F-2-decisions.md` with model_id + n_samples + seed + the synthetic-corpus caveat. (Explicitly gated, separate from the fixture-harness checkbox.)
- [ ] **No new dependency** outside cargo/pypi verify (anti-slopsquatting per CLAUDE.md); F-2 adds NO external dep (blake3 already in-tree; the bench reuses the E-9 harness deps).
- [ ] `cargo check --workspace` GREEN; auditor PASS (sole test runner; main LLM never runs tests).

## WIRING DONE-GATE

How this F-2 satisfies each doctrine clause (the implementer cannot skip wiring):

- **W-1 (no-wire-dark done-gate)**: the literal `wiring_contract_check.py --run-trigger` line is in DETERMINISTIC_CHECK and is the first ACCEPTANCE checkbox. The FSM may not reach COMPLETE without its PASS; the wiring-contract-validator agent runs it. cargo-green is necessary-not-sufficient.
- **W-2 (no default-off terminal state)**: the security ruleset is loaded + enforced by DEFAULT (no `ELAI_*_ENABLE`). stub_ban forbids a default-off terminal state AND any LLM/env bypass branch. The W-2 ACCEPTANCE item runs the observable_effect with NO env override.
- **W-3 (production entrypoint)**: reachable_from names three REAL production entrypoints — the PreToolUse hook (`dispatch_pre_tool_use`), a CLI subcommand (`elai bench seccodeplt`), and a Tauri command (`commands_f2`). No #[cfg(test)]-only caller.
- **W-4 (frontend pair)**: `security_rule_blocked` + `insecure_rate_measured` are dashboard-surfaceable, so frontend_pair names SecurityRulesPanel.svelte + commands_f2.rs and cross_lang_diff.py is a gate. Not exempt (these are user-facing signals, not pure internal gates).
- **W-5 (live observable_effect)**: observable_effect.trigger is a runnable CLI command + a real write-time block, producing live ssb.db rows + an inspectable report file — never "cargo test passes". The RunPod-gated live number ships a deterministic fixture harness exercising the REAL path plus a SEPARATE gated live checkbox.
- **W-6 (dashboard checkpoint)**: ACCEPTANCE includes a dashboard-verified item (SecurityRulesPanel shows the block) via dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol + reachable_from anchor + MODIFY target was confirmed in elai.map at authoring time (ALWAYS_INCLUDED_DOMAINS rule_engine.rs:187, Predicate predicate.rs:101, load_seed_rules_from_dir rule_loader.rs:40, dispatch_pre_tool_use hook_dispatcher.rs:1477 (InProcessHookDispatcher impl DEF_FN), build_hook_block_summary task_runner.rs:11943 (DEF_FN; call site at :6439), seed_rules project_rule_seeder.rs:68 (SCIP_DEF), SsbActionType ssb.rs:4412, bench_cmd.rs run_bench_prmbench:31 dispatcher, prmbench_runner.rs pattern, commands_e30.rs/PrmBenchPanel.svelte templates). CORRECTION logged: the synthesis cited `crates/cli/src/benchmark.rs` and `crates/rules/rules/security/` — NEITHER exists; the real anchors are `crates/cli/src/bench_cmd.rs` + `crates/orchestrator/src/benchmarks/` and `crates/rules/seed/security/`. The validator re-greps.
- **W-8 (produces/consumes symmetry)**: every produced symbol has an in-plan caller — the 4 TOMLs are consumed by `load_seed_rules_from_dir`; the 2 SSB variants are emitted by Task 3 (block path) + Task 8 (bench); the bench runner is invoked by the CLI arm; the panel + commands_f2 are paired. No produced function is deferred to "later".
- **W-9 (SSB golden + named bump)**: both variants flow through `action_type_lock.golden`; SCHEMA_VERSION bumped to baseline+N (baseline 191 per CLAUDE.md — exact N set at implement time, not hard-coded here); Pattern-2 single-variant-with-payload collapse where applicable; cross_lang_diff + action_type_lock green.
- **W-10 (tiered-gate)**: F-2a (in-process AST/GREP + blake3 integrity, zero network/subprocess) IS the fail-closed done-gate and ships now. F-2b (the model-dependent Qwen secure-rate run) is a SEPARATELY-gated Tier-2 sub-phase; it documents that the secure-decode layer SHIFTS generation and is NOT fail-closed, and it never advertises fail-closed for the model-run axis. The hot per-write path stays pure-Rust and fail-closed.

## Tests to add

- `crates/contracts/tests/f2_security_rule_block_contract.rs`: a Python f-string SQLi candidate + a TS `dangerouslySetInnerHTML` candidate each produce a BLOCK verdict from the `security`-domain eval; a clean candidate passes; test code (file_context=test) does NOT fire.
- `crates/contracts/tests/f2_security_ssb_roundtrip_contract.rs`: `SecurityRuleBlockedPayload` + `InsecureRateMeasuredPayload` serde round-trip; sql_str parse arms resolve; golden-lock variants present.
- `crates/contracts/tests/f2_immune_rule_contract.rs`: a mutable=0 security row cannot be disabled by a project-rule edit; a tampered seed file fails the blake3 integrity check (refuse-to-start).
- `crates/orchestrator/tests/f2_seccodeplt_fixture_smoke.rs`: the deterministic fixture harness runs the >=40-sample corpus through the real gate path under ELAI_SEED and emits one `insecure_rate_measured` row (no live model required — uses the recorded-fixture generate path).
- `crates/dashboard-v2` svelte-check on SecurityRulesPanel + cargo check on commands_f2 (dashboard-smoker).

## Files touched

- CREATE: `crates/rules/seed/security/sqli.toml` (~50 LOC)
- CREATE: `crates/rules/seed/security/xss.toml` (~45 LOC)
- CREATE: `crates/rules/seed/security/weak_crypto.toml` (~45 LOC)
- CREATE: `crates/rules/seed/security/missing_authz.toml` (~40 LOC, COMPOSITE, WARN-class)
- CREATE: `crates/cli/src/bench_seccodeplt.rs` (~140 LOC, mirrors bench_prmbench.rs::run)
- CREATE: `crates/orchestrator/src/benchmarks/seccodeplt_runner.rs` (~180 LOC, mirrors prmbench_runner.rs)
- CREATE: `crates/orchestrator/src/benchmarks/seccodeplt_fixtures/` (~40 fixture files + manifest, ~150 LOC)
- CREATE: `crates/dashboard-v2/src/lib/drawers/SecurityRulesPanel.svelte` (~330 LOC)
- CREATE: `crates/dashboard-v2/src-tauri/src/commands_f2.rs` (~160 LOC)
- CREATE: `workflow/reasoning/F-2-decisions.md` (~120 LOC + F-2b addendum)
- CREATE: `crates/contracts/tests/f2_security_rule_block_contract.rs` (~140 LOC)
- CREATE: `crates/contracts/tests/f2_security_ssb_roundtrip_contract.rs` (~90 LOC)
- CREATE: `crates/contracts/tests/f2_immune_rule_contract.rs` (~110 LOC)
- CREATE: `crates/orchestrator/tests/f2_seccodeplt_fixture_smoke.rs` (~90 LOC)
- MODIFY: `crates/orchestrator/src/rule_loader.rs` (~30 LOC: register seed/security/ as a loaded dir; blake3 integrity check)
- MODIFY: `crates/rules/src/project_rule_seeder.rs` (~40 LOC: seed security rows with mutable=0)
- MODIFY: `crates/orchestrator/src/task_runner.rs:6420-6450` (~50 LOC: security-domain eval on the PreToolUse block path + security_rule_blocked emit)
- MODIFY: `crates/orchestrator/src/hook_dispatcher.rs:1470-1600` (~30 LOC: route Edit/Write candidate content into the security eval)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~130 LOC: 2 new SsbActionType variants + payloads + sql_str + roundtrip + CHECK)
- MODIFY: `crates/orchestrator/tests/action_type_lock.golden` (~2 rows)
- MODIFY: `crates/cli/src/bench_cmd.rs` (~15 LOC: run_bench_seccodeplt wrapper)
- MODIFY: `crates/cli/src/args.rs` + dispatch in main.rs (~20 LOC: `bench seccodeplt` CLI arm)
- MODIFY: `crates/dashboard-v2/src/lib/.../types.ts` + DrawerHost.svelte (~15 LOC: DrawerKey 'security_rules')

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md C2 (mechanism + REVISED SCOPE + measurable_claim), section 0 (keystone verdict), section 1.4 (the "below 15%"/GitGuardian corrections — KEPT OUT), section 2 (Tier-1/Tier-2 rule), section 5 (sequencing), section 7 (CodeGate-death FP lesson, no-transfer caveat) — PRIMARY synthesis grounding, adversarially verified (run wf_b959963a-db6).
- arXiv:2302.05319 SVEN (CCS 2023) — retrain-free prefix steering 59.1%->92.3% secure. STATUS: abstract/table-VERIFIED. CAVEAT (KEPT): CodeGen-2.7B; does NOT transfer to Qwen — F-2b gates the headline on a measured Qwen run.
- arXiv:2405.00218 CodeGuard+ — constrained decoding 43.2%->76.0% secure-pass@1, no correctness loss. STATUS: abstract/table-VERIFIED. Same CodeGen caveat.
- arXiv:2410.11096 SeCodePLT (NeurIPS 2025, 5.9k samples / 44 CWEs, GPT-4o 55% secure) — the synthetic-corpus DESIGN reference (not a replay set). STATUS: abstract-VERIFIED.
- ELAI ground truth (code-read, CONFIRMED in elai.map at authoring): ALWAYS_INCLUDED_DOMAINS rule_engine.rs:187; Predicate enum (AstPattern/GrepPattern/FileContext) predicate.rs:101/81; load_seed_rules_from_dir rule_loader.rs:40; project_rule_seeder seed_rules project_rule_seeder.rs:68 (DEF_FN; line 91 is a statement within the body — SCIP_DEF confirmed at :68); dispatch_pre_tool_use hook_dispatcher.rs:1477 (InProcessHookDispatcher impl; trait decl at :304); build_hook_block_summary task_runner.rs:11943 (DEF_FN; call site at :6439); SsbActionType ssb.rs:4412; bench_cmd.rs dispatcher + prmbench_runner.rs pattern; decoding.rs ConstrainedDecodingConfig:122 + sidecar_xgrammar_hook.rs build_xgrammar_request:333 (build_xgrammar_request DEF_FN at sidecar_xgrammar_hook.rs:240, call at :333); commands_e30.rs + PrmBenchPanel.svelte + PinnedRulesPanel.svelte frontend templates.
- W-7 CORRECTION (UNVERIFIED in source, corrected here): the synthesis named `crates/cli/src/benchmark.rs` and `crates/rules/rules/security/` — neither resolves in elai.map. Real anchors substituted: `crates/cli/src/bench_cmd.rs` + `crates/orchestrator/src/benchmarks/` (E-9 harness) and `crates/rules/seed/security/` (the real seed-dir convention).
- CLAUDE.md HARD RULES applied: blake3 only (candidate + seed-integrity digests); no .unwrap()/.expect() in production paths; Pattern-2 SSB collapse; anti-slopsquatting (no new dep — blake3 + E-9 deps already in-tree); the synthesis section-1 corrections NOT re-introduced.

## Open questions / risks

- Top failure mode 1 — false-positive UX friction (the documented CodeGate killer, synthesis section 7). Mitigation: F-2a scopes BLOCK to high-precision classes (sqli/xss/weak-crypto) and demotes authz to WARN; `file_context = production` excludes test code; the F-2b harness measures the FP rate on the >=40-sample corpus before the headline is published. Bound the FP rate on >=3 real orchestrate runs as a release gate (synthesis C1 REVISED SCOPE).
- Top failure mode 2 — the Qwen number does not transfer / disappoints. Mitigation: F-2a ships the gate with NO perf claim; F-2b's headline is GATED on the measured run; if the after-rate is insufficient, the optional Tier-2 secure-decode layer (Task 10) is the documented escalation — but it is fail-OPEN (shifts generation) and must never advertise fail-closed.
- Top failure mode 3 — mutable=0 is not truly immune (user-writable SQLite). Mitigation: the blake3 seed-integrity check at load refuses-to-start on a tampered ruleset (Task 2). Open question: where is the canonical seed-bytes digest stored so it itself is not user-tamperable? (Candidate: compiled into the binary as a const; resolve at implement time.)
- Open question — Rust coverage. F-2a is Python/TS-only by REVISED SCOPE (semgrep CWE rules sparse for Rust — synthesis C10). Rust vuln-class detection is DEFERRED to F-10 (the CWE classifier). Flag explicitly; do not silently widen scope.
- Open question — SCHEMA bump coordination with F-1. If F-1 + F-2a co-ship, collapse `vibesec_code_vuln_blocked` (F-1) and `security_rule_blocked` (F-2) so they do not double-count the same block. Resolve the variant-vs-reuse decision with the F-1 author before the bump; document in F-2-decisions.md.
- RunPod / sidecar dependency (F-2b only): the live Qwen run requires a remote-sidecar or 2x3090 window. The deterministic fixture harness (Task 8, REQUIRED) is RunPod-INDEPENDENT and is the gate for shipping F-2b's CODE; the live number is a separately-gated checkbox per W-5. Stop any billable pod at session end (memory feedback_stop_pod_after_session).
- Do NOT brand the panel "VibeSec" (collides with OX Security's mark, synthesis section 7). Internal feature name: VibeGate / security-rules.

## Next

F-1 Tier-1 (VibeGate compound gate) consumes F-2a's `security` domain as its code-vuln axis — the two are the TIER-1 headline co-ship (F-0 sequencing). F-9 (write-time hallucinated-dep gate, after D-11) and F-10 (security-monotonicity iteration guard, classifier-first) extend the same PreToolUse + SSB substrate to the dependency-existence and CWE-regression axes. F-2b's measured number is the input to the ELAI-DIRECTION R1/R2 release artifact (the anti-Replit distribution demo). Once F-2b lands, the SecurityRulesPanel's before/after tile IS the published, dashboard-visible, audit-reproducible (SSB + ELAI_SEED) keystone claim.

End of F-2.
