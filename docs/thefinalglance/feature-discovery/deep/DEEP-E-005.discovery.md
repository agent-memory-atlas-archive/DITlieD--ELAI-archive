# DEEP-E-005 feature discovery

## Scope, evidence, and notation

This is the deep pass for manifest chunk **DEEP-E-005**. The only original-plan sources read were the nine exact files in the manifest block reproduced below. `05-E-SUMMARY_INDEX.discovery.md` was used only for orientation and for the summary-delta audit; it is not evidence for a feature unless an assigned original independently states it. No companion report, implementation file, cited paper/site, other E-plan original, or other original chunk was read.

Evidence/status notation:

- **[E]** explicit in the assigned source; a source claim of `SHIPPED`, `DONE`, or current code remains an attributed plan claim, not independently reverified implementation.
- **[I]** inference joining explicit atoms; never promoted to source fact.
- **[A]** ambiguous, conditional, unmeasured, or underspecified.
- **[C]** internally conflicting, cross-source conflicting, or conflicting with the orientation summary.
- Categories: **CAP** capability/action/surface; **WF** workflow/state/lifecycle; **DATA** schema/data; **CFG** flags/defaults/config; **INT** integrations/events; **DEP** dependencies; **SEC** permissions/security/privacy/compliance; **FAIL** failure/fallback/retry/recovery; **DET** determinism/idempotency/concurrency; **OBS** observability/evidence; **TEST** tests/acceptance/done gates; **CON** constraints/non-goals; **STATUS** implementation/deferred state.

An atomic unit is one uniquely keyed, source-local contract statement. Closely coupled type fields, a decision-table row set, a test-vector set, or a single file inventory may remain one atom to preserve semantics. Repetition across objective, task, wiring, deterministic-check, acceptance, tests, and file-list sections is collapsed into one atom with all relevant anchors. Structural Markdown and citation-only repetition were read but are not separate features.

## Exact processed manifest and fingerprint verification

Manifest location: `DEEP-SOURCE-MANIFEST.md:578-592`; the next heading, `DEEP-E-006`, begins at line 594.

> ### DEEP-E-005
>
> Family **E**; totals: **9 files**, **2137 logical lines**, **254945 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-E-005 | numbered/index original | 237 | 24994 | `01364088b471d5daf3d96b8a2590dff395c0cd02e921040925ffee775f96101f` | `docs/thefinalglance/E-plan/E-10_critic_of_elai_design.md` | exact match |
| DEEP-E-005 | numbered/index original | 205 | 22178 | `d2455876c798ee5eaff2439e875b06fb6b30f9b99b719d15f1e97c3594d17598` | `docs/thefinalglance/E-plan/E-13_tdd_verification_agents.md` | exact match |
| DEEP-E-005 | numbered/index original | 235 | 28137 | `bb10d0dcec97785b8e56a213577e9d203d6797803b60c2e69d60f616e96a5da3` | `docs/thefinalglance/E-plan/E-15_sandboxed_exec.md` | exact match |
| DEEP-E-005 | numbered/index original | 195 | 19981 | `ddcb9ec59a5a0ceaf6d9f4c81626fca69159f0e091f1b1bd7abc3b1a15fdd79c` | `docs/thefinalglance/E-plan/E-28_attention_kernels_2026.md` | exact match |
| DEEP-E-005 | numbered/index original | 189 | 22018 | `f1c88b75edb3266fca90919dda7f59eee8adadb8619f41251be3ebbd09d1992c` | `docs/thefinalglance/E-plan/E-31_coding_model_leaderboard.md` | exact match |
| DEEP-E-005 | numbered/index original | 257 | 33053 | `5d8a4c238a6fd5573ba29e836ac03a03dbba4ac8fb32a25677c2b7bc0a782b26` | `docs/thefinalglance/E-plan/E-33_dashboard_ux_agentic.md` | exact match |
| DEEP-E-005 | numbered/index original | 226 | 24164 | `59b840f934ad713035e7e25e03c88605378cb6be0ea2efe523d72d2a13a7dd9c` | `docs/thefinalglance/E-plan/E-43_agent_skill_systems.md` | exact match |
| DEEP-E-005 | numbered/index original | 316 | 44365 | `71f79781566132e202189d48b9af9882a30aadd7dad5a8e4d38355fc57a91d46` | `docs/thefinalglance/E-plan/E-50_saas_architecture_patterns.md` | exact match |
| DEEP-E-005 | numbered/index original | 277 | 36055 | `5d6dff6f8867458dc8db3f9333c0b9a8eebaa44c14c902337d364268033af45f` | `docs/thefinalglance/E-plan/E-52_lsp_integration.md` | exact match |
| **Verified total** | **9 files** | **2137** | **254945** | — | — | **exact match** |

Verification used `sha256sum` and `wc -lc` on the exact absolute paths. There were zero missing files and zero per-file or aggregate hash, logical-line, or byte mismatches.

## Per-file explicit ledgers

### E-10_critic_of_elai_design.md — 237/237 lines

- **E005-E10-001 [E][CAP/CON]** E-10 is a measured-then-decided simplification program: compare full versus reduced FSM, roles, and RAG; structurally harden MistakeBook; trial typed-CoT certificates; and measure VRAM-tier collapse, while expressly refusing blind simplification. (`E-10_critic_of_elai_design.md:1-3,10,22`)
- **E005-E10-002 [E][DEP/STATUS]** It requires E-6's iso-budget harness, E-9 paired-bootstrap/ClawsBench attribution, B-37 multi-backend routing, and B-41 per-level auditing; it is declared parallel-safe with E-6 through E-9. (`E-10_critic_of_elai_design.md:4-6`)
- **E005-E10-003 [E][OBS/CON]** The motivating measurements include vacuous FSM gates, eight-role MAST tax, unfaithful GRC blockquotes, a seven-layer RAG regression with four runtime-dead layers, a 25–30-point local/frontier coding gap, and raw MistakeBook ICL-poisoning exposure; E-10 treats these as hypotheses except where the poisoning evidence warrants an immediate defense. (`E-10_critic_of_elai_design.md:8-10,237`)
- **E005-E10-004 [E][INT/STATUS]** E-10 extends E-6, may contribute Typed-CoT to E-7's verifier consensus, exercises B-37 cloud routing, adds a parallel challenger to B-31's partly shipped seven-layer knowledge pipeline, replaces the MistakeBook feed, and adds a tier decision sheet. (`E-10_critic_of_elai_design.md:12-20`)
- **E005-E10-005 [E][CAP/WF/DATA]** `FsmProfile::{Full9State, Reduced3State}` maps `Planning <- {PromptTransform, ScopePlanning, ContextCompiling}`, `Executing <- {Executing, Adversary, BugReporting}`, and `Verifying <- {Auditing, MemoryWriting, Completing}`. (`E-10_critic_of_elai_design.md:48`)
- **E005-E10-006 [E][DET/OBS]** `run_fsm_ablation` must use the same fixture, model, and token budget, compare pass rate and per-MAST-mode counts, apply paired bootstrap, and emit `FsmAblationCompleted`. (`E-10_critic_of_elai_design.md:48,74-75,83,107-110,159`)
- **E005-E10-007 [E][CAP/WF/DATA]** `RoleProfile::{Full8Role, Reduced3Role}` maps `Worker <- {Executor, Researcher, Designer, MediaValidator}`, `Verifier <- {Auditor + cross-family Adversary inline}`, and `Planner <- {PromptEngineer + ContextCompiler}`. (`E-10_critic_of_elai_design.md:50`)
- **E005-E10-008 [E][DET/OBS]** Reduced-role evaluation uses Self-MoA-style `N=3` samples from one strong model for Researcher/Designer work, compares equal-budget fixtures against all eight roles, and emits `RoleAblationCompleted`. (`E-10_critic_of_elai_design.md:50,76,84,132-134,160`)
- **E005-E10-009 [E][CAP/DATA]** `ThreeLayerRag` is `BM25 -> existing dense vectors -> ms-marco cross-encoder top50->top5`; it runs beside, not instead of, the seven-layer pipeline on the same query stream and fixed eval set. (`E-10_critic_of_elai_design.md:52,77,96,136-138,161`)
- **E005-E10-010 [E][CFG/OBS/FAIL]** The reranker is install-time-staged under canonical `.elai_cc/models/onnx/ms-marco-rerank/`, both paths report MRR through `RagChallengerMrrMeasured`, and a missing model must be caught by checks because the described runtime otherwise silently degrades. (`E-10_critic_of_elai_design.md:52,85,196,224`)
- **E005-E10-011 [E][DATA/SEC]** `StructuredFailureSignal` contains `failure_category`, `affected_files: Vec<PathBuf>`, `structured_diff: DiffStat {lines_added, lines_removed, files_changed, churn_score}`, optional `MastMode`, `timestamp_ms`, and a 32-byte blake3 `content_hash`; raw prior-attempt code is never stored. (`E-10_critic_of_elai_design.md:18,54,78,102,115-121,162`)
- **E005-E10-012 [E][OBS/TEST]** Storing the structured signal emits `MistakeBookSignalStored`; a cross-context-pack contract must prove that no raw prior-attempt text reaches executor context. (`E-10_critic_of_elai_design.md:54,86,115-121,162,175`)
- **E005-E10-013 [E][DATA/CAP]** `TypedCotCertificate` parses a claim into `claim: Proposition`, `evidence: Vec<Witness>`, `derivation_step: InferenceRule`, and blake3 `verifier_signature`; witness provenance is `CodeSpan|SpecCite|TestResult|ExternalSource`, and rules are `ModusPonens|AndElim|CaseSplit|Refutation|Citation`. (`E-10_critic_of_elai_design.md:56,79,123-126`)
- **E005-E10-014 [E][CFG/CON]** Typed-CoT is opt-in through `ELAI_TYPED_COT=1`; the GRC blockquote remains the production default until data and explicit user approval justify a separate change. (`E-10_critic_of_elai_design.md:32,56,103-105,163,222-223`)
- **E005-E10-015 [E][OBS]** Successful certificate verification emits `TypedCotVerified`, and the default ablation suite must exercise the opt-in path to prevent code rot. (`E-10_critic_of_elai_design.md:87,123-126,223`)
- **E005-E10-016 [E][CAP/CFG]** `tier_matrix_eval` runs five documented 8/12/16/24/48-GB configurations on GSM8K-Platinum, LiveCodeBench v6, and a small tau2 slice at equal budget against cloud Sonnet, reporting pass rate and cost-normalized score. (`E-10_critic_of_elai_design.md:58,81,140-142,164`)
- **E005-E10-017 [E][CAP/OBS]** The public surface is `elai tier eval --suites gsm8k-platinum,livecodebench-v6,tau2 --budget 100k`; it emits `TierMatrixMeasured`. (`E-10_critic_of_elai_design.md:58,88,99`)
- **E005-E10-018 [E][TEST/OBS]** The cloud-default switch test hard-toggles to cloud Sonnet, runs a representative B-N fixture, and records wall-clock, USD-equivalent cost, pass rate, and prompt-cache hit rate through `CloudDefaultMeasured`. (`E-10_critic_of_elai_design.md:60,89,165,181`)
- **E005-E10-019 [E][DET/CON]** GRC evaluation is the cross-product of blockquote on/off over 30 tasks and measures pass rate, auditor BLOCK count, and MAST mode; even an off-condition win at paired-bootstrap `p<0.05` may only be surfaced, never auto-remove the rule. (`E-10_critic_of_elai_design.md:62,90,148-150,166,182,226`)
- **E005-E10-020 [E][CAP/OBS]** `CriticDecisionPanel.svelte` renders six tiles—FSM, Roles, RAG, MistakeBook, TypedCoT, Tier—with empirical comparisons and paired-bootstrap p-values; it labels `Recommend SIMPLIFY` only when challenger >= incumbent at `p<0.05` and never mutates architecture. (`E-10_critic_of_elai_design.md:64,106,167,183`)
- **E005-E10-021 [E][DATA/WF]** CLAUDE.md's `Critic Decisions` block records each ablation as `NotMeasured|Insufficient|Inconclusive|RecommendSimplify|RecommendKeep` and is updated from SSB by a post-benchmark hook. (`E-10_critic_of_elai_design.md:66,169,201`)
- **E005-E10-022 [E][CAP/FAIL/OBS]** `measure_cache_hit_rate` measures real Anthropic cloud caching, emits `PromptCacheHitRateMeasured`, raises a red flag below 50%, and emits `CacheCheckUnavailable` rather than running when no cloud route is configured. (`E-10_critic_of_elai_design.md:68,80,91,144-146,184,227`)
- **E005-E10-023 [E][CAP/CON]** `elai critic report` is read-only, aggregates all six results/p-values/recommendations, and writes `workflow/reasoning/critic-decisions-{ts}.md`; it performs no automatic mutation. (`E-10_critic_of_elai_design.md:70,82,99,168`)
- **E005-E10-024 [E][INT]** CLI reachability is `elai tier eval`, `elai critic report`, and `elai bench ablation [fsm|role|grc|rag|cot|cache]`; the dashboard reads `get_critic_decision_state`. (`E-10_critic_of_elai_design.md:92-100`)
- **E005-E10-025 [E][CON/SEC]** Stub bans prohibit raw MistakeBook code, production-default reduced FSM, hardcoded Typed-CoT defaulting, any auto-removal of GRC, non-test unwraps, non-blake3 hashes, non-native trait async, and unverified Python dependencies. (`E-10_critic_of_elai_design.md:101-105,152-154,170-171`)
- **E005-E10-026 [E][TEST/OBS]** The primary smoke runs `bench ablation fsm` on `critic_30.toml` at budget 5000 and requires both profile rates, a p-value, an SSB event within 30 minutes, a two-row stdout table, and a refreshed FSM tile. (`E-10_critic_of_elai_design.md:107-110`)
- **E005-E10-027 [E][TEST]** Deterministic certificate checks require a valid chain to pass and a missing witness to fail. (`E-10_critic_of_elai_design.md:123-126`)
- **E005-E10-028 [E][TEST]** FSM and role smoke JSON must expose two non-null profile objects; the role run must verify `N=3` invocation. (`E-10_critic_of_elai_design.md:128-134,177-178`)
- **E005-E10-029 [E][TEST]** RAG checks require two non-null MRR values, the tier run at least five entries, cache rate in `[0,1]`, and GRC on/off rates plus p-value. (`E-10_critic_of_elai_design.md:136-150`)
- **E005-E10-030 [E][TEST]** Acceptance additionally requires all three suites across five tiers, cloud/local comparison, six dashboard tiles, decision artifact and CLAUDE pointer, staged ONNX model, and unchanged GRC default. (`E-10_critic_of_elai_design.md:157-169`)
- **E005-E10-031 [E][TEST]** Named tests cover raw-code scanning, certificate validity/missing/unknown rules, five-task FSM and role smoke, fixed-set RAG MRR, five-tier output, cloud switch metrics, GRC no-mutation, dashboard badge logic, and mocked Anthropic cache calculation. (`E-10_critic_of_elai_design.md:173-184`)
- **E005-E10-032 [E][CAP/STATUS]** Planned files create seven backend/CLI modules, the dashboard panel, and model directory; modify MistakeBook, SSB with nine variants, CLI registration, Tauri command, and CLAUDE.md. (`E-10_critic_of_elai_design.md:186-201`)
- **E005-E10-033 [E][A/CFG]** Per-tier USD equivalents need user-set electricity/token rates; defaults are rough 2026 us-east averages explicitly marked unconfirmed. (`E-10_critic_of_elai_design.md:225`)
- **E005-E10-034 [E][A/DEP]** E-10 assumes E-6 and E-9 land first; landing early requires co-landing their surfaces at roughly 200 LOC overhead. (`E-10_critic_of_elai_design.md:228`)
- **E005-E10-035 [E][STATUS/WF]** Every `RecommendSimplify` becomes a distinct user-approved follow-on rather than part of E-10. (`E-10_critic_of_elai_design.md:231-234`)
- **E005-E10-036 [E][INT/TEST]** Any RAG simplification must compose with E-9 ClawsBench harness-vs-model attribution so the measured lift is not a scaffold artifact. (`E-10_critic_of_elai_design.md:235`)
- **E005-E10-037 [E][INT/STATUS]** A successful Typed-CoT challenger can be proposed as E-7's third verifier only after `p<0.05`; it is not automatically promoted. (`E-10_critic_of_elai_design.md:236`)
- **E005-E10-038 [E][SEC/STATUS]** MistakeBook's structured-only poisoning defense ships immediately without an ablation gate. (`E-10_critic_of_elai_design.md:237`)
- **E005-E10-039 [E][CON]** Model files are downloaded at install time and are not committed. (`E-10_critic_of_elai_design.md:196`)
- **E005-E10-040 [E][STATUS]** The source is a three-session, roughly 12-hour plan; no completion/shipped marker is present. (`E-10_critic_of_elai_design.md:3`)

