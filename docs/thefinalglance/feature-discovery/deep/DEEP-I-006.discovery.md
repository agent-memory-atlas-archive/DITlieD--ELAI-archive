# DEEP-I-006 — exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L858-L873`, heading `### DEEP-I-006`; the next chunk begins at L875.
- Boundary: the ten exact manifest originals below were read completely, first through last logical line. No original assigned to another chunk was opened. `09-I-SUMMARY_INDEX.discovery.md` was used only to orient the comparison and identify summary additions/corrections; no source atom is derived solely from it.
- Evidence classes: **E** = explicit source requirement/status; **I** = inferred composition consequence; **A** = ambiguous, unresolved, or conditional; **C** = explicit conflict/correction. Category suffixes identify capability, workflow, schema/data/lifecycle, configuration/default, integration/event, security/privacy/compliance, failure/recovery, determinism/concurrency, observability/evidence, test/done gate, constraint/non-goal, dependency/status/deferred.
- Deduplication rule: repeated premise, revised-scope, wiring-contract, grounding, composition, wargame, and acceptance prose is represented once with all load-bearing anchors. Interaction, ambiguity/conflict, and summary-delta ledgers reference primary atoms and do not clone them.
- Integrity result: fresh `sha256sum`, `wc -l`, and byte counts matched every row and aggregate. Missing paths **0**; unexpected paths **0**; duplicate paths **0**; SHA mismatches **0**; line mismatches **0**; byte mismatches **0**.

## Exact processed manifest and integrity verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 126 | 7,458 | `c8b6f7a608d4197f045c23b94805e9913f4dcfdf8a3739a03be948354026509e` | `docs/thefinalglance/I-plan/I-16_kv_cache_pruning_quant.md` | exact |
| numbered/index original | 116 | 10,706 | `078783e757b8246068f36405735f72640d971b3dd6d2c7f4d0a86119d594cb2c` | `docs/thefinalglance/I-plan/I-17_tri_layer_evaluation_stack.md` | exact |
| numbered/index original | 104 | 9,294 | `341f1b798a54b24f2e684ea1894d7bec2aba36c8857a245d9d9a25469404494c` | `docs/thefinalglance/I-plan/I-19_tdd_enforcer_preflight.md` | exact |
| numbered/index original | 122 | 12,191 | `56514d820a02e8d10d50c7e3af84b97802ed3ec4e90188153eb1835e63e8c4b9` | `docs/thefinalglance/I-plan/I-24_isomorphic_canary_antigaming.md` | exact |
| numbered/index original | 98 | 8,284 | `04e955fcd4e741584a8903d8f37b947d3a23a3c675df3280f2667d96c7dad929` | `docs/thefinalglance/I-plan/I-26_dynamic_thinking_budgets.md` | exact |
| numbered/index original | 200 | 18,184 | `5f295045bec8cf4303304f9d059735b5439481201b246b0fd22d799c7ee8e2a2` | `docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md` | exact |
| numbered/index original | 132 | 11,085 | `15c3ca9aa6bb71561208b8cf9fa22f0f205d5d64447ca6dc9b8d0c3726579048` | `docs/thefinalglance/I-plan/I-43_claim_scoper_kill_gate.md` | exact |
| numbered/index original | 80 | 5,381 | `2039b01baa9e649e99857b52136bf03f3f7258a2adedbec1bd3309af65137cfa` | `docs/thefinalglance/I-plan/I-47_verifier_input_hardening.md` | exact |
| numbered/index original | 112 | 8,179 | `dfbfcecb6b7ecfe29e19288c5d397664effd62b1fe8560e113a2f59425e23647` | `docs/thefinalglance/I-plan/I-49_bedrock_azure_direct_lanes.md` | exact |
| numbered/index original | 72 | 4,700 | `d7206d6e212dc811e543e3faea3a53b415ee2683ebfea17ce675df6e4f78be8f` | `docs/thefinalglance/I-plan/I-53_recipe_runner_tool_pack.md` | exact |
| **Aggregate** | **1,162** | **95,462** | — | **10 files** | **exact** |

## Per-file exhaustive atomic ledgers

### I-16 — KV-cache pruning and quantization evaluation

