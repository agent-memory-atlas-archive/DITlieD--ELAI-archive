STEAL ANALYSIS: goose (Block, Apache-2.0) + reference-CLI feature digest
Date: 2026-06-12. Session: paper-to-plan phases 4-5 resume (Fable), per workflow/handoff/paper-to-plan-goose-factory.md.

Inputs (the only three phase-4 reads, per handoff):
  docs/.source/goose-main-INDEX.md (merged 10-part goose map, file:line grounded)
  workflow/papers/reference-cli-feature-digest.md (sanitized neutral digest; weak grounding, ideas only, clean-room)
  workflow/papers/elai-token-usage-current-state.md (ELAI gap map, codegraph-grounded, 2026-06-12)
Live baseline re-grepped this session (doc numbers 231/542 were stale):
  SCHEMA_VERSION = 234 (crates/orchestrator/src/ssb.rs:3096)
  TOTAL_VARIANTS = 543 (crates/orchestrator/tests/action_type_lock.rs:676)
Feature-map ledger (Section 4) checked before every NEW/EXTENSION verdict. Ledger entries that bind this run:
  device-code OAuth: NOT WORTH PORTING (MiMoCode run) -> no OAuth arms in BYOK here; OAuth stays I-50.
  models.dev-style dynamic provider catalog: NOT WORTH PORTING -> I-51 static models.toml stays canonical; OpenRouter-fetch registry not adopted.
  gajae wholesale provider port: REJECTED, 3 protocol shapes survive as I-49 -> wire-format normalization is ALREADY-PLANNED territory.
  A3 rate-limit/backoff: I-10 covers -> goose retry.rs not a unit.
  LLM-observability platforms: NO PORTS, SSB is the record, composition = E-29 OTLP exporter -> OTEL counter steal not a unit.
Clean-room rule: ideas and features only; no goose or reference-tool identifiers enter ELAI code. ELAI-native names invented per steal (UsageTurnEvent, NormalizedUsage, usage rollup, live usage gauge, UsageWindow, ProviderByokEvent, custom_providers.toml). Reference digest is weak grounding by its own header: it nominates features; all plan grounding below is goose file:line + ELAI file:line.

Verdict key: NEW | EXTENSION(plan-id) | ALREADY-SHIPPED(id) | ALREADY-PLANNED(id) | ALREADY-REJECTED(ledger) | REJECT(reason). Plan mapping at the end groups NEW/EXTENSION into J-28/J-29/J-30.

================================================================
CLUSTER 1 - TOKEN-CONSUMPTION WINDOW (primary deliverable)
================================================================

S-01 Per-turn four-counter usage record, cached vs non-cached separated (digest S1; goose goose-providers/src/conversation/token_usage.rs:26-107).
  ELAI: TokenUsage already has exactly the four fields (crates/core/src/usage.rs:15); CloudSpendPayload carries cache fields (ssb.rs:11522). Map: COST COCKPIT [PARTIAL] H-9.
  Verdict: ALREADY-SHIPPED(C-14/H-9) at capture. The gap is per-turn SURFACING (no UI shows the per-turn record) -> J-29.

S-02 Option-typed counters + sum_optionals (None+None=None, never conflate 0 with unreported) (token_usage.rs:26-107, 35-45).
  ELAI: TokenUsage uses u32 zero-defaults; a backend that reports nothing is indistinguishable from a backend that reported zero. Map: COST COCKPIT H-9.
  Verdict: EXTENSION -> J-28. Adopt Option semantics ONLY in the new normalized rollup layer; do NOT churn the shipped TokenUsage struct (blast radius across SSB payloads).

S-03 Client-side usage estimator fallback: if provider reports no usage, count locally; tool-schema token cost model; blake3-keyed LRU token cache (usage_estimator.rs:9-47; token_counter.rs:75-171, 22-73).
  ELAI: L-3 ABSENT (no tokenizer-based count outside generation; audit section 6). Map: PYTHON SIDECAR phase-3J, MULTI-RUNTIME BACKEND B-37.
  Verdict: NEW -> J-28. Estimates must be per-lane and tagged ESTIMATED (Qwen tokenizer differs from any cloud tokenizer; an estimate is a labeled approximation, never silently mixed with reported counts).