### E-13_tdd_verification_agents.md — 205/205 lines

- **E005-E13-001 [E][CAP/TEST]** E-13 replaces a single example-test flip with a dual EXECUTING oracle—at least one example fail-to-pass plus at least one property/invariant pass—then adds AUDITING mutation survival and anti-over-mock gates so every promotion includes deterministic non-LLM evidence. (`E-13_tdd_verification_agents.md:1-3,8-10`)
- **E005-E13-002 [E][DEP/STATUS]** It augments C-28/C-29, depends also on C-27/C-31/C-32, extends D-3's verifier aggregate, reuses B-22b and E-11 surfaces, and is parallel-safe with E-11/E-12/E-14/E-15. (`E-13_tdd_verification_agents.md:4-5,12-20`)
- **E005-E13-003 [E][CAP/CFG]** `PropertyGate` defaults to `min_cases:10`, zero panics, and at least one nontrivial predicate; banned examples include literal true and `len >= 0`. (`E-13_tdd_verification_agents.md:40,62,65-66`)
- **E005-E13-004 [E][WF/SEC/OBS]** A PreToolUse hook rejects vacuous proptest predicates, EXECUTING requires property plus example success, and SSB emits `property_gate_pass` or `property_gate_vacuous_rejected`. (`E-13_tdd_verification_agents.md:40,87-90,102-104,118-120,132`)
- **E005-E13-005 [E][CAP/CFG]** `TargetedMutationGate` invokes `cargo mutants --in-place --baseline skip --shard <touched_files>` with STEP/PHASE/CONCEPT budgets of 60/120/300 seconds; any missed mutant in a touched function is a blocking finding. (`E-13_tdd_verification_agents.md:42,67,122-124,133`)
- **E005-E13-006 [E][DATA/DET]** Survivor caching uses `~/.cache/elai/mutation_survivors.sqlite3`, key `(file_path, blake3(file_content), function_name)`, TTL 30 days, and cache-hit short-circuit; it is a second DB compatible with C-28 schema discipline. (`E-13_tdd_verification_agents.md:42,133`)
- **E005-E13-007 [E][CAP/SEC]** `AntiMockHook::on_post_tool_use(file,diff)` counts additions of `mock.patch|vi.mock|mockall::|MockAll|jest.mock|sinon.stub`; absent a matching ignored integration test in the same diff it blocks as `OverMocking`. (`E-13_tdd_verification_agents.md:44,68,126-128,134`)
- **E005-E13-008 [E][FAIL/TEST]** If mutation tests show every mutant of the real implementation surviving under mocked tests, the hook blocks as `MockHidesImpl`; the expensive cross-check is cost-gated to implementation files under 500 LOC. (`E-13_tdd_verification_agents.md:44,135,148,197`)
- **E005-E13-009 [E][SEC/TEST]** Verifier consensus has the hard invariant “at least one of N is non-LLM,” satisfied by cargo test, cargo-mutants, panic-free proptest, or deterministic hash; if D-3 is absent it lands locally in `aggregate_verifiers` and migrates later. (`E-13_tdd_verification_agents.md:46,73-76,136,153,198,204`)
- **E005-E13-010 [E][CAP/DATA]** `.claude/agents/mutant-hunter.md` accepts `{file_path,function_name,mutant_description,current_test_file}` and must output one compiling test that makes the named mutant red. (`E-13_tdd_verification_agents.md:19,48,71`)
- **E005-E13-011 [E][WF/FAIL]** Mutant hunting runs in new `FsmSubstate::MutantHunting` during AUDITING; the auditor reruns only that mutant, permits one retry, and escalates to HITL on the second survival. (`E-13_tdd_verification_agents.md:48,72,79-80,137`)
- **E005-E13-012 [E][CFG/A]** The hunter uses local Qwen-2.5-Coder-7B/Sonnet agent-file convention, must summarize before passing long functions, and warns/skips when a file exceeds an 18K-token per-mutant guard because the stated local context limits conflict between 20K and 25K references. (`E-13_tdd_verification_agents.md:48,193,199`)
- **E005-E13-013 [E][CAP/WF]** `FuzzGate` is CONCEPT-only and restricted to parser/wire Rust files and IPC-v2 serializers; local Qwen generates about 50 seeds, cargo-fuzz runs 60–120 seconds, a panic blocks as `FuzzPanic`, and corpora persist under `~/.cache/elai/fuzz_corpus/`. (`E-13_tdd_verification_agents.md:20,50,70,138`)
- **E005-E13-014 [E][DATA/CAP]** Every crate's `properties.md` enumerates round-trip serde, idempotence, monotonicity, and no-panic invariants; `spec-grounder` reads it and `spec_link_check.py` requires it above a LOC threshold. (`E-13_tdd_verification_agents.md:52,139`)
- **E005-E13-015 [E][CFG/WF]** `[testing.properties]` sets `min_loc_threshold=500` and `block_on_missing=true`; existing large crates receive stubs, with a `property-seeder` hunter variant proposing the first three invariants. (`E-13_tdd_verification_agents.md:52`)
- **E005-E13-016 [E][SEC/DATA]** `anti_vacuous.rs::VACUOUS_PATTERNS` is a `LazyLock<Vec<Regex>>` covering Rust unwrap-only, true, nonnegative-length, nondefault, bare `is_ok`, plus Python `True`, non-None, and nonnegative-length patterns; a hit records `VacuousTest`. (`E-13_tdd_verification_agents.md:54,69,106-108,149`)
- **E005-E13-017 [E][DET/CFG]** CONCEPT `IntegrationTestRunner` runs proptest and cargo-mutants in parallel from a fixed-seed, changed-file-gated corpus, caps concurrency at four, and measures property coverage rather than line coverage. (`E-13_tdd_verification_agents.md:56,161`)
- **E005-E13-018 [E][OBS]** Each CONCEPT audit emits `integration_property_coverage`; the observable artifact is `sessions/{id}/audit/mutation_survivors.toml` with zero survivors and dashboard-visible hunter spawns. (`E-13_tdd_verification_agents.md:56,87-90`)
- **E005-E13-019 [E][INT/FAIL]** E-11's per-level critic receives latest-file `mutation_score:f32`; below C-28's default 70, a PASS is downgraded to WARN regardless of the LLM verdict. (`E-13_tdd_verification_agents.md:58,75,165`)
- **E005-E13-020 [E][CAP/OBS]** The ReasoningDrawer mounts `MutationGateGauge` over the last 50 mutants, `PropertyCoverageGauge`, and `OverMockHistogram`, backed by `get_mutation_gate_status`, `get_property_coverage`, and `get_over_mock_stats`. (`E-13_tdd_verification_agents.md:60,86,142,170-174`)
- **E005-E13-021 [E][CFG]** `[testing.mutation_targeted]` defaults are enabled, TTL 30 days, `touched-files`, and zero surviving mutants; `[testing.property]` defaults are 10 cases, vacuity blocking, LOC 500; `[testing.anti_mock]` enables both integration-test and hidden-implementation checks. (`E-13_tdd_verification_agents.md:62`)
- **E005-E13-022 [E][DATA/OBS]** MistakeBook gains exactly `OverMocking`, `MockHidesImpl`, `VacuousTest`, and `FuzzPanic`; the deterministic SSB inventory names at least seven events: property pass/reject, mutation block, anti-mock block, fuzz panic, integration coverage, and hunter spawn. (`E-13_tdd_verification_agents.md:73-77,102-104,140-141`)
- **E005-E13-023 [E][INT]** Reachability is CLI/dashboard task dispatch through AUDITING and MutantHunting; B-41 dispatch consumes mutation evidence and D-3 remains the target aggregate. (`E-13_tdd_verification_agents.md:73-80`)
- **E005-E13-024 [E][CON]** New modules ban TODO/unimplemented, `Box<dyn Error>` in favor of per-module thiserror enums, non-test unwrap, hardcoded cache paths, and `NoopSsbNotifier`. (`E-13_tdd_verification_agents.md:81-85,143`)
- **E005-E13-025 [E][TEST/OBS]** The end-to-end encoder/decoder task must emit property pass and positive property coverage during EXECUTING, zero survivor blocks at AUDITING, and `anti_mock_block` plus `OverMocking` if an unpaired `vi.mock` is added. (`E-13_tdd_verification_agents.md:87-90`)
- **E005-E13-026 [E][TEST]** Structural checks require four module definitions/use sites, the hunter file, seven event registrations, startup-compiled vacuity regexes, large-crate `properties.md` enforcement, and cargo-mutants 27.x. (`E-13_tdd_verification_agents.md:92-116`)
- **E005-E13-027 [E][TEST]** Smoke gates separately prove vacuous property rejection, blocking a survivor in a touched function, and blocking `vi.mock` without integration coverage. (`E-13_tdd_verification_agents.md:118-129`)
- **E005-E13-028 [E][TEST]** Acceptance requires targeted budgets/cache, both anti-mock modes, non-LLM consensus, one-retry/HITL lifecycle, pattern-gated fuzzing, per-crate properties, four MistakeBook categories, seven events, and all three dashboard surfaces/commands. (`E-13_tdd_verification_agents.md:131-143`)
- **E005-E13-029 [E][TEST]** Property tests cover true/len/always-pass/round-trip/panic/ten-case convergence; mutation tests cover inside/outside touched scope and cache hit/miss. (`E-13_tdd_verification_agents.md:145-147`)
- **E005-E13-030 [E][TEST]** Anti-mock tests cover paired/unpaired mocks, hidden implementation, and genuine external mocks; fuzz tests cover clean/panic/timeout. (`E-13_tdd_verification_agents.md:148-150`)
- **E005-E13-031 [E][TEST]** Hunter tests cover spawn, compilable kill-test, and second-survival HITL; properties tests cover small exemption/large requirement; D-3 tests cover all-LLM, one deterministic, mixed, and empty sets. (`E-13_tdd_verification_agents.md:151-153`)
- **E005-E13-032 [E][CAP/STATUS]** The file plan creates six orchestrator modules, one agent, about 15 properties files, three Svelte gauges, and modifies auditing, per-level evidence, SSB, TOML, link check, FSM, drawer, and Tauri commands. (`E-13_tdd_verification_agents.md:155-174`)
- **E005-E13-033 [E][A/CFG]** The 70% mutation threshold is explicitly ungrounded; the source proposes measuring five crates, targeting at least 40% local-7B kill rate, then retuning. (`E-13_tdd_verification_agents.md:191-192,202`)
- **E005-E13-034 [E][A/CFG]** The 10-case property floor is conservative versus PGS's 100 and is intended to be tuned through config. (`E-13_tdd_verification_agents.md:195`)
- **E005-E13-035 [E][A/FAIL]** Fuzzing's 60–120-second cost is accepted at CONCEPT level; per-transition seed corpora are deferred. (`E-13_tdd_verification_agents.md:196`)
- **E005-E13-036 [E][DEP/STATUS]** cargo-mutants must be pinned in config and source-verified quarterly; D-3 absorption is a scheduled post-D-3 patch. (`E-13_tdd_verification_agents.md:194,198,204`)
- **E005-E13-037 [E][INT/STATUS]** E-14 Playwright later plugs into `IntegrationTestRunner` as another non-LLM verifier. (`E-13_tdd_verification_agents.md:203`)
- **E005-E13-038 [E][STATUS]** The planned local kill-rate and anti-over-mock threshold curves remain measurement-only follow-ups after E-13 ships. (`E-13_tdd_verification_agents.md:202,205`)
- **E005-E13-039 [E][STATUS]** The source estimates two to three sessions/10–12 hours and contains no shipped/completed marker. (`E-13_tdd_verification_agents.md:3`)

### E-15_sandboxed_exec.md — 235/235 lines