Source: `docs/thefinalglance/I-plan/I-16_kv_cache_pruning_quant.md` (126/126 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I16-E001 | E/META | Track 14 `CACHE-INVARIANT ECONOMICS MIDDLEWARE`; lever `LOCAL-VRAM`; moderate grounding; `ACTIVATION+EVAL`; effort M; Wave 5; live leg RunPod-gated; binds ID-5 and IC-7. | I-16:L3 |
| I16-C001 | C/STATUS | Ratified correction: fp8/fp16 `--kv-cache-dtype`, Int4-class `--turboquant-bits`, `KVCompressor`/`_KVQuantCache`, and the `ELAI_STREAMING_LLM_SINKS` sink mask already ship; I-16 is an evaluation layer, not a new-lever implementation. | I-16:L4-L12 |
| I16-E002 | E/GAP | The absent surfaces are an ELAI-fixture evaluator, a `KvCacheTuned` decision row, and the `elai bench kv-tune` entry point. | I-16:L6-L8 |
| I16-E003 | E/SCOPE | Tier 1 is a deterministic no-GPU dry-run evaluator; it neither adds duplicate pruning/quant flags nor changes serving defaults. | I-16:L8-L10, L30-L36 |
| I16-E004 | E/DEFER | True SnapKV/ThinK-class eviction and new eviction flags remain Tier 2 because `position_ids` re-indexing is missing. | I-16:L10-L10, L37-L40 |
| I16-E005 | E/PROMOTION | MoE retargeting is a separate IC-7 checkbox; live adoption requires an ELAI-measured pass-rate band plus lower VRAM. | I-16:L11-L12, L37-L40, L107-L109 |
| I16-E006 | E/CFG | The dry-run matrix is `turboquant_bits ∈ {0,4,8}` × `kv_cache_dtype ∈ {fp16,fp8_e4m3,fp8_e5m2}` × `sinks ∈ {0,4}`: 18 explicit configurations. | I-16:L14-L18 |
| I16-E007 | E/FLOW | Each dry-run configuration instantiates the real `KVCompressor`, FA2 auto-detect, and `LogitsProcessor` assembly paths without booting a GPU model. | I-16:L16-L20 |
| I16-E008 | E/EVIDENCE | SnapKV/ThinK paper results remain design evidence only; ELAI measurements control promotion, while W-10 requires the deterministic harness to ship even if the RunPod leg does not. | I-16:L18-L20, L64-L66, L83-L85 |
| I16-E009 | E/DEP | The evaluator composes E-26/E-27/E-28 serving and FA2/`LogitsProcessor`, E-32 context-lift, B-37 transport for the live leg, and the `BenchLongcot` CLI/fixture pattern. | I-16:L22-L28, L79-L91 |
| I16-E010 | E/ARTIFACT | `python/elai_inference/eval_kv_tune.py::KvTuneEval` sweeps shipped levers, writes `metrics/kv_tune_<blake3>.toml`, and rejects an FA2-disabled configuration with a typed error. | I-16:L30-L34, L43-L44, L49-L53 |
| I16-E011 | E/SCHEMA | New flat SSB beacon `KvCacheTuned` carries `vram_gb`, `tok_s`, `fixture_pass_rate`, `config_hash`, and `adopt_decision`. | I-16:L34-L35, L49-L53 |
| I16-E012 | E/CLI | Production CLI wiring is `CliAction::BenchKvTune`, `bench_kv_tune.rs`, and the `main.rs` dispatch arm; the command supports dry-run and `--emit-ssb`. | I-16:L35-L35, L43-L45, L54-L55 |
| I16-E013 | E/DEFER | Live RunPod measurement and its Adopt/Reject decision, the MoE evaluation, and real eviction are independently gated Tier-2 work. | I-16:L37-L40 |
| I16-E014 | E/NON-GOAL | `server.py` receives no new argparse flags; evaluation sweeps the already-shipped `--turboquant-bits`, `--kv-cache-dtype`, and sink controls. | I-16:L42-L45 |
| I16-E015 | E/SCHEMA | The SSB action registry and payload must change in lockstep; `SCHEMA_VERSION` is derived from live `SSB_ACTION_TYPES.len()` and must not be hand-bumped or restated as a stale integer. | I-16:L46-L46, L52-L53, L83-L85, L107-L108 |
| I16-E016 | E/UI | There is deliberately no frontend pair or dashboard: the product surface is the CLI plus metrics TOML. | I-16:L47-L47, L72-L73, L122-L124 |
| I16-E017 | E/SYMBOL | The produced Python symbol is `python/elai_inference/eval_kv_tune.py::KvTuneEval`; the produced Rust/SSB symbols are the CLI dispatch and `KvCacheTuned`. | I-16:L49-L55 |
| I16-E018 | E/INVARIANT | Every matrix arm consumes shipped server levers and asserts that the E-26 FA2 and E-28 `LogitsProcessor` paths remain active. | I-16:L56-L59, L95-L98 |
| I16-E019 | E/REACH | Required production reachability is `cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb` to the Python evaluator. | I-16:L60-L62, L74-L77 |
| I16-E020 | E/DONE | Test-only callers do not satisfy R7 production reachability. | I-16:L61-L62 |
| I16-E021 | E/LIMIT | Duplicate `--kv-pruning-budget`/`--kv-quant-bits` and any serving-default change without a live `KvCacheTuned(adopt_decision: Adopt)` are banned. | I-16:L63-L66 |
| I16-E022 | E/LIMIT | Paper accuracy as a gate, assumed MoE retarget, FA2 disablement, and `LogitsProcessor` bypass are banned. | I-16:L66-L68 |
| I16-E023 | E/SEC | `config_hash` uses blake3 only; dry-run must leave VRAM/throughput null rather than fabricate them; production parse paths may not use `.unwrap()`/`.expect()`. | I-16:L69-L71 |
| I16-E024 | E/OBS | The no-GPU command must emit `KvCacheTuned{vram_gb:null,tok_s:null,fixture_pass_rate:1.0,config_hash:blake3(...),adopt_decision:DryRun}` and the matching metrics file; `Adopt` with null VRAM fails. | I-16:L74-L77, L93-L99 |
| I16-E025 | E/RECOVERY | Missing beacon, absent metrics, or silent FA2 disablement diagnoses unwired evaluator/CLI dispatch; the counter-move is to re-read the full `BenchLongcot` dispatch and Pattern-2 SSB lockstep, then rerun smoke. | I-16:L100-L102 |
| I16-E026 | E/FALLBACK | The default dry-run transport is subprocess `python -m elai_inference.eval_kv_tune --dry-run`; HTTP through `SidecarProvider` is deferred to the live leg. | I-16:L104-L105 |
| I16-E027 | E/ABORT | Stop rather than invent schema integers if the live action registry cannot be reread; stop and retain subprocess transport if dry-run would otherwise require a booted model. | I-16:L107-L109 |
| I16-E028 | E/VERIFY | Done gates include plan-gate check, `cargo check -p elai-cli --all-targets`, orchestrator `action_type_lock`, the production CLI smoke, and `production_caller_check` on new dispatch symbols. | I-16:L111-L116 |
| I16-E029 | E/REDTEAM | The red-team arms require failure for an Adopt row lacking live VRAM/pass-rate measurement and ban a duplicate-flag implementation that replaces shipped levers. | I-16:L118-L120 |
| I16-E030 | E/IDEMP | Clean-tree integration smoke produces exactly one DryRun beacon and one metrics artifact, with `config_hash` equal to blake3 of the relevant matrix entry. | I-16:L122-L124 |
| I16-A001 | A/THRESHOLD | The acceptable live pass-rate band and required VRAM decrease are not quantified. | I-16:L11-L12, L64-L66, L107-L109 |
| I16-A002 | A/DATA | The source defines an 18-arm matrix but the smoke requires exactly one row/file and a hash of “the matrix entry”; result aggregation, selected entry, and per-arm artifact cardinality are unspecified. | I-16:L16-L18, L95-L98, L122-L124 |

Dimension note: no user-data privacy or permission model is added. Recovery is limited to typed refusal, abort, and rerun guidance; retry policy for the live service is intentionally outside Tier 1.

### I-17 — tri-layer evaluation stack

Source: `docs/thefinalglance/I-plan/I-17_tri_layer_evaluation_stack.md` (116/116 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I17-E001 | E/META | Track 15 `DETERMINISTIC GATES + ANTI-GAMING VERIFICATION`; `TRUST-PROVABLE`; strong grounding; composition build; effort M; Wave 3; ID-4; blocking for I-4; IC-11 explicitly does not bind. | I-17:L3-L3, L85-L88 |
| I17-E002 | E/GAP | D-3/D-11/F-1/F-9/IS-8 substrate ships, but no single veto-first compositor exists; currently a hallucinated-import candidate can score nonzero before L1. | I-17:L4-L7, L24-L30 |
| I17-E003 | E/CAP | L1 composes compile, tests, F-1 VibeGate, F-9 dependency gate, and D-11 checkers. | I-17:L9-L12, L34-L37 |
| I17-E004 | E/INVARIANT | Any L1 failure is final score `0.0`; neither L2 nor L3 may rescore or resurrect the branch, and a soft `0.001` substitute is forbidden. | I-17:L10-L12, L21-L22, L70-L74 |
| I17-E005 | E/FAIL | A deterministic check exceeding the empirical 15-second bound aborts the branch and incurs a `HaltingBound` penalty; the timer must be the common `halt::BoundTimer` shared with I-8. | I-17:L12-L14, L34-L38, L98-L100 |
| I17-E006 | E/RELIABILITY | Oracles are rerun under seed variation; a result flip creates an SSB-backed flake quarantine keyed by `blake3(oracle-id + seed-range)`. | I-17:L14-L15, L34-L39, L55-L58 |
| I17-E007 | E/LIFECYCLE | A quarantined oracle is barred from both vetoing and vouching until rehabilitation; vote exclusion is a hard gate checked before every oracle read. | I-17:L14-L16, L70-L73 |
| I17-E008 | E/FLOW | L2 runs only on L1 survivors, using D-2 PPM plus I-23 unified conformal uncertainty; until I-23 lands it may call raw `conformal_calibration` through a registered stub, while E-19 PRM-lite is the pre-PPM scorer. | I-17:L15-L17, L28-L30, L38-L40, L64-L65 |
| I17-E009 | E/ESCALATION | L3 fires when survivor L2 variance exceeds a threshold and routes to D-13 for a concrete counterexample. | I-17:L17-L18, L40-L41, L65-L65 |
| I17-E010 | E/DEFER | The D-3 seam for TypeScript/Python verifier packs ships now, but the packs are Tier 2 and require per-language harness confirmation plus a W-10 checkbox. | I-17:L19-L20, L42-L44 |
| I17-E011 | E/NON-GOAL | L2-on-veto, soft veto, and absorption/reimplementation of D-3/D-11/F-1/F-9 are explicitly excluded. | I-17:L21-L22, L70-L75 |
| I17-E012 | E/STATUS | D-3, D-11, F-1, F-9, IS-8, IS-7, and D-13 are asserted shipped; exact dispatch/hook locations remain execution-time W-7 re-grep obligations. | I-17:L24-L32 |
| I17-E013 | E/API | New `crates/orchestrator/src/verifier_stack.rs::TriLayerStack` exposes `run_verdict(&Branch) -> VerdictRow`. | I-17:L46-L50, L54-L57 |
| I17-E014 | E/API | `FlakeQuarantineLedger` provides SSB read/write operations; its stable identity is the blake3 oracle/seed-range key. | I-17:L54-L58, L75-L75 |
| I17-E015 | E/EVENT | `TriLayerVerdict` kinds are exactly `HardVeto`, `ScoredSurvivor`, `DisagreementEscalated`, and `FlakeQuarantined`. | I-17:L41-L41, L49-L50, L58-L58 |
| I17-E016 | E/INTEG | The compositor calls `ConsensusVerifier`, `run_all_checkers`, `VibeGate::evaluate`, and `run_dep_gate`; it must not copy their logic. | I-17:L59-L64, L94-L99 |
| I17-E017 | E/INTEG | It consumes I-23/raw calibration for uncertainty and `crates/orchestrator/src/adversary.rs` for L3. | I-17:L64-L65, L97-L100 |
| I17-E018 | E/REACH | I-4 `search/ladder.rs::rank_candidates` invokes one verdict per branch on the production search path. | I-17:L66-L68 |
| I17-E019 | E/REACH | The existing `task_runner.rs` baby-step audit dispatch is the second production caller and replaces inline gate calls. | I-17:L47-L49, L67-L69 |
| I17-E020 | E/DONE | A `#[cfg(test)]` or contracts-only caller never satisfies R7; both production chains must be retraced at execution. | I-17:L67-L69 |
| I17-E021 | E/SEC | A hard-vetoed branch may trigger no L2/L3 call, and a quarantined oracle may participate in no vote. | I-17:L70-L73 |
| I17-E022 | E/LIMIT | Production paths ban copied verifier logic, `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. | I-17:L73-L75 |
| I17-E023 | E/UI | Existing cockpit branch cards gain verdict chips sourced from SSB viewer/subscription wiring; no new drawer is allowed and cross-language diff must pass. | I-17:L51-L52, L76-L79 |
| I17-E024 | E/OBS | The hallucinated-import fixture must emit `TriLayerVerdict(HardVeto)` with `gate=L1_compile` before any L2 call and journal L2 `call_count=0`. | I-17:L80-L83 |
| I17-E025 | E/OBS | L1 survivors emit `ScoredSurvivor`; a seeded flipping oracle emits `FlakeQuarantined`. | I-17:L80-L83 |
| I17-E026 | E/REDTEAM | The broken arm that turns HardVeto into a strong negative weight must resurrect the wrong branch and make the contract fail; the unbroken arm selects an L1 survivor. | I-17:L82-L83 |
| I17-E027 | E/STATUS | Because effort is M, I-0 requires a full-form expansion before execution. | I-17:L102-L102 |
| I17-E028 | E/EVIDENCE | J-11’s source-cited Arbor amendment supplies prior art: missing held-out verified score or failure of `is_improvement` rejects merge regardless of the LLM-reported score. | I-17:L104-L112 |
| I17-E029 | E/TEST | The same amendment reinforces separate iterative `B_dev` and milestone-only `B_test` held-out discipline. | I-17:L112-L114 |
| I17-A001 | A/CFG | `n` reruns, the seed range, quarantine persistence, and rehabilitation criteria are not defined. | I-17:L14-L16, L38-L39 |
| I17-A002 | A/THRESHOLD | The survivor-variance threshold that triggers L3 is unspecified. | I-17:L17-L18 |
| I17-A003 | A/STATUS | The exact I-4/task-runner hook locations and the registered I-23 fallback’s removal/upgrade lifecycle are deferred to execution-time discovery. | I-17:L32-L32, L38-L40, L47-L49, L67-L69 |
| I17-A004 | A/SCHEMA | The source names a `HaltingBound` penalty but does not assign it an event kind, payload, score interaction beyond branch abort, or retry/recovery behavior. | I-17:L12-L14, L37-L38 |

Dimension note: this stack governs evaluation authority but introduces no credentials, external network calls, retention policy, or multi-writer concurrency. Failure is deliberately fail-closed; only flake rehabilitation is a recovery lifecycle, and its mechanics remain open.

### I-19 — symbol-level TDD preflight

Source: `docs/thefinalglance/I-plan/I-19_tdd_enforcer_preflight.md` (104/104 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I19-E001 | E/META | Track 15; lever `CANT-WRECK`; strong grounding; extension; effort S-M; Wave 3; binds IC-12. | I-19:L3-L3 |
| I19-E002 | E/SCOPE | IC-12 correction limits I-19 to the missing symbol-level PreToolUse predicate; shipped `TESTING_FIRST`, E-13 anti-over-mock, and F-1 remain owners of their existing gates. | I-19:L4-L8, L26-L27, L31-L38 |
| I19-E003 | E/REACH | The predicate runs before writes to `.rs`, `.py`, or `.ts` source files while the FSM is `EXECUTING`. | I-19:L10-L12, L71-L73 |
| I19-E004 | E/ANALYSIS | Tree-sitter parses the pending diff and extracts modified/added Rust `pub fn`/`fn`, Python `def`/`class`, and corresponding TypeScript public symbols. | I-19:L11-L13, L40-L43 |
| I19-E005 | E/GATE | A test file must contain a symbol whose name shares a substring with the modified symbol or is linked by an optional test-attribution map. | I-19:L12-L15, L42-L44 |
| I19-E006 | E/GATE | Before disk write, staged Rust runs `cargo check --tests` and staged Python runs `pytest --collect-only`; syntax/import/type failure blocks. | I-19:L16-L18, L43-L44, L70-L70 |
| I19-E007 | E/NON-GOAL | Preflight is compile/collection only; it may not run the actual test suite because scoped tests remain auditor-owned. | I-19:L16-L18, L28-L29, L74-L76 |
| I19-E008 | E/EXEMPT | A diff with no new public symbol is a pure-refactor candidate and bypasses the symbol requirement; a line-delta versus name-delta cross-check disambiguates structural moves/renames. | I-19:L19-L21, L44-L45 |
| I19-E009 | E/DEFAULT | `ELAI_TDD_PREFLIGHT_ENABLE` ships default-off in `wire-dark-allowlist.toml` as `type=experiment`, with expiry date and `promotion_plan_id=i-19`. | I-19:L22-L25, L45-L47, L53-L57, L63-L65 |
| I19-E010 | E/PROMOTION | Default-on requires measured false-positive rate on at least three real orchestrate runs where the gate fires on a correct edit; results go to `metrics/tdd_preflight_fp.toml`. | I-19:L22-L25, L47-L50, L84-L87 |
| I19-E011 | E/NON-GOAL | Full per-edit coverage instrumentation, forced test generation, and any implementation that blocks refactor edits are excluded. | I-19:L28-L29 |
| I19-E012 | E/STATUS | `TESTING_FIRST`, E-13, the compound F-1 hook tiers, and workspace tree-sitter grammars are asserted shipped; exact hook locations must be regrepped under W-7. | I-19:L31-L38 |
| I19-E013 | E/SCOPE | Tier 1 ships the tree-sitter diff parser, symbol cross-reference, Rust/Python syntax-only staging checks, refactor exemption, allowlist registration, event, and FP harness. | I-19:L40-L48 |
| I19-E014 | E/EVENT | `TddPreflightBlocked` discriminators are `MissingTestDelta`, `SyntaxDryRunFailed`, and `RefactorExempted`. | I-19:L46-L48, L57-L57, L63-L65 |
| I19-E015 | E/DEFER | Both default-on promotion and full line-level coverage-delta instrumentation are Tier 2; promotion is a separate patch-N. | I-19:L49-L51 |
| I19-E016 | E/INTEG | New `crates/orchestrator/src/tdd_preflight.rs::TddPreflight::check(&Edit) -> PreflightResult` is added as a predicate after existing F-1 axes; the hook is not forked. | I-19:L53-L56, L61-L69 |
| I19-E017 | E/UI | TDD block reasons use the existing F-1 gate panel and IPC path, add no drawer/endpoint, and require cross-language diff. | I-19:L57-L59, L80-L83 |
| I19-E018 | E/SYMBOL | Produced durable artifacts are the predicate, `TddPreflightBlocked`, the allowlist row, and the FP TOML. | I-19:L61-L65, L84-L87 |
| I19-E019 | E/DEP | The implementation consumes `hooks/run_pre_tool_use`, E-13’s neighboring axis, existing knowledge-crate grammar bindings, and external compiler/collector commands; none is reimplemented. | I-19:L66-L70, L98-L102 |
| I19-E020 | E/DONE | Only the production PreToolUse dispatch satisfies R7; contracts-only callers do not. | I-19:L71-L73 |
| I19-E021 | E/LIMIT | `cargo test` in preflight, unmeasured default-on, blocking pure renames, and rebundling tree-sitter grammars are banned. | I-19:L74-L79 |
| I19-E022 | E/SEC | Production paths ban `.unwrap()`/`.expect()` and fabricated false-positive measurements. | I-19:L79-L79 |
| I19-E023 | E/FAIL | Adding a new public function without a matching test-symbol delta refuses the edit before disk and emits `MissingTestDelta`. | I-19:L84-L87 |
| I19-E024 | E/OBS | Staged syntax failure emits `SyntaxDryRunFailed`; rename-only passage emits `RefactorExempted`. | I-19:L84-L87 |
| I19-E025 | E/TEST | A matching failing test is a positive arm and permits the edit; a `--skip-preflight` broken arm bypasses the gate and must fail the contract. | I-19:L84-L87 |
| I19-E026 | E/ARTIFACT | The three-run report at `metrics/tdd_preflight_fp.toml` must exist before promotion. | I-19:L84-L87 |
| I19-E027 | E/INTEG | The feature composes the FSM state precondition, F-1/E-13 compound axes, and existing grammars; only the predicate and FP harness are net-new. | I-19:L89-L102 |
| I19-C001 | C/SCOPE | The source resolves “coverage-delta exemption” into a Tier-1 no-new-public-symbol plus line/name heuristic and explicitly defers full line-level coverage instrumentation. | I-19:L19-L21, L49-L51 |
| I19-A001 | A/THRESHOLD | No numerical acceptable false-positive bound is supplied after the minimum three runs. | I-19:L22-L25, L49-L50 |
| I19-A002 | A/LANGUAGE | TypeScript is in the parse/reachability scope, but no TypeScript syntax-only command is named alongside Rust/Python. | I-19:L11-L12, L16-L18, L36-L36, L71-L72 |
| I19-A003 | A/SCHEMA | The optional attribution-map path/schema, substring normalization, collision handling, and test-file discovery rules are not defined. | I-19:L12-L15, L42-L44 |
| I19-A004 | A/EVENT | `RefactorExempted` is carried by an action type named `TddPreflightBlocked` even though the edit is allowed; consumer semantics for “non-blocking blocked-row” are not specified. | I-19:L46-L48, L57-L57, L84-L87 |

Dimension note: the gate changes write permission only at PreToolUse and has no network/privacy/retention behavior. Its recovery is edit correction or exemption; retries and concurrent edit ordering are owned elsewhere.

### I-24 — isomorphic canary anti-gaming

Source: `docs/thefinalglance/I-plan/I-24_isomorphic_canary_antigaming.md` (122/122 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I24-E001 | E/META | Track 15; `ANTI-GAMING`; strong grounding; extension; effort S-M; Wave 4; binds ID-4. | I-24:L3-L3 |
| I24-E002 | E/CAP | The feature detects scorer-side style bias in I-17 L2 and complements, rather than copies, H-44 candidate-side trap detection. | I-24:L4-L9, L25-L28 |
| I24-E003 | E/DET | With probability `p`, sampled uniformly and deterministically under `ELAI_SEED`, the L2 path injects the original candidate plus a frozen isomorphic variant. | I-24:L11-L15, L47-L53 |
| I24-E004 | E/TRANSFORM | Tier-1 relation types are `ForWhileEquiv`, `RenameIdempotent`, and `ConstantExtraction`. | I-24:L13-L18, L47-L50 |
| I24-E005 | E/INVARIANT | Transformations may alter surface style but not control flow, data flow, or type constraints. | I-24:L16-L20 |
| I24-E006 | E/VALIDATION | Both variants run through L1; differing L1 outcomes identify a malformed canary/obfuscation bug, not scorer bias. | I-24:L18-L20, L85-L88 |
| I24-E007 | E/MEASURE | Bias is a score delta above an ELAI-measured known-unbiased-scorer threshold stored at `metrics/canary_bias_threshold.toml`; paper thresholds are prohibited. | I-24:L21-L24, L49-L52, L89-L89 |
| I24-E008 | E/LIFECYCLE | One positive detection is a warning; `N` consecutive detections in one run set the scorer’s weight to zero for the remainder of that run and emit neutralization. | I-24:L21-L24, L51-L52 |
| I24-E009 | E/INTEG | H-44 and I-24 run in parallel through L2 and jointly inform the run verdict, while H-44 remains sole owner of its trap logic. | I-24:L25-L28, L76-L81 |
| I24-E010 | E/SEC | `fixtures/canary_pairs.json` is static, git-versioned, frozen out of band, listed in `.claude/gate-manifest.txt`, and covered by `gate_freeze_check.py`; the evaluated model never authors canaries. | I-24:L29-L32, L52-L55, L63-L64 |
| I24-E011 | E/NON-GOAL | Runtime model-generated canaries, semantic-changing pairs, and any H-44 reimplementation are excluded. | I-24:L33-L35 |
| I24-E012 | E/STATUS | IS-8 `RelationLibrary`/`MutationRunner`, H-44, I-17’s L2 seam, D-2 `PpmClient`, and the freeze checker are asserted shipped/available composition points; H-22/C-29 relationships require W-7 re-grep. | I-24:L37-L45, L103-L113 |
| I24-E013 | E/SCOPE | Tier 1 adds the three Rust relations, seeded injector, measured threshold, run-local neutralizer, frozen fixture, freeze coverage, and SSB event. | I-24:L47-L55 |
| I24-E014 | E/DEFER | New-language canary generation waits for per-language AST verification and dependency approval; cross-scorer correlation waits for multiple live scorers. | I-24:L56-L58 |
| I24-E015 | E/INTEG | `canary_injector.rs` is called immediately before PPM scoring, RelationLibrary is extended in place, and the frozen fixture is added to the existing manifest. | I-24:L60-L65 |
| I24-E016 | E/EVENT | `PrmCanaryEvent` kinds are `CanaryInjected`, `StyleBiasDetected`, and `ScorerNeutralized`. | I-24:L55-L55, L65-L65, L69-L75 |
| I24-E017 | E/UI | The existing branch board/scoring panel gains `SCORER OK`, `BIAS DETECTED`, and `NEUTRALIZED` health states via SSB subscription; no new surface is created and cross-language diff must pass. | I-24:L66-L67, L92-L97 |
| I24-E018 | E/SYMBOL | Produced symbols/artifacts are `CanaryInjector`, the three relations, the frozen JSON pair set, `PrmCanaryEvent`, and the threshold TOML. | I-24:L69-L75 |
| I24-E019 | E/DEP | Consumers are IS-8 RelationLibrary, H-44 output, I-17’s single L2 invocation, D-2 PPM, and the existing freeze checker. | I-24:L76-L81, L115-L120 |
| I24-E020 | E/REACH | Production reach is `TriLayerStack::run_verdict`’s L2 path; contracts-only callers do not satisfy R7. | I-24:L82-L84 |
| I24-E021 | E/LIMIT | Model-authored or semantic-changing canaries are hard blocks. | I-24:L85-L88 |
| I24-E022 | E/LIMIT | The injector may neither duplicate H-44 nor use paper thresholds nor let neutralization alter L1 HardVeto supremacy. | I-24:L88-L90 |
| I24-E023 | E/SEC | Production paths ban `.unwrap()`/`.expect()`, blake2/sha2, and fabricated bias results. | I-24:L91-L91 |
| I24-E024 | E/OBS | A seeded biased fixture emits injection, then style-bias, then neutralization after `N` consecutive detections. | I-24:L98-L101 |
| I24-E025 | E/DET | Canary pair choice and both scores must recompute under `ELAI_SEED`. | I-24:L99-L101 |
| I24-E026 | E/TEST | A known-unbiased scorer must emit only `CanaryInjected`, proving the detector does not overfire. | I-24:L100-L101 |
| I24-E027 | E/REDTEAM | A runtime dynamic-canary broken arm must fail the frozen-gate check. | I-24:L100-L101 |
| I24-E028 | E/TEST | Every new relation has an L1-verified semantic-equivalence test; the UI path requires cross-language diff. | I-24:L47-L50, L92-L97 |
| I24-A001 | A/CFG | Injection probability `p`, warning threshold, consecutive-detection count `N`, and reset timing within a run are unspecified. | I-24:L11-L12, L21-L24 |
| I24-A002 | A/MEASURE | The prose uses signed `score(original)-score(obfuscated) > threshold`; it does not say whether reverse-direction or absolute divergence also constitutes style bias. | I-24:L21-L24 |
| I24-A003 | A/LIFECYCLE | The known-unbiased calibration corpus, recalibration trigger, threshold versioning, and behavior after the run ends are not specified. | I-24:L21-L24, L49-L52, L74-L75 |

Dimension note: canary authorship/freeze is the principal security and anti-self-evaluation boundary. No personal-data privacy policy, network dependency, retry loop, or concurrent scorer-update protocol is defined.

### I-26 — dynamic thinking budgets

Source: `docs/thefinalglance/I-plan/I-26_dynamic_thinking_budgets.md` (98/98 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I26-E001 | E/META | Track 16 `ROUTING + BUDGETS + MEASUREMENT`; `STEERING`; strong grounding; extension; effort M; Wave 4. | I-26:L1-L3 |
| I26-E002 | E/CAP | I-26 extends H-42’s serial-depth budget discipline from the main GRC chain to every I-4 search-node expansion. | I-26:L4-L9, L11-L14 |
| I26-E003 | E/ALGO | `BudgetController` computes `node_budget(depth) = base * depth_factor^depth`, capped by the supreme E-39 hard cost ceiling. | I-26:L13-L15, L55-L64 |
| I26-E004 | E/COLLAPSE | When accumulated spend across all active siblings reaches the 85%-collapse condition, new expansions stop. | I-26:L15-L16, L35-L38 |
| I26-E005 | E/FLOW | Collapse scores the current frontier with I-17 L2, synthesizes the best result, and emits I-27 `ConvergenceCurveMeasured` evidence. | I-26:L15-L18 |
| I26-E006 | E/EARLY-STOP | If the top L2 surviving score fails to improve by `delta_min` for five consecutive supersteps, the search collapses early. | I-26:L17-L18, L35-L38 |
| I26-E007 | E/PROVIDER | Cloud architecture/reasoning requests use `effort` (`auto`/`expert`); `budget_tokens` must be structurally absent on Opus 4.7+ paths. | I-26:L19-L21, L35-L41, L55-L59 |
| I26-E008 | E/INVARIANT | Reasoning or extended-thinking roles may never be force-wrapped as JSON. | I-26:L21-L25, L70-L74 |
| I26-E009 | E/SCHEMA | No action type is added: H-42 `ReasoningBudgetScaled` is reused, with in-place `depth_level`/`plateau_fired` payload extension only if live schema lacks them. | I-26:L22-L23, L60-L61, L84-L90 |
| I26-E010 | E/STATUS | H-42 event, E-39 ceiling, and E-19 confidence gate are plan-asserted shipped; the executor must re-grep their live definitions, and I-4 is the Wave-4 predecessor. | I-26:L27-L32 |
| I26-E011 | E/SEC | The collapse controller cannot be generator-reachable under R8, and mutable budget values may not enter the HD-2 frozen cache prefix. | I-26:L32-L33 |
| I26-E012 | E/SCOPE | Tier 1 creates `crates/orchestrator/src/routing/budget_controller.rs` with depth scaling, 85%/plateau collapse, and provider effort mapping. | I-26:L35-L38, L48-L52 |
| I26-E013 | E/TEST | `i26_dynamic_thinking_budgets_contract.rs` checks Opus request-builder field absence and `ReasoningBudgetScaled` depth/collapse metadata. | I-26:L39-L41 |
| I26-E014 | E/ARTIFACT | Tier 1 also writes `workflow/reasoning/I-26-decisions.md`. | I-26:L42-L42 |
| I26-E015 | E/DEFER | Journal-trained `depth_factor` and effort levels beyond `auto`/`expert` wait for I-27 trajectory volume and measured cost/quality deltas. | I-26:L44-L46 |
| I26-E016 | E/INTEG | The controller hooks I-4 search expansion/collapse and modifies the existing cloud request builder (for example `providers/src/anthropic_api.rs`) rather than forking either. | I-26:L48-L53, L92-L96 |
| I26-E017 | E/API | Produced API is `routing::budget_controller::BudgetController`; provider output replaces Opus `budget_tokens` with `effort`. | I-26:L55-L59 |
| I26-E018 | E/DEP | The controller consumes H-42 event schema, E-39 cap, E-19 confidence prior, and the I-4 expansion loop through their existing interfaces. | I-26:L60-L64 |
| I26-E019 | E/REACH | Production reach is `elai run <task>` → I-4 ladder → `BudgetController` at each node, with SSB rows as off-agent evidence. | I-26:L66-L68 |
| I26-E020 | E/LIMIT | Opus manual budgets, reasoning-role force-JSON, paper-derived threshold constants, `.unwrap()`/`.expect()`, blake2, and sha2 are banned. | I-26:L70-L74 |
| I26-E021 | E/UI | Existing H-15/I-4 branch cards show a per-node remaining-budget bar from `depth_level`, `budget_remaining`, and `plateau_fired`; no new dashboard crate, and cross-language diff covers Tauri→Svelte. | I-26:L53-L53, L76-L77 |
| I26-E022 | E/OBS | Deep-fixture runs journal per-node depth; cloud Opus rows contain effort and zero `budget_tokens` request bytes. | I-26:L79-L82 |
| I26-E023 | E/IDEMP | Once a collapse marker lands, no later `SearchNodeEvent(Expanded)` may exist; journal order makes post-collapse expansion structurally testable. | I-26:L79-L82 |
| I26-E024 | E/FRESHNESS | Current Anthropic effort-parameter API shape must be web-verified before modifying the builder. | I-26:L84-L87 |
| I26-E025 | E/DEP | RD-1..RD-4 bind because this controls search reasoning budgets; the action-type delta is explicitly baseline +0. | I-26:L88-L90 |
| I26-C001 | C/THRESHOLD | The source alternates among spend “exceeds 85%,” observable `budget_pct>=85`, and the title/premise’s “at 85%”; exact boundary equality is internally inconsistent. | I-26:L5-L7, L15-L16, L79-L82 |
| I26-A001 | A/CFG | `base`, `depth_factor`, `delta_min`, total-envelope calculation, token accounting unit, and synthesize-best tie-break are unspecified. | I-26:L13-L18 |
| I26-A002 | A/SCHEMA | Whether `ReasoningBudgetScaled` actually lacks fields, what an in-place migration looks like, and whether `ConvergenceCurveMeasured` or the reused row is the canonical collapse marker are edit-time decisions. | I-26:L17-L18, L22-L23, L40-L41, L79-L82 |
| I26-A003 | A/PROVIDER | Exact model-version detection and the current request enum/schema for `effort` remain external-document and live-code discoveries. | I-26:L19-L21, L39-L41, L48-L50, L84-L87 |

Dimension note: no new user permission, data retention, or retry behavior is added. Deterministic stop ordering and the hard cost ceiling dominate; concurrent siblings share one aggregate envelope, but synchronization mechanics are not specified.

### I-30 — verifier-ranked SFT exhaust

Source: `docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md` (200/200 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I30-E001 | E/META | Track 17 `OPEN-WEIGHT FLYWHEEL`; `ALIGNMENT`; strong grounding; build; effort L; Wave 5; depends on I-4; binds ID-6 and IC-7; full-form expansion is mandatory before execution. | I-30:L1-L10 |
| I30-E002 | E/STRATEGY | Rejection-sampling SFT is the first distillation step because it captures verifier-ranked search gains without RL instability; I-31 GRPO follows only after blocking guards. | I-30:L4-L8, L21-L24 |
| I30-E003 | E/ELIGIBILITY | `SftExhaust` walks only sealed RRAs; in-progress or manually closed RRAs cannot yield training rows. | I-30:L12-L18, L44-L48 |
| I30-E004 | E/SELECTION | For each sealed run, the nominal winner is a root-to-verified-`COMPLETE` path selected by `TriLayerVerdict(Promoted)` or highest-L2 `PhaseComplete` among L1 passers. | I-30:L13-L17 |
| I30-E005 | E/SCHEMA | Each JSONL record contains system prompt, every trajectory turn, and final verified output. | I-30:L16-L17 |
| I30-E006 | E/PROVENANCE | Every record carries `blake3(RRA_bytes)` and fails verification if it does not match the sealed RRA on disk. | I-30:L17-L18, L81-L83 |
| I30-E007 | E/NEGATIVE | Each winner is paired with the highest-L2 fluent sibling that received an L1 deterministic veto. | I-30:L18-L21 |
| I30-E008 | E/SCHEMA | Positive and negative examples share the JSONL and are distinguished by a label field. | I-30:L20-L21 |
| I30-E009 | E/NON-GOAL | I-30 exposes SFT extraction only: DPO fine-tuning, PPO, and GRPO are forbidden; inherited E-55 `HARD-AVOID` is not reopened. | I-30:L21-L23, L29-L31, L81-L84 |
| I30-E010 | E/TIER | Tier 1 ships offline dataset extraction and gate linting; the actual LoRA GPU run is a separate RunPod Tier-2 checkbox. | I-30:L23-L24, L44-L57 |
| I30-E011 | E/MODEL | The canonical target remains Qwen2.5-Coder; a MoE/Qwen3-Coder-Next retarget is a separate measured RunPod evaluation, never an assumed swap. | I-30:L25-L26, L54-L57, L84-L86 |
| I30-E012 | E/PROMOTION | Any LoRA checkpoint must pass H-17 trial-N, H-18 deflation, H-28 reward-tampering, D-6 held-out, and D-7 cross-domain-canary gates under ID-6. | I-30:L26-L31, L33-L42 |
| I30-E013 | E/SEC | The checkpoint evaluator/harness and training scripts are frozen artifacts outside the training loop’s write boundary under R8. | I-30:L27-L29, L42-L42 |
| I30-E014 | E/NON-GOAL | Unsealed inputs and promotion without the complete ID-6 chain are explicitly excluded. | I-30:L29-L31 |
| I30-E015 | E/DEP | The sealed source is existing IS-10 `build_rra`, `write_rra`, `read_rra`, `verify_rra`, and attestation; RRA construction/verification must not be copied. | I-30:L33-L35, L97-L108 |
| I30-E016 | E/DEP | Existing IS-16 `trial_ledger`, `deflation_gate`, `holdout_budget`, and reward/self-validation gates provide the promotion chain; I-4 provides trajectories. | I-30:L35-L39, L71-L75 |
| I30-E017 | E/INTEG | When I-28 is available, the sealed walker calls `DpoPairExtractor::run` with `DpoProvenance{rra_sealed:true}` and must not consume current live `DpoPairExtracted` rows because `task_runner` hardcodes false. | I-30:L39-L42, L71-L75, L106-L110 |
| I30-E018 | E/ARTIFACT | Training scripts/evaluator are frozen outside the loop; the dataset itself is a config-class, provenance-bearing artifact. | I-30:L36-L42, L97-L103 |
| I30-E019 | E/CLI | Tier 1 creates `crates/cli/src/sft_exhaust.rs` and `elai dataset export --format sft`. | I-30:L44-L48, L59-L63 |
| I30-E020 | E/GATE | Export checks whether the result is tied to the IS-16 chain and names missing required rows; it may not claim promotion. | I-30:L48-L50, L81-L86, L189-L194 |
| I30-E021 | E/UI | The dataset-card drawer is explicitly deferred under `owner-ui-rework-hold-2026-08-06`; only backend `SftExhaustEvent` ships in Tier 1. | I-30:L49-L51, L59-L64, L89-L90 |
| I30-E022 | E/TEST | Tier 1 includes `crates/contracts/tests/i30_sft_exhaust_contract.rs` and `workflow/reasoning/I-30-decisions.md`. | I-30:L51-L52 |
| I30-E023 | E/DEFER | LoRA training, checkpoint promotion after a produced checkpoint, and MoE retarget are separately gated Tier-2 items; any new LoRA dependency needs anti-slopsquat review and user confirmation. | I-30:L54-L57 |
| I30-E024 | E/STATUS | `workflow/training/sft/` training scripts are a Tier-2 integration location and do not ship until dependency approval. | I-30:L59-L64 |
| I30-E025 | E/API | `SftExhaust` is the sealed-RRA walker/JSONL extractor/gate checker; output is `workflow/training/sft/<run_id>.jsonl`. | I-30:L66-L75 |
| I30-E026 | E/EVENT | `SftExhaustEvent` emits `PathExtracted` per winning path, `NegativePaired` per negative, and final `DatasetSealed` with `record_count` and `provenance_verified_count`. | I-30:L77-L79, L92-L95 |
| I30-E027 | E/REACH | Production reach is the dataset-export CLI; SSB rows and JSONL are the off-agent artifacts. | I-30:L77-L79 |
| I30-E028 | E/FAIL | An unsealed RRA is rejected at the walker, yields no `PathExtracted`, and produces a visible lint/rejection. | I-30:L92-L95, L148-L164 |
| I30-E029 | E/FAIL | Provenance mismatch fails closed, emits `DatasetSealed(provenance_mismatch)` or contract-equivalent failure, and may not silently accept/write a training row. | I-30:L92-L95, L150-L165 |
| I30-E030 | E/TRACE | Negative pairs must cite real vetoed branches by `TriLayerVerdict(HardVeto)` row ID. | I-30:L92-L95 |
| I30-E031 | E/LIMIT | Dataset inputs must be sealed, no DPO/GRPO/PPO training may enter this unit, and no checkpoint may bypass the full ledger chain or assume a model swap. | I-30:L81-L86 |
| I30-E032 | E/SEC | Counts/statistics must derive from real RRAs; production paths ban `.unwrap()`/`.expect()` and blake2/sha2. | I-30:L85-L87, L173-L180 |
| I30-E033 | E/UI | Tier-1 implementation must not add the deferred UI drawer. | I-30:L89-L90, L173-L180 |
| I30-E034 | E/INTEG | Live SSB uses split `ssb/{action_type,payloads,schema,store,tests}.rs`; a monolithic `ssb.rs` or `ssb/action_type_lock.rs` must not be treated as current source. | I-30:L97-L104 |
| I30-E035 | E/EVIDENCE | J-11 amendment adopts difficulty-banded inclusion as design rationale from a cited DeNovoSWE ablation; its reported gain is never an ELAI performance target. | I-30:L114-L120 |
| I30-E036 | E/CFG | The amendment records five band floors: difficulty `[0,.2)→.90`, `[.2,.4)→.85`, `[.4,.6)→.80`, `[.6,.8)→.70`, `[.8,1]→.60`. | I-30:L122-L131 |
| I30-E037 | E/ELIGIBILITY | There is no global pass/fail inclusion gate; a trajectory is included only if its score meets its difficulty-band floor. | I-30:L133-L136 |
| I30-E038 | E/RANK | Among floor-qualified trajectories, a harder-band partial is preferred to an easier-band/trivial pass because it preserves more repository-level reasoning. | I-30:L136-L141 |
| I30-E039 | E/FEATURE | Difficulty `d_i` comes from I-25 when shipped or I-30 calibration; candidate structural signals include SCIP function count, cross-language bindings touched, and SSB variants—not Python-only executable-line count. | I-30:L142-L144 |
| I30-E040 | E/TEST | Contract cases must cover sealed, unsealed, tampered-provenance, and every difficulty band, writing only sealed/provenance-valid records. | I-30:L146-L156 |
| I30-E041 | E/TEST | At/beyond-floor scores include, below-floor scores exclude, and qualifying hard-band trajectories remain eligible without a global gate. | I-30:L150-L156 |
| I30-E042 | E/RECOVERY | Missing events/count equality require inspecting `dataset_cmd.rs` and the sealed walker; missing fixture cases must be added; unsealed/tampered/below-floor acceptance requires restoring the guard rather than caller filtering. | I-30:L158-L166 |
| I30-E043 | E/FORK | Negative-source default is the sealed direct I-28 call; live SSB pairs remain disallowed until `task_runner` carries seal status. | I-30:L168-L170 |
| I30-E044 | E/FORK | Tier-1 difficulty defaults to fixture/calibration constants; switching to I-25 later must preserve the same inclusion-floor contract fixture. | I-30:L169-L170 |
| I30-E045 | E/FORK | Module default is sibling `dataset_cmd.rs` plus `sft_exhaust.rs`; the dataset path must not be appended to `rra_cmd.rs`. | I-30:L170-L171 |
| I30-E046 | E/ABORT | Abort on unsealed ingestion, DPO/PPO/GRPO logic, promotion bypass, fabricated values, non-blake3 hashing, production unwrap/expect, or Tier-1 UI. | I-30:L173-L180 |
| I30-E047 | E/VERIFY | Required certification covers plan gate, plain-Cargo contract test, production caller check, wiring contract check, and action-type lock; off-agent verifier and serialized integrator results are recorded. | I-30:L182-L187 |
| I30-E048 | E/REDTEAM | Red-team arms cover unsealed smuggling, tampered hash, count falsification, DPO-training exposure, and absent ID-6 lint. | I-30:L189-L194 |
| I30-E049 | E/DONE | Integration smoke asserts `PathExtracted`, `DatasetSealed`, equal counts, and all sealed/unsealed/tampered/band fixtures under off-agent ownership. | I-30:L196-L196 |
| I30-E050 | E/STATUS | No recon question is declared open, but execution must re-grep RRA, task runner, split SSB, CLI, gate-chain, and action-lock seams. | I-30:L198-L198 |
| I30-C001 | C/FORK | Early prose permits re-deriving negative pairs from the search tree when I-28 is absent, but the later binding integration/amendment says call I-28 directly and “do not re-derive pairs.” | I-30:L18-L21, L74-L75, L106-L110 |
| I30-C002 | C/SCHEMA | Winner selection names `TriLayerVerdict(Promoted)`, but I-17’s exhaustive kind list contains no `Promoted` discriminator. | I-30:L13-L16; I-17:L41-L41, L49-L50, L58-L58 |
| I30-C003 | C/THRESHOLD | The amendment says the external numeric table is design rationale and not an ELAI target, yet then makes those exact floors Tier-1 inclusion behavior and requires preserving them when switching to I-25. | I-30:L114-L120, L122-L144, L168-L170 |
| I30-A001 | A/SCHEMA | JSONL field names/version, turn/tool encoding, label vocabulary, canonical RRA-byte serialization, and duplicate-run/idempotent rewrite policy are unspecified. | I-30:L13-L21, L66-L75 |
| I30-A002 | A/ELIGIBILITY | Root-to-verified-`COMPLETE` winner language conflicts conceptually with the later admission of qualifying hard-task partials; which partial trajectory endpoint becomes an export record is not defined. | I-30:L13-L17, L133-L141 |
| I30-A003 | A/GATE | Export-time ID-6 handling is variously “block,” “lint warning,” and “lint error,” while the dataset export records only gate-chain lint status; exact success/exit-code semantics are unresolved. | I-30:L48-L50, L73-L74, L81-L86, L97-L103, L189-L194 |
| I30-A004 | A/EVENT | `DatasetSealed(provenance_mismatch)` is used as a failure shape although the event’s status/error field and whether a failed dataset is semantically “sealed” are not specified. | I-30:L92-L95, L148-L165 |
| I30-A005 | A/MEASURE | I-30-owned calibration of `d_i`, normalization of the proposed structural signals, ties across bands, and score provenance are left undefined. | I-30:L133-L144, L168-L170 |

Dimension note: sealed-RRA admission, provenance, evaluator separation, and gate-chain promotion are the security/compliance boundaries. No raw-personal-data policy appears here. Export is fail-closed and deterministic by hash/count evidence; retry and concurrent exporters/file-locking are unspecified.

### I-43 — reproducibility-claim scoper and Phase-0 kill gate

Source: `docs/thefinalglance/I-plan/I-43_claim_scoper_kill_gate.md` (132/132 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I43-E001 | E/META | Track 19 `DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE`; `PROVABLE/FALSIFY-FIRST`; strong grounding; build; effort M; Wave 1; ID-5; composes I-38/G-6/H-23. | I-43:L1-L3 |
| I43-E002 | E/STRATEGY | Cheap reduced-N falsifiers must run before expensive G-6 SAFEDELTA work so build/claim investment is evidence-gated rather than faith-based. | I-43:L4-L12 |
| I43-E003 | E/PILOT | F0.1 drives the shipped SecCodePLT/SAFEDELTA runner gate-on versus gate-off over 50–100 git-stripped tasks with dynamic-exploit scoring. | I-43:L5-L8, L14-L20 |
| I43-E004 | E/MEASURE | Pilot output includes residual-insecure-rate delta sign, rough magnitude, and reviewer-reaching rate under gate-on. | I-43:L6-L8, L16-L20 |
| I43-E005 | E/KILL | The pilot fails if the security delta is flat/negative or the apparent win comes from blocking nearly everything before review. | I-43:L6-L8, L16-L20 |
| I43-E006 | E/CLAIM | Reduced-N output is a private go/no-go memo, not a headline or external marketing claim. | I-43:L19-L20, L91-L96 |
| I43-E007 | E/SCOPER | F0.2 runs about 20 identical security tasks with fixed seed on local Qwen2.5-Coder and cloud Claude lanes. | I-43:L8-L12, L21-L26 |
| I43-E008 | E/MEASURE | It measures action-stream divergence on security-critical paths; divergence above 5% forces trace-authentication-only wording. | I-43:L21-L26, L102-L110 |
| I43-E009 | E/CLAIM | Cloud may claim trace authentication/signed evidence, not deterministic replay; full recomputation is reserved for open-weight mode. | I-43:L8-L10, L21-L29 |
| I43-E010 | E/DET | Cross-lane claim scoping is distinct from H-23 same-environment replay; every measured number must recompute byte-identically from SSB under `ELAI_SEED`. | I-43:L25-L27, L102-L110 |
| I43-E011 | E/NON-GOAL | A single large-N run and cloud “deterministic replay” language are declined. | I-43:L27-L29, L91-L97 |
| I43-E012 | E/CLI | One `elai bench claim-scope` subcommand exposes pilot `--suite … --n 50` and divergence `--mode divergence` modes. | I-43:L14-L16, L43-L46 |
| I43-E013 | E/DEP | The implementation composes I-38 QDELTA, shipped G-6/SecCodePLT/SAFEDELTA dynamic-exploit/git-strip path, G-2 RRA capture, H-23 comparator shape, and B-37 lane routing. | I-43:L31-L40, L61-L68 |
| I43-E014 | E/STATUS | A reduced-N go/no-go driver and cross-lane divergence scoper are explicitly absent/net-new. | I-43:L40-L41 |
| I43-E015 | E/SCOPE | Tier 1 creates `bench_claim_scope.rs` with both modes, trap detection, claim wording, memo writer, and deterministic fixture path. | I-43:L43-L53 |
| I43-E016 | E/ARTIFACT | `workflow/reasoning/I-43-phase0-memo.md` records either three Phase-0 greens or a rescope. | I-43:L49-L53, L73-L80 |
| I43-E017 | E/EVENT | New `ClaimScopeMeasured` arms are `PilotDelta`, `CrossLaneDivergence`, and `ClaimScoped`. | I-43:L51-L53, L73-L80 |
| I43-E018 | E/TEST | A mock runner plus canned RRA traces exercises the real production code path deterministically without GPU/cloud. | I-43:L52-L53, L65-L68 |
| I43-E019 | E/DEFER | Real SecCodePLT/BaxBench/RunPod and real Claude/Qwen cross-lane runs are separate Tier-2 checkboxes. | I-43:L55-L57 |
| I43-E020 | E/NON-GOAL | F0.3 willingness-to-pay calls are business validation outside engineering scope, represented only as the memo’s third green. | I-43:L58-L59 |
| I43-E021 | E/INTEG | Tier 1 also creates the fixture contract and `I-43-decisions.md`; all production instrumentation is composed rather than rebuilt. | I-43:L61-L68 |
| I43-E022 | E/UI | An H-9-family Phase-0 cockpit tile is optional; W-6 permits no UI because bench TOML plus memo are the load-bearing surfaces. | I-43:L69-L71, L98-L101 |
| I43-E023 | E/API | Produced capabilities are `bench_claim_scope::run`, reduced-N/trap driver, cross-lane scoper/wording emitter, event, and memo. | I-43:L73-L80 |
| I43-E024 | E/DEP | Consumers are I-38, shipped SecCodePLT/SAFEDELTA, G-2 RRA, H-23 first-divergence pattern, and B-37 `PerRoleRuntime`. | I-43:L81-L86 |
| I43-E025 | E/REACH | Both CLI modes must traverse `bench_claim_scope::run` to SSB; test-only callers never satisfy R7. | I-43:L87-L90 |
| I43-E026 | E/LIMIT | Production bans big-N masquerading as pilot, replay wording on cloud, copied instrumentation, fabricated live results, `.unwrap()`/`.expect()`, and non-blake3 hashes. | I-43:L91-L97 |
| I43-E027 | E/OBS | Expected rows contain delta sign/magnitude/reviewer rate, cross-lane divergence percentage, and the selected honest wording. | I-43:L102-L107 |
| I43-E028 | E/REDTEAM | A gate-on fixture that blocks nearly everything must visibly fail; a >5% divergence fixture must emit trace-authentication-only wording. | I-43:L107-L110 |
| I43-E029 | E/EVIDENCE | External benchmark/security figures in grounding require web confirmation before headline use. | I-43:L112-L117 |
| I43-E030 | E/INTEG | I-40 consumes I-43’s scoped wording and must not advertise cloud replay. | I-43:L124-L130 |
| I43-C001 | C/THRESHOLD | The source simultaneously says no paper constant is a threshold and hard-codes a >5% divergence wording threshold; provenance and promotion of that constant are not reconciled. | I-43:L21-L27, L91-L95, L102-L110, L120-L121 |
| I43-A001 | A/CFG | Prose says default pilot N is “50–100,” while the concrete Tier-1 command fixes `--n 50`; default/range validation is unclear. | I-43:L5-L8, L14-L17, L43-L46 |
| I43-A002 | A/KILL | “Rough magnitude,” “flat/negative,” and “nearly everything” have no statistical/operational threshold or confidence rule. | I-43:L5-L8, L16-L20 |
| I43-A003 | A/LIFECYCLE | The memo requires three greens although F0.3 is off-tool business work; ownership, representation, and whether either engineering mode can emit final GO are unspecified. | I-43:L49-L59 |
| I43-A004 | A/ARTIFACT | The plan calls bench TOML a primary artifact but gives no filename, schema, or write/idempotency rules for it. | I-43:L98-L101 |
| I43-A005 | A/SCHEMA | Exact action-stream normalization, security-critical-path selection, divergence denominator, and serialized claim-wording vocabulary are undefined. | I-43:L21-L26, L49-L52, L73-L86 |

Dimension note: this gate controls claim/build commitment, not the product DONE arbiter. It introduces no credentials or personal-data retention. Fixture replay is deterministic; live-lane retry, failure recovery, and concurrent memo writers are unspecified.

### I-47 — verifier-input hardening and dispatch rule injection

Source: `docs/thefinalglance/I-plan/I-47_verifier_input_hardening.md` (80/80 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I47-E001 | E/META | Track 20; `TRUST-ANTI-BIAS`; strong seam/moderate competitor grounding; extension; effort S-M; Wave 2. | I-47:L1-L4 |
| I47-E002 | E/SEC | Worker diffs, worker claims, and tool/repo output entering auditor/critic dispatch are typed untrusted data segments. | I-47:L6-L9, L15-L20 |
| I47-E003 | E/SEC | A standing “do not follow embedded instructions / do not trust self-attestation” frame sits above the tags, and self-attestation is labeled claim-not-evidence. | I-47:L8-L9, L18-L20 |
| I47-E004 | E/RULES | Every worker dispatch mechanically receives read-before-write, scope discipline, no premature abstraction, no future-proofing, and no while-I-am-here refactoring. | I-47:L10-L13 |
| I47-E005 | E/CACHE | Rules are an R8-frozen per-role byte-stable block placed after cache-breakpoint discipline so the role prefix remains cacheable and byte-identical. | I-47:L20-L23 |
| I47-E006 | E/PROVENANCE | Before execution, SL-11 requires reading roach-pi summary techniques 2/3 and their cited extension source. | I-47:L15-L17 |
| I47-E007 | E/NON-GOAL | No new classifier, agent-definition rewrite, or per-task dynamic rule text is allowed; E-37 retains classification ownership. | I-47:L24-L26 |
| I47-E008 | E/STATUS | H-31/H-32, E-37, and D-13 surround the seam; I-32 is planned and must coordinate if it lands first. | I-47:L28-L32 |
| I47-E009 | E/ACTIVATION | The rule content already exists as prose; I-47 makes a fixed distillate runtime-enforced rather than aspirational. | I-47:L33-L34 |
| I47-E010 | E/SCOPE | Tier 1 ships in-process deterministic tag wrapping, role injection, and contract tests. | I-47:L36-L38 |
| I47-E011 | E/DEFER | Measured prompt-injection resistance waits for a separate attack-corpus bench session. | I-47:L39-L39 |
| I47-E012 | E/UI | Prompt assembly is internal, so W-6 explicitly justifies no frontend pair. | I-47:L41-L44 |
| I47-E013 | E/EVENT | Produced outputs are the tag wrapper, frozen role block, and `PromptHardeningApplied` rows. | I-47:L46-L48 |
| I47-E014 | E/DEP | The feature consumes the agent/auditor prompt builders, upstream E-37 classifier unchanged, and H-31/H-32 firewall objective. | I-47:L49-L50 |
| I47-E015 | E/REACH | Wrapping is unconditional on every auditor/critic production dispatch; rule injection is unconditional on every worker production dispatch. | I-47:L51-L52 |
| I47-E016 | E/LIMIT | Per-task rule text, model-authored tag content, and a skip flag are forbidden. | I-47:L53-L55 |
| I47-E017 | E/LIMIT | PromptArmor may not be copied and worker artifacts may not be stripped/truncated: tags wrap the actual content. | I-47:L54-L55 |
| I47-E018 | E/OBS | A malicious fixture emits `InputsTagged` and `RulesInjected`; journal bytes must show malicious instructions beneath the frame, claims labeled, and two identical worker role blocks. | I-47:L56-L65 |
| I47-E019 | E/FAIL | A no-tags build flag must be structurally absent; enabled hardening with zero evidence rows is an R1 failure. | I-47:L63-L65 |
| I47-E020 | E/SCHEMA | The declared event kinds are `InputsTagged`, `RulesInjected`, and `SelfAttestationStripped`, with baseline +1 and mandatory live-registry reread. | I-47:L73-L78 |
| I47-E021 | E/INTEG | It composes E-37, H-31/H-32, I-32, D-13, and I-21 without restating their classifier, role-contract, adversary, or separation logic. | I-47:L67-L77 |
| I47-C001 | C/SCHEMA | The prose requires self-attestation to be retained and labeled inside wrapped data, while the event vocabulary calls it `SelfAttestationStripped`; truncation is expressly banned. | I-47:L18-L20, L53-L55, L61-L65, L73-L78 |
| I47-A001 | A/SCHEMA | Exact XML/tag names, escaping/canonicalization, nesting rules, hardening-frame bytes, role-table schema, and freeze-manifest path are omitted. | I-47:L18-L23, L41-L48 |
| I47-A002 | A/DEFAULT | Reachability says behavior is unconditional/no-skip, yet failure prose refers to “while the feature is ON”; no feature flag/default/migration path is defined. | I-47:L51-L55, L63-L65 |
| I47-A003 | A/INTEG | Concrete prompt-builder modules, role taxonomy, and coordination order if I-32 changes those seams are deliberately unnamed. | I-47:L28-L32, L41-L50 |

Dimension note: this is a prompt-shape security control, not a classifier or UI. It does not define PII redaction, retention, network behavior, retries, or concurrent dispatch ordering; byte stability is its deterministic/cache invariant.

### I-49 — Bedrock/Azure direct lanes and cross-protocol shim

Source: `docs/thefinalglance/I-plan/I-49_bedrock_azure_direct_lanes.md` (112/112 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I49-E001 | E/META | Track 20; `REACH`; strong gap/moderate competitor grounding; reference-port build; effort L; Wave 4; live legs network-gated; full-form expansion and anchor re-grep are mandatory before execution. | I-49:L1-L6 |
| I49-E002 | E/NON-GOAL | The wholesale gajae provider layer is rejected; existing B-37, CLI bridges, OAuth, and local serving remain the architectural base. | I-49:L8-L13 |
| I49-E003 | E/PROVIDER | AWS Bedrock direct support implements SigV4 request signing and binary EventStream decoding for accounts without Anthropic API keys. | I-49:L10-L12, L19-L21 |
| I49-E004 | E/PROVIDER | Azure OpenAI direct support implements the Responses API shape for Azure deployments. | I-49:L11-L13, L19-L23 |
| I49-E005 | E/SHIM | An OpenAI-as-Anthropic normalization pattern presents OpenAI-shaped backends through the Anthropic-shaped internal seam. | I-49:L12-L13, L19-L23 |
| I49-E006 | E/PROVENANCE | Executor must first read `amazon-bedrock.ts`, `aws-sigv4.ts`, `aws-eventstream.ts`, `aws-credentials.ts`, `azure-openai-responses.ts`, and `openai-anthropic-shim.ts` in the named gajae provider directory. | I-49:L15-L18 |
| I49-E007 | E/API | Rust implementation lives in `crates/providers/src/runtime/bedrock.rs`, `azure_openai.rs`, and a shared normalization-shim module, modeled on `anthropic_api.rs`. | I-49:L19-L23, L44-L47 |
| I49-E008 | E/FRESHNESS | Before coding, current AWS InvokeModelWithResponseStream/Converse and Azure Responses documentation must be researched rather than trusted from training. | I-49:L24-L27 |
| I49-E009 | E/DEP | Candidate AWS crates require registry, repository, last-publish, and license checks under the anti-slopsquat protocol. | I-49:L24-L27 |
| I49-E010 | E/PERM | Every new dependency needs user approval; hand-roll protocol logic if candidate dependencies outweigh it. | I-49:L26-L27 |
| I49-E011 | E/NON-GOAL | GitLab Duo, Kimi, Vertex, replacement/removal of adapters or CLI bridges, and API-key storage changes are outside scope. | I-49:L28-L29 |
| I49-E012 | E/STATUS | Existing runtime adapters include Anthropic API, multiple CLI bridges, `genai_adapter`, Hermes, and local serving; `oauth.rs` stays. | I-49:L31-L34 |
| I49-E013 | E/GAP | Source records a dated grep finding of zero Bedrock/Azure symbols in `crates/`. | I-49:L35-L35 |
| I49-E014 | E/SCOPE | Tier 1 ships all protocol logic, deterministic recorded EventStream/Responses replay, and B-37 registry wiring without network. | I-49:L37-L40 |
| I49-E015 | E/DEFER | Real Bedrock and Azure calls are separate Tier-2 checkboxes requiring credentials/deployment and documented environment. | I-49:L41-L42 |
| I49-E016 | E/UI | `elai runtime list` and the existing `RuntimePanel` list both expose `bedrock` and `azure-openai`; no new component. | I-49:L44-L47, L60-L67 |
| I49-E017 | E/SYMBOL | Produced surfaces are Bedrock SigV4/EventStream, Azure Responses, OpenAI→Anthropic normalization, and replay fixtures. | I-49:L49-L51 |
| I49-E018 | E/DEP | The adapters consume B-37 `RuntimeAdapter`/registry, conditionally `genai_adapter`, and I-51 pricing/catalog metadata when available. | I-49:L52-L53 |
| I49-E019 | E/REACH | Selecting either backend traverses the real PerRoleRuntime request builder; `elai runtime list` proves registry reach and contracts-only callers do not satisfy R7. | I-49:L54-L55 |
| I49-E020 | E/TEST | SigV4 fixtures sign real canonical requests with recorded keys and must reproduce a documented known-good signature; mock signing is forbidden. | I-49:L56-L57, L60-L67 |
| I49-E021 | E/FAIL | Malformed EventStream frames fail closed with typed errors and may not panic or silently fall back to a different backend. | I-49:L56-L59, L65-L67 |
| I49-E022 | E/SEC | Production paths ban unwrap/expect and unverified dependencies. | I-49:L56-L59 |
| I49-E023 | E/EVENT | No action type is added; existing `RouteDecisionEvent`/runtime rows carry backend IDs. | I-49:L60-L64, L74-L77 |
| I49-E024 | E/DONE | Recorded frames and Responses payloads decode byte-exactly; corruption is the negative arm, and each enabled live checkbox must land one real completion. | I-49:L37-L42, L60-L67 |
| I49-E025 | E/INTEG | J-15 amendment requires fixtures to come from J-14 `rra export-fixture` repro packs rather than a private format. | I-49:L79-L87 |
| I49-E026 | E/DET | Replay matching key is `(model_id, blake3(message_content), tool_name)`; all three fields must match and positional indices are forbidden. | I-49:L85-L92 |
| I49-E027 | E/REDTEAM | Positional matching is a do-not-copy exhibit because insertion, retry-with-different-content, or skipped calls shift/mask subsequent fixtures. | I-49:L94-L104 |
| I49-E028 | E/PRIV | Fixtures must contain no auth headers; J-14’s export scrub is the inherited counter-design, and any header-bearing fixture is a credential leak. | I-49:L106-L110 |
| I49-E029 | E/INTEG | Composition retains B-37 as the frame, OAuth/CLI bridges as keep-list paths, I-50 as future credential supplier, and I-51 as catalog owner. | I-49:L69-L77 |
| I49-A001 | A/INTEG | The shim extends `genai_adapter` only if its live seam fits; otherwise module ownership is unresolved. | I-49:L19-L23, L52-L53, L74-L77 |
| I49-A002 | A/PROTOCOL | InvokeModelWithResponseStream versus Converse, candidate-crate versus hand-roll, retry/backoff, streaming cancellation, and Azure API-version choices remain pre-task decisions. | I-49:L24-L27 |
| I49-A003 | A/AUTH | Bedrock consumes credentials and a competitor `aws-credentials.ts` shape, but storage changes are excluded and I-50 is future; the Tier-1 credential acquisition seam is not defined. | I-49:L15-L18, L28-L29, L41-L42, L74-L77 |
| I49-A004 | A/SCHEMA | Stable backend IDs, normalized event schema, typed error variants, and runtime-row payload fields are not specified. | I-49:L44-L67 |
| I49-A005 | A/DEP | J-14 fixture availability/version compatibility and scrub-verification behavior are assumed, not status-checked in this source. | I-49:L79-L110 |
| I49-A006 | A/REACH | Tier 1 requires the production request-builder path to be reachable while all real network calls are deferred; how fixture transport is selected without a test-only caller is unspecified. | I-49:L37-L42, L54-L55, L60-L67 |

Dimension note: credential scrubbing, dependency approval, typed failure, and no-fallback behavior are explicit security boundaries. Live authentication, retry/backoff, rate limits, cancellation, idempotency, and concurrent streaming semantics are deferred/unspecified.

### I-53 — recipe runner and bounded utility tool pack

Source: `docs/thefinalglance/I-plan/I-53_recipe_runner_tool_pack.md` (72/72 lines).

| Atomic ID | Class | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| I53-E001 | E/META | Track 20; `TOOLS`; strong gap/moderate competitor grounding; build; effort S-M; Wave 2. | I-53:L1-L4 |
| I53-E002 | E/PREMISE | Repository recipes are owner-sanctioned real entry points; first-class tools replace arbitrary, undiscoverable/riskier bash shapes. | I-53:L6-L12 |
| I53-E003 | E/RECIPE | `recipe.rs` discovers recipes with descriptions and runs a selected named recipe through existing validation. | I-53:L14-L20 |
| I53-E004 | E/CAP | Tier-1 discovery covers Justfile targets, Makefile targets, and Cargo aliases. | I-53:L15-L18, L34-L36 |
| I53-E005 | E/ARCHIVE | `archive_reader.rs` lists and reads zip/tar.gz entries through read-only streams and performs no extraction to disk. | I-53:L19-L21, L51-L53 |
| I53-E006 | E/JSON | `json_tree.rs` performs path queries and bounded depth/size subtree rendering, explicitly marking truncation. | I-53:L20-L21, L51-L53 |
| I53-E007 | E/PROVENANCE | Before execution, SL-10 requires reading gajae’s recipe directory, `archive-reader.ts`, and `json-tree.ts`. | I-53:L14-L16 |
| I53-E008 | E/API | New files are `crates/tools/src/{recipe.rs,archive_reader.rs,json_tree.rs}`. | I-53:L17-L21, L38-L40 |
| I53-E009 | E/DECLINE | Cron, monitor, and background-job tools are permanently declined—not deferred—because orchestrator/Layer-1A schedules own them. | I-53:L22-L24 |
| I53-E010 | E/PERM | Existing archive dependencies must be checked first; a new format dependency requires anti-slopsquat review plus user approval, with hand-rolled tar listing as the fallback. | I-53:L25-L27 |
| I53-E011 | E/GAP | A dated SL-9 grep found no recipe/Just/Make runner in the intentionally lean tools crate. | I-53:L29-L32 |
| I53-E012 | E/SCOPE | Tier 1 ships and catalog-registers all three tools through existing validation gates. | I-53:L34-L35 |
| I53-E013 | E/DEFER | `taskfile.yml` and npm-scripts discovery wait until a target repository needs them. | I-53:L36-L36 |
| I53-E014 | E/INTEG | Catalog wiring follows the existing three-arm pattern; recipe commands flow through `bash_validation` and E-30 argv, and may become an I-46 redirect target later. | I-53:L38-L42 |
| I53-E015 | E/UI | Existing tool-call drawer rows render the three tools; no new component. | I-53:L41-L42 |
| I53-E016 | E/SYMBOL | Produced namespaces are `tools::recipe::{list,run}`, `tools::archive_reader::{list,read}`, and `tools::json_tree::{query,render}` plus registrations. | I-53:L44-L46 |
| I53-E017 | E/DEP | Recipe execution consumes existing `tools::bash_validation`, E-30 argv allowlist, and the tool registry without bypass or duplication. | I-53:L47-L48 |
| I53-E018 | E/REACH | Production agent tool dispatch on real repositories must reach each tool; contracts-only callers do not satisfy R7. | I-53:L49-L50 |
| I53-E019 | E/LIMIT | Recipe gate bypass, archive extraction, unbounded JSON output, unapproved dependencies, and production unwrap/expect are banned. | I-53:L51-L54 |
| I53-E020 | E/EVENT | No action type is added; existing `tool_call` rows carry the three tool IDs. | I-53:L55-L59, L68-L70 |
| I53-E021 | E/TEST | A live fixture must list Justfile targets and run one recipe with a journaled bash-validation verdict; absence of that verdict is the broken arm and fails R1. | I-53:L55-L62 |
| I53-E022 | E/TEST | Archive-read leaves the fixture directory byte-for-byte unwritten, while a 5 MB JSON query stays below the cap and marks truncation on the oversized arm. | I-53:L55-L62 |
| I53-E023 | E/INTEG | H-4 tool search supplies discoverability; E-35 supplies catalog pattern; I-46 is only a future redirect consumer. | I-53:L64-L70 |
| I53-A001 | A/RECIPE | Parsing rules for phony/pattern/private targets and Cargo config variants, recipe arguments/environment/cwd, descriptions, target validation, and duplicate names are unspecified. | I-53:L6-L10, L14-L20 |
| I53-A002 | A/CFG | JSON path syntax, depth/byte caps, encoding, error shape, and whether limits are configurable are unspecified. | I-53:L20-L21, L51-L53, L55-L62 |
| I53-A003 | A/SEC | Archive size/entry caps, decompression-bomb protection, symlink/path handling, encrypted entries, and malformed-archive recovery are not defined despite read-only operation. | I-53:L19-L21, L51-L62 |
| I53-A004 | A/DEP | Which archive crates already exist and exactly what “hand-roll tar listing” supports are execution-time dependency-tree discoveries. | I-53:L25-L27 |

Dimension note: the command path inherits existing permissions and journaling; archive read-only and JSON caps reduce side effects/context exposure. No retries, concurrency, privacy redaction, or extraction recovery are specified.

## Cross-source interactions and lifecycle

| Atomic ID | Class | Interaction or composition behavior | Source atoms |
|---|---|---|---|
| IX-001 | E | I-16 evaluates the already-shipped local inference levers without altering their serving defaults; only a later live measured Adopt row may change configuration. | I16-C001, I16-E003–I16-E008, I16-E021–I16-E024 |
| IX-002 | E | I-17 is the receipt-producing center of this chunk: I-24 injects into its L2 path, I-26 uses its L2 to rank a collapse frontier, and I-30 consumes its pass/veto evidence. | I17-E003–I17-E009; I24-E015, I24-E020; I26-E005; I30-E004, I30-E007, I30-E030 |
| IX-003 | I | I-19’s pre-write predicate composes F-1, which I-17 later consumes as an L1 axis; therefore a source edit rejected by I-19 should never become an I-17 candidate, while an allowed failing-test edit may still be hard-vetoed later. | I19-E002, I19-E016, I19-E023–I19-E025; I17-E003, I17-E016 |
| IX-004 | E | I-24 neutralizes only L2 scorer weight and can never weaken I-17’s L1 supremacy; malformed canaries are caught by L1 before any scorer-bias conclusion. | I24-E006–I24-E009, I24-E022; I17-E004 |
| IX-005 | I | Frozen canaries (I-24) and the off-loop frozen checkpoint evaluator (I-30) are complementary anti-self-evaluation controls at scorer and training-promotion layers. | I24-E010, I24-E021; I30-E013 |
| IX-006 | E | I-26 collapse stops exploration first, then invokes I-17 L2 on only the current frontier and forbids any later expansion row. | I26-E004–I26-E006, I26-E023; I17-E008 |
| IX-007 | E | I-26 owns budget control but emits/feeds I-27 convergence evidence rather than adding a new event family; adaptive tuning waits for I-27 data. | I26-E005, I26-E009, I26-E015, I26-A002 |
| IX-008 | E | I-30 cannot operate until I-4 yields sealed search trajectories, and its positive/negative extraction presumes I-17 L1/L2 verdict evidence. | I30-E001, I30-E003–I30-E007, I30-E016 |
| IX-009 | C | I-30’s `TriLayerVerdict(Promoted)` selector is not representable by I-17’s enumerated four-kind schema without a schema or selector correction. | I30-C002, I17-E015 |
| IX-010 | C | I-30’s negative-source lifecycle has two incompatible branches: early fallback re-derivation if I-28 is absent versus the later direct-only sealed I-28 call. | I30-E017, I30-E043, I30-C001 |
| IX-011 | E | I-30 is SFT-only and produces the initialization/data substrate; I-31 GRPO remains later/last and cannot be smuggled into this unit. | I30-E002, I30-E009–I30-E010, I30-E031, I30-E046 |
| IX-012 | E | Both I-16 and I-30 keep any MoE retarget as an independent RunPod measurement and forbid assuming a model swap. | I16-E005, I16-E013, I16-E022; I30-E011, I30-E023 |
| IX-013 | E | I-43 runs cheap I-38/G-6 instrumentation first and can kill/rescope the expensive trust build or its claim without replacing the product DONE spine. | I43-E002–I43-E006, I43-E013, I43-E026 |
| IX-014 | E | I-43 produces the cloud/open-weight wording contract consumed by I-40: cloud trace authentication is allowed, deterministic replay is not. | I43-E008–I43-E010, I43-E030 |
| IX-015 | E | I-47 hardens every worker artifact/tool result before an auditor or critic sees it, preserving I-17/D-13 evaluation authority from embedded instructions and self-attestation. | I47-E002–I47-E003, I47-E014–I47-E018; I17-E017 |
| IX-016 | I | I-53’s new tool outputs become untrusted tool-result segments under I-47 on later auditor dispatch, so adding utilities does not enlarge the auditor’s trusted-data plane. | I53-E016–I53-E020; I47-E002–I47-E003 |
| IX-017 | I | I-49 adds cloud backends that will eventually need I-26 provider-budget compatibility and I-51 capability metadata; neither source defines the cross-backend reasoning-effort mapping. | I49-E018, I49-E029, I49-A004; I26-E007, I26-A003 |
| IX-018 | E | I-49 keeps credential storage outside scope and composes future I-50 auth plus I-51 catalog rather than absorbing either concern. | I49-E011, I49-E018, I49-E029, I49-A003 |
| IX-019 | E | I-53 recipe commands must pass E-30/bash validation and are only a future I-46 redirect target; neither feature may bypass the existing command gate. | I53-E014, I53-E017, I53-E021, I53-E023 |
| IX-020 | E | Seven sources add action families (`KvCacheTuned`, `TriLayerVerdict`, `TddPreflightBlocked`, `PrmCanaryEvent`, `SftExhaustEvent`, `ClaimScopeMeasured`, `PromptHardeningApplied`); I-26, I-49, and I-53 explicitly reuse existing rows. | I16-E011; I17-E015; I19-E014; I24-E016; I30-E026; I43-E017; I47-E020; I26-E009; I49-E023; I53-E020 |
| IX-021 | I | Those seven baseline-relative registry additions are a schema/concurrency hotspot; each executor must reread and serialize against live action locks rather than independently applying dated `+1` assumptions. | IX-020; I16-E015, I17-E012, I24-E012, I43-E017, I47-E020 |
| IX-022 | E | New integrity keys use blake3: I-16 config hash, I-17 quarantine key, I-30 RRA provenance, and I-49 fixture-content hash; multiple plans expressly ban blake2/sha2. | I16-E023, I16-E030; I17-E006, I17-E022; I30-E006, I30-E032; I49-E026 |
| IX-023 | E | Live/GPU/network or new-authority work remains isolated behind checkboxes/approval: I-16 live eval, I-24 new languages, I-30 LoRA/MoE/deps, I-43 live lanes, I-49 live calls/deps, and I-53 archive deps. | I16-E013; I24-E014; I30-E023; I43-E019; I49-E010, I49-E015; I53-E010 |
| IX-024 | E | UI posture is additive/minimal: I-17/I-19/I-24/I-26/I-49/I-53 reuse existing surfaces, I-16/I-47 justify none, I-30 defers its drawer, and I-43 makes its tile optional. | I16-E016; I17-E023; I19-E017; I24-E017; I26-E021; I30-E021, I30-E033; I43-E022; I47-E012; I49-E016; I53-E015 |
| IX-025 | E | Frozen/off-agent boundaries occur at canary authorship (I-24), budget-controller authority (I-26), checkpoint evaluation/training artifacts (I-30), and role-rule injection (I-47). | I24-E010, I24-E021; I26-E011; I30-E013; I47-E005 |
| IX-026 | E | Every source except I-47 explicitly bans production unwrap/expect or equivalent unsafe failure; typed fail-closed behavior is load-bearing in I-16, I-17, I-19, I-24, I-26, I-30, I-43, I-49, and I-53. | I16-E023; I17-E022; I19-E022; I24-E023; I26-E020; I30-E032; I43-E026; I49-E021–I49-E022; I53-E019 |
| IX-027 | E | Deterministic offline fixtures are the common delivery strategy for gated live systems: I-16 no-GPU matrix, I-17 seeded flake/veto, I-19 staged edit, I-24 seeded canary, I-26 collapse, I-30 sealed RRA, I-43 canned traces, I-49 recorded protocol frames, and I-53 repository/archive/JSON fixtures. | I16-E024; I17-E024–I17-E026; I19-E023–I19-E026; I24-E024–I24-E028; I26-E022–I26-E023; I30-E040–I30-E049; I43-E018, I43-E027–I43-E028; I49-E020, I49-E024; I53-E021–I53-E022 |
| IX-028 | C | “No paper constant as product threshold” recurs, but I-30 makes a cited external five-band table binding and I-43 fixes 5%; both require explicit reconciliation rather than silent promotion. | I16-E008; I24-E007; I26-E020; I30-C003; I43-C001 |

Lifecycle summary: shipped substrates/configuration → deterministic Tier-1 fixture and event evidence → fail-closed acceptance/negative arms → separate live/RunPod/network or promotion checkbox → measured Adopt/GO/promotion only through the named ledger. No source authorizes fixture success to masquerade as live evidence.

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or correction | Atom references |
|---|---|---|---|
| AC-001 | correction/status | I-16 is evaluation over shipped KV levers, not new pruning/quant flags or default changes. | I16-C001, I16-E002–I16-E005 |
| AC-002 | ambiguity/measurement | I-16 leaves adoption band/VRAM threshold and matrix-to-single-artifact aggregation unspecified. | I16-A001–I16-A002 |
| AC-003 | ambiguity/recovery | I-17 omits flake rerun count, persistence, rehabilitation, and L3 variance threshold. | I17-A001–I17-A002 |
| AC-004 | ambiguity/integration | I-17’s live hook locations, temporary calibration stub lifecycle, and `HaltingBound` schema remain unresolved. | I17-A003–I17-A004 |
| AC-005 | correction/scope | I-19 Tier 1 uses symbol/line-name heuristics; full line-level coverage instrumentation is deferred. | I19-C001 |
| AC-006 | ambiguity/language | I-19 names TypeScript scope without a TS dry-run command and leaves attribution/name matching undefined. | I19-A002–I19-A003 |
| AC-007 | ambiguity/event | `RefactorExempted` is non-blocking but lives under `TddPreflightBlocked`. | I19-A004 |
| AC-008 | ambiguity/config | I-24 omits `p`, `N`, threshold calibration/version, reset, and reverse-direction score-delta behavior. | I24-A001–I24-A003 |
| AC-009 | conflict/threshold | I-26’s 85% boundary is variously `>`, `>=`, and “at.” | I26-C001 |
| AC-010 | ambiguity/schema | I-26’s budget constants, schema migration, collapse event owner, and provider-version mapping are open. | I26-A001–I26-A003 |
| AC-011 | conflict/dependency | I-30 both permits and forbids re-deriving negatives when I-28 is unavailable. | I30-C001 |
| AC-012 | conflict/schema | I-30 selects `TriLayerVerdict(Promoted)`, which I-17 does not define. | I30-C002, IX-009 |
| AC-013 | conflict/threshold | I-30 treats cited band numbers as non-target rationale and binding Tier-1 floors simultaneously. | I30-C003 |
| AC-014 | ambiguity/eligibility | I-30’s verified-COMPLETE winner model is not reconciled with hard-band qualifying partial trajectories. | I30-A002 |
| AC-015 | ambiguity/gate | I-30 alternates among block, warning, and error for missing ID-6 evidence; JSONL/event schemas and calibration remain incomplete. | I30-A001, I30-A003–I30-A005 |
| AC-016 | conflict/threshold | I-43’s no-paper-threshold rule coexists with a hard 5% wording cutoff. | I43-C001 |
| AC-017 | ambiguity/kill | I-43 does not quantify rough/flat/block-everything criteria, reconcile N=50 with 50–100, or define how off-tool F0.3 completes the memo. | I43-A001–I43-A003 |
| AC-018 | ambiguity/artifact | I-43 leaves bench artifact and divergence/wording schemas undefined. | I43-A004–I43-A005 |
| AC-019 | conflict/security | I-47 says label/preserve self-attestation while its event says `SelfAttestationStripped`; tag/table bytes and enablement semantics are also open. | I47-C001, I47-A001–I47-A003 |
| AC-020 | ambiguity/protocol | I-49 leaves shim placement, API/crate choices, credential acquisition, IDs/errors, J-14 compatibility, and fixture-vs-production reach unresolved. | I49-A001–I49-A006 |
| AC-021 | ambiguity/tooling | I-53 leaves recipe parsing/execution semantics, JSON caps, archive bomb/path handling, and dependency fallback scope unresolved. | I53-A001–I53-A004 |
| AC-022 | conflict/cross-cutting | Seven action families plus reused-row payload changes require serialized live-schema reconciliation; dated baseline arithmetic is not independently safe. | IX-020–IX-021 |
| AC-023 | conflict/cross-cutting | External numeric thresholds in I-30/I-43 conflict with the chunk-wide local-measurement doctrine. | IX-028 |

## Orientation-summary additions and corrections

Comparison target: `docs/thefinalglance/feature-discovery/09-I-SUMMARY_INDEX.discovery.md`. These are deltas against its I-16/I-17/I-19/I-24/I-26/I-30/I-43/I-47/I-49/I-53 entries, not duplicate feature atoms.

### I-16 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I16-01 | CORRECTION | Summary L462-L464 describes sink-protected pruning and pinning the first four tokens; the ratified source instead evaluates the shipped sink control across `sinks={0,4}` and does not mandate four sinks as the serving default. | I16-C001, I16-E003, I16-E006 |
| SD-I16-02 | CORRECTION | Summary L472 says I-16 adds sidecar flags. The source expressly bans duplicate pruning/quant flags because the relevant dtype/turboquant/sink levers already ship. | I16-C001, I16-E014, I16-E021 |
| SD-I16-03 | ADDITION | The exact 18-arm matrix, real-config-path dry run, metrics filename, null VRAM/throughput policy, `DryRun` decision, and FA2 typed refusal were omitted. | I16-E006–I16-E007, I16-E010, I16-E023–I16-E024 |
| SD-I16-04 | ADDITION | Exact CLI/module symbols, subprocess fallback, no-dashboard decision, schema auto-derivation, action-lock checks, and clean-tree cardinality gates were omitted. | I16-E012, I16-E015–I16-E020, I16-E026–I16-E030 |
| SD-I16-05 | ADDITION | True eviction is blocked on `position_ids` re-indexing; HTTP transport, live Adopt/Reject, and MoE remain distinct Tier-2 work. | I16-E004–I16-E005, I16-E013, I16-E026 |
| SD-I16-06 | UNSUPPORTED | Summary L468 requires comparison with an E-27 AWQ baseline; the deep source composes E-27 serving/parity patterns but contains no explicit AWQ-baseline comparison acceptance requirement. | I16-E009 |
| SD-I16-07 | ADDITION | The source exposes an unresolved 18-arm-versus-one-row/file aggregation contract beyond the summary’s already-noted missing pass-rate band. | I16-A001–I16-A002 |

### I-17 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I17-01 | ADDITION | The 15-second timer must be shared with I-8 and charges `HaltingBound`; the summary records abort only, not timer ownership or penalty. | I17-E005, I17-A004 |
| SD-I17-02 | ADDITION | Quarantine is SSB-backed and blake3-keyed by oracle plus seed range, with a mandatory check before every vote; rerun/rehabilitation parameters remain open. | I17-E006–I17-E007, I17-A001 |
| SD-I17-03 | QUALIFICATION | The source’s pre-I-23 fallback is a registered direct conformal-calibration call, while E-19 is the PRM-lite scorer; summary L496 compresses these two fallback roles. | I17-E008, I17-A003 |
| SD-I17-04 | ADDITION | `DisagreementEscalated`, exact production callers, no-new-drawer IPC behavior, no-soft-veto/no-copy bans, and mandatory full-form expansion were omitted. | I17-E009, I17-E015–I17-E023, I17-E027 |
| SD-I17-05 | ADDITION | J-11 adds source-cited Arbor veto prior art and held-out `B_dev`/`B_test` discipline, absent from the summary. | I17-E028–I17-E029 |

### I-19 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I19-01 | CORRECTION | Summary L558 says the exemption is determined through coverage delta; the source narrows Tier 1 to no-new-public-symbol plus line/name heuristic and defers full line-level coverage instrumentation. | I19-E008, I19-E015, I19-C001 |
| SD-I19-02 | ADDITION | Default-off `ELAI_TDD_PREFLIGHT_ENABLE` allowlist registration, experiment/expiry/promotion fields, and separate patch-N promotion were omitted. | I19-E009–I19-E010, I19-E015 |
| SD-I19-03 | ADDITION | TypeScript parse/reach scope, substring/attribution matching, `SyntaxDryRunFailed`, and the staged-content compile-only/no-test constraint were omitted. | I19-E003–I19-E007, I19-E014, I19-A002–I19-A003 |
| SD-I19-04 | ADDITION | Exact predicate symbol, FP TOML, action-lock/cross-language/UI reuse, positive failing-test arm, and skip-preflight broken arm were omitted. | I19-E016–I19-E027 |
| SD-I19-05 | AMBIGUITY | The deep source reveals the non-blocking `RefactorExempted` discriminator under a “Blocked” action family; the summary reports it without flagging this consumer-contract ambiguity. | I19-A004 |

### I-24 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I24-01 | ADDITION | `ConstantExtraction`, seeded probability `p`, uniform deterministic sampling, and L1 semantic-equivalence validation were omitted. | I24-E003–I24-E006 |
| SD-I24-02 | ADDITION | A single detection is only a warning; neutralization requires `N` consecutive detections, with `p`, `N`, and threshold left unbound. | I24-E007–I24-E008, I24-A001 |
| SD-I24-03 | CORRECTION | Summary L678 says neutralization lasts until recalibration; the source says weight zero for the current run and does not define post-run/recalibration persistence. | I24-E008, I24-A003 |
| SD-I24-04 | ADDITION | Exact frozen fixture/manifest/checker artifacts, three event kinds, production L2 caller, and dynamic-canary freeze-check broken arm were omitted. | I24-E010, I24-E015–I24-E020, I24-E024–I24-E028 |
| SD-I24-05 | ADDITION | Tier 1 covers Rust; new-language adapters and multi-scorer correlation are separately deferred. | I24-E013–I24-E014 |
| SD-I24-06 | AMBIGUITY | Signed rather than absolute score delta and the calibration/version lifecycle are unresolved in the source but not surfaced by the summary. | I24-A002–I24-A003 |

### I-26 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I26-01 | ADDITION | The exact exponential `base*depth_factor^depth` formula, all-active-sibling spend aggregation, I-17 frontier call, and I-27 event handoff were omitted. | I26-E003–I26-E006 |
| SD-I26-02 | CONFLICT | Summary L719 says “at 85 percent”; the source itself alternates `>85`, `>=85`, and “at,” so the boundary is not settled. | I26-C001 |
| SD-I26-03 | ADDITION | Generator inaccessibility, cache-prefix invariance, exact module/decision doc, production path, conditional row migration, and no-new-action decision were omitted. | I26-E009–I26-E014, I26-E016–I26-E019 |
| SD-I26-04 | ADDITION | Adaptive depth-factor and extra effort levels are explicitly deferred pending I-27 measured data. | I26-E015 |
| SD-I26-05 | UNSUPPORTED | Summary L730 says a plateau fixture reports measured token savings; the source requires depth/collapse/request evidence but contains no explicit token-savings acceptance output. | I26-E013, I26-E022–I26-E023 |
| SD-I26-06 | ADDITION | Unspecified budget constants, collapse-event ownership/schema migration, and provider-version/API discovery were not recorded as open issues. | I26-A001–I26-A003 |

### I-30 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I30-01 | CORRECTION | Summary L813 reads as an I-30 LoRA training delivery; the source makes the GPU run Tier 2 and ships only offline extraction/gate linting now. | I30-E010, I30-E023–I30-E024 |
| SD-I30-02 | CORRECTION | Summary L819 says I-30 adds a dataset card; the source explicitly defers the drawer under the owner UI hold and forbids Tier-1 UI. | I30-E021, I30-E033 |
| SD-I30-03 | ADDITION | Exact JSONL content, blake3-over-RRA bytes, three event kinds, equal record/provenance counts, and unsealed/tampered failure evidence were omitted. | I30-E005–I30-E008, I30-E026–I30-E030 |
| SD-I30-04 | ADDITION | Direct sealed `DpoPairExtractor` invocation and prohibition on current live rows were omitted. | I30-E017, I30-E043 |
| SD-I30-05 | CONFLICT | The source first allows re-derivation without I-28, then later forbids it; the summary gives neither fork nor conflict. | I30-C001 |
| SD-I30-06 | CONFLICT | `TriLayerVerdict(Promoted)` is used by I-30 but absent from I-17’s declared schema; the summary does not flag the cross-plan mismatch. | I30-C002, IX-009 |
| SD-I30-07 | ADDITION | J-11’s five difficulty bands, no-global-gate rule, hard-partial preference, I-25/calibration source, and structural features are entirely absent. | I30-E035–I30-E041 |
| SD-I30-08 | CONFLICT | The amendment’s exact external floors are both “not an ELAI target” and binding/preserved Tier-1 behavior; this conflicts with local-threshold doctrine. | I30-C003, IX-028 |
| SD-I30-09 | AMBIGUITY | Verified-COMPLETE winners versus qualifying partials, gate block/warn/error semantics, JSONL/event schema, and `d_i` calibration remain unresolved. | I30-A001–I30-A005 |
| SD-I30-10 | ADDITION | Full fork/abort/recovery/red-team/verification gates, sibling `dataset_cmd.rs` placement, split SSB warning, and mandatory full-form expansion were omitted. | I30-E001, I30-E034, I30-E042–I30-E050 |

### I-43 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I43-01 | ADDITION | Git-stripped tasks, explicit flat/negative kill, and the reduced-N-as-private-memo constraint were omitted. | I43-E003–I43-E006 |
| SD-I43-02 | ADDITION | Exact two-mode CLI, event arms, mock-runner/canned-RRA real-path fixture, and no-test-only R7 rule were omitted. | I43-E012, I43-E015, I43-E017–I43-E018, I43-E025 |
| SD-I43-03 | ADDITION | The memo requires three greens, while F0.3 is off-tool business validation; this completion dependency was omitted. | I43-E016, I43-E020, I43-A003 |
| SD-I43-04 | ADDITION | I-40’s downstream wording dependency, no-external-claim boundary, full ban list, and deterministic trap/wording negative arms were omitted. | I43-E006, I43-E026–I43-E030 |
| SD-I43-05 | CONFLICT | Summary L1187 and L1195-L1196 reproduce no-paper-threshold and fixed-5% claims without identifying their tension. | I43-C001 |
| SD-I43-06 | AMBIGUITY | Default N, statistical/trap criteria, unnamed bench TOML, and divergence/claim schemas remain open beyond the summary’s Track-19 kill-gate qualification. | I43-A001–I43-A005 |

### I-47 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I47-01 | ADDITION | The injected rule set also includes “no future-proofing,” omitted from summary L1290-L1293. | I47-E004 |
| SD-I47-02 | ADDITION | SL-11’s mandatory original competitor read, existing-prose activation status, and deferred injection-resistance corpus evaluation were omitted. | I47-E006, I47-E008–I47-E011 |
| SD-I47-03 | ADDITION | Wrapping/rules are unconditional by role; no skip flag, model-authored tags, classification copy, or artifact truncation is allowed. | I47-E015–I47-E017 |
| SD-I47-04 | CONFLICT | The summary mentions labeled self-attestation but omits the source’s `SelfAttestationStripped` kind, which conflicts with the preserve/wrap rule. | I47-C001 |
| SD-I47-05 | ADDITION | No-UI justification, exact malicious fixture bytes, structurally absent no-tags flag, and missing-row failure were omitted. | I47-E012, I47-E018–I47-E019 |
| SD-I47-06 | AMBIGUITY | Tag/frame canonical bytes, frozen-table schema, actual prompt modules, and “feature ON” versus unconditional/no-skip activation are unresolved. | I47-A001–I47-A003 |

### I-49 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I49-01 | ADDITION | All six mandatory TypeScript source files and the full registry/repository/publish/license dependency checks were omitted. | I49-E006, I49-E009–I49-E010 |
| SD-I49-02 | ADDITION | Hand-roll-if-heavier policy; GitLab/Kimi/Vertex, storage, and adapter-removal exclusions; and the existing adapter/keep-list inventory were omitted. | I49-E002, I49-E010–I49-E013 |
| SD-I49-03 | ADDITION | No silent backend fallback, real rather than mock canonical signing, user-approved deps, and typed no-panic failure are stronger than the summary’s corrupt-frame sentence. | I49-E020–I49-E022 |
| SD-I49-04 | ADDITION | J-15 makes J-14 repro packs and `(model_id, blake3(content), tool_name)` matching binding; positional fixture matching is an explicit do-not-copy pattern. | I49-E025–I49-E027 |
| SD-I49-05 | ADDITION | Fixture auth headers are prohibited and inherited J-14 scrubbing is a privacy/credential acceptance rule. | I49-E028 |
| SD-I49-06 | AMBIGUITY | Shim placement, AWS/Azure API/dependency choice, credential supply, backend/error schemas, J-14 compatibility, and production-vs-fixture reach remain open. | I49-A001–I49-A006 |
| SD-I49-07 | QUALIFICATION | Summary L1343-L1344 implies all three register as adapters; the source describes two adapter modules plus a shared normalization module whose `genai_adapter` ownership is conditional. | I49-E007, I49-E017–I49-E018, I49-A001 |

### I-53 deltas

| Delta ID | Type | Addition or correction to the orientation summary | Source atoms |
|---|---|---|---|
| SD-I53-01 | ADDITION | Recipe descriptions and owner-sanctioned-entrypoint rationale were omitted; taskfile/npm support is explicitly deferred despite taskfile appearing in the premise. | I53-E002–I53-E004, I53-E013 |
| SD-I53-02 | ADDITION | Exact Rust modules/symbols, catalog three-arm wiring, H-4 discoverability, and future I-46 redirect relationship were omitted. | I53-E008, I53-E014, I53-E016–I53-E018, I53-E023 |
| SD-I53-03 | ADDITION | Archive scope is zip/tar.gz streamed read-only; the JSON fixture is 5 MB and must stay capped with a truncation marker. | I53-E005–I53-E006, I53-E022 |
| SD-I53-04 | QUALIFICATION | Cron/monitor/background jobs are declined permanently, not deferred backlog. | I53-E009 |
| SD-I53-05 | ADDITION | The dependency fallback may hand-roll tar listing, and production unwrap/expect plus archive extraction are explicitly banned. | I53-E010, I53-E019 |
| SD-I53-06 | AMBIGUITY | Recipe syntax/arguments, JSON cap/path contract, archive bomb/path protections, and actual dependency inventory remain unspecified and were not surfaced by the summary. | I53-A001–I53-A004 |

## Zero-omission, duplicate, and exact-ID audit

| Processed original | Lines read | Primary atoms | Complete section/line coverage |
|---|---:|---:|---|
| `I-16_kv_cache_pruning_quant.md` | 126/126 | 33 | title/metadata/premise L1-L12; build L14-L20; ground truth L22-L28; tiering L30-L40; integration L42-L47; wiring/observable L49-L77; grounding/composition L79-L91; wargame/done L93-L124; terminator L126 |
| `I-17_tri_layer_evaluation_stack.md` | 116/116 | 33 | title/premise L1-L7; build/non-goals L9-L22; ground truth L24-L32; tiering L34-L44; integration L46-L52; wiring/observable L54-L83; grounding/composition/full-form L85-L102; amendment L104-L114; terminator L116 |
| `I-19_tdd_enforcer_preflight.md` | 104/104 | 32 | title/premise L1-L8; build/non-goals L10-L29; ground truth L31-L38; tiering L40-L51; integration L53-L59; wiring/observable L61-L87; grounding/composition L89-L102; terminator L104 |
| `I-24_isomorphic_canary_antigaming.md` | 122/122 | 31 | title/premise L1-L9; build/security/non-goals L11-L35; ground truth L37-L45; tiering L47-L58; integration L60-L67; wiring/observable L69-L101; grounding/composition L103-L120; terminator L122 |
| `I-26_dynamic_thinking_budgets.md` | 98/98 | 29 | title/premise L1-L9; build/non-goals L11-L25; ground truth L27-L33; tiering L35-L46; integration L48-L53; wiring/observable L55-L82; grounding/composition L84-L96; terminator L98 |
| `I-30_verifier_ranked_sft_exhaust.md` | 200/200 | 58 | title/premise/full-form L1-L10; build/non-goals L12-L31; ground truth L33-L42; tiering L44-L57; integration L59-L64; wiring/observable L66-L95; grounding/composition L97-L112; amendment L114-L144; wargame/forks/abort/verify/redteam/smoke/recon L146-L198; terminator L200 |
| `I-43_claim_scoper_kill_gate.md` | 132/132 | 36 | title/premise L1-L12; build/non-goals L14-L29; ground truth L31-L41; tiering L43-L59; integration L61-L71; wiring/observable L73-L110; grounding/composition L112-L130; terminator L132 |
| `I-47_verifier_input_hardening.md` | 80/80 | 25 | title/metadata/premise L1-L13; build/non-goals L15-L26; ground truth L28-L34; tiering L36-L39; integration L41-L44; wiring/observable L46-L65; grounding/composition L67-L78; terminator L80 |
| `I-49_bedrock_azure_direct_lanes.md` | 112/112 | 35 | title/metadata/premise L1-L13; build/non-goals L15-L29; ground truth L31-L35; tiering L37-L42; integration L44-L47; wiring/observable L49-L67; grounding/composition L69-L77; amendment L79-L110; terminator L112 |
| `I-53_recipe_runner_tool_pack.md` | 72/72 | 27 | title/metadata/premise L1-L12; build/non-goals L14-L27; ground truth L29-L32; tiering L34-L36; integration L38-L42; wiring/observable L44-L62; grounding/composition L64-L70; terminator L72 |
| **Total** | **1,162/1,162** | **339** | **every source section accounted** |

Audit results:

- Manifest membership: **10/10 exact**; other-original reads **0**; orientation artifacts **1** (`09-I-SUMMARY_INDEX.discovery.md`) used only for delta comparison.
- Integrity: **10/10 SHA-256**, **10/10 logical-line counts**, **10/10 byte counts**, aggregate **1,162 lines / 95,462 bytes** exact.
- Primary feature atoms: **339** = **292 explicit**, **0 inferred**, **39 ambiguous**, **8 conflicting/corrective**.
- Per-file counts: I-16 **33**; I-17 **33**; I-19 **32**; I-24 **31**; I-26 **29**; I-30 **58**; I-43 **36**; I-47 **25**; I-49 **35**; I-53 **27**. Sum **339**.
- Cross-source interaction atoms: **28** = **20 explicit**, **5 inferred**, **3 conflicting**. Total primary-plus-interaction feature/behavior atoms: **367** = **312 explicit**, **5 inferred**, **39 ambiguous**, **11 conflicting/corrective**.
- Analytical cross-reference records: **23** ambiguity/conflict-register IDs and **64** summary-delta IDs. Total unique IDs across primary, interaction, register, and delta namespaces: **454**.
- ID audit: duplicate IDs **0**; blank IDs **0**; sequence gaps **0**. Every `I16/I17/I19/I24/I26/I30/I43/I47/I49/I53-[EAC]`, `IX`, `AC`, and per-file `SD` namespace is contiguous from `001`/`01` through its stated maximum. Per-file inferred count is zero by design; all five inferences are isolated in `IX`.
- Semantic duplicate audit: repeated source wording was consolidated with multiple anchors; interaction, conflict, and delta records reference source atoms rather than recasting them as new features. Duplicate feature atoms **0**.
- Omission audit: every file was checked for capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Dimension notes name absent/unspecified areas rather than inventing behavior.
- Non-feature lines: headings, blanks, separators, and terminators were read and coverage-accounted but correctly receive no artificial feature atom.
- Output metrics after finalization: `output_lines=0673`; `output_bytes=100405`.

## Output digest

Canonical normalized output SHA-256: `7dea6405d615f884aa609c4add3e64b4445256a54a2f3dcad40b7eafa3eaaad9`.

Digest rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters in the preceding digest field with 64 ASCII zeroes. The byte-exact final-file SHA-256 (which necessarily differs because the digest is embedded) is reported in the completion handoff.
