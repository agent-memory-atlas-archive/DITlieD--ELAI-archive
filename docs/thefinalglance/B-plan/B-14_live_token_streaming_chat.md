# B-14: Live Token Streaming Chat
Objective: ChatWindow streams real LLM tokens via sidecar /v1/messages SSE
(currently writes only intent rows). Replace the chat_message intent stub
with the actual conversation runtime.
Estimated effort: 1-2 sessions.
Pre-requisites: none for the streaming layer; B-12 (Workspace primary).
Parallel-safe with: B-13, B-15..B-20.

## V2 REBASELINE (2026-05-07) — ChatWindow is now CommandWindow

V2 retired the V1 ChatWindow concept. The chat shell + slash palette live in
`.elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte` plus
`lib/layout/CommandPalette.svelte`. Per DOMAIN-CONTEXT §2 (COMMAND window):

  - Vertical list of cards (status / intervention / error / disambiguation /
    steering / user).
  - Bottom: outlined chat shell (`.input-shell`) with `--bg-input` interior,
    `--teal-line` border on focus.
  - Slash palette is a floating dialog anchored above the chat shell;
    pressing `/` at the start of the textarea auto-opens it.
  - Slash command groups: Workflow (/task /fix /research /continue),
    Project (/explore /status /config), Help (/ask).
  - Card buttons: Approve (primary outlined teal), Reject/Modify/Redirect
    (secondary outlined neutral). NEVER solid-fill CTAs (per DESIGN.md
    differentiator #6 — outlined-button language).

Path map (V1 -> V2):
  V1: crates/dashboard/src/commands.rs::submit_chat_message
   -> crates/dashboard-v2/src-tauri/src/lib.rs (NEW: stream_chat command)
  V1: crates/dashboard/src-frontend/src/chat/ChatWindow.svelte
   -> crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
  V1: crates/dashboard/src-frontend/src/chat/SlashPalette.svelte
   -> crates/dashboard-v2/src/lib/layout/CommandPalette.svelte
  V1: crates/dashboard/src-frontend/src/chat/InterventionCard.svelte
   -> NOT YET IN V2 — must be added under
       crates/dashboard-v2/src/lib/components/InterventionCard.svelte
       (or absorbed as a card variant inside CommandWindow.svelte;
       decision doc captures the choice).
  V1: crates/orchestrator/src/intent_classifier.rs
   -> unchanged (orchestrator-side, V2-agnostic).

Tauri-side changes from original B-14:
  - `stream_chat(message)` lives in
    crates/dashboard-v2/src-tauri/src/lib.rs (not crates/dashboard/src/
    commands.rs). Use `tauri::Window::emit("token", ...)` (Tauri 2 API).
  - Coordinate with B-36 Session 3 (Track D D.7 one_shot_prompt) — both
    use the same SSE-via-emit pattern. Keep the channel naming consistent
    ("turn", "streaming_started", "response", "token", "stop").
  - Frontend SSE consumer in CommandWindow.svelte uses
    `import { listen } from '@tauri-apps/api/event'` for the token stream
    (Tauri 2 API).
  - Stop button cancels via `invoke('cancel_stream', { handle })` —
    needs server-side cancellation handle in AppState.

Re-scoped tasks (deltas from original B-14):
  - Drop "ChatWindow.svelte" file references; everything lives in
    CommandWindow.svelte.
  - Drop the V1 SlashPalette.svelte addition (V2 already has
    CommandPalette.svelte).
  - Add InterventionCard pattern (new in V2) per DOMAIN-CONTEXT §2 COMMAND
    card list (status/intervention/error/disambiguation/steering/user).
  - Use Svelte 5 runes ($state for token buffer, $effect for SSE
    subscription cleanup) — NO `$:` reactive labels and NO
    `createEventDispatcher`.

## Context

Per audit-dashboard/inference-wiring.md and audit-implementation/phase-3J.md:
"Dashboard chat (ChatWindow → submit_chat_message) does not stream LLM
output; it writes a chat_message SSB row carrying intent and that's it.
The actual conversation loop only runs in CLI/REPL."

The ConversationRuntime → RuntimeClient(Local) → sidecar /v1/messages →
DoLaSteeringProcessor → model.generate → SSE → render path IS WIRED for
CLI but not for dashboard. Dashboard chat is currently a chat-shaped
log of user intents only.

## Reads

- audit-dashboard/inference-wiring.md
- audit-implementation/phase-3J.md WIRE.* sub-step coverage
- crates/orchestrator/src/conversation_runtime.rs (or equivalent)
- crates/dashboard/src-frontend/src/chat/ChatWindow.svelte
- crates/dashboard/src/commands.rs (submit_chat_message — current stub)
- crates/cli/src/main.rs (the working CLI streaming pattern)
- F:\.ELAI_workflow\docs\specs\guide\dashboard-design.txt WINDOW: COMMAND

## Tasks

1. **New Tauri command `stream_chat(message)`.** Returns a Stream of token
   events (or uses Tauri's event emitter pattern). Reuses the same
   ConversationRuntime + RuntimeClient + sidecar path that CLI uses.

2. **Frontend SSE consumer.** ChatWindow subscribes to the stream and
   renders tokens as they arrive (typing indicator while streaming).

3. **Intent classification.** Per spec, every user input is classified
   before routing:
   - TASK / FIX / EXPLORE / CONFIG / INTERVENE / STEER / ABORT
   Classification is deterministic (slash commands + keywords) with LLM
   fallback for ambiguous natural language. Disambiguation card UI per
   spec FLOW 4.

4. **Slash command autocomplete.** Typing "/" shows overlay above input
   with /task /fix /explore /config /status /continue + filtered list.

5. **Response cards per spec.** Status card, Intervention card, Error card,
   Disambiguation card, Steering acknowledgment — match exact layouts
   from dashboard-design.txt RESPONSE TYPES section.

6. **Input bar placeholder per FSM state.**
   - IDLE: "describe a task, ask a question, or type / for commands"
   - EXECUTING: "add constraints, or wait for intervention cards"
   - intervention active: "respond to the card above, or type to add context"

7. **No "thinking..." messages.** Per spec: "When ELAI is working, the
   ACTIVITY panel shows progress. The COMMAND window only updates when
   user input is needed or a milestone is hit."

8. **Stop button.** While streaming, show a Stop icon to cancel
   generation (cancel inflight request to sidecar).

## WIRING_CONTRACT

produces_symbols:
  - crates/dashboard-v2/src-tauri/src/commands.rs::stream_chat
  - crates/orchestrator/src/intent_classifier.rs::classify_intent
consumes_symbols:
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
  - crates/dashboard-v2/src/lib/layout/CommandPalette.svelte
reachable_from:
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte::submitInput
stub_ban:
  - submit_chat_message intent-only stub removed
  - chat tokens render in real time (not after completion)
frontend_pair: B-14 itself
observable_effect:
  trigger: cd .elai_cc && cargo test -p elai-dashboard-v2 -- b14_stream_chat_tests --nocapture
  expect_ssb: chat_message
  expect_artifact: tokens visible streaming character-by-character

## DETERMINISTIC_CHECK

```bash
# 1. stream_chat command exists
rg "stream_chat\\b" .elai_cc/crates/dashboard/src/commands.rs
# Expected: ≥1 hit.

# 2. Intent classification covers spec categories
rg "TASK|FIX|EXPLORE|CONFIG|INTERVENE|STEER|ABORT" \
   .elai_cc/crates/orchestrator/src --glob 'classifier*'
# Expected: all 7 categories present.

# 3. Smoke: dashboard chat with "/task hello" → planning state visible
```

## ACCEPTANCE

- [ ] Dashboard chat streams real tokens
- [ ] 7 intent categories classified correctly
- [ ] Slash command autocomplete works
- [ ] 5 response card types render per spec
- [ ] Placeholder text changes per FSM state
- [ ] Stop button cancels inflight generation
- [ ] No "thinking..." messages

## Files touched

- MODIFY: crates/dashboard/src/commands.rs (add stream_chat)
- MODIFY: crates/dashboard/src-frontend/src/chat/ChatWindow.svelte
- ADD: crates/dashboard/src-frontend/src/chat/SlashPalette.svelte (if not present)
- ADD: crates/dashboard/src-frontend/src/chat/InterventionCard.svelte
  (or reuse from B-5)
- ADD: crates/orchestrator/src/intent_classifier.rs (deterministic + LLM fallback)
- CREATE: workflow/reasoning/B-14-decisions.md

## GATE FOR PART II

After B-11 through B-14, the dashboard:
- Looks like dashboard-design.txt (typography, color, scale, animations)
- Workspace is primary; status bar inside; sidebar 48px / 5 icons
- Activity feed is the centerpiece with auto-scroll + preview links +
  collapsible reasoning
- Chat streams real LLM tokens in real time

The user's "doesn't feel or look like I wanted" complaint is fully addressed.
Move to PART III (B-15..20) for harness hardening.

## Next

B-15 (Phase 0 debt burndown).
