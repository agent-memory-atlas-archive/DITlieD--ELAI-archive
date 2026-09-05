# DEEP-D-003 — exhaustive source discovery ledger

## Scope, method, and evidence notation

This ledger is restricted to the eight original plans assigned to `DEEP-D-003`. The only orientation document consulted was `04-D-SUMMARY_INDEX.discovery.md`; it is used solely for delta comparison and never as evidence for a deep-source claim. No original plan from another manifest chunk was read.

- **`[E]` explicit**: directly stated by an assigned original, with a source alias and exact line anchor.
- **`[I]` inferred**: a bounded synthesis across explicit assigned-source statements; it does not assert unstated implementation.
- **`[A]` ambiguous/absent**: the assigned originals leave the behavior open or underspecified.
- **`[C]` conflicting**: two explicit statements cannot both be applied literally without qualification or a missing rule.
- **`[Δ+]` summary addition**: material detail present in an assigned original but absent from the summary layer.
- **`[Δ~]` summary correction**: an exact identifier, condition, status, or behavior in the assigned original changes or qualifies the summary layer.

Source aliases used below are `D2`, `D3`, `D13`, `D16`, `D17`, `D19`, `D25`, and `D30`, each referring to the correspondingly numbered manifest path. Atomic IDs are unique within this file. Line anchors are inclusive.

## Exact processed manifest and integrity verification

Manifest section: `DEEP-SOURCE-MANIFEST.md:484-497`. Declared family total: **8 files, 1,862 logical lines, 88,049 bytes**, status **within 1,800–2,800**. `sha256sum`, `wc -l`, and `wc -c` independently matched every row and the aggregate.

| Order | Source alias | Exact absolute path | Lines | Bytes | Manifest SHA-256 | Verification |
|---:|---|---|---:|---:|---|---|
| 1 | `D2` | `docs/thefinalglance/D-plan/D-2_ppm_sidecar_wiring.md` | 343 | 17,256 | `3641a99de8b4709ff8a7e574830f6797be9f26ecfe17e4642a737375531dd04c` | exact match |
| 2 | `D3` | `docs/thefinalglance/D-plan/D-3_verification_result_aggregator.md` | 341 | 17,622 | `f9bacb3334a1441ed4fda1fef1149c678f574e9bdf42d7334d6457c328f507fa` | exact match |
| 3 | `D13` | `docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md` | 310 | 14,877 | `f36727d2f018e3f05cdae593bf27760e56531c4b9c9d4399172d10d8398980c7` | exact match |
| 4 | `D16` | `docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md` | 264 | 12,347 | `97731ae36aa27f687453d09827818d5ee0390f46d7ab7ca92b3159fd3cdc73c2` | exact match |
| 5 | `D17` | `docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md` | 170 | 7,108 | `7d0e7eaae7ba12c412fd4f2a737372aec827fdb0a488775cf196419e7d4e3bc5` | exact match |
| 6 | `D19` | `docs/thefinalglance/D-plan/D-19_prmbench_adversarial_gaming.md` | 185 | 8,309 | `194e83b8aaf365041958d4a8612ba548920d41237468c10b395015da0e450835` | exact match |
| 7 | `D25` | `docs/thefinalglance/D-plan/D-25_determinism_archive_tamper_suite.md` | 111 | 4,082 | `20f72758b1a8165ae85d04058b167b562c29746faaad227b1414c135fcc68f11` | exact match |
| 8 | `D30` | `docs/thefinalglance/D-plan/D-30_calibration_bootstrap_corpus.md` | 138 | 6,448 | `f84076544ac127454260151e50f98013575af682e3f1bb40437953912d0ba923` | exact match |

## Per-file exhaustive ledgers

### D2 — PPM sidecar wiring

- **D003-D2-E001 `[E]` — capability/topology.** D2 introduces a second, step-level Process Preference Model sidecar, with policy `Qwen2.5-Coder-7B-Instruct-abliterated` on port `8100` and PPM `Qwen2.5-Math-PRM-7B` on `8101`; the two artifacts evolve independently against different signals and the verifier is not the policy in self-judge mode. (`D2:1-9`)
- **D003-D2-E002 `[E]` — prerequisites/status envelope.** Estimated effort is 2–3 sessions; execution requires D-1 (`ReasoningTemplate`, `VerifierKind::Prm`, `verifier_required=Prm`), B-37 (`PerRoleRuntime` and backend slot), and C-15 (`ClockProvider` plus entropy quarantine) all verified `DONE` by `spec-grounder`. (`D2:9-13`)
- **D003-D2-E003 `[E]` — concurrency.** D2 is declared parallel-safe with D-8 and the isolated D-19 benchmark surface, but not with D-3 because D-3 consumes its PPM output. (`D2:14-16`)
- **D003-D2-E004 `[E]` — model constraint/evidence.** The plan rules out the policy model as its own verifier, describes Qwen2.5-Math-PRM-7B as Qwen-family but separate-base/math-pretrained/PRM-distilled, cites 60–65 standard-category PRMBench performance, and explicitly delegates adversarial weakness measurement to D-19. (`D2:20-32`)
- **D003-D2-E005 `[E]` — deployment capacity.** The remote sidecar binds remotely and tunnels to loopback `127.0.0.1:8101`; the stated sizing is about 1.92 GB per NF4 7B model plus about 1.8 GB total KV cache, about 5.7 GB combined, fitting the stated 12/24 GB budget. (`D2:34-39`)
- **D003-D2-E006 `[E]` — existing integrations.** D2 reuses B-37 `RuntimeAdapter`, `PerRoleRuntime`, existing `HTTPClient`/retry logic in `sidecar_client.rs`, the Anthropic `/v1/messages` FastAPI speaker, D-1’s `VerifierKind::Prm` slot, and `[runtime.*]` configuration. (`D2:41-48`)
- **D003-D2-E007 `[E]` — research gate.** Before `TESTING_FIRST`, `paper-ingestor` must materialize `rstar-math-2501.04519-interface.md`, `latent-verifiers-2504.16760-interface.md`, and `prmbench-2501.03124-interface.md`. (`D2:70-79`)
- **D003-D2-E008 `[E]` — client schema.** New `ppm_client.rs` defines `PpmClient { endpoint: Url, http: Arc<reqwest::Client>, timeout: Duration, retry_policy: RetryPolicy }`. (`D2:81-87`)
- **D003-D2-E009 `[E]` — client action.** `PpmClient::new(...)` and `score_step(prompt: &str, candidate_step: &str, ctx: &PpmContext) -> Result<PpmScore, PpmError>` are the construction and scoring surfaces. (`D2:83-90`)
- **D003-D2-E010 `[E]` — score schema/range.** `PpmScore` contains `score: f32`, `confidence: f32`, and `raw_output: String`; `score` is constrained to `[0.0, 1.0]`. (`D2:90-91`)
- **D003-D2-E011 `[E]` — context schema.** `PpmContext` contains `problem_statement: String`, `prior_steps: Vec<String>`, and `domain: Domain`. (`D2:92-93`)
- **D003-D2-E012 `[E]` — error surface.** `PpmError` is a `thiserror` family containing `HttpError(#[from] reqwest::Error)`, `TimeoutError`, `ParseError(String)`, and `SidecarUnavailable`. (`D2:94-95`)
- **D003-D2-E013 `[E]` — deterministic request format.** `score_step` emits the fixed rStar-Math-style `Problem / Previous steps / Current step / Is this step correct? Rate 0.0 to 1.0` prompt; an unparseable response is a `ParseError`, not a guess. (`D2:96-98`)
- **D003-D2-E014 `[E]` — retry/failure behavior.** Every HTTP call uses at most three attempts with exponential backoff and jitter; exhausting a timeout returns `TimeoutError`, and infinite retry is forbidden. (`D2:99-100,228`)
- **D003-D2-E015 `[E]` — runtime adapter.** New `PpmAdapter { client: Arc<PpmClient> }` implements `RuntimeAdapter`, reports backend ID `"ppm"`, translates Anthropic `/v1/messages` requests to score requests, and wraps `PpmScore` as `RuntimeResponse`. (`D2:102-107`)
- **D003-D2-E016 `[E]` — permission boundary.** PPM is verifier-only: agents cannot select it for generation, only D-3’s consensus verifier may call it, and `RuntimeAdapter::is_verifier_only()` must return `true`. (`D2:108-110,220-224`)
- **D003-D2-E017 `[E]` — runtime/config defaults.** `RuntimeBackend` gains `Ppm(PpmAdapter)`; `[runtime.ppm]` defaults are endpoint `http://127.0.0.1:8101/v1/messages`, `timeout_secs=30`, `max_retries=3`, and model `huihui-ai/Qwen2.5-Math-PRM-7B` or a distilled-equivalent path. (`D2:112-117`)
- **D003-D2-E018 `[E]` — activation default.** `PerRoleRuntime::default()` leaves PPM disabled; activation requires both TOML opt-in and environment flag `ELAI_PPM_ENABLE=1`, preventing development sessions from accidentally requiring an offline second sidecar. (`D2:118-120,190-192`)
- **D003-D2-E019 `[E]` — remote boot action.** `start-ppm-sidecar.sh` uses the existing SSH/virtualenv server pattern, `CUDA_VISIBLE_DEVICES=1`, model path above, port `8101`, `--no-dola`, `--turboquant-bits 0`, `--no-steering`, and `--load-in-4bit`, and records log/PID files. (`D2:122-130`)
- **D003-D2-E020 `[E]` — placement fallback.** GPU1 is the normal PPM placement while GPU0 hosts policy; a documented single-GPU fallback serializes VRAM swapping and is supported but explicitly suboptimal. (`D2:131-133`)
- **D003-D2-E021 `[E]` — health workflow.** Health is a three-stage process check, TCP reachability check, then real `/v1/messages` curl request; the named probes are `pgrep`, `Test-NetConnection`, and `curl`. (`D2:134-137`)
- **D003-D2-E022 `[E]` — tunnel integration.** `start-tunnel.ps1` adds `127.0.0.1:8101 -> remote 127.0.0.1:8101`, leaves the existing `8100` forward unchanged, and documents the dual tunnel in `REMOTE-SIDECAR.md`. (`D2:139-143`)
- **D003-D2-E023 `[E]` — CLI surface.** `elai ppm health` wraps the three-stage health check, reports `OK` or a structured `PpmError`, and exits `0`/`1`. (`D2:145-150`)
- **D003-D2-E024 `[E]` — score event schema.** `PpmScoreEmitted` carries `template_id`, `step_index`, `score`, `confidence`, and `latency_ms`. (`D2:152-154`)
- **D003-D2-E025 `[E]` — unavailable event schema.** `PpmUnavailable` carries a string `reason` and `retry_count`, and fires only after verifier dispatch exhausts retries. (`D2:155-156`)
- **D003-D2-E026 `[E]` — schema lifecycle.** D2 scaffolds both PPM SSB variants behind `schema-v49` or a temporary marked scaffold, while D3 owns the atomic schema bump; D2 must not land an independent bump. (`D2:157-160,203-204`)
- **D003-D2-E027 `[E]` — dependency policy.** `reqwest` should already exist; no new production dependency is expected, and any missing retry-jitter helper should be hand-rolled in at most 30 LOC rather than add a crate. (`D2:162-166`)
- **D003-D2-E028 `[E]` — unit evidence.** Required client tests cover mock happy path, transient `502` success on the third attempt, persistent `504` to `TimeoutError`, malformed body to `ParseError`, and a deterministic fixed-input prompt snapshot. (`D2:168-176`)
- **D003-D2-E029 `[E]` — integration evidence.** The adapter integration test runs through `RuntimeAdapter` against a local axum mock, verifies score round-trip, and asserts `is_verifier_only() == true`. (`D2:178-182`)
- **D003-D2-E030 `[E]` — recorded decisions.** `D-2-decisions.md` must justify the same-family/different-base PPM choice as an acknowledged uncertainty, opt-in activation, and three-attempt jittered backoff. (`D2:184-195`)
- **D003-D2-E031 `[E]` — reachability.** D2 is reachable from D3’s `VerificationResult.ppm`, `elai ppm health`, `ELAI_PPM_ENABLE`, and `[runtime.ppm]`. (`D2:197-219`)
- **D003-D2-E032 `[E]` — prohibited fallbacks.** `PpmClient` must have no same-model policy fallback; an empty response must return `ParseError` rather than panic; the default endpoint must remain `8101`, not collide with policy `8100`; retry must remain bounded. (`D2:220-228`)
- **D003-D2-E033 `[E]` — observable evidence contract.** With enable flag set, sidecar booted, and health invoked, D2 expects health exit `0`, a real curl response, boot/tunnel/config artifacts, and—before D3 consensus wiring—zero SSB rows. (`D2:229-235`)
- **D003-D2-E034 `[E]` — deterministic done probes.** The plan enumerates source-symbol searches, port/CLI/health checks, workspace build and scoped tests, a negative same-model-fallback search expecting zero hits, and existence of all three paper interfaces. (`D2:237-277`)
- **D003-D2-E035 `[E]` — base acceptance gate.** Done requires client/adapter compilation, mock integration, verifier-only enforcement, backend/config loading, scaffolded SSB variants, working remote boot and dual tunnel, opt-in behavior, zero new workspace errors, all scoped tests, and at least six new PPM tests. (`D2:279-295`)
- **D003-D2-E036 `[E]` — hardware smoke gate.** Before the hardware-heavy test, chat must flag the boot; both sidecars must stay healthy together for at least five minutes without OOM and leave at least 6 GB GPU memory free. (`D2:296-299`)
- **D003-D2-E037 `[E]` — score-quality smoke gate.** A known-correct step must score at least `0.7`, a known-wrong step at most `0.3`, and the margin must exceed `0.3`. (`D2:300-302`)
- **D003-D2-E038 `[E]` — outage smoke gate.** With PPM enabled but offline, `PpmUnavailable` must be journaled and the reasoning step must not auto-pass; final consensus behavior is delegated to D3. (`D2:303-305`)
- **D003-D2-E039 `[E]` — documentary done gates.** All paper interfaces, the three-decision record, and post-`AUDITING` memory artifact `project_d2_ppm_sidecar.md` are required. (`D2:306-309`)
- **D003-D2-E040 `[E]` — code/config surfaces.** The plan adds client, adapter, CLI health, boot script, integration test, decision doc and paper interfaces; modifies role runtime, SSB, CLI commands, TOML, tunnel script and remote-sidecar documentation. (`D2:311-329`)
- **D003-D2-E041 `[E]` — non-goals.** D3 aggregation, D19 benchmarking, D28 retraining, and distilled-PPM training are excluded from D2. (`D2:331-337`)
- **D003-D2-E042 `[E]` — next dependency.** D3 is the immediate consumer; D19 is described as able to proceed alongside D3 because their write surfaces are disjoint. (`D2:339-343`)

