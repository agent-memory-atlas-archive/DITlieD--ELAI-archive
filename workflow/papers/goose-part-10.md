Goose Desktop UI: Token/Cost/Usage Display & Provider Config (Partition 10)

Repository: Block/goose (open-source AI agent desktop client)
Codebase: F:\.ELAI_workflow\docs\.source\goose-main\ui\desktop\src\
Focus: token/cost/usage/context display + provider/model configuration UX

SECTION 1: TOKEN/COST/USAGE UI COMPONENTS

A. CostTracker Component
File: components/bottom_menu/CostTracker.tsx (200 lines)

Visual: renders a compact cost display in the bottom bar
- Icon: CoinIcon (16px) + monospace text
- Display modes:
  - Disabled (pricing off): null
  - No model/provider: null
  - Loading: "..." placeholder text
  - Free providers (ollama, local, localhost): "X↑ Y↓" format (inputTokens up, outputTokens down, right-aligned in monospace)
  - Pricing unavailable: "$0.0000" with tooltip; if pricing failed: tooltip "Pricing data unavailable for {model}"; if no pricing data: tooltip with token counts "Cost data not available for {model} (X input, Y output tokens)"
  - Pricing available: "$M.MMMM" (4 decimal places) with tooltip showing multi-line breakdown

Data flow:
- Props (interface CostTrackerProps): inputTokens, outputTokens, accumulatedCost, model, provider
- Fetches model cost info: fetchCanonicalModelInfo(provider, model) -> ModelInfoData
- ModelInfoData fields: input_token_cost (per-token USD), output_token_cost (per-token USD), currency (default "$"), cache_read_token_cost, cache_write_token_cost
- Cost calculation: accumulated_cost ?? 0; per-message costs = (tokens * cost_per_token / 1_000_000)
- Tooltip format (multi-line): "Total session cost: $M.MMMM\nInput: X tokens ($M) | Output: Y tokens ($M)"

State:
- costInfo: ModelInfoData | null (fetched model pricing)
- isLoading, pricingFailed, showPricing (enabled via window.electron.getSetting('showPricing'))
- Re-fetches when currentModel or currentProvider changes

Real JSX excerpt (lines 189-199):
<Tooltip>
  <TooltipTrigger asChild>
    <div className="flex items-center justify-center h-full transition-colors cursor-default translate-y-[1px] text-text-primary/70 hover:text-text-primary">
      <CoinIcon className="mr-1" size={16} />
      <span className="text-xs font-mono">{formatCost(totalCost)}</span>
    </div>
  </TooltipTrigger>
  <TooltipContent>{getTooltipContent()}</TooltipContent>
</Tooltip>

Streaming updates:
- accumulatedCost comes from tokenState.accumulatedCost or session.accumulated_cost (updated per message from server)
- Input/output tokens update as message streams in real-time
- Cost recalculates on every render (no debounce)
- Tooltip content regenerates fresh each render

Settings integration:
- window.electron.getSetting('showPricing') gated via listener 'showPricingChanged'
- Cost tracking can be toggled off entirely (hides component)

B. ContextWindowIndicator Component
File: components/bottom_menu/ContextWindowIndicator.tsx (41 lines)

Visual: progress bar text showing token fill
- Display: "{totalTokens} / {tokenLimit}" with formatted counts (1M+, 1k+, or raw)
- Format function formatTokenCount: rounds to M/k with no decimal ("2M", "45k", "1234")
- Color coding: text-text-primary/70 <= 75%, text-orange-500 <= 90%, text-red-500 > 90%
- Layout: flex row with BottomMenuAlertPopover (tooltip that can show alert messages)

Props (interface ContextWindowIndicatorProps): totalTokens, tokenLimit, alerts
- Alerts: Alert[] (array of alert objects with message, severity)

Real JSX (lines 33-38):
<div className="flex items-center h-full">
  <BottomMenuAlertPopover alerts={alerts}>
    <span className={`text-xs font-mono ${colorClass}`}>
      {formatTokenCount(totalTokens)} / {formatTokenCount(tokenLimit)}
    </span>
  </BottomMenuAlertPopover>
</div>

Updates:
- totalTokens updated from tokenState.totalTokens or session.total_tokens
- tokenLimit pulled from session model context_limit
- Recalculates percentage and color class every render
- BottomMenuAlertPopover may show "Context window" or "Context window full" warnings

