# MiMoCode Part 4: Multi-Provider Abstraction, ACP Protocol, Auth, and Sharing

## Overview

This partition (provider/, acp/, auth/, share/) implements MiMoCode's **multi-provider model abstraction**, **Agent Client Protocol** compliance, **auth persistence**, and **session sharing**. The architecture prioritizes vendor-agnostic composition over hard-coded provider logic, with per-provider transform rules layered on top.

---

## Per-File Summary

### provider/provider.ts (1788 lines)
**Purpose**: Multi-backend model abstraction + provider lifecycle.

**Key Exports**:
- `Service` — Effect service providing `list()`, `getProvider()`, `getModel()`, `getLanguage()`, `resolveModelRef()`, `closest()`
- `Info` — provider config (id, name, source, env, key, options, models)
- `Model` — model metadata (id, cost, limit, capabilities, variants)
- `Interface` — contract for provider queries

**Mechanism**:
Provider state initialization loads models.dev catalog (default ~50 providers/2000 models) and layers: (1) env vars, (2) auth.json keys, (3) mimocode.json config overrides, (4) plugin auth loaders, (5) custom provider transforms (anthropic, openai, github-copilot, aws-bedrock, azure, gitlab, cloudflare, etc.). Each provider maps to an npm package (ai-sdk or custom). SDK discovery dynamically imports via pathToFileURL + dynamic import; models cached by hash(providerID, npm, options). Special handling: Anthropic beta headers, OpenAI Responses API vs. Chat API branching, Bedrock region prefixing, GitLab workflow model discovery, Azure ADFS auth. Fetch wrapper injects: chunk timeout (8m default, configurable), signal merging (user + timeout + request-level), SSE stall detection, and itemId stripping for OpenAI Batch API. Cost tracking per model (input/output/cache_read/cache_write) + experimentalOver200K tier. Zero-config fallback to defaults (context window 1M, output 0) with once-per-provider warnings.

### provider/models.ts (200+ lines)
**Purpose**: Model catalog fetching from models.dev, caching, and schema validation.

**Key Exports**:
- `Model` — zod schema (id, name, cost, limit, modalities, experimental.modes)
- `Provider` — provider entry structure
- `get()` — fetch/cache models.dev JSON (5m TTL)

**Mechanism**:
Lazy-loads https://models.dev model database (or custom URL via env). Validates against zod schemas. Caches in `~/.cache/mimocode/models*.json` keyed by source hash. Models include per-provider experimental modes (e.g., "reasoning" variant on gpt-5 with override cost/headers). `experimental.modes` allows modal-variant model trees without schema explosion. Flock-based concurrent write guard.

### provider/transform.ts (1323 lines)
**Purpose**: Per-provider message/schema/option transforms.

**Key Exports**:
- `message()` — normalizes messages (empty content, tool ID scrubbing, Anthropic tool ordering, Mistral tool ID length)
- `applyCaching()` — injects cache_control markers at 4 stable prefix boundaries (last system, midpoint, before-last-user)
- `supportsCacheMarkers()` — true for Anthropic SDKs, Bedrock, multi-provider bridges on Claude models only
- `temperature()`, `topP()`, `topK()` — per-model sampling defaults
- `variants()` — reasoning effort enums (low/medium/high/max/xhigh per provider)
- `options()` — provider-specific request fields (store=false for OpenAI, promptCacheKey for sessions, enable_thinking for DashScope)
- `schema()` — flattens discriminated unions for OpenAI (rejects anyOf/oneOf at root), converts integer enums to strings for Gemini

**Mechanism**:
Fixes provider API mismatch: (1) **message normalization** catches provider-specific quirks (Anthropic rejects empty content; Claude tool IDs must be alphanumeric; Mistral ≤9 chars; tool-use blocks must precede non-tool content in Anthropic). (2) **cache markers** placed at stable prefix boundaries to maximize hit rate (system prompt > conversation midpoint > last user). Respects provider support via SDK npm package + model name heuristic. (3) **variant switching** exposes reasoning effort as per-model enums (OpenAI: none/minimal/low/medium/high/xhigh; Anthropic: low/medium/high/max; Gemini: low/high; provider-variant-specific). (4) **schema flattening** converts `z.discriminatedUnion` (anyOf/oneOf root) into flat object with enum discriminator + per-variant hints in description — required because OpenAI/Bedrock reject combiner keywords at top level. Gemini-specific sanitizer converts integer enums to strings (Gemini rejects numeric enum values). Unsupported modalities converted to error text. Image size/count limiting via flags.