### D3 — typed verification aggregation and consensus

- **D003-D3-E001 `[E]` — capability.** D3 aggregates deterministic, PPM, intrinsic GG, and cross-context signals into `VerificationResult`, with a stated baseline requirement of at least two of four concurring before template promotion. (`D3:1-10`)
- **D003-D3-E002 `[E]` — prerequisites/concurrency.** Estimated effort is 1–2 sessions; D2 and D1 are prerequisites; D5 is declared parallel-safe despite consuming the result, while D4 is not parallel-safe because it monitors D3 rejection/disagreement signals. (`D3:11-17`)
- **D003-D3-E003 `[E]` — intrinsic integration.** Existing scattered `mean_logprob`, `entropy_max`, and rolling-Jaccard `novelty` metrics are consolidated into `IntrinsicSignals`. (`D3:34-37`)
- **D003-D3-E004 `[E]` — cross-context behavior.** A claim runs in two independent context windows with different prior-step ordering; agreement is treated as robustness and disagreement as a flag. (`D3:39-42`)
- **D003-D3-E005 `[E]` — foundations.** D3 reuses deterministic dispatch hooks, D2’s client/adapter, Phase-4G GG fields, and D1’s per-template `verifier_required`. (`D3:44-49`)
- **D003-D3-E006 `[E]` — research gate.** `vstar-2402.06457-interface.md` and `ccv-2603.21454-interface.md` must exist before `TESTING_FIRST`. (`D3:69-76`)
- **D003-D3-E007 `[E]` — aggregate schema.** `VerificationResult` has optional `deterministic`, `ppm`, `intrinsic`, and `cross_context` fields plus `consensus_rule` and final `verdict`. (`D3:80-85`)
- **D003-D3-E008 `[E]` — deterministic schema.** `DeterministicCheck` contains `kind`, `passed`, and optional `stderr_snippet`; `DeterministicKind` is the closed set `Compile`, `Test`, `Lint`, `Schema`, `RegistryCheck`. (`D3:86-88`)
- **D003-D3-E009 `[E]` — intrinsic schema.** `IntrinsicSignals` contains `mean_logprob`, `entropy_max`, and `novelty`, all with ranges required to be documented in the struct. (`D3:89-90`)
- **D003-D3-E010 `[E]` — cross-context schema/default.** `CrossContextAgreement` stores two Blake3 output hashes, Jaccard similarity, and Boolean agreement; the default agreement threshold is `0.7` and lives in L3-locked `[verification]` config. (`D3:91-94`)
- **D003-D3-E011 `[E]` — rule schema.** `ConsensusRule` is a closed, L3-locked enum: `AtLeastTwo`, `AtLeastThree`, `AtLeastN(u8)`, or `RequireAll(Vec<SignalKind>)`. (`D3:95-96`)
- **D003-D3-E012 `[E]` — signal/verdict schemas.** The initial `SignalKind` set is `Deterministic`, `Ppm`, `Intrinsic`, `CrossContext`; `Verdict` is `Pass`, `Reject`, or `Inconclusive`. (`D3:97-98`)
- **D003-D3-E013 `[E]` — default computation.** `AtLeastTwo` counts deterministic `passed`, PPM score `>=0.7`, intrinsic `mean_logprob >= intrinsic_threshold`, and cross-context agreement; at least two passes means `Pass`, zero or one means `Reject`, while a missing required signal makes the result `Inconclusive`. (`D3:100-108`)
- **D003-D3-E014 `[E]` — alternative computations.** `AtLeastN(n)` uses the corresponding pass count; `RequireAll(kinds)` requires every named kind to be present and passing; the absent-config default is `AtLeastTwo`. (`D3:100-108,188-192`)
- **D003-D3-E015 `[E]` — runner schema.** `CrossContextRunner` contains `Arc<dyn Executor>` and a Jaccard threshold, and `run(&Claim, &OrchContext)` returns `CrossContextAgreement`. (`D3:110-115`)
- **D003-D3-E016 `[E]` — concurrency/determinism.** The runner starts two executor futures concurrently, shuffles prior steps, Blake3-hashes each output, then computes token-level 3-shingle Jaccard similarity. (`D3:114-118,121-122`)
- **D003-D3-E017 `[E]` — timeout recovery.** Each cross-context side is bounded to 30 seconds; timeout returns `Inconclusive` rather than panic or infinite wait. (`D3:119-122`)
- **D003-D3-E018 `[E]` — dispatch workflow.** Every claim-emitting candidate dispatches an applicable deterministic check, conditional PPM when `verifier_required==Prm`, PPM enablement and health all hold, always captures intrinsic signals, conditionally runs cross-context when the rule demands it, computes a verdict, and emits an SSB row. (`D3:124-135,228-230`)
- **D003-D3-E019 `[E]` — checker fallback.** Before D11 exists, deterministic dispatch uses current compile/test hooks and skips inapplicable checks; D11 later replaces this with typed registry dispatch. (`D3:127-129,331-336`)
- **D003-D3-E020 `[E]` — schema migration.** D3 atomically bumps `SCHEMA_VERSION 48→49` for both D2 PPM events and all D3 verifier events; an old v48 DB must emit a migration warning. (`D3:137-140,286-292`)
- **D003-D3-E021 `[E]` — pass/reject events.** `VerifierConsensusPass` contains template/candidate IDs, passing signals and rule string; `VerifierConsensusReject` contains IDs, `(SignalKind, reason)` failures and rule string. (`D3:141-145`)
- **D003-D3-E022 `[E]` — disagreement event.** `VerifierDisagreement` contains template ID and signal/status pairs and fires when at least one signal passes and at least one rejects, because disagreement is operationally distinct from rejection. (`D3:146-147,202-205`)
- **D003-D3-E023 `[E]` — cross-context event.** `CrossContextDispatched` records claim ID, both context hashes, Jaccard score, and agreement; every new variant needs payload arm, SQL serialization and round-trip proof. (`D3:148-150`)
- **D003-D3-E024 `[E]` — config defaults.** `[verification]` defaults are `consensus_rule="at_least_two"`, `intrinsic_logprob_threshold=-3.0`, `intrinsic_entropy_max=3.5`, `intrinsic_novelty_min=0.2`, and `cross_context_jaccard_threshold=0.7`; evaluator-side values are L3-locked. (`D3:152-157`)
- **D003-D3-E025 `[E]` — unit evidence.** Tests cover pass-count rules, required-signal absence, TOML deserialization, timeout-to-inconclusive, deterministic Jaccard on identical input, and disagreement emission. (`D3:159-168`)
- **D003-D3-E026 `[E]` — deceptive-case correction in source.** The plan explicitly notices that deterministic fail + PPM pass + intrinsic pass would pass `AtLeastTwo`; it revises the code-claim scenario to `RequireAll([Deterministic, Ppm])`, under which deterministic failure rejects. (`D3:170-184`)
- **D003-D3-E027 `[E]` — integration scenarios.** Integration uses mock executor/PPM/deterministic checker for ordinary pass, deceptive code rejection under `RequireAll`, and cross-context disagreement, asserting rows through an in-memory SSB writer. (`D3:170-186`)
- **D003-D3-E028 `[E]` — contract evidence.** Contract tests require public schema `49`, round-trips for every new event, stable `ConsensusRule` serde, and the `AtLeastTwo` default when config is absent. (`D3:188-192`)
- **D003-D3-E029 `[E]` — decision record.** D3 must justify the `AtLeastTwo` latency/gate tradeoff, distinct Blake3 deduplication versus Jaccard similarity, and separate disagreement telemetry for D4 drift monitoring. (`D3:194-205`)
- **D003-D3-E030 `[E]` — consumers.** D5 consumes `VerifierConsensusPass` counts; D4 consumes disagreement rate; every claim-emitting post-`EXECUTING` baby step reaches `dispatch_verifiers`. (`D3:207-232`)
- **D003-D3-E031 `[E]` — fail-closed constraints.** Missing required signals cannot be swallowed, cross-context cannot wait forever, every candidate must emit one result-shaped SSB row even on unavailable signals, and the rule must load from locked config rather than an in-code constant. (`D3:233-239`)
- **D003-D3-E032 `[E]` — observable proof.** A cloud deceptive candidate that PPM accepts but compilation rejects must yield a `VerifierConsensusReject` with deterministic stderr plus `PpmScoreEmitted`, and no `TemplatePromoted`. (`D3:240-246`)
- **D003-D3-E033 `[E]` — deterministic done probes.** Required probes cover schema/version/event symbols, aggregate enums, cross-context runner, config, task-runner dispatch, workspace/scoped/contract tests, a negative single-verifier bypass search, and both paper interfaces. (`D3:248-284`)
- **D003-D3-E034 `[E]` — acceptance volume.** Done requires at least ten table-driven consensus scenarios, at least twelve new orchestrator tests, contract tests, config defaults, one result per candidate, all six D2+D3 event round-trips, zero new build errors, and timeout safety. (`D3:286-303`)
- **D003-D3-E035 `[E]` — cloud gates.** Cloud proofs require deceptive rejection/no promotion, simple-math cross-context agreement `>=0.7`, and PPM-offline behavior yielding `Inconclusive` plus `PpmUnavailable` and either rejection or an explicit inconclusive flag according to rule. (`D3:304-310`)
- **D003-D3-E036 `[E]` — documentary gates/surfaces.** Paper interfaces, three-decision doc and `project_d3_verification_result.md` are required; implementation adds aggregate/runner/test/contract/docs and modifies SSB, task runner, config and module exports. (`D3:311-329`)
- **D003-D3-E037 `[E]` — non-goals/next.** D4 tripwires, D5 BAI, D11 typed checkers and D13 adversary are outside D3; D4 and D5 are its next disjoint consumers. (`D3:331-341`)