SECTION 2: CONTEXT-LIMIT UX

BottomMenuAlertPopover Component
File: components/bottom_menu/BottomMenuAlertPopover.tsx
- Wraps children in a popover trigger
- Shows alerts (warnings) on hover/click when context approaches or exceeds limit
- Alert types: e.g., Alert with message "Context window full" when totalTokens >= tokenLimit

Workflow:
1. ContextWindowIndicator detects percentage > 90%
2. Alert system (useAlerts hook) generates Alert objects
3. BottomMenuAlertPopover renders tooltip/popover with alert text
4. User can potentially trigger session compaction or summary (guarded by alerts)

No explicit "summarize conversation" modal found in partition; alerts appear to trigger on fill state only.

SECTION 3: PROVIDER / MODEL / BYOK UI

A. Provider Configuration Flow

ProviderSettingsPage Component
File: components/settings/providers/ProviderSettingsPage.tsx
- Route: /configure-providers
- Displays provider grid (catalog of available providers)
- Loads providers via useConfig().getProviders(forceRefresh)
- Description (line 22-23): "Select an AI model provider to get started with goose. You'll need to use API keys generated by each provider which will be encrypted and stored locally. You can change your provider at any time in settings."

B. Provider Configuration Modal

ProviderConfigurationModal Component
File: components/settings/providers/modal/ProviderConfigurationModal.tsx
- Triggered when user clicks a provider tile
- Modes:
  - OAuth flow (e.g., HuggingFace): sign-in button, browser window opens
  - API key input: form with text fields for apiKey, apiHost, models (etc.)
  - External setup (Ollama, local): displays setup steps in <code> backtick blocks
  - Delete/remove existing config

Provider Metadata (from API types.gen.ts):
- ProviderDetails.metadata.config_keys: array of ConfigKey objects
- ConfigKey fields: name, oauth_flow, device_code_flow, primary, secret, default, description
- Primary keys shown first; can hide optional keys in collapsible

Form handling:
- DefaultProviderSetupForm renders input fields for each config_key
- Input type detection: "apiKey" -> masked/secret; "apiHost" -> URL placeholder; "models" -> comma-separated list
- Validation: required params checked before submit
- Storage: useConfig().upsert(key, value, isSecret) encrypts sensitive fields locally
- Deletion: useConfig().remove() after confirmation

Real JSX excerpt (ProviderConfigurationModal.tsx lines 202-207):
const descriptionText = ... hasOAuth
  ? intl.formatMessage(i18n.oauthSignInDescription, {
      providerName: provider.metadata.display_name,
    })
  : isExternalSetup ? provider.metadata.description
  : intl.formatMessage(i18n.addApiKeyDescription);

C. Model Selection & Switching

