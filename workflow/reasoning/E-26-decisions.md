E-26 decisions — inference engines 2026 (SGLang adapter, XGrammar drop-in, Ollama deprecation)

Date: 2026-05-26
Session: s-25120 (completion-drive cluster, E-plan stage)
Plan: docs/thefinalglance/E-plan/E-26_inference_engines_2026.md
Source report: workflow/improvement-sweep-2026-05-19/reports/26-inference-engines-2026.md
Re-check report: workflow/sessions/s-25120/cross-check-E/E-26_inference_engines_2026-report.md (14/14 MISSING confirmed before this run)
Head SCHEMA_VERSION at completion-drive close: 151 (was 148 after E-3; E-26 atomic +3).

DECISION-A — SGLang chosen over vLLM as the cloud throughput tier.

Anchors:
- Source report Section "Engine landscape table" + "Why SGLang and NOT vLLM V1": vLLM V1 deprecated per-request custom logits processors and moved to batch-level global processors loaded at engine init (docs.vllm.ai/en/v0.8.1/getting_started/v1_user_guide.html). ELAI's DoLa + ETC + steering + TurboQuant LogitsProcessor chain cannot live inside vLLM V1 without a fork.
- arXiv:2509.25175 EasySteer (referenced in source report; INGEST flagged in grounding sweep) — the only working precedent for ELAI-style steering on vLLM is pinned to vLLM v0.17.1 and still missing CUDA graphs as of Mar 2026. "Pin to v0.17.1 to keep steering working" is a known anti-pattern.
- Counter-evidence: SGLang shows +29% total tok/s, +117% output tok/s, and 3x faster constrained decoding vs vLLM on H100 8B-class (particula.tech), AND SGLang ships a `CustomLogitProcessor` Python plugin with batch-aware per-request execution that is closer to ELAI's needs than vLLM V1's global batch-level processors (sgl-project/sglang#2291).

Outcome: HYBRID per source report net call. Local 12GB tier (RTX 3060) stays on custom sidecar — DoLa + ETC + TurboQuant cannot be replicated in vLLM V1 today, and llama.cpp/ExLlamaV3 do not solve ELAI's research-engine needs. Cloud 24GB tier (RTX 3090 pythondev) — new SGLang adapter behind B-37 PerRoleRuntime, gated by role: research/DoLa stays on custom sidecar, planner/critic/structured-JSON routes to SGLang+XGrammar via `ELAI_SGLANG_BASE_URL` env.

DECISION-B — Ollama backend DEPRECATED, not deleted.

Anchors:
- Source report explicitly REJECTS Ollama for any ELAI tier: 10-30% raw overhead vs llama.cpp single-user, 19x peak-throughput gap at concurrency (793 vs 41 tok/s), 1.2 GB extra resident RAM, earlier VRAM spillover halves perf on 12 GB cards (morphllm.com, codersera).
- CLAUDE.md no-removal rule + B-37 commit fd5c2e5 still references `RuntimeBackend::Ollama` from `PerRoleRuntime::default` and `runtime_orchestrator::is_recoverable`.

Outcome: Adapter NOT deleted. Added `#[doc(hidden)]` on `OllamaRuntime` struct, one-shot `tracing::warn!` in `OllamaRuntime::new` citing the morphllm.com 19x figure, and seeded `dp-rt-ollama` row into `crates/rules/src/schema.rs:DEPRECATED_PATTERNS_SEED` (severity WARN, ecosystem `cargo` — the schema's CHECK constraint does not include "runtime" as a permitted ecosystem; the row's content is what matters, not the ecosystem fanout). `PerRoleRuntime::default` does NOT map any role to Ollama (the existing B-37 default already preferred Sidecar/AnthropicApi over Ollama).

DECISION-C — XGrammar placement: HEAD of LogitsProcessor chain.

Anchors:
- arXiv:2411.15100 XGrammar MLSys 2025 — <40 us/token mask + ahead-of-time compile.
- E-35 sidecar_xgrammar_hook.rs (SHIPPED) — Rust-side contract: pre-compile 64 grammars at boot, blake3 cache key, REFUSE on-demand compile at request time, `torch.inference_mode()` inside `__call__`. The plan body's "Compiled once per (model, json_schema_hash); cached in an LRU" wording was reconciled DOWN to the E-35 contract per the integration-exploration HIGH-priority warning.
- Source report Section "Why HEAD position": structural masks applied FIRST so DoLa contrasts then operate on already-masked logits (DoLa's intermediate-layer reasoning is preserved while ensuring schema validity is enforced before contrast).

Outcome: `XGrammarLogitsProcessor` lives at `python/elai_inference/server.py`. Inserted at `logits_processors.insert(0, _xgr_processor)` BEFORE `DoLaSteeringProcessor` / `ETCLogitsProcessor` / `GGSignalCollector` / `FoundInTheMiddleCalibrator`. Boot-time pre-compile via `pre_compile_xgrammar_grammars_at_boot` gated by `ELAI_XGRAMMAR_PRECOMPILE_AT_BOOT=1` (pattern-4 wire-dark default). Cache-miss at request time raises `XGrammarCacheMissError` which the orchestrator surfaces as `legacy_json_object_refused` (E-35 SSB row already wired).

xgrammar PyPI verify (anti-slopsquatting per CLAUDE.md): `curl https://pypi.org/pypi/xgrammar/json` 2026-05-26 returned `version=0.2.1`, `license=Apache 2.0`, `upload_time=2026-05-17`. Owner is mlc-ai per blog.mlc.ai/2026/05/04 citation. Added `xgrammar>=0.2.0` to `python/elai_inference/pyproject.toml`.

DECISION-D — 3 new SSB action types, distinct from E-35's 10.

Anchors:
- Integration-exploration HIGH-priority warning: confirm DISTINCT from E-35's already-shipped 10 structured-output variants (XgrammarMaskApplied + InstructorModeActivated + GrammarModeActivated + CodeactActionExecuted + SearchReplaceTier1Hit + SearchReplaceCollisionFallback + CloudStrictSchemaApplied + GrammarCacheLoaded + StructuredEngineRejected + LegacyJsonObjectRefused).

Outcome:
- `SglangBackendRegistered` — adapter init beacon, one row per `SglangRuntime::new` dispatch when `ELAI_INFERENCE_REGISTRY_ENABLE=1`. No E-35 sibling.
- `XgrammarSchemaCompiled` — first-time grammar compile event in the sidecar's boot path. SIBLING-but-not-duplicate of E-35 `GrammarCacheLoaded`: that beacon fires when the pre-compiled cache LOADS from disk; this beacon fires when a (role, block, schema) tuple is COMPILED at boot (cache MISS at warmup, NOT at request time — the E-35 contract forbids on-demand compile at request time).
- `RuntimeColdStartObserved` — generic across all backends, one row per backend's first observed `RuntimeHealth::Ready` transition in a session. Distinct from B-37 `RuntimeHealthChanged` (per-transition beacon); this fires ONCE per first-Ready so the dashboard cold-start indicator clears deterministically.

Atomic SCHEMA bump: 148 -> 151. Lockstep cascade: enum + as_sql_str + parse arm + SsbPayload + 3 payload structs + payload->action_type arm + SQL CHECK constraint + `action_type_lock.rs::TOTAL_VARIANTS` 421 -> 424 + `ALL_VARIANTS` + `exhaustive_match_guard` + `action_type_lock.golden` (+3 rows at alphabetical positions `runtime_cold_start_observed`, `sglang_backend_registered`, `xgrammar_schema_compiled`).

DECISION-E — Line-range corrections (re-check report findings).

The plan body cited stale line ranges. Re-check + grounding sweep flagged:
- `per_role_runtime.rs:158-213` (plan claim) — actual default mapping is at lines 54-91 (lines 158-213 are E-30 lineage-collision code). Corrected: PerRoleRuntime env-gated re-routing landed inside the existing `Default::default()` body (now ~lines 86-105).
- `cli_common.rs:396` (plan claim for factory arm) — `resolve_backend` + `create_runtime` actually live in `mod.rs` (lines 88, 135 region). `enforce_permission_opt_in` is at `cli_common.rs:405`, not :396. Factory arm landed in `mod.rs` per the canonical site.

DECISION-F — Pattern-4 env gates documented.

Three env gates, all default OFF (wire-dark):
- `ELAI_SGLANG_BASE_URL` — when set, the URL is consumed by `SglangRuntime::new` AND `PerRoleRuntime::default` re-routes planner-shape roles (PromptEngineer, IdeaFramer, IdeaCritic) from Sidecar to Sglang. Defined as `ELAI_SGLANG_BASE_URL_ENV` in `runtime_config.rs`. Detection helper: `sglang_per_role_routing_enabled()`.
- `ELAI_INFERENCE_REGISTRY_ENABLE` — gates emission of the `SglangBackendRegistered` SSB beacon. Defined as `ELAI_INFERENCE_REGISTRY_ENABLE_ENV` in `runtime_config.rs`.
- `ELAI_XGRAMMAR_PRECOMPILE_AT_BOOT` — gates the boot-time grammar pre-compile path in `pre_compile_xgrammar_grammars_at_boot`. Defined as `ELAI_XGRAMMAR_PRECOMPILE_AT_BOOT_ENV` in `runtime_config.rs`. Honors the E-35 contract (refuse on-demand compile at request time).

DECISION-G — EasySteer paper ingestion.

Per grounding sweep, arXiv:2509.25175 EasySteer was flagged INGEST (primary "why-not-vLLM-V1" anchor). Inline conclusion (without spawning paper-ingestor): the source report's quotation "EasySteer pinned to vLLM v0.17.1, initial support, missing CUDA graphs as of Mar 2026" is the load-bearing claim and is sufficient as a 1-sentence justification in this decision doc. Full ingestion deferred to a follow-up session if a deeper steering integration with vLLM is ever considered — DECISION-A's HYBRID outcome obviates the need today.

DECISION-H — Dashboard TS<->Rust drift.

Acknowledged drift in `crates/dashboard-v2/src/lib/api/types.ts`:
- Pre-existing: `lsp` is in Rust `RuntimeBackend::canonical_name()` but absent from `RuntimeBackendId` union (E-40 LspRuntime returns NotImplemented; surfacing it in RuntimePanel needs a separate IPC contract).
- E-26 fix: added `sglang` to `RuntimeBackendId` union, `RUNTIME_BACKEND_IDS` array, and `MOCK_RUNTIMES` browser-mode fallback. `lsp` deliberately NOT backfilled in this scope — flagged for a separate fix-it pass.

The Tauri commands `list_runtimes` / `test_runtime` / `set_role_runtime` are LIVE in `crates/dashboard-v2/src-tauri/src/commands.rs` (lines 9101, 9184, 9251) and registered in `crates/dashboard-v2/src-tauri/src/lib.rs:598-606`. They iterate `RuntimeBackend::all()` so the new `Sglang` variant surfaces automatically without command-handler changes.

OUT-OF-SCOPE (deferred):

- vLLM 0.20 native-Windows behind the same adapter — community fork (SystemPanic/vllm-windows) plus upstream Windows wheels as of 2026; the existing vllm.rs adapter remains opt-in but is NOT the E-26 cloud throughput recommendation.
- TGI v3.0 for long-context — would land in a later E-N once cosmos-tier briefings >50K tokens become common (source report Open Question 5).
- EAGLE-3 / P-EAGLE speculative decoding — RUNPOD-PENDING acceptance-rate measurement on Qwen2.5-Coder for ELAI prompts (source report Open Question 3). E-2 prompt-lookup is the lower-cost first attempt.
- `HttpSidecarXGrammarHook` production binding — `sidecar_xgrammar_hook.rs:21-23` flagged this as DEFERRED to the sidecar session. E-26 implements the Python-side `XGrammarLogitsProcessor` per the contract; the Rust-side production hook (HTTP transport that wraps the typed `XGrammarRequest` over the sidecar's `/v1/messages` endpoint) lands in a follow-up. The mock `MockSidecarXGrammarHook` in `sidecar_xgrammar_hook.rs` is sufficient for orchestrator-side tests today.

RUNPOD-PENDING:
- Live SGLang server health-probe + complete + stream against an actual SGLang instance.
- Live planner JSON-mode end-to-end: schema -> XGrammar compile -> masked sampling -> valid JSON output, byte-equivalent to baseline at temp=0 for 5 trial schemas.
- Live cold-start observation row emission against an actual SGLang first-Ready transition.
