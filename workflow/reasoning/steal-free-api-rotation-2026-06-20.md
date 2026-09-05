steal analysis — free-api rotation (Liberty) + token-efficiency/agentic-context win-both
date: 2026-06-20
source (local): F:\.ELAI_workflow\.liberty (rotation subsystem) + interface map workflow/papers/liberty-rotation-interface.md
design source (this session): the spec-eval "win on both" reasoning + oracle run oracle-agentic-token-efficiency-codeact-toolsearch-2026
elai context: docs/thefinalglance/ELAI-FEATURE-MAP.md (loaded this session)
target family: J-plan (highest detail file J-53; new = J-54, J-55). Thematic home: J = local/free-model serving + efficiency (J-30 BYOK, J-48/J-49 serving, J-51 adherence, J-34 calibration). K-plan rejected as home (K charter = "adds almost NO new capability"); these are new capability.

user decisions (binding, this session — do not re-litigate):
1. free-API rotation is USER-SELECTABLE PER ROLE via config, exactly like the existing local/cloud/hybrid PerRoleRuntime assignment (B-37). add a "free-pool" as a third assignable backend kind. NOT a hardcoded low-trust-only fence. guardrail kept regardless: free-pool resolved for executor/auditor stamps the run non-RRA-reproducible + suppresses-with-notice the G-10 drift canary (surface, never silent).
2. rotation engine = HAND-ROLLED in Rust, no litellm dependency, no external proxy. faithful port of catalog/router/quota/self_heal/providers_extra onto ELAI substrate; reuse reqwest/blake3(see CB-1)/serde_json/cloud-client; flag any new crate for user confirmation.

================================================================================
CANDIDATE STEALS
================================================================================

S1. FREE-MODEL CATALOG (static data) — catalog.py FreeModel + FREE_MODELS + DENYLISTED_MODEL_IDS + groups
  what: the typed free-model table (id/provider/group/context_window/max_output/rpm/tpm/rpd/tpd) across groq/cerebras/gemini/mistral/openrouter free tiers, grouped (big/fast-small/gemini-flash/mistral/openrouter-free/uncensored), with a denylist of dead IDs. interface map §1.
  elai subsystem: I-51 MODEL CAPABILITY CATALOG (models.toml over pricing/vram scatter) [PLANNED] + B-10 ModelCapabilityProfile [WIRED].
  verdict: NEW(J-54) — ships the free catalog as STATIC ROWS in models.toml (the I-51 canonical store), not a separate hardcoded list. compose I-51; do not author a parallel registry.

S2. ROTATION DISPATCHER — router.py build_router + _FALLBACK_CHAIN + multi-key collection + simple-shuffle
  what: one deployment per (model,key) pair; multi-key env convention (GROQ_API_KEY/_1/_2..); group routing; cross-group fallback chain big>fast-small>gemini-flash>mistral>openrouter-free>overflow-pollinations; uncensored has NO cross-group fallback; absent provider keys skipped. interface map §2.
  elai subsystem: B-37 MULTI-RUNTIME BACKEND (RuntimeAdapter trait, PerRoleRuntime, resolve_backend) [WIRED].
  verdict: NEW(J-54) — a FreePool RuntimeBackend kind whose adapter holds the deployment list + simple-shuffle + fallback chain. compose B-37 (the trait + per-role resolve); do not re-implement RuntimeOrchestrator.

S3. QUOTA LEDGER — quota.py KeyId + per-(key,model) UTC-day counters + 429 header-priority cooldown + has_headroom + soonest_reset
  what: KeyId = provider + sha256(raw_key)[:8] (NEVER stores plaintext); per-(key,model) day req/token counters persisted UTC-day-keyed to quota_state.json (survives restart, discarded on new UTC day); 429 cooldown reset priority x-ratelimit-reset-requests > Retry-After > x-ratelimit-reset-tokens > 60s default, MINIMUM-wins across parseable candidates; has_headroom enforces rpd/tpd + cooldown; soonest_reset. interface map §3, invariants I-2/I-3/I-9.
  elai subsystem: none (open whitespace) + C-5 cost caps (CloudModeRouter hard ceiling) for the spend-ceiling compose.
  verdict: NEW(J-54) — the rotation ECONOMICS layer ELAI genuinely lacks. CB-1: KeyId hash uses blake3 per CLAUDE.md anti-stale (NOT sha256/sha2) — fingerprint-only, non-RFC use, so blake3 is mandated; the Liberty sha256 choice is NOT carried.

S4. SELF-HEAL — self_heal.py _HealingProxy + self_healing_acompletion + soonest-reset sleep
  what: wrap the dispatcher; on whole-pool exhaustion (RouterRateLimitError equivalent) queue+sleep-until-soonest_reset+retry (bounded by max_total_wait/max_retries); ALL other exceptions propagate; transparent delegation for everything except the completion call. interface map §4, invariants I-9/I-10.
  elai subsystem: I-7 CASCADING ESCALATION (local>cloud waterfall) [WIRED] — the spill target when free-pool is exhausted AND a cloud lane is configured.
  verdict: NEW(J-54) — the heal/backoff wrapper. compose I-7 (pool-exhaustion can spill to the configured cloud lane instead of sleeping, when the role allows it).