ModelAndProviderContext Component
File: components/ModelAndProviderContext.tsx
- Context provides: currentModel, currentProvider, changeModel(), getCurrentModelAndProvider()
- changeModel(sessionId, model) API flow:
  1. If sessionId: updateAgentProvider() via API (updates agent's model for current session)
  2. If no sessionId: setConfigProvider() to persist as global default
  3. Triggers toast: "Successfully switched models -- using {alias} from {subtext}"

Model type (from modelInterface.ts, inferred):
- Model: { name, provider, context_limit, request_params, alias, subtext }
- Passed to provider config and used for display names

SECTION 4: STATE MANAGEMENT

A. TokenState Type Definition
File: api/types.gen.ts (generated, OpenAPI)

export type TokenState = {
  accumulatedCost?: number | null;
  accumulatedInputTokens: number;
  accumulatedOutputTokens: number;
  accumulatedTotalTokens: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
};

Semantics:
- inputTokens, outputTokens: current message tokens (streaming update)
- totalTokens: cumulative conversation tokens (message history)
- accumulatedInputTokens, accumulatedOutputTokens: sum across all messages in session
- accumulatedCost: running USD cost (populated if pricing available)

B. Stream State Management
File: hooks/useChatStream.ts

Reducer (streamReducer):
- Action SET_TOKEN_STATE updates entire tokenState object
- Action SESSION_LOADED initializes tokenState from server response
- Initial state (initialTokenState): all zeros except accumulatedTotalTokens

Flow:
1. useChatStream hook dispatches SET_TOKEN_STATE when message arrives from server
2. TokenState bubbles via BaseChat props to CostTracker & ContextWindowIndicator
3. Fallback chain: tokenState ?? session.accumulated_cost ?? undefined (if tokenState not populated)

Integration point (BaseChat.tsx lines 502-510):
const {
  session,
  messages,
  chatState,
  ...
  tokenState,
} = useChatStream({ sessionId, onStreamFinish });

// Later in render:
<CostTracker
  inputTokens={accumulatedInputTokens}
  outputTokens={accumulatedOutputTokens}
  accumulatedCost={tokenState?.accumulatedCost ?? session?.accumulated_cost ?? undefined}
  model={effectiveModel}
  provider={effectiveProvider}
/>

C. Configuration Storage
File: components/ConfigContext.tsx (inferred from usage)
- Methods: read(key, isSecret), upsert(key, value, isSecret), remove(key)
- Storage: encrypted local config file (mentioned in UI text)
- Accessed via: useConfig() hook

SECTION 5: PORTABLE IDEAS FOR TAURI V2 + SVELTE 5

1. Cost Display with Per-Message Breakdown
   File: components/bottom_menu/CostTracker.tsx (lines 98-102, 162-187)
   Pattern: Fetch model pricing from canonical API, calculate per-message costs in real-time, display running total with tooltip breakdown showing input/output costs separately.
   Portable: Use same cost calculation formula: (tokens * cost_per_token_USD / 1_000_000). Svelte reactivity can replace React state. Canary: depends on canonical model info endpoint existing on backend.

2. Token Count Formatting & Color Coding
   File: components/bottom_menu/ContextWindowIndicator.tsx (lines 10-20)
   Pattern: Helper functions formatTokenCount(count) and getProgressColor(percentage) are pure functions. Threshold: 75% yellow, 90% red. Format: "2M", "45k", or raw number.
   Portable: Direct 1:1 port. No dependencies on React. Use Svelte component lifecycle for updates.

3. Provider Configuration Flow with Form Validation
   File: components/settings/providers/modal/ProviderConfigurationModal.tsx, DefaultProviderSetupForm.tsx
   Pattern: Load provider metadata (config_keys array), render conditional form fields (API key, host, models), validate required fields, store encrypted locally, support OAuth login with browser window.
   Portable: Requires equivalent backend endpoint (GET /providers, POST /configure/{provider}). Form structure can be reused. Encrypt storage is OS-specific (Tauri can use OS keychain).

4. Real-Time Tooltip Cost Breakdown
   File: components/bottom_menu/CostTracker.tsx (lines 162-187, getTooltipContent function)
   Displays: "Total session cost: $M.MMMM\nInput: X tokens ($Y) | Output: Z tokens ($W)"
   Portable: Multi-line formatted text in tooltip. Calculate on-the-fly from accumulatedCost and pricing data. No state needed beyond props.

5. Context Window Fill Indicator with Alert Popover
   File: components/bottom_menu/ContextWindowIndicator.tsx + BottomMenuAlertPopover.tsx
   Pattern: Calculate percentage, color-code text, trigger alert system when > 90%. Alerts bubble from chat state reducer.
   Portable: Svelte component with computed percentage. Alert system hooks into existing message stream state. Color scheme can use Svelte's reactive stores.

6. Model/Provider Switching with Global Config Persistence
   File: components/ModelAndProviderContext.tsx (changeModel function, lines 66-128)
   Pattern: Distinguish session-level provider change (updateAgentProvider for current chat) vs. global default (setConfigProvider for settings). Toast feedback on success/failure.
   Portable: Requires two API endpoints (one session-scoped, one global). Context can be replaced with Svelte store. API calls are framework-agnostic.

7. Incremental Cost Calculation Per Message
   File: api/types.gen.ts (TokenState) + CostTracker.tsx (calculateCost, lines 98-100)
   Pattern: TokenState carries accumulatedInputTokens/accumulatedOutputTokens. CostTracker uses these directly, not recalculating from message history.
   Portable: Requires backend to hydrate accumulatedCost/accumulatedInputTokens on message stream events. No client-side token counting needed; trust server values.

8. Secure API Key Input with Masking
   File: components/settings/providers/modal/subcomponents/forms/DefaultProviderSetupForm.tsx (ConfigInput type, line 54-58)
   Pattern: ConfigInput stores { serverValue (masked or plaintext), value (input) }. Secret flag on ConfigKey determines whether to mask display. useConfig.read() loads existing (masked) values.
   Portable: Input UI can show masked placeholder for existing values ("•••••••"). New values sent to Electron IPC or secure Tauri command. No plaintext in DOM.

SECTION 6: DATA FLOW DURING STREAMING

Sequence during message reply:
1. User submits message via ChatInput
2. useChatStream calls sessionReply() API (streaming)
3. Server sends message events with token deltas
4. streamReducer dispatches SET_TOKEN_STATE with new TokenState
5. BaseChat re-renders, passes tokenState to ChatInput
6. ChatInput passes to CostTracker and ContextWindowIndicator
7. CostTracker fetches pricing (cached via useEffect), calculates cost tooltip
8. ContextWindowIndicator checks percentage, applies color class
9. Both update on each render frame (streaming updates flow frame-by-frame)

Real-time behavior:
- Cost updates: every 100-200ms as tokens stream in (depends on server chunk cadence)
- No debounce; calculation is O(1) multiplication
- Tooltip regenerates on each render (no memoization observed)

SECTION 7: CONFIGURATION STORAGE & LIFECYCLE

Provider API key storage:
- File-based YAML config (mentioned: "encrypted and stored locally")
- useConfig().upsert(key, value, isSecret=true) marks for encryption
- Electron IPC to main process for secure storage (Tauri equivalent: tauri::api::path::ConfigDir)
- On read: useConfig().read(key, isSecret) returns plaintext value or masked placeholder

Model/provider default:
- Persisted via setConfigProvider API call (POST /config/provider, body: { provider, model })
- Session-level override: updateAgentProvider API (different endpoint, scoped to session_id)
- Fallback in ModelAndProviderContext: GOOSE_DEFAULT_PROVIDER, GOOSE_DEFAULT_MODEL env/config vars

SECTION 8: OBSERVED LIMITATIONS & GAPS

1. Cost tracking disabled by default
   - COST_TRACKING_ENABLED flag checked in ChatInput.tsx (line 1634)
   - No UI toggle found to enable in this partition; may be settings-only
   - If disabled: CostTracker component returns null

2. Pricing data optional
   - Many models likely have no pricing in canonical API
   - Fallback: shows "0.0000" with unavailable tooltip
   - Free providers (ollama, local) special-cased to show token counts instead

3. No model-specific context window auto-limit enforcement
   - ContextWindowIndicator is read-only display; no automatic truncation triggered
   - Alerts on fill but no auto-summarize observed in this partition

4. Provider deletion gated but no warning before deletion
   - Cannot delete active provider (modal checks, line 172)
   - Deletion is final (removeCustomProvider API call)
   - No "exported config" option to backup before delete

5. OAuth flow requires browser window
   - Device code flow mentioned for some providers (line 184)
   - Clipboard auto-copy for device code (feature mentioned in i18n, line 95-96)
   - HuggingFace OAuth specific (dedicated HuggingFaceSignInPrompt component)

VERIFICATION NOTES

Verified reads:
- CostTracker.tsx: full component, 200 lines
- ContextWindowIndicator.tsx: full component, 41 lines
- BaseChat.tsx: lines 502-510 integration
- ChatInput.tsx: lines 1630-1650 integration, COST_TRACKING_ENABLED check
- useChatStream.ts: lines 1-100, TokenState types & reducer init
- api/types.gen.ts: TokenState type definition
- ModelAndProviderContext.tsx: lines 66-128 changeModel function
- ProviderSettingsPage.tsx: lines 1-100 load/refresh flow
- ProviderConfigurationModal.tsx: lines 155-210 config modes
- DefaultProviderSetupForm.tsx: lines 1-120 form rendering

Unverified (out of partition or generated):
- Exact encryption mechanism for secret storage (Electron IPC impl)
- Full list of supported providers & pricing endpoints
- Real-time streaming token update mechanism (server-side)
- BottomMenuAlertPopover.tsx internals (referenced but not fully read)
- costDatabase utility (mocked in tests, file not found in partition)