- **E005-E15-001 [E][SEC/CAP]** E-15 hardens Tier-C execution with Firecracker hot snapshots, argument-filtered seccomp, explicit privilege-drop ordering, no chroot lane, allowlisted researcher networking, per-baby-step ephemerality, and the Windows AppContainer/Job/LowIL trio. (`E-15_sandboxed_exec.md:1-3`)
- **E005-E15-002 [E][DEP/STATUS]** It augments planned B-18/C-11, reuses B-37 `TierCSpawn` and existing sandbox policy, replaces the Phase-0/1 chroot fallback, and is parallel-safe with E-11 through E-14. (`E-15_sandboxed_exec.md:4-5,12-20`)
- **E005-E15-003 [E][SEC/OBS]** The source treats the runc bind-mount CVEs, Pyodide host-bridge RCE, prompt-injection escapes, and persistent malicious package installs as the threat model; snapshot restore evicts persistence but does not prevent the first download. (`E-15_sandboxed_exec.md:8-10,229`)
- **E005-E15-004 [E][DATA/CFG]** `FirecrackerHotPool` stores `HashMap<(AgentId,AgentRole),SnapshotHandle>` with `restore_budget_ms:200`; first spawn cold-boots and CoW-snapshots, later same-pair spawns restore. (`E-15_sandboxed_exec.md:41,68-70,154`)
- **E005-E15-005 [E][DATA]** Snapshots live at `.elai_cc/sessions/{session_id}/snapshots/{agent_id}-{role}.snap`. (`E-15_sandboxed_exec.md:41`)
- **E005-E15-006 [E][FAIL/OBS]** A restore exceeding 200 ms falls back to cold boot and emits `firecracker_restore_slow`. (`E-15_sandboxed_exec.md:41,63,176`)
- **E005-E15-007 [E][SEC/DET/OBS]** Every restore regenerates TCP-ISN seed, `/dev/random` pool, and hostname salt and emits `snapshot_uniqueness_regenerated`; ten restores must prove all three orthogonal. (`E-15_sandboxed_exec.md:41,63,65,140-142,156,173`)
- **E005-E15-008 [E][SEC/CFG]** `IPC_BASELINE` denies `socket()` completely; `NETWORK_EXTRA` allows only address family 2/10 and socket type 1/2, expressly denying AF_UNIX, AF_NETLINK, AF_PACKET, AF_BLUETOOTH, and raw sockets. (`E-15_sandboxed_exec.md:43,108-110,157,174`)
- **E005-E15-009 [E][CON]** seccomp checks scalar arguments only; `sockaddr*` destination enforcement belongs to veth/iptables because kernel BPF cannot dereference it. (`E-15_sandboxed_exec.md:43`)
- **E005-E15-010 [E][SEC/WF]** `clone_cb` must run `PR_SET_NO_NEW_PRIVS` before the full `PR_CAPBSET_DROP` loop and install seccomp last; inversion is a tested failure. (`E-15_sandboxed_exec.md:45,112-114,158,177`)
- **E005-E15-011 [E][SEC/STATUS]** All chroot setup is removed rather than deprecated; supported lanes become Firecracker with KVM, gVisor without KVM, and AppContainer on Windows, with a `chroot_removed` migration note. (`E-15_sandboxed_exec.md:16,47,87,116-118,159,178`)
- **E005-E15-012 [E][FAIL/CFG/OBS]** `GvisorBackend` invokes `runsc` with `platform=systrap` when `kvm_available()==false`, accepts an expected 10–20% syscall overhead, and emits selected backend on every launch. (`E-15_sandboxed_exec.md:49,71,160,185`)
- **E005-E15-013 [E][SEC/WF]** `ResearcherNetSetup` creates a veth pair/netns and an iptables OUTPUT allowlist for resolved/pinned curated mirror IPs and trusted DNS; everything else is `REJECT`, not `DROP`, to return fast `EHOSTUNREACH`. (`E-15_sandboxed_exec.md:51,72,144-146,161`)
- **E005-E15-014 [E][SEC/CFG]** Researchers have zero capabilities including `CAP_NET_RAW` and `CAP_NET_ADMIN`; allowed FQDNs and resolvers come from `[researcher.net]` and are resolved once at boot. (`E-15_sandboxed_exec.md:51,161,175`)
- **E005-E15-015 [E][WF/SEC]** Each baby step calls `spawn_from_snapshot`; the long-lived “1” control plane remains, while all four data-plane workers are newly restored, never process-reused, and emit `agent_forked_from_snapshot`. (`E-15_sandboxed_exec.md:53,81-84,162,179`)
- **E005-E15-016 [E][SEC/STATUS]** `PyodideBridge` is a file stub while Pyodide is absent; any future import is blocked until `globalThis.require`, `process`, and Node child_process/fs/net handles are pruned before execution. (`E-15_sandboxed_exec.md:55,73,90,163,181,227`)
- **E005-E15-017 [E][SEC/DATA]** The Windows fallback requires a unique per-agent AppContainer SID with zero capabilities, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, active-process limit one, and a Low Mandatory Label on the working directory. (`E-15_sandboxed_exec.md:57,91,136-138,164,180`)
- **E005-E15-018 [E][WF/CFG/OBS]** Snapshot GC is LRU by `max_snapshots=64` and age by 24 hours since restore, emits `snapshot_evicted`, and reports directory size to the auditor. (`E-15_sandboxed_exec.md:59,125,165,176`)
- **E005-E15-019 [E][CFG]** `[sandbox.firecracker]` defaults: enabled when KVM exists, templated snapshot dir, 200-ms restore budget, 64 snapshots, 86400-second max age, uniqueness mitigation true. (`E-15_sandboxed_exec.md:61,128-130,166`)
- **E005-E15-020 [E][CFG]** `[sandbox.gvisor]` defaults to `enabled_if_no_kvm=true, platform=systrap`; `[sandbox.appcontainer]` enables kill-on-close, process limit one, and LowIL. (`E-15_sandboxed_exec.md:61`)
- **E005-E15-021 [E][CFG]** `[researcher.net]` initially allows `arxiv.org` and `huggingface.co`, trusts `1.1.1.1` and `8.8.8.8`, and sets `reject_not_drop=true`. (`E-15_sandboxed_exec.md:61`)
- **E005-E15-022 [E][CAP/OBS]** `get_sandbox_status` gains backend, restore p50, snapshot count, uniqueness count, allowed hosts, and last-ten blocked attempts; `SandboxBackendPanel` mounts in OrchestrationDrawer. (`E-15_sandboxed_exec.md:63,80,94,168,188,197-198`)
- **E005-E15-023 [E][OBS]** The six named SSB actions are `firecracker_restore_slow`, `snapshot_uniqueness_regenerated`, `snapshot_evicted`, `sandbox_backend_selected`, `agent_forked_from_snapshot`, and `researcher_net_egress_blocked`. (`E-15_sandboxed_exec.md:63,124-126,167`)
- **E005-E15-024 [E][TEST/OBS]** POSIX and PowerShell measurement scripts boot 20 sandboxes and persist cold/restore p50/p95 in `metrics/sandbox_perf.toml`; the end-to-end p50 acceptance threshold is under 100 ms. (`E-15_sandboxed_exec.md:65,148-150,155,170,189`)
- **E005-E15-025 [E][INT]** `SandboxBackend::{Firecracker,Gvisor,AppContainer}` refactors `launch_agent`, consumes `SandboxConfig`, registers Firecracker as `TierCSpawn`, and is reachable from CLI, dashboard submit, and isolation tests. (`E-15_sandboxed_exec.md:67-84`)
- **E005-E15-026 [E][CON]** New sandbox code bans TODO/unimplemented, any retained chroot or blanket socket allow, inverted ordering, unpruned Pyodide bridges, incomplete Windows Job isolation, `Box<dyn Error>`, and non-test unwraps. (`E-15_sandboxed_exec.md:85-93,169`)
- **E005-E15-027 [E][TEST/OBS]** Five-step malicious-install smoke expects one backend event per spawn, five fork and uniqueness events, blocked unknown egress, distinct PIDs, clean restored state, and an LRU-bounded snapshot directory. (`E-15_sandboxed_exec.md:95-98`)
- **E005-E15-028 [E][TEST]** Deterministic static checks require three new backend/network modules, argument-family tokens in allow/deny paths, privilege tokens in correct order, zero production chroot hits, snapshot helpers, six events, and four TOML tables. (`E-15_sandboxed_exec.md:100-130`)
- **E005-E15-029 [E][TEST]** Platform smoke checks privilege ordering under Linux capabilities, AppContainer completeness on Windows, ten-restore uniqueness, unknown-host REJECT, and measured restore p50 under 100 ms. (`E-15_sandboxed_exec.md:132-150`)
- **E005-E15-030 [E][TEST]** Seccomp tests cover both allowed families/types and denied UNIX/NETLINK/PACKET/RAW; network tests cover allow, reject, trusted DNS, and both dropped caps. (`E-15_sandboxed_exec.md:172-175`)
- **E005-E15-031 [E][TEST]** Restore tests cover under-budget, cold fallback, first cold boot, and LRU eviction; ordering tests cover both inversions; ephemerality tests prove PID uniqueness, package eviction, and clean restore. (`E-15_sandboxed_exec.md:176-180`)
- **E005-E15-032 [E][CAP/STATUS]** The file plan creates Firecracker, gVisor, networking, conditional Pyodide, panel, and two measurement scripts; modifies sandbox dispatch/policy/task runner/SSB/config/tests/dashboard. (`E-15_sandboxed_exec.md:183-198`)
- **E005-E15-033 [E][A/FAIL]** If measured baby-step p50 exceeds 100 ms, a prebooted warm pool is deferred as a follow-up. (`E-15_sandboxed_exec.md:221-222`)
- **E005-E15-034 [E][A/FAIL]** Windows Firecracker depends on Hyper-V detection through WMI and likely degrades to AppContainer+Job; the decision must be documented. (`E-15_sandboxed_exec.md:223`)
- **E005-E15-035 [E][SEC/A]** Snapshot seed regeneration does not stop an attacker reading kernel boot ID before restore; operating documentation must preserve this residual risk. (`E-15_sandboxed_exec.md:224`)
- **E005-E15-036 [E][A/DET]** Netns teardown cost is unmeasured; only if it dominates 100 ms may a netns be reused within a session, while session boundaries still tear down. (`E-15_sandboxed_exec.md:225`)
- **E005-E15-037 [E][A/FAIL]** gVisor must be benchmarked on cargo/npm/pip; overhead over 300 ms triggers documentation and possible Kata consideration. (`E-15_sandboxed_exec.md:226`)
- **E005-E15-038 [E][CON]** REJECT deliberately leaks the policy boundary in exchange for responsive UX; this is an accepted trade. (`E-15_sandboxed_exec.md:228`)
- **E005-E15-039 [E][INT/SEC]** E-11 PackageVerifier prevents suspect downloads while E-15 only evicts their state; both are explicitly complementary. (`E-15_sandboxed_exec.md:229,234`)
- **E005-E15-040 [E][INT]** E-14 Playwright requires node+npx to be added to this sandbox's program allowlist. (`E-15_sandboxed_exec.md:233`)
- **E005-E15-041 [E][STATUS/CON]** Apple/container is macOS-only and deferred; managed sandboxes are catalogued but not chosen because the system is local-first. (`E-15_sandboxed_exec.md:218-219`)
- **E005-E15-042 [E][STATUS]** The source estimates three sessions/12 hours and contains no shipped/completed marker. (`E-15_sandboxed_exec.md:3`)

### E-28_attention_kernels_2026.md — 195/195 lines

- **E005-E28-001 [E][CAP/CON]** E-28 enforces FA2 on Ampere, forbids local FA3/FA4 and MLA swaps, adds opt-in StreamingLLM sinks, bumps flash-attn, and evaluates rather than defaults YaRN. (`E-28_attention_kernels_2026.md:1-3`)
- **E005-E28-002 [E][DEP/DET]** It has no formal prerequisite, but hardware-heavy `server.py` edits must serialize with E-2/E-27; E-26 is orthogonal. (`E-28_attention_kernels_2026.md:4-6`)
- **E005-E28-003 [E][CFG/STATUS]** The current source claim is FA2 auto-detection with `ELAI_FORCE_SDPA=1` escape, `NF4+FA2` tier labeling, and an unchanged remote-sidecar SDPA recipe; E-28 preserves that fallback. (`E-28_attention_kernels_2026.md:18-25`)
- **E005-E28-004 [E][CON]** Qwen2.5-Coder-7B's dense GQA shape—28 query heads, 4 KV heads, head_dim 128, 28 layers, rope theta 1e6—supports the chosen kernels; mixed-head-dimension Gemma-family adoption is disallowed. (`E-28_attention_kernels_2026.md:14,56`)
- **E005-E28-005 [E][DET/CON]** FA2 forward is treated deterministic under fixed seed/workspace; future on-device training must set backward `deterministic=True` despite the cited ~37.9% throughput cost. (`E-28_attention_kernels_2026.md:16,57`)
- **E005-E28-006 [E][CFG/SEC]** `flash-attn` is pinned to 2.8.3 (or later 2.8.x acceptance), with PyPI identity/license verification before pinning and the Windows wheel fallback comment preserved. (`E-28_attention_kernels_2026.md:45,102-104,129,158`)
- **E005-E28-007 [E][CFG/CAP]** `ELAI_STREAMING_LLM_SINKS` accepts `0`, `1`, or integer N, defaults off, and when enabled keeps the first N tokens (paper default four) plus an `ELAI_LOCAL_MAX_CONTEXT` sliding window without altering FA2. (`E-28_attention_kernels_2026.md:47,73,84,130`)
- **E005-E28-008 [E][OBS/DET]** Each sink-enabled generation emits `streaming_llm_engaged {sink_count,window_size,context_position}`; masking is pure-Python/deterministic. (`E-28_attention_kernels_2026.md:47,60,78,95-97`)
- **E005-E28-009 [E][CFG/CAP]** `ELAI_YARN_FACTOR` accepts null/2.0/4.0/8.0, defaults off, and patches AutoConfig with Yarn factor and `original_max_position_embeddings:32768`. (`E-28_attention_kernels_2026.md:49,75,85,132`)
- **E005-E28-010 [E][FAIL/CON]** YaRN asserts max positions >=32768, remains opt-in, and factor >=4 cannot become default until the evaluation CSV receives human review. (`E-28_attention_kernels_2026.md:49,66,68,134,186`)
- **E005-E28-011 [E][CFG/FAIL]** With `ELAI_FORCE_SDPA=1`, generation must execute inside `sdpa_kernel([SDPBackend.FLASH_ATTENTION])`; unavailable FLASH_ATTENTION raises a clear error instead of silently falling to MATH. (`E-28_attention_kernels_2026.md:51,74,91,131`)
- **E005-E28-012 [E][OBS]** Boot-time SDPA selection emits `sdpa_backend_pinned`; YaRN emits `yarn_rope_scaling_applied {factor,original_max_position_embeddings,applied_at_layer}`. (`E-28_attention_kernels_2026.md:51,60,78,85,96`)
- **E005-E28-013 [E][SEC/CON]** Five rule rows block FA3/FA4 on Ampere, block an MLA weight-architecture swap, warn on mixed-head-dimension Gemma, warn that future FA2 backward needs deterministic mode, and block unpinned SDPA fallback. (`E-28_attention_kernels_2026.md:53-58,79,110-111`)
- **E005-E28-014 [E][INT/OBS]** The three SSB actions are `StreamingLlmEngaged`, `SdpaBackendPinned`, and `YarnRopeScalingApplied`; all must round-trip through the SSB contract. (`E-28_attention_kernels_2026.md:60,78,113-114,137,147`)
- **E005-E28-015 [E][CAP/STATUS]** The >25K runbook orders FA2+bf16, E-27 FP8 KV, validated YaRN factor 4, then sinks 4; MInference awaits SM86 measurement, DuoAttention calibration, and RingAttention at least two GPUs. (`E-28_attention_kernels_2026.md:12,62,68`)
- **E005-E28-016 [E][TEST/DET]** Streaming parity compares 30K FA2 baseline with sinks=4 and requires code-completion quality parity, VRAM no worse, throughput no worse, and three byte-equivalent runs. (`E-28_attention_kernels_2026.md:64,76,133,143`)
- **E005-E28-017 [E][TEST/OBS]** YaRN evaluation sweeps factors 2/4/8 at 8K/25K/64K/128K on needle and code-edit fixtures and writes `.elai_cc/metrics/yarn_eval_factor_sweep.csv`. (`E-28_attention_kernels_2026.md:66,77,134,144`)
- **E005-E28-018 [E][STATUS/CON]** `E-28-decisions.md` records Ampere FA2-only, env-gated sinks, human-gated YaRN, three deferred long-context methods, and out-of-scope MLA/Gemma/cuDNN choices. (`E-28_attention_kernels_2026.md:68,156`)
- **E005-E28-019 [E][WF/DET]** Default boot without overrides must remain byte-identical to current behavior. (`E-28_attention_kernels_2026.md:83-86,130`)
- **E005-E28-020 [E][CON]** Stub bans forbid FA3/FA4/cudnn imports, silent SDPA→MATH, and any MLA/DeepSeek-V3 architecture-swap path. (`E-28_attention_kernels_2026.md:87-93,135-136`)
- **E005-E28-021 [E][TEST/OBS]** The 30K REPL smoke expects sinks=4/window=25000/position=30000, a pinned backend boot event, baseline VRAM completion, and no forbidden imports. (`E-28_attention_kernels_2026.md:94-97`)
- **E005-E28-022 [E][TEST]** Deterministic checks cover version pin, all three helpers/envs, five rules, three events, both harnesses, absence of FA3/FA4 imports, and the decision document. (`E-28_attention_kernels_2026.md:99-125`)
- **E005-E28-023 [E][TEST]** Acceptance requires Windows/Linux install, default-off byte parity, explicit SDPA pin/error, applied YaRN event, 30K parity, factor CSV, zero FA3/FA4/MLA paths, preserved FA2 default, green workspace, and remote-3090 boot. (`E-28_attention_kernels_2026.md:127-139`)
- **E005-E28-024 [E][TEST]** Named tests cover sink parity/VRAM, YaRN sweep, SDPA invocation/error, forbidden imports, and three-action SSB round-trip. (`E-28_attention_kernels_2026.md:141-147`)
- **E005-E28-025 [E][CAP/STATUS]** Planned changes create four Python tests, one contract, and decision doc; modify sidecar, Python dependency pin, five rules, SSB, and remote setup. (`E-28_attention_kernels_2026.md:149-161`)
- **E005-E28-026 [E][A/TEST]** FA2 2.8.3 throughput on RTX3090 at 25K versus xFormers/FlexAttention is unmeasured; baseline measurement precedes any new default. (`E-28_attention_kernels_2026.md:182-184`)
- **E005-E28-027 [E][STATUS/A]** MInference at 50K–100K on SM86, DuoAttention transfer, and FlexAttention first-token cost remain deferred measurements. (`E-28_attention_kernels_2026.md:185,187-188`)
- **E005-E28-028 [E][A/CON]** The “cuDNN 75% faster” result is H100-only conjecture and is blocked from Ampere planning. (`E-28_attention_kernels_2026.md:10,189`)
- **E005-E28-029 [E][SEC/TEST]** Installation must verify the CUDA/torch compatibility matrix; hardware-heavy changes are tested on the remote 3090, not a second local sidecar. (`E-28_attention_kernels_2026.md:190-191`)
- **E005-E28-030 [E][INT]** Recommended composition is E-27 AWQ-Marlin weights + FA2 attention + FP8 KV for the 12GB profile; E-26 inherits the Ampere FA2 invariant. (`E-28_attention_kernels_2026.md:193-195`)
- **E005-E28-031 [E][STATUS]** MInference, DuoAttention, and RingAttention each retain an explicit adoption gate rather than being rejected permanently. (`E-28_attention_kernels_2026.md:195`)
- **E005-E28-032 [E][C]** Task 5 says “Add 4 Rules entries” but enumerates five and the wiring/deterministic sections also require five. (`E-28_attention_kernels_2026.md:53-58,79,110-111`)
- **E005-E28-033 [E][STATUS]** The source estimates one to two sessions/6–10 hours and contains no shipped/completed marker. (`E-28_attention_kernels_2026.md:3`)

