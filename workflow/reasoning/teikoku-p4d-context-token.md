TEIKOKU P4-D context-compiler report (READ-ONLY)
task_id: P4-D-CC1
date: 2026-06-20
scope: the per-model TOKEN COUNTER + the auto-migration THRESHOLD TRIGGER half of P4-D / 4.3 / D-09.
peer CC-2 owns: snapshot / RAG fallback / carry-forward / migrating indicator (NOT covered here).
distro: teikoku (WSL2). conductor = ~/teikoku/conductor. base-cc = ~/teikoku/base-cc.

=====================================================================================
TL;DR (the load-bearing answers)
=====================================================================================
- base-cc ALREADY has a full, accurate per-turn token meter (analyzeContext.ts +
  tokens.getCurrentUsage), and the conductor ALREADY reads the result block per turn at
  slice_driver.py:111-117 (ctx_used + ctx_cap from the `--output-format json` result's `usage`
  + `modelUsage[model].contextWindow`). That seam is PROVEN live (P0.5 slice).
- THE ONE REAL D-09 GAP for the counter: base-cc's per-model WINDOW source
  (utils/context.ts::getContextWindowForModel) defaults EVERY non-claude model to
  MODEL_CONTEXT_WINDOW_DEFAULT = 200_000 (context.ts:9). So a devin kimi-k2-7 turn (true 260k)
  or a free cerebras turn (true 131k) or a mistral turn (true 32k) all report ctx_cap=200000 ->
  the migrate-at math is WRONG for every backend except claude. The fix is NOT a new tokenizer;
  it is sourcing the per-model window from model_pool.PoolModel.context_window (already authored,
  P4-C) instead of trusting base-cc's flat 200k default for non-claude backends.
- THE TRIGGER does not exist yet. There is NO chat-context-vs-migrate_at comparison anywhere in
  the conductor. _compact_epoch (conductor_loop.py:795) is WAVE-artifact archiving, unrelated.
  The executor ADDS a trigger that reads response.model + ctx_used off the chat turn and raises
  MIGRATION_REQUIRED when ctx_used >= migrate_at(model). The natural hook is the /chat/turn route.

=====================================================================================
1. CURRENT TOKEN ACCOUNTING IN BASE-CC (and what D-09 must fix per backend)
=====================================================================================
HOW BASE-CC TRACKS USAGE PER TURN (the real meter):
- Source of truth per turn = the API response `usage` block, extracted by
  tokens.getCurrentUsage(messages) (base-cc/src/utils/tokens.ts:138-157): walks messages
  backward to the last usage-bearing message and returns {input_tokens, output_tokens,
  cache_creation_input_tokens, cache_read_input_tokens}.
- The /context display (the "TokenMeter") = utils/analyzeContext.ts. It computes the displayed
  total at analyzeContext.ts:1163-1173:
    apiUsage = getCurrentUsage(originalMessages)
    totalFromAPI = input_tokens + cache_creation_input_tokens + cache_read_input_tokens   (:1166-1170)
    finalTotalTokens = totalFromAPI ?? estimatedTotal   (:1173, falls back to a local count)
  and returns {totalTokens, maxTokens, rawMaxTokens, percentage} (:1345-1348) where
  maxTokens == rawMaxTokens == contextWindow.
- The /context slash command and the SDK get_context_usage control request both flow through
  collectContextData (commands/context/context-noninteractive.ts:33-77 -> call() :79-87), which
  runs the SAME pre-API transforms (compact boundary + microcompact + projectView) so the count
  reflects what the model actually sees, then calls analyzeContextUsage.
- PER-TURN ACCUMULATION (the streaming path) = QueryEngine.ts:786-816: on message_start it resets
  currentMessageUsage and seeds from message.usage; message_delta accumulates via updateUsage;
  message_stop folds into this.totalUsage via accumulateUsage. totalUsage is surfaced on the
  result (QueryEngine.ts:629/861/991/1146). So the cumulative per-session figure exists too.
- Auxiliary estimators (NOT the meter, used for spinners / pre-API checks): char/4 in
  getAssistantMessageContentLength (tokens.ts:~183-205), tokenCountWithEstimation (tokens.ts:~210+)
  which estimates messages added since the last API usage, and the dedicated
  countTokensWithFallback (analyzeContext.ts:77-110) which calls the API count endpoint and on
  null falls back to a haiku count.