### provider/auth.ts (150 lines)
**Purpose**: Provider-specific auth (API keys, OAuth tokens, WellKnown credentials).

**Key Exports**:
- `customFetch()` → hooks into provider SDK initialization for bearer token injection
- Custom loaders for Azure (resource name env var resolution), Bedrock (AWS credential chain), GitHub Copilot, etc.

**Mechanism**:
Auth data flows from: (1) provider.options.apiKey, (2) env var (OPENAI_API_KEY, etc.), (3) auth.json store (set via `mimocode auth <provider>` CLI), (4) plugin auth loaders (e.g., GitLab Duo plugin loads token via oauth). Per-provider customFetch wraps SDK initialization to inject credentials before HTTP calls (e.g., AWS_BEARER_TOKEN_BEDROCK for Bedrock).

### provider/schema.ts (50 lines)
**Purpose**: Branded string types for ModelID, ProviderID.

**Mechanism**:
Ensures type safety at compile-time (ProviderID.make(), ModelID.make() validate schema).

---

### acp/README.md (175 lines)
**Purpose**: Agent Client Protocol (ACP) compliance documentation.

**Key Concepts**:
- ACP = JSON-RPC over stdio. Clients (Zed, Claude Code) request agent capabilities (files, terminal, prompts).
- MiMoCode implements ACP Agent interface: `initialize()`, `session/new`, `session/load`, `session/prompt`, responds with content blocks + stop reasons.
- Maps ACP sessions → internal opencode sessions, MCP servers per session.

**Mechanism**:
ACP Server (server.ts) starts JSON-RPC listener, dispatches to Agent (agent.ts). Agent.initialize() advertises capabilities, session/new creates InstanceState + SessionPrompt, session/prompt runs SessionPrompt.prompt() (returns Effect). Client (client.ts) auto-approves file read/write + permission requests. Session (session.ts) stores ACPSessionState (id, cwd, mcpServers[], model, variant, modeId). Not yet implemented: streaming (currently blocks until complete), tool-call reporting, session persistence (session/load is stub), terminal support.

### acp/agent.ts, acp/client.ts, acp/session.ts
**Purpose**: ACP implementation pieces.