### D13 — cross-family Adversary for Mode B

- **D003-D13-E001 `[E]` — capability.** D13 adds `AgentRole::Adversary` for `GroundingMode::FalsifiabilityCrossFamily`; it must run on a different model family and return structured counter-evidence. (`D13:1-11`)
- **D003-D13-E002 `[E]` — prerequisites/concurrency.** Estimated effort is 2–3 sessions; D10 routing and D3 consensus are prerequisites; D14 and D15 are declared parallel-safe, although D14 shares the new cross-family infrastructure. (`D13:12-16`)
- **D003-D13-E003 `[E]` — routing priority.** Mode B is explicitly last-resort: most claims should route to constructive Mode A or observable Mode D first. (`D13:18-22`)
- **D003-D13-E004 `[E]` — hard verifier rules.** The adversary must be cross-family, its output must be structured with source-tier citations, a separate auditor—not the executor—decides from claim plus counter-evidence, and absence of an alternate family skips Mode B for Mode E with no self-critique. (`D13:23-31`)
- **D003-D13-E005 `[E]` — deployment fallback.** With local Qwen policy, cloud Sonnet is the default cross-family adversary; Qwen Math PPM is explicitly unsuitable, and cloud plus alternate-local unavailability results in Mode E. (`D13:33-37`)
- **D003-D13-E006 `[E]` — foundations.** The plan extends existing `AgentRole`, B37 per-role runtime, D3 verification, and D10’s `FalsifiabilityCrossFamily` branch. (`D13:39-44`)
- **D003-D13-E007 `[E]` — research gate.** Three paper interfaces—Self-Correction Bench, Constitution or Collapse, and Constitutional AI in Small LLMs—must be present. (`D13:57-65`)
- **D003-D13-E008 `[E]` — role permission metadata.** `AgentRole::Adversary` gains metadata requiring its runtime family to be cross-family from the executor, enforced at spawn. (`D13:67-73`)
- **D003-D13-E009 `[E]` — config schema.** `AdversaryConfig` contains `runtime_role`, `allowed_families`, `max_response_tokens`, and a `CounterEvidenceSchema`. (`D13:75-79`)
- **D003-D13-E010 `[E]` — model-family schema.** `ModelFamily` enumerates `Qwen25Coder`, `Qwen25Math`, `AnthropicClaude`, `OpenAiGpt`, `DeepSeek`, and extensible `Other(String)`. (`D13:80-82`)
- **D003-D13-E011 `[E]` — family predicate.** `cross_family_check(executor, adversary)` must reject same-family pairs and force the caller toward Mode E. (`D13:83-85`)
- **D003-D13-E012 `[E]` — counter-evidence schema.** `CounterEvidence` contains a `CounterVerdict`, evidence points, source-tier citations, and confidence; verdicts are `ClaimRobust`, `ClaimWeak`, or `ClaimFalse`. (`D13:87-92`)
- **D003-D13-E013 `[E]` — evidence schema.** Each `EvidencePoint` has `EvidenceKind`, a `BoundedString<2048>`, and an optional citation; kinds are `ContradictoryExample`, `MissingPrecondition`, `LogicalGap`, and `AlternateInterpretation`. (`D13:92-95`)
- **D003-D13-E014 `[E]` — citation schema.** Citations are typed as `T1Arxiv { id }`, `T1Nature { doi }`, `T2OrgBlog { url }`, or `T3Other { url, note }`. (`D13:96-97`)
- **D003-D13-E015 `[E]` — agent surface.** New `.claude/agents/adversary.md` instructs the strongest falsehood-seeking critic, requires explicit `ClaimRobust` when no counter-evidence exists, forbids hedging, returns matching JSON, cites the two motivating papers, and runs on Sonnet by default. (`D13:99-109`)
- **D003-D13-E016 `[E]` — probe schema.** `AdversaryProbe` owns a runtime adapter, config, and SSB sender; `probe` is async and returns `Result<CounterEvidence, AdversaryError>`. (`D13:111-115`)
- **D003-D13-E017 `[E]` — probe workflow/failures.** Probe first checks alternate-family availability, then rejects same-family runtime, loads the agent prompt, invokes the adapter, parses and validates JSON, and finally emits `AdversaryVerdict`; the two preflight failures are `CrossFamilyUnavailable` and `SameFamily`. (`D13:114-123`)
- **D003-D13-E018 `[E]` — mode dispatch states.** In `FalsifiabilityCrossFamily`, unavailable family returns `ClaimResult::Refused` with reason `"no cross-family adversary"` and journals `CrossFamilyUnavailable`; robust maps to `Verified`, false to `Falsified`, weak to `Inconclusive`. (`D13:125-135`)
- **D003-D13-E019 `[E]` — consensus integration.** D3 gains `SignalKind::Adversary` and `VerificationResult.adversary`; `ClaimRobust` contributes pass, `ClaimFalse` reject, and `ClaimWeak` neither/inconclusive. (`D13:137-142`)
- **D003-D13-E020 `[E]` — event schemas/version.** Schema `58→59`; `AdversaryVerdict` stores claim, verdict, confidence, evidence count, runtime family and audit ID, while `CrossFamilyUnavailable` stores claim, executor family and reason. (`D13:144-149`)
- **D003-D13-E021 `[E]` — locked defaults.** `[adversary]` is L3-locked, defaults allowed families to Anthropic Claude, OpenAI GPT and DeepSeek, and caps response at `4096` tokens. (`D13:151-154`)
- **D003-D13-E022 `[E]` — unit evidence.** Tests require Qwen Coder + Claude to pass cross-family, Qwen Coder + Qwen Math to fail, JSON round-trip, valid arXiv ID parsing and malformed-ID `ParseError`. (`D13:156-162`)
- **D003-D13-E023 `[E]` — integration evidence.** Mock Sonnet/Qwen scenarios cover robust Tokio verification, false Rust-GC falsification, and no-cloud refusal plus `CrossFamilyUnavailable`. (`D13:164-172`)
- **D003-D13-E024 `[E]` — decisions.** The decision doc must justify cloud Sonnet and session cost throttling, typed instead of narrative evidence, and fail-closed Mode E instead of same-model critique. (`D13:174-183`)
- **D003-D13-E025 `[E]` — reachability.** The feature is reachable from D10 dispatch, D3 consensus, and per-Mode-B spawning of the agent definition. (`D13:185-206`)
- **D003-D13-E026 `[E]` — schema/fallback bans.** Same-family must emit unavailability, malformed or freeform output must be an `AdversaryError`/`ParseError` rather than text fallback, allowed families must come from TOML, and structured schema validation is mandatory. (`D13:207-213`)
- **D003-D13-E027 `[E]` — observable proof.** The synthetic false Rust-GC claim must produce `ModeSelected(mode=FalsifiabilityCrossFamily)`, one `AdversaryVerdict(ClaimFalse)` with at least two evidence points, a falsified result, agent file, and config section. (`D13:214-221`)
- **D003-D13-E028 `[E]` — deterministic done probes.** Probes assert schema 59, event/types/role/signal/config symbols, agent and paper files, workspace build, and scoped adversary tests. (`D13:223-256`)
- **D003-D13-E029 `[E]` — acceptance gate.** Done requires both event round-trips, at least ten tests, role spawn enforcement, schema round-trip, config loading, D3 signal integration, zero build errors, three cloud branches, all paper interfaces and the decision doc. (`D13:258-280`)
- **D003-D13-E030 `[E]` — code/config surfaces.** D13 adds adversary Rust/agent/test/decision/paper files and modifies role, SSB, grounding mode, verification result, config and module export surfaces. (`D13:282-298`)
- **D003-D13-E031 `[E]` — non-goals/lifecycle.** D14 probing is separate; a multi-adversary ensemble and critique caching are deferred, so the first version performs one fresh probe per claim. (`D13:300-305`)
- **D003-D13-E032 `[E]` — next dependency.** D14 reuses this infrastructure and D23 later gates D14. (`D13:307-310`)