### E-31_coding_model_leaderboard.md — 189/189 lines

- **E005-E31-001 [E][CAP/STATUS]** E-31 is a strategic delta, not a rebuild: lock a contamination-aware per-VRAM matrix, enforce a dense/head_dim-128/GQA/FA2 architecture invariant, and add cloud escalation under the shipped B-37 orchestrator. (`E-31_coding_model_leaderboard.md:1-3,14-22`)
- **E005-E31-002 [E][DEP/STATUS]** It requires shipped B-37 and existing model/tier documentation and is parallel-safe with E-32 through E-35. (`E-31_coding_model_leaderboard.md:4-6`)
- **E005-E31-003 [E][CON]** Qwen3 generations are rejected because hybrid DeltaNet/attention plus head_dim 256 breaks FA2/DoLa/steering; public Verified scores are treated as contaminated and private wall-clock/cost measurement is required. (`E-31_coding_model_leaderboard.md:8-12`)
- **E005-E31-004 [E][DATA/CFG]** The canonical four rows are 8GB Qwen2.5-Coder-7B abliterated NF4, 12GB same 7B at 25–32K, 24GB Qwen2.5-Coder-32B NF4, and 48GB one 32B plus cloud overflow or two 32B multi-tenant. (`E-31_coding_model_leaderboard.md:19,38`)
- **E005-E31-005 [E][DATA/OBS]** Each tier row records path, parameters, head_dim, NF4+FA2 verification, SWE-Pro number or explicit unmeasured marker, citation, and abliteration cost. (`E-31_coding_model_leaderboard.md:38,60-61,92-95,125`)
- **E005-E31-006 [E][CAP/FAIL]** `assert_head_dim_invariant` computes hidden-size/attention-heads and requires exactly 128, rejects `linear_attention_layers|delta_net_layers|gated_delta_net`, and requires `num_key_value_heads`; failure raises detailed `SidecarBootError`, never warns/falls back. (`E-31_coding_model_leaderboard.md:40,62-63,78,97-99,126`)
- **E005-E31-007 [E][CAP/CFG]** `EscalationPolicy::route(task_kind,ctx_len,prior_audit_failures)` defaults local and only selects cloud after SessionDag handoff or explicit `--cloud`. (`E-31_coding_model_leaderboard.md:42,64-65`)
- **E005-E31-008 [E][CFG/WF]** Cloud policy defaults Sonnet 4.6; Opus 4.7 triggers on real-repo/SWE-Pro markers, more than five touched files, or prior HIGH audit BLOCK; DeepSeek V4-Flash requires `--volume`/`cheap_volume`. (`E-31_coding_model_leaderboard.md:42-44,127`)
- **E005-E31-009 [E][OBS/DATA]** Every route records chosen runtime, reason code, and estimated USD; the canonical schema surface is `SsbActionType::RouteDecisionEvent` with `RouteDecisionEventKind::{CloudRouteChosen,TierModelSwapped,LocalRouteKept}` rather than three schema variants. (`E-31_coding_model_leaderboard.md:42,54,60-67,106-108,153`)
- **E005-E31-010 [E][OBS]** `HeadDimInvariantVerified` is a separate boot beacon emitted once after the invariant passes and displayed in RuntimePanel/AuditTimeline. (`E-31_coding_model_leaderboard.md:54,67,74-76,129`)
- **E005-E31-011 [E][CAP/TEST]** `run_internal_pro.py` scaffolds 10–15 shipped plan traces across local 7B/Sonnet/Opus and records wall-clock-to-PASS plus cost in dated JSON; running it is explicitly a follow-up. (`E-31_coding_model_leaderboard.md:46,68,133`)
- **E005-E31-012 [E][CON]** CLAUDE.md only points to the matrix (does not inline it) and records the Qwen3-family rejection to preserve token bounds and anti-stale policy. (`E-31_coding_model_leaderboard.md:48,110-112,130`)
- **E005-E31-013 [E][TEST/CFG]** A static contract parses configured `config.json` without loading weights, checks 128/GQA/no-hybrid markers, and skips under `ELAI_SKIP_SIDECAR_TESTS=1`. (`E-31_coding_model_leaderboard.md:50,115-116,131,138`)
- **E005-E31-014 [E][STATUS/CON]** The decision doc explains the invariant, rejects local MoE, justifies Sonnet/Opus/DeepSeek routing, and defers Devstral, OpenHands LM 32B, and numeric 24GB Qwen measurement. (`E-31_coding_model_leaderboard.md:52,150`)
- **E005-E31-015 [E][STATUS]** E-31.B defers the private 24GB Qwen2.5-Coder-32B SWE-Pro-style measurement until suitable VRAM is available. (`E-31_coding_model_leaderboard.md:56,185`)
- **E005-E31-016 [E][INT]** The policy hooks existing `RuntimeOrchestrator`, forwards its choice to `CloudGroundingClient`, checks model config pre-load, and is exposed in `/runtime status`. (`E-31_coding_model_leaderboard.md:69-76,83`)
- **E005-E31-017 [E][CON]** Default configs ban Qwen3, any local MoE path, silent invariant fallback, production unwrap, and hardcoded unconditional Sonnet. (`E-31_coding_model_leaderboard.md:77-82`)
- **E005-E31-018 [E][TEST/OBS]** An eight-file `TaskRunnerV2` refactor must choose Opus 4.7 with reason `multi_file_refactor_5plus`, persist estimated cost, update the audit timeline, and display current policy. (`E-31_coding_model_leaderboard.md:84-87,117-120`)
- **E005-E31-019 [E][TEST]** Deterministic checks require four tier rows, invariant tokens, policy choices for Sonnet/Opus/DeepSeek, Pattern-2 event/kinds plus boot beacon, CLAUDE pointer/rejection, and static contract. (`E-31_coding_model_leaderboard.md:89-121`)
- **E005-E31-020 [E][TEST]** Acceptance requires correct four-row matrix, loud boot refusal, the three-way cloud decision table, route payload on every cloud route, one boot beacon, no dependency addition, present scaffold, and decision document. (`E-31_coding_model_leaderboard.md:123-134`)
- **E005-E31-021 [E][TEST]** Tests include static config parsing, eight routing fixtures plus payload, route-event schema validity, and synthetic Qwen2.5 pass/Qwen3+Gemma failures. (`E-31_coding_model_leaderboard.md:136-141`)
- **E005-E31-022 [E][CAP/STATUS]** Files create the matrix, provider policy module/tests, contract, benchmark scaffold, and decision doc; modify sidecar, provider runtime, SSB, RuntimePanel, and CLAUDE.md. (`E-31_coding_model_leaderboard.md:143-155`)
- **E005-E31-023 [E][A/STATUS]** Devstral Small 2 remains a 24GB candidate only if reference-implementation inspection confirms head_dim 128; otherwise it is out. (`E-31_coding_model_leaderboard.md:166,176,186`)
- **E005-E31-024 [E][A/STATUS]** Qwen2.5-Coder-32B's 24GB recommendation is architectural, not backed by public SWE-Pro data; E-31.B fills that gap. (`E-31_coding_model_leaderboard.md:177,185`)
- **E005-E31-025 [E][FAIL/A]** DeepSeek's Anthropic-compatible shim must be verified before use and is deferred entirely if integration exceeds one day. (`E-31_coding_model_leaderboard.md:178`)
- **E005-E31-026 [E][TEST/A]** Abliteration must be re-evaluated on eight in-tree BabyStepSpec authoring tasks; noisy METR data prohibits claims from a single benchmark run. (`E-31_coding_model_leaderboard.md:179-180`)
- **E005-E31-027 [E][DET/OBS]** Per-task route events must be rate-limited/deduplicated by `(task_id,chosen_runtime)` to avoid SSB cardinality flood. (`E-31_coding_model_leaderboard.md:181`)
- **E005-E31-028 [E][INT/STATUS]** B-16 later lints every default model against the tier matrix; C-13 and future D-1 dispatch through the same policy. (`E-31_coding_model_leaderboard.md:187-189`)
- **E005-E31-029 [E][C]** Task prose calls the SQL action `cloud_route_decision`, while the wiring/file inventory canonicalizes one `RouteDecisionEvent` variant with a kind discriminator; both strings must be reconciled at implementation. (`E-31_coding_model_leaderboard.md:42,66-67,86,119,128,153`)
- **E005-E31-030 [E][CON]** No runtime trait changes and no new crate/dependency are intended. (`E-31_coding_model_leaderboard.md:16,22,132`)
- **E005-E31-031 [E][STATUS]** The source estimates one to two sessions/6–10 hours and contains no shipped/completed marker. (`E-31_coding_model_leaderboard.md:3`)

### E-33_dashboard_ux_agentic.md — 257/257 lines

- **E005-E33-001 [E][STATUS]** The top-of-file rollup declares E-33 `SHIPPED` in closure session `s-25120`, schema 103 unchanged, with 11 V2 deliverables, one retarget, one drift fix, four active wires closed, and only `cargo check` run; cargo tests remain pending. (`E-33_dashboard_ux_agentic.md:1-10`)
- **E005-E33-002 [E][STATUS/OBS]** Five planned SSB variants are schema-deferred but producer-ready; the new contract freezes schema 103 and locks their future SQL labels. (`E-33_dashboard_ux_agentic.md:2,4,8,79-80`)
- **E005-E33-003 [E][CAP/CON]** The UX delta, not redesign, adds a cross-window activity ribbon, async skeletons, confidence tags, reduced-motion/Wong signals, and footer mode while preserving the nine-window/ten-drawer calm-control-room architecture. (`E-33_dashboard_ux_agentic.md:12-16,19-25`)
- **E005-E33-004 [E][DEP/STATUS]** It extends shipped B-13/B-26, should follow C-6 token alignment, supersedes only C-7's gap-discovery portion, preserves outlined-only buttons, and leaves ActivityWindow's three modes unchanged. (`E-33_dashboard_ux_agentic.md:27-35`)
- **E005-E33-005 [E][CAP/DATA]** `ActivityRibbon.svelte` persists across all nine windows and renders FSM substate, active role, last tool, conversation token count, and line-by-line last-diff hint from ActivityWindow's SSB stream. (`E-33_dashboard_ux_agentic.md:55-56,93-100,107-108`)
- **E005-E33-006 [E][CFG/STATUS]** The ribbon is default-visible, controlled by `ui.activity_ribbon.enabled`; the shipped decision mounts it at WindowHost's top edge and wires Settings through `uxPrefs`. (`E-33_dashboard_ux_agentic.md:55-56`)
- **E005-E33-007 [E][CAP/WF]** Generic `<SkeletonList rows={5} loading={isLoading}/>` renders N shimmering rows and replaces them incrementally as data arrives rather than all at completion. (`E-33_dashboard_ux_agentic.md:58-59,95,115,190`)
- **E005-E33-008 [E][STATUS]** SkeletonList is wired to Plan phase rows, Explore symbol results, Review findings, and retargeted from nonexistent `ResearchWindow.svelte` to canonical `ResearchDrawer.svelte`. (`E-33_dashboard_ux_agentic.md:5,49,58-59`)
- **E005-E33-009 [E][CAP/DATA]** `ConfidenceTag.svelte` has four shape/text/color states—needs-evidence, low, ok, verified—with thresholds 0.25/0.50/0.85 and the raw numeric value only in a hover tooltip. (`E-33_dashboard_ux_agentic.md:61-65,96,120`)
- **E005-E33-010 [E][INT/STATUS]** OrbitGrcDots receives latest `reasoning_trace` confidence through `grc.svelte.ts`; OrbitEvidenceGates maps pass/warn/block/pending into the same bands and renders tag overlays via `foreignObject`. (`E-33_dashboard_ux_agentic.md:64-65,109-110`)
- **E005-E33-011 [E][CFG/SEC]** Signal-only tokens are blue `#0072B2`, orange `#E69F00`, vermilion `#D55E00`, rose `#CC79A7`, and green `#009E73`; brand teal/purple remain chrome-only. (`E-33_dashboard_ux_agentic.md:67-68,99,142-144,176`)
- **E005-E33-012 [E][SEC/CFG]** Reduced motion stops orbit rotation, freezes gradients, and replaces particles with a static dot grid plus text; `accessibility.motion` is `auto|reduced|full`, defaults to system, persists locally, and live-listens `MediaQueryList`. (`E-33_dashboard_ux_agentic.md:70-71,100,111,146-148,177`)
- **E005-E33-013 [E][CAP]** `WindowFooterMode.svelte` persistently displays FSM state, agent substate, and cap status at every window bottom; unlike the ribbon's live operation, it shows static mode. (`E-33_dashboard_ux_agentic.md:73-74,97,114,178`)
- **E005-E33-014 [E][CAP/SEC]** `SnackBarPermission.svelte` appears across windows above input on a permission event and exposes Approve/Reject/Always-allow. (`E-33_dashboard_ux_agentic.md:76-77,98,117,179`)
- **E005-E33-015 [E][INT/STATUS]** Because a dedicated permission action is deferred, the shipped permission store polls `intervention_request`, routes handlers through `stub.logStub`, and deduplicates by request ID. (`E-33_dashboard_ux_agentic.md:77`)
- **E005-E33-016 [E][OBS/STATUS]** Deferred actions/labels are `ActivityRibbonInteracted/activity_ribbon_interacted`, `ConfidenceTagHovered/confidence_tag_hovered`, `SkeletonReplaced/skeleton_replaced`, `ReducedMotionActivated/reduced_motion_activated`, and `WindowFooterModeChanged/window_footer_mode_changed`. (`E-33_dashboard_ux_agentic.md:79-80,101-105`)
- **E005-E33-017 [E][TEST/STATUS]** Deferred round-trip tests are `#[ignore]` until a future quintuple/sextuple schema unlock; currently the contract only freezes schema and labels. (`E-33_dashboard_ux_agentic.md:80,195`)
- **E005-E33-018 [E][STATUS/CON]** Phoenix trace-cycle graph, Devin repo wiki, and global `@`/`/` pickers are deferred as E-33.B/C/D. (`E-33_dashboard_ux_agentic.md:82-86,249-251`)
- **E005-E33-019 [E][STATUS/CON]** Additional deferred items are live-preview polish, checkpoint browser, and per-interaction token/cost counter as E-33.E/F/G. (`E-33_dashboard_ux_agentic.md:88-89,252-254`)
- **E005-E33-020 [E][STATUS/OBS]** `E-33-decisions.md` records ribbon rationale, confidence choice, signal/chrome separation, system-default motion, deferrals, schema lock, retarget, consumer wiring, permission mount, and settings catalog. (`E-33_dashboard_ux_agentic.md:88-89`)
- **E005-E33-021 [E][INT]** WindowHost is the ribbon/footer/permission mount; async resolutions would emit skeleton replacement, confidence hover would emit hover telemetry, and permission gates feed the snack bar. (`E-33_dashboard_ux_agentic.md:106-117`)
- **E005-E33-022 [E][CON/SEC]** UI bans remaining target spinners, visible numeric percentages, hue-only signals, unpausable motion over five seconds, solid CTA regression, ActivityWindow behavior changes, and Tauri unwraps. (`E-33_dashboard_ux_agentic.md:118-125`)
- **E005-E33-023 [E][TEST]** The interactive smoke switches Chat→Explore→Review during a long task and toggles reduced motion; artifact expectations are persistent ribbon data, halted orbit, shaped confidence glyphs, and hover numerics. (`E-33_dashboard_ux_agentic.md:127-130`)
- **E005-E33-024 [E][C/TEST]** The smoke expects the five deferred events despite the top rollup saying they are not registered/emitted under frozen schema 103. (`E-33_dashboard_ux_agentic.md:4,79-80,128-130`)
- **E005-E33-025 [E][TEST]** Static checks require all five components, all five palette tokens, reduced-motion CSS, WindowHost ribbon, two confidence consumers, four skeleton targets, five SSB variants, and clean `svelte-check`. (`E-33_dashboard_ux_agentic.md:132-169`)
- **E005-E33-026 [E][C/TEST]** The deterministic skeleton check still names absent `ResearchWindow.svelte`, contradicting the shipped retarget to `ResearchDrawer.svelte`. (`E-33_dashboard_ux_agentic.md:5,49,58-59,157-161`)
- **E005-E33-027 [E][TEST]** Acceptance covers persistent nine-window ribbon, four skeleton surfaces, four-state tags, five signal tokens, semantic reduced-motion parity, footer, permission bar, unchanged outlined chrome/orbit modes, accessibility redundancy, and no Tauri unwrap. (`E-33_dashboard_ux_agentic.md:171-185`)
- **E005-E33-028 [E][C/TEST]** Acceptance still requires all five telemetry variants registered/emitted, in conflict with the explicit schema-deferred shipped status. (`E-33_dashboard_ux_agentic.md:4,79-80,180`)
- **E005-E33-029 [E][TEST]** Component tests cover navigation persistence, incremental skeleton fill, distinct tag glyphs/tooltip, footer state/event, permission buttons/events, and media-query motion halt/event. (`E-33_dashboard_ux_agentic.md:187-195`)
- **E005-E33-030 [E][CAP/STATUS]** The planned file set creates five components and a decision doc, modifies tokens, WindowHost, orbit surfaces, four async surfaces, Settings, and SSB. (`E-33_dashboard_ux_agentic.md:197-216`)
- **E005-E33-031 [E][C/CFG]** The shipped decision says top-edge ribbon, while the unchanged risk text says ship bottom by default; the authoritative status annotation and risk section disagree. (`E-33_dashboard_ux_agentic.md:56,238`)
- **E005-E33-032 [E][A/CFG]** Confidence thresholds are explicitly heuristic and intended for telemetry-driven recalibration. (`E-33_dashboard_ux_agentic.md:239`)
- **E005-E33-033 [E][A/TEST]** Reduced-motion semantic parity is unmeasured; the target is task completion within ±5% of motion-on. (`E-33_dashboard_ux_agentic.md:240`)
- **E005-E33-034 [E][A/SEC]** Wong fills may clash with brand chrome; the proposed fallback is brand-tone borders with Wong fill. (`E-33_dashboard_ux_agentic.md:241`)
- **E005-E33-035 [E][STATUS/DEP]** Permission integration originally depended on a possibly absent event; the shipped closure resolves this through `intervention_request`. (`E-33_dashboard_ux_agentic.md:77,242`)
- **E005-E33-036 [E][DEP]** C-6 may rebase tokens and should precede E-33 or treat additions as append-only; C-7 becomes primarily smoke/remaining work. (`E-33_dashboard_ux_agentic.md:243,255`)
- **E005-E33-037 [E][A/TEST]** Vitest/jsdom reduced-motion testing may require a media-query mock helper. (`E-33_dashboard_ux_agentic.md:244`)
- **E005-E33-038 [E][CON]** The UI addition is estimated at roughly 700–800 new Svelte LOC and has no backend Rust surface beyond eventual SSB variants. (`E-33_dashboard_ux_agentic.md:126,245`)
- **E005-E33-039 [E][C/STATUS]** Header shorthand says “3/4 SkeletonList wirings,” while detailed shipped text names Plan, Explore, Review, plus the ResearchDrawer retarget; whether the retarget counts as the fourth is not normalized. (`E-33_dashboard_ux_agentic.md:3,5,58-59`)
- **E005-E33-040 [E][STATUS]** The initial plan estimated two to three sessions/8–12 hours, but the explicit status supersedes planning-only interpretation and marks shipped-with-tests-pending. (`E-33_dashboard_ux_agentic.md:1-16`)