S-04 Dual per-turn + cumulative session fields, separate so display never confuses them (session_manager.rs:56-89).
  ELAI: UsageTracker already wraps cumulative + latest_turn (usage.rs:34).
  Verdict: ALREADY-SHIPPED(C-14). The missing piece is per-SESSION COST total (audit C-2: task_usd = most-recent row only) -> J-28 rollup.

S-05 Per-chunk cost accumulation against a per-model pricing catalog + model-name normalization before lookup (reply_parts.rs:557-575; canonical/name_builder.rs).
  ELAI: SpendAccumulator::for_model() hardcodes claude-sonnet-4-6 rates for every other model id (cloud_mode.rs:686-691, audit C-1); pricing.rs/models.toml exists but SpendAccumulator does not call it. Map: COST COCKPIT H-9, MODEL CAPABILITY CATALOG [PLANNED(I-51)].
  Verdict: EXTENSION(H-9, composes I-51) -> J-28. SpendAccumulator consults pricing_for_model/models.toml; family-alias normalization on lookup.

S-06 Compaction accounting rule: summarization output tokens become new input-context tokens; cumulative fields keep growing (reply_parts.rs:529-539).
  ELAI: auto-compaction exists (conversation.rs:171) with no accounting rule. Map: CONTEXT BUDGET ALLOCATOR phase-3E; H-50 context lifecycle [PLANNED].
  Verdict: EXTENSION rider folded into J-28 normalization rules (note composition with H-50; not a separate unit).

S-07 Usage snapshot embedded in EVERY stream event so the UI never polls (goose-server routes/reply.rs:129-138, 156-177).
  ELAI: C-5 ABSENT - token counts land only after a complete turn; dashboard polls 5-10s; no mid-stream progress. Map: COST COCKPIT H-9; DASHBOARD 1.10.
  Verdict: NEW -> J-29. ELAI-native shape: in-process live usage gauge (atomic counters updated by the stream consumer) + Tauri push/command for the dashboard chat + CLI tick line. Mid-stream ticks are NOT SSB rows (journal bloat); the per-turn row stays the durable record.

S-08 One usage-rollup surface keyed by (user, model, task, tool, file-op), fed by one aggregation endpoint (digest S2).
  ELAI: pricing_rollup per-phase/agent/tool implemented but DARK, zero consumers (observability/pricing_rollup.rs, audit C-3); BackendCostAccrued payload lacks model/task/session keys (ssb.rs:18477, audit 3.2); no session key anywhere (C-2); no identity key (C-7). Map: COST COCKPIT H-9, COMPOSER2 BACKEND C-44.
  Verdict: NEW -> J-28 (the keystone). One new uniform per-turn row (UsageTurnEvent) emitted from BOTH lanes carrying {identity(default local), lane, backend, model, task_id, session_id, 4 token counters, est_cost_micros, elapsed_ms, semantics tag} + a read-side rollup over it. J-29 consumes it.

S-09 Abstract provider-agnostic billing unit (credit-style number layered over tokens) (digest S2).
  Verdict: REJECT(display honesty). ELAI is BYOK/local-first; a synthetic unit obscures the real bill. Display = raw tokens + USD where a real rate exists, tokens-only where it does not (S-12). Revisit only if a multi-tenant story appears.

S-10 Autonomy ratio (share of work done without human approval) as a usage-dashboard stat (digest S2).
  ELAI: derivable read-side from existing SSB rows (gate/approval/critic events vs auto-executed steps); no producer change. Map: SSB JOURNAL 1.4.
  Verdict: NEW (small, read-side only) -> J-29.

S-11 CLI cost line after each response ($N.NNNN + in/out split) + colored context bar with formatted counts (goose-cli output.rs:1434-1445, 1367-1415).
  ELAI: C-6 - format_cost_report prints raw token counts only, no dollars (resume.rs:328-342, live_cli.rs:2683). Map: COST COCKPIT H-9.
  Verdict: EXTENSION(H-9) -> J-28 (CLI surface is J-28's in-plan consumer; window is J-29's).