### D16 — UniCR-style conformal refusal

- **D003-D16-E001 `[E]` — capability/claim.** D16 implements a Mode-E refusal controller presented as a distribution-free formal guarantee that refusal risk remains within user epsilon under distribution shift. (`D16:1-8`)
- **D003-D16-E002 `[E]` — prerequisites/concurrency.** Estimated effort is 3–4 sessions; D9 frozen-surface enforcement and D10 Mode-E slot are prerequisites; D17 and D22 are declared parallel-safe. (`D16:9-14`)
- **D003-D16-E003 `[E]` — risk definition/default.** The calibration set records claim/truth-availability/confidence; the plan defines empirical risk as refusing when truth exists plus accepting when truth does not, with default `epsilon=0.1`. (`D16:18-24`)
- **D003-D16-E004 `[E]` — refusal lifecycle.** Tau updates nightly under L3 control, and a claim is refused when a GG-signal composite confidence falls below tau. (`D16:25-28`)
- **D003-D16-E005 `[E]` — foundations.** D16 consumes D3 `IntrinsicSignals`, D9 locked keys, and D10 `GroundingMode::Refuse`. (`D16:33-37`)
- **D003-D16-E006 `[E]` — research gate.** UniCR, Uncertainty Outside Parametric Knowledge, and Rewarding Doubt interfaces must be ingested. (`D16:47-55`)
- **D003-D16-E007 `[E]` — tuple schema.** `CalibrationTuple` stores `claim_id`, `ground_truth_available`, `model_confidence`, `claim_type`, `claim_outcome`, and `recorded_at`; outcomes are `Verified`, `Falsified`, `Refused`, `NoCheck`. (`D16:57-65`)
- **D003-D16-E008 `[E]` — corpus schema/lifecycle.** `CalibrationCorpus` contains tuple vector and storage path; `load` walks `path/*.toml`, `append` writes `{claim_id}.toml`, and `size` reports tuple count. (`D16:65-70`)
- **D003-D16-E009 `[E]` — tau computation.** `compute_tau` evaluates `r(tau)=P(refuse|truth_exists)+P(accept|truth_doesnt_exist)`, selects the minimal candidate satisfying `r<=epsilon` by binary search over a `0.01` grid in `[0,1]`, and returns `InsufficientData` below 500 tuples. (`D16:72-81`)
- **D003-D16-E010 `[E]` — updater schema/action.** `TauUpdater` holds storage path, epsilon, and current tau; nightly update loads the corpus, recomputes tau, writes `.elai_cc/calibration/tau.txt` with timestamp, and emits `CalibrationUpdate`. (`D16:83-92`)
- **D003-D16-E011 `[E]` — permission/integrity.** `TauUpdater`, `compute_tau`, weights/config, and `tau.txt` are L3-protected; only the updater may write the threshold, preventing agent-side bias. (`D16:83-92,133-138,180-187`)
- **D003-D16-E012 `[E]` — confidence function.** `refusal_decision` computes `w1*mean_logprob + w2*(-entropy_max) + w3*novelty` using locked TOML weights, and returns true exactly when the composite is below tau. (`D16:94-99`)
- **D003-D16-E013 `[E]` — routing integration.** D10’s `intrinsic_signals_below_threshold` delegates to `refusal_decision`; `ctx.unicr_tau` loads from `tau.txt` at startup. (`D16:101-105`)
- **D003-D16-E014 `[E]` — data growth.** After each claim resolution, `task_runner` appends a tuple, derives truth availability from checker output, records decision-time composite confidence, and keeps the hook L1A/data-only with no LLM judgment. (`D16:107-111`)
- **D003-D16-E015 `[E]` — status CLI.** `elai calibration status` prints corpus size, current tau, update timestamp, epsilon, and empirical refusal rate over the last N sessions; it is informational and always exits `0`. (`D16:113-117`)
- **D003-D16-E016 `[E]` — update CLI/authorization.** `elai calibration update` manually invokes nightly logic; CI invocation requires commit tag `L3_OVERRIDE:calibration_update`, while development invocation is unrestricted. (`D16:119-124`)
- **D003-D16-E017 `[E]` — refusal event.** `RefusalDecision` stores claim ID, composite confidence, tau, and Boolean refused. (`D16:126-128`)
- **D003-D16-E018 `[E]` — update event/version.** `CalibrationUpdate` stores tuple count, new/old tau, epsilon and audit ID; both events round-trip under schema `61→62`. (`D16:126-131`)
- **D003-D16-E019 `[E]` — config defaults.** `[conformal]` defaults: `epsilon=0.1`, `min_corpus_size=500`, weights `{mean_logprob=0.4, entropy=-0.3, novelty=0.3}`, tau path `.elai_cc/calibration/tau.txt`, corpus path `.elai_cc/calibration/corpus/`. (`D16:133-138`)
- **D003-D16-E020 `[E]` — unit/idempotency evidence.** Tests cover known-split tau, insufficient data, at least eight threshold cases, and identical-corpus nightly-update idempotency. (`D16:140-145`)
- **D003-D16-E021 `[E]` — integration evidence.** A synthesized 600-tuple corpus must produce tau and an empirical refusal rate asserted within `[tau-epsilon, tau+epsilon]`. (`D16:146-150`)
- **D003-D16-E022 `[E]` — decisions.** The decision doc records epsilon `0.1`, minimum `500` based on the cited finite-sample bound, and why composite GG evidence is preferred to raw mean log probability. (`D16:152-159`)
- **D003-D16-E023 `[E]` — reachability.** D16 is reachable from the Mode-E decision point, a nightly CI/cron updater, and status/update CLI surfaces. (`D16:161-179`)
- **D003-D16-E024 `[E]` — fail-closed constraints.** No small-sample approximation is allowed, every update must journal, refusal must use the weighted composite, and direct writes to `tau.txt` are forbidden. (`D16:180-187`)
- **D003-D16-E025 `[E]` — observable proof.** A 600-tuple nightly update must emit `CalibrationUpdate`, persist tau and at least 600 tuple files, and cause a later low-confidence cloud claim to emit `RefusalDecision(refused=true)`. (`D16:188-193`)
- **D003-D16-E026 `[E]` — deterministic done probes.** The plan checks schema/event/type/config/lock symbols, build/tests, and three paper interfaces. (`D16:195-210`)
- **D003-D16-E027 `[E]` — acceptance gate.** Done requires schema 62, two event round-trips, corpus load/append, exact/insufficient-data/idempotency tests, lock registration, config load, at least twelve tests, and zero new workspace errors. (`D16:212-224`)
- **D003-D16-E028 `[E]` — cloud branches.** Low confidence must route to Mode E and journal refusal; high confidence must choose another mode with no refusal row in that branch; manual update with override must journal and update the file. (`D16:225-230`)
- **D003-D16-E029 `[E]` — documentary/surface gates.** Three paper interfaces and the decision doc are required; the code adds calibration and CLI modules/tests/docs and modifies SSB, route selection, task runner, config, locked-key registry, CLI commands, and exports. (`D16:231-252`)
- **D003-D16-E030 `[E]` — non-goals.** Distribution-shift validation is D22, generic bootstrap is deferred to D30, and per-family/multi-domain tau is deferred unless D30 establishes need. (`D16:254-259`)
- **D003-D16-E031 `[E]` — downstream.** D17 consumes refusal events; D22 validates shifted behavior. (`D16:261-264`)

### D17 — refusal-rate SLA monitoring

