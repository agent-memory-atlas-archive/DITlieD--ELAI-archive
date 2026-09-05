REDIRECTED OUTPUT (see BLOCKED note at end): this file was supposed to be written to
workflow/sessions/s-finish-0705/context-package-SP-3A.md per the context-compiler dispatch
instruction. The PreToolUse FSM hook rejected that path (global FSM state EXECUTING, phase
string SP-1A unit 1.1, allowed write targets reported as: src/, tests/, memory-bank/*.md,
workflow/reasoning/ only). workflow/handoff/teikoku-elai-finish.md was also rejected under the
same allowlist, so this handoff entry could not be appended either. workflow/reasoning/ was the
only currently-open path, used here to avoid losing the research. Orchestrator: please relocate/
copy this content to the intended path once the FSM/hook state allows, or reconfigure the hook's
EXECUTING-state allowlist to include workflow/sessions/<session_id>/ for context-compiler output.

======================================================================
CONTEXT PACKAGE — SP-3A (unit 3.1, TEIKOKU-ELAI-FINISH-PLAN.md wave 3, lines 143-147)
======================================================================
Task: wire the bmapi gpt-5.5 OpenAI-compat proxy (base https://bmapi.020212.xyz, wire_api
responses-style per codex config) as an ELAI provider through the existing crates/providers
openai_compat seam; context window sourced LIVE from proxy /v1/models; key/base_url via env only.
FSM: EXECUTING | Phase shown: SP-1A precheck routing port (unit 1.1) | Attempts 0/3 | AuditCycles 0/2
NOTE: FSM phase string is SP-1A, not SP-3A — this is shared global FSM state, likely stale/owned by
a parallel track. Not this package's concern; flag to orchestrator, do not resolve here.

(a) PROVIDER ARCHITECTURE MAP (file:line)
Dispatch chain (model string -> live client), all in elai-providers crate unless noted:
1. crates/cli/src/args.rs:2635-2645 parse_args() reads --model/--model=X, calls
   resolve_model_alias(value), stores resolved string in CliAction. cli_consts.rs:18
   DEFAULT_MODEL = "claude-opus-4-6".
2. crates/providers/src/providers/mod.rs:33-130 MODEL_REGISTRY: static &[(&str,
   ProviderMetadata)] table, alias -> {provider: ProviderKind, auth_env, base_url_env,
   default_base_url}. Currently opus/sonnet/haiku -> Anthropic; grok/grok-3/grok-mini/
   grok-3-mini/grok-2 -> Xai. NO OpenAi-kind entries in the table itself (OpenAi is reached
   only via fallback, see #4).
3. mod.rs:132-158 resolve_model_alias(model) - lowercases, looks up MODEL_REGISTRY, maps to
   canonical model id (e.g. "grok"->"grok-3"); unknown strings pass through unchanged.
4. mod.rs:198-213 detect_provider_kind(model): calls metadata_for_model() first (prefix match
   on canonical name: "claude*"->Anthropic, "grok*"->Xai, ollama-family prefixes->Ollama,
   contains ':' -> Ollama); if no name match, FALLS BACK to env-var probing in fixed order:
   anthropic::has_auth_from_env_or_saved() -> Anthropic, else openai_compat::has_api_key(
   "OPENAI_API_KEY") -> OpenAi, else openai_compat::has_api_key("XAI_API_KEY") -> Xai, else
   Anthropic default. A new provider with an unrecognized model name and its OWN api-key env
   var (not OPENAI_API_KEY) will NEVER be selected by this fallback — it MUST get a
   MODEL_REGISTRY name-prefix entry (like grok's) to be reachable by name.
5. crates/providers/src/client.rs:10-105 ProviderClient enum {Anthropic(AnthropicClient),
   Xai(OpenAiCompatClient), OpenAi(OpenAiCompatClient)}. from_model(model) [17-19] ->
   from_model_with_anthropic_auth [21-45]: resolve_model_alias -> detect_provider_kind -> match
   ProviderKind -> construct via OpenAiCompatClient::from_env(OpenAiCompatConfig::xai()/::openai()).
   provider_kind()/send_message()/stream_message() all match on the 3 variants; Xai and OpenAi
   share one arm (client.send_message/stream_message) since both are OpenAiCompatClient.
6. Real production caller: crates/tools/src/lib.rs:4377-4396 ProviderRuntimeClient::new(model,
   allowed_tools) calls resolve_model_alias then ProviderClient::from_model(&model); implements
   ApiClient trait (used by orchestrator role/tool-dispatch paths incl. agent_role_dispatcher.rs
   and adversary.rs per contracts/tests/agent_role_dispatcher_contract.rs +
   adversary_contract.rs). This is the SAME lane the "adversary" agent type (cross-family critic,
   currently Grok) uses — the natural PRODUCTION_HOOK for a new provider is this exact seam:
   register in MODEL_REGISTRY + ProviderKind + client.rs match arms, then it becomes selectable
   by role/model config the same way grok already is.
   SEPARATE PATH: crates/cli/src/runtime_client.rs uses AnthropicClient directly (ELAI's own
   CLI-facing chat loop) — does NOT go through ProviderClient/ProviderKind at all. Wiring the new
   provider there is out of scope unless the plan wants the new provider to BE the primary chat
   model (task says "as an ELAI provider", closer to the ProviderRuntimeClient/role-dispatch lane).
7. crates/providers/src/providers/openai_compat.rs (~1440 lines) - the OpenAiCompatConfig/Client
   seam itself. Config struct [27-32]: provider_name, api_key_env, base_url_env,
   default_base_url (all &'static str, const-constructible). xai() [39-46] and openai() [48-56]
   are the 2 existing constructors; credential_env_vars() [58-64] match arm keyed on
   provider_name string (needs a new arm for a 3rd provider). OpenAiCompatClient::from_env
   [95-103] reads api_key_env via read_env_non_empty, errors ApiError::missing_credentials if
   absent. read_base_url(config) [~1115] does std::env::var(base_url_env).unwrap_or(default_base_url).

(b) EXACT SEAM + WHAT A NEW PROVIDER IMPL MUST PROVIDE
Minimal-diff path (reuse OpenAiCompatClient, do NOT write a new HTTP client):
 - providers/mod.rs: add ProviderKind variant (name collision note: do NOT call it "Codex" —
   RuntimeBackend::CodexCli already exists in runtime/codex_cli.rs meaning "shell out to the
   codex binary", a DIFFERENT axis; reusing the name here will read as the same feature and
   isn't). add MODEL_REGISTRY row(s) for whatever model alias string will select it (e.g. a
   "gpt-5.5" or "bmapi" alias) so name-based detect_provider_kind works without relying on the
   OPENAI_API_KEY env-fallback (which would collide with real ProviderKind::OpenAi detection).
 - openai_compat.rs: add OpenAiCompatConfig::<new>() const fn (provider_name/api_key_env/
   base_url_env/default_base_url) + a credential_env_vars() match arm.
 - client.rs: add enum variant + match arms in from_model_with_anthropic_auth, provider_kind(),
   send_message(), stream_message() (send/stream arms can fold into the existing
   `Self::Xai(c) | Self::OpenAi(c)` pattern by adding the new variant to that same arm, IF the
   wire format matches Chat-Completions shape - see risk in (g)).
 - Context-window sourcing (the genuinely NEW piece): mod.rs model_token_limit()/
   max_tokens_for_model() are a fully static match table (no live fetch anywhere in this file).
   Task requires this provider's context window come from a live GET {base_url}/models call
   instead. No ProviderKind/OpenAiCompatConfig code path does this today. Nearest PRECEDENT
   (different subsystem, RuntimeAdapter health-checks, not ProviderKind) fetches /v1/models
   live: crates/providers/src/runtime/vllm.rs:123-138 (self.endpoint("/v1/models"), parses
   data array, empty/parse-error -> unavailable(...)) and runtime/sglang.rs:231-239 (identical
   shape). Model for the new fn: GET {base_url}/models (bmapi base already ends without
   trailing /v1 ambiguity - confirm exact path against the live proxy, do not assume), parse a
   models[].context_length-shaped field (exact field name UNVERIFIED - proxy's /v1/models
   response schema not read this session), fall back to the static table only if the fetch fails.

(c) ENV / CONFIG CONVENTIONS
Established pattern (openai_compat.rs 39-56): vendor-style, NOT ELAI_-prefixed:
  XAI_API_KEY / XAI_BASE_URL (xai()), OPENAI_API_KEY / OPENAI_BASE_URL (openai()).
No ELAI_CODEX_* precedent exists anywhere in providers/ for API credentials (ELAI_*_ENABLE
kill-switch flags are a different, unrelated naming family per wiring-enforcement R9).
Two defensible options for the new pair, NOT decided here (material fork for executor/orchestrator):
  (i) new distinct pair, e.g. BMAPI_API_KEY / BMAPI_BASE_URL or CODEX_PROXY_API_KEY/_BASE_URL,
      following the short-vendor-name convention exactly like XAI_*/OPENAI_*.
  (ii) reuse OPENAI_API_KEY/OPENAI_BASE_URL directly (bmapi IS an OpenAI-compat proxy) — this
      collides semantically with genuine ProviderKind::OpenAi (default_base_url
      api.openai.com) since both would read the same 2 env vars; a user cannot run both
      simultaneously. See (e) for the trust_dialog implication of each choice.
elai_cc.toml has default_provider="anthropic" / default_model="claude-sonnet-4-6" (~line 591-592)
but this config section sits amid the VRAM-tier/model-catalog block (lines ~476-559,
Qwen2.5-Coder local models) — read as sidecar/local-model config, NOT the --model CLI flag's
source; the CLI flag + MODEL_REGISTRY is the live selection path (a). No provider config-file
section exists for cloud providers (grok/openai are env-var-only today, matching the task's
"key/base_url via env, never committed" instruction already being the codebase norm).

(d) CONTEXT-WINDOW SOURCING TODAY + LIVE-FETCH SLOT
Today: 100% static. mod.rs:230-248 model_token_limit(model) match on canonical model string
literals (claude-opus-4-6, claude-sonnet-4-6/haiku, grok-3/grok-3-mini) returning
{max_output_tokens, context_window_tokens}; max_tokens_for_model() falls back to a 2-branch
opus/other heuristic if the table misses. preflight_message_request() [250-268] uses this to
reject oversized requests BEFORE sending (client.rs never called for the request). Grok's
131_072 entry is itself a hardcoded literal, not fetched from xAI. Adding a live fetch for the
new provider is a genuinely new code path in this module (no existing ProviderKind does this);
closest shape to copy is vllm.rs/sglang.rs's endpoint("/v1/models") health-check parse (they
target availability, not context length, so the response-shape assumption still needs
verification against bmapi's actual /v1/models payload — UNVERIFIED, not fetched this session).
Slot: a new async fn (e.g. fetch_context_window(config: OpenAiCompatConfig) -> Result<u32,
ApiError>) called once (cached, not per-request) at client construction or first use, feeding a
non-static variant of model_token_limit's result for this provider only — static match stays
authoritative for existing providers.

(e) TRUST_DIALOG REQUIREMENTS
crates/safety/src/trust_dialog.rs (149 lines, E-30, guards CVE-2025-59536 hook-RCE +
CVE-2026-21852 API-token-exfil-via-base-url-redirect). ensure_session_trust(project_root,
settings_path, token: Option<&TrustToken>) [51-77]: if token is None AND
(env ANTHROPIC_BASE_URL is set OR env OPENAI_BASE_URL is set) -> Err(HookLoadDeferred); if
token is None (regardless of env) -> Err(TrustDialogRequired); Some(token) -> Ok(()).
mint_trust_token(session_id) [84-93] produces the token after a real user prompt (flag-passing
does NOT count, per the file's own doc comment).
HARDCODED 2-ITEM CHECK: the redirect check is 2 literal env::var() calls (line 58-59), NOT a
loop over ProviderMetadata::base_url_env from the registry. Implication for 3.1's env choice
in (c): if the new provider reuses OPENAI_BASE_URL (option ii), it automatically inherits this
CVE gate for free. If it gets a distinct env var (option i, e.g. BMAPI_BASE_URL), that redirect
vector is INVISIBLE to ensure_session_trust unless this file is also extended with a 3rd
has_*_redirect check — a scope decision, not something 3.1 can silently skip given the file's
own CVE framing (an unguarded new base-url-redirect env var is exactly the CVE-2026-21852 shape).

(f) TEST PATTERNS + CRATE FLAG
Provider-kind/alias/model-limit unit tests live inline #[cfg(test)] in the same files: mod.rs
tests module [285-409] (resolve_grok_aliases, detects_provider_from_model_name_first,
keeps_existing_max_token_heuristic, returns_context_window_metadata_for_supported_models,
preflight_blocks/skips_*); client.rs tests [144-163] (resolves_existing_and_grok_aliases,
provider_detection_prefers_model_family); openai_compat.rs has its own large inline test module
(~130+ #[test] fns visible via elai.map grep, e.g. endpoint_builder_accepts_base_urls_and_full_
endpoints:1422-1433, parse_tool_arguments_* family) covering endpoint-building, SSE frame
parsing, tool-call translation, streaming usage opt-in. trust_dialog.rs has its own 4-test
module [100-148]. Crate: elai-providers (Cargo.toml name at line 2) -> `cargo test -p
elai-providers`. cli crate has cli_tests.rs with its own resolve_model_alias assertions
(line ~310-313) -> likely `-p elai-cli`. Cross-provider CONTRACT tests (crates/contracts/tests/)
found for role-dispatch, not for a single provider directly: adversary_contract.rs,
agent_role_dispatcher_contract.rs (both reference ProviderKind/Xai for the cross-family adversary
lane — grok is the current cross-family critic per the adversary agent type description).
No existing contracts/tests file is named after openai_compat/xai/grok directly — provider-kind
correctness is fenced only by the inline unit tests above.

(g) RISKS
1. WIRE-API MISMATCH (HIGH, most likely blocker): openai_compat.rs is hardwired to the OpenAI
   Chat Completions shape, not the Responses API. chat_completions_endpoint() [~1120-1123]
   appends "/chat/completions" to the base URL if missing; send_message()/stream_message()
   deserialize into a local ChatCompletionResponse struct (choices[].message, delta.content,
   finish_reason=="tool_calls", etc - all Chat-Completions vocabulary). conductor-j-burn.md:19
   explicitly states the bmapi proxy's wire_api is "responses" (i.e. codex-cli talks to it via
   POST /v1/responses with an `input` array + `response.*` SSE event types, NOT
   /chat/completions + `messages`). UNVERIFIED whether bmapi ALSO exposes a Chat-Completions-
   compatible endpoint at /chat/completions (many OpenAI-compat proxies expose both; some only
   do Responses). If it does NOT, plugging straight into OpenAiCompatClient will send the wrong
   request shape and 404/400, and the existing SSE parser (next_sse_frame, ContentBlockDelta
   mapping) will not understand Responses-API event types either — this would require either
   (a) confirming bmapi's /chat/completions works before ELAI's next executor writes code, or
   (b) a second parallel code path in openai_compat.rs (or a new sibling module) for
   Responses-API wire format. This is the single highest-value thing to verify before EXECUTING.
2. NAMING COLLISION: RuntimeBackend::CodexCli (runtime/codex_cli.rs) already means "spawn the
   codex binary as a subprocess" — a totally different mechanism from "call bmapi as an
   inference provider". Do not name the new ProviderKind variant/config anything containing
   "Codex" without a very clear distinguishing name; a future reader (or grep-based audit) will
   conflate the two.
3. MODEL_REGISTRY GAP: unlike grok, there is currently NO OpenAi entry in MODEL_REGISTRY at all
   (OpenAi is only reachable via the env-var fallback in detect_provider_kind). If the new
   provider is added the same way (fallback-only, keyed on OPENAI_API_KEY), it cannot coexist
   with a real OPENAI_API_KEY use case and has no stable model-name alias a user/role-config can
   request by name - it should get an explicit MODEL_REGISTRY row like grok did.
4. CONTEXT-WINDOW LIVE-FETCH IS NEW CODE, NOT A REUSE: nothing in ProviderKind/openai_compat.rs
   fetches remotely today; the two nearest precedents (vllm.rs/sglang.rs /v1/models parsing) are
   RuntimeAdapter health-checks in a different module family with a different config type
   (RuntimeFactoryConfig, not OpenAiCompatConfig) - expect genuine new code, not a copy-paste.
5. TRUST_DIALOG EXTENSION SCOPE (see e): adding a distinct BASE_URL env var silently bypasses the
   existing CVE-2026-21852 gate unless trust_dialog.rs is also touched - flag to orchestrator as
   an explicit in-scope-or-not decision before EXECUTING, since trust_dialog.rs is named in the
   plan text itself as a seam (TEIKOKU-ELAI-FINISH-PLAN.md:144).

SOURCES CONSULTED
TEIKOKU-ELAI-FINISH-PLAN.md:130-162 (plan text, wave 3 unit 3.1-3.4); workflow/handoff/
conductor-j-burn.md (full read, workstream 2 + machine/wire_api specifics); memory-bank/
activeContext.md (full); memory-bank/projectIndex.md (grepped: providers|openai_compat|
trust_dialog|RuntimeBackend); memory-bank/gotchas.md (grepped: providers|openai_compat|
trust_dialog terms, and separately codex|bmapi|gpt-5.5|wire_api - zero hits on the latter);
memory-bank/systemPatterns.md (grepped LOCK.*provider, inconclusive/truncated, not pursued
further); memory-bank/progress.md/operatingManual.md (operatingManual read full; progress.md
too large, not read this session - no provider-specific grep run against it, gap noted below);
crates/providers/src/providers/{mod.rs,openai_compat.rs:1-150} (read); crates/providers/src/
client.rs (read full); crates/safety/src/trust_dialog.rs (read full); crates/cli/src/args.rs:
2600-2670 (read); crates/cli/src/cli_consts.rs, cli_tests.rs, runtime_client.rs (grepped);
crates/tools/src/lib.rs:4365-4400 (read); crates/providers/src/runtime/codex_cli.rs:1-30 (read);
crates/providers/Cargo.toml (grepped); crates/contracts/tests/ (grepped for provider terms);
elai_cc.toml (grepped provider|model); elai.map (grepped openai_compat, trust_dialog, /v1/models
- three separate greps); mcp_fsm_get_state (called).

VALIDATION WARNINGS
FSM phase string returned SP-1A (unit 1.1), not SP-3A - shared global FSM state, likely a
parallel/stale track; not resolved here, surfaced for the orchestrator.
progress.md (588KB) and projectIndex.md (1.4MB) exceed direct-read size limits; only grepped,
not read in full - if a provider-relevant milestone exists outside the grepped terms it was missed.
bmapi's actual /v1/models response schema and whether it also serves /chat/completions were NOT
fetched this session (would require a live network call outside this read-only research scope) -
both are HIGH-severity open verifications, see (g)1 and (d).
HIGH: wire-api mismatch (g)1 is the dominant risk and should gate whether 3.1 proceeds via the
existing openai_compat.rs Chat-Completions path or needs a Responses-API-shaped sibling.

======================================================================
BLOCKED NOTE (R-4, agent-dispatch-preamble.md)
======================================================================
Triggered by: PreToolUse FSM hook rejected Write to the assigned output path
(workflow/sessions/s-finish-0705/context-package-SP-3A.md) and to the handoff_file
(workflow/handoff/teikoku-elai-finish.md), both with the same message: "FSM state is EXECUTING.
Write/Edit allowed to: src/, tests/, memory-bank/*.md, workflow/reasoning/." The global FSM
phase string reported is "SP-1A precheck routing port (unit 1.1)", not SP-3A — this looks like
a different sub-phase (wave 1) holding EXECUTING state while wave-3's context-compiler was
dispatched in parallel, and the hook's allowlist was set for THAT sub-phase's file scope
(src/tests/, i.e. code-writing), not for a context-compiler's doc-output path.
QUESTION FOR ORCHESTRATOR: this file (workflow/reasoning/SP-3A-context-package.md) holds the
complete, ready SP-3A context package. Do you want it (1) copied/moved to
workflow/sessions/s-finish-0705/context-package-SP-3A.md once the FSM is in a state whose hook
allowlist permits that path (e.g. after SP-1A's sub-phase completes or transitions), or (2) left
here and the downstream executor/prompt-engineer pointed at this path instead? Lean: (1) for
consistency with the session's existing convention (handoff file's own FILES TOUCHED line already
names workflow/sessions/s-finish-0705/* as this run's output area), but (2) is zero-cost if the
hook keeps blocking workflow/sessions/ writes for parallel-dispatched agents generally — worth
checking whether other parallel sub-phase agents (e.g. wave 2's context-compiler) hit the same
wall, since if so this is a recurring friction point for the whole orchestrate run, not a one-off.
HOW TO PICK UP: content above is complete and self-contained (sections a-g + sources +
warnings); no further research needed to unblock SP-3A's EXECUTING step. Only the file's final
resting place is open.