S-12 Local lane displays token counts, never a fake $0.00 (CostTracker.tsx:19 free-provider rule).
  ELAI: matches L-2 reality (cost_model_for(Sidecar)=None, est_cost_micros=0).
  Verdict: ADOPT as display rule -> J-29.

S-13 Dedicated usage window: per-model breakdown, cost split tooltip, context indicator with absolute counts (CostTracker.tsx:1-200; ContextWindowIndicator.tsx:1-41).
  ELAI: CloudPanel/SpendDrawer show aggregate day/task only, PER-MODEL: NO (audit 4.1-4.2); BackendCostPanel ABSENT (4.6); ContextStatePanel shows pct only, no absolute count (5.2). Map: COST COCKPIT H-9, COMPOSER2 BACKEND C-44, CONTEXT BUDGET ALLOCATOR phase-3E.
  Verdict: NEW -> J-29 (UsageWindow: per-model/per-task/per-session charts, cache hit-rate, autonomy ratio, tool-call + file-op counts; BackendCostAccrued hash->name resolver Tauri cmd activates C-44's dark rows; pricing_rollup gets its first consumer, closing C-3).

S-14 Canvas usage surface (user requirement; no goose equivalent - goose has no canvas).
  ELAI: product CANVAS WORKBENCH [PARTIAL, Track 2] with canvas_* ops via E-40 CanvasDispatcher.
  Verdict: NEW (small leg) -> J-29: usage badge/overlay op on the product canvas; harness canvas (.claude/mcp/canvas) gets at most an optional rider, not a unit.

S-15 OTEL monotonic token/tool counters per turn (reply.rs:420).
  Verdict: ALREADY-PLANNED(E-29 OTLP exporter path) per ledger observability verdict; SSB stays the record. No unit.

S-16 Per-call wall-clock latency stored with usage (audit C-9/L-6: sidecar returns elapsed_ms, server.py:4984, ELAI drops it).
  Verdict: NEW field (elapsed_ms) inside J-28's UsageTurnEvent; not a separate unit.

================================================================
CLUSTER 2 - MULTI-PROVIDER BYOK
================================================================

S-17 Provider trait + Arc-closure factory registry (base.rs:460+, provider_registry.rs:107-358).
  ELAI: MULTI-RUNTIME BACKEND [WIRED] B-37/phase-4M - RuntimeAdapter trait, 11 backends, PerRoleRuntime, resolve_backend.
  Verdict: ALREADY-SHIPPED(B-37).

S-18 Declarative custom-provider overlay: add a provider via config file, no code change (declarative_providers.rs:34-100 - name, engine, api_key_env, base_url, models).
  ELAI: no config-file backend registration; backends are code-defined. Map: MULTI-RUNTIME BACKEND B-37.
  Verdict: NEW -> J-30. custom_providers.toml (TOML, house style) with engine = anthropic-compat | openai-compat mapping onto the B-37 factory; covers self-hosted/vLLM/LiteLLM-style endpoints without new backend code.

S-19 Two-tier secret storage (OS keyring primary, 0600 file fallback) + get_secret lookup order env -> store -> file (config/base.rs:369-386, 627-658, 730-735).
  ELAI: keys are env-vars only today; SUBSCRIPTION AUTH BROKER [PLANNED(I-50)] owns OAuth/subscription auth with token-material-never-journaled doctrine. Map: I-50.
  Verdict: EXTENSION(I-50) -> J-30. Default = 0600-mode file store (zero new deps); OS-keyring upgrade is a flagged option requiring explicit user dep confirmation (keyring crate; anti-slopsquat verify name/version/maintenance live at author time). Env-first precedence preserved. No OAuth arms here (ledger: device-code OAuth NOT WORTH PORTING; OAuth = I-50's AuthProvider).

S-20 Base-URL override per provider via env/config (anthropic.rs:67-74, openai.rs:88-101, ollama.rs OLLAMA_HOST).
  ELAI: ANTHROPIC_BASE_URL already canonical for the sidecar lane. Map: PYTHON SIDECAR phase-3J, B-37.
  Verdict: EXTENSION(B-37) -> J-30: generalize per-backend base_url (config + env), not just the Anthropic lane.

S-21 Declarative per-provider setup schema: config keys, required/secret/primary flags, setup steps; drives wizard + masked key UI (base.rs:194-370; DefaultProviderSetupForm.tsx:54-58 masking).
  ELAI: no per-backend config descriptor. Map: B-37, I-50.
  Verdict: EXTENSION(B-37) -> J-30: per-backend config descriptor (keys, base-url, flags) consumed by CLI configure + dashboard panel; skip oauth_flow/device_code_flow arms (ledger).

S-22 Interactive configure flow ending in a real test call + atomic save (configure.rs:675-812, 7 steps).
  ELAI: config refactor landed under E-33; no backend-verify test call in a configure flow. Map: B-37; dashboard 1.10.
  Verdict: EXTENSION -> J-30, scoped down: select backend -> enter key (masked) -> optional base-url -> live test call -> atomic save (tmp+rename+fsync, goose config/base.rs:572-608 pattern as implementation hygiene).

S-23 Provider inventory with configured/models/stale status for UI polling (goose-sdk-types custom_requests.rs:1315-1379).
  ELAI: RuntimePanel exists (B-37) but no configured/verified/stale inventory shape. Map: B-37.
  Verdict: NEW (small) -> J-30: backend inventory status (configured? verified-at? models from models.toml) surfaced in the provider panel.

S-24 Active-provider selection + per-turn lock: once a turn/sequence starts, the provider is locked; no mid-sequence wire-format switch without an explicit serialize step (digest S3).
  ELAI: PerRoleRuntime/resolve_backend picks per role; turn-scoped lock semantics not explicit. Map: B-37, CALIBRATED ROUTING H-7/E-31.
  Verdict: EXTENSION(B-37) -> J-30: turn-scoped backend lock in RuntimeOrchestrator; mid-task escalation stays the existing EscalateToCloud path (S-30).

S-25 One internal tool-spec normalized to N provider wire formats at call time (digest S3).
  ELAI: BEDROCK + AZURE LANES [PLANNED(I-49)] already covers SigV4 + EventStream, Azure Responses shape, OpenAI-as-Anthropic shim.
  Verdict: ALREADY-PLANNED(I-49). Gemini wire shape is not in I-49 - recorded as an optional I-49 rider, user-demand gated (ledger precedent: niche clients user-demand only). Not a unit here.

S-26 Retry + exponential backoff + jitter + dedicated auth-refresh retry (retry.rs:14-244).
  Verdict: ALREADY-PLANNED(I-10) per ledger (A3 rate-limit/backoff); auth-refresh hook belongs to I-50. No unit.

S-27 Canonical model registry fetched from OpenRouter at release time (canonical/registry.rs, build_canonical_models).
  Verdict: ALREADY-REJECTED(ledger: models.dev dynamic providers NOT WORTH PORTING; I-51 static models.toml is canonical). Manual catalog refresh stays the process.

S-28 mTLS client certificates for providers (api_client.rs:46-117).
  Verdict: REJECT(no ELAI user story; enterprise-perimeter feature; adds config surface with zero current demand).

S-29 Provider-error enhancement: on 4xx model-availability errors, append recommended models from the catalog (reply_parts.rs:27-49).
  Verdict: small UX rider on J-30 (suggest from models.toml on backend 4xx); optional leg, not a unit.

S-30 Mid-session model escalation with per-model usage attribution across the swap (digest S4).
  ELAI: EscalateToCloud exists and fires on local->cloud promotion (ssb.rs:11542, audit 3.6); per-model attribution exists in CloudSpendPayload.model; per-turn model routing = CALIBRATED ROUTING [PARTIAL] H-7/E-31.
  Verdict: ALREADY-SHIPPED(escalation) + ALREADY-PLANNED(H-7/E-31 routing). J-28's UsageTurnEvent makes the per-model attribution VISIBLE across swaps; no new escalation mechanics.

================================================================
CLUSTER 3 - LOCAL-MODEL LANE
================================================================

S-31 Streaming usage assembled from BOTH message_start (input side) and message_delta (output side) (goose handles split-event usage; ollama.rs:77-106 fallback chain).
  ELAI: THE keystone local bug-class. anthropic.rs observe_event reacts only to MessageDelta (anthropic.rs:1313-1334); sidecar streaming message_start carries input_tokens but is not accumulated, so streaming-local AssistantEvent::Usage has input_tokens=0 (audit L-1, sections 1.3-1.4). Map: PYTHON SIDECAR phase-3J, COST COCKPIT H-9.
  Verdict: NEW-FIX -> J-28 (first leg). Accumulate MessageStart usage into the provider's latest_usage; audit explicitly flags the SSE-to-Usage assembly path for confirmation at author time - plan carries a VERIFY pre-gate on anthropic.rs:1313 + server.py:4728/4856.

S-32 Full-prompt vs suffix-only input_tokens semantics divergence must be normalized before any cross-lane display (audit L-5; server.py:4959-4963 comment confirms intentional divergence from Anthropic billing definition).
  Verdict: NEW -> J-28 core: NormalizedUsage with an explicit per-lane semantics tag; two candidate fixes decided at author time: (a) sidecar emits spec-compliant suffix-only + cache fields (server.py change; cross-language pair rule applies, runtime_client.rs read path must move in the same unit), or (b) client-side normalization keyed by lane tag. Never naively sum cloud + local (handoff KNOWN LIMITS line; digest S4 caveat).

S-33 stream_options conditional feature detection - probe/feature-flag unsupported request fields instead of timing out (ollama.rs:77-95).
  Verdict: implementation note inside J-30's openai-compat engine (only relevant once custom endpoints land). Not a unit.

S-34 Retry tuned for cold model load (max_retries=10, 2-15s backoff) (ollama.rs:48-51).
  ELAI: sidecar boot health protocol exists (3-step check, sidecar-protocol skill); request-level backoff = I-10.
  Verdict: ALREADY-SHIPPED(boot protocol) / ALREADY-PLANNED(I-10). No unit.

S-35 Context-window mapping env -> request options (GOOSE_INPUT_LIMIT -> options.num_ctx) (ollama.rs:64, 97-122).
  ELAI: ELAI_LOCAL_MAX_CONTEXT already wired (phase-3J; E-32 decisions).
  Verdict: ALREADY-SHIPPED(phase-3J).

S-36 XML tool-call fallback parser for Qwen-family + toolshim post-processing for non-tool-aware models (formats/ollama.rs:35-79; reply_parts.rs:269, 310-348).
  ELAI: STRUCTURED OUTPUT SUBSTRATE [PARTIAL] E-35/E-18 already plans tool_parser.py (absent, RUNPOD-PENDING) + XGrammar masks.
  Verdict: EXTENSION rider on E-35 (recorded here, not re-authored): XML-tag fallback parse as defense-in-depth alongside XGrammar when the local model emits tag-style tool calls. No new unit.

S-37 In-process inference runtime singleton with per-model cache (local_inference.rs:68).
  Verdict: REJECT(architecture mismatch: ELAI's local lane is the REMOTE sidecar by design, phase-3J canonical "never local"; J-19 vLLM is the planned alternative backend).

S-38 Local GPU-time cost model (neither goose nor the digest has one; goose treats local as free).
  ELAI: L-2 - est_cost_micros=0 for local lanes by design.
  Verdict: REJECT for now (a synthetic $/GPU-hour invents a number; display tokens for local per S-12). Future rider: RunPod-billed pods have a REAL hourly rate via pod-lifecycle - if/when that lane matters, attribute real pod cost, not a synthetic rate.

================================================================
OUT-OF-FOCUS / NO-STEAL (from goose-main-INDEX section 5 + themes D/E/F)
================================================================

S-39 goose multi-agent topology (sequential blocking delegation, no parallelism, no per-role routing; subagent_handler.rs:48): REJECT - strictly simpler than ELAI's FSM-gated parallel orchestrate (INDEX section 1 + 5 verdict).
S-40 Session fork/truncate time-travel (goose-server session.rs:338-410): ALREADY-PLANNED(H-49 reversibility/undo-tree). Confirmation only.
S-41 Middle-out tool-response deletion + post-compaction continuity prompt (context_mgmt/mod.rs:233-279, 30-43): ALREADY-PLANNED(H-50 context lifecycle); recorded as design references on H-50. No unit.
S-42 Recipe framework, skill system, three-tier slash dispatch (theme E): REJECT(out of focus; ELAI plan files + skills already cover; INDEX section 5).
S-43 Permission persistence w/ blake3 context hash + LLM read-only judge (permission_store.rs:11-80): REJECT(covered - rules engine + E-30 merge chain per MiMoCode ledger line; wiring-enforcement owns the gate boundary).
S-44 Pattern+ML threat detection (~40 regex + classifier): REJECT(out of focus; G-1/F-3/VibeSec own ELAI's threat surface; INDEX flags it non-priority).
S-45 Gateway/Telegram 6-digit pairing, dictation/Whisper, Nostr export, uniffi SDK, ACP proc-macro, Langfuse layer: REJECT(out of scope / INDEX section 5; observability = E-29 OTLP path per ledger).
S-46 Cost display gated default-OFF with no UI toggle (ChatInput.tsx:1634): DO-NOT-COPY exhibit. ELAI ships usage display ON by default - which also keeps J-28/J-29/J-30 at ZERO new env gates (wiring R9: no default-OFF graveyard).
S-47 LRU agent pool with per-session creation locks (execution/manager.rs:35-150): REJECT(agent_pool.rs exists; no demonstrated duplicate-spawn problem).
S-48 Stop-hook block cap / retry manager (agent.rs:361): ALREADY-SHIPPED(harness anti-spiral + auditor cycle caps).
S-49 Tool-argument schema coercion ("true"->true) (reply_parts.rs:101-122): rider candidate on C-26 typed-retry/grammar path; recorded, no unit.
S-50 Per-user multi-tenant accounting (digest S2 org rollups): REJECT(scope) - ELAI is single-user (audit C-7). J-28 carries an identity FIELD (default "local") in UsageTurnEvent so the rollup key never needs a schema break if identity ever becomes real.

================================================================
PLAN MAPPING (phase 5 authors these; fewest files = 3)
================================================================

J-28 TOKEN LEDGER - usage unification + per-turn event + rollup core (Rust data layer + CLI consumer).
  Legs: S-31 streaming input_tokens fix (keystone, VERIFY pre-gate on anthropic.rs:1313 + server.py:4728/4856) -> S-32 NormalizedUsage + lane semantics tag (cross-lang pair rule if server.py changes) -> S-08 UsageTurnEvent uniform per-turn row from both lanes (+S-16 elapsed_ms, +S-50 identity field) -> S-05 SpendAccumulator consults models.toml pricing (C-1 fix) -> S-04 session-scoped rollup (C-2 fix) -> S-03 estimator fallback (tagged ESTIMATED) -> S-02 Option semantics in rollup layer -> S-06 compaction accounting rule -> S-11 CLI dollar+bar surface (in-plan consumer; C-6 fix).
  SSB: +UsageTurnEvent, Pattern-2 single-variant, 3-arm kind (TurnRecorded / EstimateApplied / NormalizationApplied). SCHEMA_VERSION 234->235, TOTAL_VARIANTS 543->544 (re-grep at implementation time).
  No new env gate (on by default per S-46).

J-29 USAGE WINDOW - real-time surfaces (dashboard window + chat + canvas; read-side).
  Legs: S-13 UsageWindow.svelte (per-model/per-task/per-session, cache hit-rate, tool+file-op counts; BackendCostAccrued hash->name resolver cmd = C-44 activation; pricing_rollup first consumer = C-3 activation) -> S-07 live usage gauge + mid-stream push (C-5 fix; no SSB writes mid-stream) -> S-12 local-lane tokens-not-dollars display rule -> S-10 autonomy ratio (read-side) -> S-14 product-canvas usage badge (compose E-40 dispatcher) -> S-15 absolute token count in context display (phase-3E surface gap, audit 5.2).
  SSB: zero new action types (consumes J-28 rows + existing rows).
  No new env gate.

J-30 PROVIDER BYOK - key store + custom endpoints + configure flow (extends B-37/I-50, never duplicates I-49/I-50 scope).
  Legs: S-19 two-tier key store (0600 file default, keyring crate optional behind explicit user dep confirmation; env-first order) -> S-18 custom_providers.toml declarative backends (anthropic-compat/openai-compat engines onto B-37 factory; S-33 field-probe note) -> S-20 per-backend base-url override -> S-21 per-backend config descriptor -> S-22 configure flow w/ live test call + atomic write -> S-23 inventory status surface -> S-24 turn-scoped backend lock -> S-29 4xx model-suggestion rider (optional).
  SSB: +ProviderByokEvent, Pattern-2 single-variant, 3-arm kind (KeyStored / EndpointVerified / CustomBackendRegistered); key material NEVER in the payload (I-50 doctrine). SCHEMA_VERSION 235->236, TOTAL_VARIANTS 544->545.
  No new env gate (user-invoked configuration feature).

Riders recorded (NOT new units, GD-1 discipline): S-06 compaction accounting (H-50), S-25 Gemini wire shape (I-49, user-demand), S-36 XML tool-call fallback (E-35), S-40 fork/truncate (H-49), S-41 trim strategies (H-50), S-49 arg coercion (C-26), S-38 RunPod real-cost attribution (pod-lifecycle, future).

================================================================
FIVE-MULTI-AGENT-STRATEGIES CLASSIFICATION (user wrap question)
================================================================

ELAI = CREATOR-VERIFIER on a DELEGATION backbone.
  Creator-Verifier: the executor builds; verification is structurally separated and off-agent - auditor is the sole test runner (main-llm-role.md), cross-family adversary D-13 generates counter-evidence from a DIFFERENT model family, wiring gates measure artifacts the creating agent did not author (wiring-enforcement R4/R8).
  Delegation backbone: the orchestrator decomposes and dispatches scoped worker agents (context-compiler / explorers / executor / auditor / memory-writer), hierarchical parent->child, contexts garbage-collected on return; parallel cap 3; goose by contrast is pure SEQUENTIAL delegation (INDEX part-5 verdict) - same family, less structure.
  NOT Direct Communication: no agent-to-agent peer channel exists; all coordination is orchestrator-mediated.
  NOT Negotiation: agents never bargain; forks are escalated to the orchestrator/user via the handoff loop (R-4), decided, and handed down.
  Broadcast flavor only: the SSB journal is one-writer-many-readers (agents and dashboard READ shared state), which resembles a broadcast board - but no agent writes messages TO other agents through it, so it is shared observability, not Broadcast coordination.
  One line: "Creator-Verifier on a Delegation backbone, with an SSB journal that adds a Broadcast-style shared board (one writer, many readers) but no true peer messaging."

OUTCOME (phase 5, 2026-06-12, Fable): J-28_token_ledger_usage_unification.md + J-29_usage_window_realtime_surfaces.md + J-30_provider_byok_custom_endpoints.md authored under docs/thefinalglance/J-plan/ (GSW-1/GSW-2 waves); J-0_INDEX.md + J-SUMMARY_INDEX.md updated 27 -> 30 sessions with the LIVE 234/543 constants block; ELAI-FEATURE-MAP.md upserted (TOKEN LEDGER J-28 + USAGE WINDOW J-29 + PROVIDER BYOK J-30 PLANNED entries, J-family ledger line, Section-4 goose+reference verdict block, header stamp 234/543).