- **D003-D17-E001 `[E]` — metric/capability.** D17 computes per-session `refusal_rate=refusals/total_claims`; below `5%` means likely confabulation, above `50%` means checker misconfiguration or an overly hard task suite. (`D17:1-7`)
- **D003-D17-E002 `[E]` — prerequisites/concurrency.** Estimated effort is one session; D16 refusal events are required, and D18 is declared parallel-safe. (`D17:8-10`)
- **D003-D17-E003 `[E]` — default versus research uncertainty.** The design uses broad inclusive healthy band `5–50%` while treating `10–30%` as an unvalidated guess to be tuned from actual session statistics. (`D17:12-20,87-92`)
- **D003-D17-E004 `[E]` — foundations/UI dependency.** It aggregates D16 `RefusalDecision` rows through `SsbViewer`; D27 later renders the gauge. (`D17:22-25`)
- **D003-D17-E005 `[E]` — monitor schema.** `RefusalRateMonitor` stores a minute window, minimum/maximum healthy rates, and SSB DB path. (`D17:34-39`)
- **D003-D17-E006 `[E]` — snapshot schema.** `RefusalRateSnapshot` stores session ID, window start, refusal and total counts, rate, and `HealthBand`; the band is `Below {rate,min}`, `Healthy`, or `Above {rate,max}`. (`D17:40-44`)
- **D003-D17-E007 `[E]` — query/workflow.** `snapshot` queries `RefusalDecision` rows for numerator and `MODE_SELECTED` rows for denominator, computes rate and band, and emits `RefusalSlaBreach` for either nonhealthy band. (`D17:46-52`)
- **D003-D17-E008 `[E]` — alert integration.** `alert_on_breach` emits the existing B13 Bubble-Up pattern with `AlertKind::RefusalRateOutOfBand` only when the band is not healthy. (`D17:54-57`)
- **D003-D17-E009 `[E]` — scheduling.** Task runner/orchestration invokes monitoring periodically; the prose says every N minutes with 60 default, while config separately defaults the check interval to 30 minutes and the window to 60. (`D17:59-62,69-72`)
- **D003-D17-E010 `[E]` — event/version.** Schema `62→63`; `RefusalSlaBreach` contains session ID, rate, healthy min/max, band, and total claims and needs round-trip proof. (`D17:64-67`)
- **D003-D17-E011 `[E]` — config/default/security.** L3-locked `[refusal_sla]` defaults are `enabled=true`, `window_size_minutes=60`, `healthy_min=0.05`, `healthy_max=0.50`, and `check_interval_minutes=30`. (`D17:69-72`)
- **D003-D17-E012 `[E]` — unit evidence.** Tests classify `0.02` below, `0.30` healthy, `0.60` above, verify synthetic SSB aggregation, and ensure healthy snapshots do not alert. (`D17:74-78`)
- **D003-D17-E013 `[E]` — integration evidence.** With 100 selected claims, 2 refusals alerts below, 60 alerts above, and 20 is healthy/no alert. (`D17:80-85`)
- **D003-D17-E014 `[E]` — decisions.** The plan records why the broad band is provisional, why scope is per session rather than global, and why breaches Bubble-Up for operator action instead of remaining silent. (`D17:87-94`)
- **D003-D17-E015 `[E]` — reachability.** D17 consumes D16, D10, SSB viewer/writer and B13 alerts; it is reached from periodic task-runner checks and D27 dashboard gauge. (`D17:96-111`)
- **D003-D17-E016 `[E]` — zero-denominator recovery.** `total_claims=0` is defined as `Healthy`, avoiding division by zero and suppressing an alert. (`D17:112-114`)
- **D003-D17-E017 `[E]` — idempotency.** Duplicate breach alerts in the same window must be suppressed using a window-end key. (`D17:115-116`)
- **D003-D17-E018 `[E]` — config integrity.** Thresholds must be loaded from TOML, not embedded in monitor code. (`D17:117`)
- **D003-D17-E019 `[E]` — observable proof.** A 2% synthetic session yields exactly one below-band breach plus Bubble-Up; D27’s gauge should be red. (`D17:118-121`)
- **D003-D17-E020 `[E]` — deterministic done probes.** Schema/event/monitor/config searches, workspace build and scoped tests are the deterministic checks. (`D17:123-132`)
- **D003-D17-E021 `[E]` — acceptance gate.** Done requires schema 63, event round-trip, correct aggregation/bands, Bubble-Up wiring, config loading, at least six tests, zero new errors, a 2% cloud breach/alert, and no event inside the healthy band. (`D17:134-149`)
- **D003-D17-E022 `[E]` — surfaces.** D17 adds monitor, integration test and decision doc; modifies SSB, task runner, config and module exports. (`D17:151-160`)
- **D003-D17-E023 `[E]` — non-goals.** Dashboard rendering, per-family band tuning, and long-term trend reporting are excluded; the latter two are explicitly deferred. (`D17:162-166`)
- **D003-D17-E024 `[E]` — next.** D18 completes Wave 2 after D17. (`D17:168-170`)

### D19 — adversarial PRMBench evaluation

- **D003-D19-E001 `[E]` — capability.** D19 evaluates D2’s PPM on PRMBench Deception Resistance and Circular Logic, reports per category, and frames the result as a conservative promotion gate. (`D19:1-8`)
- **D003-D19-E002 `[E]` — prerequisites/compute/concurrency.** Estimated effort is two sessions; D2 and D3 are prerequisites; it needs the remote sidecar but not RunPod and is parallel-safe with D20–D25. (`D19:9-12`)
- **D003-D19-E003 `[E]` — baseline context.** The context cites 60–65% for top PRMs in the two weak categories and distinguishes discriminative, LLM-judge, and ThinkPRM evidence. (`D19:14-22`)
- **D003-D19-E004 `[E]` — failure decision.** Below-baseline evidence permits either accepting the limitation under D3 consensus or deferring promotion until D28 adversarial farming improves the PPM. (`D19:24-28`)
- **D003-D19-E005 `[E]` — research gate.** `prm-lessons-2501.07301-interface.md` and `thinkprm-2504.16828-interface.md` must be ingested. (`D19:41-47`)
- **D003-D19-E006 `[E]` — fixture lifecycle/compliance.** The public HuggingFace dataset license must be checked; fixtures are filtered to the two initial categories and stored as category/task JSON. (`D19:49-58`)
- **D003-D19-E007 `[E]` — fixture schema/volume.** Each fixture stores problem, step history, candidate step, and correct/incorrect label; bootstrap volume is at least 100 per category, 200 total. (`D19:56-59`)
- **D003-D19-E008 `[E]` — runner schema.** `PRMBenchRunner` owns PPM plus fixture/output paths; `PRMBenchReport` stores category, counts, false positives/negatives, accuracy/precision/recall, baseline, and `GatingVerdict`. (`D19:61-68`)
- **D003-D19-E009 `[E]` — verdict schema.** `GatingVerdict` is `Pass`, `FailBelowBaseline`, or `InsufficientData`. (`D19:69`)
- **D003-D19-E010 `[E]` — runner workflow.** `run_category` walks fixtures, calls `score_step`, compares ground truth, accumulates counters and metrics, then compares against a configured baseline. (`D19:70-75`)
- **D003-D19-E011 `[E]` — CLI behavior.** `elai bench prmbench` evaluates all categories and exits `0` on pass, `1` on below-baseline failure. (`D19:77-80`)
- **D003-D19-E012 `[E]` — shared benchmark events.** Schema `64→65` adds reusable `BenchmarkStarted {name,fixture_count,audit_id}` and `BenchmarkResult {name,accuracy,baseline,verdict,audit_id}` with round-trip tests for D19–D25. (`D19:82-86`)
- **D003-D19-E013 `[E]` — CI integration.** `bench-prmbench.yml` runs on push, PR and weekly cron, boots the remote PPM when credentials allow, invokes the CLI, and publishes a report artifact. (`D19:88-92`)
- **D003-D19-E014 `[E]` — nonblocking failure.** Until PPM becomes production-gating, `FailBelowBaseline` warns but does not block merge because D3 treats PPM as one consensus signal. (`D19:93-94,96-103`)
- **D003-D19-E015 `[E]` — configurable threshold.** The 35% threshold must come from TOML or CLI arguments rather than be hardcoded. (`D19:120-123`)
- **D003-D19-E016 `[E]` — timeout fallback.** A PPM timeout marks the affected task `InsufficientData` and the category run continues. (`D19:120-121`)
- **D003-D19-E017 `[E]` — evidence integrity.** Every report must expose its baseline and verdict; silent pass is forbidden. (`D19:122`)
- **D003-D19-E018 `[E]` — observable proof.** A category CLI run produces one start row, one result row, and `.elai_cc/benchmarks/prmbench/reports/{date}-{category}.json`. (`D19:124-127`)
- **D003-D19-E019 `[E]` — deterministic done probes.** Checks cover schema/events/runner, at least 200 fixtures, CI workflow, build/tests, and both paper interfaces. (`D19:129-142`)
- **D003-D19-E020 `[E]` — acceptance gate.** Done requires schema/event round-trips, compiling runner/CLI/workflow, 200+ fixtures, at least six tests, zero new errors, persisted reports, paper interfaces and the decision doc. (`D19:144-160`)
- **D003-D19-E021 `[E]` — numeric cloud gates.** Deception Resistance passes at `>=35%`; Circular Logic passes at `>=30%`; lower results are reported as informational `FailBelowBaseline`. (`D19:154-158`)
- **D003-D19-E022 `[E]` — surfaces.** D19 adds runner, CLI, 200+ fixtures, CI workflow, decision and paper docs; modifies SSB, CLI command routing and exports. (`D19:162-175`)
- **D003-D19-E023 `[E]` — non-goals.** Other PRMBench categories, ThinkPRM integration, and the D28 adversarial retraining loop are deferred/excluded. (`D19:177-181`)
- **D003-D19-E024 `[E]` — next.** D20 follows. (`D19:183-185`)

### D25 — determinism and archive-tamper suite

- **D003-D25-E001 `[E]` — capability/gate.** D25 defines ten archive/determinism scenarios; applicable tamper must be caught within five seconds, a clean repository exits `0`, and reverting tamper restores clean verification. (`D25:1-8`)
- **D003-D25-E002 `[E]` — dependencies/concurrency.** D8 archive verification and C27 `DeterminismDrift` are prerequisites; D25 is parallel-safe with D19–D24. (`D25:6-8`)
- **D003-D25-E003 `[E]` — scenario 1.** Deleting a middle `ArchiveEntry` must produce `BrokenParent` on the following entry. (`D25:15-16`)
- **D003-D25-E004 `[E]` — scenario 2.** Replacing `parent_hash` with a nonexistent hash must produce `BrokenParent`. (`D25:17`)
- **D003-D25-E005 `[E]` — scenario 3.** Editing `template_body` without recomputing `self_hash` must produce `HashMismatch`. (`D25:18`)
- **D003-D25-E006 `[E]` — scenario 4.** Reusing one `self_hash` for two entries must produce `DuplicateHash`. (`D25:19`)
- **D003-D25-E007 `[E]` — scenario 5.** A malformed JSON line must produce `MalformedJson`. (`D25:20`)
- **D003-D25-E008 `[E]` — scenario 6.** Two `elai /task` runs with the same `ELAI_SEED` but divergent `action_type` sequences must emit `DeterminismDrift`. (`D25:21-22`)
- **D003-D25-E009 `[E]` — scenario 7/control.** Same-seed identical action sequences must emit zero `DeterminismDrift` rows. (`D25:23`)
- **D003-D25-E010 `[E]` — scenario 8.** Rewriting Blake3 content-hash bytes as SHA-256 on disk must produce `HashMismatch`. (`D25:24-25`)
- **D003-D25-E011 `[E]` — scenario 9/recovery.** Restoring a tampered archive from backup must make verification exit `0` again, proving recovery/idempotency. (`D25:26,48-52`)
- **D003-D25-E012 `[E]` — scenario 10/full collection.** A two-line tamper at lines 3 and 7 must report two errors when verification runs in full-collect mode, exercising continue versus fail-fast semantics. (`D25:27,48-53`)
- **D003-D25-E013 `[E]` — fixture layout.** `.elai_cc/benchmarks/tamper/scenarios/` contains ten directories, each with setup script and expected verdict; scripts generate clean state, alter it, and invoke verification. (`D25:34-38`)
- **D003-D25-E014 `[E]` — runner workflow.** `TamperRunner` executes setup and `elai archive verify`, records exit/error types, and compares them to the expected verdict. (`D25:40-43`)
- **D003-D25-E015 `[E]` — CLI behavior.** `elai bench tamper` runs all ten and exits `0` only when every scenario matches expectation. (`D25:45-46`)
- **D003-D25-E016 `[E]` — produced/consumed interfaces.** D25 produces `TamperRunner`, `TamperReport`, ten scripts and CLI; consumes D8 CLI, C27 drift detection and `SsbWriter`; it is reachable from CLI and CI. (`D25:55-66`)
- **D003-D25-E017 `[E]` — constraints.** The contract forbids missing a scenario, false-positive failure on clean state, or failure to recover after revert. (`D25:67-70`)
- **D003-D25-E018 `[E]` — observability.** One full run expects ten `BenchmarkStarted`, ten `BenchmarkResult`, and a report under `.elai_cc/benchmarks/tamper/reports/`. (`D25:71-74`)
- **D003-D25-E019 `[E]` — deterministic check.** Static checks require exactly ten scenario entries and a clean workspace compile. (`D25:76-82`)
- **D003-D25-E020 `[E]` — acceptance gate.** Done requires all directories, compiling runner, all-match exit semantics, zero build errors, every applicable tamper within five seconds, clean exit `0`, recovery after revert, and the decision doc. (`D25:84-93`)
- **D003-D25-E021 `[E]` — surfaces.** D25 adds runner, CLI, `{1..10}/setup.sh`, `expected.toml`, and decision doc and modifies command routing. (`D25:95-101`)
- **D003-D25-E022 `[E]` — non-goals.** Git cryptographic signing and multiparty signing are deferred. (`D25:103-107`)
- **D003-D25-E023 `[E]` — lifecycle.** D25 declares Wave 3 complete and hands off all-feature dashboard work to D26–D27. (`D25:109-111`)

