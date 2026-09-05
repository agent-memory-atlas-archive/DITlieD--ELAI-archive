# E-26: Inference engines 2026 (SGLang adapter, XGrammar drop-in, demote Ollama)

PROOF_SPINE: feeds PS-4 (routing economics $/verified-task) via the SGLang throughput tier + Ollama demotion, which reshapes the per-role cost surface; the PS-4 live leg itself blocks on J-28 (usage-turn-event live row source), so this plan contributes the routing-tier prerequisite, not the live economics measurement. Owner-ack required if PS-4 feed is disputed; otherwise declared as feeding PS-4.

Objective: extend B-37's RuntimeAdapter trait with a NEW SglangBackend (cloud throughput tier), wire XGrammar as a structured-output LogitsProcessor inside the custom sidecar, and demote the existing Ollama backend per the report's 19x peak-throughput gap. Keeps the custom sidecar canonical for DoLa/ETC/steering/TurboQuant (vLLM V1 deprecated per-request logits processors); makes per-role routing prefer SGLang for planner/critic/structured-JSON traffic and the custom sidecar for research/DoLa traffic.

Estimated effort: 2-3 sessions (~8-12 h).
Pre-requisites: B-37 (RuntimeAdapter trait + 11 backends, SHIPPED commit fd5c2e5). B-19 hook-dispatcher unification is NOT a hard blocker; the SGLang adapter slots into the existing factory at `crates/providers/src/runtime/`.
Parallel-safe with: E-27 (quantization: AWQ+Marlin lives inside the python sidecar, orthogonal to the adapter trait), E-28 (attention kernels: server.py model.from_pretrained changes, orthogonal).
NOT parallel-safe with E-2: E-2 touches `python/elai_inference/server.py` LogitsProcessor wiring; sequence E-26 task-4 AFTER E-2 so the XGrammar LogitsProcessor lands on top of E-2's prompt-lookup/StaticCache surface. E-2 Part-3 (the sidecar prompt-lookup/StaticCache decode surface at server.py:2151-2156) is UNDELIVERED and escalated to the owner as an I-33-class design call (prompt-lookup losslessness vs DoLaSteeringProcessor LogitsProcessor coexistence), with RunPod-only verification. Therefore task-4 (XGrammar LogitsProcessor) is a RECON_NEEDED item gated on E-2 Part-3 owner resolution + RunPod, NOT a free-lane-executable slice.
Source report: workflow/improvement-sweep-2026-05-19/reports/26-inference-engines-2026.md

## Context

Per report TL;DR: vLLM V1 deprecated per-request custom logits processors and moved to batch-level global processors loaded at engine init (docs.vllm.ai/en/v0.8.1/getting_started/v1_user_guide.html). ELAI's DoLa + ETC + steering + TurboQuant LogitsProcessor chain therefore cannot live inside vLLM V1 without a fork. The only working precedent (EasySteer, arXiv:2509.25175) is pinned to vLLM v0.17.1 with "initial support" and still missing CUDA graphs as of Mar 2026.