### E-43_agent_skill_systems.md — 226/226 lines

- **E005-E43-001 [E][CAP/SEC]** E-43 makes `.claude/skills/` agentskills.io-v1 compliant, publishes three portable skills, adds per-skill trigger evals and telemetry, and blocks third-party imports until a recursive `mcp-scan`. (`E-43_agent_skill_systems.md:1-3`)
- **E005-E43-002 [E][DEP/STATUS]** It coordinates/absorbs E-22 export, extends planned B-19/B-20 hooks/tool registry, may use C-39 for MCP transport, and is parallel-safe with E-41/E-42/E-44/E-45. (`E-43_agent_skill_systems.md:4-5,18-25`)
- **E005-E43-003 [E][SEC/OBS]** The threat model explicitly includes malicious prompt injection in skill directories and scanner bypasses hidden under `tests/`/`scripts/`, so scanning only `SKILL.md` is insufficient. (`E-43_agent_skill_systems.md:10-12`)
- **E005-E43-004 [E][CAP/CON]** `anti-spiral`, `oracle`, and `prompt-transform` are publishable; project-shaped skills such as b-plan-runner/orchestrate/planning-workflow/idea-intake/impact-check/dashboard-handoff remain internal. (`E-43_agent_skill_systems.md:14-16`)
- **E005-E43-005 [E][DATA/SEC]** Every SKILL frontmatter is audited for `name` 1–64 kebab-case matching directory, `description` 1–1024 without angle brackets, optional license, compatibility <=500, free-form metadata, and experimental space-separated allowed-tools. (`E-43_agent_skill_systems.md:44`)
- **E005-E43-006 [E][CFG/DATA]** Internal skills declare `.elai_cc`/local-rag/FSM compatibility; publishable skills declare Claude Code design plus explicit Codex `$<skill-name>` invocation; every skill gets metadata version `1.0.0`, author `elai`, with bodies unchanged. (`E-43_agent_skill_systems.md:45-48`)
- **E005-E43-007 [E][DATA/TEST]** Each skill gains `eval/positive.jsonl` and `eval/negative.jsonl`, 10–20 prompts each, records `{prompt,expected_trigger,rationale}`, and is intended for 3–5-trial function/directive/efficiency evaluation. (`E-43_agent_skill_systems.md:50,93,127-132,160`)
- **E005-E43-008 [E][CAP/CFG]** `elai skills eval [--skill <name>] [--target claude|codex]` runs eval corpora through a configurable harness, defaulting to local-sidecar dry-run trigger detection, and reports per-skill precision/recall. (`E-43_agent_skill_systems.md:52,82,84-85,97,139-141`)
- **E005-E43-009 [E][OBS/FAIL]** The eval artifact is `workflow/audits/E-43-skill-eval-2026-05-19.md`; precision <90% or recall <80% emits `SkillEvalBelowThreshold` but is a soft gate, not a block. (`E-43_agent_skill_systems.md:52,87,109-110,152-154,161`)
- **E005-E43-010 [E][DATA/SEC/OBS]** PostToolUse telemetry appends `skill_invoked {skill_name,trigger_method:explicit|implicit,prompt_hash:blake3(prompt)[:16],outcome:passed|skipped|errored,ts}` to each session hook log and emits `SkillInvocationLogged`. (`E-43_agent_skill_systems.md:54,86,95,99,101,147-150,165`)
- **E005-E43-011 [E][CAP/WF]** `elai skills publish --target claude|codex|gemini` consumes E-22's `dist/skills/`; Claude is identity mapping/user-dir or zip, Codex writes `dist/codex/skills/<name>/SKILL.md` plus possible `agents/openai.yaml`, Gemini writes its Antigravity layout. (`E-43_agent_skill_systems.md:56-60,83,97,162-163`)
- **E005-E43-012 [E][SEC/FAIL]** Publishing re-fetches the target spec at startup; Codex warns above the roughly 8K-character initial catalog cap, and uncertain `openai.yaml` format must be feature-flagged with an unverified warning. (`E-43_agent_skill_systems.md:58-60,219`)
- **E005-E43-013 [E][SEC/WF]** `.claude/rules/skill-import-policy.md` requires `mcp-scan --recursive <skill-dir>` before any external skill reference; PreToolUse blocks unscanned skills and records scan hash/result in `workflow/skill-imports.log`. (`E-43_agent_skill_systems.md:62,89,98,104-105,134-137,164`)
- **E005-E43-014 [E][INT]** `conditional-routing.md` gains one dispatch line that loads the import policy whenever an external skill or third-party SKILL.md is imported/referenced. (`E-43_agent_skill_systems.md:64-66,187`)
- **E005-E43-015 [E][CAP/TEST]** CLAUDE.md's passive skills index must mention each publishable skill with a one-line trigger description to complement auto-trigger and explicit invocation. (`E-43_agent_skill_systems.md:68,166,188`)
- **E005-E43-016 [E][OBS/CON]** The scope-creep audit lists every skill's verbs and recommends splitting any over three, explicitly including `planning-workflow`; it makes no code change or automatic split. (`E-43_agent_skill_systems.md:70,168,195,220`)
- **E005-E43-017 [E][SEC/TEST]** Publication readiness requires validator success, body <=500 lines and <=5K tokens, positive precision >=95%, and zero recursive mcp-scan findings, output as a per-skill PASS/FAIL report. (`E-43_agent_skill_systems.md:72-77,167`)
- **E005-E43-018 [E][CAP/TEST]** `.claude/tools/agentskills_validator.py` is stdlib-only and asserts the six-field shape/limits. (`E-43_agent_skill_systems.md:73,90,115-118,177,189`)
- **E005-E43-019 [E][DATA/OBS]** Produced SSB variants are `SkillInvocationLogged`, `SkillEvalBelowThreshold`, and `SkillExternalImportScanned`. (`E-43_agent_skill_systems.md:81-90,143-145`)
- **E005-E43-020 [E][INT]** `skill-eval-runner` registers through the CLI/tool registry, export uses E-22's `export_all` when available, telemetry writes hook logs, and enforcement is reachable from PreToolUse/PostToolUse. (`E-43_agent_skill_systems.md:81-99`)
- **E005-E43-021 [E][CON/SEC]** Stub bans prohibit non-blake3 prompt hashes, production unwrap, dishonest portability claims, publishing without scan, and automatic unscanned import. (`E-43_agent_skill_systems.md:100-105,169`)
- **E005-E43-022 [E][CAP/OBS]** Optional/new `SkillsPanel.svelte` shows per-skill eval scores, last invocation, and scan state. (`E-43_agent_skill_systems.md:106,198`)
- **E005-E43-023 [E][TEST/OBS]** `elai skills eval --skill anti-spiral` must log each test invocation, only emit below-threshold when warranted, and create an audit row per skill with precision, recall, and sample sizes. (`E-43_agent_skill_systems.md:107-110`)
- **E005-E43-024 [E][TEST]** Deterministic checks validate one skill, require compatibility/version and both eval files for every directory, import rule routing, CLI symbols, three SSB variants, blake3-only hash code, and both audit artifacts. (`E-43_agent_skill_systems.md:112-155`)
- **E005-E43-025 [E][TEST]** Acceptance requires end-to-end eval, Codex/Gemini layout, policy routing, structured hook telemetry, explicit compatibility for all three publishables, readiness PASS for them, and scope-creep notes. (`E-43_agent_skill_systems.md:157-169`)
- **E005-E43-026 [E][TEST]** Tests cover eval JSON, Codex/Gemini dry-run trees, blake3 telemetry, validator valid/missing/overlong cases, import-policy contract, and eval-directory completeness. (`E-43_agent_skill_systems.md:171-179`)
- **E005-E43-027 [E][CAP/STATUS]** Files modify every skill frontmatter and routing/CLAUDE, create two corpora per skill, policy, validator, CLI/telemetry modules, three audits, runtime log, decision doc, and optionally SkillsPanel. (`E-43_agent_skill_systems.md:181-198`)
- **E005-E43-028 [E][A/FAIL]** `mcp-scan` packaging is unverified; if only an MCP server, it must use C-39 transport rather than guessing a CLI package. (`E-43_agent_skill_systems.md:217`)
- **E005-E43-029 [E][A/TEST]** The cited 53/79/100 adherence rates are not assumed to transfer; ELAI must measure its own procedural-skill behavior. (`E-43_agent_skill_systems.md:218`)
- **E005-E43-030 [E][A/CON]** Skill catalog metadata costs roughly 1.2K permanent tokens for 12 skills; post-ten-session telemetry may identify never-fired removal candidates. (`E-43_agent_skill_systems.md:221,226`)
- **E005-E43-031 [E][SEC/DEP]** The agentskills specification must be reverified at task start because it may drift. (`E-43_agent_skill_systems.md:222`)
- **E005-E43-032 [E][STATUS]** Public publication targets the three named skills; planning-workflow splitting remains a separate user-approved follow-on. (`E-43_agent_skill_systems.md:224-226`)
- **E005-E43-033 [E][CON]** Skill portability applies to the SKILL.md format, not vendor install paths. (`E-43_agent_skill_systems.md:10,56-60`)
- **E005-E43-034 [E][SEC]** Future third-party imports are categorically subject to the new policy and recursive scan. (`E-43_agent_skill_systems.md:226`)
- **E005-E43-035 [E][STATUS]** The source estimates two to three sessions/8–10 hours and contains no shipped/completed marker. (`E-43_agent_skill_systems.md:3`)

### E-50_saas_architecture_patterns.md — 316/316 lines