### D30 — generic-corpus calibration bootstrap

- **D003-D30-E001 `[E]` — status/objective.** D30 is explicitly `DEFERRED`; it spikes a generic-corpus warm start because D16 needs 500 calibration tuples and early sessions cannot supply them. (`D30:1-10`)
- **D003-D30-E002 `[E]` — dependencies/concurrency.** Unblock requires D16 and a specific public HuggingFace corpus; D30 is parallel-safe with D28 and D29. (`D30:7-11`)
- **D003-D30-E003 `[E]` — hypothesis/lifecycle.** Candidate bootstrap data begins with generic QA/factual calibration, then local per-project data refines the threshold as sessions accumulate. (`D30:13-18`)
- **D003-D30-E004 `[E]` — risk/fallback.** Distribution mismatch between generic QA and ELAI code/reasoning may miscalibrate tau; nightly D16 updates limit bootstrap to starting-point status. (`D30:20-23`)
- **D003-D30-E005 `[E]` — corpus candidates.** Candidate sources are TruthfulQA, verified MMLU subsets, and HaluEval; the chosen source must publicly expose confidence labels and truth-availability flags and then receive paper ingestion. (`D30:32-40`)
- **D003-D30-E006 `[E]` — selection/data surfaces.** Unblocking adds `selection.md`, downloads `.elai_cc/calibration/bootstrap/{corpus}/`, and converts source records to D16 `CalibrationTuple` via a shim. (`D30:42-49`)
- **D003-D30-E007 `[E]` — bootstrap persistence.** D16 `compute_tau` runs on the bootstrap corpus and persists `.elai_cc/calibration/tau_bootstrap.txt`. (`D30:50-52`)
- **D003-D30-E008 `[E]` — switchover lifecycle.** Startup uses bootstrap tau while local corpus size is `<500` and switches to local-corpus tau once size is `>=500`. (`D30:53-54`)
- **D003-D30-E009 `[E]` — validation workflow.** D22’s runner tests bootstrap tau on synthetic ELAI claims and requires held-out empirical refusal behavior within epsilon; outside epsilon produces a distribution-mismatch Path-B report. (`D30:56-60`)
- **D003-D30-E010 `[E]` — closure alternatives.** Path A productizes bootstrap selection when it improves cold start; Path B closes with a concrete shifted-distribution blocker greater than epsilon. (`D30:61-66`)
- **D003-D30-E011 `[E]` — decision lifecycle.** `D-30-decisions.md` remains deferred and is drafted only at unblock. (`D30:68-69`)
- **D003-D30-E012 `[E]` — implemented symbol inventory.** The wiring contract names `warm_start_tau` at `calibration_bootstrap.rs:346`, `load_corpus` at `:292`, `CalibrationBootstrapCorpus` at `:189`, conversion at `:216`, `PublishedCalibrationRow` at `:148`, `BootstrapWarmStartOutcome` at `:239`, resampling at `:431`, CI helper at `:525`, and defaults at `:56/:65/:72`. (`D30:71-80`)
- **D003-D30-E013 `[E]` — event/runtime inventory.** It names `CalibrationBootstrapCompleted` plus payload at `ssb.rs:6746`, and `CalibrationBootstrapMode` at `runtime_config.rs:487`, gated by `ELAI_CALIBRATION_BOOTSTRAP_ENABLE`. (`D30:80-82`)
- **D003-D30-E014 `[E]` — deferred data legs.** Corpus download, `selection.md`, and `tau_bootstrap.txt` remain deferred because no corpus is selected. (`D30:82`)
- **D003-D30-E015 `[E]` — consumed interfaces.** Conversion feeds D16 `CalibrationCorpus`, warm start calls D16 `compute_tau`, mode comes from `CalibrationBootstrapMode::from_env` at runtime config line 499, D22 is deferred validation, and SSB writer journals completion. (`D30:83-88`)
- **D003-D30-E016 `[E]` — live reachability/default.** The environment-enabled path is wired at `task_runner.rs:5980` with an off-mode early-exit at `:16213`; default is wire-dark `Off`. (`D30:89-92`)
- **D003-D30-E017 `[E]` — safety constraints.** Warm start must enforce `DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE`, tolerate empty corpus without panic, always emit completion after a run, and use bootstrap tau below `PRODUCTION_FLOOR_TUPLES=500`. (`D30:93-97`)
- **D003-D30-E018 `[E]` — observable proof contract.** With enable flag and a present corpus, task execution should emit one completion row containing corpus slug and warm-start tau and write `tau_bootstrap.txt`. (`D30:97-100`)
- **D003-D30-E019 `[E]` — partial implementation status.** The plan explicitly says data/artifact/D22 legs are pending but the Rust dispatch path at the two task-runner anchors “IS wired and live.” (`D30:101-103`)
- **D003-D30-E020 `[E]` — deterministic-check status.** The entire deterministic-check section is `DEFERRED`; it supplies no executable check despite the named live Rust path. (`D30:105-107`)
- **D003-D30-E021 `[E]` — acceptance/unblock gate.** All acceptance boxes remain open; completion requires final corpus choice, chosen-corpus paper interface, persisted bootstrap tau, D22 Path A within epsilon or a concrete Path B mismatch, and decision doc. (`D30:109-117`)
- **D003-D30-E022 `[E]` — anticipated surfaces.** Remaining anticipated writes are corpus directory, selection document, bootstrap tau, D16 startup fallback modification, and decision doc. (`D30:119-126`)
- **D003-D30-E023 `[E]` — non-goals.** Custom corpus authoring and multi-corpus blending are deferred beyond D30. (`D30:128-132`)
- **D003-D30-E024 `[E]` — terminal sequencing.** The D plan completes only after D30 unblocks; D28–D30 otherwise operate independently and are prioritized by GPU availability and research value. (`D30:134-138`)

## Explicit cross-source interactions

- **D003-INT-E001 `[E]` — D2→D3 schema transaction.** D2 deliberately scaffolds PPM events and D3 atomically promotes all six D2/D3 event families under schema 49; neither plan supports a separately versioned D2 landing. (`D2:152-160`; `D3:137-150,207-220`)
- **D003-INT-E002 `[E]` — D2→D3 availability semantics.** D3 invokes PPM only for `VerifierKind::Prm` with enable flag and a healthy client; unavailable required PPM becomes an explicit non-pass/inconclusive path rather than auto-pass. (`D2:118-120,303-305`; `D3:124-135,304-310`)
- **D003-INT-E003 `[E]` — D3 fan-out.** D3’s pass event feeds D5, disagreement feeds D4, adversary extends its signal set in D13, and its intrinsic signals become the confidence substrate in D16. (`D3:207-232,331-341`; `D13:137-142`; `D16:33-37,171-175`)
- **D003-INT-E004 `[E]` — cross-family rule consistency.** D2’s PPM may be separate-base but D13 explicitly treats Qwen Coder and Qwen Math as the same family, so the PPM cannot serve as D13’s adversary. (`D2:26-32,184-189`; `D13:33-37,156-160`)
- **D003-INT-E005 `[E]` — D13→D16 degradation.** Mode-B inability returns a Mode-E refusal, but the actual conformal refusal mechanism and tau live in D16. (`D13:30-37,125-135`; `D16:94-105,176-179`)
- **D003-INT-E006 `[E]` — D16→D17 observability chain.** D16 emits per-claim `RefusalDecision`; D17 joins those to D10 `MODE_SELECTED` for per-session/window SLA and raises `RefusalSlaBreach` plus Bubble-Up. (`D16:126-131`; `D17:46-72,96-108`)
- **D003-INT-E007 `[E]` — D2→D19 benchmark chain.** D19 calls D2 `PpmClient` across 200 adversarial fixtures and converts score decisions into category metrics and reusable benchmark events. (`D2:83-100`; `D19:49-86`)
- **D003-INT-E008 `[E]` — D19→D25 shared observability.** D19 declares `BenchmarkStarted`/`BenchmarkResult` reusable through D25, and D25 expects ten of each without declaring another event/version. (`D19:82-86`; `D25:71-74`)
- **D003-INT-E009 `[E]` — D8/C27→D25 integrity join.** D25 simultaneously tests archive-chain error variants and same-seed action-sequence drift, then validates recovery to clean state. (`D25:1-27,55-74`)
- **D003-INT-E010 `[E]` — D16/D22→D30 cold start.** D30 converts published rows to D16 corpus, calls `compute_tau`, uses bootstrap only below 500 local tuples, and delegates shifted validation to D22. (`D30:42-66,71-103`)
- **D003-INT-E011 `[E]` — schema sequence.** Assigned plans explicitly claim version transitions 48→49 (D3), 58→59 (D13), 61→62 (D16), 62→63 (D17), and 64→65 (D19); omitted intervening versions belong to unassigned D plans and were not inspected. (`D3:137-140`; `D13:144-149`; `D16:126-131`; `D17:64-67`; `D19:82-86`)
- **D003-INT-E012 `[E]` — operator surfaces.** The chunk adds `elai ppm health`, `elai calibration status`, `elai calibration update`, `elai bench prmbench`, and `elai bench tamper`, with structured exit/status semantics rather than dashboard-only evidence. (`D2:145-150`; `D16:113-124`; `D19:77-80`; `D25:45-46`)
- **D003-INT-I001 `[I]` — end-to-end verification path.** The assigned-source composition is: opt-in PPM score → typed multi-signal result → optional cross-family adversary contribution → consensus-gated outcome; low-confidence or unavailable-independent-verifier branches reach calibrated refusal, whose rate is separately monitored. This is a synthesis, not a declared transaction boundary. (`D2:197-219`; `D3:124-150`; `D13:125-142`; `D16:94-105`; `D17:46-57`)
- **D003-INT-I002 `[I]` — evidence-plane composition.** SSB supplies per-step score/unavailability, per-candidate consensus/disagreement, per-claim adversary/refusal, per-window SLA, and per-benchmark integrity rows; together these form a traceable evidence plane, but no assigned source specifies a single correlation key spanning all record types. (`D2:152-160`; `D3:137-150`; `D13:144-149`; `D16:126-131`; `D17:64-67`; `D19:82-86`)

