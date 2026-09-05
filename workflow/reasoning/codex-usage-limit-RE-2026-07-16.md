# Reverse-engineering how Codex keeps a task running after usage is "over"

Date: 2026-07-16
Question (from the Flavio Adamo tweet + the two screenshots): how does Codex decide that a task can keep getting done even though the usage limit is reached, where Claude Code kills the task mid-way? What is the exact mechanism?

Grounding: the Codex client is open source (openai/codex, Rust, `codex-rs/`). Every claim below is anchored to a source file/line read on 2026-07-16 from raw.githubusercontent.com (client-side, authoritative and readable), or to the reproduced behavior in issue openai/codex#18018. The one piece that is NOT recoverable — the server-side cloud-task admission scheduler — is flagged explicitly.


## TL;DR (the mechanism in five facts)

1. Usage state is TELEMETRY, not a client-side gate. The server stamps every response with `x-codex-*-used-percent` / `-reset-at` headers (and a `codex.rate_limits` SSE event). The client parses these into a `RateLimitSnapshot` and paints the meter ("12% used, resets at 10:40 PM"). Being at/over 100% on that meter does NOT, by itself, stop the client. Nothing in the client says "meter full -> halt."

2. The stop is a SERVER decision made PER REQUEST. The client simply fires the next request; the server either serves it (200) or rejects it with HTTP 429.

3. A 429 is DISAMBIGUATED by the response body's `error_type` (codex-api/src/api_bridge.rs). This is the crux:
   - `error_type == "usage_limit_reached"` -> `CodexErr::UsageLimitReached` -> TERMINAL (not retried).
   - any OTHER 429 -> `CodexErr::UnexpectedStatus` -> RETRYABLE (backoff + retry, the turn survives).
   That single branch is literally the tweet's "Getting rate limited is fine / Killing the task halfway is not."

4. Even the terminal usage-limit only blocks the NEXT model request. Work already returned in the current turn completes (tool calls run). So the in-flight step finishes; the ceiling is enforced at the boundary of the NEXT step, not mid-stream. That is why "it finishes the last task."

5. For the CLOUD agent (screenshot 2), the task is admitted when launched and the server keeps serving that admitted task's requests to completion — issue #18018 confirms a running task continues after the weekly quota is exhausted and does not even fall through to pay-as-you-go credits. The "You're out of usage / Add Credits" banner blocks starting NEW work, not the continuation of the committed task. This admission-at-task-boundary lives in the server-side `cloud-tasks` backend (not in the open-source client; characterized from behavior, not read).

Contrast: Claude Code enforces the ceiling on each request inside the task, so exhaustion mid-task returns the terminal error on the very next call and the task dies mid-way. Codex differs on two axes: transient rate-limits are retried rather than failed, AND the quota gate is applied at new-work admission while an already-admitted (cloud) task is served to completion.


## 1. Usage state is pushed to the client as telemetry every response

Two transport forms, both parsed in `codex-rs/codex-api/src/rate_limits.rs`:

A. HTTP headers (the default family), `parse_rate_limit_for_limit()`:
- `x-codex-primary-used-percent`, `x-codex-primary-window-minutes`, `x-codex-primary-reset-at`  (the ~5-hour rolling window)
- `x-codex-secondary-...-used-percent`, `-window-minutes`, `-reset-at`  (the weekly window)
- `x-codex-credits-has-credits`, `x-codex-credits-unlimited`, `x-codex-credits-balance`  (pay-as-you-go credits)
- `x-codex-rate-limit-reached-type`  -> `RateLimitReachedType`
- `x-codex-promo-message`, `x-<limit>-limit-name`
- multi-limit aware: `parse_all_rate_limits()` walks every `x-<id>-primary-used-percent` header, so more than one metered limit family (e.g. a per-model limit like `gpt-5.2-codex-sonic`) can be reported at once.

B. SSE event, `parse_rate_limit_event()`: a JSON event with `"type": "codex.rate_limits"` carrying `plan_type`, `rate_limits{primary, secondary}`, `credits`.