Counter-evidence for adopting a vLLM-family backend: SGLang shows +29% total tok/s, +117% output tok/s, and 3x faster constrained decoding vs vLLM on H100 8B-class (particula.tech), AND SGLang ships a `CustomLogitProcessor` Python plugin with batch-aware per-request execution that is closer to ELAI's needs than vLLM V1's global batch-level processors (sgl-project/sglang#2291). XGrammar enforces JSON/regex/EBNF at <40 us/token and is the default backend in SGLang, vLLM, TRT-LLM, MLC-LLM as of Mar 2026 (blog.mlc.ai/2026/05/04/xgrammar-2-fast-customizable-structured-generation).

Report net call: HYBRID. Local 12GB tier (RTX 3060) stays on custom sidecar (DoLa + ETC + TurboQuant cannot be replicated in vLLM V1 today; llama.cpp/ExLlamaV3 do not solve ELAI's research-engine needs). Cloud 24GB tier (RTX 3090 pythondev) adds SGLang adapter behind B-37 PerRoleRuntime, gated by role: research/DoLa stays on custom sidecar, planner/critic/structured-JSON routes to SGLang+XGrammar.

Report explicitly REJECTS Ollama for any ELAI tier: 10-30% raw overhead vs llama.cpp single-user, 19x peak-throughput gap at concurrency (793 vs 41 tok/s), 1.2 GB extra RAM, earlier VRAM spillover halves perf on 12 GB cards (morphllm.com, codersera). ELAI today has `crates/providers/src/runtime/ollama.rs` registered as a B-37 backend; that adapter is to be marked DEPRECATED but NOT deleted (per CLAUDE.md "No removal or simplification without explicit user confirmation").

## Existing state (B/C/D coverage)

- B-37 SHIPPED (commit fd5c2e5): RuntimeAdapter trait + 11 backends (Sidecar, AnthropicApi, ClaudeCode, CodexCli, OpenCode, Hermes, GeminiCli, KiroCli, CopilotCli, Ollama, Vllm). E-26 EXTENDS B-37 with a 12th backend (Sglang) and DEMOTES the 11th (Ollama).
- B-37 PerRoleRuntime maps roles to backends at `crates/orchestrator/src/per_role_runtime.rs:158-213`. E-26 adds a new default mapping: planner/critic/grammar-bound roles route to Sglang when SGLang health probe is green, else fall back to Sidecar.
- B-37 RuntimeFactory at `crates/providers/src/runtime/cli_common.rs:396` (`enforce_permission_opt_in`) and the factory-resolve path at `runtime_orchestrator.rs:457` (`is_recoverable`) are the reuse surface; the new Sglang backend reuses the same plumbing.
- VlmConfig::ModeB already references SglangConfig at `crates/orchestrator/src/media_validator/types.rs:364` (Tier-2 VLM path uses SGLang for image validation). NEW E-26 SglangBackend is a SEPARATE struct under `crates/providers/src/runtime/sglang.rs`; name disambiguation: existing `SglangConfig` is media-validator-scoped, new `SglangRuntimeConfig` is RuntimeAdapter-scoped. Do NOT collapse these into one struct (different lifecycle: VLM SGLang is request-local, runtime SGLang is long-lived health-probed adapter).
- vLLM adapter exists at `crates/providers/src/runtime/vllm.rs` (B-37). E-26 does NOT touch the vLLM adapter; it lives alongside the new SGLang adapter.
- Custom sidecar at `python/elai_inference/server.py` already runs the LogitsProcessor chain (DoLa, ETC, steering, TurboQuant per elai.map SCIP refs). XGrammar drop-in lives there as a NEW LogitsProcessor that runs BEFORE DoLa/ETC in the chain. This slice (task-4) is GATED on E-2 Part-3 landing first (see RECON_NEEDED).
- No C-plan or D-plan entry references SGLang, XGrammar, or vLLM as inference backends (Grep `sglang|xgrammar|vllm` over `docs/thefinalglance/C-plan/` and `D-plan/` returns 0 hits outside media_validator).

## Reads (mandatory before coding)

- `workflow/improvement-sweep-2026-05-19/reports/26-inference-engines-2026.md` (source report, full)
- `crates/providers/src/runtime/vllm.rs` lines 130-200 (health_check + complete fn; shape for the SGLang adapter)
- `crates/providers/src/runtime/sidecar.rs:209-380` (existing SidecarBackend complete + stream_step; preserve this path)
- `crates/providers/src/runtime/ollama.rs:130-340` (Ollama adapter; for the DEPRECATED-flag delta only)
- `crates/providers/src/runtime/cli_common.rs:240-420` (factory helpers; reuse for SGLang)
- `crates/orchestrator/src/per_role_runtime.rs:158-213` (per-role mapping; default delta lands here)
- `crates/orchestrator/src/runtime_orchestrator.rs:340-460` (fallback chain semantics)
- `python/elai_inference/server.py` LogitsProcessor wiring section (find via `rg "DoLaSteeringProcessor|ETCLogitsProcessor"`; for inserting XGrammarLogitsProcessor at the chain head, only in task-4 after E-2 Part-3 lands)
- HuggingFace docs (online, do not load): xgrammar.mlc.ai/docs (`XGrammar.compile_json_schema`), docs.sglang.ai/backend/structured_outputs.md
- arXiv:2509.25175 (EasySteer; the "why-not-vLLM-V1" anti-pattern justification)

## Vertical slices (shippable units, E-2-independent first)

The free-lane-drainable set is slices S1, S2, S3, S4 (no E-2 dependency). Slice S5 (task-4 XGrammar LogitsProcessor) is a RECON_NEEDED item gated on E-2 Part-3 owner resolution + RunPod and is NOT free-lane-executable.

### S1: SglangBackend runtime adapter + factory registration

MOVE S1.1: Create `crates/providers/src/runtime/sglang.rs` (~280 LOC) implementing the RuntimeAdapter trait against SGLang's OpenAI-compatible `/v1/chat/completions` + native `/generate` endpoints. Mirror the existing `vllm.rs` shape (health_check via `/v1/models`, complete via POST with streaming SSE). Per report Section "Engine landscape table" SGLang exposes XGrammar through `response_format: {"type": "json_schema", "schema": ...}` AND a native `regex` field; wire both. The adapter MUST NOT initialize the Python `sgl-server` itself; it speaks HTTP to an already-running SGLang instance. Add `SglangRuntimeConfig { base_url: String, api_key: Option<String>, request_timeout: Duration, structured_output_backend: StructuredOutputBackend }` where `StructuredOutputBackend` is an enum {`XGrammar`, `Outlines`, `LMFE`} per docs.sglang.ai. Use blake3 for any request-id hash (per CLAUDE.md Anti-Stale-Training).
EXPECTED_OBSERVATION: `crates/providers/src/runtime/sglang.rs` exists with `pub struct SglangBackend` and `impl RuntimeAdapter for SglangBackend`; `cargo check -p providers` exits 0 after the file lands.
ON_FAIL: failure = `cargo check -p providers` emits a trait-method-not-satisfied or missing-import error. cause = the RuntimeAdapter trait shape drifted from B-37 fd5c2e5, or a reqwest/serde import path is wrong. counter-move = re-read `crates/providers/src/runtime/vllm.rs:130-200` and mirror its imports + trait impl signature exactly; re-run `cargo check -p providers`.

MOVE S1.2: Register SglangBackend in the factory (~30 LOC delta in `crates/providers/src/runtime/cli_common.rs` + `runtime/mod.rs`). Add to the `RuntimeAdapter` enum + `resolve_backend("sglang")` arm + `create_runtime(cfg)` factory. Mirror exactly the Vllm registration site. Per CLAUDE.md `enforce_permission_opt_in` still gates the new backend the same way as vLLM/Ollama. Add `pub mod sglang;` to `runtime/mod.rs`.
EXPECTED_OBSERVATION: `rg "Sglang" crates/providers/src/runtime/cli_common.rs` returns the new arm; `rg "pub mod sglang" crates/providers/src/runtime/mod.rs` returns one hit; `cargo check -p providers` exits 0.
ON_FAIL: failure = `resolve_backend("sglang")` arm missing or factory pattern-mismatch. cause = the enum variant name or the `create_runtime` match arm was typed wrong. counter-move = diff against the Vllm registration site at `cli_common.rs:396` and align variant name + match arm shape; re-run `cargo check -p providers`.

MOVE S1.3: Add SglangBackend to per-role mapping (~50 LOC delta in `crates/orchestrator/src/per_role_runtime.rs:158-213`). New default: planner, critic, prompt_engineer, structured-JSON callers route to Sglang when SGLang health probe is green, else fall back to Sidecar. Research, executor, memory_writer stay on Sidecar/AnthropicApi as before. The mapping is gated by env `ELAI_SGLANG_BASE_URL`; when unset, all routing collapses to today's behaviour (no behaviour change in default deploy).
EXPECTED_OBSERVATION: `rg "Sglang" crates/orchestrator/src/per_role_runtime.rs` returns the new routing arm; `cargo check -p elai-orchestrator` exits 0; with `ELAI_SGLANG_BASE_URL` unset, the existing per-role routing tests pass unchanged.
ON_FAIL: failure = a per-role routing test fails because the new arm changes default behaviour when the env is unset. cause = the env-gate was placed on the wrong side of the fallback. counter-move = move the `ELAI_SGLANG_BASE_URL` check to the very top of the new arm so unset collapses to the pre-E-26 mapping before any Sglang resolution; re-run `cargo test -p elai-orchestrator per_role`.

PRODUCTION_HOOK: `crates/providers/src/runtime/cli_common.rs::resolve_backend` (new "sglang" arm) is the factory seam; `crates/orchestrator/src/per_role_runtime.rs::PerRoleRuntime::default` (new routing arm) is the consumer seam. Both ship in THIS slice. The slice is a vertical slice: producer (SglangBackend) + consumer (factory + per-role mapping) + smoke (the per-role routing contract test) in the same unit.

FORK S1: if `cargo check -p providers` fails because `RuntimeAdapter` trait shape drifted from B-37 fd5c2e5 (a method was added/renamed since fd5c2e5), trigger = compiler error naming a missing trait method. Route A (preferred): implement the missing method by mirroring `vllm.rs`'s impl of that same method. Route B: if the trait genuinely cannot be satisfied without a new helper, STOP + handoff (R-4) with the trait diff; do NOT stub the method.

### S2: Ollama deprecation flag

MOVE S2.1: Add deprecation flag on Ollama backend (~15 LOC delta in `crates/providers/src/runtime/ollama.rs`). Add `#[doc(hidden)]` and a `tracing::warn!` on construction citing report URL morphllm.com (19x gap, 1.2GB RAM overhead). Do NOT delete the adapter (per CLAUDE.md no-removal rule + B-37 commit fd5c2e5 still references it). Add a config-validation row to `crates/rules/src/schema.rs:174` (the existing "tracing is the modern Rust async logging standard" pattern): `Some("2026-05"), "WARN", "Ollama runtime adapter is DEPRECATED for ELAI use; report 26-inference-engines-2026.md cites 19x peak-throughput gap and 1.2GB extra RAM. Prefer Sidecar (local) or Sglang (cloud) backends."`.
EXPECTED_OBSERVATION: `rg "DEPRECATED|19x peak-throughput" crates/providers/src/runtime/ollama.rs` returns the warn line; `rg "Ollama runtime adapter is DEPRECATED" crates/rules/src/schema.rs` returns the new row; `cargo check -p providers -p elai-rules` exits 0.
ON_FAIL: failure = `cargo check -p elai-rules` fails on the schema row shape. cause = the row tuple shape at schema.rs:174 changed since the plan was authored. counter-move = re-read `crates/rules/src/schema.rs:170-180` and align the tuple arity + field types with the existing neighbours; re-run `cargo check -p elai-rules`.

PRODUCTION_HOOK: `crates/providers/src/runtime/ollama.rs::OllamaBackend::new` (the warn-on-construction site) is the seam; the consumer is the existing factory at `cli_common.rs::resolve_backend("ollama")` which still resolves it (no deletion). Slice ships the deprecation flag + the schema row + the contract test that asserts the warn fires once.

FORKS S2: none.

### S3: 3 new SSB action types + runtime_cold_start_observed emission

MOVE S3.1: Add 3 new SSB action types (~40 LOC in `crates/orchestrator/src/ssb.rs`). `sglang_backend_registered`, `xgrammar_schema_compiled` (with payload `{schema_hash, compile_ms}`), `runtime_cold_start_observed`. Wire each via `as_sql_str` parse + variant arm + the golden action_type lock (regenerate the lock per the existing SSB contract test).
EXPECTED_OBSERVATION: `rg "sglang_backend_registered|xgrammar_schema_compiled|runtime_cold_start_observed" crates/orchestrator/src/ssb.rs` returns all three; the SSB golden-lock contract test regenerates and passes; `cargo test -p elai-orchestrator ssb` exits 0.
ON_FAIL: failure = the golden action_type lock test fails because the new variants are not in the alphabetical slot. cause = the variants were added without regenerating the lock. counter-move = run the SSB lock regeneration command (per the existing test's failure message) and re-run `cargo test -p elai-orchestrator ssb`.

MOVE S3.2: Health-probe contract for SglangBackend (~60 LOC). Per report Section "Cold-start latency comparison" SGLang cold-start is "similar to vLLM warm (~8s)". The existing B-37 RuntimeHealth (green/yellow/red) covers this. NEW: emit `runtime_cold_start_observed` SSB row on first successful health probe with elapsed-ms field. Inform the user via RuntimePanel that cold-start has cleared.
EXPECTED_OBSERVATION: on first green health probe, an SSB row with action_type `runtime_cold_start_observed` lands in the SSB event store (sqlite WAL); `rg "runtime_cold_start_observed" crates/providers/src/runtime/sglang.rs` returns the emit site.
ON_FAIL: failure = the SSB emit site is wired but no row lands in the event store. cause = the SSB writer path is not the same as the existing B-37 emit path (e.g. a different writer handle). counter-move = re-read the existing B-37 SSB emit site (e.g. in `vllm.rs` or `sidecar.rs`) and mirror the writer handle + topic; re-run the contract test.

PRODUCTION_HOOK: `crates/orchestrator/src/ssb.rs::SsbActionType` (the 3 new variants) is the producer; the consumer is `crates/providers/src/runtime/sglang.rs::health_check` (the `runtime_cold_start_observed` emit) + the SSB event store. Slice ships the variants + the emit site + the SSB roundtrip contract test.

FORK S3: if the SSB golden lock cannot be regenerated without owner approval (the lock is a frozen artifact), trigger = the lock regeneration command reports it is frozen. Route A: STOP + handoff (R-4) asking the owner to approve the lock regeneration. Route B: if the lock is auto-regenerated by the contract test itself (verify by reading the test), let the test regenerate it.

### S4: Per-role override config arm + decision doc

MOVE S4.1: Per-role override in `.elai_cc.toml`/`elai.toml` for SGLang routing (~30 LOC). New `[agents.<role>.runtime]` arm accepts `backend = "sglang"` and `[runtime.sglang] base_url = "..."` for static config; B-37 deprecation alias chain unchanged.
EXPECTED_OBSERVATION: `rg "runtime.sglang" crates/orchestrator/src/config.rs` (or the relevant config parser) returns the new arm; `cargo check -p elai-orchestrator` exits 0; a config with `[runtime.sglang] base_url = "http://127.0.0.1:30000"` parses without error.
ON_FAIL: failure = config parser rejects the new arm. cause = the config struct does not have a `sglang` field yet. counter-move = add the field to the config struct mirroring the existing `vllm` field shape; re-run `cargo check -p elai-orchestrator`.

MOVE S4.2: Write decision doc at `workflow/reasoning/E-26-decisions.md` covering: (a) why SGLang and NOT vLLM (per-request logits processors deprecated in V1, EasySteer pinned to v0.17.1), (b) Ollama deprecation rationale (19x gap, do-not-delete per no-removal rule), (c) XGrammar placement in LogitsProcessor chain (HEAD position so DoLa contrasts run on masked logits), (d) explicit OUT-OF-SCOPE: vLLM 0.20 native-Windows behind the same adapter, TGI v3.0 for long-context (deferred to a later E-N once cosmos-tier briefings >50K tokens become common), (e) the E-2 Part-3 gating of task-4 and the owner-ack requirement.
EXPECTED_OBSERVATION: `test -f workflow/reasoning/E-26-decisions.md` returns 0; the doc has all 5 sections.
ON_FAIL: failure = the doc is missing a section. cause = the executor skipped one. counter-move = re-read the section list above and add the missing section; re-run `test -f`.

PRODUCTION_HOOK: S4.1's seam is `crates/orchestrator/src/config.rs` (the config parser). S4.2 is a prose deliverable (no PRODUCTION_HOOK per H-10 E-3 prose exception, but the doc itself is a required deliverable).

FORKS S4: none.

### S5: XGrammar LogitsProcessor (RECON_NEEDED, NOT free-lane-executable)

RECON_NEEDED: task-4 (XGrammar structured-output LogitsProcessor in `python/elai_inference/server.py`, ~180 LOC) is BLOCKED on E-2 Part-3 (the sidecar prompt-lookup/StaticCache decode surface at server.py:2151-2156). E-2 Part-3 is UNDELIVERED, escalated to the owner as an I-33-class design call (prompt-lookup losslessness vs DoLaSteeringProcessor LogitsProcessor coexistence), and its verification is RUNPOD-only (free lane cannot smoke the ON path). Tripline = E-2 Part-3 owner resolution + RunPod availability. Until the tripline clears, S5 is NOT executed.

MOVE S5.1 (GATED): Add XGrammarLogitsProcessor to the custom sidecar (~180 LOC in `python/elai_inference/server.py`). New class `XGrammarLogitsProcessor(LogitsProcessor)` wrapping `xgrammar.GrammarCompiler` + per-request `GrammarMatcher`. Compiled once per (model, json_schema_hash); cached in an LRU keyed on blake3(schema_json). Insert at the HEAD of the LogitsProcessor chain (BEFORE DoLa/ETC) so structural masks are applied first; DoLa contrasts then operate on the masked logits. Gate via the Anthropic-protocol request body field `tools[].input_schema` (when present) and a per-request `structured_output: {type:"json_schema", schema:..., backend:"xgrammar"}` extension. Defaults: when no schema specified, processor is not in the chain (no overhead). This move is GATED on the S5 RECON_NEEDED tripline clearing (E-2 Part-3 lands + RunPod available).
EXPECTED_OBSERVATION: `rg "class XGrammarLogitsProcessor" python/elai_inference/server.py` returns the class; `rg "from xgrammar" python/elai_inference/server.py` returns the import; on a RunPod smoke with a schema-bearing request, the response is valid JSON conforming to the schema with zero retry.
ON_FAIL: failure = the LogitsProcessor chain coexistence breaks prompt-lookup losslessness (the E-2 Part-3 design call). cause = the HEAD insertion masks logits in a way that conflicts with the StaticCache decode surface. counter-move = STOP + handoff (R-4) with the coexistence failure evidence; do NOT silently disable DoLa/ETC or bypass the chain. This is exactly the I-33-class design call the owner must resolve.

PRODUCTION_HOOK: `python/elai_inference/server.py` LogitsProcessor chain head is the seam. The consumer is the Anthropic-protocol request body with `tools[].input_schema` or `structured_output`. Slice ships the processor + the chain insertion + the RunPod smoke (DEFERRED, hardware-gated).

FORK S5: if the executor reaches S5 before E-2 Part-3 lands, trigger = E-2 Part-3 is not in the tree (check via `rg "prompt-lookup|StaticCache" python/elai_inference/server.py` at the expected line range). Route A (REQUIRED): STOP + handoff (R-4) flagging the E-2 Part-3 dependency; do NOT execute S5. Route B is not permitted; there is no safe way to land S5 without E-2 Part-3.

## ABORT

ABORT_CONDITIONS:
- The executor reaches S5 (task-4 XGrammar LogitsProcessor) before E-2 Part-3 lands in the tree. STOP + handoff (R-4) flagging the E-2 Part-3 dependency; do NOT execute S5.
- `cargo check -p providers` fails on a RuntimeAdapter trait-method-not-satisfied error that cannot be resolved by mirroring `vllm.rs` (the trait drifted in a non-mechanical way). STOP + handoff (R-4) with the trait diff.
- The SSB golden action_type lock is frozen and cannot be regenerated without owner approval. STOP + handoff (R-4) asking the owner to approve the lock regeneration.
- A new Rust dependency is required beyond the existing reqwest/serde stack (per CLAUDE.md no-new-deps without confirmation). STOP + handoff (R-4) naming the dep + the pypi/crates.io verify row.
- `xgrammar` Python dep fails pypi version+license verify per CLAUDE.md anti-slopsquatting. STOP + handoff (R-4) with the verify output; do NOT add the dep.

## VERIFY

VERIFY runs (off-agent pass artifacts, never the executor's own stdout):
- V1 (frozen checker, S1): `python .claude/tools/production_caller_check.py --files crates/providers/src/runtime/sglang.rs` exits 0 (no DARK pub fn). A DARK symbol (exit 1) is a BLOCK; the executor names its PRODUCTION_HOOK seam in the slice, and the auditor re-runs this checker off-agent.
- V2 (frozen checker, S1): `python .claude/tools/python_wire_check.py --files python/elai_inference/server.py` exits 0 for S5 (only when S5 lands; skipped if S5 is gated).
- V3 (cargo, S1+S2+S3+S4): `cargo check --workspace` exits 0. Run by the auditor in AUDITING, not by the executor.
- V4 (cargo test, S1+S2+S3): `cargo test -p providers -p elai-orchestrator` exits 0, including the new contract tests: `sglang_backend_contract.rs`, `sglang_per_role_routing_contract.rs`, `ollama_deprecation_warning_contract.rs`, `sglang_ssb_action_types_contract.rs`. Run by the auditor.
- V5 (SSB event store, S3): the `runtime_cold_start_observed` SSB row lands in the SSB sqlite WAL on first green health probe. The auditor queries the event store (an off-agent artifact the executor did not author) and confirms the row. Missing row = FAIL (R1), not UNVERIFIED.
- V6 (pytest, S5, DEFERRED): `pytest python/elai_inference/test_xgrammar_logits_processor.py` exits 0, asserting structural mask applied before DoLa runs + byte-equivalent JSON output across 5 fixtures. DEFERRED to RunPod (hardware-gated); the free lane cannot smoke the ON path.
- V7 (backward-compat, S1): with `ELAI_SGLANG_BASE_URL` unset, the existing B-37 per-role routing tests pass unchanged. The auditor runs `cargo test -p elai-orchestrator per_role` and confirms no behaviour change.

## REDTEAM

REDTEAM:
- Attack R1 (succeeded, patched): "the executor ships SglangBackend as a producer-only unit, with the factory arm + per-role mapping deferred to a later step." This attack succeeded against the legacy plan (the original task list had tasks 1, 2, 3 as separate steps, allowing task 1 to land without its consumer). Patch: S1 is now a vertical slice that ships the producer + the factory arm + the per-role mapping + the routing contract test in the SAME unit. The PRODUCTION_HOOK seam is named in the slice. The auditor re-runs `production_caller_check.py` off-agent; a DARK `SglangBackend` with no factory arm is a BLOCK.
- Attack R2 (failed): "the executor silently disables DoLa/ETC when SGLang is selected, routing around the LogitsProcessor chain." This attack fails against the plan because: (a) the per-role mapping explicitly keeps research/executor on Sidecar/AnthropicApi (DoLa/ETC stay attached), (b) S5's ON_FAIL explicitly forbids a bypass branch (`if sglang: skip_processors` is banned in the WIRING_CONTRACT stub_ban), (c) the contract test `sglang_per_role_routing_contract.rs` asserts research role stays on Sidecar. The attack surface is closed by the per-role split + the stub_ban + the contract test.
- Attack R3 (succeeded, patched): "the executor lands S5 (XGrammar LogitsProcessor) before E-2 Part-3, corrupting the prompt-lookup/StaticCache decode surface." This attack succeeded against the legacy plan (task-4 was unguarded, listed as a free-lane-executable step). Patch: S5 is now a RECON_NEEDED item with a tripline on E-2 Part-3 owner resolution + RunPod, an ABORT condition if the executor reaches S5 before E-2 Part-3 lands, and a FORK that forces STOP + handoff (R-4). The free lane cannot reach S5.
- Attack R4 (failed): "the executor adds a new Rust dependency (e.g. `sglang-rs`) without pypi/crates.io verify." This attack fails against the plan because: (a) the ABORT conditions explicitly forbid new Rust deps beyond reqwest/serde without owner confirmation, (b) the adapter speaks HTTP via the existing reqwest stack (no new dep needed), (c) the auditor checks `Cargo.lock` diff for new crates. The attack surface is closed by the ABORT + the HTTP-only design + the Cargo.lock audit.
- Attack R5 (succeeded, patched): "the executor ships SglangBackend behind a default-OFF flag with no wire-dark-allowlist row." This attack succeeded against the legacy plan (the ELAI_SGLANG_BASE_URL gate was mentioned but no allowlist row was required). Patch: the INTEGRATION_SMOKE section below explicitly requires a `wire-dark-allowlist.toml` row (type/owner/expiry_date/promotion_plan_id per wiring-enforcement R9) for the ELAI_SGLANG_BASE_URL default-OFF flag, and the auditor checks the allowlist.

## INTEGRATION_SMOKE

INTEGRATION_SMOKE cadence (multi-code-unit build, 4 free-lane slices S1-S4 + 1 deferred slice S5):
- IS1 (after S1+S2, free-lane, static): `cargo check --workspace` + `cargo test -p providers -p elai-orchestrator` green. This is the per-unit green, necessary-not-sufficient.
- IS2 (after S3, free-lane, SSB roundtrip): the SSB roundtrip contract test (`sglang_ssb_action_types_contract.rs`) drives the REAL SSB writer (not a test shim) and asserts all 3 new action types round-trip through msgpack + sqlite WAL. The off-agent artifact is the SSB event-store row. Missing row = FAIL (R1).
- IS3 (after S1+S2+S3+S4, free-lane, cross-seam): an INDEPENDENT auditor-driven integration smoke that drives the REAL entrypoint (`cargo test -p elai-orchestrator per_role` with `ELAI_SGLANG_BASE_URL` set to a mock SGLang endpoint) and asserts: (a) planner role routes to SglangBackend, (b) research role stays on Sidecar, (c) `runtime_cold_start_observed` SSB row lands on first green health probe, (d) with `ELAI_SGLANG_BASE_URL` unset, routing collapses to B-37 defaults. This is the R10 cross-seam smoke (strict peer = mock SGLang returning valid `/v1/models`; multi-turn = planner + critic + research roles in sequence; long enough to fire the cold-start SSB emit).
- IS4 (DEFERRED, hardware-gated, S5): the live-SGLang acceptance smoke drives the REAL entrypoint against a live SGLang endpoint via `ELAI_SGLANG_BASE_URL`. This is HARDWARE-GATED: the free lane cannot exercise the ON path (no live SGLang endpoint). DEFERRED to RunPod. The smoke asserts: (a) planner JSON-mode end-to-end: schema > XGrammar compile > masked sampling > valid JSON output, byte-equivalent to baseline at temp=0 for 5 trial schemas, (b) DoLa/ETC/steering LogitsProcessor chain remains attached on research-role calls (rg confirms no `if sglang: skip_processors` branch), (c) `xgrammar_schema_compiled` SSB row lands on first schema-bearing call. Until RunPod is available, IS4 is UNVERIFIED (state explicitly, do NOT fabricate a pass).
- WIRE-DARK-ALLOWLIST: the `ELAI_SGLANG_BASE_URL` default-OFF flag REQUIRES a row in `.elai_cc/wire-dark-allowlist.toml` carrying type (kill_switch | experiment), owner, expiry_date, promotion_plan_id, and a smoke test that exercises the ON path (per wiring-enforcement R9). The ON-path smoke is IS4 (DEFERRED, hardware-gated). The allowlist row is a REQUIRED deliverable in S1 (the slice that introduces the flag). A missing allowlist row is a BLOCK (R9). The auditor checks the allowlist.

## WIRING_CONTRACT

produces_symbols:
  - crates/providers/src/runtime/sglang.rs::SglangBackend (RuntimeAdapter impl)
  - crates/providers/src/runtime/sglang.rs::SglangRuntimeConfig
  - crates/providers/src/runtime/sglang.rs::StructuredOutputBackend
  - python/elai_inference/server.py::XGrammarLogitsProcessor (S5, GATED on E-2 Part-3)
  - SsbActionType::{SglangBackendRegistered, XgrammarSchemaCompiled, RuntimeColdStartObserved}
consumes_symbols:
  - crates/providers/src/runtime/cli_common.rs::resolve_backend (new "sglang" arm)
  - crates/providers/src/runtime/mod.rs (new `pub mod sglang;`)
  - crates/orchestrator/src/per_role_runtime.rs::PerRoleRuntime::default (new routing arm)
  - crates/orchestrator/src/runtime_orchestrator.rs::is_recoverable (Sglang errors map to existing categories)
  - crates/rules/src/schema.rs:174 (new deprecation row)
  - python/elai_inference/server.py LogitsProcessor chain head (S5, GATED)
reachable_from:
  - User issues planner/critic prompt in dashboard > PerRoleRuntime resolves to SglangBackend > HTTP /v1/chat/completions > SSE stream > orchestrator
  - REPL `/runtime show` > RuntimePanel lists SglangBackend with health
  - Custom sidecar request body with `structured_output.schema` > XGrammar masks logits > DoLa applies contrasts (S5, GATED)
stub_ban:
  - "TODO: sglang adapter" literal absent
  - "// vllm V1 might work" literal absent (report rejects)
  - No new ollama_default callsite; Ollama backend is DEPRECATED, default mapping does not select it
  - No XGrammar bypass branch on the LogitsProcessor chain that silently disables DoLa/ETC
  - No `if sglang: skip_processors` branch anywhere
frontend_pair: crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte (existing RuntimePanel surfaces the new backend; no new file needed)
observable_effect:
  trigger: "ELAI_SGLANG_BASE_URL=http://127.0.0.1:30000 elai.bat repl, then submit a planner JSON-mode prompt with input_schema set"
  expect_ssb: "sglang_backend_registered emitted at boot; xgrammar_schema_compiled emitted on first call (S5, GATED); runtime_cold_start_observed emitted when SGLang first reaches healthy state"
  expect_artifact: "Planner returns valid JSON conforming to schema with zero retry; RuntimePanel shows Sglang backend green; DoLa LogitsProcessor still attached to research-role calls"

## DETERMINISTIC_CHECK

```bash
python .claude/tools/production_caller_check.py --files crates/providers/src/runtime/sglang.rs
python .claude/tools/python_wire_check.py --files python/elai_inference/server.py
test -f .elai_cc/crates/providers/src/runtime/sglang.rs
rg "Sglang" .elai_cc/crates/providers/src/runtime/cli_common.rs
rg "Sglang" .elai_cc/crates/orchestrator/src/per_role_runtime.rs
rg "class XGrammarLogitsProcessor" .elai_cc/python/elai_inference/server.py
rg "from xgrammar" .elai_cc/python/elai_inference/server.py
rg "sglang_backend_registered|xgrammar_schema_compiled|runtime_cold_start_observed" .elai_cc/crates/orchestrator/src/ssb.rs
rg "DEPRECATED|19x peak-throughput" .elai_cc/crates/providers/src/runtime/ollama.rs
test -f workflow/reasoning/E-26-decisions.md
cd .elai_cc && cargo test -p providers --test sglang_health_check_contract
```

## ACCEPTANCE

- [ ] SglangBackend health_check returns green against a mock SGLang `/v1/models` response within 200ms
- [ ] Planner JSON-mode end-to-end (IS4, DEFERRED to RunPod): schema > XGrammar compile > masked sampling > valid JSON output, byte-equivalent to baseline at temp=0 for 5 trial schemas
- [ ] DoLa/ETC/steering LogitsProcessor chain remains attached on research-role calls (rg confirms no `if sglang: skip_processors` branch)
- [ ] Ollama backend emits the deprecation warning exactly once on construction (caplog assertion in a unit test)
- [ ] PerRoleRuntime defaults unchanged when `ELAI_SGLANG_BASE_URL` is unset (full backward-compat with B-37 shipped behaviour)
- [ ] All 3 new SSB action types round-trip through msgpack + sqlite WAL (existing SSB roundtrip contract test extends)
- [ ] No new dependency on the Rust side beyond the existing reqwest/serde stack (per CLAUDE.md no-new-deps without confirmation); `xgrammar` Python dep added to `python/elai_inference/pyproject.toml` after pypi version+license verify per CLAUDE.md anti-slopsquatting protocol
- [ ] `.unwrap()` count in non-test source unchanged or decreased
- [ ] `cargo check --workspace` and `cargo test -p providers -p elai-orchestrator` GREEN
- [ ] `ELAI_SGLANG_BASE_URL` default-OFF flag has a row in `.elai_cc/wire-dark-allowlist.toml` (type/owner/expiry_date/promotion_plan_id per R9)
- [ ] `production_caller_check.py --files crates/providers/src/runtime/sglang.rs` exits 0 (no DARK pub fn)

## Tests to add

- `crates/providers/tests/sglang_backend_contract.rs`: health_check + complete + stream_step against a tokio-test mock server (mirrors `vllm_backend_contract.rs`).
- `crates/contracts/tests/sglang_per_role_routing_contract.rs`: PerRoleRuntime with `ELAI_SGLANG_BASE_URL` set routes planner role to Sglang, research role to Sidecar; with env unset, routing collapses to B-37 defaults.
- `crates/contracts/tests/ollama_deprecation_warning_contract.rs`: caplog asserts the WARN on construction.
- `python/elai_inference/test_xgrammar_logits_processor.py` (S5, GATED): assert structural mask applied before DoLa runs, byte-equivalent JSON output across 5 fixtures.
- `crates/contracts/tests/sglang_ssb_action_types_contract.rs`: SSB roundtrip for `sglang_backend_registered`, `xgrammar_schema_compiled`, `runtime_cold_start_observed`.

## Files touched

- CREATE: `crates/providers/src/runtime/sglang.rs` (~280 LOC)
- CREATE: `crates/providers/tests/sglang_backend_contract.rs` (~120 LOC)
- CREATE: `crates/contracts/tests/sglang_per_role_routing_contract.rs` (~80 LOC)
- CREATE: `crates/contracts/tests/ollama_deprecation_warning_contract.rs` (~50 LOC)
- CREATE: `python/elai_inference/test_xgrammar_logits_processor.py` (~140 LOC, S5 GATED)
- CREATE: `workflow/reasoning/E-26-decisions.md` (~80 LOC)
- MODIFY: `crates/providers/src/runtime/mod.rs` (~10 LOC: register `pub mod sglang;`)
- MODIFY: `crates/providers/src/runtime/cli_common.rs:396` (~30 LOC: factory arm)
- MODIFY: `crates/orchestrator/src/per_role_runtime.rs:158-213` (~50 LOC: default mapping)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~40 LOC: 3 new action_types)
- MODIFY: `crates/providers/src/runtime/ollama.rs` (~15 LOC: deprecation flag + warn)
- MODIFY: `crates/rules/src/schema.rs` (~5 LOC: new rule row)
- MODIFY: `python/elai_inference/server.py` (~180 LOC: XGrammarLogitsProcessor + chain head insert + request-body extension, S5 GATED)
- MODIFY: `python/elai_inference/pyproject.toml` (~5 LOC: xgrammar dep after pypi verify, S5 GATED)
- MODIFY: `.elai_cc/wire-dark-allowlist.toml` (new row for ELAI_SGLANG_BASE_URL, S1)

## Citations / Grounding

- arXiv:2509.25175 (EasySteer; vLLM v0.17.1 pinning + missing CUDA graphs is the "why-not-vLLM" anchor)
- docs.vllm.ai/en/v0.8.1/getting_started/v1_user_guide.html (vLLM V1 logits-processor deprecation)
- docs.vllm.ai/en/stable/features/custom_logitsprocs/ (V1 batch-level apply() shape)
- particula.tech/blog/sglang-vs-vllm-inference-engine-comparison (+29% / +117% / 3x constrained decoding)
- github.com/sgl-project/sglang/issues/2291 (CustomLogitProcessor batch-aware plugin)
- blog.mlc.ai/2026/05/04/xgrammar-2-fast-customizable-structured-generation (<40 us/token, default in 4 engines)
- morphllm.com/comparisons/llama-cpp-vs-ollama (19x gap, 1.2GB RAM)
- docs.vllm.ai/en/stable/serving/integrations/claude_code/ (vLLM Anthropic /v1/messages endpoint exists; not chosen for E-26 due to V1 logits-processor issue)
- blog.premai.io/speculative-decoding-2-3x-faster-llm-inference-2026/ (EAGLE-3 industrial default; deferred per Open Questions)

## Open questions / risks

- Report Open Question 3: EAGLE-3 acceptance rate on Qwen2.5-Coder for ELAI prompts is unmeasured. E-26 does NOT include EAGLE-3 spec-decoding; that lands in a later E-N. E-2 already covers prompt-lookup-num-tokens as the lower-cost first attempt.
- Report Open Question 4: SGLang RadixAttention hit rate on ELAI's request stream (planner+critic+executor) is unknown; the 75-95% multi-turn / 85-95% few-shot numbers are for chat / few-shot. Mitigation: telemetry-only first; do not benchmark the routing default-on until SGLang RadixAttention measurements land.
- Report Open Question 5: vLLM 0.20 native-Windows stability on RTX 3060 (SM_86, 12GB) is untested for ELAI. E-26 does NOT touch the existing vllm.rs adapter; vLLM remains opt-in.
- Ungrounded conjecture flagged in report: "production speculative decoding underperforms lab numbers by 40-60%". E-26 does not depend on this; only XGrammar (proven <40 us/token) and SGLang routing.
- Anti-Stale-Training risk: vLLM Windows native is a community fork (SystemPanic/vllm-windows) plus upstream Windows wheels as of 2026; pypi verify mandatory before any version bump (CLAUDE.md anti-slopsquatting).

## Next

E-27 (quantization) lands AWQ+Marlin inside `python/elai_inference/server.py`; independent of the SGLang adapter but the two compose (Marlin-AWQ weights work behind both the custom sidecar and SGLang if/when ELAI runs SGLang with AWQ models). E-28 (attention kernels) keeps FA2 mandatory on Ampere; no interaction with E-26. Future E-N: EAGLE-3 / P-EAGLE speculative decoding once Open Question 3 has measured acceptance-rate data on ELAI's prompts; TGI v3.0 cosmos-tier adapter if cosmos briefings >50K tokens become common.