## Ambiguities, conflicts, and unclaimed coverage

- **D003-ISSUE-C001 `[C]` — PPM family wording.** D2’s objective calls the PPM a “different model family,” but the same source calls it Qwen-family/same-family-different-base and D13 explicitly rejects Qwen Coder + Qwen Math as same-family. Treat D2 as heterogeneous specialization/base, not a valid D13 cross-family verifier. (`D2:3-8,26-32,184-189`; `D13:33-37,156-160`)
- **D003-ISSUE-C002 `[C]` — D2 pre-D3 SSB expectation.** D2’s observable contract expects zero SSB rows until D3 wires consensus, while its offline cloud acceptance expects `PpmUnavailable` to be emitted. The intended scaffold-versus-runtime boundary is not resolved. (`D2:229-232,286-287,303-305`)
- **D003-ISSUE-A001 `[A]` — PPM authentication/privacy.** `[runtime.ppm]` is said to reuse per-role endpoint/auth, but the stanza lists no auth key and the plan defines no secret handling, request-data minimization, retention, or cloud privacy policy. (`D2:41-46,112-120`)
- **D003-ISSUE-A002 `[A]` — score parsing.** The response body shape and extraction rule for turning Anthropic-protocol output into one bounded float/confidence pair are not specified beyond `ParseError`. (`D2:85-100,102-107`)
- **D003-ISSUE-A003 `[A]` — retry determinism.** Backoff jitter is required but its seed, schedule and observability are unstated; only the attempt cap is deterministic. (`D2:99-100,193-195`)
- **D003-ISSUE-C003 `[C]` — default consensus versus deceptive defense.** D3’s default `AtLeastTwo` permits PPM+intrinsic to pass despite deterministic failure; the source itself corrects the test to code-specific `RequireAll`. Therefore “deterministic fail always blocks promotion” is false unless rule selection requires it. (`D3:100-108,170-184`)
- **D003-ISSUE-A004 `[A]` — required-signal semantics.** D3 says missing a “required” signal is inconclusive but does not define requiredness for `AtLeastTwo/Three/N`; only `RequireAll(kinds)` names required kinds explicitly. (`D3:100-108,233-239`)
- **D003-ISSUE-A005 `[A]` — intrinsic pass predicate.** Config exposes logprob, entropy and novelty thresholds, but the pass-count algorithm only names `mean_logprob >= intrinsic_threshold`; combination of all three fields is unspecified. (`D3:89-90,100-105,152-155`)
- **D003-ISSUE-A006 `[A]` — shuffled-context reproducibility.** D3 requires shuffled prior steps and a `SeededClock` but does not name a shuffle RNG/seed, so reproducibility of cross-context inputs is not closed. (`D3:110-122,222-227`)
- **D003-ISSUE-A007 `[A]` — disagreement verdict.** An integration scenario says cross-context disagreement yields `Inconclusive`, while generic count rules could yield pass or reject depending other signals; the governing rule for that scenario is not named. (`D3:100-108,170-186`)
- **D003-ISSUE-C004 `[C]` — ModelFamily predicate wording.** D13 says `cross_family_check` ensures different enum variants, yet its required test says two distinct variants `Qwen25Coder` and `Qwen25Math` must still compare same-family/false; the implementation needs an unstated higher-level family grouping. (`D13:80-85,156-160`)
- **D003-ISSUE-A008 `[A]` — adversary runtime failures.** Availability and same-family failures are mapped, but timeout, provider error, malformed JSON, citation validation failure, retry and cost-throttle mechanics are not mapped to claim state or Mode E. (`D13:111-135,174-183,207-213`)
- **D003-ISSUE-A009 `[A]` — adversary privacy/compliance.** Claims are sent to cloud Sonnet by default, but no consent, redaction, data residency, retention, PII, secret, or audit/compliance policy is defined. (`D13:33-37,99-109,164-177`)
- **D003-ISSUE-A010 `[A]` — auditor identity.** Context requires an auditor rather than executor to decide, but dispatch maps `CounterEvidence` directly to `ClaimResult` and does not name an auditor call/identity in that flow. (`D13:23-31,125-135`)
- **D003-ISSUE-C005 `[C]` — confidence weight sign/domain.** D16’s formula negates entropy and its configured entropy weight is already `-0.3`, producing a double-negative contribution if applied literally; mean log probability can also be negative while tau search is `[0,1]`. Normalization/sign convention is unstated. (`D16:72-99,133-138`)
- **D003-ISSUE-C006 `[C]` — risk versus refusal-rate assertion.** `compute_tau` bounds a two-term error risk, but integration asserts refusal rate lies around numeric tau; those are different quantities unless an unstated theorem/transformation equates them. (`D16:72-81,146-150`)
- **D003-ISSUE-A011 `[A]` — cold-start behavior.** D16 returns `InsufficientData` below 500 but does not define startup tau/refusal behavior in that state; D30 proposes the fallback but is deferred. (`D16:72-81,180-187,254-259`; `D30:1-10,50-54`)
- **D003-ISSUE-A012 `[A]` — tuple overwrite/idempotency.** Append writes `{claim_id}.toml`; repeated resolution of the same claim, collision/overwrite handling, atomic writes and corruption recovery are unspecified. (`D16:65-70,107-111`)
- **D003-ISSUE-C007 `[C]` — D17 schedule wording.** Periodic prose calls 60 minutes the default, while config sets a 60-minute window and a 30-minute check interval. The likely distinction is window versus cadence, but the phrase “every N minutes (60 default)” conflicts with the configured cadence. (`D17:59-72`)
- **D003-ISSUE-A013 `[A]` — SLA dedup key persistence.** Duplicate suppression uses a window-end key, but the snapshot/event schemas do not include window end or define persistence/restart behavior. (`D17:36-44,64-67,112-117`)
- **D003-ISSUE-A014 `[A]` — numerator/denominator alignment.** The plan does not state whether every `MODE_SELECTED` row can produce at most one `RefusalDecision`, how late rows/windows are aligned, or whether retried claims count once. (`D17:46-52`)
- **D003-ISSUE-C008 `[C]` — benchmark gate versus CI.** D19 calls the threshold a PPM promotion gate and CLI returns failure, yet CI explicitly warns and continues until an undefined production-gating transition. Current enforcement is informational, not a merge gate. (`D19:1-8,24-28,77-80,88-103`)
- **D003-ISSUE-A015 `[A]` — insufficient-data aggregation.** A timed-out task is called `InsufficientData`, but only report-level `GatingVerdict` is typed; minimum valid fixture count and how invalid tasks affect metrics/verdict are unspecified. (`D19:61-75,120-123`)
- **D003-ISSUE-A016 `[A]` — baseline config.** The threshold must come from TOML or args, but no TOML section, per-category keys, confidence interval, or precedence rule is defined. (`D19:74-80,120-123`)
- **D003-ISSUE-C009 `[C]` — “ten detected” wording.** D25 says all ten scenarios must be detected, but scenario 7 expects no drift and scenario 9 expects clean recovery; these are negative/recovery controls whose success is matching expectation, not detection. (`D25:15-27,45-46,67-74`)
- **D003-ISSUE-A017 `[A]` — five-second boundary.** The plan does not say whether setup, archive verify, drift comparison, or only detection latency is inside the five-second timer. (`D25:1-5,40-46,84-92`)
- **D003-ISSUE-A018 `[A]` — destructive fixture isolation.** Tamper scripts modify archives and restore backups, but temporary-copy isolation, crash cleanup and protection of a real project archive are not specified. (`D25:34-43,95-101`)
- **D003-ISSUE-A019 `[A]` — undeclared benchmark-event prerequisite.** D25 expects D19-defined benchmark events but lists only D8/C27 as prerequisites; behavior when D25 lands before schema 65 is unstated. (`D19:82-86`; `D25:6-8,55-74`)
- **D003-ISSUE-C010 `[C]` — D30 lifecycle status.** D30 is globally `DEFERRED` with all acceptance/checks deferred, yet its wiring contract claims concrete symbols and a Rust task-runner path are already wired/live. The source supports “partially wired, data/validation blocked,” not purely unimplemented or complete. (`D30:1-11,71-117`)
- **D003-ISSUE-A020 `[A]` — bootstrap reproducibility.** `bootstrap_resample` and a 90% CI helper are named as live symbols, but the plan gives no seed, iteration count, sampling policy or acceptance use. (`D30:71-80`)
- **D003-ISSUE-A021 `[A]` — bootstrap data governance.** Public availability is required, but dataset license compatibility, provenance record, download hash, update/version pin, privacy and deletion policy are unspecified. (`D30:32-49`)
- **D003-ISSUE-A022 `[A]` — switchover atomicity.** The `<500`/`>=500` rule is explicit, but concurrent corpus growth, startup caching, threshold-write atomicity and rollback are not. (`D30:50-54,89-103`)
- **D003-ISSUE-A023 `[A]` — no chunk-wide privacy feature.** Apart from local loopback endpoints, L3 locks, typed citations, dataset-license check and public-corpus requirement, none of the eight plans defines a dedicated PII, secrets, retention, telemetry export, user-consent, data residency or compliance lifecycle. (Coverage audit over `D2:1-343`; `D3:1-341`; `D13:1-310`; `D16:1-264`; `D17:1-170`; `D19:1-185`; `D25:1-111`; `D30:1-138`)
- **D003-ISSUE-A024 `[A]` — implementation status outside D30.** D2/D3/D13/D16/D17/D19/D25 contain prospective tasks and unchecked acceptance boxes but no explicit `Status:` line; this deep pass therefore does not independently claim `PLANNED`, shipped, wired, or done state for them. (`D2:279-309`; `D3:286-313`; `D13:258-280`; `D16:212-232`; `D17:134-149`; `D19:144-160`; `D25:84-93`)

