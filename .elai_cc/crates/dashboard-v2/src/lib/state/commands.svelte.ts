// ============================================================
// commands.svelte.ts — Command card history + input state
// ============================================================

import type { CommandCard, CardKind, IntentType, ResponseCardPayload } from '../api/types';
import {
  streamChat,
  cancel_stream,
  logStub,
  run_research,
  submit_task,
  run_explore_query,
  continue_task,
  type ResearchResult,
} from '../api/stub';
import { classifyIntent } from '../utils/intent';
import { parseSlashCommand, KNOWN_SLASH_COMMANDS } from '../utils/slash-parse';

function makeId() { return `cc-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`; }

const SLASH_COMMANDS = KNOWN_SLASH_COMMANDS;

// Slash commands whose backend dispatch is currently a TODO. These still
// route through the slash path so the user sees a clear error card instead
// of silently streaming the literal text into chat.
const SLASH_NO_BACKEND = new Set<string>(['/fix', '/config', '/status', '/ask']);

function createCommands() {
  // Phase B: start with no cards. Real conversation comes from streamChat() on submit
  // and from intervention/status events appended by other code paths.
  let cards = $state<CommandCard[]>([]);
  let inputValue = $state<string>('');
  let inputFocused = $state<boolean>(false);
  let slashOpen = $state<boolean>(false);
  // Tracks the request_id of the currently active stream (one at a time).
  let activeStreamId = $state<string | null>(null);
  // Cleanup fn returned by streamChat — call to unlisten + abort.
  let _streamCleanup: (() => void) | null = null;

  const isStreaming = $derived(activeStreamId !== null);

  function appendCard(kind: CardKind, payload: CommandCard['payload']) {
    const card: CommandCard = { id: makeId(), ts: Date.now(), kind, payload };
    cards = [...cards, card];
    return card;
  }

  /** Mutates the text of an in-progress response card identified by stream_id. */
  function _appendToken(streamId: string, text: string) {
    cards = cards.map((c) => {
      if (c.kind !== 'response') return c;
      const p = c.payload as ResponseCardPayload;
      if (p.stream_id !== streamId) return c;
      return { ...c, payload: { ...p, text: p.text + text } };
    });
  }

  /** Marks a response card as no longer streaming. */
  function _finalizeStream(streamId: string) {
    cards = cards.map((c) => {
      if (c.kind !== 'response') return c;
      const p = c.payload as ResponseCardPayload;
      if (p.stream_id !== streamId) return c;
      return { ...c, payload: { ...p, streaming: false } };
    });
    if (activeStreamId === streamId) activeStreamId = null;
    _streamCleanup = null;
  }

  async function dispatchSlash(command: string, arg: string, intent: IntentType) {
    // User card already appended by submit() before delegating here.
    if (arg.length === 0) {
      appendCard('error', {
        kind: 'error',
        message: `${command} requires an argument`,
        source: 'sidecar',
        step_id: undefined,
        recoverable: true,
      });
      return;
    }

    if (command === '/research') {
      const result: ResearchResult = await run_research(arg).catch(
        (err): ResearchResult => ({ ok: false, error: String(err) }),
      );
      if (result.ok) {
        appendCard('status', {
          kind: 'status',
          message: `research started${result.requestId ? ` (req ${result.requestId})` : ''}; watch the Research drawer for progress`,
          fsm_state: undefined,
          step_id: undefined,
        });
      } else {
        appendCard('error', {
          kind: 'error',
          message: result.error ?? 'run_research returned !ok',
          source: 'sidecar',
          step_id: undefined,
          recoverable: true,
        });
      }
      return;
    }

    if (command === '/task') {
      try {
        const { taskId } = await submit_task(arg);
        appendCard('status', {
          kind: 'status',
          message: `task submitted (id ${taskId}); FSM transitioning to PLANNING`,
          fsm_state: 'PLANNING',
          step_id: undefined,
        });
      } catch (err) {
        appendCard('error', {
          kind: 'error',
          message: `submit_task failed: ${String(err)}`,
          source: 'sidecar',
          step_id: undefined,
          recoverable: true,
        });
      }
      return;
    }

    if (command === '/explore') {
      try {
        const answer = await run_explore_query(arg);
        appendCard('response', {
          kind: 'response',
          text: answer.answer,
          streaming: false,
          stream_id: makeId(),
        });
      } catch (err) {
        appendCard('error', {
          kind: 'error',
          message: `run_explore_query failed: ${String(err)}`,
          source: 'sidecar',
          step_id: undefined,
          recoverable: true,
        });
      }
      return;
    }

    if (command === '/continue') {
      try {
        const { ok, restoredContext } = await continue_task(arg);
        if (ok) {
          appendCard('status', {
            kind: 'status',
            message: restoredContext,
            fsm_state: undefined,
            step_id: undefined,
          });
        } else {
          appendCard('error', {
            kind: 'error',
            message: `continue_task returned !ok for ${arg}`,
            source: 'sidecar',
            step_id: undefined,
            recoverable: true,
          });
        }
      } catch (err) {
        appendCard('error', {
          kind: 'error',
          message: `continue_task failed: ${String(err)}`,
          source: 'sidecar',
          step_id: undefined,
          recoverable: true,
        });
      }
      return;
    }

    if (SLASH_NO_BACKEND.has(command)) {
      logStub(`commands.slash.${command.slice(1)}`, { arg, intent });
      appendCard('error', {
        kind: 'error',
        message: `${command} is not yet wired to a backend command — see dashboard-v2-wiring-inventory.txt`,
        source: 'sidecar',
        step_id: undefined,
        recoverable: false,
      });
      return;
    }

    // Should not reach here for any KNOWN_SLASH_COMMANDS entry.
    appendCard('error', {
      kind: 'error',
      message: `slash command ${command} has no dispatcher`,
      source: 'sidecar',
      step_id: undefined,
      recoverable: false,
    });
  }

  async function submit(text: string) {
    if (!text.trim()) return;

    const intent = classifyIntent(text);

    // Append user card immediately
    appendCard('user', { kind: 'user', text, intent });
    inputValue = '';
    slashOpen = false;

    const slash = parseSlashCommand(text);
    if (slash !== null) {
      logStub('commands.submit.slash', { command: slash.command, arg: slash.arg, intent });
      if (slash.unknown === true) {
        appendCard('error', {
          kind: 'error',
          message: `unknown slash command: ${slash.command}`,
          source: 'sidecar',
          step_id: undefined,
          recoverable: false,
        });
        return;
      }
      await dispatchSlash(slash.command, slash.arg, intent);
      return;
    }

    logStub('commands.submit', { text, intent });

    const requestId = makeId();
    activeStreamId = requestId;

    // Add a placeholder response card that will accumulate tokens.
    appendCard('response', {
      kind: 'response',
      text: '',
      streaming: true,
      stream_id: requestId,
    });

    try {
      _streamCleanup = await streamChat(
        text,
        requestId,
        (chunk) => _appendToken(requestId, chunk),
        (_stopReason) => _finalizeStream(requestId),
        (errMsg, offline) => {
          _finalizeStream(requestId);
          // Replace the empty response card with an error message.
          cards = cards.filter(
            (c) => !(c.kind === 'response' && (c.payload as ResponseCardPayload).stream_id === requestId),
          );
          appendCard('error', {
            kind: 'error',
            message: offline
              ? 'Sidecar offline — start with `elai.bat --remote` to use the friend\'s GPU box'
              : errMsg,
            source: 'sidecar',
            step_id: undefined,
            recoverable: true,
          });
        },
      );
    } catch (err) {
      _finalizeStream(requestId);
      appendCard('status', {
        kind: 'status',
        message: `Stream error: ${String(err)}`,
        fsm_state: undefined,
        step_id: undefined,
      });
    }
  }

  function cancelStream() {
    if (_streamCleanup) {
      _streamCleanup();
      _streamCleanup = null;
    }
    if (activeStreamId) {
      cancel_stream(activeStreamId).catch(() => {});
      activeStreamId = null;
    }
  }

  function setInput(v: string) {
    inputValue = v;
    slashOpen = v.startsWith('/') && v.length <= 1;
  }

  function openSlash() {
    slashOpen = true;
    logStub('commands.openSlash');
  }

  function closeSlash() {
    slashOpen = false;
  }

  function setFocused(v: boolean) {
    inputFocused = v;
  }

  return {
    get cards() { return cards; },
    get inputValue() { return inputValue; },
    get inputFocused() { return inputFocused; },
    get slashOpen() { return slashOpen; },
    get slashCommands() { return SLASH_COMMANDS; },
    get isStreaming() { return isStreaming; },
    get activeStreamId() { return activeStreamId; },

    submit,
    appendCard,
    cancelStream,
    setInput,
    openSlash,
    closeSlash,
    setFocused,
  };
}

export const commands = createCommands();