- **E005-E50-001 [E][CAP/DATA]** E-50 turns Designer stack selection into five typed SaaS archetypes with complete stack templates, mandatory multi-tenant RLS, a hard day-one gate, scoped IaC tokens, and a MoR/Stripe fork. (`E-50_saas_architecture_patterns.md:1-3`)
- **E005-E50-002 [E][DEP/STATUS]** It consumes shipped E-11/E-12 and B-7/B-40, is parallel-safe with E-14/E-15/E-17, and feeds its catalog into existing PipelineSpec/codegen. (`E-50_saas_architecture_patterns.md:4-6,33-40`)
- **E005-E50-003 [E][DATA]** Canonical `SaasArchetype` is exactly `SaasDashboard|B2bMarketplace|InternalTooling|DeveloperApi|ConsumerSocial`, with stable snake-case labels and `ALL_SAAS_ARCHETYPES` traversal. (`E-50_saas_architecture_patterns.md:8-16,25,66,92-95`)
- **E005-E50-004 [E][DATA/CON]** Legacy mapping is shape-dependent: B2C→dashboard/social, B2B→dashboard/marketplace, API-first→DeveloperApi, and AI-app collapses into one of the five; there is no `AiApp` root. (`E-50_saas_architecture_patterns.md:18-23,37`)
- **E005-E50-005 [E][DATA/CFG]** `StackLayer` has 14 members—FE framework/host, backend, DB, multitenancy, auth, billing, email, async, realtime, storage, caching, observability, rate limit—and `OverrideVendor {layer,vendor,reason}`. (`E-50_saas_architecture_patterns.md:66,95-96`)
- **E005-E50-006 [E][DATA/CFG]** PipelineSpec adds default `archetype:SaasDashboard`, empty `block_overrides`, `MultiTenancyMode::{Disabled,SharedSchemaRls,SchemaPerTenant,DbPerTenant,Hybrid{whales}}`, realtime, async, billing MoR, and a11y threshold; `From<bool>` maps true to RLS and false disabled. (`E-50_saas_architecture_patterns.md:35-39,66,97-103,134,212,224`)
- **E005-E50-007 [E][WF/OBS]** Idea-Framer maps seven named project classes to the five shapes, Designer may override blocks, and selection/each override emit `saas_archetype_selected`/`archetype_block_overridden`. (`E-50_saas_architecture_patterns.md:36,66`)
- **E005-E50-008 [E][CAP/WF]** Under `SharedSchemaRls`, detecting tenant/organization/workspace models creates one `rls-enforce-{table}` BabyStep per multitenant table. (`E-50_saas_architecture_patterns.md:39,68,104,117-120`)
- **E005-E50-009 [E][SEC/TEST]** Each RLS step enables row security, binds `org_id` to `auth.jwt() ->> 'org_id'`, and adds a cross-tenant access test expecting an empty result. (`E-50_saas_architecture_patterns.md:68,126,156-158,214`)
- **E005-E50-010 [E][SEC/FAIL]** RLS audit additionally requires a non-superuser migration role, a connection JWT/uid surviving pooling semantics, and CI failure for every table missing explicit enable+policy. (`E-50_saas_architecture_patterns.md:68`)
- **E005-E50-011 [E][OBS/DET]** RLS emits `rls_baby_step_emitted` plus per-check `rls_audit_check_passed|failed`, and policy integrity hashes use blake3 only. (`E-50_saas_architecture_patterns.md:68,115,133,190-194`)
- **E005-E50-012 [E][CAP/SEC]** `ProductionReadiness` runs as a hard pre-ship auditor gate and checks six families: admin/internal auth, edge rate limiting, Sentry-or-OTel dependency, zero-result pinned secret scan, RLS lint, and library-based webhook signature verification. (`E-50_saas_architecture_patterns.md:70,105,124,160-162,215`)
- **E005-E50-013 [E][FAIL/OBS]** Every readiness check produces an explicit named pass/block row; `production_readiness_gate_blocked` fires per failure and no silent pass is allowed. (`E-50_saas_architecture_patterns.md:70,124,138,215`)
- **E005-E50-014 [E][SEC/CAP]** `ScopedTokenEnforcer` inspects Railway/Fly/Vercel/AWS deploy diffs, verifies project/environment scope through platform APIs, denies destroy/delete authority, and emits `unscoped_token_blocked` on raw tokens. (`E-50_saas_architecture_patterns.md:72,106,164-166,216`)
- **E005-E50-015 [E][SEC/WF]** `--allow-unscoped-token` is refuse-by-default and requires interactive confirmation or timestamped `confirm_unscoped:true`; the opt-out is audited. (`E-50_saas_architecture_patterns.md:72,125`)
- **E005-E50-016 [E][CAP/OBS]** Every archetype scaffolds OTel instrumentation, SLOs, and one runbook per SLO; missing scaffolds lower B-40 critic quality and successful generation emits `otel_slo_runbook_scaffolded`. (`E-50_saas_architecture_patterns.md:74`)
- **E005-E50-017 [E][CFG]** Default SLOs are 99.9% availability, 500-ms p95, 1% error rate, and burn alerts at critical 14x/warn 6x/notice 1x. (`E-50_saas_architecture_patterns.md:74,168-172,217`)
- **E005-E50-018 [E][DATA/WF]** `merchant_of_record:Option<MorVendor>` supports Polar/Paddle/LemonSqueezy or `None` for Stripe+Anrok; FRAMING asks geography and three MRR bands and locks the billing block from that answer. (`E-50_saas_architecture_patterns.md:76,102,174-176,218`)
- **E005-E50-019 [E][CFG/OBS]** US-only under $100K suggests Stripe+Anrok; global suggests Polar or enterprise Paddle; decision emits `billing_mor_decision_made`. (`E-50_saas_architecture_patterns.md:76,138-139,234-235`)
- **E005-E50-020 [E][FAIL/OBS]** If the user is silent for 60 seconds, MoR defaults to US/<$100K Stripe+Anrok and emits `mor_decision_defaulted_no_user_input`. (`E-50_saas_architecture_patterns.md:310`)
- **E005-E50-021 [E][CFG/CAP]** `RealtimeNeed::{None,Presence,Crdt}` defaults `None`; explicit collaboration terms enable it, mapping CRDT to Liveblocks/Yjs and presence to PartyKit/Cloudflare Durable Objects. (`E-50_saas_architecture_patterns.md:78,100,178-180,219`)
- **E005-E50-022 [E][OBS/FAIL]** Realtime emits default-off/enabled rows; configurable regex plus an LLM up-vote may recover missed “live updates” intent and emits `realtime_block_enabled_by_llm_detection`. (`E-50_saas_architecture_patterns.md:78,311`)
- **E005-E50-023 [E][CFG/WF]** `AsyncBackbone` routes <30s to serverless, 30s–15m to Inngest, 15m–2h to Trigger.dev/Fly queue, and AI inference to Modal/fly-replay/Hatchet, recording vendor and latency rationale. (`E-50_saas_architecture_patterns.md:80,101,220,237`)
- **E005-E50-024 [E][CAP/CFG]** `derive_tier_limits` seeds free/pro/business at 10/100/1000 rpm, keys Upstash by `org_id`, and creates `ratelimit-per-tier`. (`E-50_saas_architecture_patterns.md:82,107,182-184,221,238`)
- **E005-E50-025 [E][SEC/CFG]** `a11y.lighthouse_min` defaults 90 and may not be lowered; all outputs include axe-core, Lighthouse a11y/perf/SEO >=90, and an i18n tree from day one even with only `en/`. (`E-50_saas_architecture_patterns.md:84,103,186-188,222`)
- **E005-E50-026 [E][DATA/CAP]** Five read-only seed TOMLs use canonical snake-case stems and fill all 14 layers; Designer mutates chosen seeds through one to three overrides. (`E-50_saas_architecture_patterns.md:86,120,143-154,213`)
- **E005-E50-027 [E][CFG]** The SaasDashboard example stack is Next.js/Vercel, Railway container, Neon+RLS, WorkOS AuthKit, Stripe+Anrok, Postmark, Inngest, Liveblocks, R2, Upstash, Better Stack, and Upstash Ratelimit. (`E-50_saas_architecture_patterns.md:86`)
- **E005-E50-028 [E][CAP]** Dashboard adds `ArchetypePicker`, `RlsAuditPanel`, `ProductionReadinessGate`, `ScopedTokenAuditPanel`, and `BillingDecisionPanel`, five matching Tauri commands, mounted in Designer or a SaaS drawer. (`E-50_saas_architecture_patterns.md:88,135,223,258-266`)
- **E005-E50-029 [E][INT]** Reachability flows Idea-Framer classification→PipelineSpec seed→pipeline builder/codegen→AUDITING readiness, consuming E-11 templates, E-12 scaffolds, tree-sitter if present, and blake3. (`E-50_saas_architecture_patterns.md:108-120`)
- **E005-E50-030 [E][CON/SEC]** Stub bans include TODO/unwrap, silent readiness checks, unconfirmed unscoped tokens, omitted RLS contract tests, non-Postgres RLS, destructive SQL, unlogged MoR choice, Sentry without OTel, default-on realtime, and missing endpoint rate limits. (`E-50_saas_architecture_patterns.md:121-134`)
- **E005-E50-031 [E][TEST/OBS]** A multitenant B2B CRM must map to `SaasDashboard`, `SharedSchemaRls`, realtime none, Inngest, Stripe+Anrok under US/<$100K, lighthouse 90, per-table RLS steps, and at least one missing-check block in the negative fixture. (`E-50_saas_architecture_patterns.md:136-139`)
- **E005-E50-032 [E][TEST]** Deterministic checks require five enum variants, six PipelineSpec field patterns, five seed TOMLs, RLS policy strings, six readiness check names, scoped-token refusal, five SLO trees, MoR variants, realtime off, tier RPMs, a11y 90, blake3, and zero DROP SQL. (`E-50_saas_architecture_patterns.md:141-207`)
- **E005-E50-033 [E][TEST]** Tests cover per-table marketplace RLS, missing admin auth/rate limit, unscoped/scoped Railway, US/global MoR, realtime trigger/no-trigger, async routes, rate-tier derivation, forbidden SQL, and Sentry-only blocking. (`E-50_saas_architecture_patterns.md:227-241`)
- **E005-E50-034 [E][CAP/STATUS]** Files create five orchestrator modules, five seed TOMLs, fifteen OTel/SLO/runbook artifacts, five panels, and five contract families; modify PipelineSpec, builder, framing, and Tauri commands. (`E-50_saas_architecture_patterns.md:243-267`)
- **E005-E50-035 [E][A/CFG]** The old B2C default remains an open Wasp/Open-SaaS versus Next+Supabase choice; proposed default is Wasp with overrides and an A/B time-to-paying-user measurement. (`E-50_saas_architecture_patterns.md:301`)
- **E005-E50-036 [E][A/CFG]** Polar is the proposed global MoR default but records `polar_maturity_risk:true`; Paddle/LemonSqueezy remain alternatives. (`E-50_saas_architecture_patterns.md:302`)
- **E005-E50-037 [E][A/OBS]** OTel+Sentry exporter is proposed over Sentry-native, with LOC and p95 overhead to be documented. (`E-50_saas_architecture_patterns.md:303`)
- **E005-E50-038 [E][A/CFG]** Inngest is proposed for indie/B2C and Trigger.dev for long-job B2B; a 20-plan A/B remains open. (`E-50_saas_architecture_patterns.md:304`)
- **E005-E50-039 [E][STATUS/DATA]** Automatic tenant promotion is out of scope; `Hybrid {whales}` merely supplies its future target. (`E-50_saas_architecture_patterns.md:305`)
- **E005-E50-040 [E][A/DATA]** The 14-layer enum may later need CDN/A-B/search/vector DB; `Custom(String)` is the proposed extension escape hatch. (`E-50_saas_architecture_patterns.md:306`)
- **E005-E50-041 [E][FAIL/A]** If tree-sitter is unavailable, readiness ships regex fallback first; platforms without queryable token scope emit `platform_unsupported_for_scope_check` and require a `scope-verified` comment. (`E-50_saas_architecture_patterns.md:307-308`)
- **E005-E50-042 [E][CFG/FAIL]** Readiness is extensible through `[saas.production_readiness.checks]` because the six checks are explicitly not exhaustive. (`E-50_saas_architecture_patterns.md:309`)
- **E005-E50-043 [E][C/STATUS]** The closing paragraph still says “four-archetype catalog,” conflicting with the canonical five-variant dictionary/tasks; canonical five is repeated throughout the refreshed source. (`E-50_saas_architecture_patterns.md:8-25,66,86,211,316`)
- **E005-E50-044 [E][C/DATA]** The objective calls the stack “13 layers” while enumerating 14 and `StackLayer`/checks consistently require 14. (`E-50_saas_architecture_patterns.md:2,66,86,95`)
- **E005-E50-045 [E][A/INT]** Module paths alternate between `architecture/saas_archetype.rs`, root `saas_archetype.rs`, and a deterministic grep of root `saas_archetype.rs`; implementation must resolve one canonical location. (`E-50_saas_architecture_patterns.md:10,66,93,175,245`)
- **E005-E50-046 [E][STATUS]** The source estimates three sessions/12 hours and has no shipped/completed marker despite describing E-11/E-12/B prerequisites as shipped. (`E-50_saas_architecture_patterns.md:3-5`)

### E-52_lsp_integration.md — 277/277 lines