## Summary-layer additions and corrections

The following deltas compare only against `04-D-SUMMARY_INDEX.discovery.md`. Deep-source atoms remain the evidence of record.

- **D003-DELTA-001 `[Δ~]` — D2 client kind.** The summary calls `PpmClient` a trait; D2 defines a concrete `struct PpmClient` plus `impl`, with `PpmAdapter` implementing `RuntimeAdapter`. (`D2:83-110`; summary `04-D...:46,524-528`)
- **D003-DELTA-002 `[Δ~]` — D2 family claim.** “Different family” must be qualified to Qwen same-family/different-base for D2; D13 does not accept it as a cross-family adversary. (`D2:26-32,184-189`; `D13:33-37,156-160`; summary `04-D...:42-50,580-583`)
- **D003-DELTA-003 `[Δ+]` — D2 prerequisites/config.** The summary omits C15, the disabled-by-default two-part activation, exact endpoint/timeout/retry/model defaults, verifier-only permission, single-GPU serialized fallback, three-stage health command, tunnel extension, error enum and fixed prompt. (`D2:9-16,81-150`)
- **D003-DELTA-004 `[Δ+]` — D2 events/schema/evidence.** Add exact `PpmScoreEmitted`/`PpmUnavailable` payloads, D3-owned schema-49 staging, retry tests, no-fallback bans, hardware endurance/free-memory gate, offline logging and documentary outputs. (`D2:152-235,279-309`)
- **D003-DELTA-005 `[Δ+]` — D3 complete schemas.** The summary’s five aggregate fields omit `verdict`, nested deterministic/intrinsic/cross-context records, all rule/signal/verdict enum values, thresholds, timeouts, concurrent two-context dispatch and exact event payloads. (`D3:80-168`)
- **D003-DELTA-006 `[Δ~]` — D3 deceptive gate.** A deterministic failure does not block under default `AtLeastTwo` when PPM and intrinsic both pass; D3 explicitly switches code claims to `RequireAll([Deterministic,Ppm])`. The summary’s unconditional deceptive-case rejection is too broad. (`D3:170-184`; summary `04-D...:53-63,589-590`)
- **D003-DELTA-007 `[Δ+]` — D3 operational contracts.** Add one result per candidate, explicit unavailable-signal behavior, disagreement as separate D4 input, L3-loaded rule, schema-49 migration warning, test volumes and the offline/cloud branches. (`D3:188-313`)
- **D003-DELTA-008 `[Δ~]` — D13 exact mode/events.** Exact route is `GroundingMode::FalsifiabilityCrossFamily`; exact Rust event variants are `AdversaryVerdict` and `CrossFamilyUnavailable`, not the summary’s uppercase display labels. (`D13:1-5,125-149`; summary `04-D...:195-210,542-552`)
- **D003-DELTA-009 `[Δ+]` — D13 typed evidence and states.** Add full model/counter-evidence/evidence/citation schemas, 2,048-byte evidence bound, JSON-only parser, three verdict-to-claim mappings, separate auditor constraint, 4,096-token locked config, source-tier validation and cost-throttling decision. (`D13:67-183`)
- **D003-DELTA-010 `[Δ+]` — D13 non-goals.** The summary omits single-fresh-probe scope, deferred ensembles/caching, exact tests, schema 59 and all acceptance/artifact gates. (`D13:185-305`)
- **D003-DELTA-011 `[Δ~]` — D16 tuple schema.** The persisted unit is keyed by `claim_id`, not raw `claim`, and also carries `claim_type`, `ClaimOutcome`, and timestamp; corpus records live under `.elai_cc/calibration/corpus/`. (`D16:57-70,133-138`; summary `04-D...:241-256,501-505`)
- **D003-DELTA-012 `[Δ+]` — D16 algorithm/defaults.** Add risk equation, minimal tau search on 0.01 grid, hard 500-record floor, exact weighted composite, config/paths, status/update CLI, L3 override rule, event payloads, schema 62 and idempotency requirement. (`D16:72-145`)
- **D003-DELTA-013 `[Δ~]` — D16 proof scope.** D16 itself tests a synthetic 600-tuple corpus; distribution-shift families are explicitly D22/out-of-scope. The summary conflates the D16 implementation smoke gate with D22 validation. (`D16:146-150,254-264`; summary `04-D...:253-256,601-603`)
- **D003-DELTA-014 `[Δ~]` — D17 event identifier.** The original uses `RefusalSlaBreach` plus `AlertKind::RefusalRateOutOfBand`, not `AUTO_FLAG`. (`D17:46-72`; summary `04-D...:258-269,542-552`)
- **D003-DELTA-015 `[Δ~]` — D17 window.** The original monitors a 60-minute window on a 30-minute configured cadence; it does not specify the summary’s “last 20 sessions” instrumentation. (`D17:36-72`; summary `04-D...:267-269,603-604`)
- **D003-DELTA-016 `[Δ+]` — D17 recovery/idempotency.** Add zero-claim-as-healthy, per-window alert deduplication, Bubble-Up operator notification, task-suite-too-hard upper-band cause, L3 configuration and exact test matrix. (`D17:36-121`)
- **D003-DELTA-017 `[Δ+]` — D19 corpus/runner.** Add license check, exact fixture schema, 100-per-category floor, complete report metrics/verdict enum, timeout-to-insufficient-data behavior, CLI exit codes, CI triggers/artifacts and event payloads. (`D19:49-127`)
- **D003-DELTA-018 `[Δ~]` — D19 enforcement.** The 35%/30% results currently warn in CI and do not block merge; production-gating activation is future/undefined. This qualifies the summary’s direct “gates PPM promotion” wording. (`D19:88-103,144-158`; summary `04-D...:285-296`)
- **D003-DELTA-019 `[Δ~]` — D19 baseline context.** The original cites 60–65% for top PRMs/~60% for Qwen context but deliberately uses 35% and 30% conservative product gates; “published baseline” is not the numeric acceptance threshold. (`D19:14-28,61-75,96-103,154-158`; summary `04-D...:288-296`)
- **D003-DELTA-020 `[Δ+]` — D25 scenarios.** The summary lists four of ten; add duplicate hash, malformed JSON, divergent same-seed sequence, identical-sequence control, recovery, and two-line full-collect cases with exact expected errors. (`D25:15-27`; summary `04-D...:360-376,611-612`)
- **D003-DELTA-021 `[Δ+]` — D25 runner/recovery.** Add directory/setup/expected layout, `TamperRunner`/report, all-match CLI exit, 10+10 benchmark rows, exact file surfaces, fail-fast-versus-continue purpose and deferred signing non-goals. (`D25:34-107`)
- **D003-DELTA-022 `[Δ~]` — D30 partial state.** Although globally deferred, the original now explicitly names implemented Rust symbols, environment-mode config, SSB event and live task-runner anchors; only corpus/data/artifact/D22 legs remain blocked. The summary’s blanket deferred portrayal is stale/incomplete. (`D30:71-117`; summary `04-D...:430-443,654-656`)
- **D003-DELTA-023 `[Δ+]` — D30 candidates/switchover.** Add TruthfulQA/MMLU/HaluEval candidates, required label properties, exact `<500` bootstrap to `>=500` local switch, wire-dark default, empty-corpus safety and completion event fields. (`D30:32-103`)
- **D003-DELTA-024 `[Δ~]` — D30 acceptance claim.** The assigned original does not require refined empirical risk to be no worse than bootstrap; it requires Path A cold-start improvement and within-epsilon D22 validation, or Path B concrete mismatch. The summary’s stronger “refined risk no worse” gate is unsupported by D30. (`D30:56-66,109-117`; summary `04-D...:438-443,615-616`)
- **D003-DELTA-025 `[Δ+]` — chunk-wide failure and evidence detail.** The deep plans add structured error enums, bounded retry, explicit inconclusive states, no-panic/zero-denominator handling, idempotent tau and alert behavior, CI warning semantics, artifact paths, schema transitions, test-count gates and negative searches that the summary did not inventory. (Atoms `D003-D2-E012`–`D003-D2-E039`, `D003-D3-E013`–`D003-D3-E035`, `D003-D16-E020`–`D003-D17-E021`, `D003-D19-E014`–`D003-D25-E020`)

## Completeness, zero-omission, and duplicate audit

- Assigned originals processed exactly once: **8/8**.
- Original-source coverage: **1,862/1,862 logical lines (100%)**, **88,049/88,049 bytes (100%)**.
- Manifest hash/count verification: **8/8 exact**, aggregate exact; mismatches **0**.
- Unassigned original plans opened: **0**.
- Explicit per-source atoms: D2 **42**, D3 **37**, D13 **32**, D16 **31**, D17 **24**, D19 **24**, D25 **23**, D30 **24** = **237**.
- Cross-source atoms: **14** total = **12 explicit + 2 inferred**.
- Issue atoms: **34** total = **10 conflicting + 24 ambiguous/absence**.
- Summary-delta atoms: **25** total = **13 additions + 12 corrections**.
- Total uniquely defined atomic IDs: **310**.
- Duplicate atomic IDs: **0**; missing sequence IDs inside each declared prefix: **0**.
- Category audit: capabilities/actions/surfaces **covered**; workflows/states **covered**; schemas/data/lifecycle **covered**; flags/defaults/config **covered**; integrations/events **covered**; permissions/security/privacy/compliance **covered, including explicit absence**; failure/fallback/retry/recovery **covered**; determinism/idempotency/concurrency **covered**; observability/evidence **covered**; tests/acceptance/done gates **covered**; constraints/non-goals **covered**; dependencies/status/deferred **covered**.
- Double-count audit: per-file atoms record source facts; interaction atoms record only joins; issue atoms record only unresolved qualifications; delta atoms compare summary coverage. These layers are intentionally not counted as additional source features.

- Canonical payload SHA-256 (all complete bytes above this bullet, including the preceding blank line): `fff2d9148a9e351272ececef8251a34f042eaa0bf93340e313187e407928f7fc`