WHERE THE CONDUCTOR ALREADY READS USAGE (the live seam, PROVEN):
- run_cc_turn (slice_driver.py:45-58) shells `bun dist/cli.js --print --output-format json ...`
  and parses the trailing JSON line. The result schema (base-cc coreSchemas.ts:1417-1444) carries
  total_cost_usd, usage, and modelUsage = record<model, ModelUsageSchema>; ModelUsageSchema
  (coreSchemas.ts:17-25) has {inputTokens, outputTokens, ..., costUSD, contextWindow}.
- slice_driver.py:111-117 consumes it EXACTLY as the revision doc :122-128 describes:
    usage = cc.get("usage", {})
    model_usage = cc.get("modelUsage", {})
    model_key = next(iter(model_usage), "claude-opus-4-6")
    ctx_used = usage.input_tokens + usage.output_tokens
             + usage.cache_read_input_tokens + usage.cache_creation_input_tokens   (:115-116)
    ctx_cap  = int(mu.get("contextWindow", 200000))   (:117)
  -> emitted as SLICE_TURN_DONE / SLICE_COMPLETE events (ctx_used, ctx_cap). This is the
  production accounting path the trigger composes onto; it is NOT chat-loop-resident yet (it
  lives in the slice driver), so the trigger work generalizes this read into the /chat/turn path.

PER-BACKEND ACCURACY (what "more-accurate per-model CC token counter", D-09, must fix):
- claude (OAuth, default base URL): REAL usage. The Anthropic response carries true
  input/output/cache token counts; modelUsage[claude].contextWindow is real. ACCURATE today.
- devin-free (the BRIDGE :8810, devin_proxy.py): the /v1/messages response's `usage` comes from
  the Devin proto meta.input/output_tokens decode (devin-bridge explore doc :95) = REAL per-turn
  usage. BUT the bridge's /v1/messages/count_tokens endpoint is a crude char/4 ESTIMATE
  (devin_proxy.py:293-297, explore doc :251-252) -- so any base-cc path that PRE-counts via the
  count endpoint (countTokensWithFallback, the pre-API context check) is ESTIMATED, not exact, on
  the devin lane. The post-turn `usage` block is real; the pre-turn count endpoint is the weak leg.
- free-api (litellm proxy :4000): litellm's Anthropic-passthrough returns a real `usage` block
  from the upstream free provider. REAL post-turn usage (modulo provider quirks); the per-model
  contextWindow is the litellm/provider value, NOT base-cc's flat 200k.
- THE DOMINANT BUG (all non-claude lanes): base-cc's getContextWindowForModel (context.ts:51-100)
  returns MODEL_CONTEXT_WINDOW_DEFAULT=200_000 (context.ts:9, :100) for any model whose
  modelCapability max_input_tokens is unknown -- which is EVERY devin / free id (they are not in
  base-cc's antModels capability table). So contextWindow (and thus ctx_cap / migrate-at math) is
  wrong for kimi (260k), glm/swe (150k), cerebras/groq (131k), mistral/pollinations (32k), gemini
  (1M). D-09 must NOT trust base-cc's window for non-claude; it must read the window from
  model_pool (deliverable 2).
- FIX SHAPE per backend: (a) for the WINDOW, override base-cc's default with model_pool's
  per-model context_window (env CLAUDE_CODE_MAX_CONTEXT_TOKENS already exists as an override hook,
  context.ts:58-67, but is USER_TYPE==ant gated; the clean path is the conductor stamping the
  right cap from model_pool rather than reading mu.contextWindow blindly); (b) for the COUNT,
  trust the post-turn `usage` block (real on all three) and treat the devin count_tokens
  pre-estimate as approximate; (c) generalize slice_driver's ctx_used read into the chat path.

=====================================================================================
2. PER-MODEL WINDOW + MIGRATE-AT DATA (model_pool.py, already authored P4-C)
=====================================================================================
model_pool.py (P4-C) ALREADY carries the full table the trigger needs. PoolModel
(model_pool.py:71-101) fields: pool, model, harness, context_window, migrate_at, label, note.
The roster (model_pool.py:114-145):
  CONTROLLER (_CONTROLLER_MODELS :114-123):
    claude        window 200_000  migrate_at 200_000  <- OWNER quality-handoff override (NOT the
                                                          real window; claude's real window is
                                                          larger, 200k/1M[1m]); note "quality
                                                          handoff at 200k" (model_pool.py:115-116).
    kimi-k2-7     window 260_000  migrate_at 200_000   (devin-free, bridge :8810, default)
    glm-5-2       window 150_000  migrate_at 120_000
    swe-1-6       window 150_000  migrate_at 120_000
  FREE-API (_FREE_API_MODELS :127-136):
    free-fast (cerebras gpt-oss-120b)  window 131_072  migrate_at 104_000
    free-groq (groq 70B/120B)          window 131_072  migrate_at 104_000
    free-mistral                       window  32_768  migrate_at  26_000
    free-pollinations                  window  32_768  migrate_at  26_000
- migrate_at DERIVATION: currently HAND-AUTHORED per row as literals (NOT computed). They match
  the revision-doc table (:131-141): ~80% of window for the free/devin lanes (200k of 260k, 120k
  of 150k, 104k of 131k, 26k of 32k), claude pinned to the owner's 200k. There is no
  derive-from-window helper yet; P4-D may keep the literals or add a derive (window*0.8) with the
  claude row as an explicit override.
- MIN-WINDOW-ACROSS-FALLBACK rule (revision :128-130): "budget the NEXT turn against
  response.model's window; when a pool has fallbacks use the MINIMUM reachable window." This is
  the liberty catalog.min_context_window_for_group precedent (referenced in model_pool.py
  docstring :15-20, :30-36) but is NOT YET IMPLEMENTED in model_pool. Today model_pool resolves a
  SINGLE (pool, model) -> one PoolModel; there is no group-minimum reducer. P4-D ADDS a
  min-window-across-fallback function: given the pool's reachable model set (e.g. free-api can
  fall back across cerebras/groq/mistral/pollinations), take min(context_window) and
  min(migrate_at) so a 131k prompt does not overrun when the router degrades to a 32k mistral.
  The CONTROLLER claude pool has no fallback (single model); devin-free fallback set =
  {kimi 260k, glm 150k, swe 150k} -> min 150k; free-api fallback set min = 32k.