The parsed value is a `RateLimitSnapshot` (codex-rs/protocol/src/protocol.rs):
```
RateLimitSnapshot { limit_id, limit_name, primary: RateLimitWindow, secondary: RateLimitWindow,
                    credits: CreditsSnapshot, individual_limit: SpendControlLimitSnapshot,
                    spend_control_reached, plan_type, rate_limit_reached_type }
RateLimitWindow  { used_percent: f64 (0-100), window_minutes: Option<i64>, resets_at: Option<i64> (unix s) }
CreditsSnapshot  { has_credits, unlimited, balance }
```
Crucially, the snapshot rides on the normal token-count event:
```
TokenCountEvent { info: Option<TokenUsageInfo>, rate_limits: Option<RateLimitSnapshot> }   // protocol.rs
```
so the client learns its usage state as a side-channel on ordinary successful responses (this is exactly what issue #14489 "don't re-emit last_token_usage on rate-limit-only updates" is about). This drives the UI meter and the "Resets at ..." text. It is display; it gates nothing.


## 2. The gate is the server's 429, disambiguated by body (the decision point)

`codex-rs/codex-api/src/api_bridge.rs` maps a transport `ApiError` to the core `CodexErr`. For a 429:
```
} else if status == http::StatusCode::TOO_MANY_REQUESTS {
    if err.error.error_type.as_deref() == Some("usage_limit_reached") {
        let rate_limit_reached_type = headers.and_then(parse_rate_limit_reached_type);
        // attach the RateLimitSnapshot + resets_at from the error body
        return CodexErr::UsageLimitReached(UsageLimitReachedError {
            plan_type, resets_at, rate_limits, rate_limit_reached_type,
        });
    }
    // ... any other 429 falls through to:
    CodexErr::UnexpectedStatus(UnexpectedResponseError { status, ... })
}
```
So the SAME HTTP status (429) yields two completely different core errors based on the body's `error_type`. That is the fork the entire behavior hinges on.


## 3. Retryable vs terminal (why one 429 continues and the other stops)

`codex-rs/protocol/src/error.rs`, `CodexErr::is_retryable()`:
- RETRYABLE (true): `Stream(..)`, `Timeout`, `RequestTimeout`, `UnexpectedStatus(_)`, `ResponseStreamFailed(_)`, `ConnectionFailed(_)`, `InternalServerError`, `InternalAgentDied`, `Io`, `Json`, `TokioJoin`.
- TERMINAL (false): `UsageLimitReached(_)`, `UsageNotIncluded`, `QuotaExceeded`, `SessionBudgetExceeded`, `TurnAborted`, `Interrupted`, `RetryLimit(_)`, `ContextWindowExceeded`, `AgentLimitReached`, `ServerOverloaded`, `CyberPolicy`, invalid request, etc.

Therefore:
- a plain rate-limit 429 (-> `UnexpectedStatus`) is retryable -> the turn reconnects and continues. "Getting rate limited is fine."
- a usage-limit 429 (-> `UsageLimitReached`) is the ONE quota case that is terminal -> the turn ends rather than spinning forever.


## 4. Retry keeps the SAME turn alive (transient errors never kill the task)

`codex-rs/core/src/responses_retry.rs`, `handle_retryable_response_stream_error()`:
- On a retryable stream error it increments a retry counter and sleeps a backoff before returning `Ok(())` (meaning "retry the same request loop"). If the error is `CodexErr::Stream(_, requested_delay)` it honors the SERVER-REQUESTED delay (i.e. a 429 Retry-After) instead of the default exponential `backoff(n)`.
- If websocket retries are exhausted it FALLS BACK from WebSocket to HTTPS transport and resets the counter — "normal stream retry/fallback logic handles recovery on the same turn" (client.rs "Retry-Budget Tradeoff" doc). It surfaces "Reconnecting... k/max" to the UI so the user does not see a frozen screen.
- Only after `retries >= max_retries` (and no transport fallback left) does it return `Err(err)`.
Net: transient failure, including an ordinary rate-limit, reconnects the in-flight turn; it does not abort it.


## 5. The terminal usage-limit only blocks the NEXT request; current work drains

`UsageLimitReached` is returned up the stack and ends the turn's model loop. But it can only fire when the client actually ISSUES the next model request and the server 429s it. Any tool calls / output already produced by the current model response run to completion first. Hence the observed "finishes the last task" behavior (also reported in third-party writeups comparing Codex vs Claude Code).

The user-facing copy comes straight from `UsageLimitReachedError`'s Display (error.rs) and matches the screenshots verbatim:
- WorkspaceOwnerCreditsDepleted -> "Your workspace is out of credits. Add credits to continue."  (screenshot 2: "You're out of Codex and Work usage ... Add Credits")
- WorkspaceMemberCreditsDepleted -> "... Ask your workspace owner to refill ..."
- WorkspaceOwnerUsageLimitReached -> "You hit your spend cap set in your workspace. Increase your spend cap ..."
- WorkspaceMemberUsageLimitReached -> "... Ask an owner to increase your spend cap ..."
- named-model limit -> "You've hit your usage limit for {limit_name}. Switch to another model now, {resets_at}"  (screenshot 1: "Usage limit reached ... Resets at 10:40 PM")
The `resets_at` on the error and on each `RateLimitWindow` is where "Resets at 10:40 PM" / "reset on Jul 22, 9:58 AM" are rendered from.


## 6. The cloud-task continuation (screenshot 2) — the server-side boundary

Screenshot 2 is the CLOUD Codex agent (web UI, subagents panel, "Add Credits", `cloud-tasks`). Its multi-step task keeps running across Step 3/4/5 AFTER the "out of usage" banner. The continuation decision is SERVER-SIDE admission: a launched cloud task is admitted once and the backend keeps serving its requests to completion. Evidence and boundary:
- openai/codex#18018 "Codex keeps running after weekly limit is reached, without consuming remaining credits": the running task continues past the weekly cap AND does not consume pay-as-you-go credits — i.e. it is still served under the task's original grant, not re-billed.
- The relevant crates exist in the tree (`cloud-tasks`, `cloud-tasks-client`, `backend-client`) but the ADMISSION/scheduling logic is on OpenAI's backend, not in the open-source client. So this is characterized from behavior + the client types, NOT read from source. Honest boundary, same class as "the hosted model is not RE-able."
Interpretation consistent with all evidence: the meter/limit is checked at NEW-WORK admission (submitting a prompt, launching a task). An already-admitted autonomous task is a committed unit of work the server runs to the end; the banner gates the next admission, not the current run.


## 7. Why Claude Code behaves differently (design contrast, grounded on the Codex side only)

The difference is WHERE in the request lifecycle the ceiling is enforced:
- Codex: (a) transient rate-limits are retried on the same turn rather than surfaced as failures; (b) the quota gate is a terminal error applied when NEW work is admitted, and an admitted (cloud) task is served to completion. Result: the in-flight task finishes.
- Claude Code (per the tweet, not independently RE'd here): the usage ceiling is enforced on requests made DURING the task, so exhaustion mid-task returns a terminal condition on the next call and stops the task mid-way.
This is a product/lifecycle choice, not a capability gap: "usage as admission control on committed units of work" vs "usage as a hard ceiling on every call."


## 8. "Super big task": what actually decides stop-vs-continue (the per-request grain)

A big agentic task is NOT one request; it is a LOOP of many model sampling requests (each tool round-trip is a request). The usage gate is applied by the SERVER per request, and the client has no pre-check. So the arbiter of "does the big task stop after the limit is hit" is the server's per-request admission decision, and the client faithfully applies: retry transient 429s, stop on `usage_limit_reached` 429s.

Two variables decide it:
1. WHICH limit event fired. Transient rate-limit (429 without `usage_limit_reached`, or 5xx/stream drop) -> retryable -> the loop continues (backoff, same turn). Hard quota (429 + `error_type: usage_limit_reached`) -> terminal `UsageLimitReached` -> stops.
2. WHICH surface. CLOUD task: admitted once at launch; the backend keeps serving that task's requests to completion even as the meter exhausts (#18018, no credit consumption) -> a super big cloud task FINISHES; the limit blocks launching the NEXT task. INTERACTIVE (CLI/desktop): every model request in the loop is gated; the task rides through transient limits but STOPS at the first sampling request the server answers with `usage_limit_reached`. The current generation's already-streamed output + its tool calls drain; the next model round-trip does not happen. There is NO auto-wait-until-reset-and-resume (the error is terminal, is_retryable=false) -- it surfaces "usage limit reached, resets at X" and ends.

Honest correction to the "Codex finishes any task" myth: locally, Codex does NOT grant unlimited compute. A huge interactive task that needs more model calls than remaining quota stops at the next refused sampling request. What makes it FEEL like it finishes is three real mechanisms, none of which is "ignore the limit": (a) transient limits are retried not fatal, (b) the in-flight generation drains before stopping, (c) cloud tasks are admission-served to completion.

OTHER terminal stops a super big task can hit BESIDES the account usage limit (all `is_retryable()==false`, core catches them and ends the turn):
- `SessionBudgetExceeded` -- a per-session-tree token budget. `rollout_budget.rs` accounts `weighted_tokens_used` (output_tokens * sampling_weight + non_cached_input * prefill_weight) against `config.limit_tokens`; below the cap it only INJECTS soft "N tokens remaining" reminders to the model (pending_reminder / reminder_at_remaining_tokens), and the hard `SessionBudgetExceeded` fires when the configured budget is truly exhausted. This is independent of account usage.
- `ContextWindowExceeded` -- but Codex auto-compacts (compact.rs / compact_remote / compact_token_budget) to defer this, so a big task usually compacts and keeps going rather than dying on context.
- `AgentLimitReached { max_threads }` -- a cap on concurrent subagent threads (relevant to the screenshot-2 "5 working / 7 done" subagent fan-out).
- `RetryLimit` -- transient errors that never recover finally give up after max_retries.

So the precise answer to "what decides if it stops after the usage limit is hit": the SERVER's per-request admission (serve vs `usage_limit_reached` 429), gated by whether the task was admitted as a cloud task (served through) or is an interactive per-request stream (stops at the next refusal) -- plus the independent per-task budgets above that can end a big run before the account limit is even the cause.


## What is NOT recovered (honest boundary)

- The server-side cloud-task admission scheduler (the code that decides "this admitted task keeps being served though the account meter is exhausted, without charging credits"). OpenAI backend; not in the open-source client. Behavior is pinned by #18018 + the client types; the implementation is not readable.
- The exact server policy for issuing 429+`usage_limit_reached` vs continuing to serve (thresholds, grace, per-plan rules). Server-side.
- The local `codex.exe` on this machine could not be string-searched: Git Bash cannot resolve the `WindowsApps` reparse path, and MSIX blocks executing the bundle's `rg.exe`. Not needed — the open-source `codex-rs` is the same client logic and is fully readable.


## Evidence index (files read 2026-07-16)

Client source (raw.githubusercontent.com/openai/codex/main/codex-rs/...):
- codex-api/src/api_bridge.rs — 429 disambiguation: body `error_type=="usage_limit_reached"` -> `UsageLimitReached`, else `UnexpectedStatus`.
- codex-api/src/rate_limits.rs (FULL, 380 lines) — the `x-codex-*` header family + `codex.rate_limits` SSE event -> `RateLimitSnapshot`; primary/secondary windows; credits; reached-type header.
- protocol/src/protocol.rs — `TokenCountEvent{info, rate_limits}`, `RateLimitSnapshot`, `RateLimitWindow{used_percent, window_minutes, resets_at}`, `RateLimitReachedType`, `CreditsSnapshot`, `SpendControlLimitSnapshot`.
- protocol/src/error.rs — `CodexErr::UsageLimitReached(UsageLimitReachedError)`, `UsageNotIncluded`, `QuotaExceeded`; `is_retryable()` classification; `UsageLimitReachedError` Display strings (match screenshot copy).
- core/src/responses_retry.rs (FULL) — `handle_retryable_response_stream_error()`: same-turn retry, honors server requested_delay, WS->HTTPS fallback, only errs after max_retries.
- core/src/client.rs — "Retry-Budget Tradeoff" doc; per-turn `ModelClientSession`; retry/fallback recover on the same turn.
- crate listing shows `cloud-tasks`, `cloud-tasks-client`, `backend-client` (server-side task admission — boundary).

Behavior / reports:
- github.com/openai/codex/issues/18018 — task keeps running after weekly limit, no credit consumption (the exact screenshot-2 behavior).
- github.com/openai/codex/issues/14489 — rate-limit snapshot delivered via token-count events.
- Third-party: bswen "How Does Codex Handle Usage Limits Differently Than Claude Code" (Codex finishes the last task on running out).