**Mechanism**:
- **agent.ts** — implements `Agent` interface, routes initialize/session/* methods to Effect handlers.
- **client.ts** — implements `Client` interface for file ops (readTextFile/writeTextFile via AppFileSystem), permission requests (auto-approve).
- **session.ts** — creates ACPSessionState, maps to internal Session, initializes MCP servers list.

### acp/types.ts (25 lines)
**Purpose**: ACPSessionState, ACPConfig types.

---

### auth/index.ts (98 lines)
**Purpose**: Persistent auth storage service.

**Key Exports**:
- `Service` — Effect service: `get(providerID)`, `all()`, `set(key, info)`, `remove(key)`
- `Info` — union of Oauth | Api | WellKnown (discriminated by type field)
- `Oauth` — refresh + access tokens + expiry
- `Api` — raw API key + optional metadata (accountId, enterpriseUrl)
- `WellKnown` — key + token (for static providers)

**Mechanism**:
Reads/writes `~/.config/mimocode/data/auth.json` (mode 0o600, private). Schema-validates with Effect + Zod. Falls back to `MIMOCODE_AUTH_CONTENT` env var JSON if set. Normalizes keys (strips trailing slashes). Thread-safe via Effect monad.

---

### share/share-next.ts (300+ lines)
**Purpose**: Session sharing to remote API (legacy `share` + new `console` API).

**Key Exports**:
- `Service` — Effect service: `init()`, `url()`, `request()`, `create(sessionID)`, `remove(sessionID)`
- `Share` — { id, url, secret }
- `Data` — union of session | message | part | session_diff | model (queued for sync)

**Mechanism**:
Maintains in-memory queue per session (Map<SessionID, Map<string, Data>>). Syncs to remote API after 1s delay (debounced). Sends session snapshot, message log, file diffs, model list. Two API flavors: legacy `POST /api/share` and new `POST /api/shares`. Secret-based access. HTTP via Effect/unstable http-client. Disabled via `MIMOCODE_DISABLE_SHARE=true`. Database-backed via SessionShareTable (share.sql.ts).

### share/session.ts (58 lines)
**Purpose**: Session-level sharing API (wrapper around share-next).

**Key Exports**:
- `Service` — Effect service: `create()`, `share(sessionID)`, `unshare(sessionID)`

**Mechanism**:
- `create()` — creates session, auto-shares if `MIMOCODE_AUTO_SHARE=true` or config `share: auto`, returns sessionID.
- `share()` — calls shareNext.create(), runs SyncEvent.Updated to update session UI.
- `unshare()` — calls shareNext.remove(), clears share URL.

---

## Cross-References

### Composition
1. **provider/provider.ts** imports **provider/models.ts** for catalog, **provider/transform.ts** for per-provider tweaks, **provider/auth.ts** for custom SDK setup.
2. **acp/agent.ts** imports **provider/schema.ts** to accept providerID/modelID in session state.
3. **auth/index.ts** is consumed by **provider/provider.ts** at initialization (load auth keys).
4. **share/share-next.ts** depends on **share/session.ts** for session lifecycle hooks.

### Data Flow
- **Config → Provider**: mimocode.json provider overrides merged into models.dev base.
- **Auth → Provider**: Auth.Service.get(providerID) injects keys into SDK options.
- **Provider → ACP**: ACP Agent.initialize() advertises available models via Provider.list().
- **Session → Share**: SessionShare.create() hooks share-next to auto-queue sync events.

---

## Portable Mechanisms for ELAI Rust Orchestrator

### 1. Per-Provider Option Remapping (transform.ts lines 40-63, 1072-1111)
**Mechanism**: Map provider package → SDK namespace. OpenAI-SDK expects `providerOptions.openai`, but stored key is `providerID="openai"`. Remap at request-time.

**ELAI Steal**: Invert the problem. Store options by *canonical namespace* (anthropic, openai, bedrock) in message struct, not by providerID. Eliminates round-trip lookup.

**File**: provider/transform.ts::sdkKey(), providerOptions()

---

### 2. Cache Marker Placement Algorithm (transform.ts lines 260-329)
**Mechanism**: Place 4 cache breakpoints at stable prefix boundaries: (1) last system message, (2) conversation midpoint, (3) message before last user, ensuring cache hits across reruns. Respects per-provider support (Anthropic only, or SDK-level heuristic).

**ELAI Steal**: Embed cache-marker placement in the prompt compiler. For a Rust orchestrator, emit cache markers into the Effect graph rather than the message list. Pre-compute stable boundaries at session start.

**File**: provider/transform.ts::applyCaching()

---

### 3. Capability-Safe Schema Flattening (transform.ts lines 1120-1213, 1215-1322)
**Mechanism**: Discriminated unions (z.discriminatedUnion in Zod) emit anyOf/oneOf at root. Many providers (OpenAI, Bedrock) reject this. Flatten: merge all variant properties into one object, place discriminator as enum, annotate variant-specific fields with "(only when action=X)" in description.

**ELAI Steal**: Bake this flattening into the wiring-contract validator. Before emitting a tool/action schema, check if root has anyOf/oneOf; if so, flatten and annotate. Prevents tool-definition BLOCK at runtime.

**File**: provider/transform.ts::flattenDiscriminatedUnion(), schema()

---

### 4. Model Cost Tiers with Fallback (provider/provider.ts lines 966-986, 1207-1234)
**Mechanism**: Cost has shape { input, output, cache: {read, write}, experimentalOver200K: {...} }. Missing cost fields default to 0 (with once-per-key warning). Over-200K tier used only if context > 200K; fallback to base tier otherwise.

**ELAI Steal**: Generalize to a tier ladder: standard (all context), under_128k, over_200k, over_1m. Each tier overrides input/output independently. At inference time, pick tier based on actual message length.

**File**: provider/provider.ts::cost()

---

### 5. Per-Provider Fetch Wrapper with Signal Merging (provider/provider.ts lines 1491-1528)
**Mechanism**: Merge three abort signals (user-supplied, chunk timeout, request timeout) into one via AbortSignal.any(). Chunk timeout wraps SSE reader with setTimeout per-chunk to catch stalls. Custom fetch hook runs before provider SDK calls.

**ELAI Steal**: Lift this to runtime_client. When invoking a provider (Sonnet, Qwen sidecar, etc.), compose abort signals: (user cancel) + (config default timeout) + (chunk stall timeout, sidecar only). Standardize across backends.

**File**: provider/provider.ts::resolveSDK()

---

### 6. Env Var + Config + Auth Layering (provider/provider.ts lines 1251-1327)
**Mechanism**: Load in order: env vars, config file (mimocode.json), auth store (auth.json from `mimocode auth` CLI), plugin auth loaders. Each layer merges, later overrides earlier. Disabled/enabled provider lists filter at each step.

**ELAI Steal**: Formalize a "config precedence" enum: env > config > auth > default. Codify as a decision point in the boot sequence. Apply to orchestrator settings (ELAI_*_ENABLE flags vs. config file, allow env to veto config).

**File**: provider/provider.ts::InstanceState.make() (lines 1086-1424)

---

### 7. Model Variant Switching via Options (provider/transform.ts lines 504-892)
**Mechanism**: Reasoning effort (low/medium/high) is not a separate model; it's an option on the existing model. Expose variants as a map per model (claude-sonnet → { low: {...}, high: {...} }), each variant is a dict of overrides (thinking.budgetTokens, reasoningEffort, etc.). Query time: resolve variant string → override dict → merge with base options.

**ELAI Steal**: Store variants in the ModelInfo SSB row (bit-packed or JSON). At request time, if variant != null, look up ModelInfo.variants[variant] and apply. Avoids N*V model rows.

**File**: provider/transform.ts::variants()

---

### 8. Provider Discovery via Dynamic Import (provider/provider.ts lines 1530-1566)
**Mechanism**: 25+ bundled SDKs (ai-sdk/*) are lazy-imported via dynamic import(pathToFileURL(path)). Fallback to npm package install if not bundled. SDK cached by hash(providerID, npm, options) to avoid re-importing.

**ELAI Steal**: For Rust, macro-generated static dispatch table at compile-time. If using external providers (Sonnet via Anthropic SDK, Qwen via custom runtime), pre-link at build, no dynamic loading. Trade compile size for zero-import latency.

**File**: provider/provider.ts::resolveSDK()

---

### 9. ACP as Stdin/Stdout RPC Wrapper (acp/README.md lines 39-150)
**Mechanism**: Clients (Zed, Claude Code) speak JSON-RPC over stdio to the agent. Agent routes initialize/session/* to Effect handlers. Session state maps 1:1 to opencode sessions. No custom wire protocol; uses JSON-RPC library (official @agentclientprotocol/sdk).

**ELAI Steal**: Expose the orchestrator as an ACP agent. No new protocol. Clients (CI, third-party IDEs) invoke ELAI via `opencode acp`-like stdio JSON-RPC. Sessions map to SSB row + RRA bundle.

**File**: acp/agent.ts, acp/server.ts

---

### 10. Auth Persistence with Metadata (auth/index.ts lines 13-36)
**Mechanism**: Auth Info is a union { type: "api", key, metadata } | { type: "oauth", access, refresh, expires, accountId, enterpriseUrl }. Metadata dict allows providers to store extras (GCP project ID, Bedrock region, Azure resource name) without schema changes.

**ELAI Steal**: Use the same pattern for ELAI provider secrets. Store as { providerID, type, key, metadata: { ...provider-specific } }, allow runtime to read metadata to configure SDK options.

**File**: auth/index.ts

---

## Unknowns / Unverified

1. **Cache marker correctness**: Does placement at last-system + midpoint + before-last-user actually maximize cache hits? No benchmark cited. Assume empirical tuning from MiMo team.

2. **Model.variants generation logic** (transform.ts::variants): Complex switch/case per SDK npm package. If a new SDK added (e.g., vLLM provider), variants() must be updated. No extensibility mechanism. Maintenance risk.

3. **Session persistence in ACP** (acp/README.md line 117): session/load documented as "not yet implemented" — unclear if it's a design gap or planned.

4. **Share sync determinism**: Debounced flush() may drop changes if sync fails and retry is not exponential. No retry policy documented.

---

## Summary

MiMoCode Part 4 is a **vendor-agnostic multi-backend abstraction** layered with **per-provider tweaks** (message transforms, cache markers, schema flattening, option remapping). Auth and sharing are orthogonal services. The ACP protocol adapter opens the system to third-party clients. Portable steal: cache-marker placement, schema flattening, fetch wrapper signals, config layering, variant switching, and ACP-as-stdlib.