- MISSING ROW FLAGGED: gpt-5.5 (the codex paid lane, D3). It is NOT in model_pool.py (the picker
  only offers claude + devin-free + free-api; codex is a SEPARATE harness per
  model_pool.resolve_slot's account_id=None note :167-170). The codex provider
  (conductor/codex_provider.py) has no context_window for gpt-5.5, and the revision table
  (:140 row) lists it as TBD. P4-D must EITHER add a gpt-5.5 PoolModel row sourced from the bmapi
  proxy /v1/models, OR carry an owner override (like claude=200k). FLAG: this is a material data
  gap, not a code gap -- needs an owner number or a /v1/models read before the codex lane can
  auto-migrate correctly.
- gemini-2.5-flash (revision :138, 1M window, RPD 20): listed "single-shot only" in the revision
  table and is NOT in model_pool's free-api roster (only cerebras/groq/mistral/pollinations are).
  If gemini is selectable it needs a row; today it is out-of-roster, so out-of-scope for the
  trigger unless added.

=====================================================================================
3. THE THRESHOLD-TRIGGER SEAM (where ctx >= migrate_at fires MIGRATION_REQUIRED)
=====================================================================================
WHAT EXISTS:
- The chat loop entrypoint = slice_api.do_POST route "/chat/turn" (slice_api.py:411-413) ->
  chat_turn(payload) (slice_api.py:242-269). chat_turn loads toggles, calls
  chat_orchestrator.decide() then apply(), returns {ok, decision, result}. This is the REAL
  cross-process entrypoint a chat loop calls each turn (the PRODUCTION_HOOK, deliverable 5).
- The per-turn usage read (the thing to compare against migrate_at) EXISTS but only in the slice
  driver: slice_driver.py:111-117 (ctx_used + ctx_cap from the result `usage`/`modelUsage`).
  chat_turn does NOT currently read response.model or ctx_used -- apply() dispatches spawn/mailbox
  and never inspects token usage. So the trigger read must be added to the chat path.
- response.model is available: the base-cc `--output-format json` result's modelUsage keys ARE
  the model ids (slice_driver.py:113 `model_key = next(iter(model_usage), ...)`). So
  "read response.model after each turn" = read the modelUsage key (or the result.model field) off
  the chat turn's CC result.
- base-cc's OWN context lifecycle for reference (NOT the migration trigger, but the analog):
  autoCompact.ts computes getEffectiveContextWindowSize (autoCompact.ts:32-49) =
  getContextWindowForModel - reservedSummary, getAutoCompactThreshold (:72-94) = effectiveWindow -
  AUTOCOMPACT_BUFFER_TOKENS(13_000), and calculateTokenWarningState (:97-141) returns
  isAboveAutoCompactThreshold / isAtBlockingLimit. THIS is base-cc's in-session compaction gate --
  it fires on the SAME flat-200k window bug for non-claude (it calls getContextWindowForModel). The
  migration trigger is a SIBLING gate at a HIGHER (cross-session) level: instead of compacting
  in-place, it raises MIGRATION_REQUIRED so CC-2's snapshot+new-session carry-forward runs. The two
  must be reconciled (migrate_at < autoCompact blocking limit so migration fires BEFORE base-cc's
  own auto-compact would mangle the local lane -- see the H-plan Track 9 finding referenced in the
  handoff: local lanes are unprotected because base-cc runs a cloud-shaped 200k policy).
  context-window-blocked / isAtBlockingLimit is base-cc's hard wall (autoCompact.ts:121-141); the
  trigger must fire well before it.

WHAT THE EXECUTOR ADDS:
- A trigger function (new conductor module, e.g. migration_trigger.py) with:
    resolve_window(model) -> PoolModel via model_pool (NOT base-cc's flat 200k) + the
      min-window-across-fallback reducer (deliverable 2).
    should_migrate(ctx_used, model, override) -> bool : ctx_used >= migrate_at(model)
      (honoring a per-session configurable override, deliverable 4; claude=200k owner override).
  PRODUCER. Consumer wired same-changeset = chat_turn (slice_api.py:242-269): after the CC turn
  result is read, compute ctx_used (generalize slice_driver:115-116), look up the turn's model,
  compare to migrate_at, and on breach return a MIGRATION_REQUIRED signal in the chat_turn
  response (+ append a MIGRATION_REQUIRED event for the dashboard / CC-2 snapshot to consume).
- The hook point detail: chat_turn currently routes BEFORE any model turn runs (decide/apply is
  pre-dispatch). The token read is POST-turn. So the trigger lands EITHER (a) at the start of the
  NEXT /chat/turn (read the prior turn's accumulated ctx, decide migrate-or-proceed -- "budget the
  next turn", revision :128), OR (b) right after the CC result is parsed if chat_turn is extended
  to run the model turn inline. Option (a) matches the revision-doc "budget the NEXT turn against
  response.model's window" wording and is the lower-risk seam (a read of the session's last
  recorded ctx event before dispatching the new turn).

=====================================================================================
4. CONFIGURABLE THRESHOLD (per-session / per-model migrate-at override)
=====================================================================================
- The persistence substrate EXISTS: control_state KV table (migrations/003_control_state.sql:
  key TEXT PK + value_json + updated_at, no CHECK enum) accessed session-scoped by
  session_toggles.py via keys "session:<sid>:<name>" (session_toggles.py:_KEY_PREFIX :73,
  _session_key :114). set_toggle / get_toggles already round-trip JSON values.
- A migrate-at override is the natural NEW toggle: add e.g. TOGGLE_MIGRATE_AT (an int, or a
  per-model dict) to TOGGLE_NAMES (session_toggles.py:51-65) + the SessionToggles dataclass
  (:77-96) + _coerce (:106-114, add an int branch) + get_toggles read-back (:160-180). Zero schema
  migration (the table already holds arbitrary keys). The override is then read by the trigger's
  resolve_window/should_migrate (deliverable 3) alongside the model_pool default.
- The OWNER override (claude=200k) is ALREADY representable two ways: (1) it is baked into the
  claude PoolModel.migrate_at=200_000 literal (model_pool.py:115-116), and (2) a per-session
  toggle can raise/lower it. The owner-quality-handoff semantics ("applies to interactive chat AND
  the orchestrator/controller session", revision :140-141) means the claude=200k default should be
  the global floor and the per-session toggle an optional per-session override on top.
- WHERE IT PERSISTS: the same control_state KV via session_toggles (session-scoped). A per-MODEL
  (not per-session) override would instead be a small config constant or a new model_pool override
  map; the revision doc's "configurable threshold" is per-session, so the session_toggles route is
  canonical. The chat_toggles_set route (slice_api.py:271-285, POST /chat/toggles) is the existing
  write path -- the new toggle rides it for free.

=====================================================================================
5. WIRING + TESTS (PRODUCTION_HOOK + the new tests)
=====================================================================================
PRODUCTION_HOOK (D-030, the seam the trigger fires from, must be wired same-changeset):
  PRIMARY: slice_api.py::chat_turn (slice_api.py:242-269), reachable via the real
  cross-process route POST /chat/turn (slice_api.py:411-413). The new should_migrate(...) producer
  is CONSUMED here: chat_turn reads the session's last-turn ctx + model, calls should_migrate, and
  on breach returns {migration_required: true, model, ctx_used, migrate_at} in the response AND
  appends a MIGRATION_REQUIRED event (append_event, the same off-agent-artifact pattern
  slice_driver uses at :108/:122/:158). That event is the un-gameable wiring evidence (wiring R4)
  and CC-2's snapshot consumer reads it.
  SECONDARY producers and their consumers (same changeset):
    model_pool.resolve_window / min_window_across_fallback (new) -> consumed by should_migrate.
    session_toggles TOGGLE_MIGRATE_AT (new) -> consumed by should_migrate (override) + written by
      POST /chat/toggles (slice_api.py:271-285).
  Generalizing slice_driver.py:111-117 ctx_used into a shared helper (e.g. usage_utils.ctx_used)
  consumed by BOTH slice_driver and chat_turn avoids duplicate accounting logic (maintainability
  MR: do not copy-paste the 2-line sum).

NEW TESTS (the executor authors; the auditor re-runs):
  T1 per-backend token-count accuracy:
     - claude lane: a result with a real `usage` block -> ctx_used == sum of the four fields
       (exact); modelUsage[claude].contextWindow honored.
     - devin lane: assert ctx_used comes from the POST-turn `usage` (real), and that a path
       relying on the bridge count_tokens is treated as ESTIMATE (the char/4 leg, devin_proxy.py
       :293-297) -- i.e. the trigger must NOT use the pre-count endpoint for the migrate decision.
     - litellm free-api lane: a real `usage` block from the proxy -> exact ctx_used.
  T2 migrate_at fires at the right ctx PER MODEL: parametrize over the model_pool roster; assert
     should_migrate is False just under migrate_at and True at/over it, for kimi(200k of 260k),
     glm/swe(120k of 150k), cerebras/groq(104k of 131k), mistral/pollinations(26k of 32k).
  T3 MIN-window-across-fallback: a pool with a fallback set returns the MINIMUM migrate_at/window;
     a 131k-ctx turn on a pool that can degrade to a 32k mistral fires migration (the 32k floor),
     NOT the 131k cerebras window.
  T4 claude=200k owner override: claude migrate_at == 200_000 regardless of its real larger window;
     a per-session TOGGLE_MIGRATE_AT override changes the fire point and round-trips through
     control_state.
  T5 GENERALITY (core req): a NON-ELAI chat session migrates the SAME way -- pure (model, ctx_used)
     math, no ELAI literal reachable (DEFAULT_PROJECT_ID "conductor", never "elai"; mirrors the
     existing generality gate). Drive POST /chat/turn for a demo-pack session and assert the
     MIGRATION_REQUIRED event fires with zero ELAI path/template reachable.
  T6 (integration / R10) drive the REAL /chat/turn entrypoint across enough turns to cross
     migrate_at, assert the MIGRATION_REQUIRED event landed in the event log (off-agent artifact),
     reconcile that migrate_at fires BEFORE base-cc's own autoCompact blocking limit for the lane.

=====================================================================================
KEY FILE:LINE INDEX
=====================================================================================
base-cc:
  utils/context.ts:9         MODEL_CONTEXT_WINDOW_DEFAULT = 200_000 (the flat default = the bug)
  utils/context.ts:51-100    getContextWindowForModel (returns 200k for all non-claude)
  utils/context.ts:58-67     CLAUDE_CODE_MAX_CONTEXT_TOKENS override (USER_TYPE==ant gated)
  utils/tokens.ts:138-157    getCurrentUsage (the per-turn usage extractor)
  utils/tokens.ts:159-168    doesMostRecentAssistantMessageExceed200k (hardcoded 200k sibling)
  utils/analyzeContext.ts:935            contextWindow = getContextWindowForModel(...)
  utils/analyzeContext.ts:1163-1173      apiUsage / totalFromAPI / finalTotalTokens
  utils/analyzeContext.ts:1345-1348      return {totalTokens, maxTokens, rawMaxTokens, percentage}
  QueryEngine.ts:786-816                 message_start/delta/stop usage accumulation -> totalUsage
  services/compact/autoCompact.ts:32-141 effectiveWindow / autoCompactThreshold / warningState
  commands/context/context-noninteractive.ts:33-87  /context + get_context_usage collector
  entrypoints/sdk/coreSchemas.ts:17-25   ModelUsageSchema {inputTokens,...,contextWindow}
  entrypoints/sdk/coreSchemas.ts:1417-1444  result schema {usage, modelUsage, total_cost_usd}
conductor:
  slice_driver.py:45-58      run_cc_turn (bun dist/cli.js --print --output-format json, json parse)
  slice_driver.py:111-117    ctx_used + ctx_cap read from result usage/modelUsage  <- THE seam
  slice_api.py:242-269       chat_turn (the /chat/turn handler)  <- PRODUCTION_HOOK
  slice_api.py:411-413       POST /chat/turn route
  slice_api.py:271-285       chat_toggles_set (POST /chat/toggles) <- override write path
  model_pool.py:71-101       PoolModel {context_window, migrate_at}
  model_pool.py:114-145      the roster (claude/kimi/glm/swe/free-*)
  session_toggles.py:51-114  TOGGLE_NAMES / control_state KV / session-scoped keys
  chat_orchestrator.py:198-279  decide/apply (current chat routing; no token read today)
  conductor_loop.py:795-814  _compact_epoch (WAVE archiving, NOT chat migration)
devin-bridge (host, read-only, per explore doc):
  devin_proxy.py:293-297     count_tokens char/4 ESTIMATE (the weak pre-count leg)
  devin_proxy.py:~95 (meta.input/output_tokens -> usage)  real per-turn usage in the response

=====================================================================================
VALIDATION WARNINGS
=====================================================================================
- gpt-5.5 (codex paid lane) has NO context_window anywhere (not in model_pool, not in
  codex_provider, TBD in the revision table :140). MATERIAL DATA GAP -> owner override or a bmapi
  /v1/models read needed before the codex lane auto-migrates. Flag to controller.
- model_pool migrate_at values are hand-authored literals, not derived; min-window-across-fallback
  is documented (liberty precedent) but NOT yet implemented in model_pool.
- base-cc's getContextWindowForModel default-200k applies to BOTH the migration trigger AND
  base-cc's own autoCompact -> the two gates must be reconciled (migration fires first) or the
  local lane stays unprotected (H-plan Track 9 finding, per handoff).
- devin count_tokens is char/4 (pre-count only); post-turn `usage` is real. Do not migrate-decide
  off the count endpoint.
- The conductor's existing ctx read lives in slice_driver, NOT the chat loop; the trigger
  generalizes it into chat_turn (not a copy-paste).
- The devin bridge proxy + litellm proxy source were NOT re-read this session (host-side,
  out-of-distro); their behavior is taken from teikoku-p4-explore-devin-bridge.md (treated as
  prior verified exploration). base-cc + conductor facts ARE first-hand from the distro this run.

SOURCES CONSULTED
  ~/teikoku/conductor/model_pool.py (full)
  ~/teikoku/conductor/chat_orchestrator.py (full)
  ~/teikoku/conductor/session_toggles.py (full)
  ~/teikoku/conductor/slice_api.py (chat routes :225-300, routing :355-420)
  ~/teikoku/conductor/slice_driver.py (:45-160)
  ~/teikoku/conductor/conductor_loop.py (:795-840 _compact_epoch)
  ~/teikoku/base-cc/src/utils/context.ts (:1-135)
  ~/teikoku/base-cc/src/utils/tokens.ts (:120-230)
  ~/teikoku/base-cc/src/utils/analyzeContext.ts (:1-135, :1155-1350, grep)
  ~/teikoku/base-cc/src/commands/context/context-noninteractive.ts (:1-120)
  ~/teikoku/base-cc/src/QueryEngine.ts (:640-830)
  ~/teikoku/base-cc/src/services/compact/autoCompact.ts (:30-150)
  ~/teikoku/base-cc/src/entrypoints/sdk/coreSchemas.ts (ModelUsageSchema grep)
  F:\.ELAI_workflow\workflow\reasoning\TEIKOKU-P4-REVISION-2026-06-20.md (:100-169)
  F:\.ELAI_workflow\workflow\reasoning\teikoku-p4-explore-devin-bridge.md (full)
  F:\.ELAI_workflow\workflow\handoff\teikoku-build.md (HEAD :1-60)