- **E005-E52-001 [E][CAP/CON]** E-52 makes LSP an opt-in precision verifier, never the default retriever: one-shot diagnostics gate edits, headless SCIP refresh updates elai.map, and on-demand multilspy resolves overloaded/type-sensitive questions without an always-on language fleet. (`E-52_lsp_integration.md:1-3`)
- **E005-E52-002 [E][DEP/STATUS]** It depends on planned B-19 PostToolUse and B-29 symbol graph, coordinates with E-40's elai.map consumer, and is parallel-safe/complementary with E-51/E-53/E-54/E-55. (`E-52_lsp_integration.md:4-6,12-21`)
- **E005-E52-003 [E][CON/OBS]** LSP is justified for diagnostics/types while lexical+RAG remains retrieval; resident rust-analyzer/pyright/jdtls/clangd memory would exceed the sidecar envelope, and SCIP is preferred over LSIF. (`E-52_lsp_integration.md:8-10`)
- **E005-E52-004 [E][CAP/DATA]** `DiagnosticProbe` dispatches `.rs` to `rust-analyzer diagnostics --output-format=json`, `.py` to `pyright --outputjson`, `.ts/.tsx` to `tsc --noEmit --pretty false`, and `.svelte` to machine-output `svelte-check`. (`E-52_lsp_integration.md:45,68-71,125-127,192`)
- **E005-E52-005 [E][DATA]** Outcomes are `Pass|Diagnostics(Vec<Diagnostic>)|Timeout|ToolMissing`; `Diagnostic` holds file, line, column, severity, code, and message; severity is Error/Warning/Hint/Info. (`E-52_lsp_integration.md:45,68-71,117-127,193`)
- **E005-E52-006 [E][CFG/FAIL]** Defaults are 5-second Python, 10-second TS, and 15-second Rust timeouts; timeout never passes, while missing tools emit a soft warning and do not block. (`E-52_lsp_integration.md:45,98-99,192-193,212-214`)
- **E005-E52-007 [E][WF/DET]** PostToolUse resolves touched files, groups languages, runs at most three probes in parallel, and caches for 30 seconds by blake3(file content + tool version). (`E-52_lsp_integration.md:47,80-82,129-131,194,214`)
- **E005-E52-008 [E][OBS]** Dispatch emits `diagnostic_probe_dispatched {languages}`, completion emits outcome/count, and any Error emits `diagnostic_probe_block`. (`E-52_lsp_integration.md:47,73,109-112`)
- **E005-E52-009 [E][DATA/WF]** `DiagnosticProbeEvidence {language,file_count,diagnostics,outcome}` reaches AUDITING before LLM judgment; any Error forces BLOCK and structured fixer input, while warnings/hints inform but do not auto-block. (`E-52_lsp_integration.md:49,72,83,92,111-112,195`)
- **E005-E52-010 [E][CAP/DET]** `scip_refresh.py` maps changed Rust files to nearest Cargo crates, runs `rust-analyzer scip` per crate, writes `.claude/tools/scip_indices/<crate>.scip`, merges incrementally into elai.map, is idempotent, logs, and may queue/background from PostToolUse. (`E-52_lsp_integration.md:51,74-75,137-145,196,217`)
- **E005-E52-011 [E][DATA/DET]** `codebase_mapper.py --incremental` uses `.codebase_mapper.last_sha`; `--merge-scip` preserves canonical six-column header/SCIP_DEF/SCIP_REF rows and must remain stdlib-only. (`E-52_lsp_integration.md:53,75,143-145,197,218`)
- **E005-E52-012 [E][FAIL/STATUS]** If no existing vendored SCIP decoder exists, a minimal `_scip_proto_lite.py` may parse only Document/Occurrence/SymbolInformation/Range; it is conditional, not an unconditional deliverable. (`E-52_lsp_integration.md:53,227,273`)
- **E005-E52-013 [E][CAP/DATA]** `lsp_oracle.py` exposes `POST /lsp/find_references {symbol,language,repo_root}` returning `{file,line,kind}` and `POST /lsp/goto_definition` returning a canonical location. (`E-52_lsp_integration.md:55,76,147-149,198`)
- **E005-E52-014 [E][WF/DET]** multilspy servers spawn on demand, cache per language within a session, and shut down after 60 seconds idle; there is no permanent all-language daemon. (`E-52_lsp_integration.md:55,73,100-102,155-157,198,202`)
- **E005-E52-015 [E][SEC/FAIL]** Default RAM ceilings are 2GB Rust and 1GB each Python/TS; exceedance emits `lsp_server_ram_ceiling_exceeded`, kills the server, and disables further LSP for that crate/session in favor of elai.map. (`E-52_lsp_integration.md:55,73,100,151-153,219,269`)
- **E005-E52-016 [E][CAP/CON]** context-compiler/impact-mapper try elai.map first and invoke LSP only for multiple same-name SCIP definitions or type-inference questions, using results strictly as tie-breakers. (`E-52_lsp_integration.md:57,93,102,199,216,237-238`)
- **E005-E52-017 [E][CAP/CON]** `rust_ssr.py` wraps rust-analyzer structural search/replace for explicit >10-site refactors, is documented as opt-in, and is never auto-run by auditor/PostToolUse. (`E-52_lsp_integration.md:59,77,104,159-161,200,229`)
- **E005-E52-018 [E][CAP/SEC]** `check_ts_bindings.mjs` compares ts-rs bindings with Rust Tauri signatures, outputs drift/mismatches JSON, joins `cross_lang_diff.py`, and only adds pinned ts-morph after package verification and explicit dependency approval. (`E-52_lsp_integration.md:61,78,88,107,163-167,201,221,230,239,271`)
- **E005-E52-019 [E][STATUS/OBS]** `E-52-decisions.md` records verifier-not-retriever, headless default, measured diagnostic p99s, five open questions, and defers MGD, Glean, LibCST, and daemon-for-every-language. (`E-52_lsp_integration.md:63,232,266,275-277`)
- **E005-E52-020 [E][OBS]** Seven named events are diagnostic dispatch/completion/block, LSP spawn/idle shutdown/RAM exceedance, and SCIP refresh completion. (`E-52_lsp_integration.md:73`)
- **E005-E52-021 [E][INT]** Integration consumes B-19 validation dispatch/emit functions, D-1 `EvidenceGate`, elai.map/scip_indices/mapper, the sidecar server, cross-language checker, and blake3. (`E-52_lsp_integration.md:79-94`)
- **E005-E52-022 [E][CON]** New code bans production unwrap/TODO, silent ToolMissing/Timeout pass, permanent daemons, LSP-first retrieval, LSIF emit, automatic SSR, non-blake3 cache hashes, tree-sitter replacement, and unverified Python dependencies. (`E-52_lsp_integration.md:95-108`)
- **E005-E52-023 [E][CON]** There is no frontend pair; any diagnostic visualization is deferred to E-29/E-33. (`E-52_lsp_integration.md:108`)
- **E005-E52-024 [E][TEST/OBS]** A Rust `u32="string"` edit must yield dispatch, Diagnostics(1) with E0308, block, pre-LLM evidence consumption, BLOCK verdict, and diagnostic JSON in fixer context. (`E-52_lsp_integration.md:109-112`)
- **E005-E52-025 [E][TEST]** Static checks require all outcome/severity variants, four dispatches, hook registration, evidence type, stdlib-only SCIP tool, mapper flags, two endpoints, RAM/idle guards, SSR opt-in, TS drift tool, blake3-only hashing, no unwrap, and no LSIF. (`E-52_lsp_integration.md:114-181`)
- **E005-E52-026 [E][TEST]** Smoke tests prove extension dispatch, Rust error block, and timeout-not-pass. (`E-52_lsp_integration.md:183-188,206`)
- **E005-E52-027 [E][TEST]** Acceptance requires every supported write/edit to probe, pre-LLM blocking on errors, per-crate SCIP merge, on-demand endpoints/limits, LSP tie-break-only agent behavior, opt-in SSR, TS drift validation, no persistent fleet/LSIF, unchanged tree-sitter, blake3, green workspace, and three smoke passes. (`E-52_lsp_integration.md:190-206`)
- **E005-E52-028 [E][TEST]** Diagnostic tests cover four extensions, E0308 block/evidence, timeout, missing-tool soft warning, blake3 cache, and auditor-before-LLM ordering. (`E-52_lsp_integration.md:208-216`)
- **E005-E52-029 [E][TEST]** Tool/service tests cover SCIP idempotence, full-vs-incremental mapper parity, RAM kill/event, 60-second idle kill, and Rust↔Svelte drift detection. (`E-52_lsp_integration.md:217-221`)
- **E005-E52-030 [E][CAP/STATUS]** Files create diagnostic probe, SCIP refresh/conditional decoder, oracle, SSR, TS checker, 12 tests, and decision doc; modify task runner, evidence, auditor, mapper, two agents, and cross-language checker. (`E-52_lsp_integration.md:223-239`)
- **E005-E52-031 [E][A/CFG]** Rust diagnostic p99 must be below two seconds to remain a PostToolUse hook, but initial timeout is 15 seconds until `crates/cli` and orchestrator measurements retune it. (`E-52_lsp_integration.md:262-264`)
- **E005-E52-032 [E][A/TEST]** Pyright strict recall on 20 historical sidecar dtype/await bugs is a deferred first-session measurement. (`E-52_lsp_integration.md:265`)
- **E005-E52-033 [E][STATUS/A]** MGD on ten Qwen2.5-Coder-3B cross-module edits is out of E-52 ship scope and remains spec/pilot only. (`E-52_lsp_integration.md:266`)
- **E005-E52-034 [E][FAIL/A]** Windows multilspy paths are translated at the request boundary while the oracle runs on remote Linux; this assumption must be documented/tested. (`E-52_lsp_integration.md:267`)
- **E005-E52-035 [E][DET/A]** SCIP incremental merge is accepted only if symbol IDs remain stable across a three-run round trip; otherwise refresh falls back to full rebuild. (`E-52_lsp_integration.md:268`)
- **E005-E52-036 [E][FAIL/DEP]** If multilspy is unmaintained, the fallback is direct JSON-RPC to rust-analyzer/pyright/tsserver. (`E-52_lsp_integration.md:270`)
- **E005-E52-037 [E][FAIL/OBS]** Per-edit probe latency may add 1–15 seconds; retry cost is limited by content hashing, but first-edit cost remains the measurement target. (`E-52_lsp_integration.md:272`)
- **E005-E52-038 [E][INT]** E-53 consumes the diagnostic surface as a verifier input; later E-29/E-33 may visualize it. (`E-52_lsp_integration.md:275-277`)
- **E005-E52-039 [E][STATUS]** Patch E-52.1 is the candidate for Q1/Q2 measurements and tuned timeouts/pyright mode. (`E-52_lsp_integration.md:277`)
- **E005-E52-040 [E][CON]** Headless `analysis-stats` and `lsif` are mentioned as available rust-analyzer commands in context/read material, but neither is a shipped task; LSIF is explicitly prohibited. (`E-52_lsp_integration.md:10,32-33,103,179-181,203`)
- **E005-E52-041 [E][STATUS]** The source estimates two to three sessions/10–12 hours and contains no shipped/completed marker. (`E-52_lsp_integration.md:3`)

## Explicit cross-file interactions

These are source-stated interactions, not inferred architecture.

| Interaction ID | Explicit interaction | Anchors |
|---|---|---|
| E005-X-001 | E-10 and E-13 both strengthen verifier evidence: E-10 offers an opt-in typed certificate while E-13 mandates at least one deterministic non-LLM verifier; neither source says Typed-CoT can satisfy the non-LLM slot. | E10-013..015; E13-009 |
| E005-X-002 | E-10's tier/cloud experiments and E-31's production routing both consume B-37, but E-10 is measurement/read-only while E-31 changes `pick_runtime`. | E10-016..018,022..023; E31-007..010,016 |
| E005-X-003 | E-15 explicitly coordinates with E-50/E-11-style package/scoped-credential defenses: isolation evicts malicious step state, while upstream verification prevents download and E-50 constrains deploy credentials. | E15-039; E50-014..015 |
| E005-X-004 | E-15 exposes the sandbox program allowlist required by E-14 Playwright; this is a downstream requirement, not implemented by another file in this chunk. | E15-040 |
| E005-X-005 | E-28 and E-31 enforce the same local architectural lane: Qwen2.5 dense GQA/head_dim 128 with FA2, no Qwen3/Gemma/MLA/MoE default. | E28-004..005,013,020; E31-003,006,012,017 |
| E005-X-006 | E-31 adds runtime-policy state to the existing RuntimePanel; E-33 is the shipped cross-window UX delta and does not replace that panel. | E31-010,016,022; E33-003..006 |
| E005-X-007 | E-33's eventual interaction events and E-43/E-50/E-52 event families all depend on SSB schema capacity, but only E-33 explicitly freezes its five additions at schema 103. | E33-002,016..017; E43-019; E50-011..013; E52-020 |
| E005-X-008 | E-43 consumes E-22 export, B-19 hooks, B-20 registry, and optional C-39 transport; E-52 independently consumes B-19's same hook extension point for diagnostics. | E43-002,020,028; E52-002,007,021 |
| E005-X-009 | E-50's catalog is downstream of E-11/E-12 and explicitly coordinates with E-15 scoped-token/isolation work; its output reaches readiness auditing. | E50-002,014..016,029 |
| E005-X-010 | E-52's deterministic diagnostic evidence is explicitly a future E-53 verifier input and is complementary to E-13's deterministic-verifier rule, though no direct E-13 integration is specified. | E52-009,038; E13-009 |

## Inferred behaviors (not source facts)

- **E005-I-001 [I]** Taken together, E-10's read-only recommendations and E-31's automatic policy routing imply two distinct decision authorities: architecture deletion is human-gated, whereas per-task runtime choice may be automatic inside an approved table. This distinction is not stated across the two files. (E10-020,023,035; E31-007..009)
- **E005-I-002 [I]** E-13's property/mutation/fuzz chain plus E-52's compile diagnostics could form a layered deterministic verifier pipeline, but neither plan defines ordering or deduplication between those gates. (E13-003..019; E52-004..009)
- **E005-I-003 [I]** E-15 ephemerality and E-50 scoped-token refusal form defense in depth for IaC: a stolen scoped token has reduced authority and malicious local state is discarded at the next step. The combined guarantee is inferred. (E15-015,039; E50-014..015)
- **E005-I-004 [I]** E-28's kernel rules and E-31's model invariant together make local-model selection a conjunction of architecture, kernel, and tier policy rather than a simple model-path lookup. (E28-004,006,013; E31-004..008)
- **E005-I-005 [I]** E-33's persistent ribbon/footer and E-31's route events could surface cloud escalation continuously across windows, but no direct ribbon consumer for RouteDecisionEvent is specified. (E31-009..010; E33-005,013)
- **E005-I-006 [I]** E-43's eval→telemetry→scope audit loop could support evidence-based skill retirement, but retirement automation is only suggested as a future decision. (E43-007..010,016,030,032)
- **E005-I-007 [I]** E-50's five seed templates plus one-to-three block overrides aim to reduce unconstrained generation variance and should improve reproducibility, but the source supplies no determinism metric. (E50-003..007,026)
- **E005-I-008 [I]** E-52's 30-second cache and 60-second daemon lifetime may create concurrent version-skew windows after rapid edits; no explicit invalidation protocol beyond content hashes is defined. (E52-007,014)

## Ambiguity and underspecification register

- **E005-A-001 [A]** E-10 does not define user-rate defaults beyond rough us-east averages, how USD-normalization changes decisions, or the exact artifact schema for six ablation results. (E10-016,020..023,033)
- **E005-A-002 [A]** E-10 says missing reranker state may silently degrade yet acceptance says presence is asserted; behavior before/after the assertion and emitted failure type are unspecified. (E10-010)
- **E005-A-003 [A]** E-13 cites both an 18K guard, a 20K local context ceiling, and a 25K CLAUDE limit without defining which is canonical. (E13-012)
- **E005-A-004 [A]** E-13's mutation threshold, property-case floor, fuzz budget, and mock cross-check cost gate are provisional measurements rather than calibrated policy. (E13-008,033..035)
- **E005-A-005 [A]** E-15 uses reported ~28-ms restore as grounding, observable target `<50ms`, and acceptance `<100ms`; the enforcement threshold is 200ms, so performance labels have four distinct meanings that must not be collapsed. (E15-004,006,024,033)
- **E005-A-006 [A]** E-15 leaves Windows Hyper-V dispatch, netns reuse, gVisor/Kata fallback, and residual snapshot uniqueness conditional on measurements. (E15-034..037)
- **E005-A-007 [A]** E-28 leaves YaRN quality criteria, MInference SM86 viability, DuoAttention calibration transfer, FlexAttention overhead, and exact FA2 comparative throughput open. (E28-010,017,026..031)
- **E005-A-008 [A]** E-31's 24GB Qwen recommendation has no public/private Pro number; Devstral architecture and DeepSeek shim are also unresolved. (E31-014..015,023..025)
- **E005-A-009 [A]** E-33's threshold calibration, reduced-motion parity, palette interaction, and remaining cargo tests are unresolved even though the UI itself is marked shipped. (E33-001,032..034,037)
- **E005-A-010 [A]** E-43 does not yet know the install surface of mcp-scan or the exact Codex metadata schema and explicitly requires live verification. (E43-012,028,031)
- **E005-A-011 [A]** E-50's template vendor defaults, MoR maturity, OTel/Sentry trade, async vendor split, extension layers, and platform-specific scope checks remain proposals. (E50-035..042,045)
- **E005-A-012 [A]** E-52 leaves acceptable Rust diagnostic latency, pyright recall, Windows path translation, SCIP ID stability, multilspy maintenance, ts-morph approval, and optional protobuf decoding unresolved. (E52-012,018,031..037)

## Conflict register

- **E005-C-001 [C]** E-28 says “4 Rules” but defines/checks five; five is the repeated concrete inventory. (E28-013,032)
- **E005-C-002 [C]** E-31 alternates literal `cloud_route_decision` with canonical Pattern-2 `RouteDecisionEvent/RouteDecisionEventKind`; the intended SQL label versus Rust variant must be reconciled. (E31-009,029)
- **E005-C-003 [C]** E-33's closure says ribbon top-edge, while stale risk text says bottom default. (E33-006,031)
- **E005-C-004 [C]** E-33 marks five actions schema-deferred but its wiring, deterministic checks, smoke, and acceptance still require them live; the shipped status and frozen-schema contract are the latest explicit annotations. (E33-002,016..017,024..028)
- **E005-C-005 [C]** E-33 says `3/4` SkeletonList wirings in the rollup while detailed text names three windows plus ResearchDrawer; no explicit reconciliation says whether the retarget counts as four. (E33-008,026,039)
- **E005-C-006 [C]** E-50's refreshed canonical taxonomy has five variants while the closing prose retains “four-archetype”; five is confirmed by enum, templates, tests, and acceptance. (E50-003..004,026,032,043)
- **E005-C-007 [C]** E-50 calls the catalog 13 layers but enumerates/types/tests 14. (E50-005,044)
- **E005-C-008 [C]** E-50 gives three incompatible module paths for the same archetype surface. (E50-045)
- **E005-C-009 [C]** E-52 context lists `lsif` among available rust-analyzer commands, but the task contract and acceptance ban all LSIF output; availability is not adoption. (E52-003,022,040)
- **E005-C-010 [C]** The orientation summary conflicts materially with multiple originals; every such correction is itemized in the next section rather than resolved by silently following the summary.

## Summary additions and corrections

The orientation summary records only coarse session entries. The addition sets below are exhaustive by ledger ID: every cited explicit atom contains source detail absent from, more specific than, or status-corrective to the summary. Correction rows enumerate every summary atom that is contradicted or materially misnamed.

### E-10 summary delta