S5. KEYLESS OVERFLOW — providers_extra.py Pollinations always-present last-resort
  what: keyless Pollinations overflow group (openai-compat, api_key="none"), ALWAYS reachable so the pool is never fully empty; g4f mirror tier (Python-only). interface map §5, invariant I-7.
  elai subsystem: none.
  verdict: NEW(J-54) — port Pollinations keyless overflow (always-present last deployment). REJECT the g4f tier (Python-only unofficial mirrors, no clean Rust path, ToS-gray — interface map §5 + LIBERTY doc §4 caveat).

S6. PER-ROLE FREE-POOL ASSIGNMENT + TRUST-MARKER (the user-decision-1 surface)
  what: let the user assign backend kind = FreePool to any role (draft/research/executor/..) the same way local/cloud/hybrid is assigned today; when resolved for executor/auditor, stamp the run non-RRA-reproducible + suppress-with-notice the G-10 drift canary.
  elai subsystem: B-37 PerRoleRuntime + G-2 RRA + G-10 MODEL-DRIFT CANARY [WIRED].
  verdict: NEW(J-54) — the config + trust integration. composes G-2 (the seal records "free-pool, non-reproducible"), G-10 (suppress-with-notice, surface never silent). this is the load-bearing guardrail, kept regardless of how freely the user assigns the pool.

S7. CATALOG SYNC (dynamic fetch) — catalog_sync.py cheahjs/OpenRouter live fetch + merge
  what: extend the catalog at runtime from the cheahjs free-models list + OpenRouter live /models API. interface map §1 (catalog_sync).
  verdict: ALREADY-REJECTED (I-51 static models.toml canonical; OpenRouter DYNAMIC model registry rejected three times — goose verdict + local-serving verdict + I-51, feature-map ledger). The free catalog ships as DATA in models.toml, refreshed by maintainer, never auto-fetched. NOT ported.

S8. MODALITY ROUTER (win-both, this session's design) — CodeAct/constrained-call routed by model strength
  what: resolve the ACTION MODALITY per (model,role): CodeAct (E-35) for capable lanes; constrained single-tool-call + prompt-compiled minimal schemas (I-36/C-25) for the 7B local lane; universal SEARCH/REPLACE-diff edits (E-35/E-18/H-3); progressive tool disclosure (H-4/H-34/J-24). the three independent token sinks (tool-schema/action-envelope/edit-output) attacked separately; KEYSTONE = the router, because naive CodeAct-everywhere loses on ELAI's own Qwen-7B local lane.
  grounding (oracle, arXiv IDs WebSearch-verified 2026-06-20): CodeAct up to 20% higher success but open-source collapses to ~12% on multi-tool code-mode (2402.01030, ICML 2024, VERIFIED); ~14B viability threshold for small-model multi-agent code (2509.04508 ProST, VERIFIED); grammar-constrained decoding 3.6-8.2x latency on 7-9B and prompt-optimization beats it on the combined metric (2605.02363 "When Correct Isn't Usable", VERIFIED). all three independently verified this session; oracle's other cites (2604.27296 / 2604.03515 / 2603.22862) NOT load-bearing, not carried into the plan.
  elai subsystem: J-51 TOOL-CALL ADHERENCE FLOOR [PLANNED], E-31 tier matrix [PARTIAL], E-35 CodeAct+SEARCH/REPLACE [PARTIAL], I-36 prompt compilation [DARK], H-4/H-34/J-24 progressive disclosure [PARTIAL], J-34 calibration [PLANNED].
  verdict: NEW(J-55) — the ModalityRouter that composes all of the above; done-gate = J-34-style measured per-(model,lane) crossover (deterministic, never folklore). EXTENSION-heavy: composes J-51/E-31/E-35/I-36/H-4/J-34 — re-implements none.

================================================================================
SUMMARY: 6 NEW (S1-S6 grouped into J-54; S8 = J-55), 1 REJECT-not-ported (S5 g4f), 1 ALREADY-REJECTED (S7 catalog_sync dynamic fetch).
NEW SSB (Pattern-2, baseline+ from LIVE 268/577 as of 2026-06-15, re-read at edit): J-54 +FreePoolEvent (3-arm: ModelRotated / PoolExhaustedBackoff / FreePoolTrustDowngrade); J-55 +ModalityRouted (3-arm: CodeActChosen / ConstrainedCallChosen / EditFormatChosen). net +2.
CB-1: blake3 not sha256 for KeyId fingerprint (CLAUDE.md anti-stale). CB-2: g4f tier dropped. CB-3: catalog ships in models.toml (I-51), no dynamic fetch.
OUTCOME: plans created — J-54_free_api_rotation_pool.md, J-55_model_strength_modality_router.md (2026-06-20).