- **E005-SD-E10-ADD [ADDITION]** The summary does not recover the actual FSM/role maps, three-layer RAG, structured-only MistakeBook schema, typed certificate schema/opt-in, five-tier/three-suite eval, cloud switch, GRC no-removal eval, six-tile user gate, cache measurement/unavailable event, read-only report, exact events, tests, risks, or user-approved follow-ons: E10-005..040.
- **E005-SD-E10-C01 [CORRECTION]** Summary F01 says “five concrete FSM/role ablations”; the source has two profile ablations plus RAG, MistakeBook, TypedCoT, and tier decisions, displayed as six panels. (E10-001,005..020)
- **E005-SD-E10-C02 [CORRECTION]** Summary F02's “five conceded strengths” is not an E-10 task/acceptance inventory. No such five-item preservation contract appears in the assigned original.
- **E005-SD-E10-C03 [CORRECTION]** Summary F03/F05-F07 turns an iso-budget result into `ablation_pruned_role` and automatic agent-pool removal; the source emits measurement events/recommendations and explicitly requires a later user-approved change. (E10-006,008,020,035)
- **E005-SD-E10-C04 [CORRECTION]** Summary F04 names `workflow/reasoning/ablations/`; the source writes `workflow/reasoning/critic-decisions-{ts}.md` and several concrete harness modules. (E10-023,032)

### E-13 summary delta

- **E005-SD-E13-ADD [ADDITION]** Exact dual-oracle predicates, mutant command/budgets/cache key, two anti-mock categories, hunter state/retry/HITL, fuzz patterns/corpus, properties files, vacuity regexes, concurrency cap, mutation-score WARN, dashboard/config schemas, tests/risks are absent from the summary: E13-003..039.
- **E005-SD-E13-C01 [CORRECTION]** Summary F09 names `mutation_runner.rs`; the source creates `property_gate.rs`, `mutation_gate.rs`, `anti_mock.rs`, `anti_vacuous.rs`, `fuzz_gate.rs`, and `integration_test_runner.rs`. (E13-003..018,032)
- **E005-SD-E13-C02 [CORRECTION]** Summary event names `mutant_survived`, `anti_over_mock_block`, and `fuzz4all_finding` do not match the source's `mutation_gate_survivor_block`, `anti_mock_block`, and `fuzz_panic` deterministic inventory. (E13-022)
- **E005-SD-E13-C03 [CORRECTION]** Summary F14's generic “clean PR pass” understates the acceptance invariant: zero touched-function survivors plus the dual oracle and deterministic verifier requirement. (E13-003..009,025,028)

### E-15 summary delta

- **E005-SD-E15-ADD [ADDITION]** Pool keys/path/budgets/cold fallback, uniqueness regeneration, complete socket matrix, REJECT networking/DNS/caps, control/data-plane lifecycle, conditional Pyodide, Windows trio fields, GC/config/dashboard/measurement/test and fallback details are additions: E15-004..042.
- **E005-SD-E15-C01 [CORRECTION]** Summary F09 leaves the Windows trio unnamed; it is AppContainer zero-cap unique SID + Job kill-on-close/process-limit-one + LowIL working directory. (E15-017)
- **E005-SD-E15-C02 [CORRECTION]** Summary F11/F12 events `firecracker_snapshot_restored` and `seccomp_arg_blocked` are not in the original's six-event inventory. (E15-023)
- **E005-SD-E15-C03 [CORRECTION]** Summary F13 turns the cited 28-ms measurement into a smoke requirement; the source enforces `<100ms` acceptance, targets `<50ms` in the observable artifact, and falls back only over 200ms. (E15-006,024,033)
- **E005-SD-E15-C04 [CORRECTION]** Summary F14 names blocking a CVE proof of concept; the source's concrete gates remove chroot, test seccomp/order/ephemerality, and never names a CVE-PoC test. (E15-010..011,027..031)

### E-28 summary delta

- **E005-SD-E28-ADD [ADDITION]** Exact sink/YaRN/SDPA configs, five rules, deterministic semantics, runbook ordering, parity/sweep harnesses, deferral gates, tests and hardware constraints are additions: E28-003..033.
- **E005-SD-E28-C01 [CORRECTION]** Summary F07/F11 says sidecar refuses to boot without FA2; the source preserves the current `ELAI_FORCE_SDPA=1` escape and only forbids silent MATH fallback when that explicit mode cannot use FLASH_ATTENTION. (E28-003,011,019)
- **E005-SD-E28-C02 [CORRECTION]** Summary events `fa2_pinned` and `streamingllm_sink_active` are absent; the source defines `streaming_llm_engaged`, `sdpa_backend_pinned`, and `yarn_rope_scaling_applied`. (E28-008,012,014)
- **E005-SD-E28-C03 [CORRECTION]** The summary omits that YaRN is human-gated and that default boot must remain byte-identical. (E28-010,017,019)

### E-31 summary delta

- **E005-SD-E31-ADD [ADDITION]** Four exact tiers/fields, boot invariant, full escalation table, Pattern-2 event/beacon, private eval scaffold, static config test, decision/deferred/rate-limit details are additions: E31-004..031.
- **E005-SD-E31-C01 [CORRECTION]** Summary F05/F06 promises a MoE-compatible path without DoLa/steering; the source bans MoE in local defaults and provides no such path. (E31-003,014,017)
- **E005-SD-E31-C02 [CORRECTION]** Summary F07 places policy in `crates/orchestrator`; the source creates `crates/providers/src/runtime/escalation_policy.rs`. (E31-007,016,022)
- **E005-SD-E31-C03 [CORRECTION]** Summary F08 says refresh CLAUDE's tier table; the source explicitly forbids inlining it and adds only a pointer/rejection line. (E31-012)
- **E005-SD-E31-C04 [CORRECTION]** Summary F09 treats `tier_model_swap` as a standalone journal; the source makes `TierModelSwapped` one kind under `RouteDecisionEvent`. (E31-009)
- **E005-SD-E31-C05 [CORRECTION]** Summary F10-F12 omits 8GB and suggests all tier tuples resolve; the original has four tiers and defers a defensible 24GB performance result. (E31-004..005,015,024)

### E-33 summary delta

- **E005-SD-E33-ADD [ADDITION]** Shipped/test-pending/schema-deferred status, all five component APIs, four confidence states, exact Wong values, motion preference store, permission consumer, retarget, telemetry freeze, deferrals, tests and inconsistencies are additions: E33-001..040.
- **E005-SD-E33-C01 [CORRECTION]** Summary F01 says only a subset of five components is known; the original names all five: ActivityRibbon, SkeletonList, ConfidenceTag, WindowFooterMode, SnackBarPermission. (E33-005,007,009,013..014)
- **E005-SD-E33-C02 [CORRECTION]** Summary F07 says no backend overlap, but the source planned five SSB additions and then explicitly deferred them at schema 103; the current shipped work is frontend plus a schema-freezing contract. (E33-002,016..017,038)
- **E005-SD-E33-C03 [CORRECTION]** Summary events `activity_ribbon_frame` and `skeleton_screen_shown` are not source labels; future labels are `activity_ribbon_interacted` and `skeleton_replaced`. (E33-016)
- **E005-SD-E33-C04 [CORRECTION]** Summary F12's “skeleton bubbles” is Chat-style wording; E-33 uses row-by-row `SkeletonList` on four non-chat async surfaces. (E33-007..008)
- **E005-SD-E33-C05 [CORRECTION]** Summary F13's 60-fps ribbon scroll is absent; the original requires persistence and compressed activity state, with reduced motion applying to orbit/particle layers. (E33-005..006,012)

### E-43 summary delta

- **E005-SD-E43-ADD [ADDITION]** Exact frontmatter constraints/values, dual corpora schema, eval thresholds/artifacts, telemetry payload, three publish layouts/live verification, recursive policy/log, passive index, scope audit/readiness gates, tests and risks are additions: E43-005..035.
- **E005-SD-E43-C01 [CORRECTION]** Summary F13 names `scripts/skill_publish.sh`; the source implements/extends Rust `skills_cmd.rs` and `elai skills publish`. (E43-011,020,027)
- **E005-SD-E43-C02 [CORRECTION]** Summary F14 `skill_imported` is not the source event; it is `SkillExternalImportScanned`. (E43-013,019)
- **E005-SD-E43-C03 [CORRECTION]** Summary F16 uses singular `elai skill publish`; the source uses plural `elai skills publish`. (E43-011)
- **E005-SD-E43-C04 [CORRECTION]** Summary F17's signed payload accepted by `claude.skills.io` and F18's “unsigned” rejection do not appear; the source gates on recursive scan/readiness, not signatures. (E43-013,017,021)

### E-50 summary delta

- **E005-SD-E50-ADD [ADDITION]** Canonical taxonomy/mapping/wires, 14-layer schema, PipelineSpec enums/defaults, RLS templates/checks/events, six readiness algorithms, token enforcement, SLO defaults, MoR/realtime/async/rate/a11y rules, five panels, tests and risk fallbacks are additions: E50-003..046.
- **E005-SD-E50-C01 [CORRECTION]** Summary F01-F04 uses legacy B2C/B2B/API-first/AI-app roots; the refreshed original replaces them with five shape-based variants and expressly has no AiApp root. (E50-003..004)
- **E005-SD-E50-C02 [CORRECTION]** Summary F12 says four TOMLs; the original requires five canonical seed files. (E50-026)
- **E005-SD-E50-C03 [CORRECTION]** Summary F14/F15 paths `workflow/archetypes/*.toml` and `crates/orchestrator/src/archetype.rs` do not match `.elai_cc/templates/saas/*.toml` and the source's unresolved saas_archetype module paths. (E50-026,034,045)
- **E005-SD-E50-C04 [CORRECTION]** Summary F16 `archetype_selected` is named `saas_archetype_selected`; override events are also required. (E50-007)
- **E005-SD-E50-C05 [CORRECTION]** Summary F18/F19's B2C prompt is legacy; the source's smoke maps a multitenant B2B CRM to `SaasDashboard` and adds per-table RLS steps. (E50-004,008,031)

### E-52 summary delta

- **E005-SD-E52-ADD [ADDITION]** Four-language outcome schema/timeouts/cache/concurrency, pre-LLM evidence semantics, incremental SCIP/conditional decoder, oracle endpoints/idle/RAM behavior, tie-break triggers, opt-in SSR, TS drift, seven events, tests and measured fallbacks are additions: E52-004..041.
- **E005-SD-E52-C01 [CORRECTION]** Summary F03 says run `analysis-stats`; it is context-only, not a task or acceptance item. (E52-040)
- **E005-SD-E52-C02 [CORRECTION]** Summary F05 says run `lsif`; the source explicitly bans LSIF everywhere and requires SCIP only. (E52-022,040)
- **E005-SD-E52-C03 [CORRECTION]** Summary F06 presents SSR as a general headless capability; the source ships a separately invoked opt-in wrapper and forbids automatic use. (E52-017)
- **E005-SD-E52-C04 [CORRECTION]** Summary F13 path `crates/plugins/src/lsp_verifier.rs` is wrong; the main source module is `crates/orchestrator/src/diagnostic_probe.rs`, plus Python/tool surfaces. (E52-004,010..018,030)
- **E005-SD-E52-C05 [CORRECTION]** Summary F14 event `lsp_diagnostic_probed` is absent; the source defines seven dispatch/completion/block/server/SCIP events. (E52-008,020)

## Zero-omission and zero-duplicate audit

### Coverage totals

| Assigned source | Lines read | Explicit atomic units | First–last definition ID |
|---|---:|---:|---|
| `E-10_critic_of_elai_design.md` | 237/237 | 40 | E005-E10-001..E005-E10-040 |
| `E-13_tdd_verification_agents.md` | 205/205 | 39 | E005-E13-001..E005-E13-039 |
| `E-15_sandboxed_exec.md` | 235/235 | 42 | E005-E15-001..E005-E15-042 |
| `E-28_attention_kernels_2026.md` | 195/195 | 33 | E005-E28-001..E005-E28-033 |
| `E-31_coding_model_leaderboard.md` | 189/189 | 31 | E005-E31-001..E005-E31-031 |
| `E-33_dashboard_ux_agentic.md` | 257/257 | 40 | E005-E33-001..E005-E33-040 |
| `E-43_agent_skill_systems.md` | 226/226 | 35 | E005-E43-001..E005-E43-035 |
| `E-50_saas_architecture_patterns.md` | 316/316 | 46 | E005-E50-001..E005-E50-046 |
| `E-52_lsp_integration.md` | 277/277 | 41 | E005-E52-001..E005-E52-041 |
| **Total** | **2137/2137** | **347** | **347 unique explicit definitions** |

Additional trace IDs are analytical cross-references and are not counted as source atoms: **10** explicit interactions (`E005-X-*`), **8** inferences (`E005-I-*`), **12** ambiguities (`E005-A-*`), **10** conflicts (`E005-C-*`), and **47** summary-delta entries (`E005-SD-*`). Thus the document has **434 uniquely defined IDs total** and **347 source-explicit atomic units**.

Audit assertions:

- **Manifest/fingerprint completeness:** 9/9 assigned originals, 2,137/2,137 logical lines, 254,945/254,945 bytes, and all 9 SHA-256 values exactly match `DEEP-SOURCE-MANIFEST.md:578-592`; zero files are missing.
- **End-to-end read completeness:** every source was read from line 1 through its manifest-recorded final line. Title/objective, effort, prerequisites, parallel status, context, existing-state, mandatory-read inventory, every task, wiring produces/consumes/reachability/stub-ban/frontend/observable contract, deterministic checks, acceptance, tests, file inventory, citations, risks/open questions, and Next were included in the extraction pass. E-33's lines 1-10 shipped rollup and E-50's lines 8-25 taxonomy refresh were not skipped.
- **Feature-class completeness:** every required class is represented: capabilities/actions/surfaces; workflows/states/lifecycle; schemas/data; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failure/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred.
- **Structural-line handling:** headings, blanks, Markdown fences/separators, source read lists, file lists, and citation-only repetitions were read. Closely coupled inventories are retained in one atom and repeated acceptance/check/file wording is cross-anchored rather than double-counted.
- **Definition-ID audit:** a machine extraction found 347 explicit definition lines and 347 distinct explicit IDs; every per-source numeric series is contiguous from 001 to the terminal value above. Across all registers it found 434 definition IDs and 434 distinct values, with zero duplicates.
- **Duplicate-content control:** repeated task/wiring/check/acceptance claims are represented once per source. Interactions, inferences, ambiguity/conflict records, and summary deltas cite ledger atoms and do not increment the 347-atom count.
- **Explicitness/status control:** planning language is not mislabeled implementation. E-33's explicit shipped rollup is preserved, including test-pending and schema-deferred qualifications; all other assigned sources lack a shipped/completed marker and remain plans even when they describe prerequisites as shipped.
- **Summary control:** all nine relevant summary records were reconciled. The 47 delta definitions include every material addition/correction, including legacy taxonomy/event/path/status errors; nothing was promoted from the summary alone.
- **Boundary control:** only the nine assigned originals were opened. The manifest and `05-E-SUMMARY_INDEX.discovery.md` were administrative/orientation inputs; one prior generated deep-discovery artifact was inspected only to confirm the normalized self-hash convention and contributed no source claim. No companion report, code implementation, paper, web page, other E original, or other original chunk was read.
- **Output accounting:** one discovery file; **0585 lines**, **114273 bytes**; source corpus **9 files / 2,137 lines / 254,945 bytes**; explicit atomic count **347**.

### Output fingerprint

- Canonical output SHA-256, computed over this UTF-8 file with the 64-hex digest on this line replaced by sixty-four ASCII zeroes: `ae0b3938e5ffc67facc4812dbdc12c264693dcb471523f473df94c80d7f34155`
- A literal whole-file SHA-256 cannot be embedded as its own content without changing itself. The normalized replacement rule above makes the recorded digest reproducible; the literal final-file SHA-256 is supplied in the completion handoff.
